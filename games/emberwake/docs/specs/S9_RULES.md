# S9 baseline rules: region map and exploration

Status: proposed (baseline rules, design-brainstorm 2026-10-17; nothing approved; the open questions in section 3 block the EXP1 spec)
Depends on: M4-L merged (`QuestRunner.on_location_reached` / `on_encounter_won`, `GameState.loot_claimed`, `RewardGrant`, save v2), M3 (`PartyState`, `PartyBuilder`, `LevelUp.award_encounter`)
Lane: design (this file); later logic + asset (section 4)
Touches (later build, not this file): new `scripts/exploration/*`, new `data/maps/` (base, lore-free), `scripts/world/world_schema.gd` + `world_catalog.gd` (location fields), `scripts/game_state/*` (save v3), new `exploration_test_suite.gd`

Source inputs: ROADMAP "Inputs for placeholder systems" (S9 bullet), DECISIONS world-campaign "Exploration encounters" (step counter for mobs, rare elite chance; elites, bosses and scripted fights placed or roaming), archived brainstorm P-S9 (recommended hybrid: placed encounters plus a step counter only in DANGER zones).

## 1. Scope

S9 is the walkable map of ONE location: a 2D top-down tile grid the party token moves on in single steps, with SAFE and DANGER zones, placed and roaming encounters, exits, and interaction points (NPCs, chests, lore, gates). It produces the game-loop events that M4 already consumes (`on_location_reached`, `on_encounter_won`, flags).

S9 is NOT:

- the world map or travel between locations far apart (S10); S9 only owns local exits (location to location through an authored `exits` entry) and the "open world map" exit;
- dialogue, shops or reputation (S11); an NPC marker only hands the character ref to S11;
- a combat change: fights are ordinary `CombatSession` battles built from world encounters; no terrain, no carry-over HP (M3: full heal after battle);
- stealth, detection cones, puzzle switches, bridges, moving platforms (later additive rules from the ROADMAP inputs);
- a lore container: map geometry, zones and anchors are lore-free base data; the world layer binds them to its encounters, characters and text.

## 2. Baseline rules

### 2.1 Data model sketch (two layers, like quest templates in M4)

Base layer, lore-free: `data/maps/maps.json` (envelope `{"kind": "map_templates", "version": 1, "entries": [...]}`), loaded and validated by a new `MapCatalog` (same API shape as `QuestCatalog`: `get_entry`, `ids`, `upsert`, `validate_entry`, `validate_all`, `warnings_for`, `save`, `use_path`).

| Field | Type | Default | Rule (sketch) |
| --- | --- | --- | --- |
| id | String | required | `JsonUtil.id_ok`, unique |
| name | String | required | lore-free label ("Walled town, two gates") |
| rows | Array[String] | required | equal-length rows, 8..64 per side; chars from `MapRules.TILES`: `.` open, `#` blocked, `~` blocked (water, render only) |
| zones | Array[String] | all `-` | same size as `rows`; each char maps through `zone_legend`; `-` = no zone (treated as SAFE) |
| zone_legend | Dictionary | {} | char -> `{"id": zone_id, "kind": "safe" or "danger"}`; ids unique |
| anchors | Array[Dictionary] | required | `{"id", "x", "y"}` on open tiles; ids unique; `spawn` required |
| paths | Dictionary | {} | path id -> Array of `[x, y]` (each step orthogonally adjacent, open tiles; used by roamers) |
| tags | Array[String] | [] | |

World layer, additive fields on `locations` (`world_schema.gd` `_location_fields`; every ref resolves from the location's world, qualified refs allowed as today):

| Field | Type | Default | Rule (sketch) |
| --- | --- | --- | --- |
| map | string | "" | "" = no walkable map (the location is only a node / menu); else an existing `MapCatalog` id |
| zone_bindings | json | {} | zone id -> `{"encounters": [[ref, weight]...], "elites": [[ref, weight]...], "elite_pct": 10, "steps_min": 12, "steps_max": 20}` |
| markers | json | [] | Array of markers (below); each `{"id", "anchor", "kind", ...}`, ids unique in the location |

Marker kinds (`MapRules.MARKER_KINDS`); every marker also takes `requires_flags` / `unless_flags` (M4 condition grammar; a marker whose conditions fail is absent: not drawn, not solid, no trigger) and `sets_flags` (applied in order on use):

| Kind | Keys | Behaviour |
| --- | --- | --- |
| exit | `to` (an entry of the location's existing `exits` list, or `world_map`), `to_anchor` (default `spawn`), `locked_text` | step onto it = leave (2.4); `requires_flags` failing = a locked gate (solid, shows `locked_text`) instead of absent |
| encounter | `encounter`, `trigger` (`touch` or `proximity`), `radius` 0..3 (proximity only), `once` (default true) | placed fight (elite, boss, scripted) |
| roamer | `encounter`, `path`, `radius` 0..3, `once` (default true) | moves along a template path (2.3); fight on contact or within radius |
| npc | `character` | solid; interacting opens S11 (until S11: shows the character name only); absent when S11 marks the character gone |
| chest | `items` {base item id: count}, `loot` (optional world loot ref) | solid; interact once: stash items via `RewardGrant`, loot claimed through `loot_claimed` (already claimed = items only) |
| lore | `lore` (world lore ref) | interact: show the entry; `sets_flags` lets quests use "read it" as a flag objective |
| point | (none) | invisible goal tile: stepping on it applies `sets_flags` (used for "reach the altar inside the ruin") |

The existing location `exits` json stays the single source of truth for which locations connect; an exit marker only places one of those entries on the grid (validation: `to` must name an entry of `exits` or `world_map`; warning when a location with a map has an `exits` entry with no marker).

### 2.2 Movement and step order

The party is one token (all active members); `GameState.location_id` names the location, new `GameState.position` (`[x, y]`) the tile. One input = one orthogonal step (no diagonals); blocked tiles and solid markers refuse the step (no time passes). `ExploreRunner.step(gs, dir) -> Dictionary` is pure logic (no scene), returns the events of that step, and runs in this order:

1. Move the token. Refused moves stop here.
2. Point / exit markers on the new tile fire (an exit ends the step: 2.4).
3. Increment `explore.loc_steps` (steps in this location); if the new tile is in a DANGER zone, increment `explore.steps`.
4. Roamers advance one tile (2.3).
5. Trigger check, first match wins, at most ONE fight per step: placed `touch` encounter on the tile, then any roamer on the tile or within its radius, then placed `proximity` encounters within radius, then the step counter (2.3). A placed or roamer fight on this step does not also roll the counter.
6. Return events (`moved`, `blocked`, `exit`, `fight`, `flags_set`, `interact_hint`).

Interaction (`ExploreRunner.interact(gs)`) targets the marker on an orthogonally adjacent tile the token faces (npc, chest, lore). Distance is Manhattan everywhere.

### 2.3 Encounters

- Step counter (DANGER zones only): when `explore.steps >= explore.next_at`, a fight starts with the zone binding of the tile the party stands on. Pick: an elite with `elite_pct` chance when `elites` is non-empty, else a weighted pick from `encounters`. After the fight (any outcome), `steps` resets to 0 and `next_at` is drawn uniformly in `[steps_min, steps_max]`. SAFE tiles neither advance nor reset the counter (no cheese by stepping in and out). A DANGER zone with no binding rolls nothing (warning, not error, so a template can be reused peacefully).
- Placed encounters: fight on touch or proximity; `once: true` sets the done flag on a win and the marker is absent afterwards.
- Roamers: position = the path walked ping-pong by `explore.loc_steps` (index = `loc_steps mod (2 x len - 2)` folded back), so no per-roamer state is saved and the same steps give the same positions. Chase AI is out of scope (Q4).
- Done flags are system-derived, not authored: `<location qid with ':' as '.'>.<marker id>.done` (example `ashlands.ash_gate.gate_boss.done`; passes `GameState.flag_key_ok`). Authors and quests may read them; they are set only by the runtime.
- Randomness: one deterministic stream for the whole save. New `GameState.rng_seed` (int, set by `NewGame.create`) and `GameState.rolls` (`{stream: count}`); roll n of stream `explore` = `hash([rng_seed, "explore", n])`-seeded `RandomNumberGenerator`. Save, reload and repeat = the same fights. S10 reuses this with stream `travel`.
- Enemy level: authored in the world encounter (HISTORY "Enemy tier defined per world: not adopted" and "fixed-power goblins: deprecated" stay respected: enemies are class-derived units with a tier; worlds set the level). See Q5.

### 2.4 Battle bridge and outcomes

A new `BattleBridge` (exploration-agnostic, also used by S10 and the S11 `fight` effect; path proposed `scripts/exploration/battle_bridge.gd`, planner to confirm) builds a `CombatSession` from a world encounter's enemies plus `PartyBuilder.build` of the ACTIVE members (the encounter's authored `party` list is ignored in game mode), and applies the result:

| Outcome | Effects |
| --- | --- |
| won | `LevelUp.award_encounter`; `QuestRunner.on_encounter_won(gs, encounter qid)`; done flag for placed/roamer markers with `once`; full heal (M3); the party stays on its tile |
| fled (the session reports an escape through the encounter exit tiles) | no XP; the token moves back to the tile it came from; markers stay; the counter resets as after any fight |
| lost | per Q3 (recommended: back to the location's `spawn` anchor, or the last SAFE location if the location has no SAFE zone, full heal, no other loss) |

### 2.5 Entering and leaving

- Enter (from new game, an exit, S10 arrival or a load): set `location_id`, `position` = the arrival anchor (`to_anchor`, default `spawn`), `explore.loc_steps` = 0, then `QuestRunner.on_location_reached(gs, location qid)` (every entry counts, also a re-entry; the runner ignores quests not waiting for it). A location without `map` is entered the same way but shows a menu (its markers ignored).
- An exit to another location enters that location; `to` may be a qualified location in a `connects` world (existing exits rule), which sets `GameState.world_id`. An exit `world_map` hands over to S10 with the current location as the node.
- Quest hooks summary: arrival -> `on_location_reached`; win -> `on_encounter_won`; points, chests, lore and markers -> flags through `QuestRunner.set_flag` (so `refresh` runs). Sub-location goals ("reach the altar") are `point` markers plus a flag objective, not new location entries.

### 2.6 Save (version 3, one migration step 2 -> 3)

New `GameState` keys (added to `GameState.KEYS`): `position` `[x, y]` (default `[-1, -1]` = use `spawn`), `explore` `{steps, next_at, loc_steps}` (ints, defaults 0 / 16 / 0), `rng_seed` int, `rolls` `{stream: int}`. Migration fills the defaults and a seed derived from the slot (so old saves stay deterministic). Saving rule: Q6.

### 2.7 Acceptance-check themes (the EXP1 spec turns these into rule-derived checks)

Template validation (row lengths, unknown chars, anchor on a wall, broken path, missing `spawn`); binding validation (unknown zone, weights <= 0, `steps_min > steps_max`, marker on an unknown anchor, exit `to` not in `exits`); step order (blocked move costs nothing, one fight per step, placed before counter); counter (SAFE tiles neither advance nor reset; same seed = same fights after reload); roamer position equals the derived ping-pong index; `once` markers vanish after a win and stay after a flight or loss; arrival calls the quest hook exactly once per entry; chest loot already claimed gives items only; a locked exit opens when its flag is set.

## 3. Open design questions (each blocks the EXP1 spec)

Q1. Where does map geometry live?

- A. Base lore-free templates (`data/maps/`) plus world bindings on the location (this file's sketch). Reusable layouts, matches the M4 template/binding split and "base asset decides gameplay"; two places to author one map.
- B. The whole grid inside the world location (`map` json). One place to author; geometry mixed with lore, no reuse, an override replaces the whole map.
- C. A, plus per-world tile patches on top of a template. Most flexible; a merge layer to build and test.
- Recommendation: A. C can be added later without breaking A.

Q2. Step counter shape?

- A. Threshold drawn in `[steps_min, steps_max]` after each fight (sketch). Predictable spacing, no streaks of back-to-back fights, easy to test.
- B. Per-step chance (for example 6%). Classic feel; streaks and droughts; XP pacing harder to tune.
- C. Per-step chance rising with steps since the last fight. Smooth; one more formula to tune.
- Recommendation: A (defaults 12..20 steps, elite 10%; tuning numbers, not rules).

Q3. What happens when the party loses an exploration fight?

- A. Wake at the location's `spawn` (or the last SAFE location), full heal, no loss. Matches M3 full heal and "Solo difficulty" (losses expected when under-levelled); no punishment loop.
- B. Reload the last save. Classic; loses progress and needs S13 save rules first.
- C. A plus a cost (gold or XP loss). Stakes; needs a currency (S11) and adds a balance lever.
- Recommendation: A now; C can be added once S11 gold exists.

Q4. Roaming elites and bosses: how do they move?

- A. Fixed ping-pong patrol on an authored path, contact or radius trigger (sketch). Deterministic, no saved state, readable.
- B. A plus chase within a detection radius. More tension; needs pathfinding and saved roamer positions.
- C. Stationary only (placed). Simplest; loses the "visible roaming elites" input.
- Recommendation: A; B becomes the "detection cones" later rule.

Q5. Do exploration mobs scale with the party level?

- A. Fixed authored level per world encounter. Regions have a level feel; out-levelling an area is a reward; no HISTORY conflict.
- B. Scale to the average active level (clamped). Always relevant fights; removes progression feel and needs a scaling rule on top of tiers.
- C. Fixed level per zone binding (overrides the encounter's units). Reuse one encounter at several levels; an extra override path in `WorldFactory`.
- Recommendation: A.

Q6. When may the player save?

- A. Anywhere outside battle and dialogue, plus autosave on location entry. Simple, matches the single-slot write-safe save (M3).
- B. Only on SAFE tiles (and autosave on entry). Gives danger zones weight; frustrating in long dungeons.
- C. Save markers only. Classic save points; more authoring.
- Recommendation: A (the menu itself belongs to the S13 menus task).

## 4. Milestone shape (coarse)

| Milestone | Lane | Goal | Depends on |
| --- | --- | --- | --- |
| EXP1-L | logic, one branch; commit 1 = schema (MapCatalog, `data/maps/maps.json` empty envelope, location fields, validation, ref scans), commit 2 = runtime (ExploreRunner, BattleBridge, deterministic rolls, save v3 + migration, suite), commit 3 = minimal region-map scene (grid, token, markers; renders only) | engine + tests + a playable debug scene | M4-L merged; answers Q1-Q6 |
| EXP1-A | asset, one branch (seed) | 2 lore-free templates (a town, a wild area with one DANGER zone) and Ashlands bindings for the M4 sample-quest locations: zone tables, one boss marker, one roamer, a chest, a gate opened by an M4 quest flag | EXP1-L commit 1 |

Hub files shared with S10 and S11 (`world_schema.gd`, `world_catalog.gd`, `game_state.gd`, `save_game.gd`, `save_migrations.gd`): EXP1, DLG1 and TRAV1 must run one after another (each bumps the save version by one).

## 5. HISTORY check

- Row 1 (10-block hand-scripted combat layer, deprecated): the S9 inputs came from the same old notes; only the non-combat parts are kept and fights go through the current planner and resolver.
- "Enemy tier defined per world" (not adopted) and "Fixed-power goblins" (deprecated): exploration fights use world encounters of class-derived units; Q5 keeps levels authored, never a per-world tier table.
- `.context/` notes (deprecated, CLN-4): their S9 parts already live in ROADMAP inputs; nothing else re-used.
- Nothing in HISTORY deprecates step counters, SAFE/DANGER zones or roaming encounters.
