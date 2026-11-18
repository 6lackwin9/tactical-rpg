"""Place the AI-20 Gemini web-queue battle strips and the hearth story card (2026-11-18).

Usage: python3 art_src/prompts/place_backdrops.py <zone>=<png> ... hearth=<png>
Strips: find the painted band inside Gemini's letterbox, cut a 384:52 window (more sky dropped than ground),
box-downscale to 384x52 RGB at backgrounds/battle/<zone>.png. Hearth: centre cut to 1120:472, RGBA.
Also writes generic kit-library copies (kits/rpg/library/tallow_iron/backdrops|cgs).
"""
import json
import sys
from pathlib import Path
from PIL import Image, ImageStat

ROOT = Path(__file__).resolve().parents[2]
GAME = ROOT / "games/emberwake/assets/backgrounds"
LIB = ROOT / "kits/rpg/library/tallow_iron"
GENERIC = {"alley": ("slum_alley_houses", ["alley", "slum", "houses", "night", "street"]),
           "rooftops": ("night_rooftops", ["rooftops", "chimneys", "smoke", "night", "town"]),
           "canal": ("canal_truss_bridge", ["canal", "bridge", "water", "industrial", "night"]),
           "foundry": ("foundry_wall_furnaces", ["foundry", "furnace", "chains", "machinery", "interior"]),
           "tallow_vats": ("green_vat_gantry", ["vats", "pipes", "gantry", "industrial", "interior"]),
           "undercroft": ("crypt_arcade", ["crypt", "arches", "pillars", "candles", "underground"]),
           "watchpost": ("gatehouse_wall", ["fortress", "gatehouse", "wall", "braziers", "night"]),
           "hearth": ("vaulted_hearth_cellar", ["cellar", "vaults", "hearth", "fire", "interior"])}


BIAS = {"alley": 0.3, "watchpost": 0.12}  # keep roofs and tower tops


def tallow(im: Image.Image) -> Image.Image:
    """Gemini drew the vats toxic green; tallow is a dull yellow-white render, so swing green hues to tallow."""
    import colorsys
    px = im.load()
    for y in range(im.height):
        for x in range(im.width):
            r, g, b = (c / 255 for c in px[x, y])
            h, s, v = colorsys.rgb_to_hsv(r, g, b)
            if 0.15 < h < 0.42 and s > 0.2:
                r2, g2, b2 = colorsys.hsv_to_rgb(0.12, s * 0.6, v)
                px[x, y] = (int(r2 * 255), int(g2 * 255), int(b2 * 255))
    return im


def band(im: Image.Image) -> tuple[int, int]:
    g = im.convert("L")
    rows = [ImageStat.Stat(g.crop((0, y, g.width, y + 1))).stddev[0] for y in range(g.height)]
    live = [y for y, s in enumerate(rows) if s > 9]
    return live[0], live[-1] + 1


def meta(name: str, kind: str, size, tags, out: Path):
    out.write_text(json.dumps({"id": name, "kind": kind, "style": "tallow_iron", "size": list(size), "palette": "p3",
                               "tags": tags, "status": "placeholder", "source": "gemini_web", "request": None,
                               "license": "generated for this project (Gemini web), owner-held"}, indent=1) + "\n")


def main():
    for arg in sys.argv[1:]:
        zone, src = arg.split("=", 1)
        im = Image.open(src).convert("RGB")
        gname, tags = GENERIC[zone]
        if zone == "hearth":
            h = round(im.width * 472 / 1120)
            top = (im.height - h) // 2
            out = im.crop((0, top, im.width, top + h)).resize((1120, 472), Image.LANCZOS).convert("RGBA")
            out.save(GAME / "story/hearth.png")
            (LIB / "cgs").mkdir(parents=True, exist_ok=True)
            out.save(LIB / "cgs" / f"{gname}.png")
            meta(gname, "cgs", out.size, tags, LIB / "cgs" / f"{gname}.meta.json")
            continue
        y0, y1 = band(im)
        w = im.width
        h = round(w * 52 / 384)
        excess = (y1 - y0) - h
        top = y0 + int(excess * BIAS.get(zone, 0.55)) if excess >= 0 else y1 - h
        out = im.crop((0, top, w, top + h)).resize((384, 52), Image.BOX)
        if zone == "tallow_vats":
            out = tallow(out)
        out.save(GAME / "battle" / f"{zone}.png")
        (LIB / "backdrops").mkdir(parents=True, exist_ok=True)
        out.save(LIB / "backdrops" / f"{gname}.png")
        meta(gname, "backdrops", out.size, tags, LIB / "backdrops" / f"{gname}.meta.json")
        print(zone, (y0, y1), top, h)


if __name__ == "__main__":
    main()
