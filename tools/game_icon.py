#!/usr/bin/env python3
"""ICON1 recipe: the .exe icon of the game from its crest (Pillow only; the window's own icon the engine reduces itself).

  python tools/game_icon.py

games/emberwake/crates/emberwake_gui/res/emberwake.ico  16, 24, 32, 48, 64 and 256 px from assets/ui/logo_crest.png,
each box-averaged straight from the master, then an unsharp mask.

The whole crest stays readable down to 32 px. At 24 px it is cropped to its arch, and at 16 px to the eye alone, which
is what still reads there (the whole crest at 16 px is a brown smudge). Each row of SIZES is size, crop box, unsharp.
"""
import os

from PIL import Image, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CREST = os.path.join(ROOT, "games/emberwake/assets/ui/logo_crest.png")
OUT = os.path.join(ROOT, "games/emberwake/crates/emberwake_gui/res/emberwake.ico")

SIZES = [
    (16, (205, 205, 755, 755), (0.7, 150, 0)),
    (24, (100, 100, 860, 860), (1, 120, 0)),
    (32, None, (1, 120, 0)),
    (48, None, (1, 120, 0)),
    (64, None, (1, 100, 0)),
    (256, None, (1, 50, 0)),
]


def frame(crest, size, box, unsharp):
    # Averaged premultiplied ("RGBa"), so the see-through ground adds no dark fringe.
    small = crest.convert("RGBa").resize((size, size), Image.BOX, box=box)
    return small.filter(ImageFilter.UnsharpMask(*unsharp)).convert("RGBA")


def main():
    crest = Image.open(CREST).convert("RGBA")
    frames = [frame(crest, *row) for row in SIZES]
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    # Saved from the largest; append_images hands Pillow each exact frame instead of its own scaling.
    frames[-1].save(OUT, format="ICO", sizes=[(f.width, f.height) for f in frames], append_images=frames[:-1])
    print(f"wrote {OUT}: {os.path.getsize(OUT)} bytes")


if __name__ == "__main__":
    main()
