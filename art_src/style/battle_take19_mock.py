"""AI-21 battle mock take 19 (Tushar 2026-11-18 23:03): per-hero AP back on the plate, readable at a glance.

Usage: python3 art_src/style/battle_take19_mock.py <playtest_dir> <out_dir>
Take 18 A plate (panel 184) with an AP row under HP, four ways (AP cap 12; amber = planned this round):
  1. grouped pips: 3 groups of 4 (you read 4-groups without counting)
  2. bar with ticks at 4 and 8
  3. bar plus the number
  4. grouped pips plus the number
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take11_mock as T  # noqa: E402
import battle_take17_mock as T17  # noqa: E402
import battle_take18_mock as T18  # noqa: E402

STC = T17.STC
PLAN = {"kaelen": 6, "vesper": 0, "mara_voss": 4, "dunstan": 0}
AP = {"kaelen": 10, "vesper": 8, "mara_voss": 12, "dunstan": 3}
CAP = 12


def grouped(d, x, y, ap, plan, size=5, gap=1, ggap=5, h=7):
    for i in range(CAP):
        px = x + i * (size + gap) + (i // 4) * (ggap - gap)
        if i < ap - plan:
            c = B.AP_ON
        elif i < ap:
            c = B.PLANNED
        else:
            c = B.AP_OFF
        d.rectangle((px, y, px + size - 1, y + h - 1), fill=c)
    return x + CAP * (size + gap) + 2 * (ggap - gap)


def ticked(d, x, y, w, ap, plan):
    d.rectangle((x, y, x + w, y + 6), fill=B.AP_OFF)
    a = round(w * (ap - plan) / CAP)
    b = round(w * ap / CAP)
    d.rectangle((x, y, x + a, y + 6), fill=B.AP_ON)
    d.rectangle((x + a, y, x + b, y + 6), fill=B.PLANNED)
    for k in (4, 8):
        tx = x + round(w * k / CAP)
        d.line((tx, y - 1, tx, y + 7), fill=B.SLATE, width=2)


def ap_row(d, x, y, w, key, style):
    ap, plan = AP[key], PLAN[key]
    if style == 1:
        grouped(d, x, y, ap, plan)
    elif style == 2:
        ticked(d, x, y + 1, w, ap, plan)
    elif style == 3:
        ticked(d, x, y + 1, w - 30, ap, plan)
        d.text((x + w - 24, y - 3), str(ap - plan), font=B.MONO, fill=B.PARCH)
    else:
        e = grouped(d, x, y, ap, plan, size=4, gap=1, ggap=3)
        d.text((e + 3, y - 4), str(ap - plan), font=B.MONO, fill=B.PARCH)


def plate(img, d, x, y, key, name, hp, mx, ap, loss, sts, sel, style, w=176):
    h = 84
    d.rectangle((x, y, x + w, y + h), fill=B.SLATE, outline=B.GOLD if sel else B.IRON)
    img.paste(T.face(key, 48), (x + 5, y + 5))
    cx = x + 59
    cw = w - STC - 4 - 59
    d.text((cx, y + 3), name, font=B.BODY, fill=B.PARCH)
    B.intent_tile(d, cx + cw - 18, y + 4, s=18)
    T18.hp_row(d, cx, y + 26, cw - 2, hp, mx, loss)
    ap_row(d, cx, y + 56, cw - 2, key, style)
    B.poise(d, x + 5, y + 74, 5, 5, w=(w - STC - 14) // 5 - 2, h=5)
    d.line((x + w - STC - 2, y + 4, x + w - STC - 2, y + h - 4), fill=B.BASALT)
    T17.status_col(img, d, x + w - STC + 1, y + 2, h - 4, sts)


def panel(style, title):
    img = Image.new("RGB", (192, 400), B.BASALT)
    d = ImageDraw.Draw(img)
    d.text((6, 6), title, font=B.SMALL, fill=B.GOLD)
    y = 28
    for i, hrow in enumerate(T18.HEROES):
        plate(img, d, 4, y, *hrow, sel=i == 0, style=style)
        y += 90
    return img


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    names = ["1. Pips in 4s", "2. Bar, ticks at 4, 8", "3. Bar + number", "4. Pips in 4s + number"]
    ps = [panel(i + 1, n) for i, n in enumerate(names)]
    one = Image.new("RGB", (sum(p.width for p in ps) + 12 * 3, ps[0].height), B.VOID)
    x = 0
    for p in ps:
        one.paste(p, (x, 0)); x += p.width + 12
    zoom = one.resize((one.width * 2, one.height * 2), Image.NEAREST)
    sheet = Image.new("RGB", (zoom.width, one.height + 24 + zoom.height + 60), B.VOID)
    d = ImageDraw.Draw(sheet)
    d.text((8, 4), "1:1  (Kaelen 10 AP with 6 planned, Vesper 8, Mara 12 with 4 planned, Dunstan 3; amber = planned)",
           font=B.SMALL, fill=B.IRON_L)
    sheet.paste(one, (0, 24))
    d.text((8, one.height + 30), "2x zoom", font=B.SMALL, fill=B.IRON_L)
    sheet.paste(zoom, (0, one.height + 50))
    B.label(sheet, "Take 19: per-hero AP on the plate, four ways to read it at a glance").save(
        out / "battle_layout_take19_sheet.png")


if __name__ == "__main__":
    main()
