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
| W2-6 (R3-W2) | done (FB-142); FB-140 A in the landing, develop's gate green again | `develop` | R3 Wave 2 complete. Next: R3-CLOSE. |
| R3-CLOSE | released: `main` @8d863f58 ([#57](https://github.com/6lackwin9/tactical-rpg/pull/57), merge commit; tree = develop @e6450e22); audit FB-143, no blocker | `main` | - |
| R4-1 `U2.3-TIERS` | done: slice 1 (`aa311bf4`), slice 2 (`858eaab0`) | `develop` | - |
| R4-2 `R4-CLOSE` | done (FB-146); audit and balance clean | `develop` | Then the R4 release PR. |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-145 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-09). New items start at FB-146. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-146: R4-2 R4-CLOSE landed: audit fixes, verification tests, balance in band; golden record, spec gap and follow-ups

- **Spec**: `docs/READY.md` row R4-2 items (1)-(3); FB-136, FB-138, FB-143, FB-145; user rulings 2026-11-09 in the Build Session (below)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: R4-2 is on `develop`; full gate passed, `change-reviewer` clean, backend `audit-build` release-blocking: no.
  - Data (seed `r42`): L1 `brimstone_draught` description "... for 3 rounds."; L2 `brace` removed from `founder_kaelen.extra_skills` (listed once; it is the warrior class basic since R2-W2b; the `brawler` unit's 5th slot stays empty).
  - Kit: AUDIT-01 one missing-`tier` default `hero` (`rpgkit_data::DEFAULT_TIER` / `unit_tier`) for `UnitDef`, `stage_level`, `roster_at`, `WorldRoster::tier`, and also the loot soft cap (`check/reward.rs`) and the sim (`runner.rs`, `scenarios.rs`, `world.rs`), which spelled `standard`. AUDIT-02 `rpgkit_rules::unit::raise_level` auto-spends a stage-raised explicit-`spend` foe's extra pool. AUDIT-03 `roll_loop_warnings`. AUDIT-04 engine `aegis_content::store::manifest_key_problems(hooks, m)` shared by `set_manifest` and the kit's whole-folder check. AUDIT-05 `stat_kind` accepts a derived stat only if the world's ruleset keeps it (same lookup as `member_stat`). L3 `unit::basics_in_path_warnings` also on world `classes` overlay paths. M1 `sanitize_deck`: `save::decode` drops saved deck entries the data disallows, with a load warning (shape damage still refused; a still-invalid deck resets to the automatic deck).
  - AUDIT-06 (folded in from the R4-CLOSE audit; spec gap): AUDIT-03 as specced accepted an `unless` on the roll's own `key` as a guard, but a roll sets `key` false on a failure and an unset flag reads false, so the choice stays open after a failed roll and can be rerolled until it succeeds. Only an `unless` on a flag that a `set_flag` of the choice or node sets counts now, and not a `set_flag` of the roll's own key (the roll overwrites it); the warning reads "guard it with a set_flag the choice sets and an unless on that flag." Design: record this as the guard rule (or choose a runtime rule: a roll choice hides once picked).
  - M2: the frozen `timeline_kit.json` gains the CLASS-BASICS rows (class `basics` on 7 classes; `brawler.skill_path` and the `brawler` unit loadout drop `sweep_kick`; `sweep_kick` level 2 -> 1; 3 `cyclone_kick` rank rows lose `requires`; timeline ruleset basics drop `flank_cut`; skill rows `brace`, `disengage_step`, `tend_wounds`, `cinder_spark`); `sim_data.json` holds no decks, so it is unchanged. The class-unit fight is the existing `goblin_skirmish`, in `timeline_golden` but **not the default feel set** (user 2026-11-09: feel stays the 5 `tl_` fights; `goblin_skirmish` runs with `enc=`).
  - Verification: `rpgkit_world/tests/save_gates.rs` (autosave slot 0, manual slots 1..5, keyed act-start save: round-trip byte-identical, `pouch_packed` and the `GameRolls` `dialogue` stream continue the same draws); `emberwake_host/tests/rules_cli.rs` (`aegis rules` all rows ok on shipped data; `--contract` matches the contract golden); `emberwake_battle/tests/step_events.rs` (the step stream over `tl_skirmish` and `goblin_skirmish`) and an `emberwake_gui` FX delivery test (`fx::act_kind` now shared with `popped_impact`). No bugs found.
  - Goldens re-recorded (intended, all from M2 because `timeline_kit.json` is the shared catalog): `timeline_golden` (+3 `goblin_skirmish` fights; the 15 `tl_` fights byte-identical); `rules_golden` (brawler rows only: goblin_brawler, goblin_berserker, enemy_brawler, ambush_berserker, brawler, `class:brawler:*`; R4-2's acceptance did not name it); `sim_golden` (every run that fights a brawler moved, so rows move across classes: party line 4.5 -> 5.0 rounds, boss clear 20% -> 10%, role deltas for ranger/priest/rogue/mage, steel-gate campaign 0/1 -> 1/1, one priest KPI row 100 -> 0; a brawler-only variant kit reproduces it exactly). Not moved: `phasefold_golden`, `checks_golden`, journey, `rules_contract_golden`, `rng_hash`, store resave.
  - R4 balance (on `claude/r4-2`): feel 61% (5 `tl_` fights, 40 seeds; windup 17.5%, armour 0% as intended losses; reach 100, flank 100, skirmish 87.5); party (12 seeds) mob 100, line 92, wave 100, elite 92, boss 92 (party HP after the boss 0.31); campaign arms 3/6, gate 5/6 (party HP after the boss 21% / 57%). All inside the FB-145 bands; no regression vs R4-1. `goblin_skirmish` alone: 17.5% won, 20 of 40 capped, 5 fled: a stalemate from goblin kite/flee against the round cap, not a hard fight.
- **Recommendation**: Design applies the docs delta and rules on the follow-ups:
  - Docs delta: engine README `aegis_content::store::manifest_key_problems`; kit README `DEFAULT_TIER` / `unit_tier`, `raise_level`, `sanitize_deck`, the roll-loop guard rule; GAME_BRIEF checks (roll-loop warning and guard, derived stat must be in the world's ruleset, overlay class-basic warning, unregistered game.json key is a problem; saved deck entries dropped on load); STRUCTURE / kit README save gates (C27) and step stream (C28) tests; HISTORY: "R4-2: tierless rows read `hero` everywhere (loot cap and sim were `standard`); saved deck entries the data disallows are dropped with a warning; roll-loop guard = set_flag + unless (AUDIT-06); frozen `timeline_kit.json` gains CLASS-BASICS rows; re-recorded `timeline_golden`, `rules_golden`, `sim_golden` (brawler runs)".
  - Follow-ups from the audit (none hold R4): AUDIT-07 (low) the sim's `elevel=` copy path (`rpgkit_sim/src/runner.rs:233`) skips `raise_level`, so the 5 explicit-`spend` rows leave points unallocated when shifted; bundle with `R3-STAGE-SEED`. AUDIT-08 (low) M1 covers `deck` only: a stale loadout family, `loadout_ranks` entry or `can_wear` failure still refuses the save (`state.rs:764`). AUDIT-09 (low, design call) `goblin_skirmish` fires only Sweep Kick of the class basics and caps at round 6/8; queue a scripted or mage-including class fight if coverage matters, and look at its cap rate. AUDIT-10 (low) the FX test builds `Act` with fixed `magic/on_self/harms` instead of `popped_impact`'s computation; share one constructor.
  - Big picture: the frozen kit is now a hand-curated fork (patched in W2-5, R4-1, R4-2); either name the whole kit-consumer golden set whenever a row edits `timeline_kit.json`, or re-freeze it from shipped data once after R4. Report the two always-lose duels apart from feel overall. The 5 explicit-`spend` rows were authored on class shares and now sit on template bases: re-author or drop the spend. `tier_hp` / `tier_power` aliases and item `buff_stat` (parsed, never read; brimstone says `CONS` while its effect is Empower) are dead weight. A `derived_stats` formula using a stat and `tier_mult` would scale a templated unit twice: add a check before the A/I silos author formulas. Branch pushes get no CI; consider CI on `claude/*` or a `Gate:` trailer rule on golden-moving commits.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-11 are closed and moved to the archive (2026-11-08). New notes start at UN-12.
