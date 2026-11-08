# Design session A3: Shapes, anchors and reach (`A3.1`, closed 2026-11-07)

Design record for `A3.1` (Area shapes, intent-driven anchors, `affects` filtering, reach bands, and melee vs shaped lock rules). Incorporates `INTENT2-K2` (`C12`) and `FB-19..22`.

## 1. Core architecture: 6 area shapes and intent-driven anchors (`INTENT2-K2`, `C12`)

- **Retire unused `SWEEP`; standardize on 6 shapes**:
  - `PLUS`, `BURST`, `LINE`, `CONE`, `RING` (hollow ring around origin), and `ARC` (3-tile front arc replaces `SWEEP` across the 7 shipped sweep skills and `HAZARD_SHAPES`, per `FB-17`, `FB-22`).
- **Anchor by Round Intent (`C12`)**:
  - **`Stand`**: Self-anchored (`RING`, self-centred `BURST`, or directional `ARC`/`LINE`/`CONE` from the caster's tile).
  - **`Field`**: Tile-anchored within `reach_min..reach_max` (persisting ground target or directional orientation from caster to anchor tile).
  - **`Assault`**: Unit-anchored on the round target (paying `+assault_shape_blocks` extra wind-up block and `splash_mult` on secondary tiles).
- **`affects` Target Filter (`IR9`, `FB-57(2)`)**:
  - Every shaped skill or item declares `affects`, with row-derived defaults when omitted: `all` for `Field` and ground hazards (`IR9`), `enemies` for `Assault`/`Stand` attacks, and `allies` for `Support`.

## 2. Reach bands and attack-lock interaction (`C01`, `IR17`, `FB-21h`, `FB-36c`, `FB-57`)

- **Weapon & Skill Reach**:
  - Melee weapon families read `range_band` (`min`, `max`) from `item_families.json` (replacing the hardcoded `REACH_WEAPON_REACH = 2` constant per `FB-36c`).
  - Ranged weapon families keep `base_range`, while ranged/spell skills declare their own `reach_min..reach_max` (`U1`).
- **Attack Lock vs Shaped Skills (`ATTACKLOCK1-O`, `IR17`, `FB-21h`, `FB-57(1),(2)`)**:
  - Under `attack_lock = melee`, single-target attacks with `power > 0` in melee band (`reach_min == 1` and `reach_max <= 2`) lock to the target's tile when wind-up starts (`command.rs` `lock_tile`).
  - **`Field` and directional/drawn shapes (`LINE`, `CONE`, `ARC`, `RING`)** lock to their ground tiles / direction at wind-up start (`IR17`), whereas an **`Assault`-row shaped skill (`BURST`/`PLUS` aimed at a unit)** tracks and centres where the target stands at impact (`IR17`).
  - Add `RING` and `ARC` alongside `SWEEP` first (keeping `SWEEP` as a legacy alias under `classic` so the frozen kit's brawler does not change `classic` goldens, `FB-57(1)`), and remove `SWEEP` last when the frozen kit migrates.

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| 6 area shapes (`PLUS`, `BURST`, `LINE`, `CONE`, `RING`, `ARC`) & intent anchor rules | **Fixed (Locked)** | `RING`/`ARC` added first; `SWEEP` retired after frozen kit migration | Core `A3` spatial vocabulary |
| `splash_mult` | **Tunable** | `0.50` (`0.25..1.00`) | Secondary-tile multiplier when a shape is used under `Assault` (`IR17`, `FB-120`) |
| `assault_shape_blocks` | **Tunable** | `1` (`0..2`) | Extra wind-up blocks when aiming a shaped skill at a moving unit (`Assault`) |
| `attack_lock` | **Tunable** | `melee` (`none` \| `melee` \| `all`) | Unblocks `ATTACKLOCK1-O` switch-on |

## 4. Acceptance tests (`A3.2` - `A3.5`)

1. **Additive `RING` and `ARC` migration (`INTENT2-K2`, `FB-57`, `FB-114`)**: Phase A lands `affects`, `IR7` snap, `CONE`, and `RING`/`ARC` in the schema with `0` battle/sim golden diffs (`checks_golden` re-recorded for 2 `area` enum cases); Phases B-C migrate the 7 `SWEEP` skills in both `skills.json` and `timeline_kit.json` (plus `cast1a3.json`), retire `SWEEP`, and re-record `sim_golden` (as brawler kicks `sweep_kick` / `cyclone_kick` become playable in `classic` too) and `checks_golden`.
2. **Block-limit check (`FB-20`)**: `check/skill.rs` verifies that `blocks + recovery_blocks + assault_shape_blocks <= MAX_ACTION_BLOCKS (6)` across every rank row of dual `Field`/`Assault` skills (`Pulverizer`, `Cataclysm`, `Bloom`).
