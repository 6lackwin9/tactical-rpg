"""Hub bunk room take 5 (Tushar 2026-11-19 17:26): take 4's 1a (arch + peg board) with his notes, 1280x720 at 1:1.

Usage: python3 art_src/style/hub_bunks_take5_mock.py <hub shot.png> <out_dir>
  - arch niche on the built wall, gear on the peg board (his pick); the bed keepsakes are dropped (too small to read);
  - three ways to set a named founder's bunk apart from a recruit's of the same class (the class colour and weapon
    alone read alike):
      P. a pennant: a small cloth banner in the class colour with the class glyph, hung in the arch over the pillow;
         recruits have a bare niche;
      L. a lit niche: a candle on a ledge in the arch and a gold-rimmed brass plate; recruits: no candle, an iron plate;
      Q. a quilt: the founder's blanket gets a gold-trimmed border and a turned-down sheet; recruits: a plain blanket;
  - Act 1 props for the two later bunk spots as collections, not flat planks: west, a half-built bed frame on a
    sawhorse with a saw and a plank offcut; east, crates with rolled pallets on top and a coil of rope.
"""
import sys
from pathlib import Path
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parent))
import battle_layout_mock as B  # noqa: E402
import hub_bunks_take1_mock as H1  # noqa: E402
import hub_bunks_take2_mock as H2  # noqa: E402
import hub_bunks_take3_mock as H3  # noqa: E402
import hub_bunks_take4_mock as H4  # noqa: E402

WALL_Y0, WALL_Y1 = H4.WALL_Y0, H4.WALL_Y1
GOLD = (214, 170, 72)
IRON, IRON_D = (96, 100, 108), (52, 54, 60)
RECRUIT_GEAR = {"warrior": "polearm", "rogue": "knife", "priest": "censer", "ranger": "crossbow", "brawler": "gauntlet",
                "mage": "sprayer"}


def recruit_gear(cls):
    item = RECRUIT_GEAR[cls]
    for root in (H2.ITEMS, H2.KIT_ITEMS):
        f = root / item / "base.png"
        if f.exists():
            return H2.cut_icon(f, 22)
    return None


def pegboard(img, cx, gear):
    d = ImageDraw.Draw(img)
    x0, y0 = cx + 26, WALL_Y0 + 8
    d.rectangle((x0, y0, x0 + 26, y0 + 34), fill=(86, 60, 36), outline=H1.TIMBER_D)
    d.line((x0 + 1, y0 + 1, x0 + 25, y0 + 1), fill=H1.TIMBER)
    for px in (x0 + 7, x0 + 19):
        d.rectangle((px, y0 + 6, px + 1, y0 + 8), fill=(150, 150, 150))
    if gear:
        img.alpha_composite(gear, (x0 + 13 - gear.width // 2, y0 + 6))


def pennant(d, cx, cls):
    top = WALL_Y0 + 16
    d.line((cx - 9, top, cx + 9, top), fill=(150, 150, 150), width=1)
    col = H1.CLASS[cls]
    d.polygon([(cx - 8, top + 1), (cx + 8, top + 1), (cx + 8, top + 16), (cx, top + 12), (cx - 8, top + 16)], fill=col)
    d.line((cx - 8, top + 1, cx - 8, top + 16), fill=tuple(max(0, c - 40) for c in col))
    H1.glyph(d, cls, cx - 4, top + 3, GOLD)


def candle(img, cx):
    y = WALL_Y1 - 30
    glow = Image.new("RGBA", img.size, (0, 0, 0, 0))
    g = ImageDraw.Draw(glow)
    for r, a in ((12, 26), (8, 40), (5, 60)):
        g.ellipse((cx + 14 - r, y - 3 - r, cx + 14 + r, y - 3 + r), fill=(255, 180, 90, a))
    img.alpha_composite(glow)
    d = ImageDraw.Draw(img)
    d.rectangle((cx + 9, y + 6, cx + 19, y + 8), fill=(110, 108, 100))
    d.rectangle((cx + 13, y - 2, cx + 15, y + 6), fill=(230, 220, 190))
    d.ellipse((cx + 12, y - 6, cx + 16, y - 1), fill=(255, 196, 90))


def quilt(bed, cls):
    """A gold-trimmed border across the blanket and a turned-down white sheet."""
    d = ImageDraw.Draw(bed)
    px = bed.load()
    ys = [y for y in range(30, bed.height) for x in range(bed.width) if px[x, y][3] and px[x, y][:3] != (0, 0, 0)]
    d.line((7, 32, bed.width - 8, 32), fill=(226, 218, 200), width=2)
    d.line((7, 35, bed.width - 8, 35), fill=GOLD)
    d.line((7, 62, bed.width - 8, 62), fill=GOLD)
    return bed


def plate(d, cx, name, cls, style):
    if style == "gold":
        H1.plate(d, cx, WALL_Y0 - 9, name, cls, 80)
        w = int(d.textlength(name, font=H1.T12.TINY)) + 18
        d.rectangle((cx - w // 2 - 1, WALL_Y0 - 10, cx - w // 2 + w + 1, WALL_Y0 + 5), outline=GOLD)
    elif style == "iron":
        f = H1.T12.TINY
        w = int(d.textlength(name, font=f)) + 18
        x = cx - w // 2
        d.rectangle((x, WALL_Y0 - 9, x + w, WALL_Y0 + 4), fill=IRON_D, outline=IRON)
        H1.glyph(d, cls, x + 3, WALL_Y0 - 7, H1.CLASS[cls])
        d.text((x + 14, WALL_Y0 - 8), name, font=f, fill=(210, 210, 214))
    else:
        H1.plate(d, cx, WALL_Y0 - 9, name, cls, 80)


def bunk(img, cx, name, cls, state, founder, mark):
    d = ImageDraw.Draw(img)
    H4.niche(d, cx)
    bed = H3.prop("cot_side")
    bed = H3.tint(bed, H1.CLASS[cls], 30)
    if founder and mark == "Q":
        bed = quilt(bed, cls)
    top = WALL_Y1 - 26
    if founder and mark == "P":
        pennant(d, cx, cls)
    if founder and mark == "L":
        candle(img, cx)
    img.alpha_composite(bed, (cx - bed.width // 2, top))
    if state == "benched" and name in H4.FRONT:
        H4.sleeper(img, cx, top, name, bed)
    gear = H4.gear_img(name) if founder else recruit_gear(cls)
    pegboard(img, cx, gear)
    style = ("gold" if founder else "iron") if mark == "L" else "brass"
    plate(ImageDraw.Draw(img), cx, name, cls, style)


def act1_props(img):
    """West: a half-built bed frame on a sawhorse, a saw, an offcut. East: crates with rolled pallets and rope."""
    d = ImageDraw.Draw(img)
    x, y = 360, 606
    T, TD = H1.TIMBER, H1.TIMBER_D
    # the sawhorse
    d.line((x + 52, y + 46, x + 60, y + 22), fill=TD, width=3); d.line((x + 70, y + 46, x + 62, y + 22), fill=TD, width=3)
    d.rectangle((x + 44, y + 20, x + 84, y + 24), fill=T, outline=TD)
    # the half-built frame: two side rails and the head posts, no slats yet, leaning on the sawhorse
    d.rectangle((x + 2, y + 18, x + 8, y + 48), fill=T, outline=TD)              # head post
    d.rectangle((x + 2, y + 18, x + 50, y + 22), fill=(128, 90, 54), outline=TD)  # top rail
    d.rectangle((x + 2, y + 38, x + 74, y + 42), fill=(128, 90, 54), outline=TD)  # side rail on the floor
    d.rectangle((x + 12, y + 22, x + 15, y + 38), fill=T, outline=TD)
    d.rectangle((x + 26, y + 22, x + 29, y + 38), fill=T, outline=TD)
    # the saw on the sawhorse, an offcut and shavings
    d.polygon([(x + 56, y + 14), (x + 82, y + 18), (x + 82, y + 20), (x + 56, y + 19)], fill=(150, 154, 160))
    d.rectangle((x + 50, y + 13, x + 57, y + 19), fill=(96, 62, 34))
    d.rectangle((x + 78, y + 42, x + 92, y + 46), fill=(140, 100, 60), outline=TD)
    for k in range(6):
        d.point((x + 40 + k * 5, y + 48 + k % 2), fill=(170, 130, 80))
    # east: crates with rolled pallets on top and a coil of rope
    crate = Image.open(H1.ROOT / "kits/rpg/assets/props/common/crate_stack/v0.png").convert("RGBA")
    cx0 = 928 - crate.width - 4
    img.alpha_composite(crate, (cx0, 656 - crate.height))
    for k, (px, py) in enumerate(((cx0 - 34, 628), (cx0 - 30, 612), (cx0 - 60, 630))):
        d.ellipse((px, py, px + 30, py + 22), fill=(150, 140, 120), outline=(90, 84, 72))
        d.ellipse((px + 4, py + 4, px + 14, py + 18), outline=(110, 102, 88))
        d.line((px + 18, py, px + 18, py + 22), fill=(120, 60, 40))
    d.ellipse((cx0 - 84, 640, cx0 - 64, 652), outline=(160, 130, 80), width=3)


def room(shot, mark, act):
    img = H3.clean_room(shot)
    H3.planks(img, 352, 552, 400, 658)
    H4.move_stove(img, shot)
    H4.wall(img)
    for x, (name, cls, state, _) in zip(H4.XS, H1.ROSTER[:6]):
        bunk(img, x, name, cls, state, True, mark)
    if act == 1:
        act1_props(img)
    else:
        H4.south_bed(img, 354, 592, H1.ROSTER[6], False)
        H4.south_bed(img, 928 - 96 - 2, 592, H1.ROSTER[7], True)
    return img.crop(H4.CROP).convert("RGB")


def compare(shot, mark):
    """A founder's bunk and a recruit's of the same class side by side, on the built wall, 2x."""
    img = H3.clean_room(shot)
    H4.wall(img)
    pairs = [(392, ("Kaelen", "warrior", "active"), True), (472, ("Recruit", "warrior", "active"), False),
             (712, ("Maren", "priest", "active"), True), (792, ("Recruit", "priest", "active"), False)]
    for x, (n, c, s), f in pairs:
        bunk(img, x, n, c, s, f, mark)
    a = img.crop((352, 352, 512, 472)).convert("RGB")
    b = img.crop((672, 352, 832, 472)).convert("RGB")
    out = Image.new("RGB", (a.width * 2 + 8, a.height), B.VOID)
    out.paste(a, (0, 0)); out.paste(b, (a.width + 8, 0))
    return out.resize((out.width * 2, out.height * 2), Image.NEAREST)


def main():
    shot = Image.open(sys.argv[1]).convert("RGBA")
    out = Path(sys.argv[2])
    names = {"P": "P. Pennant in the arch", "L": "L. Lit niche + gold-rimmed plate", "Q": "Q. Gold-trimmed quilt"}
    cmps = [(names[m], compare(shot, m)) for m in "PLQ"]
    w, h = cmps[0][1].size
    sheet_c = Image.new("RGB", (w, (h + 22) * 3), B.VOID)
    d = ImageDraw.Draw(sheet_c)
    for k, (t, im) in enumerate(cmps):
        d.text((6, k * (h + 22) + 3), t + ": founder, then a recruit of the same class (Kaelen and a warrior, Maren and a "
               "priest), 2x", font=B.SMALL, fill=B.GOLD)
        sheet_c.paste(im, (0, k * (h + 22) + 20))
    B.label(sheet_c, "Hub bunks take 5: setting a founder apart from a recruit").save(out / "hub_bunks_take5_founders.png")
    a1, a2 = room(shot, "P", 1), room(shot, "P", 2)
    rw, rh = a1.size
    rooms = Image.new("RGB", (rw * 2 + 12, rh + 22), B.VOID)
    dr = ImageDraw.Draw(rooms)
    dr.text((6, 3), "Act 1 (P): a half-built bed frame and sawhorse; crates with rolled pallets", font=B.SMALL, fill=B.GOLD)
    dr.text((rw + 18, 3), "Act 2 (P): the two beds come in", font=B.SMALL, fill=B.GOLD)
    rooms.paste(a1, (0, 20)); rooms.paste(a2, (rw + 12, 20))
    B.label(rooms, "Hub bunks take 5: arch + peg board, keepsakes dropped, Act 1 props as collections").save(
        out / "hub_bunks_take5_room.png")
    for m in "PLQ":
        room(shot, m, 2).save(out / f"hub_bunks_take5_{m}_720p.png")
    act1 = a1.crop((40, 240, 660, 324)).resize((1240, 168), Image.NEAREST)
    act1.save(out / "hub_bunks_take5_act1_props_2x.png")


if __name__ == "__main__":
    main()
