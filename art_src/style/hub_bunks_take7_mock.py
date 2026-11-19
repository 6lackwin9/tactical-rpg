"""Hub bunk room take 7 (Tushar 2026-11-19 17:39): take 6 with his notes, 1280x720 at 1:1.

Usage: python3 art_src/style/hub_bunks_take7_mock.py <hub shot.png> <out_dir>
  - the take 6 pennant hid behind the headboard; the emblem now sits where the arch shows, two options:
    A. a lunette: an oxblood cloth filling the arch head above the headboard, the emblem in gold thread (rec.);
    B. a roundel: a painted oxblood disc hung from the keystone, over the arch top;
  - the footlocker flow (his): closed for everyone on the roster by default; it opens while you change that member's
    gear (click it, the loadout screen opens over the stash), and closes when you are done; a member whose loadout is
    locked (Hollis in the crypts) has no footlocker at all. The 2x strip shows Maren's opened;
  - the open footlocker here is a stand-in (lid up showing its inner face, kit at the rim); the real open and closed
    art is the image queue's props_hub_footlocker_1 (AI-32).
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
import hub_bunks_take6_mock as H6  # noqa: E402

WALL_Y0, WALL_Y1 = H4.WALL_Y0, H4.WALL_Y1
CLOTH, CLOTH_D, GOLD_T = H6.CLOTH, H6.CLOTH_D, (176, 132, 64)
WOOD, WOOD_L, WOOD_D, BAND = H6.WOOD, H6.WOOD_L, H6.WOOD_D, H6.BAND
BED_TOP = WALL_Y1 - 26


def emblem(img, name, cx, y):
    f = H1.PROPS / f"bunk_keepsake_{name.lower()}" / "v0.png"
    if f.exists():
        em = Image.open(f).convert("RGBA")
        a = em.getchannel("A").point(lambda v: 255 if v > 100 else 0)
        gold = Image.new("RGBA", em.size, (222, 178, 84, 255))
        gold.putalpha(a)
        img.alpha_composite(gold, (cx - em.width // 2, y))


def lunette(img, cx, name):
    """A: the arch head filled with cloth down to the headboard, gold edging on the arch, the emblem centred."""
    d = ImageDraw.Draw(img)
    top = WALL_Y0 + 12
    x0, x1 = cx - 20, cx + 20
    d.pieslice((x0, top + 2, x1, top + 24), 180, 360, fill=CLOTH)
    d.rectangle((x0, top + 13, x1, BED_TOP + 2), fill=CLOTH)
    d.arc((x0, top + 2, x1, top + 24), 180, 360, fill=GOLD_T)
    d.line((x0, top + 13, x0, BED_TOP + 2), fill=CLOTH_D)
    d.line((x1, top + 13, x1, BED_TOP + 2), fill=CLOTH_D)
    emblem(img, name, cx, top + 4)


def roundel(img, cx, name):
    """B: a painted disc hung from the keystone, its lower half over the arch."""
    d = ImageDraw.Draw(img)
    cy, r = WALL_Y0 + 18, 9
    d.ellipse((cx - r - 1, cy - r - 1, cx + r + 1, cy + r + 1), fill=GOLD_T)
    d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=CLOTH)
    d.arc((cx - r, cy - r, cx + r, cy + r), 20, 160, fill=CLOTH_D)
    emblem(img, name, cx, cy - 6)


def footlocker(img, cx, bed_bottom, state):
    """closed (default), open (while its member's loadout is open), none (loadout locked)."""
    if state == "none":
        return
    if state == "closed":
        H6.footlocker(img, cx, bed_bottom, "crypt")       # take 6's closed chest, which he found readable
        return
    d = ImageDraw.Draw(img)
    x0, y0, w = cx - 14, bed_bottom + 4, 28
    d.polygon([(x0 + 1, y0 + 1), (x0 + w - 1, y0 + 1), (x0 + w + 1, y0 - 11), (x0 - 1, y0 - 11)], fill=WOOD_D)
    d.polygon([(x0 + 3, y0), (x0 + w - 3, y0), (x0 + w - 1, y0 - 9), (x0 + 1, y0 - 9)], fill=(84, 58, 34))
    for bx in (x0 + 5, x0 + w - 5):                         # the bands on the lid's inner face
        d.line((bx, y0 - 9, bx, y0), fill=BAND)
    d.rectangle((x0, y0, x0 + w, y0 + 4), fill=(30, 22, 16), outline=WOOD_D)   # the open mouth
    d.rectangle((x0 + 4, y0 - 2, x0 + 12, y0 + 3), fill=(168, 158, 136))         # folded kit at the rim
    d.rectangle((x0 + 14, y0 - 1, x0 + 23, y0 + 3), fill=(120, 124, 132))
    d.line((x0 + 15, y0 - 1, x0 + 22, y0 - 1), fill=(176, 180, 186))
    d.rectangle((x0, y0 + 4, x0 + w, y0 + 15), fill=WOOD, outline=WOOD_D)
    for bx in (x0 + 5, x0 + w - 5):
        d.line((bx, y0 + 4, bx, y0 + 15), fill=BAND, width=2)
    d.rectangle((x0 + w // 2 - 2, y0 + 5, x0 + w // 2 + 2, y0 + 8), fill=(196, 150, 70), outline=(110, 80, 40))


def bunk(img, cx, name, cls, state, founder, style, opened):
    d = ImageDraw.Draw(img)
    H4.niche(d, cx)
    if founder:
        (lunette if style == "A" else roundel)(img, cx, name)
    bed = H3.tint(H3.prop("cot_side"), H1.CLASS[cls], 30)
    img.alpha_composite(bed, (cx - bed.width // 2, BED_TOP))
    if state == "benched" and name in H4.FRONT:
        H4.sleeper(img, cx, BED_TOP, name, bed)
    H5.pegboard(img, cx, H4.gear_img(name) if founder else H5.recruit_gear(cls))
    fl = "none" if name in H5.CRYPT else ("open" if name == opened else "closed")
    footlocker(img, cx, BED_TOP + bed.height, fl)
    H1.plate(ImageDraw.Draw(img), cx, WALL_Y0 - 9, name, cls, 80)


def room(shot, act, style, opened=None):
    img = H3.clean_room(shot)
    H3.planks(img, 352, 552, 400, 658)
    H4.move_stove(img, shot)
    H4.wall(img)
    for x, (name, cls, state, _) in zip(H4.XS, H1.ROSTER[:6]):
        bunk(img, x, name, cls, state, True, style, opened)
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
    a, b = room(shot, 1, "A"), room(shot, 1, "B")
    w, h = a.size
    top = Image.new("RGB", (w * 2 + 12, h + 22), B.VOID)
    d = ImageDraw.Draw(top)
    d.text((6, 3), "A. Lunette: the emblem fills the arch head (rec.)", font=B.SMALL, fill=B.GOLD)
    d.text((w + 18, 3), "B. Roundel hung from the keystone", font=B.SMALL, fill=B.GOLD)
    top.paste(a, (0, 20)); top.paste(b, (w + 12, 20))
    zone = (40, 20, 660, 156)
    za = a.crop(zone).resize((1240, 272), Image.NEAREST)
    zb = b.crop(zone).resize((1240, 272), Image.NEAREST)
    zo = room(shot, 1, "A", "Maren").crop(zone).resize((1240, 272), Image.NEAREST)
    caps = ["2x A: all footlockers closed (the hub at rest); Hollis is in the crypts, so none",
            "2x B: the roundel", "2x A: Maren's footlocker opened while her loadout is open (stand-in art)"]
    sheet = Image.new("RGB", (top.width, top.height + 3 * (272 + 24)), B.VOID)
    sheet.paste(top, (0, 0))
    ds = ImageDraw.Draw(sheet)
    for i, (z, c) in enumerate(zip((za, zb, zo), caps)):
        y = top.height + i * (272 + 24)
        ds.text((6, y + 4), c, font=B.SMALL, fill=B.GOLD)
        sheet.paste(z, (0, y + 22))
    B.label(sheet, "Hub bunks take 7: the emblem in the arch, footlockers closed by default").save(
        out / "hub_bunks_take7_sheet.png")
    a.save(out / "hub_bunks_take7_A_720p.png"); b.save(out / "hub_bunks_take7_B_720p.png")
    room(shot, 2, "A").save(out / "hub_bunks_take7_A_act2_720p.png")


if __name__ == "__main__":
    main()
