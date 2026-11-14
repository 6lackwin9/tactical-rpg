#!/usr/bin/env python3
"""Draft region map layouts for the Sunk Ward (AI-12, Art/UI 2026-11-14).

Each layout is written as a map template entry in the `maps.json` shape (`games/emberwake/data/maps/README.md`):
rows, zones, zone_legend, anchors, paths, props, ground, ground_legend, tags. It also carries `draft_markers`, which
is not schema: which marker each anchor is for, so Design can bind the location's `markers`. Design writes the
template spec from these (SUNKWARD-MAPS1); Build fills `maps.json` through `aegis content`. Nothing here is game data.

    python art_src/region_maps/build_layouts.py          # writes art_src/region_maps/<id>.json and checks them

The layouts follow each place's act-map landmark (art_src/lm_<location>.jpg): the landmark is the place seen from
above, so the region map keeps its features where the picture has them.
"""
import json
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]


class Layout:
    def __init__(self, w, h, fill_ground):
        self.w, self.h = w, h
        self.rows = [["." for _ in range(w)] for _ in range(h)]
        self.ground = [[fill_ground for _ in range(w)] for _ in range(h)]
        self.zones = [["-" for _ in range(w)] for _ in range(h)]
        self.props, self.anchors, self.markers = [], [], {}

    def rect(self, x0, y0, x1, y1, tile=None, ground=None, zone=None):
        """Fill the inclusive rectangle."""
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                if tile is not None:
                    self.rows[y][x] = tile
                if ground is not None:
                    self.ground[y][x] = ground
                if zone is not None:
                    self.zones[y][x] = zone

    def border(self):
        self.rect(0, 0, self.w - 1, 0, "#"); self.rect(0, self.h - 1, self.w - 1, self.h - 1, "#")
        self.rect(0, 0, 0, self.h - 1, "#"); self.rect(self.w - 1, 0, self.w - 1, self.h - 1, "#")

    def prop(self, pid, art, x, y, w=1, h=1, layer="sort", wall=True):
        """A standing prop turns its footprint into wall (`#`), as the schema asks; a ground prop leaves tiles alone."""
        if layer == "sort" and wall:
            self.rect(x, y, x + w - 1, y + h - 1, "#")
        self.props.append({"id": pid, "art": art, "x": x, "y": y, "w": w, "h": h, "layer": layer, "variant": -1,
                           "flip": False})

    def anchor(self, aid, x, y, marker=None):
        self.anchors.append({"id": aid, "x": x, "y": y})
        if marker:
            self.markers[aid] = marker

    def entry(self, tid, name, legend, ground_legend, tags):
        return {"id": tid, "name": name, "rows": ["".join(r) for r in self.rows],
                "zones": ["".join(r) for r in self.zones], "zone_legend": legend, "anchors": self.anchors,
                "paths": {}, "props": self.props, "ground": ["".join(r) for r in self.ground],
                "ground_legend": ground_legend, "tags": tags, "draft_markers": self.markers}


GROUND = {"s": "soot_yard", "c": "cobble_road", "f": "flagstone", "m": "mud", "o": "moss", "i": "iron_catwalk",
          "t": "tallow", "w": "water"}


def sluice_undercroft():
    """The hub: a pre-Extinguishing storm-drain vault three levels under the Sunk Ward (CANON 2, the four quarters).

    Landmark: a curved brick vault, the barred sluice gate arch with lit water in front of it, a ladder up the left
    wall, a bench. North: the Water-Lock Basin before the sluice gate (Talia's board, Varka's moored stall, Ivo).
    West: Sabina's Apothecary's Slab. East: Berek's Tinker's Hearth. South: the Dry Hollows (six kiln bunks round the
    brazier) with the Isolation Crypts behind bars in the south-east corner and the ladder shaft up to the streets in
    the south-west corner."""
    L = Layout(30, 20, "f")
    L.border()
    # Water-Lock Basin: the pool before the sluice gate, an iron walkway down its middle to the gate
    L.rect(10, 1, 19, 6, "~", "w")
    L.rect(14, 1, 15, 6, ".", "i")                     # the sluice walkway
    L.rect(9, 7, 20, 8, ".", "f")                      # the basin rim
    # the barred sluice gate arch in the north wall: two standing halves on the walls either side of the opening
    L.prop("sluice_arch_w", "sluice_arch", 12, 0, 2, 1)
    L.prop("sluice_arch_e", "sluice_arch", 16, 0, 2, 1); L.props[-1]["flip"] = True
    L.rect(14, 0, 15, 0, ".", "i")                     # the way through the raised bars (the exit tiles)
    L.prop("skiff", "varka_skiff", 17, 3, 2, 1, layer="ground")   # Varka's black-market stall: a moored skiff
    L.prop("board", "mission_board", 10, 7, 2, 1)       # Talia's bone-cipher mission board on the rim
    L.prop("basin_post_w", "ring_post", 9, 6); L.prop("basin_post_e", "ring_post", 20, 6)
    # vault piers between the quarters
    for x, y in ((8, 2), (8, 10), (20, 2), (20, 10)):
        L.prop(f"pier_{x}_{y}", "vault_pillar", x, y, 2, 2)
    # West: the Apothecary's Slab (Sabina)
    L.prop("retorts", "retort_shelf", 1, 1, 3, 1)
    L.prop("slab", "apothecary_slab", 3, 4, 3, 2)
    L.prop("herb_rack", "herb_rack", 1, 7, 1, 2)
    # East: the Tinker's Hearth (Berek)
    L.rect(22, 1, 28, 9, ground="s")
    L.prop("hearth", "tinker_hearth", 24, 1, 3, 2)
    L.prop("anvil", "anvil", 25, 5)
    L.prop("scrap", "scrap_heap", 27, 6, 2, 2)
    L.prop("tool_rack", "crate_stack", 22, 8, 2, 1)
    # Commons: the brazier the bunks face
    L.rect(9, 9, 20, 13, ground="c")
    L.prop("brazier", "brazier", 14, 11)
    # South: the Dry Hollows, six brick kiln bunks along the south wall, three either side
    for i, x in enumerate((7, 10, 13)):
        L.prop(f"bunk_{i + 1}", "kiln_bunk", x, 17, 2, 1)
    for i, x in enumerate((15, 18, 21)):   # the east three stop short of the crypts' door at (24, 17)
        L.prop(f"bunk_{i + 4}", "kiln_bunk", x, 17, 2, 1)
    L.rect(6, 14, 23, 18, ground="m")                    # damp trodden floor of the hollows
    L.rect(6, 15, 23, 16, ground="f")
    # Isolation Crypts: barred cells in the south-east corner, two stasis slots
    L.rect(24, 13, 24, 18, "#")
    L.prop("crypt_bars", "crypt_bars", 24, 14, 1, 3)
    L.rect(25, 13, 28, 18, ground="o")
    L.prop("stasis_1", "stasis_slab", 26, 15, 2, 1); L.prop("stasis_2", "stasis_slab", 26, 17, 2, 1)
    L.rect(24, 17, 24, 17, ".", "f")                     # the crypts' door (locked by default: a marker decides)
    # Ladder shaft up to the streets: the south-west corner
    L.rect(5, 13, 5, 18, "#")
    L.rect(1, 13, 4, 18, ground="s")
    L.prop("ladder", "iron_ladder", 0, 15, 1, 2)
    L.rect(0, 17, 0, 17, ".", "s")                       # the exit tile at the ladder's foot
    L.rect(5, 15, 5, 15, ".", "f")                       # the shaft's opening into the hollows
    L.rect(0, 0, 29, 19, zone="h")
    # anchors: the location's markers keep their anchor ids (spawn, gate, road, npc_a..d, lore_a, chest_a)
    L.anchor("spawn", 3, 17, "arrival from the world map (the ladder foot)")
    L.anchor("road", 0, 17, "exit `road`: up the ladder, World map: the Sunk Ward")
    L.anchor("gate", 14, 0, "exit `gate`: down the sluice gate to drowned_runoff_grate (to_anchor `sluice_in`)")
    L.anchor("gate_in", 15, 1, "arrival from the Grate (the Grate's `sluice` exit to_anchor)")
    L.anchor("npc_a", 4, 6, "Sabina at the Apothecary's Slab")
    L.anchor("npc_b", 25, 3, "Berek at the Tinker's Hearth")
    L.anchor("npc_c", 12, 8, "Talia at the mission board")
    L.anchor("npc_d", 17, 7, "Widow Varka at her moored skiff")
    L.anchor("lore_a", 13, 7, "Ivo beside Talia (after the Toll-Pen rescue)")
    L.anchor("chest_a", 1, 9, "the stash (herb rack corner)")
    L.anchor("rest", 14, 12, "the brazier: rest and the bunk roster")
    L.anchor("crypts", 24, 17, "the Isolation Crypts' door (Stage IV operatives, stasis slots)")
    return L.entry("sunk_ward_undercroft", "Sunk Ward: the Sluice Undercroft", {"h": {"id": "hub", "kind": "safe"}},
                   {k: v for k, v in GROUND.items() if k in "scfmoiw"}, ["sunk_ward"])


def drowned_runoff_grate():
    """Node 01: the egress where the drains meet the streets; flood-lock tablets under the silt.

    Landmark (revised take 2): a stone alcove in the north wall with two arched drain mouths (a small one west, the
    big sluice tunnel east with water running out of it), a big iron grate over swirling black water in front of
    them, square ring posts round it, steps, a lantern inside the alcove. South: the stairs up to the street ring road,
    one each side, toward Junction 7 (west) and the Third Ward road (east). The sluice tunnel leads down to the hub;
    a barred culvert on the east wall is the siphon shortcut (opens with Vesper's lock ciphers)."""
    L = Layout(32, 20, "m")
    L.border()
    # the alcove: brick back wall with the two drain mouths
    L.rect(1, 1, 30, 2, "#")
    L.prop("drain_w", "drain_arch", 5, 1, 3, 2)
    L.rect(20, 0, 20, 2, ".", "i")                      # the ledge into the tunnel (the exit to the hub)
    L.rect(19, 2, 19, 2, "~", "w"); L.rect(21, 2, 21, 2, "~", "w")
    L.prop("tunnel_w", "sluice_tunnel", 18, 1, 2, 1)     # the big tunnel mouth: two standing halves on the wall
    L.prop("tunnel_e", "sluice_tunnel", 21, 1, 2, 1); L.props[-1]["flip"] = True
    # the run-off: water from both mouths into the grate pool
    L.rect(6, 3, 6, 5, "~", "w")
    L.rect(19, 3, 21, 5, "~", "w"); L.rect(20, 3, 20, 5, ".", "i")   # the tunnel's catwalk over its own channel
    L.rect(9, 6, 22, 11, "~", "w")
    L.rect(7, 5, 8, 6, "~", "w")
    # the big iron grate: walkable iron over the pool, the focal point
    L.rect(12, 7, 19, 10, ".", "i")
    L.prop("grate", "drain_grate", 12, 7, 8, 4, layer="ground")
    L.rect(15, 11, 16, 12, ".", "i")                    # a short iron causeway from the grate to the yard
    L.rect(20, 6, 20, 6, ".", "i")                      # the tunnel catwalk lands on the grate's rim
    L.rect(20, 6, 20, 7, ".", "i")
    # square ring posts round the pool
    for i, (x, y) in enumerate(((8, 7), (8, 11), (23, 7), (23, 11), (11, 12), (20, 12))):
        L.prop(f"post_{i + 1}", "ring_post", x, y)
    # flood-lock tablets under the silt, west of the pool by the small drain mouth
    L.rect(1, 3, 5, 8, ground="m")
    L.prop("tablet_1", "lock_tablet", 2, 4); L.prop("tablet_2", "lock_tablet", 4, 4)
    L.prop("silt", "silt_bank", 1, 6, 4, 2, layer="ground")
    L.prop("lantern", "lantern_pole", 11, 3)
    # the yard and the two stairs up to the street ring road
    L.rect(1, 12, 30, 18, ground="c")
    L.rect(1, 13, 30, 13, ground="f")
    L.prop("steps_w", "stone_steps", 3, 16, 3, 3, layer="ground")
    L.prop("steps_e", "stone_steps", 26, 16, 3, 3, layer="ground")
    L.rect(4, 19, 4, 19, ".", "c"); L.rect(27, 19, 27, 19, ".", "c")
    L.prop("rail_w", "rubble_wall", 7, 17, 4, 1); L.prop("rail_e", "rubble_wall", 21, 17, 4, 1)
    # the barred culvert in the east wall: the siphon shortcut to the Tallow Vats
    L.rect(25, 3, 30, 10, ground="s")
    L.rect(31, 8, 31, 8, ".", "i")
    L.rect(26, 8, 30, 8, "~", "w"); L.rect(26, 8, 30, 8, ".", "i")
    L.prop("culvert", "culvert_gate", 31, 6, 1, 2)
    L.rect(0, 0, 31, 19, zone="g")
    L.anchor("spawn", 15, 17, "arrival from the world map (the yard, between the stairs)")
    L.anchor("streets_w", 4, 19, "exit: the west stair, World map: the drains (toward Junction 7)")
    L.anchor("streets_e", 27, 19, "exit: the east stair, World map: the drains (toward the Third Ward road)")
    L.anchor("sluice", 20, 0, "exit `sluice`: down the tunnel to sluice_undercroft (to_anchor `gate_in`)")
    L.anchor("sluice_in", 20, 3, "arrival from the hub (the hub's `gate` exit to_anchor)")
    L.anchor("tablet", 3, 5, "the `lock_tablet` point (sets sunk_ward.lock_ciphers_mapped)")
    L.anchor("culvert", 31, 8, "exit: the siphon culvert, World map (requires sunk_ward.lock_ciphers_done; Vats shortcut)")
    L.anchor("grate", 15, 8, "on the grate: a lore point or a set piece (optional)")
    return L.entry("sunk_ward_runoff_grate", "Sunk Ward: the Drowned Runoff Grate", {"g": {"id": "grate", "kind": "safe"}},
                   {k: v for k, v in GROUND.items() if k in "scfmiw"}, ["sunk_ward"])


def check(e):
    """The template rules a draft must pass (maps README, PROPS1 P-checks, REGIONGROUND1 G7)."""
    errs, rows, w, h = [], e["rows"], len(e["rows"][0]), len(e["rows"])
    if not (8 <= w <= 64 and 8 <= h <= 64) or any(len(r) != w for r in rows):
        errs.append("rows must be 8..64 square-cornered")
    for key in ("zones", "ground"):
        if len(e[key]) != h or any(len(r) != w for r in e[key]):
            errs.append(f"{key} grid size differs from rows")
    for y, r in enumerate(e["ground"]):
        for x, g in enumerate(r):
            water = e["ground_legend"].get(g) == "water"
            if rows[y][x] == "~" and not water:
                errs.append(f"G7: land ground on water at ({x},{y})")
            if rows[y][x] == "." and water:
                errs.append(f"G7: water ground on open land at ({x},{y})")
            if g not in e["ground_legend"]:
                errs.append(f"ground char {g!r} not in legend at ({x},{y})")
    seen = {}
    for p in e["props"]:
        if p["x"] < 0 or p["y"] < 0 or p["x"] + p["w"] > w or p["y"] + p["h"] > h:
            errs.append(f"prop {p['id']} outside the map")
            continue
        if p["layer"] == "sort":
            for y in range(p["y"], p["y"] + p["h"]):
                for x in range(p["x"], p["x"] + p["w"]):
                    if rows[y][x] != "#":
                        errs.append(f"standing prop {p['id']} on a non-wall tile ({x},{y})")
                    if (x, y) in seen:
                        errs.append(f"standing props {seen[(x, y)]} and {p['id']} share ({x},{y})")
                    seen[(x, y)] = p["id"]
    ids = [a["id"] for a in e["anchors"]]
    if ids.count("spawn") != 1:
        errs.append("exactly one spawn anchor")
    if len(set(ids)) != len(ids):
        errs.append("anchor ids repeat")
    for a in e["anchors"]:
        if rows[a["y"]][a["x"]] != ".":
            errs.append(f"anchor {a['id']} not on an open tile ({a['x']},{a['y']})")
    if len(e["props"]) > 128:
        errs.append("more than 128 props")
    return errs


def main():
    ok = True
    for fn, loc in ((sluice_undercroft, "sluice_undercroft"), (drowned_runoff_grate, "drowned_runoff_grate")):
        e = fn()
        e["location"] = loc
        errs = check(e)
        (HERE / f"{loc}.json").write_text(json.dumps(e, indent=1) + "\n", encoding="utf-8", newline="\n")
        print(f"{loc}: {len(e['rows'][0])} x {len(e['rows'])}, {len(e['props'])} props, {len(e['anchors'])} anchors,"
              f" {'OK' if not errs else str(len(errs)) + ' problems'}")
        for m in errs:
            print("   ", m)
        ok = ok and not errs
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
