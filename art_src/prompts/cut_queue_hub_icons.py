#!/usr/bin/env python3
"""Cut the 2026-11-19 image queue run (AI-25, AI-31 amended, AI-32, AI-39) into runtime art.

Usage: python3 -I art_src/prompts/cut_queue_hub_icons.py <sheets_dir> <repo_root> [<contact_out.png>]

Each sheet is a grid of props or icons on flat #FF00FF magenta (the 2 x 2 sheets also carry black gutters, avoided by
an inset). Per cell: key out the magenta and its fringe, take the bounding box of what is left, drop specks, fit it in
the target box (keeping its aspect, bottom aligned for props), downscale with a box filter and threshold the alpha so
edges stay crisp. Passive icons go on the shared I2 slate tile (`i2_recut.slate_tile`, 64 px).
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFilter

ROOT_HINT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT_HINT / "art_src/style"))
import i2_recut  # noqa: E402

FOUNDERS = ("kaelen", "vesper", "maren", "hollis", "dunstan", "elyse")
CLASSES = ("warrior", "ranger", "rogue", "priest", "mage", "brawler")
PASSIVES = ("weapon_drill", "armour_drill", "might", "intellect", "perception", "agility", "poise_up", "resist",
            "ap_gain", "crit_up", "heal_up", "status_potency", "plan_sight", "immovable", "cover_ally", "last_stand")
G = "games/emberwake/assets/"
P = G + "props/sunk_ward/"


def key(im):
    """RGBA with the magenta background and its anti-aliased fringe made transparent."""
    im = im.convert("RGBA")
    px = im.load()
    W, H = im.size
    for y in range(H):
        for x in range(W):
            r, g, b, _ = px[x, y]
            if (min(r, b) - g > 55 and abs(r - b) < 110) or (b > g + 20 and r > g + 40):  # magenta and pink fringe
                px[x, y] = (0, 0, 0, 0)
    return im


def bbox(cell, min_px=40, th=2):
    """Bounding box of the opaque pixels, ignoring rows/columns with fewer than a few pixels (specks)."""
    a = cell.getchannel("A").point(lambda v: 255 if v else 0)
    W, H = a.size
    d = a.load()
    cols = [sum(1 for y in range(0, H, 2) if d[x, y]) for x in range(W)]
    rows = [sum(1 for x in range(0, W, 2) if d[x, y]) for y in range(H)]
    xs = [i for i, c in enumerate(cols) if c >= th]
    ys = [i for i, c in enumerate(rows) if c >= th]
    if len(xs) < min_px // 4 or not ys:
        return None
    return xs[0], ys[0], xs[-1] + 1, ys[-1] + 1


def shrink(obj, box_w, box_h):
    """Fit inside box_w x box_h keeping aspect: box-filter downscale with premultiplied alpha, then threshold."""
    w, h = obj.size
    s = min(box_w / w, box_h / h)
    nw, nh = max(1, round(w * s)), max(1, round(h * s))
    pm = Image.new("RGBA", obj.size, (0, 0, 0, 0))
    pm.paste(obj, (0, 0), obj)
    rgb = pm.convert("RGB").resize((nw, nh), Image.BOX)
    a = obj.getchannel("A").resize((nw, nh), Image.BOX)
    # un-premultiply: the box filter averaged transparent black in, so lift colour by 255/alpha
    rp, ap = rgb.load(), a.load()
    out = Image.new("RGBA", (nw, nh), (0, 0, 0, 0))
    op = out.load()
    for y in range(nh):
        for x in range(nw):
            al = ap[x, y]
            if al >= 110:
                r, g, b = rp[x, y]
                k = 255 / max(al, 1)
                op[x, y] = (min(255, int(r * k)), min(255, int(g * k)), min(255, int(b * k)), 255)
    return out


def lift_roll(sheet, box):
    """A bedroll lifted off the crates behind it: keep its pale canvas, close over its bands and outline, drop magenta."""
    im = sheet.convert("RGB").crop(box)
    px = im.load()
    W, H = im.size
    mask = Image.new("L", (W, H), 0)
    mp = mask.load()
    for y in range(H):
        for x in range(W):
            r, g, b = px[x, y]
            if r > 120 and g > 105 and b > 75 and min(r, b) - g <= 55:
                mp[x, y] = 255
    mask = mask.filter(ImageFilter.MaxFilter(9)).filter(ImageFilter.MinFilter(7)).filter(ImageFilter.MaxFilter(3))
    mask = mask.filter(ImageFilter.MinFilter(5)).filter(ImageFilter.MaxFilter(5))  # opening: drop specks
    for y in range(H):
        for x in range(W):
            r, g, b = px[x, y]
            if min(r, b) - g > 55:
                mp[x, y] = 0
    out = im.convert("RGBA")
    out.putalpha(mask)
    return out.crop(out.getbbox())


def cells(sheet, cols, rows, inset=0, row_edges=None, th=2):
    W, H = sheet.size
    ys = row_edges or [round(H * i / rows) for i in range(rows + 1)]
    out = []
    for r in range(rows):
        for c in range(cols):
            x0, x1 = round(W * c / cols) + inset, round(W * (c + 1) / cols) - inset
            y0, y1 = ys[r] + inset, ys[r + 1] - inset
            cell = sheet.crop((x0, y0, x1, y1))
            bb = bbox(cell, th=th)
            out.append(cell.crop(bb) if bb else None)
    return out


def place(obj, w, h, bottom=True):
    """The fitted object on a w x h canvas (props: bottom centred; icons: centred)."""
    if (w, h) == obj.size:
        return obj
    can = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    x = (w - obj.width) // 2
    y = h - obj.height if bottom else (h - obj.height) // 2
    can.paste(obj, (x, y), obj)
    return can


def tight(obj, w, h):
    """Fit in w x h and crop to the object (props keep their own size, like the existing cuts)."""
    return shrink(obj, w, h)


def run(src, root):
    S = lambda n: key(Image.open(src / n))  # noqa: E731
    jobs = []  # (repo path, image, note)
    # AI-39 kit passives: sheet 2 (cleaner silhouettes), 4 x 4, on the slate tile
    tile = i2_recut.slate_tile(64)
    for pid, c in zip(PASSIVES, cells(S("icons_kit_passives_1_2.png"), 4, 4, inset=6)):
        ic = tile.copy()
        ic.alpha_composite(place(shrink(c, 48, 48), 64, 64, bottom=False))
        jobs.append((f"kits/rpg/assets/passives/{pid}.png", ic, "passive"))
    # AI-33 founder emblems (icons_founder_emblems_1) are not placed: at 17 x 13 the drawn set signed off 17:56 reads
    # better (compare sheet in the project files art-ui/queue_2026-11-19/); the generated sheets stay as masters.
    # AI-25 class markers: 3 x 2 brass glyphs, 18 x 18 (16 px glyph centred)
    for k, c in zip(CLASSES, cells(S("icons_class_glyphs_1.png"), 3, 2, inset=6)):
        jobs.append((G + f"ui/markers/class_{k}.png", place(shrink(c, 16, 16), 18, 18, bottom=False), "marker"))
    # AI-25 bed keepsakes, the timber bunk and its screen are retired (PR7-53 take 8 layout); masters keep them
    # AI-25 founder peg boards (cells 1..6); the bunk cells 7..9 stay in the master only
    fb = cells(S("props_hub_founder_bunks_1.png"), 3, 3, inset=4, row_edges=[0, 300, 590, 1024])
    for f, c in zip(FOUNDERS, fb[:6]):
        jobs.append((P + f"bunk_gear_{f}/v0.png", tight(c, 24, 25), "gear"))
    # AI-32 footlocker: sheet 1, tile 1 closed and tile 3 open (2 x 2 with gutters), both on Build's 33 x 29 canvas with
    # the same bottom line; the open one lives at `footlocker_open/v0.png` (the asset test takes only v0.png names)
    fl = cells(S("props_hub_footlocker_2_1.png"), 2, 2, inset=14)
    closed, opened = shrink(fl[0], 33, 29), shrink(fl[2], 33, 29)
    jobs.append((P + "footlocker/v0.png", place(closed, 33, 29), "footlocker"))
    jobs.append((P + "footlocker_open/v0.png", place(opened, 33, 29), "footlocker"))
    # AI-31 (amended: the take 5 set that take 8 shows) at Build's seed paths: `bed_timber_stack` (2 cells, the
    # half-built frame on a sawhorse, sheet 1 tile 1) and `rolled_pallet` (1 cell, two bedrolls lifted off the crates
    # of sheet 2 and sheet 1 tile 4; the crates beside them are the kit's `crate_stack`)
    us = cells(S("props_hub_upgrade_spots_2_1.png"), 2, 2, inset=14)
    jobs.append((P + "bed_timber_stack/v0.png", tight(us[0], 74, 56), "spot"))
    top = lift_roll(Image.open(src / "props_hub_upgrade_spots_2_2.png"), (730, 770, 990, 950))
    low = lift_roll(Image.open(src / "props_hub_upgrade_spots_2_1.png"), (700, 750, 980, 950))
    pile = Image.new("RGBA", (max(top.width, low.width) + 20, low.height + top.height // 2), (0, 0, 0, 0))
    pile.alpha_composite(top, (20, 0))
    pile.alpha_composite(low, (0, pile.height - low.height))
    jobs.append((P + "rolled_pallet/v0.png", tight(pile.crop(pile.getbbox()), 32, 32), "spot"))
    for path, im, _ in jobs:
        dst = root / path
        dst.parent.mkdir(parents=True, exist_ok=True)
        im.save(dst)
    return jobs


def contact(jobs, out):
    """Every cut at 4x on the hub floor colour and on the slate, with its path."""
    rows = []
    for path, im, kind in jobs:
        z = 4 if im.width <= 48 else 2
        rows.append((path, im.resize((im.width * z, im.height * z), Image.NEAREST), im.resize((im.width, im.height))))
    W = 1400
    H = sum(max(r[1].height, 20) + 14 for r in rows) + 20
    sheet = Image.new("RGB", (W, H), (22, 24, 30))
    d = ImageDraw.Draw(sheet)
    y = 10
    for path, big, one in rows:
        sheet.paste((54, 48, 42), (10, y, 10 + big.width + 8, y + big.height + 8))
        sheet.paste(big, (14, y + 4), big)
        sheet.paste(one, (40 + big.width, y + 4), one)
        d.text((60 + big.width + one.width, y + 4), f"{path}  {one.width}x{one.height}", fill=(220, 210, 190))
        y += max(big.height, 20) + 14
    sheet.save(out)


def main():
    src, root = Path(sys.argv[1]), Path(sys.argv[2])
    jobs = run(src, root)
    if len(sys.argv) > 3:
        contact(jobs, sys.argv[3])
    for p, im, _ in jobs:
        print(p, im.size)


if __name__ == "__main__":
    main()
