# REWARD1: Rewards per defeated enemy

Status: approved (user 2026-10-20: all eight open questions with the recommended answers; 1, 2, 4 and 7 asked one by one, 3, 5, 6 and 8 confirmed as a set)
Depends on: L: PURSE1-L (done); after RUNNER2 (same `emberwake_world` battle path and `emberwake_gui/src/runner.rs`). S: REWARD1-L commit 2. A: REWARD1-L commit 1 (schema slice); not alongside MORALE1-A, PACE1-A or ASHPACE1 (same `encounters.json`). B: REWARD1-A and REWARD1-S on main
Lane: logic (REWARD1-L, REWARD1-S), asset (REWARD1-A), balance (REWARD1-B)
Touches: L: `games/emberwake/crates/emberwake_battle/src/session.rs` (`Session::fates`), `games/emberwake/crates/emberwake_world/src/{battle,purse,progression,rolls,state,save,explore,travel,dialogue}.rs`, `games/emberwake/crates/emberwake_content/src/check/{unit,encounter,world,mod}.rs`, `games/emberwake/schema.json`, `games/emberwake/crates/emberwake_gui/src/{battle,hub,runner}.rs`, `games/emberwake/crates/emberwake_world/tests/{journey,saves}.rs`, new `games/emberwake/crates/emberwake_world/tests/rewards.rs`. S: `games/emberwake/crates/emberwake_sim/src/{runner,campaign,party}.rs`. A: `games/emberwake/data/catalog/units.json`, `games/emberwake/data/worlds/ashlands/{encounters,loot}.json` via `aegis content` (seed `games/emberwake/data/seeds/reward1.json`). B: `games/emberwake/docs/BALANCE_REPORT.md`

## Goal

Today a fight pays only on `won` (`emberwake_world::battle::apply`): XP for every enemy ref of the encounter, waves included, the quest hook and the PURSE1 purse; `lost` and `fled` pay nothing. The user's rule (2026-10-20) is that a fight pays for what the party beat, whatever the outcome: every enemy that was killed, surrendered or fled on its own gives its XP, its share of the encounter purse and a roll on its unit type's loot table. Key loot (a named world loot entry) needs that enemy killed and the fight won. An objective may carry its own bonus. A Scene Runner skip counts every enemy as killed. This spec makes the per-unit defeated test exact, picks the weight and the division, adds the loot-table, key-loot and objective-reward fields, and lists the save, parity, validation and balance impact.

## History check

- S11 rules Q4 option A (kill gold `round((2 + level) x tier_mult)` per enemy) was not chosen because "authored income keeps the reward budget the only lever"; PURSE1 kept that. That reason still holds for gold, so this spec does not add a per-unit gold number: the purse total stays authored per encounter and is only divided among the enemies (section 1, option G1). The per-unit gold option (G2) would re-propose S11 Q4 option A, and nothing in the new decision removes its reason.
- World-campaign "BATTLE1-L battle link" and the S11 baseline said "no kill loot or kill gold". The REWARD1 decision (user 2026-10-20) replaces the "no kill loot" part with per-unit-type loot tables; kill gold stays out (above).
- PURSE1 rules 1 and 2 (purse on a win only; fled and lost pay nothing) and PURSE1 rule 5 (`purse.loot` paid on any win) are replaced by sections 4 and 5 here. PURSE1's roll seed, gold range, `once` flag, gold cap and qualified ids stay.
- World-campaign "M4 variant answers + world loot rewards": key loot is a world loot entry claimed the same way (it leaves every quest pool), never an item list.
- HEAL1 and HISTORY "Injuries and a defeat cost": a defeat costs supplies or a walk back, never gold or items. REWARD1 pays on a loss but takes nothing; the HEAL1 rules are unchanged.
- M3: every active member gets the full XP of the defeated enemies, downed members included. Unchanged, now on every outcome.

## 1. Options considered and the recommendation

How each enemy's share is weighed and divided:

| Option | Rule | Effect | Cost | History |
| --- | --- | --- | --- | --- |
| G1 Purse split by kill weight (recommended; planner's suggestion) | weight = the enemy's kill XP at equal level (tier and level, from `xp_curve.json`); gold share = purse roll x defeated weight / roster weight | Budget stays one authored number per encounter; a unit's share follows tier and level with no new data; XP and gold follow the same weights | One pure function over existing data | Keeps S11 Q4 / PURSE1 reason |
| G2 Per-unit gold value | each unit type carries `gold`; a fight pays the sum over defeated units | Income follows composition, not the author; story fights pay like mob fights (S11 formula: courtyard 6 instead of 15-20, Ashmaw stand 55, raiders 9) | New field on every unit type plus retuning PURSE1 | Re-proposes S11 Q4 option A; its reason still holds |
| G3 Tier-only share points | minion 1, standard 2, elite 4, boss 10, ignoring level | Same as G1 at equal level; a higher-level foe of the same tier is not worth more | Small table in tuning | None |
| G4 Authored share per encounter | `purse.shares` {ref: weight} | Exact control, but every encounter needs a hand table | Field and validation | None |

Which enemies are in the divisor:

| Option | Divisor | Effect |
| --- | --- | --- |
| D1 Every roster enemy (recommended) | enemies + `wave_units` + phase-wave units that resolve | Each enemy's share is fixed before the fight and never moves; a wave that never came gives nothing, as the decision says; fleeing before a wave pays less per kill than staying |
| D2 Enemies that took the field (planner's suggestion) | only units that were placed on the grid | A wave that never came hands its share to the units that did: a fast win or an early flight pays the full purse for fewer foes, so "waves that never spawned give nothing" holds for XP but not for gold |

Where loot tables live:

| Option | Place | Effect | Cost |
| --- | --- | --- | --- |
| L1 Inline field `loot` on the catalog unit type (recommended) | `games/emberwake/data/catalog/units.json` | Exactly "a per-unit-type table"; lore-free base items; every world character on the base unit drops the same (GAME_BRIEF: base asset decides gameplay) | One `json` field and its check |
| L2 New catalog kind `loot_tables` plus a unit field naming one | new `catalog/loot_tables.json` | Tables shared across unit types | New kind in `check/mod.rs` KINDS / PORTED, store, Forge |
| L3 World data per character | world `characters.json` | Lore can change drops; two characters on one base unit stop behaving alike | Breaks the base-asset rule |

How a table rolls: R1 (recommended) one roll per defeated unit, rows hold a percent chance each, chances add up to at most 100, the rest is "nothing", so a unit drops 0 or 1 row and each row's chance is its true probability. R2 every row rolls on its own (several drops per unit possible, harder to budget).

Recommendation: G1 + D1 + L1 + R1. The weight already follows tier and level, the purse stays the only gold lever, every enemy's XP and gold share are fixed numbers known before the fight, and the user's 1 elite + 4 mob example lands at 50% (equal level) to 55% (elite one level up), section 3. A kit layer does not carry data yet (ASSETORG2-K is assets only), so the table sits in the game catalog; it is lore-free and can move to the RPG kit later.

## 2. The defeated test (logic lane)

1. **Roster.** An encounter's roster is, in this order: `enemies`, `wave_units`, then the `units` of every `wave` effect of every phase (phase order, effect order), keeping only refs that resolve to a unit (the same refs `battle::enemy_levels` reads and `Session::start_with` spawns). Guests, protected guests and party refs are never in it. Each roster unit has a key `ref#n`, where n counts that ref's earlier roster entries (0-based): `ash_brawler` in `enemies` and again in `wave_units` gives `ash_brawler#0` and `ash_brawler#1`.
2. **Fate.** At the end of the battle each roster unit has exactly one fate, read from the session units:
   - `killed`: `!is_alive()` (HP 0), whoever dealt the blow (heroes, guests, a friendly bomb, round-end damage).
   - `surrendered`: `is_withdrawn` and its morale key is in `Morale.surrendered` (MORALE1: a broken group withdraws in place).
   - `fled`: `is_withdrawn` and not surrendered (it stepped onto an exit or off an open arena, AI `flee_ratio`).
   - `standing`: alive and not withdrawn. This covers units still fighting, a `wave_units` wave that never entered (`reinforcements_spawned` false), phase-wave units still reserved because their phase never fired, and wave units that found no free tile.
3. **Defeated** = `killed`, `surrendered` or `fled`. The same in every fight: rout, hold, escape, slay, survival and wave fights. A hold or escape win, or a slay win, leaves the other foes `standing`, and they give nothing.
4. **Source of the fates.** `Session::fates() -> Vec<(String, Fate)>` (emberwake_battle) lists every enemy-side unit in spawn order (enemies, wave, phase waves) with its ref; `emberwake_world::battle::Tally::from_session` keys them by rule 1. The GUI battle `settle` builds it before calling `after_fight`.
5. **No battle.** `after_fight(..., tally: None)` uses the default tally: `won` = every roster unit `killed`; `lost` and `fled` = every roster unit `standing`. Used by the dev outcome buttons (`DebugFight`), the journey replay, and the RUNNER2 skip (decision "Skipped fights (RUNNER2)": every enemy defeated, full rewards).
6. **Mismatch.** A tally entry whose ref is not in the roster is ignored with a note "Tally foe '%s' is not in the encounter."; a roster key missing from the tally is `standing`.

## 3. Weight and shares (logic lane)

1. **Weight points** of a roster unit: `weight = roundi(100 x (base + per_level x level) x tier_mult)`, minimum 1, with `base`, `per_level` and `tier_mult` from `games/emberwake/data/progression/xp_curve.json` `kill_xp` (10, 5; minion 0.5, standard 1, elite 2, boss 5, hero 1) and the level and tier of the unit's base unit entry (`Data::character_unit_entry`). It is `kill_xp` at equal level, scaled to integers. New pure function `progression::kill_weight(d, level, tier) -> i64`. The roster weight `W` is the sum over the roster (D1).
2. **XP.** Unchanged formula: each active member gets `sum of kill_xp(level, tier, member_level)` over the defeated units only (`progression::award_encounter` with the defeated list), downed members included. No defeated unit: no award at all (`xp` is `{}`).
3. **Gold roll.** As PURSE1 rule 3: `G = randi_range(gold_min, gold_max)` on a `GodotRng` seeded with `string_hash("<qid>|<time>|<paid>")`, `paid` = `purses[qid]` before this fight. Rolled only when the purse exists and at least one unit is newly paid (rule 4).
4. **Share.** `paid_weight` = sum of weights of the defeated units that are paid this fight (all defeated units; for a `once` purse only those whose key is not yet in `reward_paid[qid]`). Gold share = `G` when `paid_weight == W`, else `floor(G x paid_weight / W)` in integer arithmetic. So a full clear pays the exact roll, a part pays rounded down, and the parts of a `once` purse over several fights never add up to more than `gold_max`.
5. **Repeat fights.** `once: false`: every fight pays its defeated share. `once: true`: each roster key pays its share once per save; a flight that killed two of four pays their share, and the later win pays the other two. `purses[qid]` += 1 on every settled fight of a purse encounter in which at least one unit was defeated (paid or not), so the next roll differs.

Weights and XP (party at level 5, the Ashlands start level):

| Unit | Tier, level | Weight | XP per hero at L5 |
| --- | --- | --- | --- |
| goblin minions (all `ash_*` grunts, Skarn, Vex, Nok, Gruk) | minion 3 | 1250 | 9 (8.75) |
| Horde Warchief (`ambush_berserker`) | standard 4 | 3000 | 26 (25.5) |
| an elite at level 4 (none shipped) | elite 4 | 6000 | 51 |
| Grakka Ashmaw (`goblin_warlord`) | boss 6 | 20000 | 230 |

Worked numbers (purse ranges from PURSE1 section 4, which REWARD1-A ships; G is one example roll):

| Encounter, roster (W) | Case | Gold | XP each | Other |
| --- | --- | --- | --- | --- |
| User example: 1 elite L4 + 4 minions L3 (11000); elite 54.5%, each mob 11.4% (elite L3: exactly 50%) | the party kills the four mobs and flees from the elite, G 20 | floor(20 x 5000 / 11000) = 9 | 36 | drop rolls x4 |
| `ash_patrol`: warchief + berserker (4250); warchief 70.6% | win, both killed, G 15 | 15 | 35 | Warchief's Axe (key loot), drop rolls x2 |
| same | berserker killed, the party flees | floor(15 x 1250 / 4250) = 4 | 9 | no axe |
| same | warchief killed, the party is beaten | floor(15 x 3000 / 4250) = 10 | 26 | no axe (a loss pays only common rewards) |
| same | win, berserker killed, warchief fled on its own | 15 | 35 | axe missed (not killed) |
| `ash_raiders`: 3 minions, morale (3750) | shaman killed, brawler surrenders, the party flees, G 7 | floor(7 x 2500 / 3750) = 4 | 18 | drop rolls x2 |
| `pass_ambush`: 2 minions + 2-minion wave at round 1 block 4 (5000) | both first goblins killed, the party flees before the wave, G 28 | D1: 14 (D2 would pay 28) | 18 | |
| `courtyard_skirmish`: hold, Skarn + Vex (2500) | hold win, Vex killed, Skarn standing, G 18 | 9 + objective 10 | 9 | Skarn gives nothing |
| same | hold win, nobody killed | 0 + objective 10 | none | |
| `ashmaw_stand`: Grakka + shaman + archer, wave of 3 at round 2 block 1, slay (26250); Grakka 76.2% | Grakka slain in round 1, shaman and archer standing, G 55 | D1: floor(55 x 20000 / 26250) = 41 (D2: 48) | 230 | |
| same | every foe down, Grakka last | 55 | 275 | |
| same | beaten after killing shaman and archer | floor(55 x 2500 / 26250) = 5 | 18 | |

## 4. What each outcome pays (logic lane)

| Reward | won | fled (party withdrew or round limit) | lost |
| --- | --- | --- | --- |
| XP per defeated unit (common) | yes | yes | yes |
| Purse share per defeated unit (common) | yes | yes | yes |
| Loot-table roll per defeated unit (common) | yes | yes | yes |
| Key loot (its `kill` unit killed) | yes | no | no |
| Objective reward | yes | no | no |
| Quest hook (`quest::on_encounter_won`), marker done flag and `sets_flags`, scripted road event | yes (as today) | no | no |
| HEAL1 rally / flee / wake rules | as today | as today | as today |

A RUNNER2 skip is `won` with every unit killed: every row of the `won` column, full purse, no cost.

## 5. Schema contract (logic lane, schema-first: yes)

### 5.1 Unit field `loot` (catalog units)

`loot`: Array, default `[]` (drops nothing; old entries normalise to `[]` and play identically). Schema row: `{"key": "loot", "label": "Loot table", "type": "json", "default": [], "help": "[{\"item\": \"catalog item id\", \"chance\": 3, \"count\": 1}] (REWARD1): one roll per defeated unit, at most one row drops; chance in % (1-100), chances add up to at most 100, the rest drops nothing. [] = no loot."}`.

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `item` | String | (required) | a catalog item id that resolves |
| `chance` | int | (required) | 1..100 |
| `count` | int | 1 | 1..`LOOT_COUNT_MAX` (9) |

Constants (`emberwake_world::battle` or a new `reward.rs`, the builder picks; typed): `LOOT_COUNT_MAX = 9`, `LOOT_CHANCE_TOTAL = 100`, `LOOT_EV_SOFT` by tier (item value per defeat, warning only): minion 5, standard 10, elite 20, boss 30, hero 0.

### 5.2 World encounter field `key_loot`

`key_loot`: Array, default `[]`. World encounters only (catalog encounters have no world loot). Schema row help: `[{"loot": "world loot id", "kill": "roster ref"}] (REWARD1): paid on a win in which every unit of that ref was killed (not surrendered, not fled); claimed once per save.`

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `loot` | String | (required) | a loot id of the encounter's world (or qualified `world:id`) that resolves, `reward_kind` `item` |
| `kill` | String | (required) | a ref in the encounter's roster (section 2 rule 1) |

### 5.3 World encounter field `objective_reward`

`objective_reward`: Dictionary, default `{}` (none). World encounters only. Paid on `won` (any win completes the objective, since every objective text says "or defeat every enemy").

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `gold` | int | 0 | 0..500 (`purse::GOLD_CAP`); above 100: warning |
| `loot` | String | "" | as `key_loot.loot` |
| `once` | bool | true | bool |

### 5.4 Purse change

`purse.loot` is retired (replaced by `key_loot`); `gold_min`, `gold_max` and `once` keep their PURSE1 meaning and validation. No shipped entry has a purse yet (PURSE1-A not run), so nothing migrates. The schema help of both `purse` rows becomes `{"gold_min": 0, "gold_max": gold_min, "once": true} (PURSE1, REWARD1): gold 0..500, split among the enemies by kill weight and paid per defeated enemy on any outcome; once = each enemy's share pays once per save. {} = no purse.`

### 5.5 State and API

- `GameState.reward_paid: Dict` {qualified encounter id: [string]}: the roster keys (`ref#n`) whose `once` purse share was paid, plus the word `objective` once a `once` objective reward was paid (keys always contain `#`, so the word cannot clash). Saved.
- `GameState.purses` {qid: int}: now counts settled fights with a defeated unit (section 3 rule 5); old values (wins) carry over.
- `GameState.rolls["loot"]`: the counter of the new GameRolls stream `rolls::LOOT = "loot"` (inside the existing `rolls` dict).
- `emberwake_battle::session::Fate` {Killed, Surrendered, Fled, Standing}; `Session::fates()`.
- `emberwake_world::battle::Tally` (`foes: Vec<Foe>`, `Foe {key, unit_ref, fate}`) with `from_session(&Session)`, `default_for(d, world, qid, outcome)`; `battle::roster(d, world, qid) -> Vec<RosterFoe {key, unit_ref, level, tier, weight}>` (`enemy_levels` becomes a view of it).
- `battle::apply(d, gs, encounter_qid, outcome, tally: &Tally) -> {ok, errors, xp, advanced, rewards}`; the three `after_fight` functions take `tally: Option<&Tally>` (None = rule 2.5).

## 6. Runtime rules (logic lane)

`battle::apply` on any valid outcome, in this order:

1. Fates from the tally (section 2); `defeated` = the defeated roster units in roster order.
2. XP (section 3 rule 2).
3. `won` only: the quest hook, as today.
4. Purse share (section 3 rules 3-5); records `reward_paid` keys for `once` purses and bumps `purses[qid]`.
5. Loot-table rolls: for each defeated unit in roster order whose base unit has a non-empty `loot`: one `rolls::range_i(gs, rolls::LOOT, 0, 99)` draw `r`; walk the rows in order, subtracting each `chance`; the first row with `r < chance` drops `count` x `item` into `party.stash` (`add_to_stash`). `r` past every row: nothing. A unit with an empty table draws nothing, so the stream only moves when a table exists. The stream never touches the combat RNG, so battles and recordings replay identically; the counter is saved with the stash in the same state change, so reloading after the fight does not reroll.
6. `won` only, key loot: for each `key_loot` row in order, if every roster unit of `kill` is `killed` and the entry is unclaimed and its `requires_flags` hold: `quest::reward_apply(..., [entry])` and `quest::claim_loot` (as PURSE1 rule 5). If the entry is unclaimed and its flags hold but the kill or the win is missing, it goes to `key_missed`. A claimed or gated entry is skipped silently.
7. `won` only, objective reward: if set and (`once` false or `objective` not in `reward_paid[qid]`): gold and loot as rule 6 (loot claimed or gated: skipped, the gold still pays); `once` records `objective`.
8. Gold: share plus objective gold in one `reward_apply` (`PartyState.add_gold` clamp, as quest gold).

Result `rewards` (always present on an `ok` result): `{foes: int, defeated: {slain, surrendered, fled}, gold: int (share), drops: [{item, count, from (unit name)}], key_loot: [loot qid], key_missed: [{loot, kill}], objective: {gold, loot}, notes: [string]}`. The `after_fight` results carry `rewards` and a top-level `gold` (share + objective gold) when above 0, as today; the top-level `loot` key is retired (its only source was `purse.loot`).

## 7. Results screen text (`emberwake_gui/src/hub.rs` `Results`)

- Head unchanged: "Victory", "Defeat", "Fled".
- Tally line under the head, when the roster is not empty: "Foes defeated: 3 of 5 (2 slain, 1 surrendered)." Parts with 0 are left out; when every defeated foe was slain the bracket is left out ("Foes defeated: 5 of 5."). Fled foes read "1 fled".
- Note line (replaces today's "... No XP." lines, which are wrong once losses pay): lost with a defeated foe: "The party was beaten, but keeps the rewards of the foes it defeated."; lost with none: "The party was beaten before it defeated a foe. No rewards."; fled with a defeated foe: "The party got away with the rewards of the foes it defeated."; fled with none: "The party got away. No foe was defeated, so no rewards." Won: no note.
- XP rows per member as today (none when `xp` is `{}`).
- Reward lines, in this order, each only when non-empty: "Gold +N" (the share); "Objective bonus: Gold +N, Keep Ring" (the loot name); "Dropped: Shortbow, Healing Potion x2" (same items merged, item names); "Key loot: Warchief's Axe"; "Missed: Warchief's Axe (slay Horde Warchief and win)" (one line per `key_missed`, Q5); "Quests advanced: ..." as today; else "No other rewards."
- `Results::state()` adds `rewards` for the agent link.

## 8. Save impact

- Next save version (v11 -> v12 today: SKILLUI1-L took 11; the builder takes the next free number). Step: `reward_paid` = `{}`, then for every `purses[qid] >= 1` whose encounter resolves and has a `once: true` purse, every roster key of that encounter (a story fight won before REWARD1 does not pay again). `save.rs` `KEYS` gains `reward_paid`.
- Validation: `reward_paid` must be a dictionary of string arrays; anything else is a load problem (as the other keys).
- `rolls["loot"]` and the new meaning of `purses` need no shape change. Key loot uses `loot_claimed`, as today.

## 9. Parity retired (logic lane, same commit as the rule change)

Godot paid XP and the purse only on `won`, for every enemy ref, and nothing on `lost` / `fled` (`BattleBridge.apply`). The journey replay passes no tally, so the default tally (section 2 rule 5) gives Godot's numbers: won = every enemy, lost and fled = none; the journeys keep comparing XP, quests, flags, shops and travel. The Godot exports the journeys and saves load have no `loot`, `key_loot` or `objective_reward`, so no drop or key loot appears there. Retired, by name:

1. `games/emberwake/crates/emberwake_world/tests/journey.rs`: the after-fight result key `rewards` and the state key `reward_paid` are not compared (Godot has neither). The diff skips exactly these two keys.
2. `games/emberwake/crates/emberwake_world/tests/saves.rs`: the loaded state of a Godot fixture is compared without `reward_paid` (the v12 step adds it).
3. No battle trace or world battle parity changes: `Session::fates` only reads the session.

HISTORY rows (for the planner):

| Date | Concept | Status | Replaced by | Why |
| --- | --- | --- | --- | --- |
| 2026-10-20 | Battle rewards on a win only (`BattleBridge.apply`: XP for every enemy ref, waves included, and the PURSE1 purse on `won`; nothing on `lost` / `fled`; world-campaign "BATTLE1-L battle link", PURSE1 rules 1-2) | replaced (REWARD1) | Rewards per defeated enemy (decision "Rewards per defeated enemy (REWARD1, user 2026-10-20)", spec `games/emberwake/docs/specs/REWARD1.md`): XP, a kill-weight share of the purse and a unit-type loot roll for every enemy killed, surrendered or fled on its own, on any outcome; key loot needs the kill and a win; objective rewards | User 2026-10-20: a fight pays for what the party beat, also when it flees or falls. Parity retired: journey result key `rewards` and state key `reward_paid`, save fixtures' `reward_paid`; the journeys still match Godot through the default tally |
| 2026-10-20 | "No kill loot" (world-campaign "BATTLE1-L battle link", S11 baseline) | replaced (REWARD1) | Per-unit-type loot tables (`loot` on catalog units, one roll per defeated unit) | User 2026-10-20; kill gold stays out (S11 Q4 reason holds: the purse stays the authored gold budget) |
| 2026-10-20 | PURSE1 `purse.loot` (one world loot entry paid on any win) | replaced (REWARD1, before any data used it) | `key_loot` rows {loot, kill} on world encounters | Decision REWARD1: key loot needs that enemy killed |

## 10. Validation (`aegis content check`)

Errors (unit `loot`): "Loot must be a list of rows." / "Loot row %d: key '%s' is not allowed." / "Loot row %d: item '%s' does not resolve." / "Loot row %d: chance must be a whole number 1..100." / "Loot row %d: count must be a whole number 1..9." / "Loot item '%s' is listed twice." / "Loot chances add up to %d (at most 100)."
Warnings (unit `loot`): "Party-side unit '%s' has a loot table; party units never drop." / "Loot drops special item '%s'." / "Loot is worth %d gold per defeat on average (soft cap %d for %s)." (EV = sum of chance x item `value` x count / 100, against `LOOT_EV_SOFT` of the unit's tier).
Errors (`key_loot`, world encounters): "Key loot must be a list of rows." / "Key loot row %d: key '%s' is not allowed." / "Key loot '%s' does not resolve." / "Key loot '%s' must be an item reward." / "Key loot kill '%s' is not an enemy of this encounter." / "Key loot '%s' is listed twice."
Warnings (`key_loot`): "Key loot kill '%s' is listed %d times: every one must be slain." / "Key loot '%s' is also in a quest pool: whichever claims it first wins." A kill target that can surrender or flee is allowed (no warning): the kill is the failsafe.
Errors (`objective_reward`): "Objective reward key '%s' is not allowed." / "Objective reward gold must be a whole number 0..500." / "Objective loot '%s' does not resolve." / "Objective loot '%s' must be an item reward." Warnings: "Objective reward gold above 100: check the budget." / "Empty objective reward."
Errors (`purse`): "Purse key 'loot' moved to key_loot (REWARD1)." (catalog and world); the PURSE1 gold messages stay.
Save: `reward_paid` not a dictionary of string arrays: "reward_paid must map encounter ids to lists of keys."

## 11. Acceptance checks (`tests/rewards.rs`, plus the existing suites)

- Fates: a session where one foe dies, one surrenders (morale) and one walks off the exit gives killed / surrendered / fled; a `wave_units` wave that never entered and a reserved phase wave are standing; keys follow rule 2.1 for a ref listed twice.
- Default tally: `won` without a tally pays the XP `enemy_levels` paid before REWARD1 (same totals for every Ashlands encounter); `lost` / `fled` without a tally pay nothing and leave state as today.
- XP: a fled fight with one minion L3 killed gives 9 XP to each L5 member, downed members included; nothing defeated gives `xp` `{}`.
- Shares: every row of section 3's worked table, from a test purse with `gold_min = gold_max = G`; a full clear pays exactly G; the shares of every subset never exceed G.
- `once`: flee after two of four keys, then win: the second fight pays only the other two shares and the total is at most `gold_max`; a third win pays 0 gold and still awards XP and drop rolls; `purses[qid]` counts 3.
- Loot table: a unit with rows 3 / 3 / 2 over `rolls::LOOT` draws 0..99 drops each row exactly for its range and nothing for 8..99; an empty table draws nothing (`rolls["loot"]` unchanged); chances add up to 100 always drop.
- Key loot: win with the kill unit killed claims the entry and stashes the item; the same unit surrendered or fled: not paid, listed in `key_missed`; killed then `fled` or `lost`: not paid; already claimed: skipped silently, the gold still pays.
- Objective reward: paid on a hold win, not on fled or lost; `once` pays the second win nothing.
- Skip: RUNNER2's skipped fight pays the full purse, every drop roll, key loot and the objective reward.
- Determinism: a shipped playtest recording replays with identical `RoundOutcome`s; the same save, fight and tally give the same gold and drops.
- Save: `reward_paid` round-trips; the v11 -> v12 step gives `{}`, or every roster key for a `once` purse already won.
- Results: the text lines of section 7 for won, fled and lost, with and without defeated foes.
- Validation: each message of section 10 fires for its bad entry; shipped data is clean.
- Parity: journeys and Godot saves pass with exactly the section 9 keys skipped.

## 12. Assets (asset lane, REWARD1-A; replaces PURSE1-A)

Through `aegis content` only (seed `games/emberwake/data/seeds/reward1.json`). Item values from `catalog/items.json`; EV = item value per defeat.

Unit loot tables (`catalog/units.json`); minions 8% in total, the standard 20%, the boss 60%:

| Unit | Rows (item chance) | Total | EV |
| --- | --- | --- | --- |
| `goblin_brawler` | `wpn_knuckles` 3, `arm_leather` 3, `con_heal_potion` 2 | 8 | 2.1 |
| `goblin_skirmisher` | `wpn_stiletto` 3, `arm_leather` 3, `con_heal_potion` 2 | 8 | 2.85 |
| `goblin_archer` | `wpn_shortbow` 3, `arm_leather` 3, `con_heal_potion` 2 | 8 | 3.15 |
| `goblin_shaman` | `wpn_cleric_mace` 3, `arm_robe` 3, `con_focus_elixir` 2 | 8 | 2.6 |
| `goblin_berserker` | `wpn_hand_axe` 3, `arm_leather` 3, `con_heal_potion` 2 | 8 | 2.88 |
| `ambush_berserker` (Horde Warchief) | `wpn_hand_axe` 8, `shd_buckler` 6, `con_heal_potion` 6 | 20 | 6.38 |
| `goblin_warlord` (Grakka) | `wpn_broadsword` 25, `shd_kite` 20, `con_focus_elixir` 15 | 60 | 21.5 |

Every row has `count` 1. `enemy_brawler` and `fleeing_scout` (sim and arena units) and every party-side unit keep `[]`.

Purses (`worlds/ashlands/encounters.json`), the PURSE1 section 4 table without its `loot` column: `courtyard_skirmish` 15-20 once, `pass_ambush` 25-30 once, `wagon_raid` 25-30 once, `ashmaw_stand` 50-60 once, `ashmaw_stand_barred` 50-60 once, `ash_patrol` 12-18 every fight, `ash_raiders` 5-9 every fight, `ash_scouts` 3-6 every fight, `ash_straggler` 2-4 every fight.

Key loot: `ash_patrol` `key_loot` `[{"loot": "warchiefs_axe", "kill": "ash_warchief"}]`. New loot entry in `worlds/ashlands/loot.json` (as PURSE1 section 4): `warchiefs_axe`, name "Warchief's Axe", description "A notched Horde axe, still warm from the warchief's grip.", `reward_kind` item, `ref` `wpn_battle_axe`, count 1, rarity rare, `requires_flags` [], tags []; in no quest pool.

Objective reward: `courtyard_skirmish` `objective_reward` `{"gold": 10, "once": true}` (a hold win can leave both goblins standing, so the hold itself pays). No other encounter gets one in this pass (Q6).

## 13. Balance check (REWARD1-S tooling, REWARD1-B report; replaces PURSE1-B)

REWARD1-S (logic lane, sim): the party and campaign modes print per fight the fates (`def` defeated of roster, `slain`, `surr`, `fled`), the XP per hero by `kill_xp` at the sim level, the purse share at the range mean, the expected drop value and key loot earned; the campaign summary adds per-run totals. This also gives the morale flee and surrender counts the MORALE1 report could not measure (BALANCE_REPORT "Per-run morale counts ... not measured by the sim").

REWARD1-B (balance lane, report only; levers are data):

1. Gold: a full Ashlands run tally as PURSE1-B (start 100, quests 180, chests 90, gifts 35, purse shares from the sim's defeated weights, the courtyard objective 10) against the band 550-650. Lever: the mob purse ranges.
2. Drops: drops per run 3-5, sell value (`sell_mult` 0.5) 40-70, potions and elixirs dropped at most 2 per run (HEAL1-B found potions decisive for the finale). Estimate from data: about 37 minion, 2 warchief and 1 boss defeats per run give about 3.0 + 0.4 + 0.6 = 4 drops and about 67 gold at sell value. Lever: the table chances.
3. Flee and loss pay: per Ashlands encounter, the mean XP and gold of lost and fled fights against won fights; target at most half of a won fight. A lost mob fight pays at most 6 gold (one forced rest's 3 supplies). Lever: none expected; report.
4. XP pace: XP per run under REWARD1 against the win-only rule on the same fights (the HEAL1 chain, `campaign 5 10 heal=rally rest=3 extra=4:ash_raiders`); target within one level at the finale.
5. Farming: a flee loop on a `once` purse never pays more than `gold_max`; gold per round of a flee loop on `ash_raiders` stays below a won fight's.

## Out of scope

Kill gold or any per-unit gold number (S11 Q4 reason holds), shared loot-table kinds (L2) and kit-layer data, world-character loot overrides, drop tables on purses or quests, rarity-weighted random gear, showing rewards or shares before a fight (world preview, intro, Forge preview), objective bonus conditions (no hero downed, under N rounds), loot or gold loss on a defeat, rewards on other worlds' encounters (their units drop by the catalog tables; their encounters get no purse, key loot or objective data in this pass), shop prices or stock, the sim's gold economy beyond REWARD1-S's columns, guests' rewards.

## Open questions

1. Divisor: A (recommended) D1, every roster enemy, so a wave that never came gives nothing and shares never move; B D2, only enemies that took the field (the planner's first suggestion), so a fast win or an early flight pays the full purse for fewer foes.
2. Weight: A (recommended) G1, kill XP at equal level (1 elite L4 + 4 minions L3 puts the elite at 54.5%, 50% at equal level); B G3, tier-only points; C add an optional per-encounter `purse.shares` override for exact splits later. The user said "about 40-50%": is 50-55% close enough?
3. Loot place: A (recommended) inline `loot` on the catalog unit type; B a `loot_tables` kind shared by unit types.
4. Key loot: A (recommended) needs the kill and a win (the decision's "a loss or a flight pays only common rewards"); B the kill alone, even if the party then flees or falls.
5. Missed key loot on the results: A (recommended) one line "Missed: Warchief's Axe (slay Horde Warchief and win)" (loot name, unit name), only while the entry is still claimable; B show nothing (the reward stays a surprise).
6. Objective rewards in the Ashlands: A (recommended) only the courtyard hold, 10 gold once; B also `wagon_raid` (guest Wenn kept alive), 10 gold once; C none until a story pass asks.
7. Drop rates: A (recommended) minions 8%, the warchief 20%, Grakka 60% (about 4 drops a run); B half that (drops stay rare); C double (gear upgrades come mostly from fights).
8. Surrendered and fled foes and loot tables: A (recommended) they roll like killed foes (the decision counts them defeated for common rewards); B only killed foes roll the table, surrender and flight pay XP and gold only.
