# scripts/exploration

System: S9. Status: in progress (EXP1-L commits 1-2: schema and runtime; commit 3 = debug scene). Spec: docs/specs/EXP1_L.md (rules docs/specs/S9_RULES.md).

Purpose: Region maps, location interactions, encounter triggers.

Files:

- `map_catalog.gd` (`MapCatalog`): lore-free map templates in `data/maps/maps.json`; QuestCatalog contract (upsert, validate_all, save, use_path for tests); `used_by(id)` lists the locations using a template, and `remove` / rename refuse while any does.
- `map_rules.gd` (`MapRules`): tile/zone/anchor constants and lookups, template validation, location binding validation (`map`, `zone_bindings`, `markers`, called from `WorldCatalog._location_errors`), soft warnings, `start_location` resolution, roamer path index, system flag names (`done_flag`, `visited_flag`).
- `explore_runner.gd` (`ExploreRunner`): stateless walking over GameState: `enter`, `begin`, `resume`, `take_exit`, `menu`, `step` (fixed order: blocked / locked, move + points / exits, counters, roamers, first trigger, interact hints), `interact` (npc, chest, lore; a chest adds its `supplies` after the reward grant, TRAV2-L: above 99 they are lost with a `notes` line and the chest still opens; the result carries `supplies` = the amount added), `after_fight`, `wake` (defeat rule), `presence`.
- `battle_bridge.gd` (`BattleBridge`): `encounter_for` (a GameEncounter with the active party), `outcome(session)` (won / lost / fled; round-limit stalemate = fled, X1), `enemy_levels`, `apply` (XP + quest hook on won). Shared with TRAV1 and DLG1. The region map plays fights through `BattleScreen` (`scripts/ui/overworld/battle_screen.gd`, BATTLE1-L): encounter_for, then the combat arena, then outcome.
- `game_encounter.gd` (`GameEncounter`): a WorldEncounter whose party is the saved PartyState (PartyBuilder) and whose zone may be overridden.

The per-save random streams live in `scripts/game_state/game_rolls.gd` (`GameRolls`).

Rules: gameplay logic as RefCounted rules engines, data validated through a catalog, one test suite in scripts/tests/ (`exploration_test_suite.gd`).
