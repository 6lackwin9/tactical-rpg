"""LOOK1-T: review board of one tile set.

    board.py SET_DIR --out C:/dev/localgen/review/look1/<set>/

SET_DIR is a `tilesets/<set>` folder (`<tile>/v<n>.png`, `wall_face/v<n>.png`, `<upper>/over_<lower>/m<NN>.png`).
Writes per tile a 3 x 3 sheet at 1x and 2x, per pair a sheet of the 14 pieces, the fixture map (`fixtures/sample_map.txt`,
`.` floor, `#` wall, `~` shallow water) square and, for 4:3 tiles, tilted at 1x-3x, and `index.html` with file:/// links
and under each tile its checks (size, seam ratio, colours, mean luminance).
"""
import argparse
import html
import os
import sys

from PIL import Image

import seam

HERE = os.path.dirname(os.path.abspath(__file__))
# Kit checks A13-A15 (rpgkit_content check/assets.rs) by role; a piece is cell-sized like a tile (a stage piece is
# 64 x 48 art px, drawn at 32 x 24 logical px by LOOK1-L1, as compose.py writes it).
SIZES = {
    "region": {"tile": {(32, 32), (32, 24)}, "face": {(32, 16), (32, 12)}, "piece": {(32, 32), (32, 24)}},
    "stage": {"tile": {(64, 48)}, "face": {(64, 24)}, "piece": {(64, 48)}},
}
MAX_SEAM, MAX_COLOURS, MAX_LUM = 1.5, 16, 25.0


def find_tiles(set_dir):
    """[(kind, file name, path)] of every `<kind>/v<n>.png`, sorted."""
    out = []
    for kind in sorted(os.listdir(set_dir)):
        d = os.path.join(set_dir, kind)
        if os.path.isdir(d):
            for f in sorted(os.listdir(d)):
                if f.startswith("v") and f.endswith(".png"):
                    out.append((kind, f, os.path.join(d, f)))
    return out


def find_pairs(set_dir):
    """[(upper, lower, folder)] of every `<upper>/over_<lower>` folder."""
    out = []
    for upper in sorted(os.listdir(set_dir)):
        d = os.path.join(set_dir, upper)
        if os.path.isdir(d):
            for sub in sorted(os.listdir(d)):
                if sub.startswith("over_") and os.path.isdir(os.path.join(d, sub)):
                    out.append((upper, sub[5:], os.path.join(d, sub)))
    return out


def detect_role(set_dir):
    """"stage" when the set's floor is 64 x 48, else "region"."""
    p = os.path.join(set_dir, "floor", "v0.png")
    if os.path.exists(p):
        with Image.open(p) as im:
            return "stage" if im.size == (64, 48) else "region"
    return "region"


def checks(kind, img, role="region"):
    """List of (text, ok) for a tile image."""
    img = img.convert("RGB")
    sizes = SIZES[role]["face" if kind == "wall_face" else "tile"]
    px = [img.getpixel((x, y)) for y in range(img.size[1]) for x in range(img.size[0])]
    lum = sum(0.2126 * r + 0.7152 * g + 0.0722 * b for r, g, b in px) / len(px) / 2.55
    ratio = seam.score(img)
    ncol = len(set(px))
    return [
        (f"size {img.size[0]}x{img.size[1]}", img.size in sizes),
        (f"seam {ratio:.2f}", ratio <= MAX_SEAM),
        (f"colours {ncol}", ncol <= MAX_COLOURS),
        (f"luminance {lum:.1f}%", lum <= MAX_LUM),
    ]


def tiled(img, n, scale):
    w, h = img.size
    sheet = Image.new("RGB", (w * n, h * n))
    for y in range(n):
        for x in range(n):
            sheet.paste(img, (x * w, y * h))
    return sheet.resize((w * n * scale, h * n * scale), Image.NEAREST)


def load_map(path=None):
    path = path or os.path.join(HERE, "fixtures", "sample_map.txt")
    with open(path, encoding="utf-8") as f:
        return [line.rstrip("\n") for line in f if line.strip()]


def vertex_mask(at, vx, vy, upper):
    """Corner mask at vertex (vx, vy): bit 1 cell (vx-1, vy-1), 2 (vx, vy-1), 4 (vx-1, vy), 8 (vx, vy) when
    `upper(cell char)` (LOOK1 5.4, the `aegis_ui::autotile::corner_mask` convention)."""
    cells = [at(vx - 1, vy - 1), at(vx, vy - 1), at(vx - 1, vy), at(vx, vy)]
    return sum(1 << i for i, c in enumerate(cells) if upper(c))


def _first(set_dir, kind):
    p = os.path.join(set_dir, kind, "v0.png")
    return Image.open(p).convert("RGB") if os.path.exists(p) else None


def render_map(set_dir, rows, tilted):
    """The map at 1x (a tile per cell). Tilted draws walls as blocks: the cap raised by the face height, the face under it."""
    floor = _first(set_dir, "floor")
    if floor is None:
        return None
    w, h = floor.size
    if tilted and w * 3 != h * 4:
        return None
    wall = _first(set_dir, "wall")
    face = _first(set_dir, "wall_face")
    water = _first(set_dir, "shallow_water")
    fh = face.size[1] if face else h // 2
    rise = fh if tilted else 0
    cols, nrows = len(rows[0]), len(rows)
    sheet = Image.new("RGB", (cols * w, nrows * h + rise), (10, 10, 10))

    def at(x, y):
        return rows[y][x] if 0 <= x < cols and 0 <= y < nrows else "."

    for y in range(nrows):
        for x in range(cols):
            t = water if at(x, y) == "~" and water else floor
            sheet.paste(t, (x * w, y * h + rise))
    pair = [p for p in find_pairs(set_dir) if p[0] == "floor" and p[1] == "shallow_water"]
    if pair and water:
        for vy in range(1, nrows):
            for vx in range(1, cols):
                m = vertex_mask(at, vx, vy, lambda c: c != "~")
                p = os.path.join(pair[0][2], f"m{m:02d}.png")
                if 0 < m < 15 and os.path.exists(p):
                    sheet.paste(Image.open(p).convert("RGB"), (vx * w - w // 2, vy * h - h // 2 + rise))
    for y in range(nrows):
        for x in range(cols):
            if at(x, y) != "#":
                continue
            if wall:
                sheet.paste(wall, (x * w, y * h))
            if face and at(x, y + 1) != "#":
                sheet.paste(face, (x * w, y * h + (h if tilted else h - fh)))
    return sheet


def _uri(path):
    return "file:///" + os.path.abspath(path).replace("\\", "/")


def build(set_dir, out_dir, role="auto"):
    role = detect_role(set_dir) if role == "auto" else role
    os.makedirs(out_dir, exist_ok=True)
    items = []

    def save(img, name, scale=1):
        if scale != 1:
            img = img.resize((img.size[0] * scale, img.size[1] * scale), Image.NEAREST)
        p = os.path.join(out_dir, name)
        img.save(p)
        return p

    def link(p):
        return f'<a href="{_uri(p)}"><img src="{_uri(p)}"></a>'

    for kind, fname, path in find_tiles(set_dir):
        img = Image.open(path).convert("RGB")
        stem = f"{kind}_{fname[:-4]}"
        a = save(tiled(img, 3, 1), f"tile_{stem}_x1.png")
        b = save(tiled(img, 3, 2), f"tile_{stem}_x2.png")
        line = " | ".join(
            f'<span style="color:{"#4c4" if ok else "#e55"}">{html.escape(t)}</span>' for t, ok in checks(kind, img, role)
        )
        items.append(f"<div><h3>{html.escape(kind)}/{fname}</h3>{link(a)} {link(b)}<p>{line}</p></div>")
    for upper, lower, folder in find_pairs(set_dir):
        files = [os.path.join(folder, f"m{m:02d}.png") for m in range(1, 15)]
        have = [f for f in files if os.path.exists(f)]
        if not have:
            continue
        with Image.open(have[0]) as first:
            w, h = first.size
        sheet = Image.new("RGB", (7 * (w + 2), 2 * (h + 2)), (30, 30, 30))
        for i, f in enumerate(files):
            if os.path.exists(f):
                sheet.paste(Image.open(f).convert("RGB"), ((i % 7) * (w + 2), (i // 7) * (h + 2)))
        bad = []
        for f in have:
            with Image.open(f) as im:
                if im.size not in SIZES[role]["piece"]:
                    bad.append(f"{os.path.basename(f)} {im.size[0]}x{im.size[1]}")
        note = f' <span style="color:#e55">wrong size: {html.escape(", ".join(bad))}</span>' if bad else ""
        p = save(sheet, f"pieces_{upper}_over_{lower}.png")
        items.append(f"<div><h3>{upper} over {lower}: {len(have)}/14 pieces{note}</h3>{link(p)}</div>")
    rows = load_map()
    sq = render_map(set_dir, rows, False)
    if sq is not None:
        items.append(f"<div><h3>map, square 1x</h3>{link(save(sq, 'map_square.png'))}</div>")
    tl = render_map(set_dir, rows, True)
    if tl is not None:
        for s in (1, 2, 3):
            items.append(f"<div><h3>map, tilted {s}x</h3>{link(save(tl, f'map_tilted_x{s}.png', s))}</div>")
    page = (
        '<!doctype html><meta charset="utf-8"><title>tiles</title>'
        '<body style="background:#222;color:#ddd;font-family:sans-serif">'
        f"<h2>{html.escape(os.path.basename(os.path.abspath(set_dir)))}</h2>" + "\n".join(items) + "</body>"
    )
    index = os.path.join(out_dir, "index.html")
    with open(index, "w", encoding="utf-8") as f:
        f.write(page)
    return index


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("set_dir")
    ap.add_argument("--out", required=True)
    ap.add_argument("--role", choices=["auto", "region", "stage"], default="auto")
    a = ap.parse_args(argv)
    print(_uri(build(a.set_dir, a.out, a.role)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
