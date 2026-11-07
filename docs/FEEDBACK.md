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
| R1.1-3 `R1-BAL` | next | - | UI Option A locked by Design. |
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

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-09 are closed and moved to the archive (2026-11-07). New notes start at UN-10.
