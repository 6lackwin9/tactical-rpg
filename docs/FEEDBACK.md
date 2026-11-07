# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 release | released: `main` @98ab973a (PR #55, merge commit; tree = develop @f7f5cb71) | `main` | Next: Phase R1.1. |
| R1.1-1 `AUDIT-R1b` (AUDIT-08..12) | done (FB-95) | `develop` | 3 parallel lanes + FB-94 additions; checks_golden re-recorded. |
| R1.1-2 `C02-A` | done (FB-96) | `develop` | asset + logic lanes in parallel; phasefold re-recorded. |
| R1.1-3 `R1-BAL` | (2a, 2c) done (FB-99); (1, 2b) parked FB-97; tuning FB-98 awaiting approval | `develop` | - |
| READY rows 14-20 (R2-W1, R3-W1) | queued (after R1.1) | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-95 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07). New items start at FB-96. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-96: C02-A built: family Level 2/3, 11 mastery passives, 13 hero temperaments; sim applies character AI keys; phasefold re-recorded

- **Spec**: `docs/READY.md` R1.1-2 `C02-A`; FB-81, FB-83, FB-93 table
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Seed `games/emberwake/data/seeds/c02a.json` (46 steps, idempotent; applying it to the old data gives exactly the new data): `shd_tower` reads "A wall of iron. Superb protection, but heavy."; `level2_bonus` and `level3_passive` on the 21 trained families as tabled (accessory, consumable, material, quest untouched); 11 `mastery_*` passives (slot ""); `temperament` and `default_intent` on the 13 party heroes only. Sim (FB-83): `rpgkit_sim` `world::has_extras` also counts a character's `ai_role` / `default_intent`, and `Runner::write` lays the character AI keys over its copy of the base unit (as `Data::character_unit_entry` does in the game), so sim units from world encounters play with their temperament, role and starting intent; new `Runner::spawn_row`. Most of the wiring is in `runner.rs`, not `world.rs` as the row said. Re-recorded (named): `phasefold_golden`, 60 of 72 fights; a probe with only the old `item_families.json` matches the old golden exactly (first divergence from `level3_passive`: `goblin_ambush` balanced seed 1 round 1 step 8, Ranger 173 -> 172; `level2_bonus` drives later ones; temperaments and intents move nothing there). `rules_golden`, `timeline_golden`, `sim_golden` and `checks_golden` are named but did not move: they read frozen fixtures (`timeline_kit.json`, `sim_data.json`, the checks fixtures) that carry none of the seeded fields. `emberwake_world/tests/unit_builder.rs`: its old-build oracle now re-adds the temperament the save leaves out (U4.2-S3); Bryn Hale was the first shipped hero with one; no product code changed. Tests: `rpgkit_sim/tests/world_characters.rs` (2). Full gate passed (2423 tests).
- **Recommendation**: HISTORY: "C02-A: family Level 2 crit/burden and Level 3 mastery passives (11 rows), 13 hero temperaments and default intents seeded; the sim applies a character's temperament, `ai_role` and `default_intent`; re-recorded on purpose: `phasefold_golden` (family Level 2/3 values)". GAME_BRIEF proficiency: the Level 2 / Level 3 table is live (crit +2 / +1 or burden -1 at proficiency 2, the family's mastery passive at 3); GAME_BRIEF passives: 13 party heroes carry a temperament and starting intent. STRUCTURE: test `rpgkit_sim/tests/world_characters.rs`; `Runner::spawn_row`. Question for Design (suggestion, not blocking): the frozen fixtures behind `timeline_golden` and `sim_golden` do not see C02-A (or any later shipped-data seed), so those goldens cannot catch balance moves from data. Re-freeze them from shipped data in a named slice (likely R1-BAL, which already names both), or keep them frozen by design?

### FB-97: R1-BAL parts (1) and (2b) each move a golden the row says stays unchanged

- **Spec**: `docs/READY.md` R1.1-3 `R1-BAL` parts (1) and (2) ("Telemetry/UI fixes land with `0` golden diffs"); FB-92, FB-88
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: Parts (2a) (Dojo wield-penalty badge, reusing `party::wield_badge`) and (2c) (break forecast adds "queues Stagger next round (locks B1, B4)" under pips, not on a `steadfast` target, blocks read from the `stagger` row; new `poise::break_would_queue_stagger` / `break_stagger_note`) are built on `claude/r1-bal` with 0 golden diffs (the forecast lines live in `emberwake_gui/src/battle/marks.rs` `HitForecast::lines`, not `tips.rs`). Two clauses are parked. (1) The enemy "0 AP/round" is not a metric gap: `Feel::round_end` counts both sides the same way, but the timeline's `enemy_moves` rule makes enemies pay no AP (`World::pays_ap` false, `rpgkit_battle/src/world.rs:448`), so 0 is the true value. `sim_golden` records it (case `feel 5 1 enc=tl_duel_windup,tl_skirmish party=both`, 5 lines), so any change to the number or its wording moves `sim_golden`. (2b) Appending `(Stagger queued)` to the `[Break]` log line (tried, then fully reverted) moves `timeline_golden`: 8 of 15 fights, first `tl_duel_windup` seed 7 `log[7][12]`, "[Break] Vanguard (test)'s Heavy Cleave is broken" -> "... is broken (Stagger queued)". The portrait break matcher still matches the longer line.
- **Recommendation**: (1): (A, recommended) close FB-92's metric item as "by rule" and print "no AP (enemy_moves)" for a side that pays no AP, re-recording `sim_golden` (text only); (B) report the AP the enemy's started commands would cost; (C) add an enemy "recovery blocks per round" metric instead. B and C also re-record `sim_golden`. (2b): (A, recommended) approve a text-only `timeline_golden` re-record so the log suffix lands with R1-BAL; (B) drop the log suffix and keep only the forecast line. Build lands (2a) and (2c) with the R1-BAL balance options, and adds whichever of these Design approves.

### FB-98: R1-BAL balance options for the timeline fights (50-75 % duels, >= 75 % overall): H6 recommended

- **Spec**: `docs/READY.md` R1.1-3 `R1-BAL` part (3); FB-75, FB-76, FB-77, FB-89, FB-92; UN-09 band
- **Kind**: suggestion
- **Status**: open
- **Finding**: Sims on develop @af543321 (`tools/fc.sh sim feel 5 ruleset=timeline seeds=N`, auto driver). Baseline at 10 seeds equals FB-92 exactly (AUDIT-R1b and C02-A do not move the five `tl_` fights); at 40 seeds: windup 10 %, reach 57.5 %, armour 87.5 %, flank 100 %, skirmish 80 %, overall 67 %. Windup is a damage and HP gap against the armour-35 elite `tl_enforcer` (it out-trades the Vanguard and breaks Heavy Cleave in round 3): only its armour or tier moves windup. `know_potency` 0.5 / 1.0 / 2.0 is inert in all five fights (they use almost no resistable statuses). The stagger and break levers (`stagger_windup_mult`, `stagger_repeat_mult`, `break_chance_*`, `poise_pips`, `max_pip_loss_per_hit`) move reach and skirmish, never windup; `poise_pips` 3 / 8 and `max_pip_loss_per_hit` 2 swing reach and armour too far. `tl_duel_armour` is won by armour-piercing dagger hits, so only the Captain's level moves it. Only windup, reach and armour are duels; flank (2v2) stays near 100 %, and the overall target needs flank and skirmish high. Options at 40 seeds (win %, windup / reach / armour / flank / skirmish / overall): baseline 10 / 57.5 / 87.5 / 100 / 80 / 67; C (`tl_enforcer` armour 20, `tl_captain` level 2, `tl_footman` armour 35, `stagger_windup_mult` 1.25) 50 / 77.5 / 60 / 100 / 75 / 72.5; H5 (data only: `tl_enforcer` armour 15, `tl_captain` level 2, `tl_footman` armour 35, `tl_skirmish` max_rounds 12) 52.5 / 47.5 / 60 / 97.5 / 87.5 / 69; H6 (`tl_enforcer` armour 15, `tl_captain` level 2, `tl_footman` armour 40, `tl_skirmish` max_rounds 12, `stagger_windup_mult` 1.0) 65 / 75 / 65 / 100 / 92.5 / 79.5. Crit shift (FB-75, `sim duel <L> 10`): warrior 81 / 88 % at L5 / L7 (out of the 40-60 bracket before and after), ranger 74 / 56 (L7 down from 66 before PERC -> Crit), rogue 43 / 37, brawler 58 / 54; priest 0-22 % at L3-L7 (pre-existing). `tl_skirmish` seed 7 (FB-77, measured on 98ab973a): ends "fled (round limit)" at round 10 with the party AP-starved (0 or -1) and the enemy Enforcer and Crossbowman alive, not lost on damage; `max_rounds` 12 turns such fights into wins. Hover-card text fit (FB-77): not checked (UI).
- **Recommendation**: (A, recommended) H6: `units.json` `tl_enforcer` armour 35 -> 15, `tl_captain` level 1 -> 2, `tl_footman` armour 25 -> 40; `encounters.json` `tl_skirmish` max_rounds 10 -> 12; `rulesets.json` `timeline` `stagger_windup_mult` 2.0 -> 1.0 (the only ruleset number; it halves the windup stagger bonus, a rule-level change). Reach sits on the 75 % ceiling; `tl_footman` armour is the trim knob. (B) H5, data only (no ruleset number moves): duels in band but overall 69 %. (C) Keep `know_potency` at 0.0 and re-test it on status-heavy fights later. Goldens H6 likely moves (named in the R1-BAL row): `sim_golden`, `phasefold_golden` (both read shipped data), `rules_contract_golden` and `rules_golden` through `stagger_windup_mult`; `timeline_golden` reads the frozen `timeline_kit.json`, so it moves only if the ruleset number reaches that fixture. Warrior's out-of-bracket duel rate and priest's 0-22 % are pre-existing and left for a later class pass. BALANCE_REPORT lines: "2026-11-07 timeline feel on af543321 (40 seeds): 67 % overall (windup 10, reach 57.5, armour 87.5, flank 100, skirmish 80); windup is a damage/HP gap against the armour-35 `tl_enforcer`, `know_potency` inert; recommended H6 gives 65 / 75 / 65 / 100 / 92.5, overall 79.5."

### FB-99: R1-BAL parts (2a) and (2c) landed: Dojo wield-penalty badge, break forecast names the queued stagger

- **Spec**: `docs/READY.md` R1.1-3 `R1-BAL` part (2), UI Option A; FB-80, FB-88
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (2a) Dojo gear slots under a wield penalty (`untrained` or stat deficit) show the party screen's red badge (`party::wield_badge`, now `pub(crate)`, e.g. `x0.85`), right-aligned in `th().foe`, and the slot card gets the "Wield penalty: ..." sentence (screenshot sent to the user). (2c) Under pips, a Breaks / May break forecast on a non-`steadfast` target adds "queues Stagger next round (locks B1, B4)" in `th().foe`, blocks read from the `stagger` row (`poise::break_would_queue_stagger`, `break_stagger_note`; `break_queues_stagger` uses the first, behaviour unchanged). The forecast lines live in `emberwake_gui/src/battle/marks.rs` `HitForecast::lines`, not `tips.rs`. 0 golden diffs; parts (1) and (2b) are parked in FB-97 and the tuning in FB-98. Tests: `poise_pips.rs` (+1), `slice5_tests.rs` (+1), a `dojo.rs` in-file GUI test.
- **Recommendation**: GAME_BRIEF poise: "Under pips the hit forecast notes that a break queues Stagger next round (locks B1, B4)"; GAME_BRIEF Dojo: "Dojo gear slots show the party screen's wield-penalty badge". STRUCTURE: `poise::break_stagger_note`. Two small UI calls for Design: under a "May break" verdict the note reads as certain (an "if it breaks" wording would be clearer), and a long item name in a narrow Dojo gear panel could overlap the badge (no text-overlap test covers it).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-09 are closed and moved to the archive (2026-11-07). New notes start at UN-10.

### UN-10: start R2 Wave 1 while R1-BAL waits on tuning approval?

- **For**: user
- **Status**: open
- **Note**: Phase R1.1 is built except what waits on your answers: FB-97 (two R1-BAL clauses that each move a golden) and FB-98 (the timeline tuning numbers, H6 recommended). READY runs R1.1 before R2, so Build is idle until those come back. R2 Wave 1 batch 1 (rows 14, 16, 19: damage/heal pipeline port, IR21 derive + RING/ARC shapes, AI status predicates) is independent of the R1-BAL numbers and expects 0 battle/sim golden diffs.
- **Ask**: Start R2 Wave 1 batch 1 now and fold R1-BAL's remaining clauses and tuning in as their own commits when answered (default), or hold R2 until R1-BAL is fully closed?
