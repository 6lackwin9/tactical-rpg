# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 to PR4 rows (R1 to R4 releases, the Post-R4 queue PR4-1..PR4-9) | released or done | `main` / `develop` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-10) |
| PR5-1 to PR5-6, PR5-8, PR5-16, PR5-17, PR5-20 to PR5-25 | done (FB-168 to FB-189, FB-194 to FB-196, answered) | `develop` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-10) |
| PR5-7, PR5-9 to PR5-13b, PR5-18, PR5-19, PR5-26 to PR5-35 | done, released to `main` in #62 (FB-190 to FB-234, answered @cf3d93b4) | `main` | moved to `docs/archive/FEEDBACK_CLOSED.md` (Build Status rows, 2026-11-11) |
| PR5-36 to PR5-48 | done (FB-235 to FB-259, answered through @35c365b9) | `develop` | full rows in `docs/archive/FEEDBACK_CLOSED.md` |
| PR5-49 `MAP-POLISH1` | done (FB-260; FB-261 questions): region `map_reveal` with grey locked places, sidecar beside the drawn background, location `art`, saved Legend, Act 1 `map_pos` = sidecar, editor end anchors and pins; 0 golden diffs | `develop` | - |
| PR5-50 `KIT-HOOKS-CLEANUP` | done (FB-262; FB-263 questions): `skill_reach`, `friendly` / `hostile` sides, preset and training checks; `checks_golden` re-recorded (3 lines) | `develop` | - |
| PR5-51a `GEAR-REWORK1-KIT` | next | - | - |
| PR5-51b to PR5-56 | queued, in order (user 2026-11-14) | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-261 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-14). New items start at FB-262. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-262: PR5-50 `KIT-HOOKS-CLEANUP` landed

- **Spec**: READY Section 1 row PR5-50; FB-171, FB-189, FB-204, FB-212 (the `PR5-50` squash commit on `develop`, 2026-11-14)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) `skill_reach` replaces `spell_reach` (FEEDBACK kit README line); `wpn_orb` migrated to `{"type": "skill_reach", "reach": 1, "kind": "SPELL"}` (seed `seeds/kithooks1.json`, the only shipped `spell_reach`); battle reach unchanged (every battle / sim golden byte-identical; `W_SPELL_REACH` kept so `rules_contract_golden` does not move; the frozen `timeline_kit.json` reads through the alias). (2) Phase `side.to` `friendly` / `hostile` / `hostile_all` with `party` / `enemy` aliases (no `_all` alias was needed); the unit-row `side`, character `role`, trigger unit names and host JSON keep `party` (they name hero rows, not battle sides); an AI-controlled `friendly` ally is today's allied guest (battle log equal cell for cell). (3) `dojo_presets.gear` unknown item: the check existed since PR5-22, a test added; new `train_proficiency` warning. **`checks_golden` re-recorded**: 3 lines, the unknown-effect lists name `skill_reach` where they named `spell_reach`. Tests: kit `skill_reach.rs` (7), `skill_reach_checks.rs` (3), `phase_side_swap.rs` (+3), `dojo_preset_checks.rs` (+1), `recipe_checks.rs` (+1), `phase_side_swap_checks.rs` (message updated); game `passive_grants.rs` oracle pushes `SkillReach`. Gate 3341 passed, 0 failed.
- **Recommendation**: HISTORY: "`checks_golden` re-recorded (PR5-50): the unknown effect type lists name `skill_reach` where they named `spell_reach` (3 lines)." GAME_BRIEF: "`wpn_orb`'s reach bonus is `skill_reach {reach: 1, kind: SPELL}`; phase sides are `friendly` / `hostile` / `hostile_all`." STRUCTURE: tests `rpgkit_battle/tests/skill_reach.rs`, `rpgkit_content/tests/skill_reach_checks.rs`; seed `seeds/kithooks1.json`.

### FB-263: PR5-50 readings and questions

- **Spec**: READY row PR5-50 (@ the `PR5-50` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (a) The row asks for a warning when a `train_proficiency` targets "a family that 0 classes hold at minor proficiency (1)"; the effect names a class role, not a family (`{role, to}`), so the warning is per role at level `to - 1` and covers every role, and since no shipped class sets a proficiency `role` yet it raises 3 warnings (`train_secondary`, `train_third`, `train_minor`) until PR5-51b adds the roles. (b) `kind: SPELL` is not a skill kind in code; it reads as the old spell group (ATTACK with a spell source), which is what reproduces `spell_reach`. (c) The constant `W_SPELL_REACH` keeps its name (it now weighs every `skill_reach` tile) so `rules_contract_golden` stays unchanged.
- **Recommendation**: (a) keep every role (recommended; PR5-51b's roles clear the warnings) or limit to `minor`; (c) rename to `W_SKILL_REACH` with a named 1-line `rules_contract_golden` re-record in a later row if wanted; (d) aliasing the unit-row `side` field to `friendly` / `hostile` too is a larger change (many `== "party"` checks) left out.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
