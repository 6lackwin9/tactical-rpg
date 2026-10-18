# Port Notes (Godot -> Aegis)

Working log for the combat port. One row per Godot file: where it went in Rust and how it is checked.
Pass bar: statistical parity (ENGINE_SPEC section 6). Dev tool: seeded battles replay step for step,
because the RNG is bit-exact.

## Godot behaviours to reproduce exactly

| Godot behaviour | Rust |
| :--- | :--- |
| `RandomNumberGenerator` (PCG32, two-draw `randf`, bounded `randi_range`) | `aegis_rules::rng::GodotRng` (tested against exported draws) |
| `roundf` / `roundi` round half away from zero | `f64::round` (`stats::roundi`) |
| `Array.sort_custom` is Godot's `SortArray` introsort (insertion sort at 16 or fewer, so stable there; unstable above) | `aegis_rules::sort::godot_sort` (port of `SortArray`) |
| Dictionary iteration is insertion order | `IndexMap` everywhere order can matter |
| JSON numbers come back as floats; `x is int` checks fail on parsed data | lenient `aegis_data::value` reads; checked by parity |
| Integer division truncates toward zero, `floori` floors | match per call site |

## Files

| Godot | Rust | Checked by |
| :--- | :--- | :--- |
| `core/specs/stat_rules.gd`, `character_stats.gd`, `proficiency.gd`, `stat_modifier.gd` | `aegis_rules::stats` | unit snapshots |
| `core/inventory/equipment.gd`, `inventory.gd` | `aegis_rules::gear` | unit snapshots |
| `core/combat/combatant.gd` (build, skills, unlocks) | `aegis_rules::unit` | unit snapshots |
| `content/content_factory.gd` (`unit`, `loadout_of`, `_equip`) | `aegis_rules::unit::build_unit` | unit snapshots |
| `core/specs/passives.gd` (`apply_effects`) | `Unit::apply_effects` | unit snapshots (gear effects) |
| `core/combat/damage_calculator.gd` | `aegis_rules::damage` | damage cases |
| `combat/status/status_effects.gd`, `status_registry.gd` | `aegis_rules::status` | battle traces |
| `combat/arena/combat_grid.gd`, `terrain.gd`, `combat_zones.gd` | `aegis_battle::grid`, `world` | battle traces |
| `combat/arena/grid_path.gd` | `aegis_battle::path` | battle traces |
| `combat/arena/cover_rules.gd`, `hazard_rules.gd`, `status/riposte_rules.gd`, `status/support_rules.gd` | `aegis_battle::command` | battle traces |
| `core/combat/battle_command.gd`, `combat/actions/*` (no item command yet) | `aegis_battle::command` | battle traces |
| `combat/arena/combat_resolver.gd` | `aegis_battle::resolver` | battle traces |
| `combat/planning/combat_plan.gd`, `plan_step.gd`, `plan_entry.gd` (no options / UI probes yet) | `aegis_battle::plan` | battle traces |
| `combat/planning/skill_def.gd`, `skill_catalog.gd`, `content/skill_kinds.gd` | `aegis_battle::skill` | battle traces |
| `combat/ai/tactical_ai.gd`, `ai_profile.gd`, `plan_presets.gd` (no item use yet) | `aegis_battle::ai`, `world` | battle traces |
| `combat/session/combat_session.gd`, `catalog_encounter.gd`, `combat_encounter.gd` (rout only; no waves, phases, morale) | `aegis_battle::session` | battle traces |
| `tools/sim/sim_runner.gd` (`fight`), `sim_variants.gd` (jitter) | `aegis_battle::sim` | battle traces |

## Parity status (2026-10-19)

- Unit snapshots: 102 rows exact. Damage: 18,216 cases exact. RNG: 180 draws exact.
- Battle traces: 488 balance-sim fights replay step for step: 360 duels (6 classes, levels 1 / 5 / 10, 2 seeds,
  both sides, jitter on), 72 group fights (hero vs 3 minion / standard units) and 56 team fights (mixed trios).
  Compared after every step: HP, AP, tile, stance, active and pending statuses; and every round's schedule.
  About 80 distinct skills appear (dashes, disengages, areas, heals, cleanses, taunts, buffs, Focus, riposte);
  statuses seen: stagger, stun, root, bleed, weaken, haste, taunt, riposte.
- Not covered yet: statuses expose, fortify, regen, poison, burn, empower, slow (the shipped classes rarely use
  them in these fights), battle items, boss phases, morale, hold / escape / slay objectives,
  ambush and centre deployments, terrain zones other than the stone arena.
- Suite: `cargo test -p aegis_battle --test trace_parity` replays every `parity/out/traces*.json` (about 8 s).
- Encounters: the three shipped catalog encounters (Ruined Courtyard and Forest Clearing terrain, exits, the
  goblin_ambush reinforcement wave) replay step for step too (`trace.sh mode=encounters`).
