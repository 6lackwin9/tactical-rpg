# Design session A2: Dynamic hit/heal resolution and re-derivation triggers (`A2.1`, closed 2026-11-07)

Design record for `A2.1` (Dynamic combat evaluation of the 5-stage `U1` pipeline for hits and heals, plus explicit mid-battle re-derivation triggers). Connects `U1` (`C06`) and `A1` (`C07`).

## 1. Core architecture: Dynamic hit and heal evaluation (`Base -> Flat -> Mult -> Mitigation -> Round`)

`U1.2-S1` (`rpgkit_rules::pipeline`) established the canonical 5-stage order (`Stage::ORDER`: `Base`, `Flat`, `Mult`, `Mitigation`, `Round`). `A2` evaluates every hit, heal, and direct damage effect (`take_damage` per `FB-43`) through this pipeline with a per-hit `Hit.trace`:

- **Damage Hit Resolution**:
  1. **`Base`**: `(Weapon Power + Proficiency Level + roundi(Scaling Stat * Proficiency Scaling)) * tier_power` (rounded at the attack-power point, or replaced by authored unit `power` or consumable/hazard flat `base_value`).
  2. **`Flat`**: Additive flat damage modifiers (`Op::Add`, applied after attack power's rounding per `FB-42`).
  3. **`Mult`**: Multiplicative amplifiers (`Op::Mul`): `Skill Power Mult`, `Empower`/`Weaken` (`S1`), `Focus`, `Stance` (when `stances = on`), `Exploit` (`flanked` `1.25x`, `hp_below` execute), `Crit` (`crit_mult = 1.5x`, 1 RNG draw per hit), `Splash/Field Mult` (`splash_mult`), and `Riposte` (`counter_mult = 0.5x`).
  4. **`Mitigation`**:
     - **Layer 1 (Rating Mitigation)**: `Armor = R / (R + armor_k)` (`armor_k = 50`), where `R = max(0, base_armor + fortify - expose)`. Skipped when `pierce` rule and skill `armor_piercing` are active.
     - **Layer 2 (Situational Mitigation)**: `Situational = min(guard_cap, max(running_defend, guard_status) + ranged_cover)`. Guard term is skipped when `ignore_guard` is active (ranged cover is kept).
  5. **`Round`**: `max(1, roundi(result))` (clamped to target `current_hp` when applied).
- **Heal Resolution (`BASERULES1-H`)**:
  1. **`Base`**: `Skill Heal Base + KNOW * Skill Heal Mult` (or consumable flat/`% max_hp` heal base).
  2. **`Flat` & `Mult`**: Additive and multiplicative outgoing/incoming heal modifiers (`1 + KNOW_Potency + passive_heal_mult`).
  3. **`Round`**: `max(1, roundi(result))`, clamped to `max_hp - current_hp`.

## 2. Mid-fight re-derivation triggers (`U1.2` <-> `A2.2`)

To keep combat state deterministic and avoid mid-fight max-pool shifts (`FB-21b`):

- **Never re-derived mid-fight**: Primary stats (`STR`, `DEX`, `INT`, `CONS`, `PERC`, `KNOW`) and standing Pool maximums (`Max HP`, `Max Poise`, `Max AP`).
- **Dynamic combat ratings (`Armor Rating`, `Move Rate`, `Initiative`, `Unit.guard`, and active pool-band modifiers)** re-evaluate **only on 3 explicit triggers**:
  1. **Round `roll_over`**: Active statuses tick down, queued next-round statuses activate, and Poise refills to `5` pips (`Max Poise`).
  2. **Mid-round status/guard change**: A Guard skill starts its wind-up (`Option 1C` in `S1.1`), a unit is Poise-broken (`0` pips drops active guard and cancels wind-up), or an `intent_bound` status (`stealth`, `riposte`) is dropped.
  3. **Pool band crossing**: `HP` or a secondary unit meter (`channel`, `composure`) crosses a declared band threshold (`< 25 %`, `< 50 %`, `>= 80 %`).

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| 5-stage pipeline order & 3 re-derivation triggers | **Fixed (Locked)** | `Base -> Flat -> Mult -> Mitigation -> Round` | Core `U1`/`A2` contract |
| `armor_k` | **Tunable** | `50` (`10..200`) | Armor rating curve constant `R / (R + K)` |
| `guard_cap` | **Tunable** | `0.90` -> `0.75` (`0.30..0.95`) | Situational mitigation cap (`FB-06` balance commit) |
| `crit_mult` / `counter_mult` / `flank_power_mult` | **Tunable** | `1.50` / `0.50` / `1.25` (`0.10..3.00`) | Stage 3 (`Mult`) combat multipliers |

## 4. Feasibility resolutions (`FB-56`, accepted 2026-11-07)

1. **Exact ports & trace-only folded factors (`FB-56`)**:
   - `A2.2-S1`: Port `take_damage` (`unit.rs:1091`, keeping `min 1` only when `guard > 0` initially), `heal`, and `heal_amount` (`heal_base + roundi(stat * mult)`) onto `rpgkit_rules::pipeline` with `0` golden diffs.
   - `A2.2-S2`: Record the pre-multiplied `skill_mult` components (`flank`, `splash`, `counter`) as trace-only entries on `Hit.trace` rather than splitting the `f64` multiplication order, preserving exact rounding across all 1,652 golden rows.
   - `A2.2-S3`: Wire pool-band modifier re-evaluation after `U1` bands land; land the universal `min 1` floor and `guard_cap` `0.90 -> 0.75` as separate, named balance commits.

## 5. Acceptance tests (`A2.2` - `A2.5`)

1. **Exact port & `take_damage` unification (`A2.2-S1`, `FB-43`, `FB-56`)**: Routing hits, heals, and `take_damage` through `rpgkit_rules::pipeline` keeps existing `rules_golden`, `timeline_golden`, and `sim_golden` byte-identical before balance parameter commits.
2. **Re-derivation trigger isolation**: Applying `fortify`/`expose` or crossing a `< 50 %` pool band immediately updates `Armor Rating` / band modifiers on the next hit without changing `Max HP` or `Max Poise`.
