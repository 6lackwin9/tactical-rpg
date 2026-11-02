"""LOOK1-T, REGIONGROUND1-T: review boards of one tile set.

    board.py SET_DIR --out C:/dev/localgen/review/look1/<set>/
    board.py SET_DIR --ground --out DIR [--figures ROOT] [--layout FIXTURE] [--set-colours] [--wear X] [--props FILE]
    board.py SET_DIR --mix --out DIR [--layout FIXTURE] [--set-colours] [--wear X] [--props FILE]

--mix: one picture, `mix.png`: the set's grounds laid out as a small map (`fixtures/mix_outdoor.txt`, after the mock; or
FIXTURE) with their real textures, so sameness or too strong a ground shows at a glance, nothing written over the map; beside
it the grounds, how far each sits from its colour, the pairs that are too alike and the warnings. A ground without art draws
its row's flat colour (EN 2.2). --set-colours sets each texture's mean to its row's colour first, as `flatten.py --colour`
does; off, the art shows as it is. The picture comes from the engine's `ground_compose` (EN 6), so it is the game's: its
edges, wear, strays, tone and random patches. --wear X: the table's wear times X (default 1). --props FILE: props and units
drawn over the map at their own pixels, sorted by feet (`fixtures/mix_outdoor_props.txt`), because bare shapes cannot be judged.

--ground: the set's `ground/<id>/v0.png`: each ground repeated over 16 x 10 and 32 x 20 cells at 1x-3x with its scores and
pass or fail (no tone, so a loop shows); each pair on a straight edge, a T crossing, a 1-cell lane and a lone cell, each
through `ground_compose`; a figures sheet (each ground under the six class bodies, a crate, a tent and a wall
face, at 1x-2x) with `read_contrast` under each option; `index.html` with file:/// links. ROOT is the asset layer of the
figures (default the RPG kit's). The board holds the mix too, and every pair's separation.

SET_DIR is a `tilesets/<set>` folder (`<tile>/v<n>.png`, `wall_face/v<n>.png`, `<upper>/over_<lower>/m<NN>.png`).
Writes per tile a 3 x 3 sheet at 1x and 2x, per pair a sheet of the 14 pieces, the fixture map (`fixtures/sample_map.txt`,
`.` floor, `#` wall, `~` shallow water) square and, for 4:3 tiles, tilted at 1x-3x, and `index.html` with file:/// links
and under each tile its checks (size, seam ratio, colours, mean luminance).
"""
import argparse
import html
import json
import math
import os
import re
import string
import subprocess
import sys
import tempfile

from PIL import Image, ImageDraw, ImageFont

import flatten
import groundcheck
import seam

HERE = os.path.dirname(os.path.abspath(__file__))
_composer = None                       # the built ground_compose, once a run
# Kit checks A13-A15 (rpgkit_content check/assets.rs) by role; a piece is cell-sized like a tile (a stage piece is
# 64 x 48 art px, drawn at 32 x 24 logical px by LOOK1-L1, as compose.py writes it).
SIZES = {
    "region": {"tile": {(32, 32), (32, 24)}, "face": {(32, 16), (32, 12), (128, 48)}, "piece": {(32, 32), (32, 24)}},
    "stage": {"tile": {(64, 48)}, "face": {(64, 24)}, "piece": {(64, 48)}},
    "ground": {"tile": {(256, 256)}, "face": set(), "piece": set()},
}
MAX_SEAM, MAX_COLOURS, MAX_LUM = 1.5, 16, 25.0

# --ground. Spec section 3: id -> order, wear, strays, tone, colour, patches. `data/catalog/grounds.json` has no rows until D1.
GROUND_ROWS = {
    "water": (5, 0.0, False, 0.04, "#0e1018", 0.0),
    "ash": (10, 0.03, False, 0.13, "#2a2622", 0.10),
    "soot_yard": (12, 0.03, False, 0.13, "#1e1e23", 0.10),
    "scrub": (20, 0.03, False, 0.13, "#2b2d22", 0.0),
    "moss": (22, 0.03, False, 0.13, "#20261f", 0.0),
    "scorched": (30, 0.03, False, 0.13, "#1c1917", 0.0),
    "tallow": (32, 0.03, False, 0.13, "#2a251e", 0.0),
    "mud": (40, 0.03, False, 0.13, "#231d18", 0.075),
    "packed_dirt": (50, 0.05, False, 0.13, "#352c23", 0.10),
    "dirt_road": (60, 0.05, False, 0.13, "#3c3126", 0.075),
    "flagstone": (70, 0.05, False, 0.08, "#373636", 0.0),
    "cobble_road": (78, 0.05, False, 0.08, "#2e3137", 0.0),
    "paved_road": (80, 0.05, False, 0.08, "#3e3f42", 0.0),
    "iron_catwalk": (90, 0.03, False, 0.08, "#303032", 0.0),
}
OTHER_ROW = (100, 0.06, False, 0.13, "#2a2622", 0.0)   # a ground the table does not list
CELL_PX = 16                           # art px per cell (EN 1.1)
REPEATS = ((16, 10), (32, 20))         # cells
PAIR_SIZE = (12, 8)                    # cells of one layout
SEED = 11                              # the job's seed
WEAR_MAX = 3.0                         # --wear: the table's largest wear, .10, times this is EN's limit of .30
LEGEND_CHARS = string.digits + string.ascii_uppercase + string.ascii_lowercase   # ascending, so any JSON reader keeps the order
COMPOSE_TIMEOUT = 900                  # s for a build or a compose, a first build included
BLOCKED_TRIES = 3                      # runs of a binary Smart App Control blocks
CLASS_BODIES = ("brawler", "mage", "priest", "ranger", "rogue", "warrior")
FIGURE_PROPS = ("crate_stack", "tent")
READ_OPTIONS = ("shadow", "pool", "rim")
MIX_LAYOUT = os.path.join(HERE, "fixtures", "mix_outdoor.txt")
MIX_SCALE = 2                          # physical px per art px, as at 1280 x 720
MIX_PANEL = 300                        # px of the panel beside the map
MAX_MIX_CELLS = 128                    # a layout's side, refused above it
MAX_PROPS = 256                        # lines of a props file, refused above it
MAX_MASTER = 4096                      # px of a texture's side, refused above it
MEAN_OFF = 1.0                         # the panel marks a texture further than this from its row's colour
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
KIT_ASSETS = os.path.join(REPO, "kits", "rpg", "assets")
ART_ROOTS = (os.path.join(REPO, "games", "emberwake", "assets"), KIT_ASSETS)
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


def row_of(ground_id):
    return GROUND_ROWS.get(ground_id, OTHER_ROW)


def cargo_env():
    """The environment cargo runs in. As `tools/fc.sh`: a linked worktree (its `.git` is a file) builds into its own folder,
    so a board in one lane never waits on another's build."""
    env = dict(os.environ)
    if "CARGO_TARGET_DIR" not in env and os.path.isfile(os.path.join(REPO, ".git")):
        with open(os.path.join(REPO, ".cargo", "config.toml"), encoding="utf-8") as f:
            base = re.search(r'^target-dir = "(.*)"', f.read(), re.M)
        if base:
            env["CARGO_TARGET_DIR"] = f"{base.group(1)}-{os.path.basename(REPO)}"
    return env


def run_tool(args, what, env=None):
    """`args` run to their end; (stdout). ValueError when they time out or fail, with their message; OSError when they cannot
    start."""
    try:
        done = subprocess.run(args, cwd=REPO, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace",
                              timeout=COMPOSE_TIMEOUT)
    except subprocess.TimeoutExpired:
        raise ValueError(f"{what}: no end after {COMPOSE_TIMEOUT} s") from None
    if done.returncode != 0:
        raise ValueError(f"{what}: {done.stderr.strip()[-2000:]}")
    return done.stdout


def composer():
    """The path of `ground_compose` (EN 6), the one implementation of the edge. cargo decides whether it needs building, once
    a run: a board runs it once per picture, and `cargo run` per pair would cost a second each."""
    global _composer
    if _composer is None:
        out = run_tool(["cargo", "build", "-q", "-p", "aegis_ui", "--bin", "ground_compose", "--message-format=json-render-diagnostics"],
                       "building ground_compose", cargo_env())
        found = [m.get("executable") for m in map(json.loads, out.splitlines())
                 if m.get("reason") == "compiler-artifact" and m.get("target", {}).get("name") == "ground_compose"]
        if not found or not found[-1]:
            raise ValueError("building ground_compose: cargo named no executable")
        _composer = found[-1]
    return _composer


def compose_file(job_path, out_path):
    """Runs the composer on a job file. Windows Smart App Control sometimes blocks a freshly linked binary (os error 4551,
    `engine/docs/PORT_NOTES.md`): that one is deleted and relinked, as `tools/fc.sh` does."""
    global _composer
    for attempt in range(BLOCKED_TRIES):
        exe = composer()
        try:
            return run_tool([exe, job_path, out_path], "ground_compose")
        except OSError as e:
            if getattr(e, "winerror", None) != 4551 or attempt == BLOCKED_TRIES - 1:
                raise
            os.remove(exe)
            _composer = None
    raise AssertionError("unreachable")


def layer_ids(grid):
    """The grounds of `grid` in the game's layer order (`rpgkit_content`'s `plan`): those the map uses, by order, then id. A
    texture's offset comes from the place in this list, so another order is another picture."""
    return sorted({g for row in grid for g in row}, key=lambda g: (row_of(g)[0], g))


def ground_job(grid, tex_paths, wear=1.0, seed=SEED):
    """The job file's content for `grid` (rows of ground ids): a legend in the game's order, from the table's rows, each row's
    wear times `wear`. `tex_paths`: id -> PNG; a ground without one draws its row's colour. ValueError on a `wear` off
    0..WEAR_MAX, which would pass the composer's limit on the table's strongest row."""
    if not 0 <= wear <= WEAR_MAX:
        raise ValueError(f"wear {wear} is off 0..{WEAR_MAX}")
    ids = layer_ids(grid)
    if len(ids) > len(LEGEND_CHARS):
        raise ValueError(f"{len(ids)} grounds on one map, at most {len(LEGEND_CHARS)}")
    char = dict(zip(ids, LEGEND_CHARS))
    legend = {}
    for g in ids:
        _, row_wear, strays, tone, colour, patches = row_of(g)
        legend[char[g]] = {"colour": list(flatten.parse_colour(colour)), "wear": row_wear * wear, "strays": strays, "tone": tone,
                           "patches": patches}
        if g in tex_paths:
            legend[char[g]]["tex"] = os.path.abspath(tex_paths[g]).replace(os.sep, "/")
    return {"seed": seed, "cell": [CELL_PX, CELL_PX], "cells": ["".join(char[g] for g in row) for row in grid], "legend": legend}


def draw_ground(grid, tex_paths, work_dir, wear=1.0):
    """The picture of `grid` from `ground_compose`, one art px per logical px, RGB. Job and picture are `work_dir`'s."""
    fd, job_path = tempfile.mkstemp(suffix=".json", dir=work_dir)
    with os.fdopen(fd, "w", encoding="utf-8") as f:
        json.dump(ground_job(grid, tex_paths, wear), f)
    out_path = job_path[:-len(".json")] + ".png"
    compose_file(job_path, out_path)
    with Image.open(out_path) as im:
        return im.convert("RGB")


def write_textures(images, work_dir):
    """{id: path} of [(id, image)], saved in `work_dir`."""
    paths = {}
    for g, img in images:
        paths[g] = os.path.join(work_dir, f"tex_{g}.png")
        img.save(paths[g])
    return paths


def pair_sheet(lower, upper, tex_paths, layouts, work_dir, wear=1.0):
    """The layouts of a pair side by side, each a map of its own."""
    cols, rows = PAIR_SIZE
    w, h = cols * CELL_PX, rows * CELL_PX
    gap = CELL_PX // 2
    sheet = Image.new("RGB", (len(layouts) * (w + gap) - gap, h), (10, 10, 10))
    for i, cells in enumerate(layouts):
        grid = [[upper if (x, y) in cells else lower for x in range(cols)] for y in range(rows)]
        sheet.paste(draw_ground(grid, tex_paths, work_dir, wear), (i * (w + gap), 0))
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


def score_line(result, cls, ground=None):
    """The scores as coloured HTML."""
    parts = []
    for c, name, _ in groundcheck.CHECKS:
        limit = groundcheck.limits_of(cls, ground)[c]
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


def parse_layout(path):
    """(cols, rows, grid of ground ids) of a mix fixture: `size W H`, `default ID`, then `ID X0-X1 Y0-Y1` rects (inclusive,
    later over earlier), `#` comments. ValueError on a line it cannot read, a ground the table does not list, a rect off the
    map or a map past MAX_MIX_CELLS: nothing is cut to fit."""
    size = default = None
    rects = []
    with open(path, encoding="utf-8") as f:
        for n, line in enumerate(f, 1):
            parts = line.split("#")[0].split()
            where = f"{os.path.basename(path)}:{n}"
            if not parts:
                continue
            try:
                if parts[0] == "size" and len(parts) == 3:
                    if size:
                        raise ValueError("a second size")
                    size = int(parts[1]), int(parts[2])
                elif parts[0] == "default" and len(parts) == 2:
                    if default:
                        raise ValueError("a second default")
                    default = parts[1]
                elif len(parts) == 3:
                    x0, x1 = (int(v) for v in parts[1].split("-"))
                    y0, y1 = (int(v) for v in parts[2].split("-"))
                    rects.append((parts[0], x0, x1, y0, y1, where))
                else:
                    raise ValueError("not `size W H`, `default ID` or `ID X0-X1 Y0-Y1`")
            except ValueError as e:
                raise ValueError(f"{where}: {e}") from None
    if size is None or default is None:
        raise ValueError(f"{os.path.basename(path)}: needs a size and a default")
    cols, rows = size
    if not (1 <= cols <= MAX_MIX_CELLS and 1 <= rows <= MAX_MIX_CELLS):
        raise ValueError(f"{os.path.basename(path)}: size {cols} x {rows} is off 1..{MAX_MIX_CELLS}")
    if default not in GROUND_ROWS:
        raise ValueError(f"{os.path.basename(path)}: default {default} is not in the table")
    grid = [[default] * cols for _ in range(rows)]
    for g, x0, x1, y0, y1, where in rects:
        if g not in GROUND_ROWS:
            raise ValueError(f"{where}: ground {g} is not in the table")
        if not (0 <= x0 <= x1 < cols and 0 <= y0 <= y1 < rows):
            raise ValueError(f"{where}: rect {x0}-{x1} {y0}-{y1} is off the {cols} x {rows} map")
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                grid[y][x] = g
    return cols, rows, grid


def art_file(path, where):
    """`path` as RGBA. ValueError past MAX_MASTER."""
    with Image.open(path) as im:
        if max(im.size) > MAX_MASTER:
            raise ValueError(f"{where}: {path} is over {MAX_MASTER} px a side")
        return im.convert("RGBA")


def find_art(kind, name, roots, where):
    """The picture of a `prop` (its lowest version) or a `unit` (frame 0 of its sheet), from the first of `roots` that holds it."""
    for root in roots:
        folder = os.path.join(root, f"{kind}s", *name.split("/"))
        if kind == "unit":
            if os.path.exists(os.path.join(folder, "sheet.png")):
                with open(os.path.join(folder, "clips.json"), encoding="utf-8") as f:
                    fw, fh = json.load(f)["frame"]
                return art_file(os.path.join(folder, "sheet.png"), where).crop((0, 0, fw, fh))
        elif os.path.isdir(folder):
            versions = [(int(v[1:-4]), v) for v in os.listdir(folder) if re.fullmatch(r"v[0-9]+\.png", v)]
            if versions:
                return art_file(os.path.join(folder, min(versions)[1]), where)
    raise ValueError(f"{where}: no {kind} {name}")


def parse_props(path, cols, rows, roots=None):
    """[(art RGBA, x, y)] of a props file by feet, front last: `prop FOLDER X Y` or `unit FOLDER X Y`, `#` comments, X and Y the
    foot's place in cells (fractions welcome). FOLDER is under `props/` or `units/` of the game's assets, then the kit's. In
    a tie, the file's order. ValueError on a line it cannot read, art it cannot find, a foot off the map or past MAX_PROPS."""
    roots = ART_ROOTS if roots is None else roots
    items = []
    with open(path, encoding="utf-8") as f:
        for n, line in enumerate(f, 1):
            parts = line.split("#")[0].split()
            where = f"{os.path.basename(path)}:{n}"
            if not parts:
                continue
            if len(items) == MAX_PROPS:
                raise ValueError(f"{where}: at most {MAX_PROPS} props")
            if len(parts) != 4 or parts[0] not in ("prop", "unit"):
                raise ValueError(f"{where}: not `prop FOLDER X Y` or `unit FOLDER X Y`")
            if not re.fullmatch(r"[a-z0-9_]+(/[a-z0-9_]+)*", parts[1]):
                raise ValueError(f"{where}: {parts[1]} is not a folder name")
            try:
                x, y = float(parts[2]), float(parts[3])
            except ValueError as e:
                raise ValueError(f"{where}: {e}") from None
            if not (0 <= x <= cols and 0 <= y <= rows):
                raise ValueError(f"{where}: foot {parts[2]} {parts[3]} is off the {cols} x {rows} map")
            items.append((find_art(parts[0], parts[1], roots, where), x, y))
    return sorted(items, key=lambda item: item[2])


def draw_props(the_map, items, cell):
    """`items` (`parse_props`) over `the_map` at their own pixels, `cell` px a cell, feet at the art's bottom centre. Art past
    the map's side is cut there, as the map area's clip does."""
    out = the_map.convert("RGBA")
    for art, x, y in items:
        left, top = math.floor(x * cell - art.width / 2), math.floor(y * cell - art.height)
        out.alpha_composite(art, (max(left, 0), max(top, 0)), (max(-left, 0), max(-top, 0)))
    return out.convert("RGB")


def load_masters(set_dir, set_colours=False):
    """[(id, master)] of a set's grounds by the table's order, each set to its row's colour when asked. ValueError when the
    set has none or a texture's side is past MAX_MASTER."""
    ground_dir = os.path.join(set_dir, "ground")
    found = groundcheck.find_grounds(ground_dir)
    if not found:
        raise ValueError(f"{ground_dir} holds no <id>/v0.png")
    masters = []
    for g, path in sorted(found, key=lambda f: (row_of(f[0])[0], f[0])):
        with Image.open(path) as im:
            if max(im.size) > MAX_MASTER:
                raise ValueError(f"{path}: {im.size[0]}x{im.size[1]} is over {MAX_MASTER} px a side")
            master = im.convert("RGB")
        if set_colours:
            master = flatten.set_mean(master, flatten.parse_colour(row_of(g)[4]))
        masters.append((g, master))
    return masters


def mix_notes(textures, masters):
    """The lines beside the mix: (pairs too alike, built grounds without stones)."""
    alike = [f"{a} / {b}: colour {r['colour']:.1f}, pattern {r['pattern']:.2f}"
             for a, b, r in groundcheck.pair_report(textures) if not r["clear"]]
    plain = [f"{g}: {groundcheck.stones(m)} stones, no joints" for g, m in masters if groundcheck.stone_warning(m, g)]
    return alike, plain


def mix_picture(masters, layout_path=MIX_LAYOUT, wear=1.0, props_path=None):
    """The map from `ground_compose` at MIX_SCALE, its props and units over it, beside its panel; ([warnings], image). A ground
    of the set the layout does not use is a warning: it is not on the picture."""
    cols, rows, grid = parse_layout(layout_path)
    props = parse_props(props_path, cols, rows) if props_path else []
    textures = [(g, groundcheck.form(m)) for g, m in masters]
    used = {g for row in grid for g in row}
    missing = [g for g, _ in textures if g not in used]
    with tempfile.TemporaryDirectory() as work:
        the_map = draw_ground(grid, write_textures(masters, work), work, wear)
    the_map = the_map.resize((the_map.size[0] * MIX_SCALE, the_map.size[1] * MIX_SCALE), Image.NEAREST)
    the_map = draw_props(the_map, props, CELL_PX * MIX_SCALE)
    sheet = Image.new("RGB", (the_map.size[0] + MIX_PANEL, the_map.size[1]), (14, 15, 20))
    sheet.paste(the_map, (0, 0))
    draw = ImageDraw.Draw(sheet)
    font = ImageFont.load_default()
    x, y = the_map.size[0] + 10, 8
    have = dict(textures)
    for g in sorted(used | set(have), key=lambda i: (row_of(i)[0], i)):
        target = flatten.parse_colour(row_of(g)[4])
        tex = have.get(g)
        if tex is None:
            sheet.paste(row_of(g)[4], (x, y, x + 24, y + 24))
            note = "no art: flat"
        else:
            sheet.paste(tex.crop((0, 0, 24, 24)), (x, y))
            mean = groundcheck.mean_rgb(tex)
            off = max(abs(a - b) for a, b in zip(mean, target))
            note = "#%02x%02x%02x" % tuple(int(v + 0.5) for v in mean)
            note += f" off table {off:.0f}" if off > MEAN_OFF else ""
        draw.rectangle((x - 1, y - 1, x + 24, y + 24), outline=(90, 90, 100))
        draw.text((x + 32, y), g + ("" if g in used else " (not on this layout)"), fill=(225, 225, 225), font=font)
        draw.text((x + 32, y + 12), note, fill=(150, 150, 160), font=font)
        y += 30
    alike, plain = mix_notes(textures, masters)
    y += 6
    draw.text((x, y), f"wear x {wear:g}, {len(props)} props and units", fill=(150, 150, 160), font=font)
    y += 14
    draw.text((x, y), f"{len(alike)} pairs too alike", fill=(230, 120, 90) if alike else (120, 200, 120), font=font)
    y += 14
    notes = plain + alike
    room = (sheet.size[1] - y) // 12 - 1
    for line in notes[:room]:
        draw.text((x, y), line, fill=(200, 170, 120), font=font)
        y += 12
    if len(notes) > room:
        draw.text((x, y), f"and {len(notes) - room} more on the tool's output", fill=(150, 150, 160), font=font)
    warnings = [f"{g}: not on this layout" for g in missing] + [f"too alike: {line}" for line in alike] + plain
    return warnings, sheet


def build_mix(set_dir, out_dir, layout_path=MIX_LAYOUT, set_colours=False, wear=1.0, props_path=None):
    """`mix.png` of `set_dir`; (its path, [warnings]). ValueError on a bad layout or a set without ground art."""
    warnings, sheet = mix_picture(load_masters(set_dir, set_colours), layout_path, wear, props_path)
    os.makedirs(out_dir, exist_ok=True)
    path = os.path.join(out_dir, "mix.png")
    sheet.save(path)
    return path, warnings


def build_ground(set_dir, out_dir, figures_root=KIT_ASSETS, layout_path=MIX_LAYOUT, set_colours=False, wear=1.0, props_path=None):
    """The ground board of `set_dir`; the path of its index. ValueError when the set has no ground art."""
    masters = load_masters(set_dir, set_colours)
    os.makedirs(out_dir, exist_ok=True)
    textures = [(g, groundcheck.form(master)) for g, master in masters]
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
        bad = groundcheck.failures(result, cls, g)
        stone_note = groundcheck.stone_warning(master, g)
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
                     f"<p>{score_line(result, cls, g)}</p>"
                     + "".join(f'<p style="color:#ea5">warning: {html.escape(n)}</p>' for n in (stone_note, size_note) if n)
                     + "</div>")
    mix_warnings, mix = mix_picture(masters, layout_path, wear, props_path)
    items.insert(0, f"<div><h2>The mix</h2>{thumb(save(mix, 'mix.png'))}</div>")
    for line in mix_warnings:
        print(f"warning: {line}", file=sys.stderr)
    layouts = pair_layouts()
    cell_sets = [set(cells) for _, cells in layouts]
    titles = ", ".join(t for t, _ in layouts)
    items.append(f"<h2>Pairs, lower then upper ({html.escape(titles)})</h2>")
    with tempfile.TemporaryDirectory() as work:
        tex_paths = write_textures(masters, work)
        for i, (low, tex_low) in enumerate(textures):
            for up, tex_up in textures[i + 1:]:
                sheet = pair_sheet(low, up, tex_paths, cell_sets, work, wear)
                p = save(sheet, f"pair_{low}_{up}.png")
                q = save(sheet, f"pair_{low}_{up}_x2.png", 2)
                r = groundcheck.pair_scores(tex_low, tex_up)
                warn = "" if r["clear"] else ' <span style="color:#e55">FAIL: too alike</span>'
                items.append(f"<div><h3>{html.escape(low)} / {html.escape(up)}: colour {r['colour']:.1f}, pattern {r['pattern']:.2f}{warn}</h3>"
                             f"{thumb(p)}<p>{html.escape(groundcheck.pair_line(low, up, r))}</p><p>{link(q, '2x')}</p></div>")
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
    ap.add_argument("--mix", action="store_true", help="only the one-picture mix of the set's grounds, mix.png")
    ap.add_argument("--layout", default=MIX_LAYOUT, help="with --mix or --ground: the mix fixture")
    ap.add_argument("--set-colours", action="store_true", help="with --mix or --ground: set each texture's mean to its row's colour")
    ap.add_argument("--wear", type=float, default=1.0, help="with --mix or --ground: the table's wear times this")
    ap.add_argument("--props", help="with --mix or --ground: a props file drawn over the mix (fixtures/mix_outdoor_props.txt)")
    a = ap.parse_args(argv)
    if not a.ground and not a.mix:
        print(_uri(build(a.set_dir, a.out, a.role)))
        return 0
    try:
        if a.mix:
            path, warnings = build_mix(a.set_dir, a.out, a.layout, a.set_colours, a.wear, a.props)
            for line in warnings:
                print(f"warning: {line}", file=sys.stderr)
            print(_uri(path))
        else:
            print(_uri(build_ground(a.set_dir, a.out, a.figures, a.layout, a.set_colours, a.wear, a.props)))
    except (OSError, ValueError) as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
