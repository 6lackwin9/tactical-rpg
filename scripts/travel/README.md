# scripts/travel

System: S10 (world map and travel). Status: in progress (TRAV1-L commits 1-3: schema, runtime, debug scene; TRAV2-L supplies). Spec:
`docs/specs/TRAV1_L.md`, `docs/specs/TRAV2_L.md`; rules: `docs/specs/S10_RULES.md`.

## Files

- `travel_rules.gd` (`TravelRules`): pure rules over the world kind `routes`.
  - `route_errors(world_id, e)`: route validation (called by `WorldCatalog.validate_entry`); `shape_errors(e)`: the
    blocking list-type rules (called by `WorldCatalog.blocking_errors`).
  - `warnings(world_id)`: a chance with no encounters or elites; a route into another world with no way back.
  - `graph(world_id)`: the world's merged routes plus the two-way routes of connected worlds that touch a location
    it can see; each copy carries `_from` / `_to` (endpoint qids as seen from `world_id`).
  - `nodes(world_id)`: every endpoint qid of the graph (first-seen order); world map nodes are derived, not authored.
  - Runtime reads: `is_open`, `legs` (allowed directions of open routes), `known` (fog: visited, or one open leg from
    a visited node), `path` (least time, then fewest legs, then the route qid list), `leg_chance(gs, route, starved)` /
    `repeat_chance` / `starved_chance` (TRAV2-L: x`STARVED_PCT` 150 / 100, rounded down, capped at 100),
    `start_supplies(world_id)` (nearest manifest `start_supplies` >= 0, else 10), `route_in`, `view` (what the world
    map draws, with `supplies`), and the flag names `traversed_flag` / `event_done_flag`.
- `travel_runner.gd` (`TravelRunner`): `travel(gs, to_ref)` runs the legs of a trip (clock, scripted fight, road roll on
  the GameRolls stream `travel`, leg completion through `ExploreRunner.enter`); a fight stops the trip and the caller
  plays it, then calls `after_fight(gs, fight, outcome)` (won completes the leg, fled stays, lost wakes at the leg's
  start). One `autosave` event at the end of a trip, none while a fight is pending; the remaining path is not saved.
  Supplies (TRAV2-L): per leg, `starved` = supplies 0 at the leg start, then the clock, then
  `spend_supplies(route.time)` (clamped at 0; travel is never refused); scripted fights ignore supplies, the road roll
  uses the starved chance (same number of draws). A fled or lost leg keeps its supplies spent; a won fight does not
  charge again. Results carry `supplies_spent` and `starved_legs`.

## Data

Routes live in `data/worlds/<id>/routes.json` (inherited and overridable like every world kind; a missing file is an
empty kind). Fields: `from`, `to` (location refs; `to` may be qualified into a connected world), `two_way`, `time`
(1..20), `zone` ("" or a combat zone), `encounter_pct`, `repeat_pct` (-1 = half), `encounters` / `elites`
(`[[encounter, weight]...]`, as in EXP1 zone bindings), `elite_pct`, `scripted` (one-shot road fights),
`requires_flags`, `unless_flags`.

Related rules elsewhere: `MapRules.world_map_exit_errors` (a location with a `world_map` exit must be a node),
`WorldCatalog._route_json_refs` (encounter refs inside the route lists, for rename and remove).

State: `GameState.time` (world clock, save version 5); discovery, traversal and scripted progress are flags
(`<location>.visited`, `<route>.traversed`, `<route>.<event>.done`). `ExploreRunner.menu` lists a `world_map` entry
on map-less nodes.

## Debug scene

`scenes/overworld/world_map_debug.tscn` + `scripts/ui/overworld/world_map_view.gd` (`WorldMapView`): known nodes as
pins on an automatic circle layout (graph node order), routes as lines (closed dashed, one-way with an arrow, the
travel time at the middle), click a pin = `TravelRunner.travel`, the road fight panel (real arena, debug outcomes as a
fallback) and the clock. The region map (`RegionMapView`) opens it on a `world_map` exit (marker or menu entry) and
gets it back on arrival, after a lost road fight, or on Back. A `world_map` exit marker's `sets_flags` apply when the
party steps onto it.

Tests: `scripts/tests/travel_test_suite.gd` (checks 1-4), `travel_runtime_checks.gd` (checks 5-17) and
`world_map_checks.gd` (the debug scene and the region map hand-over).
