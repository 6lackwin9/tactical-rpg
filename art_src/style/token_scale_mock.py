"""AI-29 party token scale mock (ART-19, FB-354): the four options on a hub crop and a region crop, 1280x720 at 1:1.

Usage: python3 art_src/style/token_scale_mock.py <hub shot.png> <region shot.png> <out_dir>
(the shots: scenario story_battle_hub `hub`, job_fight_repair `kiln_loaded`, from develop).

The options, edited into the real shots (no game code changes; Art/UI cannot touch the crates):
  (C) as now: tokens are the 48 px frame drawn 1:1 (mapview::SPRITE 24 logical), about 40 px of figure;
  (A) SPRITE 24 to 32: the same 48 px frame drawn at 64 px, a 4:3 nearest scale (uneven pixels);
  (B) tall props (standing props over 64 px) drawn at 3/4 size through prop_fit, tokens as now;
  (D) tokens redrawn at 32 px and drawn at 2x (64 px, even pixels); stand-in: the 48 px frame reduced to 32 px with a
      box filter, then doubled. A real redraw would be cleaner than this reduction.
Each token is cut from the shot along its closed rig outline; what it covered is filled with the floor 32 px away
(the tile repeat), so a seam can show where the floor is not a repeat.
"""
import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
OUT_RGB = np.array([20, 17, 15])
CELL = 32
LAYERS = [ROOT / "games/emberwake/assets/props/sunk_ward", ROOT / "kits/rpg/assets/props/common"]
TALL = 64                     # B: standing props taller than this shrink
SHRINK = 0.75                 # B: 24/32, the same ratio as A the other way round
SKIP_B = {"vault_pillar", "iron_ladder", "portcullis", "wall_lantern", "fourth_kiln"}   # architecture, not furniture


def prop_png(art):
    for lay in LAYERS:
        f = lay / art / "v0.png"
        if f.exists():
            return Image.open(f).convert("RGBA")
    return None


def variants(art):
    """The art and its drawn variants (kiln_bunk_active, timber_bunk_blanket_mage, ...), same size as the base."""
    base = prop_png(art)
    out = [base] if base is not None else []
    for lay in LAYERS:
        for d in sorted(lay.glob(art + "_*")):
            f = d / "v0.png"
            if f.exists() and base is not None and Image.open(f).size == base.size:
                out.append(Image.open(f).convert("RGBA"))
    return out


def map_props(map_id):
    j = json.loads((ROOT / "games/emberwake/data/maps/maps.json").read_text())
    return [e for e in j["entries"] if e["id"] == map_id][0]["props"]


def prop_rect(p, png, ox, oy):
    """Natural size, bottom-centred on the footprint (PROPS1 4.3)."""
    cx = (p["x"] + p["w"] / 2) * CELL + ox
    by = (p["y"] + p["h"]) * CELL + oy
    return int(round(cx - png.width / 2)), int(by - png.height)


def calibrate(shot, p, guess, r=8):
    png = prop_png(p["art"])
    a = np.asarray(png).astype(int)
    m = a[..., 3] > 128
    best = None
    for dy in range(-r, r + 1):
        for dx in range(-r, r + 1):
            ox, oy = guess[0] + dx, guess[1] + dy
            x, y = prop_rect(p, png, ox, oy)
            s = shot[y:y + png.height, x:x + png.width]
            if s.shape[:2] != m.shape:
                continue
            e = np.abs(s - a[..., :3]).sum(2)[m].mean()
            if best is None or e < best[0]:
                best = (e, ox, oy)
    return best[1], best[2]


def token_mask(shot, feet, drawn):
    """The sprite in the 48 px box over its feet: pixels unlike the floor one tile (32 px) left, right and up, less the
    pixels that match a prop drawn there (`drawn`: an image of the props' own pixels, `None` where there is no prop)."""
    fx, fy = feet
    x0, y0 = fx - 24, fy - 48
    b = shot[y0:fy, x0:fx + 24]
    near = [shot[y0:fy, x0 - 32:fx - 8], shot[y0:fy, x0 + 32:fx + 56], shot[y0 - 32:fy - 32, x0:fx + 24]]
    diff = np.min([np.abs(b - n).sum(2) for n in near], axis=0) > 40
    pr, on = drawn
    prop = on[y0:fy, x0:fx + 24] & (np.abs(b - pr[y0:fy, x0:fx + 24]).sum(2) < 36)
    keep = diff & ~prop
    # only what joins the body: drop a marker or a label that reaches into the box
    seen = np.zeros_like(keep)
    stack = [(y, x) for y in range(18, 44) for x in range(18, 30) if keep[y, x]]
    while stack:
        y, x = stack.pop()
        if 0 <= y < 48 and 0 <= x < 48 and keep[y, x] and not seen[y, x]:
            seen[y, x] = True
            stack += [(y + dy, x + dx) for dy in (-1, 0, 1) for dx in (-1, 0, 1)]
    m = np.zeros(shot.shape[:2], bool)
    m[y0:fy, x0:fx + 24] = seen
    return m


def fill(img, mask, avoid):
    """Fill `mask` from the floor 32 px (one tile repeat) to either side, whichever source is clear of `avoid`."""
    ys, xs = np.where(mask)
    if not len(ys):
        return
    for k in (1, -1, 2, -2, 3, -3, 4, -4):
        sx = xs + k * CELL
        ok = (sx >= 0) & (sx < img.shape[1])
        if ok.all() and not avoid[ys, sx].any():
            img[ys, xs] = img[ys, sx]
            return
    for k in (1, -1, 2, -2):
        sy = ys + k * CELL
        if ((sy >= 0) & (sy < img.shape[0])).all() and not avoid[sy, xs].any():
            img[ys, xs] = img[sy, xs]
            return
    img[ys, xs] = img[ys, np.clip(xs + CELL, 0, img.shape[1] - 1)]


def paste(img, rgba, x, y):
    a = np.asarray(rgba).astype(int)
    h, w = a.shape[:2]
    x0, y0 = max(0, x), max(0, y)
    x1, y1 = min(img.shape[1], x + w), min(img.shape[0], y + h)
    sub = a[y0 - y:y1 - y, x0 - x:x1 - x]
    on = sub[..., 3] > 128
    reg = img[y0:y1, x0:x1]
    reg[on] = sub[..., :3][on]


def cut(img, mask, box):
    x0, y0, x1, y1 = box
    rgba = np.zeros((y1 - y0, x1 - x0, 4), np.uint8)
    rgba[..., :3] = img[y0:y1, x0:x1]
    rgba[..., 3] = mask[y0:y1, x0:x1] * 255
    return Image.fromarray(rgba, "RGBA")


def option(base, tokens, props, drawn, opt):
    img = base.copy()
    tmask = [token_mask(base, f, drawn) for f in tokens]
    alltok = np.zeros(base.shape[:2], bool)
    for m in tmask:
        alltok |= m
    occupied = alltok | drawn[1]
    sprites = [cut(base, m, (f[0] - 24, f[1] - 48, f[0] + 24, f[1])) for f, m in zip(tokens, tmask)]
    if opt == "B":
        for art, m, box, tall in props:
            if not tall:
                continue
            pr = cut(base, m & ~alltok, box)
            fill(img, m, occupied)
            w, h = round(pr.width * SHRINK), round(pr.height * SHRINK)
            small = pr.resize((w, h), Image.BOX)
            paste(img, small, box[0] + (pr.width - w) // 2, box[3] - h)
        for f, s in zip(tokens, sprites):
            paste(img, s, f[0] - 24, f[1] - 48)
    if opt in ("A", "D"):
        for m in tmask:
            fill(img, m, occupied)
        for f, s in zip(tokens, sprites):
            if opt == "A":
                big = s.resize((64, 64), Image.NEAREST)
            else:
                big = s.resize((32, 32), Image.BOX).resize((64, 64), Image.NEAREST)
            paste(img, big, f[0] - 32, f[1] - 64)
    return img


def placed_props(shot, map_id, ox, oy):
    """Every standing prop on screen with its best-matching variant: (art, mask, box, tall), and an image of the props'
    own pixels with where they are."""
    out = []
    pr = np.zeros_like(shot)
    on = np.zeros(shot.shape[:2], bool)
    for p in map_props(map_id):
        cands = variants(p["art"])
        if not cands or p.get("layer") != "sort":
            continue
        if p.get("flip"):
            cands = [c.transpose(Image.FLIP_LEFT_RIGHT) for c in cands]
        x, y = prop_rect(p, cands[0], ox, oy)
        w, h = cands[0].size
        x0, y0 = max(0, x), max(0, y)
        x1, y1 = min(shot.shape[1], x + w), min(shot.shape[0], y + h)
        if x1 <= x0 or y1 <= y0:
            continue
        s = shot[y0:y1, x0:x1]

        def err(c):
            a = np.asarray(c).astype(int)[y0 - y:y1 - y, x0 - x:x1 - x]
            al = a[..., 3] > 128
            return np.abs(s - a[..., :3]).sum(2)[al].mean() if al.any() else 1e9
        best = min(cands, key=err)
        a = np.asarray(best).astype(int)[y0 - y:y1 - y, x0 - x:x1 - x]
        al = a[..., 3] > 128
        m = np.zeros(shot.shape[:2], bool)
        m[y0:y1, x0:x1] = al
        pr[y0:y1, x0:x1][al] = a[..., :3][al]
        on |= m
        tall = p["art"] not in SKIP_B and h > TALL and p["w"] < 4      # furniture: buildings (4+ cells) stay
        out.append((p["art"], m, (x0, y0, x1, y1), tall))
    return out, (pr, on)


SCENES = {
    # map id, a calibration prop index and guess, the token feet, the crop
    "hub": ("sunk_ward_undercroft", "brazier", (32, -84),
            [(528, 301), (239, 494), (493, 590), (557, 590), (717, 590)], (200, 190, 1000, 660), (360, 380, 760, 600)),
    "region": ("sunk_ward_fourth_kiln", "crate_stack", (-79, -18), [(640, 559)], (0, 300, 900, 660), (300, 400, 700, 620)),
}


def scene(path, key):
    map_id, anchor, guess, tokens, crop, zoom = SCENES[key]
    shot = np.asarray(Image.open(path).convert("RGB")).astype(int)
    p = [q for q in map_props(map_id) if q["art"] == anchor][0]
    ox, oy = calibrate(shot, p, guess)
    props, drawn = placed_props(shot, map_id, ox, oy)
    tiles = []
    for opt, label in (("C", "C. As now: 48 px frame at 1:1"), ("A", "A. SPRITE 24 to 32: 4:3 scale"),
                       ("B", "B. Tall props at 3/4 (prop_fit)"), ("D", "D. Tokens redrawn at 32 px, 2x")):
        img = option(shot, tokens, props, drawn, opt)
        full = Image.fromarray(img.clip(0, 255).astype(np.uint8))
        tiles.append((label, full.crop(crop), opt, full.crop(zoom)))
    return tiles


def sheet(tiles, title):
    w, h = tiles[0][1].size
    out = Image.new("RGB", (w * 2 + 12, (h + 22) * 2 + 30), (14, 16, 22))
    d = ImageDraw.Draw(out)
    d.text((6, 6), title, fill=(214, 170, 72))
    for i, (label, t, _, _) in enumerate(tiles):
        x, y = (i % 2) * (w + 12), 30 + (i // 2) * (h + 22)
        d.text((x + 4, y + 4), label, fill=(214, 170, 72))
        out.paste(t, (x, y + 20))
    return out


def main():
    hub, region, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    out.mkdir(parents=True, exist_ok=True)
    for key, path in (("hub", hub), ("region", region)):
        tiles = scene(path, key)
        for label, t, opt, z in tiles:
            t.save(out / f"{key}_{opt}.png")
            z.resize((z.width * 2, z.height * 2), Image.NEAREST).save(out / f"{key}_{opt}_zoom2x.png")
        sheet(tiles, f"Party token scale (ART-19), {key} crop at 1:1").save(out / f"{key}_sheet.png")


if __name__ == "__main__":
    main()
