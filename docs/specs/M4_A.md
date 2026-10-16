# M4-A: Sample campaign quests (S8)

Status: proposed (spec written 2026-10-17; awaiting approval and the M4-L answers)
Depends on: M4-L commit 1 merged (QuestCatalog, world quest fields and validation)
Lane: asset (one branch, seed `data/seeds/m4_quests.gd`)
Touches: `data/quests/quests.json` (via `QuestCatalog.upsert` + `QuestCatalog.save`), `data/worlds/ashlands/quests.json` and `data/worlds/iron_reaches/quests.json` (via `WorldCatalog.upsert(world_id, "quests", entry)` + `WorldCatalog.save_world`). No code, no tests, no edit to `data/catalog/*`. Tidewrack needs no data (it inherits the Ashlands quests).

## Goal

Ship four lore-free quest templates and a small linked Ashlands campaign (the two existing quests bound to templates plus two new ones) that exercises every objective kind, every reward type and the flag chain: courtyard -> (gate watch, oath); the oath's first objective waits for the gate watch's flag.

It also shows quest metadata (`links`, `turn_in`) and flag-driven variants: Mara's death changes one quest's giver and reward and moves another quest's target; Tev's death makes the oath unavailable. The base answers are settled (DECISIONS world-campaign "M4 quest answers (user 2026-10-17)"); the variant tables follow the recommendations V1-V3 in `docs/specs/M4_L.md`; if the user picks differently, the planner re-checks them before dispatch.

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
| turn_in | "" (= giver) | "" (= giver) | "" (= giver) |
| links | leads_to [cinder_gate_watch, warden_oath]; related_characters [mara_voss, tev_quill]; related_locations [warden_keep, ruined_courtyard]; related_factions [ember_wardens, ashen_horde] | follows [hold_the_courtyard]; leads_to [warden_oath]; related_characters [mara_voss]; related_factions [ashen_horde] | follows [hold_the_courtyard, cinder_gate_watch]; related_characters [tev_quill, mara_voss]; related_locations [warden_keep]; related_factions [ember_wardens] |
| variants | [] | see "Variants" below | see "Variants" below |

### Variants (flag-driven; nothing in M4 sets these flags yet, the smoke script and later S9/S11 do)

| Quest | Variant id | when_flags | unless_flags | set |
| --- | --- | --- | --- | --- |
| cinder_gate_watch | mara_fallen | [{key ashlands.mara_fallen, value true}] | [] | giver tev_quill; turn_in tev_quill; description "Mara is gone. Tev means to finish what she started and clear the pass himself."; reward "Tev's thanks and the keep's spare potions."; rewards_from reach_and_clear (no Warden drill without Mara: xp 60 + 2 healing potions instead of xp 80 + steady_nerves) |
| warden_oath | mara_fallen | [{key ashlands.mara_fallen, value true}] | [] | description "With Mara gone, Tev asks the party to swear the oath over her empty seat."; steps {"1": {text "Swear the oath in the Ruined Courtyard where Mara fell", location ruined_courtyard}} (the objective stays reach_location, only its target moves) |
| warden_oath | tev_fallen | [{key ashlands.tev_fallen, value true}] | [] | available false (no one left to speak for the party) |

Order inside `warden_oath.variants`: mara_fallen, then tev_fallen. With both flags set the quest is unavailable (or blocked if active), and the resolved description still shows the mara_fallen text (ordered merge).

### Iron Reaches (`data/worlds/iron_reaches/quests.json`)

Existing `iron_for_embers`: keep every current field and value (steps: `slag_gate`, then the cross-world `ashlands:warden_keep`); add `template` two_stop_delivery, `turn_in` "" (= giver), `requires_flags` [], `complete_flags` [{key iron_reaches.shipment_delivered, value true}], `auto_start` false, `links` {related_characters [orsa_tallow], related_locations [slag_gate, ashlands:warden_keep], related_factions [forge_guild, ashlands:ember_wardens]}, `variants` [].

JSON encoding: conditions are `{"key": "...", "value": true}`; `steps` overrides use string indexes (`{"1": {...}}`); refs in variants and links use the same local/qualified form as the quest's own fields.

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
7. Variants: with `ashlands.mara_fallen` true, `QuestRunner.resolved` of `cinder_gate_watch` has giver and turn_in `tev_quill` and its completion grants xp 60 + 2 `con_heal_potion` (no passive); an active `warden_oath` on objective 1 ignores `warden_keep` and advances on `ruined_courtyard`; setting `ashlands.tev_fallen` makes an active `warden_oath` blocked and a not-started one unavailable; `QuestRules.link_warnings` is empty for both worlds.

Checks 4-7 are the planner's smoke script after merge (the asset lane writes no tests); a failure there is reported, not fixed in the seed.

## Out of scope

Any quest in Tidewrack's own data; new locations, encounters, characters or items; dialogue text for givers; reward tuning (balance pass); the two `reward` texts of the existing quests (unchanged).

## Open questions

None beyond `docs/specs/M4_L.md` V1-V5 (V1 to V3 change the variant tables if answered differently).
