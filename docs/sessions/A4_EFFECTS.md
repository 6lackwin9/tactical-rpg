# Design session A4: Unified status/effect schema and intent derivation (`A4.1`, closed 2026-11-07)

Design record for `A4.1` (Unified status/effect application across all 5 vocabularies, triggers/exploits, and deterministic `IR21` intent derivation). Incorporates `C08` and `FB-16` to `FB-22`.

## 1. Core architecture: One status/effect application schema (`C08`)

Statuses are currently applied through 5 separate schemas: skill `inflict`/`grant`, trigger `inflict`/`grant`, ground hazard rows, arena hazard effects, and encounter phase `status` effects.

- **Unified `EffectSpec` (`C08`, `FB-57(5)`, `FB-58`)**:
  - All 5 vocabularies normalize at load/execution into a single `EffectSpec { status_id, potency, rounds, magnitude, resistable: Option<bool>, source }` routed through `S1` (built after `S1.2-S1`'s status table):
    - When `resistable` resolves to `true` (defaulting to the status row's `resistable` for skills/hazards and `false` for encounter phase `status` effects so no extra RNG draw occurs on phases per `FB-58`), calls `S1::try_inflict`.
    - Buffs and non-resistable effects call `S1::grant_now` or `S1::queue` (using round-granular timing from `S1.1`).
- **Triggers, Riders, and Exploits**:
  - Passive (`U4`), item (`U3`), and skill (`A4`) triggers share the 15-event / 10-condition / 14-effect vocabulary (`trigger.rs`), preserving `source#index` trigger limit keys (`FB-30`).
  - Exploits (`flanked`, `behind`, `hp_below` execute) feed Stage 3 (`Mult`) of the `U1`/`A2` pipeline.

## 2. Deterministic `IR21` intent derivation (`INTENT2-K1`, `FB-17` to `FB-22`, `FB-57(3),(4)`)

One derivation function in `rpgkit_rules` replaces the three diverging copies (`round_intent::skill_intent`, `skill::intent_id`, `check`'s `intent_of`), returning up to 2 allowed intents (the first is the primary intent, `IR15`) and the derived `Field` bonus:

- **Skill Intent Precedence Order (`IR21`, `FB-19`, `FB-57(3)`)**:
  1. **Engine kinds (`MOVE`, `REST`, `TARGET`, `STATUS`)**: Neutral (no round intent required).
  2. **`DEFEND` and `FOCUS`**: `Stand` (`hazard` on a `DEFEND` such as `ash_cloud_fade` stays `Stand`).
  3. **`DISENGAGE`**: `Maneuver`.
  4. **`SUPPORT`**: `Support` (plus `Stand` when self-targetable).
  5. **`ATTACK` with `dash_tiles` or `position`**: `Maneuver` (`Lunge`, `Pounce`, and `Charge In` derive `Maneuver`, per `FB-17`; `skill::position_by` reads the derived intent).
  6. **`ATTACK` with `area` shape (`PLUS`, `BURST`, `LINE`, `CONE`, `RING`, `ARC`)**:
     - `RING` -> `Stand`.
     - `ARC`, `LINE`, and `CONE` (without `hazard`), and `BURST` / `PLUS` (with `reach_max > 1` and no `hazard`) -> `Field` and `Assault` (otherwise `Field`, e.g., when a shape places a ground `hazard`).
     - In `Field` mode, `ARC` and `LINE` are ground-aimed (`WASD` / source tile) and receive the `area` `Field` bonus (`ARC 3 -> 5` wide, `LINE 3 -> 4` long); in `Assault` mode, their direction is locked to the picked enemy target at impact (`IR7`), using base shape size, `foes only` (`IR9`), `splash_mult` off the primary target, and `+assault_shape_blocks` (`+1` wind-up block, capped at `MAX_ACTION_BLOCKS = 6`).
  7. **`ATTACK` with `hazard` and no shape (`tripwire_snare`)**: `Field` (must precede the unshaped `Assault` rule, `FB-19`).
  8. **Unshaped `ATTACK`**: `Assault`.
- **Consumable Item Intent Mapping (`FB-19`)**:
  - Heal item -> `Support` and `Stand`.
  - Buff item -> `Stand`.
  - Damage item with `area` and `use_reach > 1` -> `Field` and `Assault`.
  - Single-target damage item -> `Assault`.
- **Derived `Field` Bonus (`IR20`, `FB-19`, `FB-22`)**:
  - Derived automatically from the skill's properties (no authored `field_bonus` field): `reach` (`+1` `reach_max`, 11 skills), `lingering` (`hazard.rounds = row_duration + 1` capped at `5`, 5 skills), `area` (`+1` radius/length, 7 skills), `power` (reserved future default, 0 current skills).
- **Row-aware planner & checks (`FB-19`, `FB-20`)**:
  - Planner picks carry their chosen intent row so dual-intent skills resolve the picked row's version (`Field` vs `Assault` extra block/splash); `deck_errors` and ~15 intent equality sites check `row_allows` instead of single-intent equality.

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| `IR21` precedence order, item intent mapping & `EffectSpec` | **Fixed (Locked)** | 7-step precedence in `rpgkit_rules` | Core `A4` contract |
| `field_bonus_values` (`reach`, `lingering`, `area`, `power`) | **Tunable** | `+1` reach, `+1` round (cap 5), `+1` size, `1.15x` power | Derived Field bonuses |
| `field_fill_once` | **Tunable** | `on` | Planner fills a Field skill once per round |

## 4. Acceptance tests (`A4.2` - `A4.5`)

1. **Single derivation function (`FB-19`)**: Running the derivation function across all 129 skills after the 7 `SWEEP -> ARC/RING` data fixes yields a deterministic 1-or-2 intent list for every skill (`Assault` 59, `Maneuver` 14, 26 dual-intent skills: 8 `Support, Stand` + 18 `Field, Assault`; `11` reach, `5` lingering, `7` area, `0` power Field bonuses).
2. **Unified `EffectSpec` (`C08`)**: Statuses applied from skills, triggers, ground hazards, arena hazards, and encounter phases all route through `S1` using the same resistance and stacking rules.
