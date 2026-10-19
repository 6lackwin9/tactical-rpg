# PURSE1: Encounter purse (authored fight gold)

Status: approved (user 2026-10-18 12:28 UTC, "purse all recs": Q1 A, Q2 A, Q3 A; direction approved: user 2026-10-18, DR1 item 21; spec picked 12:14 UTC)
Depends on: L: none on develop (sequence with other save bumps). A: PURSE1-L commit 1 (schema slice); not alongside MORALE1-A, PACE1-A or ASHPACE1 (same `encounters.json`). B: PURSE1-A and ITEMPOW1-A on main
Lane: logic (PURSE1-L), asset (PURSE1-A), balance (PURSE1-B); no UI task (rule 1.6)
Touches: L: new `scripts/exploration/purse_rules.gd`, `scripts/exploration/battle_bridge.gd`, `scripts/world/{world_schema,world_catalog}.gd` and `scripts/content/{content_schema,content_catalog}.gd` (where encounter fields are validated, as in MORALE1-L), `scripts/game_state/{game_state,save_migrations}.gd`, the three after-fight callers only if their result drops `gold` / `loot` (`ExploreRunner.after_fight`, `TravelRunner`, `DialogueRunner.after_fight`; the builder reports it), new `scripts/tests/purse_test_suite.gd`, `scripts/tests/suite_registry.gd`. A: `data/worlds/ashlands/{encounters,loot}.json` via new `data/seeds/purse1.gd`. B: `games/emberwake/docs/BALANCE_REPORT.md`

## Goal

Today a won fight gives XP and nothing else (`BattleBridge.apply`; decision world-campaign "BATTLE1-L battle link": no kill loot or kill gold). HEAL1 made fights cost HP and supplies, and ITEMUSE1 and ITEMPOW1 made shop items worth buying, so a win should also pay a little. Each encounter may carry an authored `purse`: a gold range rolled on a win and, optionally, one world loot entry. It shows in the reward lines of the results popup. Gold stays authored and boundable per encounter: no per-enemy formula, no drop tables.

## History check

- S11 rules Q4 option A (kill gold `round((2 + level) x tier_mult)` per enemy) was not chosen because "authored income keeps the reward budget the only lever". PURSE1 keeps it authored: every gold amount is a number in the encounter data, so the budget stays one table (section 2). DR1 5.7 gives the new reason: fights pay only XP, and 4.2 / 4.5 (HEAL1 camp, battle items) add things to spend gold on.
- World-campaign "M4 variant answers + world loot rewards": special rewards come from the shared world loot table and leave every pool once claimed. The purse `loot` is a loot-table id claimed the same way, never an item list.
- HEAL1 3.3.3 and HISTORY "Injuries and a defeat cost": a defeat never costs gold. The purse pays nothing on a loss and takes nothing either.

## 1. Rules (logic lane)

1. **Win only.** The purse pays when `BattleBridge.apply` gets `won`, after XP and the quest hook. `lost` and `fled` (party withdrew, or the round limit) pay nothing and record nothing (Q2).
2. **A morale break is a win.** MORALE1 rule 11: a break win is `won`, so the purse pays in full. Fled and surrendered enemies count as beaten, as for XP.
3. **Gold roll.** `gold = rng.randi_range(gold_min, gold_max)` on a fresh `RandomNumberGenerator` seeded with `("%s|%d|%d" % [encounter_qid, gs.time, paid]).hash()`, where `paid` = `GameState.purses[encounter_qid]` before this win (0 if absent). It never draws from the combat RNG, so battles and recordings replay identically. Same save, same fight, same roll: reloading does not reroll.
4. **Repeat wins (Q1).** `once: true` pays only while `paid` is 0 (story fights; re-fightable markers such as the courtyard `warband`, `once: false` today). `once: false` pays every win (step-counter mobs, road fights, roamers in shared lists). `paid` += 1 on every win of an encounter that has a purse, paid or not.
5. **Loot.** On a paying win with `loot` set: if the entry is unclaimed and its `requires_flags` hold, `RewardGrant.claim_loot` marks it claimed (it leaves every quest pool) and `RewardGrant.apply(gs, {}, "", [entry])` puts the item in the stash. Already claimed or gated: skipped silently, the gold still pays. So loot drops at most once per save, whatever `once` says.
6. **Where it shows.** `BattleBridge.apply` returns `purse: {gold, loot (qid or ""), notes}` and the settled fight result carries top-level `gold` and `loot`; `BattleResultsModel.rewards_of` already lifts them into the reward lines ("Gold +12", the loot name). No view change, no number before the fight. BREPORT1's per-hero report is separate and unchanged.
7. **Gold cap.** Gold goes through `PartyState.add_gold` (clamp at `GOLD_MAX`, RewardGrant note), as quest gold does.
8. **Qualified ids.** A purse belongs to its encounter's world: `wagon_raid` fought from an `iron_reaches` route records `ashlands:wagon_raid` and resolves its loot in `ashlands`.
9. **Old saves.** Migration sets `purses` to `{}`; a story fight won before PURSE1 pays once more if re-fought (accepted: small and once).

## 2. Economy target (Ashlands, one full run, party of three)

Income today (after ITEMPOW1-A) vs. with purses (data plan in section 4). Wins per run are estimates; PURSE1-B measures them.

| Source | Today | With PURSE1 |
| --- | --- | --- |
| Start gold (`world.json`, ITEMPOW1-A) | 100 | 100 |
| Quest gold (6 quests: 30, 40, 20, 40, 20, 30) | 180 | 180 |
| Chests (stores 10, ruin cache 25, camp cache 15, war chest 40) | 90 | 90 |
| Dialogue gifts (Mara 20, Tev 15) | 35 | 35 |
| Purse, story fights (4 once: courtyard, ambush, wagon, one stand) | 0 | 115-140 |
| Purse, elites (2 patrol roamers) | 0 | 24-36 |
| Purse, mobs and road fights (about 10 wins, mean about 5) | 0 | about 50 |
| **Total (selling on top, about 30-60)** | **405** | **about 600 (band 550-650)** |

What Hest sells: gear 330 (buckler 30, leather 40, broadsword 2 x 35, claymore 65, kite 45, plate 80), elixirs 5 x 25 = 125, bombs 3 x 40 = 120, Keep Ring 150: 725 of limited stock. Potions 15 each (unlimited), supplies 2 each (a camp = 3 = 6 gold). Target: about 600 buys the gear a trio uses, a potion habit (about 8 = 120) and a few camps, but not the whole counter, so the Keep Ring or the bombs stay a real choice. Today's 405 buys about half. Mob anchor: a mob purse (mean 4-7) is about one camp (6), so grinding mobs barely pays for the rest it costs: no farm cap needed.

## 3. Schema contract (logic lane, schema-first: yes)

Encounter field `purse`: Dictionary, default `{}` (no purse; old entries normalise to `{}` and play identically). Allowed keys (unknown keys are errors):

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `gold_min` | int | 0 | 0..`PURSE_GOLD_CAP` (500); not an int: error |
| `gold_max` | int | `gold_min` | `gold_min`..500; above the soft cap 100: warning "Purse gold above 100: check the budget." |
| `loot` | String | "" | a loot id of the encounter's world (or a qualified `world:id`) that resolves; its `reward_kind` must be `item` (no recipient picker on results): error otherwise |
| `once` | bool | true | bool |

Messages: "Purse key '%s' is not allowed.", "Purse gold_min must be a whole number 0..500.", "Purse gold_max must be gold_min..500.", "Purse loot '%s' does not resolve.", "Purse loot '%s' must be an item reward." Warning: a non-empty purse with `gold_max` 0 and no `loot` ("Empty purse."). Safe default `once: true`: a purse added without thinking cannot be farmed.

State: `GameState.purses: Dictionary` {qualified encounter id: int wins} (saved; next save version, v9 -> v10 today, the builder takes the next free number; migration `{}`).

API (`PurseRules`, static, typed): `roll(gs, encounter_qid) -> Dictionary` (pure: `{gold, loot, pays: bool}`), `pay(gs, encounter_qid) -> Dictionary` (applies rule 1.3-1.5, returns `{gold, loot, notes}`).

Commit 1 (schema slice): field, normalise, validation, `GameState.purses` and migration, tests for validation. Commit 2: `PurseRules`, `BattleBridge.apply` hook, result plumbing, runtime tests. PURSE1-A starts after commit 1.

## 4. Data plan (asset lane, `data/seeds/purse1.gd`, `ContentCatalog.upsert` only)

| Encounter | Kind | gold_min | gold_max | once | loot |
| --- | --- | --- | --- | --- | --- |
| `courtyard_skirmish` | story (B1, marker re-fightable) | 15 | 20 | true | |
| `pass_ambush` | story (B2, marker re-fightable) | 25 | 30 | true | |
| `wagon_raid` | story (B3, scripted road) | 25 | 30 | true | |
| `ashmaw_stand` | boss (arms) | 50 | 60 | true | |
| `ashmaw_stand_barred` | boss (gate) | 50 | 60 | true | |
| `ash_patrol` | elite (roamers, high_road) | 12 | 18 | false | `warchiefs_axe` |
| `ash_raiders` | mob, 3 units | 5 | 9 | false | |
| `ash_scouts` | mob, 2 units | 3 | 6 | false | |
| `ash_straggler` | mob, 1 unit | 2 | 4 | false | |

New loot entry in `data/worlds/ashlands/loot.json` (Q3): `warchiefs_axe`, name "Warchief's Axe", description "A notched Horde axe, still warm from the warchief's grip.", `reward_kind` item, `ref` `wpn_battle_axe`, count 1, rarity rare, `requires_flags` [], tags []. It is in no quest pool. Only one of the two stands is ever fought (the steel choice), so the boss rows count once.

## 5. Acceptance checks (`purse_test_suite`, plus existing suites)

- Validation: each message above fires for its bad entry; a valid purse is clean in both catalogs; an entry with no `purse` normalises to `{}`.
- No purse: `BattleBridge.apply` result, party gold and `purses` are identical to today for won, lost and fled.
- Purse 10-10, `won`: gold +10, `purse.gold` 10, `purses[qid]` 1; `lost` and `fled`: gold and `purses` unchanged.
- Roll: same `gs.time` and `paid` give the same gold; every roll is in range; over `paid` 0..199 a 3-6 range hits both 3 and 6.
- `once: true`: the second win pays 0 and adds no `gold` key, `purses[qid]` becomes 2. `once: false`: every win pays.
- Loot: the first paying win stashes `wpn_battle_axe` and claims `warchiefs_axe`; the second win gives gold only; a loot already claimed by a quest is skipped and the gold pays; unmet `requires_flags` leave it unclaimed.
- Morale: a break win of a purse encounter pays the same as a full rout (MORALE1 rule 11).
- Determinism: one shipped playtest recording replays with identical `RoundOutcome`s (no combat RNG draw).
- Save: `purses` round-trips; the migration from the previous version gives `{}`.
- Results: `BattleResultsModel.rewards_of(settled res)` holds `gold` and `loot`; the reward lines show "Gold +N" (map, road and dialogue fights).
- Cross-world: `ashlands:wagon_raid` won from an `iron_reaches` route records the qualified id.

## 6. Balance check (PURSE1-B, report only, no tuning)

After PURSE1-A and ITEMPOW1-A on main: tally a full Ashlands run from data (authored sources plus purse means x wins), with wins per run from route `encounter_pct`, step thresholds and the DR2 (9) recorded run when it exists. Report total gold against the band 550-650 and against the Hest stock (section 2), plus mob purse mean vs the 6-gold camp cost. If outside the band, the lever is the mob rows' ranges (data), never quest or chest gold.

## Out of scope

Kill gold or per-enemy formulas, random drop tables, item lists in a purse, purses on other worlds (one world first), showing the purse before a fight (world preview, intro), shop price or stock changes, quest or chest gold changes, gold loss on defeat, sim gold tracking, new UI.

## Open questions

1. Repeat wins: A (recommended) story and boss purses pay once, mobs, elites and road fights every win (rule 1.4); B every purse pays once (mobs pay their first win only; simplest, fights stop paying); C every win pays (re-fighting the courtyard marker becomes a farm).
2. Fled battles: A (recommended) nothing, same as XP; B half the roll if at least half the counted foes are down, fled or surrendered (kill gold through the back door, more rules).
3. Loot in the Ashlands: A (recommended) one new entry, `warchiefs_axe` on `ash_patrol` (the first patrol you beat drops it); B gold only for now; C also a loot on both stands (needs a second new entry, since `chieftains_torc` is a quest pool reward).
