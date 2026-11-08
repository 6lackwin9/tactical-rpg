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
| R2-W2b CLASS-BASICS | logic half (FB-132) and Maneuver AI row (FB-134) done; data half held (FB-133) | - | Then R2-CLOSE. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-131 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-132. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-132: R2-W2b CLASS-BASICS logic half landed: class `basics` merged into the deck rule (inert until data)

- **Spec**: `docs/READY.md` row R2-W2b (CLASS-BASICS)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Class field `basics` (skill ids; schema, `ClassDef.basics`, stored only when set so classes re-save byte for byte). Under rule `deck` a unit's deck rule adds its class's basics after the ruleset's (`DeckRule::with_class`, by family, each once): open, no deck slot, never an active; heroes and enemies alike (`build_from`, the session fallback); the auto order and hotkeys append them; the Party / Dojo deck views and the saved-deck check count them. Checks: a class basic must be tier 1 and non-engine; class basics count as basics in unit, member, phase-effect and round-script deck checks; a deck `active` naming one is an error (`order` may name it); warning when a class basic is also in the class's `skill_path`. Classic unchanged. No class sets `basics` yet, so every golden is byte-identical. Tests: `rpgkit_rules/tests/class_basics.rs` (6), `rpgkit_battle/tests/class_basics.rs` (2), `rpgkit_content/tests/class_basics_checks.rs` (4), `rpgkit_world/tests/deck.rs` (+1). To watch once data lands: the Party tab reads the world catalog's class while the saved-deck check reads the base catalog (they differ only if a world overrides a class's `basics`); a saved deck whose `active` names a family that becomes a class basic starts failing its check (migration or tolerance for saves).
- **Recommendation**: GAME_BRIEF / kit README: the class `basics` rule above; STRUCTURE: the three new test files; HISTORY: none (no golden).

### FB-133: CLASS-BASICS data half: three skill rows, `disengage_step`'s recovery, `sweep_kick`'s level

- **Spec**: `docs/READY.md` row R2-W2b (the three new skills in prose); FB-123 (A), FB-125 (1), FB-130 (answered: path slots, Brimstone text, Maneuver AI row)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: The seed is drafted (skills, classes, units, ruleset in that order) and now also takes FB-123 (A) (`brace` leaves `warrior.skill_path`) and FB-125 (1) (Brimstone's text). Three parts still need Design. (1) The row gives the three new skills in prose; Build's reading: `disengage_step` DISENGAGE, level 1, 1 AP, 1 block, `jump_tiles` 2, `learned_only`, `round_intent` maneuver; `tend_wounds` SUPPORT, level 1, 2 AP, 2 blocks, `heal_base` 15, `heal_stat` KNOW, `heal_mult` 1.0, reach 0-2, `target_mode` ALLY_OR_SELF, `learned_only`; `cinder_spark` ATTACK, level 1, 2 AP, 2 blocks, area PLUS, `power_mult` 0.5, reach 2-3, `learned_only`; names and descriptions are not given. (2) `disengage_step`'s "1b + 1b rec" cannot be stored: the schema allows `recovery_blocks` only on ATTACK, SUPPORT and FOCUS skills. (3) `sweep_kick` is level 2; as the brawler's basic, should it become level 1 (and leave `brawler.skill_path`, as `brace` leaves the warrior's)? Deck orders follow the row: `founder_hollis` (ranger) `maneuver` -> `["disengage_step"]`; `founder_dunstan`, `founder_kaelen`, `founder_maren` drop their `maneuver` key. The Maneuver AI row (FB-130) is pure logic and is being built meanwhile.
- **Recommendation**: supply the three skill rows (names, descriptions; for `disengage_step` either widen the schema so DISENGAGE may carry `recovery_blocks`, or drop the recovery block) and confirm `sweep_kick`'s level / path. Build then seeds the data and re-records the named `checks_golden`, `sim_golden`, `phasefold_golden`.

### FB-134: CLASS-BASICS Maneuver gap-close AI row landed (FB-130 A); readings to confirm

- **Spec**: `docs/READY.md` row R2-W2b (FB-130 Option A); `docs/specs/INTENT2_MATRIX.md` section 2; `docs/sessions/I_AI.md` section 1
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `ROW_GAP_CLOSE` (under `field_intent`; `ROWS` now 11): any role, reckless included, holding a gap-close attack (single-target, `dash_tiles` > 0, no `position`: `lunge`, `pounce`, `charge_in`; positioned attacks such as `flank_cut` stay with row 4) in its Maneuver order picks a Maneuver Engage round opened by it when no visible foe is inside its basic attack band, a ranked visible foe beyond the band is within the dash (`round_intent::engage_reach`), and the landing tile is safe; taunter-first; hidden foes never targeted. No golden moved (no unit in the golden suites holds a dash skill under `field_intent`); feel unchanged (76 %). Test `rpgkit_battle/tests/gap_close_row.rs` (4). Readings (the spec does not say): (1) position after row Z (Field), before row 5 (Assault) (the other reading: right after row 4; one line in `ROWS`); (2) "safe landing" = no `ground::harms`, no `arena::due_on`, and the round's likely incoming damage below the unit's HP (the existing `ai_pressure` lethal test, not the stricter 0.5 x HP focus test); (3) "out of normal melee reach" = no visible foe inside the basic band at all. Also: under `round_intent` a dash without `position` lands on the target's flank tiles (MR1), so a 3-tile dash reaches a foe 2 tiles away head-on but not 3; intended?
- **Recommendation**: GAME_BRIEF / kit README AI rows: the gap-close row as above; STRUCTURE: test `gap_close_row.rs`; HISTORY: none (no golden). Confirm readings (1)-(3) and the MR1 dash landing.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
