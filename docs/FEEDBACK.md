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
| PR5-51a `GEAR-REWORK1-KIT` | done (FB-264; FB-265 questions and notes for 51b): Switch Weapon retired, role-gated strict gear rule, slot rules, off-hand mechanics and button, knife Throw; 0 golden diffs | `develop` | - |
| PR5-51b `GEAR-REWORK1-DATA` | next | - | - |
| PR5-52 to PR5-56 | queued, in order (user 2026-11-14) | - | - |

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

### FB-264: PR5-51a `GEAR-REWORK1-KIT` landed

- **Spec**: READY Section 1 row PR5-51a; `docs/sessions/U3_GEAR_REWORK.md` §1-6 (the `PR5-51a` squash commit on `develop`, 2026-11-15)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Kit / schema / GUI mechanics (FEEDBACK kit README line). Every new rule needs metadata only PR5-51b seeds (class proficiency `role`; family `slot_rule`, `pairs_with`, `off_hand_skill`, `empower_next`, `power_stat_override`, `armor_weight`), so today's data keeps today's numbers: **0 golden diffs**. The one ungated change: a 2H main beside an off-hand weapon is refused (Switch Weapon retired); only two shipped Dojo presets held such a pair (`claymore_backup`, the shortbow + dagger preset), and they now apply with the off hand empty. GUI: an "Off-Hand" button in the battle planner's neutral row (the worn off-hand's action: jab, Reload, Fade-Step, Cite Passage; greyed with the kit's reason) and an off-hand chip on the Gear tab's Off Hand row. Tests: `rpgkit_rules` `strict_gear.rs` (6), `slot_rules.rs` (5); `rpgkit_world` `strict_wear.rs` (3); `rpgkit_battle` `offhand.rs` (8); `rpgkit_content` `gear_rework_checks.rs` (5), `hand_rule_checks.rs` (5); `rpgkit_host` `switch_retired.rs` (1); GUI `battle/tests/offhand_tests.rs` (4) and a Gear-tab test; switch-era tests rewritten (`weapon_switch.rs`, `item_grants.rs`, `unit_templates.rs`, `party_ops.rs`, `family_rules.rs`, `recipe_checks.rs`, `emberwake_world/tests/dojo.rs`). Gate: 3379 passed, 0 failed.
- **Recommendation**: engine/docs/PORT_NOTES.md (line ~108): `needs_weapon_switch` / `toggle_active_weapon` -> retired by GEAR-REWORK1 (PR5-51a). GAME_BRIEF (gear, line ~226 `Inventory::ready_gear` / `active_weapon`): replace with "Off hand (GEAR-REWORK1): family `slot_rule` fixes the hands; a `2h_rig` main shares the off hand with its `pairs_with` families; a `1h_light` off-hand jab reaches the weapon's band; ammo `empower_next` empowers ranged shots, Reload only at 0, and gives a light main a pip-costed Throw; orb lends INT when higher; strict classes (with roles) lock Prof 0 and unmet raw requirements and scale power / mitigation 0.80 / 1.00 / 1.25." STRUCTURE: `rpgkit_battle/src/offhand.rs` and the test files above. HISTORY: "U3.2 Switch Weapon, the stowed backup weapon and the 2H + off-hand weapon pair retired by GEAR-REWORK1 (PR5-51a); the soft untrained penalty remains for classes without roles until PR5-51b."

### FB-265: PR5-51a questions and notes for PR5-51b

- **Spec**: READY rows PR5-51a / PR5-51b (@ the `PR5-51a` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (a) Reload is open only at 0 ammo (the row's "Reload basic when 0"), checked against the current meter, so "shoot, then Reload" in one round at 1 pip is refused at planning time; the AI never chooses to Reload early. (b) The planner's damage preview shows the un-empowered number (the empower applies when the attack starts). (c) The Throw goes to any `1h_light` main (knife, dagger, fist) beside ammo; data can make it knife-only (`weapon_types: ["knife"]` with `min_proficiency >= 1` on the throw skill). (d) §1's retirement of the soft `untrained_penalty` / `unfit_penalty` is not done yet: `PROF_GEAR_MULT[0] = 1.0` keeps the soft penalty for gear worn while locked (e.g. enemy loadouts). (e) For 51b: the strict lock is enforced in party / save paths and the Dojo, not in battle builds or the content loadout check, so strict-class units or templates could wear locked gear under the old soft penalty; `keepsake_kaelen` / `keepsake_vesper` (3 bonuses each) will fail the accessory limit once roles exist; the two Dojo presets above need new pairs. (f) Not built (not in the clause text): retiring `+level` training power under the strict rule.
- **Recommendation**: (a) keep (recommended) or "any time below max", plus plan-aware meter checks (`SimState::meter`) in a later kit row; (b) add the empowered number to the preview later if wanted; (c) knife-only through data in 51b (recommended); (d)-(e) 51b: a strict-mode loadout content check and `*_drill` passives for enemies, retire the soft penalty with the data, trim the two keepsakes, fix the presets; (f) decide with the 51b numbers.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
