"""AI-38 battle mock take 35 (Tushar 2026-11-19 19:44 on take 34), the full timeline view (T), 1280x720.

Usage: python3 art_src/style/battle_take35_timeline_mock.py <playtest_dir> <out_dir>
  - a unit keeps one target for the whole round (no target switching inside a round), so the row names it once:
    the unit's face and name, then a small arrow and the target's face, then the unit's timeline;
  - the per-blow marks (the red tick and the target face at the end of each attack) are gone; the tiles keep take 34's
    calm look (a dark tint of the kind, a 3 px kind band, a kind rim, 24 px icons); locks keep their hatch;
  - a unit with no target this round (support or rest only) shows a dim dash in the target slot.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take13_mock as T13  # noqa: E402
import battle_take23_timeline_mock as T23  # noqa: E402
import battle_take33_status_mock as T33  # noqa: E402

NAME_W = 150                       # face, name, arrow, target face


def row(img, d, y, face, name, moves, blows, foe, sel, cursor):
    X0, X1, RH = T23.X0, T23.X1, T23.ROW_H
    bw = (X1 - X0 - NAME_W - 8) / 6
    d.rectangle((X0, y, X1, y + RH - 2), fill=B.SLATE if not foe else (34, 30, 36), outline=B.GOLD if sel else B.BASALT)
    img.paste(face.resize((26, 26)), (X0 + 4, y + 3))
    d.text((X0 + 34, y + 8), name, font=B.SMALL, fill=(222, 160, 150) if foe else B.PARCH)
    tx = X0 + NAME_W - 30
    if blows:
        ax = tx - 12
        d.polygon([(ax, y + 12), (ax + 6, y + 16), (ax, y + 20)], fill=B.IRON_L)
        d.rectangle((tx - 1, y + 4, tx + 24, y + 29), outline=(150, 50, 45))
        img.paste(T23.face_of(blows[0][1], 24), (tx, y + 5))
    else:
        d.text((tx + 8, y + 8), "-", font=B.MONO, fill=(70, 76, 90))
    bx0 = X0 + NAME_W
    for icon, s, n, ghost in moves:
        x = bx0 + s * bw + 2
        w = n * bw - 4
        kind = T13.KIND_COL[T13.SKILL_KIND[icon]]
        rim = T13.mix(kind, (20, 22, 28), 0.45)
        fill = T13.mix(kind, (22, 25, 32), 0.3)
        if ghost:
            kind, rim = T13.mix(kind, fill, 0.55), T13.mix(rim, fill, 0.5)
        d.rectangle((x, y + 3, x + w, y + RH - 6), fill=fill, outline=rim)
        d.rectangle((x, y + 3, x + w, y + 5), fill=kind)
        ic = B.slot_icon(icon, 64).resize((24, 24), Image.BOX)
        if ghost:
            ic.putalpha(ic.getchannel("A").point(lambda v: min(v, 140)))
        img.paste(ic, (int(x + w / 2 - 12), y + 4), ic)
    if cursor is not None:
        cx = bx0 + cursor * bw
        d.rectangle((cx, y, cx + bw, y + RH - 2), outline=B.GOLD)


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    T23.row = row
    T23.NAME_W = NAME_W
    img = T33.locks(T33.base(shot))
    d = ImageDraw.Draw(img)
    bw = (T23.X1 - T23.X0 - NAME_W - 8) / 6                       # header numbers over the new columns
    d.rectangle((T23.X0 + 100, 70 + 22, T23.X1 - 2, 70 + 40), fill=B.BASALT)
    for b in range(6):
        d.text((T23.X0 + NAME_W + b * bw + bw / 2 - 3, 70 + 25), str(b + 1), font=B.MONO, fill=B.IRON_L)
    d.text((T23.X0 + NAME_W - 46, 70 + 25), "Target", font=T33.T12.TINY, fill=B.IRON_L)
    img.save(out / "battle_take35_timeline_720p.png")
    T33.z2(img, (204, 70, 1076, 430)).save(out / "battle_take35_timeline_2x.png")


if __name__ == "__main__":
    main()
