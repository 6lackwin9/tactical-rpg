# TRAV1-L: World map, routes, road encounters and the world clock (S10)

Status: approved (user 2026-10-17) (rules approved: DECISIONS world-campaign "S9-S11 baseline answers (user 2026-10-17)")
Depends on: EXP1-L merged (entry rule, `visited` flags, `BattleBridge`, `GameRolls`, `ExploreRunner.wake`), DLG1-L merged (shared hub files, save v4)
Lane: logic (one branch; commit 1 is the schema slice that releases TRAV1-A)
Touches: new `scripts/travel/{travel_rules,travel_runner}.gd` + `README.md`, `scripts/world/world_schema.gd` (kind `routes`), `scripts/world/world_catalog.gd` (dispatch, `_shape_errors`, `_json_refs_to` route part), `scripts/tests/world_test_suite.gd` (kind list), `scripts/exploration/{map_rules,explore_runner}.gd` (world-map exit node check, menu entry), `scripts/game_state/{game_state,save_game,save_migrations}.gd`, new `scripts/tests/travel_test_suite.gd` + one line in `suite_registry.gd`, `scripts/tests/save_test_suite.gd` (version 5), new `scenes/overworld/world_map_debug.tscn` + `scripts/ui/overworld/world_map_view.gd`, `scripts/ui/overworld/region_map_view.gd` (opens the world map).

Rules source: `games/emberwake/docs/specs/S10_RULES.md` (Q1 A, Q2 B, Q3 A, Q4 B, Q5 A). Supplies are TRAV2 (idea).

## Goal

An overland node graph: authored routes between locations (nodes are derived), fog of war from `visited` flags, auto-pathed multi-leg trips that spend clock time and may stop for a road fight (scripted one-shots first, then a seeded roll with a calmer repeat chance), cross-world arrival, and save version 5 (`time`).

## 1. Schema contract (schema-first: yes)

### 1.1 World kind `routes` (`data/worlds/<id>/routes.json`)

Added to `WorldSchema.KINDS` (inherited, overridable, missing file = empty kind). Validation `TravelRules.route_errors(world_id, e)`.

| Field | Type (schema) | Default | Validation (message) |
| --- | --- | --- | --- |
| id, name | id, string | required, "New Route" | generic |
| from | ref locations | required | resolves from the route's world ("Choose where the route starts.") |
| to | ref locations | required | resolves (qualified into a `connects` world or an ancestor allowed, existing ref rule); not the same qid as `from` ("A route can't lead to its own start.") |
| two_way | bool | true | |
| time | int | 3 | 1..20 ("Travel time must be 1..20.") |
| zone | string | "" | "" or a `CombatZones.zone_ids()` id ("Unknown zone '%s'.") |
| encounter_pct | int | 20 | 0..100 |
| repeat_pct | int | -1 | -1 (= `encounter_pct / 2`, rounded down) or 0..100 |
| encounters | json | [] | `[[encounter ref, weight 1..100]...]`, max 8, no duplicate ref (same rules as an EXP1 zone binding) |
| elites | json | [] | same |
| elite_pct | int | 5 | 0..100 |
| scripted | json | [] | max 4; each exactly `{id, encounter, requires_flags, unless_flags, sets_flags}`; id `id_ok`, unique in the route; encounter required and resolving; the three lists are M4 conditions (default []) |
| requires_flags | json | [] | conditions; any failing = closed |
| unless_flags | json | [] | conditions; any holding = closed |

Warnings (`TravelRules.warnings(world_id)`): `encounter_pct > 0` with empty `encounters` and `elites`; a route into another world with no way back (no open-or-closed route of the graph (2.1) leads from its `to` world back to its `from` world). Error added to `MapRules.location_errors`: a location with a `world_map` exit marker that is not a node of its world's graph ("'%s' has a world map exit but no route."). Ref scans: `from`, `to`, encounter refs in `encounters`, `elites`, `scripted`.

### 1.2 State and save version 5

`GameState.time` int >= 0 (default 0; added to `KEYS`; "time must be a whole number >= 0."). `SaveGame.VERSION` = 5; step 4 -> 5 adds only `time: 0`. Discovery, traversal and scripted progress are flags (1.3); the current node is `location_id`; rolls use `GameRolls` stream `travel`.

### 1.3 System flags (set only by the runtime)

| Flag | Set when |
| --- | --- |
| `<location qid dotted>.visited` | any entry (EXP1 already sets it, including legs' end nodes) |
| `<route qid dotted>.traversed` | a leg on the route completes, either direction |
| `<route qid dotted>.<event id>.done` | a scripted road fight is won |

## 2. Rules (logic lane)

### 2.1 Graph, nodes, open and known

- `TravelRules.graph(world_id) -> Array[Dictionary]`: the merged routes of `world_id`, then the `two_way` routes of every other world X with `WorldCatalog.can_reach_world(X, world_id)` that have an endpoint resolving (from X) to a location also resolvable from `world_id` (X in world id order, routes in catalog order). This makes one authored two-way cross-world route usable from both sides.
- Nodes: every endpoint qid of the graph (`TravelRules.nodes(world_id) -> Array[String]`, first-seen order).
- A route is open when all `requires_flags` hold and no `unless_flags` holds. Directions: `from -> to` always, `to -> from` only when `two_way`.
- A node is known when its `visited` flag is set, or an open route leads to it (in an allowed direction) from a visited node. Unknown nodes are not drawn; closed routes are drawn as closed only when both endpoints are known.

### 2.2 Path (`TravelRules.path(gs, from_qid, to_qid) -> Array[String]`)

Route qids of the best path over open routes in allowed directions, every node on it known (start included). Order: least total `time`, then fewest legs, then the route qid list compared element by element (ascending). Destination equal to the start, unknown, or unreachable = [] (the caller refuses).

### 2.3 Trip (`TravelRunner.travel(gs, to_ref) -> Dictionary`)

Refused (`{ok: false, errors}`, state unchanged) unless the current location is a node and `path` is non-empty. Per leg, in order:

1. `gs.time += route.time`.
2. Scripted: the first entry in list order whose `requires_flags` hold, no `unless_flags` holds, and done flag is not set -> fight (`source: "scripted"`).
3. Else the roll: chance = `repeat` (resolved `repeat_pct`) when the route's `traversed` flag is set, else `encounter_pct`; only when chance > 0 and `encounters` or `elites` is non-empty: `GameRolls.chance(gs, "travel", chance)`; then the EXP1 pick rule (2.4 of EXP1_L) on stream `travel` with `elite_pct`.
4. A fight stops the trip: returns `{ok: true, status: "fight", fight: {source: "scripted" | "road", encounter, route, event, elite, from: <leg start qid>, to: <leg end qid>, zone}, legs_done, time}`; `BattleBridge.encounter_for(gs, encounter, route.zone)` builds it.
5. No fight: complete the leg (2.4) and continue.

After the last leg: `{ok: true, status: "arrived", location, legs_done, time, events}`.

### 2.4 Completing a leg

Set the route's `traversed` flag, then `ExploreRunner.enter(gs, leg end, "spawn")` (world switch when needed, `visited`, `last_safe`, `refresh`, `on_location_reached`; every intermediate node counts as a visit). Autosave events of one trip collapse into one at the end.

### 2.5 After a road fight (`TravelRunner.after_fight(gs, fight, outcome) -> Dictionary`)

- won: `BattleBridge.apply`; scripted: set the event's done flag, then its `sets_flags` (through `QuestRunner.set_flag`); complete the leg (2.4). The trip does not resume by itself: the caller calls `travel` again with the same or a new destination (re-pathed from the new node, deterministic).
- fled: the party stays at the leg's start node (`location_id` never changed); the leg's time stays spent; nothing else.
- lost: `ExploreRunner.wake(gs, fight.from)` (the EXP1 defeat rule applied at the leg's start node).

Saving: allowed between legs only when no fight is pending; the remaining path is never saved.

### 2.6 Menus and the world map

- Opening: an S9 `world_map` exit event, or `ExploreRunner.menu(gs)` of a map-less node, which gains a `{to: "world_map"}` entry when the location is a node.
- `TravelRules.view(gs) -> Dictionary`: `{current, nodes: [{qid, name, visited}], routes: [{qid, from, to, two_way, open, time}]}` for known nodes only; the debug scene draws it.

## 3. Acceptance checks (`travel_test_suite.gd`; forked world root, never shipped data)

Checks 1-4 ship with commit 1, 5-17 with commit 2.

1. Route validation: unknown `from`, unknown `to`, `from == to`, qualified `to` outside `connects`, time 0 and 21, `encounter_pct` 101, `repeat_pct` -2, weight 0, unknown zone, scripted without encounter, duplicate scripted id, unknown scripted key: one error each; a valid route has none.
2. Warnings: `encounter_pct 20` with no encounters; a one-way cross-world route with no way back; a two-way cross-world route has no warning.
3. A location with a `world_map` exit marker and no route = error; adding a route clears it.
4. Ref scans: renaming an encounter used in a route `encounters` and a `scripted` entry rewrites both; a child world inherits and can override a parent route.
5. Nodes and graph: endpoints only; a two-way route owned by a connected world appears in both worlds' graphs; a one-way one only in its owner's.
6. Fog: at start only the visited node and its open neighbours are known; a closed route reveals nothing; setting its flag reveals the neighbour.
7. Path: shortest time wins over fewer legs; a tie in time picks fewer legs; a full tie picks the lower route qid list; one-way routes are not used backwards; closed routes and unknown intermediate nodes are skipped; unreachable = [].
8. Time: a trip of legs 2 + 3 adds 5 to `time`; a refused trip adds nothing.
9. Chances: `encounter_pct 100` fights on the first traversal; with `repeat_pct 0` the second traversal never fights; `repeat_pct -1` with `encounter_pct 41` uses 20.
10. Scripted: fires before the roll on the first matching leg, once; after the win its done flag and `sets_flags` are set and the next traversal rolls normally; a loss leaves it armed.
11. Determinism: two equal states travelling the same trip give equal results; save, load and repeat give the same fights; `rolls.travel` rises by exactly the rolls drawn; `rolls.explore` is unchanged.
12. Intermediate nodes: a 2-leg trip through X sets `X.visited` and advances a quest waiting to reach X.
13. Fled: the party is at the leg's start, `time` includes the leg; lost: the EXP1 wake rule from the leg's start.
14. Won mid-trip: the leg completes (`traversed`, end node entered); calling `travel` again with the same destination finishes the trip.
15. Cross-world: arriving at a location of a connected world sets `world_id`; travelling back through the same two-way route restores it.
16. Menu: a map-less node lists `world_map`; a map-less non-node does not.
17. Save v5: a v4 envelope migrates to `time: 0` and nothing else changes; a negative `time` fails `validate` and `write`.

## 4. Commit slices (one branch)

1. Schema (releases TRAV1-A): kind `routes`, `TravelRules` validation, warnings, graph and nodes, the `world_map` exit error in `MapRules`, ref scans, world suite kind list, checks 1-4, README.
2. Runtime: open/known/path, `TravelRunner` (legs, scripted, rolls, `after_fight`), menu entry, `GameState.time`, save v5 + migration, checks 5-17, save suite update.
3. Debug scene: `scenes/overworld/world_map_debug.tscn` + `world_map_view.gd`: known nodes as pins on an automatic circle layout (catalog order), routes as lines (closed dashed, two-way plain, one-way with an arrow), click a node = `travel`, the fight panel with debug outcome buttons, the clock; the region map scene opens it on a `world_map` exit and returns on arrival.

## 5. Proposed STRUCTURE.md rows (planner adds)

- `scripts/travel/` (S10, TRAV1): TravelRules, TravelRunner (placeholder -> in progress).
- `scripts/tests/travel_test_suite.gd` (S10).

## Out of scope

Supplies (TRAV2), weather, hazards, caravans, mounts, fast travel, day/night, pin positions or region art for the world map (automatic layout only), saving the remaining path, restocks or timed events reading `time`, shipped data (TRAV1-A).

## Open questions

None (the cross-world visibility rule in 2.1 implements S10's "a two-way route inherited by both" for worlds linked by `connects`, which do not inherit each other).
