"""LOOK1-T, REGIONGROUND1-T: review boards of one tile set.

    board.py SET_DIR --out C:/dev/localgen/review/look1/<set>/
    board.py SET_DIR --ground --out DIR [--figures ROOT]

--ground: the set's `ground/<id>/v0.png`: each ground repeated over 16 x 10 and 32 x 20 cells at 1x-3x with its scores and
pass or fail (no tone, so a loop shows); each pair on a straight edge, a T crossing, a 1-cell lane and a lone cell, drawn
by the prototype's edge (EN 2.3, 2.4); a figures sheet (each ground under the six class bodies, a crate, a tent and a wall
face, at 1x-2x) with `read_contrast` under each option; `index.html` with file:/// links. ROOT is the asset layer of the
figures (default the RPG kit's). The composed mix is REGIONGROUND1-T2.

SET_DIR is a `tilesets/<set>` folder (`<tile>/v<n>.png`, `wall_face/v<n>.png`, `<upper>/over_<lower>/m<NN>.png`).
Writes per tile a 3 x 3 sheet at 1x and 2x, per pair a sheet of the 14 pieces, the fixture map (`fixtures/sample_map.txt`,
`.` floor, `#` wall, `~` shallow water) square and, for 4:3 tiles, tilted at 1x-3x, and `index.html` with file:/// links
and under each tile its checks (size, seam ratio, colours, mean luminance).
"""
import argparse
import html
import json
import os
import random
import sys

from PIL import Image, ImageFilter

import groundcheck
import seam

HERE = os.path.dirname(os.path.abspath(__file__))
# Kit checks A13-A15 (rpgkit_content check/assets.rs) by role; a piece is cell-sized like a tile (a stage piece is
# 64 x 48 art px, drawn at 32 x 24 logical px by LOOK1-L1, as compose.py writes it).
SIZES = {
    "region": {"tile": {(32, 32), (32, 24)}, "face": {(32, 16), (32, 12), (128, 48)}, "piece": {(32, 32), (32, 24)}},
    "stage": {"tile": {(64, 48)}, "face": {(64, 24)}, "piece": {(64, 48)}},
    "ground": {"tile": {(256, 256)}, "face": set(), "piece": set()},
}
MAX_SEAM, MAX_COLOURS, MAX_LUM = 1.5, 16, 25.0

# --ground. Spec section 3: id -> order, wear, strays, tone, colour. The catalog holds them once D1 lands; T2's mix reads it.
GROUND_ROWS = {
    "water": (5, 0.0, False, 0.04, "#0e1018"),
    "ash": (10, 0.06, False, 0.13, "#2a2622"),
    "soot_yard": (12, 0.06, False, 0.13, "#1e1e23"),
    "scrub": (20, 0.06, False, 0.13, "#2b2d22"),
    "moss": (22, 0.06, False, 0.13, "#20261f"),
    "scorched": (30, 0.06, False, 0.13, "#1c1917"),
    "tallow": (32, 0.06, False, 0.13, "#2a251e"),
    "mud": (40, 0.06, False, 0.13, "#231d18"),
    "packed_dirt": (50, 0.10, False, 0.13, "#332e28"),
    "dirt_road": (60, 0.10, False, 0.13, "#39342e"),
    "flagstone": (70, 0.10, True, 0.08, "#373636"),
    "cobble_road": (78, 0.10, True, 0.08, "#2e3137"),
    "paved_road": (80, 0.10, True, 0.08, "#3e3f42"),
    "iron_catwalk": (90, 0.06, False, 0.08, "#303032"),
}
OTHER_ROW = (100, 0.06, False, 0.13, "#2a2622")   # a ground the table does not list
CELL_PX = 16                           # art px per cell (EN 1.1)
REPEATS = ((16, 10), (32, 20))         # cells
PAIR_SIZE = (12, 8)                    # cells of one layout
SEED = 11                              # the board's noise
CLASS_BODIES = ("brawler", "mage", "priest", "ranger", "rogue", "warrior")
FIGURE_PROPS = ("crate_stack", "tent")
READ_OPTIONS = ("shadow", "pool", "rim")
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
KIT_ASSETS = os.path.join(REPO, "kits", "rpg", "assets")
ENGINE_FACE = os.path.join(REPO, "engine", "assets", "tilesets", "common_stage", "wall_face", "v0.png")


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


def pair_layouts():
    """[(title, set of upper cells)] on a PAIR_SIZE map."""
    cols, rows = PAIR_SIZE
    return [
        ("straight edge", {(x, y) for y in range(rows) for x in range(cols // 2, cols)}),
        ("T crossing", {(x, y) for y in (2, 3) for x in range(cols)} | {(x, y) for y in range(4, rows) for x in (5, 6)}),
        ("1-cell lane", {(x, 3) for x in range(9)} | {(8, y) for y in range(3, rows)}),
        ("lone cell", {(6, 4)}),
    ]


def noise_field(w, h, step, seed):
    """Value noise as a flat list of 0..1: a seeded lattice of `step` art px, resized by bicubic (the prototype's)."""
    rnd = random.Random(seed)
    lw, lh = w // step + 2, h // step + 2
    lattice = Image.frombytes("L", (lw, lh), bytes(rnd.randrange(256) for _ in range(lw * lh)))
    return [v / 255 for v in lattice.resize((w, h), Image.BICUBIC).tobytes()]


def edge_fields(w, h):
    """The noise a pair sheet shares (EN 2.1): edge, stray, bite, tone."""
    a, b = noise_field(w, h, 10, SEED + 1), noise_field(w, h, 4, SEED + 2)
    t0, t1 = noise_field(w, h, 48, SEED + 7), noise_field(w, h, 20, SEED + 8)
    return {
        "edge": [0.6 * p + 0.4 * q for p, q in zip(a, b)],
        "stray": noise_field(w, h, 3, SEED + 3),
        "bite": noise_field(w, h, 3, SEED + 4),
        "tone": [0.7 * p + 0.3 * q - 0.5 for p, q in zip(t0, t1)],
    }


def upper_field(cells):
    """EN 2.3's `bl`: the upper cells as 0..1 per art px, blurred by sigma 5.5."""
    w, h = PAIR_SIZE[0] * CELL_PX, PAIR_SIZE[1] * CELL_PX
    mask = Image.new("L", (w, h), 0)
    for x, y in cells:
        mask.paste(255, (x * CELL_PX, y * CELL_PX, (x + 1) * CELL_PX, (y + 1) * CELL_PX))
    return [v / 255 for v in mask.filter(ImageFilter.GaussianBlur(5.5)).tobytes()]


def row_of(ground_id):
    return GROUND_ROWS.get(ground_id, OTHER_ROW)


def draw_pair(lower, upper, lower_row, upper_row, field, noise):
    """The lower and upper 64 x 64 textures on one layout's field (EN 2.3, 2.4): a hard edge at art px, the worn band on
    the lower ground, strays of the upper one, bites of the lower one. Colours are never blended between grounds."""
    w, h = PAIR_SIZE[0] * CELL_PX, PAIR_SIZE[1] * CELL_PX
    tl, tu = lower.tobytes(), upper.tobytes()
    tw, th = lower.size
    out = bytearray(w * h * 3)
    tone_l = lower_row[3]
    wear, strays, tone_u = upper_row[1:4]
    for y in range(h):
        for x in range(w):
            i = y * w + x
            bl = field[i]
            v = bl + (noise["edge"][i] - 0.5) * 0.62
            tone = noise["tone"][i]
            lo = ((y % th) * tw + x % tw) * 3
            up = (((y + 21) % th) * tw + (x + 37) % tw) * 3
            scale_l, scale_u = 1 + tone_l * tone, 1 + tone_u * tone
            if v > 0.5:
                if v < 0.56 and noise["bite"][i] > 0.7:
                    c, k = tl[lo:lo + 3], scale_l * 0.95
                else:
                    c, k = tu[up:up + 3], scale_u
            elif strays and 0.16 < bl < 0.42 and noise["stray"][i] > 0.82:
                c, k = tu[up:up + 3], scale_u * 0.92
            elif v > 0.34:
                c, k = tl[lo:lo + 3], scale_l * (1 - wear)
            else:
                c, k = tl[lo:lo + 3], scale_l
            o = i * 3
            out[o], out[o + 1], out[o + 2] = min(255, int(c[0] * k)), min(255, int(c[1] * k)), min(255, int(c[2] * k))
    return Image.frombytes("RGB", (w, h), bytes(out))


def pair_sheet(lower, upper, lower_row, upper_row, fields, noise):
    """The four layouts of a pair side by side."""
    w, h = PAIR_SIZE[0] * CELL_PX, PAIR_SIZE[1] * CELL_PX
    gap = CELL_PX // 2
    sheet = Image.new("RGB", (len(fields) * (w + gap) - gap, h), (10, 10, 10))
    for i, field in enumerate(fields):
        sheet.paste(draw_pair(lower, upper, lower_row, upper_row, field, noise), (i * (w + gap), 0))
    return sheet


def repeated(tex, cells):
    """`tex` repeated over `cells` (W, H), one art px per logical px, no tone."""
    w, h = cells[0] * CELL_PX, cells[1] * CELL_PX
    sheet = Image.new("RGB", (w, h))
    for y in range(0, h, tex.size[1]):
        for x in range(0, w, tex.size[0]):
            sheet.paste(tex, (x, y))
    return sheet


def body_frame(root, body):
    """Frame 0 of a class body's sheet in `root`, RGBA; None when missing."""
    folder = os.path.join(root, "units", body, "base")
    sheet = os.path.join(folder, "sheet.png")
    clips = os.path.join(folder, "clips.json")
    if not os.path.exists(sheet) or not os.path.exists(clips):
        return None
    with open(clips, encoding="utf-8") as f:
        fw, fh = json.load(f)["frame"]
    with Image.open(sheet) as im:
        return im.convert("RGBA").crop((0, 0, fw, fh))


def find_figures(set_dir, root):
    """([(class body, frame)], [(name, art)] of the crate, the tent and the set's wall face). What is missing is left out."""
    bodies = [(b, body_frame(root, b)) for b in CLASS_BODIES]
    others = []
    for name in FIGURE_PROPS:
        p = os.path.join(root, "props", "common", name, "v0.png")
        if os.path.exists(p):
            with Image.open(p) as im:
                others.append((name, im.convert("RGBA")))
    face = os.path.join(set_dir, "wall_face", "v0.png")
    face = face if os.path.exists(face) else ENGINE_FACE
    if os.path.exists(face):
        with Image.open(face) as im:
            others.append(("wall_face", im.convert("RGBA")))
    return [(n, f) for n, f in bodies if f is not None], others


def figures_sheet(textures, figures):
    """One row per ground, its art px doubled, under the figures at their own art px: 1280 x 720's physical px."""
    if not textures or not figures:
        return None
    pad = 8
    width = sum(f.size[0] + pad for _, f in figures) + pad
    high = max(f.size[1] for _, f in figures) + 2 * pad
    sheet = Image.new("RGB", (width, high * len(textures)))
    for row, (_, tex) in enumerate(textures):
        big = tex.resize((tex.size[0] * 2, tex.size[1] * 2), Image.NEAREST)
        for y in range(0, high, big.size[1]):
            for x in range(0, width, big.size[0]):
                sheet.paste(big, (x, row * high + y))
        x = pad
        for _, f in figures:
            sheet.paste(f, (x, (row + 1) * high - pad - f.size[1]), f)
            x += f.size[0] + pad
    return sheet


def score_line(result, cls):
    """The scores as coloured HTML."""
    parts = []
    for c, name, _ in groundcheck.CHECKS:
        limit = groundcheck.LIMITS[cls][c]
        colour = "#e55" if result[c] > limit else "#4c4"
        parts.append(f'<span style="color:{colour}">{html.escape(name)} {result[c]:.2f} / {limit:g}</span>')
    return " | ".join(parts)


def read_table(textures, bodies):
    """HTML: read_contrast (and the ring step) of each class body on each ground under each option."""
    if not textures or not bodies:
        return ""
    head = "".join(f"<th>{html.escape(n)}</th>" for n, _ in bodies)
    rows = []
    for g, tex in textures:
        for option in READ_OPTIONS:
            cells = "".join(f"<td>{groundcheck.read_contrast(f, tex, option):.1f} ({groundcheck.ring_step(f, tex, option):.1f})</td>"
                            for _, f in bodies)
            rows.append(f"<tr><td>{html.escape(g)}</td><td>{option}</td>{cells}</tr>")
    return ("<h3>read_contrast (ring step in brackets)</h3><table border=1 cellspacing=0 cellpadding=3>"
            f"<tr><th>ground</th><th>option</th>{head}</tr>" + "".join(rows) + "</table>")


def build_ground(set_dir, out_dir, figures_root=KIT_ASSETS):
    """The ground board of `set_dir`; the path of its index. ValueError when the set has no ground art."""
    ground_dir = os.path.join(set_dir, "ground")
    found = groundcheck.find_grounds(ground_dir)
    if not found:
        raise ValueError(f"{ground_dir} holds no <id>/v0.png")
    os.makedirs(out_dir, exist_ok=True)
    masters, textures = [], []
    for g, path in sorted(found, key=lambda f: (row_of(f[0])[0], f[0])):
        with Image.open(path) as im:
            master = im.convert("RGB")
        masters.append((g, master))
        textures.append((g, groundcheck.form(master)))
    items = []

    def save(img, name, scale=1):
        if scale != 1:
            img = img.resize((img.size[0] * scale, img.size[1] * scale), Image.NEAREST)
        p = os.path.join(out_dir, name)
        img.save(p)
        return p

    def link(p, text):
        return f'<a href="{_uri(p)}">{html.escape(text)}</a>'

    def thumb(p):
        return f'<a href="{_uri(p)}"><img src="{_uri(p)}"></a>'

    for (g, master), (_, tex) in zip(masters, textures):
        cls = groundcheck.class_of(g)
        result = groundcheck.scores(master)
        bad = groundcheck.failures(result, cls)
        size_note = groundcheck.size_warning(master)
        if size_note:
            print(f"warning: {g}: {size_note}", file=sys.stderr)
        verdict = f'<span style="color:#e55">FAIL: {html.escape(", ".join(bad))}</span>' if bad else '<span style="color:#4c4">pass</span>'
        links = []
        first = None
        for cells in REPEATS:
            for scale in (1, 2, 3):
                p = save(repeated(tex, cells), f"repeat_{g}_{cells[0]}x{cells[1]}_x{scale}.png", scale)
                first = first or p
                links.append(link(p, f"{cells[0]} x {cells[1]} at {scale}x"))
        items.append(f"<div><h3>{html.escape(g)} ({cls}): {verdict}</h3>{thumb(first)}<p>{' | '.join(links)}</p>"
                     f"<p>{score_line(result, cls)}</p>"
                     + (f'<p style="color:#ea5">warning: {html.escape(size_note)}</p>' if size_note else "") + "</div>")
    w, h = PAIR_SIZE[0] * CELL_PX, PAIR_SIZE[1] * CELL_PX
    noise = edge_fields(w, h)
    layouts = pair_layouts()
    fields = [upper_field(cells) for _, cells in layouts]
    titles = ", ".join(t for t, _ in layouts)
    items.append(f"<h2>Pairs, lower then upper ({html.escape(titles)})</h2>")
    for i, (low, tex_low) in enumerate(textures):
        for up, tex_up in textures[i + 1:]:
            sheet = pair_sheet(tex_low, tex_up, row_of(low), row_of(up), fields, noise)
            p = save(sheet, f"pair_{low}_{up}.png")
            q = save(sheet, f"pair_{low}_{up}_x2.png", 2)
            d = groundcheck.pair_distance(tex_low, tex_up)
            warn = "" if d >= groundcheck.MIN_PAIR_DISTANCE else f' <span style="color:#ea5">warning: under {groundcheck.MIN_PAIR_DISTANCE:.0f}, the edge may not read</span>'
            items.append(f"<div><h3>{html.escape(low)} / {html.escape(up)}: means {d:.1f} apart{warn}</h3>{thumb(p)}<p>{link(q, '2x')}</p></div>")
    bodies, others = find_figures(set_dir, figures_root)
    sheet = figures_sheet(textures, bodies + others)
    if sheet is not None:
        p = save(sheet, "figures_x1.png")
        q = save(sheet, "figures_x2.png", 2)
        warnings = groundcheck.figure_warnings(textures, bodies + others)
        notes = "".join(f"<li>{html.escape(line)}</li>" for line in warnings) or "<li>none</li>"
        items.append(f"<h2>Figures on each ground</h2>{thumb(p)}<p>{link(q, '2x')}</p><h3>Figure warnings</h3><ul>{notes}</ul>")
    items.append(read_table(textures, bodies))
    page = (
        '<!doctype html><meta charset="utf-8"><title>grounds</title>'
        '<body style="background:#222;color:#ddd;font-family:sans-serif">'
        f"<h2>{html.escape(os.path.basename(os.path.abspath(set_dir)))}: grounds</h2>" + "\n".join(items) + "</body>"
    )
    index = os.path.join(out_dir, "index.html")
    with open(index, "w", encoding="utf-8") as f:
        f.write(page)
    return index


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
    ap.add_argument("--ground", action="store_true", help="the set's ground/<id>/v0.png instead of its tiles")
    ap.add_argument("--figures", default=KIT_ASSETS, help="with --ground: the asset layer of the class bodies and props")
    a = ap.parse_args(argv)
    if not a.ground:
        print(_uri(build(a.set_dir, a.out, a.role)))
        return 0
    try:
        print(_uri(build_ground(a.set_dir, a.out, a.figures)))
    except ValueError as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
