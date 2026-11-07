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
| READY row 18 `P1.2-S1` + `P2.2-S1` | next: rebase, asset terrain seed, contract re-record | `claude/p12-s1` | - |
| READY row 20 (R3-W1) | queued | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-107 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-108. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-108: F1.2-S1 + F2.2-S1 (row 17) landed: zero-power attacks take no lock, `Unit::initiative_rating`, debt status ids as ruleset values

- **Spec**: `docs/READY.md` row 17; `docs/sessions/F_FIGHT_FLOW.md` sections 1-2; FB-58 a, b, c
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (a) `Command::lock_tile` takes no lock when the attack's `skill_mult <= 0.0` (the resolver's "damaging" test), so `intimidate` and `tripwire_snare` never lock; shipped rulesets keep `attack_lock = none` (W2-5 turns it on). A unit weakened to a 0 factor would also skip the lock (an edge case). (c) `Unit::initiative_rating()` is the one read point for block order: `Command::initiative` (`priority * 1000 + rating`) and the planner's within-block move order (`plan.rs`, review fix) read it; named `initiative_rating` so it does not clash with the `initiative` field. (b) New row-only string ruleset fields `debt_stagger_status` / `debt_stun_status` (defaults `stagger` / `stun`, empty reads the default, an unknown status id is a notes / content-check error); the thresholds were already ruleset numbers (S1.2-S3). `roll_over` puts the debt status into `st.active` (not queued) with `rounds_left` 1 when it has none (a longer existing count is kept): golden-neutral, since a count of 1 drops at the next roll-over as a missing count did, and both statuses stack `refresh_max`. Row-only strings round-trip through the writer (as `poise_mode` does), so no schema entry was needed. Re-recorded (named): `rules_contract_golden` (+2 tunable lines). 0 battle/sim golden diffs. Tests: `attack_lock.rs` (+2), `move_rate.rs` (+1), `status_reads.rs` (+2). Landed after row 15 (merge resolved `FIELD_RULES` to 32 entries).
- **Recommendation**: GAME_BRIEF fight flow: "AP debt at a round start of `debt_stagger` (3) or `debt_stun` (5) puts the ruleset's `debt_stagger_status` (`stagger`) or `debt_stun_status` (`stun`) into `st.active` with `rounds_left` 1; attack lock never locks a zero-power attack; block order reads `Unit::initiative_rating`". HISTORY: "F1.2-S1 / F2.2-S1: re-recorded on purpose: `rules_contract_golden` (+`debt_stagger_status`, `debt_stun_status`)". Kit README / `rule_library.md`: the two row-only fields next to `debt_stagger` / `debt_stun`. ATTACKLOCK1-O (W2-5) no longer waits on the power test. Open: an empty debt status id reads as the default, not "no penalty"; add an explicit off value if wanted.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
