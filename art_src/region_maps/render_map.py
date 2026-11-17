#!/usr/bin/env python3
"""Render a draft region map with real art: ground textures, walls, boundary and props (Art/UI 2026-11-15).

    python art_src/region_maps/render_map.py <location> [--tiles DIR] [--props DIR] [--out x.png] [--inside ZONE]

Defaults: the tiles in art_src/sunk_ward/tiles, the cut props in art_src/sunk_ward/props_cut, and the picture to
art_src/region_maps/previews/<location>.png.

A look-and-feel preview, not the game's renderer: it draws art_src/region_maps/<location>.json at 32 px a tile, the
props' art scale. Grounds are the web-queue tile textures (TILES names the picked take), toned in code toward each
ground's catalog colour, as REGIONGROUND1 does at the cut; a ground with no texture yet draws its flat colour. Wall
tiles draw a wall set's top, and its face where open ground lies south of them: each wall cell takes the set of the
room it faces (ROOM_WALLS, from the layout's draft_room_styles), and walls outside every room the boundary set. Standing props stand
bottom-centred on their footprint and sort by their feet; ground props lie under them; roofs draw last unless
--inside names their zone.
"""
import argparse
import glob
import json
import pathlib

from PIL import Image, ImageDraw, ImageEnhance, ImageFilter

ROOT = pathlib.Path(__file__).resolve().parents[2]
T = 32
TILES = {  # ground id -> (file stem, take); the picks of ART_UI_STUDIO 3.10
    "floorboards": ("ground_floorboards", 1), "cobble_road": ("ground_cobble_road", 1), "flagstone": ("ground_flagstone", 2),
    "mud": ("ground_mud", 2), "soot_yard": ("ground_soot_yard", 1), "iron_catwalk": ("ground_iron_catwalk", 2),
    "tallow": ("ground_tallow", 1), "water": ("ground_water", 1), "moss": ("ground_moss", 1),
}
BOUNDARY = {"sluice_undercroft": "drain_brick", "drowned_runoff_grate": "drain_brick", "crucible_watchpost": "gorge_rock",
            "slag_gate": "gorge_rock"}   # the rest: blind_tenements
WALL_TAKES = {"drain_brick": (1, 2), "blind_tenements": (2, 1), "gorge_rock": (1, 2),
              "interior_plaster": (2, 1), "interior_stone": (1, 2)}   # (top take, face take)
CELLS = {"flagstone": 4, "cobble_road": 6, "floorboards": 6}   # tiles a texture spans (default 8): sets the size of its stones
# Per-room wall sets (art_src/region_maps/README.md open item 3): a room's draft_room_styles text names its walls.
# Only the sets the text asks for are listed; every other room keeps its location's boundary set.
ROOM_WALLS = {"sluice_undercroft": {"slab": "interior_plaster", "crypts": "interior_stone"}}
WALL_TONES = {"interior_plaster": ((50, 47, 44), (96, 90, 82)), "interior_stone": ((34, 37, 42), (66, 70, 80))}  # (top, face); default below
GREY = {"cobble_road": 0.75, "flagstone": 0.5}                # how far a texture's own hue is pulled to grey before toning
PATCHY = {"soot_yard": 0.52, "moss": 0.46}   # share of a cell these grounds cover: ash and moss lie in patches on the floor
UNDER = "flagstone"                           # what shows between the patches
FIT = {"portcullis": (72, 84), "kiln_bunk": (64, 56), "iron_ladder": (40, 56), "crypt_bars": (48, 52)}   # (w, h) px a wall-mounted art is scaled into
BRIGHT = 1.9   # the catalog colours are near black; the preview lifts everything by this, like the grid preview


def hexrgb(h):
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))


def tone(im, colour, spread=0.38, cells=8, grey=0.0):
    """A texture toned toward `colour`: scaled to `cells` tiles, its mean moved onto the colour, its contrast cut."""
    im = im.convert("RGB").resize((cells * T, cells * T), Image.LANCZOS)
    if grey:
        im = Image.blend(im, im.convert("L").convert("RGB"), grey)
    px = im.load(); w, h = im.size
    n = w * h
    mean = [sum(px[x, y][c] for y in range(0, h, 3) for x in range(0, w, 3)) / len(range(0, h, 3)) / len(range(0, w, 3)) for c in range(3)]
    for y in range(h):
        for x in range(w):
            p = px[x, y]
            px[x, y] = tuple(max(0, min(255, round(colour[c] + (p[c] - mean[c]) * spread))) for c in range(3))
    return im


def find(folder, stem, take):
    f = glob.glob(str(pathlib.Path(folder) / f"{stem}_q*_{take}.png"))
    return Image.open(sorted(f)[-1]) if f else None   # the latest request of that name (a re-run wins)


def tiled(tex, x, y):
    w, h = tex.size
    return tex.crop(((x * T) % w, (y * T) % h, (x * T) % w + T, (y * T) % h + T))


def render(loc, tiles_dir, props_root, out, inside=""):
    e = json.loads((ROOT / "art_src/region_maps" / f"{loc}.json").read_text(encoding="utf-8"))
    cat = {g["id"]: hexrgb(g["colour"]) for g in json.loads((ROOT / "games/emberwake/data/catalog/grounds.json").read_text(encoding="utf-8"))["entries"]}
    cat.setdefault("floorboards", hexrgb("#3a2c22"))
    lift = lambda c: tuple(min(255, round(v * BRIGHT)) for v in c)
    rows, W, H, M = [list(r) for r in e["rows"]], len(e["rows"][0]), len(e["rows"]), 2
    # A standing prop's footprint is wall only so nobody walks through it. Drawn as wall, the brick shows round the
    # prop and it seems to float; a free-standing prop (open ground on some side of its footprint) stands on the floor.
    for p in e["props"]:
        if p["layer"] != "sort":
            continue
        cells = [(x, y) for y in range(p["y"], p["y"] + p["h"]) for x in range(p["x"], p["x"] + p["w"])]
        ring = [(x + dx, y + dy) for x, y in cells for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))]
        if any(0 <= x < W and 0 <= y < H and e["rows"][y][x] != "#" for x, y in ring) and not any(x in (0, W - 1) or y in (0, H - 1) for x, y in cells):
            for x, y in cells:
                rows[y][x] = "."
    tex = {}
    for gid in set(e["ground_legend"].values()) | {UNDER}:
        src = find(tiles_dir, *TILES[gid]) if gid in TILES else None
        tex[gid] = tone(src, lift(cat[gid]), cells=CELLS.get(gid, 8), grey=GREY.get(gid, 0.0)) if src else Image.new("RGB", (8 * T, 8 * T), lift(cat[gid]))
    bset = BOUNDARY.get(e.get("location", loc), "blind_tenements")

    def wall_set(name):
        """(top, face, face_dark) textures of one wall set, toned dark like the boundary."""
        kind = "boundary" if name in BOUNDARY.values() or name == "blind_tenements" else "walls"
        top_take, face_take = WALL_TAKES[name]
        tc, fc = WALL_TONES.get(name, ((40, 40, 46), (60, 58, 64)))
        t_ = find(tiles_dir, f"{kind}_{name}_wall", top_take)
        f_ = find(tiles_dir, f"{kind}_{name}_wall_face", face_take)
        t_ = tone(t_, tc, 0.3) if t_ else Image.new("RGB", (8 * T, 8 * T), tc)
        f_ = tone(f_, fc, 0.5, cells=4) if f_ else Image.new("RGB", (4 * T, 4 * T), fc)
        return t_, f_, ImageEnhance.Brightness(f_).enhance(0.72)

    top, face, _ = wall_set(bset)
    sets = {bset: wall_set(bset)}
    room_walls = ROOM_WALLS.get(e.get("location", loc), {})
    for name in set(room_walls.values()):
        sets[name] = wall_set(name)
    zone_id = {k: v["id"] for k, v in e.get("zone_legend", {}).items()}
    zrows = e.get("zones", [])

    def room_at(x, y):
        return zone_id.get(zrows[y][x]) if 0 <= y < len(zrows) and 0 <= x < len(zrows[y]) else None

    def wall_of(x, y):
        """The wall set of wall cell (x, y): the room whose floor it faces (the open cell south of it), else the nearest
        room within two cells (south first, then west, east, north), else the boundary set."""
        if not room_walls:
            return sets[bset]
        for k in (1, 2):
            for dx, dy in ((0, k), (-k, 0), (k, 0), (0, -k)):
                r = room_at(x + dx, y + dy)
                if r:
                    return sets[room_walls.get(r, bset)]
        return sets[bset]
    im = Image.new("RGBA", ((W + 2 * M) * T, (H + 2 * M) * T))
    # Ground: one picture composed from the textures, with soft irregular borders between land grounds (REGIONGROUND1
    # EN 2: an edge wanders up to about 0.3 cell). The lookup position of every pixel is pushed by smooth noise, so a
    # border wobbles instead of running along the grid. Water never moves: `rows` decide where water is (G7).
    import numpy as np
    rng = np.random.default_rng(7)
    gh, gw = (H + 2 * M) * T, (W + 2 * M) * T

    def noise(cells_per_bump):
        low = rng.random((max(2, round((H + 2 * M) / cells_per_bump) + 2), max(2, round((W + 2 * M) / cells_per_bump) + 2))).astype("float32")
        return np.asarray(Image.fromarray((low * 255).astype("uint8")).resize((gw, gh), Image.BICUBIC), dtype="float32") / 255 - 0.5

    amp = 0.62   # cells, peak to peak
    nx = (noise(1.3) + 0.5 * noise(0.6)) * amp
    ny = (noise(1.3) + 0.5 * noise(0.6)) * amp
    yy, xx = np.mgrid[0:gh, 0:gw]
    cu, cv = xx / T - M, yy / T - M                       # this pixel's cell
    lu = np.clip(np.floor(cu + nx), 0, W - 1).astype(int)  # the cell its ground is looked up in
    lv = np.clip(np.floor(cv + ny), 0, H - 1).astype(int)
    ou = np.clip(np.floor(cu), 0, W - 1).astype(int)
    ov = np.clip(np.floor(cv), 0, H - 1).astype(int)
    ids = sorted(set(e["ground_legend"].values()))
    gid = np.array([[ids.index(e["ground_legend"][c]) for c in r] for r in e["ground"]])
    water = np.array([ids.index(g) if g == "water" else -1 for g in ids]).max()
    own, look = gid[ov, ou], gid[lv, lu]
    pick = np.where((own == water) | (look == water), own, look)
    canvas = np.zeros((gh, gw, 3), dtype="uint8")
    def full_of(g):
        tx = np.asarray(tex[g]); th, tw = tx.shape[:2]
        return tx[(yy % th), (xx % tw)]

    blobs = noise(1.6) + 0.6 * noise(0.7) + 0.25 * noise(0.3)       # where a patchy ground lies thick
    lo, hi = float(blobs.min()), float(blobs.max())
    blobs = (blobs - lo) / (hi - lo)
    under = full_of(UNDER)
    for k, g in enumerate(ids):
        full = full_of(g)
        sel = pick == k
        if g in PATCHY:
            cut = float(np.quantile(blobs, 1 - PATCHY[g]))
            thick = sel & (blobs >= cut)
            thin = sel & (blobs < cut) & (blobs >= cut - 0.08)        # a soft edge: half ash, half floor
            canvas[sel] = under[sel]
            canvas[thin] = (under[thin].astype("int16") * 0.5 + full[thin].astype("int16") * 0.5).astype("uint8")
            canvas[thick] = full[thick]
        else:
            canvas[sel] = full[sel]
    im.paste(Image.fromarray(canvas), (0, 0))
    # a stone rim where land meets water: a dark line on the land side, a pale lip on the water side
    rim = Image.new("RGBA", im.size, (0, 0, 0, 0)); dr = ImageDraw.Draw(rim)
    for y in range(H):
        for x in range(W):
            if rows[y][x] != "~":
                continue
            X0, Y0 = (x + M) * T, (y + M) * T
            for dx, dy, box in ((0, -1, (X0, Y0, X0 + T - 1, Y0 + 3)), (0, 1, (X0, Y0 + T - 4, X0 + T - 1, Y0 + T - 1)),
                                (-1, 0, (X0, Y0, X0 + 3, Y0 + T - 1)), (1, 0, (X0 + T - 4, Y0, X0 + T - 1, Y0 + T - 1))):
                n = rows[y + dy][x + dx] if 0 <= y + dy < H and 0 <= x + dx < W else "#"
                if n == ".":
                    dr.rectangle(box, fill=(96, 98, 108, 255))
                    b = (box[0] - dx * 4 if dx else box[0], box[1] - dy * 4 if dy else box[1], box[2] - dx * 4 if dx else box[2], box[3] - dy * 4 if dy else box[3])
                    dr.rectangle(b, fill=(0, 0, 0, 80))
    im.alpha_composite(rim)
    # DEPTH (Tushar 2026-11-15: "need to create the sense of depth to the map, it feels flat"). Light comes from the
    # upper left. (1) Walls stand two tiles tall: the two wall cells above open ground show the wall's face, the upper
    # one lighter, with a pale cap line where the face meets the wall's top. (2) Walls cast a soft shadow on the floor
    # to their south and east. (3) Water lies below the floor: a dark inner edge along its north and west sides.
    def open_at(x, y):
        return 0 <= x < W and 0 <= y < H and rows[y][x] != "#"

    FACE_H = 1   # every wall shows the same one-tile face under its top (2026-11-15: wall joins must be consistent)

    def _depth(x, y):
        if open_at(x, y):
            return 0
        dd = next((k for k in range(1, FACE_H + 1) if open_at(x, y + k)), 0)
        return 0 if dd == 2 and open_at(x, y - 1) else dd

    def _is_face(x, y):
        return _depth(x, y) > 0

    for y in range(-M, H + M):
        for x in range(-M, W + M):
            if open_at(x, y):
                continue
            depth = next((k for k in range(1, FACE_H + 1) if open_at(x, y + k)), 0)
            if depth == 2 and open_at(x, y - 1):
                depth = 0        # a wall two tiles thick: its far row shows its top, its near row its face
            X0, Y0 = (x + M) * T, (y + M) * T
            top, face, face_dark = wall_of(x, y)
            if depth:
                # the near row of a face is a little darker (it is lower, further from the light); the row that meets
                # the wall's top carries a pale cap line
                upper = depth == 2 or open_at(x, y - 1) or not (0 <= y - 1) or depth == 1 and not any(open_at(x, y - k) for k in (1,)) and not _is_face(x, y - 1)
                im.paste(tiled(face_dark if depth == 1 and _is_face(x, y - 1) else face, x, 4 - depth), (X0, Y0))
                if not _is_face(x, y - 1):
                    ImageDraw.Draw(im).rectangle([X0, Y0, X0 + T - 1, Y0 + 2], fill=(132, 130, 140, 255))
            else:
                im.paste(tiled(top, x, y), (X0, Y0))
    top, face, face_dark = sets[bset]
    edge = ImageDraw.Draw(im)
    for y in range(-1, H + 1):
        for x in range(-1, W + 1):
            if open_at(x, y) or _is_face(x, y):
                continue
            X0, Y0 = (x + M) * T, (y + M) * T
            if open_at(x + 1, y) or _is_face(x + 1, y):
                edge.line([X0 + T - 1, Y0, X0 + T - 1, Y0 + T - 1], fill=(14, 14, 18, 255), width=2)
            if open_at(x - 1, y) or _is_face(x - 1, y):
                edge.line([X0, Y0, X0, Y0 + T - 1], fill=(70, 70, 78, 255), width=1)
            if open_at(x, y - 1):
                edge.line([X0, Y0, X0 + T - 1, Y0], fill=(70, 70, 78, 255), width=1)
    sh = Image.new("RGBA", im.size, (0, 0, 0, 0)); d = ImageDraw.Draw(sh)
    for y in range(H):
        for x in range(W):
            if not open_at(x, y):
                continue
            X0, Y0 = (x + M) * T, (y + M) * T
            if not open_at(x, y - 1):                       # a wall to the north: its shadow falls south
                for k in range(14):
                    d.line([X0, Y0 + k, X0 + T - 1, Y0 + k], fill=(0, 0, 0, round(120 * (1 - k / 14) ** 1.5)))
            if not open_at(x - 1, y):                       # a wall to the west: its shadow falls east
                for k in range(10):
                    d.line([X0 + k, Y0, X0 + k, Y0 + T - 1], fill=(0, 0, 0, round(95 * (1 - k / 10) ** 1.5)))
            if rows[y][x] == "~":                           # water sits below the floor
                if open_at(x, y - 1) and rows[y - 1][x] != "~" or not open_at(x, y - 1):
                    for k in range(9):
                        d.line([X0, Y0 + 4 + k, X0 + T - 1, Y0 + 4 + k], fill=(0, 0, 0, round(130 * (1 - k / 9))))
                if open_at(x - 1, y) and rows[y][x - 1] != "~":
                    for k in range(7):
                        d.line([X0 + 4 + k, Y0, X0 + 4 + k, Y0 + T - 1], fill=(0, 0, 0, round(110 * (1 - k / 7))))
    im.alpha_composite(sh)

    def art(p):
        for base in (pathlib.Path(props_root) / "games/emberwake/assets/props/sunk_ward", pathlib.Path(props_root), ROOT / "games/emberwake/assets/props/sunk_ward",
                     ROOT / "kits/rpg/assets/props/common"):
            f = base / p["art"] / "v0.png"
            if f.exists():
                a = Image.open(f).convert("RGBA")
                return a.transpose(Image.FLIP_LEFT_RIGHT) if p.get("flip") else a
        return None

    face_mean = [sum(c[k] for c in face.resize((8, 8)).getdata()) / 64 for k in range(3)]

    def wall_mounted(p):
        """A standing prop whose whole footprint is a wall's face row: an arch, a niche, bars, a ladder."""
        return p["layer"] == "sort" and all(_is_face(x, p["y"] + p["h"] - 1) and e["rows"][p["y"] + p["h"] - 1][x] == "#"
                                            for x in range(p["x"], p["x"] + p["w"]))

    def into_wall(a, p):
        """The prop's stone and brick pulled to the wall's colour, and a tall arch scaled to sit in a low wall."""
        limit = FIT.get(p["art"])
        if limit and (a.width > limit[0] or a.height > limit[1]):
            k = min(limit[0] / a.width, limit[1] / a.height)
            a = a.resize((round(a.width * k), round(a.height * k)), Image.LANCZOS)
        px = a.load()
        opaque = [px[x, y] for y in range(0, a.height, 2) for x in range(0, a.width, 2) if px[x, y][3] > 128]
        if not opaque:
            return a
        mean = [sum(c[k] for c in opaque) / len(opaque) for k in range(3)]
        for y in range(a.height):
            for x in range(a.width):
                r, g, b, al = px[x, y]
                if al and max(r, g, b) - min(r, g, b) < 40 and max(r, g, b) < 150:      # grey masonry only: fire, wood and cloth keep their colour
                    px[x, y] = tuple(max(0, min(255, round(c + (face_mean[k] - mean[k]) * 0.75))) for k, c in enumerate((r, g, b))) + (al,)
        return a

    def place(p, a):
        mounted = wall_mounted(p)
        if mounted:
            a = into_wall(a, p)
            p = dict(p, no_shadow=True)
        x = (p["x"] + M) * T + (p["w"] * T - a.width) // 2
        if p["layer"] == "sort" and not p.get("no_shadow"):
            # a contact shadow under the base, so the prop sits on the floor
            box = a.getbbox() or (0, 0, a.width, a.height)
            by = (p["y"] + p["h"] + M) * T - (a.height - box[3])
            sh2 = Image.new("RGBA", im.size, (0, 0, 0, 0))
            # no cast shadow (Tushar 2026-11-15: an oval under a front-view prop makes it float): only a thin dark
            # line where the base meets the floor
            ImageDraw.Draw(sh2).rectangle([x + box[0] + 1, by - 2, x + box[2] - 1, by + 1], fill=(0, 0, 0, 120))
            sh2 = sh2.filter(ImageFilter.GaussianBlur(1))
            im.alpha_composite(sh2)
        if p["layer"] == "ground":
            y = (p["y"] + M) * T + (p["h"] * T - a.height) // 2
        else:
            y = (p["y"] + p["h"] + M) * T - a.height
        im.alpha_composite(a, (x, max(0, y)))

    missing = []
    order = sorted(e["props"], key=lambda p: (p["layer"] != "ground", p["y"] + p["h"], p["x"]))
    for p in order:
        if p["layer"] == "roof":
            continue
        a = art(p)
        if a is None:
            missing.append(p["art"])
            d2 = ImageDraw.Draw(im)
            d2.rectangle([(p["x"] + M) * T + 3, (p["y"] + M) * T + 3, (p["x"] + p["w"] + M) * T - 4, (p["y"] + p["h"] + M) * T - 4], outline=(255, 0, 255, 255), width=2)
            continue
        place(p, a)
    for p in e["props"]:
        if p["layer"] == "roof" and p.get("roof_of") != inside:
            a = art(p)
            if a is None:
                missing.append(p["art"])
            else:
                place(dict(p, layer="sort"), a)
    # (4) Light and dark: the whole map is dimmed, and each fire or lamp lifts a warm pool round itself, so the room
    # has near and far, lit and unlit.
    LIGHTS = {"brazier": (150, (255, 170, 80)), "tinker_hearth": (140, (255, 140, 60)), "furnace_mouth": (170, (255, 120, 40)),
              "lantern_pole": (95, (255, 190, 110)), "watch_torch": (95, (255, 130, 60)), "violet_lamp_post": (100, (150, 130, 255)),
              "apothecary_slab": (85, (255, 200, 130)), "retort_shelf": (60, (255, 200, 130)), "kiln_bunk": (45, (255, 190, 120)),
              "hearth_stove": (70, (255, 150, 80)), "candle_stand": (70, (255, 200, 120)), "shrine_altar": (80, (255, 200, 120))}
    dark = Image.new("L", im.size, 96)                      # how much darkness lies over the map (0 none, 255 black)
    glow = Image.new("RGBA", im.size, (0, 0, 0, 0))
    dd, dg = ImageDraw.Draw(dark), ImageDraw.Draw(glow)
    for p in e["props"]:
        if p["art"] in LIGHTS and (p["layer"] != "roof"):
            r, col = LIGHTS[p["art"]]
            cx, cy = (p["x"] + p["w"] / 2 + M) * T, (p["y"] + p["h"] + M) * T - 14
            for k in range(12, 0, -1):
                rr = r * k / 12
                dd.ellipse([cx - rr, cy - rr * 0.8, cx + rr, cy + rr * 0.8], fill=round(96 * (k / 12) ** 1.6))
                dg.ellipse([cx - rr, cy - rr * 0.8, cx + rr, cy + rr * 0.8], fill=col + (round(5 + 26 * (1 - k / 12)),))
    dark = dark.filter(ImageFilter.GaussianBlur(10))
    im.alpha_composite(glow.filter(ImageFilter.GaussianBlur(8)))
    night = Image.new("RGBA", im.size, (6, 8, 16, 255)); night.putalpha(dark)
    im.alpha_composite(night)
    # who stands where: a small figure-sized marker on each NPC and arrival anchor, for scale (a unit is 16 x 24 art px)
    d3 = ImageDraw.Draw(im)
    for a in e["anchors"]:
        note = e.get("draft_markers", {}).get(a["id"], "")
        col = (63, 201, 224) if a["id"].startswith("npc") or a["id"] == "lore_a" else (242, 180, 55) if a["id"] == "spawn" else None
        if col:
            cx, by = (a["x"] + M) * T + T // 2, (a["y"] + M + 1) * T - 4
            d3.ellipse([cx - 9, by - 5, cx + 9, by + 3], fill=(0, 0, 0, 90))
            d3.rounded_rectangle([cx - 8, by - 40, cx + 8, by], radius=5, fill=col + (235,), outline=(10, 10, 14, 255), width=2)
    im.convert("RGB").save(out)
    print(f"{loc}: {im.size[0]}x{im.size[1]} -> {out}; no art for: {sorted(set(missing)) or 'none'}")


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("location"); ap.add_argument("--tiles", default=str(ROOT / "art_src/sunk_ward/tiles")); ap.add_argument("--props", default=str(ROOT / "art_src/sunk_ward/props_cut"))
    ap.add_argument("--out", default=""); ap.add_argument("--inside", default="")
    a = ap.parse_args()
    render(a.location, a.tiles, a.props, a.out or str(ROOT / "art_src/region_maps/previews" / f"{a.location}.png"), a.inside)
