# S10 baseline rules: world map and travel

Status update (2026-10-20): answered (decision "S9-S11 baseline answers (user 2026-10-17)") and built as TRAV1. The status line below is the original.

Status: proposed (baseline rules, design-brainstorm 2026-10-17; nothing approved; the open questions in section 3 block the TRAV1 spec)
Depends on: EXP1-L (S9: location entry, `BattleBridge`, deterministic rolls, defeat rule, save v3), M4-L (flags, `QuestRunner.on_location_reached`)
Lane: design (this file); later logic + asset (section 4)
Touches (later build, not this file): new `scripts/travel/*`, `scripts/world/world_schema.gd` + `world_catalog.gd` (new world kind `routes`), `scripts/game_state/*` (save bump), new `travel_test_suite.gd`

Source inputs: ROADMAP "Inputs for placeholder systems" (S10 bullet: node graph with an adjacency list, flat time per route, example 3 units, a fixed encounter chance per leg fought in the route's biome; later weather, hazards, supplies, caravans), DECISIONS world-campaign "Travel supplies" (reduced: single counter, no fail state, later slice), archived brainstorm P-S10 (recommended: node graph with fixed cost first, supplies later).

## 1. Scope

S10 is the overland layer: a graph whose nodes are locations and whose edges are authored routes. The player picks a destination, the party travels leg by leg, each leg spends time and may roll a road fight, and arrival enters the destination location through the S9 entry rule.

S10 is NOT:

- walking inside a location (S9) or local location-to-location exits (S9 `exits` markers);
- a free-roam overworld grid (the world map is a node graph, rendered as a picture with node pins);
- supplies in the first slice (DECISIONS "Travel supplies": later slice, reduced; sketched in 2.6 only);
- weather, road hazards, merchant caravans, mounts or vehicles (later additive rules);
- lore: routes carry gameplay numbers and refs; names and descriptions are world text, the travel rules are lore-free.

## 2. Baseline rules

### 2.1 Data model sketch: world kind `routes` (`data/worlds/<id>/routes.json`)

A new kind in `WorldSchema.KINDS`, inherited and overridable like every world kind (`merged_entries`, `override`, qualified refs). A missing file loads as an empty kind.

| Field | Type (schema) | Default | Rule (sketch) |
| --- | --- | --- | --- |
| id | id | required | unique in the world |
| name | string | "New Route" | display ("north road") |
| from | ref locations | required | resolves from the route's world |
| to | ref locations | required | may be qualified into a world named in `connects` (or an ancestor); not equal to `from` |
| two_way | bool | true | false = travel only `from` -> `to` |
| time | int | 3 | 1..20 travel units |
| zone | enum (zones) | "" | "" = each encounter's own arena zone; else the route fights use this zone (the route's biome) |
| encounter_pct | int | 20 | 0..100, chance per leg on the first traversal |
| repeat_pct | int | -1 | -1 = `encounter_pct / 2` rounded down; else 0..100 for later traversals (Q4) |
| encounters | json | [] | `[[encounter ref, weight]...]`; empty with `encounter_pct > 0` = warning |
| elites | json | [] | same shape; `elite_pct` chance replaces the pick |
| elite_pct | int | 5 | 0..100 |
| scripted | json | [] | one-shot road events: `{"id", "encounter", "requires_flags", "unless_flags", "sets_flags"}`; fires on the first leg whose conditions hold, before the random roll, then its done flag is set |
| requires_flags | json | [] | M4 conditions; failing = the route is closed (drawn as closed once known) |
| unless_flags | json | [] | any holding = closed |
| tags | tags | [] | |

Nodes are derived, not authored: a location is a world-map node when it is an endpoint of at least one route of the current world's merged routes. Validation: a location with an S9 `world_map` exit must be a node (error); a node without a `map` and without any S9 markers is allowed (menu-only stop).

Route state is not stored per route: done flags of scripted events and "traversed" markers are flags (2.4), so the flat flag registry stays the only progress store (DECISIONS "M4 quest answers").

### 2.2 Core loop

1. Open the world map: from an S9 `world_map` exit, or from a menu-only location. The current node is `GameState.location_id`.
2. Known nodes and open routes are shown (Q2). The player picks a known destination.
3. `TravelRules.path(gs, from, to) -> Array[String]` (route qids): shortest total `time` over open routes in allowed directions, known nodes only; ties broken by fewer legs, then by route qid ascending (deterministic). No path = the destination is refused.
4. `TravelRunner.travel(gs, to) -> Dictionary` runs legs in order (Q3). Per leg:
   1. `GameState.time += route.time` (Q5);
   2. scripted event check (first matching entry of `scripted`, in list order);
   3. else the random roll: chance = `encounter_pct` on the first traversal, `repeat_pct` afterwards; pick as S9 (elite chance, then weighted), RNG stream `travel` from the S9 deterministic stream;
   4. a fight goes through `BattleBridge` (S9 2.4) with the route `zone` override; travel pauses and returns `{stopped_at, leg, fight}`; after a win the player continues (same destination, remaining legs) or picks a new one;
   5. the leg's end node is entered as visited (2.4).
5. Arrival: the final node is entered through the S9 entry rule (position = its `spawn` anchor, `on_location_reached`).

Outcomes of a road fight: won = XP, `on_encounter_won`, travel may continue; fled = the party returns to the leg's start node, the leg's time stays spent; lost = the S9 defeat rule (EXP1 Q3), applied at the leg's start node.

### 2.3 Cross-world travel

A route whose `to` is qualified into a connected world moves the party there: arrival sets `GameState.world_id` to the target world, then the S9 entry rule runs. Quest keys are qualified ids and flags are one flat registry, so nothing else changes. The return trip needs a route in the target world's data (or a `two_way` route inherited by both); validation warns when a cross-world route has no way back.

### 2.4 Discovery and flags (system-derived keys, set only by the runtime)

| Flag | Set when | Used for |
| --- | --- | --- |
| `<location qid dotted>.visited` | the party enters the location (any layer: S9 exit, travel leg end, new game start) | node known (Q2), quest flag objectives, dialogue conditions |
| `<route qid dotted>.traversed` | a leg on that route completes (any direction) | `repeat_pct` switch (Q4) |
| `<route qid dotted>.<event id>.done` | a scripted road event's fight is won | one-shot events |

`<qid dotted>` = the qualified id with `:` replaced by `.` (same convention as S9 done flags); all pass `GameState.flag_key_ok`. Passing through an intermediate node counts as visiting it and calls `on_location_reached` for it (a quest "reach X" is met by travelling through X; the leg order keeps it deterministic).

### 2.5 Save

New `GameState.time` int (default 0) if Q5 = A. Nothing else: the current node is `location_id`, discovery and traversal are flags, rolls use the S9 `rolls` counter. The save bump is one migration step that only adds `time: 0`. Travel is atomic per leg: a save between legs (after a fight) stores the party at the leg's end node; the remaining path is NOT saved (the player re-picks the destination).

### 2.6 Later slice: supplies (sketch only, reduced per DECISIONS "Travel supplies")

Single counter `PartyState.supplies` (0..99); each leg costs `route.time` supplies; supplies come from shops (S11) and chests (S9). No fail state: travel is never refused. See Q6 for the effect at 0. Not part of TRAV1. Build specs: `games/emberwake/docs/specs/TRAV2_L.md` (logic, Q6 A) and `games/emberwake/docs/specs/TRAV2_A.md` (data).

### 2.7 Acceptance-check themes (the TRAV1 spec turns these into rule-derived checks)

Route validation (unknown endpoints, `from == to`, qualified `to` outside `connects`, time 0 or 21, pct above 100, bad weights, scripted entry without encounter); derived nodes; path choice (shortest time, tie by legs then qid, one-way honoured, closed routes skipped, unknown nodes skipped); per-leg time; first vs repeat chance; the same seed gives the same fights after a reload; a scripted event fires once and before the roll; fled returns to the leg start; intermediate nodes get `visited` and the quest hook; cross-world arrival changes `world_id`; save migration adds `time` only.

## 3. Open design questions (each blocks the TRAV1 spec)

Q1. What are world-map nodes?

- A. Locations that are route endpoints (sketch). Arrival lands in a concrete S9 map; reuses `location` refs that quests already bind; regions stay grouping and lore.
- B. Regions, each with an entry location. Matches the old "graph of regions" input literally; two levels of indirection (region -> entry location -> map) and regions carry no gameplay today.
- C. A new node kind separate from locations. Free placement; a third place-like kind to keep in sync.
- Recommendation: A (a region-level picture can still group the pins).

Q2. Which nodes does the player see and reach?

- A. All nodes known from the start; only flags close routes. Simple; no discovery.
- B. Fog: a node is known when visited or one open route away from a visited node; travel only to known nodes (sketch). Exploration feel with zero extra authoring.
- C. Only nodes revealed by explicit flags. Full author control; every node needs a reveal step.
- Recommendation: B (an author can still hide a node behind a route `requires_flags`).

Q3. How does a multi-leg trip run?

- A. Auto-path to a far destination, rolling per leg, pausing at each fight (sketch). Fewest clicks; one call per trip in tests.
- B. One leg per click (adjacent nodes only). Most control; tedious on long trips.
- C. A plus instant fast travel between visited nodes (no rolls). Comfort; removes road fights from the late game entirely.
- Recommendation: A; fast travel stays a later rule if Q4 does not tame repetition.

Q4. Repeated road fights on travelled routes?

- A. Same chance on every traversal. Simplest; grindy back-and-forth.
- B. First traversal `encounter_pct`, later `repeat_pct` (default half) (sketch). Keeps roads alive but calmer; one optional field.
- C. No random fights after the first traversal. Calm; road XP disappears after the first trip.
- Recommendation: B.

Q5. Keep a world clock?

- A. One integer `GameState.time`, advanced only by travel, no day/night (sketch). Cheap; supplies, restocks and timed events can read it later.
- B. No clock until a system needs it. Nothing to save now; a later save bump and migration.
- C. Day/night cycle with effects. Atmosphere; touches encounter tables and rendering.
- Recommendation: A.

Q6 (later slice, does not block TRAV1). Supplies at 0?

- A. Road encounter chance x1.5 (capped 100) while at 0. A cost without a fail state; one constant.
- B. Flavour only (a warning). No pressure; the counter has no point.
- C. Arrival without supplies starts the next fight with a weaken debuff. Real stakes; touches combat and contradicts "no fail state" in spirit.
- Recommendation: A, decided when the supplies slice is specced.

## 4. Milestone shape (coarse)

| Milestone | Lane | Goal | Depends on |
| --- | --- | --- | --- |
| TRAV1-L | logic, one branch; commit 1 = schema (kind `routes`, validation, ref scans, derived nodes), commit 2 = runtime (TravelRules path, TravelRunner legs, discovery flags, save bump, suite), commit 3 = minimal world-map scene (node pins, routes, travel button; renders only) | engine + tests + a debug scene | EXP1-L merged (BattleBridge, rolls, entry rule); answers Q1-Q5 |
| TRAV1-A | asset, one branch (seed) | Ashlands routes between the M4 sample locations (one closed by a quest flag, one scripted ambush), one cross-world route to Iron Reaches and back | TRAV1-L commit 1, EXP1-A merged |
| TRAV2 (idea) | logic + asset | supplies counter (2.6, Q6) | TRAV1, DLG1 (shops sell supplies) |

Order per ROADMAP "Suggested build order" step 5: S9, S11-A, then S10-A. TRAV1 shares hub files with EXP1 and DLG1 (`world_schema.gd`, `world_catalog.gd`, `game_state.gd`, save files), so it runs after them, not in parallel.

## 5. HISTORY check

- Row 1 (10-block combat layer, deprecated): the travel inputs came from the same early notes; only the graph, time and encounter-chance parts are kept; road fights use the current combat.
- "Enemy tier defined per world" (not adopted): route encounters are world encounters of class-derived units; the route sets only the arena zone, never stats.
- `.context/` notes (deprecated, CLN-4): their S10 parts already live in ROADMAP inputs.
- Nothing in HISTORY deprecates node-graph travel, per-leg rolls or a clock. Supplies are decided (DECISIONS "Travel supplies"), not re-proposed: 2.6 keeps the reduced form.
