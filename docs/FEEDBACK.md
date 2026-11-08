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
| W2-4 (R2-W2) | slices 1, 2, 3 (A-C), 5 done; slice 4 needs the `brace` row (FB-119); slice 6 (K1b) next, then 7 | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-117 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-118. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-118: W2-4 slice 3 (K2) phases B-C landed: SWEEP retired; 7 skills now ARC / RING; classic brawler kicks fire

- **Spec**: `docs/READY.md` row W2-4 (K2); `docs/sessions/A3_SHAPES.md`; FB-101, FB-114, FB-116
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Phase B (seed `games/emberwake/data/seeds/w24k2.json`, aegis content apply): `foundry_shatter`, `crushing_judgment`, `gilded_billhook_sunder`, `pneumatic_hammer_strike` -> ARC; `sweep_kick`, `cyclone_kick`, `tendril_flail` -> RING; `tendril_flail` `round_intent` cleared (IR21 Stand); the two FB-114 templates verbatim; `seeds/cast1a3.json` (a seed input) updated so re-applying it keeps the migration. Phase C: SWEEP removed from `Area`, ground `Shape`, `HAZARD_SHAPES` (7), the plan / AI / check paths and the schema options and help; a stored SWEEP is an unknown-area content error (and reads as one tile); the frozen kit's 6 kick rows in `timeline_kit.json` are RING; `ir21_intent.rs` asserts 26 = 8 + 18 dual-intent and the primary totals 59 / 23 / 16 / 12 / 14 / 5 on shipped data. Re-recorded (named in W2-4, FB-116): `sim_golden` (12 of 22 runs: classic brawler-class units now use their kicks), `checks_golden` (SWEEP leaves the area enum message), `phasefold_golden` (49 of 72 fights: the Goblin Berserker picks Body Blow II over the RING kick with no foe adjacent). Every other golden byte-identical. New tests: a stored SWEEP is an error / one tile; the split ships with its shapes. Note: re-applying `cast1a3.json` still changes unrelated rows (a 0.35 `mitigation` back to 0.5, the Bile Phial placeholder): drift from before this branch.
- **Recommendation**: HISTORY: "W2-4 K2 phases B-C: area SWEEP retired; the 7 SWEEP skills are ARC / RING (FB-101, FB-114); re-recorded on purpose: `sim_golden` (classic kicks fire), `checks_golden` (enum message), `phasefold_golden` (FB-116)". GAME_BRIEF shapes: area shapes NONE, PLUS, BURST, LINE, CONE, RING, ARC; hazard shapes SKILL, NONE, PLUS, BURST, LINE, RING, ARC. Stale SWEEP mentions to update (Design-owned): `engine/docs/authoring/combat_hooks.md:71` (hazard shapes), GAME_BRIEF lines ~72 and ~79, `docs/specs/INTENT2.md`, the CAST1 specs, `decisions/combat.md`. `tendril_flail`'s description still opens "Assault, Standard: ... a sweep of" though it now derives Stand: reword?

### FB-119: W2-4 slice 4 (INTENT2-A "brace") has no row to seed

- **Spec**: `docs/READY.md` row W2-4 ("`INTENT2-A` content seed (`brace` + 7 shapes)"); `docs/specs/INTENT2.md` (INTENT2-A, question 5: "a separate, costlier multi-round guard skill (`brace`, `0.20` mitigation, queuing the full-round `guard` status on Round 2+)"); `docs/sessions/S1_STATUS.md`; FB-18, FB-48
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: The 7 shape fixes are done (FB-118). For `brace`, no spec gives the row: name, `blocks`, `recovery_blocks`, AP cost, `guard_rounds` (2-5), whether `mitigation` is authored or left to `guard_multi_round_mitigation` (0.20), `rounds` / ranks, who has it (classes, units, characters, a drill or deck slot), and its description. INTENT2_SKILLS lists only `tl_brace_counter` (a different, test skill). The full INTENT2-A line also names gap-close moves (`lunge`, `pounce`, `charge_in`), removing 72 redundant `round_intent` values, `strike` for two bosses, Brand recovery, an item table, terms and a `tl_` area fight; the W2-4 row names only `brace` + 7 shapes, so Build reads the rest as out of W2-4.
- **Recommendation**: (A, recommended) Design supplies the `brace` row (the fields above) and where it is granted; Build seeds it through `aegis content` with a named golden check. (B) confirm the remaining INTENT2-A items are out of W2-4 (or list which belong). Build continues with slice 6 (K1b) meanwhile; slice 4 lands when answered.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
