# S12T-A: terrain numbers (one catalog row per terrain type)

Status: spec (design-brainstorm 2026-10-17; DECISIONS combat "S12 baseline answers"; awaiting approval)
Depends on: S12T-L commit 1 (terrain kind, fields, validator)
Lane: asset
Touches: new `data/catalog/terrain.json` (through `ContentCatalog.upsert` / `upsert_many("terrain", ...)` inside `begin_system_edit` / `end_system_edit`, rows are system origin), seed `data/seeds/s12t.gd`, `data/seeds/README.md`

## Goal

Write the 17 terrain rows. Move costs equal the values `Terrain.INFO` already recorded (so movement in the shipped zones matches the recorded intent); cover and hazard numbers are PLACEHOLDER starting values for the balance pass. Cover levels of the old data (1 half, 2 full) map to `cover_guard` 0.15 and 0.30.

## Schema contract

None (S12T-L fields).

## Assets

| id | move_cost | cover_guard | hazard_status | hazard_potency | hazard_magnitude | hazard_rounds | hazard_on |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `floor` | 1 | 0.0 | "" | 100 | 0 | 1 | both |
| `grass` | 1 | 0.0 | "" | 100 | 0 | 1 | both |
| `dirt` | 1 | 0.0 | "" | 100 | 0 | 1 | both |
| `stone` | 1 | 0.0 | "" | 100 | 0 | 1 | both |
| `sand` | 2 | 0.0 | "" | 100 | 0 | 1 | both |
| `snow` | 2 | 0.0 | "" | 100 | 0 | 1 | both |
| `shallow_water` | 2 | 0.0 | "" | 100 | 0 | 1 | both |
| `deep_water` | 1 | 0.0 | "" | 100 | 0 | 1 | both |
| `mud` | 2 | 0.0 | "" | 100 | 0 | 1 | both |
| `lava` | 1 | 0.0 | "" | 100 | 0 | 1 | both |
| `wall` | 1 | 0.30 | "" | 100 | 0 | 1 | both |
| `tree` | 1 | 0.15 | "" | 100 | 0 | 1 | both |
| `bridge` | 1 | 0.0 | "" | 100 | 0 | 1 | both |
| `rubble` | 2 | 0.15 | "" | 100 | 0 | 1 | both |
| `burning_ground` | 1 | 0.0 | `burn` | 100 | 5 | 1 | both |
| `toxic_bog` | 2 | 0.0 | `poison` | 80 | 4 | 2 | both |
| `ice` | 1 | 0.0 | `slow` | 100 | 0 | 1 | enter |

Non-walkable rows (`deep_water`, `lava`, `wall`, `tree`) store `move_cost` 1 (ignored by the rules) and no hazard; their `cover_guard` matters only for directional cover (wall, tree).

## Acceptance (rule-derived)

1. `ContentCatalog.validate_all()` is `{}` and has no `terrain:missing`; `warnings_all()` has no terrain entry.
2. Every `Terrain.Type` has exactly one row; for the 14 old types `move_cost` equals the old `Terrain.INFO` value.
3. `Terrain.describe(RUBBLE)` shows its cover; `describe(BURNING_GROUND)` names Burning.
4. `tools/fc.sh sim duel 5` equals the pre-seed run (sim arenas are all-floor).
5. Seed listed in `data/seeds/README.md`.

## Out of scope

Zone map changes (code, S12T-L rule 9), new terrain types, world or region terrain.

## Open questions

None.
