# Ready Queue (Design Session writes, Build Session reads)

This file is the **single active queue** for the Build Session under the two-session split (decision "Two-session split: Design and Build", 2026-11-07).

- **Design Session** is the sole writer of `docs/READY.md` and all specs/shared docs.
- **Build Session** builds **only** items marked `ready-to-build` below, runs read-only code audits on items marked `feasibility-check`, and writes all status updates, feasibility notes, and docs deltas exclusively to `docs/FEEDBACK.md` (moving answered/closed `FB-nn` items into Build-owned `docs/archive/FEEDBACK_CLOSED.md` to keep `docs/FEEDBACK.md` under 25 KB).
- Do not build directly from `docs/TASKS.md`, `docs/WORK_ORDER.md`, or pre-masterplan specs unless an item is listed here.

## 1. Ready to Build (`ready-to-build`)

Execute these behaviour-neutral tasks in order. Existing engine goldens must remain unchanged.

| Order | Task ID | Masterplan | Layer | Summary & Spec | Acceptance Test |
| --- | --- | --- | --- | --- | --- |
| 1 | `TESTCLEAN1` | `X6` | engine | Tests in `aegis_ui/tests/` leave `aegis_*` temp folders behind. Add a shared scratch helper or cleanup so every test removes its temp directory on exit. | Running `cargo test -p aegis_ui` leaves 0 new `aegis_*` folders in `$TEMP`. |
| 2 | `SKILLS1` | `X6` | process / tools | Move planner helper scripts to `tools/planner/` per `docs/specs/AGENTROLES1.md`; update `tools/fc.sh test` so panic text is printed on test failure. | `tools/fc.sh test` prints panic message lines on a synthetic panic; scripts live in `tools/planner/`. |
| 3 | `KITIDS1` | `U2.2` prep | kit + game | Remove hard-coded class IDs from `rpgkit_rules/src/unit.rs`, `rpgkit_data/src/lib.rs`, and `rpgkit_sim/src/{args,roles,campaign}.rs`; pass role/comp defaults from `emberwake_sim` (`docs/specs/SILOS1.md` T4). | `tools/fc.sh layers` and `sim_golden` pass unchanged; kit crates contain no hardcoded class ID assumptions. |
| 4 | `BASERULES1-M` | `U1.2` prep | kit | Rule metadata register, `check/ruleset.rs`, bundle tests, and `aegis rules` CLI with `LOCKED` and `--contract` (`docs/specs/BASERULES1.md` sections 6-7 and `BASERULES1_LIBRARY.md` section 7). | `aegis rules --contract` lists ruleset fields with locked/tunable metadata; bundle validation tests pass. |
| 5 | `U1.2-S1` | `U1.2` | kit | Slice 1 of `U1.2` (`docs/sessions/U1_STATS.md` + `FB-04..05`): exact port of the 4-stage pipeline (`base -> flat -> mult -> mitigation -> round`) and typed pool facade (`Unit.hp`, `Unit.ap`, `poise` kept as fields). | All existing `rules_golden` and `sim_golden` tests pass byte-identical (`0` delta). |
| 6 | `U2.2-S1` | `U2.2` | kit + data | Slice 1 of `U2.2` (`docs/sessions/U2_CLASS.md` + `FB-10`, `FB-15`): `unit_budget` module with two share vectors (`base_shares` + `growth_shares`) and largest-remainder rounding, plus computed `base_stats`/`growth` compatibility view. | Levels 1-10 reproduce existing classes; `rules_golden`, `party.rs`, `forge.rs`, and `timeline_kit.json` pass unchanged. |

## 2. Feasibility Check (`feasibility-check`)

Read-only code audit requested from the Build Session for `U3.2` and `U4.2` (Block `U` design is now 100% closed across `U1.1-U4.1`). Once `U3.2` and `U4.2` are audited, the remaining slices of `U1.2-U4.2` will move into `ready-to-build`.

| Order | Sub-block / ID | Spec to Audit | Audit Focus | Status |
| --- | --- | --- | --- | --- |
| 1 | `U3.2` | `docs/sessions/U3_GEAR.md` | Audit gear family table migration (`weapon_kind`), soft wield penalty (`Proficiency 0`), dropping `block_cost`, and `OFFHAND1` via `grant_skill`. | `feasibility-check` (new) |
| 2 | `U4.2` | `docs/sessions/U4_PASSIVES.md` | Audit collapsing the 13 passive effect variants into 4 unified grant kinds (`modifier`, `proficiency`, `grant_skill`, `trigger`) with compatibility loader, 4-source deduplication (`C04`), and `ROSTER1-T` temperament fold-in. | `feasibility-check` (new) |
| 3 | `U1.2` | `docs/sessions/U1_STATS.md` | Audited (`FB-04` to `FB-09`): 6 slices (2S + 3M + 1L). All recommendations accepted in `U1_STATS.md`. | Audited; Slice 1 in `ready-to-build`. |
| 4 | `U2.2` | `docs/sessions/U2_CLASS.md` | Audited (`FB-10` to `FB-15`): 6 slices (1S + 4M + 1L). All recommendations accepted in `U2_CLASS.md`. | Audited; Slice 1 in `ready-to-build`. |
| 5 | `A4.2` (`INTENT2`) | `docs/specs/INTENT2.md` | Audited (`FB-16` to `FB-22`): `IR21` needs no rollback; K1 M, K2 M, K3 M, AI S + guard-status task. | Audited; Design updating `INTENT2*.md` per `FB-16..22`. |

## 3. Held / Waiting on Design

- **`ATTACKLOCK1-O`**: Built behind off-switch (`ATTACKLOCK1` slice 1). Holds until `A1`/`F2` design confirms power > 0 lock rule and melee band check.
- **`WORLDFIGHT1` (`C18`)**: Holds until `U` and `F` seams settle (`MASTERPLAN1` Q5).
- **`S1.1` & `S2.1`**: Next design session (Status class & Pools).

## 4. Feedback Responses (Design answers to `docs/FEEDBACK.md`)

| Feedback ID | Spec / Task | Design Decision / Response | Date |
| --- | --- | --- | --- |
| `FB-01` | `BASERULES1-M` | **Accepted**: Updated section 1 row 4 above to point to `BASERULES1.md` sections 6-7 and `BASERULES1_LIBRARY.md` section 7. | 2026-11-07 |
| `FB-02` | `planner`, `task-landing` | **Accepted**: Updated `.claude/skills/planner/SKILL.md` step 3.3 and `.claude/skills/task-landing/SKILL.md` so Build writes agent runs and docs deltas only to `docs/FEEDBACK.md` (plus crate-local READMEs). | 2026-11-07 |
| `FB-03` | `fc.sh docs` | **Accepted**: Design runs `fc.sh docs` (size + lint) before every commit; if a doc ever blocks a gate, Build logs an `FB` item and waits. | 2026-11-07 |
| `FB-04` | `U1_STATS.md` | **Accepted**: Keep `Unit.hp`, `Unit.ap`, and `poise` as typed fields with the `U1` Pool layer as a facade over them; only newly added data pools (`composure`) use the generic pool store. Recorded in `U1_STATS.md`. | 2026-11-07 |
| `FB-05` | `U1_STATS.md` | **Accepted**: Allow `tier_power` multiplier in the base stage (flat adds remain base-only); keep fixed-stat and authored-power overrides; leave flat hazard/push (`take_damage`) and ally intercepts on their paths until `A2.2`; Slice 1 of `U1.2` is an exact port with byte-identical goldens. | 2026-11-07 |
| `FB-06` | `U1_STATS.md`, `INTENT2` | **Accepted**: Keep `Unit.guard` as the numeric accumulator for Situational Mitigation (Layer 2), with guard statuses/skills feeding it; move the cap (`0.9` -> `0.75`) in a separate named balance commit. | 2026-11-07 |
| `FB-07` | `U1_STATS.md` | **Accepted**: Burden and carry overload lower `Initiative` first (and `Move Rate` only if Initiative hits floor 1); legacy fixed-stat `speed` maps to `initiative`. Land each formula change (`PERC`->crit, `KNOW`->potency, speed split) as its own commit with a `HISTORY` row. | 2026-11-07 |
| `FB-08` | `U1_STATS.md` | **Accepted**: Drop the unbuilt `corruption` pool clause from `U1.2`'s composure proof test; composure's max is the formula `KNOW x 2 + CONS` (capped at 100); add formula max, percent bands, and `within` distance filter to `U1.2`; widen/parameterize the `-10..10` pool delta clamp so `-10`/`+5` and larger pool effects work. | 2026-11-07 |
| `FB-09` | `U1_STATS.md`, `U2_CLASS.md` | **Accepted**: Represent tier in formulas via a data table of per-tier factors read as `tier_hp` and `tier_power`. | 2026-11-07 |
| `FB-10` | `U2_CLASS.md` | **Accepted**: Use two share vectors per class (`base_shares` for budget 62, `growth_shares` for +5/level) with largest-remainder rounding (ties broken by largest fraction, then stat order) so levels 1-10 reproduce existing classes and `rules_golden` holds. | 2026-11-07 |
| `FB-11` | `U2_CLASS.md` | **Accepted**: Replace `power_scale` with `tier_power` in the `U1` pipeline and `tier_hp` in the HP base formula; apply the `6..18` stat check to the class row only; keep AI skill cap on tier; build this slice after `U1` derived formulas land. | 2026-11-07 |
| `FB-12` | `U2_CLASS.md` | **Accepted**: Define static units as derived at a fixed level/budget, reuse `base_unit`, `tuning`, `named_items`, and `passives` as the override mechanism, migrate `training_dummy`, and route all unit builders (heroes, NPCs, Dojo, sim) through one function. | 2026-11-07 |
| `FB-13` | `U2_CLASS.md` | **Accepted**: Pass `stage` on `WorldRoster` (`campaign.act` when in a campaign world, `0` when no campaign/sim/host), and scale enemy XP weight (`roster()`) by the stage-resolved level. Leave `WORLDFIGHT1` held. | 2026-11-07 |
| `FB-14` | `U2_CLASS.md`, `U3` | **Accepted**: Hold the proficiency bonus slice until `U3.1` and `U4.1` close so Level 1-3 rules are built once. | 2026-11-07 |
| `FB-15` | `U2_CLASS.md` | **Accepted**: Name the module `unit_budget`; keep `base_stats` and `growth` as a computed compatibility view so `party.rs`, `forge.rs`, `sim`, and `timeline_kit.json` hold; keep loadout templates as an optional last slice. | 2026-11-07 |
| `FB-16` | `INTENT2.md`, `S1` | **Accepted**: Add a dedicated pre-Stand slice for "guard as a status feeding `Unit.guard`" aligned with `FB-06` and `S1`. | 2026-11-07 |
| `FB-17` to `FB-22` | `INTENT2*.md` | **Accepted all recommendations**: Design will update `INTENT2.md`, `INTENT2_MATRIX.md`, `INTENT2_CASES.md`, and `INTENT2_SKILLS.md` with the `IR21` precedence order, item mapping, block-limit check, and `lingering = duration + 1` rule before queueing `INTENT2` for build. | 2026-11-07 |
