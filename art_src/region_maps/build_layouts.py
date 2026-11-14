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


def alleyway_junction_7():
    """Node 02: a street crossing in the Seventh Ward where Captain Vane has sealed the grates (M1, the Sweep).

    Landmark (take B): crooked tenements round a cobbled yard, barred iron gates, a spiked timber barricade, a
    rust-crimson pennant and one Watch torch. Two alleys cross in the middle; four tenement blocks fill the corners,
    the old Watch armory in the south-east one. The route from the Grate arrives over the bridge from the east, the
    road to the Kiln leaves north; the west alley ends at a sealed barred gate."""
    L = Layout(32, 20, "m")
    L.border()
    blocks = {"nw": (1, 1, 12, 6), "ne": (19, 1, 30, 6), "sw": (1, 13, 12, 18), "se": (19, 13, 30, 18)}
    for x0, y0, x1, y1 in blocks.values():
        L.rect(x0, y0, x1, y1, "#")
    L.prop("tenement_1", "tenement_block", 1, 1, 6, 5); L.prop("tenement_2", "tenement_block", 7, 1, 6, 5)
    L.prop("tenement_3", "tenement_block", 19, 1, 6, 5); L.prop("tenement_4", "tenement_block", 25, 1, 6, 5)
    L.prop("tenement_5", "tenement_block", 1, 14, 6, 5); L.prop("tenement_6", "tenement_block", 7, 14, 6, 5)
    L.prop("armory", "watch_armory", 19, 14, 6, 5); L.prop("tenement_7", "tenement_block", 25, 14, 6, 5)
    for i, (x, y) in enumerate(((12, 6), (19, 6), (12, 13), (19, 13))):
        L.prop(f"torch_{i + 1}", "watch_torch", x, y)
    L.prop("pennant", "rust_pennant", 18, 6)
    L.rect(1, 7, 30, 12, ground="c"); L.rect(13, 1, 18, 18, ground="c")
    L.rect(13, 7, 18, 12, ground="f")                   # the yard where the alleys cross
    L.prop("barricade_n", "spiked_barricade", 14, 4, 3, 1)
    L.prop("barricade_s", "spiked_barricade", 15, 15, 3, 1)
    L.prop("barricade_w", "spiked_barricade", 8, 9, 3, 1)
    L.prop("grate_w", "street_grate", 4, 10, 2, 2, layer="ground")    # the grates Vane sealed
    L.prop("grate_e", "street_grate", 24, 7, 2, 2, layer="ground")
    L.prop("sealed_gate", "barred_gate", 0, 8, 1, 2)
    L.rect(15, 0, 16, 0, ".", "c"); L.rect(31, 9, 31, 10, ".", "c")
    L.rect(0, 0, 31, 19, zone="d"); L.rect(24, 7, 31, 12, zone="s")
    L.anchor("spawn", 29, 10, "arrival from the world map (the bridge from the Grate)")
    L.anchor("exit_bridge", 31, 10, "exit `world_map` (was anchor `east`): the bridge east, World map: the Seventh Ward")
    L.anchor("exit_north", 15, 0, "exit (new, optional): the road north toward the Kiln, World map")
    L.anchor("sweep", 15, 10, "encounter `sweep` (was `site_b`): Captain Vane's sweep in the yard")
    L.anchor("cache", 21, 12, "point `bellows_cache` (was `altar`): the old Watch armory's door")
    return L.entry("sunk_ward_junction_7", "Sunk Ward: Alleyway Junction 7",
                   {"s": {"id": "bridge", "kind": "safe"}, "d": {"id": "alley", "kind": "danger"}},
                   {k: v for k, v in GROUND.items() if k in "scfm"}, ["sunk_ward"])


def ash_crawl_crossroads():
    """Node 03: an open-air culling station where the slum alleys meet, the Seventh Ward Toll-Pen beside the cages
    (M2, Proctor Quell's titration cages).

    Landmark (take A): cages on a wooden platform, the chained toll-pen gate, a crooked violet lamp-post and its
    violet pool of light, ash-roofed shacks. An ashy plaza in the middle, the cage platforms on its north side, the
    fenced Toll-Pen east of them, shacks round the edge. The route from the Grate arrives from the east; the alley
    toward the Vats leaves north."""
    L = Layout(32, 20, "s")
    L.border()
    # shacks round the edge, gaps where the alleys run
    L.rect(1, 1, 13, 3, "#"); L.rect(18, 1, 30, 3, "#"); L.rect(1, 16, 30, 18, "#"); L.rect(1, 4, 4, 15, "#")
    L.rect(27, 4, 30, 7, "#"); L.rect(27, 12, 30, 15, "#")
    for i, (x, y) in enumerate(((1, 1), (5, 1), (9, 1), (18, 1), (22, 1), (26, 1))):
        L.prop(f"shack_n{i + 1}", "ash_shack", x, y, 4, 3)
    for i, x in enumerate((1, 5, 9, 13, 17, 21, 25)):
        L.prop(f"shack_s{i + 1}", "ash_shack", x, 16, 4, 3)
    L.prop("shack_w1", "ash_shack", 1, 5, 4, 3); L.prop("shack_w2", "ash_shack", 1, 10, 4, 3)
    L.prop("shack_e1", "ash_shack", 27, 4, 4, 3); L.prop("shack_e2", "ash_shack", 27, 13, 4, 3)
    L.rect(14, 0, 17, 3, ".", "c")                      # the alley north toward the Vats
    L.rect(14, 0, 17, 0, "#"); L.rect(15, 0, 16, 0, ".", "c")
    L.rect(27, 8, 31, 11, ".", "c")                     # the alley east toward the Grate
    L.rect(31, 8, 31, 8, "#"); L.rect(31, 11, 31, 11, "#")
    # the cages on their platforms, the Proctor's titration station
    L.rect(6, 5, 15, 7, ground="f")
    L.prop("cages_1", "cage_platform", 7, 5, 3, 2); L.prop("cages_2", "cage_platform", 11, 5, 3, 2)
    # the Toll-Pen: a fenced pen east of the cages, its chained gate on the south side
    L.rect(18, 4, 24, 8, "#"); L.rect(19, 5, 23, 7, ".", "m")
    L.prop("pen_fence_n1", "pen_fence", 18, 4, 3, 1); L.prop("pen_fence_n2", "pen_fence", 21, 4, 3, 1)
    L.prop("pen_gate", "toll_pen_gate", 20, 8, 2, 1)
    L.prop("pen_fence_e", "pen_fence", 22, 8, 3, 1)
    L.rect(18, 5, 18, 7, "#"); L.rect(24, 5, 24, 7, "#")
    L.prop("lamp", "violet_lamp_post", 17, 12)
    L.prop("lamp_light", "violet_pool", 15, 12, 2, 2, layer="ground")
    L.prop("crates", "crate_stack", 6, 13, 2, 1)
    L.prop("ash_1", "ash_drift", 8, 10, 3, 2, layer="ground")
    L.rect(0, 0, 31, 19, zone="d"); L.rect(26, 8, 31, 11, zone="s")
    L.anchor("spawn", 29, 10, "arrival from the world map (the alley from the Grate)")
    L.anchor("exit_alley", 31, 10, "exit `world_map` (was `east`): the alley east, World map: the Ash-Crawl")
    L.anchor("exit_north", 15, 0, "exit (new, optional): the alley north toward the Vats, World map")
    L.anchor("cages", 12, 9, "encounter `cages` (was `site_b`): Proctor Quell's titration cages")
    L.anchor("toll_pen", 21, 9, "point `toll_pen` (was `altar`): the Seventh Ward Toll-Pen's chained gate")
    L.anchor("rubbings", 8, 8, "point `rubbings` (was `ruin_lore`): the cage-seal rubbings")
    return L.entry("sunk_ward_crossroads", "Sunk Ward: the Ash-Crawl Crossroads",
                   {"s": {"id": "alley_mouth", "kind": "safe"}, "d": {"id": "culling_yard", "kind": "danger"}},
                   {k: v for k, v in GROUND.items() if k in "scfm"}, ["sunk_ward"])


def corvos_gate():
    """Node 1d: the Third Ward gate, where Bailiff Corvo keeps a bribable post (the side job; the loud answer is a
    fight).

    Landmark (take A): a warm crumbling brick wall with a breach, the guard booth and its lantern, the striped
    barrier and the toll drum. The road from the Grate comes up from the south to the barrier in the wall; Corvo
    stands at his booth; the breach west is the loud way through; beyond the wall, a strip of Third Ward street."""
    L = Layout(32, 20, "c")
    L.border()
    L.rect(1, 7, 30, 8, "#")                            # the Third Ward wall
    for i, x in enumerate((9, 19, 23, 27)):
        L.prop(f"wall_{i + 1}", "brick_wall_run", x, 7, 4, 2)
    L.rect(5, 7, 6, 8, ".", "m")                        # the breach
    L.prop("breach_w", "brick_breach", 3, 7, 2, 2); L.prop("breach_e", "brick_breach", 7, 7, 2, 2)
    L.props[-1]["flip"] = True
    L.rect(14, 7, 17, 7, ".", "c")                      # the gateway; the barrier closes its south side
    L.prop("barrier", "striped_barrier", 14, 8, 3, 1); L.prop("gatepost", "gatepost", 17, 8)
    L.rect(13, 7, 13, 8, "#"); L.rect(18, 7, 18, 8, "#")
    L.prop("booth", "toll_booth", 19, 10, 2, 2)
    L.prop("drum", "toll_drum", 12, 10)
    L.prop("lantern", "lantern_pole", 21, 12)
    L.rect(1, 9, 30, 18, ground="c"); L.rect(1, 9, 12, 18, ground="m"); L.rect(23, 9, 30, 18, ground="m")
    L.rect(13, 9, 18, 18, ground="f")                   # the paved road up to the barrier
    L.rect(1, 1, 30, 6, ground="c")
    L.prop("ward_1", "tenement_row", 1, 1, 6, 3); L.prop("ward_2", "tenement_row", 22, 1, 6, 3)
    L.rect(1, 1, 6, 3, "#"); L.rect(22, 1, 27, 3, "#")
    L.prop("rubble", "rubble_wall", 2, 10, 4, 1)
    L.rect(15, 19, 16, 19, ".", "f")
    L.rect(0, 0, 31, 19, zone="s"); L.rect(1, 1, 30, 6, zone="d")
    L.anchor("spawn", 15, 17, "arrival from the world map (the Third Ward road from the Grate)")
    L.anchor("exit_road", 15, 19, "exit `world_map` (was `east`): the road south, World map: the Third Ward road")
    L.anchor("corvo", 18, 12, "npc `corvo` (was `camp_npc`): Bailiff Corvo at his booth")
    L.anchor("breach", 5, 9, "the breach west (the loud way through; a point or the job's fight start)")
    L.anchor("ward_street", 15, 4, "encounter start for the loud answer (`enc_job_corvo_gate`), beyond the wall")
    return L.entry("sunk_ward_corvos_gate", "Sunk Ward: Corvo's Gate",
                   {"s": {"id": "toll_road", "kind": "safe"}, "d": {"id": "third_ward", "kind": "danger"}},
                   {k: v for k, v in GROUND.items() if k in "cfm"}, ["sunk_ward"])


def fourth_kiln():
    """Node 04: the bone-mill that mortars the Inquisition's watch-posts out of uncremated Spent (M3, Overseer
    Grendel; Dunstan joins).

    Landmark (take B): a soot-black brick kiln with three glowing chimney tops and a furnace mouth, a conveyor chute up
    its side, a bone-meal heap, a brass tank and pipes on a cobbled apron. The kiln fills the north; the mill floor
    in front of the furnace is the fight; the road from Junction 7 arrives from the south, the climb toward the
    Crucible leaves east."""
    L = Layout(34, 22, "s")
    L.border()
    L.rect(9, 1, 24, 7, "#")
    L.prop("kiln", "fourth_kiln", 10, 1, 14, 6)
    L.rect(16, 7, 18, 7, "#"); L.prop("furnace", "furnace_mouth", 16, 7, 3, 1)
    L.rect(6, 3, 7, 7, "#"); L.prop("chute", "conveyor_chute", 6, 4, 2, 3)
    L.rect(26, 2, 29, 5, "#"); L.prop("tank", "brass_tank", 26, 2, 2, 2); L.prop("tank_2", "brass_tank", 28, 2, 2, 2)
    L.prop("pipes", "pipe_run", 26, 4, 2, 2); L.prop("pipes_2", "pipe_run", 28, 4, 2, 2)
    L.rect(8, 8, 25, 13, ground="c")                    # the cobbled apron, the mill floor
    L.rect(14, 8, 20, 10, ground="t")                   # ash and bone dust before the furnace mouth
    L.prop("heap_1", "bone_meal_heap", 5, 8, 2, 2, layer="ground")
    L.prop("heap_2", "bone_meal_heap", 10, 9, 2, 2, layer="ground")
    L.prop("carts", "crate_stack", 22, 11, 2, 1)
    L.prop("carts_2", "crate_stack", 9, 13, 2, 1)
    L.prop("wedges", "wedge_pile", 24, 9, 1, 1, layer="ground")
    L.rect(1, 15, 32, 20, ground="c"); L.rect(15, 14, 18, 20, ground="f")
    L.rect(1, 14, 12, 14, "#"); L.rect(21, 14, 32, 14, "#")
    L.prop("mill_wall_w", "rubble_wall", 2, 14, 4, 1); L.prop("mill_wall_w2", "rubble_wall", 7, 14, 4, 1)
    L.prop("mill_wall_e", "rubble_wall", 22, 14, 4, 1); L.prop("mill_wall_e2", "rubble_wall", 27, 14, 4, 1)
    L.rect(16, 21, 17, 21, ".", "f"); L.rect(33, 10, 33, 11, ".", "c")
    L.rect(0, 0, 33, 21, zone="d"); L.rect(1, 15, 32, 20, zone="s")
    L.anchor("spawn", 16, 19, "arrival from the world map (the road from Junction 7)")
    L.anchor("exit_road", 16, 21, "exit `world_map` (was `east`): the road south, World map: the Fourth Kiln")
    L.anchor("exit_climb", 33, 10, "exit (new, optional): the climb east toward the Crucible, World map")
    L.anchor("mill", 17, 10, "encounter `bone_mill` (was `site_b`): Overseer Grendel on the mill floor")
    L.anchor("skerrit", 6, 17, "npc `skerrit` (was `camp_npc`): Skerrit the bone-wright, after M3")
    L.anchor("wedges", 23, 9, "point `wedge_pile` (was `altar`): the black-iron wedges")
    return L.entry("sunk_ward_fourth_kiln", "Sunk Ward: the Fourth Kiln",
                   {"s": {"id": "kiln_road", "kind": "safe"}, "d": {"id": "mill_floor", "kind": "danger"}},
                   {k: v for k, v in GROUND.items() if k in "scft"}, ["sunk_ward"])


def tallow_vats():
    """Node 05: the Guild's rendering vats and still-rooms, where Tallow Dredgers boil the district's dead (M4,
    Guild-Master Orlov; Elyse joins).

    Landmark (take A, tinted): three big vats with glowing tallow tops, catwalks, a shed, pipes, barrels and spilt
    tallow. The vats stand in a tallow-soaked yard ringed by iron catwalk; the still-room shed north-east; Orlov's
    store south-east; a canal down the west side, where the siphon shortcut from the Grate comes in by culvert; the
    alley from the Crossroads arrives from the south."""
    L = Layout(34, 22, "t")
    L.border()
    L.rect(1, 1, 3, 20, "~", "w")                       # the canal
    L.rect(1, 10, 3, 10, ".", "i"); L.rect(0, 10, 0, 10, ".", "i")    # the culvert catwalk over it
    L.rect(4, 1, 5, 20, ground="c")                     # the canal towpath
    L.rect(10, 2, 23, 14, ground="i")                   # the catwalk ring
    L.rect(11, 3, 22, 13, ground="t")
    for i, (x, y) in enumerate(((12, 3), (18, 3), (15, 9))):
        L.prop(f"vat_{i + 1}", "tallow_vat", x, y, 3, 3)
    L.prop("stair", "catwalk_stair", 22, 3, 1, 3)
    L.prop("spill_1", "tallow_spill", 11, 7, 3, 2, layer="ground"); L.prop("spill_2", "tallow_spill", 19, 11, 3, 2, layer="ground")
    L.rect(25, 1, 32, 5, "#"); L.prop("still_shed", "still_room", 26, 1, 6, 4)
    L.rect(25, 15, 32, 18, "#"); L.prop("store", "guild_store", 26, 15, 6, 3)
    L.prop("barrels_1", "barrel_stack", 24, 9, 2, 1); L.prop("barrels_2", "barrel_stack", 27, 9, 2, 1)
    L.prop("barrels_3", "barrel_stack", 7, 15, 2, 1); L.prop("pipes", "pipe_run", 7, 3, 2, 2)
    L.rect(1, 17, 3, 18, "~", "w")
    L.rect(6, 17, 24, 20, ground="c"); L.rect(15, 15, 18, 21, ground="f")
    L.rect(16, 21, 17, 21, ".", "f")
    L.rect(0, 0, 33, 21, zone="d"); L.rect(6, 18, 24, 20, zone="s"); L.rect(0, 9, 5, 11, zone="s")
    L.anchor("spawn", 16, 19, "arrival from the world map (the alley from the Crossroads)")
    L.anchor("exit_alley", 16, 21, "exit `world_map` (was `east`): the alley south, World map: the Tallow Vats")
    L.anchor("siphon", 0, 10, "exit (new): the siphon culvert, World map (the Grate's shortcut; requires sunk_ward.lock_ciphers_done)")
    L.anchor("siphon_in", 4, 10, "arrival by the siphon shortcut (the towpath at the culvert)")
    L.anchor("vats", 17, 7, "encounter `vats` (was `site_b`): Guild-Master Orlov among the vats")
    L.anchor("stores", 28, 14, "chest `orlov_stores` (was `ruin_chest`): Orlov's store, after M4")
    L.anchor("still", 28, 6, "point `counter_still` (was `altar`): Elyse's counter-still at the still-room")
    return L.entry("sunk_ward_tallow_vats", "Sunk Ward: the Tallow Vats",
                   {"s": {"id": "vats_gate", "kind": "safe"}, "d": {"id": "rendering_yard", "kind": "danger"}},
                   {k: v for k, v in GROUND.items() if k in "cfitw"}, ["sunk_ward"])


def crucible_watchpost():
    """Node 06: the fortified post above the Slag Gate, with the winch that lifts it (M5: one founder holds the winch,
    three get out; the Overlook variant).

    Landmark (take B): a fortress tower with a row of red-orange slit windows and rust-trimmed battlements, the winch
    drum and chain, the crane arm, a violet glint in the tower window. The road climbs from the gorge mouth (south)
    to the gatehouse in the curtain wall; inside, the yard, the tower west and the winch house on the north wall
    over the Slag Gate; the Crag-Sight Overlook is a ledge on the west cliff; a postern leads north up the gorge."""
    L = Layout(32, 22, "m")
    L.border()
    L.rect(1, 1, 4, 20, "#"); L.rect(27, 1, 30, 20, "#")          # the gorge cliffs
    for i, y in enumerate((2, 8, 14)):
        L.prop(f"cliff_w{i + 1}", "cliff_face", 1, y, 4, 1); L.prop(f"cliff_e{i + 1}", "cliff_face", 27, y, 4, 1)
    L.rect(1, 3, 3, 7, ".", "s")                        # the Crag-Sight Overlook ledge in the west cliff
    L.rect(4, 6, 4, 6, ".", "s")
    L.rect(5, 1, 26, 2, "#"); L.rect(5, 9, 26, 10, "#"); L.rect(5, 3, 6, 8, "#"); L.rect(25, 3, 26, 8, "#")
    L.prop("winch", "winch_drum", 15, 1, 2, 2)
    L.prop("north_wall_w", "watch_wall", 7, 1, 4, 2); L.prop("north_wall_e", "watch_wall", 17, 1, 4, 2)
    L.rect(23, 1, 24, 2, ".", "f"); L.rect(23, 0, 23, 0, ".", "f")   # the postern up the gorge
    L.prop("crane", "crane_arm", 21, 1)
    L.rect(5, 6, 6, 6, ".", "f")                        # the sally door from the yard out to the Overlook ledge
    L.rect(7, 3, 11, 5, "#"); L.prop("tower", "slit_window_tower", 7, 3, 5, 3)
    L.prop("curtain_w", "watch_wall", 10, 9, 4, 2); L.prop("curtain_e", "watch_wall", 18, 9, 4, 2)
    L.rect(15, 9, 16, 10, ".", "f")                     # the gatehouse passage
    L.prop("gate_w", "gatepost", 14, 10); L.prop("gate_e", "gatepost", 17, 10)
    L.rect(14, 9, 14, 10, "#"); L.rect(17, 9, 17, 10, "#")
    L.rect(7, 3, 24, 8, ground="f"); L.rect(7, 3, 11, 6, ground="f")
    L.prop("braziers", "brazier", 20, 5)
    L.rect(5, 11, 26, 20, ground="m"); L.rect(14, 11, 17, 21, ground="c")   # the gorge floor and the road up
    L.prop("boulder_1", "boulder", 8, 14, 2, 2); L.prop("boulder_2", "boulder", 22, 16, 2, 2)
    L.rect(15, 21, 16, 21, ".", "c")
    L.rect(0, 0, 31, 21, zone="d"); L.rect(5, 17, 26, 20, zone="s"); L.rect(1, 3, 4, 7, zone="s")
    L.anchor("spawn", 15, 19, "arrival from the world map (the road up from the gorge mouth)")
    L.anchor("exit_road", 15, 21, "exit `world_map` (was `east`): the road south, World map: the Crucible")
    L.anchor("postern", 23, 0, "exit (new, optional): the postern up the high gorge to the Slag Gate, World map")
    L.anchor("crucible", 15, 6, "encounter `crucible` (was `site_b`): the yard before the winch house (M5)")
    L.anchor("crucible_overlook", 15, 12, "encounter `crucible_overlook` (was `site_a`): before the gatehouse, the Overlook variant")
    L.anchor("winch_post", 16, 3, "the winch (the founder who holds it; a point or a set piece)")
    L.anchor("overlook", 2, 4, "point `overlook` (was `altar`): the Crag-Sight Overlook, after the Interlude")
    return L.entry("sunk_ward_crucible", "Sunk Ward: Crucible Watchpost",
                   {"s": {"id": "gorge_mouth", "kind": "safe"}, "d": {"id": "watchpost", "kind": "danger"}},
                   {k: v for k, v in GROUND.items() if k in "scfm"}, ["sunk_ward"])


def slag_gate():
    """Node 07 (optional: it has no map today, only exit buttons): the gate out of the Sunk Ward, beneath the Crucible.

    Landmark (take B): a shut portcullis set into a fortified cliff wall, red banners, torches, a short paved road
    and a glowing L-shaped slag channel. The high gorge comes in from the south; the portcullis in the north cliff is
    the way to Upper Cinderreach once the winch lifts it."""
    L = Layout(24, 14, "m")
    L.border()
    L.rect(1, 1, 22, 3, "#"); L.rect(1, 4, 3, 12, "#"); L.rect(20, 4, 22, 12, "#")
    L.prop("bastion_w", "cliff_bastion", 3, 1, 5, 3); L.prop("bastion_e", "cliff_bastion", 16, 1, 5, 3)
    L.props[-1]["flip"] = True
    L.rect(10, 1, 13, 3, "#"); L.prop("portcullis", "portcullis", 10, 2, 4, 2)
    L.rect(11, 0, 12, 1, ".", "f"); L.rect(10, 1, 10, 1, "#"); L.rect(13, 1, 13, 1, "#")
    L.rows[2][11] = L.rows[2][12] = "#"
    L.prop("torch_w", "watch_torch", 9, 3); L.prop("torch_e", "watch_torch", 14, 3)
    L.prop("banner_w", "banner_pole", 8, 3); L.prop("banner_e", "banner_pole", 15, 3)
    L.rect(10, 4, 13, 13, ground="f")                   # the short paved road
    L.prop("slag", "slag_channel", 6, 4, 4, 8, layer="ground")   # one L-shaped decal: down the west side, then east
    L.rect(11, 13, 12, 13, ".", "f")
    L.rect(0, 0, 23, 13, zone="d")
    L.anchor("spawn", 11, 11, "arrival from the world map (the high gorge from the Crucible)")
    L.anchor("exit_gorge", 11, 13, "exit: back down the high gorge, World map")
    L.anchor("portcullis", 11, 4, "the portcullis: the way to Upper Cinderreach once the winch has lifted it (M5)")
    return L.entry("sunk_ward_slag_gate", "Sunk Ward: the Slag Gate (optional)",
                   {"d": {"id": "slag_gate", "kind": "danger"}},
                   {k: v for k, v in GROUND.items() if k in "fm"}, ["sunk_ward"])


PLACES = [(sluice_undercroft, "sluice_undercroft"), (drowned_runoff_grate, "drowned_runoff_grate"),
          (alleyway_junction_7, "alleyway_junction_7"), (ash_crawl_crossroads, "ash_crawl_crossroads"),
          (fourth_kiln, "fourth_kiln"), (tallow_vats, "tallow_vats"), (crucible_watchpost, "crucible_watchpost"),
          (slag_gate, "slag_gate"), (corvos_gate, "corvos_gate")]


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
    # every anchor reachable from the spawn on foot (orthogonal steps over open tiles)
    sp = next((a for a in e["anchors"] if a["id"] == "spawn"), None)
    if sp:
        seen_t, todo = {(sp["x"], sp["y"])}, [(sp["x"], sp["y"])]
        while todo:
            x, y = todo.pop()
            for nx, ny in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
                if 0 <= nx < w and 0 <= ny < h and rows[ny][nx] == "." and (nx, ny) not in seen_t:
                    seen_t.add((nx, ny)); todo.append((nx, ny))
        for a in e["anchors"]:
            if (a["x"], a["y"]) not in seen_t:
                errs.append(f"anchor {a['id']} cannot be reached from the spawn")
    return errs


def main():
    ok = True
    for fn, loc in PLACES:
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
