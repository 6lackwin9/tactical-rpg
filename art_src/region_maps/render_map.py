#!/usr/bin/env python3
"""Render a draft region map with real art: ground textures, walls, boundary and props (Art/UI 2026-11-15).

    python art_src/region_maps/render_map.py <location> --tiles <folder of tile PNGs> --props <root with props/> --out x.png

A look-and-feel preview, not the game's renderer: it draws art_src/region_maps/<location>.json at 32 px a tile, the
props' art scale. Grounds are the web-queue tile textures (TILES names the picked take), toned in code toward each
ground's catalog colour, as REGIONGROUND1 does at the cut; a ground with no texture yet draws its flat colour. Wall
tiles draw the boundary set's top, and its face where open ground lies south of them. Standing props stand
bottom-centred on their footprint and sort by their feet; ground props lie under them; roofs draw last unless
--inside names their zone.
"""
import argparse
import glob
import json
import pathlib

from PIL import Image, ImageDraw, ImageEnhance

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
GREY = {"cobble_road": 0.75, "flagstone": 0.5}                # how far a texture's own hue is pulled to grey before toning
BRIGHT = 1.9   # the catalog colours are near black; the preview lifts everything by this, like the grid preview


def hexrgb(h):
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))


def tone(im, colour, spread=0.55, cells=8, grey=0.0):
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
    rows, W, H, M = e["rows"], len(e["rows"][0]), len(e["rows"]), 2
    tex = {}
    for gid in set(e["ground_legend"].values()):
        src = find(tiles_dir, *TILES[gid]) if gid in TILES else None
        tex[gid] = tone(src, lift(cat[gid]), cells=CELLS.get(gid, 8), grey=GREY.get(gid, 0.0)) if src else Image.new("RGB", (8 * T, 8 * T), lift(cat[gid]))
    bset = BOUNDARY.get(e.get("location", loc), "blind_tenements")
    top_take, face_take = WALL_TAKES[bset]
    top = find(tiles_dir, f"boundary_{bset}_wall", top_take)
    face = find(tiles_dir, f"boundary_{bset}_wall_face", face_take)
    top = tone(top, (44, 44, 50), 0.6) if top else Image.new("RGB", (8 * T, 8 * T), (40, 40, 46))
    face = tone(face, (62, 60, 66), 0.7, cells=4) if face else Image.new("RGB", (4 * T, 4 * T), (60, 58, 64))
    im = Image.new("RGBA", ((W + 2 * M) * T, (H + 2 * M) * T))
    for y in range(-M, H + M):
        for x in range(-M, W + M):
            inside_map = 0 <= x < W and 0 <= y < H
            ch = rows[y][x] if inside_map else "#"
            if ch == "#":
                below = rows[y + 1][x] if 0 <= x < W and -1 <= y < H - 1 else "#"
                t = tiled(face, x, 3) if below != "#" else tiled(top, x, y)
            else:
                t = tiled(tex[e["ground_legend"][e["ground"][y][x]]], x, y)
            im.paste(t, ((x + M) * T, (y + M) * T))
    # a soft shade on open ground under a wall to its north, so walls stand up
    sh = Image.new("RGBA", im.size, (0, 0, 0, 0)); d = ImageDraw.Draw(sh)
    for y in range(H):
        for x in range(W):
            if rows[y][x] != "#" and y > 0 and rows[y - 1][x] == "#":
                d.rectangle([(x + M) * T, (y + M) * T, (x + M + 1) * T - 1, (y + M) * T + 7], fill=(0, 0, 0, 70))
    im.alpha_composite(sh)

    def art(p):
        for base in (pathlib.Path(props_root) / "games/emberwake/assets/props/sunk_ward", ROOT / "games/emberwake/assets/props/sunk_ward",
                     ROOT / "kits/rpg/assets/props/common"):
            f = base / p["art"] / "v0.png"
            if f.exists():
                a = Image.open(f).convert("RGBA")
                return a.transpose(Image.FLIP_LEFT_RIGHT) if p.get("flip") else a
        return None

    def place(p, a):
        x = (p["x"] + M) * T + (p["w"] * T - a.width) // 2
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
    ap.add_argument("location"); ap.add_argument("--tiles", required=True); ap.add_argument("--props", required=True)
    ap.add_argument("--out", required=True); ap.add_argument("--inside", default="")
    a = ap.parse_args()
    render(a.location, a.tiles, a.props, a.out, a.inside)
