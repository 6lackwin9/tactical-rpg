"""AI-35 battle mock take 27 (Tushar 2026-11-19 18:24 on take 26), 1280x720 at 1:1.

Usage: python3 art_src/style/battle_take27_boss_mock.py <playtest_dir> <out_dir>
  - phase markers flipped: every phase still to fight is lit gold from the start, the current one is lit and ringed,
    a beaten one goes dim (so a fight opens showing all its phases);
  - poise readable at a glance: the 10 pips in two groups of 5 with the count beside them ("6/10"), then the per-pip tag;
  - the boss intent drawer: while a hero targets the boss (or the boss is pinned), a drawer opens under the boss bar
    in place of the foe card: one lane per party member with their hits by block (HP and poise pips), the selected
    hero's lane ringed, and the party total lane with the boss's span and poise left after each block (BREAK and the
    next phase called out where they land). It only covers the board's back rows; a unit standing there pushes the
    camera down by the drawer's height (rule for Build), so it never hides a unit.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take12_mock as T12  # noqa: E402
import battle_take17_mock as T17  # noqa: E402
import battle_take24_mock as T24  # noqa: E402
import battle_take26_boss_mock as T26  # noqa: E402

LIT, LIT_D = (251, 191, 36), (120, 84, 24)
DONE = (52, 58, 70)
POISE_ON = {}
# the planned round on the boss: hero -> {block: (HP, poise pips)}
HITS = {"kaelen": {1: (34, 2), 6: (14, 0)}, "vesper": {2: (22, 0), 5: (18, 1)}, "mara": {2: (10, 0)},
        "dunstan": {3: (40, 3), 4: (12, 0)}}
NAMES = {"kaelen": "Kaelen", "vesper": "Vesper", "mara": "Mara", "dunstan": "Dunstan"}
HITS["mara_voss"] = HITS.pop("mara")
NAMES["mara_voss"] = NAMES.pop("mara")


def markers(d, x, y, n, cur, s=10):
    """Flipped (user 18:24): coming phases lit gold, the current one lit with a ring, beaten ones dim."""
    for i in range(n):
        cx, cy = x + i * (s + 5) + s // 2, y + s // 2
        pts = [(cx, cy - s // 2), (cx + s // 2, cy), (cx, cy + s // 2), (cx - s // 2, cy)]
        if i < cur:
            d.polygon(pts, fill=(22, 25, 33), outline=DONE)
        elif i == cur:
            ring = [(cx, cy - s // 2 - 3), (cx + s // 2 + 3, cy), (cx, cy + s // 2 + 3), (cx - s // 2 - 3, cy)]
            d.polygon(ring, outline=(255, 236, 170))
            d.polygon(pts, fill=(255, 220, 110), outline=LIT)
        else:
            d.polygon(pts, fill=LIT_D, outline=LIT)
    return x + n * (s + 5)


def poise_grouped(d, x, y, n, on, w=14, h=4, gap=2):
    """Two groups of 5 (a gap between), so 10 reads at a glance; the count follows (drawn with the tag)."""
    POISE_ON["on"], POISE_ON["n"] = on, n
    pw = 6
    for i in range(n):
        gx = x + i * (pw + 2) + (i // 5) * 5
        d.rectangle((gx, y - 1, gx + pw - 1, y + h), fill=B.POISE if i < on else B.AP_OFF)


ORIG_TAG = B.pip_tag


def count_and_tag(d, x, y, t):
    d.text((x - 28, y + 2), f"{POISE_ON.get('on', 0)}/{POISE_ON.get('n', 10)}", font=T12.TINY, fill=B.POISE)
    ORIG_TAG(d, x + 9, y, t)


EXTRA = 58


def boss_bar(img, x0, x1, y, intent=False, **kw):
    """The take 26 bar with the flipped markers and grouped poise; `intent` grows it by two rows (the boss intent)."""
    T26.markers = markers
    op, ot, on, osh = B.poise, B.pip_tag, T24.nine, T24.shadow
    B.poise, B.pip_tag = poise_grouped, count_and_tag
    if intent:
        def grow(box):
            return (box[0], box[1], box[2], box[3] + EXTRA) if box[1] == y and box[3] - box[1] == 84 else box
        T24.nine = lambda im, box, name, scale=2: on(im, grow(box), name, scale)
        T24.shadow = lambda im, box, off=3: osh(im, grow(box), off)
    try:
        yb = T26.boss_bar(img, x0, x1, y, **kw)
    finally:
        B.poise, B.pip_tag, T24.nine, T24.shadow = op, ot, on, osh
    if intent:
        yb = intent_rows(img, x0, x1, y + 82)
    return yb


def chip(img, d, x, y, k, hp, pp, on):
    w = 40 + 9 * pp
    d.rectangle((x, y, x + w, y + 17), fill=(52, 40, 22) if on else (36, 22, 24), outline=B.GOLD if on else (70, 40, 40))
    img.paste(T17.T.face(k, 15), (x + 1, y + 1))
    d.text((x + 18, y + 1), f"-{hp}", font=B.MONO, fill=B.IMPACT)
    for p in range(pp):
        B.poise_glyph(d, x + 39 + p * 9, y + 4, 8)
    return w


def intent_rows(img, x0, x1, y, sel="kaelen", span_left=160, pips=6):
    """BOSSUI1 section 5: two rows in the bar. Hits: every planned hit on the boss by block, the selected hero's
    ringed. After: the phase span and poise left after each block, BREAK where it lands; totals at the right."""
    d = ImageDraw.Draw(img)
    d.line((x0 + 10, y, x1 - 10, y), fill=(52, 58, 70))
    tx, sw = x0 + 76, 170
    cw = (x1 - 14 - sw - tx) // 6
    d.text((x0 + 12, y + 7), "Hits", font=B.SMALL, fill=B.IRON_L)
    d.text((x0 + 12, y + 29), "After", font=B.SMALL, fill=B.IRON_L)
    left, pl, brk = span_left, pips, False
    for b in range(6):
        bx = tx + b * cw
        d.text((bx, y + 2), str(b + 1), font=T12.TINY, fill=(90, 98, 112))
        cx = bx + 6
        for k in HITS:
            if b + 1 in HITS[k]:
                hp, pp = HITS[k][b + 1]
                cx += chip(img, d, cx, y + 4, k, hp, pp, k == sel) + 3
                left, pl = left - hp, max(0, pl - pp)
        B.bar(d, bx + 10, y + 29, cw - 46, 5, left / span_left, (150, 50, 45))
        d.text((bx + cw - 32, y + 25), str(left), font=T12.TINY, fill=B.PARCH)
        for p in range(pips):
            d.rectangle((bx + 10 + p * 7, y + 38, bx + 14 + p * 7, y + 41), fill=B.POISE if p < pl else B.AP_OFF)
        if pl == 0 and not brk:
            brk = True
            d.rounded_rectangle((bx + 56, y + 35, bx + cw - 4, y + 46), radius=2, fill=(80, 50, 10), outline=B.AMBER)
            d.text((bx + 59, y + 35), "BREAK", font=T12.TINY, fill=B.AMBER)
        if b:
            d.line((bx + 2, y + 4, bx + 2, y + 44), fill=(40, 45, 56))
    sx = x1 - 14 - sw + 10
    d.line((sx - 6, y + 4, sx - 6, y + 44), fill=B.IRON)
    mine = sum(v[0] for v in HITS[sel].values())
    d.text((sx, y + 5), f"{NAMES[sel]} -{mine}", font=B.SMALL, fill=B.GOLD)
    d.text((sx, y + 26), f"Party -{span_left - left}", font=B.SMALL, fill=B.PARCH)
    d.text((sx + 92, y + 27), f"{left} to ph. 2", font=T12.TINY, fill=B.GOLD)
    return y + EXTRA


def screen(shot, ph=0, with_intent=True):
    T24.boss_bar = lambda img, x0, x1, y: boss_bar(img, x0, x1, y, intent=with_intent, ph=ph)
    if not with_intent:
        return T24.screen(shot)
    keep = T24.T14.card_hero
    T24.T14.card_hero = lambda s: Image.new("RGB", (13, 13), (20, 23, 31))
    img = T24.screen(shot)
    T24.T14.card_hero = keep
    # the boss is the target, so the foe card stays closed: the shot's own card is painted out from the board
    img.paste(img.crop((330, 204, 706, 330)), (700, 204))
    boss_bar(img, 212, 1068, 74, intent=True, ph=ph)
    return img


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    T26.PHASES[0] = T26.PHASES[0][:7] + (150,) + T26.PHASES[0][8:]   # the bar's planned loss = the drawer's total
    s = screen(shot)
    s.save(out / "battle_take27_boss_intent_720p.png")
    s.crop((204, 66, 1076, 216)).resize((872 * 2, 300), Image.NEAREST).save(out / "battle_take27_boss_intent_2x.png")
    # contract 5 rule at 720p: the foe card docks top right unless it would cover the acting unit or its target
    # (the Castellan Footman's helmet is under the take 26 card); then it docks top left of the board
    T24.boss_bar = lambda img, x0, x1, y: boss_bar(img, x0, x1, y, ph=0)
    keep = T24.T14.card_hero
    card = keep(shot)
    T24.T14.card_hero = lambda s: Image.new("RGB", (13, 13), (20, 23, 31))
    m = T24.screen(shot)
    T24.T14.card_hero = keep
    m.paste(m.crop((330, 160, 706, 318)), (700, 160))
    c = card.crop((6, 6, card.width - 6, card.height - 6))
    m.paste(c, (216, 166))
    m.save(out / "battle_take27_card_clear_720p.png")
    # the markers through the fight, and the grouped poise, 2x
    rows = []
    for ph, lab in ((0, "Start: phase 1 lit and ringed, phases 2 and 3 lit"), (1, "Phase 1 beaten: its marker dims"),
                    (2, "Phase 3")):
        base = T24.screen.__globals__["B"].mock10(shot)
        boss_bar(base, 212, 1068, 74, ph=ph)
        rows.append((lab, base.crop((204, 66, 760, 166)).resize((556 * 2, 200), Image.NEAREST)))
    T26.sheet(rows, w=1160, title="Take 27: phase markers flipped, poise in two fives with the count, 2x").save(
        out / "battle_take27_markers_2x.png")


if __name__ == "__main__":
    main()
