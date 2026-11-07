# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 release | released: `main` @98ab973a (PR #55, merge commit; tree = develop @f7f5cb71) | `main` | Next: Phase R1.1. |
| R1.1-1 `AUDIT-R1b` (AUDIT-08..12) | next | - | Proposed lane plan presented to the user. |
| R1.1-2 `C02-A`, R1.1-3 `R1-BAL` | queued (after AUDIT-R1b) | - | - |
| READY rows 14-20 (R2-W1, R3-W1) | queued (after R1.1) | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-92 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07). New items start at FB-93. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-06 are closed and moved to the archive (2026-11-07). New notes start at UN-07.

### UN-07: Phase R1.1 dispatch plan and check-in cadence

- **For**: user
- **Status**: open
- **Note**: Build proposes running R1.1 as three steps. (1) `AUDIT-R1b`: three `logic-builder` lanes in parallel worktrees on disjoint files (A: AUDIT-08 + 09 in `rpgkit_rules` `unit.rs` / `gear.rs`; B: AUDIT-11 + 12 in `rpgkit_battle` `skill.rs` / `command.rs` / `dojo.rs`; C: AUDIT-10 in `rpgkit_content` `check/skill.rs`, the only lane that re-records `checks_golden`), merged into one branch with one review, one full gate and one squash commit. (2) `C02-A`: `asset-builder` seeds family Level 2 / Level 3 values, `shd_tower` text and character `temperament` / `default_intent` through `aegis content`, while a `logic-builder` wires sim `character_overrides`; five named re-records in one commit. (3) `R1-BAL`: a `logic-builder` does the feel metric, break-forecast / `BREAK` log and Dojo badge work while the `balance` agent runs the sims. Risks: AUDIT-08 / 09 could move a battle or sim golden if a fixture unit carries a 2H + 1H pair, and AUDIT-11 must tell an authored `mitigation` from the loader's 0.5 default. Either case stops that lane with a limit-blocker FB item, as the Authority Matrix requires.
- **Ask**: (a) Approve the plan as written? (b) Cadence: run R1.1 straight through and report in this file at the end (default), or stop after each step for a check-in?

### UN-08: Run R2 Wave 1 rows 14-19 in parallel instead of strictly in order

- **For**: both
- **Status**: open
- **Note**: Rows 14-19 all expect 0 battle / sim golden diffs (some re-record only `rules_contract_golden` or `checks_golden`, as each row names), and most touch separate files: 14 `pipeline` damage / heal, 15 `item_ap` + token renderer, 16 IR21 + shapes, 17 `lock_tile` / debt numbers, 18 terrain + schema, 19 `ai.rs`. Running them two or three at a time (each on its own branch, landed one by one with a gate after merging the latest `develop`) would shorten R2 Wave 1 a lot. Rows 15, 17 and 18 each re-record `rules_contract_golden`, so they would land one after another with a re-record on each rebase.
- **Ask**: Allow parallel dispatch of rows 14-19 after R1.1 lands (default if approved: batches {14, 16, 19} then {15, 17, 18}), or keep the listed order?

### UN-09: R1-BAL tuning numbers need your approval before they land

- **For**: user
- **Status**: open
- **Note**: The `R1-BAL` sims (`tl_duel_windup` at a 20 % win rate, warrior / ranger crit shift, `know_potency` weight, `tl_skirmish` seed 7) will produce proposed number changes. These are gameplay decisions, so Build will log them as one FB item with options and sim results, and will not apply any tuning until the answer comes back through `docs/READY.md` section 4. The fixes that change no number (enemy AP metric, forecast and log text, Dojo badge) land without waiting.
- **Ask**: Confirm this split. Is there a target win-rate band for the timeline duels (for example 40-70 %), so the balance lane can propose against it?
