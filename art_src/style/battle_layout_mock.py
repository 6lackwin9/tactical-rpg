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


# take 2 (Tushar 2026-11-18: "remove as much text as you can"; blocks and AP as quick visuals; AP pips in the game's
# own style: blue square pips like the party plate's 10/10 row, poise as light-blue segments)
AP_ON, AP_OFF, POISE = (96, 150, 230), (34, 42, 58), (150, 190, 235)


def ap_pips(d, x, y, n, on, size=8, gap=2, spent=None):
    for i in range(n):
        c = AP_ON if i < on else AP_OFF
        if spent is not None and i < spent:
            c = (60, 92, 140)
        d.rectangle((x + i * (size + gap), y, x + i * (size + gap) + size - 1, y + size - 1), fill=c)
    return x + n * (size + gap)


def poise(d, x, y, n, on, w=14, h=4, gap=2):
    for i in range(n):
        d.rectangle((x + i * (w + gap), y, x + i * (w + gap) + w - 1, y + h - 1), fill=POISE if i < on else AP_OFF)


def glyph(d, kind, x, y, s=18, col=PARCH):
    if kind == "shield":
        d.polygon([(x, y), (x + s, y), (x + s, y + s * 0.55), (x + s / 2, y + s), (x, y + s * 0.55)], outline=col, width=2)
    elif kind == "hero":
        d.ellipse((x + s * 0.32, y, x + s * 0.68, y + s * 0.36), outline=col, width=2)
        d.arc((x + s * 0.1, y + s * 0.42, x + s * 0.9, y + s * 1.3), 180, 360, fill=col, width=2)
    elif kind == "auto":
        d.arc((x, y, x + s, y + s), 30, 320, fill=col, width=2)
        d.polygon([(x + s, y + s * 0.2), (x + s, y + s * 0.6), (x + s * 0.65, y + s * 0.42)], fill=col)
    elif kind == "clear":
        d.line((x + 3, y + 3, x + s - 3, y + s - 3), fill=col, width=2)
        d.line((x + s - 3, y + 3, x + 3, y + s - 3), fill=col, width=2)
    elif kind == "menu":
        for k in range(3):
            d.rectangle((x, y + 3 + k * 6, x + s, y + 4 + k * 6), fill=col)
    elif kind == "arrow":
        d.line((x, y + s / 2, x + s - 5, y + s / 2), fill=col, width=2)
        d.polygon([(x + s, y + s / 2), (x + s - 7, y + s / 2 - 5), (x + s - 7, y + s / 2 + 5)], fill=col)
    elif kind == "sword":
        d.line((x + 2, y + s - 2, x + s - 2, y + 2), fill=col, width=3)
        d.line((x + 3, y + s - 9, x + 9, y + s - 3), fill=col, width=2)
    elif kind == "check":
        d.line((x + 2, y + s / 2, x + s / 2 - 1, y + s - 3), fill=col, width=3)
        d.line((x + s / 2 - 1, y + s - 3, x + s - 1, y + 2), fill=col, width=3)


def intent_tile(d, x, y, col=ASSAULT, s=20):
    d.rectangle((x, y, x + s, y + s), fill=col, outline=(190, 110, 95))
    glyph(d, "sword", x + 2, y + 2, s - 4, PARCH)


def mock2(shot: Image.Image) -> Image.Image:
    img = Image.new("RGB", (1280, 720), VOID)
    d = ImageDraw.Draw(img)
    stage = shot.crop((291, 112, 899, 492)).resize((912, 570), Image.NEAREST)
    img.paste(stage, (184, 72))
    d.rectangle((184, 72, 1095, 641), outline=IRON, width=1)
    kae = shot.crop((20, 166, 84, 230))
    foe_p = shot.crop((1044, 220, 1108, 284))

    # top strip: who is planning (face), round track, six slots, AP row, icon group, one action
    d.rectangle((0, 0, 1280, 64), fill=BASALT)
    d.line((0, 64, 1280, 64), fill=IRON)
    img.paste(kae.resize((48, 48)), (10, 8))
    d.rectangle((10, 8, 57, 55), outline=GOLD)
    for r in range(8):                     # round track: 8 marks, the current one lit
        cx = 72 + r * 14
        d.ellipse((cx, 26, cx + 8, 34), fill=GOLD if r == 0 else (45, 50, 60))
    icons = [shot.crop((186, 12, 222, 44))] + [shot.crop((236 + i * 50, 12, 272 + i * 50, 44)) for i in range(4)] + [
        shot.crop((436, 12, 480, 44))]
    costs = [2, 2, 2, 2, 1, 0]
    x = 210
    for i, (ic, c) in enumerate(zip(icons, costs)):
        w = 56
        sel = i == 1
        d.rectangle((x, 4, x + w, 60), fill=SLATE, outline=GOLD if sel else IRON, width=2 if sel else 1)
        img.paste(ic.resize((40, 34)), (x + 8, 8))
        ap_pips(d, x + (w - (c * 10 - 2)) // 2, 48, c, c) if c else None
        x += w + 6
    # one AP row under the whole strip span: spent per block, the rest free
    ap_pips(d, x + 10, 22, 10, 10, size=10, gap=3, spent=9)
    gx = 900
    for i, (k, act) in enumerate((("shield", True), ("hero", True), ("auto", False), ("clear", False))):
        bx = gx + i * 46
        d.rectangle((bx, 12, bx + 40, 52), fill=SLATE, outline=IRON)
        glyph(d, k, bx + 11, 20, 18, PARCH)
        if act:
            d.polygon([(bx + 30, 44), (bx + 38, 44), (bx + 34, 49)], fill=IRON_L)
    d.rectangle((1108, 10, 1268, 54), fill=(60, 34, 10), outline=AMBER, width=2)
    glyph(d, "check", 1124, 22, 20, GOLD)
    d.text((1152, 21), "Confirm", font=BODY, fill=GOLD)
    # tooltip on the selected slot: the one place a name appears
    frame_a(d, (262, 68, 420, 96), fill=(16, 18, 25), top=False)
    d.text((272, 72), "Pommel Strike", font=SMALL, fill=GOLD)

    # left: party
    frame_a(d, (8, 72, 176, 641))
    y = 84
    d.rectangle((16, y, 168, y + 70), fill=SLATE, outline=GOLD)
    img.paste(kae.resize((48, 48)), (22, y + 6))
    intent_tile(d, 140, y + 6)
    d.text((76, y + 6), "Kaelen", font=BODY, fill=PARCH)
    bar(d, 76, y + 30, 86, 6, 1.0, HP)
    ap_pips(d, 76, y + 42, 10, 10, size=6, gap=2, spent=9)
    poise(d, 22, y + 60, 5, 5, w=26, h=4)
    # right: foes, name once, next move as arrow + target face
    frame_a(d, (1104, 72, 1272, 641))
    for k in range(2):
        y = 84 + k * 80
        sel = k == 0
        d.rectangle((1112, y, 1264, y + 70), fill=SLATE, outline=(170, 60, 50) if sel else IRON)
        img.paste(foe_p.resize((48, 48)), (1118, y + 6))
        d.text((1172, y + 4), "Footman", font=BODY, fill=PARCH)
        bar(d, 1172, y + 26, 86, 6, 1.0, (150, 50, 45))
        glyph(d, "arrow", 1172, y + 38, 20, (200, 160, 120))
        img.paste(kae.resize((20, 20)), (1198, y + 38))
        d.ellipse((1224, y + 42, 1236, y + 54), fill=SLATE, outline=GOLD)
        d.text((1227, y + 41), "1", font=MONO, fill=GOLD)
        poise(d, 1118, y + 60, 5, 5, w=26, h=4)

    # detail card: per slot, the skill icon and what it does to the foe's poise
    cx, cy = 800, 96
    frame_a(d, (cx, cy, cx + 280, cy + 186), fill=(16, 18, 25))
    img.paste(foe_p.resize((40, 40)), (cx + 14, cy + 14))
    d.text((cx + 62, cy + 14), "Castellan Footman", font=TITLE_S, fill=GOLD)
    poise(d, cx + 62, cy + 40, 5, 5, w=20, h=5)
    rows = [(icons[1], 5, 3), (icons[2], 3, 1), (icons[3], 1, 0)]
    for i, (ic, a, b) in enumerate(rows):
        yy = cy + 66 + i * 30
        d.ellipse((cx + 14, yy + 6, cx + 26, yy + 18), fill=SLATE, outline=IRON_L)
        d.text((cx + 17, yy + 5), str(i + 2), font=MONO, fill=PARCH)
        img.paste(ic.resize((28, 24)), (cx + 34, yy))
        poise(d, cx + 74, yy + 9, 5, b, w=20, h=6)
        for j in range(b, a):                  # the pips this hit takes, in amber
            d.rectangle((cx + 74 + j * 22, yy + 9, cx + 74 + j * 22 + 19, yy + 14), fill=AMBER)
        if b == 0:
            d.text((cx + 192, yy + 3), "BREAK", font=MONO_L, fill=AMBER)
    yy = cy + 160
    glyph(d, "sword", cx + 14, yy - 2, 16, AMBER)
    d.text((cx + 36, yy - 2), "29", font=MONO_L, fill=PARCH)
    glyph(d, "shield", cx + 80, yy - 2, 16, (200, 90, 80))
    d.text((cx + 102, yy - 2), "22", font=MONO_L, fill=PARCH)

    # bottom: the story line only, menu as an icon
    d.rectangle((0, 648, 1280, 720), fill=BASALT)
    d.line((0, 648, 1280, 648), fill=IRON)
    msg = ["The curfew bell tolls and two Castellan Footmen close the alley.",
           "Strike while they wind up: a quick Pommel Strike breaks their guard."]
    for i, t in enumerate(msg):
        d.text((24, 658 + i * 22), t, font=BODY, fill=PARCH)
    d.rectangle((1220, 662, 1264, 704), fill=SLATE, outline=IRON)
    glyph(d, "menu", 1232, 671, 20, PARCH)
    return img


# take 3 (Tushar 2026-11-18 21:05): icons centred, two 2-block moves, room for 4 AP pips per slot, the 10-pip AP row
# grouped into one pill per move, round as text
SK = ROOT / "games/emberwake/assets/skills"


def slot_icon(name: str, px: int) -> Image.Image:
    """A skill icon (I2 slate tile) or a kit intent glyph centred on a drawn slate tile, px square."""
    if name == "arrow":
        tile = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
        td = ImageDraw.Draw(tile)
        td.rounded_rectangle((0, 0, 63, 63), radius=8, fill=(48, 58, 70), outline=(76, 96, 107), width=2)
        td.line((14, 32, 42, 32), fill=PARCH, width=6)
        td.polygon([(52, 32), (38, 20), (38, 44)], fill=PARCH)
        return tile.resize((px, px), Image.LANCZOS)
    if name.startswith("intent:"):
        g = Image.open(ROOT / "kits/rpg/assets/ui/intents" / (name[7:] + ".png")).convert("RGBA")
        tile = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
        ImageDraw.Draw(tile).rounded_rectangle((0, 0, 63, 63), radius=8, fill=(48, 58, 70), outline=(76, 96, 107), width=2)
        g = g.resize((40, 40), Image.NEAREST)
        tile.paste(g, (12, 12), g)
    else:
        tile = Image.open(SK / (name + ".png")).convert("RGBA")
    return tile.resize((px, px), Image.LANCZOS)


def tactic_badge(d, x, y):
    """Small gold pennant: marks a button as a tactic (stance), not a unit pick."""
    d.line((x, y, x, y + 14), fill=IRON_L, width=2)
    d.polygon([(x + 1, y), (x + 12, y + 3), (x + 1, y + 7)], fill=GOLD)


def control_group(img, d, gx=896):
    party = Image.open(ROOT / "kits/rpg/assets/ui/hud/party.png").convert("RGBA")
    hero = party.crop((9, 4, 23, 28)).resize((17, 29), Image.NEAREST)
    party = party.resize((30, 30), Image.NEAREST)
    d.rectangle((gx - 4, 0, 1100, 63), fill=BASALT)
    btns = [("party", True), ("hero", True), ("auto", False), ("clear", False)]
    for i, (k, tactic) in enumerate(btns):
        bx = gx + i * 50
        d.rectangle((bx, 10, bx + 44, 54), fill=SLATE, outline=IRON)
        if k == "party":
            img.paste(party, (bx + 5, 17), party)
        elif k == "hero":
            img.paste(hero, (bx + 11, 15), hero)
        elif k == "auto":                      # play + AUTO tag (gacha/SRPG convention); >> stays free for battle speed
            d.polygon([(bx + 15, 14), (bx + 15, 36), (bx + 32, 25)], fill=PARCH)
            d.text((bx + 22 - d.textlength("AUTO", font=SMALL) / 2, 37), "AUTO", font=SMALL, fill=GOLD)
        else:
            glyph(d, "clear", bx + 12, 22, 20, PARCH)
        if tactic:
            tactic_badge(d, bx + 30, 13)
            d.polygon([(bx + 32, 46), (bx + 40, 46), (bx + 36, 51)], fill=IRON_L)


def mock3(shot: Image.Image) -> Image.Image:
    img = mock2(shot)
    d = ImageDraw.Draw(img)
    stage = shot.crop((291, 112, 899, 492)).resize((912, 570), Image.NEAREST)
    d.rectangle((184, 65, 1095, 71), fill=VOID)
    img.paste(stage.crop((0, 0, 912, 40)), (184, 72))       # remove take 2's tooltip
    d.rectangle((184, 72, 1095, 641), outline=IRON, width=1)
    d.rectangle((60, 0, 896, 63), fill=BASALT)
    d.line((0, 64, 1280, 64), fill=IRON)
    d.text((70, 12), "Round 1 of 8", font=TITLE_S, fill=GOLD)
    d.text((70, 36), "Planning", font=SMALL, fill=IRON_L)
    # plan: (icon, blocks, AP, colour of its pill)
    plan = [("arrow", 1, 2, (90, 120, 150)), ("pneumatic_hammer_strike", 2, 4, (160, 80, 60)),
            ("pommel_strike", 1, 2, (160, 80, 60)), ("shield_wall", 2, 2, (110, 120, 90))]
    W, G = 54, 6
    x = 196
    sel = 1
    for i, (ic, blocks, ap, col) in enumerate(plan):
        w = blocks * W + (blocks - 1) * G
        on = i == sel
        d.rectangle((x, 4, x + w, 60), fill=SLATE, outline=GOLD if on else IRON, width=2 if on else 1)
        if blocks > 1:                       # faint seam marks the block boundary inside a long move
            for b in range(1, blocks):
                sx = x + b * (W + G) - G // 2
                for yy in range(8, 56, 4):
                    d.point((sx, yy), fill=IRON)
        t = slot_icon(ic, 36)
        img.paste(t, (x + (w - 36) // 2, 7), t)
        pw = 4 * 10 - 2                       # room for 4 pips, cost lit from the left, centred
        ap_pips(d, x + (w - pw) // 2, 48, 4, ap, size=8, gap=2) if ap else None
        x += w + G
    # AP row: one pill per planned move, so each move is visible as a block of pips
    ax, ay = x + 18, 24
    d.text((ax, 6), "AP", font=MONO, fill=IRON_L)
    for i, (ic, blocks, ap, col) in enumerate(plan):
        pw = ap * 12 - 2
        d.rounded_rectangle((ax - 3, ay - 3, ax + pw + 2, ay + 12), radius=6, outline=GOLD if i == sel else col, width=1)
        ap_pips(d, ax, ay, ap, ap, size=10, gap=2)
        ax += pw + 10
    control_group(img, d)
    return img


# take 4 (Tushar 2026-11-18 21:10): no AP pills; a round AP tracker (max, start, planned in another colour);
# block span obvious via a block ruler under the slots; per-move AP not capped at 4
PLANNED = (217, 150, 60)


def move_pips(d, x, w, y, ap):
    """Pips centred in the tile; a tile holds 4 pips per block it spans; past that, one pip and a number."""
    cap = 4 * max(1, round((w + 6) / 60))
    if ap <= cap:
        pw = ap * 10 - 2
        ap_pips(d, x + (w - pw) // 2, y, ap, ap, size=8, gap=2)
    else:
        ap_pips(d, x + w // 2 - 14, y, 1, 1, size=8, gap=2)
        d.text((x + w // 2 - 2, y - 4), str(ap), font=MONO_L, fill=AP_ON)


def mock4(shot: Image.Image) -> Image.Image:
    img = mock3(shot)
    d = ImageDraw.Draw(img)
    d.rectangle((196, 0, 896, 63), fill=BASALT)
    plan = [("arrow", 1, 2), ("pneumatic_hammer_strike", 3, 5), ("shield_wall", 2, 2)]
    W, G = 54, 6
    x0 = 196
    x = x0
    sel = 1
    for i, (ic, blocks, ap) in enumerate(plan):
        w = blocks * W + (blocks - 1) * G
        on = i == sel
        d.rectangle((x, 3, x + w, 50), fill=SLATE, outline=GOLD if on else IRON, width=2 if on else 1)
        t = slot_icon(ic, 32)
        img.paste(t, (x + (w - 32) // 2, 6), t)
        move_pips(d, x, w, 40, ap)
        # block ruler: one notch per block, notches of one move joined, a gap between moves
        for b in range(blocks):
            bx = x + b * (W + G)
            d.rectangle((bx, 55, bx + W, 60), fill=GOLD if on else IRON_L)
            if b < blocks - 1:
                d.rectangle((bx + W, 57, bx + W + G, 58), fill=GOLD if on else IRON_L)
        x += w + G
    # round AP tracker: max cap as outlined slots, start-of-round AP filled blue, the plan's AP over it in amber
    MAX, START, USED = 12, 10, 9
    ax, ay, sz, gp = x + 20, 18, 12, 3
    for i in range(MAX):
        px = ax + i * (sz + gp)
        if i < USED:
            d.rectangle((px, ay, px + sz - 1, ay + sz - 1), fill=PLANNED)
        elif i < START:
            d.rectangle((px, ay, px + sz - 1, ay + sz - 1), fill=AP_ON)
        else:
            d.rectangle((px, ay, px + sz - 1, ay + sz - 1), outline=IRON)
    d.text((ax, ay + 18), "9 / 10", font=MONO_L, fill=PARCH)
    d.text((ax + 52, ay + 19), "max 12", font=SMALL, fill=IRON_L)
    return img


# take 5 (Tushar 2026-11-18 21:14): AP tracker without the max outlines and with an AP label; the foe card split
# into the foe's own move and what your plan does to its poise
def poise_glyph(d, x, y, s=16, col=POISE):
    """Poise: a shield outline with a crack."""
    d.polygon([(x, y), (x + s, y), (x + s, y + s * 0.55), (x + s / 2, y + s), (x, y + s * 0.55)], outline=col, width=2)
    d.line((x + s * 0.55, y + 2, x + s * 0.4, y + s * 0.45, x + s * 0.6, y + s * 0.7), fill=col, width=1)


def mock5(shot: Image.Image) -> Image.Image:
    img = mock4(shot)
    d = ImageDraw.Draw(img)
    stage = shot.crop((291, 112, 899, 492)).resize((912, 570), Image.NEAREST)
    # AP tracker: label, the start-of-round AP as pips, the plan's share in amber, numbers
    d.rectangle((560, 0, 890, 63), fill=BASALT)
    ax, ay = 576, 14
    d.text((ax, ay - 2), "AP", font=TITLE_S, fill=GOLD)
    START, USED = 10, 9
    for i in range(START):
        px = ax + 34 + i * 15
        d.rectangle((px, ay, px + 11, ay + 11), fill=PLANNED if i < USED else AP_ON)
    d.text((ax + 34, ay + 20), "9 planned", font=SMALL, fill=PLANNED)
    d.text((ax + 120, ay + 20), "1 left", font=SMALL, fill=AP_ON)
    # foe card: restore the board under the old card, then draw the new one
    img.paste(stage.crop((612, 18, 900, 262)), (796, 90))
    kae = shot.crop((20, 166, 84, 230))
    foe_p = shot.crop((1044, 220, 1108, 284))
    cx, cy = 796, 96
    frame_a(d, (cx, cy, cx + 288, cy + 214), fill=(16, 18, 25))
    img.paste(foe_p.resize((40, 40)), (cx + 14, cy + 14))
    d.text((cx + 62, cy + 12), "Castellan Footman", font=TITLE_S, fill=GOLD)
    bar(d, cx + 62, cy + 36, 120, 6, 1.0, (150, 50, 45))
    poise_glyph(d, cx + 192, cy + 31, 14)
    poise(d, cx + 212, cy + 36, 5, 5, w=12, h=6)
    # section 1: its own move (red rule)
    y = cy + 64
    d.rectangle((cx + 12, y, cx + 15, y + 26), fill=(170, 60, 50))
    img.paste(foe_p.resize((22, 22)), (cx + 24, y + 2))
    glyph(d, "arrow", cx + 52, y + 3, 20, (200, 160, 120))
    img.paste(kae.resize((22, 22)), (cx + 78, y + 2))
    d.ellipse((cx + 108, y + 6, cx + 122, y + 20), fill=SLATE, outline=GOLD)
    d.text((cx + 112, y + 5), "1", font=MONO, fill=GOLD)
    d.text((cx + 132, y + 6), "moves on you", font=SMALL, fill=IRON_L)
    # section 2: your plan against it (gold rule): your face, your move, the poise it strips
    y = cy + 104
    d.line((cx + 12, y - 8, cx + 276, y - 8), fill=IRON)
    d.rectangle((cx + 12, y, cx + 15, y + 74), fill=GOLD)
    img.paste(kae.resize((22, 22)), (cx + 24, y + 2))
    glyph(d, "arrow", cx + 52, y + 3, 20, GOLD)
    img.paste(foe_p.resize((22, 22)), (cx + 78, y + 2))
    d.text((cx + 108, y + 5), "your plan on it", font=SMALL, fill=GOLD)
    y += 32
    t = slot_icon("pneumatic_hammer_strike", 26)
    img.paste(t, (cx + 24, y), t)
    d.text((cx + 56, y + 5), "2-4", font=MONO_L, fill=PARCH)
    poise_glyph(d, cx + 92, y + 5, 14)
    for j in range(5):                      # all five poise pips stripped: amber = what this move takes
        d.rectangle((cx + 112 + j * 16, y + 9, cx + 112 + j * 16 + 13, y + 15), fill=AMBER)
    d.text((cx + 196, y + 4), "BREAK", font=MONO_L, fill=AMBER)
    yy = cy + 186
    glyph(d, "sword", cx + 24, yy - 2, 16, AMBER)
    d.text((cx + 46, yy - 2), "34 to it", font=SMALL, fill=PARCH)
    glyph(d, "shield", cx + 130, yy - 2, 16, (200, 90, 80))
    d.text((cx + 152, yy - 2), "22 to you", font=SMALL, fill=PARCH)
    return img


# take 6 (Tushar 2026-11-18 21:18): foe card keeps only "your plan on it"; HP vs poise icons made consistent;
# the foe list shows when it acts as a mini block ruler instead of a number
def heart(d, x, y, s=14, col=(200, 80, 70)):
    d.ellipse((x, y, x + s / 2, y + s / 2), fill=col)
    d.ellipse((x + s / 2, y, x + s, y + s / 2), fill=col)
    d.polygon([(x, y + s / 4), (x + s, y + s / 4), (x + s / 2, y + s)], fill=col)


def mini_ruler(d, x, y, act, n=6, w=10, g=2):
    for b in range(n):
        d.rectangle((x + b * (w + g), y, x + b * (w + g) + w - 1, y + 4), fill=(200, 90, 80) if b in act else AP_OFF)


def mock6(shot: Image.Image) -> Image.Image:
    img = mock5(shot)
    d = ImageDraw.Draw(img)
    stage = shot.crop((291, 112, 899, 492)).resize((912, 570), Image.NEAREST)
    img.paste(stage.crop((612, 18, 904, 262)), (796, 90))
    kae = shot.crop((20, 166, 84, 230))
    foe_p = shot.crop((1044, 220, 1108, 284))
    # foe list: next move = arrow + target face, and when it acts as a mini block ruler (block 1 lit)
    for k in range(2):
        y = 84 + k * 80
        d.rectangle((1170, y + 36, 1262, y + 58), fill=SLATE)
        glyph(d, "arrow", 1172, y + 36, 18, (200, 160, 120))
        img.paste(kae.resize((18, 18)), (1194, y + 36))
        mini_ruler(d, 1216, y + 43, {0}, w=6, g=2)
    cx, cy = 796, 96
    frame_a(d, (cx, cy, cx + 288, cy + 176), fill=(16, 18, 25))
    img.paste(foe_p.resize((40, 40)), (cx + 14, cy + 14))
    d.text((cx + 62, cy + 12), "Castellan Footman", font=TITLE_S, fill=GOLD)
    heart(d, cx + 62, cy + 33, 11)
    bar(d, cx + 78, cy + 36, 100, 6, 1.0, (150, 50, 45))
    poise_glyph(d, cx + 192, cy + 31, 14)
    poise(d, cx + 212, cy + 36, 5, 5, w=12, h=6)
    y = cy + 66
    d.rectangle((cx + 12, y, cx + 15, y + 96), fill=GOLD)
    img.paste(kae.resize((22, 22)), (cx + 24, y + 2))
    glyph(d, "arrow", cx + 52, y + 3, 20, GOLD)
    img.paste(foe_p.resize((22, 22)), (cx + 78, y + 2))
    d.text((cx + 108, y + 5), "your plan on it", font=SMALL, fill=GOLD)
    y += 32
    t = slot_icon("pneumatic_hammer_strike", 26)
    img.paste(t, (cx + 24, y), t)
    for b in range(6):
        c = GOLD if b in (1, 2, 3) else AP_OFF
        d.rectangle((cx + 56 + b * 9, y + 11, cx + 56 + b * 9 + 7, y + 15), fill=c)
    poise_glyph(d, cx + 116, y + 5, 14)
    for j in range(5):
        d.rectangle((cx + 136 + j * 13, y + 9, cx + 136 + j * 13 + 10, y + 15), fill=AMBER)
    d.text((cx + 204, y + 4), "BREAK", font=MONO_L, fill=AMBER)
    # HP trade, both as hearts with who-hits-whom faces
    y += 34
    img.paste(kae.resize((18, 18)), (cx + 24, y))
    glyph(d, "arrow", cx + 44, y, 16, GOLD)
    img.paste(foe_p.resize((18, 18)), (cx + 62, y))
    heart(d, cx + 86, y + 3, 12)
    d.text((cx + 102, y), "-34", font=MONO_L, fill=PARCH)
    img.paste(foe_p.resize((18, 18)), (cx + 150, y))
    glyph(d, "arrow", cx + 170, y, 16, (200, 160, 120))
    img.paste(kae.resize((18, 18)), (cx + 188, y))
    heart(d, cx + 212, y + 3, 12)
    d.text((cx + 228, y), "-22", font=MONO_L, fill=PARCH)
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
    m2 = mock2(shot)
    m2.save(out / "battle_layout_take2.png")
    a, b = label(m, "Take 1"), label(m2, "Take 2: fewer words; slots, AP, round and moves as pips and icons")
    sheet = Image.new("RGB", (a.width, a.height * 2 + 12), VOID)
    sheet.paste(a, (0, 0))
    sheet.paste(b, (0, a.height + 12))
    sheet.save(out / "battle_layout_take1_vs_take2.png")
    m3 = mock3(shot)
    m3.save(out / "battle_layout_take3.png")
    a, b = label(m2, "Take 2"), label(m3, "Take 3: centred icons, 2-block moves, 4 pip slots, AP pills per move, round as text")
    sheet = Image.new("RGB", (a.width, a.height * 2 + 12), VOID)
    sheet.paste(a, (0, 0))
    sheet.paste(b, (0, a.height + 12))
    sheet.save(out / "battle_layout_take2_vs_take3.png")
    m4 = mock4(shot)
    m4.save(out / "battle_layout_take4.png")
    a, b = label(m3, "Take 3"), label(m4, "Take 4: round AP tracker, block ruler under the slots, a 3-block move, AP past 4")
    sheet = Image.new("RGB", (a.width, a.height * 2 + 12), VOID)
    sheet.paste(a, (0, 0))
    sheet.paste(b, (0, a.height + 12))
    sheet.save(out / "battle_layout_take3_vs_take4.png")
    m5 = mock5(shot)
    m5.save(out / "battle_layout_take5.png")
    m6 = mock6(shot)
    m6.save(out / "battle_layout_take6.png")
    strip = m6.crop((0, 0, 1280, 66)).resize((2560, 132), Image.NEAREST)
    card = m6.crop((790, 90, 1090, 280)).resize((900, 570), Image.NEAREST)
    foes = m6.crop((1104, 72, 1272, 250)).resize((504, 534), Image.NEAREST)
    sh = Image.new("RGB", (2560, 1440 + 20 + 132 + 20 + 570), VOID)
    sh.paste(m6.resize((2560, 1440), Image.NEAREST), (0, 0))
    sh.paste(strip, (0, 1460))
    sh.paste(card, (0, 1612))
    sh.paste(foes, (940, 1612))
    sh.save(out / "battle_layout_take6_sheet.png")


if __name__ == "__main__":
    main()
