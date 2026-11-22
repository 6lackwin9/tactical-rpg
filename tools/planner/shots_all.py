#!/usr/bin/env python3
"""TOOLS-HELP2 (ENGINE-16): the shots half of `tools/fc.sh scenario shots-all out=<dir> [compare=<dir>]`.

`tools/fc.sh` runs every scenario of the game (`scenario run all`) into a scratch folder of bundles
(`<scenario>/<WxH>/<shot>.png`, beside `result.json`, `trail.json` ...); this copies every shot into one flat folder as
`<scenario>__<shot>.png` (`@<WxH>` added when a scenario ran at more than one size; golden `.diff.png` images left out)
and writes `index.txt` there (one shot per line). `compare=<dir>` (a folder an earlier run wrote: the before of a
before/after loop) then lists which shots changed (most pixels first), which are new and which are gone. A pixel
changed when one channel moved more than `tolerance` (default 8); a shot changed when more than `pixels` (default 300)
did. Two runs of the same build already differ a little (idle animations: up to a few hundred pixels on the hub screens,
far more on scenes mid-animation), so the counts are printed for every changed shot to tell drift from a real change.

  shots_all.py collect <bundles> <out> [crop=x,y,w,h]
  shots_all.py compare <before> <after> [tolerance=N] [pixels=N] [strict=1] [crop=x,y,w,h] [sheets=0]

TOOLS-HELP3 (ENGINE-20 item 2): `crop=x,y,w,h` (pixels of the shot, clipped to it) narrows the comparison to that
rectangle, and `collect` also writes each shot's crop into `<out>/crops/`. `compare` writes a sheet per changed shot
into `<after>/compare/<name>`: the before, the after and the diff side by side (of the crop when there is one; the diff
shows the after dimmed with each changed pixel in red); `sheets=0` writes none. Plain Python (no Pillow).

`strict=1` (the gate's `shotsexact` step, ENGINE-17: shots on the shot clock are frame-exact, so it runs with tolerance
0 and pixels 0) exits 1 when any shot changed, is new or is gone.

ENGINE-33 (XL-SWEEP1): a run with `audit=1` leaves `<shot>.audit.json` beside each shot (the text audit of that frame:
cut, edge, overlap, spill); `collect` copies each beside its shot as `<flat name>.audit.json` and writes `audit.txt`,
one line per shot with a problem (most first), each problem under it.
"""
import json
import shutil
import struct
import sys
import zlib
from pathlib import Path

CHANNELS = {0: 1, 2: 3, 4: 2, 6: 4}


def png_rows(path):
    """(width, bytes per pixel, [row bytes]) of an 8-bit, non-interlaced PNG; plain Python (no Pillow)."""
    data = Path(path).read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("%s: not a PNG" % path)
    i, idat, w = 8, [], None
    while i < len(data):
        n, kind = struct.unpack(">I4s", data[i:i + 8])
        body = data[i + 8:i + 8 + n]
        if kind == b"IHDR":
            w, h, depth, ctype, _, _, inter = struct.unpack(">IIBBBBB", body)
            if depth != 8 or ctype not in CHANNELS or inter:
                raise ValueError("%s: only 8-bit, non-interlaced grey/RGB/RGBA PNGs" % path)
            bpp = CHANNELS[ctype]
        elif kind == b"IDAT":
            idat.append(body)
        i += 12 + n
    raw, stride, rows, prev = zlib.decompress(b"".join(idat)), w * bpp, [], bytearray(w * bpp)
    for y in range(h):
        f, line = raw[y * (stride + 1)], bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for x in range(stride):
            a = line[x - bpp] if x >= bpp else 0
            b = prev[x]
            c = prev[x - bpp] if x >= bpp else 0
            if f == 1:
                line[x] = (line[x] + a) & 255
            elif f == 2:
                line[x] = (line[x] + b) & 255
            elif f == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif f == 4:
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(bytes(line))
        prev = line
    return w, bpp, rows


def parse_crop(text):
    """(x, y, w, h) from "x,y,w,h" (whole numbers, w and h above 0); None for an empty text."""
    if not text:
        return None
    try:
        x, y, w, h = (int(v) for v in text.split(","))
    except ValueError:
        raise ValueError("crop is x,y,w,h in whole pixels, not %r" % text)
    if w <= 0 or h <= 0 or x < 0 or y < 0:
        raise ValueError("crop %r: x and y from 0, w and h above 0" % text)
    return x, y, w, h


def crop_rows(w, bpp, rows, crop):
    """(width, bpp, rows) of the crop of a picture, clipped to it (the whole picture without a crop)."""
    if crop is None:
        return w, bpp, rows
    x, y, cw, ch = crop
    x0, y0 = min(x, w), min(y, len(rows))
    x1, y1 = min(x + cw, w), min(y + ch, len(rows))
    return x1 - x0, bpp, [row[x0 * bpp:x1 * bpp] for row in rows[y0:y1]]


def rgb(bpp, row):
    """An 8-bit row of 1 to 4 channels as RGB bytes (grey spread, alpha dropped)."""
    if bpp == 3:
        return bytes(row)
    out = bytearray()
    for i in range(0, len(row), bpp):
        px = row[i:i + bpp]
        out += bytes((px[0],) * 3) if bpp in (1, 2) else bytes(px[:3])
    return bytes(out)


def write_png(path, w, rows):
    """An 8-bit RGB PNG of RGB `rows` (filter 0)."""
    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)
    raw = b"".join(b"\x00" + r for r in rows)
    data = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, len(rows), 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 6)) + chunk(b"IEND", b"")
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_bytes(data)


GAP = 8


def sheet(a, b, out, tolerance=8, crop=None):
    """Writes before | after | diff side by side (the crop of each) to `out`; the diff is the after at a third of its
    brightness with every changed pixel red. False (nothing written) when the sizes differ."""
    wa, pa, ra = crop_rows(*png_rows(a), crop)
    wb, pb, rb = crop_rows(*png_rows(b), crop)
    if (wa, len(ra)) != (wb, len(rb)) or wa == 0 or not ra:
        return False
    gap = bytes((40, 40, 40)) * GAP
    rows = []
    for x, y in zip(ra, rb):
        x3, y3 = rgb(pa, x), rgb(pb, y)
        d = bytearray(v // 3 for v in y3)
        for i in range(wa):
            if any(abs(x[i * pa + k] - y[i * pb + k]) > tolerance for k in range(min(pa, pb))):
                d[i * 3:i * 3 + 3] = b"\xff\x20\x20"
        rows.append(x3 + gap + y3 + gap + bytes(d))
    write_png(out, wa * 3 + GAP * 2, rows)
    return True


def changed_pixels(a, b, tolerance=8, crop=None):
    """Pixels where one channel moved more than tolerance (inside `crop` when given); None when the sizes differ."""
    wa, pa, ra = png_rows(a)
    wb, pb, rb = png_rows(b)
    if (wa, pa, len(ra)) != (wb, pb, len(rb)):
        return None
    wa, pa, ra = crop_rows(wa, pa, ra, crop)
    wb, pb, rb = crop_rows(wb, pb, rb, crop)
    n = 0
    for x, y in zip(ra, rb):
        if x == y:
            continue
        for i in range(0, len(x), pa):
            if any(abs(x[i + k] - y[i + k]) > tolerance for k in range(pa)):
                n += 1
    return n


def shots(bundles):
    """{flat name: path} for every shot under the bundles folder."""
    found = sorted(p for p in Path(bundles).glob("*/*/*.png") if not p.name.endswith(".diff.png"))
    sizes = {}
    for p in found:
        sizes.setdefault(p.parent.parent.name, set()).add(p.parent.name)
    out = {}
    for p in found:
        scen, size = p.parent.parent.name, p.parent.name
        name = "%s__%s%s.png" % (scen, p.stem, "@" + size if len(sizes[scen]) > 1 else "")
        out[name] = p
    return out


AUDIT_KINDS = ("cut", "edge", "overlap", "spill")


def audit_lines(audits):
    """`audit.txt`'s lines from {flat name: audit reply}: a head count, then per shot with a problem (most first) its
    counts and each problem listed (a cut's full text and what showed; an overlap's two texts)."""
    rows = []
    for name, a in audits.items():
        counts = {k: int((a.get("counts") or {}).get(k, 0)) for k in AUDIT_KINDS}
        if sum(counts.values()):
            rows.append((name, counts, a))
    rows.sort(key=lambda r: (-sum(r[1].values()), r[0]))
    lines = ["[audit] %d of %d shot(s) with a text problem" % (len(rows), len(audits))]
    for name, counts, a in rows:
        lines.append("%s (%s) screen %s scale %s" % (name, ", ".join("%s %d" % (k, n) for k, n in counts.items() if n), a.get("screen", "?"), a.get("scale", "?")))
        for c in a.get("cut") or []:
            lines.append("  cut %r shown %r at %s" % (c.get("text"), c.get("shown"), c.get("at")))
        for e in a.get("edge") or []:
            lines.append("  edge %r at %s" % (e.get("text"), e.get("at")))
        for o in a.get("overlap") or []:
            lines.append("  overlap %r / %r at %s / %s" % (o.get("a"), o.get("b"), o.get("at_a"), o.get("at_b")))
        for p in a.get("spill") or []:
            lines.append("  spill %r out of %s at %s" % (p.get("text"), p.get("widget"), p.get("at")))
    return lines


def collect(bundles, out, crop=None):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    for old in list(out.glob("*.png")) + list(out.glob("crops/*.png")) + list(out.glob("*.audit.json")) + list(out.glob("audit.txt")):
        old.unlink()
    found = shots(bundles)
    audits = {}
    for name, p in found.items():
        shutil.copyfile(p, out / name)
        aud = p.with_name(p.stem + ".audit.json")
        if aud.is_file():
            shutil.copyfile(aud, out / (name[:-4] + ".audit.json"))
            try:
                audits[name] = json.loads(aud.read_text(encoding="utf-8"))
            except ValueError:
                audits[name] = {"counts": {}, "screen": "unreadable audit"}
        if crop is not None:
            w, bpp, rows = crop_rows(*png_rows(p), crop)
            if w and rows:
                write_png(out / "crops" / name, w, [rgb(bpp, r) for r in rows])
    (out / "index.txt").write_text("".join(n + "\n" for n in found), encoding="utf-8")
    if audits:
        (out / "audit.txt").write_text("\n".join(audit_lines(audits)) + "\n", encoding="utf-8")
    return sorted(found)


def compare(before, after, tolerance=8, pixels=300, crop=None, sheets=False):
    """([(name, changed pixels or None for a new size)], new, gone, unchanged count); `sheets`: a before | after | diff
    sheet per changed shot in `<after>/compare/`."""
    a = {p.name: p for p in Path(before).glob("*.png")}
    b = {p.name: p for p in Path(after).glob("*.png")}
    for old in Path(after).glob("compare/*.png"):
        old.unlink()
    changed, same = [], 0
    for n in sorted(a.keys() & b.keys()):
        if a[n].read_bytes() == b[n].read_bytes():
            same += 1
            continue
        k = changed_pixels(a[n], b[n], tolerance, crop)
        if k is None or k > pixels:
            changed.append((n, k))
            if sheets and k is not None:
                sheet(a[n], b[n], Path(after) / "compare" / n, tolerance, crop)
        else:
            same += 1
    changed.sort(key=lambda c: (-(c[1] if c[1] is not None else 1 << 40), c[0]))
    return changed, sorted(b.keys() - a.keys()), sorted(a.keys() - b.keys()), same


def report(changed, new, gone, same, tolerance=8, pixels=300):
    lines = ["[shots-all] compare: %d changed, %d new, %d gone, %d unchanged (a pixel past %d per channel; a shot past %d pixels)"
             % (len(changed), len(new), len(gone), same, tolerance, pixels)]
    lines += ["  changed %s (%s)" % (n, "new size" if k is None else "%d px" % k) for n, k in changed]
    lines += ["  new %s" % n for n in new] + ["  gone %s" % n for n in gone]
    return lines


def main(argv):
    opts = dict(a.split("=", 1) for a in argv[3:] if "=" in a)
    try:
        crop = parse_crop(opts.get("crop", ""))
    except ValueError as e:
        print("[shots-all] %s" % e)
        return 2
    if len(argv) >= 3 and argv[0] == "collect":
        names = collect(argv[1], argv[2], crop)
        print("[shots-all] %d shot(s) of %d scenario(s) in %s" % (len(names), len({n.split("__")[0] for n in names}), argv[2]))
        if (Path(argv[2]) / "audit.txt").is_file():
            print((Path(argv[2]) / "audit.txt").read_text(encoding="utf-8").splitlines()[0] + ": " + str(Path(argv[2]) / "audit.txt"))
        return 0
    if len(argv) >= 3 and argv[0] == "compare":
        tol, px = int(opts.get("tolerance", 8)), int(opts.get("pixels", 300))
        found = compare(argv[1], argv[2], tol, px, crop, opts.get("sheets", "1") != "0")
        lines = report(*found, tolerance=tol, pixels=px)
        if crop is not None:
            lines[0] += " inside crop %s" % ",".join(map(str, crop))
        if found[0] and opts.get("sheets", "1") != "0":
            lines.append("  sheets (before | after | diff): %s" % (Path(argv[2]) / "compare"))
        print("\n".join(lines))
        return 1 if opts.get("strict") == "1" and (found[0] or found[1] or found[2]) else 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
