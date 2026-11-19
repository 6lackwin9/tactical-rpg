"""Hub bunk room take 3 (Tushar 2026-11-19 17:13): over the current hub shot (PR7-43 built), 1280x720 at 1:1.

Usage: python3 art_src/style/hub_bunks_take3_mock.py <hub shot.png> <out_dir>
(the shot: scenario story_battle_hub `hub`, from develop). His notes on take 2 and what changes:
  - the south-right bunk ran into the table's stool: both south bunks now lie sideways (the `cot` art, head to the
    west), the right one moved east of the stool, clear of the stove;
  - the plate sat on the pillow: north plates go on the wall band above the bed head, south plates stand on a short
    post at the head end, clear of the bed;
  - the timber screens read oddly: gone; one plank floor under the whole hall and the beds;
  - the kiln arch behind each bed read oddly: the bed (the `cot_side` art) now stands on the planks against the wall,
    with either 1. a shallow arched niche in the wall over the head (rec.) or 2. no arch, a timber headboard plank and
    a peg shelf on the wall.
Gear on the wall pegs and keepsakes on the bed are the shipped PR7-43 placeholders; blankets take the class colour.
Benched founders sleep in their bunk (the take 2 stand-in: the sprite's head on the pillow).
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import hub_bunks_take1_mock as H1  # noqa: E402
import hub_bunks_take2_mock as H2  # noqa: E402

PROPS = H1.PROPS
RENDER = H1.RENDER                    # the PR7-23 render: the same camera, a clean wall between the old arches
WALL_Y0, WALL_Y1, FLOOR_Y1 = 368, 420, 470
DOOR = H1.DOOR
XS = [392, 472, 552, 712, 792, 872]   # the six founder beds, 80 px apart, the door between the third and fourth
CROP = (300, 340, 960, 664)


def prop(name):
    return Image.open(PROPS / name / "v0.png").convert("RGBA")


def tint(im, col, y0):
    """The grey blanket (below row y0) in the class colour, its shading kept."""
    px = im.load()
    for y in range(y0, im.height):
        for x in range(im.width):
            r, g, b, a = px[x, y]
            if a and abs(r - g) < 16 and abs(g - b) < 20 and 70 < r < 225:
                k = r / 165
                px[x, y] = (min(255, int(col[0] * k)), min(255, int(col[1] * k)), min(255, int(col[2] * k)), a)
    return im


def planks(base, x0, y0, x1, y1):
    """The hall's planks (a clean run from the hall floor) over a rect."""
    plank = base.crop((606, 604, 700, 644))
    for y in range(y0, y1, plank.height):
        for x in range(x0, x1, plank.width):
            base.paste(plank.crop((0, 0, min(plank.width, x1 - x), min(plank.height, y1 - y))), (x, y))


def clean_room(shot):
    """The north wall without the kiln arches, planks from the wall foot to the hall, the south cots taken out."""
    base = shot.copy()
    render = Image.open(RENDER).convert("RGBA")
    wall = render.crop((790, WALL_Y0 - 10, 810, WALL_Y1))      # from above the old plates' top edge
    for x in range(352, 928, 20):
        if not DOOR[0] <= x < DOOR[1]:
            base.paste(wall, (x, WALL_Y0 - 10))
    for x0, x1 in ((352, DOOR[0]), (DOOR[1], 928)):
        planks(base, x0, WALL_Y1, x1, FLOOR_Y1)
    planks(base, 400, 574, 470, 658)            # the old south-left cot
    planks(base, 818, 574, 880, 658)            # the old south-right cot
    return base


def niche(d, cx):
    """1: a shallow arched recess in the wall over the bed head (wall band only, it never reaches the floor)."""
    x0, x1, top, bot = cx - 22, cx + 22, WALL_Y0 + 14, WALL_Y1
    d.rectangle((x0, top + 10, x1, bot), fill=(22, 24, 28))
    d.pieslice((x0, top, x1, top + 22), 180, 360, fill=(22, 24, 28))
    d.arc((x0 - 2, top - 2, x1 + 2, top + 24), 180, 360, fill=(88, 86, 80), width=2)
    d.line((x0 - 2, top + 11, x0 - 2, bot), fill=(88, 86, 80), width=2)
    d.line((x1 + 2, top + 11, x1 + 2, bot), fill=(58, 56, 52), width=2)


def headboard(d, cx):
    """2: no arch; a timber headboard plank on the wall and a peg shelf beside it."""
    d.rectangle((cx - 26, WALL_Y0 + 22, cx + 26, WALL_Y1 - 2), fill=H1.TIMBER, outline=H1.TIMBER_D)
    for k in (-1, 1):
        d.line((cx + 13 * k, WALL_Y0 + 23, cx + 13 * k, WALL_Y1 - 3), fill=H1.TIMBER_D)


def north_bed(img, cx, entry, style):
    name, cls, state, _ = entry
    d = ImageDraw.Draw(img)
    (niche if style == 1 else headboard)(d, cx)
    bed = prop("cot_side")
    if cls:
        bed = tint(bed, H1.CLASS[cls], 30)
    top = WALL_Y1 - 26
    img.alpha_composite(bed, (cx - bed.width // 2, top))
    if state == "benched":
        H2.sleeper(img, cx, top - 34, name)
    gear = PROPS / f"bunk_gear_{name.lower()}" / "v0.png"
    if gear.exists():
        g = Image.open(gear).convert("RGBA")
        img.alpha_composite(g, (cx + 25, WALL_Y0 + 22))
    ks = PROPS / f"bunk_keepsake_{name.lower()}" / "v0.png"
    if ks.exists():
        img.alpha_composite(Image.open(ks).convert("RGBA"), (cx - 14, top + 62))
    H1.plate(ImageDraw.Draw(img), cx, WALL_Y0 + 1, name, cls, 80)


def south_bed(img, x, y, entry):
    """A Timber Frames bunk lying sideways (`cot`, head to the west); its plate on a short post at the head end."""
    name, cls, state, _ = entry
    bed = prop("cot")
    if cls and state != "empty":
        bed = tint(bed, H1.CLASS[cls], 0)
    img.alpha_composite(bed, (x, y))
    d = ImageDraw.Draw(img)
    px = x - 6
    d.rectangle((px - 1, y + 8, px + 2, y + bed.height - 6), fill=H1.TIMBER, outline=H1.TIMBER_D)
    if name:
        H1.plate(d, px, y - 8, name, cls, 80)
    else:
        d.rectangle((px - 12, y - 8, px + 14, y + 5), fill=H1.BRASS_D, outline=H1.BRASS)


def scene(shot, style):
    img = clean_room(shot)
    for x, e in zip(XS, H1.ROSTER[:6]):
        north_bed(img, x, e, style)
    south_bed(img, 394, 590, H1.ROSTER[6])          # left: east of the barrels, under the west table
    south_bed(img, 826, 594, H1.ROSTER[7])          # right: east of the stool, under the stove
    return img.crop(CROP).convert("RGB")


def main():
    shot = Image.open(sys.argv[1]).convert("RGBA")
    out = Path(sys.argv[2])
    before = shot.crop(CROP).convert("RGB")
    a, b = scene(shot, 1), scene(shot, 2)
    w, h = a.size
    one = Image.new("RGB", (w * 3 + 24, h + 24), B.VOID)
    d = ImageDraw.Draw(one)
    for i, (t, im) in enumerate((("Now (PR7-43 as built)", before), ("1. Wall niche over the head (rec.)", a),
                                 ("2. No arch: timber headboard", b))):
        d.text((i * (w + 12) + 6, 4), t, font=B.SMALL, fill=B.GOLD)
        one.paste(im, (i * (w + 12), 24))
    zoom = Image.new("RGB", (w * 2 * 2 + 12, h * 2 + 24), B.VOID)
    dz = ImageDraw.Draw(zoom)
    for i, (t, im) in enumerate((("1. Wall niche, 2x", a), ("2. Timber headboard, 2x", b))):
        dz.text((i * (w * 2 + 12) + 6, 4), t, font=B.SMALL, fill=B.GOLD)
        zoom.paste(im.resize((w * 2, h * 2), Image.NEAREST), (i * (w * 2 + 12), 24))
    sheet = Image.new("RGB", (max(one.width, zoom.width), one.height + zoom.height + 40), B.VOID)
    sheet.paste(one, (0, 0))
    sheet.paste(zoom, (0, one.height + 20))
    B.label(sheet, "Hub bunks take 3: beds on the planks, no screens, sideways Timber Frames bunks clear of the table").save(
        out / "hub_bunks_take3_sheet.png")
    a.save(out / "hub_bunks_take3_niche_720p.png")
    b.save(out / "hub_bunks_take3_headboard_720p.png")


if __name__ == "__main__":
    main()
