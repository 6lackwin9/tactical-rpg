# assets

System: S14. Status: folder pattern built (S14-L1 commit 1); placeholder art stays until the base game is done. Spec: docs/specs/S14_L1.md. See docs/STRUCTURE.md and docs/ROADMAP.md.

Purpose: file overrides for art and sound keys. Content names art by key; `AssetRegistry` (scripts/art/) resolves a key to its manifest row in `data/assets/manifest.json` (`AssetCatalog`), else the generated SVG placeholder, else the namespace fallback.

Folders (a row `path` must follow the pattern; the file must exist and load):

- `art/unit/`, `art/item/`, `art/tile/`, `art/skill/`, `art/status/`, `art/ui/`: `<key>_<variant>.png` or `.svg` (ui: png)
- `audio/sfx/`: `<key>.ogg` or `.wav`; `audio/music/`: `<key>.ogg`

Rules: add a file and its manifest row through `AssetCatalog.upsert` in a seed script (no code change); `tools/export_placeholder.gd` writes the current placeholder as a starting PNG. Fonts are not planned yet.

Placeholder art (ART1-A): `python3 tools/gen_art.py [group ...]` (needs Pillow and numpy; deterministic) regenerates every placeholder PNG under `art/` (groups: characters classes items skills statuses terrain overlays region markers pins world_map backdrops menu; `--list` prints them). After a run, import the PNGs (`godot --headless --import`, commit the `.png.import` files) and re-run `tools/fc.sh run data/seeds/art1_a.gd`, which upserts one manifest row per file (size read from the file), sets the Ashlands character `art` keys and the four `ashlands_*` map templates. Swap a file by replacing the PNG at the same path (keep the size); the manifest row needs no change.
