# Design session I: AI decisions, tactics and behaviour-driven status reads (`I1.1`, closed 2026-11-07)

Design record for the AI block (`I1.1` Decisions, tactics, weighted AI role profiles, `INTENT2-AI` row ladder, and replacing the 8 hardcoded status IDs in `ai.rs`). Connects `S1`/`U2`/`F3` (`C16`).

## 1. Core architecture: Behaviour-driven status reads (Fixes 8 Hardcoded Status IDs in `ai.rs`)

Currently, `ai.rs` checks 8 literal status IDs (`taunt`, `riposte`, `regen`, `root`, `stun`, `slow`, `expose`, `fortify`), which prevents level-3 silo tests from covering new statuses (`MASTERPLAN1` §1).

- **Query `S1.1`'s 6 Composable Behaviour Primitives Instead of Status IDs**:
  - `ai.rs` inspects a unit's active and pending statuses through behaviour predicates on `S1` rows:
    1. **`has_targeting_pull()`** (replaces `"taunt"`): Biases enemy target selection toward the taunting unit and triggers the `taunt` guard AI rule (`BASERULES1_LIBRARY` §6).
    2. **`has_reactive_counter()`** (replaces `"riposte"`): Deters non-flanking melee swings or weights ranged/area/flank options.
    3. **`has_tick(Heal | Damage)`** (replaces `"regen"`): Avoids redundant heal-over-time stacking and weights cleanse/support on heavy DoTs.
    4. **`locks_movement()` / `locks_blocks()`** (replaces `"root"`, `"stun"`): Skips unreachable move/dash plans when movement is locked and weights follow-up pressure on stunned/staggered targets.
    5. **`modifies_rating(Armor | MoveRate | AttackPower)`** (replaces `"slow"`, `"expose"`, `"fortify"` in `ai.rs` — where `fortify` and `expose` map to `Armor`, `slow` and `haste` to `MoveRate`, and `weaken`/`empower` to `AttackPower` per `FB-52`):
       - **Slice 1 (`I1.2-S1`, after `S1.2-S1`)**: Replace the 8 status ID checks in `ai.rs` 1-to-1 with `S1` behaviour predicates (`Armor` for `fortify`/`expose`, `MoveRate` for `slow`) so `timeline_golden` and `sim_golden` remain byte-identical.
       - **Slice 2 (`I1.2-S2`)**: Add a generic rating-modifier scorer behind a ruleset flag (default `off` until balance re-record) so scoring `haste`, `weaken`, and `empower` is opt-in.
       - **Slice 3 (`I1.2-S3`, after `A3.2`, `A4.2`, `F3`)**: Add `Row Z`, `Maneuver`, `Field`, and `RING` rows to `ROWS` (`ai.rs:1590`) in a named `timeline_golden` and `sim_golden` re-record.
- **Role Names (`FB-52`)**:
  - Keep the schema `ai_role` enum (`AGGRESSOR`, `SKIRMISHER`, `SNIPER`, `SUPPORT`) as the canonical role list, with unit rows or `U4` character `temperament` (`FB-29`) overriding role thresholds (`heal_below`, `defend_below`, `flee_ratio`, `reckless`, `TargetPref`).

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| Behaviour-driven status predicates & `ROWS` evaluation order | **Fixed (Locked)** | 5 `S1` predicate families + `ROWS` ladder | Zero hardcoded status IDs in `ai.rs` |
| `ai_role` thresholds (`heal_below`, `defend_below`, `flee_ratio`, `reckless`) | **Tunable** | Role table in data/ruleset | Per-role AI weights |
| `ai_tactics` (`SOFTEST`, `TOUGHEST`, `brace_after_hit`, `hold_line`, `flush_cover`) | **Tunable** | Extension `ai_tactics` | Tactical modifiers |

## 4. Acceptance tests (`I1.2` - `I1.5`)

1. **Zero hardcoded status IDs in `ai.rs`**: Replacing the 8 literal status ID checks with `S1` behaviour predicates keeps `timeline_golden` and `sim_golden` byte-identical on shipped data.
2. **Custom status AI recognition (`I1.5` level-3 test)**: Adding a brand-new data-only debuff with `modifies_rating(Armor, -15)` or `locks_movement()` causes the AI to react identically to `expose` or `root` without touching Rust.
