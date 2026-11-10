# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 to R2 rows (R1 release, R1.1, R2 Waves 1-2, R2 release) | released (`main` @98ab973a, @3cc73f2b) | `main` | Rows moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10, size). |
| W2-6 (R3-W2) | done (FB-142); FB-140 A in the landing, develop's gate green again | `develop` | R3 Wave 2 complete. Next: R3-CLOSE. |
| R3-CLOSE | released: `main` @8d863f58 ([#57](https://github.com/6lackwin9/tactical-rpg/pull/57), merge commit; tree = develop @e6450e22); audit FB-143, no blocker | `main` | - |
| R4-1 `U2.3-TIERS` | done: slice 1 (`aa311bf4`), slice 2 (`858eaab0`) | `develop` | - |
| R4-2 `R4-CLOSE` | done (`3353e62b`) | `develop` | - |
| R4 release | released: `main` @32ae1ada ([#58](https://github.com/6lackwin9/tactical-rpg/pull/58), merge commit; tree = develop @ad903f92, which adds a whitespace fix for an MD012 lint error from 92a7282b) | `main` | Ready queue empty; next: Section 3 `R3-STAGE-SEED`, `POST-R4-CLEANUP` when Design queues them. |
| WEBREL1 feasibility | done: FB-147 (read-only audit, no code) | - | Waiting on Design: E1 split (Option A) and slice order. |
| PR4-1 `R3-STAGE-SIM` | done: part 1 (`stage=N`, `55ef14d8`), part 2 (AUDIT-07, `sim_golden` run 11 re-recorded, FB-154) | `develop` | - |
| PR4-2 `POST-R4-CLEANUP` | done: phase A (`feadda4d`), phase B re-freeze (`c49194b2`, FB-152; FB-153 suggestion); item 2 stays held | `develop` | - |
| PR4-3 `WEBREL1-E1a` | done (FB-155): wasm32 check clean, `aegis_core::clock`, clock lint | `develop` | - |
| PR4-4 `WEBREL1-E1b` | done (FB-156; FB-157 suggestion): `aegis_core::vfs`, `aegis_save::store`, wasm `cfg`, fs lint | `develop` | - |
| PR4-8 `SKILLCHECK-FOLD` | done (FB-158 suggestion): fold error when nothing else is reported; ranks normalised before the fold | `develop` | - |
| PR4-5 `WEBREL1-E2` | done (FB-159; FB-160 suggestion): `shell::start_web`, async WebGL2 renderer, DPR, audio on first gesture; `web_min` passes in headless Chromium | `develop` | - |
| PR4-6 `WEBREL1-G1` | done (FB-161; FB-162 suggestion): the game runs in a browser (web entry, bundle, `dev_tools`, `fc.sh web`, smoke 40/40, WEBREL1 6.2) | `develop` | - |
| PR4-7 `WEBREL1-P1` | done (FB-163; FB-164 suggestion): advisory `web` CI job, `web.yml` itch.io deploy; GitHub-side acceptance observable at the next release | `develop` | - |
| PR4-9 `WEBOPT1` | done (FB-166): pinned Binaryen version_125 on CI, `wasm-opt` version guard in `fc.sh web`; first real `wasm-opt` run at the next release PR | `develop` | Post-R4 queue PR4-1..PR4-9 complete (@381b8746). Watch at the next release PR: the `web` job (`web-build`, pinned `wasm-opt` sizes, smoke on the optimised build). |
| PR5-1 `ROLLTABLE1` | done (FB-168; FB-169 suggestion): trigger effect `roll_table` (weighted rows on the seeded battle RNG), content check, schema help; 0 golden diffs | `develop` | Post-R4 queue: PR5-1, PR5-2, ACTS23-R5 feasibility (FB-167)
| PR5-2 `PHASEFX1` | done (FB-170, FB-171): encounter phase effects `side` (hostile_all, party, enemy) and `swap` (keep_pct, full), content checks; 0 golden diffs | `develop` | PR5-1, PR5-2 done; FB-167 answered
| PR5-3 `PK1-S1` | done (FB-173; FB-174 questions): PK1 section 3 hooks (`heal_taken`, recoil, `strike_nearest`, `ap set`, `mark`, hazard `meter`, `unmark`); 0 golden diffs | `develop` | -
| PR5-16 `ROLLTABLE1-GUARD` | done (FB-175): the check refuses `guard` inside a `roll_table` on an `ally_hit` row; 0 golden diffs | `develop` | -
| PR5-17 `FBARCHIVE-FENCE` | done (FB-172, answered): `feedback_check.py` treats headings inside fenced code blocks as text; an unclosed fence is a parse error | `develop` | -

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-172 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10). New items start at FB-173. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-173: PR5-3 `PK1-S1` landed (PK1 section 3 hooks 1-7)

- **Spec**: READY Section 1 row PR5-3; `docs/sessions/PK1_CORRUPTION.md` section 3 as amended by `FB-167` (the `PR5-3` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) band-only `heal_taken` (`BAND_EFFECTS`, `Grant::band_heal_taken`, `Unit.band_heal_taken`): every in-battle heal (item, support, regen) passes `World::heal`, `amount * (100 + pct) / 100` rounded down, at least 0, then the missing-HP cap; unchanged without the band. (2) `damage` `pct_of_hit`: a share of the event's hit (`Event.hit`, set on `hit_taken`, `guarded_hit`, `hit_dealt`, `kill` and riders; `trigger::part`, at least 1 when the hit dealt damage), never below 1 HP, not mitigated again; the check refuses `value` with it, `to: other`, and a rider without `to: self`. (3) `strike_nearest` {power, then_status, then_rounds}: the nearest other unit of any side (`World::dist`, ties to the lowest id), resolved like riposte (`roll_hit`, one crit roll, cover), no AP and no hit events, `[Spasm]` line; the status lands next round. (4) `ap` {set, rounds}: `trigger::apply_ap_sets` at round start (after `round_start` triggers, before planning) holds AP at most `set`; a unit that pays no AP gets a 1-round stun (`debt_stun_status`). (5) `mark` {mark, value, to}: `mark.rs` `Marks`, `HeroEnd.marks`, the settle's `marks` key (only when non-empty), the Results screen line "X is marked: calcify +1." (`emberwake_gui` `hub.rs`). (6) hazard `meter` {meter, delta}: arena effects (`strike_unit`) and ground hazard rows (`HazardDef.meter`, applied at `ground::settle`), holders only (`meter::from_hazard`). (7) consumable `unmark` {mark}: removes the target's latest mark of that id; on a `hostile_all` rogue that started on the party side it calms it at the next round start (`World::rejoin_party` at its start place; a start guest is a guest again, protected again if it was; joining the party side drops `unkillable`). Schema: `hazards.meter`, `items.unmark` (json, `omit_default`; empty writes no key). Tests: `rpgkit_battle/tests/pk1_hooks.rs` (13), `rpgkit_content/tests/pk1_hook_checks.rs` (9), plus roster and Results tests; two kit test expectations updated mechanically (`hooks_schema.rs`). Gate 2798 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: change-reviewer PASS, 1 medium (calmed guest) and 3 low fixed; 3 low in FB-174.
- **Recommendation**: `kits/rpg/README.md` (after PHASEFX1): "PK1-S1 (PK1_CORRUPTION 3, FB-167): band-only `heal_taken` (a percent on every heal after its formula, `World::heal`); trigger `damage` `pct_of_hit` (recoil on the holder, `to: self`, never below 1 HP); `strike_nearest` {power, then_status, then_rounds} (nearest other unit of any side, ties to the lowest id, `[Spasm]` line); `ap` {set, rounds} (AP held at most `set` at the next round starts; a unit that pays no AP gets a 1-round stun); `mark` {mark, value, to} (`World::marks`, `HeroEnd::marks`, the settle's `marks`); consumable `unmark` {mark} (removes the latest mark; a turned ally rejoins the party side at the next round start in its start role); hazard `meter` {meter, delta} on arena effects and ground hazard rows (holders only)." `engine/docs/authoring/combat_hooks.md`: trigger effects (:124) add `strike_nearest`, `mark`, `ap` `set`/`rounds`, `damage` `pct_of_hit`; hazards table (:67) a row "`hazards` row: `meter` {meter, delta}, a holder's meter change"; arena `effect` (:95) add `meter`; unit_meters bands add `heal_taken` (bands only); meter-rule effects add the new trigger effects. GAME_BRIEF: "Trigger effects also `strike_nearest`, `mark`, `ap set`, `damage pct_of_hit`; band effect `heal_taken`; hazard `meter`; consumable `unmark` (PK1-S1; no shipped data uses them yet)." STRUCTURE (rpgkit_battle row): "PK1-S1: `mark.rs` (battle marks, unmark, calm); `trigger.rs` `strike_nearest`, `apply_ap_sets`; `world.rs` `heal_taken`, `rejoin_party`; `meter.rs` `from_hazard`; `Session` `start_guests` / `start_protected`; tests `tests/pk1_hooks.rs`." No HISTORY row.

### FB-174: PR5-3 readings, limits and questions

- **Spec**: READY Section 1 row PR5-3 (@ the `PR5-3` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Ranges chosen (the spec gives none): `ap` `set` 0-4 and `rounds` 1-5; `strike_nearest` `power` 0.1-3.0 and `then_rounds` 1-5; `mark` id up to 24 characters and `value` 1-99; hazard meter `delta` -100..100, not 0; `heal_taken` -100..100, not 0. Limits: (L2) spec 3.4 "the planner shows why": the kit exposes `State::ap_held(c)`, but no planner or GUI text reads it yet; (L5) `strike_nearest` has no reach limit (a melee unit may strike a unit five tiles away), as the spec reads literally; (L6) `unmark` calms any `hostile_all` rogue that started on the party side and lost a mark, not only one a breach turned (fine for PK1; PK2's breach decides). The AI and planner heal estimates do not apply `heal_taken`; the Dojo heal bypasses it (a practice screen).
- **Recommendation**: Questions, each with the built default: (1) bands cannot hold triggers, so "the Aberrant Edge band holds a hit_dealt trigger" is a meter-row `triggers` entry with `when.meter {id: strain, at_least: 75}` (default: keep; alternative: band `triggers`); (2) recoil is at least 1 when the hit dealt damage (default: keep); (3) `strike_nearest` raises no hit events (no riders, recoil or `hit_taken` on it) (default: keep); (4) an `unmark` with nothing to undo is spent with a log line "no calcify to undo" (default: keep; alternative: refused at planning); (5) for PK2: a hero still hostile at the battle end breaks `hero_ends`' member pairing, so the Stage V turn must settle it; (6) the priest keystone's skill version of `unmark` (`uses_per_battle`) is not in this slice (PK2). Follow-up rows: wire `ap_held` into the planner's reason text with PR5-6 (game GUI), and decide `strike_nearest` reach (recommended: keep any distance, it is a spasm).

### FB-175: PR5-16 `ROLLTABLE1-GUARD` landed

- **Spec**: READY Section 1 row PR5-16; `FB-169` (5) Option A (the `PR5-16` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `rpgkit_content/src/check/hooks.rs` `effects_errors`: a `roll_table` on an `ally_hit` row whose rows hold a `guard` is one error, "a roll_table on ally_hit cannot guard (it rolls after the hit); put guard at the top level." (`table_guards`). The same table on another event and a top-level `guard` on `ally_hit` check clean. Test `a_table_on_ally_hit_cannot_guard` (`rpgkit_content/tests/roll_table_checks.rs`). Gate 2799 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Built by the planner (small standalone logic).
- **Recommendation**: `kits/rpg/README.md` ROLLTABLE1 line: append "; on `ally_hit` a table cannot hold a `guard` (only a top-level guard shapes the hit)". `engine/docs/authoring/combat_hooks.md` (`roll_table` entry): the same clause.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
