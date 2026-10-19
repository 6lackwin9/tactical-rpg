#!/usr/bin/env python3
# Writes the pixel-art unit token and portrait files (SPRITE1-A) from the sprite rig's idle frame, one folder per unit
# (unit/<key>/token.png and portrait.png, ASSETORG1).
#   python3 tools/sprites/export_units.py [--out DIR]   (default: games/emberwake/assets/art/unit)
# Token = the 48 px figure at 2x in the 104 px inner square; portrait = a 24x24 head-and-shoulders crop at 10x (240 px).
# The party classes and the named party characters that use them are listed in SPRITE_KEYS; tools/gen_art.py skips them.
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from PIL import Image  # noqa: E402

import classes  # noqa: E402

SPRITE_KEYS = {
    "warrior": ["warrior", "mara_voss", "bryn_hale"],
    "ranger": ["ranger", "tev_quill", "sella_marsh"],
    "brawler": ["brawler", "dagny_coal"],
    "priest": ["priest", "alda_wren"],
    "rogue": ["rogue", "kit_vane"],
    "mage": ["mage", "oren_dusk"],
}


def token(frame: Image.Image) -> Image.Image:
    out = Image.new("RGBA", (104, 104), (0, 0, 0, 0))
    out.alpha_composite(frame.resize((96, 96), Image.NEAREST), (4, 8))
    return out


def portrait(frame: Image.Image) -> Image.Image:
    box = frame.getbbox()
    x0 = max(0, (box[0] + box[2]) // 2 - 12)
    return frame.crop((x0, box[1] - 1, x0 + 24, box[1] + 23)).resize((240, 240), Image.NEAREST)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="games/emberwake/assets/art/unit")
    args = ap.parse_args()
    for cls, keys in SPRITE_KEYS.items():
        frame = classes.clips(cls)[0]["idle"][0]
        for key in keys:
            folder = os.path.join(args.out, key)   # ASSETORG1: unit/<key>/token.png, portrait.png
            os.makedirs(folder, exist_ok=True)
            token(frame).save(os.path.join(folder, "token.png"))
            portrait(frame).save(os.path.join(folder, "portrait.png"))
            print("wrote", key)


if __name__ == "__main__":
    main()
