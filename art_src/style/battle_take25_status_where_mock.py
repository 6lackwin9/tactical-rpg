"""AI-21 battle mock take 25 (Tushar 2026-11-19 18:05; STATUSVIS1, AI-34): statuses shown where they act, 1280x720.

Usage: python3 art_src/style/battle_take25_status_where_mock.py <playtest_dir> <out_dir>
Over take 24 B (uncounted tabs) and the take 23 timeline view. One frame per STATUSVIS1 primitive (section 2); the
plate's status column stays as the list (SV2, SV3):
  1. block_lock (stagger, stun, root): locked blocks hatched in the status's rim colour on a near-black block, the
     icon on the first locked block, on the unit's timeline lane and at the end of the top strip's AP row (-2);
  2. tick (bleed, poison, burn, regen): the coming tick as a pending oxblood slice after the planned impact on the HP
     bar, a drip and the amount by the impact number, and a tick mark at the round's end on the lane;
  3. attack_power (weaken, empower): the changed number in the hit preview with a small down (oxblood) or up (gold)
     arrow, the base value struck through beside it;
  4. move_rate (haste, slow): the tiles slow takes away drawn as a dim ring at the range edge, the icon on it;
  5. armor / guard (expose, fortify, guard): the incoming number already includes it, a shield glyph and sign beside;
  6. targeting (taunt, stealth): a taunter pulls the foe intent lines to it (crimson hoop); a stealthed unit is greyed
     as a target while choosing;
  7. reactive (riposte): a counter chip in the hit preview ("Riposte -9": the damage it hits back);
  8. rule_flag (steadfast): the plate only, first in the column (priority 1).
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

STAGGER = T12.rim(T12.KIND["stagger"])
BLEED = (176, 36, 52)
WEAK = (176, 52, 48)                 # oxblood down arrow (STATUSVIS1 2); gold up for empower
OXB = (120, 30, 34)
SLOW = (110, 170, 220)


def hatch(d, box, col, bg=(24, 22, 28), step=3):
    x0, y0, x1, y1 = box
    d.rectangle(box, fill=bg, outline=col)
    w, h = x1 - x0, y1 - y0
    for k in range(-h, w, step):                 # 45 degree lines clipped to the box
        sx, sy = max(0, k), max(0, -k)
        n = min(w - sx, h - sy)
        d.line((x0 + sx, y0 + sy, x0 + sx + n, y0 + sy + n), fill=col)


def icon(img, st, x, y):
    ic = T17.icon16(st)
    img.paste(ic, (x, y), ic)


LANE_X, LANE_W = 324, 124            # take 23: six block columns from x 324, 124 px each


def lane_y(row):
    return 70 + 44 + row * 34           # take 23: header 44 px, rows of 34


def lock_block(img, d, row, b0, b1, st):
    y = lane_y(row)
    col = T12.rim(T12.KIND[st])
    hatch(d, (LANE_X + LANE_W * b0, y, LANE_X + LANE_W * b1 - 2, y + 28), col, bg=(14, 14, 18), step=5)
    x = LANE_X + LANE_W * b0 + 6
    d.rectangle((x - 1, y + 5, x + 16, y + 22), fill=(14, 14, 18), outline=col)
    icon(img, st, x, y + 6)


def screen(shot):
    T24.MODE = "none"
    img = T24.screen(shot)
    d = ImageDraw.Draw(img)
    # 1. stagger: the last two AP blocks locked in the top strip, and on Kaelen's AP bar
    for i in (10, 11):
        x = 588 + 13 * i
        hatch(d, (x, 12, x + 10, 22), STAGGER)
    icon(img, "stagger", 752, 34)
    d.text((770, 34), "-2", font=B.MONO, fill=STAGGER)
    hatch(d, (112, 135, 124, 141), STAGGER, step=2)
    # 2. bleed: the round-end tick after the planned impact on Kaelen's HP bar
    d.rectangle((146, 106, 152, 112), fill=OXB)
    d.rectangle((154, 119, 156, 125), fill=BLEED)
    d.point((155, 118), fill=BLEED)
    d.text((157, 118), "8", font=T12.TINY, fill=BLEED)
    # 3. weaken: Kaelen's damage in the preview card, the base struck through
    for yy, base, now in ((271, 34, 27), (301, 12, 10), (331, 12, 10)):
        d.rectangle((842, yy, 905, yy + 16), fill=(16, 18, 25))
        d.text((844, yy), f"-{now}", font=B.MONO_L, fill=B.PARCH)
        d.polygon([(874, yy + 6), (882, yy + 6), (878, yy + 12)], fill=WEAK)
        d.text((886, yy + 2), str(base), font=T12.TINY, fill=B.IRON_L)
        d.line((885, yy + 8, 899, yy + 8), fill=B.IRON_L)
    # 4. slow: the far move tile lost, a dim ring, the icon on the range edge
    dim = Image.new("RGBA", img.size)
    ImageDraw.Draw(dim).rectangle((904, 442, 994, 506), fill=(8, 10, 14, 150))
    base = img.convert("RGBA"); base.alpha_composite(dim); img.paste(base.convert("RGB"))
    d = ImageDraw.Draw(img)
    for k in range(904, 995, 6):
        d.point((k, 442), fill=SLOW); d.point((k, 506), fill=SLOW)
    icon(img, "slow", 940, 466)
    # 5. bleed drips on Kaelen's sprite (the existing status VFX)
    for x, y in ((628, 452), (640, 466), (652, 458), (634, 478)):
        d.rectangle((x, y, x + 1, y + 3), fill=BLEED)
    return img


def preview_variant(img, kind):
    """Frames 5 and 7: the hit preview card with a fortify shield or a riposte chip (over the take 24 card)."""
    d = ImageDraw.Draw(img)
    if kind == "guard":
        for yy, val, cut in ((271, 28, 6), (301, 8, 4), (331, 8, 4)):
            d.rectangle((842, yy, 905, yy + 16), fill=(16, 18, 25))
            d.text((844, yy), f"-{val}", font=B.MONO_L, fill=B.PARCH)
            icon(img, "fortify", 872, yy)
            d.text((890, yy + 2), f"-{cut}", font=T12.TINY, fill=T12.rim("buff"))
    else:
        x, y = 962, 236
        d.rounded_rectangle((x, y, x + 84, y + 17), radius=3, fill=(40, 16, 18), outline=WEAK)
        icon(img, "riposte", x + 1, y + 1)
        d.text((x + 19, y + 3), "Riposte -9", font=T12.TINY, fill=B.PARCH)


def targeting(img):
    """Frame 6: the Footman taunts: a crimson hoop at its feet, the Archer's intent line bent to it."""
    d = ImageDraw.Draw(img)
    d.ellipse((812, 478, 900, 502), outline=(190, 40, 50), width=2)
    for k in range(0, 170, 12):                       # the Brawler's intent, bent from the panel to the taunter
        x = 1076 - k; y = 432 + k * 0.3
        d.line((x, y, x - 6, y + 2), fill=(190, 40, 50), width=2)
    icon(img, "taunt", 900, 470)
    return img


def timeline(img):
    d = ImageDraw.Draw(img)
    T23.timeline_view(img, d, 70)
    lock_block(img, d, 0, 5, 6, "stagger")            # Kaelen: stagger takes block 6
    lock_block(img, d, 3, 0, 6, "stun")               # Dunstan: stunned all round
    y = lane_y(0)
    d.rectangle((1066, y + 2, 1070, y + 26), fill=OXB)  # Kaelen: the bleed tick at the round's end
    d.rectangle((1067, y - 6, 1069, y), fill=BLEED)
    return img


def steadfast(img, d):
    """Frame 8: steadfast first in Kaelen's column (priority 1), the other statuses after it."""
    x, y = 168, 84
    for i, st in enumerate(("steadfast", "bleed", "stagger")):
        T24.tab(img, d, x, y + i * 20, st, 1, False)


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    s = screen(shot)
    s.save(out / "battle_take25_status_where_720p.png")
    tl = timeline(screen(shot))
    tl.save(out / "battle_take25_timeline_locks_720p.png")
    g = screen(shot); preview_variant(g, "guard")
    r = screen(shot); preview_variant(r, "riposte")
    t = targeting(screen(shot))
    sf = screen(shot); steadfast(sf, ImageDraw.Draw(sf))
    crops = [("1. block_lock: stagger and stun lock blocks on the lanes (T view)", tl, (204, 108, 1076, 252)),
             ("1. block_lock: the top strip's AP row and the plate's AP bar (-2)", s, (570, 4, 800, 56)),
             ("2. tick: the coming bleed as a pending slice, a drip and its amount", s, (14, 84, 196, 150)),
             ("3. attack_power: weaken, the reduced number with its arrow", s, (760, 260, 1000, 356)),
             ("4. move_rate: slow dims the lost tile at the range edge", s, (840, 400, 1010, 512)),
             ("5. armor / guard: the foe's fortify cuts the number, shown beside it", g, (760, 260, 1000, 356)),
             ("6. targeting: a taunter's hoop, the foe intents bent to it", t, (780, 428, 1080, 512)),
             ("7. reactive: riposte as a counter chip in the hit preview", r, (760, 180, 1070, 262)),
             ("8. rule_flag: steadfast lives only on the plate, first", sf, (14, 80, 198, 150))]
    tiles = [(lab, im.crop(b).resize(((b[2] - b[0]) * 2, (b[3] - b[1]) * 2), Image.NEAREST)) for lab, im, b in crops]
    cols = [tiles[:1], tiles[1:]]
    W = max(t.width for _, t in tiles)
    H = sum(t.height + 26 for _, t in tiles) + 30
    sheet = Image.new("RGB", (W + 12, H), B.VOID)
    d = ImageDraw.Draw(sheet)
    y = 30
    for label, t in tiles:
        d.text((6, y), label, font=B.SMALL, fill=B.GOLD)
        sheet.paste(t, (6, y + 18))
        y += t.height + 26
    B.label(sheet, "Take 25: statuses shown where they act (STATUSVIS1, 2x)").save(out / "battle_take25_status_where_2x.png")


if __name__ == "__main__":
    main()
