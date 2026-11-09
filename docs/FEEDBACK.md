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
| R1.1-3 `R1-BAL` | done (FB-99, FB-100, FB-103) | `develop` | Phase R1.1 complete. |
| READY row 14 `A2.2-S1/S2` | done (FB-102) | `develop` | - |
| READY row 16 `A4.2-K1a` + `A3.2-S1` | done (FB-105; `field_intent` gate moved to W2-4 per FB-106) | `develop` | R2-W1 batch 1 complete. |
| READY row 19 `I1.2-S1` | done (FB-104) | `develop` | - |
| READY row 15 `A1.2-S1/S2` | done (FB-107) | `develop` | - |
| READY row 17 `F1.2-S1` + `F2.2-S1` | done (FB-108) | `develop` | - |
| READY row 18 `P1.2-S1` + `P2.2-S1` | done (FB-109) | `develop` | R2 Wave 1 complete. |
| READY row 20 (R3-W1) | done (FB-111) | `develop` | R3 Wave 1 complete. |
| W2-4 (R2-W2) | done (slices 1-7) | `develop` | - |
| W2-5 (R2-W2) | done: slices 1-4 (FB-127..129, 131); open FB-128, FB-130 | `develop` | - |
| R2-W2b CLASS-BASICS | done (FB-132, FB-134, FB-135) | `develop` | - |
| R2 release | released: `main` @3cc73f2b ([#56](https://github.com/6lackwin9/tactical-rpg/pull/56), merge commit; tree = develop @f5c6568a) | `main` | - |
| W2-6 (R3-W2) | done (FB-142); FB-140 A in the landing, develop's gate green again | `develop` | R3 Wave 2 complete. Next: R3-CLOSE. |
| R3-CLOSE | released: `main` @8d863f58 ([#57](https://github.com/6lackwin9/tactical-rpg/pull/57), merge commit; tree = develop @e6450e22); audit FB-143, no blocker | `main` | - |
| R4-1 `U2.3-TIERS` | done: slice 1 (`aa311bf4`), slice 2 (`858eaab0`) | `develop` | - |
| R4-2 `R4-CLOSE` | done (`3353e62b`) | `develop` | - |
| R4 release | released: `main` @32ae1ada ([#58](https://github.com/6lackwin9/tactical-rpg/pull/58), merge commit; tree = develop @ad903f92, which adds a whitespace fix for an MD012 lint error from 92a7282b) | `main` | Ready queue empty; next: Section 3 `R3-STAGE-SEED`, `POST-R4-CLEANUP` when Design queues them. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-146 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-09). New items start at FB-147. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
