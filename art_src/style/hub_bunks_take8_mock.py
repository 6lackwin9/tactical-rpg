"""Hub bunk room take 8 (Tushar 2026-11-19 17:44): take 7's A (the lunette) with his notes, 1280x720 at 1:1.

Usage: python3 art_src/style/hub_bunks_take8_mock.py <hub shot.png> <out_dir>
  - the keepsake silhouettes in flat gold did not read as the keepsakes (Dunstan's least): each founder now has a drawn
    emblem icon, bold gold shapes with a dark outline and one accent colour where it helps, after the keepsake
    (CAST1_GEAR 7): Kaelen the gouged sigil, Vesper the runner whistle, Maren the copyist quill, Hollis the focusing
    monocle, Dunstan the shackle link (two links, one broken), Elyse the cracked retort;
  - the footlockers get a dark 1 px outline so they sit on the planks like the beds (the real pair is queued:
    props_hub_footlocker_2);
  - the sleepers: the blanket now comes up over the shoulders, no strip of pillow between the chin and the cover.
"""
import json
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import hub_bunks_take1_mock as H1  # noqa: E402
import hub_bunks_take2_mock as H2  # noqa: E402
import hub_bunks_take3_mock as H3  # noqa: E402
import hub_bunks_take4_mock as H4  # noqa: E402
import hub_bunks_take5_mock as H5  # noqa: E402
import hub_bunks_take7_mock as H7  # noqa: E402

WALL_Y0 = H4.WALL_Y0
BED_TOP = H7.BED_TOP
G, HI, LINE = (222, 178, 84, 255), (252, 228, 150, 255), (34, 12, 10, 255)
LENS, BREW = (176, 206, 220, 255), (96, 176, 150, 255)


def outline(im, col):
    """A 1 px outline (4-neighbour) round every opaque pixel, on a canvas 1 px larger each side."""
    out = Image.new("RGBA", (im.width + 2, im.height + 2))
    out.alpha_composite(im, (1, 1))
    src, px = out.copy().load(), out.load()
    for y in range(out.height):
        for x in range(out.width):
            if src[x, y][3]:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                if 0 <= x + dx < out.width and 0 <= y + dy < out.height and src[x + dx, y + dy][3]:
                    px[x, y] = col
                    break
    return out


def icon(name):
    im = Image.new("RGBA", (15, 11))
    d = ImageDraw.Draw(im)
    if name == "Kaelen":        # the gouged sigil: a seal disc, a ring inset, a deep gouge across it
        d.ellipse((2, 0, 12, 10), fill=G)
        d.arc((2, 0, 12, 10), 140, 250, fill=HI)
        d.ellipse((5, 3, 9, 7), outline=LINE)
        d.line((3, 9, 11, 1), fill=LINE, width=2)
    elif name == "Vesper":      # the runner whistle: a round barrel, the vent on top, a long mouthpiece
        d.ellipse((1, 2, 8, 9), fill=G)
        d.rectangle((5, 2, 14, 5), fill=G)
        d.line((5, 2, 13, 2), fill=HI)
        d.point((2, 4), fill=HI)
        d.rectangle((6, 3, 7, 4), fill=LINE)
        d.ellipse((3, 4, 6, 7), outline=(150, 104, 40, 255))
    elif name == "Maren":       # the copyist quill: a curved vane with barbs, a bare shaft, the inked nib
        d.polygon([(14, 0), (11, 0), (6, 4), (4, 7), (6, 7), (10, 5), (14, 2)], fill=G)
        d.line((13, 1, 3, 9), fill=HI)
        for x, y in ((11, 3), (9, 4), (7, 6)):
            d.point((x, y + 1), fill=LINE)
        d.line((4, 8, 2, 10), fill=G)
        d.point((1, 10), fill=LINE)
        d.point((2, 10), fill=LINE)
    elif name == "Hollis":      # the focusing monocle: a thick rim, a pale lens with a glint, its chain
        d.ellipse((1, 0, 9, 8), fill=G)
        d.ellipse((3, 2, 7, 6), fill=LENS)
        d.point((4, 3), fill=(240, 250, 255, 255))
        for i, (x, y) in enumerate(((9, 7), (10, 8), (11, 8), (12, 9), (13, 10))):
            d.point((x, y), fill=G if i % 2 == 0 else HI)
    elif name == "Dunstan":     # the shackle link: a long link through an upright one, the upright split open
        d.ellipse((0, 2, 9, 8), outline=G, width=2)
        d.ellipse((7, 0, 13, 10), outline=G, width=2)
        d.rectangle((9, 9, 11, 10), fill=(0, 0, 0, 0))
        d.line((7, 4, 8, 4), fill=G)
        d.point((2, 3), fill=HI)
        d.point((9, 1), fill=HI)
    elif name == "Elyse":       # the cracked retort: a round bulb with its brew, the long bent neck, a crack
        d.ellipse((1, 3, 8, 10), fill=G)
        d.chord((2, 4, 7, 9), 0, 180, fill=BREW)
        d.line((6, 4, 13, 0), fill=G, width=2)
        d.point((3, 4), fill=HI)
        d.line((4, 5, 5, 7), fill=LINE)
        d.point((6, 7), fill=LINE)
    return outline(im, LINE)


def lunette(img, cx, name):
    d = ImageDraw.Draw(img)
    top = WALL_Y0 + 12
    x0, x1 = cx - 20, cx + 20
    d.pieslice((x0, top + 2, x1, top + 24), 180, 360, fill=H7.CLOTH)
    d.rectangle((x0, top + 13, x1, BED_TOP + 2), fill=H7.CLOTH)
    d.arc((x0, top + 2, x1, top + 24), 180, 360, fill=H7.GOLD_T)
    d.line((x0, top + 13, x0, BED_TOP + 2), fill=H7.CLOTH_D)
    d.line((x1, top + 13, x1, BED_TOP + 2), fill=H7.CLOTH_D)
    ic = icon(name)
    img.alpha_composite(ic, (cx - ic.width // 2, top + 3))


def sleeper(img, cx, top, name, bed):
    """The head and shoulders lower on the pillow, the blanket turned down over the shoulders."""
    body, box = H4.FRONT[name]
    j = json.loads((H2.UNITS / body / "clips.json").read_text())
    row = [c["id"] for c in j["clips"]].index("idle_down")
    sheet = Image.open(H2.UNITS / body / "sheet.png").convert("RGBA")
    head = sheet.crop((box[0], row * 48 + box[1], box[2], row * 48 + box[3] + 6))
    img.alpha_composite(head, (cx - head.width // 2, top + 31 - head.height))
    cover = bed.crop((0, 27, bed.width, bed.height))
    img.alpha_composite(cover, (cx - bed.width // 2, top + 27))


def footlocker(img, cx, bed_bottom, state):
    if state == "none":
        return
    layer = Image.new("RGBA", img.size)
    H7.footlocker(layer, cx, bed_bottom, state)
    box = layer.getbbox()
    piece = outline(layer.crop(box), (22, 14, 8, 255))
    img.alpha_composite(piece, (box[0] - 1, box[1] - 1))


def bunk(img, cx, name, cls, state, opened):
    d = ImageDraw.Draw(img)
    H4.niche(d, cx)
    lunette(img, cx, name)
    bed = H3.tint(H3.prop("cot_side"), H1.CLASS[cls], 30)
    img.alpha_composite(bed, (cx - bed.width // 2, BED_TOP))
    if state == "benched" and name in H4.FRONT:
        sleeper(img, cx, BED_TOP, name, bed)
    H5.pegboard(img, cx, H4.gear_img(name))
    fl = "none" if name in H5.CRYPT else ("open" if name == opened else "closed")
    footlocker(img, cx, BED_TOP + bed.height, fl)
    H1.plate(ImageDraw.Draw(img), cx, WALL_Y0 - 9, name, cls, 80)


def room(shot, act, opened=None):
    img = H3.clean_room(shot)
    H3.planks(img, 352, 552, 400, 658)
    H4.move_stove(img, shot)
    H4.wall(img)
    for x, (name, cls, state, _) in zip(H4.XS, H1.ROSTER[:6]):
        bunk(img, x, name, cls, state, opened)
    if act == 1:
        H5.act1_props(img)
    else:
        H4.south_bed(img, 354, 592, H1.ROSTER[6], False)
        H4.south_bed(img, 928 - 96 - 2, 592, H1.ROSTER[7], True)
        footlocker(img, 354 + 96 + 18, 618, "closed")
    return img.crop(H4.CROP).convert("RGB")


def main():
    shot = Image.open(sys.argv[1]).convert("RGBA")
    out = Path(sys.argv[2])
    a = room(shot, 1, "Maren")
    w, h = a.size
    icons = Image.new("RGB", (6 * 19 + 2, 15), H7.CLOTH)
    for i, (name, *_r) in enumerate(H1.ROSTER[:6]):
        icons.paste(icon(name), (2 + i * 19, 1), icon(name))
    icons = icons.resize((icons.width * 8, icons.height * 8), Image.NEAREST)
    z = a.crop((40, 20, 660, 156)).resize((1240, 272), Image.NEAREST)
    sheet = Image.new("RGB", (max(w, 1240, icons.width) + 12, 22 + h + 26 + z.height + 26 + icons.height + 30), B.VOID)
    d = ImageDraw.Draw(sheet)
    y = 3
    d.text((6, y), "1:1, Act 1: lunette emblems; Maren's loadout open (her footlocker open), Hollis in the crypts (none)",
           font=B.SMALL, fill=B.GOLD)
    sheet.paste(a, (0, y + 18)); y += 18 + h + 8
    d.text((6, y), "2x: the emblems, outlined footlockers, the blanket over the shoulders", font=B.SMALL, fill=B.GOLD)
    sheet.paste(z, (0, y + 18)); y += 18 + z.height + 8
    d.text((6, y), "8x: Kaelen gouged sigil, Vesper runner whistle, Maren copyist quill, Hollis focusing monocle, "
           "Dunstan shackle link, Elyse cracked retort", font=B.SMALL, fill=B.GOLD)
    sheet.paste(icons, (0, y + 18))
    B.label(sheet, "Hub bunks take 8: founder emblem icons in the lunette, clean footlocker edges, sleepers tucked in").save(
        out / "hub_bunks_take8_sheet.png")
    a.save(out / "hub_bunks_take8_act1_720p.png")
    room(shot, 2).save(out / "hub_bunks_take8_act2_720p.png")


if __name__ == "__main__":
    main()
