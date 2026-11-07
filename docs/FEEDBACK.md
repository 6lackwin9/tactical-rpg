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
| R1.1-2 `C02-A` | next | - | FB-93 table answered in READY. |
| R1.1-3 `R1-BAL` | queued (after C02-A) | - | - |
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

### FB-94: AUDIT-11 lowers two shipped guards to 35 %; their text still says 50 %, and an authored 0.5 does not survive a save

- **Spec**: `docs/READY.md` R1.1-1 `AUDIT-R1b` item (4) `AUDIT-11`; FB-85 (1) ruling
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: Built as specified on `claude/audit-r1b` (not yet on `develop`; 0 goldens moved, since the frozen fixtures carry an explicit `mitigation`). The loader now drops the schema-filled `mitigation` from an unauthored DEFEND row, so those rows take `guard_common_mitigation` 0.35 / `guard_multi_round_mitigation` 0.20. In shipped data this moves two skills that never set `mitigation`: `active_guard` (the timeline's basic Guard, carried by many units) and `seventh_ward_bulwark`, both from 50 % to 35 % in real play. No golden covers it. Two side effects the row does not settle: (a) skill text renders `{mitigation}` at load time, before the key is dropped and with no ruleset in reach, so `active_guard` still reads "-50% damage" against a real 35 %; (b) the writer (`aegis content`, unchanged) drops a `mitigation` equal to the schema default 0.5 on save, so an explicitly authored 0.5 is saved without the key and then loads as 0.35. Also noted, outside AUDIT-12: a Dojo Poise break still keeps the hero's guard and queued `guard` status (the battle clears both, FB-85 (2)).
- **Recommendation**: (A, recommended) Land AUDIT-R1b as built, and in the same commit have the asset lane author an explicit `mitigation` on the two shipped rows at whatever value Design picks (0.35 keeps the new behaviour and makes the text read "-35%"; 0.5 keeps today's play). That fixes (a) for shipped data. For (b), queue a follow-up where the writer keeps an authored `mitigation` on DEFEND rows; that is a store change that re-records the resave golden, so it needs naming. (B) Render `{mitigation}` from the ruleset at display time (a token-renderer change touching the skill-text path). (C) Hold AUDIT-11 until Design rules. Build default if unanswered when the gate is green: (A) with an explicit 0.35 on both rows, so text and play agree with the FB-85 ruling; the Dojo break gap is left for a later row.

### FB-95: AUDIT-R1b (AUDIT-08..12 + FB-94) built; checks_golden re-recorded

- **Spec**: `docs/READY.md` R1.1-1 `AUDIT-R1b`; FB-85, FB-91, FB-94 rulings
- **Kind**: docs-delta
- **Status**: open
- **Finding**: AUDIT-08: in a 2H + off-hand pair a stowed weapon adds only its burden (flat and tier `stat_bonuses` and `total_flat_mitigation` read `ready_gear()`). AUDIT-09: `weapon_context`, `has_reach_weapon`, base range and `wielded_type("main")` read `Inventory::active_weapon()`; the row's `set_active_weapon_slot` does not exist, and the only switch (Dojo `switch_weapon` -> `toggle_active_weapon`) already resyncs. AUDIT-10: the tier-match check reads a missing key as its schema default; `checks_golden` re-recorded: 182 false messages removed in 91 of 1002 cases (144 `position`, 38 `guard_rounds`; the row's "73" undercounted), nothing added. AUDIT-11: the loader drops the schema-filled `mitigation` from unauthored DEFEND rows (`source.rs` `unset_default_mitigation`), so they take `guard_common_mitigation` / `guard_multi_round_mitigation`; Round 2+ `guard` is queued at `guard_later_mitigation` (own value if authored, else multi-round) via the shared `command::queue_later_guard`; shipped `active_guard` and `seventh_ward_bulwark` now author `mitigation: 0.35` (FB-94 A), so text and play read 35 %; the writer keeps an authored DEFEND `mitigation` (rank modifiers too) even at the 0.5 default, folded in with 0 golden diffs (P2.2-S1 not needed for it). AUDIT-12: the Dojo dummy uses the ruleset's wield terms; a DEFEND with `guard_rounds >= 2` queues its later-round guard in the Dojo; `next_round` re-reads `guard_status_mitigation`; a Dojo Poise break ends the broken side's guard and `guard` status, queued rounds included (`trigger::end_unit_guard`, shared with battle). Every other golden byte-identical. Tests: `rpgkit_rules/tests/weapon_switch.rs` (2), `rpgkit_content/tests/skill_family_defaults.rs` (2), `guard_rows.rs` (+2), `guard_timing.rs` (+1), `dojo_timeline.rs` (+2, 1 extended).
- **Recommendation**: HISTORY: "AUDIT-R1b (AUDIT-08..12, FB-94): a stowed 2H-pair weapon adds only burden and type/range/reach follow the ready weapon; unauthored DEFEND rows take the ruleset guard fractions (shipped `active_guard`, `seventh_ward_bulwark` author 0.35); the writer keeps an authored DEFEND `mitigation`; Dojo guard parity (wield terms, multi-round guard, break ends guard); re-recorded on purpose: `checks_golden` (182 false tier-match messages removed)". GAME_BRIEF gear: "In a 2H + off-hand pair only the ready weapon counts: the stowed one adds only its burden; weapon type, range and reach are the ready weapon's." GAME_BRIEF guard: "A DEFEND skill that authors no `mitigation` guards at `guard_common_mitigation` (1 round) or `guard_multi_round_mitigation` (multi-round); Round 2+ `guard` uses the multi-round fraction unless the skill authors its own; the writer keeps an authored value; the Dojo plays the same guard rules, and a Dojo break ends the guard." STRUCTURE: tests `weapon_switch.rs`, `skill_family_defaults.rs`; `command::queue_later_guard`, `trigger::end_unit_guard`. Also: a future DEFEND row that authors no `mitigation` still renders `{mitigation}` from the schema default (50 %) while play uses 35 % / 20 % (no shipped row today), and the schema help text for skill `mitigation` still reads as if the 0.5 default applies to DEFEND rows (Design-owned schema wording, if it should change). Environment notes for EFFICIENCY / known-issues: a cloud worktree lane must pass `CARGO_TARGET_DIR=<worktree>/target` inline (the sandbox refuses `export`), and the session-start hook's ALSA install did not run in this container (`apt-get install -y libasound2-dev` fixed `alsa-sys`).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-09 are closed and moved to the archive (2026-11-07). New notes start at UN-10.
