# M4-A: Sample campaign quests (S8)

Status update (2026-10-20): approved and built; the rows are done (docs/archive/TASKS_DONE.md). The status line below is the original.

Status: proposed (design answered: DECISIONS world-campaign "M4 quest answers" and "M4 variant answers + world loot rewards" (user 2026-10-17); M4-L open question L1 only changes check 8; approval by the user)
Depends on: M4-L commit 1 merged (QuestCatalog, world quest fields, world kind `loot`, validation)
Lane: asset (one branch, seed `data/seeds/m4_quests.gd`)
Touches: `data/quests/quests.json` (via `QuestCatalog.upsert` + `QuestCatalog.save`), `data/worlds/ashlands/loot.json` (new), `data/worlds/ashlands/quests.json` and `data/worlds/iron_reaches/quests.json` (via `WorldCatalog.upsert(world_id, kind, entry)` + `WorldCatalog.save_world`). No code, no tests, no edit to `data/catalog/*`. Tidewrack needs no data (it inherits the Ashlands quests and loot).

## Goal

Ship four lore-free quest templates, a four-entry Ashlands loot table, and a small linked Ashlands campaign (the two existing quests bound to templates plus two new ones) that exercises every objective kind, every reward type, the flag chain (courtyard -> gate watch, oath; the oath's first objective waits for the gate watch's flag), quest metadata (`links`, `turn_in`), flag-driven variants (Mara's death changes a giver, a reward and a target; Tev's death makes the oath unavailable) and shared loot pools (a claimed entry disappears from the other quests).

## Templates (`data/quests/quests.json`)

Generic rewards only (the special passive and skill moved to the loot table). All `description` texts are lore-free; `tags` = [].

| id | name | description | objectives (in order) | rewards |
| --- | --- | --- | --- | --- |
| reach_and_clear | Reach and clear | Go to a place, then win the fight there. | reach_location, win_encounter | xp 60; items con_heal_potion x2 |
| two_stop_delivery | Two-stop delivery | Pick something up at one place and bring it to another. | reach_location, reach_location | xp 40; items con_focus_elixir x1 |
| clear_site | Clear a site | Win one fight. | win_encounter | xp 80; items con_heal_potion x1 |
| signal_and_return | Signal and return | Wait for a signal, then return to a place. | flag, reach_location | xp 50; items con_focus_elixir x1 |

JSON shape example (first row): `{"id": "reach_and_clear", "name": "Reach and clear", "description": "Go to a place, then win the fight there.", "objectives": [{"kind": "reach_location"}, {"kind": "win_encounter"}], "rewards": {"xp": 60, "items": {"con_heal_potion": 2}}, "tags": []}`. Rows without items/passives/skills omit those keys.

Number rationale (starting values, not balance): quest XP sits below one encounter's kill XP at the Ashlands levels (a five-goblin fight at level 3 gives up to about 125 per member by the M3 kill formula, less for minion tiers). Generic rewards hold no passive or skill, so generic + one special stays inside every soft cap (1 passive, 1 skill, 6 items).

## Ashlands loot table (`data/worlds/ashlands/loot.json`)

| id | name | description | reward_kind | ref | count | rarity | requires_flags |
| --- | --- | --- | --- | --- | --- | --- | --- |
| wardens_drill | Wardens' Drill | Mara's drill against fear, taught to those who held the line. | passive | steady_nerves | 1 | rare | [] |
| rallying_cry | Rallying Cry | The Wardens' call that steadies a wavering line. | skill | rally | 1 | rare | [] |
| keep_ring | Keep Ring | A heavy ring from the keep's strongbox. | item | acc_bear_ring | 1 | rare | [{key ashlands.courtyard_held, value true}] |
| keep_stores | Keep Stores | A satchel of the keep's healing draughts. | item | con_heal_potion | 3 | common | [] |

`tags` = [] on all. `steady_nerves` (resist 25) is a defensive source passive; `rally` (priest path SUPPORT family, no weapon gate) is a cross-class skill, so a non-priest recipient gains a usable family. `keep_ring` is gated on the courtyard flag, which is set AFTER the courtyard's grant, so the courtyard never offers it while later quests do.

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
| reward (text) | (unchanged) | The Wardens' thanks, and a pick from the keep. | A Warden's gift, and the oath. |
| tags | [] | [] | [] |
| turn_in | "" (= giver) | "" (= giver) | "" (= giver) |
| loot_pool | [keep_stores, wardens_drill, keep_ring] | [wardens_drill, keep_ring, rallying_cry] | [rallying_cry, wardens_drill] |
| loot_picks | 1 | 1 | 1 |
| links | leads_to [cinder_gate_watch, warden_oath]; related_characters [mara_voss, tev_quill]; related_locations [warden_keep, ruined_courtyard]; related_factions [ember_wardens, ashen_horde] | follows [hold_the_courtyard]; leads_to [warden_oath]; related_characters [mara_voss]; related_factions [ashen_horde] | follows [hold_the_courtyard, cinder_gate_watch]; related_characters [tev_quill, mara_voss]; related_locations [warden_keep]; related_factions [ember_wardens] |
| variants | [] | see "Variants" below | see "Variants" below |

### Variants (flag-driven; nothing in M4 sets these flags yet, the smoke script and later S9/S11 do)

| Quest | Variant id | when_flags | unless_flags | set |
| --- | --- | --- | --- | --- |
| cinder_gate_watch | mara_fallen | [{key ashlands.mara_fallen, value true}] | [] | giver tev_quill; turn_in tev_quill; description "Mara is gone. Tev means to finish what she started and clear the pass himself."; reward "Tev's thanks and what the keep can spare."; rewards_from reach_and_clear (xp 60 + 2 potions instead of xp 80 + 1 potion); loot_pool [keep_stores, rallying_cry] (no Wardens' Drill without Mara) |
| warden_oath | mara_fallen | [{key ashlands.mara_fallen, value true}] | [] | description "With Mara gone, Tev asks the party to swear the oath over her empty seat."; steps {"1": {text "Swear the oath in the Ruined Courtyard where Mara fell", location ruined_courtyard}} (the objective stays reach_location, only its target moves) |
| warden_oath | tev_fallen | [{key ashlands.tev_fallen, value true}] | [] | available false (no one left to speak for the party) |

Order inside `warden_oath.variants`: mara_fallen, then tev_fallen. With both flags set the quest is unavailable (or blocked if active), and the resolved description still shows the mara_fallen text (ordered merge).

Budget check (M4-L `reward_warnings`, worst case per quest): passives 0 + 1, skills 0 + 1, items at most 2 (generic) + 3 (keep_stores) = 5 <= 6, picks 1: no warning.

### Iron Reaches (`data/worlds/iron_reaches/quests.json`)

Existing `iron_for_embers`: keep every current field and value (steps: `slag_gate`, then the cross-world `ashlands:warden_keep`); add `template` two_stop_delivery, `turn_in` "" (= giver), `requires_flags` [], `complete_flags` [{key iron_reaches.shipment_delivered, value true}], `auto_start` false, `loot_pool` [], `loot_picks` 1, `links` {related_characters [orsa_tallow], related_locations [slag_gate, ashlands:warden_keep], related_factions [forge_guild, ashlands:ember_wardens]}, `variants` [].

JSON encoding: conditions are `{"key": "...", "value": true}`; `steps` overrides use string indexes (`{"1": {...}}`); refs in variants, pools and links use the same local/qualified form as the quest's own fields.

## Authoring rules

- Seed order: the four templates (`QuestCatalog.upsert` each, then `QuestCatalog.save()`), then the Ashlands loot entries (`WorldCatalog.upsert("ashlands", "loot", entry)`), then the world quests (`WorldCatalog.upsert` per entry), then `save_world("ashlands")` and `save_world("iron_reaches")`. For the two existing quests, read the current entry with `WorldCatalog.get_own`, merge the new fields, and upsert it under its own id (no rename).
- Every upsert must return no problems; stop and report on the first problem (do not patch JSON by hand).
- Register the seed in `data/seeds/README.md` (task M4-A); it leaves `data/seeds/` in the cleanup pass after merge (CLN-2 rule).

## Acceptance checks (asset lane; run headless after the seed)

1. `QuestCatalog.validate_all()` == {} and `QuestCatalog.warnings_all()` is empty; `QuestRules.reward_warnings` and `QuestRules.link_warnings` are empty for ashlands and iron_reaches.
2. `WorldCatalog.validate_all()` == {} (no new errors in any world, Tidewrack included).
3. `QuestCatalog.ids()` == [reach_and_clear, two_stop_delivery, clear_site, signal_and_return] (this order).
4. From a fresh `GameState` in `ashlands` after `QuestRunner.refresh`: `hold_the_courtyard` is active (auto start), the other two are locked.
5. From `tidewrack`, `QuestRunner.quests_with(gs, "active")` after `refresh` contains `ashlands:hold_the_courtyard`, and its offers resolve the inherited loot entries.
6. Scripted run in ashlands: reach `warden_keep`, win `courtyard_skirmish`; offers == [keep_stores, wardens_drill] (keep_ring flag not yet set); `complete` with pick `wardens_drill` -> generic xp 60 + 2 potions plus `steady_nerves` on the recipient; `cinder_gate_watch` and `warden_oath` become available. Start the oath: it stays on objective 0; start and win the gate watch: offers == [keep_ring, rallying_cry] (drill claimed, ring now unlocked); `complete` with pick `keep_ring` -> the oath advances to objective 1 in the same call; reach `warden_keep`: oath offers == [rallying_cry]; `complete` with a non-priest recipient and pick `rallying_cry` -> that member owns `rally`; `loot_claimed` == [ashlands:keep_ring, ashlands:rallying_cry, ashlands:wardens_drill].
7. Variants: with `ashlands.mara_fallen` true, `QuestRunner.resolved` of `cinder_gate_watch` has giver and turn_in `tev_quill`, offers exclude `wardens_drill`, and completion grants xp 60 + 2 `con_heal_potion` plus the pick; an active `warden_oath` on objective 1 ignores `warden_keep` and advances on `ruined_courtyard`; setting `ashlands.tev_fallen` makes an active `warden_oath` blocked and a not-started one unavailable.
8. Generic floor: completing a quest with 0 picks (L1 recommendation) still grants its template rewards and claims nothing.

Checks 4-8 are the planner's smoke script after merge (the asset lane writes no tests); a failure there is reported, not fixed in the seed.

## Out of scope

Any quest or loot in Tidewrack's or Iron Reaches' own data; new locations, encounters, characters or base items; dialogue text for givers; reward tuning (balance pass); the `reward` texts of `hold_the_courtyard` and `iron_for_embers` (unchanged).

## Open questions

None beyond M4-L L1 (if the user requires exact picks, check 8 is dropped and the smoke run always picks).
