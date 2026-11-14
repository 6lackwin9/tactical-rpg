"""Sunk Ward region map layouts: a standalone top-down preview (AI-12, Art/UI 2026-11-14).

    python3 -m tools.mock_ui.previews.region_map --out <file>.html

Reads the draft layouts in art_src/region_maps/<location>.json (written by art_src/region_maps/build_layouts.py, the
maps.json template shape), the ground colours from games/emberwake/data/catalog/grounds.json and each location's
act-map landmark (art_src/lm_<location>.jpg, keyed out with cut_ai_sheets), and draws every layout as the exploration
map's grid: grounds in their catalog colours (the game draws a ground without art as that colour), walls, water,
standing and ground props as labelled footprints, anchors with the marker each is for, and the zones. Rule 1 of the
Art/UI session: look and layout are signed off here before Design specs the templates (SUNKWARD-MAPS1).
"""
from __future__ import annotations

import argparse
import base64
import io
import json
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
LAYOUTS = ROOT / "art_src" / "region_maps"
ORDER = ["sluice_undercroft", "drowned_runoff_grate", "alleyway_junction_7", "ash_crawl_crossroads", "fourth_kiln",
         "tallow_vats", "crucible_watchpost", "slag_gate", "corvos_gate"]
TITLES = {"sluice_undercroft": "00 Undercroft", "drowned_runoff_grate": "01 Grate", "alleyway_junction_7": "02 Junction 7",
          "ash_crawl_crossroads": "03 Crossroads", "fourth_kiln": "04 Kiln", "tallow_vats": "05 Vats",
          "crucible_watchpost": "06 Crucible", "slag_gate": "07 Slag Gate", "corvos_gate": "1d Corvo's Gate"}
FIXES = {"sluice_undercroft": {"desmokes": ["0.69,0.30,0.80,0.52"]}, "tallow_vats": {"tints": ["0.05-0.16:0.19@0.33,0,1,0.62"]}}


def landmark_uri(location: str) -> str:
    src = ROOT / "art_src" / f"lm_{location}.jpg"
    if not src.exists():
        return ""
    sys.path.insert(0, str(ROOT / "art_src" / "prompts"))
    import cut_ai_sheets as C  # noqa: E402
    from PIL import Image
    fixes = FIXES.get(location, {})
    with tempfile.TemporaryDirectory() as tmp:
        C.cut_landmark(str(src), "x", tmp, size=288, **fixes)
        im = Image.open(Path(tmp) / "games/emberwake/assets/map/landmarks/x.png")
        buf = io.BytesIO(); im.save(buf, "PNG", optimize=True)
    return "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()


def build(out: Path) -> None:
    grounds = {g["id"]: g.get("colour", "#333333")
               for g in json.loads((ROOT / "games/emberwake/data/catalog/grounds.json").read_text(encoding="utf-8"))["entries"]}
    maps = []
    for loc in ORDER:
        e = json.loads((LAYOUTS / f"{loc}.json").read_text(encoding="utf-8"))
        e["title"] = TITLES.get(loc, loc)
        e["landmark"] = landmark_uri(loc)
        maps.append(e)
    html = (Path(__file__).with_name("region_map.html").read_text(encoding="utf-8")
            .replace("/*DATA*/null", json.dumps({"maps": maps, "grounds": grounds})))
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(html, encoding="utf-8", newline="\n")
    print(f"[region_map] Wrote {out} ({out.stat().st_size / 1024:.1f} KB)")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--out", type=Path, default=ROOT / "target" / "region_map_preview.html")
    build(ap.parse_args().out)


if __name__ == "__main__":
    main()
