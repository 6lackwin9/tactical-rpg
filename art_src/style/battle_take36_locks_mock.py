"""AI-38 battle mock take 36 (Tushar 2026-11-19 19:45 on take 35): locked blocks as grey blocks, 1280x720.

Usage: python3 art_src/style/battle_take36_locks_mock.py <playtest_dir> <out_dir>
  - a locked block is a grey block with the status icon, built like any other tile so it sits with the plan (no
    hatch): in the full timeline a grey tint with a grey band and rim; on the top strip a grey tile with the icon in
    the skill-icon frame and a grey bar under it;
  - stun is one continuous grey block over the blocks it locks (1 and 2); stagger is two separate one-block grey
    blocks (1 and 4), so the paced 2-block moves read between them;
  - the strip frame shows Kaelen staggered: lock, a 2-block Pommel Strike, lock, a 2-block Pommel Strike.
  - targets (STATUSVIS1 SV14): a Support row names the ally (teal frame and arrow); an area skill keeps its main
    target with a small amber ring on the face's corner; Maneuver or Rest only shows the dim dash; self shows
    the unit's own face (teal); a ground tile (field spells) shows a 3 x 3 floor patch with the target tile lit, and
    hovering it rings that tile on the board (user 19:46).
Everything else is take 35.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take23_timeline_mock as T23  # noqa: E402
import battle_take25_status_where_mock as T25  # noqa: E402
import battle_take33_status_mock as T33  # noqa: E402
import battle_take35_timeline_mock as T35  # noqa: E402

G_FILL, G_BAND, G_RIM, G_BOX = (50, 54, 64), (120, 126, 140), (78, 84, 96), (30, 33, 41)
STRIP_BG = (20, 23, 31)


def icon_box(img, d, cx, y, st, s=18):
    d.rectangle((cx - s // 2 - 1, y, cx + s // 2, y + s + 1), fill=G_BOX, outline=G_RIM)
    T25.icon(img, st, cx - 8, y + 2)


def lock_tile(img, d, x0, x1, y0, y1, st):
    d.rectangle((x0, y0, x1, y1), fill=G_FILL, outline=G_RIM)
    d.rectangle((x0, y0, x1, y0 + 2), fill=G_BAND)
    icon_box(img, d, (x0 + x1) // 2, y0 + 4, st)


def locks(img):
    """The full timeline with Kaelen staggered (1, 4) and Dunstan stunned (1-2), locks as grey blocks."""
    T23.ROWS[0] = ("kaelen", "Kaelen", [("pommel_strike", 1, 2, False), ("pommel_strike", 4, 2, False)],
                   [(2, "f0"), (5, "f0")])
    T23.ROWS[3] = ("dunstan", "Dunstan", [("pommel_strike", 2, 2, False), ("arrow", 4, 1, False),
                                          ("shield_wall", 5, 1, False)], [(3, "f0")])
    d = ImageDraw.Draw(img)
    T23.timeline_view(img, d, 70)
    bw = (T23.X1 - T23.X0 - T23.NAME_W - 8) / 6
    for row, spans, st in ((0, ((0, 1), (3, 1)), "stagger"), (3, ((0, 2),), "stun")):
        y = 70 + 42 + row * T23.ROW_H
        for b, n in spans:
            x0 = int(T23.X0 + T23.NAME_W + b * bw + 2)
            lock_tile(img, d, x0, int(x0 + n * bw - 4), y + 3, y + T23.ROW_H - 6, st)
    return img


def strip(img):
    """The top strip for staggered Kaelen: block i at x 196 + 60 i, 56 px wide (a span of n blocks 60 n - 4)."""
    d = ImageDraw.Draw(img)
    d.rectangle((194, 0, 556, 63), fill=STRIP_BG)
    plan = [("lock", 0, 1), ("pommel_strike", 1, 2), ("lock", 3, 1), ("pommel_strike", 4, 2)]
    for what, b, n in plan:
        x0, x1 = 196 + 60 * b, 196 + 60 * b + 60 * n - 5
        if what == "lock":
            d.rectangle((x0, 6, x1, 52), fill=G_FILL, outline=G_RIM)
            cx = (x0 + x1) // 2
            d.rectangle((cx - 17, 8, cx + 16, 41), fill=G_BOX, outline=G_RIM)
            ic = T25.T17.icon16("stagger").resize((24, 24), Image.NEAREST)
            img.paste(ic, ((x0 + x1) // 2 - 12, 13), ic)
            d.rectangle((x0, 56, x1, 61), fill=G_RIM)
        else:
            sel = b == 1
            d.rectangle((x0, 6, x1, 52), fill=(160, 56, 56), outline=B.GOLD if sel else (160, 56, 56))
            cx = (x0 + x1) // 2
            d.rectangle((cx - 17, 8, cx + 16, 41), fill=(34, 38, 46), outline=(20, 22, 28))
            ic = B.slot_icon(what, 64).resize((30, 30), Image.BOX)
            img.paste(ic, (cx - 15, 10), ic)
            for k in range(2):
                px = cx - 10 + k * 12
                d.rectangle((px, 44, px + 7, 50), outline=(20, 22, 28), fill=(96, 150, 230))
            d.rectangle((x0, 56, x1, 61), fill=B.IMPACT)
    return img


SUPPORT, AREA = (69, 133, 124), (232, 140, 40)
# SV14 edge cases: a Support row names the ally it helps (teal frame); an area skill keeps its main target with an
# area mark (a small amber ring on the face's corner)
ROW_TARGET = {"Mara": ("kaelen", "support"), "Grakka": ("dunstan", "area"), "Shaman": (None, "tile")}
TILE = (217, 150, 60)


def tile_glyph(d, x, y):
    """A ground-tile target: a 3 x 3 patch of floor squares with the target tile lit; hovering it rings the tile on
    the board (and a field spell's area is drawn there, as in planning)."""
    d.rectangle((x - 1, y - 1, x + 24, y + 24), fill=(26, 29, 36), outline=TILE)
    for i in range(3):
        for j in range(3):
            cx, cy = x + 1 + i * 8, y + 1 + j * 8
            on = i == 1 and j == 1
            d.rectangle((cx, cy, cx + 6, cy + 6), fill=TILE if on else (60, 64, 74))


def row(img, d, y, face, name, moves, blows, foe, sel, cursor):
    tgt, kind = ROW_TARGET.get(name, (None, None))
    T35.row(img, d, y, face, name, moves, [(0, tgt)] if tgt else (blows if not kind else []), foe, sel, cursor)
    if kind:
        tx = T23.X0 + T35.NAME_W - 30
        if kind == "tile":
            d.rectangle((tx - 14, y + 4, tx + 26, y + 29), fill=(34, 30, 36))
            d.polygon([(tx - 12, y + 12), (tx - 6, y + 16), (tx - 12, y + 20)], fill=B.IRON_L)
            tile_glyph(d, tx, y + 5)
        elif kind == "self":
            d.rectangle((tx - 1, y + 4, tx + 24, y + 29), outline=SUPPORT)
            d.polygon([(tx - 12, y + 12), (tx - 6, y + 16), (tx - 12, y + 20)], fill=SUPPORT)
        elif kind == "support":
            d.rectangle((tx - 1, y + 4, tx + 24, y + 29), outline=SUPPORT)
            d.polygon([(tx - 12, y + 12), (tx - 6, y + 16), (tx - 12, y + 20)], fill=SUPPORT)
        else:
            d.ellipse((tx + 14, y + 19, tx + 27, y + 32), fill=(20, 22, 28), outline=AREA, width=2)
            d.ellipse((tx + 18, y + 23, tx + 23, y + 28), outline=AREA)


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    T23.row, T23.NAME_W = row, T35.NAME_W
    T23.ROWS[2] = ("mara_voss", "Mara", [("shield_wall", 0, 2, False), ("rally", 2, 2, False), ("rally", 4, 2, True)], [])
    full = locks(strip(T33.base(shot)))
    d = ImageDraw.Draw(full)
    bw = (T23.X1 - T23.X0 - T35.NAME_W - 8) / 6
    d.rectangle((T23.X0 + 100, 70 + 22, T23.X1 - 2, 70 + 40), fill=B.BASALT)
    for b in range(6):
        d.text((T23.X0 + T35.NAME_W + b * bw + bw / 2 - 3, 70 + 25), str(b + 1), font=B.MONO, fill=B.IRON_L)
    d.text((T23.X0 + T35.NAME_W - 46, 70 + 25), "Target", font=T33.T12.TINY, fill=B.IRON_L)
    full.save(out / "battle_take36_locks_720p.png")
    ROW_TARGET["Shaman"] = ("f3", "self")              # the Shaman (foe index 3) targets itself instead
    T23.FOES[4] = ("mage_cohort_vat", "Shaman", [("greater_mend", 0, 3, False), ("shield_wall", 3, 3, False)], [])
    me = locks(strip(T33.base(shot)))
    tiles = [("Top strip: Kaelen staggered, grey locks on blocks 1 and 4, his 2-block moves between", T33.z2(full, (180, 0, 600, 66))),
             ("Full timeline: stagger as two one-block grey locks (Kaelen), stun as one continuous lock (Dunstan)",
              T33.z2(full, (204, 70, 1076, 252))),
             ("Targets (SV14): Mara supports Kaelen (teal); Grakka's hammer is an area skill on Dunstan (amber ring)",
              T33.z2(full, (204, 180, 520, 290))),
             ("Ground tile target (field spell): a 3 x 3 floor patch with the target tile lit; hover rings it on the board",
              T33.z2(full, (204, 384, 520, 420))),
             ("Self target: the unit's own face in a teal frame", T33.z2(me, (204, 384, 520, 420)))]
    T33.label_tiles(tiles, "Take 36: locked blocks as grey blocks with the status icon (2x)").save(out / "battle_take36_locks_2x.png")


if __name__ == "__main__":
    main()
