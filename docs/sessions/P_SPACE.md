# Design session P: Space, terrain, hazards and movement (`P1.1-P2.1`, closed 2026-11-07)

Design record for the Space block (`P1.1` Board, terrain & hazards, `P2.1` Movement & position). Eliminates the hardcoded terrain walkability special case in `grid.rs:68`.

## 1. `P1.1` — Board, terrain and ground hazards (`C14`)

- **Data-Driven Terrain Walkability (Fixes `grid.rs:68` Special Case)**:
  - Currently, `grid.rs:68` hardcodes walkability by terrain index.
  - **Fix**: Add `walkable: bool` (default `true`, `false` for walls/water/deep chasms) and `stops_forced_move: bool` to the terrain data row (`terrain.json`), so no terrain index or ID is special-cased in Rust.
- **Unified Cell Schema across Terrain and Ground Hazards (`C14`)**:
  - Both permanent terrain (`terrain.json`) and temporary ground hazards (`hazards.json`, placed by skills, Defends, or consumables under `ground_hazards`) share the same spatial cell properties:
    - `walkable: bool`, `move_cost: i32` (`1..3`), `cover_guard: f64` (`0.0..0.5`, ranged hits only, feeds `A2` Layer 2 Situational Mitigation).
    - `on_enter` and `round_end` effects expressed via `A4.1`'s unified `EffectSpec` (status infliction/grant + optional damage routed through `rpgkit_rules::pipeline` `take_damage` per `FB-43`).
    - Hazard-only lifecycle fields: `duration` (`rounds`, extended by `+1` under `Field` `lingering` bonus per `FB-22`), `trap: bool`, `hidden: bool`, `affects` (`enemies` | `allies` | `all`), and `stop_moves: bool`.
  - Scheduled arena hazards (`arena_hazards` rule) strike at block end using the same `EffectSpec` and `take_damage` pipeline.

## 2. `P2.1` — Movement, positioning, flanks and forced movement (`C09`, `C12`, `C13`)

- **Move Rate & Opener/After-Move Caps (`C09`, `FB-07`, `FB-40b`)**:
  - `Move Rate` (`1..4` tiles/block) is a `U1` Rating derived from `DEX` plus `S1` `haste`/`slow` modifiers (normal gear/carry burden lowers `Initiative` first and only touches `Move Rate` if `Initiative` hits floor `1`, per `FB-07`).
  - Opener `Move` takes up to `MAX_MOVE_BLOCKS = 2`; post-skill `Move` takes up to `move_after_blocks = 3` (promoted to `kits/rpg/schema.json` in the `FB-40b` schema slice), both at `1` AP per tile.
- **Flanks, Reach, and Maneuver Dash (`C12`, `C13`)**:
  - Under `round_intent` and `flank`, a unit engaged with an enemy exposes side and back tiles; attacks from those tiles trigger `flanked` (`flank_power_mult = 1.25`, `flank_stagger_mult = 1.5`, no riposte).
  - `Maneuver` skills with `position` (`flank` | `behind`) or `dash_tiles` dash up to `reach_max` (plus `1` chase tile if the target moved) and strike from that tile (`melee_diagonal` `none` | `maneuver` | `all`).
- **Forced Movement (`forced_move`)**:
  - `push` and `pull` move damaged targets up to `distance + forced_move_bonus_tiles` (cap `4`), respecting `S1`/`U4` forced-move immunity (`or_flag` per `FB-26`).
  - Blocking terrain (`!walkable` or `stops_forced_move`) or another unit triggers a collision (`collide_mult * attacker_power` through `take_damage`, plus `collide_stagger` poise damage and `collide_break` on wind-ups).

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| Terrain `walkable` & hazard cell schema in data | **Fixed (Locked)** | `walkable`, `move_cost`, `cover_guard`, `EffectSpec` | Removes `grid.rs:68` hardcode |
| `max_move_blocks` / `move_after_blocks` | **Tunable** | `2` / `3` (`1..4`) | `move_after_blocks` added to schema (`FB-40b`) |
| `melee_diagonal` | **Tunable** | `maneuver` (`none` \| `maneuver` \| `all`) | Diagonal melee rule |
| `flank_protected_cost` / `flank_protected_ap` | **Tunable** | `full` / `0..6` | Cost of a blocked flank dash |
| `collide_mult` / `collide_stagger` | **Tunable** | `0.50` / `10` | Forced-move collision values |

## 4. Acceptance tests (`P1.2-P2.2`)

1. **Data-driven `walkable` (`P1.2`)**: `grid.rs` contains no hardcoded terrain-index walkability table; toggling `walkable` on a terrain row in a scratch test blocks/allows pathfinding and forced movement with `0` golden diffs on shipped data.
2. **Level-3 terrain/hazard test (`P1.5`)**: Every terrain and hazard row in the catalog obeys `walkable`, `move_cost`, `cover_guard`, and `EffectSpec` resistance rules without per-row code branches.
