#!/usr/bin/env python3
# Cuts the two 3 x 3 sheets of art_src/prompts/ai_icons.md (Art/UI batch 2026-11-12) into their in-place targets,
# reusing tools/portraits (sheets.cells_of / key_out / snap, pixelize.mode_reduce) so the cut matches ART2 and PLAYUI1.
#   python art_src/prompts/cut_ai_sheets.py keepsakes_1 <sheet.png> [--pick kaelen=7,vesper=8,elyse=9] [--out DIR]
#   python art_src/prompts/cut_ai_sheets.py services_hud_1 <sheet.png> [--pick scrap=6,...] [--out DIR]
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


def main(argv):
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
