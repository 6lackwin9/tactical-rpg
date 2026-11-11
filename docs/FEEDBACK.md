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
| PR5-7 `PK1-S3` | done (FB-190 data; FB-191 balance run, 6 findings for Design) | `develop` | -
| PR5-18 `WEAPONROSTER1` | done (FB-192; FB-193 sims and questions): class lineup, default loadouts, placeholder `warrior/2h_polearm`, FB-188 rulings; `phasefold_golden` re-recorded | `develop` | -
| PR5-26 `AUDIT-PR5-BUGS` | done (FB-197; FB-198 limit-blocker for the RANK_KEYS clause and questions): audit 1A bugs in five groups; 0 golden diffs | `develop` | -
| PR5-27 `PK1-S3-FIX` | data done (FB-200); the PR5-26 Vent bug fixed (FB-202); Vent numbers wait for Design (FB-201) | `develop` | -
| PR5-28 `R14-SPEC-CLEANUP` | items (2)-(3) done (FB-203); item (1) `spell_reach` parked (FB-204 limit-blocker) | `develop` | -
| PR5-29 `LAYER-DOWN-1` | parts (3) tools and (1) kit names done (FB-205; `rules_contract_golden` re-recorded); part (2) engine waits (FB-206) | `develop` | -
| PR5-30 `LAYER-DOWN-2` | done (FB-207): gear families and terrain types from data, Dojo presets and cast as data rows; 0 golden diffs | `develop` | -
| PR5-26b `AUDIT-PR5-FOLLOWUP` | done (FB-208): non-rank keys in rank modifiers refused, a crisis set skips that round's decay, Vent -50 / 10%; 0 golden diffs | `develop` | -
| PR5-9 `PK2-S2` | done (FB-209; `checks_golden` re-recorded: item types gain GRAFT); questions in FB-210 | `develop` | -
| PR5-10 `PK2-S3` | done (FB-211; FB-212 questions): recipes kind, NPC `recipe` service and pane, member and item effects; drill / repair / reforge refused until PR5-12 / PR5-13b; 0 golden diffs | `develop` | -
| PR5-12 `PK2-S4` | done (FB-213; `checks_golden` re-recorded: item types gain MASTERY_BOOK; FB-214 questions): weapon mastery, techniques, the family keystone, `durability.from_act` | `develop` | next: PR5-13a

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-206 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-11). New items start at FB-207. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-207: PR5-30 `LAYER-DOWN-2` landed (gear families and terrain types from data; Dojo presets and cast as data)

- **Spec**: READY Section 1 row PR5-30; audit 2.1 (the `PR5-30` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-208: PR5-26b `AUDIT-PR5-FOLLOWUP` landed (rank-modifier keys, crisis decay, Vent numbers)

- **Spec**: READY Section 1 row PR5-26b; `FB-198` B9 Option B and Q2 Option A; `FB-201` (a) Option A (the `PR5-26b` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: (1) `hp_cost_pct` and `uses_per_battle` stay `"rank": false`; `source::rank_key_errors`: a key in a skill's rank modifier that is not a rank field (`RANK_KEYS`, the schema's `"rank": true`) or the rank's `description` is one error naming the skill, the rank and the key ("... which is not a rank field; it is the same at every rank (set it on the entry)."); the loader reports it (the entry still loads) and the writer's validate hook refuses the entry (it would otherwise drop the key on save); no shipped skill and no `checks_golden` fixture has one. (2) A unit meter set by a `round_end` trigger row's `meter_set` this round skips that round's decay (per unit and meter; a `meter` delta, a meter rule's own set, or a set on another event marks nothing), so a crisis leaves Strain at 50 (PK1 2.1). (3) Seed `pr526b.json`: `vent` Strain -20 -> -50, `hp_cost_pct` 15 -> 10, description "Channel basic: -50 Strain for 10% of max HP; only while Strain is 50 or more." (`channel_power` stays 1.25). `sim feel 2 seeds=40 enc=pk1_channel_duel ruleset=timeline counts=on`: won 37.5% (15 of 40), median 5.5 rounds, Channel rounds 159, channeled actions 119, Vents 40 (1.0 a fight), crises 37 (0.93 a fight), bands Acute 76, Flesh-Tension 91, Aberrant Edge 40, Rupture 2; 0 plan fails. Tests: `rpgkit_content/tests/rank_modifier_keys.rs` (3, incl. every schema skill field following its rank flag), `pk1_kitfix.rs` (the crisis test updated: Strain ends the round at 50; new `only_a_round_end_set_skips_the_decay`). Gate 2964 passed, 0 failed, 4 ignored, 0 warnings; 0 golden diffs. Review: planner (small; acceptance and the sim confirmed).
- **Recommendation**: `combat_hooks.md` (UM3 decay): "decay skips a round in which a `round_end` trigger's `meter_set` set the meter (that unit, that meter); a raise by a `round_end` trigger still decays"; (skill ranks) "a rank modifier holds only rank fields and `description`; any other key is a check error and the writer refuses it". GAME_BRIEF (Channel numbers): "Vent: -50 Strain for 10% of max HP (Strain 50 or more, Channel only); a Rupture crisis sets Strain to 50 and it stays 50 through that round's end." `kits/rpg/README.md`: done. No HISTORY row.

### FB-209: PR5-9 `PK2-S2` landed (body slots, grafts, the floor table, the keystone graft, remains; `checks_golden` re-recorded)

- **Spec**: READY Section 1 row PR5-9; `docs/sessions/PK2_GRAFTS.md` section 3; `FB-167` Q6 (the `PR5-9` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-210: PR5-9 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 3 (@ the `PR5-9` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (1) the channeled-active limit is the graft count for a unit with a body, bounded by the deck's 3 actives (a deck without a deck rule could mark more); a member with 0 grafts in a catalog with body slots marks 0, and **once PR5-19 ships body slots, members' saved channeled marks beyond their graft count are cut at load with a warning** (spec-intended: "PK2 replaces it with one per installed graft"; the balance lane should know); (2) item type spelled `GRAFT` (the uppercase convention; the row says `graft`); (3) every member that dies with the field held leaves remains, grafted or not ("a fallen body yields one organ"); (4) "or a lost one where the fallen are recovered under the roster rules" is not built: a lost fight settles through the downed rules with no recovered body today (PK3 bleed-out / revive, PR5-11); (5) Tier III's +1 track step on install, the excision's max HP cost and all costs are PR5-10's recipes (install and remove are free state changes); (6) several class slots are allowed (each opens with the class rite); (7) a non-keystone graft's skill takes an active deck slot under the deck rule; (8) `floors` is allowed on any passive's meter effect, not only on `body_effects`; (9) **remains' type**: `remains` (class and character) accepts GRAFT items only, while PK2 speaks of a "core" (an Apex core, a class core); if a core is a material that a surgery turns into a graft, both filters should take that material instead.
- **Recommendation**: confirm (1)-(8) as built; for (9) name what `remains` holds (the graft itself, recommended by the built filter, or the surgery's core material) so PR5-19's data matches.

### FB-211: PR5-10 `PK2-S3` landed (recipes, the NPC `recipe` service and pane)

- **Spec**: READY Section 1 row PR5-10; `docs/sessions/PK2_GRAFTS.md` section 4 (the `PR5-10` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Full text**: moved for size (UN-04) to `docs/archive/FEEDBACK_CLOSED.md`, "Open items, full text".

### FB-212: PR5-10 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 4 (@ the `PR5-10` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Readings: pending effects are refused rather than no-ops (no data can author an effect that silently does nothing); target `item` added for salvage; item `theme` added; "tier" is the item's rarity and Scrap is the purse gold; the surgery's core is a cost item (nothing ties the installed graft to a cost item; data lists both); `doctrine` {axis, delta} applies nothing; the pane has no family picker, so a role holding several families (minor) cannot be trained from it yet (the kit refuses with "Pick a minor family (...)"); recipes whose `requires` fail are not shown.
- **Recommendation**: Questions, built default first: (a) a Tier III surgery (`track +1`) on a member one stage before departure is refused (alternative: allowed, the member departs at the next settle); bone-scraping (`track -1`) at stage 0 is refused; (b) `doctrine` records nothing until PK4 (alternative: a party key now); (c) training recipes whose role the member's class lacks are listed and refused with a reason (alternative: hidden); (d) a family picker in the pane before PR5-19 ships minor training (a small game row).

### FB-213: PR5-12 `PK2-S4` landed (weapon mastery and techniques; `checks_golden` re-recorded)

- **Spec**: READY Section 1 row PR5-12; `docs/sessions/PK2_GRAFTS.md` section 5 (the `PR5-12` squash commit on `develop`, 2026-11-11)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: `rpgkit_data/src/mastery.rs` (levels 1-3 at `MASTERY_LEVEL_XP` 100 / 300 / 600, `USE_XP` 10, rate x1 at proficiency 2 and x1.5 at 3, `MASTERABLE_MAX` 3, `Technique` {skill, grip}, `MasteryBook`); `rpgkit_world/src/mastery.rs` (`open`, `gain`, `settle`, `read_book`, `drill_family`). Manifest block `durability` {`from_act` 1-9} (a `MANIFEST_KEYS` key; none = mastery is never gained). Member save keys `mastery_xp` {gear type: XP} and `forge_rite` (written only when set). Sources: use (an attack skill started by a member-built hero records its main-hand family once per battle, `World::weapon_attacks`, `HeroEnd::attacked`; `USE_XP` per family to the members who came home, `Settled::mastery`, result key `mastery`, `PARTY_KEYS` 11), item type `MASTERY_BOOK` (`mastery_family`, `mastery_xp` 1-999, schema default 25), the recipe effect `drill` {xp} (out of `RECIPE_PENDING`). Item family keys `techniques` (0-3 {skill, grip ""/1h/2h}) and `keystone` (stored only when set; derived onto items as `family_techniques` / `family_keystone`, out of the stored item form); a weapon's own `technique` (masterwork, PR5-13a). Build: the main-hand family's first n techniques open at mastery n (`Unit::techniques`, rebuilt each sync), at mastery 3 with the forge rite the family keystone (`Unit::mastery_keystone`, set at build, takes a keystone slot; none while a keystone graft holds it); `Unit::mastery` None for a member with no mastery, so Act 1 units are unchanged. A technique is owned and open without a deck slot, standard rounds only (`channel::opens`: "Channel round"), greyed under the other grip (`Unit::technique_problem`). Checks: `technique_problems`, `book_problems`, `technique_entry_errors`, drill xp, class `masterable_warnings` (more than 3 masterable families: a warning). Tests: `rpgkit_world/tests/mastery.rs` (7), `rpgkit_battle/tests/techniques.rs` (4), `rpgkit_content/tests/mastery_checks.rs` (5). Gate 3020+ passed, 0 failed, 0 warnings. **`checks_golden` re-recorded**: the item-type list messages now end `..., GRAFT, MASTERY_BOOK.` (17 lines, no other change; the row said "0 golden diffs" and did not name it, same pattern as PR5-9's GRAFT); no other golden moved. Review: change-reviewer PASS (fixed: `steel_keystone` renamed `mastery_keystone`, a clippy closure, the keystone's build-time scope documented).
- **Recommendation**: HISTORY: "2026-11-11 PR5-12 (PK2-S4): `checks_golden` re-recorded; the item-type list messages name MASTERY_BOOK (17 lines, no other change); intended." STRUCTURE: the two `mastery.rs` files and the three test files. PORT_NOTES (save keys): member `mastery_xp`, `forge_rite`; result key `mastery`; manifest `durability`. GAME_BRIEF / `combat_hooks.md`: mastery, techniques, the family keystone, item type `MASTERY_BOOK`, recipe effect `drill`; where to look `mastery::settle`, `Unit::open_techniques`. **READY**: PR5-19's data needs `durability.from_act` in `game.json` (Act 2) for any mastery to be gained. `kits/rpg/README.md`: done.

### FB-214: PR5-12 readings and questions

- **Spec**: `docs/sessions/PK2_GRAFTS.md` section 5 (@ the `PR5-12` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Shipped data now gives one new content warning: `classes:tl_drill` (the timeline drill class, outside the PR5-18 lineup) has 4 masterable families (sword, mace, dagger, polearm); inert while mastery is closed, and trimming it risks the timeline and sim goldens, so no data was touched. Readings: the keystone opens at mastery 3 plus the forge rite while any weapon of the family is wielded (the spec says Bram forges it "into a masterwork weapon of that family"; the row says "mastery 3 plus the forge-rite flag"); nothing sets `forge_rite` yet (only `UnitState::set_forge_rite`); the keystone is fixed at build from the saved main hand while techniques follow a mid-battle swap; a technique skill the class also owns through its path or deck is not refused by a check (it would be standard-rounds-only and grip-greyed); no game GUI reads a book yet (`mastery::read_book` is kit-only).
- **Recommendation**: Questions, built default first: (a) `[PROVISIONAL]` kit constants not in the spec: XP 100 / 300 / 600, use XP 10, book default 25 (alternative: a ruleset or `tuning.json` field before balance); (b) `tl_drill`: accept the warning (alternatives: exempt timeline classes, or trim it in a data row with the golden re-record named); (c) keystone on any family weapon (alternative: only a masterwork, once PR5-13a lands); (d) the forge-rite source: a recipe effect `forge_rite` in PR5-13b or a quest flag in PR5-19; (e) a content check refusing a technique that is also a path or deck skill (a small kit row); (f) a book-reading action in the inventory GUI (a small game row).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
