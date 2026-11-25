"""AI-36 (3): the six kit icons Build placed as copies (PR7-40, FB-356), drawn in place.

Usage: python3 art_src/style/ai36_icons.py [out_root]   (default: the repo)
  - statuses/empower.png, statuses/guard.png (64 px): the kit status tile (dark slate, 2 px rim, buff green as the
    shipped buffs), empower two rising chevrons (the mirror of weaken), guard a round buckler with a boss (fortify is
    the kite shield);
  - ui/badges/crown.png, crit_star.png, split_shield.png (32 px) and heart.png (64 px): bare glyphs on transparency
    with a 1 px dark outline, as the battle mocks draw them on the plate, KO tag, break tag and card rows.
Hard pixels only (no anti-aliasing); colours from the locked palette (gold, amber, iron, parchment, blood).
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[2]
TILE, RIM = (30, 35, 45, 255), (74, 160, 90, 255)        # the shipped buff tile (fortify.png) colours
GOLD, GOLD_D, GOLD_L = (251, 191, 36, 255), (146, 64, 14, 255), (253, 230, 138, 255)
IRON, IRON_L, IRON_D = (156, 163, 175, 255), (209, 213, 219, 255), (75, 85, 99, 255)
BLOOD, BLOOD_L, BLOOD_D = (185, 38, 38, 255), (225, 90, 80, 255), (110, 20, 20, 255)
OUT = (12, 10, 10, 255)


def sample_tile(src):
    """The shipped buff tile's fill and rim, so the new icons sit with the set."""
    im = Image.open(src).convert("RGBA")
    return im.getpixel((32, 10)), im.getpixel((2, 32))[:3] + (255,)


def tile():
    im = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle((2, 2, 61, 61), radius=4, fill=TILE, outline=RIM, width=1)
    d.rounded_rectangle((3, 3, 60, 60), radius=3, outline=TILE[:3] + (255,), width=1)
    return im, d


def outline(glyph, px=1):
    a = glyph.getchannel("A").point(lambda v: 255 if v else 0)
    grown = a.filter(ImageFilter.MaxFilter(2 * px + 1))
    base = Image.new("RGBA", glyph.size, OUT)
    base.putalpha(grown)
    base.alpha_composite(glyph)
    return base


def empower():
    im, d = tile()
    for k, col in ((0, GOLD), (14, GOLD_D)):
        y = 18 + k
        d.polygon([(12, y + 14), (32, y), (52, y + 14), (52, y + 22), (32, y + 8), (12, y + 22)], fill=col)
    # the light edge on the top chevron
    d.line([(14, 32), (32, 19), (50, 32)], fill=GOLD_L, width=1)
    return im


def guard():
    im, d = tile()
    d.ellipse((12, 14, 52, 54), fill=IRON_D)
    d.ellipse((14, 16, 50, 52), fill=IRON)
    d.ellipse((18, 20, 46, 48), outline=IRON_D, width=1)
    d.ellipse((27, 29, 37, 39), fill=IRON_L, outline=IRON_D)
    d.arc((14, 16, 50, 52), 200, 290, fill=IRON_L, width=2)
    return im


def glyph_canvas(s):
    g = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    return g, ImageDraw.Draw(g)


def crown():
    g, d = glyph_canvas(32)
    d.polygon([(4, 24), (4, 10), (10, 17), (16, 5), (22, 17), (28, 10), (28, 24)], fill=GOLD)
    d.rectangle((4, 24, 28, 27), fill=GOLD_D)
    d.line([(5, 23), (27, 23)], fill=GOLD_L)
    for x in (9, 16, 23):
        d.rectangle((x - 1, 25, x, 26), fill=BLOOD)
    return outline(g)


def crit_star():
    g, d = glyph_canvas(32)
    import math
    pts = []
    for i in range(10):
        r = 13 if i % 2 == 0 else 5.5
        a = -math.pi / 2 + i * math.pi / 5
        pts.append((16 + r * math.cos(a), 16.5 + r * math.sin(a)))
    d.polygon(pts, fill=GOLD)
    d.polygon([pts[0], pts[1], (16, 16.5), pts[9]], fill=GOLD_L)
    return outline(g)


def split_shield():
    g, d = glyph_canvas(32)
    left = [(4, 4), (14, 4), (12, 11), (15, 16), (12, 22), (14, 29), (4, 20)]
    right = [(18, 4), (28, 4), (28, 20), (18, 29), (16, 22), (19, 16), (16, 11)]
    d.polygon([(x - 1, y + 1) for x, y in left], fill=IRON)
    d.polygon([(x + 1, y - 1) for x, y in right], fill=IRON)
    d.line([(4, 5), (12, 5)], fill=IRON_L)
    d.line([(20, 3), (28, 3)], fill=IRON_L)
    d.line([(3, 20), (12, 28)], fill=IRON_D)
    return outline(g)


def heart():
    g, d = glyph_canvas(64)
    d.ellipse((6, 10, 34, 38), fill=BLOOD)
    d.ellipse((30, 10, 58, 38), fill=BLOOD)
    d.polygon([(7, 28), (57, 28), (32, 56)], fill=BLOOD)
    d.polygon([(32, 56), (57, 28), (58, 24), (46, 42)], fill=BLOOD_D)
    d.ellipse((13, 16, 23, 26), fill=BLOOD_L)
    d.rectangle((16, 18, 19, 21), fill=(240, 160, 140, 255))
    return outline(g, 2)


def main():
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT
    global TILE, RIM
    TILE, RIM = sample_tile(ROOT / "kits/rpg/assets/statuses/fortify.png")
    out = {"kits/rpg/assets/statuses/empower.png": empower(), "kits/rpg/assets/statuses/guard.png": guard(),
           "kits/rpg/assets/ui/badges/crown.png": crown(), "kits/rpg/assets/ui/badges/crit_star.png": crit_star(),
           "kits/rpg/assets/ui/badges/split_shield.png": split_shield(), "kits/rpg/assets/ui/badges/heart.png": heart()}
    for rel, im in out.items():
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        im.save(p)


if __name__ == "__main__":
    main()
