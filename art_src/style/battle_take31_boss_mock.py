"""AI-37 battle mock take 31 (Tushar 2026-11-19 19:02 and 19:03 on take 30), 1280x720 at 1:1.

Usage: python3 art_src/style/battle_take31_boss_mock.py <playtest_dir> <out_dir>
  - take 31: every impact cell has fixed slots, so nothing shifts with the numbers: HP (5 mono chars, right
    aligned), poise (the shield glyph and 2 digits), statuses (one chip and "+N"; the Total column three chips and
    "+N"; the hover's Total four); an empty slot stays empty; numbers past 999 use K (1.2K, 12K, 120K) and past
    999K use M, so HP never needs more than "-999" / "-1.2K";
  - the intent rows show only while planning (a hero selected, the plan not yet confirmed); in resolution and the
    enemy turn the bar folds back to the 84 px boss bar, so the board loses 58 px only while planning;
  - the phase break note moves to the Total header ("Ends block N, ph. 2 next round") so the totals keep their slots.
Take 30 notes:
  - take 30: the boss poise drops the 5 + 5 split: (A, recommended) one continuous bar with a faint tick per pip and
    the count "6/10", read like the plate AP bar; (B) ten even pips with the count; the cells write poise as one
    shield glyph and the pips taken as a number ("2") instead of stacked shields; a cell shows at most one status
    chip and "+N"; the party row: (P1, recommended) the whole party in one row, hover or press it for one line per
    member; (P2) the selected hero and "Others" (the rest summed), the total column adds both.
Take 29 notes:
  - each block cell shows the impact, as the small foe intent does: HP, poise pips taken, the statuses it lands (no
    skill names: the skills are on the strip above);
  - phase break (user 18:38, replaces BU14 carry): when the planned hits take the phase span to 0 in a block, the
    phase breaks there and the round ends: the later blocks are cancelled (struck, no AP spent), nothing carries,
    and the next phase starts next round. The party row says so in the breaking block; the totals count only what
    lands.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take12_mock as T12  # noqa: E402
import battle_take14_mock as T14  # noqa: E402
import battle_take17_mock as T17  # noqa: E402
import battle_take26_boss_mock as T26  # noqa: E402
import battle_take27_boss_mock as T27  # noqa: E402

DIM = (70, 76, 90)
GONE = (58, 62, 74)
# statuses a hit lands: (hero, block) -> statuses
LANDS = {("vesper", 2): ["bleed"], ("mara_voss", 2): ["weaken"], ("dunstan", 3): ["stagger"]}


def fmt(n):
    """HP in at most 4 chars: 999, 1.2K, 12K, 120K, 1.2M (user 19:03)."""
    for div, suf in ((1_000_000, "M"), (1_000, "K")):
        if n >= div:
            v = n / div
            return (f"{v:.1f}".rstrip("0").rstrip(".") if v < 10 else f"{int(v)}") + suf
    return str(n)


HP_W, PO_W, CHIP = 36, 26, 15          # fixed slot widths (5 mono chars; glyph + 2 digits; one 12 px chip)


def slots(img, d, x, y, hp, pips, sts, gone=False, chips=1):
    """One impact in fixed slots: HP right aligned in HP_W, poise in PO_W, then `chips` chips and "+N"."""
    col = GONE if gone else B.IMPACT
    t = f"-{fmt(hp)}" if hp else "-"
    tw = int(d.textlength(t, font=B.MONO))
    d.text((x + HP_W - tw, y + 1), t, font=B.MONO, fill=col if hp else DIM)
    if gone and hp:
        d.line((x + HP_W - tw - 2, y + 8, x + HP_W + 2, y + 8), fill=(120, 126, 140))
    px = x + HP_W + 4
    if pips:
        pc = GONE if gone else B.POISE
        B.poise_glyph(d, px, y + 3, 10, col=pc)
        d.text((px + 12, y + 1), str(pips), font=B.MONO, fill=pc)
    sx = px + PO_W
    if sts and not gone:
        for i, st in enumerate(sts[:chips]):
            T12.mini(img, d, sx + i * CHIP, y + 1, st, px=12)
        if len(sts) > chips:
            d.text((sx + chips * CHIP, y + 2), f"+{len(sts) - chips}", font=T12.TINY, fill=B.IRON_L)
    return sx + chips * CHIP + 12


def impact(img, d, x, y, hp, pips, sts, gone=False):
    return slots(img, d, x - 4, y, hp, pips, sts, gone)


MODE = {"party": "all"}


def intent_rows(img, x0, x1, y, sel="kaelen", span_left=160, pips=6, next_span=450):
    d = ImageDraw.Draw(img)
    d.line((x0 + 10, y, x1 - 10, y), fill=(52, 58, 70))
    tx, sw = x0 + 76, 150
    cw = (x1 - 14 - sw - tx) // 6
    hits = T27.HITS
    r1, r2 = y + 17, y + 36
    img.paste(T17.T.face(sel, 15), (x0 + 12, r1))
    d.text((x0 + 31, r1), T27.NAMES[sel], font=B.SMALL, fill=B.GOLD)
    others = MODE["party"] == "others"
    d.text((x0 + 12, r2), "Others" if others else "Party", font=B.SMALL, fill=B.PARCH)
    left, pl, brk, ended = span_left, pips, False, None
    mine_tot = party_tot = 0
    for b in range(6):
        bx = tx + b * cw
        d.text((bx + 6, y + 3), f"Block {b + 1}", font=T12.TINY, fill=DIM)
        if b:
            d.line((bx, y + 4, bx, y + 50), fill=(40, 45, 56))
        gone = ended is not None
        if gone:
            for k in range(bx + 2, bx + cw - 2, 6):                  # cancelled blocks: a faint hatch
                d.line((k, r1 - 1, k + 8, r2 + 15), fill=(26, 29, 37))
        mh, mp = hits[sel].get(b + 1, (0, 0))
        msts = LANDS.get((sel, b + 1), [])
        if mh:
            if not gone:
                d.rectangle((bx + 4, r1, bx + cw - 4, r1 + 16), fill=(52, 40, 22), outline=B.GOLD)
                mine_tot += mh
            impact(img, d, bx + 10, r1, mh, mp, msts, gone)
        else:
            impact(img, d, bx + 10, r1, 0, 0, [])
        ph = sum(hits[k].get(b + 1, (0, 0))[0] for k in hits)
        pp = sum(hits[k].get(b + 1, (0, 0))[1] for k in hits)
        psts = [s for k in hits for s in LANDS.get((k, b + 1), [])]
        rk = [k for k in hits if not (others and k == sel)]
        sh = sum(hits[k].get(b + 1, (0, 0))[0] for k in rk)
        sp = sum(hits[k].get(b + 1, (0, 0))[1] for k in rk)
        ss = [s for k in rk for s in LANDS.get((k, b + 1), [])]
        if sh:
            gx = impact(img, d, bx + 10, r2, sh, sp, ss, gone)
        else:
            gx = impact(img, d, bx + 10, r2, 0, 0, [])
        if gone:
            continue
        party_tot += min(ph, left)
        left, pl = left - ph, pl - pp
        if pl <= 0 and not brk:
            brk = True
            T14.broken_icon(d, bx + cw - (36 if left - 0 <= 0 else 20), r2 + 1, 14)
        if left <= 0:
            ended = b
            m = bx + cw - 10
            d.rectangle((bx + 3, r2 - 1, bx + cw - 3, r2 + 16), outline=B.GOLD)
            d.polygon([(m, r2 + 2), (m + 5, r2 + 7), (m, r2 + 12), (m - 5, r2 + 7)], fill=(255, 220, 110))
    sx = x1 - 14 - sw + 8
    d.line((sx - 6, y + 4, sx - 6, y + 50), fill=B.IRON)
    d.text((sx, y + 3), "Total", font=T12.TINY, fill=DIM)
    last = 6 if ended is None else ended + 1
    rest = [k for k in hits if not (others and k == sel)]
    slots(img, d, sx - 4, r1, *row_totals([sel], last), chips=3)
    slots(img, d, sx - 4, r2, *row_totals(rest, last), chips=3)
    # user 18:59: no party total here (the boss bar's planned loss already says it)
    if ended is None:
        pass                                   # user 18:59: no "N to phase 2" (the HP bar shows it)
    else:
        # the note takes the headers (the breaking block's and the first cancelled one's), so the totals keep slots
        bx = tx + ended * cw
        d.rectangle((bx + 4, y + 2, bx + cw - 2, y + 13), fill=(10, 11, 16))
        d.text((bx + 6, y + 3), f"Block {ended + 1}: ends", font=T12.TINY, fill=B.GOLD)
        if ended < 5:
            nx = bx + cw
            d.rectangle((nx + 2, y + 2, nx + cw - 2, y + 13), fill=(10, 11, 16))
            d.text((nx + 6, y + 3), "Ph. 2 next round", font=T12.TINY, fill=B.GOLD)
    return y + T27.EXTRA


POISE_STYLE = {"v": "bar"}


def row_totals(keys, last=6):
    """(HP, poise pips, statuses) summed over the blocks that run, for the Total column (user 19:00)."""
    hp = sum(T27.HITS[k].get(b, (0, 0))[0] for k in keys for b in range(1, last + 1))
    pp = sum(T27.HITS[k].get(b, (0, 0))[1] for k in keys for b in range(1, last + 1))
    sts = [st for k in keys for b in range(1, last + 1) for st in LANDS.get((k, b), [])]
    return hp, pp, sts


def poise_bar(d, x, y, n, on, w=14, h=4, gap=2):
    """A: one bar, a faint tick per pip; B: ten even pips. The count follows (count_and_tag)."""
    T27.POISE_ON["on"], T27.POISE_ON["n"] = on, n
    if POISE_STYLE["v"] == "bar":
        bw = 80
        d.rectangle((x, y - 1, x + bw, y + h), fill=B.AP_OFF)
        d.rectangle((x, y - 1, x + round(bw * on / n), y + h), fill=B.POISE)
        for i in range(1, n):
            tx = x + round(bw * i / n)
            d.line((tx, y + h - 1, tx, y + h), fill=(90, 120, 160) if i < on else (60, 68, 84))
    else:
        for i in range(n):
            px = x + i * 8
            d.rectangle((px, y - 1, px + 5, y + h), fill=B.POISE if i < on else B.AP_OFF)


def party_hover(img, x0, x1, y, sel="kaelen"):
    """P1: hover or press the Party row: one line per member, each block's impact in the bar's own columns."""
    hits = T27.HITS
    tx, sw = x0 + 76, 150
    cw = (x1 - 14 - sw - tx) // 6
    w0, w1 = x0, x1
    sx = x1 - 14 - sw + 8
    h = 30 + 18 * (len(hits) + 1) + 10
    T26.T24.shadow(img, (w0, y, w1, y + h))
    T26.T24.nine(img, (w0, y, w1, y + h), "hud")
    d = ImageDraw.Draw(img)
    d.text((w0 + 12, y + 7), "Party by member", font=B.TITLE_S, fill=B.GOLD)
    d.text((sx, y + 10), "Total", font=T12.TINY, fill=DIM)
    d.line((sx - 6, y + 26, sx - 6, y + h - 10), fill=B.IRON)
    for i, k in enumerate(hits):
        ry = y + 30 + i * 18
        img.paste(T17.T.face(k, 14), (w0 + 10, ry))
        d.text((w0 + 27, ry), T27.NAMES[k], font=B.SMALL, fill=B.GOLD if k == sel else B.PARCH)
        for b in range(6):
            if b + 1 in hits[k]:
                hp, pp = hits[k][b + 1]
                impact(img, d, tx + b * cw + 10, ry - 1, hp, pp, LANDS.get((k, b + 1), []))
        hp, pp, sts = row_totals([k])
        all_chips(img, d, sx, ry - 1, hp, pp, sts)
    # the party's total line under the members
    ry = y + 30 + len(hits) * 18 + 2
    d.line((w0 + 10, ry - 2, x1 - 10, ry - 2), fill=B.IRON)
    d.text((w0 + 27, ry), "Party", font=B.SMALL, fill=B.PARCH)
    for b in range(6):
        ph = sum(hits[k].get(b + 1, (0, 0))[0] for k in hits)
        pp = sum(hits[k].get(b + 1, (0, 0))[1] for k in hits)
        if ph:
            impact(img, d, tx + b * cw + 10, ry - 1, ph, pp, [st for k in hits for st in LANDS.get((k, b + 1), [])])
    all_chips(img, d, sx, ry - 1, *row_totals(list(hits)))
    return h


def all_chips(img, d, x, y, hp, pp, sts):
    """The hover's Total column: the same slots with room for four chips."""
    slots(img, d, x - 4, y, hp, pp, sts, chips=4)


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    T27.intent_rows = intent_rows
    T27.poise_grouped = poise_bar
    T26.PHASES[0] = T26.PHASES[0][:7] + (150,) + T26.PHASES[0][8:]
    MODE["party"] = "others"
    crop = (204, 66, 1076, 216)
    # planning (rows shown) vs resolution / enemy turn (rows folded), full 720p, the footprint marked
    plan, res = T27.screen(shot), T27.screen(shot, with_intent=False)
    plan.save(out / "battle_take31_planning_720p.png")
    res.save(out / "battle_take31_resolving_720p.png")
    pair = Image.new("RGB", (1280 * 2 + 20, 720 + 40), B.VOID)
    for i, (im, lab, h) in enumerate(((plan, "Planning: bar + intent rows", 142), (res, "Resolving / enemy turn: bar only", 84))):
        x = i * 1300
        pair.paste(im, (x, 40))
        dd = ImageDraw.Draw(pair)
        dd.text((x + 8, 10), f"{lab}  ({h} px of 720)", font=B.TITLE_S, fill=B.GOLD)
        top = 40 + 70
        dd.rectangle((x + 1084, top, x + 1090, top + h), fill=B.AMBER)
        dd.text((x + 1096, top + h // 2 - 6), f"{h} px", font=B.SMALL, fill=B.AMBER)
    pair.resize((pair.width // 2, pair.height // 2), Image.LANCZOS).save(out / "battle_take31_plan_vs_resolve.png")
    plan.crop(crop).resize((872 * 2, 300), Image.NEAREST).save(out / "battle_take31_slots_2x.png")
    # stress: big and small numbers side by side, the slots stay put
    keep = {k: dict(v) for k, v in T27.HITS.items()}
    T27.HITS["kaelen"] = {1: (1250, 10), 3: (8, 0), 6: (14, 0)}
    T27.HITS["vesper"][2] = (215, 0)
    T26.PHASES[0] = T26.PHASES[0][:7] + (150,) + T26.PHASES[0][8:]
    T27.intent_rows = lambda *a, **k: intent_rows(*a, **{**k, "span_left": 99999, "pips": 99})
    st = T27.screen(shot)
    T27.intent_rows = intent_rows
    party_hover(st, 212, 1068, 218)
    st.crop((204, 66, 1076, 360)).resize((872 * 2, 588), Image.NEAREST).save(out / "battle_take31_stress_hover_2x.png")
    T27.HITS.clear(); T27.HITS.update(keep)
    # the phase break frame with the note in the Total header
    T27.HITS["dunstan"][4] = (36, 0)
    T26.PHASES[0] = T26.PHASES[0][:7] + (160,) + T26.PHASES[0][8:]
    c = T27.screen(shot)
    c.crop(crop).resize((872 * 2, 300), Image.NEAREST).save(out / "battle_take31_phase_break_2x.png")
    print("fmt", [fmt(n) for n in (8, 999, 1000, 1250, 12400, 120000, 999999, 1_200_000)])


if __name__ == "__main__":
    main()
