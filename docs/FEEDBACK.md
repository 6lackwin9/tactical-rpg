# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| R1 to PR5-56 (`FB-01`..`FB-273`, `UN-01`..`UN-12`) | released or done | `main` / `develop` | Archived in `docs/archive/FEEDBACK_CLOSED.md` (moved 2026-11-15) |
| PR5-57 (`GEAR-REWORK1-FOLLOWUP`) | built (ready for verify) | `swarm-build-lead-71764ac0` | Retired `untrained_*`, enemy drills + strict `wear_lock`, `catalyst_pulse`, `counts=on` gear counters, `FB-268` sim balance, 5 named goldens re-recorded |
| PR5-62a (`CLASSPASS1-KIT`) | built (ready for verify) | `swarm-build-lead-71764ac0` | `ClassDef::passives`, `when.dual_wield` + conditional grants, `"crit"` effect, `"channel"` group, `SkillReach::ranged` + `always_ranged`, `STAT_OVERRIDE_FACTOR=0.75`, `duplicate_skill_errors` (`basics` + `off_hand_skill`); 1-line `checks_golden.json` update (`crit`) |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-273 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-15). New items start at FB-274. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-274 — PR5-57 Build Summary, Sim Telemetry (`FB-268`), and `HISTORY.md` Golden Delta

- **Spec**: `PR5-57` (`docs/sessions/U3_GEAR_REWORK.md`, `FB-268`, `FB-269`)
- **Kind**: docs-delta
- **Status**: open
- **Finding**:
  - Implementation landed: retired `untrained_*` ruleset switches; added unit-level drill resolution + unit passives on the 5 catalog enemies (`calcified_husk`, `overseer_grendel`, `guild_master_orlov`, `mortuary_flagellant`, `proctor_quell`); enforced strict `wear_lock` and content wear errors; kept `shield`/`buckler`/`braces` provisionally at 0 extra skills until `CLASSPASS1`; authored `catalyst_pulse` on `wpn_orb` (`power_mult: 0.6`, `stagger: 8`, `expose` potency `80` / mag `4` / `2` rounds) and verified `sprayer` `spell_source`; updated `strict_loadouts.rs` (5 pinned warrior NPCs, `aberrant_test`, 4 Dojo presets); added `counts=on` counters (`ammo_empower`, `reload`, `knife_throw`, `orb_override`).
  - `FB-268` tuning + Crucible & `orb_override` telemetry: dropped `skill_potency: 0.1` from `wpn_orb`, added `Def::tuning` (`rpgkit_sim/src/runner.rs`) with `+40%` `hp_pct` on generated `sim party` `@boss` (`rpgkit_sim/src/party.rs`), and adjusted `games/emberwake/sim.json` generated `wave` (arrival `brawler`/`rogue` promoted to `standard`) & `elite` (`+ranger minion`). Observed post-edit results:
    - `sim party 5 10`: `mob 100 (>=90)`, `line 80 (70-90)`, `wave 60 (55-80)`, `elite 60 (50-75)`, `boss 40 (30-50)` (`0 METRIC FAIL`, `0 PLAN FAIL`).
    - `sim sweep 5 10` (`15` 4-class comps): `mob 100 (>=90)`, `line 89 (70-90)`, `wave 56 (55-80)`, `elite 53 (50-75)`, `boss 47 (30-50)` (`0 METRIC FAIL`, `0 PLAN FAIL`).
    - `sim campaign 5`: `warrior,priest,mage` survived `arms 2/3` (`mean party hp% 52%`), `gate 3/3` (`64%`), `0 METRIC FAIL`, `0 PLAN FAIL`.
    - `sim feel world=sunk_ward enc=enc_evt_06 seeds=10 counts=on` (Crucible): unchanged (`won 0%` — `0` won, `5` lost, `3` capped, `2` fled; `median_rounds 6.5`; `ammo_empower 27 (2.7/fight)`, `reload 0`, `knife_throw 0`, `orb_override 0` since Crucible heroes pair `wpn_orb` only with `INT` weapons; `feel_counts` `15 passed`).
  - `HISTORY.md` delta for re-recorded fixtures (`timeline_kit.json` and `sim_data.json` untouched):
    - `games/emberwake/crates/emberwake_rules/tests/fixtures/rules_golden.json`: `tl_captain` (frozen `timeline_kit` has no live `*_drill` passives + retired `untrained_*`) and strict `wear_lock` / `trained_fists` updates on the 5 catalog enemies (`calcified_husk`, `overseer_grendel`, `guild_master_orlov`, `mortuary_flagellant`, `proctor_quell`).
    - `kits/rpg/crates/rpgkit_rules/tests/fixtures/rules_contract_golden.json`: `-4` lines removing retired `untrained_*` ruleset keys.
    - `games/emberwake/crates/emberwake_battle/tests/fixtures/timeline_golden.json`: `3/18` fights (`tl_duel_armour`) shifted from updated `tl_captain` stats under `PR5-57`.
    - `games/emberwake/crates/emberwake_content/tests/fixtures/checks_golden.json`: `wear_lock` errors added on frozen `dojo_warrior` (`Duelist Buckler`, `Leather Brigandine`) and `mara_voss` (`Ember-Scale Brigandine`) fixture cases.
    - `games/emberwake/crates/emberwake_sim/tests/fixtures/sim_golden.json`: run 10 (`wave` & `elite` roster tuning) and run 12 (`elite`) updated to match tuned `sim.json` + `PR5-57` rules.
    - `games/emberwake/crates/emberwake_world/tests/fixtures/phasefold_golden.json`: `mara_voss` `+1` HP (`305 -> 306`) from lowering `arm_leather` `DEX` requirement (`10 -> 9`).
- **Recommendation**: Design Session: note `PR5-57` sim targets green across `sim party`/`sim sweep`, verify `HISTORY.md` golden re-recording note from the deltas above, and close `FB-274`.

### FB-275 — PR5-62a (`CLASSPASS1-KIT`) Build Summary & Kit/Golden Deltas

- **Spec**: `PR5-62a` (`docs/sessions/U2_CLASSPASS_TECHNIQUES.md`, `docs/READY.md` §1)
- **Kind**: docs-delta
- **Status**: open
- **Finding**:
  - Landed all 7 story-free kit hooks (`rpgkit_data`, `rpgkit_rules`, `rpgkit_content`, `rpgkit_battle`, `kits/rpg/schema.json`) with `games/emberwake/data/` untouched (`tools/fc.sh layers` clean):
    1. `ClassDef::passives` (`omit_default` in `schema.json`; `class_passives_errors` requiring `slot == ""`; applied before deck passives on derived units and `Growth` members under `DeckRule`).
    2. `when.dual_wield` (`Unit::has_dual_wield`, `trigger::matches`, `hooks::when_errors`) + conditional passive/gear grants (`Unit::cond` / `refresh_conditional_grants` synced in `rebuild_gear_effects` and `!gear` `apply_effects`, covering `Crit`, stats, potencies, `SkillReach`, resists, `StaggerTaken`, `Unlock`, `SkillBlock`, `ForcedTraits`, `ForcedTileParts`, `Skill`).
    3. `"crit"` passive/effect type (`simple::TYPES`, `Unit::passive_crit` + `cond.crit` summed in `damage::crit_chance`).
    4. `"channel"` skill group (`simple::GROUPS`, `skill_in_group` matching `s.channel.only`).
    5. `SkillReach::ranged: Option<bool>` (`SKILL_REACH_KEYS`), derived `SkillDef::always_ranged` (from `ALWAYS_RANGED_RULE` in `source::mark_spell_sources`, stripped on save in `store.rs`), and `skill_is_ranged` (ignoring default `s.area == "NONE"` so positioned single-target dash attacks are not classified as ranged).
    6. `STAT_OVERRIDE_FACTOR = 0.75` gating off-hand `power_stat_override` and scaling overridden `unit_power` in `damage::attack_power`.
    7. `check/unit::duplicate_skill_errors` (lives in `kits/rpg/crates/rpgkit_content/src/check/unit.rs`, not a separate `check/class.rs`) extended across `class.basics` and reachable `off_hand_skill` (`proficiency >= 1` + `class.loadout.get("off_hand")` item family) vs `class.skill_path`, `extra_skills`, and `masterable_families`.
  - **Approved Test/Golden Deltas (`#73`)**:
    - `games/emberwake/crates/emberwake_content/tests/fixtures/checks_golden.json` (line `671`, `passives sword_training`): unknown effect type message now appends `, crit` from `simple::TYPES`. All other goldens (`rules_contract_golden`, `rules_golden`, `timeline_golden`, `sim_golden`, `phasefold_golden`, `journey_golden`) have `0` diffs.
    - `games/emberwake/crates/emberwake_content/tests/ruleset_switches.rs` (`245-256`): filters legacy `"brace"` from `founder_kaelen.extra_skills` during the historical `CLASS-BASICS` replay step right before `r42.json` drops it (same pattern as `GEAR-REWORK1` class patching).
    - `kits/rpg/crates/rpgkit_content/tests/class_basics_checks.rs` (`87`): `drill` uses non-colliding `"skill_path": {"flank_cut": 1}` so line `92` continues testing higher-tier validation in isolation.
- **Recommendation**: Design Session: record the 1-line `checks_golden.json` delta (`crit` added to `simple::TYPES`), note `check/unit.rs` module placement (if referenced in docs/structure), approve `PR5-62a` handoff, and unlock `PR5-62b` (`CLASSPASS1-DATA`).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
