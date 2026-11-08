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
| R2-W2b CLASS-BASICS | done: logic half (FB-132), Maneuver AI row (FB-134), data half (FB-135); open FB-136 | `develop` | - |
| R2-CLOSE | balance pass done (FB-137), audit done (FB-138); release PR held for FB-136 (1) and FB-138 | `develop` | Gate + release PR once Design rules. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-134 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-08). New items start at FB-135. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-135: R2-W2b CLASS-BASICS data half landed: class basics in data, phasefold re-recorded

- **Spec**: `docs/READY.md` row R2-W2b (CLASS-BASICS; FB-123 A, FB-125 (1), FB-130 A, FB-133 answer)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Seed `games/emberwake/data/seeds/classbasics.json` (19 steps, applied through the writer, idempotent): skills `disengage_step`, `tend_wounds`, `cinder_spark` as READY gives them (the writer drops the schema defaults `target_mode` ALLY_OR_SELF and `reach_min` 0); `sweep_kick` level 2 -> 1; `cyclone_kick` loses `requires: ["sweep_kick"]`; class `basics` warrior `brace`, brawler `sweep_kick`, rogue `flank_cut`, ranger `disengage_step`, priest `tend_wounds`, mage `cinder_spark`, `tl_drill` `flank_cut`; `brace` leaves `warrior.skill_path` and `sweep_kick` leaves `brawler.skill_path`; deck orders: `founder_hollis` `maneuver` -> `["disengage_step"]`, `founder_dunstan`, `founder_kaelen`, `founder_maren` drop `maneuver`; timeline ruleset basics drop `flank_cut` (now the rogue's class basic): `move`, `rest`, `strike`, `offhand_jab`, `active_guard`; `brimstone_draught` text "+30% attack power (Empower) for 2 rounds" (FB-125 (1), verbatim). The writer refused the brawler class upsert while the `brawler` unit still listed `sweep_kick` in its loadout (a class basic cannot take a loadout slot), so the seed also drops it from that unit (see FB-136 (2)). Goldens: `phasefold_golden` re-recorded (named), 61 of 72 fights moved: 13 are `ashmaw_stand` (Grakka gets `sunder` back, FB-123 A), 48 are `goblin_ambush` / `pass_ambush` (the classic Enemy Brawler no longer has Sweep Kick: class basics enter only through the deck rule, which classic does not play). `checks_golden` and `sim_golden` were named but did not move: both read frozen fixtures, not shipped data (see FB-136 (4)). `timeline_golden`, `rules_golden`, contract, journey and resave unchanged. Feel (40 seeds, timeline) unchanged: 65 / 62.5 / 65 / 100 / 87.5, overall 76 %, 0 PLAN FAIL. Tests updated: `ir21_intent.rs` (133 skills, 28 dual = 9 + 19, primary 59/24/17/13/15/5), `ruleset_switches.rs` (replays the seed), `unit_builder.rs` (oracle adds class basics), GUI `input1_tests.rs` (helper `open_a_positioned_basic`).
- **Recommendation**: GAME_BRIEF / kit README: each class has one class basic (list above), outside its skill path and its deck slots; timeline basics now `move`, `rest`, `strike`, `offhand_jab`, `active_guard`; STRUCTURE: seed `classbasics.json`; HISTORY: "R2-W2b CLASS-BASICS data (FB-133): three class-basic skills, class `basics` per class, `brace` and `sweep_kick` off their skill paths, `flank_cut` from timeline basics to the rogue's class basic; `phasefold_golden` re-recorded (61/72: Grakka's `sunder` back, the classic Enemy Brawler without Sweep Kick)".

### FB-136: CLASS-BASICS follow-ups: classic loses class basics, a freed loadout slot, frozen fixtures

- **Spec**: `docs/READY.md` row R2-W2b (@ FB-135); FB-130 (expected `sim_golden` to exercise `sweep_kick` / `cinder_spark`)
- **Kind**: suggestion
- **Status**: open
- **Finding**: (1) In classic (no `deck` rule) a class basic is reachable only through the skill path or a loadout; `brace` and `sweep_kick` left both, so classic warriors and brawlers lose them (this is what moved 48 phasefold fights). (2) The `brawler` unit's loadout is now `body_blow`, `lunge`, `hamstring`, `haymaker` (one slot freed, left empty). (3) Brimstone's text is FB-125's string verbatim, so it loses the old "Brimstone Tallow Draught:" prefix and full stop the other item texts carry. (4) `checks_golden` and `sim_golden` read frozen fixtures (`sim_data.json`, the checks fixtures), so a shipped-data change never reaches them; the `tl_` decks there still lack the class basics FB-130 expected them to exercise. (5) The `aegis_cli` scenario test failed once under parallel load and passed on re-run (no code cause found).
- **Recommendation**: (1) A (recommended): accept; classic is the legacy ruleset and the timeline ruleset is the target. B: give classic builds the class basics too (one line in the deck-rule fallback; moves phasefold again). (2) Leave empty, or name a replacement skill for the slot. (3) Confirm the text, or give one with the prefix. (4) A follow-up row that adds the class basics to the frozen `sim_data.json` / `timeline_kit.json` decks and re-records `sim_golden` / `timeline_golden`, so the goldens cover RING, Field and Maneuver class basics. (5) Build adds a known-issues row ("aegis_cli scenario test flakes under parallel load: re-run alone").

### FB-137: R2-CLOSE balance pass: baseline inside the bands; `know_potency` 1.0 and `guard_cap` 0.75 move nothing

- **Spec**: `docs/READY.md` row R2-CLOSE (FB-76 / FB-98 `know_potency`; C06 / FB-06 / FB-56 `guard_cap`) (@4ed71e44)
- **Kind**: sim
- **Status**: open
- **Finding**: `tools/fc.sh sim feel 5 ruleset=timeline seeds=40 counts=on`, auto driver, develop @4ed71e44. Baseline: `tl_duel_windup` 65 % (26/14/0, median 5 rounds), `tl_duel_reach` 62.5 % (25/15/0), `tl_duel_armour` 65 % (26/14/0), `tl_flank` 100 %, `tl_skirmish` 87.5 % (35/2/3), overall 76 %, 0 PLAN FAIL / SKIP: inside the bands (duels 50-75 %, overall >= 75 %, 1 point of margin). `know_potency` 1.0 (and 3.0, the range top) in the timeline row: byte-identical; it is read only by `status::know_bonus` on a skill's status roll, and no `tl_` fight seems to roll one. `guard_cap` 0.75: identical; it is the code constant `GUARD_CAP` (`rpgkit_rules/src/damage.rs:25`), not a ruleset field, and no guard plus cover in these fights passes 0.75 (highest authored guard `mitigation` 0.55). Both variants were throwaway edits, not committed. Notes: the party uses no items (0); heavy party Rest in `tl_skirmish` (5.68 blocks a round against the enemy's 1.4; its 3 capped fights are the party's); 29 party no-active rounds in `tl_duel_armour`; the feel fights field only `tl_drill` and `tl_` ranger units, so they cannot measure the class basics (FB-138 M2).
- **Recommendation**: keep `know_potency` 0.0 (re-evaluate with a status-heavy timeline fight) and `guard_cap` 0.90 (changing it moves `item_values.rs` and `rules_contract_golden` for no feel gain). BALANCE_REPORT: a R2-CLOSE section with the table above.

### FB-138: R2-CLOSE backend audit: one ruling holds the release; save, coverage and text follow-ups

- **Spec**: `docs/READY.md` row R2-CLOSE (backend `audit-build` over `origin/main..origin/develop`, 74 commits, @4ed71e44)
- **Kind**: limit-blocker
- **Status**: open
- **Finding**: Authority Matrix clean (every golden re-record named in READY or an FB answer; no Design-owned doc edited; data through seeds except 249a8b02's two `mitigation` keys; the six new row-only ruleset keys follow the accepted Row 17 pattern); no unlogged behaviour change. Findings, verified against the code: **H1** = FB-136 (1): class basics enter only through the deck rule (`rpgkit_rules/src/unit.rs:156`; the classic `(_, None)` arm at :1155 adds none), and only `sunk_ward` plays timeline, so in ashlands, tidewrack and iron_reaches `goblin_brawler`, `goblin_berserker` and `ambush_berserker` lose Sweep Kick (the 48 phasefold fights READY did not name). **M1** an older v13 save whose edited deck holds `order.maneuver: ["flank_cut"]` (copied from the founders' auto order, `emberwake_gui/src/party.rs:426`), or a brawler with `sweep_kick` active, now fails `save::decode` -> `UnitState::deck_errors` (`rpgkit_world/src/state.rs:585`, :639); `VERSION` stays 13, no migration (v13 is unreleased). **M2** R2-CLOSE's feel acceptance is empty for the new content: the feel fights and the frozen-fixture goldens (`sim_golden`, `checks_golden`, `timeline_golden`) hold no class basic, RING, `cinder_spark` Field/Assault or `ROW_GAP_CLOSE` user. **L1** `brimstone_draught` reads "for 2 rounds" but its row has `duration: 3` and the battle log says 3 (`rpgkit_battle/src/command.rs:797`). **L2** `founder_kaelen` keeps `brace` in `extra_skills`, so in classic Kaelen is the only warrior with Brace; the writer refuses a class basic in a loadout but allows it in extras. **L3** the "class basic also in the skill path" warning reads base class rows only, not world-overlay `skill_path`s (latent: no overlay sets one). Big picture: timeline-only features (class basics, Field rows, gap-close) thin classic in 3 of 4 worlds; phasefold (classic ashlands) is the only golden on shipped data; save validity depends on data; the ruleset field table is split between `schema.json` (29 keys) and `ruleset.rs` (22 more).
- **Recommendation**: Build holds the `develop -> main` release PR (the full gate is green) until Design rules on: (1) H1 / FB-136 (1): A accept classic losing class basics (HISTORY row; recommended if classic is retiring), or B give classic units their class basics as known skills (re-record phasefold). (2) M2: release R2 now with the coverage gap logged (recommended), or first add the class basics to the frozen `tl_` decks plus one class-unit timeline feel fight, naming `sim_golden` / `timeline_golden`. Follow-up rows (any order, none blocking R2): M1 deck sanitize on load (drop entries the data no longer allows, warn, fixture-save test), size S; L1 one seed step (text "3 rounds", or `duration` 2); L2 drop `brace` from Kaelen's extras or add a check warning; L3 read overlay paths in the warning; Design's call on classic's future and one ruleset field table.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
