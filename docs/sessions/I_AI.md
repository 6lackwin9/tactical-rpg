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
    5. **`modifies_rating(Armor | MoveRate | AttackPower)`** (replaces `"slow"`, `"expose"`, `"fortify"`): Evaluates net rating shift so any buff/debuff status in data is automatically scored by the AI without naming its ID in Rust.

## 2. Unified AI role weights and `INTENT2-AI` row ladder (`C16`, `FB-21e`, `FB-22`)

- **Priority Row Ladder (`ai.rs:1590` `ROWS` + `INTENT2-AI`)**:
  - The AI evaluates candidate round intents (`Assault`, `Stand`, `Support`, `Maneuver`, `Field`) and targets through the ordered row table:
    - **`Row Z` (Hazard / Area Escape, `INTENT2-AI`)**: Steps out of visible ground/arena hazards or telegraphed area zones when survival threshold requires it.
    - **Flee / Morale Row**: Triggers when HP ratio `< flee_ratio`.
    - **Support / Heal Row**: Triggers when an ally's HP ratio `< heal_below` (using skills or pouch heal items, `FB-21d`).
    - **Stand / Guard / Cover / Self-`RING` Row (`INTENT2-AI`)**: Triggers when HP ratio `< defend_below`, when holding a `taunt` buff (`Shield Wall` candidate), when surrounded (`RING` row), or after taking heavy focus fire (`brace_after_hit`).
    - **Maneuver / Flank Engage Row**: Skirmishers dash to open `flank`/`behind` tiles on high-priority targets.
    - **Field / Area Row (`INTENT2-AI`)**: Places tile-anchored `Field` shapes or hazards when hitting `>= 2` hostiles (or choking an approach).
    - **Assault / Advance Row**: Strikes the top-ranked target (`TargetPref`: `DEFAULT`, `SOFTEST`, `TOUGHEST`) or advances into cover/Stand.
- **Role Thresholds & Temperament Overrides (`U2`, `U4`, `FB-29`)**:
  - Each `ai_role` (`tank`, `striker`, `skirmisher`, `ranged`, `support`, `caster`, `boss`) sets default weights (`heal_below`, `defend_below`, `flee_ratio`, `reckless`, `TargetPref`), which a unit row or `U4` character `temperament` (`FB-29`) can override.

## 3. Fixed vs tunable parameters (`aegis rules --contract`)

| Parameter | Kind | Default & Range | Notes |
| --- | --- | --- | --- |
| Behaviour-driven status predicates & `ROWS` evaluation order | **Fixed (Locked)** | 5 `S1` predicate families + `ROWS` ladder | Zero hardcoded status IDs in `ai.rs` |
| `ai_role` thresholds (`heal_below`, `defend_below`, `flee_ratio`, `reckless`) | **Tunable** | Role table in data/ruleset | Per-role AI weights |
| `ai_tactics` (`SOFTEST`, `TOUGHEST`, `brace_after_hit`, `hold_line`, `flush_cover`) | **Tunable** | Extension `ai_tactics` | Tactical modifiers |

## 4. Acceptance tests (`I1.2` - `I1.5`)

1. **Zero hardcoded status IDs in `ai.rs`**: Replacing the 8 literal status ID checks with `S1` behaviour predicates keeps `timeline_golden` and `sim_golden` byte-identical on shipped data.
2. **Custom status AI recognition (`I1.5` level-3 test)**: Adding a brand-new data-only debuff with `modifies_rating(Armor, -15)` or `locks_movement()` causes the AI to react identically to `expose` or `root` without touching Rust.
