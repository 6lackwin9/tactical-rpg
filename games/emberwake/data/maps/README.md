# data/maps

System: S9. Status: schema built (EXP1-L commit 1); templates come from EXP1-A. Specs: docs/specs/EXP1_L.md, docs/specs/EXP1_A.md.

Purpose: Lore-free region map templates (JSON).

`maps.json`: envelope `{"kind": "map_templates", "version": 1, "entries": [...]}`. Each entry is `{id, name, rows, zones, zone_legend, anchors, paths, tags}`: `rows` are 8..64 strings of 8..64 tiles (`.` open, `#` wall, `~` water), `zones` the same grid of `-` (no zone, SAFE) or legend chars, `zone_legend` char -> `{id, kind: safe|danger}`, `anchors` `[{id, x, y}]` on open tiles with one `spawn`, `paths` id -> orthogonal `[[x, y], ...]` (roamer patrols). `x` is the column, `y` the row, `(0, 0)` top-left.

Edit only through `MapCatalog.upsert` + `MapCatalog.save` (seed scripts), never raw JSON. World locations bind a template through their `map`, `zone_bindings` and `markers` fields (`MapRules` validates them).
