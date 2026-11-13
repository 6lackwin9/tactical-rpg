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
| PR5-36 to PR5-42 | done (FB-235 to FB-248, answered @6edec2a1 and @08ccf4f5) | `develop` | full rows in `docs/archive/FEEDBACK_CLOSED.md`; follow-ups queued as PR5-43, PR5-44 |
| PR5-43 `ACT1-HOOKS2` | done (FB-249; FB-250 readings and the Crucible finding): flag skill grants, `install`, `in_active_party`, meter `when` / `else`, `skill_block_cost`, `meter_delta`, `armor_piercing`, turn-in tick and `location` on any, T12 / T13 probes; 0 golden diffs | `develop` | - |
| PR5-44 `ACT1-DATA2` | next | - | - |

## 2. Feedback Items (Append-Only)

Entry fields: `Spec`, `Kind` (feasibility, conflict, cost, sim, docs-delta), `Status` (open, answered, closed), `Finding`, `Recommendation`.

FB-01 to FB-248 are closed and moved to `docs/archive/FEEDBACK_CLOSED.md` (2026-11-13). New items start at FB-249. Archived items record `closed: <disposition> (@<commit>)` (UN-05).

### FB-249: PR5-43 `ACT1-HOOKS2` landed

- **Spec**: READY Section 1 row PR5-43; FB-243 (b-d), FB-244 (1, 2, 4-6), FB-246 (a), FB-248 (b-c) (the `PR5-43` squash commit on `develop`, 2026-11-13)
- **Kind**: docs-delta
- **Status**: open
- **Finding**: Kit hooks, all story-free (layers G3 clean): (1) world encounter `flag_skills` [{`member_from_flag`, `skill`, `requires_flags`}] (battle-only grant, `Unit::battle_skills`), effect `install` {`member_from_flag`, `item`, `slot`} (recipe install path, idempotent), dialogue condition `in_active_party` {`character`}; (2) meter-rule effects `when` + `else` (if / else-if / else chains, `campaign::apply_rule_effects`), floored `add_flag` through its existing `min`; (3) effect `skill_block_cost` {`skill`, `delta` -3..3} (summed, at least 1 block, `Unit::skill_blocks` / `World::blocks_of` everywhere a unit's blocks are read), consumable `meter_delta` {`meter`, `delta`} and `armor_piercing`; (4) `survivors` flee warning, reforge refusal "... Use a plain reforge or a repair to restore it.", `classes_errors` re-checks world units' `extra_skills`, `confirm_turn_in` always ticks the campaign, `location` on `on: any` triggers; (5) `rpgkit_sim::world::encounter` carries `stay_behind` (sim party / sweep / campaign), game-layer T12 / T13 probes. Tests: kit `rule_effect_gates`, `graft_install_effect`, `flag_skills`, `battle_skill_grants`, `rule_effect_gate_checks`, `flag_skill_checks`, `install_effect_checks`, `gear_consumable_hooks`, `gear_consumable_hook_checks`, `turn_in_tick` (new), plus extended `survivors_checks`, `duplicate_skills`, `campaign_schema`, `weapon_recipes`; game `emberwake_world/tests/cast1_probes.rs` (T12 smoke placed by `ash_cloud_fade` on 0b; T13 a footman springs a `tripwire_snare` on 0c), `emberwake_sim/tests/world_feel.rs` (stay-behind in every sim path). Edited existing tests only for intended text / struct changes (the unknown-effect list now names `install`; `ItemHit.armor_piercing`; the reforge text). Gate 3221 passed, 0 failed; 0 golden diffs; shipped data unchanged and clean (the new warnings do not fire on it).
- **Recommendation**: GAME_BRIEF "Where to look": the hooks above (kit `battle.rs` `flag_skills`, `dialogue.rs` `install` / `in_active_party`, `campaign/mod.rs` `apply_rule_effects`, `unit.rs` `skill_blocks`, `battle/meter.rs` `from_item`, `damage.rs` `armor_piercing`, `trigger.rs` `at_location`); "Turn-ins tick the campaign; an `on: any` trigger with `location` fires on any tick while the party is there"; "CAST1 T12 / T13 probes: `emberwake_world/tests/cast1_probes.rs`". STRUCTURE: the new test files. HISTORY: "ACT1-HOOKS2: a turn-in ran no campaign tick unless effects followed it, and `location` was enter-only, so the Interlude waited for a hub re-entry (FB-246 a)." Known issues (cloud): never unset `CARGO_TARGET_DIR` (`.cargo/config.toml` holds a Windows target path; unsetting it creates a stray `C:/` folder).

### FB-250: PR5-43 readings and the Crucible finding

- **Spec**: READY row PR5-43 (1), (3), (5); `PK2_GRAFTS.md` 3; FB-248 (c) (@ the `PR5-43` squash commit)
- **Kind**: suggestion
- **Status**: open
- **Finding**: Readings built (the specs are silent): (a) "0 Act 1 Strain": a unit without the Strain meter uses a granted `channel.only` skill in Standard rounds as authored and pays nothing, and never becomes a Strain carrier (PK2_GRAFTS 3: a carrier only from the Act 2 graft); a unit that has the meter keeps the normal channel rule for it. (b) The dose record: `dlg_int_dose` stores the founder's id in `sunk_ward.sabina_dose_to` (`sunk_ward.sabina_dose` is a bool), so PR5-44 keys `flag_skills` and `install` on `sabina_dose_to`; one entry per founder (gated on the flag's value) if each founder gets a different skill. (c) `meter_delta` acts on an area-less use only (an area-thrown consumable ignores it, and the content check does not warn yet; the command line prints the unclamped delta before the `[Meter]` line shows the clamped one); it moves the target's meter by the full delta (not capped by the meter's `delta_max`, so -25 stays -25); `skill_block_cost` deltas are limited to -3..3; `armor_piercing` on a non-damage item is an error. (d) **Crucible**: FB-248's "the autoplay has no escape / winch logic" was a misreading: `sim feel world:` already carried `stay_behind` and ran `ai::hold_choice` (the `world.rs` fix only helps `sim party`, `sweep` and `campaign`), so the 10-seed `enc_evt_06` result is unchanged (0 won, 9 lost, 1 fled). The cause, from `aegis battle` seeds 1-10: Inquisitor-Prelate Nemeth kills the winch holder (Kaelen, 242 HP) in round 1 in 10 of 10 seeds (273, or 409 on a crit); with `objective_count` 3 and four founders, one death makes three escapers plus a holder impossible.
- **Recommendation**: (a)-(c) confirm or adjust. (d) Options for PR5-44's Crucible pass: (1) lower Nemeth's round-1 damage or keep him off the winch path; (2) a sturdier default holder (`pick_default`, or a holder with more HP); (3) `objective_count` 2, or a count that drops when a founder falls; (4) deploy the holder away from Nemeth. Recommended: (4), then (2) (data-only, and they keep Nemeth as tuned for human play per the FB-248 (c) ruling not to nerf bosses for autoplay); (1) only if the user wants it. PR5-44 re-sweeps the Crucible either way; until Design picks, PR5-44 records the unchanged result rather than tuning it.

## 3. Notes for the user (non-build, read by the Design Session)

The user may not read the build session's chat, so anything outside build work goes here, numbered `UN-nn`: questions or permissions for the user, things that need the user's eyes, process suggestions, and state the user should know. The Design Session relays them; the user's answers come back through `docs/READY.md` section 4 like any other. Fields: `For` (user, design, both), `Status` (open, answered, closed), `Note`, `Ask`.

UN-01 to UN-12 are closed and moved to the archive (2026-11-11). New notes start at UN-13.
