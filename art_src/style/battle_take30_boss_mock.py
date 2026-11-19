"""AI-37 battle mock take 30 (Tushar 2026-11-19 18:43 on take 29; built on take 29), 1280x720 at 1:1.

Usage: python3 art_src/style/battle_take30_boss_mock.py <playtest_dir> <out_dir>
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


def impact(img, d, x, y, hp, pips, sts, gone=False):
    """One block's impact: HP, a poise glyph per pip, a 12 px chip per status landed."""
    col = GONE if gone else B.IMPACT
    d.text((x, y + 1), f"-{hp}", font=B.MONO, fill=col)
    tw = int(d.textlength(f"-{hp}", font=B.MONO))
    if gone:
        d.line((x - 2, y + 8, x + tw + 2, y + 8), fill=(120, 126, 140))
    gx = x + tw + 5
    if pips:
        pc = GONE if gone else B.POISE
        B.poise_glyph(d, gx, y + 3, 10, col=pc)
        d.text((gx + 12, y + 1), str(pips), font=B.MONO, fill=pc)
        gx += 22
    if sts and not gone:
        T12.mini(img, d, gx, y + 1, sts[0], px=12)
        gx += 17
        if len(sts) > 1:
            d.text((gx, y + 2), f"+{len(sts) - 1}", font=T12.TINY, fill=B.IRON_L)
            gx += 14
    return gx


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
            d.text((bx + 10, r1 + 1), "-", font=B.MONO, fill=DIM)
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
            d.text((bx + 10, r2 + 1), "-", font=B.MONO, fill=DIM)
            gx = bx + 24
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
    impact(img, d, sx, r1, mine_tot, *row_totals([sel], last)[1:])
    impact(img, d, sx, r2, party_tot - (mine_tot if others else 0), *row_totals(rest, last)[1:])
    # user 18:59: no party total here (the boss bar's planned loss already says it)
    if ended is None:
        pass                                   # user 18:59: no "N to phase 2" (the HP bar shows it)
    else:
        d.text((sx + 46, r1 + 3 if not others else r2 - 8), f"Ends block {ended + 1}", font=T12.TINY, fill=B.GOLD)
        d.text((sx + 46, r2 + 3), "Ph. 2 next round", font=T12.TINY, fill=B.GOLD)
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
    """The hover's Total column: HP, poise, and every status chip (the hover has the room)."""
    d.text((x, y + 1), f"-{hp}", font=B.MONO, fill=B.IMPACT)
    gx = x + int(d.textlength(f"-{hp}", font=B.MONO)) + 5
    if pp:
        B.poise_glyph(d, gx, y + 3, 10)
        d.text((gx + 12, y + 1), str(pp), font=B.MONO, fill=B.POISE)
        gx += 24
    for st in sts:
        T12.mini(img, d, gx, y + 1, st, px=12)
        gx += 17


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    T27.intent_rows = intent_rows
    T27.poise_grouped = poise_bar
    T26.PHASES[0] = T26.PHASES[0][:7] + (150,) + T26.PHASES[0][8:]
    crop = (204, 66, 1076, 216)
    s = T27.screen(shot)
    s.save(out / "battle_take30_boss_intent_720p.png")
    s.crop(crop).resize((872 * 2, 300), Image.NEAREST).save(out / "battle_take30_A_bar_P1_party_2x.png")
    POISE_STYLE["v"] = "pips"
    s.crop((280, 116, 520, 150)).resize((480, 68), Image.NEAREST).save(out / "_a.png")
    b = T27.screen(shot)
    b.crop((280, 116, 520, 150)).resize((480, 68), Image.NEAREST).save(out / "_b.png")
    POISE_STYLE["v"] = "bar"
    MODE["party"] = "others"
    o = T27.screen(shot)
    o.crop(crop).resize((872 * 2, 300), Image.NEAREST).save(out / "battle_take30_P2_others_2x.png")
    MODE["party"] = "all"
    hv = T27.screen(shot)
    party_hover(hv, 212, 1068, 218)
    hv.crop((204, 66, 1076, 340)).resize((872 * 2, 548), Image.NEAREST).save(out / "battle_take30_P1_hover_2x.png")
    # the pick (user 2026-11-19 18:57): A with P2 in the bar and the hover listing every member
    MODE["party"] = "others"
    ho = T27.screen(shot)
    party_hover(ho, 212, 1068, 218)
    ho.crop((204, 66, 1076, 360)).resize((872 * 2, 588), Image.NEAREST).save(out / "battle_take30_pick_others_hover_2x.png")
    MODE["party"] = "all"
    # poise A vs B side by side
    pa, pb = Image.open(out / "_a.png"), Image.open(out / "_b.png")
    cmp_ = Image.new("RGB", (1000, 110), B.VOID)
    dd = ImageDraw.Draw(cmp_)
    dd.text((10, 6), "A (recommended): one bar, a tick per pip, the count", font=B.SMALL, fill=B.PARCH)
    dd.text((510, 6), "B: ten even pips, the count", font=B.SMALL, fill=B.PARCH)
    cmp_.paste(pa, (10, 30)); cmp_.paste(pb, (510, 30))
    cmp_.save(out / "battle_take30_poise_A_B_2x.png")
    (out / "_a.png").unlink(); (out / "_b.png").unlink()
    # the phase break frame in the new notation
    T27.HITS["dunstan"][4] = (36, 0)
    T26.PHASES[0] = T26.PHASES[0][:7] + (160,) + T26.PHASES[0][8:]
    c = T27.screen(shot)
    ImageDraw.Draw(c).polygon([(1024, 119), (1029, 124), (1024, 129), (1019, 124)], fill=(255, 220, 110))
    c.crop(crop).resize((872 * 2, 300), Image.NEAREST).save(out / "battle_take30_phase_break_2x.png")


if __name__ == "__main__":
    main()
