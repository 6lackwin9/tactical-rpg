"""AI-21 battle mock take 23 (contract section 9): the full timeline view (T) and the playback controls, 1280x720.

Usage: python3 art_src/style/battle_take23_timeline_mock.py <playtest_dir> <out_dir>
[PROVISIONAL] (Tushar away; for his review batch):
  - T opens the whole round over the board: one row per unit (heroes, guests, then foes), 6 block columns. Each move is
    a tile in its kind colour spanning its blocks (ghost moves faded, foe moves read from their intent), and every blow
    shows as a red tick on the block where it lands with the face of the unit it lands on. The selected hero's row is
    rimmed gold; a click on a hero row selects that hero and closes the view; T or Esc closes it.
  - Playback (after Confirm): a bar under the board with step back, play or pause, step forward, the speed (1x, 2x,
    4x), "Block 3 of 6" and Skip to the end of the round; the timeline view shows a gold cursor on the block playing.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take11_mock as T  # noqa: E402
import battle_take12_mock as T12  # noqa: E402
import battle_take13_mock as T13  # noqa: E402
import battle_take15_mock as T15  # noqa: E402
import battle_take22_boss_mock as T22  # noqa: E402

X0, X1 = 204, 1076
NAME_W = 118
ROW_H = 34
# (portrait key or foe face, name, moves: (icon, start block, blocks, ghost), blows: (block, target key))
ROWS = [("kaelen", "Kaelen", [("arrow", 0, 1, False), ("pneumatic_hammer_strike", 1, 3, False),
                             ("pommel_strike", 4, 1, True), ("pommel_strike", 5, 1, True)], [(1, "f0"), (4, "f0")]),
        ("vesper", "Vesper", [("bleeding_cut", 0, 2, False), ("arrow", 2, 1, False), ("bleeding_cut", 3, 2, False)],
         [(1, "f2"), (4, "f2")]),
        ("mara_voss", "Mara", [("shield_wall", 0, 2, False), ("pommel_strike", 2, 2, False), ("rally", 4, 2, True)],
         [(3, "f1")]),
        ("dunstan", "Dunstan", [("intent:rest", 0, 6, False)], [])]
FOES = [("grakka_ashmaw", "Grakka", [("pneumatic_hammer_strike", 0, 3, False), ("shield_wall", 3, 1, False),
                                      ("pneumatic_hammer_strike", 4, 2, False)], [(2, "dunstan"), (5, "kaelen")]),
        ("brawler_cohort_breaker", "Brawler", [("pommel_strike", 0, 2, False), ("pommel_strike", 3, 2, False)],
         [(1, "kaelen"), (4, "kaelen")]),
        ("ranger_cohort_bolt", "Archer", [("arrow", 0, 1, False), ("bleeding_cut", 1, 3, False)], [(3, "vesper")]),
        ("rogue_cohort_flue", "Skirmisher", [("bleeding_cut", 0, 2, False), ("arrow", 2, 1, False),
                                             ("bleeding_cut", 3, 3, False)], [(1, "mara_voss"), (5, "mara_voss")]),
        ("mage_cohort_vat", "Shaman", [("greater_mend", 0, 3, False), ("shield_wall", 3, 3, False)], [])]


def foe_face(fk, px):
    return T15.portrait(f"bosses/{fk}.png" if fk == "grakka_ashmaw" else f"common/{fk}.png", px)


def face_of(key, px):
    if key.startswith("f"):
        return foe_face(FOES[int(key[1]) + 1][0], px)
    return T.face(key, px)


def row(img, d, y, face, name, moves, blows, foe, sel, cursor):
    bw = (X1 - X0 - NAME_W - 8) / 6
    d.rectangle((X0, y, X1, y + ROW_H - 2), fill=B.SLATE if not foe else (34, 30, 36),
                outline=B.GOLD if sel else B.BASALT)
    img.paste(face.resize((26, 26)), (X0 + 4, y + 3))
    d.text((X0 + 34, y + 8), name, font=B.SMALL, fill=(222, 160, 150) if foe else B.PARCH)
    bx0 = X0 + NAME_W
    for icon, s, n, ghost in moves:
        x = bx0 + s * bw + 2
        w = n * bw - 4
        base = T13.KIND_COL[T13.SKILL_KIND[icon]]
        d.rectangle((x, y + 3, x + w, y + ROW_H - 6), fill=T13.mix(base, B.BASALT, T13.GHOST) if ghost else base)
        ic = B.slot_icon(icon, 64).resize((22, 22), Image.BOX)
        if ghost:
            ic.putalpha(ic.getchannel("A").point(lambda v: min(v, 150)))
        img.paste(ic, (int(x + w / 2 - 11), y + 5), ic)
    for blk, tgt in blows:
        x = bx0 + blk * bw + bw - 22
        d.rectangle((x - 2, y + ROW_H - 6, x + 18, y + ROW_H - 3), fill=B.IMPACT)
        img.paste(face_of(tgt, 14), (int(x + 2), y + 4))
    if cursor is not None:
        cx = bx0 + cursor * bw
        d.rectangle((cx, y, cx + bw, y + ROW_H - 2), outline=B.GOLD)


def timeline_view(img, d, y0, cursor=None):
    h = 42 + (len(ROWS) + len(FOES)) * ROW_H + 18
    d.rectangle((X0 - 4, y0, X1 + 4, y0 + h), fill=B.BASALT, outline=(116, 84, 60))
    d.line((X0 - 3, y0 + 1, X1 + 3, y0 + 1), fill=(153, 27, 27))
    d.text((X0 + 4, y0 + 5), "Round 1: the whole plan", font=B.TITLE_S, fill=B.GOLD)
    d.text((X1 - 150, y0 + 7), "T or Esc to close", font=B.SMALL, fill=B.IRON_L)
    bw = (X1 - X0 - NAME_W - 8) / 6
    for b in range(6):
        d.text((X0 + NAME_W + b * bw + bw / 2 - 3, y0 + 25), str(b + 1), font=B.MONO,
               fill=B.GOLD if cursor == b else B.IRON_L)
    y = y0 + 42
    for i, (key, name, moves, blows) in enumerate(ROWS):
        row(img, d, y, T.face(key, 26), name, moves, blows, False, i == 0, cursor)
        y += ROW_H
    d.line((X0, y + 1, X1, y + 1), fill=(150, 50, 45))
    y += 4
    for fk, name, moves, blows in FOES:
        row(img, d, y, foe_face(fk, 26), name, moves, blows, True, False, cursor)
        if fk == "grakka_ashmaw":
            T22.T20.crown(d, X0 + 2, y + 1, 9)
        y += ROW_H
    return y0 + h


def playback(d, y, block):
    x = 420
    d.rectangle((x, y, x + 440, y + 34), fill=B.BASALT, outline=(116, 84, 60))
    bx = x + 8
    for label, w in (("<<", 30), ("||", 30), (">>", 30)):
        d.rectangle((bx, y + 5, bx + w, y + 29), fill=B.SLATE, outline=B.IRON)
        d.text((bx + 7, y + 9), label, font=B.MONO, fill=B.PARCH)
        bx += w + 4
    bx += 6
    for sp in ("1x", "2x", "4x"):
        on = sp == "2x"
        d.rectangle((bx, y + 5, bx + 28, y + 29), fill=(70, 52, 24) if on else B.SLATE, outline=B.AMBER if on else B.IRON)
        d.text((bx + 6, y + 9), sp, font=B.MONO, fill=B.AMBER if on else B.PARCH)
        bx += 32
    bx += 8
    d.text((bx, y + 9), f"Block {block + 1} of 6", font=B.SMALL, fill=B.GOLD)
    bx += 92
    for b in range(6):
        d.rectangle((bx + b * 9, y + 14, bx + b * 9 + 6, y + 20), fill=B.GOLD if b <= block else B.AP_OFF)
    d.rectangle((x + 370, y + 5, x + 432, y + 29), fill=B.SLATE, outline=B.IRON)
    d.text((x + 380, y + 9), "Skip", font=B.SMALL, fill=B.PARCH)


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    a = T22.screen(shot)
    d = ImageDraw.Draw(a)
    timeline_view(a, d, 70)
    a.save(out / "battle_take23_timeline_720p.png")
    b = T22.screen(shot)
    d = ImageDraw.Draw(b)
    timeline_view(b, d, 70, cursor=2)
    playback(d, 596, 2)
    b.save(out / "battle_take23_playback_720p.png")


if __name__ == "__main__":
    main()
