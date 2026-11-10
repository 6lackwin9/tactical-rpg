# Design session S1: Status class and Guard status (`S1.1`, closed 2026-11-07)

Design record for `S1.1` (Status class: behaviours, properties, the 17 existing statuses plus `guard` as data rows, and single status resistance). Supersedes `STATUSROWS1` and `BASERULES1-G`.

## 1. Core architecture: 18 statuses as data over 6 composable behaviours

Instead of hardcoding `ROWS: [StatusRow; 17]` and per-status branches across `status.rs`, `unit.rs`, `command.rs`, and `resolver.rs`, every status is a data row (`statuses.json` or ruleset catalog) combining a shared **Lifecycle header** with one or more of **6 composable behaviour primitives**:

- **Lifecycle header**: `id`, `display`, `kind` (`buff` | `debuff`), `resistable: bool`, `default_rounds: i32`, `stacking` (`refresh_max` | `extend_duration` | `stack_potency` | `none`), `cap: i32`, `magnitude_min..max`, `magnitude_unit` (`pct` | `flat` | `pct_max_hp`), `intent_bound: bool` (clears if unit leaves the round intent it was granted under, e.g., `riposte`, `stealth`).
- **Round-aligned timing (no mid-round or cross-round block tracking)**:
  - All status durations are **round-granular** and tick down at round `roll_over`.
  - **Guard timing (`Option 1C`, wind-up start + full-round roll-over)**: In the round a unit uses a Guard/Defend skill, `Unit.guard` activates as soon as the Guard skill **starts its wind-up** (Block 1 if opened with Guard, or the skill's start block after a step, matching `resolver.rs:114`) and lasts through **Block 6 of that round** (unless Poise-broken or Stunned). Its impact block fires any secondary rider (taunt, bash, counter, or queuing a next-round `guard` status). If a skill grants a **multi-round guard** (`rounds >= 2`, e.g., `brace` / `bulwark_stance` for `INTENT2-A`), Round 1 is covered from wind-up start to Round 1 end, and Round 2+ receives the standard full-round `guard` status (Block 1 to Block 6), avoiding any mid-round buff expiration or cross-round block counters.
- **6 Behaviour primitives**:
  1. **`modifier` (`U1` 4-stage pipeline, `C09`)**:
     - `weaken` (`-mag%` outgoing attack mult), `empower` (`+mag%` outgoing attack mult).
     - `fortify` (`+mag` flat Armor Rating), `expose` (`-mag` flat Armor Rating).
     - `haste` (`+1` flat Move Rate), `slow` (`-1` flat Move Rate).
     - **`guard` (`FB-06`, `FB-16`, `FB-18`)**: sets Situational Mitigation (`Unit.guard`). Multiple guard sources (`Defend` action, 1-round Stand guard, multi-round `guard` status) do **not** stack with each other: `guard_mitigation = max(running_defend, max_active_guard_status)`. Ranged cover adds to `guard_mitigation` up to `guard_cap` (`0.90`, kept in `R2-CLOSE` per `FB-136`).
  2. **`tick` (DoT / HoT)**:
     - `poison` (`flat` HP at round end), `burn` (`pct_max_hp` at round end), `regen` (`+pct_max_hp` heal at round end), `bleed` (`6% max_hp` on move/action or round end).
     - **Pipeline rule**: Status ticks bypass Armor and Situational Guard, respecting only `min 1`.
  3. **`block_lock` (`C10`)**:
     - `stagger` (locks timeline blocks `1` and `4` — discrete block list `[1, 4]`, leaving blocks `2, 3, 5, 6` open), `stun` (locks timeline blocks `1` and `2` — discrete list `[1, 2]`), `root` (blocks `Move` commands and stops chase when `root_stops_chase` is on).
  4. **`targeting` (`C16`)**:
     - `taunt` (biases enemy AI targeting toward the taunting unit), `stealth` (`intent_bound: true`, untargetable by single-target enemy skills; breaks on attacking or taking direct damage).
  5. **`reactive` (`A4` trigger)**:
     - `riposte` (`intent_bound: true`, counters melee hits at `counter_mult = 0.5` while active).
  6. **`rule_flag` (`S1`/`S2` immunity)**:
     - `steadfast` (`cannot_be_broken`: prevents Poise from dropping to 0 / triggering a break).

## 2. Single unified status resistance (`C06`, `C08`)

- One resistance formula across all resistable debuffs:
  `chance = clamp(skill_potency + attacker_know_potency - target_int_resist - target_status_resist(id), 0, 100)`.
- Per-element resistance matrices remain parked (`BASERULES1_LIBRARY.md` section 6); per-status resistance bonuses from `U4` passives (`resist_for(status_id)`) add directly to `target_status_resist(id)`.

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| 6 status behaviour primitives & round-granular lifecycle | **Fixed (Locked)** | `modifier`, `tick`, `block_lock`, `targeting`, `reactive`, `rule_flag` | Core `S1` schema |
| `bleed_pct` | **Tunable** | `0.06` (`0.01..0.25`) | Fraction of `max_hp` dealt by `bleed` |
| `counter_mult` | **Tunable** | `0.50` (`0.10..1.50`) | `riposte` damage multiplier |
| `guard_common_mitigation` / `guard_multi_round_mitigation` | **Tunable** | `0.35` / `0.20` (`0.10..0.90`, per skill) | Mitigation fractions (`1 - guard`), with multi-round (`brace`) lower than 1-round (`FB-48`) |

## 4. Feasibility resolutions (`FB-48`, accepted 2026-11-07)

1. **Guard mitigation fractions & `HeldGuard` mechanism (`FB-48(1),(2)`)**: Express guard values as mitigation fractions per skill (e.g., `0.15..0.35`, with multi-round guard lower than 1-round common guard). Implement `Option 1C` (wind-up start through Block 6) on the existing `HeldGuard { mitigation, last_block }` mechanism with `Unit.guard` kept as the accumulator, removing the guard instance on Poise break and exposing active guard to the AI so it avoids redundant Defends.
2. **Frozen kit status table & behaviour reads (`FB-48(3)-(6)`)**: Load the 17+1 rows as a frozen kit table behind the existing `cap_of`/`stacking_of`/`display_name`/`row_of` signatures first (`S1.2-S1`, neutral); convert ID literals to primitive reads (`S1.2-S2`, neutral); promote `BLEED_PCT`, `DEBT_STAGGER`, `DEBT_STUN`, and `COUNTER_MULT` to ruleset numbers and honour `resistable` in `try_inflict` (`S1.2-S3`); route the trigger `resist` effect through `U4`'s combine rule; keep `stealth` (moved-this-round hide) and `bleed` (round-end tick) at today's behaviour until named balance commits.

## 5. Acceptance tests (`S1.2` - `S1.5`)

1. **Slice 1 exact port (`S1.2-S1`)**: Loading the 17 existing statuses through the frozen kit status table behind the existing function signatures produces `0` diffs on `rules_golden`, `timeline_golden`, and `sim_golden`.
2. **Guard status & wind-up timing (`S1.2-S4`, `FB-06`, `FB-16`, `FB-48`)**: A 2-block Guard skill started at Block 2 raises `Unit.guard` via `HeldGuard` from Block 2 through Block 6; a multi-round Guard skill (`brace`) queues the full-round `guard` status for Round 2+ at the lower mitigation fraction (`0.20`), and `max(running_defend, guard_status)` prevents double-guard stacking.
3. **Level-3 silo limit test (`S1.5`)**: Sweeping `INT` (`Status Resist`) from `6` to `20` monotonically lowers infliction rate across every `resistable: true` status in the catalog.
