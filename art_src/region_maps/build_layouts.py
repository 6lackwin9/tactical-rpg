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

    def indoor(self, zone_char, x0, y0, x1, y1, door, ground, art, roof_h=None):
        """An enterable building (REVEAL1 draft): a wall ring round an open floor, one doorway, the floor its own
        zone, and the building's picture as a `roof` prop over the whole footprint. Outside, the roof hides the room;
        with the party inside its zone, the roof and front wall fade (REGIONGROUND1_SCENE: indoor reveal)."""
        self.rect(x0, y0, x1, y1, "#")
        self.rect(x0 + 1, y0 + 1, x1 - 1, y1 - 1, ".", ground, zone_char)
        self.rect(door[0], door[1], door[0], door[1], ".", ground, zone_char)
        self.props.append({"id": f"roof_{art}", "art": art, "x": x0, "y": y0, "w": x1 - x0 + 1, "h": y1 - y0 + 1,
                           "layer": "roof", "variant": -1, "flip": False, "roof_of": zone_char})

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


def fill_block(L, name, x0, y0, x1, y1, art="tenement_block", pw=6, ph=5):
    """A city block: all wall, packed with as many `art` buildings (pw x ph) as fit; the rest stays plain wall."""
    L.rect(x0, y0, x1, y1, "#")
    n = 0
    for by in range(y0, y1 - ph + 2, ph):
        for bx in range(x0, x1 - pw + 2, pw):
            n += 1
            L.prop(f"{name}_{n}", art, bx, by, pw, ph)


def alleyway_junction_7():
    """Junction 7 at mission-ground size (Tushar 2026-11-14: a sense of size; larger open maps with dedicated zones,
    danger areas to go through and safe meeting areas, against small rooms joined by corridors).

    The rooms-and-corridors feel inside one map: safe rooms (the bridge landing where the party arrives, the doss-house
    court where it can meet a contact) joined by danger streets and alleys. A danger zone bound to an encounter list
    (`zone_bindings`) starts a fight every 12 to 20 steps walked inside it (rpgkit_world explore.rs); a Watch patrol walks
    the main street as a roamer. The mission site, Vane's barricaded yard where the alleys cross, is at the far west end;
    the road north toward the Kiln leaves from the yard. About 56 x 36 tiles: three screens at the default zoom."""
    L = Layout(56, 36, "m")
    L.border()
    L.rect(1, 1, 54, 34, "#")
    # the streets and rooms, carved out of the solid ward
    L.rect(46, 13, 54, 22, ".", "f")                    # the bridge landing (safe)
    L.rect(55, 17, 55, 18, ".", "f")                    # the bridge exit
    L.rect(22, 16, 45, 19, ".", "c")                    # the main street (danger: the Watch patrol, street fights)
    L.rect(10, 12, 21, 23, ".", "c"); L.rect(12, 14, 19, 21, ground="f")   # the Junction 7 yard (the mission)
    L.rect(15, 0, 16, 11, ".", "c")                     # the road north toward the Kiln
    L.rect(22, 5, 44, 7, ".", "m")                      # the back alley north (danger)
    L.rect(29, 8, 30, 15, ".", "m"); L.rect(42, 8, 43, 15, ".", "m")   # its two lanes down to the main street
    L.rect(16, 5, 21, 6, ".", "m")                      # the back alley's west end, onto the north road
    L.rect(33, 20, 34, 23, ".", "c")                    # the lane down to the doss-house court
    L.rect(29, 24, 39, 28, ".", "f")                    # the doss-house court (safe)
    L.indoor("h", 29, 29, 39, 33, (34, 29), "f", "doss_house")      # the doss house: the meeting room inside
    L.rect(18, 24, 19, 25, ".", "m"); L.rect(16, 26, 21, 28, ".", "m")   # the drain run south of the yard (danger)
    L.rect(22, 27, 28, 28, ".", "m")                    # it reaches the court's west door
    # buildings fill the solid ward between the streets
    fill_block(L, "tenement_n", 22, 8, 28, 15); fill_block(L, "tenement_m", 31, 8, 41, 15)
    fill_block(L, "tenement_ne", 45, 1, 54, 12); fill_block(L, "tenement_nw", 1, 1, 14, 11)
    fill_block(L, "tenement_s", 22, 20, 32, 23); fill_block(L, "tenement_se", 40, 23, 54, 34)
    fill_block(L, "tenement_sw", 1, 29, 28, 34); fill_block(L, "tenement_w", 1, 12, 9, 23)
    fill_block(L, "tenement_ss", 35, 20, 45, 22, art="tenement_row", pw=6, ph=3)
    L.rect(22, 1, 43, 4, "#"); fill_block(L, "tenement_far", 22, 1, 43, 4, art="tenement_row", pw=6, ph=3)
    # Vane's yard: a barricade across most of the north road's mouth, the sealed grates, the Watch armory south of it
    L.rect(16, 12, 18, 12, "#"); L.prop("barricade_n", "spiked_barricade", 16, 12, 3, 1)
    L.prop("grate_yard", "street_grate", 12, 19, 2, 2, layer="ground")
    L.prop("grate_street", "street_grate", 36, 17, 2, 2, layer="ground")
    L.indoor("a", 10, 24, 15, 28, (13, 24), "f", "watch_armory")   # the old Watch armory: the cache inside
    for i, (x, y) in enumerate(((21, 16), (21, 19), (45, 15), (45, 20))):
        L.rows[y][x] = "#"; L.prop(f"torch_{i + 1}", "watch_torch", x, y)
    L.prop("pennant", "rust_pennant", 9, 17); L.rows[17][9] = "#"
    # the doss-house court: a brazier, crates, its house along the south wall
    L.prop("court_brazier", "brazier", 31, 26); L.rows[26][31] = "#"
    L.prop("court_crates", "crate_stack", 37, 25, 2, 1); L.rect(37, 25, 38, 25, "#")
    L.prop("landing_crates", "crate_stack", 50, 13, 2, 1); L.rect(50, 13, 51, 13, "#")
    # zones: two safe rooms, three danger stretches, the yard
    L.rect(0, 0, 55, 35, zone="-")
    L.rect(46, 13, 55, 22, zone="s"); L.rect(29, 24, 39, 28, zone="c")
    L.rect(22, 16, 45, 19, zone="d"); L.rect(16, 0, 44, 15, zone="b"); L.rect(16, 24, 28, 28, zone="b")
    L.rect(10, 12, 21, 23, zone="y"); L.rect(15, 0, 16, 11, zone="b")
    L.rect(33, 20, 34, 23, zone="c")
    L.rect(11, 25, 14, 27, zone="a"); L.rect(13, 24, 13, 24, zone="a")
    L.rect(30, 30, 38, 32, zone="h"); L.rect(34, 29, 34, 29, zone="h")
    # the Watch patrol walks the main street, east and back
    L.paths = {"watch_patrol": [[x, 18] for x in range(24, 45)] + [[x, 17] for x in range(44, 23, -1)]}
    L.anchor("spawn", 51, 17, "arrival from the world map (the bridge landing from the Grate)")
    L.anchor("exit_bridge", 55, 17, "exit `world_map`: the bridge east, World map: the Seventh Ward")
    L.anchor("exit_north", 15, 0, "exit (optional): the road north toward the Kiln, World map")
    L.anchor("sweep", 16, 18, "encounter `sweep`: Captain Vane's sweep in the yard (the mission)")
    L.anchor("cache", 12, 26, "point `bellows_cache`: inside the old Watch armory, after M1 (indoor)")
    L.anchor("contact", 34, 31, "inside the doss house: a contact or a rest (safe, indoor)")
    L.anchor("patrol_start", 24, 18, "roamer `watch_patrol` (path `watch_patrol`): the Watch on the main street")
    e = L.entry("sunk_ward_junction_7", "Sunk Ward: Alleyway Junction 7",
                {"s": {"id": "bridge_landing", "kind": "safe"}, "c": {"id": "doss_court", "kind": "safe"},
                 "d": {"id": "main_street", "kind": "danger"}, "b": {"id": "back_alleys", "kind": "danger"},
                 "y": {"id": "yard", "kind": "danger"},
                 "a": {"id": "armory", "kind": "safe", "indoor": True},
                 "h": {"id": "doss_house", "kind": "safe", "indoor": True}},
                {k: v for k, v in GROUND.items() if k in "scfm"}, ["sunk_ward"])
    e["paths"] = L.paths
    e["draft_zone_bindings"] = {"main_street": "street fights (Watch pairs, enforcers); the roamer as well",
                                "back_alleys": "scavengers and stray Watch; 12-20 steps a fight (the kit's default)",
                                "yard": "no binding: the yard's fight is the `sweep` encounter marker"}
    return e


def path_through(corners):
    """An orthogonal patrol path through the corner points, one tile a step (roamers walk `paths`)."""
    out = [list(corners[0])]
    for (x0, y0), (x1, y1) in zip(corners, corners[1:]):
        dx = (x1 > x0) - (x1 < x0); dy = (y1 > y0) - (y1 < y0)
        x, y = x0, y0
        while (x, y) != (x1, y1):
            x += dx if x != x1 else 0
            y += dy if x == x1 and y != y1 else 0
            out.append([x, y])
    return out


def mission_entry(L, tid, name, legend, grounds, bindings):
    e = L.entry(tid, name, legend, {k: v for k, v in GROUND.items() if k in grounds}, ["sunk_ward"])
    e["paths"] = getattr(L, "paths", {})
    e["draft_zone_bindings"] = bindings
    return e


def ash_crawl_crossroads():
    """Node 03 at mission-ground size (Tushar 2026-11-14: "Go large for missions"): an open-air culling station where
    the slum alleys meet, the Seventh Ward Toll-Pen beside the cages (M2, Proctor Quell's titration cages).

    The party arrives at the alley mouth from the Grate (east, safe). The slum lane west is danger (the Proctor's
    wardens patrol it), with a loop of back alleys north of it; south of the lane, the safe rag-market court and the
    ragpicker's hovel (enterable: a contact). The culling plaza at the far west is the mission ground: the cage
    platforms, the fenced Toll-Pen and its chained gate, the cold violet lamp. The alley toward the Vats leaves the
    plaza north."""
    L = Layout(56, 36, "s")
    L.border(); L.rect(1, 1, 54, 34, "#")
    L.rect(46, 14, 54, 21, ".", "c"); L.rect(55, 17, 55, 18, ".", "c")      # the alley mouth (arrival, safe)
    L.rect(26, 16, 45, 19, ".", "s")                                         # the slum lane (danger)
    L.rect(5, 7, 25, 27, ".", "s"); L.rect(8, 8, 22, 13, ground="f")        # the culling plaza (the mission)
    L.rect(15, 0, 16, 6, ".", "c")                                           # the alley north toward the Vats
    L.rect(26, 6, 44, 8, ".", "m"); L.rect(30, 9, 31, 15, ".", "m"); L.rect(41, 9, 42, 15, ".", "m")   # back alleys
    L.rect(34, 20, 35, 23, ".", "c"); L.rect(29, 24, 41, 28, ".", "f")      # the lane down, the rag-market court
    L.rect(26, 27, 28, 28, ".", "m")                                         # the drain run from the plaza
    for b in ((1, 1, 14, 6), (17, 1, 54, 5), (26, 9, 29, 15), (32, 9, 40, 15), (43, 9, 54, 13), (1, 7, 4, 27),
              (26, 20, 33, 23), (36, 20, 45, 23), (1, 28, 25, 34), (42, 24, 54, 34)):
        fill_block(L, f"shacks_{b[0]}_{b[1]}", *b, art="ash_shack", pw=4, ph=3)
    # the plaza: three cage platforms, the Toll-Pen, the lamp
    for i, x in enumerate((9, 13, 17)):
        L.prop(f"cages_{i + 1}", "cage_platform", x, 9, 3, 2)
    L.rect(8, 16, 14, 20, "#"); L.rect(9, 17, 13, 19, ".", "m")
    L.prop("pen_fence_n1", "pen_fence", 8, 16, 3, 1); L.prop("pen_fence_n2", "pen_fence", 11, 16, 3, 1)
    L.prop("pen_gate", "toll_pen_gate", 10, 20, 2, 1); L.prop("pen_fence_s", "pen_fence", 12, 20, 3, 1)
    L.rows[22][20] = "#"; L.prop("lamp", "violet_lamp_post", 20, 22)
    L.prop("lamp_light", "violet_pool", 18, 22, 2, 2, layer="ground")
    L.prop("ash_1", "ash_drift", 6, 24, 3, 2, layer="ground"); L.prop("ash_2", "ash_drift", 33, 17, 3, 2, layer="ground")
    L.rect(22, 25, 23, 25, "#"); L.prop("crates", "crate_stack", 22, 25, 2, 1)
    L.rect(51, 14, 52, 14, "#"); L.prop("mouth_crates", "crate_stack", 51, 14, 2, 1)
    L.rect(0, 0, 55, 35, zone="-")
    L.rect(46, 14, 55, 21, zone="s"); L.rect(26, 16, 45, 19, zone="d"); L.rect(15, 0, 16, 6, zone="b")
    L.rect(17, 6, 44, 15, zone="b"); L.rect(26, 27, 28, 28, zone="b"); L.rect(5, 7, 25, 27, zone="y")
    L.rect(29, 24, 41, 28, zone="c"); L.rect(34, 20, 35, 23, zone="c")
    L.indoor("h", 30, 29, 40, 33, (35, 29), "f", "rag_hovel")              # the ragpicker's hovel (a contact)
    L.paths = {"warden_patrol": path_through([(28, 18), (44, 18), (44, 17), (28, 17), (28, 18)])[:-1]}
    L.anchor("spawn", 51, 17, "arrival from the world map (the alley mouth from the Grate)")
    L.anchor("exit_alley", 55, 17, "exit `world_map` (was `east`): the alley east, World map: the Ash-Crawl")
    L.anchor("exit_north", 15, 0, "exit (optional): the alley north toward the Vats, World map")
    L.anchor("cages", 15, 12, "encounter `cages` (was `site_b`): Proctor Quell's titration cages (the mission)")
    L.anchor("toll_pen", 10, 21, "point `toll_pen` (was `altar`): the Seventh Ward Toll-Pen's chained gate")
    L.anchor("rubbings", 12, 11, "point `rubbings` (was `ruin_lore`): the cage-seal rubbings, after M2")
    L.anchor("contact", 35, 31, "inside the ragpicker's hovel: a contact or a rest (safe, indoor)")
    L.anchor("patrol_start", 28, 18, "roamer `warden_patrol`: the Proctor's wardens on the slum lane")
    return mission_entry(L, "sunk_ward_crossroads", "Sunk Ward: the Ash-Crawl Crossroads",
                         {"s": {"id": "alley_mouth", "kind": "safe"}, "d": {"id": "slum_lane", "kind": "danger"},
                          "b": {"id": "back_alleys", "kind": "danger"}, "y": {"id": "culling_plaza", "kind": "danger"},
                          "c": {"id": "rag_market", "kind": "safe"},
                          "h": {"id": "rag_hovel", "kind": "safe", "indoor": True}}, "scfm",
                         {"slum_lane": "the Proctor's wardens; the roamer as well",
                          "back_alleys": "cage-runners and scavengers; 12-20 steps a fight (the kit's default)",
                          "culling_plaza": "no binding: the plaza's fight is the `cages` encounter marker"})


def fourth_kiln():
    """Node 04 at mission-ground size: the bone-mill that mortars the Inquisition's watch-posts out of uncremated Spent
    (M3, Overseer Grendel; Dunstan joins).

    The party arrives in the gate yard (south, safe), beside Skerrit's workshop (enterable: Skerrit after M3). The cart
    track north is danger (bone-cart haulers patrol it); the ash yards west, with their bone-meal heaps, are danger too.
    The mill floor before the furnace mouth, under the 14-wide kiln, is the mission ground; the brass tanks and the
    conveyor chute flank the kiln. A lane east from the mill floor climbs toward the Crucible."""
    L = Layout(56, 36, "s")
    L.border(); L.rect(1, 1, 54, 34, "#")
    L.rect(17, 27, 38, 33, ".", "c"); L.rect(27, 34, 28, 35, ".", "f")      # the gate yard (arrival, safe)
    L.rect(26, 17, 29, 26, ".", "c")                                         # the cart track (danger)
    L.rect(12, 8, 43, 16, ".", "c"); L.rect(23, 8, 32, 10, ground="t")      # the mill floor (the mission)
    L.rect(3, 8, 11, 28, ".", "s"); L.rect(12, 27, 16, 28, ".", "s")        # the ash yards (danger)
    L.rect(44, 11, 54, 12, ".", "c"); L.rect(55, 11, 55, 12, ".", "c")      # the lane east, the climb
    L.rect(21, 1, 34, 7, "#"); L.prop("kiln", "fourth_kiln", 21, 1, 14, 6)
    L.prop("furnace", "furnace_mouth", 26, 7, 3, 1)
    L.rect(17, 2, 20, 7, "#"); L.prop("chute", "conveyor_chute", 18, 4, 2, 3)
    L.rect(35, 2, 40, 7, "#")
    L.prop("tank_1", "brass_tank", 35, 2, 2, 2); L.prop("tank_2", "brass_tank", 37, 2, 2, 2)
    L.prop("pipes_1", "pipe_run", 35, 4, 2, 2); L.prop("pipes_2", "pipe_run", 37, 4, 2, 2)
    for b in ((1, 1, 16, 7), (41, 1, 54, 7), (44, 13, 54, 26), (12, 17, 25, 26), (30, 17, 43, 26), (39, 27, 54, 34),
              (1, 29, 6, 34)):
        fill_block(L, f"row_{b[0]}_{b[1]}", *b, art="tenement_row", pw=6, ph=3)
    for i, (x, y) in enumerate(((4, 10), (7, 15), (4, 21))):
        L.prop(f"heap_{i + 1}", "bone_meal_heap", x, y, 2, 2, layer="ground")
    L.rect(14, 14, 15, 14, "#"); L.prop("carts_1", "crate_stack", 14, 14, 2, 1)
    L.rect(40, 15, 41, 15, "#"); L.prop("carts_2", "crate_stack", 40, 15, 2, 1)
    L.prop("wedges", "wedge_pile", 33, 9, 1, 1, layer="ground")
    L.rect(19, 28, 20, 28, "#"); L.prop("yard_crates", "crate_stack", 19, 28, 2, 1)
    L.rect(0, 0, 55, 35, zone="-")
    L.rect(17, 27, 38, 33, zone="s"); L.rect(27, 34, 28, 35, zone="s"); L.rect(26, 17, 29, 26, zone="d")
    L.rect(12, 8, 43, 16, zone="y"); L.rect(3, 8, 11, 28, zone="b"); L.rect(12, 27, 16, 28, zone="b")
    L.rect(44, 11, 55, 12, zone="b")
    L.indoor("h", 7, 29, 16, 34, (16, 31), "f", "bonewright_shop")         # Skerrit's workshop
    L.paths = {"hauler_patrol": path_through([(27, 25), (27, 18), (28, 18), (28, 25), (27, 25)])[:-1]}
    L.anchor("spawn", 27, 31, "arrival from the world map (the gate yard, the road from Junction 7)")
    L.anchor("exit_road", 27, 35, "exit `world_map` (was `east`): the road south, World map: the Fourth Kiln")
    L.anchor("exit_climb", 55, 11, "exit (optional): the lane east, the climb toward the Crucible, World map")
    L.anchor("mill", 27, 12, "encounter `bone_mill` (was `site_b`): Overseer Grendel on the mill floor (the mission)")
    L.anchor("skerrit", 11, 31, "npc `skerrit` (was `camp_npc`): Skerrit in his workshop, after M3 (indoor)")
    L.anchor("wedges", 32, 9, "point `wedge_pile` (was `altar`): the black-iron wedges by the furnace")
    L.anchor("patrol_start", 27, 25, "roamer `hauler_patrol`: bone-cart haulers on the cart track")
    return mission_entry(L, "sunk_ward_fourth_kiln", "Sunk Ward: the Fourth Kiln",
                         {"s": {"id": "gate_yard", "kind": "safe"}, "d": {"id": "cart_track", "kind": "danger"},
                          "b": {"id": "ash_yards", "kind": "danger"}, "y": {"id": "mill_floor", "kind": "danger"},
                          "h": {"id": "bonewright_shop", "kind": "safe", "indoor": True}}, "scft",
                         {"cart_track": "bone-cart haulers and kiln guards; the roamer as well",
                          "ash_yards": "Spent scavengers among the heaps; 12-20 steps a fight",
                          "mill_floor": "no binding: the floor's fight is the `bone_mill` encounter marker"})


def tallow_vats():
    """Node 05 at mission-ground size: the Guild's rendering vats and still-rooms, where Tallow Dredgers boil the
    district's dead (M4, Guild-Master Orlov; Elyse joins).

    The party arrives in the gate yard (south, safe) by the alley from the Crossroads, or at the siphon landing on the
    west canal (safe) by the Grate's culvert shortcut. The lane north into the vats is danger; so are the towpath and
    the barrel yard east. The vats yard, three vats in a catwalk ring that Tallow Dredgers patrol, is the mission
    ground. The still-room (Elyse's counter-still) and Orlov's store (the chest after M4) are enterable."""
    L = Layout(56, 36, "t")
    L.border(); L.rect(1, 1, 54, 34, "#")
    L.rect(1, 1, 3, 34, "~", "w"); L.rect(0, 17, 3, 17, ".", "i")          # the canal, the culvert catwalk
    L.rect(4, 1, 5, 34, ".", "c"); L.rect(6, 15, 10, 19, ".", "c")          # the towpath, the siphon landing
    L.rect(12, 3, 34, 20, ".", "i"); L.rect(13, 4, 33, 19, ground="t")      # the vats yard in its catwalk ring
    L.rect(6, 8, 11, 9, ".", "c")                                            # the towpath way into the yard
    L.rect(22, 21, 25, 28, ".", "c")                                         # the lane north (danger)
    L.rect(17, 29, 32, 33, ".", "c"); L.rect(24, 34, 25, 35, ".", "f")      # the gate yard (arrival, safe)
    L.rect(6, 31, 16, 32, ".", "c")                                          # the towpath to the gate yard
    L.rect(35, 9, 48, 21, ".", "s")                                          # the barrel yard (danger)
    for i, (x, y) in enumerate(((15, 6), (23, 6), (19, 13))):
        L.prop(f"vat_{i + 1}", "tallow_vat", x, y, 3, 3)
    L.prop("stair", "catwalk_stair", 30, 6, 1, 3)
    L.prop("spill_1", "tallow_spill", 14, 11, 3, 2, layer="ground"); L.prop("spill_2", "tallow_spill", 26, 15, 3, 2, layer="ground")
    L.prop("pipes", "pipe_run", 8, 3, 2, 2)
    for i, (x, y) in enumerate(((38, 12), (44, 12), (40, 17))):
        L.rect(x, y, x + 1, y, "#"); L.prop(f"barrels_{i + 1}", "barrel_stack", x, y, 2, 1)
    for b in ((6, 21, 21, 28), (26, 21, 36, 28), (37, 28, 54, 34), (49, 1, 54, 27)):
        fill_block(L, f"row_{b[0]}_{b[1]}", *b, art="tenement_row", pw=6, ph=3)
    L.rect(0, 0, 55, 35, zone="-")
    L.rect(17, 29, 32, 33, zone="s"); L.rect(24, 34, 25, 35, zone="s")
    L.rect(6, 15, 10, 19, zone="w"); L.rect(0, 17, 3, 17, zone="w")
    L.rect(22, 21, 25, 28, zone="d"); L.rect(12, 3, 34, 20, zone="y")
    L.rect(4, 1, 5, 34, zone="b"); L.rect(6, 8, 11, 9, zone="b"); L.rect(6, 31, 16, 32, zone="b")
    L.rect(35, 9, 48, 21, zone="b"); L.rect(4, 15, 5, 19, zone="w")
    L.indoor("h", 37, 2, 47, 8, (42, 8), "f", "still_room")                 # the still-room (Elyse's counter-still)
    L.indoor("o", 37, 22, 47, 27, (42, 22), "f", "guild_store")             # Orlov's store
    L.paths = {"dredger_patrol": path_through([(13, 4), (33, 4), (33, 19), (13, 19), (13, 5)])}
    L.anchor("spawn", 24, 31, "arrival from the world map (the gate yard, the alley from the Crossroads)")
    L.anchor("exit_alley", 24, 35, "exit `world_map` (was `east`): the alley south, World map: the Tallow Vats")
    L.anchor("siphon", 0, 17, "exit: the siphon culvert, World map (the Grate's shortcut; requires sunk_ward.lock_ciphers_done)")
    L.anchor("siphon_in", 7, 17, "arrival by the siphon shortcut (the landing, safe)")
    L.anchor("vats", 22, 11, "encounter `vats` (was `site_b`): Guild-Master Orlov among the vats (the mission)")
    L.anchor("still", 42, 5, "point `counter_still` (was `altar`): Elyse's counter-still, inside the still-room (indoor)")
    L.anchor("stores", 42, 25, "chest `orlov_stores` (was `ruin_chest`): inside Orlov's store, after M4 (indoor)")
    L.anchor("patrol_start", 13, 4, "roamer `dredger_patrol`: Tallow Dredgers round the catwalk ring")
    return mission_entry(L, "sunk_ward_tallow_vats", "Sunk Ward: the Tallow Vats",
                         {"s": {"id": "vats_gate", "kind": "safe"}, "w": {"id": "siphon_landing", "kind": "safe"},
                          "d": {"id": "vats_lane", "kind": "danger"}, "b": {"id": "towpath_and_yard", "kind": "danger"},
                          "y": {"id": "rendering_yard", "kind": "danger"},
                          "h": {"id": "still_room", "kind": "safe", "indoor": True},
                          "o": {"id": "guild_store", "kind": "safe", "indoor": True}}, "cfistw",
                         {"vats_lane": "Guild bruisers at the vats' gate",
                          "towpath_and_yard": "Tallow Dredgers and canal scavengers; 12-20 steps a fight",
                          "rendering_yard": "no binding: the dredger roamer; the yard's fight is the `vats` encounter marker"})


def crucible_watchpost():
    """Node 06 at mission-ground size: the fortified post above the Slag Gate, with the winch that lifts it (M5: one
    founder holds the winch, three get out; the Overlook variant).

    The party arrives at the gorge mouth (south, safe). The road switchbacks up the gorge floor between boulders
    (danger: Watch sentries patrol it) to the gatehouse in the curtain wall; an old sentry hut in the east cliff is a
    safe stop (enterable). A side path climbs the west cliff to the Crag-Sight Overlook. Inside the walls, the yard
    before the winch house is the mission ground, with the slit-window tower west and a postern north up the gorge."""
    L = Layout(48, 40, "m")
    L.border(); L.rect(1, 1, 46, 38, "#")
    L.rect(14, 33, 33, 37, ".", "m"); L.rect(23, 38, 24, 39, ".", "c")      # the gorge mouth (arrival, safe)
    L.rect(8, 17, 39, 32, ".", "m")                                          # the gorge floor (danger)
    for x0, y0, x1, y1 in ((22, 28, 25, 32), (11, 27, 25, 28), (11, 21, 14, 27), (11, 20, 24, 21), (22, 17, 25, 20)):
        L.rect(x0, y0, x1, y1, ground="c")                                   # the switchback road
    L.rect(22, 33, 25, 37, ground="c")
    L.rect(3, 20, 7, 21, ".", "s"); L.rect(3, 8, 4, 19, ".", "s"); L.rect(3, 5, 6, 7, ".", "s")   # the Overlook path (its ledge walled off from the yard)
    L.rect(9, 3, 39, 13, ".", "f")                                           # the yard (the mission)
    L.rect(23, 14, 24, 16, ".", "f")                                         # the gatehouse passage
    for i, x in enumerate((9, 13, 17, 26, 30, 34)):
        L.prop(f"curtain_{i + 1}", "watch_wall", x, 14, 4, 2)
    L.rows[16][22] = L.rows[16][25] = "#"
    L.prop("gate_w", "gatepost", 22, 16); L.prop("gate_e", "gatepost", 25, 16)
    L.rect(10, 4, 14, 6, "#"); L.prop("tower", "slit_window_tower", 10, 4, 5, 3)
    L.prop("winch", "winch_drum", 23, 1, 2, 2); L.prop("crane", "crane_arm", 25, 1)
    for i, x in enumerate((14, 18, 27, 31)):
        L.prop(f"north_wall_{i + 1}", "watch_wall", x, 1, 4, 2)
    L.rect(36, 0, 36, 0, ".", "f"); L.rect(36, 1, 37, 2, ".", "f")          # the postern up the gorge
    L.rows[6][30] = "#"; L.prop("brazier", "brazier", 30, 6)
    for i, (x, y) in enumerate(((16, 30), (30, 24), (32, 29), (17, 24), (34, 19))):
        L.prop(f"boulder_{i + 1}", "boulder", x, y, 2, 2)
    for i, (x, y) in enumerate(((41, 18), (41, 31), (1, 26), (1, 31))):
        L.prop(f"cliff_{i + 1}", "cliff_face", x, y, 4, 1)
    L.rect(0, 0, 47, 39, zone="-")
    L.rect(14, 33, 33, 37, zone="s"); L.rect(23, 38, 24, 39, zone="s"); L.rect(8, 17, 39, 32, zone="d")
    L.rect(3, 5, 7, 21, zone="b"); L.rect(9, 3, 39, 13, zone="y"); L.rect(23, 14, 24, 16, zone="y")
    L.rect(36, 0, 37, 2, zone="y")
    L.indoor("h", 40, 24, 46, 29, (40, 26), "f", "sentry_hut")              # the old sentry hut (a safe stop)
    L.paths = {"sentry_patrol": path_through([(23, 31), (23, 28), (12, 28), (12, 21), (23, 21), (23, 18)])}
    L.anchor("spawn", 23, 35, "arrival from the world map (the gorge mouth)")
    L.anchor("exit_road", 23, 39, "exit `world_map` (was `east`): the road south, World map: the Crucible")
    L.anchor("postern", 36, 0, "exit (optional): the postern up the high gorge to the Slag Gate, World map")
    L.anchor("crucible", 24, 8, "encounter `crucible` (was `site_b`): the yard before the winch house (M5, the mission)")
    L.anchor("crucible_overlook", 23, 19, "encounter `crucible_overlook` (was `site_a`): before the gatehouse, the Overlook variant")
    L.anchor("winch_post", 24, 3, "the winch (the founder who holds it; a point or a set piece)")
    L.anchor("overlook", 5, 6, "point `overlook` (was `altar`): the Crag-Sight Overlook, after the Interlude")
    L.anchor("hut", 43, 26, "inside the old sentry hut: a safe stop (rest or a contact, indoor)")
    L.anchor("patrol_start", 23, 31, "roamer `sentry_patrol`: Watch sentries on the switchback road")
    return mission_entry(L, "sunk_ward_crucible", "Sunk Ward: Crucible Watchpost",
                         {"s": {"id": "gorge_mouth", "kind": "safe"}, "d": {"id": "gorge_road", "kind": "danger"},
                          "b": {"id": "overlook_path", "kind": "danger"}, "y": {"id": "watchpost", "kind": "danger"},
                          "h": {"id": "sentry_hut", "kind": "safe", "indoor": True}}, "scfm",
                         {"gorge_road": "Watch sentries and crossbowmen; the roamer as well",
                          "overlook_path": "cliff scavengers; 12-20 steps a fight",
                          "watchpost": "no binding: the fights are the `crucible` / `crucible_overlook` encounter markers"})


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
    for zc, z in e["zone_legend"].items():
        if z.get("indoor"):
            roofs = [p for p in e["props"] if p.get("roof_of") == zc]
            if not roofs:
                errs.append(f"indoor zone {z['id']} has no roof prop")
            for y, r in enumerate(e["zones"]):
                for x, c in enumerate(r):
                    if c == zc and not any(p["x"] <= x < p["x"] + p["w"] and p["y"] <= y < p["y"] + p["h"] for p in roofs):
                        errs.append(f"indoor zone {z['id']} cell ({x},{y}) outside its roof")
    for pid, pts in e.get("paths", {}).items():
        for i, (x, y) in enumerate(pts):
            if rows[y][x] != ".":
                errs.append(f"path {pid} step {i} on a blocked tile ({x},{y})")
            if i and abs(x - pts[i - 1][0]) + abs(y - pts[i - 1][1]) != 1:
                errs.append(f"path {pid} step {i} is not one orthogonal step")
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
        e["location"] = loc.removesuffix("_large")      # a large variant belongs to the same place
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
