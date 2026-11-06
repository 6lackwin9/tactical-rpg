# Design session P: Space, terrain, hazards and movement (`P1.1-P2.1`, closed 2026-11-07)

Design record for the Space block (`P1.1` Board, terrain & hazards, `P2.1` Movement & position). Eliminates the hardcoded terrain walkability special case in `grid.rs:68`.

## 1. `P1.1` — Board, terrain and ground hazards (`C14`)

- **Data-Driven Terrain Walkability (`FB-51`, Fixes `grid.rs:68` Special Case)**:
  - Currently, `grid.rs:64-72` fixes `BLOCKED` and `SLOW` by index.
  - **Fix (`P1.2-S1`)**: Store `walkable: bool` (with a missing-key fallback for the frozen kit's 17 terrain rows), `stops_forced_move: bool` (default `!walkable`), and `move_cost` default on the 17 terrain rows (`terrain.json` + schema + re-save), keeping the 17 `TERRAIN_IDS` and glyph indices fixed while `grid.is_walkable` and `forced.rs` read the row fields.

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| Terrain `walkable`, `stops_forced_move` & hazard cell schema in data | **Fixed (Locked)** | `walkable`, `move_cost`, `cover_guard`, `EffectSpec` | Removes `grid.rs:68` walkability hardcode (`FB-51`) |
| `move_after_blocks` | **Tunable** | `3` (`1..4`) | Added to schema with re-save (`FB-40b`, `P2.2-S1`); `MAX_MOVE_BLOCKS = 2` stays locked |
| `melee_diagonal` | **Tunable** | `maneuver` (`none` \| `maneuver` \| `all`) | Diagonal melee rule |
| `flank_protected_cost` / `flank_protected_ap` | **Tunable** | `full` / `0..6` | Cost of a blocked flank dash (per-skill `forced_move` keys hold collide values) |

## 4. Acceptance tests (`P1.2-P2.2`)

1. **Data-driven `walkable` (`P1.2-S1`, `FB-51`)**: `grid.rs` reads `walkable`, `stops_forced_move`, and `move_cost` from the terrain row (with a missing-key fallback for the frozen kit); toggling `walkable` on a terrain row in a scratch test blocks/allows pathfinding and forced movement with `0` golden diffs on shipped data.
2. **Unified cell `EffectSpec` (`P1.2-S2`, after `A4.2`)**: Terrain and hazard rows share the `EffectSpec` vocabulary; hazard damage stays "through guard, not armour" initially so goldens hold.
