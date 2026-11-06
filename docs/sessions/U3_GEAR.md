# Session U3: gear, proficiency and carry

MASTERPLAN1 sub-block U3. Interactive design session with the user, 2026-11-07. Status: design closed (user 2026-11-07).

## Decided (user 2026-11-07, all defaults approved)

- **Gear as a bundle of `U1` modifiers and action tags.** Every equipped item emits standard `U1` modifiers (`Weapon Power`, `Armour`, `Initiative`, `Move Rate`, `Crit`, `Resist`, `Potency`, `Max HP`, `Max Poise`, or conditional 4-stage pipeline modifiers) plus weapon reach/family tags read by `A1`/`A3` and optional `grant_skill` / passive grants read by `U4`.
- **Slots and hands (reuse built model).** Six equipment slots (`main_hand`, `off_hand`, `head`, `body`, `accessory_1`, `accessory_2`) and a 2-hand capacity (a 2H weapon uses both hands; 1H main + off-hand uses 1 each). Unarmed defaults to the implicit `fist` weapon from `U1` when `main_hand` is empty.
- **Drop `block_cost` from gear.** `block_cost` was priced in the item budget (`budget.rs`) but never read by combat. Under `U1`/`A1`, wind-up block speed belongs to the skill (`A1`), while heavy gear pays through weight and burden (`Initiative` / `Move Rate`). Dropping `block_cost` removes a dead field and makes the item budget honest.
- **Static vs budgeted items.** Like `U2` units, an item may be **Budgeted** (`budget = sum(stat_value * weight) * tier_mult`, base target `23 +/- 20 %`) or **Static** (explicit values for unique, boss, or keepsake items; explicit overrides always win).
- **Gear families in data (no hardcoded family IDs in kit rules).** Instead of hardcoding `fist`, `bow`, `crossbow`, `sling`, `shield`, `tome`, `staff`, or `wand` in Rust, each gear family row in data declares its properties: `slot`, `hands` (`1` or `2`), `range_band` (`min`, `max`), `scaling_stats` (weights for `U1` weapon power), and its Level-3 proficiency mastery effect.
- **Proficiency (`0-3`) and soft wield penalty (`FB-14`).**
  - **Level 0 (Untrained) and unmet stat requirements**: Soft penalty, not a hard equip lock. Any unit may equip any item (retiring the `wears_untrained` exception flag), but equipping at Proficiency 0 or below an item's stat requirement applies a tunable **wield penalty** (`untrained_mult`, default `-25 %` to the item's `Weapon Power` / `Armour`, plus `+1` burden per missing proficiency or stat point).
  - **Level 1 (Trained)**: Full item stats (no wield penalty), base requirement discount (`level x 2`), and access to tier-1 family skills.
  - **Level 2 (Adept)**: Requirement discount (`level x 2`), secondary bonus (`+crit` or `-burden` from the family row), and access to tier-2 family skills.
  - **Level 3 (Master)**: Requirement discount (`level x 2`), Level-2 bonus, plus the family's data-driven **Level-3 mastery modifier/passive** (`U4`), and access to tier-3 family skills.
- **Carry and burden (`C01`).** Carry capacity is a `U1` derived rating (`STR x 3 + CONS x 1`, off by default per `U1`, tunable when enabled). Equipped burden and carry overload lower `Initiative` first (and `Move Rate` only if `Initiative` reaches its floor of 1, per `FB-07`).
- **`OFFHAND1` (Off-hand skill items) included as data.** Off-hand items (bucklers, torches, parrying daggers, foci) grant their signature skill while worn via the standard `grant_skill` hook (`U4`), needing no bespoke off-hand code path.

## Fixed vs parameter (`U3`)

| Item | Fixed (kit behaviour) | Parameter (data), with default |
| --- | --- | --- |
| Slots and hands | 6 slots, 2 hands, implicit `fist` when `main_hand` empty | Slot and hand cost per gear family |
| Gear families | Data table drives reach, scaling stats, slot, hands, Level-3 bonus; no family ID hardcoded in kit rules | Family rows (`range_band`, `scaling_stats`, `level2_bonus`, `level3_passive`) |
| Item budget | `budget = sum(stat_value * weight) * tier_mult`; static override for unique/keepsake items; `block_cost` removed | Base item budget (`23`), tolerance (`+/- 20 %`), stat weights, tier multipliers |
| Proficiency (`0-3`) | 4 levels; soft wield penalty at Level 0 or below stat requirement; requirement discount `level x N`; Level-3 grants family mastery passive | `req_discount_per_level` (`2`), `untrained_power_mult` (`0.75`), `untrained_burden_add` (`1`), per-family Level 2/3 bonuses |
| Carry & burden | Burden and carry overload lower `Initiative` first, then `Move Rate` at Initiative floor 1 | Carry formula (`STR x 3 + CONS`), on/off switch (off by default) |
| Off-hand items (`OFFHAND1`) | Worn item grants skills/passives through `U4` | Which skill/passive each off-hand item grants |

## Acceptance test for `U3.2`

1. No gear family ID (`bow`, `shield`, `staff`, etc.) is hardcoded in `rpgkit_rules`; adding a new weapon family in data using existing `engine_rules`, a custom scaling stat, range band, and Level-3 mastery bonus works with zero Rust changes.
2. Equipping an untrained weapon or an item above the unit's stat requirement succeeds without `wears_untrained` and applies the configured wield penalty to `Weapon Power` / `Armour` and `Initiative`.
3. `block_cost` is removed from the gear budget check without breaking existing item saves/resaves.

## Feasibility resolutions (`FB-31` to `FB-37`, accepted 2026-11-07)

- **`FB-31` & `FB-32` (`item_families.json` & `weapon_type`)**: Keep the field name `weapon_type` and `item_families.json`. "Family migration" means wiring the remaining hardcoded sites (`ALWAYS_RANGED_TYPES`, `off_hand_ok`, `is_shield`, `skill_in_group`) and deriving `GEAR_TYPES`, `WEAPON_TYPES`, `OFF_CATEGORIES`, and `FOCUS_TYPES` in `check/` from `item_families.json` and `engine_rules`. A new `engine_rule` remains a code change.
- **`FB-33` & `FB-34` (Wield penalty, `+level` power, unarmed, and party screen)**:
  - **Stat deficit**: Keep the existing `15 %` power penalty per missing stat point (capped at `60 %`, after `-2` requirement discount per proficiency level) and extend it to armour as well.
  - **Proficiency 0 (Untrained)**: Soft penalty replaces the hard equip block (`0.75x` power/armour multiplier + `+1` burden when `level == 0`, stacking multiplicatively with any stat-deficit penalty). Keep `+level` weapon power for trained levels (`1..3`).
  - **Unarmed (`fist`) & classless units**: Keep unarmed weapon power at `0` unless `fist` proficiency is `>= 2` (then `15 + level`), and keep "no proficiency list on a unit/class means no restriction" so classless/hand-built units are not penalised.
  - **Skill gates & UI**: Skill `min_proficiency` and `weapon_types` gates stay hard; include the party-screen penalty badge (mocked first) in Slice 3.
- **`FB-35` (Reversible item grants & `OFFHAND1`)**: Recompute item grants and `U4` sources on every sync inside the shared `FB-12` builder. Worn item skills (`OFFHAND1`) are always active while worn, take no deck slot, and follow the built hand rule.
- **`FB-36` (Hands, loadout budget, reach, carry, skill gating)**:
  - **(a) Hands**: Keep the built 3-hand rule (`main_1h`, `main_2h`, `off`, where `main_2h` may pair with a non-focus off-hand weapon via weapon switch; shields and foci are excluded).
  - **(b) Budget**: `23 +/- 20 %` is the **class loadout** budget target (`budget.rs:19`); individual items use `special` and `special_note` as their static override.
  - **(c) Reach**: `range_band.max` on the family replaces the hardcoded `REACH_WEAPON_REACH = 2` constant, while ranged families keep `base_range` and `U1` moves ranged skill reach to the skill.
  - **(d) Carry**: Party-wide carry (`carry.rs`) stays alongside per-unit burden -> `Initiative` (`U1` speed split).
  - **(e) Skill gating**: Expressed as data on the skill rows (`min_proficiency`, `weapon_types`, path level).
- **`FB-37` (5 slices)**: Slice 1 (drop `block_cost`, re-record `checks_golden`) and Slice 2 (`item_families.json` drives all rules, `0` golden diffs) are ready to build now; Slice 3 (soft penalty + UI), Slice 4 (reversible item grants + `OFFHAND1` + Level 2-3 bonuses), and Slice 5 (carry/burden speed split) follow `U1.2` and `U4.2`.
