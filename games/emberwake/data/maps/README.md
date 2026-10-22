# data/maps

System: S9. Status: schema built (EXP1-L commit 1); templates come from EXP1-A. Specs: games/emberwake/docs/specs/EXP1_L.md, games/emberwake/docs/specs/EXP1_A.md.

Purpose: Lore-free region map templates (JSON).

`maps.json`: envelope `{"kind": "map_templates", "version": 1, "entries": [...]}`. Each entry is `{id, name, rows, zones, zone_legend, anchors, paths, props, tags}`: `rows` are 8..64 strings of 8..64 tiles (`.` open, `#` wall, `~` water), `zones` the same grid of `-` (no zone, SAFE) or legend chars, `zone_legend` char -> `{id, kind: safe|danger}`, `anchors` `[{id, x, y}]` on open tiles with one `spawn`, `paths` id -> orthogonal `[[x, y], ...]` (roamer patrols), `props` the art drawn over the tiles (PROPS1, below). `x` is the column, `y` the row, `(0, 0)` top-left.

`props` (default `[]`, at most 128): `[{id, art, x, y, w, h, layer, variant, flip}]`, stored whole in that order. `id` and `art` lowercase `[a-z0-9_]` (2-24 and 2-32 characters, ids unique per map); `x, y` the footprint's top-left tile; `w, h` 1-16 (default 1); `layer` `sort` (default: stands up, sorted with the characters by their feet; its whole footprint must be `#` wall tiles and no two standing props share a tile) or `ground` (flat, under every character, any tiles); `variant` -1 (default: picked by position) or 0-15; `flip` mirrors it. A prop never changes where anyone can step: walls stay in `rows`. The art is `props/<set>/<art>/v<n>.png` along the world lineage, then `common` (32 px per tile; a standing prop up to one tile wider and four taller than its footprint). Checks P1-P12 (errors, `maps:<id>`) and A7-A10 (asset warnings): `games/emberwake/docs/specs/PROPS1.md` sections 2-3.

Edit only through `aegis content` (`tools/fc.sh content upsert` of kind `maps` with the full entry, or a seed applied with `content apply`), never raw JSON. World locations bind a template through their `map`, `zone_bindings` and `markers` fields (`MapRules` validates them).
