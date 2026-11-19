"""AI-21 battle mock take 20 (Tushar 2026-11-18 23:07): party plate A, foe plates with the intent on the portrait.

Usage: python3 art_src/style/battle_take20_mock.py <out_dir>
  Party (184 px panel): take 18 A, name and intent on one row, AP style 4 from take 19 (provisional until he picks).
  Foes (160 px panel, as now): the intent rides as a badge on the portrait; the plate shows the unit's short name
  (Nemeth, Crossbowman); a boss gets a crown and a double rim. The full name and the title go in the docked foe card
  header, which has the room (up to 220 px for the name, the title on its own line).
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take12_mock as T12  # noqa: E402
import battle_take15_mock as T15  # noqa: E402
import battle_take17_mock as T17  # noqa: E402
import battle_take18_mock as T18  # noqa: E402
import battle_take19_mock as T19  # noqa: E402

STC = T17.STC
FOE_RIM = T17.FOE_RIM
BOSS_RIM = (150, 40, 36)
NAME_W = 66                      # the foe plate's name room (152 px plate: status column, 44 px portrait)


def crown(d, x, y, s=10):
    d.polygon([(x, y + s), (x, y + 3), (x + s * 0.3, y + s * 0.55), (x + s / 2, y), (x + s * 0.7, y + s * 0.55),
               (x + s, y + 3), (x + s, y + s)], fill=B.GOLD, outline=(60, 40, 20))


def fit_name(d, name, room):
    """BODY, then SMALL, then cut with an ellipsis: the reliable fallback for any name."""
    for f in (B.BODY, B.SMALL):
        if d.textlength(name, font=f) <= room:
            return name, f
    while name and d.textlength(name + "…", font=B.SMALL) > room:
        name = name[:-1]
    return name + "…", B.SMALL


def foe_plate(img, d, x, y, face, name, hp, mx, pips, sts, lands, sel=False, boss=False, w=152):
    h = 78
    if boss:
        d.rectangle((x - 2, y - 2, x + w + 2, y + h + 2), outline=B.GOLD)
    d.rectangle((x, y, x + w, y + h), fill=B.SLATE, outline=FOE_RIM if sel else (BOSS_RIM if boss else B.IRON))
    T17.status_col(img, d, x + 2, y + 2, h - 4, sts)
    d.line((x + STC + 2, y + 4, x + STC + 2, y + h - 4), fill=B.BASALT)
    fx = x + STC + 5
    img.paste(face.resize((44, 44)), (fx, y + 5))
    B.intent_tile(d, fx + 30, y + 33, s=16)                  # the round intent, a badge on the portrait corner
    if boss:
        crown(d, fx - 1, y + 3)
    tx = fx + 49
    nm, fnt = fit_name(d, name, x + w - 4 - tx)
    d.text((tx, y + 2 if fnt == B.BODY else y + 4), nm, font=fnt, fill=B.GOLD if boss else B.PARCH)
    bw = x + w - 6 - tx
    B.bar(d, tx, y + 24, bw, 5, hp / mx, (150, 50, 45))
    d.text((tx, y + 31), f"{hp}/{mx}", font=B.MONO, fill=B.PARCH)
    who, blk = lands
    ly = y + 54
    B.glyph(d, "sword", fx, ly, 12, (200, 160, 120))
    img.paste(T17.T.face(who, 14), (fx + 16, ly - 1))
    for b in range(6):
        d.rectangle((fx + 36 + b * 12, ly + 4, fx + 36 + b * 12 + 9, ly + 8), fill=B.IMPACT if b == blk else B.AP_OFF)
    B.poise(d, fx, y + h - 7, 5, pips, w=(x + w - 6 - fx) // 5 - 2, h=3)


def foe_panel(title, pw=152):
    img = Image.new("RGB", (pw + 16, 420), B.BASALT)
    d = ImageDraw.Draw(img)
    d.text((6, 6), title, font=B.SMALL, fill=B.GOLD)
    d.text((6, 24), "Hostiles 4", font=B.SMALL, fill=B.PARCH)
    p = T15.portrait
    foes = [(p("bosses/inquisitor_prelate_nemeth.png", 64), "Nemeth", 410, 480, 5, [("fortify", 2, False)],
             ("kaelen", 2), True, True),
            (p("common/ranger_cohort_bolt.png", 64), "Crossbowman", 70, 70, 5, [], ("vesper", 1), False, False),
            (p("common/warrior_cohort_levy.png", 64), "Footman", 64, 144, 4, [("bleed", 3, True)], ("kaelen", 3),
             False, False),
            (p("common/priest_cohort_novice.png", 64), "Flagellant", 90, 96, 5, [("weaken", 1, False)], ("mara_voss", 4),
             False, False)]
    y = 44
    for face, name, hp, mx, pips, sts, lands, sel, boss in foes:
        foe_plate(img, d, 6, y, face, name, hp, mx, pips, sts, lands, sel, boss, w=pw)
        y += 88
    return img


def party_panel(title):
    img = Image.new("RGB", (192, 420), B.BASALT)
    d = ImageDraw.Draw(img)
    d.text((6, 6), title, font=B.SMALL, fill=B.GOLD)
    y = 44
    for i, hrow in enumerate(T18.HEROES):
        T19.plate(img, d, 4, y, *hrow, sel=i == 0, style=4)
        y += 90
    return img


class _Off:
    """Draw with a y offset (the card sits under its caption)."""
    def __init__(self, base, dy):
        self.base, self.dy, self.d = base, dy, ImageDraw.Draw(base)

    def paste(self, im, xy, *a):
        self.base.paste(im, (xy[0], xy[1] + self.dy), *a)

    def __getattr__(self, k):
        f = getattr(self.d, k)

        def g(xy, *a, **kw):
            if isinstance(xy, tuple) and len(xy) in (2, 4):
                xy = tuple(v + (self.dy if i % 2 else 0) for i, v in enumerate(xy))
            elif isinstance(xy, list):
                xy = [(px, py + self.dy) for px, py in xy]
            return f(xy, *a, **kw)
        return g


def _shift(img, dy):
    o = _Off(img, dy)
    return o, o


def card_header(title, name, rank, por):
    """The docked foe card's header: the full name, the title under it, then HP and poise."""
    img = Image.new("RGB", (300, 106), B.VOID)
    d = ImageDraw.Draw(img)
    d.text((2, 0), title, font=B.SMALL, fill=B.IRON_L)
    card = Image.new("RGB", (300, 86), B.BASALT)
    cd = ImageDraw.Draw(card)
    cd.rectangle((0, 0, 299, 85), outline=(116, 84, 60))
    cd.rectangle((3, 3, 296, 82), outline=(75, 85, 99))
    cd.line((4, 4, 295, 4), fill=(153, 27, 27))
    img.paste(card, (0, 18))
    img, d = _shift(img, 18)
    img.paste(T15.portrait(por, 48), (12, 12))
    crown(d, 11, 10)
    d.text((70, 8), name, font=B.TITLE_S, fill=B.GOLD)
    if rank:
        d.text((70, 28), rank, font=B.SMALL, fill=B.IRON_L)
    B.heart(d, 70, 48, 11)
    B.bar(d, 86, 51, 130, 6, 410 / 480, (150, 50, 45))
    d.text((224, 46), "410/480", font=B.MONO, fill=B.PARCH)
    B.poise_glyph(d, 69, 64, 13)
    B.poise(d, 86, 68, 5, 5, w=24, h=5)
    return img.base


def main():
    out = Path(sys.argv[1])
    pa = party_panel("Party: 184 px, name + intent")
    pf = foe_panel("Foes 160 (as now)")
    pg = foe_panel("Foes 184 (rec.)", pw=176)
    c1 = card_header("Card header: the full name (Nemeth has no title)", "Inquisitor-Prelate Nemeth", "",
                     "bosses/inquisitor_prelate_nemeth.png")
    c2 = card_header("Card header: name + title (room for 220 px)", "Grakka Ashmaw", "Chieftain of the Horde",
                     "bosses/grakka_ashmaw.png")
    # what the full names do on a foe plate (why plates take the short name)
    cut = Image.new("RGB", (300, 120), B.BASALT)
    cd = ImageDraw.Draw(cut)
    cd.text((8, 6), f"Full names in a {NAME_W} px plate row:", font=B.SMALL, fill=B.GOLD)
    for i, n in enumerate(["Watch Crossbowman", "Mortuary Flagellant", "Inquisitor-Prelate Nemeth"]):
        nm, f = fit_name(cd, n, NAME_W)
        cd.rectangle((8, 28 + i * 28, 8 + NAME_W, 48 + i * 28), outline=B.IRON)
        cd.text((10, 30 + i * 28 if f == B.BODY else 32 + i * 28), nm, font=f, fill=B.PARCH)
        cd.text((84, 32 + i * 28), f"<- {n}", font=B.SMALL, fill=B.IRON_L)
    right = Image.new("RGB", (300, c1.height + c2.height + cut.height + 24), B.VOID)
    right.paste(c1, (0, 0)); right.paste(c2, (0, c1.height + 12)); right.paste(cut, (0, c1.height + c2.height + 24))
    one = Image.new("RGB", (pa.width + pf.width + pg.width + right.width + 36, max(pa.height, right.height)), B.VOID)
    one.paste(pa, (0, 0)); one.paste(pf, (pa.width + 12, 0)); one.paste(pg, (pa.width + pf.width + 24, 0))
    one.paste(right, (pa.width + pf.width + pg.width + 36, 0))
    zoom = one.resize((one.width * 2, one.height * 2), Image.NEAREST)
    sheet = Image.new("RGB", (zoom.width, one.height + 24 + zoom.height + 60), B.VOID)
    d = ImageDraw.Draw(sheet)
    d.text((8, 4), "1:1", font=B.SMALL, fill=B.IRON_L)
    sheet.paste(one, (0, 24))
    d.text((8, one.height + 30), "2x zoom", font=B.SMALL, fill=B.IRON_L)
    sheet.paste(zoom, (0, one.height + 50))
    B.label(sheet, "Take 20: party name + intent (184), foes intent on the portrait, full name and title in the card").save(
        out / "battle_layout_take20_sheet.png")


if __name__ == "__main__":
    main()
