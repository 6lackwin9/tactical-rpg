# Ready Queue (Design Session writes, Build Session reads)

This file is the **single active queue** for the Build Session under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Design Session** is the sole writer of `docs/READY.md` and all specs/shared docs.
- **Build Session** builds **only** items marked `ready-to-build` below, runs read-only code audits on items marked `feasibility-check`, and writes all status updates, feasibility notes, and docs deltas exclusively to `docs/FEEDBACK.md`.
- Do not build directly from `docs/TASKS.md`, `docs/WORK_ORDER.md`, or pre-masterplan specs unless an item is listed here.

## 1. Ready to Build (`ready-to-build`)

Execute these behaviour-neutral tasks in order. Existing engine goldens must remain unchanged.

| Order | Task ID | Masterplan | Layer | Summary & Spec | Acceptance Test |
| --- | --- | --- | --- | --- | --- |
| 1 | `TESTCLEAN1` | `X6` | engine | Tests in `aegis_ui/tests/` leave `aegis_*` temp folders behind. Add a shared scratch helper or cleanup so every test removes its temp directory on exit. | Running `cargo test -p aegis_ui` leaves 0 new `aegis_*` folders in `$TEMP`. |
| 2 | `SKILLS1` | `X6` | process / tools | Move planner helper scripts to `tools/planner/` per `docs/specs/AGENTROLES1.md`; update `tools/fc.sh test` so panic text is printed on test failure. | `tools/fc.sh test` prints panic message lines on a Deliberate/synthetic panic; scripts live in `tools/planner/`. |
| 3 | `KITIDS1` | `U2.2` prep | kit + game | Remove hard-coded class IDs from `rpgkit_rules/src/unit.rs`, `rpgkit_data/src/lib.rs`, and `rpgkit_sim/src/{args,roles,campaign}.rs`; pass role/comp defaults from `emberwake_sim` (`docs/specs/SILOS1.md` T4). | `tools/fc.sh layers` and `sim_golden` pass unchanged; kit crates contain no hardcoded class ID assumptions. |
| 4 | `BASERULES1-M` | `U1.2` prep | kit | Rule metadata register, `check/ruleset.rs`, bundle tests, and `aegis rules` CLI with `LOCKED` and `--contract` (`docs/specs/BASERULES1.md` sections 1-2 only). | `aegis rules --contract` lists ruleset fields with locked/tunable metadata; bundle validation tests pass. |

## 2. Feasibility Check (`feasibility-check`)

Read-only code audit by the Build Session before these move to `ready-to-build`. Check against current crates, estimate slice size, flag any hidden cost or U3/U4/combat touchpoints in `docs/FEEDBACK.md`. **Do not write code for these yet.**

| Order | Sub-block / ID | Spec to Audit | Scope of Audit | Fixed vs Tunable & Acceptance Target |
| --- | --- | --- | --- | --- |
| 1 | `U1.2` | `docs/sessions/U1_STATS.md` | Unified derived stat class (`Pool` mode: HP, poise, AP; `Rating` mode: initiative, move_rate, armour, crit, resist, potency, weapon_power), 2-layer multiplicative mitigation, and the 4-stage modifier schema (`base`, `amplifiers`, `mitigation`, `finish`) for standing/derived stats. Note: `BASERULES1-F` (`feel_flat`) is retired; Static units (`U2`) replace it. | Fixed vs tunable table in `U1_STATS.md` "Fixed vs tunable". Proof: Composure added as a Pool in data with zero Rust changes. |
| 2 | `U2.2` | `docs/sessions/U2_CLASS.md` | Static vs Growth units, `budget = sum(stat * weight)` with largest-remainder rounding, class stat shares, tier-scaled HP base, and world-stage mob growth curves. Check how `build_unit_in` (`C05`) and `WORLDFIGHT1` (`C18`) should pass world stage to enemy unit creation. | Fixed vs tunable table in `U2_CLASS.md`. Proof: any unit (static or growth) fights in `rpgkit_battle` with no unit-specific code. |
| 3 | `A4.2` (`INTENT2`) | `docs/specs/INTENT2.md` | Audit `INTENT2.md` with `IR21` enforced (`also_intents`, `intent_versions`, and authored `field_bonus` are **not** in schema; derived by one kit function from baseline skill fields). Flag any remaining conflicts in `INTENT2.md` sections 2-8. | Baseline skill fields only in schema; pure kit derivation for intent tags and Field bonus. |

## 3. Held / Waiting on Design

- **`ATTACKLOCK1-O`**: Built behind off-switch (`ATTACKLOCK1` slice 1). Holds until `A1`/`F2` design confirms power > 0 lock rule and melee band check.
- **`WORLDFIGHT1` (`C18`)**: Holds until `U` and `F` seams settle (`MASTERPLAN1` Q5).
- **`U3.1` & `U4.1`**: In active design session (gear block_cost/proficiency and passive/loadout sources). Once closed, `U1.2-U4.2` will move from `feasibility-check` to `ready-to-build`.

## 4. Feedback Responses (Design answers to `docs/FEEDBACK.md`)

| Feedback ID | Spec / Task | Design Decision / Response | Date |
| --- | --- | --- | --- |
| - | - | No open feedback items yet. | 2026-11-07 |
