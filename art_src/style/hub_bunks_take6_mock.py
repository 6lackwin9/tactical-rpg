"""Hub bunk room take 6 (Tushar 2026-11-19 17:34): take 5's P with his notes, 1280x720 at 1:1.

Usage: python3 art_src/style/hub_bunks_take6_mock.py <hub shot.png> <out_dir>
  - the pennant carries the founder's own emblem (their keepsake's silhouette in gold thread, from `bunk_keepsake_<id>`), not the class glyph (the
    class is the blanket's colour); the cloth is one dark oxblood for all founders; recruits have no pennant;
  - one brass plate for everyone: no founder or recruit difference on the plates;
  - the footlocker redrawn plainer: closed, a flat timber chest with its lid seam, two iron bands and a brass hasp, the
    lid top showing; open, the lid up behind it and one bundle of kit inside;
  - the Act 1 props and the footlocker art are stand-ins: the real ones are in the image queue (AI-31, AI-32).
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import hub_bunks_take1_mock as H1  # noqa: E402
import hub_bunks_take3_mock as H3  # noqa: E402
import hub_bunks_take4_mock as H4  # noqa: E402
import hub_bunks_take5_mock as H5  # noqa: E402

WALL_Y0, WALL_Y1 = H4.WALL_Y0, H4.WALL_Y1
CLOTH, CLOTH_D = (112, 34, 30), (72, 22, 20)
WOOD, WOOD_L, WOOD_D, BAND = (104, 72, 42), (134, 96, 58), (62, 40, 24), (78, 80, 86)


def pennant(img, cx, name):
    d = ImageDraw.Draw(img)
    top = WALL_Y0 + 15
    d.line((cx - 10, top, cx + 10, top), fill=(150, 150, 150))
    d.polygon([(cx - 9, top + 1), (cx + 9, top + 1), (cx + 9, top + 19), (cx, top + 15), (cx - 9, top + 19)], fill=CLOTH)
    d.line((cx - 9, top + 1, cx - 9, top + 19), fill=CLOTH_D)
    d.line((cx - 7, top + 3, cx + 7, top + 3), fill=(176, 132, 64))
    f = H1.PROPS / f"bunk_keepsake_{name.lower()}" / "v0.png"
    if f.exists():
        # stitched in gold thread: the keepsake's silhouette, so it reads on the cloth at 1:1
        em = Image.open(f).convert("RGBA")
        a = em.getchannel("A").point(lambda v: 255 if v > 100 else 0)
        gold = Image.new("RGBA", em.size, (222, 178, 84, 255)); gold.putalpha(a)
        img.alpha_composite(gold, (cx - em.width // 2, top + 5))


def footlocker(img, cx, bed_bottom, state):
    if state == "empty":
        return
    d = ImageDraw.Draw(img)
    x0, y0, w = cx - 14, bed_bottom + 4, 28
    if state == "crypt":                                   # closed: the lid top, its seam, bands and the hasp
        d.rectangle((x0, y0, x0 + w, y0 + 5), fill=WOOD_L, outline=WOOD_D)
        d.rectangle((x0, y0 + 5, x0 + w, y0 + 15), fill=WOOD, outline=WOOD_D)
        d.line((x0 + 1, y0 + 7, x0 + w - 1, y0 + 7), fill=WOOD_D)
        for bx in (x0 + 5, x0 + w - 5):
            d.line((bx, y0, bx, y0 + 15), fill=BAND, width=2)
        d.rectangle((x0 + w // 2 - 2, y0 + 6, x0 + w // 2 + 2, y0 + 10), fill=(196, 150, 70), outline=(110, 80, 40))
    else:                                                  # open: the lid up behind, one bundle inside
        d.rectangle((x0, y0 - 8, x0 + w, y0), fill=WOOD, outline=WOOD_D)
        d.rectangle((x0, y0, x0 + w, y0 + 12), fill=WOOD, outline=WOOD_D)
        d.rectangle((x0 + 2, y0 + 1, x0 + w - 2, y0 + 3), fill=(38, 28, 20))
        d.rectangle((x0 + 6, y0, x0 + w - 6, y0 + 3), fill=(168, 158, 136))
        for bx in (x0 + 5, x0 + w - 5):
            d.line((bx, y0 + 4, bx, y0 + 12), fill=BAND, width=2)


def bunk(img, cx, name, cls, state, founder):
    d = ImageDraw.Draw(img)
    H4.niche(d, cx)
    if founder:
        pennant(img, cx, name)
    bed = H3.tint(H3.prop("cot_side"), H1.CLASS[cls], 30)
    top = WALL_Y1 - 26
    img.alpha_composite(bed, (cx - bed.width // 2, top))
    if state == "benched" and name in H4.FRONT:
        H4.sleeper(img, cx, top, name, bed)
    H5.pegboard(img, cx, H4.gear_img(name) if founder else H5.recruit_gear(cls))
    footlocker(img, cx, top + bed.height, "crypt" if name in H5.CRYPT else state)
    H1.plate(ImageDraw.Draw(img), cx, WALL_Y0 - 9, name, cls, 80)


def room(shot, act):
    img = H3.clean_room(shot)
    H3.planks(img, 352, 552, 400, 658)
    H4.move_stove(img, shot)
    H4.wall(img)
    for x, (name, cls, state, _) in zip(H4.XS, H1.ROSTER[:6]):
        bunk(img, x, name, cls, state, True)
    if act == 1:
        H5.act1_props(img)
    else:
        H4.south_bed(img, 354, 592, H1.ROSTER[6], False)
        H4.south_bed(img, 928 - 96 - 2, 592, H1.ROSTER[7], True)
        footlocker(img, 354 + 96 + 18, 618, H1.ROSTER[6][2])
    return img.crop(H4.CROP).convert("RGB")


def main():
    shot = Image.open(sys.argv[1]).convert("RGBA")
    out = Path(sys.argv[2])
    a1, a2 = room(shot, 1), room(shot, 2)
    w, h = a1.size
    top = Image.new("RGB", (w * 2 + 12, h + 22), B.VOID)
    d = ImageDraw.Draw(top)
    d.text((6, 3), "Act 1 (Hollis in the crypts: closed footlocker)", font=B.SMALL, fill=B.GOLD)
    d.text((w + 18, 3), "Act 2 (a recruit in the west bunk: no pennant, the same plate)", font=B.SMALL, fill=B.GOLD)
    top.paste(a1, (0, 20)); top.paste(a2, (w + 12, 20))
    z = a1.crop((40, 20, 660, 156)).resize((1240, 272), Image.NEAREST)
    sheet = Image.new("RGB", (top.width, top.height + z.height + 24), B.VOID)
    sheet.paste(top, (0, 0)); sheet.paste(z, (0, top.height + 22))
    ImageDraw.Draw(sheet).text((6, top.height + 4), "2x: each founder's own emblem on the pennant; open footlockers, "
                               "Hollis's closed", font=B.SMALL, fill=B.GOLD)
    B.label(sheet, "Hub bunks take 6: founder emblems on the pennants, plainer footlockers").save(
        out / "hub_bunks_take6_sheet.png")
    a1.save(out / "hub_bunks_take6_act1_720p.png"); a2.save(out / "hub_bunks_take6_act2_720p.png")


if __name__ == "__main__":
    main()
