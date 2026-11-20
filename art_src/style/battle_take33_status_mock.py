"""AI-38 battle mock take 33, the status take after take 25 (Tushar 2026-11-19 19:26; STATUSVIS1 section 5), 1280x720.

Usage: python3 art_src/style/battle_take33_status_mock.py <playtest_dir> <out_dir>
One frame per SV5..SV10 over take 24 B (icon-only tabs) and the take 23 timeline view:
  - SV5 locks: the take 25 hatch on the locked blocks, the icon framed on each locked block; stagger locks blocks 1
    and 4 (Kaelen: two paced 2-block moves, no 3-block move fits), stun locks 1 and 2 (Dunstan opens in block 3);
  - SV6 one HP loss: the hits and the round-end bleed tick are one planned-loss slice and one number (-30); hover
    breaks it down (each hit, each tick);
  - SV7 numbers after modifiers: the card shows the final number only, tinted and with a small arrow when a modifier
    changed it (down oxblood, up gold); hover shows the base and each modifier with its percent;
  - SV8 taunt on the unit: a small badge over the taunted unit's sprite (it bobs 1 px), its hover names the taunter;
  - SV9 riposte: the chip reads "Riposte", hover gives the likely return on the selected hero;
  - SV10 loadout passives: two small icons under the portrait (apart from the status column), hover names them;
    steadfast stays first in the column.
Passive icons are placeholders (skill icons): no passive icon set exists yet.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take12_mock as T12  # noqa: E402
import battle_take17_mock as T17  # noqa: E402
import battle_take23_timeline_mock as T23  # noqa: E402
import battle_take24_mock as T24  # noqa: E402
import battle_take25_status_where_mock as T25  # noqa: E402

PLATE = (30, 35, 45)
DOWN, UP = (226, 112, 100), (240, 196, 80)     # final number tints: lowered (oxblood light), raised (gold)
DOWN_A, UP_A = (176, 52, 48), (251, 191, 36)


def base(shot):
    T24.MODE = "none"
    return T24.screen(shot)


def tip(img, x, y, head, rows, w=250, foot=None):
    """A hover: a title, then label / value rows with the values right aligned, an optional total line."""
    h = 30 + 17 * len(rows) + (22 if foot else 0) + 6
    T24.shadow(img, (x, y, x + w, y + h))
    T24.nine(img, (x, y, x + w, y + h), "hud")
    d = ImageDraw.Draw(img)
    d.text((x + 12, y + 7), head, font=B.TITLE_S, fill=B.GOLD)
    for i, (lab, val, col) in enumerate(rows):
        yy = y + 30 + i * 17
        d.text((x + 12, yy), lab, font=B.SMALL, fill=B.IRON_L)
        d.text((x + w - 12 - d.textlength(val, font=B.MONO), yy + 1), val, font=B.MONO, fill=col)
    if foot:
        yy = y + 30 + 17 * len(rows) + 3
        d.line((x + 10, yy, x + w - 10, yy), fill=B.IRON)
        lab, val, col = foot
        d.text((x + 12, yy + 4), lab, font=B.SMALL, fill=B.PARCH)
        d.text((x + w - 12 - d.textlength(val, font=B.MONO), yy + 5), val, font=B.MONO, fill=col)
    return h


# SV5 -------------------------------------------------------------------------------------------------------------
def locks(img):
    T23.ROWS[0] = ("kaelen", "Kaelen", [("pommel_strike", 1, 2, False), ("pommel_strike", 4, 2, False)],
                   [(2, "f0"), (5, "f0")])
    T23.ROWS[3] = ("dunstan", "Dunstan", [("pommel_strike", 2, 2, False), ("arrow", 4, 1, False),
                                          ("shield_wall", 5, 1, False)], [(3, "f0")])
    d = ImageDraw.Draw(img)
    T23.timeline_view(img, d, 70)
    bw = (T23.X1 - T23.X0 - T23.NAME_W - 8) / 6
    for row, blocks, st in ((0, (0, 3), "stagger"), (3, (0, 1), "stun")):
        y = 70 + 42 + row * T23.ROW_H
        col = T12.rim(T12.KIND[st])
        for b in blocks:
            x0 = int(T23.X0 + T23.NAME_W + b * bw + 2)
            x1 = int(x0 + bw - 4)
            T25.hatch(d, (x0, y + 3, x1, y + T23.ROW_H - 6), col, bg=(14, 14, 18), step=5)
            ix = x0 + int(bw / 2) - 9
            d.rectangle((ix - 1, y + 6, ix + 16, y + 23), fill=(14, 14, 18), outline=col)
            T25.icon(img, st, ix, y + 7)
    return img


# SV6 -------------------------------------------------------------------------------------------------------------
def one_loss(img, hover=True):
    d = ImageDraw.Draw(img)
    d.rectangle((146, 106, 161, 113), fill=B.IMPACT)                   # hits and the tick: one slice
    d.rectangle((130, 115, 166, 129), fill=PLATE)
    d.text((132, 117), "-30", font=B.MONO, fill=B.IMPACT)
    if hover:
        d.rectangle((130, 115, 154, 129), outline=B.GOLD)
        tip(img, 200, 92, "Kaelen this round",
            [("Brawler, block 2", "-12", B.IMPACT), ("Brawler, block 5", "-10", B.IMPACT),
             ("Bleed, round end", "-8", B.IMPACT)], w=230, foot=("Planned loss", "-30", B.IMPACT))
    return img


# SV7 -------------------------------------------------------------------------------------------------------------
ROWS_Y = (271, 301, 331)


def final_numbers(img, vals, up=False, hover=True):
    d = ImageDraw.Draw(img)
    tint, arrow = (UP, UP_A) if up else (DOWN, DOWN_A)
    for yy, v in zip(ROWS_Y, vals):
        d.rectangle((842, yy, 905, yy + 16), fill=(16, 18, 25))
        d.text((844, yy), f"-{v}", font=B.MONO_L, fill=tint)
        ax = 846 + int(d.textlength(f"-{v}", font=B.MONO_L)) + 3
        if up:
            d.polygon([(ax, yy + 11), (ax + 8, yy + 11), (ax + 4, yy + 5)], fill=arrow)
        else:
            d.polygon([(ax, yy + 6), (ax + 8, yy + 6), (ax + 4, yy + 12)], fill=arrow)
    if hover:
        d.rectangle((841, ROWS_Y[0] - 1, 900, ROWS_Y[0] + 17), outline=B.GOLD)
        if up:
            rows = [("Base", "34", B.PARCH), ("Empower", "+20%", UP_A), ("Fortify", "-6%", DOWN_A)]
            foot = ("Hammer strike", f"-{vals[0]}", UP)
        else:
            rows = [("Base", "34", B.PARCH), ("Weaken", "-15%", DOWN_A), ("Fortify", "-6%", DOWN_A)]
            foot = ("Hammer strike", f"-{vals[0]}", DOWN)
        tip(img, 610, 262, "Damage to Footman", rows, w=210, foot=foot)
    return img


# SV8 -------------------------------------------------------------------------------------------------------------
def taunt_badge(img, x, y, bob=0, hover=False):
    """A badge over the sprite's head: dark slate, crimson rim, the taunt icon, a pointer down to the unit."""
    d = ImageDraw.Draw(img)
    y += bob
    col = (190, 40, 50)
    d.rectangle((x, y, x + 19, y + 19), fill=(16, 18, 25), outline=col)
    d.polygon([(x + 6, y + 20), (x + 13, y + 20), (x + 9, y + 24)], fill=col)
    T25.icon(img, "taunt", x + 2, y + 2)
    if hover:
        tip(img, x - 210, y - 8, "Taunted", [("Must target", "Footman", B.PARCH), ("Rounds left", "1", B.PARCH)],
            w=200)


def taunt(img, bob=0, hover=False):
    taunt_badge(img, 629, 364, bob, hover)
    return img


# SV9 -------------------------------------------------------------------------------------------------------------
def riposte(img, hover=True):
    d = ImageDraw.Draw(img)
    x, y = 962, 236
    d.rectangle((x - 2, y - 1, x + 100, y + 19), fill=(16, 18, 25))
    d.rounded_rectangle((x, y, x + 64, y + 17), radius=3, fill=(40, 16, 18), outline=DOWN_A)
    T25.icon(img, "riposte", x + 1, y + 1)
    d.text((x + 20, y + 3), "Riposte", font=T12.TINY, fill=B.PARCH)
    if hover:
        d.rounded_rectangle((x - 1, y - 1, x + 65, y + 18), radius=3, outline=B.GOLD)
        tip(img, 830, y + 26, "Riposte", [("Hits back on each melee hit", "", B.PARCH),
                                          ("Likely on Kaelen", "about -9", B.IMPACT),
                                          ("Hammer strike lands", "1 return", B.IRON_L)], w=236)
    return img


# SV10 ------------------------------------------------------------------------------------------------------------
PASSIVES = (("brace", "Sword Training"), ("aegis", "Hardy"))


def passives(img, hover=True):
    d = ImageDraw.Draw(img)
    for i, (ic, _) in enumerate(PASSIVES):
        x, y = 22 + i * 17, 135
        d.rectangle((x, y, x + 15, y + 15), fill=(16, 18, 25), outline=B.BRONZE)
        im = B.slot_icon(ic, 64).resize((14, 14), Image.BOX)
        img.paste(im, (x + 1, y + 1), im)
    T24.tab(img, d, 168, 84, "steadfast", 1, False)
    if hover:
        d.rectangle((21, 134, 38, 151), outline=B.GOLD)
        tip(img, 200, 120, "Loadout passives", [("Sword Training", "always on", B.IRON_L),
                                                ("Hardy", "always on", B.IRON_L)], w=220)
    return img


def label_tiles(tiles, title):
    W = max(t.width for _, t in tiles) + 12
    H = sum(t.height + 26 for _, t in tiles) + 30
    sheet = Image.new("RGB", (W, H), B.VOID)
    d = ImageDraw.Draw(sheet)
    y = 30
    for lab, t in tiles:
        d.text((6, y), lab, font=B.SMALL, fill=B.GOLD)
        sheet.paste(t, (6, y + 18))
        y += t.height + 26
    return B.label(sheet, title)


def z2(im, box):
    return im.crop(box).resize(((box[2] - box[0]) * 2, (box[3] - box[1]) * 2), Image.NEAREST)


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    lk = locks(base(shot))
    lk.save(out / "battle_take33_locks_720p.png")
    hp = one_loss(base(shot))
    wk = final_numbers(base(shot), (27, 10, 10))
    em = final_numbers(base(shot), (39, 13, 13), up=True)
    t0, t1 = taunt(base(shot)), taunt(base(shot), bob=-1, hover=True)
    rp = riposte(base(shot))
    pv = passives(base(shot))
    full = passives(riposte(taunt(final_numbers(one_loss(base(shot), hover=False), (27, 10, 10), hover=False))),
                    hover=False)
    full.save(out / "battle_take33_status_720p.png")
    tiles = [("SV5 locks: stagger locks blocks 1 and 4, stun locks 1 and 2 (hatch, icon on each)", z2(lk, (204, 108, 1076, 252))),
             ("SV6 one HP loss: hits and the bleed tick in one slice and one number; hover breaks it down",
              z2(hp, (14, 84, 440, 200))),
             ("SV7 numbers after modifiers: weakened (down), hover shows the base and each modifier",
              z2(wk, (600, 256, 1000, 384))),
             ("SV7 the same hit empowered (up)", z2(em, (600, 256, 1000, 384))),
             ("SV8 taunt: a badge over the taunted unit (bobs 1 px), hover names the taunter",
              Image.fromarray(__import__("numpy").hstack([__import__("numpy").array(z2(t0, (400, 340, 720, 510))),
                                                           __import__("numpy").array(z2(t1, (400, 340, 720, 510)))]))),
             ("SV9 riposte: the chip without a number, hover gives the likely return", z2(rp, (820, 228, 1076, 344))),
             ("SV10 loadout passives under the portrait (placeholder icons), steadfast first in the column",
              z2(pv, (14, 80, 430, 190)))]
    label_tiles(tiles, "Take 33: statuses where they act, after take 25 review (STATUSVIS1 section 5, 2x)").save(
        out / "battle_take33_status_2x.png")


if __name__ == "__main__":
    main()
