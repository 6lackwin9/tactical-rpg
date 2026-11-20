"""AI-37 battle mock take 29 (Tushar 2026-11-19 18:38 on take 28), 1280x720 at 1:1.

Usage: python3 art_src/style/battle_take29_boss_mock.py <playtest_dir> <out_dir>
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
    gx = x + tw + 4
    for _ in range(pips):
        B.poise_glyph(d, gx, y + 3, 9, col=GONE if gone else B.POISE)
        gx += 9
    gx += 2 if pips else 0
    for st in sts:
        if not gone:
            T12.mini(img, d, gx, y + 2, st, px=12)
        gx += 15
    return gx


def intent_rows(img, x0, x1, y, sel="kaelen", span_left=160, pips=6, next_span=450):
    d = ImageDraw.Draw(img)
    d.line((x0 + 10, y, x1 - 10, y), fill=(52, 58, 70))
    tx, sw = x0 + 76, 150
    cw = (x1 - 14 - sw - tx) // 6
    hits = T27.HITS
    r1, r2 = y + 17, y + 36
    img.paste(T17.T.face(sel, 15), (x0 + 12, r1))
    d.text((x0 + 31, r1), T27.NAMES[sel], font=B.SMALL, fill=B.GOLD)
    d.text((x0 + 12, r2), "Party", font=B.SMALL, fill=B.PARCH)
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
        if ph:
            gx = impact(img, d, bx + 10, r2, ph, min(pp, max(0, pl)) if not gone else pp, psts, gone)
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
    d.text((sx, r1 + 1), f"-{mine_tot}", font=B.MONO, fill=B.IMPACT)
    d.text((sx, r2 + 1), f"-{party_tot}", font=B.MONO, fill=B.IMPACT)
    if ended is None:
        d.text((sx + 46, r2 + 3), f"{left} to phase 2", font=T12.TINY, fill=B.GOLD)
    else:
        d.text((sx + 46, r1 + 3), f"Ends block {ended + 1}", font=T12.TINY, fill=B.GOLD)
        d.text((sx + 46, r2 + 3), "Ph. 2 next round", font=T12.TINY, fill=B.GOLD)
    return y + T27.EXTRA


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    T27.intent_rows = intent_rows
    T26.PHASES[0] = T26.PHASES[0][:7] + (150,) + T26.PHASES[0][8:]
    s = T27.screen(shot)
    s.save(out / "battle_take29_boss_intent_720p.png")
    s.crop((204, 66, 1076, 216)).resize((872 * 2, 300), Image.NEAREST).save(out / "battle_take29_boss_intent_2x.png")
    # the phase break: Dunstan's block 4 hit is 36, so the span hits 0 in block 5; block 6 is cancelled
    T27.HITS["dunstan"][4] = (36, 0)
    T26.PHASES[0] = T26.PHASES[0][:7] + (160,) + T26.PHASES[0][8:]
    c = T27.screen(shot)
    cd = ImageDraw.Draw(c)
    cd.polygon([(1024, 119), (1029, 124), (1024, 129), (1019, 124)], fill=(255, 220, 110))
    c.crop((204, 66, 1076, 216)).resize((872 * 2, 300), Image.NEAREST).save(out / "battle_take29_phase_break_2x.png")


if __name__ == "__main__":
    main()
