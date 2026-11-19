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

  shots_all.py collect <bundles> <out>
  shots_all.py compare <before> <after> [tolerance=N] [pixels=N] [strict=1]

`strict=1` (the gate's `shotsexact` step, ENGINE-17: shots on the shot clock are frame-exact, so it runs with tolerance
0 and pixels 0) exits 1 when any shot changed, is new or is gone.
"""
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


def changed_pixels(a, b, tolerance=8):
    """Pixels where one channel moved more than tolerance; None when the sizes differ."""
    wa, pa, ra = png_rows(a)
    wb, pb, rb = png_rows(b)
    if (wa, pa, len(ra)) != (wb, pb, len(rb)):
        return None
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


def collect(bundles, out):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    for old in out.glob("*.png"):
        old.unlink()
    found = shots(bundles)
    for name, p in found.items():
        shutil.copyfile(p, out / name)
    (out / "index.txt").write_text("".join(n + "\n" for n in found), encoding="utf-8")
    return sorted(found)


def compare(before, after, tolerance=8, pixels=300):
    """([(name, changed pixels or None for a new size)], new, gone, unchanged count)."""
    a = {p.name: p for p in Path(before).glob("*.png")}
    b = {p.name: p for p in Path(after).glob("*.png")}
    changed, same = [], 0
    for n in sorted(a.keys() & b.keys()):
        if a[n].read_bytes() == b[n].read_bytes():
            same += 1
            continue
        k = changed_pixels(a[n], b[n], tolerance)
        if k is None or k > pixels:
            changed.append((n, k))
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
    if len(argv) == 3 and argv[0] == "collect":
        names = collect(argv[1], argv[2])
        print("[shots-all] %d shot(s) of %d scenario(s) in %s" % (len(names), len({n.split("__")[0] for n in names}), argv[2]))
        return 0
    if len(argv) >= 3 and argv[0] == "compare":
        opts = dict(a.split("=", 1) for a in argv[3:] if "=" in a)
        tol, px = int(opts.get("tolerance", 8)), int(opts.get("pixels", 300))
        found = compare(argv[1], argv[2], tol, px)
        print("\n".join(report(*found, tolerance=tol, pixels=px)))
        return 1 if opts.get("strict") == "1" and (found[0] or found[1] or found[2]) else 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
