#!/usr/bin/env python3
"""LOOK1-R1 recipe: the title's picked art from art_src/ into the game's assets (scale only, no palette pass; the picks:
docs/decisions/workflow.md). The ornate frame has its own: `gen_art.py ui_frames`.

  python tools/title_art.py [--src ART_SRC_DIR]

backgrounds/menu/title.png  the clean backdrop A, centre cover crop to the size of main_menu.png: nearest x4, then a box
                            down, so pixels keep hard edges without uneven widths
ui/logo_title.png           title_logo_final.png, an unscaled copy (how it was composed: the .json beside it)
ui/logo_crest.png           crest B, the near-black ground flood-keyed from the corners (the mark's outline is blacker
                            than the ground, so the flood stops there), trimmed to a transparent square
ui/banner_full.png          card banner take 1, magenta key, finials kept
ui/banner_bar.png           card banner take 2's plain middle, BAR_BOX: no finials, fully opaque, its ends cut on the
                            arches' axes (the columns of best local mirror symmetry), so mirrored copies join
"""
import argparse
import os
import shutil

import numpy as np
from PIL import Image

from gen_art import WORKSPACE, key_magenta, largest_piece

ASSETS = os.path.join(WORKSPACE, "games", "emberwake", "assets")
BAR_BOX = (306, 308, 1022, 460)


def asset(rel: str) -> str:
    return os.path.join(ASSETS, *rel.split("/"))


def backdrop(src: str) -> None:
    w, h = Image.open(asset("backgrounds/menu/main_menu.png")).size
    b = Image.open(os.path.join(src, "title_backdrop_clean_gen1793615014_1.png")).convert("RGB")
    k = max(w / b.width, h / b.height)
    sw, sh = w / k, h / k
    x0, y0 = (b.width - sw) / 2, (b.height - sh) / 2
    big = b.resize((b.width * 4, b.height * 4), Image.NEAREST)
    big.resize((w, h), Image.BOX, box=(x0 * 4, y0 * 4, (x0 + sw) * 4, (y0 + sh) * 4)).save(asset("backgrounds/menu/title.png"), optimize=True)


def crest(src: str) -> None:
    c = np.array(Image.open(os.path.join(src, "logo_crest_gen1793612836_2.png")).convert("RGBA"))
    ground = np.median(c[:40, :40, :3].reshape(-1, 3), axis=0)
    near = np.abs(c[..., :3].astype(int) - ground).max(axis=2) <= 7
    h, w = near.shape
    seen = np.zeros_like(near)
    stack = [(0, 0), (0, w - 1), (h - 1, 0), (h - 1, w - 1)]
    for p in stack:
        seen[p] = True
    while stack:
        y, x = stack.pop()
        for ny, nx in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
            if 0 <= ny < h and 0 <= nx < w and near[ny, nx] and not seen[ny, nx]:
                seen[ny, nx] = True
                stack.append((ny, nx))
    c[seen] = 0
    img = Image.fromarray(c, "RGBA")
    box = img.getbbox()
    bw, bh = box[2] - box[0], box[3] - box[1]
    side = max(bw, bh)
    square = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    square.paste(img.crop(box), ((side - bw) // 2, (side - bh) // 2))
    square.save(asset("ui/logo_crest.png"), optimize=True)


def banners(src: str) -> None:
    full = largest_piece(key_magenta(Image.open(os.path.join(src, "card_banner_gen1793616575_1.png"))))
    full.crop(full.getbbox()).save(asset("ui/banner_full.png"), optimize=True)
    take = largest_piece(key_magenta(Image.open(os.path.join(src, "card_banner_gen1793616575_2.png"))))
    bar = take.crop(BAR_BOX)
    assert np.array(bar)[..., 3].min() == 255, "the bar has see-through pixels"
    bar.save(asset("ui/banner_bar.png"), optimize=True)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", default=os.path.join(WORKSPACE, "art_src"))
    src = ap.parse_args().src
    backdrop(src)
    shutil.copyfile(os.path.join(src, "title_logo_final.png"), asset("ui/logo_title.png"))
    crest(src)
    banners(src)


if __name__ == "__main__":
    main()
