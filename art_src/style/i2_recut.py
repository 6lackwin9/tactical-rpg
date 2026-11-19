#!/usr/bin/env python3
# ART-12 (agent playtest 2026-11-18, inbox AI-20): moves the I1 skill icons (a painted object on a full red, navy or
# green tile) onto the I2 slate tile (STYLE_BIBLE 4, palette.json "icon_tile"). The old tile is keyed out by its colour
# (sampled from the icon's border ring), the object kept, and the shared slate tile drawn under it in code.
#   python art_src/style/i2_recut.py [--out DIR] [names...]     (default: every 64 px skill icon whose corner is opaque)
import colorsys, json, pathlib, sys
from PIL import Image, ImageDraw, ImageFilter

ROOT = pathlib.Path(__file__).resolve().parents[2]
SKILLS = ROOT / "games/emberwake/assets/skills"
TILE = json.loads((ROOT / "art_src/style/palette.json").read_text(encoding="utf-8"))["icon_tile"]


def hexrgb(h):
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))


def slate_tile(px=64):
    s = 4
    big = Image.new("RGBA", (px * s, px * s), (0, 0, 0, 0))
    d = ImageDraw.Draw(big)
    r = TILE["corner_radius_px"] * s
    d.rounded_rectangle([s, s, px * s - s - 1, px * s - s - 1], radius=r, fill=hexrgb(TILE["rim"]) + (255,))
    d.rounded_rectangle([3 * s, 3 * s, px * s - 3 * s - 1, px * s - 3 * s - 1], radius=r - 2 * s, fill=hexrgb(TILE["fill"]) + (255,))
    top = Image.new("RGBA", big.size, (0, 0, 0, 0))   # the lighter top half of the fill
    ImageDraw.Draw(top).rounded_rectangle([3 * s, 3 * s, px * s - 3 * s - 1, px * s // 2], radius=r - 2 * s,
                                          fill=hexrgb(TILE["fill_light"]) + (255,))
    mask = Image.new("L", big.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle([3 * s, 3 * s, px * s - 3 * s - 1, px * s - 3 * s - 1], radius=r - 2 * s, fill=255)
    top.putalpha(Image.composite(top.getchannel("A"), Image.new("L", big.size, 0), mask))
    big.alpha_composite(top)
    return big.resize((px, px), Image.LANCZOS)


def hsv(p):
    return colorsys.rgb_to_hsv(p[0] / 255, p[1] / 255, p[2] / 255)


def glyph(im):
    im = im.convert("RGBA")
    W, H = im.size
    px = im.load()
    ring = [px[x, y] for x in range(W) for y in range(H) if min(x, y, W - 1 - x, H - 1 - y) in (3, 4, 5)]
    hs = sorted(hsv(p)[0] for p in ring)
    h0 = hs[len(hs) // 2]
    vs = sorted(hsv(p)[2] for p in ring)
    v_hi = vs[int(len(vs) * 0.9)]
    out = Image.new("RGBA", im.size, (0, 0, 0, 0))
    o = out.load()
    for y in range(H):
        for x in range(W):
            r, g, b, a = px[x, y]
            if min(x, y, W - 1 - x, H - 1 - y) < 5:
                continue
            h, s, v = hsv((r, g, b))
            dh = min(abs(h - h0), 1 - abs(h - h0))
            bg = (dh < 0.07 and v <= v_hi * 1.15) or v < 0.10 or (s < 0.25 and v < 0.22)
            if not bg:
                o[x, y] = (r, g, b, 255)
    # drop specks: keep pixels with at least three solid neighbours
    clean = out.copy(); c = clean.load()
    for y in range(1, H - 1):
        for x in range(1, W - 1):
            if o[x, y][3] and sum(o[x + dx, y + dy][3] > 0 for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1))) < 3:
                c[x, y] = (0, 0, 0, 0)
    # a dark 1 px outline so the object reads on slate
    a = clean.getchannel("A")
    ring_ = a.filter(ImageFilter.MaxFilter(3))
    outline = Image.new("RGBA", im.size, (14, 12, 14, 255)); outline.putalpha(ring_)
    res = Image.new("RGBA", im.size, (0, 0, 0, 0)); res.alpha_composite(outline); res.alpha_composite(clean)
    return res


def main():
    args = sys.argv[1:]
    out_dir = SKILLS
    if "--out" in args:
        i = args.index("--out"); out_dir = pathlib.Path(args[i + 1]); del args[i:i + 2]
    names = args or [f.stem for f in sorted(SKILLS.glob("*.png")) if Image.open(f).convert("RGBA").getpixel((1, 1))[3] == 255]
    tile = slate_tile()
    for n in names:
        g = glyph(Image.open(SKILLS / f"{n}.png"))
        res = tile.copy(); res.alpha_composite(g)
        out_dir.mkdir(parents=True, exist_ok=True)
        res.save(out_dir / f"{n}.png")
    print(len(names), "icons")


if __name__ == "__main__":
    main()
