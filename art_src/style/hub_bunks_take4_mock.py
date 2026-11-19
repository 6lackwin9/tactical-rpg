"""Hub bunk room take 4 (Tushar 2026-11-19 17:19): take 3 with his notes, over the current hub shot, 1280x720 at 1:1.

Usage: python3 art_src/style/hub_bunks_take4_mock.py <hub shot.png> <out_dir>
  - the west barrels are gone, so the south-left bunk lies against the west wall;
  - the south-right bunk is mirrored: head on the east wall, its foot to the room;
  - the stove moves to the middle of the hall, between the two tables, away from the beds;
  - the north wall is built properly (dressed stone courses, a capstone line, a plinth), so the arch niche and the
    timber headboard are judged on a real wall;
  - a benched founder sleeps in the front pose (head and shoulders on the pillow, blanket to the chin), not the side pose;
  - each founder's gear in one of two ways: a: hung on a framed peg board on the wall beside the bed, b: laid on a
    footlocker at the foot of the bed. The keepsake stays on the bed.
Four panels: arch + peg board, arch + footlocker, headboard + peg board, headboard + footlocker.
"""
import json
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import hub_bunks_take1_mock as H1  # noqa: E402
import hub_bunks_take2_mock as H2  # noqa: E402
import hub_bunks_take3_mock as H3  # noqa: E402

PROPS = H1.PROPS
WALL_Y0, WALL_Y1 = H3.WALL_Y0, H3.WALL_Y1
DOOR = H1.DOOR
XS = H3.XS
CROP = (300, 340, 960, 664)
STONE, STONE_L, STONE_D, MORTAR = (62, 64, 70), (92, 92, 96), (40, 41, 46), (26, 27, 31)


def wall(img):
    """Dressed stone: a capstone line, courses of 20 x 9 px blocks laid in bond, a darker plinth at the floor."""
    d = ImageDraw.Draw(img)
    top = WALL_Y0 - 10
    for x0, x1 in ((352, DOOR[0]), (DOOR[1], 928)):
        d.rectangle((x0, top, x1 - 1, WALL_Y1 - 1), fill=MORTAR)
        y, row = top + 5, 0
        while y < WALL_Y1 - 6:
            off = 10 if row % 2 else 0
            x = x0 - off
            while x < x1:
                bx0, bx1 = max(x0, x), min(x1 - 1, x + 18)
                shade = STONE if (x // 20 + row) % 3 else (58, 60, 66)
                d.rectangle((bx0, y, bx1, y + 7), fill=shade)
                d.line((bx0, y, bx1, y), fill=STONE_L)
                x += 20
            y += 9
            row += 1
        d.rectangle((x0, top, x1 - 1, top + 3), fill=STONE_L)              # capstone
        d.line((x0, top + 4, x1 - 1, top + 4), fill=STONE_D)
        d.rectangle((x0, WALL_Y1 - 6, x1 - 1, WALL_Y1 - 1), fill=STONE_D)  # plinth
        d.line((x0, WALL_Y1 - 6, x1 - 1, WALL_Y1 - 6), fill=STONE)


def niche(d, cx):
    x0, x1, top, bot = cx - 22, cx + 22, WALL_Y0 + 12, WALL_Y1 - 6
    d.rectangle((x0 - 3, top + 8, x1 + 3, bot), fill=STONE_L)              # the dressed jambs
    d.pieslice((x0 - 3, top - 3, x1 + 3, top + 25), 180, 360, fill=STONE_L)
    d.rectangle((x0, top + 10, x1, bot), fill=(20, 21, 25))
    d.pieslice((x0, top, x1, top + 22), 180, 360, fill=(20, 21, 25))
    d.rectangle((cx - 2, top - 4, cx + 2, top + 1), fill=(118, 116, 110))   # keystone


def headboard(d, cx):
    d.rectangle((cx - 26, WALL_Y0 + 20, cx + 26, WALL_Y1 - 4), fill=H1.TIMBER, outline=H1.TIMBER_D)
    for k in (-1, 1):
        d.line((cx + 13 * k, WALL_Y0 + 21, cx + 13 * k, WALL_Y1 - 5), fill=H1.TIMBER_D)
    d.rectangle((cx - 28, WALL_Y0 + 18, cx + 28, WALL_Y0 + 21), fill=H1.TIMBER_D)


def gear_img(name):
    f = PROPS / f"bunk_gear_{name.lower()}" / "v0.png"
    return Image.open(f).convert("RGBA") if f.exists() else None


def pegboard(img, cx, name):
    """a: a framed timber board on the wall beside the bed, two pegs, the gear hung on it."""
    d = ImageDraw.Draw(img)
    x0, y0 = cx + 26, WALL_Y0 + 8
    d.rectangle((x0, y0, x0 + 26, y0 + 34), fill=(86, 60, 36), outline=H1.TIMBER_D)
    d.line((x0 + 1, y0 + 1, x0 + 25, y0 + 1), fill=H1.TIMBER)
    for px in (x0 + 7, x0 + 19):
        d.rectangle((px, y0 + 6, px + 1, y0 + 8), fill=(150, 150, 150))
    g = gear_img(name)
    if g:
        img.alpha_composite(g, (x0 + 1, y0 + 6))


def footlocker(img, cx, bed_bottom, name):
    """b: a small timber footlocker with iron bands at the foot of the bed, the gear laid on its lid."""
    d = ImageDraw.Draw(img)
    x0, y0 = cx - 15, bed_bottom + 2
    d.rectangle((x0, y0, x0 + 30, y0 + 13), fill=(98, 68, 40), outline=H1.TIMBER_D)
    d.rectangle((x0, y0, x0 + 30, y0 + 4), fill=(122, 86, 52), outline=H1.TIMBER_D)
    for bx in (x0 + 5, x0 + 24):
        d.line((bx, y0, bx, y0 + 13), fill=(70, 72, 78), width=2)
    d.rectangle((x0 + 13, y0 + 5, x0 + 17, y0 + 8), fill=(176, 132, 64))
    g = gear_img(name)
    if g:
        g = g.resize((20, 20), Image.BOX).rotate(90 if name in ("Kaelen", "Hollis") else 0, expand=True)
        img.alpha_composite(g, (cx - g.width // 2, y0 - g.height + 4))


FRONT = {"Dunstan": ("brawler/gauntlet", (14, 3, 34, 19)), "Elyse": ("mage/1h_sprayer", (14, 0, 34, 19))}


def sleeper(img, cx, top, name, bed):
    """A benched founder asleep: the front idle pose's head and shoulders on the pillow, the blanket up to the chin
    (stand-in for the resting pose, AI-17)."""
    body, box = FRONT[name]
    j = json.loads((H2.UNITS / body / "clips.json").read_text())
    row = [c["id"] for c in j["clips"]].index("idle_down")
    sheet = Image.open(H2.UNITS / body / "sheet.png").convert("RGBA")
    head = sheet.crop((box[0], row * 48 + box[1], box[2], row * 48 + box[3]))
    img.alpha_composite(head, (cx - head.width // 2, top + 22 - head.height))
    cover = bed.crop((0, 28, bed.width, bed.height))
    img.alpha_composite(cover, (cx - bed.width // 2, top + 28))


def north_bed(img, cx, entry, back, items):
    name, cls, state, _ = entry
    d = ImageDraw.Draw(img)
    (niche if back == "arch" else headboard)(d, cx)
    bed = H3.prop("cot_side")
    if cls:
        bed = H3.tint(bed, H1.CLASS[cls], 30)
    top = WALL_Y1 - 26
    img.alpha_composite(bed, (cx - bed.width // 2, top))
    if state == "benched":
        sleeper(img, cx, top, name, bed)
    ks = PROPS / f"bunk_keepsake_{name.lower()}" / "v0.png"
    if ks.exists():
        img.alpha_composite(Image.open(ks).convert("RGBA"), (cx - 14, top + 62))
    if items == "board":
        pegboard(img, cx, name)
    else:
        footlocker(img, cx, top + bed.height, name)
    H1.plate(ImageDraw.Draw(img), cx, WALL_Y0 - 9, name, cls, 80)


def south_bed(img, x, y, entry, mirror):
    """A Timber Frames bunk lying sideways, its head on a side wall (west, or east when mirrored); plate on a post."""
    name, cls, state, _ = entry
    bed = H3.prop("cot")
    if cls and state != "empty":
        bed = H3.tint(bed, H1.CLASS[cls], 0)
    if mirror:
        bed = bed.transpose(Image.FLIP_LEFT_RIGHT)
    img.alpha_composite(bed, (x, y))
    d = ImageDraw.Draw(img)
    px = x + bed.width + 6 if not mirror else x - 6
    px = x + 34 if not mirror else x + bed.width - 34          # the post stands by the pillow, inside the room
    d.rectangle((px - 1, y - 6, px + 2, y + 6), fill=H1.TIMBER, outline=H1.TIMBER_D)
    if name:
        H1.plate(d, px, y - 20, name, cls, 80)
    else:
        d.rectangle((px - 12, y - 20, px + 14, y - 7), fill=H1.BRASS_D, outline=H1.BRASS)


def move_stove(base, shot):
    """The stove from the east corner to the middle of the hall, between the two tables."""
    stove = shot.crop((876, 516, 922, 594))
    H3.planks(base, 870, 512, 928, 598)
    tmp = Image.new("RGBA", stove.size)
    tmp.paste(stove)
    base.paste(stove.crop((0, 0, 46, 78)), (617, 512))


def scene(shot, back, items):
    img = H3.clean_room(shot)
    H3.planks(img, 352, 552, 400, 658)                 # the west barrels go
    move_stove(img, shot)
    wall(img)
    for x, e in zip(XS, H1.ROSTER[:6]):
        north_bed(img, x, e, back, items)
    south_bed(img, 354, 592, H1.ROSTER[6], False)       # head on the west wall
    south_bed(img, 928 - 96 - 2, 592, H1.ROSTER[7], True)   # head on the east wall
    return img.crop(CROP).convert("RGB")


def main():
    shot = Image.open(sys.argv[1]).convert("RGBA")
    out = Path(sys.argv[2])
    panels = [("1a. Arch + peg board", "arch", "board"), ("1b. Arch + footlocker", "arch", "locker"),
              ("2a. Headboard + peg board", "head", "board"), ("2b. Headboard + footlocker", "head", "locker")]
    ims = [(t, scene(shot, b, i)) for t, b, i in panels]
    w, h = ims[0][1].size
    one = Image.new("RGB", (w * 2 + 12, (h + 22) * 2), B.VOID)
    d = ImageDraw.Draw(one)
    for k, (t, im) in enumerate(ims):
        x, y = (k % 2) * (w + 12), (k // 2) * (h + 22)
        d.text((x + 6, y + 3), t, font=B.SMALL, fill=B.GOLD)
        one.paste(im, (x, y + 20))
        im.save(out / f"hub_bunks_take4_{t.split('.')[0]}_720p.png")
    z = ims[0][1].crop((40, 0, 400, 180)).resize((720, 360), Image.NEAREST)
    z2 = ims[1][1].crop((380, 0, 740, 180)).resize((720, 360), Image.NEAREST)
    zz = Image.new("RGB", (1452, 382), B.VOID)
    dz = ImageDraw.Draw(zz)
    dz.text((6, 3), "1a, 2x", font=B.SMALL, fill=B.GOLD)
    dz.text((738, 3), "1b east side, 2x (Dunstan and Elyse asleep)", font=B.SMALL, fill=B.GOLD)
    zz.paste(z, (0, 22)); zz.paste(z2, (732, 22))
    sheet = Image.new("RGB", (max(one.width, zz.width), one.height + zz.height + 12), B.VOID)
    sheet.paste(one, (0, 0)); sheet.paste(zz, (0, one.height + 12))
    B.label(sheet, "Hub bunks take 4: built wall, arch or headboard, gear on a peg board or a footlocker").save(
        out / "hub_bunks_take4_sheet.png")


if __name__ == "__main__":
    main()
