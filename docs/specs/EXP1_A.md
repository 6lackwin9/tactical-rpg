# EXP1-A: Map templates and Ashlands exploration data (S9)

Status: spec (rules approved: DECISIONS world-campaign "S9-S11 baseline answers (user 2026-10-17)"; this build spec awaits the user's approval)
Depends on: EXP1-L commit 1 merged (MapCatalog, location fields, `start_location`, validation); M4-A merged
Lane: asset (one branch, seed `data/seeds/exp1a.gd`)
Touches: `data/maps/maps.json` (via `MapCatalog.upsert` + `MapCatalog.save`), `data/worlds/ashlands/{world,locations,characters,encounters}.json` (via `WorldCatalog.update_manifest` / `upsert` + `save_world`), `data/seeds/exp1a.gd`, `data/seeds/README.md`. No code, no tests, no `data/catalog/*` edit.

## Goal

Two lore-free templates (a walled town, a gorge with a camp and ruins), the Ashlands start location, bindings for Warden Keep, the Ruined Courtyard and a new Cinder Gate location (re-using the gorge template), three regular mob encounters with generic Horde characters, the M4 quest fights as placed markers, one patrolling elite, chests (one with world loot), lore, a point marker and a gate opened by the M4 flag `ashlands.courtyard_held`.

## 1. Map templates (`data/maps/maps.json`)

### 1.1 `walled_town` (16 x 10)

name "Walled town, south gate"; tags []; all tiles one SAFE zone.

```text
################
#..............#
#.###......###.#
#.###......###.#
#..............#
#..............#
#.###......###.#
#.###......###.#
#..............#
#######..#######
```

- zones: 10 rows of `ssssssssssssssss`; zone_legend `{"s": {"id": "town", "kind": "safe"}}`.
- anchors (`id x y`): spawn 7 8; gate 7 9; road 1 8; npc_a 4 4; npc_b 11 4; npc_c 7 1; npc_d 14 1; chest_a 1 1; lore_a 8 5.
- paths: {}.

### 1.2 `gorge_ruins` (20 x 12)

name "Gorge: camp west, ruins east, two openings"; tags []; camp columns 0-4 SAFE, the rest one DANGER zone.

```text
####################
#....#........~~~..#
#....#..##....~~~..#
#.........#........#
#....#....#...##...#
.....#..............
#....#...##........#
#.........#...~~...#
#....#....#...~~...#
#....#..........##.#
#....#.............#
####################
```

- zones: 12 rows of `sssssddddddddddddddd`; zone_legend `{"s": {"id": "camp", "kind": "safe"}, "d": {"id": "ruins", "kind": "danger"}}`.
- anchors: spawn 2 5; west 0 5; west_in 1 5; east 19 5; east_in 18 5; camp_npc 2 2; camp_chest 1 10; site_a 12 5; site_b 16 5; altar 17 1; ruin_chest 18 10; ruin_lore 7 9.
- paths: `patrol` = [[8,10],[9,10],[10,10],[11,10],[12,10],[13,10],[14,10]].

Unused anchors (`road`, `npc_d`, `camp_npc`, `east` in the Cinder Gate) are reserved for TRAV1-A and DLG1-A.

## 2. Ashlands manifest

`WorldCatalog.update_manifest("ashlands", {"start_location": "warden_keep"})`.

## 3. New characters (`characters`, all `role` enemy, faction `ashen_horde`, home "", art "", disciplines/passives/named_items [], tuning {}, tags [])

| id | name | title | base_unit | background |
| --- | --- | --- | --- | --- |
| ash_brawler | Horde Brawler | Warband Muscle | goblin_brawler | One of many who fight for whoever feeds them. |
| ash_skirmisher | Horde Skirmisher | Warband Runner | goblin_skirmisher | Fast, light and never alone. |
| ash_archer | Horde Archer | Warband Bow | goblin_archer | Shoots from the ridges and runs. |
| ash_shaman | Horde Smokecaller | Warband Shaman | goblin_shaman | Mutters at the ash and patches the wounded. |
| ash_berserker | Horde Berserker | Warband Berserker | goblin_berserker | Screams first, thinks never. |
| ash_warchief | Horde Warchief | Patrol Leader | ambush_berserker | Leads the patrols that circle the ruins. |

## 4. Encounters (`encounters`)

New entries share: arena CLOSED, exits [[8, 4]], max_rounds 6, total_blocks 6, ap_regen 4, unit_ap 12, rng_seed 1337, party [mara_voss, tev_quill] (required by validation, replaced by the active party in game mode), wave_units [], wave_round 1, wave_block 4, wave_label "Reinforcements arrive!", tags [].

| id | name | description | intro | location | zone | enemies |
| --- | --- | --- | --- | --- | --- | --- |
| ash_scouts | Horde Scouts | Two scouts testing the Wardens' patrols. | Movement in the rubble: scouts. | ruined_courtyard | ruined_courtyard | ash_skirmisher, ash_archer |
| ash_raiders | Horde Raiders | A small raiding party looking for easy prey. | Raiders spill out of the ruins. | ruined_courtyard | ruined_courtyard | ash_brawler, ash_skirmisher, ash_shaman |
| ash_patrol | Horde Patrol | A warchief and his berserker on their rounds. | The patrol has seen you. | ruined_courtyard | ruined_courtyard | ash_warchief, ash_berserker |

Existing `pass_ambush`: change only `location` from `ruined_courtyard` to `cinder_gate` (read with `get_own`, merge, upsert under its id).

## 5. Locations (`locations`)

### 5.1 New `cinder_gate`

name "The Cinder Gate"; description "A collapsed toll arch at the top of the gorge, where the pass opens onto the wastes."; kind gate; region cinder_pass; exits [{to ruined_courtyard, label "back down the gorge"}]; encounters [pass_ambush, ash_scouts, ash_raiders, ash_patrol]; tags []; map gorge_ruins.

### 5.2 Field values per location

`ruined_courtyard`: keep all current fields; `exits` becomes [{to warden_keep, label "back to the keep"}, {to cinder_gate, label "the gate road"}]; `encounters` becomes [courtyard_skirmish, ash_scouts, ash_raiders, ash_patrol]. `warden_keep`: keep all current fields.

| Location | map | zone_bindings |
| --- | --- | --- |
| warden_keep | walled_town | {} |
| ruined_courtyard | gorge_ruins | `{"ruins": {"encounters": [["ash_scouts", 3], ["ash_raiders", 2]], "elites": [["ash_patrol", 1]], "elite_pct": 10, "steps_min": 12, "steps_max": 20}}` |
| cinder_gate | gorge_ruins | `{"ruins": {"encounters": [["ash_scouts", 2], ["ash_raiders", 3]], "elites": [["ash_patrol", 1]], "elite_pct": 15, "steps_min": 10, "steps_max": 16}}` |

### 5.3 Markers (keys not listed take their defaults; conditions are `{"key": K, "value": true}`)

| Location | id | kind | anchor | keys |
| --- | --- | --- | --- | --- |
| warden_keep | gate | exit | gate | to ruined_courtyard, to_anchor west_in |
| warden_keep | mara | npc | npc_a | character mara_voss |
| warden_keep | tev | npc | npc_b | character tev_quill |
| warden_keep | hest | npc | npc_c | character hest_the_quartermaster |
| warden_keep | stores | chest | chest_a | items {con_heal_potion: 1} |
| warden_keep | hearth | lore | lore_a | lore one_fire_lit; sets_flags [ashlands.hearth_read] |
| ruined_courtyard | to_keep | exit | west | to warden_keep, to_anchor spawn |
| ruined_courtyard | gate_road | exit | east | to cinder_gate, to_anchor west_in; requires_flags [ashlands.courtyard_held]; locked_text "Warden barricades close the gate road until the Courtyard is held." |
| ruined_courtyard | warband | encounter | site_a | encounter courtyard_skirmish, trigger proximity, radius 1, once false, unless_flags [ashlands.courtyard_held] |
| ruined_courtyard | patrol | roamer | (none) | encounter ash_patrol, path patrol, radius 1, once true |
| ruined_courtyard | ruin_cache | chest | ruin_chest | items {con_focus_elixir: 1}, loot keep_stores |
| ruined_courtyard | old_stones | lore | ruin_lore | lore the_long_dry |
| cinder_gate | to_gorge | exit | west | to ruined_courtyard, to_anchor east_in |
| cinder_gate | ambush | encounter | site_b | encounter pass_ambush, trigger proximity, radius 2, once false, unless_flags [ashlands.pass_cleared] |
| cinder_gate | beacon | point | altar | sets_flags [ashlands.gate_beacon_lit] |
| cinder_gate | camp_cache | chest | camp_chest | items {con_heal_potion: 1} |

Why `once: false` with `unless_flags` on the two quest fights: a win while the quest is not yet active must not remove the fight for good (M4 gives no retroactive credit); the marker disappears when the quest's `complete_flags` are set. `keep_stores` in the ruin chest exercises the shared claim rule: opening the chest first removes it from the quest pools; completing `hold_the_courtyard` with that pick first makes the chest give the elixir only.

## 6. Authoring rules

- Seed order: the two templates (`MapCatalog.upsert` each, then `MapCatalog.save()`), the manifest, the six characters, the three encounters, `pass_ambush`, `cinder_gate`, then `ruined_courtyard` and `warden_keep` (read with `get_own`, merge, upsert under the same id), then `save_world("ashlands")`.
- `upsert` returns only blocking (shape) errors; the cross-references (exits, anchors on other maps) are checked by `validate_all` at the end. Stop and report on the first problem; never patch JSON by hand.
- Register the seed in `data/seeds/README.md` (task EXP1-A).

## 7. Acceptance checks (asset lane, headless after the seed)

1. `MapCatalog.validate_all()` == {} and `warnings_all()` empty; `MapCatalog.ids()` == [walled_town, gorge_ruins].
2. `WorldCatalog.validate_all()` == {} (Tidewrack and Iron Reaches included); `MapRules.warnings("ashlands")` empty.
3. Planner smoke script after merge (the asset lane writes no tests): `NewGame.create("ashlands", ["warrior", "ranger"], 7)` + `ExploreRunner.begin` -> `location_id` `ashlands:warden_keep`, position [7, 8], `hold_the_courtyard` active on objective 1 (the start location met objective 0).
4. Step down from [7, 8] -> the Ruined Courtyard at [1, 5]; walking through the camp opening at [5, 3] to a tile within 1 of [12, 5] fights `courtyard_skirmish` (a counter fight on the way is resolved first); `after_fight` won -> the quest is `ready`; the gate road at [19, 5] is still `locked` until `QuestRunner.complete` sets `ashlands.courtyard_held`, then it leads to the Cinder Gate at [1, 5].
5. In the courtyard, DANGER steps fight within 12..20 steps; with the same seed and inputs the same encounter ids come up twice.
6. The roamer stands at [8, 10] on entry, [9, 10] after one step and returns along the path; winning against it once sets `ashlands.ruined_courtyard.patrol.done` and removes it.
7. Opening `ruin_cache` before completing the courtyard grants the elixir and three potions and adds `ashlands:keep_stores` to `loot_claimed`; the courtyard quest then no longer offers `keep_stores`.
8. Stepping on the Cinder Gate altar sets `ashlands.gate_beacon_lit`; reading the hearth sets `ashlands.hearth_read`.

## Out of scope

Dialogue trees, shops and gold (DLG1-A), routes and `world_map` exits (TRAV1-A: anchors `road` and the Cinder Gate `east` are reserved), maps for Tidewrack or Iron Reaches locations (they stay menu locations), new base units or items, balance of encounter sizes.

## Open questions

None.
