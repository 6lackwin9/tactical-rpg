"""AI-21 battle mock take 24 (Tushar 2026-11-19 18:00): take 22's boss fight with his notes, 1280x720 at 1:1.

Usage: python3 art_src/style/battle_take24_mock.py <playtest_dir> <out_dir>
  - status rounds: the 3 pips per tab read poorly at this size; each tab now shows the rounds left as a bold
    pixel digit (3x5 drawn at 2x) beside the icon; a lasting status keeps the full bar;
  - the boss bar is its own element: the HUD double-line frame (`ui/frames/hud.png`, nine-slice 8 at 2x) with a
    shadow, inset from the board edge so no border runs through it; three rows: name and title with the phase and
    enrage cues; one continuous HP bar (phases trigger on the one HP pool, `hp_below`, so it never refills) with
    each threshold marked and labelled, and the planned impact; poise, statuses (up to 6, numbered) and the boss's
    intent with its impact, as on the foe plates (the target's face and the block it lands in);
  - the side panels sit in the shipped panel frame (`ui/frames/panel.png`) instead of flat fills.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take12_mock as T12  # noqa: E402
import battle_take14_mock as T14  # noqa: E402
import battle_take15_mock as T15  # noqa: E402
import battle_take17_mock as T17  # noqa: E402
import battle_take18_mock as T18  # noqa: E402
import battle_take20_mock as T20  # noqa: E402
import battle_take21_mock as T21  # noqa: E402

FRAMES = B.ROOT / "games/emberwake/assets/ui/frames"
BOSS_HP, BOSS_MAX, LOSS = 610, 900, 96
PHASES = [(0.50, "Phase 2", "50% or round 3"), (0.25, "Phase 3", "25%")]
DIGITS = {"0": "111101101101111", "1": "010110010010111", "2": "111001111100111", "3": "111001111001111",
          "4": "101101111001001", "5": "111100111001111", "6": "111100111101111", "7": "111001001001001",
          "8": "111101111101111", "9": "111101111001111"}


def digit(d, x, y, ch, col, s=2):
    for i, bit in enumerate(DIGITS[ch]):
        if bit == "1":
            px, py = x + (i % 3) * s, y + (i // 3) * s
            d.rectangle((px, py, px + s - 1, py + s - 1), fill=col)


def tab(img, d, x, y, st, n, inc):
    """The take 17 tab (16 px icon, rim by kind) with the rounds left as a digit instead of pips."""
    kind = T12.KIND[st]
    col = T12.rim(kind)
    d.rectangle((x, y, x + 24, y + 18), fill=(16, 18, 25))
    if inc:
        for k in range(0, 25, 3):
            d.point((x + k, y), fill=col); d.point((x + k, y + 18), fill=col)
        for k in range(0, 19, 3):
            d.point((x, y + k), fill=col); d.point((x + 24, y + k), fill=col)
    else:
        d.rectangle((x, y, x + 24, y + 18), outline=col)
    ic = T17.icon16(st)
    img.paste(ic, (x + 2 if MODE == "count" else x + 4, y + 1), ic)
    if MODE == "count":
        if n < 0:
            d.rectangle((x + 19, y + 2, x + 23, y + 17), fill=col)
        else:
            digit(d, x + 18, y + 4, str(min(n, 9)), (240, 232, 214))


MODE = "count"                 # "count": the rounds as a digit; "none": no count on the plate (his 18:02 option)


def ordered(sts):
    """The shipped order in both (debuffs first, then the soonest to expire); in B the + view lists the rounds."""
    return ORIG_ORDERED(sts)


ORIG_ORDERED = T12.ordered


def nine(img, box, name, scale=2):
    src = Image.open(FRAMES / f"{name}.png").convert("RGBA")
    s = 8
    x0, y0, x1, y1 = box
    w, h = x1 - x0, y1 - y0
    c = s * scale
    out = Image.new("RGBA", (w, h))
    parts = {}
    xs = [(0, s), (s, src.width - s), (src.width - s, src.width)]
    ys = [(0, s), (s, src.height - s), (src.height - s, src.height)]
    dx = [(0, c), (c, w - c), (w - c, w)]
    dy = [(0, c), (c, h - c), (h - c, h)]
    for j in range(3):
        for i in range(3):
            p = src.crop((xs[i][0], ys[j][0], xs[i][1], ys[j][1]))
            tw, th = dx[i][1] - dx[i][0], dy[j][1] - dy[j][0]
            if tw > 0 and th > 0:
                out.paste(p.resize((tw, th), Image.NEAREST), (dx[i][0], dy[j][0]))
    base = img.convert("RGBA")
    base.alpha_composite(out, (x0, y0))
    img.paste(base.convert("RGB"))
    return parts


def shadow(img, box, off=3):
    x0, y0, x1, y1 = box
    sh = Image.new("RGBA", img.size)
    ImageDraw.Draw(sh).rectangle((x0 + off, y0 + off, x1 + off, y1 + off), fill=(0, 0, 0, 120))
    base = img.convert("RGBA")
    base.alpha_composite(sh)
    img.paste(base.convert("RGB"))


def boss_bar(img, x0, x1, y):
    h = 84
    shadow(img, (x0, y, x1, y + h))
    nine(img, (x0, y, x1, y + h), "hud")
    d = ImageDraw.Draw(img)
    img.paste(T15.portrait("bosses/grakka_ashmaw.png", 56), (x0 + 10, y + 12))
    d.rectangle((x0 + 9, y + 11, x0 + 66, y + 68), outline=B.GOLD)
    T20.crown(d, x0 + 8, y + 7, 12)
    tx = x0 + 76
    d.text((tx, y + 8), "Grakka Ashmaw", font=B.TITLE_S, fill=B.GOLD)
    d.text((tx + d.textlength("Grakka Ashmaw", font=B.TITLE_S) + 10, y + 11), "Chieftain of the Horde", font=B.SMALL,
           fill=B.IRON_L)
    cx = x1 - 12
    for text, col in (("Enrage round 6", B.AMBER), ("Next: Phase 2 at 50% or round 3", B.GOLD)):
        cw = d.textlength(text, font=B.SMALL) + 12
        d.rounded_rectangle((cx - cw, y + 9, cx, y + 25), radius=3, outline=col)
        d.text((cx - cw + 6, y + 10), text, font=B.SMALL, fill=col)
        cx -= cw + 6
    # one continuous HP bar, the phase thresholds marked and labelled, the planned impact
    bx, bw = tx, x1 - 14 - 76 - tx
    by = y + 32
    B.bar(d, bx, by, bw, 10, BOSS_HP / BOSS_MAX, (150, 50, 45))
    e = bx + round(bw * BOSS_HP / BOSS_MAX)
    d.rectangle((e - round(bw * LOSS / BOSS_MAX), by, e, by + 10), fill=B.IMPACT)
    for f, name, _ in PHASES:
        px = bx + round(bw * f)
        d.rectangle((px - 1, by - 3, px + 1, by + 13), fill=B.GOLD)
        d.text((px + 4, by + 11), name, font=T12.TINY, fill=B.GOLD)
    d.text((bx + bw + 8, by - 3), f"{BOSS_HP}/{BOSS_MAX}", font=B.MONO, fill=B.PARCH)
    d.text((bx + bw + 8, by + 11), f"-{LOSS}", font=B.MONO, fill=B.IMPACT)
    # row 3: poise, statuses (numbered), the intent and its impact
    ry = y + 56
    B.poise_glyph(d, tx, ry + 2, 12)
    B.poise(d, tx + 16, ry + 6, 5, 3, w=16, h=5)
    B.pip_tag(d, tx + 110, ry + 2, 18)
    sx = tx + 162
    for st, n, inc in ordered([("fortify", 2, False), ("haste", 1, False), ("bleed", 3, True), ("burn", 2, False)]):
        tab(img, d, sx, ry, st, n, inc)
        sx += 28
    ix = sx + 18
    d.line((ix - 10, ry, ix - 10, ry + 18), fill=B.IRON)
    B.intent_tile(d, ix, ry + 1, s=16)
    d.text((ix + 22, ry + 1), "Cleaving Roar", font=B.SMALL, fill=B.PARCH)
    gx = ix + 26 + int(d.textlength("Cleaving Roar", font=B.SMALL))
    B.glyph(d, "sword", gx, ry + 3, 12, (200, 160, 120))
    img.paste(T17.T.face("kaelen", 16), (gx + 16, ry + 1))
    for b in range(6):
        d.rectangle((gx + 38 + b * 13, ry + 7, gx + 38 + b * 13 + 10, ry + 12), fill=B.IMPACT if b == 1 else B.AP_OFF)
    d.text((gx + 38 + 6 * 13 + 4, ry + 1), "-42", font=B.MONO, fill=B.IMPACT)
    return y + h


def screen(shot):
    T17.tab = tab                                  # every plate's status column draws the take 24 tabs
    T12.ordered = ordered
    img = T17.screen(shot)
    fresh = B.mock10(shot)
    box = (199, 64, 1081, 420)
    img.paste(fresh.crop(box), box[:2])
    img.paste(img.crop((260, 120, 590, 142)), (750, 120))
    # side panels in the panel frame
    for px0, px1 in ((10, 200), (1080, 1270)):
        img.paste(B.BASALT, (px0, 66, px1, 644))
        nine(img, (px0, 66, px1, 644), "panel")
    d = ImageDraw.Draw(img)
    y = 80
    for i, hrow in enumerate(T18.HEROES):
        T21.plate(img, d, 16, y, *hrow, sel=i == 0, style="A")
        y += 90
    d.line((16, y + 2, 192, y + 2), fill=T15.GUEST)
    d.text((16, y + 5), "Guests", font=B.SMALL, fill=T15.GUEST)
    T17.guest_plate(img, d, 16, y + 24, T15.portrait("heroes/hollis.png", 40), "Hollis", 60, 90, True)
    d.text((1092, 80), "Hostiles 4", font=B.SMALL, fill=B.PARCH)
    d.text((1170, 80), "Boss above", font=B.SMALL, fill=B.GOLD)
    p = T15.portrait
    mobs = [(p("common/brawler_cohort_breaker.png", 64), "Brawler", 88, 120, 5, [("stagger", 1, False)], ("kaelen", 1)),
            (p("common/ranger_cohort_bolt.png", 64), "Archer", 70, 70, 5, [], ("vesper", 3)),
            (p("common/rogue_cohort_flue.png", 64), "Skirmisher", 60, 80, 4, [("burn", 2, False)], ("mara_voss", 2)),
            (p("common/mage_cohort_vat.png", 64), "Shaman", 70, 70, 5, [], ("dunstan", 4))]
    y = 100
    for face, name, hp, mx, pips, sts, lands in mobs:
        T20.foe_plate(img, d, 1088, y, face, name, hp, mx, pips, sts, lands, w=172)
        y += 88
    yb = boss_bar(img, 212, 1068, 74)
    card = T14.card_hero(shot)
    c = card.crop((6, 6, card.width - 6, card.height - 6))
    img.paste(c, (1068 - c.width, yb + 8))
    return img


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    global MODE
    for MODE, tag in (("count", "A_counts"), ("none", "B_no_counts")):
        s = screen(shot)
        s.save(out / f"battle_take24_{tag}_720p.png")
        s.crop((204, 66, 1076, 166)).resize((872 * 2, 200), Image.NEAREST).save(out / f"battle_take24_{tag}_boss_bar_2x.png")
        s.crop((10, 66, 200, 300)).resize((380, 468), Image.NEAREST).save(out / f"battle_take24_{tag}_status_2x.png")


if __name__ == "__main__":
    main()
