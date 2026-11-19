"""AI-21 battle mock take 21 (Tushar 2026-11-18 23:11): the party plate's AP as a continuous bar, the balance after the plan.

Usage: python3 art_src/style/battle_take21_mock.py <out_dir>
The plate shows what is left at the end of the round once the plan runs (10 AP, plan 6: "4/12"); the bar shows the
start AP with the planned part in another colour. The timeline keeps its pips (how the AP is spent).
  A. planned part in amber (the strip's planned colour)
  B. planned part as a faded blue ghost (the strip's ghost look)
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take11_mock as T  # noqa: E402
import battle_take13_mock as T13  # noqa: E402
import battle_take17_mock as T17  # noqa: E402
import battle_take18_mock as T18  # noqa: E402

STC = T17.STC
CAP = 12
AP = {"kaelen": 10, "vesper": 8, "mara_voss": 12, "dunstan": 3}
PLAN = {"kaelen": 6, "vesper": 0, "mara_voss": 4, "dunstan": 3}


def ap_row(d, x, y, w, key, style):
    ap, plan = AP[key], PLAN[key]
    left = ap - plan
    bw = w - 36
    B.bar(d, x, y, bw, 5, 0, B.AP_ON)
    a = round(bw * left / CAP)
    b = round(bw * ap / CAP)
    if left:
        d.rectangle((x, y, x + a, y + 5), fill=B.AP_ON)
    if plan:
        col = B.PLANNED if style == "A" else T13.mix(B.AP_ON, B.BASALT, 0.45)
        d.rectangle((x + a + (1 if left else 0), y, x + b, y + 5), fill=col)
    d.text((x + bw + 4, y - 5), f"{left}/{CAP}", font=B.MONO, fill=B.PARCH if left else B.IRON_L)


def plate(img, d, x, y, key, name, hp, mx, ap, loss, sts, sel, style, w=176):
    h = 84
    d.rectangle((x, y, x + w, y + h), fill=B.SLATE, outline=B.GOLD if sel else B.IRON)
    img.paste(T.face(key, 48), (x + 5, y + 5))
    cx = x + 59
    cw = w - STC - 4 - 59
    fnt = B.BODY if d.textlength(name, font=B.BODY) <= cw - 22 else B.SMALL
    d.text((cx, y + 3 if fnt == B.BODY else y + 5), name, font=fnt, fill=B.PARCH)
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
    out = Path(sys.argv[1])
    ps = [panel("A", "A. Planned AP in amber"), panel("B", "B. Planned AP faded")]
    one = Image.new("RGB", (sum(p.width for p in ps) + 12, ps[0].height), B.VOID)
    one.paste(ps[0], (0, 0)); one.paste(ps[1], (ps[0].width + 12, 0))
    zoom = one.resize((one.width * 2, one.height * 2), Image.NEAREST)
    sheet = Image.new("RGB", (one.width + 24 + zoom.width, zoom.height + 54), B.VOID)
    d = ImageDraw.Draw(sheet)
    d.text((8, 4), "1:1", font=B.SMALL, fill=B.IRON_L)
    sheet.paste(one, (0, 24))
    d.text((one.width + 32, 4), "2x zoom   (Kaelen 10 AP, plan 6 -> 4/12; Vesper 8, no plan; Mara 12, plan 4; "
           "Dunstan 3, plan 3 -> 0/12)", font=B.SMALL, fill=B.IRON_L)
    sheet.paste(zoom, (one.width + 24, 24))
    B.label(sheet, "Take 21: party AP as a bar, the number is the balance after the plan").save(
        out / "battle_layout_take21_sheet.png")


if __name__ == "__main__":
    main()
