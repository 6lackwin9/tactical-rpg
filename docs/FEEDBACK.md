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
| W2-4 (R2-W2) | slice 2 done (FB-112), slice 3 phase A done (FB-113); slice 1 (`field_intent`) gated, in review; K2 phases B-C held (FB-114) | `claude/w24-field-intent` | Next: slice 5 (A4.2 EffectSpec). |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-112 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-113. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-113: W2-4 slice 3 (K2) phase A landed: `affects`, IR7 snap, CONE, hazard ARC geometry; no play change

- **Spec**: `docs/READY.md` row W2-4 (K2); `docs/sessions/A3_SHAPES.md`; `docs/specs/INTENT2.md` IR6, IR7, IR9; FB-57, FB-101, FB-106
- **Kind**: docs-delta
- **Status**: open
- **Finding**: New skill field `affects` (schema skills, `""` / `all` / `enemies` / `allies`, ATTACK, section `area`; `rpgkit_data::Affects`); `skill::affects_by`: an explicit value wins, else a skill that leaves a hazard hits all, the Assault and Stand rows hit enemies, Support hits allies, other rows (Maneuver, Channel, the coming Field) and classic hit all; the user is never hit; applied at impact, in the plan estimate and in AI scoring. IR7 snap (`skill::snap`: dominant axis, a tie is the exact diagonal); LINE / ARC / new CONE are drawn on the source tile; every adjacent source gives exactly the pre-K2 tiles (tested for all 8 neighbours) and the planner still offers only adjacent sources for drawn shapes, so play is unchanged. Hazard `Shape::Arc` goes through `skill::arc_cells` (FB-106); hazard LINE uses the snap; `HAZARD_SHAPES` and the schema `area` enum gain RING and ARC (and CONE for skills); SWEEP stays until phase C. New check: "Affects needs an area". Re-recorded (named): `checks_golden` (the 2 BOGUS-area cases list the new options). Every other golden byte-identical (`timeline_golden`, `sim_golden`, `phasefold`, `rules`, `journey`, contract, resave). Tests: `area_affects.rs` (5), `shape_direction.rs` (6), `area_affects_field.rs` (3), `ground_hazards.rs` (+1); `ir21_intent.rs` now holds before and after the SWEEP migration.
- **Recommendation**: GAME_BRIEF shapes: the IR7 snap and source-drawn LINE / ARC / CONE; the `affects` defaults. STRUCTURE: the three new tests. Kit README (`kits/rpg/README.md:16` still says "SWEEP stays until W2-4"): `Affects`, `affects_by`, CONE, snap. `engine/docs/authoring/combat_hooks.md:71`: hazard shapes now include RING and ARC. HISTORY: "A3.2-K2 phase A: `affects`, IR7 snap, CONE, hazard ARC = skill ARC; re-recorded on purpose: `checks_golden` (2 `area` enum cases)". Follow-up nits: the HOOKS1 8.4 "inside" set (`command.rs` ~534) still counts a friend the filter spares; AI area filtering has no direct test.

### FB-114: K2 phases B and C (SWEEP -> RING / ARC) change classic play; four questions before migrating

- **Spec**: `docs/sessions/A3_SHAPES.md` section 2 ("keep SWEEP as a legacy alias under classic so the frozen kit's brawler does not change classic goldens; remove SWEEP last when the frozen kit migrates") and acceptance 1 ("classic goldens remain byte-identical"); FB-57(1), FB-96, FB-101; INTENT2 IR9
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: (1) Today the planner refuses SWEEP, so the SWEEP skills never fire. Migrating them to RING makes them live: the builder migrated the frozen kit's `sweep_kick` / `cyclone_kick` ranks (6 rows in `timeline_kit.json`) to RING, and the classic `sim_golden` moved in 12 of 22 runs (99 lines; e.g. brawler vs 3 minions at level 3 goes 0/0 -> 100/100), because every brawler-class unit (`goblin_brawler`, `goblin_berserker`, `enemy_brawler`, `ambush_berserker`, `goblin_warlord`) starts kicking under classic. A probe confirmed the move comes only from the fixture. Build reverted that fixture change to keep acceptance 1, so phase A landed with no play change. The shipped-data migration (phase B) does the same to classic world fights (no golden covers them). (2) `tendril_flail` becomes RING but authors `round_intent: "assault"`, while IR21 derives Stand for RING. (3) The shipped `sweep_kick` / `cyclone_kick` templates say "Aim at a diagonal neighbour to pick the quadrant" and "hits allies too", which no longer hold (RING has no quadrant; with `affects` defaults allies are spared outside classic). (4) Spec conflict on `affects`: A3_SHAPES section 1 says every shaped skill declares it with row-derived defaults (built: an explicit value always wins, a hazard skill hits all in every row), while IR9 says an Assault shape "cannot be marked, so it hits foes only, as does a Stand ring"; and the field uses `enemies` / `allies` (A3_SHAPES) while IR9 and the hazard-row `affects` (`combat_hooks.md:69`) use `foes` / `friends`. Also: seed `cast1a3.json` still holds 3 SWEEP rows (re-applying it would undo a migration).
- **Recommendation**: (1) (A, recommended) accept the classic change as the point of the migration: migrate the frozen kit and the shipped rows together in phase B, re-recording `sim_golden` (named in W2-4) and noting in HISTORY that brawler kicks now fire in classic; (B) keep SWEEP firing nowhere in classic by adding a classic-only alias (more code, keeps acceptance 1 literally). (2) set `tendril_flail` `round_intent` to `""` so the row follows IR21 (Stand), or keep Assault. (3) Supply the two template texts (or let the asset lane drop the quadrant sentence and "hits allies too"). (4) Pick the rule (A3_SHAPES: explicit wins in every row; or IR9: Assault and Stand shapes always hit foes only) and one vocabulary (`enemies` / `allies` or `foes` / `friends`). Build continues with W2-4 slices 4-7 meanwhile only where they do not depend on the migration (slice 4, the INTENT2-A seed, needs it, so slice 5 A4.2 goes next).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-10 are closed and moved to the archive (2026-11-08). New notes start at UN-11.
