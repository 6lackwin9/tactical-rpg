# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| Done on `develop` | done | `develop` | ATTACKLOCK1 slice 1 (off until F1.2-S1), SIMCOUNT1, VISCLAR1-D (user look after the combat silos, UN-02), TESTCLEAN1, SKILLS1, SKILLS1-FB, KITIDS1, BASERULES1-M and -M2, U1.2-S1 and -S2, U2.2-S1 to -S3, U3.2-S1 and -S2, U4.2-S1 and -S2, S1.2-S1 to -S3, S2.2-S1 and -S2, U1.2-S3a to -S5, U3.2-S3 to -S5, U4.2-S3 and -S4, S1.2-S4, S2.2-S3 and -S4, AUDIT-R1. |
| R1 release | released: `main` @98ab973a (PR #55, merge commit; tree = develop @f7f5cb71) | `main` | Next: Phase R1.1. |
| R1.1-1 `AUDIT-R1b` (AUDIT-08..12) | building: 3 `logic-builder` lanes (A: AUDIT-08+09, B: AUDIT-11+12, C: AUDIT-10) | `claude/audit-r1b` | UN-07 approved; R1.1 runs straight through. |
| R1.1-2 `C02-A`, R1.1-3 `R1-BAL` | queued (after AUDIT-R1b) | - | C02-A value table asked in FB-93. |
| READY rows 14-20 (R2-W1, R3-W1) | queued (after R1.1) | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-92 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07). New items start at FB-93. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-93: C02-A (R1.1-2) needs the full value table before it can be seeded

- **Spec**: `docs/READY.md` R1.1-2 `C02-A`; FB-81 and FB-83 (Build proposals accepted in READY section 4)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: A pre-dispatch check of the shipped data shows that four parts of the row would need Build to guess design intent. (1) Temperaments: the row names Tev Quill `keen_senses` and Mara Voss `steady_nerves` "etc."; the four worlds hold 40 characters (ashlands 23, sunk_ward 7, tidewrack 7, iron_reaches 1) and none has a `temperament` today, so "etc." has no list behind it. (2) `default_intent`: no character values are given (vocabulary: the ruleset's intents). (3) Level 3 passives (FB-81 proposal): none of the 13 proposed rows exists in `passives.json`, so each needs an id, name, description and effect. Two do not fit the checker: a Level 3 passive may only carry gear-legal effects (`grants::Grant::gear_ok`: no `unlock`, `foresight`, `stagger_taken`, `meter` or `proficiency`), so "resist stagger 15" works only as `resist` against the `stagger` status, not as reduced poise damage (`stagger_taken`); and "forced-move immunity while guarding" (shield) has no conditional effect type (it would be a `trigger` row, a new behaviour). (4) "A signature skill" for bow, crossbow and fist names no skill. The Level 2 table (crit +2 / +1, burden -1 by family) and the `shd_tower` text rewrite are fully specified.
- **Recommendation**: (A, recommended) Design supplies the table in READY: temperament (or none) per character, `default_intent` (or none) per hero, the 13 Level 3 passive rows (id, effect, value), the shield effect (drop it, or a plain `resist` / `forced_move` substitute) and the three signature skill ids (or leave those families without a Level 3). Build seeds it all in one commit, so the five named goldens are re-recorded once. (B) If the table is not answered by the time AUDIT-R1b lands, Build seeds only the specified part (Level 2 table, `shd_tower` text, Tev and Mara temperaments, sim `character_overrides` wiring) under C02-A and parks the rest as C02-B, which re-records the same goldens a second time.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-09 are closed and moved to the archive (2026-11-07). New notes start at UN-10.
