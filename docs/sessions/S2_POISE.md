# Design session S2: Pools, Poise pip thresholds, and unit meters (`S2.1`, closed 2026-11-07)

Design record for `S2.1` (Poise as a pip-threshold pool, stagger/break rules, AP overspend debt, and unit meters). Supersedes `POISE1`.

## 1. Core architecture: Strict Per-Hit Pip Threshold Poise (`Option 2A`)

While `U1` derives continuous `Max Poise` (`CONS * 2 + STR + gear_poise_bonus`, with `Unit.poise` kept as a typed field facade per `FB-04`), combat poise loss operates on **discrete Poise Pips with a per-pip damage threshold** so tanks shrug off small chip hits and only lose pips when a hit breaches their threshold:

- **Poise Pips & Pip Threshold (`T`)**:
  - Every unit has `P = poise_pips` (`5` default) discrete Poise Pips at full poise.
  - Each unit's **Pip Threshold (`T`)** is derived from their `Max Poise`:
    `T = max(1, roundi(max_poise / poise_pips))`.
    *(Example: A heavy tank with `Max Poise = 60` has `T = 12` per pip; a light unit with `Max Poise = 25` has `T = 5` per pip.)*
- **Raw Poise Damage (`D`) on a Hit**:
  - Calculated from the skill's base `stagger` and situational multipliers:
    `D = roundi(skill_stagger * windup_mult * guard_mult * flank_mult * repeat_mult * passive_mult)`.
  - **Multipliers**:
    - Target is in **wind-up**: `stagger_windup_mult` (`2.0x`).
    - Target has **active Guard** (`Unit.guard > 0`, whether from a running Defend or the `S1` `guard` status): `stagger_guard_mult` (`0.5x`).
    - Attacker **flanks** target: `flank_stagger_mult` (`1.5x`).
    - Repeat hit in same action: `stagger_repeat_mult` (`0.5x`).
- **Quantized Pip Loss Rule (`floor(D / T)`)**:
  - A hit dealing `D` raw poise damage removes:
    `pips_lost = clamp(D / T, 0, max_pip_loss_per_hit)` (integer division `floor(D / T)`, capped at `max_pip_loss_per_hit = 3`).
  - **Complete Chip Immunity (`D < T`)**: Deals **`0` Pip loss**. Multiple weak hits below `T` cannot chip-break a high-`T` tank.
  - **Solid Hit (`T <= D < 2T`)**: Breaks **`1` Pip**.
  - **Heavy / Wind-up / Flank Hit (`D >= 2T`)**: Breaks **`2` to `3` Pips**.
- **Poise Break (`0 Pips` remaining)**:
  - When remaining Poise Pips reach `0` (unless `steadfast` from `S1` is active):
    1. Cancels the target's winding-up action in the current/remaining blocks of this round.
    2. Queues the `stagger` status (`blocks [1, 4]` locked next round).
    3. Refills all `5 Pips` (`Max Poise`) at round `roll_over` so units cannot be chain-broken across consecutive rounds without refilling first.

## 2. AP overspend debt (`C10`) and unit meters (`unit_meters`)

- **AP Debt to Stagger/Stun (`C10`)**:
  - Units may overspend AP during planning; ending a round plan at `>= debt_stagger` (`3` AP debt) queues `stagger` for next round, and `>= debt_stun` (`5` AP debt) queues `stun`.
- **Unit Meters (`unit_meters`)**:
  - Secondary resource meters (`channel`, `composure`) use the `U1` generic pool store (unlocked via `U4`'s `grant_pool` passive kind per `FB-26`).
  - Crossing a `U1` pool band (`< 25%`, `< 50%`, `>= 80%`) applies its band modifiers (`FB-25`) or grants a status (`S1`).

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| Quantized pip loss (`floor(D / T)`) & break-at-0 refill | **Fixed (Locked)** | `clamp(floor(D / T), 0, max_pip_loss)` | Core `S2` anti-chip poise rule |
| `poise_pips` | **Tunable** | `5` (`3..8`) | Number of discrete poise pips per unit |
| `max_pip_loss_per_hit` | **Tunable** | `3` (`1..5`) | Cap on pips lost from a single hit |
| `stagger_windup_mult` / `stagger_guard_mult` / `flank_stagger_mult` | **Tunable** | `2.0` / `0.5` / `1.5` (`0.25..3.0`) | Situational poise damage multipliers |
| `debt_stagger` / `debt_stun` | **Tunable** | `3` / `5` (`1..10`) | AP debt thresholds queuing `stagger` / `stun` |

## 4. Acceptance tests (`S2.2` - `S2.5`)

1. **Chip-immunity proof**: A tank with `Max Poise = 60` (`5` pips, `T = 12`) hit 5 times in one round for `D = 10` poise damage each loses `0` pips (`10 < 12`) and never breaks; a single heavy wind-up flank hit dealing `D = 26` (`floor(26 / 12) = 2`) removes `2` pips.
2. **Guard halves poise damage**: Raising `Unit.guard > 0` (via Defend or `S1` `guard` status) applies `stagger_guard_mult` (`0.5x`) before `floor(D / T)`, reducing a `D = 20` hit to `10` (`0` pips lost against `T = 12`).
3. **Break & refill**: Dropping to `0` pips cancels the current wind-up, queues `stagger` for next round, and refills to `5` pips at `roll_over`.
