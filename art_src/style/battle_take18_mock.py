"""AI-21 battle mock take 18 (Tushar 2026-11-18 23:01): the hero plate re-laid out.

Usage: python3 art_src/style/battle_take18_mock.py <playtest_dir> <out_dir>
The plate keeps: portrait, name, round intent (never merged with the statuses), HP as bar and number, poise; AP leaves
the plate for a team AP gauge in the panel header. Statuses keep their own column inside the frame.
  A. Panel 24 px wider (184): name and intent on one row, every name fits (Dunstan included); the board gives up 24 px.
  B. Panel as now (160): the intent rides as a badge on the portrait's lower right corner; the name gets the full row.
Both shown at 1:1 next to a 2x zoom.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take11_mock as T  # noqa: E402
import battle_take15_mock as T15  # noqa: E402
import battle_take17_mock as T17  # noqa: E402

STC = T17.STC
HEROES = [("kaelen", "Kaelen", 242, 242, 10, 22, [("bleed", 1, True), ("fortify", 2, False), ("haste", 1, False),
                                                  ("regen", 2, False), ("slow", 3, False)]),
          ("vesper", "Vesper", 150, 190, 8, 0, [("weaken", 1, False)]),
          ("mara_voss", "Mara", 200, 210, 12, 0, []),
          ("dunstan", "Dunstan", 260, 300, 6, 0, [("haste", 1, False), ("fortify", 1, False)])]


def team_ap(d, x, y, w):
    """The party's AP in one gauge: a segment per hero (its share of 12), the number on the right."""
    d.text((x, y), "Team AP", font=B.SMALL, fill=B.IRON_L)
    tot = sum(h[4] for h in HEROES)
    d.text((x + w - 44, y), f"{tot}/48", font=B.MONO, fill=B.PARCH)
    gx, gw = x, w
    seg = (gw - 3 * 3) / 4
    for i, h in enumerate(HEROES):
        sx = gx + i * (seg + 3)
        d.rectangle((sx, y + 18, sx + seg, y + 23), fill=B.AP_OFF)
        d.rectangle((sx, y + 18, sx + seg * h[4] / 12, y + 23), fill=B.AP_ON)
    for i, h in enumerate(HEROES):
        sx = gx + i * (seg + 3)
        d.text((sx, y + 26), f"{h[1][0]} {h[4]}", font=T17.T12.TINY, fill=B.IRON_L)
    return y + 42


def hp_row(d, x, y, w, hp, mx, loss):
    B.bar(d, x, y, w, 6, hp / mx, B.HP)
    if loss:
        l = round(w * loss / mx)
        e = x + round(w * hp / mx)
        d.rectangle((e - l, y, e, y + 6), fill=B.IMPACT)
    d.text((x, y + 9), f"{hp}/{mx}", font=B.MONO, fill=B.PARCH)
    if loss and w >= 80:                      # B's narrower row keeps the loss on the bar only
        d.text((x + 56, y + 9), f"-{loss}", font=B.MONO, fill=B.IMPACT)


def plate_a(img, d, x, y, key, name, hp, mx, ap, loss, sts, sel, w=176):
    h = 84
    d.rectangle((x, y, x + w, y + h), fill=B.SLATE, outline=B.GOLD if sel else B.IRON)
    img.paste(T.face(key, 48), (x + 5, y + 5))
    cx = x + 59
    cw = w - STC - 4 - 59
    d.text((cx, y + 4), name, font=B.BODY, fill=B.PARCH)
    B.intent_tile(d, cx + cw - 18, y + 4, s=18)
    hp_row(d, cx, y + 30, cw - 2, hp, mx, loss)
    B.poise(d, x + 5, y + 66, 5, 5, w=(w - STC - 14) // 5 - 2, h=5)
    d.text((x + 5, y + 72), "", font=B.SMALL)
    d.line((x + w - STC - 2, y + 4, x + w - STC - 2, y + h - 4), fill=B.BASALT)
    T17.status_col(img, d, x + w - STC + 1, y + 2, h - 4, sts)


def plate_b(img, d, x, y, key, name, hp, mx, ap, loss, sts, sel, w=152):
    h = 84
    d.rectangle((x, y, x + w, y + h), fill=B.SLATE, outline=B.GOLD if sel else B.IRON)
    img.paste(T.face(key, 48), (x + 5, y + 5))
    B.intent_tile(d, x + 37, y + 37, s=18)                     # the intent as a badge on the portrait corner
    cx = x + 59
    cw = w - STC - 4 - 59
    fnt = B.BODY if d.textlength(name, font=B.BODY) <= cw else B.SMALL
    d.text((cx, y + 4), name, font=fnt, fill=B.PARCH)
    hp_row(d, cx, y + 30, cw - 2, hp, mx, loss)
    B.poise(d, x + 5, y + 66, 5, 5, w=(w - STC - 14) // 5 - 2, h=5)
    d.line((x + w - STC - 2, y + 4, x + w - STC - 2, y + h - 4), fill=B.BASALT)
    T17.status_col(img, d, x + w - STC + 1, y + 2, h - 4, sts)


def panel(plate, pw, title):
    img = Image.new("RGB", (pw + 8, 560), B.BASALT)
    d = ImageDraw.Draw(img)
    y = team_ap(d, 6, 6, pw - 4)
    for i, hrow in enumerate(HEROES):
        plate(img, d, 4, y, *hrow, sel=i == 0, w=pw)
        y += 90
    d.line((4, y + 2, pw + 4, y + 2), fill=T15.GUEST)
    d.text((4, y + 5), "Guests", font=B.SMALL, fill=T15.GUEST)
    T17.guest_plate(img, d, 4, y + 24, T15.portrait("heroes/hollis.png", 40), "Hollis", 60, 90, True)
    return img, title


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    pa, ta = panel(plate_a, 176, "A. Panel 184 px (+24): name and intent on one row")
    pb, tb = panel(plate_b, 152, "B. Panel as now (160): intent as a badge on the portrait")
    one = Image.new("RGB", (pa.width + pb.width + 40, pa.height), B.VOID)
    one.paste(pa, (0, 0)); one.paste(pb, (pa.width + 40, 0))
    zoom = one.resize((one.width * 2, one.height * 2), Image.NEAREST)
    sheet = Image.new("RGB", (one.width + 30 + zoom.width, zoom.height + 60), B.VOID)
    d = ImageDraw.Draw(sheet)
    d.text((8, 8), "1:1", font=B.SMALL, fill=B.GOLD)
    sheet.paste(one, (0, 30))
    zx = one.width + 30
    d.text((zx + 8, 8), ta + "        |        " + tb + "   (2x zoom)", font=B.SMALL, fill=B.GOLD)
    sheet.paste(zoom, (zx, 30))
    B.label(sheet, "Take 18: hero plate without AP (team AP gauge in the header), statuses in their own column").save(
        out / "battle_layout_take18_sheet.png")


if __name__ == "__main__":
    main()
