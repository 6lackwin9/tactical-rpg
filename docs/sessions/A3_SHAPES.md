# Design session A3: Shapes, anchors and reach (`A3.1`, closed 2026-11-07)

Design record for `A3.1` (Area shapes, intent-driven anchors, `affects` filtering, reach bands, and melee vs shaped lock rules). Incorporates `INTENT2-K2` (`C12`) and `FB-19..22`.

## 1. Core architecture: 6 area shapes and intent-driven anchors (`INTENT2-K2`, `C12`)

- **Retire unused `SWEEP`; standardize on 6 shapes**:
  - `PLUS`, `BURST`, `LINE`, `CONE`, `RING` (hollow ring around origin), and `ARC` (3-tile front arc replaces `SWEEP` across the 7 shipped sweep skills and `HAZARD_SHAPES`, per `FB-17`, `FB-22`).
- **Anchor by Round Intent (`C12`)**:
  - **`Stand`**: Self-anchored (`RING`, self-centred `BURST`, or directional `ARC`/`LINE`/`CONE` from the caster's tile).
  - **`Field`**: Tile-anchored within `reach_min..reach_max` (persisting ground target or directional orientation from caster to anchor tile).
  - **`Assault`**: Unit-anchored on the round target (paying `+assault_shape_blocks` extra wind-up block and `splash_mult` on secondary tiles).
- **`affects` Target Filter**:
  - Every shaped skill or item declares `affects`: `enemies` (default for attacks), `allies` (default for support), or `all` (hazards and volatile blasts).

## 2. Reach bands and attack-lock interaction (`C01`, `FB-21h`, `FB-36c`)

- **Weapon & Skill Reach**:
  - Melee weapon families read `range_band` (`min`, `max`) from `item_families.json` (replacing the hardcoded `REACH_WEAPON_REACH = 2` constant per `FB-36c`).
  - Ranged weapon families keep `base_range`, while ranged/spell skills declare their own `reach_min..reach_max` (`U1`).
- **Attack Lock vs Shaped Skills (`ATTACKLOCK1-O`, `FB-21h`)**:
  - Under `attack_lock = melee`, single-target attacks with `power > 0` in melee band (`reach_min == 1` and `reach_max <= 2`) lock to the target's tile when wind-up starts (`command.rs` `lock_tile`).
  - Shaped skills (`PLUS`, `BURST`, `LINE`, `CONE`, `RING`, `ARC`) always lock to their ground tiles at wind-up start (`IR17`), so minor differences between melee band (`1..2`) and thrown reach (`> 1`) on shaped skills (`Transfusion Pulse`, `Cataclysm`) never conflict.

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| 6 area shapes (`PLUS`, `BURST`, `LINE`, `CONE`, `RING`, `ARC`) & intent anchor rules | **Fixed (Locked)** | `SWEEP` replaced by `RING`/`ARC` | Core `A3` spatial vocabulary |
| `splash_mult` | **Tunable** | `0.75` (`0.25..1.00`) | Secondary-tile multiplier when a shape is used under `Assault` |
| `assault_shape_blocks` | **Tunable** | `1` (`0..2`) | Extra wind-up blocks when aiming a shaped skill at a moving unit (`Assault`) |
| `attack_lock` | **Tunable** | `melee` (`none` \| `melee` \| `all`) | Unblocks `ATTACKLOCK1-O` switch-on |

## 4. Acceptance tests (`A3.2` - `A3.5`)

1. **`RING` and `ARC` migration (`INTENT2-K2`)**: All 7 former `SWEEP` skills and `HAZARD_SHAPES` resolve as `ARC` or `RING` with `affects` filtering; named `checks_golden` re-record for the `area` enum (`2` cases per `FB-22`), with `timeline_golden`, `sim_golden`, and `rules_golden` unchanged.
2. **Block-limit check (`FB-20`)**: `check/skill.rs` verifies that `blocks + recovery_blocks + assault_shape_blocks <= MAX_ACTION_BLOCKS (6)` across every rank row of dual `Field`/`Assault` skills (`Pulverizer`, `Cataclysm`, `Bloom`).
