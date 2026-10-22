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
    from PIL import Image
    import pixelize as X, portrait as P
    small = X.mode_reduce(key_out(tile), grid); sp = small.load()
    mask = [(x, y) for y in range(grid) for x in range(grid) if sp[x, y][3] >= 128]
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
    out = Image.new("RGBA", (grid, grid), (0, 0, 0, 0)); op = out.load()
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
        ring = [(x, y) for y in range(grid) for x in range(grid) if op[x, y][3] == 0 and
                any(0 <= x + ox < grid and 0 <= y + oy < grid and op[x + ox, y + oy][3] for ox, oy in ((1, 0), (-1, 0), (0, 1), (0, -1)))]
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
