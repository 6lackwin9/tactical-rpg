"""AI-21 battle mock take 12 (Tushar 2026-11-18 21:55), built on take 11.

Usage: python3 art_src/style/battle_take12_mock.py <playtest_dir> <out_dir>
Panels: (1) the foe card's two modes: a hero selected (that hero's skill rows, take 10) vs only the foe inspected (every
ally per block, take 11); (2) every kit status has its own icon, rim colour says buff or debuff; (3) effect chips move
onto the tile's top edge so a 1-block tile's skill icon stays clear; (4) condensed statuses on the plates: two chips on
tabs on the plate edge, a "+N" for the rest, and a detail popover listing every status with its rounds left.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take11_mock as T  # noqa: E402

ST = T.ST
TINY = ImageFont.truetype(str(B.F / "JetBrainsMono-Bold.ttf"), 10)
KIND = {"fortify": "buff", "haste": "buff", "regen": "buff", "riposte": "buff", "steadfast": "buff", "stealth": "buff",
        "bleed": "debuff", "burn": "debuff", "expose": "debuff", "poison": "debuff", "root": "debuff", "slow": "debuff",
        "stagger": "debuff", "stun": "debuff", "taunt": "debuff", "weaken": "debuff"}
W, G, OY = T.W, T.G, 12          # OY: the tiles sit 12 px lower so chips can ride on their top edge


def rim(kind):
    return T.BUFF if kind == "buff" else T.DEBUFF


def mini(img, d, x, y, status, turns=None, incoming=False, px=12):
    """A compact chip: icon, rim by kind (dashed if incoming), the turns as a tiny digit in its corner."""
    col = rim(KIND.get(status, "buff")) if status != "heal" else T.HEAL
    s = px + 4
    d.rectangle((x, y, x + s, y + s), fill=B.SLATE)
    if incoming:
        for k in range(0, s + 1, 3):
            d.point((x + k, y), fill=col); d.point((x + k, y + s), fill=col)
            d.point((x, y + k), fill=col); d.point((x + s, y + k), fill=col)
    else:
        d.rectangle((x, y, x + s, y + s), outline=col)
    if status == "heal":
        B.heart(d, x + 2, y + 3, px, T.HEAL)
    else:
        ic = Image.open(ST / f"{status}.png").convert("RGBA").resize((px, px), Image.LANCZOS)
        img.paste(ic, (x + 2, y + 2), ic)
    if turns:
        tw = 7
        d.rectangle((x + s - tw + 1, y + s - 9, x + s + 2, y + s + 2), fill=B.VOID)
        d.text((x + s - tw + 2, y + s - 10), str(turns), font=TINY, fill=B.PARCH)
    return s + 1


# ---------------------------------------------------------------- 1. foe card modes
def panel_modes(shot):
    m10 = B.mock10(shot)
    a = m10.crop((794, 94, 1088, 310))
    # the same plan as the left card (Kaelen: Hammer lands block 4, Pommel 5 and 6) plus Vesper's hits
    rows = lambda kae, ves: [(3, [ves], 17, 26, (("bleed", "debuff", 3),)), (4, [kae], 34, 40, ()),
                             (5, [kae, ves], 22, 25, ()), (6, [kae], 12, 15, ())]
    b = T.panel_foe_blocks(shot, labelled=False, rows=rows, brk="BREAK in 4").crop((2, 2, 314, 262))
    img = Image.new("RGB", (a.width + b.width + 40, max(a.height, b.height) + 60), B.VOID)
    d = ImageDraw.Draw(img)
    d.text((8, 6), "Kaelen selected (planning): only his moves on this foe", font=B.SMALL, fill=B.GOLD)
    d.text((8, 22), "one row per skill, as in take 10", font=B.SMALL, fill=B.IRON_L)
    img.paste(a, (0, 44))
    bx = a.width + 30
    d.text((bx + 8, 6), "No hero selected (click or hover the foe): everyone", font=B.SMALL, fill=B.GOLD)
    d.text((bx + 8, 22), "one row per block, every ally summed, as in take 11", font=B.SMALL, fill=B.IRON_L)
    img.paste(b, (bx, 44))
    return B.label(img, "1. Foe card: two modes")


# ---------------------------------------------------------------- 2. status icons
def panel_icons():
    names = [n for n in KIND if KIND[n] == "buff"] + [n for n in KIND if KIND[n] == "debuff"]
    img = Image.new("RGB", (900, 176), B.VOID)
    d = ImageDraw.Draw(img)
    d.text((8, 4), "Buffs (gold rim)", font=B.SMALL, fill=T.BUFF)
    d.text((8, 86), "Debuffs (oxblood rim)", font=B.SMALL, fill=T.DEBUFF)
    bi = di = 0
    for n in names:
        if KIND[n] == "buff":
            x, y = 8 + bi * 90, 24; bi += 1
        else:
            x, y = 8 + di * 88, 106; di += 1
        T.chip(img, d, x + 18, y, n, KIND[n], px=24)
        tw = d.textlength(n.capitalize(), font=B.SMALL)
        d.text((x + 33 - tw / 2, y + 32), n.capitalize(), font=B.SMALL, fill=B.PARCH)
    d.text((560, 30), "Each status has its own kit icon", font=B.SMALL, fill=B.IRON_L)
    d.text((560, 46), "(kits/rpg/assets/statuses); the rim", font=B.SMALL, fill=B.IRON_L)
    d.text((560, 62), "only says buff or debuff.", font=B.SMALL, fill=B.IRON_L)
    return B.label(img, "2. One icon per status, rim colour for buff or debuff")


# ---------------------------------------------------------------- 3. chips on the tile edge
def tile12(img, d, x, icon, blocks, ap, on=False, chips=()):
    w = blocks * W + (blocks - 1) * G
    y0 = OY
    d.rectangle((x, y0 + 3, x + w, y0 + 48), fill=B.SLATE, outline=B.GOLD if on else B.IRON, width=2 if on else 1)
    t = B.slot_icon(icon, 30)
    img.paste(t, (x + (w - 30) // 2, y0 + 6), t)
    if ap:
        B.move_pips(d, x, w, y0 + 38, ap)
    for b in range(blocks):
        bx = x + b * (W + G)
        c = B.GOLD if on else B.IRON_L
        d.rectangle((bx, y0 + 55, bx + W, y0 + 60), fill=c)
        if b < blocks - 1:
            d.rectangle((bx + W, y0 + 57, bx + W + G, y0 + 58), fill=c)
    n = len(chips)
    cw = n * 17 - 1
    cx = x + (w - cw) // 2                 # centred tab on the top edge, half above the tile
    for st in chips:
        cx += mini(img, d, cx, y0 - 5, st, px=12)
    return x + w + G


def panel_tiles():
    before = Image.new("RGB", (420, 70), B.BASALT)
    db = ImageDraw.Draw(before)
    x = 12
    for p in [("arrow", 1, 1, False), ("bleeding_cut", 1, 2, False, False, (("bleed", "debuff"),)),
              ("greater_mend", 1, 3, False, False, (("heal", "buff"),)), ("rally", 1, 1, False, True, (("haste", "buff"), ("heal", "buff")))]:
        x = T.tile(before, db, x, *p[:3], ghost=p[3], on=p[4] if len(p) > 4 else False, chips=p[5] if len(p) > 5 else ())
    after = Image.new("RGB", (420, 82), B.BASALT)
    da = ImageDraw.Draw(after)
    x = 12
    for p in [("arrow", 1, 1, False, ()), ("bleeding_cut", 1, 2, False, ("bleed",)), ("greater_mend", 1, 3, False, ("heal",)),
              ("rally", 1, 1, True, ("haste", "heal"))]:
        x = tile12(after, da, x, *p[:3], on=p[3], chips=p[4])
    img = Image.new("RGB", (900, 120), B.VOID)
    d = ImageDraw.Draw(img)
    d.text((8, 2), "Take 11: chip in the corner covers a 1-block icon", font=B.SMALL, fill=B.IRON_L)
    img.paste(before, (0, 22))
    d.text((452, 2), "Take 12: chips ride on the tile's top edge, centred", font=B.SMALL, fill=B.GOLD)
    img.paste(after, (444, 22))
    out = B.label(img, "3. Effect chips move onto the tile's top edge")
    return out


# ---------------------------------------------------------------- 4. condensed statuses
KAE_ST = [("bleed", 1, True), ("weaken", 2, False), ("slow", 3, False), ("fortify", 1, False), ("regen", 2, False), ("haste", 3, True)]
FOE_ST = [("stagger", 1, False), ("bleed", 3, True), ("expose", 2, False), ("riposte", 2, False)]
ORDER = "Order: debuffs first, then the soonest to run out; dashed = lands this round from a planned move"


def ordered(sts):
    return sorted(sts, key=lambda s: (KIND[s[0]] != "debuff", s[1]))


def edge_tabs(img, d, x, y, sts, side="right", shown=3):
    """Status tabs hanging off the plate's map-facing edge: up to three chips with their turns, then +N. Nothing is
    added inside the plate."""
    o = ordered(sts)
    tw = 32
    tx = x if side == "right" else x - tw
    rest = len(o) - shown
    items = o[:shown] + ([None] if rest > 0 else [])
    for i, it in enumerate(items):
        ty = y + 2 + i * 20
        d.rectangle((tx, ty, tx + tw, ty + 18), fill=(16, 18, 25), outline=B.IRON)
        if it is None:
            d.text((tx + 7, ty + 2), f"+{rest}", font=B.MONO, fill=B.PARCH)
            continue
        st, n, inc = it
        mini(img, d, tx + 2, ty + 1, st, incoming=inc, px=12)
        d.text((tx + 21, ty + 2), str(n), font=B.MONO, fill=B.PARCH)


def ally_plate(img, d, x, y, fc, name, sts, sel=False):
    d.rectangle((x, y, x + 152, y + 84), fill=B.SLATE, outline=B.GOLD if sel else B.IRON)
    img.paste(fc.resize((48, 48)), (x + 6, y + 6))
    B.intent_tile(d, x + 124, y + 6)
    d.text((x + 60, y + 6), name, font=B.BODY, fill=B.PARCH)
    B.bar(d, x + 60, y + 30, 86, 6, 180 / 242, B.HP)
    d.text((x + 60, y + 39), "180/242", font=B.MONO, fill=B.PARCH)
    B.ap_pips(d, x + 60, y + 56, 10, 10, size=6, gap=2, spent=6)
    B.poise(d, x + 6, y + 72, 5, 4, w=26, h=4)
    edge_tabs(img, d, x + 153, y, sts, "right")


def foe_plate(img, d, x, y, fp, sts, sel=False):
    d.rectangle((x, y, x + 152, y + 84), fill=B.SLATE, outline=(170, 60, 50) if sel else B.IRON)
    img.paste(fp.resize((48, 48)), (x + 6, y + 6))
    d.text((x + 60, y + 4), "Footman", font=B.BODY, fill=B.PARCH)
    B.bar(d, x + 60, y + 26, 86, 6, 0.8, (150, 50, 45))
    d.text((x + 60, y + 34), "115/144", font=B.MONO, fill=B.PARCH)
    B.glyph(d, "sword", x + 60, y + 52, 14, (200, 160, 120))
    for b in range(6):
        d.rectangle((x + 100 + b * 8, y + 57, x + 100 + b * 8 + 5, y + 61), fill=B.IMPACT if b == 1 else B.AP_OFF)
    B.poise(d, x + 6, y + 72, 5, 4, w=26, h=4)
    edge_tabs(img, d, x - 1, y, sts, "left")


LABEL = {"bleed": "loses HP each round", "weaken": "deals less damage", "slow": "moves cost 1 AP more",
         "fortify": "takes less damage", "regen": "heals each round", "haste": "+1 AP next round",
         "stagger": "poise does not refill", "expose": "takes more damage", "riposte": "strikes back when hit"}


def popover(img, d, x, y, title, sts):
    o = ordered(sts)
    h = 36 + len(o) * 26 + 8
    B.frame_a(d, (x, y, x + 300, y + h), fill=(16, 18, 25))
    d.text((x + 12, y + 8), title, font=B.TITLE_S, fill=B.GOLD)
    for i, (st, n, inc) in enumerate(o):
        ry = y + 36 + i * 26
        mini(img, d, x + 12, ry, st, incoming=inc, px=14)
        d.text((x + 36, ry + 1), st.capitalize(), font=B.SMALL, fill=B.PARCH)
        d.text((x + 100, ry + 1), LABEL.get(st, ""), font=B.SMALL, fill=B.IRON_L)
        for k in range(3):
            d.rectangle((x + 242 + k * 9, ry + 6, x + 248 + k * 9, ry + 12), fill=rim(KIND[st]) if k < n else B.AP_OFF)
        d.text((x + 272, ry + 1), str(n), font=B.MONO, fill=B.PARCH)
    return h


def panel_condensed(shot):
    img = Image.new("RGB", (1150, 300), B.VOID)
    d = ImageDraw.Draw(img)
    kae, ves = T.face("kaelen", 48), T.face("vesper", 48)
    fp = shot.crop((1044, 220, 1108, 284))
    d.text((8, 4), "Party plates", font=B.SMALL, fill=B.GOLD)
    ally_plate(img, d, 8, 24, kae, "Kaelen", KAE_ST, sel=True)
    ally_plate(img, d, 8, 116, ves, "Vesper", [("weaken", 1, False)])
    d.text((200, 30), "6 effects:", font=B.SMALL, fill=B.IRON_L)
    d.text((200, 46), "3 tabs, +3", font=B.SMALL, fill=B.IRON_L)
    d.text((200, 122), "1 effect:", font=B.SMALL, fill=B.IRON_L)
    d.text((200, 138), "1 tab", font=B.SMALL, fill=B.IRON_L)
    ph = popover(img, d, 290, 24, "Kaelen: 6 effects", KAE_ST)
    d.text((290, 24 + ph + 6), "Hover the +3 (or press the plate) for the full list", font=B.SMALL, fill=B.IRON_L)
    d.text((640, 4), "Foe plates: tabs on the map side", font=B.SMALL, fill=B.GOLD)
    foe_plate(img, d, 680, 24, fp, FOE_ST, sel=True)
    foe_plate(img, d, 680, 116, fp, [])
    popover(img, d, 840, 24, "Footman: 4 effects", FOE_ST)
    d.text((8, 280), ORDER, font=B.SMALL, fill=B.IRON_L)
    return B.label(img, "4. Condensed statuses: up to 3 tabs on the plate edge with turns, +N for the rest, full list on hover")


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    parts = [panel_modes(shot), panel_icons(), panel_tiles(), panel_condensed(shot)]
    names = ["modes", "icons", "tiles", "condensed"]
    for p, n in zip(parts, names):
        p.save(out / f"take12_{n}.png")
    W2 = max(p.width for p in parts)
    sh = Image.new("RGB", (W2, sum(p.height for p in parts) + 12 * len(parts)), B.VOID)
    y = 0
    for p in parts:
        sh.paste(p, (0, y))
        y += p.height + 12
    sh.save(out / "battle_layout_take12_sheet.png")


if __name__ == "__main__":
    main()
