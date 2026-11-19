"""AI-21 battle mock take 15 (Tushar 2026-11-18 22:41): side panel capacity without scrolling.

Usage: python3 art_src/style/battle_take15_mock.py <playtest_dir> <out_dir>
The shipped data fields at most 7 foes at once (sunk_ward encounters; phases add up to 4 more) and 2 guests. The panel
column is 1280x720's side panel, y 80 to 700 (620 px). (A) party: 4 full plates, then a Guests divider with compact
AI plates (teal rim, the Protect tag for a protect objective). (B) 7 foes: the target keeps the full plate, the rest go
compact (52 px). (C) 11 foes (a phase's reinforcements in): identical foes group into one row with stacked portraits
and one HP bar each; a click expands the group.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take11_mock as T  # noqa: E402
import battle_take12_mock as T12  # noqa: E402

P = B.ROOT / "games/emberwake/assets/portraits"
GUEST = (60, 140, 140)
FOE_RIM = (170, 60, 50)
TOP, BOTTOM = 80, 700
COL_W = 210


def portrait(rel, px):
    return Image.open(P / rel).convert("RGB").resize((px, px), Image.LANCZOS)


def tabs(img, d, x, y, sts, side, shown):
    o = T12.ordered(sts)
    tw = 32
    tx = x if side == "right" else x - tw
    items = o[:shown] if len(o) <= shown else o[:shown - 1] + [None]
    for i, it in enumerate(items):
        ty = y + 2 + i * 20
        d.rectangle((tx, ty, tx + tw, ty + 18), fill=(16, 18, 25), outline=B.IRON)
        if it is None:
            d.text((tx + 7, ty + 2), f"+{len(o) - shown + 1}", font=B.MONO, fill=B.PARCH)
            continue
        st, n, inc = it
        T12.mini(img, d, tx + 2, ty + 1, st, incoming=inc, px=12)
        d.text((tx + 21, ty + 2), str(n), font=B.MONO, fill=B.PARCH)


def compact_foe(img, d, x, y, face, name, hp, mx, act, sts=(), sel=False, w=152):
    d.rectangle((x, y, x + w, y + 52), fill=B.SLATE, outline=FOE_RIM if sel else B.IRON)
    img.paste(face.resize((40, 40)), (x + 6, y + 6))
    d.text((x + 52, y + 3), name, font=B.SMALL, fill=B.PARCH)
    B.bar(d, x + 52, y + 21, w - 100, 5, hp / mx, (150, 50, 45))
    d.text((x + w - 44, y + 16), f"{hp}/{mx}", font=T12.TINY, fill=B.PARCH)
    B.glyph(d, "sword", x + 52, y + 31, 12, (200, 160, 120))
    for b in range(6):
        d.rectangle((x + 70 + b * 8, y + 36, x + 70 + b * 8 + 5, y + 40), fill=B.IMPACT if b in act else B.AP_OFF)
    B.poise(d, x + 52, y + 46, 5, 5, w=14, h=3)
    if sts:
        tabs(img, d, x - 1, y, sts, "left", 2)


def group_row(img, d, x, y, faces, name, hps, mx, act, w=152):
    """Identical foes in one row: stacked portraits, a count, one thin HP bar each."""
    n = len(hps)
    h = 30 + n * 9
    d.rectangle((x, y, x + w, y + h), fill=B.SLATE, outline=B.IRON)
    for k in range(min(3, n)):
        img.paste(faces.resize((30, 30)), (x + 6 + k * 6, y + 6 + k * 3))
    d.text((x + 52, y + 3), f"{name} \u00d7{n}", font=B.SMALL, fill=B.PARCH)
    for i, hp in enumerate(hps):
        B.bar(d, x + 52, y + 22 + i * 9, w - 74, 4, hp / mx, (150, 50, 45))
        if act[i] is not None:
            d.rectangle((x + w - 14, y + 20 + i * 9, x + w - 8, y + 26 + i * 9), fill=B.IMPACT)
    d.polygon([(x + w - 14, y + 8), (x + w - 6, y + 8), (x + w - 10, y + 13)], fill=B.IRON_L)   # expand
    return h


def compact_guest(img, d, x, y, face, name, hp, mx, protect=False, w=152):
    d.rectangle((x, y, x + w, y + 52), fill=B.SLATE, outline=GUEST)
    img.paste(face.resize((40, 40)), (x + 6, y + 6))
    d.text((x + 52, y + 3), name, font=B.SMALL, fill=B.PARCH)
    B.bar(d, x + 52, y + 21, w - 100, 5, hp / mx, B.HP)
    d.text((x + w - 44, y + 16), f"{hp}/{mx}", font=T12.TINY, fill=B.PARCH)
    d.rounded_rectangle((x + 52, y + 31, x + 70, y + 45), radius=3, fill=(30, 60, 60))
    d.text((x + 55, y + 31), "AI", font=T12.TINY, fill=B.PARCH)
    B.intent_tile(d, x + 76, y + 30, s=14)
    if protect:
        d.rounded_rectangle((x + 96, y + 31, x + w - 6, y + 45), radius=3, outline=B.GOLD)
        d.text((x + 101, y + 31), "Protect", font=T12.TINY, fill=B.GOLD)


def column(title, note):
    img = Image.new("RGB", (COL_W + 40, BOTTOM + 30), B.VOID)
    d = ImageDraw.Draw(img)
    d.rectangle((20, TOP - 4, 20 + 170, BOTTOM), fill=B.BASALT, outline=B.IRON)
    d.text((8, 8), title, font=B.SMALL, fill=B.GOLD)
    for i, line in enumerate(note):
        d.text((8, 26 + i * 15), line, font=B.SMALL, fill=B.IRON_L)
    d.line((0, BOTTOM, COL_W + 40, BOTTOM), fill=B.BLOOD)
    d.text((8, BOTTOM + 6), "bottom of a 720p screen", font=B.SMALL, fill=B.BLOOD)
    return img, d


def col_party():
    img, d = column("A. Party 4 + guests 2", ["Heroes keep the full plate;", "guests (AI) go compact below"])
    heroes = ["kaelen", "vesper", "mara_voss", "dunstan"]
    sts = [[("bleed", 1, True), ("fortify", 2, False)], [("weaken", 1, False)], [], [("haste", 1, False)]]
    y = TOP + 4
    for i, h in enumerate(heroes):
        T12.ally_plate(img, d, 26, y, T.face(h, 48), h.split("_")[0].capitalize(), sts[i], sel=i == 0)
        y += 92
    d.line((26, y + 4, 184, y + 4), fill=GUEST)
    d.text((26, y + 8), "Guests", font=B.SMALL, fill=GUEST)
    y += 28
    compact_guest(img, d, 26, y, portrait("heroes/hollis.png", 40), "Hollis", 60, 90, protect=True)
    y += 58
    compact_guest(img, d, 26, y, portrait("common/warrior_cohort_levy.png", 40), "Levy", 70, 70)
    y += 58
    d.text((26, y + 6), f"{BOTTOM - y - 6} px to spare", font=B.SMALL, fill=B.IRON_L)
    return img


def foe_faces(shot):
    return {"footman": shot.crop((1044, 220, 1108, 284)), "crawler": portrait("common/crawler_ash_crawler.png", 64),
            "bolt": portrait("common/ranger_cohort_bolt.png", 64), "flue": portrait("common/rogue_cohort_flue.png", 64),
            "breaker": portrait("common/brawler_cohort_breaker.png", 64), "vat": portrait("common/mage_cohort_vat.png", 64)}


def col_seven(shot):
    f = foe_faces(shot)
    img, d = column("B. 7 foes (the most in the data)", ["The target keeps the full plate;", "the rest compact (52 px)"])
    y = TOP + 4
    T12.foe_plate(img, d, 34, y, f["footman"], [("stagger", 1, False), ("bleed", 3, True)], sel=True)
    y += 90
    rows = [("footman", "Footman", 144, 144, {3}, ()), ("crawler", "Ash Crawler", 60, 80, {1}, (("burn", 2, False),)),
            ("crawler", "Ash Crawler", 80, 80, {2}, ()), ("bolt", "Bolt Ranger", 70, 70, {0, 4}, ()),
            ("flue", "Flue Rogue", 55, 66, {5}, (("bleed", 1, False), ("slow", 2, False), ("expose", 1, False))),
            ("vat", "Vat Mage", 50, 50, {2}, ())]
    for key, name, hp, mx, act, sts in rows:
        compact_foe(img, d, 34, y, f[key], name, hp, mx, act, sts)
        y += 58
    d.text((34, y + 4), f"{BOTTOM - y - 4} px to spare", font=B.SMALL, fill=B.IRON_L)
    return img


def col_eleven(shot):
    f = foe_faces(shot)
    img, d = column("C. 11 foes (a phase adds 4)", ["Identical foes group in one row;", "click a group to expand it"])
    y = TOP + 4
    T12.foe_plate(img, d, 34, y, f["footman"], [("stagger", 1, False)], sel=True)
    y += 90
    y += group_row(img, d, 34, y, f["footman"], "Footman", [144, 120], 144, [3, None]) + 6
    y += group_row(img, d, 34, y, f["crawler"], "Ash Crawler", [60, 80, 80, 40], 80, [1, 2, None, 4]) + 6
    compact_foe(img, d, 34, y, f["bolt"], "Bolt Ranger", 70, 70, {0, 4})
    y += 58
    y += group_row(img, d, 34, y, f["flue"], "Flue Rogue", [55, 66], 66, [5, None]) + 6
    compact_foe(img, d, 34, y, f["vat"], "Vat Mage", 50, 50, {2})
    y += 58
    d.text((34, y + 4), f"{BOTTOM - y - 4} px to spare", font=B.SMALL, fill=B.IRON_L)
    return img


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    cols = [col_party(), col_seven(shot), col_eleven(shot)]
    sheet = Image.new("RGB", (sum(c.width for c in cols) + 30, cols[0].height), B.VOID)
    x = 0
    for c in cols:
        sheet.paste(c, (x, 0))
        x += c.width + 15
    B.label(sheet, "Take 15: side panels at 720p without scrolling").save(out / "battle_layout_take15_sheet.png")


if __name__ == "__main__":
    main()
