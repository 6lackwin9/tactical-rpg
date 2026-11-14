#!/usr/bin/env python3
"""Cut the Sunk Ward prop sheets into props/sunk_ward/<art>/v0.png (Art/UI 2026-11-15).

    python art_src/prompts/cut_props.py <folder of sheet PNGs> [--out DIR] [--only art,art]

The sheets are the web-queue results (`<sheet id>_q<time>_<take>.png`). PICKS says which sheet, take and cell each
art comes from: the first run's keepers (props that read the same from any angle, and the flat decals) and the
front-on re-run (sunk_ward_props.md section 4). Sizes and layers are read from the tables in sunk_ward_props.md.
Each cell is keyed, trimmed, scaled smoothly to fit its PNG size (scale only: the art is pixel art already) and
placed bottom-centred (standing) or centred (flat), with `sheets.prop_image`. --out writes a review copy and a
contact sheet instead of the repository files.
"""
import glob
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools" / "portraits"))
import pixelize as X  # noqa: E402
import sheets as S  # noqa: E402
from PIL import Image  # noqa: E402

GAME = "games/emberwake/assets"
# sheet id -> (grid, {art: (take, cell)}); cells count from 1, row by row
PICKS = {
    "props_sunk_undercroft": (4, {"retort_shelf": (1, 5), "herb_rack": (1, 7), "anvil": (1, 9), "scrap_heap": (1, 10),
                                  "brazier": (1, 11), "mission_board": (1, 15)}),
    "props_sunk_drains": (3, {"lock_tablet": (1, 2), "lantern_pole": (1, 3), "culvert_gate": (1, 4), "silt_bank": (1, 7)}),
    "props_sunk_streets": (4, {"tenement_row": (1, 2), "watch_armory": (1, 3), "watch_torch": (1, 5),
                               "rust_pennant": (1, 6), "spiked_barricade": (1, 7), "toll_pen_gate": (2, 11),
                               "violet_lamp_post": (1, 12), "brick_wall_run": (1, 13), "brick_breach": (1, 14)}),
    "props_sunk_works": (4, {"conveyor_chute": (1, 2), "brass_tank": (1, 3), "pipe_run": (2, 4), "tallow_vat": (1, 5),
                             "watch_wall": (2, 8), "barrel_stack": (2, 9), "toll_drum": (1, 10), "crane_arm": (2, 12)}),
    "props_sunk_ground": (3, {"street_grate": (1, 1), "violet_pool": (2, 2), "ash_drift": (1, 3), "bone_meal_heap": (1, 4),
                              "wedge_pile": (1, 5), "tallow_spill": (1, 6), "slag_channel": (1, 7)}),
    "props_sunk_interiors": (4, {"goods_shelf": (1, 2), "hearth_stove": (1, 5), "candle_stand": (1, 7),
                                 "weapon_rack": (1, 8), "bone_rack": (1, 11), "copper_still": (1, 12), "rag_pile": (1, 13)}),
    "props_sunk_front_a": (4, {"sluice_arch": (1, 1), "sluice_tunnel": (1, 2), "drain_arch": (1, 3), "vault_pillar": (1, 4),
                               "crypt_bars": (1, 5), "iron_ladder": (1, 6), "furnace_mouth": (1, 7), "portcullis": (1, 8),
                               "tenement_block": (1, 9), "ash_shack": (1, 10), "toll_booth": (1, 11),
                               "slit_window_tower": (1, 12), "cliff_bastion": (2, 13), "cage_platform": (1, 14),
                               "winch_drum": (1, 15), "catwalk_stair": (1, 16)}),
    "props_sunk_front_b": (4, {"shop_counter": (1, 1), "cot": (1, 2), "table_stools": (1, 3), "shrine_altar": (1, 4),
                               "strongbox": (1, 5), "workbench": (1, 6), "apothecary_slab": (1, 7), "tinker_hearth": (1, 8),
                               "kiln_bunk": (1, 9), "stasis_slab": (2, 10), "striped_barrier": (1, 11), "pen_fence": (1, 12),
                               "stone_steps": (1, 13), "drain_grate": (1, 14), "varka_skiff": (1, 15), "ring_post": (1, 16)}),
    "props_sunk_kiln_front": (1, {"fourth_kiln": (1, 1)}),
}
KEEP_WATER = {"drain_grate"}   # drawn on its own black water edge to edge: kept whole, not keyed


def specs():
    """art -> ((w, h) px, layer) from the tables of sunk_ward_props.md."""
    text = (pathlib.Path(__file__).with_name("sunk_ward_props.md")).read_text(encoding="utf-8")
    out = {}
    for m in re.finditer(r"^\| `([a-z0-9_]+)` \| (\d+) x (\d+) \| [^|]+ \| (sort|ground|roof) \|", text, re.M):
        out[m.group(1)] = ((int(m.group(2)), int(m.group(3))), m.group(4))
    return out


def cut(folder, out_root, only=None):
    sp, done = specs(), []
    for sid, (grid, picks) in PICKS.items():
        sheets = {}
        for art, (take, cell) in picks.items():
            if only and art not in only:
                continue
            if take not in sheets:
                found = glob.glob(str(pathlib.Path(folder) / f"{sid}_q*_{take}.png"))
                if not found:
                    print(f"{art}: no sheet {sid} take {take}"); continue
                im = X.clear_corner(Image.open(found[0]).convert("RGBA"))
                sheets[take] = (im, S.cells_of(im, grid) if grid > 1 else [(0, 0) + im.size])
            im, boxes = sheets[take]
            size, layer = sp[art]
            tile = im.crop(boxes[cell - 1])
            if art in KEEP_WATER:
                w, h = tile.size; d = int(w * 0.03)
                img = tile.crop((d, d, w - d, h - d)).resize(size, Image.LANCZOS)
            else:
                img, _ = S.prop_image(tile, size, "ground" if layer == "ground" else "sort", reduce="scale")
            dest = pathlib.Path(out_root) / GAME / "props" / "sunk_ward" / art / "v0.png"
            dest.parent.mkdir(parents=True, exist_ok=True)
            img.save(dest)
            done.append((art, img))
    return done


def contact(done, path, cols=10):
    cw, ch = 200, 200
    rows = (len(done) + cols - 1) // cols
    board = Image.new("RGBA", (cols * cw, rows * ch), (58, 54, 60, 255))
    from PIL import ImageDraw
    d = ImageDraw.Draw(board)
    for i, (art, img) in enumerate(done):
        k = min(1.0, (cw - 8) / img.width, (ch - 22) / img.height)
        small = img.resize((max(1, round(img.width * k)), max(1, round(img.height * k))), Image.LANCZOS)
        x, y = (i % cols) * cw, (i // cols) * ch
        board.alpha_composite(small, (x + (cw - small.width) // 2, y + ch - 18 - small.height))
        d.text((x + 4, y + ch - 14), f"{art} {img.width}x{img.height}"[:30], fill=(255, 255, 0))
    board.save(path)


def main(argv):
    if not argv:
        sys.exit(__doc__)
    out = pathlib.Path(argv[argv.index("--out") + 1]) if "--out" in argv else ROOT
    only = set(argv[argv.index("--only") + 1].split(",")) if "--only" in argv else None
    done = cut(argv[0], out, only)
    print(f"{len(done)} props cut -> {out}")
    if "--out" in argv:
        contact(done, out / "_props.png")
    missing = sorted(set(a for _, (_, p) in PICKS.items() for a in p) - {a for a, _ in done})
    if missing and not only:
        print("not cut:", missing)


if __name__ == "__main__":
    main(sys.argv[1:])
