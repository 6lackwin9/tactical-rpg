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
| `core/combat/battle_command.gd`, `combat/actions/*` (with `item_command.gd`) | `aegis_battle::command` | battle traces, item traces |
| `combat/arena/combat_resolver.gd` | `aegis_battle::resolver` | battle traces |
| `combat/planning/combat_plan.gd`, `plan_step.gd`, `plan_entry.gd` (no options / UI probes yet) | `aegis_battle::plan` | battle traces |
| `combat/planning/skill_def.gd`, `skill_catalog.gd`, `content/skill_kinds.gd` | `aegis_battle::skill` | battle traces |
| `combat/ai/tactical_ai.gd` (with `_try_item`), `ai_profile.gd`, `plan_presets.gd` | `aegis_battle::ai`, `world` | battle traces, item traces |
| `combat/session/combat_session.gd`, `catalog_encounter.gd`, `combat_encounter.gd` | `aegis_battle::session` | battle traces, world traces |
| `combat/session/encounter_objectives.gd`, `encounter_phases.gd`, `encounter_morale.gd`, `world_encounter.gd` (guests, start buffs) | `aegis_battle::encounter`, `session`; `aegis_world::battle::WorldRoster` | world traces |
| `tools/sim/sim_runner.gd` (`fight`, `apply_pouch`), `sim_variants.gd` (jitter) | `aegis_battle::sim` | battle traces |

## Parity status (2026-10-19)

- Unit snapshots: 102 rows exact. Damage: 18,216 cases exact. RNG: 180 draws exact.
- Battle traces: 488 balance-sim fights replay step for step: 360 duels (6 classes, levels 1 / 5 / 10, 2 seeds,
  both sides, jitter on), 72 group fights (hero vs 3 minion / standard units) and 56 team fights (mixed trios).
  Compared after every step: HP, AP, tile, stance, active and pending statuses; and every round's schedule.
  About 80 distinct skills appear (dashes, disengages, areas, heals, cleanses, taunts, buffs, Focus, riposte);
  statuses seen: stagger, stun, root, bleed, weaken, haste, taunt, riposte.
- Battle items (ITEMUSE1): 180 more sim fights with `pouch=` (`trace.sh ... pouch=<ids>`, written as
  `traces_<mode>_l<level>_items.json`): 120 duels at level 5 (potion, bomb, focus elixir), 24 team fights and 36
  group fights at level 7; 250 Healing Draught and 39 Throwing Fire Bomb uses, all step for step. Mutating the
  potion threshold or the bomb damage makes 8 to 12 fights diverge, so both item paths are checked.
- World battles (`trace.sh mode=world preset=<p>`, `aegis_world/tests/world_traces.rs`): 240 fights over every
  world encounter plus synthetic variants (escape, centre deployment, morale, custom hold tiles, start buffs) with
  the balanced, aggressive, withdraw and defensive presets: hold, slay, escape, fled, lost and round-limit verdicts,
  boss phases (hp_below, round, waves, status, ai, enrage), morale breaks and surrenders, ambush and centre
  deployments, a protected guest and start buffs, all step for step.
- Not covered yet: statuses expose, fortify, regen, poison, burn, empower, slow (the shipped classes rarely use
  them in these fights), terrain zones other than the stone arena and the two catalog ones.
- Suite: `cargo test -p aegis_battle --test trace_parity` replays every `parity/out/traces*.json` except the world
  ones (677 fights, about 8 s).
- Encounters: the three shipped catalog encounters (Ruined Courtyard and Forest Clearing terrain, exits, the
  goblin_ambush reinforcement wave) replay step for step too (`trace.sh mode=encounters`).
- Windows Smart App Control sometimes blocks a built binary (`os error 4551` from cargo test, "Permission denied" from
  Git Bash), also one that ran before: touching the test file (or deleting the exe) relinks it and the next run
  usually passes; `aegis-gui.exe` that Git Bash cannot start still starts with PowerShell `Start-Process`.

## Game loop (aegis_world, 2026-10-19)

| Godot | Rust | Checked by |
| :--- | :--- | :--- |
| `game_state/game_state.gd`, `progression/party_state.gd`, `unit_state.gd` | `aegis_world::state` | journeys (state after every action) |
| `game_state/new_game.gd`, `game_rolls.gd`, `json_util.gd` | `new_game`, `rolls`, `json` | journeys (start state, every roll) |
| `game_state/save_game.gd`, `save_migrations.gd` (all steps 1 -> 10) | `save` | `tests/saves.rs`: every Godot save fixture v1..v10 |
| `progression/xp_curve.gd`, `level_up.gd`, `party_builder.gd`, `heal_rules.gd` | `progression` | journeys (HP views, level-ups, camp) |
| `world/world_catalog.gd` (lineage, merged, resolve, party rules), `world_factory.gd` (unit entry, passives) | `data` | journeys |
| `exploration/map_rules.gd` (runtime), `explore_runner.gd`, `purse_rules.gd`, `battle_bridge.gd` (apply) | `explore`, `purse`, `battle` | journeys (marker views too) |
| `quests/quest_rules.gd` (runtime), `quest_runner.gd`, `reward_grant.gd` | `quest` | journeys (statuses, objectives, offers) |
| `dialogue/rep_rules.gd`, `dialogue_rules.gd` (runtime), `dialogue_runner.gd`, `shop_rules.gd` (runtime) | `dialogue`, `shop` | journeys (dialogue and shop views) |
| `travel/travel_rules.gd` (runtime), `travel_runner.gd` | `travel` | journeys (world map view) |
| `exploration/game_encounter.gd` (saved party in battle, HP and pouch write-back, start-status flags) | `aegis_gui::battle` (`BattleScreen::world`) | manual (GUI) |

Godot behaviours found on the way:

- `hash([...])` is murmur3 over the array (Variant ARRAY = 28), Thomas Wang 64 -> 32 for ints, djb2 for strings
  (`aegis_rules::ghash`); `String.hash()` is djb2.
- `JSON.stringify` sorts keys by default, so a Godot save round trip re-orders every dictionary (stash, flags,
  quests): engine saves sort keys too, and the journey replay's reload does the same.
- `JSON.parse` makes every number a float and `stringify` writes it back as `3.0`; the 2 -> 3 save step hashes that
  text (rebuilt by `save::godot_text`).

Journeys (`parity/journey.sh`, `tests/journey.rs`): seeded random play through the shipped rules, goal-directed
walking, dialogue biased to quest services, shops, chests, travel, camp, level-ups, abandons, journal turn-ins, save /
load round trips and battle outcomes drawn at random (won / fled / lost). Each action's result, the state after it and
the screen views (quest statuses, objectives, turn-in offers, HP, camp, exits menu, markers, dialogue, shop, world
map) must equal Godot's. Last run: 8 journeys, 5608 actions, all equal; includes quest completion, recruits and a
`connects` world crossing.

Not ported yet: content validation (`*_errors`, warnings), save import / export.

## Data loader (aegis_world::source, 2026-10-19)

The engine reads the project's `data/` folder itself; the Godot exports are only needed for parity checks.

| Godot | Rust | Checked by |
| :--- | :--- | :--- |
| `content/content_catalog.gd` (`reload_from_disk`, `_read_kind`, `_read_ranked_skills`, `_read_items`) | `source::load_catalog` | `tests/data_source.rs` |
| `content/content_schema.gd` (`normalize`, `coerce`, `_copy`), `world/world_schema.gd` (`normalize`) | `source::normalize`, `coerce`, field tables in `engine/data/schema.json` | `tests/data_source.rs` |
| `content/skill_ranks.gd` (`expand`, `effective`, `resolve`), `content/skill_text.gd` (`render`, `token_value`) | `source::expand`, `render` | `tests/data_source.rs` (151 rank rows, every description) |
| `core/specs/item_families.gd` (`derive`, hands, base), `item_modifiers.gd` (`resolve`, `legacy_tiers`), `item_text.gd` (`tier_desc`) | `source::derive_family`, `resolve_item` | `tests/data_source.rs` |
| `world/world_catalog.gd` (`reload_from_disk`, `lineage`, `own_entries`) | `source::load_worlds` | `tests/data_source.rs` |
| `exploration/map_catalog.gd` + `map_rules.gd` (`normalize_template`), `quests/quest_catalog.gd`, `dialogue/service_catalog.gd`, `progression/xp_curve.gd` | `source::load_list`, `xp_curve` | `tests/data_source.rs` |

- `tests/data_source.rs` compares the loader's output with `catalog.json`, `worlds.json` and `game_data.json`: keys, key
  order and int / float types exactly, floats to 1e-9 (Godot's `JSON.stringify` writes fewer digits than it computes
  with: `0.1 + 0.2` is written `0.3`), and the playable worlds. 239 catalog rows and 3 worlds match.
- Run on loader-built data instead of the exports, the journeys (5608 actions), save fixtures and world battles (240)
  still match Godot, and the CLI balance sim prints the same table.
- `engine/data/schema.json` is the committed copy of Godot's field tables (`parity/export.sh only=schema` writes
  `parity/out/schema.json`; the test fails when they drift). It carries labels, help, options and groups too, for
  content editing in the engine later.
- Not read: version 1 (flat) skills and items files, pre-CLASS1 class `skill_path` lists (the shipped data has none;
  the loader reports them), and the load-time validation messages that only block saving in Godot.
