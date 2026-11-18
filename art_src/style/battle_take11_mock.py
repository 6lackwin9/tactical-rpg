"""AI-21 battle mock take 11 (Tushar 2026-11-18 21:43), built on battle_layout_mock.py take 10.

Usage: python3 art_src/style/battle_take11_mock.py <playtest_dir> <out_dir>
Panels: (1) the ghost plan (the planner's suggestion) and how picks replace it; (2) the skill menu open with the foe
intents visible at the side; (3) the foe card as one row per block, summed over every ally; (4) how heals, buffs,
debuffs and statuses read on the timeline, the plates and the card.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402

ROOT = B.ROOT
ST = ROOT / "kits/rpg/assets/statuses"
HEAL = (110, 170, 90)
BUFF, DEBUFF = B.GOLD, (190, 70, 60)
W, G = 54, 6


def face(name, px):
    return Image.open(ROOT / "games/emberwake/assets/portraits/heroes" / f"{name}.png").convert("RGB").resize((px, px), Image.LANCZOS)


def chip(img, d, x, y, status, kind, turns=None, incoming=False, px=18):
    """A status chip: the kit status icon on a dark tile, gold rim for a buff, oxblood for a debuff; dashed when it
    is only planned (incoming), with the turns in mono beside it."""
    col = BUFF if kind == "buff" else DEBUFF
    d.rectangle((x, y, x + px + 3, y + px + 3), fill=B.SLATE)
    if incoming:
        for k in range(0, px + 4, 4):
            d.point((x + k, y), fill=col); d.point((x + k, y + px + 3), fill=col)
            d.point((x, y + k), fill=col); d.point((x + px + 3, y + k), fill=col)
    else:
        d.rectangle((x, y, x + px + 3, y + px + 3), outline=col)
    ic = Image.open(ST / f"{status}.png").convert("RGBA").resize((px, px), Image.LANCZOS)
    img.paste(ic, (x + 2, y + 2), ic)
    w = px + 4
    if turns:
        d.text((x + w + 2, y + 2), str(turns), font=B.MONO, fill=B.PARCH)
        w += 12
    return w


def tile(img, d, x, icon, blocks, ap, ghost=False, on=False, chips=()):
    w = blocks * W + (blocks - 1) * G
    if ghost:
        d.rectangle((x, 3, x + w, 48), fill=B.BASALT)
        for k in range(x, x + w, 6):
            d.line((k, 3, min(k + 3, x + w), 3), fill=B.IRON_L); d.line((k, 48, min(k + 3, x + w), 48), fill=B.IRON_L)
        for k in range(3, 48, 6):
            d.line((x, k, x, min(k + 3, 48)), fill=B.IRON_L); d.line((x + w, k, x + w, min(k + 3, 48)), fill=B.IRON_L)
    else:
        d.rectangle((x, 3, x + w, 48), fill=B.SLATE, outline=B.GOLD if on else B.IRON, width=2 if on else 1)
    t = B.slot_icon(icon, 30)
    if ghost:
        t.putalpha(t.getchannel("A").point(lambda a: a * 45 // 100))
    img.paste(t, (x + (w - 30) // 2, 6), t)
    if ap:
        if ghost:
            pw = ap * 10 - 2
            for i in range(ap):
                px = x + (w - pw) // 2 + i * 10
                d.rectangle((px, 38, px + 7, 45), outline=B.AP_ON)
        else:
            B.move_pips(d, x, w, 38, ap)
    for b in range(blocks):
        bx = x + b * (W + G)
        c = B.IRON if ghost else (B.GOLD if on else B.IRON_L)
        d.rectangle((bx, 55, bx + W, 60), fill=c)
        if b < blocks - 1:
            d.rectangle((bx + W, 57, bx + W + G, 58), fill=c)
    cx = x + 3
    for st, kind in chips:
        if st == "heal":                     # a heal: small green heart with a plus
            d.rectangle((cx, 5, cx + 15, 20), fill=B.SLATE, outline=HEAL)
            B.heart(d, cx + 2, 8, 11, HEAL)
            cx += 17
            continue
        cx += chip(img, d, cx, 5, st, kind, px=12) + 1
    return x + w + G


def strip(plan, title, start=10, cap=12):
    img = Image.new("RGB", (900, 66), B.BASALT)
    d = ImageDraw.Draw(img)
    x = 12
    used = 0
    for p in plan:
        x = tile(img, d, x, *p[:3], ghost=p[3], on=p[4] if len(p) > 4 else False, chips=p[5] if len(p) > 5 else ())
        if not p[3]:
            used += p[2]
    ghost_ap = sum(p[2] for p in plan if p[3])
    ax, ay = x + 18, 12
    for i in range(cap):
        px = ax + i * 14
        if i < used:
            d.rectangle((px, ay, px + 10, ay + 10), fill=B.PLANNED)
        elif i < used + ghost_ap:
            d.rectangle((px, ay, px + 10, ay + 10), fill=B.AP_ON, outline=B.PLANNED)
        elif i < start:
            d.rectangle((px, ay, px + 10, ay + 10), fill=B.AP_ON)
        else:
            d.rectangle((px, ay + 3, px + 10, ay + 7), fill=B.AP_OFF)
    d.text((ax, ay + 18), "AP", font=B.MONO_L, fill=B.GOLD)
    d.text((ax + 24, ay + 18), f"{start}/{cap}", font=B.MONO_L, fill=B.PARCH)
    d.text((ax + 76, ay + 18), f"-{used}", font=B.MONO_L, fill=B.PLANNED)
    if ghost_ap:
        d.text((ax + 108, ay + 18), f"(-{ghost_ap})", font=B.MONO_L, fill=B.IRON_L)
    return B.label(img, title)


def panel_ghost():
    g = [("arrow", 1, 1, True), ("pneumatic_hammer_strike", 3, 5, True), ("pommel_strike", 1, 2, True), ("pommel_strike", 1, 2, True)]
    a = strip(g, "1. No picks yet: the planner's suggestion shows as a ghost (dashed, faded, hollow AP). Confirm takes it as is.")
    p = [("arrow", 1, 1, False), ("bleeding_cut", 1, 2, False, True, (("bleed", "debuff"),)), ("pneumatic_hammer_strike", 3, 5, True),
         ("intent:rest", 1, 0, True)]
    b = strip(p, "2. You pick Move, then Bleeding Cut in block 2: those turn solid; the ghost refills blocks 3-6 around them.")
    c = strip([("arrow", 1, 1, False), ("bleeding_cut", 1, 2, False, False, (("bleed", "debuff"),)),
               ("pneumatic_hammer_strike", 3, 5, False, True), ("intent:rest", 1, 0, False)],
              "3. Every block picked: no ghost left. Undo takes back the last pick; Clear returns to step 1.")
    sh = Image.new("RGB", (900, a.height * 3 + 16), B.VOID)
    for i, s in enumerate((a, b, c)):
        sh.paste(s, (0, i * (a.height + 8)))
    return sh


SKILLS = [("pommel_strike", "Pommel Strike", 1, 2, 12, 15, ()),
          ("pneumatic_hammer_strike", "Hammer Strike", 3, 5, 34, 40, ()),
          ("bleeding_cut", "Bleeding Cut", 1, 2, 8, 5, (("bleed", "debuff", 3),)),
          ("shield_wall", "Shield Wall", 2, 2, 0, 0, (("fortify", "buff", 2),)),
          ("rally", "Rally", 1, 1, 0, 0, (("haste", "buff", 1),))]


def panel_menu(shot):
    img = B.mock10(shot)
    d = ImageDraw.Draw(img)
    x0, y0, w = 330, 210, 420
    h = 44 + len(SKILLS) * 40 + 26
    B.frame_a(d, (x0, y0, x0 + w, y0 + h), fill=(16, 18, 25))
    d.text((x0 + 14, y0 + 10), "Block 2", font=B.TITLE_S, fill=B.GOLD)
    d.text((x0 + 90, y0 + 13), "vs", font=B.SMALL, fill=B.IRON_L)
    img.paste(shot.crop((1044, 220, 1108, 284)).resize((20, 20)), (x0 + 112, y0 + 10))
    for i, (ic, name, blocks, ap, hp, pz, sts) in enumerate(SKILLS):
        y = y0 + 40 + i * 40
        if i == 2:
            d.rectangle((x0 + 8, y - 3, x0 + w - 8, y + 35), fill=B.SLATE, outline=B.GOLD)
        t = B.slot_icon(ic, 32)
        img.paste(t, (x0 + 14, y), t)
        d.text((x0 + 54, y + 1), name, font=B.BODY, fill=B.PARCH)
        B.ap_pips(d, x0 + 54, y + 22, ap, ap, size=7, gap=2)
        for b in range(blocks):
            d.rectangle((x0 + 112 + b * 9, y + 23, x0 + 112 + b * 9 + 6, y + 27), fill=B.IRON_L)
        cx = x0 + 230
        if hp:
            B.heart(d, cx, y + 9, 12)
            d.text((cx + 16, y + 6), f"-{hp}", font=B.MONO_L, fill=B.PARCH)
            B.poise_glyph(d, cx + 56, y + 7, 13)
            d.text((cx + 72, y + 6), str(pz), font=B.MONO, fill=B.POISE)
            cx += 100
        for st, kind, n in sts:
            cx += chip(img, d, cx, y + 6, st, kind, turns=n) + 4
    d.text((x0 + 14, y0 + h - 20), "Hover a row: its blocks light on the timeline", font=B.SMALL, fill=B.IRON)
    stage = shot.crop((291, 112, 899, 492)).resize((912, 570), Image.NEAREST)
    img.paste(stage.crop((604, 14, 912, 330)), (788, 86))      # clear take 10's card, then the per-block card
    card = panel_foe_blocks(shot, labelled=False)
    img.paste(card.crop((0, 0, 318, 246)), (782, 88))
    return img


def panel_foe_blocks(shot, labelled=True):
    img = Image.new("RGB", (320, 300), B.VOID)
    d = ImageDraw.Draw(img)
    foe_p = shot.crop((1044, 220, 1108, 284))
    kae, ves = face("kaelen", 18), face("vesper", 18)
    cx, cy = 8, 8
    rows = [(2, [kae], 8, 5, (("bleed", "debuff", 3),)), (3, [kae, ves], 22, 30, ()), (4, [kae], 11, 13, ()),
            (5, [ves], 17, 26, ())]
    H = 76 + len(rows) * 30 + 40
    B.frame_a(d, (cx, cy, cx + 300, cy + H), fill=(16, 18, 25))
    img.paste(foe_p.resize((48, 48)), (cx + 14, cy + 14))
    d.text((cx + 72, cy + 10), "Castellan Footman", font=B.TITLE_S, fill=B.GOLD)
    B.heart(d, cx + 72, cy + 33, 11)
    B.bar(d, cx + 88, cy + 36, 130, 6, 1.0, (150, 50, 45))
    d.text((cx + 224, cy + 30), "144/144", font=B.MONO, fill=B.PARCH)
    B.poise_glyph(d, cx + 71, cy + 47, 13)
    B.poise(d, cx + 88, cy + 51, 5, 5, w=24, h=6, gap=2)
    B.pip_tag(d, cx + 224, cy + 47, B.FOE_T)
    left, tot = 5, 0
    y = cy + 76
    for blk, who, hp, pz, sts in rows:
        d.ellipse((cx + 14, y + 2, cx + 32, y + 20), fill=B.SLATE, outline=B.GOLD)
        d.text((cx + 19, y + 3), str(blk), font=B.MONO_L, fill=B.GOLD)
        for k, f in enumerate(who):
            img.paste(f, (cx + 38 + k * 20, y + 2))
        B.heart(d, cx + 84, y + 6, 12)
        d.text((cx + 100, y + 3), f"-{hp}", font=B.MONO_L, fill=B.PARCH)
        pp = min(3, pz // B.FOE_T)
        B.poise_glyph(d, cx + 138, y + 4, 13)
        d.text((cx + 154, y + 3), str(pz), font=B.MONO, fill=B.POISE)
        for j in range(5):
            c = B.POISE if j < left - pp else (B.AMBER if j < left else B.AP_OFF)
            d.rectangle((cx + 176 + j * 10, y + 8, cx + 176 + j * 10 + 7, y + 14), fill=c)
        left -= pp
        tot += hp
        if left <= 0:
            d.text((cx + 230, y + 3), "BREAK", font=B.SMALL, fill=B.AMBER)
        sx = cx + 230 if left > 0 else cx + 270
        for st, kind, n in sts:
            chip(img, d, sx, y, st, kind, px=14)
        y += 30
    d.line((cx + 14, y + 2, cx + 286, y + 2), fill=B.IRON)
    d.text((cx + 14, y + 10), "Total", font=B.SMALL, fill=B.IRON_L)
    B.heart(d, cx + 84, y + 13, 12)
    d.text((cx + 100, y + 10), f"-{tot}", font=B.MONO_L, fill=B.GOLD)
    B.poise_glyph(d, cx + 138, y + 11, 13)
    d.text((cx + 156, y + 10), "BREAK in 5", font=B.SMALL, fill=B.AMBER)
    return B.label(img, "3. Foe card: one row per block, every ally summed") if labelled else img


def plate(img, d, x, y, fc, name, hp, mx, gain=0, loss=0, chips=()):
    d.rectangle((x, y, x + 152, y + 84), fill=B.SLATE, outline=B.IRON)
    img.paste(fc.resize((48, 48)), (x + 6, y + 6))
    d.text((x + 60, y + 6), name, font=B.BODY, fill=B.PARCH)
    B.bar(d, x + 60, y + 30, 86, 6, hp / mx, B.HP)
    if gain:
        g = round(86 * gain / mx)
        e = x + 60 + round(86 * hp / mx)
        d.rectangle((e, y + 30, e + g, y + 36), fill=(170, 220, 140))
        d.text((x + 120, y + 39), f"+{gain}", font=B.MONO, fill=HEAL)
    if loss:
        l = round(86 * loss / mx)
        e = x + 60 + round(86 * hp / mx)
        d.rectangle((e - l, y + 30, e, y + 36), fill=B.IMPACT)
        d.text((x + 120, y + 39), f"-{loss}", font=B.MONO, fill=B.IMPACT)
    d.text((x + 60, y + 39), f"{hp}/{mx}", font=B.MONO, fill=B.PARCH)
    cx = x + 60
    for st, kind, n, inc in chips:
        cx += chip(img, d, cx, y + 56, st, kind, turns=n, incoming=inc, px=14) + 3


def panel_effects():
    img = Image.new("RGB", (900, 330), B.VOID)
    d = ImageDraw.Draw(img)
    s = strip([("arrow", 1, 1, False), ("bleeding_cut", 1, 2, False, False, (("bleed", "debuff"),)),
               ("shield_wall", 2, 2, False, True, (("fortify", "buff"),)), ("greater_mend", 1, 3, False, False, (("heal", "buff"),)),
               ("rally", 1, 1, False, False, (("haste", "buff"),))],
              "Timeline: a tile carries a chip for each effect: green heart = heal, gold rim = buff, oxblood = debuff")
    img.paste(s, (0, 0))
    y = s.height + 16
    d.text((8, y), "Plates: a heal previews green on the HP bar; statuses as chips with turns; dashed = coming from a planned move",
           font=B.SMALL, fill=B.PARCH)
    plate(img, d, 8, y + 22, face("kaelen", 48), "Kaelen", 180, 242, gain=36, chips=(("fortify", "buff", 2, True), ("haste", "buff", 1, False)))
    plate(img, d, 176, y + 22, face("vesper", 48), "Vesper", 150, 190, loss=22,
          chips=(("bleed", "debuff", 2, True), ("weaken", "debuff", 1, False)))
    d.text((360, y + 30), "Kaelen: Mend in block 5 heals +36 (light green),", font=B.SMALL, fill=B.IRON_L)
    d.text((360, y + 48), "Shield Wall will give Fortify 2 (dashed: planned).", font=B.SMALL, fill=B.IRON_L)
    d.text((360, y + 72), "Vesper: a foe's bleed lands this round (dashed),", font=B.SMALL, fill=B.IRON_L)
    d.text((360, y + 90), "Weaken 1 is already on her (solid).", font=B.SMALL, fill=B.IRON_L)
    d.text((360, y + 114), "Foe card rows show the debuffs the plan puts on it.", font=B.SMALL, fill=B.IRON_L)
    return B.label(img, "4. Heals, buffs, debuffs and statuses")


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    g = panel_ghost()
    m = B.label(panel_menu(shot), "2. Skill menu open on block 2, foe intents on the right")
    f = panel_foe_blocks(shot)
    e = panel_effects()
    g.save(out / "take11_ghost.png"); m.save(out / "take11_menu.png"); f.save(out / "take11_foe_blocks.png"); e.save(out / "take11_effects.png")
    W2 = max(m.width, g.width + f.width + 12)
    sh = Image.new("RGB", (W2, m.height + 12 + max(g.height, f.height) + 12 + e.height), B.VOID)
    sh.paste(m, (0, 0))
    sh.paste(B.label(g, "1. Ghost plan and how picks fill it") if False else g, (0, m.height + 12))
    sh.paste(f, (g.width + 12, m.height + 12))
    sh.paste(e, (0, m.height + 24 + max(g.height, f.height)))
    sh.save(out / "battle_layout_take11_sheet.png")


if __name__ == "__main__":
    main()
