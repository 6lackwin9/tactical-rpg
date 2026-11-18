"""AI-21 battle mock take 14 (Tushar 2026-11-18 22:21), built on take 13.

Usage: python3 art_src/style/battle_take14_mock.py <playtest_dir> <out_dir>
The two foe cards again, with: a fixed-width poise area (3 pip slots, the per-hit cap) where a hit that empties the bar
shows a broken-poise icon in place, and an already broken foe a dim one; status chips right after the damage, at most 3
then +N; hits after the forecast KO kept valid with a soft "after KO" hint (the forecast ignores crits, moves and
enemy reactions); crits: base damage stays the number, a star marks a hero whose crit chance is high, the hover gives
the crit damage (x1.5, rpgkit_rules::damage::CRIT_MULT) and the header says when a crit would kill a block sooner.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take11_mock as T  # noqa: E402
import battle_take12_mock as T12  # noqa: E402
import battle_take13_mock as T13  # noqa: E402

T13.FOE_HP = 64
T13.HITS = [(3, "vesper", "bleeding_cut", 1, 17, 26, ("bleed", "expose")),
            (4, "kaelen", "pneumatic_hammer_strike", 3, 34, 40, ()),
            (5, "kaelen", "pommel_strike", 1, 12, 15, ("taunt",)),
            (5, "vesper", "bleeding_cut", 1, 10, 12, ("bleed", "weaken", "slow")),
            (6, "kaelen", "pommel_strike", 1, 12, 15, ())]
CRIT = {"kaelen": 8, "vesper": 24}          # crit chance %, the hero's derived crit rate
HIGH_CRIT = 20                              # at or above it the row shows the star
CRIT_MULT = 1.5
CARD_W = 318
OVERKILL = (150, 120, 70)


def crit_ko():
    """The block the foe falls in if the first hit that can crit does (each hit tried alone): the earliest."""
    best = None
    for i in range(len(T13.HITS)):
        hp = T13.FOE_HP
        for j, h in enumerate(T13.HITS):
            hp -= round(h[4] * CRIT_MULT) if j == i else h[4]
            if hp <= 0:
                best = h[0] if best is None else min(best, h[0])
                break
    return best


def broken_icon(d, x, y, s=14, col=B.AMBER, dim=False):
    """Broken poise: the poise shield split in two, filled."""
    c = (90, 70, 40) if dim else col
    h = s
    left = [(x, y), (x + s * 0.45, y), (x + s * 0.35, y + h * 0.4), (x + s * 0.5, y + h * 0.65), (x + s * 0.45, y + h),
            (x, y + h * 0.55)]
    right = [(x + s * 0.6 + 2, y), (x + s + 2, y), (x + s + 2, y + h * 0.55), (x + s * 0.55 + 2, y + h),
             (x + s * 0.6 + 2, y + h * 0.65), (x + s * 0.45 + 2, y + h * 0.4)]
    d.polygon(left, fill=c)
    d.polygon(right, fill=c)


def poise_cell(d, x, y, p, left, brk_here):
    """The fixed poise area: 3 slots wide. Pips the hit breaks; the broken icon when it empties the bar; a dim broken
    icon when the foe is already broken."""
    for k in range(3):
        d.rectangle((x + k * 9, y + 5, x + k * 9 + 6, y + 11), fill=B.AP_OFF)
    if brk_here:
        d.rectangle((x - 1, y, x + 27, y + 16), fill=(16, 18, 25))
        broken_icon(d, x + 5, y + 1, 14)
    elif p == 0 and left == 0:
        d.rectangle((x - 1, y, x + 27, y + 16), fill=(16, 18, 25))
        broken_icon(d, x + 5, y + 1, 14, dim=True)
    else:
        for k in range(p):
            d.rectangle((x + k * 9, y + 5, x + k * 9 + 6, y + 11), fill=B.AMBER)


def chips(img, d, x, y, sts, cap=3):
    """At most 3 chips, then +N (the hover or the hero view lists them all)."""
    for st in sts[:cap]:
        x += T12.mini(img, d, x, y, st, px=12) + 2
    if len(sts) > cap:
        d.rectangle((x, y, x + 18, y + 16), fill=B.VOID, outline=B.IRON_L)
        d.text((x + 2, y + 1), f"+{len(sts) - cap}", font=B.TINY if hasattr(B, "TINY") else T12.TINY, fill=B.PARCH)


def star(d, x, y, s=9, col=B.GOLD):
    pts = []
    import math
    for k in range(10):
        r = s / 2 if k % 2 == 0 else s / 5
        a = -math.pi / 2 + k * math.pi / 5
        pts.append((x + s / 2 + r * math.cos(a), y + s / 2 + r * math.sin(a)))
    d.polygon(pts, fill=col)


def header(img, d, shot, cx, cy, me):
    _, brk, ko = T13.simulate()
    fp = shot.crop((1044, 220, 1108, 284))
    img.paste(fp.resize((48, 48)), (cx + 14, cy + 14))
    d.text((cx + 72, cy + 10), "Castellan Footman", font=B.TITLE_S, fill=B.GOLD)
    B.heart(d, cx + 72, cy + 33, 11)
    T13.hp_preview(d, cx + 88, cy + 36, 130, me)
    d.text((cx + 228, cy + 30), f"{T13.FOE_HP}/{T13.FOE_MAX}", font=B.MONO, fill=B.PARCH)
    if brk:
        broken_icon(d, cx + 70, cy + 47, 13)         # the plan breaks it: the poise glyph turns broken
    else:
        B.poise_glyph(d, cx + 71, cy + 47, 13)
    T13.poise_preview(d, cx + 88, cy + 51, me)
    B.pip_tag(d, cx + 228, cy + 47, T13.TH)
    x = cx + 14
    x += T13.tag(d, x, cy + 68, f"KO in block {ko}", (130, 30, 30)) + 4
    ck = crit_ko()
    if ck and ck < ko:
        d.rounded_rectangle((x, cy + 68, x + 70, cy + 84), radius=3, outline=(130, 30, 30))
        star(d, x + 5, cy + 72, 9)
        d.text((x + 17, cy + 69), f"crit: {ck}", font=B.SMALL, fill=B.PARCH)
        x += 74
    T13.tag(d, x, cy + 68, f"BREAK in block {brk}", (120, 70, 20))
    return cy + 94


def hint_overkill(d, x, y):
    d.rounded_rectangle((x, y, x + 58, y + 16), radius=3, outline=OVERKILL)
    d.text((x + 5, y + 1), "after KO", font=B.SMALL, fill=OVERKILL)


def row_hp(d, x, y, hp, who):
    B.heart(d, x, y + 3, 12)
    d.text((x + 16, y), f"-{hp}", font=B.MONO_L, fill=B.PARCH)
    if CRIT[who] >= HIGH_CRIT:
        star(d, x + 44, y - 1, 9)


def card_hero(shot, me="kaelen"):
    seq, brk, ko = T13.simulate()
    mine = [s for s in seq if s[0][1] == me]
    img = Image.new("RGB", (CARD_W + 16, 120 + len(mine) * 30 + 64), B.VOID)
    d = ImageDraw.Draw(img)
    cx, cy = 8, 8
    B.frame_a(d, (cx, cy, cx + CARD_W, img.height - 8), fill=(16, 18, 25))
    y = header(img, d, shot, cx, cy, me) + 4
    for (blk, who, icon, span, hp, stg, sts), p, left, after in mine:
        t = B.slot_icon(icon, 24)
        img.paste(t, (cx + 14, y), t)
        for b in range(6):
            on = blk - span < b + 1 <= blk
            d.rectangle((cx + 44 + b * 8, y + 10, cx + 44 + b * 8 + 5, y + 14), fill=B.GOLD if on else B.AP_OFF)
        row_hp(d, cx + 98, y + 3, hp, who)
        mid = y + 11
        if after:
            hint_overkill(d, cx + 156, mid - 8)
        else:
            poise_cell(d, cx + 156, mid - 8, p, left, blk == brk and left == 0 and p > 0)
            chips(img, d, cx + 190, mid - 8, list(sts))
        y += 30
    d.line((cx + 14, y + 2, cx + CARD_W - 14, y + 2), fill=B.IRON)
    my_hp = sum(s[0][4] for s in mine)
    party = sum(s[0][4] for s in seq if not s[3])
    d.text((cx + 14, y + 10), "Kaelen", font=B.SMALL, fill=B.IRON_L)
    B.heart(d, cx + 98, y + 13, 12)
    d.text((cx + 114, y + 10), f"-{my_hp}", font=B.MONO_L, fill=B.GOLD)
    d.text((cx + 14, y + 32), "Party", font=B.SMALL, fill=B.IRON_L)
    B.heart(d, cx + 98, y + 35, 12, T13.OTHERS)
    d.text((cx + 114, y + 32), f"-{party}", font=B.MONO_L, fill=B.PARCH)
    d.text((cx + 156, y + 33), f"KO in block {ko}", font=B.SMALL, fill=B.IMPACT)
    return img


def card_blocks(shot):
    seq, brk, ko = T13.simulate()
    blocks = sorted({s[0][0] for s in seq})
    img = Image.new("RGB", (CARD_W + 16, 120 + len(blocks) * 30 + 44), B.VOID)
    d = ImageDraw.Draw(img)
    cx, cy = 8, 8
    B.frame_a(d, (cx, cy, cx + CARD_W, img.height - 8), fill=(16, 18, 25))
    y = header(img, d, shot, cx, cy, None) + 4
    for blk in blocks:
        rows = [s for s in seq if s[0][0] == blk]
        after = all(s[3] for s in rows)
        d.ellipse((cx + 14, y + 2, cx + 32, y + 20), fill=B.SLATE, outline=B.GOLD)
        d.text((cx + 19, y + 3), str(blk), font=B.MONO_L, fill=B.GOLD)
        for k, s in enumerate(rows):
            img.paste(T.face(s[0][1], 18), (cx + 38 + k * 20, y + 2))
        B.heart(d, cx + 84, y + 6, 12)
        d.text((cx + 100, y + 3), f"-{sum(s[0][4] for s in rows)}", font=B.MONO_L, fill=B.PARCH)
        if any(CRIT[s[0][1]] >= HIGH_CRIT for s in rows):
            star(d, cx + 128, y + 2, 9)
        mid = y + 11
        if after:
            hint_overkill(d, cx + 146, mid - 8)
        else:
            p = sum(s[1] for s in rows)
            left = [s[2] for s in rows if s[2] is not None]
            poise_cell(d, cx + 146, mid - 8, p, left[-1] if left else 0, blk == brk)
            sts = [st for s in rows for st in s[0][6]]
            chips(img, d, cx + 180, mid - 8, sts)
            if blk == ko:
                T13.tag(d, cx + 268, mid - 8, "KO", (130, 30, 30))
        y += 30
    d.line((cx + 14, y + 2, cx + CARD_W - 14, y + 2), fill=B.IRON)
    party = sum(s[0][4] for s in seq if not s[3])
    d.text((cx + 14, y + 10), "Total", font=B.SMALL, fill=B.IRON_L)
    B.heart(d, cx + 84, y + 13, 12)
    d.text((cx + 100, y + 10), f"-{party}", font=B.MONO_L, fill=B.GOLD)
    d.text((cx + 146, y + 11), f"KO in block {ko}", font=B.SMALL, fill=B.IMPACT)
    return img


def hover_tip():
    img = Image.new("RGB", (250, 112), B.VOID)
    d = ImageDraw.Draw(img)
    B.frame_a(d, (4, 4, 246, 108), fill=(16, 18, 25))
    d.text((14, 12), "Bleeding Cut (Vesper)", font=B.TITLE_S, fill=B.GOLD)
    B.heart(d, 14, 40, 12)
    d.text((30, 37), "-17", font=B.MONO_L, fill=B.PARCH)
    star(d, 70, 38, 11)
    d.text((86, 37), "crit -26", font=B.MONO_L, fill=B.GOLD)
    d.text((160, 38), "24%", font=B.MONO, fill=B.IRON_L)
    d.text((14, 60), "Poise 26: breaks 2 pips", font=B.SMALL, fill=B.IRON_L)
    d.text((14, 78), "Applies Bleed 3, Expose 2", font=B.SMALL, fill=B.IRON_L)
    return img


def legend():
    img = Image.new("RGB", (250, 150), B.VOID)
    d = ImageDraw.Draw(img)
    for k in range(3):
        d.rectangle((10 + k * 9, 15, 16 + k * 9, 21), fill=B.AMBER if k < 2 else B.AP_OFF)
    d.text((48, 9), "pips this hit breaks", font=B.SMALL, fill=B.PARCH)
    broken_icon(d, 13, 34, 14)
    d.text((48, 34), "this hit breaks the foe", font=B.SMALL, fill=B.PARCH)
    broken_icon(d, 13, 59, 14, dim=True)
    d.text((48, 59), "already broken: no pips", font=B.SMALL, fill=B.PARCH)
    star(d, 14, 86, 11)
    d.text((48, 84), "high crit chance (20%+)", font=B.SMALL, fill=B.PARCH)
    hint_overkill(d, 8, 110)
    d.text((76, 110), "still valid, a hint", font=B.SMALL, fill=B.PARCH)
    return img


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    a, b, tip, lg = card_hero(shot), card_blocks(shot), hover_tip(), legend()
    img = Image.new("RGB", (a.width + b.width + 290, max(a.height, b.height) + 60), B.VOID)
    d = ImageDraw.Draw(img)
    d.text((8, 4), "Kaelen selected", font=B.SMALL, fill=B.GOLD)
    d.text((8, 20), "his moves; the bars show the party", font=B.SMALL, fill=B.IRON_L)
    img.paste(a, (0, 42))
    bx = a.width + 12
    d.text((bx + 8, 4), "No hero selected", font=B.SMALL, fill=B.GOLD)
    d.text((bx + 8, 20), "every ally per block; 4 chips -> 3 and +1", font=B.SMALL, fill=B.IRON_L)
    img.paste(b, (bx, 42))
    nx = bx + b.width + 14
    d.text((nx + 8, 4), "Hover a row", font=B.SMALL, fill=B.GOLD)
    img.paste(tip, (nx, 26))
    img.paste(lg, (nx, 150))
    out_img = B.label(img, "Take 14: fixed poise area with a broken icon, chips after the damage, overkill hint, crit marker")
    out_img.save(out / "battle_layout_take14_sheet.png")


if __name__ == "__main__":
    main()
