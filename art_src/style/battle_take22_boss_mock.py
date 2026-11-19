"""AI-21 battle mock take 22 (Tushar 2026-11-18 23:19, low priority): the boss fight variant, 1280x720 at 1:1.

Usage: python3 art_src/style/battle_take22_boss_mock.py <playtest_dir> <out_dir>
Built on the take 17 screen with the signed-off panels (184 px each side, take 20/21 plates):
  - a boss bar across the top of the board: crown and portrait, the full name and title, a long HP bar with a mark at
    each phase threshold (the next one labelled), the poise pips with the per-pip tag, its statuses, and the next
    phase or enrage cue ("Phase 2 at 66%", "Enrage round 6");
  - the boss leaves the foe panel: the panel lists the mobs only, with a "Boss above" line;
  - the docked foe card moves under the boss bar (it shows the boss when the boss is the target).
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

BOSS_HP, BOSS_MAX = 610, 900
PHASES = [0.66, 0.33]                      # HP fractions where a phase begins


def boss_bar(img, d, x0, x1, y):
    h = 64
    d.rectangle((x0, y, x1, y + h), fill=B.BASALT, outline=(116, 84, 60))
    d.rectangle((x0 + 3, y + 3, x1 - 3, y + h - 3), outline=(75, 85, 99))
    d.line((x0 + 4, y + 4, x1 - 4, y + 4), fill=(153, 27, 27))
    img.paste(T15.portrait("bosses/grakka_ashmaw.png", 46), (x0 + 8, y + 7))
    T20.crown(d, x0 + 7, y + 5, 11)
    tx = x0 + 62
    d.text((tx, y + 5), "Grakka Ashmaw", font=B.TITLE_S, fill=B.GOLD)
    d.text((tx + d.textlength("Grakka Ashmaw", font=B.TITLE_S) + 10, y + 8), "Chieftain of the Horde", font=B.SMALL,
           fill=B.IRON_L)
    # HP: one long bar, a mark per phase threshold, the next one labelled
    bx, bw = tx, x1 - 12 - 92 - tx
    B.bar(d, bx, y + 28, bw, 9, BOSS_HP / BOSS_MAX, (150, 50, 45))
    loss = 96
    e = bx + round(bw * BOSS_HP / BOSS_MAX)
    d.rectangle((e - round(bw * loss / BOSS_MAX), y + 28, e, y + 37), fill=B.IMPACT)
    for f in PHASES:
        px = bx + round(bw * f)
        d.rectangle((px - 1, y + 25, px + 1, y + 40), fill=B.GOLD)
    nx = bx + round(bw * PHASES[0])
    d.text((nx + 4, y + 40), "Phase 2", font=T12.TINY, fill=B.GOLD)
    d.text((bx + bw + 6, y + 25), f"{BOSS_HP}/{BOSS_MAX}", font=B.MONO, fill=B.PARCH)
    d.text((bx + bw + 6, y + 40), f"-{loss}", font=B.MONO, fill=B.IMPACT)
    # poise and the per-pip tag
    B.poise_glyph(d, tx, y + 42, 11)
    B.poise(d, tx + 16, y + 45, 5, 3, w=18, h=5)
    B.pip_tag(d, tx + 120, y + 41, 18)
    # statuses (tabs in a row) and the cue on the right of the title row
    sx = tx + 170
    for st, n, inc in [("fortify", 2, False), ("haste", 1, False), ("bleed", 3, True)]:
        T17.tab(img, d, sx, y + 41, st, n, inc)
        sx += 22
    cue = "Enrage round 6"
    cw = d.textlength(cue, font=B.SMALL) + 12
    d.rounded_rectangle((x1 - 10 - cw, y + 6, x1 - 10, y + 22), radius=3, outline=B.AMBER)
    d.text((x1 - 4 - cw, y + 7), cue, font=B.SMALL, fill=B.AMBER)
    return y + h


def screen(shot):
    img = T17.screen(shot)
    d = ImageDraw.Draw(img)
    # party panel at 184 (take 20/21 plates)
    d.rectangle((12, 70, 198, 640), fill=B.BASALT)
    y = 80
    for i, hrow in enumerate(T18.HEROES):
        T21.plate(img, d, 16, y, *hrow, sel=i == 0, style="A")
        y += 90
    d.line((16, y + 2, 192, y + 2), fill=T15.GUEST)
    d.text((16, y + 5), "Guests", font=B.SMALL, fill=T15.GUEST)
    T17.guest_plate(img, d, 16, y + 24, T15.portrait("heroes/hollis.png", 40), "Hollis", 60, 90, True)
    # foe panel at 184: mobs only
    d.rectangle((1082, 70, 1268, 640), fill=B.BASALT)
    d.text((1088, 80), "Hostiles 4", font=B.SMALL, fill=B.PARCH)
    d.text((1166, 80), "Boss above", font=B.SMALL, fill=B.GOLD)
    p = T15.portrait
    mobs = [(p("common/brawler_cohort_breaker.png", 64), "Brawler", 88, 120, 5, [("stagger", 1, False)], ("kaelen", 1)),
            (p("common/ranger_cohort_bolt.png", 64), "Archer", 70, 70, 5, [], ("vesper", 3)),
            (p("common/rogue_cohort_flue.png", 64), "Skirmisher", 60, 80, 4, [("burn", 2, False)], ("mara_voss", 2)),
            (p("common/mage_cohort_vat.png", 64), "Shaman", 70, 70, 5, [], ("dunstan", 4))]
    y = 100
    for face, name, hp, mx, pips, sts, lands in mobs:
        T20.foe_plate(img, d, 1086, y, face, name, hp, mx, pips, sts, lands, w=176)
        y += 88
    # the boss bar over the board, the docked card under it (clear the take 17 card first)
    fresh = B.mock10(shot)
    box = (199, 64, 1081, 420)
    img.paste(fresh.crop(box), box[:2])
    img.paste(img.crop((260, 120, 590, 142)), (750, 120))      # mock10's own card edge: cover with plain wall
    yb = boss_bar(img, d, 206, 1074, 68)
    card = T14.card_hero(shot)
    c = card.crop((6, 6, card.width - 6, card.height - 6))
    img.paste(c, (1074 - c.width, yb + 6))
    return img


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    s = screen(shot)
    s.save(out / "battle_take22_boss_720p.png")
    z = s.crop((196, 60, 1084, 140)).resize((888 * 2, 160), Image.NEAREST)
    z.save(out / "battle_take22_boss_bar_2x.png")


if __name__ == "__main__":
    main()
