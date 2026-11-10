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
| PR5-6 `PK1-S2c` | done (FB-180): Channel toggle, placeholder tint, marrow names, Feed on Rest, refusals in words, AP-hold line; host request `channel`; 0 golden diffs; part 2 (review lows) next | `develop` | -
| PR5-16 `ROLLTABLE1-GUARD` | done (FB-175): the check refuses `guard` inside a `roll_table` on an `ally_hit` row; 0 golden diffs | `develop` | -
| PR5-17 `FBARCHIVE-FENCE` | done (FB-172, answered): `feedback_check.py` treats headings inside fenced code blocks as text; an unclosed fence is a parse error | `develop` | -

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-177 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10). New items start at FB-178. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-178: PR5-5 `PK1-S2b` landed (Channel tells and the AI Channel row)

- **Spec**: READY Section 1 row PR5-5; `docs/sessions/PK1_CORRUPTION.md` 4.3, 4.4 (the `PR5-5` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Tells: `channel::tell_intent` (Channel while the round flag is set, even with nothing planned); `Intent.round_intent` is `Channel` in a Channel round, with `Intent.channel` and `Intent.row_intent` (the round's own row, which the rules read; the Maneuver kind follows it), so the plan strip, unit card, enemy badges, threat intel and trace show Channel through the existing `RoundIntent::Channel` label. Marrow versions (the twin of a Cold Steel skill, not `channel.only` skills such as Feed and Vent) are marked: `Command.marrow`, `Line.marrow`, `channel::marrow_name` ("Cleave (Marrow)" from the new game term `marrow_mark`, renameable by a `terms` row). `rpgkit_host` `intel_json` adds `channel`, `row_intent` and a line's `marrow` only when set. `plan.rs` `estimate_damage` counts a marrow version's armour piercing (`expected_with`). AI: `ai::channel_row` after the table or tactic commits a Cold Steel pick, for units that can Channel only: it tries the same pick as a Channel round through `round_intent_plan::flip_channel` (shared with the toggle) and keeps it when the Cold Steel value is above 0, the round plans at least one marrow version, its value (expected damage plus heal) is at least `CHANNEL_AI_EDGE` (1.2) times the Cold Steel value and the planned Strain stays below the meter max; otherwise the unit, round state and steps are restored. Economy opener-only rounds stay Cold Steel; no RNG. Tests: `rpgkit_battle/tests/channel.rs` now 22 (8 new: edge, below the max, tells, mark term, enemy holder, hurt holder out of reach, rejected trial keeps an intent-bound buff, Economy). Game layer: `slice5_tests.rs` struct literals. Gate 2827 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: change-reviewer PASS, 2 medium fixed (Feed marked as a marrow version; Channel opened just to Feed).
- **Recommendation**: `kits/rpg/README.md`: "PK1-S2b: tells show Channel while the round flag is set (`channel::tell_intent`; `Intent.round_intent` = Channel, `Intent.channel`, `Intent.row_intent`); marrow versions are marked (`Line.marrow`, `channel::marrow_name`, game term `marrow_mark`); the host intel adds `channel`, `row_intent`, `marrow` when set; the AI Channel row `ai::channel_row` (`CHANNEL_AI_EDGE` 1.2: a marrow version planned, value at least the edge times the Cold Steel round, the meter below its max), the shared flip `round_intent_plan::flip_channel`." The README's game-term list (TERMS1, :120): add `intent_field` (missing) and `marrow_mark`. GAME_BRIEF (combat): "AI Strain carriers open Channel when a marrow round is worth at least 1.2 times the Cold Steel round and Strain stays below 100." STRUCTURE (rpgkit_battle row): "PK1-S2b: `intent.rs` Channel tells, `ai.rs` `channel_row`; tests in `tests/channel.rs`." `combat_hooks.md`: the term `marrow_mark`.

### FB-179: PR5-5 `channel_ai_edge` is a kit constant, not a ruleset number (spec 7 vs the row's 0 golden diffs)

- **Spec**: `docs/sessions/PK1_CORRUPTION.md` section 7 (`channel_ai_edge` listed as tunable under `aegis rules --contract`) vs READY row PR5-5 (acceptance: 0 golden diffs)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Every ruleset field is a `tunable` line in the contract, so a ruleset number `channel_ai_edge` moves `rules_contract_golden`, which the row's acceptance forbids. Built as the kit constant `ai::CHANNEL_AI_EDGE = 1.2` (not in `LOCKED`; the doc comment states the deviation). Other readings: "the best Cold Steel option" is the AI table's committed pick and its Channel trial is the same pick as a Channel round (a Field or dual-intent pick never Channels); "below 100" is below the meter's max, counting only what the round's actions pay; AI-drafted heroes holding Strain auto-Channel under autoplay and presets; a Channel round made only of `channel.only` skills (PK2 graft skills) never opens from the AI.
- **Recommendation**: Option A (recommended): a ruleset number `channel_ai_edge` (float, default 1.2, `needs=channel`) folded into `PR5-20` (`HEALPCT1`, a kit row that may name `rules_contract_golden`) or `PR5-7`, with a named contract re-record and a HISTORY row. Option B: keep the constant and drop it from section 7's tunable list. Questions: (Q2) should the AI also try Channel from other picks or intents (default: no, the committed pick only); (Q3) the mark "Name (Marrow)" from the term's short form (default: keep); (Q4) PK2: should graft-only Channel rounds be an AI option (default: revisit with PR5-8).

### FB-180: PR5-6 `PK1-S2c` landed (Channel GUI; host request `channel`)

- **Spec**: READY Section 1 row PR5-6; `docs/sessions/PK1_CORRUPTION.md` 4.3 (the `PR5-6` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Kit (one game-agnostic addition): host request `channel {unit, on}` (`rpgkit_host` `channel_req` over `Session::set_channel`; mutating, recorded for replay, refused in a locked round, for a unit that cannot Channel, an enemy, outside planning or without `on`; an error records nothing). Game (`emberwake_gui`): a "Channel" button in the skill menu's neutral row (after Rest, before Select) for units that can Channel only, always live (0 AP, 0 blocks), lit while on; a placeholder violet `CHANNEL_TINT` on the unit token, the strip blocks and the timeline segments of a Channel round, plus a hero-panel line "Channel round: marrow versions only"; marrow names ("Shield Bash II (Marrow)") on strip entries, hero-panel lines and timeline segments (`input::skill_text`, only when the version is offered); `tokens::plan_pose` reads `channel::tell_intent`; the AP hold "AP held at N this round (source)" in the hero panel (`tips::ap_held_text`); Cold Steel rows greyed with the kit's reasons; Feed on the Rest button and its card in a Channel round (`MenuGrid::rest_name`); menu refusals in words (`intent_text::menu_refusal_text`: "Not enough HP: 9, it costs 10", "Needs Strain 50 or more to start: 30"). Without a holder nothing changes (menu widths and places, agent-link JSON: `channel_units` only under the rule, `strip_labels[].channel` only when true). One wording change on existing screens: greyed pick-menu rows with an HP cost now read "Not enough HP: 9, it costs 10" (was "HP 9 <= cost 10"; no fixture held it). Tests: `battle/tests/channel_tests.rs` (7) and `rpgkit_host/tests/pick_requests.rs` (+1). Gate 2835 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: change-reviewer PASS (lows to a part 2: past Feed or Vent timeline segments tint by the current round's mode; the closed-menu hotkey path prints the raw reason; three tests can pass without their precondition; no replay-equality or locked-round test for `channel`).
- **Recommendation**: `kits/rpg/README.md` (PK1-S2a line): "host request `channel {unit, on}` (recorded; refused in a locked round) over `Session::set_channel`". `engine/docs/PORT_NOTES.md` (host request list, :284) and `games/emberwake/docs/SCREENS.md` (battle-screen requests, :205): add `channel {unit, on}`. GAME_BRIEF (after the Channel line): "Channel GUI (PK1-S2c): the toggle is a 0 AP neutral button for Strain carriers; a placeholder violet tint marks a Channel round's unit and its strip and timeline blocks; marrow versions show as 'Name (Marrow)'; Rest's button reads Feed; refusals in words; the hero panel names an AP hold." Questions (built defaults): (a) Vent shows the one reason the kit returns (HP is checked first) rather than both (spec 4.3 says "both reasons"); (b) "Cold Steel" is placeholder text with no `terms` entry; should it become a game term; (c) the toggle ends intent-bound buffs at once (4.3 item 1), not deferred to Confirm as IN6 does for ghost intent changes.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
