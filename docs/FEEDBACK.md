# Build Status and Feedback (Build Session writes, Design Session reads)

This file is the **only doc file edited by the Build Session** under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Build Session** updates section 1 and appends items to section 2 (findings, conflicts, sim results, docs deltas).
- **Design Session** reads this file, applies docs deltas to shared docs, updates specs, and records answers in `docs/READY.md` section 4 (`Feedback Responses`). Once answered, Build marks the item `closed` on its next pass.

## 1. Build Status

| Task / Sub-block | Stage / Status | Branch / Commit | Notes |
| --- | --- | --- | --- |
| R1 to PR5-62c (`FB-01`..`FB-277`, `UN-01`..`UN-12`) | released or done | `main` / `develop` (`48f24342`) | Archived in `docs/archive/FEEDBACK_CLOSED.md` (moved 2026-11-16) |
| PR6-1 (`WAVE2-BALANCE`) | released (`29dfc241`) | `develop` (`29dfc241`) | `rpgkit_sim` per-class `enemy_mastery` (`line`: `warrior:1, rogue:1`); seed `pr6_1_balance.json`; `sweep 5 30` `brawler -2.6`, `mage -0.6`, `line 88%`, `boss 32%`; `party 5 30` `line 83%`, `boss 40%` |
| PR6-2 (`PK2-STAGE-PASSIVES`) | built (ready for verify) | `swarm-build-lead-71764ac0` | Kit `Target::Armor` / `Target::MoveRate` / `GROUPS`+`"ranged"` / `skill_potency.min_distance` (`0..=12`); seed `pr6_2_stage_passives.json` (`calcify_2`, `calcify_3`, `unit_tracks.json` stages 2-3); `0` golden diffs; `429` suites (`3,481` passed, `0` failed, `4` ignored), `0` warnings |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-277 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-16). New items start at FB-278. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-278 — PR6-1 (`WAVE2-BALANCE`) Tuning, `enemy_mastery` Schema, Mage AoE Ratification, and Golden Deltas

- **Spec**: `PK8_ACT23_CONTENT.md` (§3), `READY.md` (§1 `PR6-1`), `HISTORY.md`
- **Kind**: sim / docs-delta
- **Status**: open
- **Finding**: Completed `PR6-1` (`WAVE2-BALANCE`) via `games/emberwake/data/seeds/pr6_1_balance.json` (8 steps, 2-pass `0`-diff idempotent), `games/emberwake/sim.json`, and `kits/rpg/crates/rpgkit_sim`:
  - **Brawler (`brawler` `-9.4` -> `-2.6`, `top5_count: 3`)**: Raised `item_families["gauntlet"].base.main_1h.power` `13 -> 15`, added `adjust +1 mitigation` (`3 -> 4`) on `items["forearm_braces"]`, and raised `brawler_clinch` `0.70 -> 0.95` + `slag_charge` `1.35 -> 1.60` (`forearm_braces` keeps `item_families["braces"]` untouched because `braces.off_hand_category: "braces"` is pre-existing legacy). `@boss` `+40% hp_pct` in `rpgkit_sim::party` needed no change.
  - **`line` (`93%` -> `83%` `party 5 30` / `88%` `sweep 5 30`)**: Added optional `Generated.enemy_mastery` (`{class_id: level}`) in `rpgkit_sim` (`config.rs`, `scenarios.rs`, `runner.rs`, `party.rs`, `world.rs`, tests `classes.rs`, `config.rs`). Equipping all 4 `line` standard foes (`* : 1`) cratered `line` to `17%` (`brawler` `knuckle_jolt` + `ranger` `aimed_shot`); `warrior: 1` alone stayed at `97%`; **`{"warrior": 1, "rogue": 1}`** (`hook_drag` + `quick_stab`) placed `line` cleanly at **`83%`** (`party 5 30`) and **`88%`** (`sweep 5 30`).
  - **Mage AoE (`0.28/0.28/0.28/0.22` -> `0.32/0.30/0.32/0.25`)**: In `aoe_swarm` (`role 5 role=aoe class=mage`), `0.28/0.28/0.28/0.22` cleared in `6.86` rounds (`67%` clear, `63%` dmg share) vs `0.32/0.30/0.32/0.25` clearing in `5.89` rounds (`61%` clear, `64%` dmg share, `1.67` foes/cast); across `sim sweep 5 30` (`2,247` fights), `0.32/0.30/0.32/0.25` held **`mage` `swap_delta: -0.6`** (inside `[-3, +3]`, `top5_count: 3`) and **`boss: 32%`**, so `cinder_spark 0.32` / `firebomb 0.30` / `glacial_burst 0.32` / `choke_cloud 0.25` is ratified.
  - **Verification**: `sim sweep 5 30`: `mob 100%, line 88%, wave 60%, elite 78% X, boss 32%`, `warrior +1.6 (top5 3)`, `ranger +7.9 (top5 5)`, `priest -5.6 (top5 2)`, `brawler -2.6 (top5 3)`, `rogue -0.9 (top5 4)`, `mage -0.6 (top5 3)`, `187,869` checks / `2,247` fights (`0 METRIC FAIL`, `0 PLAN FAIL`, `3 PLAN SKIP`). `sim party 5 30`: `mob 100%, line 83%, wave 67%, elite 80% X, boss 40%`, `13,184` checks / `150` fights (`0 METRIC FAIL`, `0 PLAN FAIL`). `sim feel world=sunk_ward enc=enc_evt_06 seeds=10 counts=on`: `won 0% (0/9/0/1)`, `median rounds 6`, `0 PLAN FAIL`. `fc.sh content manifest && fc.sh content check` (`0` errors), `fc.sh layers` ok, `cargo check --workspace --tests` `0` warnings, `cargo test --workspace` `427` suites (`3,475 passed, 0 failed, 4 ignored`), `0` `.ogg` diff vs `b7fa30db`.
  - **Goldens**: Re-froze `timeline_kit.json` (which now pulls in both `PR5-62c` and `PR6-1` catalog edits) and re-recorded intentionally moved named goldens (`timeline_golden.json`, `checks_golden.json`, `rules_golden.json`, `sim_golden.json`, `phasefold_golden.json`); kept `sim_data.json`, `journey_golden.json` (`2/2`), and `rules_contract_golden.json` at **`0` diff**.
- **Recommendation**: Ratify `cinder_spark 0.32 / firebomb 0.30 / glacial_burst 0.32 / choke_cloud 0.25`, `brawler` (`gauntlet 15`, `forearm_braces +1 mit`, `brawler_clinch 0.95`, `slag_charge 1.60`), and `sim.json` `line.enemy_mastery: {"warrior": 1, "rogue": 1}`; record the 5 re-recorded goldens + `timeline_kit.json` in `HISTORY.md` (and note `elite` `78-80%` stays `3-5%` above `50-75%` band unless tuned separately).

### FB-279 — PR6-2 (`PK2-STAGE-PASSIVES`): Kit Rating Stats, `"ranged"` Group, `min_distance` Potency, and `calcify_2` / `calcify_3`

- **Spec**: `PK3_ROSTER_LIFECYCLE.md` (§1), `READY.md` (§1 `PR6-2`), `HISTORY.md`
- **Kind**: docs-delta / verification
- **Status**: open
- **Finding**: Built `PR6-2` (`PK2-STAGE-PASSIVES`) atop `29dfc241`:
  - **Kit (`kits/rpg/`, story-free)**: Added `Target::Armor` and `Target::MoveRate` in `rpgkit_rules::grants` (`stat: "armor"` / `"move_rate"`), wired into `damage::base_armor_rating(u)` (clamped `>= 0`) and `Unit::move_per_block(&self)` (`+ passive + gear + cond + hasted - slowed`, clamped `>= 1`, keeping `rating_set(MoveRate)` override), plus `CondGrants`/`BandApplied` lifecycle resets. Expanded `rpgkit_rules::unit::GROUPS` and `rpgkit_content::check::simple::GROUPS` with `"ranged"` (`skill_in_group(s, "ranged") => skill_is_ranged(s)`), accepted flat `armor` / `move_rate` in `simple::effect_errors`, and added optional whole `min_distance` (`0..=12`, default `0`) on `skill_potency` across `simple.rs`, `grants.rs`, `unit.rs` (`potency_for_skill_at` / `power_for_at` / `distance_potency_*`), `damage.rs` (`HitInput.distance` + `distance_potency` stage when `distance > 0`), and `rpgkit_battle` (`command::Command::land` + `plan::expected_from` / `expected_with` / `estimate_damage`). When `min_distance == 0` / `distance == 0`, evaluation is bit-identical to pre-`PR6-2`.
  - **Content (`games/emberwake/data/`)**: Applied `games/emberwake/data/seeds/pr6_2_stage_passives.json` (2-pass `0`-diff idempotent via `aegis content apply`) + `fc.sh content manifest`, authoring slotless `calcify_2` (*Calloused Chitin*: `armor +2`, `move_rate -1`) and `calcify_3` (*Sensory Rot*: `weaken` 100%, `stun` 100%, `skill_potency` `group: "ranged"`, `min_distance: 4`, `value: -0.15`) in `passives.json` and wiring them onto stages 2–3 of `unit_tracks.json["calcification"]`. Zero changes to `ashlands`, `iron_reaches`, `tidewrack`, `engine/`, `assets/`, `*.ogg`, `game.json`, or any golden file.
  - **Tests & Verification**: Added `kits/rpg/crates/rpgkit_rules/tests/stage_passives.rs` (`2/2`), `min_distance` + rating stat checks in `rpgkit_content::class_passives_checks` (`3/3`), battle forecast/hit `min_distance` test in `rpgkit_battle::unit_tracks` (`8/8`), updated oracle in `emberwake_rules::passive_grants` (`11/11`) / `hit_modifiers` (`2/2`), and added `calcification_stage_ii_and_iii_passives_apply_in_crucible` in `emberwake_world::techniques_grafts` (`5/5`). `cargo check --workspace --tests` (`0` warnings), `cargo test --workspace --no-fail-fast` (`429` suites, `3,481` passed, `0` failed, `4` pre-existing ignored), `fc.sh content check` (`data ok`), `fc.sh layers` (`ok`), `fc.sh docs` (`ok`).
- **Recommendation**: Mark `PR6-2` done in `READY.md`, note `calcify_2` / `calcify_3` + `stat: "armor"` / `"move_rate"` / `group: "ranged"` / `min_distance` in `PK3_ROSTER_LIFECYCLE.md` / `HISTORY.md` (and optionally refresh `kits/rpg/schema.json` passive `effects` help string if desired).

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
