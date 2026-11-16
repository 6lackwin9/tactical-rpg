# Ready Queue (Design Session writes, Build Session reads)

Single active queue for the Build Session under [WORKFLOW.md](WORKFLOW.md) (`2026-11-07`):

- **Ownership**: Design writes `READY.md` and shared specs; Build runs `ready-to-build` / `feasibility-check` slices and writes updates/deltas exclusively to [FEEDBACK.md](FEEDBACK.md) (archiving closed items in [FEEDBACK_CLOSED.md](archive/FEEDBACK_CLOSED.md)).
- **Build Lease Guard**: Before resuming or pushing, run `python tools/planner/feedback_check.py lease check`; halt immediately on exit `1`.
- **Art/UI Ownership & Placeholders**: `ART_UI_REQUESTS.md`, `tools/gen_music.py`, audio files, and `assets/**/README.md` are Art/UI-owned. Missing art/audio uses a working placeholder copied to the final path and logged in `FEEDBACK.md` so Art/UI replaces it in place.
- Do not build directly from legacy docs unless queued here.

## 1. Ready to Build (`ready-to-build`)

Execute these slices in order. Existing engine goldens must remain unchanged except where a named golden re-record is specified. `R1`..`R4`, `PR4-1..PR4-9`, and `PR5-1..PR5-62c` (`113` slices through `48f24342`) are `(done)` and archived in [READY_DONE_ARCHIVE.md](archive/READY_DONE_ARCHIVE.md). Wave 2 (`PR6-1..PR6-5`) begins below per [WAVE2_PLAN.md](sessions/WAVE2_PLAN.md).

| Order | Task ID | Masterplan | Layer | Summary & Spec | Acceptance Test |
| --- | --- | --- | --- | --- | --- |
| 1 | `PR6-1` (`WAVE2-BALANCE`) | `U2`, `E2` | data + sim | `[PROVISIONAL]` Tune `brawler -9.4` swap delta (`brawler_clinch 0.70->0.90x`, `slag_charge 1.35->1.50x`, `gauntlet`/`forearm_braces`), equip `sim.json` `line` enemies w/ `M1` (`hook_drag`), review `[PROVISIONAL]` Mage AoE (`0.28..0.35`), verify `sim party 5 30` & `sim sweep 5 30` (`boss 30–50%`, `line 70–90%`, Crucible `0%`). Spec: [PK8_ACT23_CONTENT.md](sessions/PK8_ACT23_CONTENT.md) §3. | `fc.sh content check`, `sim sweep 5 30`, `sim party 5 30`, `sim feel world=sunk_ward enc=enc_evt_06 seeds=10`; named goldens if intentionally tuned: `sim_golden.json`, `rules_golden.json`, `phasefold_golden.json` (+ re-freeze `timeline_kit.json` if catalog changes) |
| 2 | `PR6-2` (`PK2-STAGE-PASSIVES`) | `U4`, `S2` | kit + asset | `[PROVISIONAL]` Story-free kit passive grants `armor` (`Target::Armor` in `damage::base_armor_rating`), `move_rate` (`Target::MoveRate` in `move_per_block`), and `skill_potency` group `"ranged"` + optional `min_distance`; author `calcify_2` (`+2` armor, `-1` move_rate) & `calcify_3` (`resist weaken/stun 100`, `-15%` ranged at `distance >= 4`) in `passives.json` / `unit_tracks.json`. Spec: [PK3_ROSTER_LIFECYCLE.md](sessions/PK3_ROSTER_LIFECYCLE.md) §1. | New unit/content tests in `rpgkit_rules` & `rpgkit_content` + `emberwake_world/tests/techniques_grafts.rs`, `fc.sh layers`, `fc.sh content check`; named golden ONLY if passive type list error text appends new types: `checks_golden.json` |
| 3 | `PR6-3` (`PK3-BLEEDOUT-REVIVE`) | `E3`, `F` | kit + game + asset | `[PROVISIONAL]` Wire `Session.bleed_out_rounds` (`0` default classic/`ashlands`/Act 1; `3` `sunk_ward` Acts `2..3`), round-end countdown (`fate: dead`, `cause: bled_out`), `revive: bool` on `rally` & new `smelling_salts` (`15 Scrap`) w/ occupied-tile nearest-cell resolve (`"Space is blocked"` refusal), and downed HUD bleed countdown. Spec: [PK3_ROSTER_LIFECYCLE.md](sessions/PK3_ROSTER_LIFECYCLE.md) §2. | `rpgkit_battle/tests/bleed_out.rs` (new), `rpgkit_world/tests/roster.rs`, `fc.sh layers`, `fc.sh content check`, `cargo test --workspace`; **0 golden diffs** |
| 4 | `PR6-4` (`PK3-RECRUITS-KEEPSAKES`) | `E3` | kit + game + asset | `[PROVISIONAL]` Add `UnitState` (`name`, `housing`, `fuse`, `origin`), `party.pool`, `party.keepsakes`, `world.recruits` Cohort generator (`18-24` visored/cowled, `"recruit"` roll stream), `rescue`/`recruit {pool}`, `Isolation Crypts` (`crypts: 2`), `Living Martyrs` fuse, and `keepsakes.by_class` procedural instances (`6` templates). Spec: [PK3_ROSTER_LIFECYCLE.md](sessions/PK3_ROSTER_LIFECYCLE.md) §§3–4. | `rpgkit_world/tests/{roster,temperament,campaign}.rs`, `rpgkit_content/tests/roster_schema.rs`, `emberwake_world/tests/sunk_ward_roster.rs`, `fc.sh layers`, `fc.sh content check`; **0 golden diffs** (`save`/`data_source` additive checks updated) |
| 5 | `PR6-5` (`ACT2-DROPS-PK2-FLAGS`) | `U3`, `E2` | asset | `[PROVISIONAL]` Seed deferred `FB-277` Act 2 hooks via `aegis content`: `15` `MASTERY_BOOK` manuals (`150` XP), `slot_*`/organ drops on `sunk_ward` mobs/loot, limited `black_iron_ingot` stock/rewards, and `pk2.*` (`masterwork_quest`, `minor_quest`, `apex_open`, `bone_scraping`, `rite_<class>`) setters (`0` `ashlands`/`iron_reaches`/`tidewrack` edits). Spec: [PK8_ACT23_CONTENT.md](sessions/PK8_ACT23_CONTENT.md) §1. | `fc.sh content manifest`, `fc.sh content check`, `emberwake_world/tests/{techniques_grafts,act1_story,sunk_ward_roster}.rs`; **0 battle/sim golden diffs** |

## 2. Feasibility Check (`feasibility-check`)

`MASTERPLAN1` (`25` sub-blocks), `WEBREL1` (`FB-147`), and `ACTS23-R5` (`FB-167`) are audited (`FB-01`..`FB-59`, `FB-147`, `FB-167`; archived in [READY_ANSWERS_ARCHIVE.md](archive/READY_ANSWERS_ARCHIVE.md)). Audit Wave 2 `R6` / `R7` kit seams below before slicing into §1:

| Task ID | Blocks | Specs to Audit | Questions for Build |
| --- | --- | --- | --- |
| `ACTS23-R6` | `PK4`, `PK5` (`W1`, `W3`, `E2`, `T1`) | [PK4_PK5_CAMPAIGN_HUB.md](sessions/PK4_PK5_CAMPAIGN_HUB.md) | Audit `campaign/trigger.rs` (`recipe` / `battle` inputs), `shop.rs` per-act pricing, new `facilities` service + `sieges.json` connected-room HP/downed carry-over vs existing `story.rs` queue, cost, and `0`-diff gating. |
| `ACTS23-R7` | `PK6`, `PK7` (`F`, `P`, `T2`, `X1`) | [PK6_PK7_BOSSES_ENDINGS.md](sessions/PK6_PK7_BOSSES_ENDINGS.md) | Audit `start_meter` regional Strain, `arena.rs`/`ground.rs` `when_meter`, `"$crucible_captive"` lookup + `PHASEFX1`, `StoryCampPrep` + `retry_day_cost`/`echo_fallen`, and `END_KINDS::ending` + Epilogue screen. |

## 3. Held / Post-Release Pre-Staged Queue (`Post-R4` & Wave 2 `R6/R7`)

Post-Act-1 or audit-gated slices: Build takes no action until promoted to Section 1. Pre-split `Original row:` notes for `PR4-1`/`PR4-2` live in [READY_ANSWERS_ARCHIVE.md](archive/READY_ANSWERS_ARCHIVE.md) §3.

| Wave 2 Order | Task ID | Masterplan | Summary & Dependencies | Named Golden Re-record |
| --- | --- | --- | --- | --- |
| R6 (`After PR6-5`) | `PK4-PRESSURE` | `W3`, `T1`, `E2` | **Held until `ACTS23-R6` feasibility**: `doctrine` gauge (`-100..+100`), play-input triggers, `unit_tracks.lean`, `2.4` Great Tithing clock relabel, Act 2–3 checkpoints, Act 3 hard clock & `2.0x` Tallow price ([PK4_PK5_CAMPAIGN_HUB.md](sessions/PK4_PK5_CAMPAIGN_HUB.md) Part A). | `rules_contract_golden` (if rules/triggers expand) |
| R6 (`After PK4`) | `PK5-HUB-SIEGES` | `W1`, `T2` | **Held until `ACTS23-R6` feasibility**: `5 × 3` Hub `facilities` (`Hearth`, `Slab`, `Board`, `Bunks`, `Crypts`) + Alert `100` connected-room Hub Sieges w/ `Ruthven` `25%` retreat ([PK4_PK5_CAMPAIGN_HUB.md](sessions/PK4_PK5_CAMPAIGN_HUB.md) Part B). | none expected |
| R6 (`After PK5`) | `PK8-ACT2-BASTION` | `W`, `T2`, `E`, `F` | **Held (`After R6`)**: `sunk_ward` `high_bastion` Act 2 map/routes/nodes (`08-15`), Bestiary, `6` Loyalty + Survivor Inheritance arcs ([PK8_ACT23_CONTENT.md](sessions/PK8_ACT23_CONTENT.md) §2; `ashlands`/`iron_reaches`/`tidewrack` stay classic parity). | none (`sunk_ward`-only) |
| R7 (`After R6`) | `PK6-PK7-ACT3-TECH` | `F`, `P`, `T2`, `X1` | **Held until `ACTS23-R7` feasibility**: Bell/Mantle Resonance hazards, `Node 20` `$crucible_captive`, `camp_prep` + Dream-Echo retries, and Endings `A / B / C` + Epilogue screen ([PK6_PK7_BOSSES_ENDINGS.md](sessions/PK6_PK7_BOSSES_ENDINGS.md)). | none expected |
| R7 (`After PK7`) | `PK8-ACT3-FISSURE` | `W`, `T2`, `E`, `F` | **Held (`After R7`)**: `sunk_ward` `mantle_fissure` Act 3 nodes (`16-22`), `Malachi`, `Ulric`, `Ysolde`, `Nemeth`, `Othemar`, `Dreaming Core` & Ending cards ([PK8_ACT23_CONTENT.md](sessions/PK8_ACT23_CONTENT.md) §2). | none (`sunk_ward`-only) |
| Post-R4 | `BASERULES1-PP` | `U1.2`, `S1.2` | Parked (`FB-79`): optional 3-stat world mode merging `STR+CONS`, `DEX+PERC`, `INT+KNOW` (plus `Composure` `shaken` at 0 / Rest `+5`). | `rules_golden`, `timeline_golden`, `sim_golden` |
| Post-R4 | `R3-STAGE-SEED` | `W.2`, `E1.2`, `T1.2` | **Split (`PR4-1` shipped `stage=N`, `AUDIT-07`)**: holds campaign `growth_per_stage` values and interactive-story `roll` choice. | `journey_golden`, `sim_golden` |
| Post-R4 | `POST-R4-CLEANUP` | `U1.2`, `U2.3`, `C27`, `C28` | **Split (`PR4-2` shipped items 1,3,4)**: holds item (2), playthrough review of `goblin_skirmish` (`AUDIT-09`) and 5 explicit-`spend` enemies (`ash_stalker`, `cinder_hound`, `ember_wraith`, `gilded_Automaton_Sentry`, `sunken_dredger`). | `timeline_golden`, `rules_golden`, `sim_golden`, `phasefold_golden` |
| Later | `INTEG-ASSETS1` | `X3`, `X8` | **Held**: post-R5-R7 integration pass (`integration` agent + `aegis scenario`) verifying replaced Art/UI assets against sizes, clip/frame counts, loop tags, palettes, names. | none expected |
| Later | `ENGINE-PURITY-2` | `X6` | **Held (`FB-227(c-e)`, `P-ARCH-01`)**: optional cleanup for non-turn-based second game (`TimelineStyle.hatched`, `bridge.rs`, `aegis_ui::timeline` labels). | none expected |
| Later | `RULESET-CLEANUP` | `R1` | **Parked (`FB-269(a),(c)`, `P-ARCH-02`)**: optional typed Forge `modes` editor & retiring remaining `classic` legacy seeds (`cln44.json`, `cln45.json`, `attacklock1.json`, `p12s1.json`). | none expected |
| Later | `VFX-DATA2` | `FX1` | **Parked (`FB-270(b),(c)`, `P-UI-02`)**: neutral-look tint/glow (`buff` vs `heal`) & shared generic `fall_dust`/`apply_burst` presets. | none expected |
| Later | `ASSETORG4-2` | `A11` | **Parked (`FB-273(c)`, `AO-08`, [ASSETORG4.md](archive/specs/ASSETORG4.md) §1.1)**: finish moving kit-owned assets (`items/<weight>` rename `plate`/`leather`/`cloth` -> `heavy`/`medium`/`light`, generic textures/SFX) to `kits/rpg/assets/` or `engine/`. | none expected |

## 4. Feedback Responses (Design answers to `docs/FEEDBACK.md`)

`FB-01`..`FB-277` and `UN-01`..`UN-12` live in [FEEDBACK_CLOSED.md](archive/FEEDBACK_CLOSED.md); `FB-33`..`FB-277` and `AO-01`..`AO-10` answers are archived in [READY_ANSWERS_ARCHIVE.md](archive/READY_ANSWERS_ARCHIVE.md). Zero open Build items remain (`48f24342`).

| Feedback ID | Disposition | Follow-up | Design Decision / Response | Date |
| --- | --- | --- | --- | --- |
