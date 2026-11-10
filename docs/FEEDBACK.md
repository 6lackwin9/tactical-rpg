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
| PR5-4 `PK1-S2a` | done (FB-176; FB-177 questions): rule `channel`, marrow versions, the converting toggle, Channel order and Feed, deck `marrow`, `requires_meter`, `hp_cost_pct`; `rules_contract_golden` re-recorded | `develop` | next: PR5-5, PR5-6, then PR5-7
| PR5-5 `PK1-S2b` | done (FB-178; FB-179 question): Channel tells, marrow mark, AI Channel row (`CHANNEL_AI_EDGE` 1.2 as a kit constant); 0 golden diffs | `develop` | next: PR5-6, PR5-20, PR5-7
| PR5-6 `PK1-S2c` | done (FB-180): Channel toggle, placeholder tint, marrow names, Feed on Rest, refusals in words, AP-hold line; host request `channel`; 0 golden diffs; follow-ups are PR5-21 | `develop` | -
| PR5-20 `HEALPCT1` | part 1 done (FB-181): SUPPORT skill field `heal_pct` through `World::heal`, 0 golden diffs; part 2 (`channel_ai_edge` ruleset number, FB-179 A, added to the row) next | `develop` | -
| PR5-21 `PK1-S2c-2` | done (FB-182): past-round tint, hotkey refusals in words, precondition asserts, `channel` replay and locked-round tests, Vent both reasons, term `cold_steel_mark`; 0 golden diffs | `develop` | next: PR5-20 part 2, PR5-7
| PR5-16 `ROLLTABLE1-GUARD` | done (FB-175): the check refuses `guard` inside a `roll_table` on an `ally_hit` row; 0 golden diffs | `develop` | -
| PR5-17 `FBARCHIVE-FENCE` | done (FB-172, answered): `feedback_check.py` treats headings inside fenced code blocks as text; an unclosed fence is a parse error | `develop` | -

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-180 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10). New items start at FB-181. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-181: PR5-20 `HEALPCT1` landed (skill field `heal_pct`)

- **Spec**: READY Section 1 row PR5-20; `FB-177` Option A (the `PR5-20` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Part 1 of the row (the row gained `channel_ai_edge`, FB-179 A, after this part was built; part 2 follows). SUPPORT skill field `heal_pct` (0-100; a percent of the target's max HP, `max(1, floor(max HP * pct / 100))`, not per rank, no rule needed): added to the heal formula after the caster's potency (`command::heal_pct_of`, `command::heal_amount_on`) and applied through `World::heal`, so `heal_taken` applies. Read by the command (`support_apply`), the planner's `est_heal` (so `ai::round_value` sees Feed), and the overheal metric. A `heal_pct`-only skill is a heal skill (`skill::heals`, the passive group `heal`, the support check, whose message is unchanged). Content: range 0-100 ("Heal % must be 0-100"), SUPPORT only; the `{heal}` token adds "N% of max HP" when set; schema row after `heal_mult` (`rank: false`); the writer stores nothing at 0. Tests: `rpgkit_battle/tests/heal_pct.rs` (6), `rpgkit_content/tests/heal_pct_checks.rs` (5). Gate 2846 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: change-reviewer PASS.
- **Recommendation**: `kits/rpg/README.md` (after PK1-S2a): "HEALPCT1 (PR5-20, FB-177 A): SUPPORT skill field `heal_pct` (0-100, a percent of the target's max HP, rounded down, at least 1 when set, not per rank), added to the heal formula after the caster's potency and run through `World::heal` (`heal_taken` applies); the command, `est_heal` / `ai::round_value` and the overheal metric read it; a `heal_pct`-only skill is a heal skill; the `{heal}` token adds 'N% of max HP'." `combat_hooks.md` (skill fields, beside `hp_cost_pct`): "`heal_pct` (SUPPORT; a percent of the target's max HP added after the heal formula; `heal_taken` applies)". GAME_BRIEF: "HEALPCT1 (kit): skill field `heal_pct` (Feed's 4% self-heal; PR5-7 authors it)." Questions (built defaults): (a) the caster's potency does not scale the percent part (Feed stays 4%); (b) `est_heal` never applied `heal_taken` (formula heals too), so at high Strain the planner and AI overstate Feed; a follow-up could apply it in the estimate (golden-safe until a shipped band holds `heal_taken`, but it changes AI values then); (c) not per rank (as `hp_cost_pct`).

### FB-182: PR5-21 `PK1-S2c-2` landed (Channel GUI follow-ups)

- **Spec**: READY Section 1 row PR5-21; `FB-180` (the `PR5-21` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) `tips.rs` `channel_action` tints an action only when it ran as a marrow version or is a marrow-only skill, so past rounds keep their own mode (`a_past_rounds_blocks_keep_that_rounds_mode`). (2) The closed-menu hotkey path shows the marked name and refusals in words. (3) The three tests that could pass without their precondition now assert it (`pick_live_skill`, `expect`). (4) Host tests `channel_flips_the_round_and_replays` (on then off, the replay reaches the same `round_plan` and `plan_view`) and `a_locked_round_refuses_channel`. (5) Vent shows both reasons: `round_intent_plan.rs` `pick_steps` joins the HP and gate refusals with `REASONS_JOIN` ("; ", HP first) when both fail (single-reason text unchanged; the auto-fill and `plan.add_step` unchanged); the GUI words each part ("Not enough HP: 9, it costs 10; Needs Strain 50 or more to start: 30"). (6) Game term `cold_steel_mark` (default "Cold Steel"; terms `REGISTER` 10 entries; renameable, tested), read by the toggle's detail, card and message (`input::cold_steel_name`). Tests: kit `vents_cell_shows_both_reasons_when_both_apply`; host 2; GUI 2 new plus 3 tightened. Gate 2851 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: change-reviewer PASS (lows: the closed-menu hotkey path has no GUI test; `menu_refusal_text` splits any reason containing "; ", harmless since non-HP and non-gate parts come back unchanged).
- **Recommendation**: `kits/rpg/README.md`: "PK1-S2c-2 (PR5-21): a pick refused for both its HP cost and its meter gate gives both reasons (`round_intent_plan::REASONS_JOIN`, HP first); term `cold_steel_mark` (the name of a non-Channel round, renameable like `marrow_mark`)"; the README's game-term list (TERMS1) adds `cold_steel_mark`. GAME_BRIEF (Channel GUI line): "Vent shows both reasons when both apply; 'Cold Steel' is the game term `cold_steel_mark`; past rounds' timeline blocks tint by the round they were played in." No HISTORY row; STRUCTURE unchanged. known-issues: two rows added this landing (one-line `schema.json` merge conflicts; a linked-worktree gate sharing the target folder).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
