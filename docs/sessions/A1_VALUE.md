# Design session A1: Value, cost, consumables and skill budget (`A1.1`, closed 2026-11-07)

Design record for `A1.1` (Skills and consumables: value, AP/HP/meter costs, blocks, recovery, stance cleanup, and advisory skill budget). Supersedes `BASERULES1-H` and `BASERULES1-B`.

## 1. Core architecture: Consumables unified with skills (`BASERULES1-H`, `INTENT2-K3`)

Consumables (`consumable_type`: damage, heal, buff) no longer bypass combat rules (`w.damage(target, n, true)` or `buff_stat` overwriting `Focus`):

- **Damage items** (bombs, thrown flasks):
  - Enter Stage 1 (`Base`) of the `U1`/`A2` damage pipeline with their flat `damage` value (`base_value`), pass through Stage 2/3 (`Flat`/`Mult`, including `splash_mult` on outer area tiles), and pass through Stage 4 (`Mitigation`: Target Armor + Situational Guard/Cover, unless the item explicitly sets `armor_piercing` or `ignore_guard`), rounding to `min 1` in Stage 5 (`Round`).
- **Heal items** (salves, draughts):
  - Enter Stage 1 (`Base`) of the `A2` heal pipeline with their flat `heal` value (or `% max_hp`) and pass through heal modifiers, rounding to `min 1` and clamping to `max_hp - current_hp`.
- **Buff items** (e.g., Brimstone Draught, `FB-21b`):
  - Grant a standard **`S1` status** (`empower`, `fortify`, `regen`, `haste`, `guard`) instead of mutating raw primary stats (`CONS`, `STR`) mid-fight, preserving the rule that standing primary/derived pools (`Max HP`, `Max Poise`) are not re-derived mid-battle.
- **Item cost, stacks, and unit symmetry (`INTENT2-K3`, `FB-21d`)**:
  - Using an item costs `item_ap = 0` AP (tunable `0..2`), paying its timeline `use_blocks` commitment and consuming `1` copy from its stack (`max_stack`, weight counted per copy in `carry.rs`).
  - Any unit with a pouch (hero or enemy) can use items through the same command path with no hero-only restriction.

## 2. Skill cost, stance leftover cleanup, and advisory skill budget (`BASERULES1-B`)

- **Skill costs & timing**:
  - `ap_cost` (explicit on the skill row under `ap_cost` rule, else skill level clamped `1..5`), `blocks` (`1..6`), `recovery_blocks` (booked after impact under `enemy_moves`), optional `hp_cost` (`hp_cost` rule), and optional `meter_cost` (`unit_meters` rule).
  - Hard equip/training gates (`weapon_types`, `min_proficiency`) remain on the skill rows (`FB-34`, `FB-36e`).
- **Retire orphaned stance fields when `stances` is off**:
  - When `stances` is `off` (the timeline default), `ends_in_stance` (`feint`), `disrupt` (`crushing_blow`), and stance tokens (`{flow}`, `{disrupt}`) are inert; rebalance affected skills (`feint`, `crushing_blow`) so no rank-1 skill is strictly dominated by basic `Strike`, and warn in `check/skill.rs` if a non-stance ruleset references stance-only tokens.
- **Advisory skill budget check (`check/skill.rs`)**:
  - A non-blocking balance warning computes a skill's effective value (`power_mult / blocks` + weighted riders for status infliction, forced move, area shape, pierce/ignore-guard) against its `ap_cost` and `blocks`, surfacing underpriced or strictly dominated skills in `aegis rules` / content checks.

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| Consumables routed through `A2` pipeline & `S1` status grants | **Fixed (Locked)** | Damage/heal via `A2`, buffs via `S1` | Eliminates rule bypass (`FB-21a,b`) |
| `item_ap` | **Tunable** | `0` (`0..2`) | AP cost to use a consumable (`INTENT2-K3`) |
| `thrown_miss_spends` / `pouch_stacks` | **Tunable** | `on` / `on` | Stackable pouch consumables |
| Advisory skill budget weights | **Tunable** | Rider weights in `check/skill.rs` | Warning-only balance check |

## 4. Feasibility resolutions (`FB-55`, accepted 2026-11-07)

1. **Immediate buff items & `empower` cap (`FB-55(1)`)**: Buff consumables apply immediately via `status::grant_now` (not next-round `queue`), and raise `empower`'s magnitude cap (or allow item-sourced magnitude up to `50 %`) so Elixir of Focus (`50 %`) and Brimstone Draught (`30 %`) are not clipped.
2. **Exact port before item mitigation (`FB-55(2),(3)`)**: Make `item_ap` a ruleset number (`1` under `classic`, `0` under `timeline`, `A1.2-S1`); port item damage through `rpgkit_rules::pipeline` matching today's unmitigated value first (`A1.2-S3`), then flip item mitigation on in its own rule-gated balance commit (`A1.2-S4`).
3. **Ruleset-aware stance tokens & flag-gated budget check (`FB-55(4),(5)`)**: Pass a `stances` flag to the skill token renderer so `{flow}` and `{disrupt}` render empty when `stances` is off without breaking `classic` (`A1.2-S2`); keep `feint` (`1` block, `0.7x`) as-is; gate the advisory skill budget check behind a flag (off by default in `checks_golden`, `A1.2-S5`).

## 5. Acceptance tests (`A1.2` - `A1.5`)

1. **Consumable pipeline parity**: Throwing a damage bomb at an armored, guarding target applies Armor and Guard mitigation (`min 1`) via `rpgkit_rules::pipeline` when item mitigation is on; drinking a buff draught grants its `S1` status immediately via `grant_now` without altering `Max HP`.
2. **Stance-off cleanliness & skill budget**: Under `stances = off`, no skill description prints `{flow}` or `{disrupt}`, and `check/skill.rs` emits advisory budget warnings when the budget flag is enabled.
