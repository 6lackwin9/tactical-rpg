# assets

System: S14. Status: folder pattern built (S14-L1 commit 1); placeholder art stays until the base game is done. Spec: docs/specs/S14_L1.md. See docs/STRUCTURE.md and docs/ROADMAP.md.

Purpose: file overrides for art and sound keys. Content names art by key; `AssetRegistry` (scripts/art/) resolves a key to its manifest row in `data/assets/manifest.json` (`AssetCatalog`), else the generated SVG placeholder, else the namespace fallback.

Folders (a row `path` must follow the pattern; the file must exist and load):

- `art/unit/`, `art/item/`, `art/tile/`, `art/skill/`, `art/status/`, `art/ui/`: `<key>_<variant>.png` or `.svg` (ui: png)
- `audio/sfx/`: `<key>.ogg` or `.wav`; `audio/music/`: `<key>.ogg`

Rules: add a file and its manifest row through `AssetCatalog.upsert` in a seed script (no code change); `tools/export_placeholder.gd` writes the current placeholder as a starting PNG. Fonts are not planned yet.
