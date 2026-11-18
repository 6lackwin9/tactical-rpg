"""AI-21 screen mocks after battle (2026-11-18 night, [PROVISIONAL] for Tushar's review): victory and defeat report
(UX-19), story stage (UX-12, UX-13), character stats and gear (UX-20..24), shop (UX-32), notice board (UX-31), save
and load (UX-10). All at 1280x720, 1:1, on the shared blocks UX-01..07: the current ornate frame for main screens
("Main current"), frame A for popups, the shared row (32 px icon, name, grey sub-line, mono numbers on the right), four
type roles, colour by meaning (gold titles and selection, amber action, oxblood danger).

Usage: python3 art_src/style/screens_ai21_mock.py <playtest_dir> <out_dir>
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageEnhance, ImageOps

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402

ROOT = B.ROOT
GA = ROOT / "games/emberwake/assets"
KA = ROOT / "kits/rpg/assets"
FILL = (13, 14, 20)
AMBER_BG = (60, 36, 14)
MOSS = (110, 140, 70)
OX = (153, 27, 27)


# ---------------------------------------------------------------- shared blocks
class Shell:
    """The current ornate frame (cut from the Settings screen: corner star and edge line), for main screens."""

    def __init__(self, settings_shot):
        s = Image.open(settings_shot).convert("RGB")
        c = s.crop((40, 12, 96, 54)).convert("RGBA")       # the frame lines cross at (22, 23) of this crop
        px = c.load()
        for yy in range(c.height):                          # keep the star and lines, drop the dark ground
            for xx in range(c.width):
                r, g, b, _ = px[xx, yy]
                if 0.3 * r + 0.59 * g + 0.11 * b < 34:
                    px[xx, yy] = (0, 0, 0, 0)
        self.corner = c
        self.top = s.crop((600, 30, 601, 44))             # line at row 4
        self.left = s.crop((56, 360, 70, 361))            # line at column 6

    def draw(self, img, box, title=None, close=True):
        x0, y0, x1, y1 = box
        d = ImageDraw.Draw(img)
        d.rectangle(box, fill=FILL)
        top = self.top.resize((x1 - x0, 14))
        img.paste(top, (x0, y0 - 4)); img.paste(ImageOps.flip(top), (x0, y1 - 9))
        left = self.left.resize((14, y1 - y0))
        img.paste(left, (x0 - 6, y0)); img.paste(ImageOps.mirror(left), (x1 - 7, y0))
        c = self.corner
        cw, ch = c.size
        for im2, px_, py_ in ((c, x0 - 22, y0 - 23), (ImageOps.mirror(c), x1 - (cw - 1 - 22), y0 - 23),
                              (ImageOps.flip(c), x0 - 22, y1 - (ch - 1 - 23)),
                              (ImageOps.flip(ImageOps.mirror(c)), x1 - (cw - 1 - 22), y1 - (ch - 1 - 23))):
            img.paste(im2, (px_, py_), im2)
        if title:
            d.text((x0 + 26, y0 + 18), title, font=B.TITLE, fill=B.GOLD)
        if close:
            d.rectangle((x1 - 60, y0 + 16, x1 - 26, y0 + 50), outline=B.IRON_L)
            d.line((x1 - 50, y0 + 26, x1 - 36, y0 + 40), fill=B.PARCH, width=1)
            d.line((x1 - 50, y0 + 40, x1 - 36, y0 + 26), fill=B.PARCH, width=1)
        return d


def button(d, x, y, text, primary=False, w=None, h=32):
    w = w or int(d.textlength(text, font=B.BODY)) + 36
    if primary:
        d.rectangle((x, y, x + w, y + h), fill=AMBER_BG, outline=B.AMBER, width=2)
        col = (250, 200, 120)
    else:
        d.rectangle((x, y, x + w, y + h), fill=(20, 22, 30), outline=B.IRON)
        col = B.PARCH
    tw = d.textlength(text, font=B.BODY)
    d.text((x + (w - tw) / 2, y + (h - 18) / 2), text, font=B.BODY, fill=col)
    return w


def hint(d, x, y, text):
    d.text((x, y), text, font=B.SMALL, fill=B.IRON_L)


def icon(rel, px):
    for base in (GA, KA):
        p = base / rel
        if p.exists():
            im = Image.open(p).convert("RGBA")
            return im.resize((px, px), Image.BOX if im.width % px == 0 else Image.LANCZOS)
    raise FileNotFoundError(rel)


def tile_bg(d, x, y, px, on=False):
    d.rectangle((x - 2, y - 2, x + px + 1, y + px + 1), fill=(24, 27, 36), outline=B.GOLD if on else B.IRON)


def row(img, d, x, y, w, ic, name, sub, right=None, right2=None, on=False, h=44):
    """The shared list row (UX-03): 32 px icon, name, one grey sub-line, mono numbers right-aligned."""
    if on:
        d.rectangle((x, y, x + w, y + h), fill=(36, 30, 22), outline=B.GOLD)
    if ic is not None:
        tile_bg(d, x + 8, y + 6, 32)
        img.paste(ic, (x + 8, y + 6), ic)
    tx = x + 50 if ic is not None else x + 10
    d.text((tx, y + 4), name, font=B.BODY, fill=B.GOLD if on else B.PARCH)
    if sub:
        d.text((tx, y + 23), sub, font=B.SMALL, fill=B.IRON_L)
    if right:
        tw = d.textlength(right, font=B.MONO_L)
        d.text((x + w - 10 - tw, y + 6), right, font=B.MONO_L, fill=B.PARCH)
    if right2:
        tw = d.textlength(right2, font=B.SMALL)
        d.text((x + w - 10 - tw, y + 24), right2, font=B.SMALL, fill=B.IRON_L)


def face(name, px):
    for sub in ("heroes", "npcs", "common"):
        p = GA / "portraits" / sub / f"{name}.png"
        if p.exists():
            return Image.open(p).convert("RGB").resize((px, px), Image.LANCZOS)
    raise FileNotFoundError(name)


def framed(img, d, im, x, y, col=B.BRONZE):
    d.rectangle((x - 2, y - 2, x + im.width + 1, y + im.height + 1), outline=col, width=2)
    img.paste(im, (x, y))


def dim(im, k=0.4):
    return ImageEnhance.Brightness(im).enhance(k)


def xp_bar(d, x, y, w, before, gain, need):
    d.rectangle((x, y, x + w, y + 6), fill=(14, 16, 22))
    a = round(w * min(before, need) / need)
    b = round(w * min(before + gain, need) / need)
    d.rectangle((x, y, x + a, y + 6), fill=(150, 120, 60))
    d.rectangle((x + a, y, x + b, y + 6), fill=B.AMBER)


def chip(d, x, y, text, col, fill=None):
    w = d.textlength(text, font=B.SMALL) + 12
    d.rounded_rectangle((x, y, x + w, y + 18), radius=3, fill=fill, outline=col)
    d.text((x + 6, y + 1), text, font=B.SMALL, fill=col if fill is None else B.PARCH)
    return w


def hud_bar(shot):
    return Image.open(shot).convert("RGB").crop((0, 0, 1280, 40))


# ---------------------------------------------------------------- UX-19 report
HEROES = [("kaelen", "Kaelen", "Vanguard", 2, 46, 46, 110, True, None),
          ("vesper", "Vesper", "Rogue", 2, 30, 46, 110, False, "Wounded"),
          ("maren", "Maren", "Priest", 2, 70, 46, 110, False, None),
          ("hollis", "Hollis", "Ranger", 1, 50, 46, 64, True, None)]
LOOT = [("consumable/smelling_salts.png", "Smelling Salts", 2), ("material/sinew_braid.png", "Sinew Braid", 1),
        ("gauntlet/riveted_gauntlets.png", "Riveted Gauntlets", 1)]


def report(battle_png, shell, victory=True):
    img = dim(Image.open(battle_png).convert("RGB"), 0.35)
    d = ImageDraw.Draw(img)
    band = (110, 80, 1170, 150)
    d.rectangle(band, fill=(8, 9, 12))
    d.line((110, 80, 1170, 80), fill=B.GOLD if victory else OX); d.line((110, 150, 1170, 150), fill=B.GOLD if victory else OX)
    big = B.ImageFont.truetype(str(B.F / "AlegreyaSC-Bold.ttf"), 40)
    t = "Victory" if victory else "Driven off"
    tw = d.textlength(t, font=big)
    d.text((640 - tw / 2, 84), t, font=big, fill=B.GOLD if victory else (200, 70, 60))
    line = "Both footmen fled the alley." if victory else "The Watch holds the alley. The party falls back to the Undercroft."
    lw = d.textlength(line, font=B.BODY)
    d.text((640 - lw / 2, 128), line, font=B.BODY, fill=B.PARCH)
    x0, y0, x1, y1 = 320, 180, 960, 660
    shell.draw(img, (x0, y0, x1, y1), "Report" if victory else "Aftermath", close=False)
    y = y0 + 64
    for key, name, cls, lv, before, gain, need, up, mark in HEROES:
        framed(img, d, face(key, 40), x0 + 28, y)
        d.text((x0 + 80, y - 2), name, font=B.BODY, fill=B.PARCH)
        d.text((x0 + 80, y + 18), f"{cls}  Lv {lv}", font=B.SMALL, fill=B.IRON_L)
        if victory:
            xp_bar(d, x0 + 220, y + 8, 220, before, gain, need)
            d.text((x0 + 452, y + 2), f"+{gain} XP", font=B.MONO_L, fill=B.PARCH)
            if up:
                chip(d, x0 + 530, y + 2, "Level up", B.AMBER)
        else:
            if key in ("kaelen", "vesper"):
                chip(d, x0 + 220, y + 6, "Fell", (200, 70, 60))
                chip(d, x0 + 272, y + 6, "Wounded: needs the Slab", (200, 70, 60))
            else:
                d.text((x0 + 220, y + 8), "Walked out", font=B.SMALL, fill=B.IRON_L)
        if victory and mark:
            chip(d, x0 + 530, y + 2, mark, (200, 70, 60))
        y += 56
    d.line((x0 + 28, y + 4, x1 - 28, y + 4), fill=B.IRON)
    y += 16
    if victory:
        d.text((x0 + 28, y), "Spoils", font=B.BODY, fill=B.GOLD)
        x = x0 + 28
        y += 26
        for rel, name, n in LOOT:
            tile_bg(d, x, y, 48)
            ic = icon("items/" + rel, 48) if (GA / "items" / rel).exists() else icon("items/" + rel, 48)
            img.paste(ic, (x, y), ic)
            d.text((x + 34, y + 32), f"{n}", font=B.MONO, fill=B.PARCH)
            x += 64
        tile_bg(d, x, y, 48)
        sc = icon("ui/hud/gold.png", 32)
        img.paste(sc, (x + 8, y + 2), sc)
        d.text((x + 6, y + 32), "+40", font=B.MONO, fill=B.AMBER)
        hint(d, x0 + 28, y + 62, "Hover a tile for the item")
    else:
        d.text((x0 + 28, y), "Lost", font=B.BODY, fill=(200, 70, 60))
        d.text((x0 + 28, y + 26), "12 Scrap dropped in the run.  Tithing: 1 day passes.", font=B.SMALL, fill=B.PARCH)
        d.text((x0 + 28, y + 46), "Kept: everything you carried, and the XP from the fight (+20 each).", font=B.SMALL, fill=B.IRON_L)
        d.text((x0 + 28, y + 70), "(placeholder wording: the defeat rules are Design's)", font=B.SMALL, fill=B.IRON)
    if victory:
        button(d, x1 - 28 - 140, y1 - 56, "Continue", True, w=140)
        hint(d, x0 + 28, y1 - 48, "Enter continue")
    else:
        button(d, x1 - 28 - 140, y1 - 56, "Retry battle", True, w=140)
        button(d, x0 + 28, y1 - 56, "Back to the Undercroft", w=210)
    return img


# ---------------------------------------------------------------- UX-12, UX-13 story stage
def story(speaking=True):
    bg = Image.open(GA / "backgrounds/story/vn_bridge_stage.png").convert("RGB").resize((1280, 720))
    img = dim(bg, 0.8 if speaking else 1.0)
    d = ImageDraw.Draw(img)
    x0, y0, x1, y1 = 40, 560, 1240, 700
    B.frame_a(d, (x0, y0, x1, y1), fill=(14, 15, 21))
    if speaking:
        p = face("vesper", 128)
        framed(img, d, p, x0 + 20, y0 - 64, col=B.BRONZE)
        tx = x0 + 172
        d.text((tx, y0 + 14), "Vesper", font=B.TITLE_S, fill=B.GOLD)
        lines = ["Two of them in the cart, and the Watch didn't even bother with a lock on the axle.",
                 "Cut the chain and we are gone before the bell finishes."]
    else:
        tx = x0 + 28
        lines = ["The canal bridge. An iron mortuary cart stands on the span, its shutters chained,",
                 "and two prisoners rattle inside."]
    for i, l in enumerate(lines):
        d.text((tx, y0 + (42 if speaking else 24) + i * 24), l, font=B.BODY, fill=B.PARCH)
    d.polygon([(x1 - 34, y1 - 28), (x1 - 22, y1 - 28), (x1 - 28, y1 - 20)], fill=B.AMBER)    # continue mark (pulses)
    d.text((1180, 16), "Skip", font=B.SMALL, fill=B.IRON_L)
    d.line((1180, 33, 1206, 33), fill=B.IRON)
    return img


# ---------------------------------------------------------------- UX-20..24 character
ROSTER = ["kaelen", "vesper", "maren", "hollis", "dunstan", "elyse"]


def char_left(img, d, x0, y0):
    """The hero column: the member row, the portrait at 2x, name, class, level, XP and HP."""
    x = x0 + 26
    for i, k in enumerate(ROSTER):
        f = face(k, 36)
        if i >= 4:
            f = dim(f, 0.45)
        framed(img, d, f, x + i * 42, y0 + 70, col=B.GOLD if i == 0 else (B.BRONZE if i < 4 else B.IRON))
    hint(d, x, y0 + 112, "Q / E member   (4 in the party)")
    p = Image.open(GA / "portraits/heroes/kaelen.png").convert("RGB").resize((192, 192), Image.LANCZOS)
    framed(img, d, p, x + 28, y0 + 140)
    d.text((x, y0 + 344), "Kaelen", font=B.TITLE, fill=B.GOLD)
    d.text((x, y0 + 372), "Vanguard  Level 2", font=B.BODY, fill=B.PARCH)
    xp_bar(d, x, y0 + 400, 236, 46, 0, 110)
    d.text((x, y0 + 410), "XP 46 / 110", font=B.SMALL, fill=B.IRON_L)
    B.bar(d, x, y0 + 438, 236, 6, 1.0, MOSS)
    d.text((x, y0 + 448), "HP 242 / 242", font=B.SMALL, fill=B.IRON_L)
    chip(d, x, y0 + 474, "In party", B.GOLD)
    d.text((x + 80, y0 + 475), "Passive: Castellan's Penance", font=B.SMALL, fill=B.IRON_L)


def tabs(d, x, y, names, on, badge=None):
    for i, n in enumerate(names):
        w = int(d.textlength(n, font=B.BODY)) + 34
        if i == on:
            d.rectangle((x, y, x + w, y + 32), fill=(36, 30, 22), outline=B.GOLD)
            d.text((x + 17, y + 6), n, font=B.BODY, fill=B.GOLD)
        else:
            d.rectangle((x, y, x + w, y + 32), fill=(20, 22, 30), outline=B.IRON)
            d.text((x + 17, y + 6), n, font=B.BODY, fill=B.PARCH)
        if badge and badge[0] == i:
            d.ellipse((x + w - 10, y - 8, x + w + 8, y + 10), fill=B.AMBER)
            d.text((x + w - 4, y - 8), str(badge[1]), font=B.MONO, fill=(20, 12, 4))
        x += w + 8


ATTR = [("STR", 15, 2), ("DEX", 7, 0), ("INT", 6, 0), ("CONS", 19, 2), ("PERC", 10, 1), ("KNOW", 10, 0)]
DERIVED = [("Offence", [("Melee power", "43"), ("Ranged power", "11"), ("Crit", "7%")]),
           ("Defence", [("Max HP", "242"), ("Armor", "22"), ("Debuff resist", "15%"), ("Poise", "71"), ("Composure", "39")]),
           ("Tempo", [("Initiative", "15"), ("AP", "10"), ("Tiles per move", "3")])]


def character(shell, tab="stats"):
    img = Image.new("RGB", (1280, 720), B.VOID)
    box = (40, 30, 1240, 690)
    d = shell.draw(img, box, "Character")
    x0, y0 = box[0], box[1]
    char_left(img, d, x0, y0)
    d.line((x0 + 300, y0 + 70, x0 + 300, y0 + 560), fill=B.IRON)
    rx = x0 + 324
    tabs(d, rx, y0 + 70, ["Stats", "Level up", "Skills", "Gear"], {"stats": 0, "gear": 3}[tab], badge=(1, 5))
    if tab == "stats":
        d.text((rx, y0 + 124), "Attributes", font=B.BODY, fill=B.GOLD)
        for i, (k, v, g) in enumerate(ATTR):
            cx, cy = rx + (i % 3) * 150, y0 + 152 + (i // 3) * 74
            d.rectangle((cx, cy, cx + 138, cy + 64), fill=(20, 22, 30), outline=B.IRON)
            d.text((cx + 12, cy + 8), k, font=B.SMALL, fill=B.IRON_L)
            d.text((cx + 12, cy + 24), str(v), font=B.ImageFont.truetype(str(B.F / "JetBrainsMono-Bold.ttf"), 24), fill=B.PARCH)
            if g:
                d.text((cx + 80, cy + 34), f"+{g} lvl", font=B.SMALL, fill=B.IRON_L)
        gx = rx + 480
        for j, (g, items) in enumerate(DERIVED):
            gy = y0 + 124 + sum(len(DERIVED[k][1]) * 22 + 40 for k in range(j))
            d.text((gx, gy), g, font=B.BODY, fill=B.GOLD)
            for i, (k, v) in enumerate(items):
                yy = gy + 26 + i * 22
                d.text((gx, yy), k, font=B.SMALL, fill=B.IRON_L)
                tw = d.textlength(v, font=B.MONO_L)
                d.text((gx + 300 - tw, yy - 1), v, font=B.MONO_L, fill=B.PARCH)
                d.line((gx, yy + 19, gx + 300, yy + 19), fill=(28, 31, 40))
        d.text((rx, y0 + 320), "Hover a stat for what it does; Level up shows the change live (old -> new, in amber).",
               font=B.SMALL, fill=B.IRON_L)
    else:
        # gear: six slot tiles around a figure, three pouch tiles, the stash as an icon grid, a compare line
        slots = [("Main hand", "hammer/titan_warhammer.png", "Notched Billhook"), ("Off hand", "items/shield/base.png", "Iron Kite Shield"),
                 ("Head", None, "empty"), ("Body", "items/plate/base.png", "Dented Cuirass"), ("Accessory", None, "empty"),
                 ("Accessory", "items/accessory/keepsake_kaelen.png", "Kaelen's keepsake")]
        d.text((rx, y0 + 124), "Equipped", font=B.BODY, fill=B.GOLD)
        for i, (lab, rel, name) in enumerate(slots):
            cx, cy = rx + (i % 2) * 210, y0 + 152 + (i // 2) * 70
            tile_bg(d, cx, cy, 48, on=i == 0)
            if rel:
                ic = icon(rel if rel.startswith("items/") else "items/" + rel, 48)
                img.paste(ic, (cx, cy), ic)
            d.text((cx + 58, cy + 4), lab, font=B.SMALL, fill=B.IRON_L)
            d.text((cx + 58, cy + 22), name, font=B.SMALL, fill=B.PARCH if rel else B.IRON)
        d.text((rx, y0 + 370), "Battle pouch", font=B.BODY, fill=B.GOLD)
        for k in range(3):
            tile_bg(d, rx + k * 60, y0 + 398, 40)
        ic = icon("items/consumable/smelling_salts.png", 40)
        img.paste(ic, (rx, y0 + 398), ic)
        sx = rx + 450
        d.text((sx, y0 + 124), "Party stash", font=B.BODY, fill=B.GOLD)
        stash = ["consumable/smelling_salts.png", "material/sinew_braid.png", "material/ossified_bellows.png",
                 "gauntlet/riveted_gauntlets.png", "consumable/elixir.png", "consumable/bomb.png", "leather/ember_brigandine.png",
                 "accessory/amulet.png"]
        for i, rel in enumerate(stash):
            cx, cy = sx + (i % 5) * 58, y0 + 152 + (i // 5) * 58
            tile_bg(d, cx, cy, 48, on=i == 3)
            ic = icon("items/" + rel, 48)
            img.paste(ic, (cx, cy), ic)
            d.text((cx + 36, cy + 34), str([3, 1, 1, 1, 2, 1, 1, 1][i]), font=B.MONO, fill=B.PARCH)
        B.frame_a(d, (sx, y0 + 290, sx + 288, y0 + 380), fill=(16, 18, 25))
        d.text((sx + 12, y0 + 298), "Riveted Gauntlets", font=B.TITLE_S, fill=B.GOLD)
        d.text((sx + 12, y0 + 322), "vs empty hands slot: Armor +6", font=B.SMALL, fill=B.PARCH)
        d.text((sx + 12, y0 + 340), "Melee power +2,  Initiative -1", font=B.SMALL, fill=B.PARCH)
        d.text((sx + 12, y0 + 358), "Click to equip", font=B.SMALL, fill=B.AMBER)
    B.frame_a(d, (x0 + 324, y0 + 520, box[2] - 26, y0 + 578), fill=(16, 18, 25), top=False)
    d.text((x0 + 338, y0 + 530), "CONS: Max HP and resistances; +2 from each Vanguard level.", font=B.SMALL, fill=B.PARCH)
    hint(d, x0 + 26, box[3] - 40, "Q/E member   Tab next tab   Esc close")
    button(d, box[2] - 26 - 120, box[3] - 52, "Done", True, w=120)
    return img


# ---------------------------------------------------------------- UX-32 shop
STOCK = [("hammer/titan_warhammer.png", "Notched Billhook", "Polearm  2H  Power 18", "40", "2"),
         ("shield/base.png", "Heater Shield", "Off hand  Guard 12", "35", "2"),
         ("plate/base.png", "Watch Cuirass", "Body  Armor 14", "50", "1"),
         ("leather/ember_brigandine.png", "Soot Leathers", "Body  Armor 8  Dodge 3", "30", "2"),
         ("crossbow/base.png", "Stirrup Arbalest", "Crossbow  Power 16", "40", "2"),
         ("consumable/elixir.png", "Tallow Tonic", "Consumable  Heal 40", "12", "5")]
STASH = [("consumable/smelling_salts.png", "Smelling Salts", "Consumable", "4", "x3"),
         ("material/sinew_braid.png", "Sinew Braid", "Material", "6", "x1"),
         ("material/ossified_bellows.png", "Ossified Bellows", "Material", "9", "x1")]


def shop(hub_shot, shell):
    img = dim(Image.open(hub_shot).convert("RGB"), 0.4)
    img.paste(hud_bar(hub_shot), (0, 0))
    box = (90, 70, 1190, 680)
    d = shell.draw(img, box, None)
    x0, y0, x1, y1 = box
    framed(img, d, face("berek_dorn", 64), x0 + 26, y0 + 18)
    d.text((x0 + 104, y0 + 18), "The Tinker's Hearth", font=B.TITLE, fill=B.GOLD)
    d.text((x0 + 104, y0 + 48), "Berek: \"Watch steel, barely dented. The Watch won't miss it.\"", font=B.BODY, fill=B.PARCH)
    lx, w = x0 + 26, 500
    d.text((lx, y0 + 100), "Stock", font=B.BODY, fill=B.GOLD)
    d.text((lx + w - 120, y0 + 102), "Scrap   left", font=B.SMALL, fill=B.IRON_L)
    for i, (rel, n, sub, price, left) in enumerate(STOCK):
        y = y0 + 126 + i * 48
        row(img, d, lx, y, w, icon("items/" + rel, 32), n, sub, None, None, on=i == 0)
        tw = d.textlength(price, font=B.MONO_L)
        d.text((lx + w - 76 - tw, y + 12), price, font=B.MONO_L, fill=B.AMBER)
        d.text((lx + w - 30, y + 12), left, font=B.MONO_L, fill=B.IRON_L)
        if i == 0:
            d.rectangle((lx + 260, y + 10, lx + 360, y + 34), fill=(20, 22, 30), outline=B.IRON)
            d.text((lx + 270, y + 12), "-", font=B.MONO_L, fill=B.PARCH)
            d.text((lx + 305, y + 12), "1", font=B.MONO_L, fill=B.GOLD)
            d.text((lx + 342, y + 12), "+", font=B.MONO_L, fill=B.PARCH)
    rx = x0 + 560
    d.text((rx, y0 + 100), "Your stash", font=B.BODY, fill=B.GOLD)
    d.text((rx + 420, y0 + 102), "sells for", font=B.SMALL, fill=B.IRON_L)
    for i, (rel, n, sub, price, cnt) in enumerate(STASH):
        y = y0 + 126 + i * 48
        row(img, d, rx, y, 512, icon("items/" + rel, 32), n, sub, price, cnt)
    B.frame_a(d, (rx, y0 + 290, rx + 512, y0 + 400), fill=(16, 18, 25))
    d.text((rx + 14, y0 + 298), "Notched Billhook", font=B.TITLE_S, fill=B.GOLD)
    d.text((rx + 14, y0 + 322), "vs Kaelen's Notched Billhook: same item (he wields one)", font=B.SMALL, fill=B.PARCH)
    d.text((rx + 14, y0 + 340), "vs Maren's Censer: Power +6, Reach 2, no Prayer slot", font=B.SMALL, fill=B.PARCH)
    d.text((rx + 14, y0 + 362), "Polearm  2H  Power 18  Stagger 22  Wear 40/40", font=B.MONO, fill=B.IRON_L)
    # the basket strip
    by = y1 - 120
    d.rectangle((x0 + 26, by, x1 - 26, by + 56), fill=(18, 20, 28), outline=B.IRON)
    d.text((x0 + 40, by + 6), "Trade", font=B.SMALL, fill=B.IRON_L)
    for k, (rel, n) in enumerate([("hammer/titan_warhammer.png", 1), ("consumable/elixir.png", 2), ("material/sinew_braid.png", -1)]):
        cx = x0 + 90 + k * 58
        tile_bg(d, cx, by + 10, 36)
        ic = icon("items/" + rel, 36)
        img.paste(ic, (cx, by + 10), ic)
        d.text((cx + 22, by + 30), f"{n:+d}" if n < 0 else f"{n}", font=B.MONO, fill=(200, 70, 60) if n < 0 else B.PARCH)
    d.text((x0 + 290, by + 10), "Total  -58 Scrap", font=B.MONO_L, fill=B.AMBER)
    d.text((x0 + 290, by + 30), "Scrap 85 -> 27", font=B.MONO, fill=B.PARCH)
    button(d, x1 - 26 - 160, by + 12, "Confirm trade", True, w=160)
    button(d, x1 - 26 - 260, by + 12, "Clear", w=88)
    button(d, x0 + 26, y1 - 52, "Close", w=100)
    hint(d, x0 + 140, y1 - 44, "Click a row to add one   Shift-click 5   Right click to sell")
    return img


# ---------------------------------------------------------------- UX-31 notice board
def board(hub_shot, shell):
    img = dim(Image.open(hub_shot).convert("RGB"), 0.4)
    img.paste(hud_bar(hub_shot), (0, 0))
    box = (90, 70, 1190, 680)
    d = shell.draw(img, box, None)
    x0, y0, x1, y1 = box
    framed(img, d, face("talia_rusk", 64), x0 + 26, y0 + 18)
    d.text((x0 + 104, y0 + 18), "Talia's Cipher Board", font=B.TITLE, fill=B.GOLD)
    d.text((x0 + 104, y0 + 48), "Bone-cipher rods and chalk tallies: what the cell can bite off next.", font=B.BODY, fill=B.PARCH)
    lx, w = x0 + 26, 400
    y = y0 + 104
    slips = [("main", "The Sweep", "Alleyway Junction 7", True), ("main", "The Crossroads Cages", "Ash-Crawl Crossroads", False),
             ("job", "Corvo's Gate", "Corvo's Gate", False), ("job", "The Night of the Red Pen", "Seventh Ward", False),
             ("job", "The Sluice Lock-Ciphers", "The Sluice", False)]
    for kind, name, where, on in slips:
        d.rectangle((lx, y, lx + w, y + 46), fill=(42, 36, 28) if on else (26, 24, 22), outline=B.GOLD if on else (70, 60, 48))
        if kind == "main":
            d.ellipse((lx + 10, y + 12, lx + 32, y + 34), fill=OX, outline=(110, 20, 20))
            d.ellipse((lx + 16, y + 18, lx + 26, y + 28), outline=(200, 90, 80))
        else:
            d.rectangle((lx + 16, y + 6, lx + 26, y + 12), fill=(150, 140, 120))      # a pin
        d.text((lx + 44, y + 4), name, font=B.BODY, fill=B.GOLD if on else B.PARCH)
        d.text((lx + 44, y + 24), where, font=B.SMALL, fill=B.IRON_L)
        y += 52
    d.rectangle((lx, y, lx + w, y + 30), fill=(18, 20, 28), outline=B.IRON)
    d.text((lx + 14, y + 6), "Sealed (4)", font=B.BODY, fill=B.IRON_L)
    d.polygon([(lx + w - 22, y + 12), (lx + w - 12, y + 12), (lx + w - 17, y + 19)], fill=B.IRON_L)
    rx = x0 + 450
    d.rectangle((rx, y0 + 104, x1 - 26, y1 - 70), fill=(30, 27, 23), outline=(90, 76, 58))
    d.ellipse((x1 - 70, y0 + 112, x1 - 40, y0 + 142), fill=OX)
    d.text((rx + 20, y0 + 116), "The Sweep", font=B.TITLE, fill=B.GOLD)
    d.text((rx + 20, y0 + 144), "Main job  On offer", font=B.SMALL, fill=B.IRON_L)
    body = ["Captain Vane has sealed the Seventh Ward grates and the Watch is selling",
            "captives for a Bastion commission.", "", "Break the sweep at Alleyway Junction 7."]
    for i, l in enumerate(body):
        d.text((rx + 20, y0 + 172 + i * 22), l, font=B.BODY, fill=B.PARCH)
    d.text((rx + 20, y0 + 270), "Rewards", font=B.BODY, fill=B.GOLD)
    x = rx + 20
    for lab, rel in (("120 XP", None), ("40", "ui/hud/gold.png"), ("x1", "items/consumable/smelling_salts.png")):
        tile_bg(d, x, y0 + 298, 48)
        if rel:
            ic = icon(rel, 40 if "hud" in rel else 48)
            img.paste(ic, (x + (4 if "hud" in rel else 0), y0 + 298), ic)
            d.text((x + 28, y0 + 330), lab, font=B.MONO, fill=B.PARCH)
        else:
            d.text((x + 4, y0 + 314), lab, font=B.MONO, fill=B.AMBER)
        x += 64
    d.text((x + 8, y0 + 300), "Pick 1 of", font=B.SMALL, fill=B.IRON_L)
    tile_bg(d, x + 8, y0 + 320, 26)
    ic = icon("items/shield/base.png", 26)
    img.paste(ic, (x + 8, y0 + 320), ic)
    d.text((x + 40, y0 + 324), "Gilded Pavise", font=B.SMALL, fill=B.PARCH)
    d.text((rx + 20, y0 + 380), "Where: Alleyway Junction 7  ->  show on the map", font=B.BODY, fill=B.AMBER)
    button(d, x1 - 26 - 120, y1 - 52, "Accept", True, w=120)
    button(d, x0 + 26, y1 - 52, "Close", w=100)
    return img


# ---------------------------------------------------------------- UX-10 save and load
def saves(shell, title_png):
    img = dim(Image.open(title_png).convert("RGB"), 0.35)
    box = (190, 110, 1090, 600)                            # sized to its content (UX-02)
    d = shell.draw(img, box, "Save game")
    x0, y0, x1, y1 = box
    y = y0 + 76
    filled = [("Autosave", "kaelen", "The Sluice Undercroft", "Day 4  Tithing 42 days", "today 20:09", False),
              ("Slot 1", "kaelen", "Alleyway Junction 7", "Day 3  Tithing 43 days", "today 20:12", True)]
    for name, lead, place, sub, when, focus in filled:
        h = 88
        d.rectangle((x0 + 26, y, x1 - 26, y + h), fill=(36, 30, 22) if focus else (20, 22, 30), outline=B.GOLD if focus else B.IRON)
        framed(img, d, face(lead, 64), x0 + 40, y + 12)
        d.text((x0 + 120, y + 8), place, font=B.TITLE_S, fill=B.GOLD)
        d.text((x0 + 120, y + 32), f"{name}   {sub}", font=B.SMALL, fill=B.IRON_L)
        for k, m in enumerate(["kaelen", "vesper", "maren", "hollis"]):
            img.paste(face(m, 24), (x0 + 120 + k * 64, y + 54))
            d.text((x0 + 148 + k * 64, y + 58), "3", font=B.MONO, fill=B.PARCH)
        tw = d.textlength(when, font=B.SMALL)
        d.text((x1 - 40 - tw, y + 10), when, font=B.SMALL, fill=B.IRON_L)
        if focus:
            for k, rel in enumerate(("lock.png", "delete.png")):
                p = ROOT / "engine/assets/ui" / rel
                ic = Image.open(p).convert("RGBA").resize((20, 20), Image.LANCZOS)
                img.paste(ic, (x1 - 90 + k * 34, y + 54), ic)
        y += h + 10
    for k in range(3):
        for xx in range(x0 + 26, x1 - 26, 8):
            d.line((xx, y, xx + 4, y), fill=B.IRON); d.line((xx, y + 30, xx + 4, y + 30), fill=B.IRON)
        d.text((x0 + 40, y + 6), f"Slot {k + 2}", font=B.SMALL, fill=B.IRON_L)
        d.text((x0 + 110, y + 6), "empty", font=B.SMALL, fill=B.IRON)
        y += 40
    button(d, x0 + 26, y1 - 52, "Back", w=100)
    button(d, x1 - 26 - 150, y1 - 52, "Save over Slot 1", True, w=150)
    hint(d, x0 + 140, y1 - 44, "Lock keeps a slot from being overwritten")
    return img


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    shell = Shell(pt / "003_click_menu_settings.png")
    battle = Path("/mnt/project-files/art-ui/battle_ux14/battle_take16_720p.png")
    outs = {"report_victory": report(battle, shell, True), "report_defeat": report(battle, shell, False),
            "story_speaking": story(True), "story_narration": story(False),
            "character_stats": character(shell, "stats"), "character_gear": character(shell, "gear"),
            "shop": shop(pt / "198_hub.png", shell), "board": board(pt / "198_hub.png", shell),
            "saves": saves(shell, pt / "002_title.png")}
    for k, im in outs.items():
        im.save(out / f"{k}.png")


if __name__ == "__main__":
    main()
