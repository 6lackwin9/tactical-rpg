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
| W2-4 (R2-W2) | slices 1, 2, 3, 5 done; slice 6a (K1b rules) in review rework, lands with 6b (UI); slice 4 waits on FB-119; FB-120 open | `claude/w24-k1b` | Then slice 7. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-117 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-118. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-119: W2-4 slice 4 (INTENT2-A "brace") has no row to seed

- **Spec**: `docs/READY.md` row W2-4 ("`INTENT2-A` content seed (`brace` + 7 shapes)"); `docs/specs/INTENT2.md` (INTENT2-A, question 5: "a separate, costlier multi-round guard skill (`brace`, `0.20` mitigation, queuing the full-round `guard` status on Round 2+)"); `docs/sessions/S1_STATUS.md`; FB-18, FB-48
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: The 7 shape fixes are done (FB-118). For `brace`, no spec gives the row: name, `blocks`, `recovery_blocks`, AP cost, `guard_rounds` (2-5), whether `mitigation` is authored or left to `guard_multi_round_mitigation` (0.20), `rounds` / ranks, who has it (classes, units, characters, a drill or deck slot), and its description. INTENT2_SKILLS lists only `tl_brace_counter` (a different, test skill). The full INTENT2-A line also names gap-close moves (`lunge`, `pounce`, `charge_in`), removing 72 redundant `round_intent` values, `strike` for two bosses, Brand recovery, an item table, terms and a `tl_` area fight; the W2-4 row names only `brace` + 7 shapes, so Build reads the rest as out of W2-4.
- **Recommendation**: (A, recommended) Design supplies the `brace` row (the fields above) and where it is granted; Build seeds it through `aegis content` with a named golden check. (B) confirm the remaining INTENT2-A items are out of W2-4 (or list which belong). Build continues with slice 6 (K1b) meanwhile; slice 4 lands when answered.

### FB-120: W2-4 slice 6 (K1b): two spec values disagree, and the frozen kit does not exercise Field

- **Spec**: `docs/READY.md` row W2-4 (K1b); `docs/specs/INTENT2.md` IR17, question 1 ("half splash stays as the first value", user 2026-11-05), the Field-bonus table (`power` x 1.25); `docs/sessions/A3_SHAPES.md` section 3 (`splash_mult` 0.75, 0.25..1.00); `docs/sessions/A4_EFFECTS.md` section 3 (`field_bonus_values`: power 1.15x, Tunable)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: Slice 6a (rules and planner: `RoundIntent::Field`, skill rows from `intent::allowed`, Field source tile and bonus, the target-locked shaped Assault row with `splash_mult` and `assault_shape_blocks`, the block-limit check, a minimal AI Field row) is built on `claude/w24-k1b` and in rework after review; it lands together with 6b (the UI), since the shipped timeline already lists `field_intent`. Two values disagree between specs: (1) `splash_mult`: 0.5 (INTENT2 IR17 / question 1, the user's answer) vs 0.75 (A3_SHAPES); built as 0.5, range 0.25..1.00 (one row-only number; switching is a one-line change plus a `rules_contract_golden` line). (2) The Field `power` bonus: x1.25 (INTENT2) vs x1.15 (A4_EFFECTS); not applied (no shipped skill gets it yet). Also, A4_EFFECTS marks `field_bonus_values` Tunable, but the built sizes (ARC 5, LINE 4, hazard rounds cap 5) are constants. (3) The battle goldens read the frozen `timeline_kit.json`, whose timeline row does not list `field_intent`, so no golden exercises the Field / Assault rows (shipped play does).
- **Recommendation**: (1) confirm 0.5 (the user's answer; A3_SHAPES to follow) or 0.75; (2) pick 1.25 or 1.15 (Build then applies it where the derived bonus is `power`); confirm whether the Field bonus sizes become ruleset numbers now or stay constants; (3) (A, recommended) add `field_intent` to the frozen kit's timeline row in this slice, naming `timeline_golden` / `sim_golden` re-records, so the rows are covered; (B) keep the frozen kit as is until a later re-freeze. A balance run on shipped data (`sim feel`) is advisable before R2-CLOSE, since dual area skills now reach enemies through the Field row.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
