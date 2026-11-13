"""Standalone preview: the Sunk Ward district map recut to the Act 1 two-tier diamond (AI-09, 2026-11-13).

Usage:
    python3 -m tools.mock_ui.previews.world_diamond --out /path/to/world_diamond_preview.html [--cache BUNDLE.json]

Reads the shipped graph straight from the game data (`games/emberwake/data/worlds/sunk_ward/{locations,routes,
encounters}.json`: node numbers, map_pos, routes and their flag gates), so the preview is the map the game ships, not a
reading of CANON. It reuses the mock UI's styles, fonts, backdrop and pins (`tools/mock_ui/assets.py`), shows the
district map at each step of Act 1 (Day 1 -> Tier 1 -> Tier 2 + Interlude -> M5 -> Slag Gate), and the mock's
current pre-diamond map beside it for comparison. Look only: nothing travels.
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
# within the map so they don't seem out of place"; the diamond need not stay mirrored).
REGIONS = {
    "sluice_undercroft": (0.60, 0.99),     # the flooded basin
    "drowned_runoff_grate": (0.50, 0.79),  # the arched drain painted in the basin's top wall
    "alleyway_junction_7": (0.17, 0.66),   # the left tenement alleys
    "ash_crawl_crossroads": (0.80, 0.64),  # the right-hand slum block
    "fourth_kiln": (0.15, 0.33),           # the chimneys and furnace glow, upper left
    "tallow_vats": (0.84, 0.34),           # on the painted vat tanks
    "crucible_watchpost": (0.50, 0.31),    # the gorge mouth
    "slag_gate": (0.50, 0.10),             # up the gorge, on the slag seam
    "corvos_gate": (0.76, 0.85),           # across the right canal
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
        mx, my = loc["map_pos"]
        nodes.append({
            "id": loc["id"], "num": num, "name": loc["name"], "desc": desc[:1].upper() + desc[1:], "mission": mission,
            "who": who, "pin": pin, "x": px(mx), "y": py(my),
            "pos": {"data": [px(mx), py(my)], "spaced": [px(SPACED.get(loc["id"], (mx, my))[0]), py(SPACED.get(loc["id"], (mx, my))[1])],
                    "regions": [px(REGIONS.get(loc["id"], (mx, my))[0]), py(REGIONS.get(loc["id"], (mx, my))[1])]},
            "encounters": [{"id": i, "name": encs[i].get("name", i), "desc": encs[i].get("description", "")} for i in loc.get("encounters", []) if i in encs],
        })
    ids = {n["id"] for n in nodes}
    edges = [{"id": r["id"], "name": r["name"], "a": r["from"], "b": r["to"], "time": r["time"],
              "req": [f["key"] for f in r["requires_flags"] if f.get("value") is True]}
             for r in routes if r["from"] in ids and r["to"] in ids]
    return {"nodes": nodes, "edges": edges}


def build(out: Path, cache: Path | None, bg: Path | None = None, landmarks: Path | None = None) -> None:
    bundle = load_or_build_bundle(cache_path=cache)
    styles = ((ROOT / "tools/mock_ui/styles.css").read_text(encoding="utf-8")
              .replace("__FONT_DISPLAY__", bundle["fonts"]["display"])
              .replace("__FONT_BODY__", bundle["fonts"]["body"])
              .replace("__FONT_MONO__", bundle["fonts"]["mono"])
              .replace("__ORNATE_CORNER__", bundle["ui"]["ornate_corner"]))
    keys = ["pin_town", "pin_gate", "pin_ruin", "pin_fortress", "pin_camp", "pin_here", "badge_quest", "gold",
            "ornate_tl", "ornate_tr", "ornate_bl", "ornate_br"]
    assets = {"ui": {k: bundle["ui"][k] for k in keys if k in bundle["ui"]}, "map": bundle["bg"]["sunk_ward"]}
    if bg:  # a candidate act map background (a take under review): shown as is, cropped to the map viewport
        import base64, io
        from PIL import Image
        buf = io.BytesIO()
        Image.open(bg).convert("RGB").save(buf, "JPEG", quality=88)
        assets["map"] = "data:image/jpeg;base64," + base64.b64encode(buf.getvalue()).decode()
    if landmarks:  # cut landmarks (map/landmarks/<location>.png) drawn in place of the kind pins
        import base64
        assets["lm"] = {f.stem: "data:image/png;base64," + base64.b64encode(f.read_bytes()).decode()
                        for f in sorted(Path(landmarks).glob("*.png"))}
    data = {"graph": load_graph(), "steps": STEPS, "current": CURRENT, "bgIsMap": bool(bg)}
    html = ((HERE / "world_diamond.html").read_text(encoding="utf-8")
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
    args = ap.parse_args()
    build(args.out, args.cache, args.bg, args.landmarks)


if __name__ == "__main__":
    main()
