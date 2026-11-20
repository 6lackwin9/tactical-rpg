"""AI-37 battle mock take 32 (Tushar 2026-11-19 19:08 on take 31), 1280x720 at 1:1.

Usage: python3 art_src/style/battle_take32_boss_mock.py <playtest_dir> <out_dir>
  - the Total column was cluttered: every impact now sits in hidden cells with gaps between them (HP right aligned
    in its cell, the poise glyph at a fixed x with its number left aligned after it, each status chip in its own
    cell at a fixed pitch), so a short or long number never moves the poise or the chips;
  - numbers stay whole up to 9999 (so 1250 sits beside 233 as digits, not as 1.3K); K starts at 10K, M at 1M;
  - more statuses than chip cells: "+N" has its own cell after the chips (block cells 1 chip, the Totals 3), so it
    never overlaps a chip;
  - the guide frame draws the hidden cells so the grid can be checked.
Everything else is take 31.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take12_mock as T12  # noqa: E402
import battle_take26_boss_mock as T26  # noqa: E402
import battle_take27_boss_mock as T27  # noqa: E402
import battle_take31_boss_mock as T31  # noqa: E402

T31.impact = lambda img, d, x, y, hp, pips, sts, gone=False: slots(img, d, x - 8, y, hp, pips, sts, gone)
T31.all_chips = lambda img, d, x, y, hp, pp, sts: slots(img, d, x - 4, y, hp, pp, sts, chips=3)

HP_W, HGAP, GAP, PO_W, CHIP, PLUS = 36, 6, 3, 22, 19, 10   # a chip is 17 px drawn, 2 px apart      # hidden cells: HP (5 mono chars), gap, poise (glyph + 2 digits), chip pitch
GUIDE = {"on": False}


def fmt(n):
    if n >= 1_000_000:
        v = n / 1_000_000
        return (f"{v:.1f}".rstrip("0").rstrip(".") if v < 10 else f"{int(v)}") + "M"
    if n >= 10_000:
        return f"{n // 1000}K"
    return str(n)


def slots(img, d, x, y, hp, pips, sts, gone=False, chips=1):
    col = T31.GONE if gone else B.IMPACT
    hg, g, pw = (HGAP, GAP, PO_W) if chips == 1 else (HGAP + 2, GAP + 5, PO_W + 4)   # the Totals have room
    cells = [(x, HP_W), (x + HP_W + hg, pw)] + [(x + HP_W + hg + g + pw + i * CHIP, 17) for i in range(chips)]
    cells.append((cells[-1][0] + 19, PLUS))                 # its own cell for "+N", never over a chip
    if GUIDE["on"]:
        for cx, cw in cells:
            d.rectangle((cx, y, cx + cw - 1, y + 17), outline=(70, 110, 70))
    t = f"-{fmt(hp)}" if hp else "-"
    tw = int(d.textlength(t, font=B.MONO))
    d.text((x + HP_W - tw, y + 1), t, font=B.MONO, fill=col if hp else T31.DIM)
    if gone and hp:
        d.line((x + HP_W - tw - 2, y + 8, x + HP_W + 2, y + 8), fill=(120, 126, 140))
    px = cells[1][0]
    if pips:
        pc = T31.GONE if gone else B.POISE
        B.poise_glyph(d, px, y + 3, 10, col=pc)
        d.text((px + 12, y + 1), str(pips), font=B.MONO, fill=pc)
    sx = cells[2][0]
    if sts and not gone:
        for i, st in enumerate(sts[:chips]):
            T12.mini(img, d, sx + i * CHIP, y + 1, st, px=12)
        if len(sts) > chips:
            d.text((cells[-1][0] + 1, y + 2), f"+{len(sts) - chips}", font=T12.TINY, fill=B.IRON_L)
    return cells[-1][0] + PLUS


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    T31.fmt, T31.slots = fmt, slots
    T27.intent_rows = T31.intent_rows
    T27.poise_grouped = T31.poise_bar
    T31.MODE["party"] = "others"
    keep = {k: dict(v) for k, v in T27.HITS.items()}
    for name, guide in (("battle_take32_totals_2x.png", False), ("battle_take32_cells_guide_2x.png", True)):
        GUIDE["on"] = guide
        T27.HITS.clear(); T27.HITS.update({k: dict(v) for k, v in keep.items()})
        T27.HITS["kaelen"] = {1: (1250, 10), 3: (8, 0), 6: (14, 0)}
        T27.HITS["vesper"][2] = (215, 0)
        T26.PHASES[0] = T26.PHASES[0][:7] + (150,) + T26.PHASES[0][8:]
        T27.intent_rows = lambda *a, **k: T31.intent_rows(*a, **{**k, "span_left": 99999, "pips": 99})
        st = T27.screen(shot)
        T31.party_hover(st, 212, 1068, 218)
        st.crop((204, 66, 1076, 360)).resize((872 * 2, 588), Image.NEAREST).save(out / name)
    GUIDE["on"] = False
    T27.HITS.clear(); T27.HITS.update(keep)
    T27.intent_rows = T31.intent_rows
    s = T27.screen(shot)
    T31.party_hover(s, 212, 1068, 218)
    s.crop((204, 66, 1076, 360)).resize((872 * 2, 588), Image.NEAREST).save(out / "battle_take32_normal_2x.png")


if __name__ == "__main__":
    main()
