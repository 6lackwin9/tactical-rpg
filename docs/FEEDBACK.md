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
| READY row 16 `A4.2-K1a` + `A3.2-S1` | done (FB-105; gate rename parked FB-106) | `develop` | R2-W1 batch 1 complete. |
| READY row 19 `I1.2-S1` | done (FB-104) | `develop` | - |
| READY rows 15, 17, 18 (R2-W1 batch 2) | building: 3 parallel `logic-builder` worktrees | - | Land one by one (each re-records `rules_contract_golden`); row 18 then needs an asset-lane terrain re-save. |
| READY row 20 (R3-W1) | queued | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-104 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-105. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-105: A4.2-K1a + A3.2-S1 (row 16) landed: IR21 derive and additive RING / ARC shapes; FB-101 split checked in memory

- **Spec**: `docs/READY.md` row 16; `docs/sessions/A3_SHAPES.md`, `A4_EFFECTS.md`; FB-57, FB-101
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `rpgkit_rules::intent::derive` (IR21, the 8-step precedence of A4_EFFECTS section 2): ARC / LINE / CONE without a hazard and BURST / PLUS with `reach_max > 1` and no hazard derive `[Field, Assault]`; a shaped row with a hazard, or BURST / PLUS at reach 1 or less, derives `[Field]`; RING derives `[Stand]`; a `SELF`-only SUPPORT derives `[Stand]`, `ALLY` `[Support]`, `""` / `ALLY_OR_SELF` `[Support, Stand]`; SWEEP is left out of the IR21 shapes (legacy alias until W2-4). `rpgkit_battle` gains additive `Area::Ring` / `Area::Arc` (skill.rs, plan.rs, ai.rs, ground.rs); SWEEP / LINE / PLUS / BURST behaviour unchanged. Shipped `skills.json` and `kits/rpg/schema.json` untouched. `emberwake_rules/tests/ir21_intent.rs` (6) covers all 129 skills: raw data 22 dual-intent (8 Support,Stand + 14 Field,Assault); with the FB-101 split in memory (4 ARC, 3 RING; the 7 ids are exactly the shipped SWEEP rows) 26 = 8 + 18; primary-intent totals Assault 59, Field 23, Stand 16, Support 12, Maneuver 14, Neutral 5 (sum 129); "listed anywhere" totals 77 / 23 / 24 / 12 / 14 / 5 (each dual row counted twice). Kit tests: `rpgkit_rules/tests/ir21_intent.rs` (8), `rpgkit_battle/tests/area_shapes.rs`, `ground_hazards.rs` (+1). 0 golden diffs. Built by the desktop session (e07054b0) and finished here; first change-reviewer and full gate on this branch both clean.
- **Recommendation**: GAME_BRIEF intents: "IR21 derive (`rpgkit_rules::intent::derive`): shipped data gives 22 dual-intent skills (8 Support,Stand + 14 Field,Assault); after the W2-4 SWEEP -> 4 ARC / 3 RING split, 26 (8 + 18); primary totals Assault 59, Field 23, Stand 16, Support 12, Maneuver 14, Neutral 5". STRUCTURE: `rpgkit_rules/src/intent.rs`; tests `rpgkit_rules/tests/ir21_intent.rs`, `emberwake_rules/tests/ir21_intent.rs`, `rpgkit_battle/tests/area_shapes.rs`. PORT_NOTES / kit README: `Area::Ring` / `Area::Arc` are additive; `SWEEP` stays until W2-4 migrates `skills.json` and the schema `area` enum. HISTORY: none (no golden re-recorded).

### FB-106: row 16 gates IR21 on `round_intent`; the specs name `field_intent`

- **Spec**: `docs/READY.md` row 16 ("single `IR21` derive function in `rpgkit_rules` (rule-gated)"); `docs/specs/INTENT2.md:123` (INTENT2-K1: "Rule `field_intent`"); `docs/specs/BASERULES1_LIBRARY.md:57` (`field_intent` depends on `round_intent`)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: `intent::allowed()` (`kits/rpg/crates/rpgkit_rules/src/intent.rs:121`) checks `rules.has(ROUND_INTENT)`, so a ruleset with round intents on and `field_intent` off would get IR21 / Field intents. No code calls `allowed()` yet, so nothing plays differently and no golden moves; K1b (W2-4) would inherit the gate. Adding `field_intent` to the rule register re-records `rules_contract_golden`, which row 16 does not name, so Build landed the gate as built and parks the rename. Related W2-4 note (low): a hazard placed with `Shape::Arc` uses the cardinal heading geometry in `ground.rs`, while a skill's own ARC (`skill::arc_cells`) takes the diagonal quadrant when aimed off-axis, so the two can cover different tiles; unreachable today (`HAZARD_SHAPES` has no ARC or RING, and no shipped skill uses ARC).
- **Recommendation**: (A, recommended) a small follow-up (or the start of W2-4 K1b): register `field_intent` (depends on `round_intent`; on in `timeline`, off in `classic`), switch `allowed()` to it, and name a `rules_contract_golden` re-record. (B) keep `round_intent` until K1b and record that `field_intent` arrives with it. Either way, make the hazard ARC cells follow `skill::arc_cells` in A3.2-K2 (IR7 directional snap).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
