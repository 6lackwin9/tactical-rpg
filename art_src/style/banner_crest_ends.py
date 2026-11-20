"""Crest-capped story banner (LOOK1-A6 "Story card banner: crests as the ends", user 2026-11-02; Design 2026-11-20).

Usage: python3 art_src/style/banner_crest_ends.py [out.png]   (default games/emberwake/assets/ui/banner_full.png)

`rpgkit_ui::banner` draws the picture in three slices: each end is END_SHARE (0.78) of the picture's height wide and
kept as is, the middle columns repeat to the title's length. So the picture is H tall: the plain bar
(`ui/banner_bar.png`, tiling as a pair with its mirror) runs across the middle with one whole tile exactly in the middle slice, and
crest B (`ui/logo_crest.png`, the eye) caps each end, 0.78 H wide, standing above and below the bar.
"""
import sys
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
UI = ROOT / "games/emberwake/assets/ui"
H = 243                   # the old banner_full height, so the card's banner keeps its size
END_SHARE = 0.78          # rpgkit_ui::banner::END_SHARE
BAR_H = 112               # the bar's height inside the picture


def crisp(im, w, h):
    """Box-filter downscale with premultiplied alpha, then a hard alpha edge (pixel art)."""
    a = im.getchannel("A").resize((w, h), Image.BOX)
    pm = Image.new("RGBA", im.size, (0, 0, 0, 0))
    pm.paste(im, (0, 0), im)
    rgb = pm.convert("RGB").resize((w, h), Image.BOX)
    out = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    rp, ap, op = rgb.load(), a.load(), out.load()
    for y in range(h):
        for x in range(w):
            al = ap[x, y]
            if al >= 110:
                k = 255 / al
                r, g, b = rp[x, y]
                op[x, y] = (min(255, int(r * k)), min(255, int(g * k)), min(255, int(b * k)), 255)
    return out


def main():
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else UI / "banner_full.png"
    end = round(H * END_SHARE)
    bar = Image.open(UI / "banner_bar.png").convert("RGBA")
    bar = bar.resize((round(bar.width * BAR_H / bar.height), BAR_H), Image.LANCZOS)
    pair = Image.new("RGBA", (bar.width * 2, BAR_H))          # the bar tiles only when every other copy is mirrored
    pair.paste(bar, (0, 0))
    pair.paste(bar.transpose(Image.FLIP_LEFT_RIGHT), (bar.width, 0))
    bar = pair
    W = 2 * end + bar.width                        # the middle slice holds exactly one bar tile
    img = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    y0 = (H - BAR_H) // 2
    x = end - bar.width                            # tiles start so one begins at the middle slice's first column
    while x < W:
        img.alpha_composite(bar, (x, y0)) if x >= 0 else img.alpha_composite(bar.crop((-x, 0, bar.width, BAR_H)), (0, y0))
        x += bar.width
    crest = Image.open(UI / "logo_crest.png").convert("RGBA")
    crest = crest.crop(crest.getbbox())
    s = min((end - 4) / crest.width, (H - 2) / crest.height)
    c = crisp(crest, round(crest.width * s), round(crest.height * s))
    cy = (H - c.height) // 2
    img.alpha_composite(c, ((end - c.width) // 2, cy))
    img.alpha_composite(c, (W - end + (end - c.width) // 2, cy))
    img.save(out)
    print(out, img.size, "end", end, "middle", bar.width)


if __name__ == "__main__":
    main()
