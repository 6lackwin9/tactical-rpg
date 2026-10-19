# S12T-L: terrain rules (terrain catalog kind, cost-weighted pathing, cover, hazards, three hazard tile types)

Status: approved (user 2026-10-17) (design-brainstorm 2026-10-17; DECISIONS combat "S12 baseline answers" and docs/specs/S12_RULES.md 2.3)
Depends on: S12S-L merged (slow, poison, burn rows); not in parallel with S12C-L (both edit `combat_plan.gd`)
Lane: logic (schema first; commit 1 releases S12T-A)
Touches: `scripts/combat/arena/{terrain,combat_zones,grid_path,combat_grid}.gd`, new `scripts/combat/arena/cover_rules.gd`, new `scripts/combat/arena/hazard_rules.gd`, `scripts/combat/planning/combat_plan.gd` (path cost, move chunking, reach), `scripts/combat/actions/{walk_command,attack_command,area_attack_command,disengage_command}.gd`, `scripts/core/combat/damage_calculator.gd` (`compute` gains a `cover_ratio` input), `scripts/combat/arena/combat_resolver.gd` (round-end hazards), `scripts/combat/ai/tactical_ai.gd`, `scripts/content/content_schema.gd` (kind list), new `scripts/content/terrain_fields.gd`, new `scripts/content/validators/terrain_validator.gd`, `scripts/content/content_catalog.gd` (dispatch only), `scripts/art/tile_art.gd` (three drawings), new `scripts/tests/terrain_rules_test_suite.gd` + one line in `scripts/tests/suite_registry.gd`

## Goal

The terrain numbers `Terrain.INFO` already records but never applies become rules: entering a tile costs its move cost, ranged hits on covered targets lose damage through the guard input, and hazard tiles apply registry statuses. Numbers live in a catalog kind `terrain` (numbers only; types, walkability and glyphs stay code; zone maps stay in `CombatZones` until S9 map data). `DamageCalculator.compute` keeps its formula; cover is an input to its guard term.

## Schema contract (logic lane, schema first: yes)

### Terrain types (code)

`Terrain.Type` gains three walkable hazard types APPENDED to the enum (existing ordinals unchanged; `TileArt` cache keys use the int):

| Type | id | Glyph | Name | walkable |
| --- | --- | --- | --- | --- |
| `BURNING_GROUND` | `burning_ground` | `f` | Burning Ground | true |
| `TOXIC_BOG` | `toxic_bog` | `p` | Toxic Bog | true |
| `ICE` | `ice` | `i` | Ice | true |

`Terrain.INFO` keeps `name`, `glyph`, `walkable` and loses `cover`, `move_cost`, `hazard` (moved to the catalog; `describe()` reads the catalog). The id of a type is its enum name in lower snake case (`Terrain.id_of(t)`, `Terrain.from_id(id)`).

### Catalog kind `terrain` (`data/catalog/terrain.json`, version 1; `ContentCatalog` kind list gains `terrain`)

One row per `Terrain.Type`. Fields (`terrain_fields.gd`):

| Field | Type | Default | Validation (`TerrainValidator.entry_errors`, exact message) |
| --- | --- | --- | --- |
| `id` | string | required | a `Terrain.Type` id: `"Unknown terrain type '%s'."` |
| `move_cost` | int | 1 | walkable types 1..3: `"Move cost must be 1-3."`; non-walkable types ignore it (saved as 1) |
| `cover_guard` | float | 0.0 | 0.0..0.5: `"Cover must be 0.0-0.5."` |
| `hazard_status` | string | "" | "" or a `StatusRegistry` debuff: `"Hazard status '%s' is not a debuff."`; only on walkable types: `"Only walkable terrain can carry a hazard."` |
| `hazard_potency` | int | 100 | 1..100: `"Hazard potency must be 1-100."` |
| `hazard_magnitude` | int | 0 | inside the status's registry range (0 when it has none): `"%s needs a magnitude %d-%d"` / `"magnitude only applies to %s"` (same texts as skills) |
| `hazard_rounds` | int | 1 | 1..5: `"Hazard rounds must be 1-5."` |
| `hazard_on` | enum `enter`, `round_end`, `both` | `both` | one of three: `"Hazard trigger must be enter, round_end or both."` |
| `origin` | `system`, `user` | `system` | every row is system (key field: `id`) |

`validate_all()` adds `terrain:missing` when a type has no row; until S12T-A writes the file, `warnings_all()` reports missing rows instead and the code defaults below apply (no error, so `validate_all` stays `{}` between commit 1 and S12T-A).

Code defaults when a row is missing (today's INFO values, so behaviour equals the shipped data): `move_cost` 2 for sand, snow, shallow_water, mud, rubble; 1 otherwise; `cover_guard` 0.0; no hazard.

### Inputs

`DamageCalculator.compute(..., include_guard := true, cover_ratio: float = 0.0)`: guard term = `minf(GUARD_CAP, (guard if include_guard else 0.0) + cover_ratio)`; new structural constant `GUARD_CAP: float = 0.9` (PLACEHOLDER: never fully negates a hit). Result dict gains `"cover_ratio"`; the breakdown text adds `"cover -%d%%"` when > 0.

## Rules (logic lane)

1. Path cost: `GridPath.find_path` becomes uniform-cost search; entering a cell costs its `move_cost` (exit goals cost 1). Order among equal-cost routes: fewer steps, then the destination nearest `CombatGrid.CENTER`, then `DIRS` order (today's tie rules). Friend-vacates-tile is still evaluated fastest-first (HISTORY row 16). `CombatPlan.find_path` keeps delegating.
2. Move blocks: a move block has a budget of `move_per_block()` cost units. `CombatPlan._add_move` chunks the step list by cost: a chunk takes tiles while the summed cost stays within the budget; the FIRST tile of a chunk is always taken even if it costs more than the budget (minimum progress, so mud never roots). The same chunking applies to `WalkCommand._repath` and to the planner reach (`move_per_block() x MAX_MOVE_BLOCKS` becomes a cost budget over the cost-weighted search). AP per tile walked is unchanged (cost changes tiles per block, not AP).
3. Dash (`dash_tiles`) is a cost budget with the minimum-progress rule; disengage (`jump_tiles`) ignores move cost but its landing tile's `enter` hazard applies.
4. Cover (`CoverRules.cover_for(grid, attacker_pos, target_pos, is_ranged) -> float`): 0.0 for melee (`is_ranged` false). Else the larger of (a) `cover_guard` of the target's tile and (b) directional cover: `dx = sign(attacker.x - target.x)`, `dy = sign(attacker.y - target.y)`; candidate tiles are `target + (dx, dy)` when one of dx, dy is 0, else both `target + (dx, 0)` and `target + (0, dy)`; a candidate counts only if it is not walkable (wall, tree, deep water, lava); (b) is the largest counting `cover_guard`. An adjacent ranged attacker gets no directional cover from its own tile (walkable).
5. Attack and area commands pass `CoverRules.cover_for(...)` as `cover_ratio`; area hits use each hit unit's tile and the caster's position. The planner estimate (`include_guard` false) also passes cover (cover is known ahead). Riposte (melee) never sees cover.
6. Hazards (`HazardRules`): on `enter` (or `both`), when a unit's walk, dash or disengage enters a hazard tile, the tile's status is rolled once per unit per command per status: `StatusEffects.try_inflict(unit, hazard_status, hazard_potency, rng, hazard_rounds, hazard_magnitude, "terrain")` (resistance applies; next-round timing as for every queued status). At round end (`round_end` or `both`), after the S12S-L round-end ticks, every active unit standing on a hazard tile is rolled the same way. Source name `"terrain"` (sim metrics credit `dot_dealt` to no unit). Log: `"[Hazard] %s is %s by %s"` % [unit, status display, terrain name] on success.
7. Planner preview: a planned path through a hazard tile shows a warning chip `"Hazard: <terrain name> (<status>)"`; the expected hit shows cover.
8. AI (smart path): (a) path search for AI moves adds `AI_HAZARD_PENALTY` (placeholder 3) cost per hazard tile entered, through an optional `penalty: Callable` param of `find_path` (players see true costs); (b) when kiting or picking a firing cell, among equally ranked cells prefer the one with the larger `cover_for` against the nearest ranged foe; (c) target ranking already uses the estimate, so covered targets rank lower.
9. Zones (code, this task): add hazard tiles to three existing zones without touching the arena core (columns 2-6, rows 2-6) or the row-4 exit lane: coordinates are (x, y) = (column, row) and replace walkable tiles only: `lava_cavern` gets `f` at (3,1), (1,3), (5,7) (floor today); `swamp_bog` gets `p` at (0,5) (grass) and (7,5) (mud); `frozen_pass` gets `i` at (1,2), (7,3), (8,5) (snow). Encounters using `stone_arena` (the sim and most tests) are unaffected.
10. `TileArt`: one drawing per new type (4 variants by the existing scatter rule): burning ground (dark soil, orange ember dots), toxic bog (green-grey mud, purple bubbles), ice (pale blue, white cracks).

### Commits

1. Schema (releases S12T-A): new types, ids, catalog kind, fields, validator, code defaults, `describe()` from the catalog, `compute` cover input (0.0 everywhere, no behaviour change), suite part A.
2. Rules: path cost, move chunking, dash and disengage, cover, hazards, previews, zone glyphs, TileArt, suite part B.
3. AI: rule 8, suite part C.

## Acceptance checks (rule-derived; `terrain_rules_test_suite.gd`)

1. Every `Terrain.Type` has an id, `from_id(id_of(t)) == t`; existing ordinals are unchanged (FLOOR 0 ... RUBBLE 13).
2. Each validator message fires for one bad row; a walkable row with `hazard_status: burn`, magnitude 5 is valid; a WALL row with a hazard is refused.
3. With no catalog file, `move_cost` of sand is 2 and of grass 1 (code defaults equal to the old INFO values).
4. On a corridor floor, mud, floor with budget 1 per block: the unit enters the mud tile in its block (minimum progress) and needs 3 blocks for 3 tiles; with budget 2 it takes floor then stops before mud (cost 1 + 2 > 2) and enters mud next block.
5. Among two equal-length routes, the cheaper one is chosen; among equal-cost routes, the old tie order holds (pinned against the BFS result on an all-floor map: identical paths).
6. Ranged hit on a target standing on rubble with `cover_guard` 0.25: damage equals `compute(..., cover_ratio = 0.25)`; the same attacker in melee deals the uncovered amount; a target behind a wall (attacker due east, wall on the target's east neighbour) gets the wall's cover; a diagonal attacker gets the larger of the two orthogonal neighbours.
7. `GUARD_CAP`: a defending target (guard 0.8) with cover 0.25 takes `(1 - 0.9)` of the pre-guard hit.
8. A unit walking through two burning tiles in one move is rolled once; standing on one at round end is rolled again; the burn starts next round.
9. Disengage over mud lands with no cost check; landing on toxic bog rolls poison.
10. The sim on `stone_arena` (`tools/fc.sh sim duel 5`) equals the pre-change result; all existing suites pass (paths on all-floor maps unchanged).

## Out of scope

Skill-created terrain (S12_RULES Q-T3, later slice), line of sight, height, forced movement into hazards, terrain on the world or region map (S9), editing zone maps as data (S9 decides), new zones, terrain numbers (S12T-A).

## Open questions

None (Q-T1 catalog kind, Q-T2 target tile plus directional neighbour, Q-T3 static only: answered).
