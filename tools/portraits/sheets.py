# Icon sheets (ART2): the prompts for 4 x 4 icon sheets (one image, sixteen icons) and the slicer that cuts a
# generated sheet back into one icon per cell, reduced to the icon grid with pixelize's colour rules.
# UI sheets (PLAYUI1-I): 5 x 5 sheets of interface icons on a flat magenta key, cut to transparent 32 x 32 icons of at
# most 16 colours and written straight to their layer paths (engine/assets/ui/, kits/rpg/assets/ui/...).
#   python sheets.py prompts <out.md>                writes games/emberwake/docs/specs/ART2_ICONS.md
#   python sheets.py slice <sheet.png> <sheet_id> <out_dir> [grid] [size]
#   python sheets.py ui-prompts <out.md>             writes games/emberwake/docs/specs/PLAYUI1_ICONS.md
#   python sheets.py ui <sheet.png> <sheet_id> [--root R] [--spares DIR] [--contact out.png] [--palette quantize|ramps]
#       cuts a UI sheet: icons with a layer path go under R (default the repo root), spares to DIR (default a temp
#       folder); --contact also writes a 4x contact sheet of every cell on a dark and a light background
# Prop sheets (PROPS1-T): 4 x 4, 2 x 2 and 3 x 3 sheets of map props on the same magenta key, each cell cut to its own
# PNG size (spec PROPS1.md 6.2, 32 px per tile, not square) and written to props/<set>/<art>/v<n>.png in its layer.
#   python sheets.py props-prompts <out.md>          writes games/emberwake/docs/specs/PROPS1_PROMPTS.md
#   python sheets.py props <sheet.png> <sheet_id> [--root R] [--spares DIR] [--contact out.png] [--palette quantize|ramps]
#       (also `slice-props <sheet.png> <sheet_id> [R]`); prints each prop's path and how much of its box it fills
# Expression sheets (PORTRAITFX1-XT, spec PORTRAITFX1.md 7): a 2x2 sheet from generate.py --faces (top-left neutral,
# top-right hurt, bottom-left angry, bottom-right afraid) cut into the four 240 px frames of a unique portrait.
#   python sheets.py faces <sheet.png> <id> <tier> [--base] [--crop l,t,r,b] [--contact out.png] [--faction F] [--root R]
#       tier hero|npc|elite|boss (or the folder name); clears the image model's mark on the whole sheet, cuts four equal
#       quadrants, runs each through pixelize with the same crop (l,t,r,b in pixels inside a quadrant; default the
#       whole quadrant) and writes R/<folder>/faces/<id>/<expression>.png (R default games/emberwake/assets/portraits);
#       --base also writes the neutral frame as R/<folder>/<id>.png; --contact writes the four frames beside the
#       portrait R/<folder>/<id>.png as it was before the run; the backdrop is --faction, else the `faction` of the
#       sheet's record, else of art_src/<id>.json, else hearth. face.py writes the frames' sidecars afterwards.
#       --reduce scale (default pixelize): for art that is already pixel art (the Gemini route) each cell is only cut
#       from its background, scaled to the 120 grid and framed, with no palette pass; --align (scale only) scales
#       (0.85 .. 1.15) and shifts each other cell onto the neutral cell's figure and prints the scale and offset;
#       --cell name=image (repeatable, scale only) replaces one cell by a 1:1 edit of the neutral cell, resized to the
#       cell, before the alignment.
# Standard library and Pillow only, so it can be ported to Rust.
import pathlib, sys, tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]

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


KEY = (255, 0, 255)  # the flat key colour of UI sheet tiles (pure magenta), cut to transparent by the slicer

UI_STYLE = ("A sheet of 25 pixel-art icons for the interface of a dark fantasy tactical RPG, in the style of detailed "
            "16-bit SNES/GBA-era RPG icons. Square 1:1 image, an exact 5 by 5 grid of 25 equal square tiles separated by "
            "thin straight gutters of pure black (#000000), no outer border beyond the gutters. Every tile is filled edge to "
            "edge with flat, solid pure magenta (#FF00FF): square corners, no gradient, no vignette, no texture, no shadow "
            "on it, so the background can be keyed out. Each icon is one single object drawn large and centred on its tile, "
            "filling about three quarters of it, with clear dark outlines, hand-placed pixel clusters, a limited palette "
            "with 4 shades per colour, a warm light from the upper left and a thin warm rim light; no magenta, pink or "
            "purple anywhere in the icons. The icons {what}, each a simple bold silhouette that reads at a glance at 16 "
            "pixels. No text, no letters, no numbers (the question mark is the only glyph), no labels, no frame, no watermark. "
            "Row by row, left to right:")

# UI sheets: id -> (what the icons show, [(name, layer path or None for a spare, description)])
UI_SHEETS = {
 "ui_1": ("are interface glyphs, HUD buttons, map markers and NPC service badges", [
  ("lock", "engine/assets/ui/lock.png", "a closed iron padlock"),
  ("unlock", "engine/assets/ui/unlock.png", "an open iron padlock, the shackle swung up and open"),
  ("delete", "engine/assets/ui/delete.png", "a metal trash bin with a lid"),
  ("help", "engine/assets/ui/help.png", "a bold question mark inside a round ring"),
  ("chevron", "engine/assets/ui/chevron.png", "a thick chevron arrow pointing down"),
  ("gold", "kits/rpg/assets/ui/hud/gold.png", "a stack of gold coins"),
  ("supplies", "kits/rpg/assets/ui/hud/supplies.png", "a tied supply sack with a loaf of bread and a coil of rope"),
  ("party", "kits/rpg/assets/ui/hud/party.png", "three helmeted heads side by side, the middle one in front"),
  ("journal", "kits/rpg/assets/ui/hud/journal.png", "a closed leather journal with a quill"),
  ("stash", "kits/rpg/assets/ui/hud/stash.png", "a wooden chest with iron bands, seen from the front"),
  ("menu", "kits/rpg/assets/ui/hud/menu.png", "three thick horizontal bars stacked, light stone colour"),
  ("npc", "kits/rpg/assets/ui/markers/npc.png", "the bust of a hooded figure, face in shadow"),
  ("chest", "kits/rpg/assets/ui/markers/chest.png", "a closed treasure chest with a gold lock plate"),
  ("chest_open", "kits/rpg/assets/ui/markers/chest_open.png", "an open treasure chest with a gold glint inside"),
  ("lore", "kits/rpg/assets/ui/markers/lore.png", "an open book with a glowing rune on its pages"),
  ("exit", "kits/rpg/assets/ui/markers/exit.png", "a stone doorway with a bold arrow pointing out through it"),
  ("world_map", "kits/rpg/assets/ui/markers/world_map.png", "a rolled parchment world map, partly unrolled"),
  ("encounter", "kits/rpg/assets/ui/markers/encounter.png", "two crossed swords"),
  ("roamer", "kits/rpg/assets/ui/markers/roamer.png", "a snarling grey wolf head in profile, prowling"),
  ("point", "kits/rpg/assets/ui/markers/point.png", "a glowing golden waypoint pin"),
  ("shop", "kits/rpg/assets/ui/services/shop.png", "brass merchant scales over a gold coin"),
  ("recruit", "kits/rpg/assets/ui/services/recruit.png", "a gauntleted hand clasping a sword hilt"),
  ("spare_lock_gold", None, "a closed padlock with a gold rim"),
  ("spare_coin", None, "a single large gold coin"),
  ("spare_scroll", None, "a rolled parchment scroll tied with a red ribbon")]),
}


def ui_prompt(sheet_id):
    what, cells = UI_SHEETS[sheet_id]
    rows = [f"Row {r + 1}: " + "; ".join(d for _, _, d in cells[r * 5:(r + 1) * 5]) + "." for r in range(5)]
    return UI_STYLE.format(what=what) + "\n" + "\n".join(rows)


def write_ui_md(path):
    out = ["# PLAYUI1: UI icon sheet", "",
           "Status: approved for one API image by the user (2026-10-22: \"Generate via API (1 image)\"). Task PLAYUI1-I, "
           "spec `PLAYUI1.md` section 6: 22 icons (32 x 32 PNG, transparent, at most 16 colours, readable at 12 to 16 "
           "px) plus 3 spare variants, as one 5 x 5 sheet on a flat magenta key. Generate with "
           "`tools/portraits/generate.py run <model> ui_1 --file games/emberwake/docs/specs/PLAYUI1_ICONS.md --no-ref`; "
           "cut with `tools/portraits/sheets.py ui art_src/ui_1.png ui_1`, which writes each icon to its layer path.", ""]
    for sid, (what, cells) in UI_SHEETS.items():
        out += [f"## Sheet {sid} (`{sid}`)", "", "Icons, row by row (cell: layer path):", ""]
        out += [f"{i + 1}. `{n}`: " + (f"`{p}`" if p else "spare (scratch folder)") for i, (n, p, _) in enumerate(cells)]
        out += ["", "```text", ui_prompt(sid), "```", ""]
    pathlib.Path(path).write_text("\n".join(out).rstrip() + "\n", encoding="utf-8", newline="\n")


def cells_of(im, n=4):
    """The n x n tile boxes of a sheet: gutters are the rows and columns that are almost all near black."""
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
    if len(xs) != n or len(ys) != n:                       # gutters not found: fall back to equal cells
        xs = [(w * i // n, w * (i + 1) // n) for i in range(n)]; ys = [(h * i // n, h * (i + 1) // n) for i in range(n)]
    return [(x0, y0, x1, y1) for (y0, y1) in ys for (x0, x1) in xs]


def is_key(r, g, b):
    """The magenta key and everything the model blends from it (dark fringes along outlines, pale pink glows): a
    magenta-to-pink hue (285 to 335 degrees) with some saturation. The prompt keeps magenta, pink and purple out of the
    icons themselves."""
    hi, lo = max(r, g, b), min(r, g, b)
    if hi < 40 or hi - lo < 0.3 * hi:
        return False
    if hi == r:
        hue = 60 * ((g - b) / (hi - lo)) % 360
    elif hi == g:
        hue = 60 * ((b - r) / (hi - lo) + 2)
    else:
        hue = 60 * ((r - g) / (hi - lo) + 4)
    return 285 <= hue <= 335


def key_out(tile, inset=0.02):
    """The tile without its gutter edge and with the magenta key (and its darker, anti-aliased fringe) made clear,
    then a glow the model blended into the key (light pink to peach, touching the cleared key) cleared too; padded to
    a square."""
    from PIL import Image
    w, h = tile.size; dx, dy = int(w * inset), int(h * inset)
    tile = tile.crop((dx, dy, w - dx, h - dy)).convert("RGBA")
    px = tile.load(); tw, th = tile.size
    grow = []
    for y in range(th):
        for x in range(tw):
            r, g, b, _ = px[x, y]
            if is_key(r, g, b):
                px[x, y] = (0, 0, 0, 0); grow.append((x, y))
    glow = lambda c: c[3] and c[0] > 220 and c[2] > 120 and c[1] < 200
    while grow:
        x, y = grow.pop()
        for nx, ny in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
            if 0 <= nx < tw and 0 <= ny < th and glow(px[nx, ny]):
                px[nx, ny] = (0, 0, 0, 0); grow.append((nx, ny))
    side = max(tile.size)                                 # square, centred, so the grid does not stretch the icon
    square = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    square.paste(tile, ((side - tile.size[0]) // 2, (side - tile.size[1]) // 2))
    return square


def ui_icon(tile, grid=32, colors=16, outline=True, palette="quantize"):
    """One UI icon: keyed, reduced to `grid` by the most common colour per cell (pixelize.mode_reduce), its colours
    cut to `colors` (the outline colour included) and outlined. palette "quantize": the icon's own most-used colours
    (median cut with k-means); "ramps": pixelize's ramps (4 shades per colour family, as many families as fit)."""
    import pixelize as X
    return snap(X.mode_reduce(key_out(tile), grid), colors, outline, palette)


def snap(small, colors=16, outline=True, palette="quantize"):
    """The colour step of ui_icon on any w x h image: its opaque pixels cut to `colors` (the outline colour included)
    and a dark 1 px outline (portrait.OUT) on the clear pixels touching them; palette as in ui_icon."""
    from PIL import Image
    import pixelize as X, portrait as P
    sp = small.load(); gw, gh = small.size
    mask = [(x, y) for y in range(gh) for x in range(gw) if sp[x, y][3] >= 128]
    if not mask:
        return small
    flat = Image.new("RGB", (len(mask), 1)); fp = flat.load()
    for i, (x, y) in enumerate(mask):
        fp[i, 0] = sp[x, y][:3]
    budget = colors - (1 if outline else 0)
    if palette == "quantize":
        q = flat.quantize(colors=budget, method=Image.Quantize.MEDIANCUT, kmeans=4)
    else:
        q = flat.quantize(colors=40, method=Image.Quantize.MEDIANCUT)
    pal = q.getpalette(); qp = q.load()
    out = Image.new("RGBA", (gw, gh), (0, 0, 0, 0)); op = out.load()
    if palette == "quantize":
        for i, (x, y) in enumerate(mask):
            op[x, y] = tuple(pal[qp[i, 0] * 3:qp[i, 0] * 3 + 3]) + (255,)
    else:
        found = [tuple(pal[i * 3:i * 3 + 3]) for i in sorted({qp[i, 0] for i in range(len(mask))})]
        ramp = []
        for families in range(budget // 4, 0, -1):        # the most families whose ramps fit the budget
            ramp = list(dict.fromkeys(X.ramps(found, families, 4)))
            if len(ramp) <= budget:
                break
        ramp = ramp[:budget]
        for x, y in mask:
            r, g, b = sp[x, y][:3]
            op[x, y] = min(ramp, key=lambda c: (c[0] - r) ** 2 * 3 + (c[1] - g) ** 2 * 4 + (c[2] - b) ** 2 * 2) + (255,)
    if outline:
        ring = [(x, y) for y in range(gh) for x in range(gw) if op[x, y][3] == 0 and
                any(0 <= x + ox < gw and 0 <= y + oy < gh and op[x + ox, y + oy][3] for ox, oy in ((1, 0), (-1, 0), (0, 1), (0, -1)))]
        for x, y in ring:
            op[x, y] = P.OUT + (255,)
    return out


def slice_ui(path, sheet_id, root=ROOT, spares=None, grid=32, palette="quantize"):
    """Cuts a UI sheet: each cell's icon to its layer path under `root`, spares to `spares`; returns (name, path)."""
    from PIL import Image
    import pixelize as X
    im = X.clear_corner(Image.open(path).convert("RGBA"))
    spares = pathlib.Path(spares or pathlib.Path(tempfile.gettempdir()) / f"{sheet_id}_spares")
    cells = UI_SHEETS[sheet_id][1]
    saved = []
    for (name, rel, _), box in zip(cells, cells_of(im, 5)):
        dest = pathlib.Path(root) / rel if rel else spares / f"{name}.png"
        dest.parent.mkdir(parents=True, exist_ok=True)
        ui_icon(im.crop(box), grid, palette=palette).save(dest)
        saved.append((name, dest))
    return saved


def contact(icons, out, scale=4, cols=5):
    """A contact sheet: every icon at `scale` on a dark and on a light panel, and at 1x under each panel."""
    from PIL import Image
    ims = [Image.open(p).convert("RGBA") for p in icons]
    s = ims[0].size[0]; cell = s * scale + 8; rows = (len(ims) + cols - 1) // cols
    small = s + 4; per_line = max(1, (cols * cell) // small); lines = (len(ims) + per_line - 1) // per_line
    pw, ph = cols * cell + 8, rows * cell + 16 + lines * small
    sheet = Image.new("RGB", (pw * 2, ph))
    for k, bg in enumerate(((26, 22, 30), (222, 214, 196))):
        panel = Image.new("RGBA", (pw, ph), bg + (255,))
        for i, im in enumerate(ims):
            panel.alpha_composite(im.resize((s * scale, s * scale), Image.NEAREST), (8 + (i % cols) * cell, 8 + (i // cols) * cell))
            panel.alpha_composite(im, (8 + (i % per_line) * small, rows * cell + 12 + (i // per_line) * small))
        sheet.paste(panel.convert("RGB"), (k * pw, 0))
    sheet.save(out)
    return out


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


# Prop sheets (PROPS1-T, spec PROPS1.md sections 6 and 7): square cells on the magenta key, each cut to its own
# non-square PNG size (32 px of art per tile; a standing prop up to one tile wider than its footprint and up to 4 tiles
# taller, a ground prop exactly its footprint) and written to props/<set>/<art>/v<n>.png in its layer root.
PROP_ROOTS = {"kit": "kits/rpg/assets/props", "game": "games/emberwake/assets/props"}
PROP_TILE = 32
GROUND_OUTLINE = True  # spec 7: every prop gets the dark 1 px outline; False leaves flat decals unoutlined

PROP_STYLE = ("A sheet of {count} pixel-art props for the top-down region maps of a dark fantasy tactical RPG, in the "
              "style of detailed 16-bit SNES/GBA-era RPG map art. Square 1:1 image, an exact {n} by {n} grid of {count} "
              "equal square tiles separated by thin straight gutters of pure black (#000000), no outer border beyond the "
              "gutters. Every tile is filled edge to edge with flat, solid pure magenta (#FF00FF): square corners, no "
              "gradient, no vignette, no texture, no ground, no cast shadow on it, so the background can be keyed out. "
              "Each tile holds one single prop drawn as large as the tile allows in the proportions named for it (width "
              "by height), in the three-quarter top-down view of a tile map (the top and the south face visible), with "
              "clear dark outlines, hand-placed pixel clusters, at most 16 colours per prop with 4 shades per colour, and "
              "light from the upper left; no magenta, pink or purple anywhere in the props. A standing prop stands on the "
              "bottom edge of its tile, its base as wide as the drawing, anything above the base roof, chimney, banner "
              "or canopy; a flat prop is a decal lying on the ground seen from above, filling its proportions, with "
              "ragged edges and nothing rising from it. The props {what}. No text, no letters, no numbers, no runes, no "
              "labels, no frame, no watermark. Row by row, left to right:")

# id -> (run title, what the props are, grid n, [(art, variant, root, set, (w, h) px, layer, description)]);
# root None is a spare take (written to the spares folder, the user picks)
PROP_SHEETS = {
 "props_kit": ("S1 kit standing props", "are generic camp, ruin and rock pieces of weathered dark stone, wood and hide", 4, [
  ("tent", 0, "kit", "common", (96, 144), "sort", "a hide tent on poles with rope pegs, tan hide"),
  ("tent", 1, "kit", "common", (96, 144), "sort", "the same hide tent on poles with rope pegs, grey hide"),
  ("broken_pillar", 0, "kit", "common", (64, 112), "sort", "a broken stone pillar on a square base, rubble at its foot, broken off high"),
  ("broken_pillar", 1, "kit", "common", (64, 112), "sort", "a broken stone pillar on a square base, rubble at its foot, broken off at half height"),
  ("broken_pillar", 2, "kit", "common", (64, 112), "sort", "a broken stone pillar on a square base, rubble at its foot, broken off low to a stump"),
  ("fallen_column", 0, "kit", "common", (96, 48), "sort", "a toppled stone column lying east to west"),
  ("rubble_wall", 0, "kit", "common", (128, 64), "sort", "a low broken wall of fitted stones"),
  ("boulder", 0, "kit", "common", (64, 80), "sort", "a rounded dark rock boulder"),
  ("boulder", 1, "kit", "common", (64, 80), "sort", "a split, angular dark rock boulder"),
  ("rock_spire", 0, "kit", "common", (64, 144), "sort", "a tall jagged dark rock spire"),
  ("cliff_face", 0, "kit", "common", (128, 64), "sort", "a south-facing dark rock cliff face with horizontal strata, its left and right edges cut straight so it tiles left to right"),
  ("cliff_face", 1, "kit", "common", (128, 64), "sort", "a second take of the same cliff face, other cracks, the strata at the same heights so the two tile left to right"),
  ("signpost", 0, "kit", "common", (32, 64), "sort", "a wooden post with two blank arrow boards"),
  ("banner_pole", 0, "kit", "common", (32, 96), "sort", "a tall wooden pole with a plain red cloth banner"),
  ("gatepost", 0, "kit", "common", (32, 80), "sort", "a squat stone gatepost with a cap and a lit torch bracket"),
  ("crate_stack", 0, "kit", "common", (64, 64), "sort", "two wooden crates and a barrel")]),
 "props_buildings": ("S2 Ashlands buildings", "are the four buildings of a Warden keep yard in the Ashlands, a land of ash "
                     "and black basalt: basalt walls, slate roofs, warm ember light, each building filling its tile's width", 2, [
  ("keep_hall", 0, "game", "ashlands", (320, 256), "sort", "a squat basalt keep hall with a slate roof, two short corner towers, red Warden banners with a gold ember sun, a glowing hearth door"),
  ("barracks", 0, "game", "ashlands", (320, 224), "sort", "a long basalt barracks with a timber and slate roof, shuttered windows, weapon racks in front"),
  ("forge", 0, "game", "ashlands", (288, 272), "sort", "a basalt smithy with a tall chimney, an open front with a glowing forge and an anvil"),
  ("storehouse", 0, "game", "ashlands", (288, 256), "sort", "a basalt storehouse with wide double doors, crates and sacks in front")]),
 "props_ashlands": ("S3 Ashlands and ground", "are Ashlands pieces (a goblin war camp, a ruined basalt toll arch, a "
                    "cistern) and flat ground decals of sooty ash country", 3, [
  ("war_tent", 0, "game", "ashlands", (96, 160), "sort", "a goblin warlord's large hide tent on bone poles, red goblin pennants, skulls"),
  ("toll_arch", 0, "game", "ashlands", (64, 192), "sort", "the surviving half of a collapsed basalt toll arch: one tall column, a broken stub of the arch leaning west from its top"),
  ("cistern", 0, "game", "ashlands", (224, 160), "ground", "a rectangular stone rim one unit thick around a dark pool of still water, the water 5 by 3 in the middle"),
  ("fire_pit", 0, "kit", "common", (32, 32), "ground", "a ring of stones around glowing embers"),
  ("cobbles", 0, "kit", "common", (128, 96), "ground", "a patch of sooty cobblestones"),
  ("cobbles", 1, "kit", "common", (128, 96), "ground", "a second patch of sooty cobblestones, other stones and gaps"),
  ("scorch_mark", 0, "kit", "common", (64, 64), "ground", "burnt ground and ash"),
  ("scorch_mark", 1, "kit", "common", (64, 64), "ground", "a second scorch mark of burnt ground and ash, another shape"),
  ("war_tent", None, None, "ashlands", (96, 160), "sort", "a second take of the goblin warlord's war tent: large hide tent, bone poles, red pennants, skulls")]),
}


def prop_name(cell):
    art, variant, root = cell[:3]
    return f"{art}_v{variant}" if root else f"{art}_take2"


def prop_path(cell):
    """The cell's file relative to the repo root (props/<set>/<art>/v<n>.png in its layer), or None for a spare."""
    art, variant, root, pset = cell[:4]
    return f"{PROP_ROOTS[root]}/{pset}/{art}/v{variant}.png" if root else None


def props_prompt(sheet_id):
    title, what, n, cells = PROP_SHEETS[sheet_id]
    def cell_text(c):
        (w, h), layer, desc = c[4], c[5], c[6]
        kind = "standing" if layer == "sort" else "flat"
        return f"{desc} ({kind}, {w / PROP_TILE:g} wide by {h / PROP_TILE:g} {'tall' if layer == 'sort' else 'deep'})"
    rows = [f"Row {r + 1}: " + "; ".join(cell_text(c) for c in cells[r * n:(r + 1) * n]) + "." for r in range(n)]
    return PROP_STYLE.format(count=n * n, n=n, what=what) + "\n" + "\n".join(rows)


def write_props_md(path):
    files = [c for s in PROP_SHEETS.values() for c in s[3] if c[2]]
    kit, game = sum(c[2] == "kit" for c in files), sum(c[2] == "game" for c in files)
    out = ["# PROPS1: prop sheets", "",
           "Status: prompts ready, each sheet needs the user's yes (task PROPS1-A2 asks before every run: the prompt, "
           "the model, one image). Spec `PROPS1.md` sections 6.2 and 7: " + f"{len(files)} files ({kit} kit, {game} game) "
           "and one spare take from three sheets on a flat magenta key. After the user's yes, generate a sheet with "
           "`tools/portraits/generate.py run <model> <sheet_id> --file games/emberwake/docs/specs/PROPS1_PROMPTS.md "
           "--no-ref`; cut it with `tools/portraits/sheets.py props art_src/<sheet_id>.png <sheet_id> --contact "
           "<out.png>`, which keys the magenta out, trims each prop, fits it to its PNG size keeping its aspect "
           "(standing props on the bottom edge, flat props centred), cuts it to at most 16 colours with a dark 1 px "
           "outline and writes it to its layer path.", ""]
    for sid, (title, what, n, cells) in PROP_SHEETS.items():
        out += [f"## Sheet {title} (`{sid}`)", "", f"{n} x {n} cells, row by row (cell: layer path, PNG size, layer):", ""]
        out += [f"{i + 1}. `{c[0]}`" + (f" v{c[1]}: `{prop_path(c)}`" if c[2] else ": spare take (scratch folder)") +
                f", {c[4][0]} x {c[4][1]}, {c[5]}" for i, c in enumerate(cells)]
        out += ["", "```text", props_prompt(sid), "```", ""]
    pathlib.Path(path).write_text("\n".join(out).rstrip() + "\n", encoding="utf-8", newline="\n")


def key_cell(tile, inset=0.02):
    """A prop cell with its background clear: the magenta key as in key_out when the cell's border is mostly key,
    else the border colour flooded from the edges (pixelize.cut_background's rule, for a sheet on a flat dark ground)."""
    from collections import deque
    import pixelize as X
    w, h = tile.size; dx, dy = int(w * inset), int(h * inset)
    inner = tile.crop((dx, dy, w - dx, h - dy)).convert("RGBA"); px = inner.load(); iw, ih = inner.size
    border = [px[x, y] for x in range(0, iw, 4) for y in (0, ih - 1)] + [px[x, y] for y in range(0, ih, 4) for x in (0, iw - 1)]
    if sum(is_key(*c[:3]) for c in border) * 2 >= len(border):
        return clear_edge_dark(key_out(tile, inset))
    opaque = sorted((c for c in border if not max(c[:3]) < 16), key=sum) or border
    bg = opaque[len(opaque) // 2][:3]                      # the ground's colour: the border's median, gutter crumbs left out
    im = clear_edge_dark(inner); px = im.load()
    near = lambda c: c[3] == 0 or sum((a - b) ** 2 for a, b in zip(c[:3], bg)) <= X.BG_TOL ** 2
    seen = bytearray(iw * ih); q = deque([(x, y) for x in range(iw) for y in (0, ih - 1)] + [(x, y) for y in range(ih) for x in (0, iw - 1)])
    while q:                                               # pixelize.cut_background's edge flood, through cleared crumbs too
        x, y = q.popleft()
        if not (0 <= x < iw and 0 <= y < ih) or seen[y * iw + x]:
            continue
        seen[y * iw + x] = 1
        if near(px[x, y]):
            px[x, y] = (0, 0, 0, 0); q.extend(((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)))
    return im


def clear_edge_dark(im, dark=16):
    """Clears what is left of the black gutters: a flood from the edges through opaque near-black pixels only (not
    through cleared ones, so it never reaches the prop's own outline), so a crumb of gutter along a cell edge does not
    count as the prop or as the background colour."""
    from collections import deque
    w, h = im.size; px = im.load(); seen = bytearray(w * h)
    def near(c):
        return c[3] and max(c[:3]) < dark
    q = deque([(x, y) for x in range(w) for y in (0, h - 1)] + [(x, y) for y in range(h) for x in (0, w - 1)])
    while q:
        x, y = q.popleft()
        if not (0 <= x < w and 0 <= y < h) or seen[y * w + x]:
            continue
        seen[y * w + x] = 1
        if not near(px[x, y]):
            continue
        px[x, y] = (0, 0, 0, 0)
        q.extend(((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)))
    return im


def drop_specks(im, frac=0.002):
    """Clears the opaque 4-connected pieces smaller than `frac` of the image (key fringe, gutter crumbs) so they do
    not stretch the trim box."""
    w, h = im.size; px = im.load(); seen = bytearray(w * h); least = max(4, int(w * h * frac))
    for y0 in range(h):
        for x0 in range(w):
            if seen[y0 * w + x0] or px[x0, y0][3] < 128:
                continue
            piece, stack = [], [(x0, y0)]; seen[y0 * w + x0] = 1
            while stack:
                x, y = stack.pop(); piece.append((x, y))
                for nx, ny in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
                    if 0 <= nx < w and 0 <= ny < h and not seen[ny * w + nx] and px[nx, ny][3] >= 128:
                        seen[ny * w + nx] = 1; stack.append((nx, ny))
            if len(piece) < least:
                for x, y in piece:
                    px[x, y] = (0, 0, 0, 0)
    return im


def reduce_to(im, w, h, colors=64):
    """pixelize.mode_reduce to a w x h grid (not only square): each output pixel the most common colour of its block,
    clear when the block is mostly clear; a source smaller than the grid repeats its pixels (nearest neighbour)."""
    from PIL import Image
    sw, sh = im.size
    alpha = im.getchannel("A"); ap = alpha.load()
    q = im.convert("RGB").quantize(colors=colors, method=Image.Quantize.MEDIANCUT)
    pal = q.getpalette(); qp = q.load()
    out = Image.new("RGBA", (w, h), (0, 0, 0, 0)); op = out.load()
    for gy in range(h):
        y0 = gy * sh // h; y1 = max(y0 + 1, (gy + 1) * sh // h); sy = 2 if y1 - y0 >= 4 else 1
        for gx in range(w):
            x0 = gx * sw // w; x1 = max(x0 + 1, (gx + 1) * sw // w); sx = 2 if x1 - x0 >= 4 else 1
            count, clear = {}, 0
            for y in range(y0, y1, sy):
                for x in range(x0, x1, sx):
                    if ap[x, y] < 128:
                        clear += 1
                    else:
                        i = qp[x, y]; count[i] = count.get(i, 0) + 1
            if not count or clear > sum(count.values()):
                continue
            i = max(count, key=count.get)
            op[gx, gy] = tuple(pal[i * 3:i * 3 + 3]) + (255,)
    return out


def prop_image(tile, size, layer, colors=16, palette="quantize"):
    """One prop from its cell: background cleared, specks dropped, trimmed to the content, reduced to fit `size`
    less the 1 px outline margin keeping its aspect, placed bottom-centred (standing, layer "sort") or centred (flat,
    layer "ground") on a clear canvas of exactly `size`, cut to `colors` and outlined. Returns (image, fill): fill is
    the content's (width, height) share of the box it was fitted to."""
    from PIL import Image
    w, h = size
    outline = GROUND_OUTLINE or layer != "ground"
    m = 1 if outline else 0
    keyed = drop_specks(key_cell(tile))
    box = keyed.getchannel("A").point(lambda a: 255 if a >= 128 else 0).getbbox()
    canvas = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    if not box:
        return canvas, (0.0, 0.0)
    content = keyed.crop(box); cw, ch = content.size
    bw, bh = w - 2 * m, h - 2 * m
    k = min(bw / cw, bh / ch)
    tw, th = min(bw, max(1, round(cw * k))), min(bh, max(1, round(ch * k)))
    small = reduce_to(content, tw, th)
    y = h - m - th if layer != "ground" else (h - th) // 2
    canvas.paste(small, ((w - tw) // 2, y))
    return snap(canvas, colors, outline, palette), (tw / bw, th / bh)


def slice_props(path, sheet_id, root=ROOT, spares=None, palette="quantize"):
    """Cuts a prop sheet: each cell to its layer path under `root` at its PNG size, spare takes to `spares`;
    returns (name, path, fill)."""
    from PIL import Image
    import pixelize as X
    title, what, n, cells = PROP_SHEETS[sheet_id]
    im = X.clear_corner(Image.open(path).convert("RGBA"))
    spares = pathlib.Path(spares or pathlib.Path(tempfile.gettempdir()) / f"{sheet_id}_spares")
    saved = []
    for cell, box in zip(cells, cells_of(im, n)):
        rel = prop_path(cell)
        dest = pathlib.Path(root) / rel if rel else spares / f"{prop_name(cell)}.png"
        dest.parent.mkdir(parents=True, exist_ok=True)
        img, fill = prop_image(im.crop(box), cell[4], cell[5], palette=palette)
        img.save(dest)
        saved.append((prop_name(cell), dest, fill))
    return saved


def contact_props(images, out, scale=2, cols=4):
    """A contact sheet for props of mixed sizes: each at `scale`, bottom-aligned in equal cells, on a dark and on a
    light panel side by side."""
    from PIL import Image
    ims = [Image.open(p).convert("RGBA") for p in images]
    cw = max(i.size[0] for i in ims) * scale + 8; ch = max(i.size[1] for i in ims) * scale + 8
    cols = min(cols, len(ims)); rows = (len(ims) + cols - 1) // cols
    pw, ph = cols * cw + 8, rows * ch + 8
    sheet = Image.new("RGB", (pw * 2, ph))
    for k, bg in enumerate(((26, 22, 30), (222, 214, 196))):
        panel = Image.new("RGBA", (pw, ph), bg + (255,))
        for i, im in enumerate(ims):
            big = im.resize((im.size[0] * scale, im.size[1] * scale), Image.NEAREST)
            x = 8 + (i % cols) * cw + (cw - 8 - big.size[0]) // 2; y = 8 + (i // cols) * ch + ch - 8 - big.size[1]
            panel.alpha_composite(big, (x, y))
        sheet.paste(panel.convert("RGB"), (k * pw, 0))
    sheet.save(out)
    return out


FACE_CELLS = ("neutral", "hurt", "angry", "afraid")   # the sheet's quadrants in reading order (PORTRAITFX1 7)
FACE_FOLDERS = {"hero": "heroes", "npc": "npcs", "elite": "elites", "boss": "bosses"}   # unique tiers (art.rs)
PORTRAITS = ROOT / "games/emberwake/assets/portraits"


def face_folder(tier):
    """The portrait folder of a unique tier, given as the tier (`boss`) or its folder (`bosses`)."""
    if tier in FACE_FOLDERS:
        return FACE_FOLDERS[tier]
    if tier in FACE_FOLDERS.values():
        return tier
    raise ValueError(f"unknown tier {tier!r} (one of {', '.join(FACE_FOLDERS)}; common portraits have no frames)")


def parse_crop(text):
    """`l,t,r,b` -> a 4-tuple of ints."""
    parts = [int(v) for v in text.split(",")]
    if len(parts) != 4:
        raise ValueError(f"--crop wants l,t,r,b, got {text!r}")
    return tuple(parts)


def face_boxes(size, crop=None):
    """The four quadrant boxes of a sheet of `size` (equal halves; an odd last pixel row or column is dropped), each
    narrowed by `crop` (l, t, r, b inside a quadrant), in FACE_CELLS order."""
    hw, hh = size[0] // 2, size[1] // 2
    l, t, r, b = crop or (0, 0, hw, hh)
    if not (0 <= l < r <= hw and 0 <= t < b <= hh):
        raise ValueError(f"crop {(l, t, r, b)} is not inside a {hw} x {hh} quadrant")
    return [(qx + l, qy + t, qx + r, qy + b) for qy in (0, hh) for qx in (0, hw)]


def face_faction(sheet, cid):
    """The backdrop faction of a sheet: its record's `faction`, else the character's source record's, else None."""
    import pixelize as X
    return X.faction_of(sheet) or X.faction_of(ROOT / "art_src" / f"{cid}.png")


def clear_sheet(im):
    """The image model's mark cleared on the whole sheet (pixelize's clear_mark, then clear_corner): the mark sits in
    the sheet's bottom-right corner, not in a quadrant's, and pixelize clears nothing in a crop."""
    import pixelize as X
    return X.clear_corner(X.clear_mark(im.convert("RGBA")))


ALIGN_SCALES = [round(0.85 + 0.01 * i, 2) for i in range(31)]   # the scales tried when aligning a cell (0.85 .. 1.15)
ALIGN_WORK = 160    # the longest side of the masks the coarse search compares
ALIGN_REACH = 3     # work-pixel offsets tried around the bounding-box match


def place(im, scale, dx, dy, fill, resample):
    """`im` scaled by `scale` about its centre and shifted by (dx, dy), on a canvas of its own size filled with `fill`
    (a point p lands at (p - c) * scale + c + d). Returns the image and the scale actually applied (a whole pixel size)."""
    from PIL import Image
    w, h = im.size
    nw, nh = max(1, round(w * scale)), max(1, round(h * scale))
    big = im.resize((nw, nh), resample)
    out = Image.new(im.mode, (w, h), fill)
    out.paste(big, (round(w / 2 + dx - nw / 2), round(h / 2 + dy - nh / 2)))
    return out, nw / w


def mask_diff(a, b):
    """The number of pixels in which two 0/255 masks differ."""
    from PIL import ImageChops
    return ImageChops.difference(a, b).histogram()[255]


def hard(mask):
    return mask.point(lambda v: 255 if v >= 128 else 0)


def align_to(cell, neutral):
    """`cell` scaled (0.85 .. 1.15, steps of 0.01) and shifted so that its figure best covers the neutral cell's figure
    (the figure is what pixelize.cut_background leaves); returns (aligned, scale, (dx, dy)) with dx, dy in cell pixels
    (a point p moves to (p - c) * scale + c + d). Deterministic: a coarse search on small masks (every scale, the
    bounding-box offset plus or minus ALIGN_REACH), then the offset refined on full-size masks."""
    import pixelize as X
    from PIL import Image
    bg = cell.getpixel((2, 2))
    def mask(im):
        return hard(X.cut_background(im.copy()).getchannel("A"))
    want, have = mask(neutral), mask(cell)
    wb, hb = want.getbbox(), have.getbbox()
    if wb is None or hb is None:
        return cell, 1.0, (0, 0)
    w, h = cell.size
    f = max(1.0, max(w, h) / ALIGN_WORK)
    size = (max(1, round(w / f)), max(1, round(h / f)))
    want_s, have_s = hard(want.resize(size, Image.BOX)), hard(have.resize(size, Image.BOX))
    wcx, wcy, hcx, hcy = (wb[0] + wb[2]) / 2, (wb[1] + wb[3]) / 2, (hb[0] + hb[2]) / 2, (hb[1] + hb[3]) / 2
    best = None
    for s in ALIGN_SCALES:
        # the offset that puts the scaled bounding-box centre on the neutral one, in work pixels
        ox, oy = (wcx - (hcx - w / 2) * s - w / 2) / f, (wcy - (hcy - h / 2) * s - h / 2) / f
        for ddy in range(-ALIGN_REACH, ALIGN_REACH + 1):
            for ddx in range(-ALIGN_REACH, ALIGN_REACH + 1):
                dx, dy = round(ox) + ddx, round(oy) + ddy
                m, _ = place(have_s, s, dx, dy, 0, Image.BOX)
                key = (mask_diff(hard(m), want_s), abs(s - 1), abs(dx) + abs(dy), s, dx, dy)
                if best is None or key < best:
                    best = key
    s, dx, dy = best[3:]
    fx, fy = round(dx * f), round(dy * f)
    top = None
    reach = int(f) + 1
    for ddy in range(-reach, reach + 1):
        for ddx in range(-reach, reach + 1):
            m, _ = place(have, s, fx + ddx, fy + ddy, 0, Image.BOX)
            key = (mask_diff(hard(m), want), abs(ddx) + abs(ddy), ddx, ddy)
            if top is None or key < top:
                top = key
    fx, fy = fx + top[2], fy + top[3]
    aligned, applied = place(cell, s, fx, fy, bg, Image.LANCZOS)
    return aligned, applied, (fx, fy)


def slice_faces(sheet, cid, tier, base=False, crop=None, contact_out=None, faction=None, root=PORTRAITS,
                reduce="pixelize", align=False, cells=None, report=None):
    """Cuts an expression sheet into its four frames (see the header); returns the paths written, frames first.
    reduce "scale": each cell is cut from its background, scaled to the grid and framed without the palette pass;
    `align` (scale only) lines the other three cells up with the neutral one; `cells` maps a cell name to an image
    that replaces it (resized to the cell, before the alignment); `report(text)` hears each alignment."""
    import pixelize as X
    from PIL import Image
    if reduce not in ("pixelize", "scale"):
        raise ValueError(f"unknown --reduce {reduce!r} (pixelize or scale)")
    cells = dict(cells or {})
    if (align or cells) and reduce != "scale":
        raise ValueError("--align and --cell need --reduce scale")
    for name in cells:
        if name not in FACE_CELLS:
            raise ValueError(f"unknown cell {name!r} (one of {', '.join(FACE_CELLS)})")
    folder = pathlib.Path(root) / face_folder(tier)
    im = Image.open(sheet).convert("RGBA")
    boxes = face_boxes(im.size, crop)
    faction = faction or face_faction(sheet, cid)
    X.faction_key(faction)   # an unknown faction fails before anything is written
    current = folder / f"{cid}.png"
    before = Image.open(current).convert("RGBA") if current.is_file() else None
    im = clear_sheet(im)
    out_dir = folder / "faces" / cid
    out_dir.mkdir(parents=True, exist_ok=True)
    written = []
    frames = {}
    if reduce == "scale":
        raw = {}
        for name, box in zip(FACE_CELLS, boxes):
            cell = im.crop(box)
            if name in cells:
                cell = Image.open(cells[name]).convert("RGBA").resize(cell.size, Image.LANCZOS)
            raw[name] = X.clear_corner(cell)
        for name in FACE_CELLS:
            cell = raw[name]
            if align and name != "neutral":
                cell, scale, (dx, dy) = align_to(cell, raw["neutral"])
                if report:
                    report(f"{name}: scale {scale:.3f} offset {dx},{dy}")
            frames[name] = X.finish(X.smooth_reduce(X.cut_background(cell), X.GRID), faction=faction)
    else:
        with tempfile.TemporaryDirectory(prefix="faces_") as tmp:
            cleared = pathlib.Path(tmp) / "sheet.png"
            im.save(cleared)
            for name, box in zip(FACE_CELLS, boxes):
                frames[name], _ = X.pixelize(cleared, crop=box, mark=False, faction=faction)
    for name in FACE_CELLS:
        dest = out_dir / f"{name}.png"
        frames[name].save(dest); written.append(dest)
    if base:
        frames["neutral"].save(current); written.append(current)
    if contact_out:
        written.append(face_contact([before] + [frames[n] for n in FACE_CELLS], contact_out,
                                    ["current" if before else "no portrait"] + list(FACE_CELLS)))
    return written


def face_contact(images, out, labels, cell=240, gap=8):
    """One row: the portrait as it was (None: an empty cell) and the four frames, each `cell` px, captioned."""
    from PIL import Image, ImageDraw
    cap = 14
    sheet = Image.new("RGB", (gap + len(images) * (cell + gap), gap + cell + cap + gap), (26, 22, 30))
    d = ImageDraw.Draw(sheet)
    for i, (im, label) in enumerate(zip(images, labels)):
        x = gap + i * (cell + gap)
        if im is not None:
            sheet.paste(im.convert("RGB").resize((cell, cell), Image.NEAREST), (x, gap))
        else:
            d.rectangle([x, gap, x + cell - 1, gap + cell - 1], outline=(90, 80, 90))
        d.text((x + 2, gap + cell + 2), label, fill=(222, 214, 196))
    sheet.save(out)
    return pathlib.Path(out)


if __name__ == "__main__":
    if sys.argv[1] == "prompts":
        write_md(sys.argv[2]); print(len(SHEETS), "sheets")
    elif sys.argv[1] == "slice":
        grid = int(sys.argv[5]) if len(sys.argv) > 5 else 32; size = int(sys.argv[6]) if len(sys.argv) > 6 else 64
        print(slice_sheet(sys.argv[2], sys.argv[3], sys.argv[4], grid, size))
    elif sys.argv[1] == "ui-prompts":
        write_ui_md(sys.argv[2]); print(len(UI_SHEETS), "sheets")
    elif sys.argv[1] == "ui":
        args = sys.argv[2:]; opt = {}
        for flag in ("--root", "--spares", "--contact", "--palette"):
            if flag in args:
                i = args.index(flag); opt[flag] = args[i + 1]; del args[i:i + 2]
        saved = slice_ui(args[0], args[1], opt.get("--root", ROOT), opt.get("--spares"), palette=opt.get("--palette", "quantize"))
        for name, dest in saved:
            print(name, dest)
        if "--contact" in opt:
            print("contact", contact([d for _, d in saved], opt["--contact"]))
    elif sys.argv[1] == "props-prompts":
        write_props_md(sys.argv[2]); print(len(PROP_SHEETS), "sheets")
    elif sys.argv[1] == "faces":
        args = sys.argv[2:]; opt = {}
        cells = {}
        while "--cell" in args:
            i = args.index("--cell"); name, _, img = args[i + 1].partition("="); cells[name] = img; del args[i:i + 2]
        for flag in ("--crop", "--contact", "--faction", "--root", "--reduce"):
            if flag in args:
                i = args.index(flag); opt[flag] = args[i + 1]; del args[i:i + 2]
        base = "--base" in args
        if base:
            args.remove("--base")
        align = "--align" in args
        if align:
            args.remove("--align")
        if len(args) != 3:
            sys.exit("usage: sheets.py faces <sheet.png> <id> <tier> [--base] [--crop l,t,r,b] [--contact out.png] "
                     "[--faction F] [--root R] [--reduce pixelize|scale] [--align] [--cell name=image]...")
        try:
            for path in slice_faces(args[0], args[1], args[2], base, parse_crop(opt["--crop"]) if "--crop" in opt else None,
                                    opt.get("--contact"), opt.get("--faction"), opt.get("--root", PORTRAITS),
                                    opt.get("--reduce", "pixelize"), align, cells, print):
                print(path)
        except ValueError as e:
            sys.exit(f"faces: {e}")
    elif sys.argv[1] in ("props", "slice-props"):
        args = sys.argv[2:]; opt = {}
        for flag in ("--root", "--spares", "--contact", "--palette"):
            if flag in args:
                i = args.index(flag); opt[flag] = args[i + 1]; del args[i:i + 2]
        if len(args) < 2 or args[1] not in PROP_SHEETS:
            sys.exit("usage: sheets.py props <sheet.png> <" + "|".join(PROP_SHEETS) + "> [--root R] [--spares DIR] "
                     "[--contact out.png] [--palette quantize|ramps]")
        root = opt.get("--root", args[2] if len(args) > 2 else ROOT)   # slice-props takes the root as a third argument
        saved = slice_props(args[0], args[1], root, opt.get("--spares"), palette=opt.get("--palette", "quantize"))
        for name, dest, (fw, fh) in saved:
            print(f"{name} {dest} fill {fw:.0%} x {fh:.0%}")
        if "--contact" in opt:
            print("contact", contact_props([d for _, d, _ in saved], opt["--contact"]))
