"""AI-40: the poise badges for the boss bar, intent cells and plates (PR7-54, PR7-59 (3)), generic kit icons.

Usage: python3 art_src/style/ai40_poise_badges.py [out_root]   (default: the repo)
  - kits/rpg/assets/ui/badges/poise.png (32 px): a heater shield in the code's poise blue (150, 190, 235), lit top
    left, shaded bottom right, a 1 px dark outline;
  - kits/rpg/assets/ui/badges/broken_poise.png (32 px): the same shield split by a jagged crack, the halves pushed
    apart (the gap stays open at 7 px) with pale sparks in the break.
Bare glyphs on transparency like the other badges (ai36_icons.py): the code draws them in 5 x 6 to 7 x 7 px cells, too
small for a slate tile, so the outline does the tile's job. Hard pixels only.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import ai36_icons as A  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
P, P_L, P_D, P_DD = (150, 190, 235, 255), (214, 232, 250, 255), (92, 128, 178, 255), (52, 74, 112, 255)
CRACK = (246, 240, 214, 255)
SHIELD = [(5, 4), (27, 4), (27, 16), (16, 29), (5, 16)]


def shield(d, dx=0, dy=0, pts=SHIELD):
    pts = [(x + dx, y + dy) for x, y in pts]
    d.polygon(pts, fill=P)
    return pts


def poise():
    g, d = A.glyph_canvas(32)
    shield(d)
    d.polygon([(16, 4), (27, 4), (27, 16), (16, 29)], fill=P_D)           # the shaded right half
    d.line([(6, 5), (15, 5)], fill=P_L)                                     # lit top edge
    d.line([(6, 5), (6, 15)], fill=P_L)
    d.line([(16, 8), (16, 25)], fill=P_DD)                                  # the central ridge
    d.rectangle((14, 11, 18, 14), fill=P_L)                                 # the boss
    d.rectangle((15, 12, 17, 13), fill=P_DD)
    return A.outline(g)


def broken_poise():
    g, d = A.glyph_canvas(32)
    crack = [(16, 4), (13, 10), (17, 15), (12, 21), (16, 29)]
    left = [(5, 4)] + crack + [(5, 16)]
    right = [(27, 4), (27, 16), (16, 29)] + list(reversed(crack[:-1]))
    lg, ld = A.glyph_canvas(32)
    ld.polygon(left, fill=P)
    ld.line([(6, 5), (6, 15)], fill=P_L)
    ld.line([(6, 5), (14, 5)], fill=P_L)
    rg, rd = A.glyph_canvas(32)
    rd.polygon(right, fill=P_D)
    # the halves pushed apart (a gap wider than the two outlines, so it stays open at 7 px) and tilted down
    g.alpha_composite(A.outline(lg), (-4, 1))
    g.alpha_composite(A.outline(rg), (4, 2))
    d = ImageDraw.Draw(g)
    for (x0, y0), (x1, y1) in zip(crack, crack[1:]):                        # pale sparks along the break
        d.point(((x0 + x1) // 2, (y0 + y1) // 2 + 1), fill=CRACK)
    return g


def main():
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT
    for rel, im in {"kits/rpg/assets/ui/badges/poise.png": poise(),
                    "kits/rpg/assets/ui/badges/broken_poise.png": broken_poise()}.items():
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        im.save(p)


if __name__ == "__main__":
    main()
