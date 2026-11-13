#!/usr/bin/env python3
# Cuts the two 3 x 3 sheets of art_src/prompts/ai_icons.md (Art/UI batch 2026-11-12) into their in-place targets,
# reusing tools/portraits (sheets.cells_of / key_out / snap, pixelize.mode_reduce) so the cut matches ART2 and PLAYUI1.
#   python art_src/prompts/cut_ai_sheets.py keepsakes_1 <sheet.png> [--pick kaelen=7,vesper=8,elyse=9] [--out DIR]
#   python art_src/prompts/cut_ai_sheets.py services_hud_1 <sheet.png> [--pick scrap=6,...] [--out DIR]
#   python art_src/prompts/cut_ai_sheets.py landmark <image.png> <location_id> [--tint LO-HI:TO[@x0,y0,x1,y1]] [--out DIR]
#       --tint (a colour fix in code, preferences "Colour and small pixel fixes are code"): saturated, bright pixels whose
#       hue is in LO..HI (0..1) move to hue TO, toned down to a murkier version, inside the box (fractions of the cut).
#       The Tallow Vats pick: --tint 0.05-0.16:0.19@0.33,0,1,0.62 (the orange vat tops to sickly yellow-green).
#       one act-map landmark (art_src/prompts/act_map_landmarks.md): the magenta key cleared, cropped to the landmark,
#       box-downscaled to map/landmarks/<location_id>.png (96 x 96, shown at 48 logical px)
# Cells are numbered 1..9 row by row. Without --pick the first take of each icon is used. --out writes under DIR
# (a review folder) instead of the repository, keeping the same relative paths; a contact sheet of all nine cells is
# always written beside the sheet as <sheet>_cells.png.
# keepsakes_1: ART2 item-icon look, the tile kept: each cell reduced to a 32 grid, scaled to 96 x 96 (nearest).
# services_hud_1: magenta key cleared; recipe -> 32 x 32, the HUD chips -> 14 x 14, each with snap's palette and outline.
import pathlib, sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools" / "portraits"))
import pixelize as X  # noqa: E402
import sheets as S  # noqa: E402
from PIL import Image  # noqa: E402

GAME, KIT = "games/emberwake/assets", "kits/rpg/assets"
SHEETS = {
    # cell -> (name, target path, size, first-take cell)
    "keepsakes_1": {
        "cells": ["kaelen", "vesper", "maren", "hollis", "dunstan", "elyse", "kaelen", "vesper", "elyse"],
        "path": lambda n: f"{GAME}/items/accessory/keepsake_{n}.png",
        "size": lambda n: 96,
    },
    "services_hud_1": {
        "cells": ["recipe", "scrap", "tallow", "alert", "doom_clock", "scrap", "tallow", "alert", "doom_clock"],
        "path": lambda n: f"{KIT}/ui/services/recipe.png" if n == "recipe" else f"{GAME}/ui/hud/{n}.png",
        "size": lambda n: 32 if n == "recipe" else 14,
    },
}


def cut(sheet_id, sheet_path, picks, out_root):
    spec = SHEETS[sheet_id]
    im = X.clear_corner(Image.open(sheet_path).convert("RGBA"))
    boxes = S.cells_of(im, 3)
    tiles = [im.crop(b) for b in boxes]
    chosen = {}
    for i, name in enumerate(spec["cells"], 1):
        chosen.setdefault(name, i)
    chosen.update(picks)
    written = []
    for name, cell in chosen.items():
        tile, size = tiles[cell - 1], spec["size"](name)
        if sheet_id == "keepsakes_1":
            icon = X.mode_reduce(tile, 32).resize((size, size), Image.NEAREST)
        else:
            icon = S.snap(X.mode_reduce(S.key_out(tile), size))
        dest = pathlib.Path(out_root) / spec["path"](name)
        dest.parent.mkdir(parents=True, exist_ok=True)
        icon.save(dest)
        written.append(f"{name} <- cell {cell}: {dest} {icon.size[0]}x{icon.size[1]}")
    cw = 120
    board = Image.new("RGBA", (cw * 3 + 8, cw * 3 + 8), (18, 14, 18, 255))
    for i, t in enumerate(tiles):
        board.paste(t.resize((cw - 8, cw - 8), Image.NEAREST), (8 + (i % 3) * cw, 8 + (i // 3) * cw))
    contact = pathlib.Path(sheet_path).with_name(pathlib.Path(sheet_path).stem + "_cells.png")
    board.save(contact)
    written.append(f"contact: {contact}")
    return written


def tint(icon, spec):
    """--tint LO-HI:TO[@x0,y0,x1,y1]: moves the bright, saturated pixels of a hue range to another hue (a colour fix)."""
    import colorsys
    rng, _, rest = spec.partition(":")
    to, _, box = rest.partition("@")
    lo, hi = (float(v) for v in rng.split("-"))
    x0, y0, x1, y1 = (float(v) for v in box.split(",")) if box else (0, 0, 1, 1)
    w, h = icon.size; px = icon.load()
    for y in range(int(h * y0), int(h * y1)):
        for x in range(int(w * x0), int(w * x1)):
            r, g, b, a = px[x, y]
            if not a:
                continue
            hh, ss, vv = colorsys.rgb_to_hsv(r / 255, g / 255, b / 255)
            if lo <= hh <= hi and ss > 0.55 and vv > 0.55:
                r2, g2, b2 = colorsys.hsv_to_rgb(float(to), ss * 0.72, vv * 0.84)
                px[x, y] = (int(r2 * 255), int(g2 * 255), int(b2 * 255), a)
    return icon


def cut_landmark(path, location, out_root, size=96, tints=()):
    """One landmark on the magenta key -> GAME/map/landmarks/<location>.png, `size` x `size` (shown at 48 logical px).
    The source is already pixel art (the image model's), so it is only keyed, cropped with air around it and
    box-downscaled with a hard alpha edge: a palette reduce at this size smears the detail (checked on the hub takes)."""
    im = X.clear_corner(Image.open(path).convert("RGBA"))
    keyed = S.drop_specks(S.key_out(im, inset=0.0))      # ash flecks floating on the key would stretch the box
    box = keyed.getbbox()
    if not box:
        sys.exit(f"{path}: nothing left after keying out the magenta")
    fig = keyed.crop(box)
    side = int(max(fig.size) * 1.12)                       # air so the outline never touches the edge
    square = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    square.paste(fig, ((side - fig.size[0]) // 2, (side - fig.size[1]) // 2))
    icon = square.resize((size, size), Image.BOX)
    icon.putalpha(icon.getchannel("A").point(lambda v: 255 if v >= 128 else 0))
    for spec in tints:
        icon = tint(icon, spec)
    dest = pathlib.Path(out_root) / f"{GAME}/map/landmarks/{location}.png"
    dest.parent.mkdir(parents=True, exist_ok=True)
    icon.save(dest)
    return [f"{location} <- {path}: {dest} {icon.size[0]}x{icon.size[1]}"]


def main(argv):
    if argv and argv[0] == "landmark":
        if len(argv) < 3:
            sys.exit("usage: cut_ai_sheets.py landmark <image.png> <location_id> [--tint LO-HI:TO[@box]] [--out DIR]")
        out = pathlib.Path(argv[argv.index("--out") + 1]) if "--out" in argv else ROOT
        tints = [argv[i + 1] for i, a in enumerate(argv) if a == "--tint"]
        for line in cut_landmark(argv[1], argv[2], out, tints=tints):
            print(line)
        return
    if len(argv) < 2 or argv[0] not in SHEETS:
        sys.exit("usage: cut_ai_sheets.py keepsakes_1|services_hud_1 <sheet.png> [--pick name=cell,...] [--out DIR]")
    sheet_id, path = argv[0], argv[1]
    picks, out = {}, ROOT
    if "--pick" in argv:
        for kv in argv[argv.index("--pick") + 1].split(","):
            k, v = kv.split("=")
            if k not in SHEETS[sheet_id]["cells"] or not 1 <= int(v) <= 9:
                sys.exit(f"bad pick {kv}")
            picks[k] = int(v)
    if "--out" in argv:
        out = pathlib.Path(argv[argv.index("--out") + 1])
    for line in cut(sheet_id, path, picks, out):
        print(line)


if __name__ == "__main__":
    main(sys.argv[1:])
