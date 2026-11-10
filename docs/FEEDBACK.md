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
| PR5-3 `PK1-S1` | building: PK1 section 3 hooks 1-7 | `claude/optimistic-newton-ewkyqt` | -
| PR5-17 `FBARCHIVE-FENCE` | done (FB-172): `feedback_check.py` treats headings inside fenced code blocks as text; an unclosed fence is a parse error | `develop` | -

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-171 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-10). New items start at FB-172. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-172: PR5-17 `FBARCHIVE-FENCE` landed (archive and check are fence-aware)

- **Spec**: READY Section 1 row PR5-17; `FB-169` (6) (the `PR5-17` squash commit on `develop`, 2026-11-10)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `tools/planner/feedback_check.py` gains `outside_fences(lines)` (CommonMark fences: 3 or more backticks or tildes; closed by the same character, at least as long, nothing after; any indent counts). `read_items`, `sections`, the archived-id set and `parse_building_rows` skip `## ` / `### ` lines inside a fence, and `archive` rewrites only the item's own Status line (the first outside a fence). New behaviour: a fence in FEEDBACK.md that never closes is a parse error (exit 2) instead of silently swallowing every item after it. Tests: `FenceTest` in `tools/planner/test_feedback_check.py` (4 tests, four fence variants; they fail on the old script). `tools/fc.sh tooltests` ok (144 tools/tiles, 22 planner); the check on the live docs is unchanged. Rust untouched (identical to the gated `develop`).
- **Recommendation**: `tools/planner/README.md`, the `feedback_check.py` row: append "; `archive` and `check` treat `## ` / `### ` lines inside a fenced code block as text, so an item that quotes Markdown moves whole, and a fence in FEEDBACK.md that never closes is a parse error (exit 2) (FB-169)". Confirm the unclosed-fence error (the alternative is the CommonMark reading: the fence runs to the end of the file).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
