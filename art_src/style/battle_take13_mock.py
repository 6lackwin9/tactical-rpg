"""AI-21 battle mock take 13 (Tushar 2026-11-18 22:06), built on takes 11 and 12.

Usage: python3 art_src/style/battle_take13_mock.py <playtest_dir> <out_dir>
Panels: (1) the two foe card modes with poise shown as pips per hit (the kit settles every hit on its own:
rpgkit_battle::poise::pip_loss, pips = clamp(floor(D / T), 0, 3); nothing pools across a block), the party's whole
impact on the selected hero's card (HP preview split by who deals it, KO marker, hits after the KO greyed), and status
chips aligned on the pip bar; (2) status order (soonest to expire first) and duration pips on the tabs, numbers in the list.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take11_mock as T  # noqa: E402
import battle_take12_mock as T12  # noqa: E402

FOE_HP, FOE_MAX, PIPS, TH = 70, 144, 5, B.FOE_T
OTHERS = (120, 45, 40)          # the rest of the party's damage on the bar, darker than this hero's
DIM_AMBER = (120, 80, 36)
GREY = (90, 96, 108)
CARD_W = 300


def lost(stg):
    return min(3, stg // TH)


# (block, who, icon, skill blocks, hp, stagger, chips)
HITS = [(3, "vesper", "bleeding_cut", 1, 17, 26, ("bleed",)),
        (4, "kaelen", "pneumatic_hammer_strike", 3, 34, 40, ()),
        (5, "kaelen", "pommel_strike", 1, 12, 15, ()),
        (5, "vesper", "bleeding_cut", 1, 10, 12, ()),
        (6, "kaelen", "pommel_strike", 1, 12, 15, ())]


def simulate():
    """Walk the hits in order: HP and pips after each, the block the foe breaks in and the one it falls in."""
    hp, pips, out, brk, ko = FOE_HP, PIPS, [], None, None
    for h in HITS:
        blk, _, _, _, dmg, stg, _ = h
        after_ko = ko is not None
        if after_ko:
            out.append((h, 0, None, True))
            continue
        p = 0 if pips <= 0 else min(pips, lost(stg))
        pips -= p
        hp -= dmg
        if pips <= 0 and brk is None:
            brk = blk
        if hp <= 0:
            ko = blk
        out.append((h, p, pips, False))
    return out, brk, ko


def hp_preview(d, x, y, w, me):
    """The HP bar with the plan's damage cut from its right end: this hero's in bright red, the rest darker; once the
    total reaches 0 the whole bar is preview and the KO tag shows."""
    d.rectangle((x, y, x + w, y + 6), fill=(14, 16, 22))
    fill = round(w * FOE_HP / FOE_MAX)
    d.rectangle((x, y, x + fill, y + 6), fill=(150, 50, 45))
    seq, _, _ = simulate()
    e = x + fill
    for (h, _, _, dead) in seq:
        if dead:
            continue
        seg = round(w * h[4] / FOE_MAX)
        s = max(x, e - seg)
        d.rectangle((s, y, e, y + 6), fill=B.IMPACT if h[1] == me or me is None else OTHERS)
        e = s
        if e <= x:
            break


def poise_preview(d, x, y, me):
    """The five pips: lit = left after the plan; amber = this hero breaks them; dim amber = another ally does."""
    seq, _, _ = simulate()
    who = []
    for (h, p, _, dead) in seq:
        who += [h[1]] * p
    for j in range(PIPS):
        k = PIPS - 1 - j                     # pips break from the right
        c = B.POISE
        if k < len(who):
            c = B.AMBER if (me is None or who[k] == me) else DIM_AMBER
        d.rectangle((x + j * 26, y, x + j * 26 + 23, y + 5), fill=c)


def tag(d, x, y, text, col):
    w = d.textlength(text, font=B.SMALL) + 10
    d.rounded_rectangle((x, y, x + w, y + 16), radius=3, fill=col)
    d.text((x + 5, y + 1), text, font=B.SMALL, fill=B.PARCH)
    return w


def header(img, d, shot, cx, cy, me):
    _, brk, ko = simulate()
    fp = shot.crop((1044, 220, 1108, 284))
    img.paste(fp.resize((48, 48)), (cx + 14, cy + 14))
    d.text((cx + 72, cy + 10), "Castellan Footman", font=B.TITLE_S, fill=B.GOLD)
    B.heart(d, cx + 72, cy + 33, 11)
    hp_preview(d, cx + 88, cy + 36, 130, me)
    d.text((cx + 224, cy + 30), f"{FOE_HP}/{FOE_MAX}", font=B.MONO, fill=B.PARCH)
    B.poise_glyph(d, cx + 71, cy + 47, 13)
    poise_preview(d, cx + 88, cy + 51, me)
    B.pip_tag(d, cx + 224, cy + 47, TH)
    x = cx + 14
    x += tag(d, x, cy + 68, f"KO in block {ko}", (130, 30, 30)) + 6
    tag(d, x, cy + 68, f"BREAK in block {brk}", (120, 70, 20))
    return cy + 94


def pip_marks(d, x, y, n, col=B.AMBER):
    for k in range(n):
        d.rectangle((x + k * 9, y, x + k * 9 + 6, y + 6), fill=col)


def card_hero(shot, me="kaelen"):
    seq, brk, ko = simulate()
    mine = [s for s in seq if s[0][1] == me]
    img = Image.new("RGB", (CARD_W + 16, 120 + len(mine) * 30 + 64), B.VOID)
    d = ImageDraw.Draw(img)
    cx, cy = 8, 8
    B.frame_a(d, (cx, cy, cx + CARD_W, img.height - 8), fill=(16, 18, 25))
    y = header(img, d, shot, cx, cy, me) + 4
    for (blk, _, icon, span, hp, stg, chips), p, left, dead in mine:
        col = GREY if dead else B.PARCH
        t = B.slot_icon(icon, 24)
        if dead:
            t.putalpha(t.getchannel("A").point(lambda a: a * 40 // 100))
        img.paste(t, (cx + 14, y), t)
        for b in range(6):
            on = blk - span < b + 1 <= blk
            d.rectangle((cx + 44 + b * 8, y + 10, cx + 44 + b * 8 + 5, y + 14), fill=(GREY if dead else B.GOLD) if on else B.AP_OFF)
        B.heart(d, cx + 98, y + 6, 12, GREY if dead else (200, 80, 70))
        d.text((cx + 114, y + 3), f"-{hp}", font=B.MONO_L, fill=col)
        B.poise_glyph(d, cx + 152, y + 4, 13, GREY if dead else B.POISE)
        if dead:
            tag(d, cx + 196, y + 2, "after KO", (55, 60, 70))
        elif p:
            pip_marks(d, cx + 170, y + 8, p)
            if left == 0 and blk == brk:
                d.text((cx + 202, y + 3), "BREAK", font=B.SMALL, fill=B.AMBER)
        else:
            d.text((cx + 170, y + 3), "broken" if left == 0 else "0", font=B.SMALL, fill=B.IRON_L)
        y += 30
    d.line((cx + 14, y + 2, cx + CARD_W - 14, y + 2), fill=B.IRON)
    my_hp = sum(s[0][4] for s in mine)
    waste = sum(s[0][4] for s in mine if s[3])
    party = sum(s[0][4] for s in seq if not s[3])
    d.text((cx + 14, y + 10), "Kaelen", font=B.SMALL, fill=B.IRON_L)
    B.heart(d, cx + 98, y + 13, 12)
    d.text((cx + 114, y + 10), f"-{my_hp}", font=B.MONO_L, fill=B.GOLD)
    d.text((cx + 152, y + 11), f"({waste} after KO)", font=B.SMALL, fill=B.IRON_L)
    d.text((cx + 14, y + 32), "Party", font=B.SMALL, fill=B.IRON_L)
    B.heart(d, cx + 98, y + 35, 12, OTHERS)
    d.text((cx + 114, y + 32), f"-{party}", font=B.MONO_L, fill=B.PARCH)
    d.text((cx + 152, y + 33), f"KO in block {ko}", font=B.SMALL, fill=B.IMPACT)
    return img


def card_blocks(shot):
    seq, brk, ko = simulate()
    blocks = sorted({s[0][0] for s in seq})
    img = Image.new("RGB", (CARD_W + 16, 120 + len(blocks) * 30 + 44), B.VOID)
    d = ImageDraw.Draw(img)
    cx, cy = 8, 8
    B.frame_a(d, (cx, cy, cx + CARD_W, img.height - 8), fill=(16, 18, 25))
    y = header(img, d, shot, cx, cy, None) + 4
    for blk in blocks:
        rows = [s for s in seq if s[0][0] == blk]
        dead = all(s[3] for s in rows)
        col = GREY if dead else B.PARCH
        d.ellipse((cx + 14, y + 2, cx + 32, y + 20), fill=B.SLATE, outline=GREY if dead else B.GOLD)
        d.text((cx + 19, y + 3), str(blk), font=B.MONO_L, fill=GREY if dead else B.GOLD)
        for k, s in enumerate(rows):
            f = T.face(s[0][1], 18)
            if dead:
                f = Image.blend(f, Image.new("RGB", f.size, (16, 18, 25)), 0.6)
            img.paste(f, (cx + 38 + k * 20, y + 2))
        B.heart(d, cx + 84, y + 6, 12, GREY if dead else (200, 80, 70))
        d.text((cx + 100, y + 3), f"-{sum(s[0][4] for s in rows)}", font=B.MONO_L, fill=col)
        B.poise_glyph(d, cx + 138, y + 4, 13, GREY if dead else B.POISE)
        p = sum(s[1] for s in rows)
        left = [s[2] for s in rows if s[2] is not None]
        mid = y + 11                         # the row's centre line: pips and chips sit on it
        if dead:
            tag(d, cx + 160, mid - 8, "after KO", (55, 60, 70))
        else:
            # one pip mark per hit, split by ally: each hit settles on its own
            px = cx + 156
            for s in rows:
                pip_marks(d, px, mid - 3, s[1])
                px += s[1] * 9 + (4 if s[1] else 0)
            if p == 0:
                d.text((cx + 156, mid - 8), "broken" if left and left[-1] == 0 else "0", font=B.SMALL, fill=B.IRON_L)
            if blk == brk:
                tag(d, cx + 206, mid - 8, "BREAK", (120, 70, 20))
            if blk == ko:
                tag(d, cx + 206, mid - 8, "KO", (130, 30, 30))
            cxs = cx + 250
            for s in rows:
                for st in s[0][6]:
                    cxs += T12.mini(img, d, cxs, mid - 8, st, px=12) + 2
        y += 30
    d.line((cx + 14, y + 2, cx + CARD_W - 14, y + 2), fill=B.IRON)
    party = sum(s[0][4] for s in seq if not s[3])
    d.text((cx + 14, y + 10), "Total", font=B.SMALL, fill=B.IRON_L)
    B.heart(d, cx + 84, y + 13, 12)
    d.text((cx + 100, y + 10), f"-{party}", font=B.MONO_L, fill=B.GOLD)
    d.text((cx + 140, y + 11), f"KO in block {ko}", font=B.SMALL, fill=B.IMPACT)
    return img


def panel_cards(shot):
    a, b = card_hero(shot), card_blocks(shot)
    img = Image.new("RGB", (a.width + b.width + 300, max(a.height, b.height) + 52), B.VOID)
    d = ImageDraw.Draw(img)
    d.text((8, 4), "Kaelen selected: his moves, party on the bars", font=B.SMALL, fill=B.GOLD)
    d.text((8, 20), "bright red / amber = his, dark = the others'", font=B.SMALL, fill=B.IRON_L)
    img.paste(a, (0, 42))
    bx = a.width + 16
    d.text((bx + 8, 4), "No hero selected: every ally, per block", font=B.SMALL, fill=B.GOLD)
    d.text((bx + 8, 20), "one pip mark per hit; chips on the row's centre line", font=B.SMALL, fill=B.IRON_L)
    img.paste(b, (bx, 42))
    nx = bx + b.width + 16
    notes = ["Poise in the code: every hit", "settles on its own, nothing pools", "across a block. A hit breaks",
             "floor(poise / 13) pips, at most 3;", "the rest is lost. So two 10s break", "nothing, one 26 breaks 2.",
             "", "So the cards show pips per hit,", "not summed poise numbers.", "",
             "Foe here at 70/144 HP: the party", "kills it in block 5, so Kaelen's", "block 6 Pommel Strike greys out",
             "as \"after KO\": swap it for", "another target."]
    for i, line in enumerate(notes):
        d.text((nx, 50 + i * 17), line, font=B.SMALL, fill=B.PARCH if i < 6 else B.IRON_L)
    return B.label(img, "2. Foe card: pips per hit, the party's total impact, KO and BREAK markers")


# ---------------------------------------------------------------- 2. status order and duration pips
KAE = [("fortify", 1, False, 1), ("bleed", 1, False, 2), ("weaken", 2, False, 3), ("regen", 2, False, 4),
       ("slow", 3, False, 5), ("haste", 2, True, 6)]   # (status, rounds left, incoming, order applied)


def ordered(sts):
    """Soonest to run out first; ties by the one applied first; statuses a planned move adds (dashed) last."""
    return sorted(sts, key=lambda s: (s[2], s[1], s[3]))


def round_pips(d, x, y, n, col, h=4):
    """Rounds left as up to 3 stacked pips beside the icon (the data's longest is 3); a permanent one is a full bar."""
    if n < 0:
        d.rectangle((x, y, x + 3, y + 3 * (h + 2) - 2), fill=col)
        return
    for k in range(3):
        yy = y + (2 - k) * (h + 2)
        d.rectangle((x, yy, x + 3, yy + h - 1), fill=col if k < n else B.AP_OFF)


def tabs(img, d, x, y, sts, shown=3):
    o = ordered(sts)
    rest = len(o) - shown
    items = o[:shown] + ([None] if rest > 0 else [])
    for i, it in enumerate(items):
        ty = y + 2 + i * 20
        d.rectangle((x, ty, x + 26, ty + 18), fill=(16, 18, 25), outline=B.IRON)
        if it is None:
            d.text((x + 4, ty + 2), f"+{rest}", font=B.MONO, fill=B.PARCH)
            continue
        st, n, inc, _ = it
        T12.mini(img, d, x + 2, ty + 1, st, incoming=inc, px=12)
        round_pips(d, x + 20, ty + 2, n, T12.rim(T12.KIND[st]))


def popover(img, d, x, y, title, sts):
    o = ordered(sts)
    h = 36 + len(o) * 26 + 8
    B.frame_a(d, (x, y, x + 290, y + h), fill=(16, 18, 25))
    d.text((x + 12, y + 8), title, font=B.TITLE_S, fill=B.GOLD)
    d.text((x + 236, y + 12), "rounds", font=B.SMALL, fill=B.IRON_L)
    for i, (st, n, inc, _) in enumerate(o):
        ry = y + 36 + i * 26
        T12.mini(img, d, x + 12, ry, st, incoming=inc, px=14)
        d.text((x + 36, ry + 1), st.capitalize(), font=B.SMALL, fill=B.PARCH)
        d.text((x + 100, ry + 1), T12.LABEL.get(st, ""), font=B.SMALL, fill=B.IRON_L)
        d.text((x + 256, ry + 1), "new" if inc else str(n), font=B.MONO, fill=B.IRON_L if inc else B.PARCH)
    return h


def panel_status(shot):
    img = Image.new("RGB", (1000, 262), B.VOID)
    d = ImageDraw.Draw(img)
    kae = T.face("kaelen", 48)
    d.rectangle((8, 24, 160, 108), fill=B.SLATE, outline=B.GOLD)
    img.paste(kae, (14, 30))
    B.intent_tile(d, 132, 30)
    d.text((68, 30), "Kaelen", font=B.BODY, fill=B.PARCH)
    B.bar(d, 68, 54, 86, 6, 180 / 242, B.HP)
    d.text((68, 63), "180/242", font=B.MONO, fill=B.PARCH)
    B.ap_pips(d, 68, 80, 10, 10, size=6, gap=2, spent=6)
    B.poise(d, 14, 96, 5, 4, w=26, h=4)
    tabs(img, d, 161, 24, KAE)
    d.text((8, 4), "Tabs: icon + rounds left as pips", font=B.SMALL, fill=B.GOLD)
    ph = popover(img, d, 210, 24, "Kaelen: 6 effects", KAE)
    d.text((210, 24 + ph + 6), "The list keeps numbers; the tabs use pips", font=B.SMALL, fill=B.IRON_L)
    lx = 530
    notes = [("Order (recommended): the soonest to run out first,", B.PARCH),
             ("so the top tabs are the ones about to change;", B.PARCH),
             ("ties go to the one applied first (first in, first out).", B.PARCH),
             ("Statuses a planned move adds come last, dashed, marked new.", B.PARCH),
             ("", B.PARCH),
             ("Rounds: the shipped data lasts 1 to 3 rounds (skills,", B.IRON_L),
             ("items, hazards), so 3 pips cover it. A few passives", B.IRON_L),
             ("and meters are permanent: a full bar instead of pips.", B.IRON_L),
             ("", B.PARCH),
             ("Other orders I considered: newest first (what you just", B.IRON_L),
             ("did on top, but the expiring ones sink), debuffs first", B.IRON_L),
             ("(take 12; hides a buff about to run out).", B.IRON_L)]
    for i, (line, c) in enumerate(notes):
        d.text((lx, 26 + i * 17), line, font=B.SMALL, fill=c)
    # permanent example
    d.text((8, 124), "Permanent (a passive):", font=B.SMALL, fill=B.IRON_L)
    d.rectangle((8, 142, 34, 160), fill=(16, 18, 25), outline=B.IRON)
    T12.mini(img, d, 10, 143, "steadfast", px=12)
    round_pips(d, 28, 144, -1, T.BUFF)
    return B.label(img, "3. Statuses: soonest to expire first, rounds as pips on the tabs")


# ---------------------------------------------------------------- 3. tiles in the skill kind's colour, ghost = faded
KIND_COL = {"ATTACK": (160, 56, 56), "MOVE": (52, 92, 160), "DEFEND": (40, 122, 122), "SUPPORT": (52, 128, 68),
            "FOCUS": (128, 98, 34), "REST": (74, 80, 96), "STATUS": (104, 56, 128)}     # tips.rs kind_color
SKILL_KIND = {"arrow": "MOVE", "pommel_strike": "ATTACK", "pneumatic_hammer_strike": "ATTACK", "bleeding_cut": "ATTACK",
              "shield_wall": "DEFEND", "rally": "SUPPORT", "greater_mend": "SUPPORT", "intent:rest": "REST"}
GHOST = 0.45                                   # aegis_ui::timeline::GHOST_ALPHA


def mix(a, b, t):
    return tuple(round(a[i] * t + b[i] * (1 - t)) for i in range(3))


def tile13(img, d, x, icon, blocks, ap, ghost=False, on=False, chips=()):
    W, G, y0 = T.W, T.G, T12.OY
    w = blocks * W + (blocks - 1) * G
    base = KIND_COL[SKILL_KIND[icon]]
    fill = mix(base, B.BASALT, GHOST) if ghost else base
    d.rectangle((x, y0 + 3, x + w, y0 + 48), fill=fill)
    if on:
        d.rectangle((x, y0 + 3, x + w, y0 + 48), outline=B.GOLD, width=2)
    t = B.slot_icon(icon, 28)
    ix, iy = x + (w - 28) // 2, y0 + 6
    light = (238, 230, 210)
    d.rectangle((ix - 1, iy - 1, ix + 28, iy + 28), outline=mix(light, fill, GHOST) if ghost else light)
    if ghost:
        t.putalpha(t.getchannel("A").point(lambda a: round(a * 150 / 255)))     # BLOCK_GHOST_ALPHA
    img.paste(t, (ix, iy), t)
    if ap:
        pw = ap * 10 - 2
        for i in range(ap):
            px = x + (w - pw) // 2 + i * 10
            if ghost:
                d.rectangle((px, y0 + 38, px + 7, y0 + 45), outline=mix(B.AP_ON, fill, 0.7))
            else:
                d.rectangle((px, y0 + 38, px + 7, y0 + 45), fill=B.AP_ON, outline=(12, 12, 16))
    for b in range(blocks):
        bx = x + b * (W + G)
        c = mix(base, B.BASALT, GHOST) if ghost else (B.GOLD if on else B.IRON_L)
        d.rectangle((bx, y0 + 55, bx + W, y0 + 60), fill=c)
        if b < blocks - 1:
            d.rectangle((bx + W, y0 + 57, bx + W + G, y0 + 58), fill=c)
    cx = x + (w - (len(chips) * 17 - 1)) // 2
    for st in chips:
        cx += T12.mini(img, d, cx, y0 - 5, st, px=12)
    return x + w + G


def strip13(plan, title):
    img = Image.new("RGB", (560, 80), B.BASALT)
    d = ImageDraw.Draw(img)
    x = 12
    used = ghost_ap = 0
    for p in plan:
        icon, blocks, ap, ghost = p[:4]
        x = tile13(img, d, x, icon, blocks, ap, ghost=ghost, on=p[4] if len(p) > 4 else False, chips=p[5] if len(p) > 5 else ())
        if ghost:
            ghost_ap += ap
        else:
            used += ap
    ax = x + 14
    d.text((ax, 30), "AP", font=B.MONO_L, fill=B.GOLD)
    d.text((ax + 24, 30), "10/12", font=B.MONO_L, fill=B.PARCH)
    d.text((ax + 70, 30), (f"-{used} " if used else "") + (f"(-{ghost_ap})" if ghost_ap else ""), font=B.MONO_L, fill=B.PLANNED)
    return img, title


def panel_kinds():
    rows = [strip13([("arrow", 1, 1, True), ("pneumatic_hammer_strike", 3, 5, True), ("pommel_strike", 1, 2, True),
                     ("pommel_strike", 1, 2, True)], "No picks: the ghost plan, faded kind colours"),
            strip13([("arrow", 1, 1, False), ("bleeding_cut", 1, 2, False, True, ("bleed",)), ("pneumatic_hammer_strike", 3, 5, True),
                     ("intent:rest", 1, 0, True)], "2 picks turn solid, the rest stays faded"),
            strip13([("arrow", 1, 1, False), ("bleeding_cut", 1, 2, False, False, ("bleed",)), ("shield_wall", 2, 2, False, True, ("fortify",)),
                     ("greater_mend", 1, 3, False, False, ("heal",)), ("rally", 1, 1, False, False, ("haste",))],
                    "Mixed round: attack, move, defend, support")]
    img = Image.new("RGB", (1000, len(rows) * 84 + 8), B.VOID)
    d = ImageDraw.Draw(img)
    for i, (s, t) in enumerate(rows):
        img.paste(s, (0, i * 84))
        d.text((574, i * 84 + 30), t, font=B.SMALL, fill=B.PARCH)
    return B.label(img, "1. Timeline tiles in the skill kind's colour; the ghost plan is the faded colour, no dashes")


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    a, b, c = panel_cards(shot), panel_status(shot), panel_kinds()
    a.save(out / "take13_cards.png"); b.save(out / "take13_status.png"); c.save(out / "take13_kinds.png")
    sh = Image.new("RGB", (max(a.width, b.width, c.width), a.height + b.height + c.height + 24), B.VOID)
    sh.paste(c, (0, 0)); sh.paste(a, (0, c.height + 12)); sh.paste(b, (0, c.height + a.height + 24))
    sh.save(out / "battle_layout_take13_sheet.png")


if __name__ == "__main__":
    main()
