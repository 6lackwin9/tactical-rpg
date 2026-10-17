# CAMP1: Ashlands campaign "The Long Dry" (story pass, goal G4)

Status: spec (design-brainstorm 2026-10-17; needs the user's answers below)
Depends on: TRAV1-A, REP1-A, BATTLE1-L (all on develop); BAL3-A for the one catalog unit (same file)
Lane: asset (data only; no logic task needed)
Touches: `data/worlds/ashlands/*`, `data/worlds/iron_reaches/{characters,dialogues,locations,routes}.json`, `data/catalog/units.json` (one row), seeds `data/seeds/camp1_*.gd`

## Summary

1. The Long Dry came early: Grakka Ashmaw has united the Horde in a camp past the Cinder Gate.
2. Start = the three sample quests (courtyard, gate, oath), unchanged except one text and two links.
3. Middle = new quest "Steel for the Gate": a trip to Slag Gate for Orsa's steel, raided on the way back.
4. Choice = what the steel becomes: bar the gate (rep up, cheaper shop, easier finale) or arm the party (gear in the shop).
5. Ending = new quest "Keep One Fire Lit": a parley that turns into a boss fight at the new Ashmaw Camp; Mara closes the story.

## 1. Beats in order

| # | Beat | Quest | Where | Fight | New? |
| --- | --- | --- | --- | --- | --- |
| 1 | The Horde hits the Courtyard | hold_the_courtyard (auto) | warden_keep, ruined_courtyard | courtyard_skirmish | existing; new description |
| 2 | Ambush at the pass; a chieftain's banner | cinder_gate_watch | cinder_gate | pass_ambush | existing; new lore marker |
| 3 | Sworn in as Wardens | warden_oath | warden_keep | - | existing |
| 4 | Fetch steel from the Iron Reaches | steel_for_the_gate (auto on oath) | iron_reaches:slag_gate, south road | ash_raiders (scripted road raid) | new |
| 5 | The steel choice (Mara) | - (dialogue) | warden_keep | - | new |
| 6 | Break the chieftain | keep_one_fire_lit | ashmaw_camp (new) | ashmaw_stand or ashmaw_stand_barred | new |
| 7 | Epilogue (Mara), sets `ashlands.campaign_done` | - | warden_keep | - | new |

Side quest `ember_patrol` (Hest, rep 30) stays optional. `iron_reaches:iron_for_embers` is left out (finding F1).

## 2. Findings that shape the data (no engine change needed)

- F1. Quests resolve only inside a world's `extends` lineage, not `connects`: while the party is in the Ashlands, `iron_for_embers` (an Iron Reaches quest) is invisible, so its "deliver to Warden Keep" step can never count. CAMP1 keeps the middle quest in the Ashlands and uses a flag set at Slag Gate; Orsa's tree has `services: false` so the broken quest is never offered. A later fix is a separate idea row (QUESTX).
- F2. Slag Gate is a menu location (`map: ""`) and menus show no NPCs, so Orsa cannot be talked to. CAMP1 gives it the `walled_town` map.
- F3. Battle link: BATTLE1-L is on develop, so map, road and dialogue fights are real battles. No dialogue on develop has a `fight` effect yet (G1 lists "the Cinder Gate dialogue fight"); Grakka's parley adds the first one.
- F4. `ExploreRunner.enter` refreshes quests before the arrival check, so "flag set in another world, then reach the keep" completes on arrival.

## 3. New quests (Ashlands, via `WorldCatalog.upsert`)

| Field | steel_for_the_gate | keep_one_fire_lit |
| --- | --- | --- |
| name | Steel for the Gate | Keep One Fire Lit |
| description | The Cinder Gate will not hold another Long Dry on timber. Mara sends you up the south road to Slag Gate for Orsa Tallow's steel. | Grakka Ashmaw has gathered the warbands past the Cinder Gate. Break him and the Horde scatters. |
| giver / turn_in | mara_voss / hest_the_quartermaster | mara_voss / "" |
| faction / region | ember_wardens / ashen_marches | ember_wardens / cinder_pass |
| requires | [warden_oath] | [steel_for_the_gate] |
| requires_flags | [] | [ashlands.steel_spent] |
| template | signal_and_return | reach_and_clear |
| steps | 1: "Load Orsa's steel at Slag Gate", flag `ashlands.steel_loaded`; 2: "Bring the steel to Warden Keep", location warden_keep | 1: "Find the Ashmaw Camp", location ashmaw_camp; 2: "Break Grakka Ashmaw", encounter ashmaw_stand |
| auto_start | true | false |
| complete_flags | ashlands.steel_delivered | ashlands.horde_broken |
| variants | mara_fallen: giver tev_quill | mara_fallen: giver and turn_in tev_quill; steel_to_gate (when `ashlands.steel_to_gate`): steps {"1": text "Break Grakka; the barred gate cut off his flankers", encounter ashmaw_stand_barred} |
| loot_pool / picks | [keep_stores, keep_ring] / 1 | [chieftains_torc, keep_ring, rallying_cry, wardens_drill] / 1 |
| reputation | {ember_wardens: 10} | {ember_wardens: 15, ashen_horde: -20} |
| reward text | Hest's thanks and a pick from the stores. | The Wardens' gratitude and the chieftain's torc. |
| links | follows [warden_oath], leads_to [keep_one_fire_lit], related_characters [mara_voss, hest_the_quartermaster] | follows [steel_for_the_gate], related_characters [mara_voss, grakka_ashmaw], related_locations [ashmaw_camp] |

Existing quest edits: `hold_the_courtyard` description = "The Long Dry has come early, and the Horde with it. Mara needs the Courtyard held while the keep's gates are shored up."; `warden_oath` links.leads_to += steel_for_the_gate. Mara-gone variants are there for the gone-NPC lint; nothing in CAMP1 sets `ashlands.mara_fallen`.

## 4. The choice (Mara's tree)

Mara's `hello` gets a choice "The steel is here. Where does it go?" (requires quest steel_for_the_gate completed; unless `ashlands.steel_spent`) to node `steel_choice`: "Mara weighs a bar of slag-steel. "Enough for one thing done well. The gate, or your hands?""

| Option | Effects | Consequence |
| --- | --- | --- |
| A "Bar the Cinder Gate." | set_flag steel_spent, set_flag steel_to_gate, add_flag rep.ashlands.ember_wardens +10 | Wardens 55 = honoured: Hest prices x0.9 (potion 14, elixir 23, buckler 27, leather 36); finale without the wave |
| B "Arm the party." | set_flag steel_spent, set_flag steel_to_arms, give {arm_plate: 1} | Hest stocks 4 steel rows (below); full finale with the wave |
| C "Let me think." | none | back to `hello` (the finale stays locked until A or B) |

Reply nodes: `gate_barred` "Then the gorge is shut behind you. Whatever waits in the wastes waits alone." and `party_armed` "Then go out sharp. Hest has the rest on his counter."; both `next: hello`. Rep path: courtyard 10, gate 20, oath 35, steel 45, A 55 (B stays 45; the optional ember_patrol adds 5).

Hest stock rows (append to `hest_stores.stock`, all `price -1`, requires_flags [steel_to_arms]): wpn_broadsword x2, wpn_claymore x1, shd_kite x1, arm_plate x1 (friendly prices 34 / 62 / 43 / 76).

## 5. Encounters

All copy the shipped encounter defaults (arena CLOSED, exits [[11,4]], total_blocks 6, ap_regen 4, unit_ap 12, rng_seed 1337, party [mara_voss, tev_quill]; the battle link swaps in the active party), location ashmaw_camp, zone desert_outpost, max_rounds 8.

| id | Enemies | Wave | Tier (BAL2 archetype) |
| --- | --- | --- | --- |
| ashmaw_stand | grakka_ashmaw, ash_shaman, ash_archer | round 2 block 1: ash_berserker, ash_brawler, ash_skirmisher; label "The flankers pour out of the gorge!" | boss |
| ashmaw_stand_barred | grakka_ashmaw, ash_shaman, ash_archer | none | boss, lighter |
| ash_raiders (reused) | road raid on the way back from Slag Gate | - | line |

Intros: "Grakka rises from the fire, nail-crown glinting." / "Grakka looks past you to the gorge. Nobody comes." Step-counter and route fights reuse ash_scouts, ash_raiders, ash_patrol.

New catalog unit `goblin_warlord` (`ContentCatalog.upsert`, units.json): name Goblin Warlord, side enemy, class_id warrior, level 6, tier boss, art berserker, flee_ratio 0.0, reckless false, everything else default. Party starts at level 5; BAL3 tunes the boss band later (G3).

## 6. Characters, dialogues, places, routes, lore, loot

| Kind | id | Data |
| --- | --- | --- |
| character (ashlands) | grakka_ashmaw | "Grakka Ashmaw", title "Chieftain of the Horde", role enemy, base_unit goblin_warlord, faction ashen_horde, dialogue grakka_parley, gone_flags [quest.ashlands.keep_one_fire_lit.ready, ashlands.horde_broken], background "Wears a crown of rusted nails, one for every Warden post he burned." |
| character edit (iron_reaches) | orsa_tallow | dialogue orsa_talk |
| dialogue (iron_reaches) | orsa_talk | services false; hello "Orsa wipes soot from her hands. "Wardens. Thought you'd come.""; choices: "Why sell to the Wardens at cost?" to `why` ("A Warden captain pulled me out of a slag fire. I pay the hearth, not the order."); "Mara sent us for the steel." requires `ashlands.oath_sworn`, unless `ashlands.steel_loaded`, effects set_flag steel_loaded + add_flag rep.iron_reaches.forge_guild +5, to `loaded` ("Forty bars on one wagon. The Horde smells steel on a road, so ride hard."); "Farewell." to "" |
| dialogue (ashlands) | grakka_parley | services false; hello "A goblin in a crown of rusted nails grins over the fire. "Come to beg for your little flame?""; choices: "Leave the pass and live." to `scorn` ("The Long Dry is mine. Your fire goes out tonight.", next hello); "Then we fight." requires quest keep_one_fire_lit active, unless steel_to_gate, effects fight ashmaw_stand; "Then we fight. Your flankers are not coming." requires quest active + steel_to_gate, effects fight ashmaw_stand_barred; "Leave." to "" |
| dialogue edit | tev_talk | hello += "Where do we get steel?" requires oath_sworn, unless steel_delivered, to `steel` ("Slag Gate, up the south road. Orsa sells to us at cost. Take the signpost to the world map.", next hello) |
| dialogue edit | mara_talk | hello += the choice (section 4) and two epilogue choices "It's over. The Horde is scattered." (requires quest keep_one_fire_lit completed + steel_to_gate, or + steel_to_arms; unless campaign_done; effects set_flag `ashlands.campaign_done`) to `end_gate` ("One fire lit, and a door that holds. The Long Dry is over. This year.") or `end_arms` ("The wastes will remember that steel. Rest; you have earned the hearth."); hello ends at 6 authored choices (the cap) |
| location (ashlands, new) | ashmaw_camp | "The Ashmaw Camp", kind camp, region cinder_pass, "A ring of hide tents in the wastes, smoke rising from a dozen fires.", map gorge_ruins, exits [{to cinder_gate}], encounters [ashmaw_stand, ashmaw_stand_barred, ash_raiders, ash_scouts, ash_patrol], zone_bindings ruins: encounters [[ash_raiders,3],[ash_scouts,1]], elites [[ash_patrol,1]], elite_pct 15, steps 10..16; markers: `to_world` exit anchor west to world_map, `grakka` npc anchor site_b, `war_chest` chest anchor ruin_chest {con_heal_potion: 2} gold 40, `patrol` roamer ash_patrol path patrol once |
| location edit (iron_reaches) | slag_gate | map walled_town; markers `orsa` npc anchor npc_a, `signpost` exit anchor road to world_map, `yard_crate` chest anchor chest_a {con_heal_potion: 1} gold 20 |
| location edit (ashlands) | cinder_gate | marker `banner` lore anchor ruin_lore, requires_flags [quest.ashlands.cinder_gate_watch.ready], lore ashmaw_banner |
| route (ashlands, new) | wastes_track | "The wastes track", cinder_gate to ashmaw_camp, two_way, time 3, zone desert_outpost, encounter_pct 30, repeat_pct -1, encounters [[ash_raiders,2],[ash_scouts,1]], elites [[ash_patrol,1]], elite_pct 10, requires_flags [ashlands.steel_spent] |
| route edit (iron_reaches) | south_road | scripted += {id wagon_raid, encounter ashlands:ash_raiders, requires_flags [ashlands.steel_loaded], unless_flags [ashlands.steel_delivered], sets_flags []} |
| lore (ashlands) | ashmaw_banner | category rumor, "The Ashmaw Banner": "A hide banner nailed to the arch, one skull per warband. Somebody is uniting the Horde." |
| loot (ashlands) | chieftains_torc | "Chieftain's Torc", reward_kind item, ref acc_swift_amulet, count 1, rarity unique |

New flags: `ashlands.steel_loaded`, `steel_delivered`, `steel_spent`, `steel_to_gate`, `steel_to_arms`, `horde_broken`, `campaign_done` (all `ashlands.`-prefixed, value true).

## 7. Acceptance checks (asset lane, headless after the seeds)

1. `WorldCatalog.validate_all()` == {} and `ContentCatalog.validate_all()` == {}; `QuestRules.reward_warnings`, `link_warnings`, `DialogueRules.quest_warnings`, `TravelRules.warnings` add nothing for ashlands and iron_reaches.
2. After warden_oath completes, steel_for_the_gate is active (auto start) and Orsa's steel choice is visible at Slag Gate; Orsa offers no quest.
3. Choosing it sets `ashlands.steel_loaded`; the trip to warden_keep stops once for ash_raiders; on arrival the quest is ready (F4) and Hest turns it in (Wardens 45).
4. A gives Wardens 55 and Hest's potion price 14; B gives one arm_plate and shows the 4 stock rows; either way `wastes_track` opens and Mara offers keep_one_fire_lit.
5. With steel_to_gate the finale's step 2 targets ashmaw_stand_barred and Grakka offers only the barred fight; winning makes the quest ready and Grakka disappears.
6. Turning in at Mara sets `ashlands.horde_broken`; the matching epilogue choice sets `ashlands.campaign_done`.
7. If a suite pins shipped node lists or Hest's stock, the builder reports it and the planner routes the test update to Gameplay.

## 8. Tasks

| Id | What | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- |
| CAMP1-A1 | Catalog unit goblin_warlord (section 5) | asset | CAMP1 approved; not beside BAL3-A | data/catalog/units.json, data/seeds/camp1_unit.gd |
| CAMP1-A2 | Story data, sections 3, 4, 6 | asset | CAMP1-A1 | data/worlds/ashlands/*, data/worlds/iron_reaches/*, data/seeds/camp1_story.gd |
| CAMP1-B | Party sim vs ashmaw_stand and ashmaw_stand_barred at L5, report only | balance | CAMP1-A2 merged to main | docs/BALANCE_REPORT.md |
| CAMP1-P | New Game to `campaign_done` playthrough, recorded (G4 finish line; exact overworld replay needs PLAYTEST2) | planner + user | CAMP1-A2 | playtest recording only |

## Out of scope

Engine changes (cross-world quests, F1), an ending screen or credits, new map templates, art and audio for Grakka, price and boss tuning (G3), Tidewrack, TRAV2 supplies, Mara or Tev dying.

## Open decisions for Tushar

1. The choice hook?
   - steel (bar the gate or arm the party) **(recommended)**
   - deserter (a Horde deserter at the Cinder Gate: jail him for Warden rep, or free him for a back way that drops the finale wave; +1 character, +1 dialogue)
2. Ending tone?
   - victory **(recommended)**
   - bittersweet (same mechanics; Mara's epilogue lines mourn the keep's dead)
3. The boss unit?
   - new (goblin_warlord, boss tier; waits for BAL3-A) **(recommended)**
   - reuse (ambush_berserker plus world tuning; no catalog touch, starts now)
4. Slag Gate map?
   - reuse (walled_town, looks like the keep) **(recommended)**
   - new (a mine-gate template; one more asset row)
