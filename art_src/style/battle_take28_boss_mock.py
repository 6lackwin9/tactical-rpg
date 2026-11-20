"""AI-37 battle mock take 28 (Tushar 2026-11-19 18:34 on take 27), 1280x720 at 1:1.

Usage: python3 art_src/style/battle_take28_boss_mock.py <playtest_dir> <out_dir>
  - the boss intent drops the per-member chips (too busy to parse): two rows of damage by block, the selected hero's
    and the whole party's, each with its total at the right; the party row also marks where BREAK lands (the broken
    poise glyph) and where the next phase starts (its diamond); everything else as take 27 (markers, poise, clearance).
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


def intent_rows(img, x0, x1, y, sel="kaelen", span_left=160, pips=6, next_span=450):
    d = ImageDraw.Draw(img)
    d.line((x0 + 10, y, x1 - 10, y), fill=(52, 58, 70))
    tx, sw = x0 + 76, 150
    cw = (x1 - 14 - sw - tx) // 6
    hits = T27.HITS
    mine = [hits[sel].get(b + 1, (0, 0))[0] for b in range(6)]
    party = [sum(hits[k].get(b + 1, (0, 0))[0] for k in hits) for b in range(6)]
    ppips = [sum(hits[k].get(b + 1, (0, 0))[1] for k in hits) for b in range(6)]
    for b in range(6):
        bx = tx + b * cw
        d.text((bx + 6, y + 3), f"Block {b + 1}", font=T12.TINY, fill=DIM)
        if b:
            d.line((bx, y + 4, bx, y + 50), fill=(40, 45, 56))
    # row 1: the selected hero
    r1, r2 = y + 17, y + 36
    img.paste(T17.T.face(sel, 15), (x0 + 12, r1))
    d.text((x0 + 31, r1), T27.NAMES[sel], font=B.SMALL, fill=B.GOLD)
    d.text((x0 + 12, r2), "Party", font=B.SMALL, fill=B.PARCH)
    left, pl, brk, crossed = span_left, pips, False, False
    for b in range(6):
        bx = tx + b * cw
        if mine[b]:
            d.rectangle((bx + 4, r1, bx + cw - 4, r1 + 16), fill=(52, 40, 22), outline=B.GOLD)
            d.text((bx + 10, r1 + 1), f"-{mine[b]}", font=B.MONO, fill=B.IMPACT)
        else:
            d.text((bx + 10, r1 + 1), "-", font=B.MONO, fill=DIM)
        # row 2: the party's damage this block, BREAK and the phase change where they land
        left, pl = left - party[b], max(0, pl - ppips[b])
        if party[b]:
            d.text((bx + 10, r2 + 1), f"-{party[b]}", font=B.MONO, fill=B.IMPACT)
        else:
            d.text((bx + 10, r2 + 1), "-", font=B.MONO, fill=DIM)
        gx = bx + 46
        if pl == 0 and not brk:
            brk = True
            T14.broken_icon(d, gx, r2 + 1, 14)
            d.text((gx + 17, r2 + 2), "BREAK", font=T12.TINY, fill=B.AMBER)
        if left <= 0 and not crossed:
            crossed = True
            m = gx + 6
            d.polygon([(m, r2 + 3), (m + 6, r2 + 9), (m, r2 + 15), (m - 6, r2 + 9)], fill=(255, 220, 110), outline=B.GOLD)
            d.text((m + 10, r2 + 2), "Ph. 2", font=T12.TINY, fill=B.GOLD)
    sx = x1 - 14 - sw + 8
    d.line((sx - 6, y + 4, sx - 6, y + 50), fill=B.IRON)
    d.text((sx, y + 3), "Total", font=T12.TINY, fill=DIM)
    d.text((sx, r1 + 1), f"-{sum(mine)}", font=B.MONO, fill=B.IMPACT)
    d.text((sx, r2 + 1), f"-{sum(party)}", font=B.MONO, fill=B.IMPACT)
    tail = f"{left} to ph. 2" if left > 0 else f"+{-left} to ph. 2"
    d.text((sx + 46, r2 + 3), tail, font=T12.TINY, fill=B.GOLD)
    return y + T27.EXTRA


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    sys.argv[1] = str(pt)
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    T27.intent_rows = intent_rows
    T26.PHASES[0] = T26.PHASES[0][:7] + (150,) + T26.PHASES[0][8:]
    s = T27.screen(shot)
    s.save(out / "battle_take28_boss_intent_720p.png")
    s.crop((204, 66, 1076, 216)).resize((872 * 2, 300), Image.NEAREST).save(out / "battle_take28_boss_intent_2x.png")
    T27.HITS["kaelen"][6] = (30, 0)
    T26.PHASES[0] = T26.PHASES[0][:7] + (160,) + T26.PHASES[0][8:]
    c = T27.screen(shot)
    cd = ImageDraw.Draw(c)
    cd.rectangle((976, 116, 1060, 131), fill=c.getpixel((1050, 112)))
    cd.text((978, 117), "-166", font=B.MONO, fill=B.IMPACT)
    cd.polygon([(1024, 119), (1029, 124), (1024, 129), (1019, 124)], fill=(255, 220, 110))
    c.crop((204, 66, 1076, 216)).resize((872 * 2, 300), Image.NEAREST).save(out / "battle_take28_carry_2x.png")


if __name__ == "__main__":
    main()
