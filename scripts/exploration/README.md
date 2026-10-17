# scripts/exploration

System: S9. Status: in progress (EXP1-L commit 1: schema). Spec: docs/specs/EXP1_L.md (rules docs/specs/S9_RULES.md).

Purpose: Region maps, location interactions, encounter triggers.

Files:

- `map_catalog.gd` (`MapCatalog`): lore-free map templates in `data/maps/maps.json`; QuestCatalog contract (upsert, validate_all, save, use_path for tests).
- `map_rules.gd` (`MapRules`): tile/zone/anchor constants and lookups, template validation, location binding validation (`map`, `zone_bindings`, `markers`, called from `WorldCatalog._location_errors`), soft warnings, `start_location` resolution, roamer path index, system flag names (`done_flag`, `visited_flag`).

Planned (commit 2): `explore_runner.gd`, `battle_bridge.gd`, `game_encounter.gd`.

Rules: gameplay logic as RefCounted rules engines, data validated through a catalog, one test suite in scripts/tests/ (`exploration_test_suite.gd`).
