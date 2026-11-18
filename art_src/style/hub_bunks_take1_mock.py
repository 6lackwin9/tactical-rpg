"""Hub bunk room take 1 (Tushar 2026-11-18 23:27, 23:28): name plates on the wall, 8 bunks laid out, bunks that belong.

Usage: python3 art_src/style/hub_bunks_take1_mock.py <out_dir>
Drawn over the PR7-23 hub render at 720p (1:1), the hall only.
  A. 8 bunks in a row as now (64 px apart): a plate on the wall band above each arch; names are cramped, long ones
     drop to the first name.
  B. (recommended) the 6 founder bunks on the north wall, 80 px apart (plates up to 72 px); the 2 Timber Frames bunks
     (the Act 2 unlock) stand free on the south side, facing up, with the plate on their timber headboard.
Each named founder's bunk gets a blanket in the class colour and a personal piece on the wall beside the arch (their
weapon or keepsake); a recruit's bunk only gets the class colour and the class glyph on its plate. All drawn markers
here are placeholders: the final props go through the image queue.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import battle_take12_mock as T12  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
PROPS = ROOT / "games/emberwake/assets/props/sunk_ward"
RENDER = Path("/mnt/project-files/pr7-23/hub_region_map_720.png")
X0, Y0, X1, Y1 = 344, 356, 936, 664            # the hall crop
WALL_Y0, ARCH_Y, BED_Y1 = 370, 383, 462
DOOR = (608, 672)
BRASS, BRASS_D = (176, 132, 64), (92, 66, 34)
TIMBER, TIMBER_D = (120, 84, 50), (70, 46, 28)
# class colours (proposal, [PROVISIONAL]): warm and muted to sit in Tallow and Iron
CLASS = {"warrior": (150, 52, 46), "ranger": (84, 118, 62), "rogue": (88, 76, 118), "priest": (176, 150, 96),
         "mage": (52, 118, 122), "brawler": (164, 96, 44)}
ROSTER = [("Kaelen", "warrior", "active", "billhook"), ("Vesper", "rogue", "active", "whistle"),
          ("Maren", "priest", "active", "censer"), ("Hollis", "ranger", "active", "arbalest"),
          ("Dunstan", "brawler", "benched", "chain"), ("Elyse", "mage", "benched", "retort"),
          ("Recruit", "warrior", "active", None), (None, None, "empty", None)]


def prop(state):
    return Image.open(PROPS / f"kiln_bunk_{state}/v0.png").convert("RGBA")


def tint_blanket(im, col):
    """Recolour the grey blanket (lower half of the prop) with the class colour, keeping its shading."""
    px = im.load()
    for y in range(44, im.height):
        for x in range(im.width):
            r, g, b, a = px[x, y]
            if a and abs(r - g) < 14 and abs(g - b) < 18 and 70 < r < 215:
                k = r / 160
                px[x, y] = (min(255, int(col[0] * k)), min(255, int(col[1] * k)), min(255, int(col[2] * k)), a)
    return im


def glyph(d, cls, x, y, c):
    """9 px class glyphs (placeholder; the kit has no class icons yet)."""
    if cls == "warrior":
        d.line((x + 1, y + 8, x + 7, y + 2), fill=c, width=2); d.line((x + 2, y + 4, x + 5, y + 7), fill=c)
    elif cls == "ranger":
        d.arc((x - 3, y, x + 6, y + 9), 270, 90, fill=c, width=2); d.line((x + 1, y, x + 1, y + 9), fill=c)
    elif cls == "rogue":
        d.polygon([(x + 4, y), (x + 6, y + 6), (x + 2, y + 6)], fill=c); d.line((x + 1, y + 7, x + 7, y + 7), fill=c)
    elif cls == "priest":
        d.ellipse((x + 1, y + 2, x + 7, y + 8), outline=c, width=2); d.line((x + 4, y - 1, x + 4, y + 2), fill=c)
    elif cls == "mage":
        d.polygon([(x + 3, y), (x + 5, y), (x + 5, y + 3), (x + 8, y + 8), (x, y + 8), (x + 3, y + 3)], fill=c)
    elif cls == "brawler":
        d.rectangle((x + 1, y + 2, x + 7, y + 8), fill=c); d.line((x + 1, y + 4, x + 7, y + 4), fill=B.BASALT)


def plate(d, cx, y, name, cls, room):
    """A brass name plate: class glyph then the name; the first name only when the full one does not fit."""
    if name is None:
        return
    f = T12.TINY
    nm = name if d.textlength(name, font=f) + 14 <= room - 6 else name.split()[0]
    w = int(d.textlength(nm, font=f)) + 18
    x = cx - w // 2
    d.rectangle((x, y, x + w, y + 13), fill=BRASS_D, outline=BRASS)
    glyph(d, cls, x + 3, y + 2, CLASS[cls] if cls else BRASS)
    d.text((x + 14, y + 1), nm, font=f, fill=(236, 220, 180))


def keepsake(d, kind, x, y):
    """A personal piece hung on the wall beside the arch (placeholder marks; final props via the image queue)."""
    s = (200, 196, 186)
    if kind == "billhook":                       # Kaelen: the notched billhook
        d.line((x + 4, y + 26, x + 4, y), fill=TIMBER, width=2); d.arc((x + 2, y - 2, x + 12, y + 8), 200, 360, fill=s, width=2)
    elif kind == "whistle":                      # Vesper: the runner whistle on a cord, a coil of roof rope
        d.ellipse((x, y + 10, x + 12, y + 22), outline=TIMBER, width=2); d.line((x + 6, y, x + 6, y + 10), fill=s)
        d.rectangle((x + 4, y + 2, x + 9, y + 4), fill=s)
    elif kind == "censer":                       # Maren: the bronze censer on its chain
        d.line((x + 6, y, x + 6, y + 12), fill=s); d.ellipse((x + 1, y + 12, x + 11, y + 22), fill=BRASS, outline=BRASS_D)
    elif kind == "arbalest":                     # Hollis: the arbalest racked on two pegs
        d.line((x, y + 8, x + 12, y + 8), fill=s, width=2); d.arc((x - 2, y + 2, x + 14, y + 16), 180, 360, fill=TIMBER, width=2)
    elif kind == "chain":                        # Dunstan: the broken shackle link
        for k in range(3):
            d.ellipse((x + 3, y + k * 7, x + 9, y + k * 7 + 8), outline=s, width=2)
    elif kind == "retort":                       # Elyse: the cracked retort on a shelf
        d.line((x - 1, y + 20, x + 13, y + 20), fill=TIMBER, width=2); d.ellipse((x + 2, y + 9, x + 11, y + 19), outline=(120, 190, 160), width=2)
        d.line((x + 9, y + 11, x + 14, y + 5), fill=(120, 190, 160), width=2)


def bunk(img, cx, entry, flip=False, top=ARCH_Y):
    name, cls, state, ks = entry
    im = prop(state)
    if cls:
        im = tint_blanket(im, CLASS[cls])
    if flip:                                     # the free-standing Timber Frames bunk: no arch, head to the south
        im = im.crop((0, 36, 80, 80)).transpose(Image.FLIP_TOP_BOTTOM)
        img.alpha_composite(im, (cx - 40, top))
        d = ImageDraw.Draw(img)
        hb = top + im.height - 2
        d.rectangle((cx - 22, hb, cx + 22, hb + 6), fill=TIMBER, outline=TIMBER_D)
        d.rectangle((cx - 24, top + 4, cx - 21, hb + 6), fill=TIMBER_D); d.rectangle((cx + 21, top + 4, cx + 24, hb + 6), fill=TIMBER_D)
        return
    img.alpha_composite(im, (cx - 40, top))


def scene(option):
    base = Image.open(RENDER).convert("RGBA")
    clean = base.crop((790, WALL_Y0, 810, BED_Y1))           # a clean strip of wall between two arches
    for x in range(352, 928, 20):
        if not DOOR[0] <= x < DOOR[1]:
            base.paste(clean, (x, WALL_Y0))
    d = ImageDraw.Draw(base)
    if option == "A":
        xs = [384, 448, 512, 576, 704, 768, 832, 896]
        order = [7, 0, 1, 2, 3, 4, 5, 6]                     # the Timber Frames pair at the two ends, as in v22
        for x, i in zip(xs, order):
            bunk(base, x, ROSTER[i] if i != 7 else ROSTER[6] if x == 384 else ROSTER[7])
        d = ImageDraw.Draw(base)
        for x, i in zip(xs, order):
            e = ROSTER[i] if i != 7 else ROSTER[6] if x == 384 else ROSTER[7]
            plate(d, x, WALL_Y0, e[0], e[1], 64)
    else:
        xs = [392, 472, 552, 712, 792, 872]
        for x, e in zip(xs, ROSTER[:6]):
            bunk(base, x, e)
        d = ImageDraw.Draw(base)
        for x, e in zip(xs, ROSTER[:6]):
            plate(d, x, WALL_Y0, e[0], e[1], 80)
            if e[3]:
                keepsake(d, e[3], x + 24, ARCH_Y + 10)
        for x, e in ((420, ROSTER[6]), (860, ROSTER[7])):
            bunk(base, x, e, flip=True, top=596)
            d = ImageDraw.Draw(base)
            plate(d, x, 640, e[0] or "", e[1], 80) if e[0] else None
    if option == "A":
        d = ImageDraw.Draw(base)
    return base.crop((X0, Y0, X1, Y1)).convert("RGB")


def variants():
    """Named bunks (class colour + personal piece) next to the recruit bunks (class colour + glyph only)."""
    img = Image.new("RGBA", (6 * 88 + 8, 2 * 118 + 40), (20, 23, 31, 255))
    d = ImageDraw.Draw(img)
    d.text((6, 4), "Named founders: class colour + their own piece", font=B.SMALL, fill=B.GOLD)
    d.text((6, 140), "Recruits by class: class colour + glyph only", font=B.SMALL, fill=B.GOLD)
    wall = Image.open(RENDER).convert("RGBA").crop((790, WALL_Y0, 810, BED_Y1))
    for row, entries in enumerate([ROSTER[:6], [(c.title(), c, "active", None) for c in CLASS]]):
        for i, e in enumerate(entries):
            cx = 48 + i * 88
            y = 22 + row * 136
            for k in range(-2, 2):
                img.paste(wall, (cx + k * 20, y))
            bunk(img, cx, e, top=y + 13)
            d = ImageDraw.Draw(img)
            plate(d, cx, y, e[0], e[1], 80)
            if e[3]:
                keepsake(d, e[3], cx + 24, y + 22)
    return img.convert("RGB")


def main():
    out = Path(sys.argv[1])
    a, b, v = scene("A"), scene("B"), variants()
    one = Image.new("RGB", (a.width + b.width + 12, a.height + 24), B.VOID)
    dd = ImageDraw.Draw(one)
    dd.text((6, 4), "A. 8 in a row as now (64 px apart)", font=B.SMALL, fill=B.GOLD)
    dd.text((a.width + 18, 4), "B. 6 on the wall (80 px apart) + 2 Timber Frames bunks on the south side (rec.)",
            font=B.SMALL, fill=B.GOLD)
    one.paste(a, (0, 24)); one.paste(b, (a.width + 12, 24))
    zoom = one.resize((one.width * 2, one.height * 2), Image.NEAREST)
    vz = v.resize((v.width * 2, v.height * 2), Image.NEAREST)
    sheet = Image.new("RGB", (zoom.width, 24 + one.height + 20 + zoom.height + 20 + vz.height + 40), B.VOID)
    d = ImageDraw.Draw(sheet)
    d.text((8, 4), "1:1 at 720p", font=B.SMALL, fill=B.IRON_L)
    sheet.paste(one, (0, 24))
    y = 24 + one.height + 20
    d.text((8, y - 18), "2x zoom", font=B.SMALL, fill=B.IRON_L)
    sheet.paste(zoom, (0, y))
    y += zoom.height + 20
    d.text((8, y - 18), "Bunk looks, 2x (class colours [PROVISIONAL]; drawn pieces are placeholders for the image queue)",
           font=B.SMALL, fill=B.IRON_L)
    sheet.paste(vz, (0, y))
    B.label(sheet, "Hub bunks take 1: name plates on the wall, 8 bunks laid out, a bunk that belongs").save(
        out / "hub_bunks_take1_sheet.png")


if __name__ == "__main__":
    main()
