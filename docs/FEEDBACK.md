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
| PR5-62b (`CLASSPASS1-DATA`) | built (ready for verify) | `swarm-build-lead-71764ac0` | Seed `classpass1_data.json` (`102` steps): 6 flat decks, 24 skills, 12 passives, `riveted_gauntlets`, `wpn_orb` `0.75` `stat_override`, `twin_light_tempo` `1h_light`, `[PROVISIONAL]` Mage AoE `0.28/0.28/0.28/0.22`; re-froze `timeline_kit.json` + re-recorded `timeline`/`rules`/`sim`/`phasefold`/`checks` goldens |
| PR5-62c (`TECHNIQUES-GRAFTS-DATA`) | built (ready for verify) | `swarm-build-lead-71764ac0` | Seed `techniques_grafts_data.json` (`213` steps): 15 weapon decks (45 skills + 15 Cold Steel keystones), 12 Class/Graft skills, `sg_*` 30 surgeries (`gp_*` 18 grafts), `calcified_*` enemies, 4 passives (`mastery_crossbow`/`mastery_bow` `+1 PERC`, `mastery_fist` `+1 STR`, `dirty_tricks` `+5% crit`), `hearth_rite`; approved `phasefold_golden.json` re-recorded; `0` frozen golden diffs; 4 `techniques_grafts.rs` world tests |

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

### FB-276 — `PR5-62b` (`CLASSPASS1-DATA`) Build Summary, Mage AoE / Dual-Wield Sims, and `HISTORY.md` Golden Delta

- **Spec**: `PR5-62b` (`docs/sessions/U2_CLASSPASS_TECHNIQUES.md`, `docs/sessions/U2_CLASS_DECKS.md`)
- **Kind**: docs-delta
- **Status**: open
- **Finding**:
  - Applied `games/emberwake/data/seeds/classpass1_data.json` (`102` idempotent steps from `6e662c53`): `riveted_gauntlets` (`assets/items/gauntlet/riveted_gauntlets.png` present), `wpn_orb` (`stat_override: 0.75` `KNOW`->`INT`), 6 flat 3-skill class decks, 24 class skills, 12 `CLASS` passives including `twin_light_tempo` (`dual_wield: "1h_light"`), retired player `_2`/`_3` skill tiers and `mara_voss` skill overrides, synced `games/emberwake/sim.json` (`l7_swaps: []`); shipped `rogue` keeps `off_hand: "ash_cloak"`.
  - `[PROVISIONAL]` Mage AoE scalars settled at `cinder_spark: 0.28`, `firebomb: 0.28` (`burn` mag `2`), `glacial_burst: 0.28`, `choke_cloud: 0.22` (`poison` mag `3`), `reactive_reagents` `expose` mag `3` `limit: "round"`:
    - `sim sweep 5 10`: `mob 100 (>=90)`, `line 95 X (70-90; enemy warrior awaits PR5-62c polearm attacks)`, `wave 61`, `elite 61`, `boss 31`; `swap_delta` `warrior -2.0, ranger +4.6, priest -0.2, brawler -8.6 (top5_count 1 X), rogue +4.0, mage +2.2` (`0 METRIC FAIL`, `0 PLAN FAIL`, `2 PLAN SKIP` / `748` fights).
    - `sim party 5 30`: `mob 100, line 87, wave 57, elite 70, boss 37` (`0` band fails, `0 METRIC FAIL`, `0 PLAN FAIL` / `150` fights); `sim party 5 10`: `mob 100, line 90, wave 50 X, elite 50, boss 40`; `sim campaign 5`: `0 METRIC FAIL`, `0 PLAN FAIL` / `236` fights; `sim feel world=sunk_ward enc=enc_evt_06 seeds=10 counts=on`: `won 0% (0/9/1)` Crucible intact, `0 PLAN FAIL`.
  - Rogue `dual_wield` parity check (`wpn_dagger` off-hand vs `ash_cloak`, restored cleanly): `sim duel 5 10` `avg 51/53` vs `48/53`; `sim party 5 10 comp=warrior,ranger,priest,rogue` `boss 50%` (`523` dealt vs `486`), `mob 100 / line 100 / wave 60 / elite 60`.
  - `HISTORY.md` delta & golden re-records: deleted & re-froze `timeline_kit.json`; re-recorded `timeline_golden`, `rules_golden`, `sim_golden`, `phasefold_golden` (`72/72` diverged on new/renamed class skills), and `checks_golden` (`skills 123/409`, `units 37/43`, `ashlands characters 22/26` vs re-frozen `timeline_kit.json`, cleared by coordinator). Also updated `(6, 4)` skill-menu sizing/flat-deck paths in `battle/skill_menu.rs`, `draw.rs`, `party.rs`, and `rpgkit_world::party_ops::deck.rs` (`0` warnings; `journey`, `rules_report`, `rules_contract`, `aegis_*`, `rpgkit_*`, `emberwake_*` all pass).
- **Recommendation**: Design Session: ratify settled Mage AoE / `burn`/`poison`/`reactive_reagents` scalars, record `PR5-62b` in `HISTORY.md` (naming the 5 re-recorded goldens + `timeline_kit.json`), archive answered items, and unlock `PR5-62c`.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.

### FB-277 — PR5-62c Build Summary, 4 Passive Conversions, `hearth_rite`, Sim Telemetry, and `phasefold_golden` Re-Record

- **Spec**: `U2_CLASSPASS_TECHNIQUES.md` (§2.1 `PR5-62c`, §2.2), `U2_WEAPON_TECHNIQUES.md`, `U2_GRAFTS_SKILLS.md`, `PK2_GRAFTS.md`, `HISTORY.md`
- **Kind**: sim / docs-delta
- **Status**: open
- **Finding**: Implemented and verified `PR5-62c` (`TECHNIQUES-GRAFTS-DATA`) atop `PR5-62b` (`techniques_grafts_data.json` at `213` steps with `0`-diff 2-pass idempotency; catalog at `220` skills, zero remaining `tq_*`):
  - **4 legacy `c02a` skill-grant passives converted**: Stripped free M1 skill `grant` effects. Because `check/item.rs:1386` rejects `"crit"` on `ITEM_TYPES` `level3_passive` slots (`mastery_crossbow`), `mastery_crossbow` uses `+1 PERC` (`"Sighted along the prod: +1 PERC."`); `mastery_bow` uses `+1 PERC` (`"Steady draw: +1 PERC."`); `mastery_fist` uses `+1 STR` (`"Hardened knuckles: +1 STR."`); and character temperament `dirty_tricks` uses `+5% crit` (`"Street-taught: +5% critical chance."`). Zero passive description warnings in `fc.sh content check`.
  - **`hearth_rite` + Wave 2 deferral**: Added `hearth_rite` (`forge_rite` on `berek_dorn`, requires `pk2.hearth_open` + `pk2.masterwork_quest`, cost `150` gold + `1 black_iron_ingot`, `+1 preservationist` doctrine) as step `213/213`. Act 2 drops/quest setters (`slot_*`, `black_iron_ingot`, `MASTERY_BOOK`, `pk2.apex_open`/`pk2.masterwork_quest`/`pk2.rite_*`) remain deferred to Wave 2 (`PK3..PK8`), with `0` Act 1 `sunk_ward` additions.
  - **Goldens & tests**: Re-recorded only approved `games/emberwake/crates/emberwake_world/tests/fixtures/phasefold_golden.json` (`RECORD=1`, `5/5` passing unrecorded afterward); `timeline_kit.json` and all frozen goldens (`timeline_golden`, `rules_golden`, `checks_golden`, `sim_golden`, `journey_golden`, `rules_contract_golden`) stay at `0` diff. Updated `ir21_intent.rs` (`220` skills, `40` duals, primary `[107, 43, 30, 15, 20, 5]`, listed `[137, 43, 40, 15, 20, 5]`) and `ruleset_switches.rs` (`calcified_husk`/`ash_crawler`/`calcified_amalgam`), and added 4 integration tests in `games/emberwake/crates/emberwake_world/tests/techniques_grafts.rs` covering all 15 weapon families (`Mastery 1/2/3`, `hearth_reforge`, `hearth_rite`), `sg_*` surgeries/grafts (`5/15/30/50`), Channel gating, keystone exclusion, `purge_marrow` Stage V calm rescue in `world:sunk_ward/enc_evt_06`, and legacy mastery passive non-leakage.
  - **Verification**: `fc.sh content manifest && fc.sh content check` ok (`0` errors); `fc.sh layers` ok; `fc.sh docs` ok; `cargo check --workspace --tests` `0` warnings; `cargo test --workspace` `427` suites (`3,475` passed, `0` failed, `4` ignored); `git diff --name-only a1b1efb1 -- "*.ogg"` empty; `sim feel world=sunk_ward enc=enc_evt_06 seeds=10 counts=on` `0 PLAN FAIL` (`ammo empowers 30`); `sim sweep 5 10` `mob 100%, line 92%, wave 60% (55-80), elite 65% (50-75), boss 34% (30-50)`, swap deltas `ranger +7.9, warrior +3.8, rogue +0.1, mage -1.0, priest -1.6, brawler -9.4`, `0 METRIC FAIL` (`66,430` checks / `747` fights), `0 PLAN FAIL` (`3 PLAN SKIP`); `sim party 5 10` (`warrior,ranger,priest,mage`) `mob 100%, line 100%, wave 70%, elite 70%, boss 20%`, `0 METRIC FAIL` (`4,746` checks / `50` fights), `0 PLAN FAIL`.
- **Recommendation**: Confirm scalar `+1 PERC` on `mastery_crossbow` (`level3_passive` constraint in `check/item.rs`), confirm `sim sweep 5 10` `boss 34%` vs `sim party 5 10` `boss 20%` and `brawler -9.4` are tracked for Wave 2 / balance tuning, and record `phasefold_golden.json` + `HISTORY.md` delta on close of `PR5-62c`.
