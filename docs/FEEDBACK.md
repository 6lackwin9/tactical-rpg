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
| W2-6 (R3-W2) | built, combined and `growth_per_stage` added (FB-141); landing held: develop's gate is red (FB-140) | `claude/w26` | Then R3-CLOSE. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-139 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-140. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-140: develop's gate is red: six full-body portraits sit outside the ASSETORG2 asset tree

- **Spec**: `docs/specs/ASSETORG2.md` section 3 (game asset tree); test `emberwake_gui` `region::tests::the_assets_follow_the_layered_tree` (`region.rs:2143`); develop @60d0dfc7 ("feat(mock_ui,assets): hybrid story full-body sprites ...", a user commit)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: 60d0dfc7 added `games/emberwake/assets/portraits/full_body/{aline,kaelen,maren,vane,vesper,watchman}.png`. ASSETORG2 section 3 allows only `portraits/{heroes,npcs,elites,bosses}/<id>.png` and `portraits/common/`, so the layered-tree test fails on develop itself (`cargo test -p emberwake_gui the_assets_follow_the_layered_tree`, reproduced at 60d0dfc7 alone). The full gate stops there, so no task can land on develop under the standing rule (push only on a clean gate). W2-6 is ready on `claude/w26` and held. Build cannot fix it: moving the user's assets or widening the layout is a design call.
- **Recommendation**: A (recommended if full-body story sprites are shipped game art): Design adds a full-body slot to ASSETORG2 section 3 (e.g. `portraits/full_body/<id>.png`, unique tiers only, or `bodies/full/<id>.png`) and Build adds it to the test's tree in the W2-6 landing. B: if they are mock-UI reference art only, move them out of `games/emberwake/assets/` (e.g. under `tools/mock_ui/`), with the user's word. Build default: none; W2-6 waits.

### FB-141: W2-6 growth_per_stage as built; one reading to confirm (the level cap)

- **Spec**: `docs/READY.md` FB-139 Option A; `docs/sessions/E_ECONOMY.md` line 12; `U2_CLASS.md` item 4 (@87fc8197 on `claude/w26`)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Units field `growth_per_stage` (schema int 0..10, default 0, the writer drops 0; `UnitDef`; content check). `foe_level(level, stage, growth)` = `level + stage x growth`, applied in `Roster::entry` and in `roster_at` / `enemy_levels_at` alike, so battle level and XP weight agree; never applied to tier `hero` or to the encounter's `party` / `guests` refs (`WorldRoster.allies`). No shipped unit sets it, so no golden moved. Not in the spec: the builder caps `foe_level` at `MAX_LEVEL` (10, or the base level when higher), the clamp the sim already uses, because levels above 10 are past the XP curve. The 0..10 schema range is also Build's choice.
- **Recommendation**: confirm the cap at `MAX_LEVEL` (recommended: an uncapped level leaves the XP curve and the stat budget tables) and the 0..10 range, or name other bounds.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
