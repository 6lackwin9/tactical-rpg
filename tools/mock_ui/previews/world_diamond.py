"""Standalone preview: an act's district map as a diamond of landmarks (Act 1: AI-09, Act 2: Art/UI 2026-11-13).

Usage:
    python3 -m tools.mock_ui.previews.world_diamond --out /path/to/world_diamond_preview.html [--cache BUNDLE.json]
    python3 -m tools.mock_ui.previews.world_diamond --act 2 --bg <act 2 background> --landmarks <cuts> --out ...

Reads the shipped graph straight from the game data (`games/emberwake/data/worlds/sunk_ward/{locations,routes,
encounters}.json`: node numbers, map_pos, routes and their flag gates), so the preview is the map the game ships, not a
reading of CANON. It reuses the mock UI's styles, fonts, backdrop and pins (`tools/mock_ui/assets.py`), shows the
district map at each step of Act 1 (Day 1 -> Tier 1 -> Tier 2 + Interlude -> M5 -> Slag Gate), and the mock's
current pre-diamond map beside it for comparison. Look only: nothing travels.

Act 2 (`--act 2`) has no game data yet, so its graph is written out below (ACT2) from the canon campaign map
(`lore/background/upper_bastion_campaign_map.md`): the nine places at the layout Tushar settled on background take A
(each with its own map scale), the roads along the painted streets, and each leg's street (1 day) and rooftop (2 days)
ways. Its cut landmarks are drawn at 96 x their scale (`cut_ai_sheets.py landmark ... --size N`).
"""
import argparse
import json
import re
from pathlib import Path

from tools.mock_ui.assets import load_or_build_bundle
from tools.mock_ui.build import DEFAULT_CACHE

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
WORLD = ROOT / "games/emberwake/data/worlds/sunk_ward"

# Map viewport (worldmap.rs MAP = Rect(0, 22, 500, 316), drawn 496 x 328 in the mock): map_pos 0..1 -> pixels.
VIEW_W, VIEW_H, PAD_X, PAD_Y = 496, 328, 34, 22

# Per node: mission label, boss / who, pin. Names, descriptions and node numbers come from locations.json.
ROLE = {
    "sluice_undercroft": ("Hub", "Sabina, Berek, Talia · 6 bunks", "pin_town"),
    "drowned_runoff_grate": ("Egress", "The drains meet the streets", "pin_gate"),
    "alleyway_junction_7": ("M1 · Tier 1", "Captain Vane", "pin_ruin"),
    "ash_crawl_crossroads": ("M2 · Tier 1", "Proctor Quell", "pin_camp"),
    "fourth_kiln": ("M3 · Tier 2", "Overseer Grendel · Dunstan joins", "pin_fortress"),
    "tallow_vats": ("M4 · Tier 2", "Guild-Master Orlov · Elyse joins", "pin_fortress"),
    "crucible_watchpost": ("M5 · Act 1 climax", "Inquisitor-Prelate Nemeth", "pin_fortress"),
    "slag_gate": ("Exit to Act 2", "Upper Cinderreach beyond", "pin_gate"),
    "corvos_gate": ("Side job", "Bailiff Corvo", "pin_gate"),
}

# The Act 1 steps the stepper walks through: the flags set, the missions cleared, and what the step shows.
STEPS = [
    {"id": "day1", "label": "1. Day 1", "flags": [], "cleared": [], "note": "Tier 1 is open: the Sweep (M1) and the Crossroads Cages (M2), both reached from the grate."},
    {"id": "m1", "label": "2. M1 done", "flags": ["sunk_ward.m1_done"], "cleared": ["alleyway_junction_7"], "note": "The Third Ward road opens: Bailiff Corvo's gate is a side job from here on."},
    {"id": "tier1", "label": "3. Tier 1 done", "flags": ["sunk_ward.m1_done", "sunk_ward.m2_done"], "cleared": ["alleyway_junction_7", "ash_crawl_crossroads"], "note": "Tier 2 opens on both sides of the diamond: the pipe duct to the Kiln (M3), the slum alleys to the Vats (M4). Vesper's lock ciphers add the Siphon Shortcut straight to the Vats."},
    {"id": "tier2", "label": "4. Tier 2 → Interlude", "flags": ["sunk_ward.m1_done", "sunk_ward.m2_done", "sunk_ward.lock_ciphers_done"], "cleared": ["alleyway_junction_7", "ash_crawl_crossroads", "fourth_kiln", "tallow_vats"], "interlude": True, "note": "Both Tier 2 raids are turned in: the marrow back-surge breaks into the Undercroft. The Interlude (the Calcified Amalgam) is fought at the hub; the climb stays shut until it is done."},
    {"id": "m5", "label": "5. Interlude → M5", "flags": ["sunk_ward.m1_done", "sunk_ward.m2_done", "sunk_ward.lock_ciphers_done", "sunk_ward.interlude_done"], "cleared": ["alleyway_junction_7", "ash_crawl_crossroads", "fourth_kiln", "tallow_vats"], "note": "The flue climb (from the Kiln) and the convoy track (from the Vats) open: the diamond closes on the Crucible Watchpost (M5)."},
    {"id": "exit", "label": "6. M5 → Slag Gate", "flags": ["sunk_ward.m1_done", "sunk_ward.m2_done", "sunk_ward.lock_ciphers_done", "sunk_ward.interlude_done", "sunk_ward.m5_done"], "cleared": ["alleyway_junction_7", "ash_crawl_crossroads", "fourth_kiln", "tallow_vats", "crucible_watchpost"], "note": "One founder held the winch; the high gorge leads out through the Slag Gate to Act 2."},
]

# The mock UI's map as it is today (tools/mock_ui/js/01_state_and_data.js WORLD_NODES), for the comparison view.
CURRENT = [
    {"id": "n_sluice", "name": "00 · The Sluice Undercroft", "pin": "pin_town", "x": 115, "y": 225, "edges": ["n_grate", "n_corvo"]},
    {"id": "n_grate", "name": "01 · Drowned Runoff Grate", "pin": "pin_ruin", "x": 195, "y": 265, "edges": ["n_sluice", "n_corvo"]},
    {"id": "n_corvo", "name": "02 · Third Ward Gate (Corvo's Toll)", "pin": "pin_gate", "x": 245, "y": 170, "edges": ["n_sluice", "n_grate", "n_cross", "n_kiln"]},
    {"id": "n_cross", "name": "03 · Ash-Crawl Crossroads", "pin": "pin_camp", "x": 310, "y": 115, "edges": ["n_corvo", "n_kiln", "n_crucible"]},
    {"id": "n_kiln", "name": "04 · The Fourth Kiln (Bone-Mill)", "pin": "pin_fortress", "x": 375, "y": 195, "edges": ["n_corvo", "n_cross", "n_crucible"]},
    {"id": "n_crucible", "name": "06 · Crucible Watchpost (Climax)", "pin": "pin_fortress", "x": 435, "y": 95, "edges": ["n_cross", "n_kiln"]},
]


# A proposed spacing (Tushar 2026-11-13: the hub and grate, 02/04, 03/05 and 06/07 landmarks nearly touch at the data's
# map_pos). Preview only; the real positions are game data (Outbox AO-01).
SPACED = {  # each same-column pair (00/01, 02/04, 03/05, 06/07) about 60 px apart: a 48 px landmark plus its label
    "slag_gate": (0.5, 0.113), "crucible_watchpost": (0.5, 0.324),
    "fourth_kiln": (0.31, 0.514), "tallow_vats": (0.69, 0.514),
    "alleyway_junction_7": (0.31, 0.725), "ash_crawl_crossroads": (0.69, 0.725),
    "drowned_runoff_grate": (0.5, 0.768), "corvos_gate": (0.86, 0.768), "sluice_undercroft": (0.5, 0.986),
}


# Each landmark on its own painted place in Act 1 background take 1 (Tushar 2026-11-13: "tie the landmark to the region
# within the map so they don't seem out of place"; the diamond need not stay mirrored). Final spots and map scales from
# the Act Map Layout page (Tushar 2026-11-14), the same numbers as art_src/prompts/act1_map_layout.json.
REGIONS = {
    "sluice_undercroft": (0.597, 0.972),     # the flooded basin
    "drowned_runoff_grate": (0.504, 0.756),  # above the arched drain in the basin's top wall
    "alleyway_junction_7": (0.159, 0.649),   # the left tenement alleys
    "ash_crawl_crossroads": (0.847, 0.676),  # the right-hand slum block
    "fourth_kiln": (0.185, 0.382),           # the chimneys and furnace glow, upper left
    "tallow_vats": (0.787, 0.432),           # on the painted vat tanks
    "crucible_watchpost": (0.487, 0.353),    # the gorge mouth
    "slag_gate": (0.488, 0.132),             # up the gorge, on the slag seam
    "corvos_gate": (0.752, 0.855),           # across the right canal
}
ACT1_SCALE = {"drowned_runoff_grate": 1.1, "alleyway_junction_7": 1.2, "ash_crawl_crossroads": 1.2, "fourth_kiln": 1.2,
              "tallow_vats": 1.2, "crucible_watchpost": 1.2, "slag_gate": 1.2}  # the rest x1.0


# Route waypoints along the painted roads of background take 1 (Tushar 2026-11-13: routes along the roads instead of
# straight lines). Each list runs from the route's `from` to its `to` location (routes.json), endpoints excluded.
# Redrawn 2026-11-14 for the final Act 1 spots, on the road centrelines measured from the background (the ring above the
# basin wall at y 0.67, the left loop past the bridge at x 0.25, the right loop up to the gorge at x 0.62-0.65, and the
# roads down both sides of the basin).
ROUTE_PATHS = {
    "sluice_gate": [(0.585, 0.88), (0.54, 0.80)],                                                    # across the basin
    "water_siphon": [(0.50, 0.67), (0.42, 0.678), (0.375, 0.615), (0.322, 0.58), (0.262, 0.575), (0.21, 0.61)],  # ring, left loop, bridge
    "drain_flue": [(0.515, 0.675), (0.57, 0.67), (0.61, 0.688), (0.65, 0.71), (0.70, 0.76), (0.735, 0.795), (0.79, 0.74)],  # ring, right road
    "third_ward_road": [(0.515, 0.675), (0.57, 0.67), (0.61, 0.688), (0.65, 0.71), (0.70, 0.76), (0.735, 0.80)],  # down the basin's right side
    "siphon_shortcut": [(0.515, 0.675), (0.59, 0.672), (0.633, 0.63), (0.68, 0.60), (0.71, 0.54), (0.73, 0.47)],  # ring, up the right canal
    "kiln_link": [(0.21, 0.61), (0.262, 0.565), (0.26, 0.49), (0.258, 0.425)],                       # bridge, up the left loop
    "crossroads_link": [(0.84, 0.60), (0.82, 0.52), (0.80, 0.47)],                                    # the slum alleys
    "flue_climb": [(0.265, 0.405), (0.31, 0.37), (0.345, 0.34), (0.372, 0.305), (0.415, 0.29), (0.455, 0.33)],  # the left loop's top
    "convoy_track": [(0.72, 0.44), (0.66, 0.43), (0.618, 0.425), (0.60, 0.385), (0.585, 0.362), (0.54, 0.352)],  # over the canal, right loop
    "high_gorge": [(0.50, 0.30), (0.505, 0.22)],                                                      # the gorge floor
}


def px(mx: float) -> int:
    return round(PAD_X + mx * (VIEW_W - 2 * PAD_X))


def py(my: float) -> int:
    return round(PAD_Y + my * (VIEW_H - 2 * PAD_Y))


def load_graph() -> dict:
    locs = json.loads((WORLD / "locations.json").read_text(encoding="utf-8"))["entries"]
    routes = json.loads((WORLD / "routes.json").read_text(encoding="utf-8"))["entries"]
    encs = {e["id"]: e for e in json.loads((WORLD / "encounters.json").read_text(encoding="utf-8"))["entries"]}
    nodes = []
    for loc in locs:
        if "prologue" in loc.get("tags", []) or loc["id"] not in ROLE:
            continue
        m = re.match(r"Node (\w+)[:,]?\s*(.*)", loc["description"])
        num, desc = (m.group(1), m.group(2)) if m else ("", loc["description"])
        mission, who, pin = ROLE[loc["id"]]
        main, quest, kind = act1_kind(mission)
        mx, my = loc["map_pos"]
        nodes.append({
            "id": loc["id"], "num": num, "name": loc["name"], "desc": desc[:1].upper() + desc[1:], "mission": mission,
            "main": main, "quest": quest, "kind": kind,
            "who": who, "pin": pin, "x": px(mx), "y": py(my), "scale": ACT1_SCALE.get(loc["id"], 1.0),
            "pos": {"data": [px(mx), py(my)], "spaced": [px(SPACED.get(loc["id"], (mx, my))[0]), py(SPACED.get(loc["id"], (mx, my))[1])],
                    "regions": [px(REGIONS.get(loc["id"], (mx, my))[0]), py(REGIONS.get(loc["id"], (mx, my))[1])]},
            "encounters": [{"id": i, "name": encs[i].get("name", i), "desc": encs[i].get("description", "")} for i in loc.get("encounters", []) if i in encs],
        })
    ids = {n["id"] for n in nodes}
    edges = [{"id": r["id"], "name": r["name"], "a": r["from"], "b": r["to"], "time": r["time"],
              "req": [f["key"] for f in r["requires_flags"] if f.get("value") is True],
              "shortcut": any(f["key"] == "sunk_ward.lock_ciphers_done" for f in r["requires_flags"]),
              "via": [[px(x), py(y)] for x, y in ROUTE_PATHS.get(r["id"], [])]}
             for r in routes if r["from"] in ids and r["to"] in ids]
    return {"nodes": nodes, "edges": edges}


ACT1_META = {
    "title": "Sunk Ward Diamond Map", "place": "The Sunk Ward", "viewLabel": "Proposed: Act 1 diamond",
    "progress": "Act 1 progress:", "hub": "sluice_undercroft", "sel": "alleyway_junction_7", "party": "sluice_undercroft",
    "flagNames": {"sunk_ward.m1_done": "M1", "sunk_ward.m2_done": "M2", "sunk_ward.interlude_done": "the Interlude",
                  "sunk_ward.m5_done": "M5", "sunk_ward.lock_ciphers_done": "Vesper's lock ciphers"},
    "days": [1, 4, 7, 12, 14, 17], "tithing": 42, "legend": [392, 4], "spacings": True,
    "tiers": [["TIER 1 · M1 / M2", "alleyway_junction_7"], ["TIER 2 · M3 / M4", "fourth_kiln"],
              ["CLIMB · M5", "crucible_watchpost"], ["EXIT", "slag_gate"]],
    "source": "Graph, node numbers and gates read from the shipped <code>worlds/sunk_ward</code> data; click a node for "
              "its routes and what unlocks them.",
}


def act1_kind(mission: str) -> tuple[bool, bool, str]:
    """(main quest, shown as a quest, player-facing kind) for an Act 1 node: no backend codes on the map."""
    main = bool(re.match(r"M\d", mission))
    kind = (("Main quest · Act 1 climax" if "climax" in mission else "Main quest") if main else
            {"Side job": "Side job", "Hub": "Your hideout", "Egress": "Checkpoint", "Exit to Act 2": "The way out"}.get(mission, mission))
    return main, main or mission == "Side job", kind


# ---- Act 2: the High Bastion (canon graph; no game data yet) ----
# Positions and map scales: the layout Tushar settled on background take A (2026-11-13; Outbox AO-01).
ACT2_NODES = [
    # id, node number, name, kind, main quest?, who / boss, quest name, position, scale, description
    ("sluice_undercroft", "00", "The Sluice Undercroft", "Your hideout", False, "Sabina, Berek, Talia · 6 bunks (8 with Berek's frames)", "",
     (0.424, 1.042), 1, "The cell's one hideout in every act, reached up the concealed siphon runoff and the culvert."),
    ("slag_gate_breach", "08", "Slag Gate Breach", "Checkpoint", False, "The way in from the Sunk Ward", "",
     (0.5, 0.879), 1.15, "The broken Slag portcullis in the ridge's lower wall: the way up from the slums."),
    ("gilded_boulevard", "09", "Gilded Boulevard", "Main quest", True, "The Bastion curfew cordon · Ruthven watches", "Ingress",
     (0.502, 0.637), 1.25, "Polished basalt under magnesium lamps; the Castellan cordon holds the avenue."),
    ("culling_stockades", "12", "Culling Stockades", "Main quest", True, "The culling wagon · Lucan in the pens", "Jailbreak",
     (0.115, 0.846), 1.2, "Iron transit pens of Unsealed levies bound for Lazar's cloisters; break the gate before the wagon leaves."),
    ("tallow_reservoir", "10", "Tallow Reservoir", "Main quest", True, "Three Guild silos", "Sabotage",
     (0.898, 0.987), 1.2, "Upper Cinderreach's tallow silos: wreck them and the arc-lamps go dark (-30% Alert, +20 Warded Tallow)."),
    ("grand_belltower", "13", "Grand Belltower", "Main quest", True, "The Penitent Bell-Warden", "The Chimes",
     (0.05, 0.386), 1.4, "The curfew bells of St. Lyra, rung by a martyr pinned into the yoke: +10 Strain in every battle until they fall silent."),
    ("embalming_cloisters", "11", "Embalming Cloisters", "Main quest", True, "Chief Anatomist Lazar · Sabina's quest", "The Cloisters",
     (0.886, 0.221), 1.2, "Lazar's crimson-and-pearl surgical cloister; through the leaded glass, the winch captive is handed over."),
    ("reliquary_crypts", "14", "Reliquary Crypts", "Main quest · Act 2 climax", True, "Inquisitor-General Ruthven · the First Apostle Saint-Warden", "The Reliquary",
     (0.499, 0.413), 1.25, "The tomb of the First Extinguishers beneath the nave, and the Key of the Mantle Chasm."),
    ("grand_spire_gate", "15", "Grand Spire Gate", "The way out", False, "The Mantle Fissure below (Act 3)", "",
     (0.498, 0.158), 1.35, "The gate between the cathedral's spires on the lip of the city: the way down to the Mantle Fissure."),
]
# Routes: id, name, from, to, days, flags it needs, waypoints along the painted streets of take A (endpoints excluded).
# Every leg between the boulevard and the crypts is a street way (1 day, +10% Alert) and also has a rooftop way
# (2 days, +2% Alert; canon 2.1): the same course pushed out over the roofs. Every corner of the map holds a place, so
# the legend starts in the empty right half of the plaza and unlocked, ready to be dragged (Tushar 2026-11-13).
ACT2_ROUTES = [
    ("siphon_runoff", "The concealed siphon runoff", "sluice_undercroft", "slag_gate_breach", 1, [], [(0.47, 0.96), (0.50, 0.93)]),
    ("slag_portcullis", "The breached Slag portcullis", "slag_gate_breach", "gilded_boulevard", 1, [], [(0.50, 0.77), (0.50, 0.71)]),
    ("gallows_avenue", "Gallows Avenue", "gilded_boulevard", "culling_stockades", 1, ["high_bastion.ingress_done"],
     [(0.45, 0.75), (0.36, 0.73), (0.26, 0.67), (0.18, 0.63), (0.14, 0.71)]),
    ("viaduct_aqueduct", "The Viaduct Aqueduct", "gilded_boulevard", "tallow_reservoir", 1, ["high_bastion.ingress_done"],
     [(0.55, 0.75), (0.64, 0.73), (0.74, 0.67), (0.83, 0.62), (0.86, 0.80), (0.89, 0.93)]),
    ("gargoyle_rafters", "The Gargoyle Rafters", "culling_stockades", "grand_belltower", 1, ["high_bastion.jailbreak_done"],
     [(0.12, 0.66), (0.07, 0.57), (0.05, 0.48)]),
    ("scriptorium_duct", "The Scriptorium Duct", "tallow_reservoir", "embalming_cloisters", 1, ["high_bastion.sabotage_done"],
     [(0.89, 0.84), (0.88, 0.70), (0.93, 0.57), (0.95, 0.48), (0.92, 0.40), (0.86, 0.33), (0.87, 0.26)]),
    ("spire_buttress", "The Spire Buttress", "grand_belltower", "reliquary_crypts", 1, ["high_bastion.chimes_done"],
     [(0.09, 0.40), (0.15, 0.34), (0.23, 0.28), (0.32, 0.23), (0.41, 0.215), (0.47, 0.29)]),
    ("catacomb_flue", "The Catacomb Flue", "embalming_cloisters", "reliquary_crypts", 1, ["high_bastion.cloisters_done"],
     [(0.80, 0.27), (0.69, 0.225), (0.59, 0.215), (0.53, 0.29)]),
    ("sacred_chancel", "The Sacred Chancel", "reliquary_crypts", "grand_spire_gate", 1, ["high_bastion.reliquary_done"],
     [(0.50, 0.28), (0.50, 0.22)]),
]
ACT2_ROOFTOPS = ["gallows_avenue", "viaduct_aqueduct", "gargoyle_rafters", "scriptorium_duct", "spire_buttress", "catacomb_flue"]
ACT2_STEPS = [
    {"id": "arrive", "label": "1. Arrival", "flags": [], "cleared": [],
     "note": "Through the Slag Gate breach: the curfew cordon on the Gilded Boulevard (Ingress, 2.1) is the first foothold."},
    {"id": "ingress", "label": "2. Ingress done", "flags": ["high_bastion.ingress_done"], "cleared": ["gilded_boulevard"],
     "note": "Gallows Avenue opens to the Culling Stockades (Jailbreak, 2.2) and the Viaduct Aqueduct to the Tallow Reservoir (Sabotage, 2.3). Every leg up here has a street way (1 day, +10% Alert) and a rooftop way (2 days, +2% Alert): pick one on the map or in the panel."},
    {"id": "tier", "label": "3. Jailbreak + Sabotage", "flags": ["high_bastion.ingress_done", "high_bastion.jailbreak_done", "high_bastion.sabotage_done"],
     "cleared": ["gilded_boulevard", "culling_stockades", "tallow_reservoir"],
     "note": "The Gargoyle Rafters climb to the Grand Belltower (The Chimes, 2.5); the Scriptorium Duct runs to the Embalming Cloisters (The Cloisters, 2.4)."},
    {"id": "climb", "label": "4. Cloisters + Chimes", "flags": ["high_bastion.ingress_done", "high_bastion.jailbreak_done", "high_bastion.sabotage_done",
                                                           "high_bastion.cloisters_done", "high_bastion.chimes_done"],
     "cleared": ["gilded_boulevard", "culling_stockades", "tallow_reservoir", "embalming_cloisters", "grand_belltower"],
     "note": "Both sides close on the Reliquary Crypts beneath the nave (The Reliquary, 2.6): Ruthven makes his stand."},
    {"id": "exit", "label": "5. Reliquary → Spire Gate", "flags": ["high_bastion.ingress_done", "high_bastion.jailbreak_done", "high_bastion.sabotage_done",
                                                                "high_bastion.cloisters_done", "high_bastion.chimes_done", "high_bastion.reliquary_done"],
     "cleared": ["gilded_boulevard", "culling_stockades", "tallow_reservoir", "embalming_cloisters", "grand_belltower", "reliquary_crypts"],
     "note": "The Key of the Mantle Chasm opens the Sacred Chancel to the Grand Spire Gate, the way down into Act 3."},
]
ACT2_META = {
    "title": "High Bastion Diamond Map", "place": "The High Bastion", "viewLabel": "Proposed: Act 2 diamond",
    "progress": "Act 2 progress:", "hub": "sluice_undercroft", "sel": "gilded_boulevard", "party": "slag_gate_breach",
    "flagNames": {"high_bastion.ingress_done": "Ingress", "high_bastion.jailbreak_done": "the Jailbreak",
                  "high_bastion.sabotage_done": "the Sabotage", "high_bastion.cloisters_done": "the Cloisters",
                  "high_bastion.chimes_done": "the Chimes", "high_bastion.reliquary_done": "the Reliquary"},
    "days": [19, 22, 27, 32, 36], "tithing": 56, "legend": [305, 128], "legendUnlocked": True, "spacings": False, "tiers": [],
    "source": "Act 2 has no game data yet: the graph is the canon campaign map (<code>upper_bastion_campaign_map.md</code>), "
              "at the layout settled on background take A; the gates, days and Tithing count are placeholders.",
}


def act2_graph() -> dict:
    nodes = []
    for lid, num, name, kind, main, who, quest, (mx, my), scale, desc in ACT2_NODES:
        p = [px(mx), py(my)]
        nodes.append({"id": lid, "num": num, "name": name, "desc": desc, "mission": kind, "kind": kind, "main": main,
                      "quest": main, "who": who, "pin": "pin_fortress" if main else "pin_gate", "x": p[0], "y": p[1],
                      "scale": scale, "pos": {"data": p, "spaced": p, "regions": p},
                      "encounters": [{"id": lid, "name": quest, "desc": ""}] if quest else []})
    edges = []
    for rid, name, a, b, days, req, via in ACT2_ROUTES:
        base = name[4:] if name.startswith("The ") else name
        roofed = rid in ACT2_ROOFTOPS
        edges.append({"id": rid, "name": name, "base": base, "a": a, "b": b, "time": days, "req": req,
                      "way": "street" if roofed else "", "alert": 10 if roofed else 0,
                      "via": [[px(x), py(y)] for x, y in via]})
        if rid in ACT2_ROOFTOPS:  # the rooftop way: the same course pushed 0.035 out from the plaza's middle
            roof = []
            for x, y in via:
                dx, dy = x - 0.5, y - 0.5
                n = max((dx * dx + dy * dy) ** 0.5, 1e-6)
                roof.append([px(x + 0.035 * dx / n), py(y + 0.035 * dy / n)])
            edges.append({"id": rid + "_roofs", "name": "The rooftops over " + base, "base": base, "a": a, "b": b,
                          "time": days + 1, "req": req, "way": "rooftops", "alert": 2, "via": roof})
    return {"nodes": nodes, "edges": edges}


def build(out: Path, cache: Path | None, bg: Path | None = None, landmarks: Path | None = None,
          embed_fonts: bool = False, act: int = 1) -> None:
    bundle = load_or_build_bundle(cache_path=cache)
    styles = (ROOT / "tools/mock_ui/styles.css").read_text(encoding="utf-8")
    fonts_link = ""
    if embed_fonts:
        styles = (styles.replace("__FONT_DISPLAY__", bundle["fonts"]["display"])
                  .replace("__FONT_BODY__", bundle["fonts"]["body"])
                  .replace("__FONT_MONO__", bundle["fonts"]["mono"]))
    else:  # light (default): the same three families from Google Fonts instead of ~1 MB of embedded TTF
        styles = re.sub(r"@font-face \{[^}]*\}\s*", "", styles)
        styles = (styles.replace("'AlegreyaSC'", "'Alegreya SC'").replace("'OpenSans'", "'Open Sans'")
                  .replace("'JetBrainsMono'", "'JetBrains Mono'"))
        fonts_link = ('<link rel="preconnect" href="https://fonts.googleapis.com">'
                      '<link href="https://fonts.googleapis.com/css2?family=Alegreya+SC:wght@700&family=JetBrains+Mono:wght@700'
                      '&family=Open+Sans:wght@600&display=swap" rel="stylesheet">')
    styles = (styles
              .replace("__ORNATE_CORNER__", bundle["ui"]["ornate_corner"]))
    keys = ["pin_town", "pin_gate", "pin_ruin", "pin_fortress", "pin_camp", "pin_here", "badge_quest", "gold",
            "ornate_tl", "ornate_tr", "ornate_bl", "ornate_br"]
    assets = {"ui": {k: bundle["ui"][k] for k in keys if k in bundle["ui"]}, "map": bundle["bg"]["sunk_ward"]}
    if bg:  # a candidate act map background (a take under review): shown as is, cropped to the map viewport
        import base64, io
        from PIL import Image
        buf = io.BytesIO()
        Image.open(bg).convert("RGB").save(buf, "JPEG", quality=82)
        assets["map"] = "data:image/jpeg;base64," + base64.b64encode(buf.getvalue()).decode()
    if landmarks:  # cut landmarks (map/landmarks/<location>.png) drawn in place of the kind pins
        import base64
        assets["lm"] = {f.stem: "data:image/png;base64," + base64.b64encode(f.read_bytes()).decode()
                        for f in sorted(Path(landmarks).glob("*.png"))}
    if act == 2:
        data = {"graph": act2_graph(), "steps": ACT2_STEPS, "current": None, "bgIsMap": bool(bg), "act": ACT2_META}
    else:
        data = {"graph": load_graph(), "steps": STEPS, "current": CURRENT, "bgIsMap": bool(bg), "act": ACT1_META}
    html = ((HERE / "world_diamond.html").read_text(encoding="utf-8")
            .replace("__TITLE__", data["act"]["title"])
            .replace("__FONTS_LINK__", fonts_link)
            .replace("__STYLES_CSS__", styles)
            .replace("__ASSETS_JSON__", json.dumps(assets))
            .replace("__DATA_JSON__", json.dumps(data)))
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(html, encoding="utf-8")
    print(f"[world_diamond] Wrote {out} ({len(html) / 1024:.1f} KB)")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--out", type=Path, default=ROOT / "target" / "world_diamond_preview.html")
    ap.add_argument("--cache", type=Path, default=DEFAULT_CACHE, help="the mock UI's asset bundle cache")
    ap.add_argument("--bg", type=Path, help="a candidate act map background to review (default: the mock's title card)")
    ap.add_argument("--landmarks", type=Path, help="a folder of cut landmarks <location_id>.png drawn instead of pins")
    ap.add_argument("--embed-fonts", action="store_true", help="embed the TTF fonts (about 1 MB more) instead of Google Fonts")
    ap.add_argument("--act", type=int, choices=(1, 2), default=1, help="which act's map (2: the canon graph, no game data yet)")
    args = ap.parse_args()
    build(args.out, args.cache, args.bg, args.landmarks, args.embed_fonts, args.act)


if __name__ == "__main__":
    main()
