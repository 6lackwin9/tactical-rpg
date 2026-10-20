# Icon sheets (ART2): the prompts for 4 x 4 icon sheets (one image, sixteen icons) and the slicer that cuts a
# generated sheet back into one icon per cell, reduced to the icon grid with pixelize's colour rules.
#   python sheets.py prompts <out.md>                writes games/emberwake/docs/specs/ART2_ICONS.md
#   python sheets.py slice <sheet.png> <sheet_id> <out_dir> [grid] [size]
# Standard library and Pillow only, so it can be ported to Rust.
import pathlib, sys

TINT = {"attack": "deep dark red", "support": "deep dark green", "defend": "deep dark blue", "move": "deep dark teal",
        "focus": "deep dark gold-brown", "status": "deep dark purple", "rest": "dark slate grey", "item": "dark slate grey"}

STYLE = ("A sheet of 16 pixel-art icons for a dark fantasy tactical RPG, in the style of detailed 16-bit SNES/GBA-era RPG "
         "icons. Square 1:1 image, an exact 4 by 4 grid of 16 equal square tiles separated by thin straight gutters of pure "
         "black (#000000), no outer border beyond the gutters. Each tile is one icon on its own rounded-square tile filled "
         "with the colour named for it, with a subtle darker vignette. Every icon is drawn large and centred, with clear dark "
         "outlines, hand-placed pixel clusters, a limited palette with 4 shades per colour, a warm light from the upper left "
         "and a thin warm rim light. The icons {what}, each showing the action itself so it reads at a glance at small size. "
         "No text, no letters, no numbers, no labels, no frame, no watermark. Row by row, left to right:")

SHEETS = {
 "skills_1": ("show basic actions and warrior skills", [
  ("strike", "attack", "a sword mid-slash with a bright arc"), ("offhand_jab", "attack", "a short off-hand jab with a small blade"),
  ("reckless_swing", "attack", "a wild two-handed overhead swing, motion lines"), ("shockwave", "attack", "a shock wave rippling forward along the ground in a line"),
  ("guard", "defend", "a raised forearm guard with a faint shimmer"), ("move", "move", "a boot stepping forward with dust and a direction arrow"),
  ("switch_target", "focus", "a crosshair jumping from one target to another"), ("rest", "rest", "a resting figure's breath, a soft swirl and a small campfire glow"),
  ("shield_bash", "attack", "a kite shield slamming forward with an impact burst"), ("sunder", "attack", "an axe chopping into cracked armour plate"),
  ("taunt_shout", "support", "a shouting helmeted face with sound waves"), ("riposte", "defend", "a sword parrying a blade and turning it back, a spark"),
  ("shield_wall", "defend", "a big kite shield planted in front with a glowing barrier"), ("rend_armor", "attack", "claw-like blade tearing a breastplate open"),
  ("intimidate", "support", "a roaring face with a menacing red aura"), ("focus", "focus", "a narrowed eye with a gold glint and concentration lines")]),
 "skills_2": ("show ranger and brawler skills", [
  ("volley", "attack", "a rain of several arrows arcing down onto an area"), ("aimed_shot", "attack", "a drawn bow with a single arrow and a sighting line"),
  ("pinning_shot", "attack", "an arrow pinning a foot to the ground"), ("barbed_arrow", "attack", "a barbed arrowhead dripping blood"),
  ("deadeye", "attack", "an arrow striking the centre of a target"), ("crippling_shot", "attack", "an arrow striking a knee with a slowing chain"),
  ("leap_back", "move", "a figure leaping backwards away with motion lines"), ("hamstring", "attack", "a low cut to the back of a leg, the leg buckling"),
  ("haymaker", "attack", "a wide hooking punch landing with a big impact star"), ("body_blow", "attack", "a fist driving into a stomach, a shock ring"),
  ("lunge", "attack", "a figure dashing forward low with a trail, fist leading"), ("sweep_kick", "attack", "a low spinning leg sweep with a dust arc"),
  ("charge_in", "attack", "a shoulder charge bursting forward with speed lines"), ("cyclone_kick", "attack", "a spinning kick inside a whirlwind ring"),
  ("crushing_blow", "attack", "a fist smashing down cracking the ground"), ("staggered", "status", "a reeling head with stars and a question mark")]),
 "skills_3": ("show rogue and priest skills", [
  ("quick_stab", "attack", "a fast dagger thrust with a short speed line"), ("bleeding_cut", "attack", "a dagger slash leaving drops of blood"),
  ("backflip", "move", "a figure in a backflip, a curved arrow"), ("pounce", "attack", "a hooded figure pouncing forward, daggers out"),
  ("lacerate", "attack", "two crossing dagger slashes with blood"), ("assassinate", "attack", "a dagger striking from shadow at a neck, a red glint"),
  ("envenom", "attack", "a dagger blade coated in dripping green poison"), ("feint", "attack", "a dagger with a ghostly decoy copy beside it"),
  ("opportunist", "attack", "a dagger striking an exposed back, a gold glint"), ("mend", "support", "two hands glowing with soft healing light"),
  ("rally", "support", "a raised banner with upward speed arrows"), ("purify", "support", "a white flame cleansing dark smoke"),
  ("smite", "attack", "a beam of holy gold light striking down"), ("judgement", "attack", "a glowing gold mace head coming down with a halo"),
  ("greater_mend", "support", "a large glowing healing cross with radiant rays"), ("renewal", "support", "a sprouting leaf in healing light with a small up arrow")]),
 "skills_4": ("show mage and priest skills and the fallback icons per kind", [
  ("arcane_bolt", "attack", "a blue bolt of raw arcane force flying forward"), ("firebomb", "attack", "a thrown clay pot bursting into fire"),
  ("hex", "attack", "a purple cursed sigil on a cracked mask"), ("arc_lance", "attack", "a straight lance of crackling lightning"),
  ("inferno", "attack", "a roaring burst of flames over an area"), ("glacial_burst", "attack", "a burst of ice shards freezing the ground"),
  ("aegis", "support", "a glowing gold shield sigil over a figure"), ("rejuvenate", "support", "a green healing swirl with small leaves"),
  ("stunned", "status", "a dazed head with a spiral above it"), ("base_attack", "attack", "a plain sword and axe crossed (any attack)"),
  ("base_attack_ranged", "attack", "a plain bow with a nocked arrow (any ranged attack)"), ("base_attack_magic", "attack", "a plain glowing wand tip with sparks (any spell)"),
  ("base_support", "support", "an open hand with a soft green glow (any support)"), ("base_defend", "defend", "a plain round shield (any defence)"),
  ("base_disengage", "move", "a figure stepping back away from a blade (any disengage)"), ("base_status", "status", "a swirl of purple status motes (any status)")]),
 "items_1": ("show weapons and item family icons, each a single object on its tile", [
  ("hatchet", "item", "a one-handed hatchet with a single-bitted head"), ("battle_axe", "item", "a two-handed battle axe with a double-sided crescent head on a long haft"),
  ("dagger", "item", "a short dagger with a narrow straight blade and a small cross guard, clearly smaller than a sword"), ("shortsword", "item", "a short sword, a broad leaf blade twice the dagger's length with a proper cross guard"),
  ("hammer", "item", "a two-handed war hammer, the square head centred and aligned on the haft"), ("titan_warhammer", "item", "a colossal ornate warhammer with a huge runed head, aligned on its haft"),
  ("staff", "item", "a long wooden staff taller than the tile with a carved knot at the top, no gem"), ("wand", "item", "a short slender wand with a small glowing tip, clearly tiny next to a staff"),
  ("mace", "item", "a one-handed flanged mace"), ("orb", "item", "a glass focus orb on a small stand, glowing softly"),
  ("crossbow", "item", "a wooden crossbow with a loaded bolt"), ("cloth", "item", "a folded cloth robe"),
  ("material", "item", "a pile of crafting materials: an ingot, leather and a gem"), ("quest", "item", "a sealed letter with a wax seal and a ribbon"),
  ("misc", "item", "a small travel pouch tied with string"), ("fist", "item", "cloth hand wraps wound round a fist")]),
}


def prompt(sheet_id):
    what, cells = SHEETS[sheet_id]
    rows = []
    for r in range(4):
        row = cells[r * 4:(r + 1) * 4]
        rows.append(f"Row {r + 1}: " + "; ".join(f"{d} (tile {TINT[k]})" for _, k, d in row) + ".")
    return STYLE.format(what=what) + "\n" + "\n".join(rows)


def write_md(path):
    out = ["# ART2: icon sheets", "", "Status: approved for the API (user 2026-10-20: \"go ahead with icons generation\"). One image per sheet, sixteen "
           "icons each; `tools/portraits/sheets.py slice` cuts a sheet into icons. Tile tints by kind: " +
           ", ".join(f"{k} {v}" for k, v in TINT.items()) + ".", ""]
    for sid, (what, cells) in SHEETS.items():
        out += [f"## Sheet {sid} (`{sid}`)", "", "Icons, row by row: " + ", ".join(f"`{c[0]}`" for c in cells) + ".", "", "```text", prompt(sid), "```", ""]
    pathlib.Path(path).write_text("\n".join(out).rstrip() + "\n", encoding="utf-8", newline="\n")


def cells_of(im):
    """The 16 tile boxes of a sheet: gutters are the rows and columns that are almost all near black."""
    from PIL import Image
    g = im.convert("L"); w, h = g.size; px = g.load()
    dark_col = [sum(px[x, y] < 10 for y in range(0, h, 2)) > h * 0.30 for x in range(w)]
    dark_row = [sum(px[x, y] < 10 for x in range(0, w, 2)) > w * 0.30 for y in range(h)]
    def runs(flags):
        out, start = [], None
        for i, f in enumerate(flags + [True]):
            if not f and start is None:
                start = i
            elif f and start is not None:
                if i - start > len(flags) // 12:
                    out.append((start, i))
                start = None
        return out
    xs, ys = runs(dark_col), runs(dark_row)
    if len(xs) != 4 or len(ys) != 4:                       # gutters not found: fall back to equal cells
        xs = [(w * i // 4, w * (i + 1) // 4) for i in range(4)]; ys = [(h * i // 4, h * (i + 1) // 4) for i in range(4)]
    return [(x0, y0, x1, y1) for (y0, y1) in ys for (x0, x1) in xs]


def slice_sheet(path, sheet_id, out_dir, grid=32, size=64):
    from PIL import Image
    import pixelize as X
    im = X.clear_corner(Image.open(path).convert("RGBA"))
    out = pathlib.Path(out_dir); out.mkdir(parents=True, exist_ok=True)
    names = [c[0] for c in SHEETS[sheet_id][1]]
    saved = []
    for name, box in zip(names, cells_of(im)):
        tile = im.crop(box)
        small = X.mode_reduce(tile, grid)
        small.resize((size, size), Image.NEAREST).save(out / f"{name}.png")
        saved.append(name)
    return saved


if __name__ == "__main__":
    if sys.argv[1] == "prompts":
        write_md(sys.argv[2]); print(len(SHEETS), "sheets")
    elif sys.argv[1] == "slice":
        grid = int(sys.argv[5]) if len(sys.argv) > 5 else 32; size = int(sys.argv[6]) if len(sys.argv) > 6 else 64
        print(slice_sheet(sys.argv[2], sys.argv[3], sys.argv[4], grid, size))
