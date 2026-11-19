"""AI-38 battle mock take 34 (Tushar 2026-11-19 19:39 on take 33), 1280x720.

Usage: python3 art_src/style/battle_take34_status_mock.py <playtest_dir> <out_dir>
  - hovered numbers get no frame: hovering the number shows the breakdown of every damage source and nothing else
    changes on the plate or card;
  - loadout passives: two dedicated slots centred under the portrait (a loadout holds at most 2; an empty slot stays
    as a dim socket); hovering a slot gives that passive's name and what it does. Passives never enter the status
    column (it is for statuses that last a few rounds);
  - the full timeline view (T) calmed: move tiles are a dark tint of their kind colour with a 3 px kind-coloured top band and a kind tinted
    rim, the skill icon drawn larger and brighter; the only solid red left is the blow tick where a hit lands.
    Locks keep their hatch.
Everything else is take 33 (signed off in principle).
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take13_mock as T13  # noqa: E402
import battle_take23_timeline_mock as T23  # noqa: E402
import battle_take24_mock as T24  # noqa: E402
import battle_take33_status_mock as T33  # noqa: E402

PASSIVES = (("brace", "Sword Training", "Drilled with blades: +1 sword proficiency."),
            ("aegis", "Hardy", "Tough constitution: +2 CONS."))


def one_loss(img):
    T33.one_loss(img, hover=False)
    T33.tip(img, 200, 92, "Kaelen this round",
            [("Brawler, block 2", "-12", B.IMPACT), ("Brawler, block 5", "-10", B.IMPACT),
             ("Bleed, round end", "-8", B.IMPACT)], w=230, foot=("Planned loss", "-30", B.IMPACT))
    return img


def final_numbers(img, vals, up=False):
    T33.final_numbers(img, vals, up=up, hover=False)
    if up:
        rows = [("Base", "34", B.PARCH), ("Empower", "+20%", T33.UP_A), ("Fortify", "-6%", T33.DOWN_A)]
        foot = ("Hammer strike", f"-{vals[0]}", T33.UP)
    else:
        rows = [("Base", "34", B.PARCH), ("Weaken", "-15%", T33.DOWN_A), ("Fortify", "-6%", T33.DOWN_A)]
        foot = ("Hammer strike", f"-{vals[0]}", T33.DOWN)
    T33.tip(img, 610, 262, "Damage to Footman", rows, w=210, foot=foot)
    return img


def passives(img, hover=0, slots=PASSIVES):
    """Two sockets centred under the 48 px portrait (x 21..68): 18 px each, 6 px apart."""
    d = ImageDraw.Draw(img)
    d.rectangle((20, 134, 70, 153), fill=T33.PLATE)
    x0 = 21 + (48 - (18 * 2 + 6)) // 2
    for i in range(2):
        x, y = x0 + i * 24, 135
        on = i < len(slots)
        d.rectangle((x, y, x + 17, y + 17), fill=(16, 18, 25), outline=B.BRONZE if on else (52, 58, 70))
        if on:
            im = B.slot_icon(slots[i][0], 64).resize((16, 16), Image.BOX)
            img.paste(im, (x + 1, y + 1), im)
    if hover is not None and hover < len(slots):
        x = x0 + hover * 24
        d.rectangle((x - 1, 134, x + 18, 153), outline=B.GOLD)
        _, name, what = slots[hover]
        w = 26 + int(d.textlength(what, font=B.SMALL))
        h = 30 + 17 + 6
        T24.shadow(img, (200, 126, 200 + w, 126 + h))
        T24.nine(img, (200, 126, 200 + w, 126 + h), "hud")
        d = ImageDraw.Draw(img)
        d.text((212, 133), name, font=B.TITLE_S, fill=B.GOLD)
        d.text((212, 156), what, font=B.SMALL, fill=B.PARCH)
    return img


def calm_row(img, d, y, face, name, moves, blows, foe, sel, cursor):
    bw = (T23.X1 - T23.X0 - T23.NAME_W - 8) / 6
    d.rectangle((T23.X0, y, T23.X1, y + T23.ROW_H - 2), fill=B.SLATE if not foe else (34, 30, 36),
                outline=B.GOLD if sel else B.BASALT)
    img.paste(face.resize((26, 26)), (T23.X0 + 4, y + 3))
    d.text((T23.X0 + 34, y + 8), name, font=B.SMALL, fill=(222, 160, 150) if foe else B.PARCH)
    bx0 = T23.X0 + T23.NAME_W
    for icon, s, n, ghost in moves:
        x = bx0 + s * bw + 2
        w = n * bw - 4
        kind = T13.KIND_COL[T13.SKILL_KIND[icon]]
        rim = T13.mix(kind, (20, 22, 28), 0.45)
        fill = T13.mix(kind, (22, 25, 32), 0.3)           # a dark tint of the kind, not the full colour
        if ghost:
            kind, rim = T13.mix(kind, fill, 0.55), T13.mix(rim, fill, 0.5)
        d.rectangle((x, y + 3, x + w, y + T23.ROW_H - 6), fill=fill, outline=rim)
        d.rectangle((x, y + 3, x + w, y + 5), fill=kind)
        ic = B.slot_icon(icon, 64).resize((24, 24), Image.BOX)
        if ghost:
            ic.putalpha(ic.getchannel("A").point(lambda v: min(v, 140)))
        img.paste(ic, (int(x + w / 2 - 12), y + 4), ic)
    for blk, tgt in blows:
        x = bx0 + blk * bw + bw - 22
        d.rectangle((x - 2, y + T23.ROW_H - 8, x + 18, y + T23.ROW_H - 5), fill=B.IMPACT)
        img.paste(T23.face_of(tgt, 14), (int(x + 2), y + 6))
    if cursor is not None:
        cx = bx0 + cursor * bw
        d.rectangle((cx, y, cx + bw, y + T23.ROW_H - 2), outline=B.GOLD)


def main():
    pt, out = Path(sys.argv[1]), Path(sys.argv[2])
    shot = Image.open(pt / "048_battle_plan.png").convert("RGB")
    hp = one_loss(T33.base(shot))
    wk = final_numbers(T33.base(shot), (27, 10, 10))
    p0, p1 = passives(T33.base(shot), hover=0), passives(T33.base(shot), hover=None, slots=PASSIVES[:1])
    old = T33.locks(T33.base(shot))
    T23.row = calm_row
    calm = T33.locks(T33.base(shot))
    calm.save(out / "battle_take34_timeline_calm_720p.png")
    tiles = [("Hover on the HP number: no frame, the breakdown only", T33.z2(hp, (14, 84, 440, 200))),
             ("Hover on a hit number: no frame, base and each modifier", T33.z2(wk, (600, 256, 1000, 384))),
             ("Two passive slots centred under the portrait, hover one for its name and effect", T33.z2(p0, (14, 80, 470, 190))),
             ("One passive equipped: the second slot stays an empty socket", T33.z2(p1, (14, 80, 200, 170))),
             ("Full timeline (T) now: solid kind fills", T33.z2(old, (204, 70, 1076, 430)).resize((872, 360))),
             ("Full timeline (T) calmed: dark kind tints, a kind band and rim, bigger icons; red only where a hit lands",
              T33.z2(calm, (204, 70, 1076, 430)).resize((872, 360)))]
    T33.label_tiles(tiles, "Take 34: status follow-ups after the take 33 review (2x)").save(out / "battle_take34_status_2x.png")


if __name__ == "__main__":
    main()
