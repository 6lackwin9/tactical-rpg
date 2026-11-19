#!/usr/bin/env python3
# ART-13 / ART-14 (agent playtest 2026-11-18, inbox AI-20): Emberwake's copies of the engine's common_stage battle tiles,
# graded to the ground rule (palette.json "ground": mean brightness 24-40, spread at most 14) and the bible's palette
# (saturation at most 0.55; water near-black green-grey, not royal blue). The engine tiles stay as they are; these sit
# in the game layer at the same relative paths, which the game resolves first.
#   python art_src/style/stage_ground.py            (writes games/emberwake/assets/tilesets/common_stage/**)
#   python art_src/style/stage_ground.py --check    (prints mean and spread of each written tile)
import colorsys, json, pathlib, sys
from PIL import Image, ImageStat

ROOT = pathlib.Path(__file__).resolve().parents[2]
SRC = ROOT / "engine/assets/tilesets/common_stage"
DST = ROOT / "games/emberwake/assets/tilesets/common_stage"
RULE = json.loads((ROOT / "art_src/style/palette.json").read_text(encoding="utf-8"))["ground"]
# terrain: (target mean brightness, max saturation, tint rgb or None)
GROUND = {"floor": (34, 0.30, None), "stone": (36, 0.30, None), "bridge": (34, 0.50, None), "dirt": (33, 0.50, None),
          "rubble": (35, 0.30, None), "mud": (30, 0.50, None), "grass": (34, 0.45, None),
          "shallow_water": (37, 0.30, (34, 44, 40)), "deep_water": (24, 0.30, (24, 31, 30))}
SPREAD = RULE["spread_max"] - 2


def grade(im, mean, sat_max, tint):
    im = im.convert("RGBA")
    L = ImageStat.Stat(im.convert("L"))
    m0, s0 = L.mean[0], max(L.stddev[0], 1e-3)
    k = min(1.0, SPREAD / s0)
    px = im.load()
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = px[x, y]
            if a == 0:
                continue
            lum = 0.299 * r + 0.587 * g + 0.114 * b
            nl = max(4.0, mean + (lum - m0) * k)
            if tint:   # water: the tint's hue and saturation, this pixel's new brightness
                tl = 0.299 * tint[0] + 0.587 * tint[1] + 0.114 * tint[2]
                r2, g2, b2 = (c * nl / tl for c in tint)
            else:
                h, s, v = colorsys.rgb_to_hsv(r / 255, g / 255, b / 255)
                r1, g1, b1 = colorsys.hsv_to_rgb(h, min(s, sat_max), v)
                l1 = max(1e-3, 0.299 * r1 + 0.587 * g1 + 0.114 * b1) * 255
                r2, g2, b2 = (c * 255 * nl / l1 for c in (r1, g1, b1))
            px[x, y] = tuple(max(0, min(255, round(c))) for c in (r2, g2, b2)) + (a,)
    return im


def main():
    for terrain, (mean, sat, tint) in GROUND.items():
        for f in sorted((SRC / terrain).glob("v*.png")):
            out = DST / terrain / f.name
            if "--check" not in sys.argv:
                out.parent.mkdir(parents=True, exist_ok=True)
                grade(Image.open(f), mean, sat, tint).save(out)
            st = ImageStat.Stat(Image.open(out).convert("L"))
            print(f"{terrain}/{f.name}: mean {st.mean[0]:.0f}, spread {st.stddev[0]:.0f}")


if __name__ == "__main__":
    main()
