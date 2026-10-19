# EXP1-L: Region maps and exploration runtime (S9)

Status: spec (rules approved: DECISIONS world-campaign "S9-S11 baseline answers (user 2026-10-17)"; this build spec awaits the user's approval)
Depends on: M4-L merged (slice 2: `QuestRunner`, `RewardGrant`, `GameState.loot_claimed`, save v2), M3
Lane: logic (one branch; commit 1 is the schema slice that releases EXP1-A)
Touches: new `scripts/exploration/{map_catalog,map_rules,explore_runner,battle_bridge,game_encounter}.gd` + `README.md`, new `scripts/game_state/game_rolls.gd`, new `data/maps/maps.json` (empty envelope) + `data/maps/README.md`, `scripts/world/world_schema.gd` (location fields, world field `start_location`), `scripts/world/world_catalog.gd` (`_location_errors` dispatch, `manifest_errors`, `_json_refs_to` location part), `scripts/game_state/{game_state,save_game,save_migrations,new_game}.gd`, `scripts/quests/reward_grant.gd` (one helper `claim_loot`), new `scripts/tests/exploration_test_suite.gd` + one line in `suite_registry.gd`, `scripts/tests/save_test_suite.gd` (version 3), new `scenes/overworld/region_map_debug.tscn` + `scripts/ui/overworld/region_map_view.gd`. Reads `PartyBuilder`, `LevelUp`, `WorldEncounter`, `CombatSession` (no edits).

Rules source: `games/emberwake/docs/specs/S9_RULES.md` (Q1-Q6 answered A, A, A, A, A, A). Section numbers below refer to this file.

## Goal

A walkable top-down tile map per location: lore-free map templates, world bindings (zones, markers), single-step movement with a fixed step order, step-counter fights in DANGER zones, placed and patrolling encounters, chests, lore, points and exits, location entry hooks into M4, a shared battle bridge (also used by TRAV1 and DLG1), one deterministic random stream per save, and save version 3.

## 1. Schema contract (schema-first: yes)

### 1.1 Map templates: `data/maps/maps.json` (`MapCatalog`)

Envelope `{"kind": "map_templates", "version": 1, "entries": [...]}`; commit 1 ships `entries: []`. `MapCatalog` has the `QuestCatalog` contract: `get_entry`, `has`, `ids`, `entries`, `upsert(entry, replace_id := "")`, `remove`, `validate_entry`, `validate_all`, `warnings_for`, `warnings_all`, `save`, `use_path`, `clear_cache`, save-blocked when the file loaded with errors. Coordinates: `x` = column, `y` = row, `(0, 0)` top-left.

| Field | Type | Default | Validation (`MapRules.template_errors`, message) |
| --- | --- | --- | --- |
| id | String | required | `JsonUtil.id_ok`, unique ("Map id '%s' is already used.") |
| name | String | required | non-empty after strip ("Name is required.") |
| rows | Array[String] | required | `MIN_SIDE` 8..`MAX_SIDE` 64 rows, each 8..64 chars, all equal ("Rows: 8..64 rows of 8..64 tiles, all the same length."); chars only from `MapRules.TILES` ("Row %d: unknown tile '%s' (use . # ~).") |
| zones | Array[String] | [] | [] = every tile `-`; else the same size as `rows` ("Zones must match the rows (%d x %d)."); each char is `-` or a legend key ("Zone char '%s' at (%d, %d) is not in the legend.") |
| zone_legend | Dictionary | {} | key = one char, not `-` ("Zone legend keys are single characters other than '-'."); value exactly `{"id", "kind"}`, id `id_ok` and unique, kind `safe` or `danger` ("Zone '%s': kind must be safe or danger.") |
| anchors | Array[Dictionary] | required | each exactly `{"id", "x", "y"}`, id `id_ok`, unique, at most `MAX_ANCHORS` 64; on an open `.` tile ("Anchor '%s' is not on an open tile."); one anchor `spawn` ("Every map needs a 'spawn' anchor.") |
| paths | Dictionary | {} | key `id_ok`; at most 16 paths; value 1..`MAX_PATH` 32 points `[x, y]`, each on an open tile, consecutive points orthogonally adjacent ("Path '%s' step %d is not next to the step before.") |
| tags | Array[String] | [] | strings |

Any other key = error ("Unknown map field '%s'."). `MapRules.TILES` = `{".": "open", "#": "wall", "~": "water"}`; only `.` is walkable (water is a render-only wall). Zone `-` = no zone = SAFE.

### 1.2 World layer: location fields (`world_schema.gd` `_location_fields`, additive)

| Field | Type (schema) | Default | Validation (`MapRules.location_errors(world_id, e)`, called from `_location_errors`) |
| --- | --- | --- | --- |
| map | `string` | "" | "" = no walkable map (menu location); else an existing `MapCatalog` id ("Unknown map template '%s'."); with "" both fields below must be empty ("Zone bindings and markers need a map.") |
| zone_bindings | `json` | {} | object zone id -> binding (1.3) |
| markers | `json` | [] | Array (max `MAX_MARKERS` 64) of markers (1.4) |

World manifest field (`_world_fields`): `start_location` `ref` locations, optional, default "": where `ExploreRunner.begin` enters a new game. "" = the nearest ancestor's value (lineage order); `manifest_errors` resolves it ("Start location '%s' is unknown.").

### 1.3 Zone binding

Keys exactly `encounters`, `elites`, `elite_pct`, `steps_min`, `steps_max` (others: "Zone binding '%s': unknown key '%s'.").

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| (zone id) | | | a zone id of the template legend ("Zone '%s' is not in map '%s'."); a `safe` zone = error ("Zone '%s' is safe: safe zones roll nothing.") |
| encounters | `[[encounter ref, weight]...]` | [] | max 8; ref resolves as a world encounter from the location's world (qualified allowed); weight int 1..100; no duplicate ref |
| elites | same | [] | same rules |
| elite_pct | int | 10 | 0..100 |
| steps_min | int | 12 | 1..99 |
| steps_max | int | 20 | 1..99 and >= `steps_min` ("steps_min is above steps_max.") |

`encounters` and `elites` both empty = error ("Zone binding '%s' binds no encounter."). Warning (`MapRules.warnings(world_id)`): a DANGER zone of the template with no binding.

### 1.4 Markers (`MapRules.MARKER_KINDS`)

Common keys: `id` (`id_ok`, unique in the location), `kind`, `anchor` (an anchor of the template; required for every kind except `roamer`, which takes none; "Marker '%s': unknown anchor '%s'."; one marker per anchor; `spawn` holds no marker), `requires_flags`, `unless_flags`, `sets_flags` (each `QuestRules.condition_errors` shape, default []). Unknown key for the kind = error ("Marker '%s' (%s): unknown key '%s'.").

| Kind | Keys (default) | Validation | Solid |
| --- | --- | --- | --- |
| exit | `to` (required), `to_anchor` ("spawn"), `locked_text` ("The way is shut.") | `to` equals the `to` string of one entry of the location's `exits`, or is `world_map` ("Exit '%s' is not in this location's exits."); when `to` resolves to a location with a map, `to_anchor` is an anchor of that map, else it must be `spawn` | only when locked (2.1) |
| encounter | `encounter` (required), `trigger` (`touch`; or `proximity`), `radius` (1), `once` (true) | encounter resolves; radius 0..3 | no |
| roamer | `encounter` (required), `path` (required), `radius` (0), `once` (true) | path is a template path ("Unknown path '%s'."); radius 0..3 | no |
| npc | `character` (required) | resolves as a world character | yes |
| chest | `items` ({}), `loot` ("") | items: base item id -> int 1..99, max 8 keys; loot: a world loot ref whose `reward_kind` is `item` ("Chest loot must be an item entry."); items empty and loot "" = error ("Chest '%s' holds nothing.") | yes |
| lore | `lore` (required) | resolves as a world lore entry | yes |
| point | (none) | | no |

Warning: an `exits` entry with no exit marker in a location that has a map ("Exit to '%s' has no marker on the map."). Ref scans (`_json_refs_to`, location part): encounter refs in bindings and markers, marker `character`, `lore`, `loot`, and exit `to` (when not `world_map`), so rename/delete keeps them correct.

### 1.5 System flags (set only by the runtime; authors and quests may read them)

| Flag | Set when |
| --- | --- |
| `<location qid dotted>.<marker id>.done` | a `once` encounter/roamer fight is won; a chest is opened |
| `<location qid dotted>.visited` | the party enters the location (any cause); pulled forward from S10 2.4 so TRAV1 fog works on EXP1 saves |

`<qid dotted>` = the qualified id with `:` replaced by `.` (`ashlands:ruined_courtyard` -> `ashlands.ruined_courtyard`); `MapRules.done_flag(location_qid, marker_id)` and `MapRules.visited_flag(location_qid)` build them.

### 1.6 GameState keys and save version 3

Added to `GameState.KEYS`; `location_id` now always holds a QUALIFIED location id (`ashlands:warden_keep`) or "".

| Key | Type | Default | `GameState.validate` |
| --- | --- | --- | --- |
| position | Array `[x, y]` | `[-1, -1]` | two ints >= -1 ("position must be [x, y].") |
| explore | Dictionary | `{"steps": 0, "next_at": 16, "loc_steps": 0, "last_safe": ""}` | exactly these keys; steps, loc_steps int >= 0; next_at 1..99; last_safe "" or a qid ("explore must be {steps, next_at, loc_steps, last_safe}.") |
| rng_seed | int | 0 | 0..2147483647 |
| rolls | Dictionary | {} | stream (`id_ok`) -> int >= 0 |

`SaveGame.VERSION` = 3; built-in migration step 2 -> 3 adds the defaults and `rng_seed = String(JSON.stringify(state, "", true)).hash() & 0x7FFFFFFF` (deterministic per file); an old local `location_id` is qualified with `world_id` when it resolves, else set to "". `NewGame.create(world_id, units, seed := -1)` sets `rng_seed` (`seed` >= 0 is used as given; -1 = from the system clock).

## 2. Rules (logic lane)

### 2.1 Presence and solidity

A marker is present when every `requires_flags` holds, no `unless_flags` holds and (for `once` encounter/roamer) its done flag is not set. Exception: an exit whose `requires_flags` fail but `unless_flags` pass is a LOCKED gate: present, solid, refuses the step with `locked_text`. Absent markers are not drawn, not solid and never trigger. An opened chest stays present and solid and is shown empty. DLG1 adds "npc absent when its character is gone".

### 2.2 Step (`ExploreRunner.step(gs, dir: Vector2i) -> Dictionary`)

`dir` is one of the four orthogonal unit vectors (else error). Returns `{ok, errors, events: Array[Dictionary]}`; events have `kind` in `moved`, `blocked`, `locked`, `exit`, `flags_set`, `fight`, `interact_hint`. Order:

1. Target tile out of bounds, not `.`, or holding a present solid marker: `blocked` (or `locked` with the text); nothing else changes (no counter, no roll, no roamer move).
2. Move the token (`moved`). A present `point` on the tile applies its `sets_flags` (`flags_set`); a present exit applies its `sets_flags`, then leaves (2.5) and the step ends (`exit` event with `to` and the entry events).
3. `explore.loc_steps += 1`; if the tile's zone is DANGER and bound, `explore.steps += 1` (SAFE tiles and unbound DANGER tiles neither count nor reset).
4. Roamers take their position for the new `loc_steps` (2.3).
5. Trigger, first match only (at most one fight per step): a present `touch` encounter marker on the tile; then present roamers in marker order on the tile or within `radius` (Manhattan); then present `proximity` encounter markers in marker order within `radius`; then the counter: `steps >= next_at` on a bound DANGER tile picks from that tile's binding (2.4). A marker or roamer fight does not also roll the counter.
6. `interact_hint` for a present npc / chest / lore marker orthogonally adjacent.

`fight` event: `{kind: "fight", source: "marker" | "roamer" | "counter", encounter: <encounter qid>, marker: <id or "">, elite: bool, from: [x, y] (the tile before the step), location: <qid>}`. The caller starts the battle at once; saving is refused while a fight is pending.

### 2.3 Roamers

Path of `L` points: `L == 1` = always point 0; else period `P = 2L - 2`, `t = loc_steps mod P`, index `t` if `t < L` else `P - t`. No per-roamer saved state; a roamer is not solid (contact = the token on its tile or within radius after step 4).

### 2.4 Rolls and the counter (`GameRolls`, `scripts/game_state/game_rolls.gd`)

Roll n of stream s (n = `gs.rolls.get(s, 0)`): a `RandomNumberGenerator` seeded with `hash([gs.rng_seed, s, n])`, one `randi()` draw, then `gs.rolls[s] = n + 1`. API: `roll(gs, s) -> int`, `range_i(gs, s, lo, hi) -> int` (inclusive), `chance(gs, s, pct) -> bool` (always consumes one roll; true when `range_i(0, 99) < pct`), `pick(gs, s, pairs) -> String` (one roll, weighted, pairs in authored order). Exploration uses stream `explore`; TRAV1 uses `travel`.

Counter pick: elites non-empty and encounters non-empty: `chance(elite_pct)` then `pick` from the chosen list; only elites: `pick(elites)` (elite true); only encounters: `pick(encounters)`.

After every fight (any source, any outcome): `explore.steps = 0`; `next_at = range_i(steps_min, steps_max)` of the bound DANGER zone under the party's final tile (after a flee or wake), else `range_i(12, 20)` (`MapRules.DEFAULT_STEPS_MIN/MAX`).

### 2.5 Entering, leaving, menus

`ExploreRunner.enter(gs, location_ref, anchor := "spawn") -> Dictionary` (`{ok, errors, events}`; errors change nothing):

1. Resolve the location from `gs.world_id` (overrides apply; qualified refs allowed); a qualified ref `w:id` that `gs.world_id` cannot reach resolves in world `w` (TRAV1 arrivals over a route owned by a connecting world, 2.1 of TRAV1_L). When the location's world is not in `WorldCatalog.lineage(gs.world_id)`, set `gs.world_id` to that world (a `connects` crossing).
2. `location_id` = its qid; `position` = the anchor tile (unknown anchor = error), or `[-1, -1]` without a map; `explore.loc_steps = 0`.
3. Set the `visited` flag; when the location has no map or its `spawn` tile is SAFE, `explore.last_safe` = the qid.
4. `QuestRunner.refresh(gs)` (auto-start quests), then `QuestRunner.on_location_reached(gs, qid)` (every entry counts, re-entries too).
5. Events: `entered` `{location, advanced quests}` and `autosave` (the scene writes the save; the runner does no I/O). Entering never fires markers or triggers.

`begin(gs)` enters the world's resolved `start_location` (error when ""). `resume(gs)` after a load: no hooks, no autosave; a position out of bounds or not open becomes `spawn`. `take_exit(gs, to, anchor := "spawn")`: `to` must be the `to` of one `exits` entry of the current location; enters it at `anchor` (the menu path for locations without a map; exit markers call it with their `to_anchor`). `menu(gs) -> Array[Dictionary]` lists `{to, label}` of the exits of a map-less location ([] with a map). `world_map` exits return `{kind: "exit", to: "world_map"}` without changing state (TRAV1 consumes it).

`interact(gs, dir) -> Dictionary`: the marker on `position + dir`. npc: `{kind: "npc", character: qid}` (until DLG1 the scene shows the name). chest: first open grants `items` plus the loot item when the loot entry is unclaimed and its `requires_flags` hold (folded into one `RewardGrant.apply(gs, {"items": ...}, "")`, then `RewardGrant.claim_loot(gs, loot_qid)`), sets the done flag; later opens return `{kind: "chest", empty: true}`. lore: `{kind: "lore", lore, name, text}`. Every successful interaction applies the marker's `sets_flags`. Flags always go through `QuestRunner.set_flag` (so `refresh` runs).

`RewardGrant.claim_loot(gs, qid) -> String` (new helper, unless slice 2 already has one): appends a unique qid and keeps `loot_claimed` sorted; "" or an error text.

### 2.6 Battle bridge (`scripts/exploration/battle_bridge.gd`, exploration-agnostic)

- `BattleBridge.encounter_for(gs, encounter_qid, zone := "") -> Dictionary` `{ok, errors, encounter: GameEncounter}`. `GameEncounter` (extends `WorldEncounter`) replaces the authored `party` with `PartyBuilder.build_active(gs.party)` (placed by the existing ally placement for that many allies) and uses `zone` when non-empty (TRAV1 route biome).
- `BattleBridge.outcome(session) -> String`: `won` (combat over, a party member stands), `lost` (combat over, none stands), `fled` (round limit reached with both sides standing; see Open questions).
- `BattleBridge.enemy_levels(world_id, encounter_qid) -> Array[Dictionary]`: `{level, tier}` of `enemies` + `wave_units` via `WorldFactory.unit_entry`.
- `BattleBridge.apply(gs, encounter_qid, outcome) -> Dictionary` `{ok, errors, xp, advanced}`: `won` = `LevelUp.award_encounter(gs.party, enemy_levels)` then `QuestRunner.on_encounter_won(gs, qid)`; other outcomes change nothing here. Full heal is implicit (M3 stores no HP).

`ExploreRunner.after_fight(gs, fight, outcome) -> Dictionary`: `BattleBridge.apply`; won: done flag when the marker is `once`, then the marker's `sets_flags`; fled: `position = fight.from`; lost: `wake(gs)`; then the counter reset (2.4).

`ExploreRunner.wake(gs, at := "")` (defeat rule, also used by TRAV1 and DLG1): target = `at` or the current location; when it has no map or its spawn is SAFE, enter it at `spawn`, else enter `explore.last_safe`, else `start_location`. No other loss.

### 2.7 Saving rule

Save anywhere outside battle and dialogue (the scene enforces it), autosave on every `entered` event. The runner owns no "in battle" state.

## 3. Acceptance checks (rule-derived, `exploration_test_suite.gd`; fixtures via `MapCatalog.use_path` + `TestHelpers.sandbox`/`fork_worlds`, never shipped data)

Checks 1-5 ship with commit 1, 6-20 with commit 2.

1. Template: 7 rows, a 65-char row, unequal rows, char `x`, anchor on `#`, missing `spawn`, duplicate anchor, path with a diagonal step, path point on water, zone grid of a different size, unknown zone char, legend key `-`, kind `calm`, unknown key: one error each; a valid template has none; `validate_all()` = {}.
2. Bindings: unknown zone, safe zone bound, weight 0 and 101, unknown encounter, `steps_min 21 > steps_max 20`, empty binding, unknown key: errors; an unbound DANGER zone is a warning only.
3. Markers: unknown anchor, two markers on one anchor, a marker on `spawn`, roamer with an anchor, unknown path, exit `to` not in `exits`, `to_anchor` unknown on the target map, chest with nothing, chest with a passive loot entry, radius 4, unknown key per kind: errors; `map: ""` with markers: error.
4. Manifest `start_location` unknown: error; a child world with "" inherits the parent's.
5. Ref scans: renaming an encounter used in a binding and a marker rewrites both; deleting a lore entry reports the lore marker.
6. Step order: walking into `#`, out of bounds and onto an npc leaves `loc_steps`, `steps`, `rolls` and roamers unchanged; a locked exit returns `locked` with its text.
7. Counter: SAFE steps change neither `steps` nor `next_at`; DANGER steps count; with `next_at = 3` the third DANGER step fights; SAFE steps in between do not reset; an unbound DANGER zone never counts.
8. One fight per step: a touch marker and a ripe counter on the same step give the marker fight and leave `steps` unreset until `after_fight`.
9. Priority: touch marker before a roamer in radius before a proximity marker.
10. Determinism: two copies of a state stepping the same inputs give equal events; save, load (`SaveGame`) and repeat give the same fights; `rolls.explore` rises by exactly the rolls drawn.
11. Elite rule: with `elite_pct 100` every counter fight is elite; with 0 none; elites only = always elite.
12. Roamer: path of 4 points visits indices 0,1,2,3,2,1,0 for `loc_steps` 0..6; `L = 1` stays; re-entering resets `loc_steps` and the roamer to index 0.
13. `once`: a won marker fight sets the done flag and the marker is absent; after `fled` or `lost` it stays; `once: false` with `unless_flags` disappears only when the flag is set.
14. Flee and defeat: fled puts the token on `from`; lost in a location with a SAFE spawn wakes at its spawn; with a DANGER spawn at `last_safe`; counter reset to 0 with a new `next_at` in range.
15. Entry: `enter` sets `visited`, calls `on_location_reached` once per entry (re-entry counts again), refreshes auto-start quests first, emits one `autosave`; entering never fights even onto a proximity radius.
16. Exits: stepping on an exit moves to the target at `to_anchor`; a qualified exit into a connected world sets `world_id`; an inherited location keeps the child `world_id`; a `world_map` exit leaves the state equal.
17. Chest: first open grants items and the unclaimed loot and claims it (sorted); a chest whose loot was already claimed by a quest grants items only; a second open is empty and grants nothing.
18. Point and lore `sets_flags` reach a quest flag objective (the quest advances in the same call).
19. Bridge: `apply(won)` raises each active member's XP by `XpCurve.encounter_xp` of `enemy_levels` and advances a waiting `win_encounter` quest; `lost` and `fled` change nothing; `encounter_for` holds exactly the active members as allies.
20. Save v3: a v2 envelope migrates with the defaults, a qualified `location_id` and the same `rng_seed` on two runs; bad `position`, `explore` with an extra key, negative `rolls` fail `validate` and `write`; `resume` fixes a position on a wall to `spawn`.

## 4. Commit slices (one branch)

1. Schema (releases EXP1-A): `MapCatalog`, `MapRules` (constants, template, binding, marker and location validation, `warnings`, flag builders, roamer index), empty `data/maps/maps.json`, location fields, `start_location`, ref scans, checks 1-5, READMEs.
2. Runtime: `GameRolls`, `ExploreRunner`, `BattleBridge`, `GameEncounter`, `RewardGrant.claim_loot`, GameState keys, save v3 + migration, `NewGame` seed, checks 6-20, `save_test_suite` update.
3. Debug scene: `scenes/overworld/region_map_debug.tscn` + `scripts/ui/overworld/region_map_view.gd`: grid, zone tint, markers, token, arrow keys = `step`, E = `interact` in the facing direction, a fight panel with three debug outcome buttons (won / fled / lost -> `after_fight`), autosave to debug slot 9, a world picker that calls `NewGame.create` + `begin`. Renders and calls the runner only; launching the real combat arena is out of scope.

## 5. Proposed STRUCTURE.md rows (planner adds)

- `scripts/exploration/` (S9, EXP1): MapCatalog, MapRules, ExploreRunner, BattleBridge, GameEncounter (placeholder -> in progress).
- `data/maps/` (S9): `maps.json` lore-free map templates.
- `scripts/game_state/game_rolls.gd` (S9, S10): the per-save deterministic random stream.
- `scripts/ui/overworld/` + `scenes/overworld/` (S9, S10, S11): debug region map, world map and dialogue/shop panels.
- `scripts/tests/exploration_test_suite.gd` (S9).

## Out of scope

Chase AI and detection cones, stealth, terrain or carry-over HP, bridges and switches, per-world tile patches (S9 Q1 C), mob level scaling, a save/load menu (UI1), launching the real battle from the debug scene, gold in chests (DLG1), dialogue (DLG1), the world map (TRAV1), any shipped data (EXP1-A).

## Open questions

- X1. Does a battle that hits its round limit with both sides standing count as `fled` (the only party-escape signal the session has today)? R: yes; alternative: treat it as `lost`.
