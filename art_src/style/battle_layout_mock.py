"""AI-21 battle layout mock (LAYOUT_NOTES UX-14..UX-17), painted at 1280x720 from playtest shot 048.

Usage: python3 art_src/style/battle_layout_mock.py <playtest_dir> <out_dir>
Writes battle_layout_mock.png (proposed) and battle_layout_side_by_side.png (now vs proposed).
The board and portraits are cut from the shot; panels, strip and bar are redrawn on the UX-01..05 rules
(popup frame A, four colour roles, Alegreya SC titles, mono numbers).
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]
F = ROOT / "games/emberwake/assets/fonts"
TITLE = ImageFont.truetype(str(F / "AlegreyaSC-Bold.ttf"), 20)
TITLE_S = ImageFont.truetype(str(F / "AlegreyaSC-Bold.ttf"), 16)
BODY = ImageFont.truetype(str(ROOT / "engine/assets/fonts/OpenSans-SemiBold.ttf"), 14)
SMALL = ImageFont.truetype(str(ROOT / "engine/assets/fonts/OpenSans-SemiBold.ttf"), 12)
MONO = ImageFont.truetype(str(F / "JetBrainsMono-Bold.ttf"), 12)
MONO_L = ImageFont.truetype(str(F / "JetBrainsMono-Bold.ttf"), 13)

VOID, BASALT, SLATE = (10, 11, 16), (20, 23, 31), (30, 35, 45)
IRON, IRON_L = (75, 85, 99), (156, 163, 175)
BLOOD, AMBER, GOLD, PARCH = (153, 27, 27), (217, 119, 6), (251, 191, 36), (217, 203, 180)
BRONZE = (116, 84, 60)
HP = (110, 140, 70)
ASSAULT = (150, 60, 50)  # muted intent chip


def frame_a(d: ImageDraw.ImageDraw, box, fill=BASALT, top=True):
    x0, y0, x1, y1 = box
    d.rectangle(box, fill=fill)
    d.rectangle(box, outline=BRONZE, width=2)
    d.rectangle((x0 + 4, y0 + 4, x1 - 4, y1 - 4), outline=IRON, width=1)
    if top:
        d.rectangle((x0, y0, x1, y0 + 1), fill=BLOOD)


def bar(d, x, y, w, h, frac, col):
    d.rectangle((x, y, x + w, y + h), fill=(14, 16, 22))
    d.rectangle((x, y, x + int(w * frac), y + h), fill=col)


def pips(d, x, y, n, on, col=AMBER, size=7, gap=3):
    for i in range(n):
        c = col if i < on else (45, 50, 60)
        d.rectangle((x + i * (size + gap), y, x + i * (size + gap) + size, y + size), fill=c)


def chip(d, x, y, text, col):
    w = d.textlength(text, font=SMALL) + 12
    d.rounded_rectangle((x, y, x + w, y + 18), radius=4, fill=col)
    d.text((x + 6, y + 2), text, font=SMALL, fill=PARCH)
    return w


def mock(shot: Image.Image) -> Image.Image:
    img = Image.new("RGB", (1280, 720), VOID)
    d = ImageDraw.Draw(img)

    # stage: the encounter's used area, camera-fit at 1.5x (UX-15)
    stage = shot.crop((291, 112, 899, 492)).resize((912, 570), Image.NEAREST)
    img.paste(stage, (184, 72))
    d.rectangle((184, 72, 1095, 641), outline=IRON, width=1)

    # top strip (UX-16): round left, six numbered blocks with AP cost, AP pips, compact group, one amber action
    d.rectangle((0, 0, 1280, 64), fill=BASALT)
    d.line((0, 64, 1280, 64), fill=IRON)
    d.text((14, 10), "Kaelen's plan", font=TITLE, fill=GOLD)
    d.text((14, 38), "Round 1 of 8  ·  Planning", font=SMALL, fill=IRON_L)
    icons = [shot.crop((186, 12, 222, 44))] + [shot.crop((236 + i * 50, 12, 272 + i * 50, 44)) for i in range(4)] + [
        shot.crop((436, 12, 480, 44))]
    labels = [("Move", 2), ("Strike", 2), ("Strike", 2), ("Strike", 2), ("Strike", 2), ("Rest", 0)]
    bx = 300
    for i, (ic, (nm, ap)) in enumerate(zip(icons, labels)):
        x = bx + i * 74
        sel = i == 0
        d.rectangle((x, 6, x + 66, 58), fill=SLATE, outline=GOLD if sel else IRON, width=2 if sel else 1)
        d.text((x + 5, 8), f"B{i + 1}", font=MONO, fill=GOLD if sel else IRON_L)
        img.paste(ic.resize((32, 28)), (x + 30, 10))
        d.text((x + 5, 40), nm, font=SMALL, fill=PARCH)
        d.text((x + 55, 40), str(ap), font=MONO, fill=AMBER)
    ax = bx + 6 * 74 + 6
    d.text((ax, 10), "AP", font=MONO, fill=IRON_L)
    pips(d, ax, 30, 10, 9, size=8, gap=3)
    d.text((ax, 44), "9 of 10 planned", font=SMALL, fill=IRON_L)
    gx = 900
    for i, t in enumerate(("Party: Balanced", "Hero: Balanced")):
        d.rectangle((gx, 8 + i * 25, gx + 132, 30 + i * 25), fill=SLATE, outline=IRON)
        d.text((gx + 8, 11 + i * 25), t + "  v", font=SMALL, fill=PARCH)
    for i, t in enumerate(("Auto", "Clear")):
        d.rectangle((gx + 138, 8 + i * 25, gx + 196, 30 + i * 25), fill=SLATE, outline=IRON)
        d.text((gx + 150, 11 + i * 25), t, font=SMALL, fill=PARCH)
    d.rectangle((1108, 10, 1268, 54), fill=(60, 34, 10), outline=AMBER, width=2)
    d.text((1128, 21), "Confirm round", font=BODY, fill=GOLD)

    # left: party list only (UX-14)
    frame_a(d, (8, 72, 176, 641))
    d.text((20, 82), "Party", font=TITLE_S, fill=GOLD)
    d.text((136, 85), "1 / 1", font=MONO, fill=IRON_L)
    y = 110
    d.rectangle((16, y, 168, y + 92), fill=SLATE, outline=GOLD)
    img.paste(shot.crop((20, 166, 84, 230)).resize((48, 48)), (22, y + 6))
    d.text((76, y + 6), "Kaelen", font=BODY, fill=PARCH)
    chip(d, 76, y + 28, "Assault", ASSAULT)
    bar(d, 22, y + 60, 140, 6, 1.0, HP)
    d.text((22, y + 70), "HP", font=MONO, fill=IRON_L)
    d.text((114, y + 70), "242", font=MONO, fill=PARCH)
    pips(d, 50, y + 72, 5, 5, size=6, gap=2)
    d.text((22, 612), "Q / E  member", font=SMALL, fill=IRON_L)

    # right: foe list only (UX-14)
    frame_a(d, (1104, 72, 1272, 641))
    d.text((1116, 82), "Hostiles", font=TITLE_S, fill=(200, 90, 80))
    d.text((1244, 85), "2", font=MONO, fill=IRON_L)
    foe = shot.crop((1044, 220, 1108, 284)).resize((40, 40))
    for k in range(2):
        y = 110 + k * 82
        sel = k == 0
        d.rectangle((1112, y, 1264, y + 74), fill=SLATE, outline=(170, 60, 50) if sel else IRON)
        img.paste(foe, (1118, y + 6))
        d.text((1164, y + 4), "Castellan", font=BODY, fill=PARCH)
        d.text((1164, y + 22), "Footman", font=SMALL, fill=IRON_L)
        bar(d, 1118, y + 52, 140, 5, 1.0, (150, 50, 45))
        d.text((1118, y + 59), "Move to Kaelen, B1", font=SMALL, fill=(200, 160, 120))

    # detail card for the selected foe (UX-14: the long forecast moves here, opens on select or hover)
    cx, cy = 760, 96
    frame_a(d, (cx, cy, cx + 320, cy + 196), fill=(16, 18, 25))
    img.paste(shot.crop((1044, 220, 1108, 284)).resize((44, 44)), (cx + 14, cy + 14))
    d.text((cx + 68, cy + 12), "Castellan Footman", font=TITLE_S, fill=GOLD)
    d.text((cx + 68, cy + 34), "Lv 2 Vanguard  ·  Poise 5/5", font=SMALL, fill=IRON_L)
    lines = [("B1", "moves toward Kaelen", PARCH),
             ("B2", "your Pommel Strike: 20 % to break", AMBER),
             ("B3", "your Pommel Strike: 33 % to break", AMBER),
             ("B4", "your Pommel Strike: breaks poise", AMBER),
             ("", "Trade: you deal ~29, take ~22", IRON_L)]
    for i, (b, t, c) in enumerate(lines):
        yy = cy + 70 + i * 22
        if b:
            d.text((cx + 14, yy), b, font=MONO_L, fill=GOLD)
        d.text((cx + 44, yy), t, font=SMALL, fill=c)
    d.text((cx + 14, cy + 176), "Click a foe to pin this card", font=SMALL, fill=IRON)

    # bottom bar (UX-17): round and phase left, message full width on two lines, Menu holds Restart and Step
    d.rectangle((0, 648, 1280, 720), fill=BASALT)
    d.line((0, 648, 1280, 648), fill=IRON)
    d.text((14, 658), "Round 1/8", font=MONO_L, fill=GOLD)
    d.text((14, 680), "Planning", font=SMALL, fill=IRON_L)
    msg = ["The curfew bell tolls and two Castellan Footmen close the alley.",
           "Strike while they wind up: a quick Pommel Strike breaks their guard."]
    for i, t in enumerate(msg):
        d.text((130, 658 + i * 22), t, font=BODY, fill=PARCH)
    d.rectangle((1180, 664, 1268, 700), fill=SLATE, outline=IRON)
    d.text((1204, 672), "Menu", font=BODY, fill=PARCH)
    return img


def label(img, text):
    out = Image.new("RGB", (img.width, img.height + 34), BASALT)
    out.paste(img, (0, 34))
    ImageDraw.Draw(out).text((14, 8), text, font=BODY, fill=PARCH)
    return out


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    m = mock(shot)
    m.save(out / "battle_layout_mock.png")
    a, b = label(shot, "Now"), label(m, "Proposed: UX-14 to UX-17")
    sheet = Image.new("RGB", (a.width, a.height * 2 + 12), VOID)
    sheet.paste(a, (0, 0))
    sheet.paste(b, (0, a.height + 12))
    sheet.save(out / "battle_layout_side_by_side.png")


if __name__ == "__main__":
    main()
