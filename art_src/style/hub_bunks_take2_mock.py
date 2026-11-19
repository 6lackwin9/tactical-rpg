"""Hub bunk room take 2 (Tushar 2026-11-18 23:37): layout B, real item props, a sleeper on the bed, one floor.

Usage: python3 art_src/style/hub_bunks_take2_mock.py <out_dir>
Layout B (picked 23:37): 6 founder bunks on the north wall 80 px apart, the 2 Timber Frames bunks free-standing on
the south side. Changes from take 1:
  - the south bunks use the whole free-standing cot (`cot_side`) turned head to the wall, not a cut kiln bunk;
  - each founder's corner holds their real gear as a prop (weapon on wall pegs) and their keepsake on the bed, cut from
    the item icons as stand-ins (final props via the image queue);
  - a member who is in the hub but benched sleeps in the bunk (stand-in: the unit sprite's head on the pillow and the
    blanket pulled up; the resting pose waits for Flow, AI-17);
  - the bunks share the hall's plank floor, with low timber screens between them and a step down into the hall (one
    room, each bunk a nook), the alternative: a stone floor strip with the hall kept separate.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import hub_bunks_take1_mock as H1  # noqa: E402

ROOT = H1.ROOT
ITEMS = ROOT / "games/emberwake/assets/items"
KIT_ITEMS = ROOT / "kits/rpg/assets/items"
UNITS = ROOT / "games/emberwake/assets/units"
PROPS = H1.PROPS
WALL_Y0, ARCH_Y, BED_Y1, DOOR = H1.WALL_Y0, H1.ARCH_Y, H1.BED_Y1, H1.DOOR
FLOOR_Y = 422                                   # where the plank floor starts under the bunks (option 1)
GEAR = {"Kaelen": ITEMS / "polearm/base.png", "Vesper": ITEMS / "knife/base.png", "Maren": ITEMS / "censer/base.png",
        "Hollis": KIT_ITEMS / "crossbow/base.png", "Dunstan": ITEMS / "gauntlet/riveted_gauntlets.png",
        "Elyse": ITEMS / "sprayer/base.png"}
SPRITE = {"Dunstan": UNITS / "brawler/gauntlet/sheet.png", "Elyse": UNITS / "mage/1h_sprayer/sheet.png"}
ROSTER = H1.ROSTER


def cut_icon(path, px):
    """An item icon without its slate tile: key out the tile colours, keep the drawing."""
    im = Image.open(path).convert("RGBA").crop((6, 6, 90, 90))
    p = im.load()
    bg = [p[2, 2], p[im.width - 3, im.height - 3], p[im.width // 2, 2]]
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = p[x, y]
            if min(abs(r - c[0]) + abs(g - c[1]) + abs(b - c[2]) for c in bg) < 46 or x < 3 or y < 3 \
                    or x > im.width - 4 or y > im.height - 4:
                p[x, y] = (0, 0, 0, 0)
    bb = im.getbbox()
    im = im.crop(bb) if bb else im
    im.thumbnail((px, px), Image.LANCZOS)
    return im


def keepsake_icon(name, px):
    return cut_icon(ITEMS / f"accessory/keepsake_{name.lower()}.png", px)


def sleeper(img, cx, top, name):
    """Stand-in for the resting pose: the unit sprite's head on the pillow, the blanket pulled up to the chin."""
    sheet = Image.open(SPRITE[name]).convert("RGBA")
    head = sheet.crop((12, 4, 36, 22))
    img.alpha_composite(head, (cx - 12, top + 40))


def floor_and_screens(base, xs):
    """Option 1: the hall's planks run under the bunks; low timber screens part the nooks; a timber step edge."""
    plank = base.crop((606, 604, 700, 644))           # a clean run of hall planks
    for x in range(352, 928, plank.width):
        if not DOOR[0] <= x < DOOR[1]:
            base.paste(plank.crop((0, 0, min(plank.width, 928 - x), BED_Y1 - FLOOR_Y)), (x, FLOOR_Y))
    d = ImageDraw.Draw(base)
    for x0, x1 in ((352, DOOR[0]), (DOOR[1], 928)):
        d.rectangle((x0, BED_Y1 - 2, x1, BED_Y1 + 1), fill=H1.TIMBER_D)          # the step into the hall
    for a, b in zip(xs, xs[1:]):
        if a < DOOR[0] < b:
            continue
        sx = (a + b) // 2 + 10
        d.rectangle((sx, FLOOR_Y - 6, sx + 3, BED_Y1 - 3), fill=H1.TIMBER, outline=H1.TIMBER_D)   # the screen
        d.rectangle((sx - 1, FLOOR_Y - 8, sx + 4, FLOOR_Y - 5), fill=H1.TIMBER_D)


def south_bunk(img, cx, entry):
    """A free-standing Timber Frames bunk: the whole cot turned with its head to the south wall."""
    name, cls, state, _ = entry
    cot = Image.open(PROPS / "cot_side/v0.png").convert("RGBA").transpose(Image.FLIP_TOP_BOTTOM)
    if cls:
        cot = H1.tint_blanket(cot, H1.CLASS[cls]) if state != "empty" else cot
    top = 655 - cot.height
    img.alpha_composite(cot, (cx - cot.width // 2, top))
    d = ImageDraw.Draw(img)
    if name:
        H1.plate(d, cx, 655 - 16, name, cls, 80)


def scene(floor):
    base = Image.open(H1.RENDER).convert("RGBA")
    clean = base.crop((790, WALL_Y0, 810, BED_Y1))
    for x in range(352, 928, 20):
        if not DOOR[0] <= x < DOOR[1]:
            base.paste(clean, (x, WALL_Y0))
    xs = [392, 472, 552, 712, 792, 872]
    if floor:
        floor_and_screens(base, xs)
    for x, e in zip(xs, ROSTER[:6]):
        H1.bunk(base, x, e)
        name = e[0]
        gear = cut_icon(GEAR[name], 22)
        base.alpha_composite(gear, (x + 22, ARCH_Y + 8))                        # their gear on wall pegs
        d = ImageDraw.Draw(base)
        d.line((x + 22, ARCH_Y + 7, x + 22 + gear.width, ARCH_Y + 7), fill=H1.TIMBER_D, width=2)
        if e[2] == "benched":
            sleeper(base, x, ARCH_Y, name)
        ks = keepsake_icon(name, 12)
        base.alpha_composite(ks, (x + 6, ARCH_Y + 58))                            # the keepsake on the bed foot
        H1.plate(ImageDraw.Draw(base), x, WALL_Y0, name, e[1], 80)
    south_bunk(base, 440, ROSTER[6])
    south_bunk(base, 800, ROSTER[7])
    return base.crop((H1.X0, H1.Y0, H1.X1, H1.Y1)).convert("RGB")


def main():
    out = Path(sys.argv[1])
    a, b = scene(True), scene(False)
    one = Image.new("RGB", (a.width + b.width + 12, a.height + 24), B.VOID)
    dd = ImageDraw.Draw(one)
    dd.text((6, 4), "1. One floor: planks under the bunks, timber screens, a step (rec.)", font=B.SMALL, fill=B.GOLD)
    dd.text((a.width + 18, 4), "2. As now: bunks on the stone band, the hall apart", font=B.SMALL, fill=B.GOLD)
    one.paste(a, (0, 24)); one.paste(b, (a.width + 12, 24))
    zoom = one.resize((one.width * 2, one.height * 2), Image.NEAREST)
    sheet = Image.new("RGB", (zoom.width, 24 + one.height + 20 + zoom.height + 30), B.VOID)
    d = ImageDraw.Draw(sheet)
    d.text((8, 4), "1:1 at 720p (Dunstan and Elyse benched: asleep in their bunks; gear and keepsakes are stand-ins cut from"
           " the item icons)", font=B.SMALL, fill=B.IRON_L)
    sheet.paste(one, (0, 24))
    y = 24 + one.height + 20
    d.text((8, y - 18), "2x zoom", font=B.SMALL, fill=B.IRON_L)
    sheet.paste(zoom, (0, y))
    B.label(sheet, "Hub bunks take 2: layout B, their own gear and keepsakes, a sleeper, one floor").save(
        out / "hub_bunks_take2_sheet.png")


if __name__ == "__main__":
    main()
