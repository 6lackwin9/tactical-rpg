# TRAV1-A: Ashlands routes, world-map exits and the road to Iron Reaches (S10)

Status: spec (rules approved: DECISIONS world-campaign "S9-S11 baseline answers (user 2026-10-17)"; this build spec awaits the user's approval)
Depends on: TRAV1-L commit 1 merged; EXP1-A merged (templates, anchors `road` and `east`, the Cinder Gate)
Lane: asset (one branch, seed `data/seeds/trav1a.gd`)
Touches: new `data/worlds/ashlands/routes.json`, new `data/worlds/iron_reaches/routes.json`, `data/worlds/ashlands/locations.json` (two exit markers), via `WorldCatalog.upsert` + `save_world`; `data/seeds/trav1a.gd`, `data/seeds/README.md`. No code, no tests.

## Goal

Three routes that exercise every TRAV1 rule on the sample campaign: a road with a one-shot scripted ambush, a road closed until the M4 flag `ashlands.courtyard_held`, and one two-way cross-world road to Iron Reaches that serves both directions; plus the two `world_map` exits that open the world map from the region maps.

## 1. Routes

All: `unless_flags` [] unless listed, tags []. Conditions are `{"key": K, "value": true}`.

| Field | `ashlands:old_road` | `ashlands:high_road` | `iron_reaches:south_road` |
| --- | --- | --- | --- |
| name | The old road | The high road | The south road |
| from | warden_keep | warden_keep | slag_gate |
| to | ruined_courtyard | cinder_gate | ashlands:warden_keep |
| two_way | true | true | true |
| time | 2 | 4 | 6 |
| zone | ruined_courtyard | desert_outpost | frozen_pass |
| encounter_pct | 15 | 25 | 20 |
| repeat_pct | -1 (= 7) | -1 (= 12) | -1 (= 10) |
| encounters | [["ash_scouts", 1]] | [["ash_scouts", 2], ["ash_raiders", 2]] | [["ashlands:ash_scouts", 1]] |
| elites | [] | [["ash_patrol", 1]] | [] |
| elite_pct | 5 | 5 | 0 |
| scripted | [{id first_raid, encounter ash_raiders, requires_flags [], unless_flags [ashlands.courtyard_held], sets_flags [ashlands.old_road_raided]}] | [] | [] |
| requires_flags | [] | [ashlands.courtyard_held] | [] |

`south_road` is owned by Iron Reaches (which `connects` Ashlands); by the TRAV1 graph rule it also appears in the Ashlands graph, so no Ashlands copy is authored (a copy would double the edge).

## 2. World-map exit markers (`locations`, merge into `markers`)

| Location | id | kind | anchor | keys |
| --- | --- | --- | --- | --- |
| warden_keep | signpost | exit | road | to world_map |
| cinder_gate | wastes_road | exit | east | to world_map |

`ruined_courtyard` stays without a world-map exit (it is a node: arrive by road, leave by its local exits). `slag_gate` has no map; its menu gains the world map automatically because it is a node.

## 3. Authoring rules

- Seed order: the two Ashlands routes, the Iron Reaches route, the two markers (read the location with `get_own`, append to `markers`, upsert under the same id), then `save_world("ashlands")` and `save_world("iron_reaches")`.
- Stop on the first problem; never patch JSON by hand. Register the seed in `data/seeds/README.md` (task TRAV1-A).

## 4. Acceptance checks (asset lane, headless after the seed)

1. `WorldCatalog.validate_all()` == {} (Tidewrack included); `TravelRules.warnings` empty for ashlands and iron_reaches.
2. `TravelRules.nodes("ashlands")` == [ashlands:warden_keep, ashlands:ruined_courtyard, ashlands:cinder_gate, iron_reaches:slag_gate]; `TravelRules.nodes("iron_reaches")` == [iron_reaches:slag_gate, ashlands:warden_keep].
3. Planner smoke script after merge: a new Ashlands game knows warden_keep, ruined_courtyard and iron_reaches:slag_gate, not cinder_gate (high road closed).
4. The first trip to the Ruined Courtyard (before `courtyard_held`) stops for `ash_raiders` (scripted) with `time` 2; after the win `ashlands.old_road.first_raid.done` and `ashlands.old_road_raided` are set and the party enters the courtyard at its spawn.
5. With `ashlands.courtyard_held` set, the Cinder Gate becomes known and `TravelRules.path` from the keep is [ashlands:high_road].
6. Travelling to `iron_reaches:slag_gate` adds 6 to `time` (unless a fight stops it), sets `world_id` to iron_reaches, and the Slag Gate menu offers its exit and the world map; travelling back to `ashlands:warden_keep` restores `world_id` ashlands.

## Out of scope

Supplies (TRAV2), Tidewrack routes, new locations, encounter tuning, world-map art.

## Open questions

None.
