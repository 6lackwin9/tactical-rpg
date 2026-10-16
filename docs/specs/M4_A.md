# M4-A: Sample campaign quests (S8)

Status: proposed (spec written 2026-10-17; awaiting approval and the M4-L answers)
Depends on: M4-L commit 1 merged (QuestCatalog, world quest fields and validation)
Lane: asset (one branch, seed `data/seeds/m4_quests.gd`)
Touches: `data/quests/quests.json` (via `QuestCatalog.upsert` + `QuestCatalog.save`), `data/worlds/ashlands/quests.json` and `data/worlds/iron_reaches/quests.json` (via `WorldCatalog.upsert(world_id, "quests", entry)` + `WorldCatalog.save_world`). No code, no tests, no edit to `data/catalog/*`. Tidewrack needs no data (it inherits the Ashlands quests).

## Goal

Ship four lore-free quest templates and a small linked Ashlands campaign (the two existing quests bound to templates plus two new ones) that exercises every objective kind, every reward type and the flag chain: courtyard -> (gate watch, oath); the oath's first objective waits for the gate watch's flag.

This spec follows the recommended answers in `docs/specs/M4_L.md` "Open questions for the user"; if the user picks differently, the planner re-checks the tables below before dispatch.

## Templates (`data/quests/quests.json`)

All `description` texts are lore-free; `tags` = [].

| id | name | description | objectives (in order) | rewards |
| --- | --- | --- | --- | --- |
| reach_and_clear | Reach and clear | Go to a place, then win the fight there. | reach_location, win_encounter | xp 60; items con_heal_potion x2 |
| two_stop_delivery | Two-stop delivery | Pick something up at one place and bring it to another. | reach_location, reach_location | xp 40; items con_focus_elixir x1 |
| clear_site | Clear a site | Win one fight. | win_encounter | xp 80; passives steady_nerves |
| signal_and_return | Signal and return | Wait for a signal, then return to a place. | flag, reach_location | xp 50; skills rally |

JSON shape example (first row): `{"id": "reach_and_clear", "name": "Reach and clear", "description": "Go to a place, then win the fight there.", "objectives": [{"kind": "reach_location"}, {"kind": "win_encounter"}], "rewards": {"xp": 60, "items": {"con_heal_potion": 2}}, "tags": []}`. Rows without items/passives/skills omit those keys (defaults apply).

Number rationale (starting values, not balance): every reward is under the M4-L soft caps (xp 300, 1 passive, 1 skill, 6 items), so `warnings_all()` is empty. XP sits below one encounter's kill XP at the Ashlands levels (a five-goblin fight at level 3 gives up to about 125 per member by the M3 kill formula, less for minion tiers), so quests add progress without replacing fights. `steady_nerves` (resist 25) is a defensive source passive; `rally` (priest path SUPPORT family, no weapon gate) is the cross-class skill reward, so any non-priest recipient gains a usable family.

## World quests

### Ashlands (`data/worlds/ashlands/quests.json`)

Existing `hold_the_courtyard`: keep every current field and value; add the new fields below (steps unchanged: step 1 location `warden_keep`, step 2 location `ruined_courtyard` + encounter `courtyard_skirmish`).

| Field | hold_the_courtyard | cinder_gate_watch (new) | warden_oath (new) |
| --- | --- | --- | --- |
| name | (unchanged) | Watch at the Cinder Gate | The Warden's Oath |
| description | (unchanged) | Mara suspects the warband left scouts in the pass. Find them before they find the keep. | Once the pass is safe, Tev will speak for the party before the Wardens. |
| giver | (unchanged: mara_voss) | mara_voss | tev_quill |
| faction | (unchanged: ember_wardens) | ember_wardens | ember_wardens |
| region | (unchanged: cinder_pass) | cinder_pass | ashen_marches |
| requires | [] | [hold_the_courtyard] | [hold_the_courtyard] |
| template | reach_and_clear | clear_site | signal_and_return |
| steps | (unchanged) | 1: text "Break the ambush at the Cinder Gate", encounter pass_ambush | 1: text "Wait for word that the Cinder Gate is clear", flag ashlands.pass_cleared, value true; 2: text "Return to Warden Keep and swear the oath", location warden_keep |
| requires_flags | [] | [] | [] |
| complete_flags | [{key ashlands.courtyard_held, value true}] | [{key ashlands.pass_cleared, value true}] | [{key ashlands.oath_sworn, value true}] |
| auto_start | true | false | false |
| reward (text) | (unchanged) | The Wardens' drill against fear. | A Warden's rallying cry. |
| tags | [] | [] | [] |

### Iron Reaches (`data/worlds/iron_reaches/quests.json`)

Existing `iron_for_embers`: keep every current field and value (steps: `slag_gate`, then the cross-world `ashlands:warden_keep`); add `template` two_stop_delivery, `requires_flags` [], `complete_flags` [{key iron_reaches.shipment_delivered, value true}], `auto_start` false.

## Authoring rules

- Seed order: the four templates first (`QuestCatalog.upsert` each, then `QuestCatalog.save()`), then the world quests (`WorldCatalog.upsert` per entry, then `save_world("ashlands")` and `save_world("iron_reaches")`). For the two existing quests, read the current entry with `WorldCatalog.get_own`, merge the new fields, and upsert it under its own id (no rename).
- Every upsert must return no problems; stop and report on the first problem (do not patch JSON by hand).
- Register the seed in `data/seeds/README.md` (task M4-A); it leaves `data/seeds/` in the cleanup pass after merge (CLN-2 rule).

## Acceptance checks (asset lane; run headless after the seed)

1. `QuestCatalog.validate_all()` == {} and `QuestCatalog.warnings_all()` is empty.
2. `WorldCatalog.validate_all()` == {} (no new errors in any world, Tidewrack included).
3. `QuestCatalog.ids()` == [reach_and_clear, two_stop_delivery, clear_site, signal_and_return] (this order).
4. From a fresh `GameState` in `ashlands` after `QuestRunner.refresh`: `hold_the_courtyard` is active (auto start), the other two are locked.
5. From `tidewrack`, `QuestRunner.quests_with(gs, "active")` after `refresh` contains `ashlands:hold_the_courtyard` (inherited; the encounter resolves through the Tidewrack overrides).
6. Scripted run in ashlands: reach `warden_keep`, win `courtyard_skirmish`, `complete` -> `cinder_gate_watch` and `warden_oath` become available; start the oath: it stays on objective 0; start and finish the gate watch -> the oath advances to objective 1 in the same `complete` call; reach `warden_keep`, `complete` with a non-priest recipient -> that member owns `rally`.

Checks 4-6 are the planner's smoke script after merge (the asset lane writes no tests); a failure there is reported, not fixed in the seed.

## Out of scope

Any quest in Tidewrack's own data; new locations, encounters, characters or items; dialogue text for givers; reward tuning (balance pass); the two `reward` texts of the existing quests (unchanged).

## Open questions

None beyond `docs/specs/M4_L.md` (questions 3 and 4 change these tables if answered differently).
