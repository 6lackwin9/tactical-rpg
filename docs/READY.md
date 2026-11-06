# Ready Queue (Design Session writes, Build Session reads)

This file is the **single active queue** for the Build Session under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Design Session** is the sole writer of `docs/READY.md` and all specs/shared docs.
- **Build Session** builds **only** items marked `ready-to-build` below, runs read-only code audits on items marked `feasibility-check`, and writes all status updates, feasibility notes, and docs deltas exclusively to `docs/FEEDBACK.md` (moving answered/closed `FB-nn` items into Build-owned `docs/archive/FEEDBACK_CLOSED.md` to keep `docs/FEEDBACK.md` under 25 KB).
- Do not build directly from `docs/TASKS.md`, `docs/WORK_ORDER.md`, or pre-masterplan specs unless an item is listed here.

## 1. Ready to Build (`ready-to-build`)

Execute these slices in order. Existing engine goldens must remain unchanged except where a named `checks_golden` re-record is specified.

| Order | Task ID | Masterplan | Layer | Summary & Spec | Acceptance Test |
| --- | --- | --- | --- | --- | --- |
| 1 | `TESTCLEAN1` | `X6` | engine | Tests in `aegis_ui/tests/` leave `aegis_*` temp folders behind. Add a shared scratch helper or cleanup so every test removes its temp directory on exit. | Running `cargo test -p aegis_ui` leaves 0 new `aegis_*` folders in `$TEMP`. |
| 2 | `SKILLS1` | `X6` | process / tools | Move planner helper scripts to `tools/planner/` per `docs/specs/AGENTROLES1.md`; update `tools/fc.sh test` so panic text is printed on test failure. | `tools/fc.sh test` prints panic message lines on a synthetic panic; scripts live in `tools/planner/`. |
| 3 | `KITIDS1` | `U2.2` prep | kit + game | Remove hard-coded class IDs from `rpgkit_rules/src/unit.rs`, `rpgkit_data/src/lib.rs`, and `rpgkit_sim/src/{args,roles,campaign}.rs`; pass role/comp defaults from `emberwake_sim` (`docs/specs/SILOS1.md` T4). | `tools/fc.sh layers` and `sim_golden` pass unchanged; kit crates contain no hardcoded class ID assumptions. |
| 4 | `BASERULES1-M` | `U1.2` prep | kit | Rule metadata register, `check/ruleset.rs`, bundle tests, and `aegis rules` CLI with `LOCKED` and `--contract` (`docs/specs/BASERULES1.md` sections 6-7 and `BASERULES1_LIBRARY.md` section 7). | `aegis rules --contract` lists ruleset fields with locked/tunable metadata; bundle validation tests pass. |
| 5 | `U1.2-S1` | `U1.2` | kit | Slice 1 of `U1.2` (`docs/sessions/U1_STATS.md` + `FB-04..05`): exact port of the 4-stage pipeline (`base -> flat -> mult -> mitigation -> round`) and typed pool facade (`Unit.hp`, `Unit.ap`, `poise` kept as fields). | All existing `rules_golden` and `sim_golden` tests pass byte-identical (`0` delta). |
| 6 | `U2.2-S1` | `U2.2` | kit + data | Slice 1 of `U2.2` (`docs/sessions/U2_CLASS.md` + `FB-10`, `FB-15`): `unit_budget` module with two share vectors (`base_shares` + `growth_shares`) and largest-remainder rounding, plus computed `base_stats`/`growth` compatibility view. | Levels 1-10 reproduce existing classes; `rules_golden`, `party.rs`, `forge.rs`, and `timeline_kit.json` pass unchanged. |
| 7 | `U3.2-S1` | `U3.2` | kit + data | Slice 1 of `U3.2` (`docs/sessions/U3_GEAR.md` + `FB-37`): drop `block_cost` from `ItemDef`, `budget.rs` (`W_EXTRA_BLOCK`), `check/`, `source.rs`, `store.rs`, schema, and `item_families.json` (loader ignores stored key and drops on resave). | Named `checks_golden` re-record (`43` `block_cost` lines); `rules_golden` and `sim_golden` byte-identical. |
| 8 | `U3.2-S2` | `U3.2` | kit | Slice 2 of `U3.2` (`docs/sessions/U3_GEAR.md` + `FB-31..32`, `FB-36c`, `FB-37`): wire `ALWAYS_RANGED_TYPES`, `off_hand_ok`, `is_shield`, `skill_in_group`, `REACH_WEAPON_REACH`, and check lists (`GEAR_TYPES`, `WEAPON_TYPES`, `OFF_CATEGORIES`, `FOCUS_TYPES`) to read `item_families.json` and `engine_rules`. | Every golden (`rules_golden`, `sim_golden`, `checks_golden`) passes byte-identical. |
| 9 | `U4.2-S1` | `U4.2` | kit | Slice 1 of `U4.2` (`docs/sessions/U4_PASSIVES.md` + `FB-25..27`, `FB-30`): read-time typed grant view (`modifier` with combine rules, `proficiency` replacing `with_drills`, `grant_skill`, `grant_pool`, `trigger`) with today's accumulators kept as evaluators first. | Double-load test asserts old and new are equal across all 55 shipped rows + 16 fixture rows; `0` golden diffs. |
| 10 | `U2.2-S2` / `U4.2-S2` | `U2.2` + `U4.2` | kit + game | Single unit builder (`FB-12`, `FB-23`, `FB-28`, `FB-30`): unify hero, NPC, Dojo, and sim unit builds + `character_passives` source gathering (keeping today's gather order), give `training_dummy` a static/explicit class (`FB-23`), and change `kits/rpg/schema.json` class `art` default to `""` (`FB-24`). | All goldens byte-identical; no silent first-class fallback needed. |

## 2. Feasibility Check (`feasibility-check`)

Read-only code audit requested from the Build Session for `S1.2` and `S2.2` (Session 3 design is now closed).

| Order | Sub-block / ID | Spec to Audit | Audit Focus | Status |
| --- | --- | --- | --- | --- |
| 1 | `S1.2` | `docs/sessions/S1_STATUS.md` | Audit data-driven 18-status class over 6 primitives (`modifier`, `tick`, `block_lock`, `targeting`, `reactive`, `rule_flag`), round-granular Guard timing (wind-up start + full-round roll-over, `Option 1C`), and single status resist. | `feasibility-check` (new) |
| 2 | `S2.2` | `docs/sessions/S2_POISE.md` | Audit strict per-hit Poise Pip Threshold rule (`5` pips, `T = Max_Poise / 5`, `pips_lost = clamp(floor(D / T), 0, 3)`, chip immunity for `D < T`, guard `0.5x` poise mult, break at `0` pips + refill at `roll_over`). | `feasibility-check` (new) |
| 3 | `U1.2` - `U4.2` | `docs/sessions/U1..U4*.md` | Audited (`FB-04..15`, `FB-25..37`). Slices 1-2 queued in Section 1 above; remaining slices follow. | Audited |
| 4 | `A4.2` (`INTENT2`) | `docs/specs/INTENT2.md` | Audited (`FB-16..22`). Design applying `FB-17..22` text updates to `INTENT2*.md`. | Audited |

## 3. Held / Waiting on Design

- **`ATTACKLOCK1-O`**: Built behind off-switch (`ATTACKLOCK1` slice 1). Holds until `A1`/`F2` design confirms power > 0 lock rule and melee band check.
- **`WORLDFIGHT1` (`C18`)**: Holds until `U` and `F` seams settle (`MASTERPLAN1` Q5).
- **`A1.1` - `A4.1`**: Next design session (Action block: skills, consumables, resolution, shapes, triggers).

## 4. Feedback Responses (Design answers to `docs/FEEDBACK.md`)

`FB-01` to `FB-22` were accepted and archived in `docs/archive/FEEDBACK_CLOSED.md` (2026-11-07).

| Feedback ID | Spec / Task | Design Decision / Response | Date |
| --- | --- | --- | --- |
| `FB-23` | `KITIDS1`, `U2.2` | **Accepted**: Give `training_dummy` an explicit class / migrate it in the `FB-12` static unit builder (`U2.2-S2`), with a content error if a classless unit has no fallback. | 2026-11-07 |
| `FB-24` | `schema.json` | **Accepted**: Change class `art` default in `kits/rpg/schema.json` from `"warrior"` to `""` (verifying no shipped class relies on the omitted default) as part of `U2.2-S2`. | 2026-11-07 |
| `FB-25` | `U4_PASSIVES.md` | **Accepted**: Name items (`apply_effects`) and meter bands (incl. `crit`) as consumers of the unified grant kinds; retire unused `spell_reach` (`0` shipped rows). Recorded in `U4_PASSIVES.md`. | 2026-11-07 |
| `FB-26` | `U4_PASSIVES.md` | **Accepted**: Add per-target combine rules (`sum`, `sum_capped`, `product`, `max`, `or_flag`) to `modifier`, and add a 5th grant kind `grant_pool` for `meter`. Recorded in `U4_PASSIVES.md`. | 2026-11-07 |
| `FB-27` | `U4_PASSIVES.md` | **Accepted**: The typed grant loader replaces `Proficiency::with_drills` too, keeping the rule that slottable passives do not train outside battle. | 2026-11-07 |
| `FB-28` | `U4_PASSIVES.md` | **Accepted**: Keep today's `character_passives` gather order (`character -> faction -> discipline -> named items`) so `foresight` and trigger-order goldens hold; deduplicate by passive ID while inline item effects (`gear_*`) rebuild on sync; re-gather live in the `FB-12` builder. | 2026-11-07 |
| `FB-29` | `U4_PASSIVES.md` | **Accepted**: Put `temperament` on the character row (`characters.temperament` naming a slotless passive + optional `default_intent` and `ai_role` keys), keeping passive rows pure. | 2026-11-07 |
| `FB-30` | `U4_PASSIVES.md` | **Accepted**: Queue `U4.2-S1` (typed grant loader + double-load test, `0` golden diffs) and `U4.2-S2` (unified source gatherer) in `ready-to-build` now; move evaluators onto `U1.2` pipeline after `U1.2` lands. | 2026-11-07 |
| `FB-31` | `U3_GEAR.md` | **Accepted**: Keep field name `weapon_type` and `item_families.json`; wire the remaining hardcoded sites to read `item_families.json` and `engine_rules`. Recorded in `U3_GEAR.md`. | 2026-11-07 |
| `FB-32` | `U3_GEAR.md` | **Accepted**: Derive `GEAR_TYPES`, `WEAPON_TYPES`, `OFF_CATEGORIES`, and `FOCUS_TYPES` from `item_families.json`; word test 1 as "a new family using existing engine rules, in data only". | 2026-11-07 |
| `FB-33` | `U3_GEAR.md` | **Accepted**: Keep the `15 %`/point stat-deficit penalty (capped at `60 %`, `-2` req/level, extended to armour) + `+level` weapon power; add `Proficiency 0` soft penalty (`0.75x` power/armour + `+1` burden, stacking multiplicatively); keep unarmed at `0` power unless `fist >= 2`; keep "no proficiency list = no restriction". | 2026-11-07 |
| `FB-34` | `U3_GEAR.md` | **Accepted**: Include the party-screen penalty display (mocked first) in `U3.2` Slice 3; skill `min_proficiency` and `weapon_types` gates stay hard. | 2026-11-07 |
| `FB-35` | `U3_GEAR.md`, `U4` | **Accepted**: Recompute item grants and `U4` sources on every sync in the shared builder; worn item skills (`OFFHAND1`) are always active while worn and take no deck slot. | 2026-11-07 |
| `FB-36` | `U3_GEAR.md` | **Accepted**: Keep the built 3-hand rule (`main_2h` pairs with non-focus off-hand weapon via switch), state `23 +/- 20 %` as the class loadout budget (`special`/`special_note` for static item override), let `range_band.max` replace `REACH_WEAPON_REACH`, keep party-wide carry alongside per-unit burden -> `Initiative`, and keep skill gating on skill rows. | 2026-11-07 |
| `FB-37` | `U3_GEAR.md` | **Accepted**: Queue `U3.2-S1` (drop `block_cost`) and `U3.2-S2` (family table drives rules) in `ready-to-build` now; Slices 3-5 follow `U1.2` and `U4.2`. | 2026-11-07 |
| `FB-38` | Docs delta | **Accepted**: Applied `SKILLS1` and `KITIDS1` docs deltas to `docs/WORKFLOW.md`, `GAME_BRIEF.md`, `STRUCTURE.md`, `.claude/skills/`, and marked `SKILLS1` + `KITIDS1` done in `docs/TASKS.md`. | 2026-11-07 |
| `FB-39` | Docs delta | **Accepted**: Applied `TESTCLEAN1` (`aegis_core::scratch::Scratch`) docs delta to `docs/STRUCTURE.md`, `engine/README.md`, `.claude/skills/rust-conventions/SKILL.md`, and marked `TESTCLEAN1` done in `docs/TASKS.md`. | 2026-11-07 |
| `UN-01` | Temp folder cleanup | **Approved (user 2026-11-07)**: Temp cleanup is allowed. Build Session (or Design) may delete old pre-`TESTCLEAN1` `emberwake_*` and `rpgkit_*` folders in `$TEMP`. | 2026-11-07 |
| `UN-02` | `VISCLAR1-D` / M1 look | **Deferred to post-combat-silo build (user 2026-11-07)**: Pushed to backlog/superseded during the engine refactor so the user reviews battle visuals once after the `U`/`S`/`A`/`F` combat silos finish building, rather than reviewing transient state. | 2026-11-07 |
| `UN-03` | `develop` -> `main` release | **Silo-based releases (`R1-R4`, user 2026-11-07)**: Replace old `M1..M4` milestone releases with Masterplan Silo releases: **`R1`** after Unit + Status (`U1-U4`, `S1-S2`, `C01-C05`) lock; **`R2`** after Action + Combat (`A1-A4`, `F1-F4`, `P1-P2`, `I1`, `C06-C17`) + consolidated user Arena visual/feel review; **`R3`** after World + Economy + Story (`E`, `W`, `T`, `C18-C28`); **`R4`** after Final Shells (`X1-X8`). Hold `main` release until `R1`. | 2026-11-07 |
