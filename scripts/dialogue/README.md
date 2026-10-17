# scripts/dialogue

System: S11 (spec `docs/specs/DLG1_L.md`, rules `docs/specs/S11_RULES.md`). Status: in progress (commit 1: schema and validation).

Purpose: dialogue trees, quest services, NPC presence, shops and gold.

## Files

- `dialogue_rules.gd` (`DialogueRules`): constants (`MAX_NODES`, `MAX_CHOICES`, node / choice keys, `CONDITION_KINDS`, `QUEST_STATUSES`, `EFFECTS`, `NODE_EFFECTS`, json ref keys, `SERVICE_IDS`, `DEFAULT_LINES`, `TOKENS`) and pure validation: `tree_errors(world_id, e)` (WorldCatalog kind `dialogues`), `tree_warnings(e)` / `dialogue_warnings(world_id)` (unreachable nodes), `conditions_errors`, `effects_errors` (choice order rules: `turn_in` first, `open_shop` / `fight` last), `line_errors` / `service_lines_errors` (service lines and tokens), and the gone-NPC lint `quest_warnings(world_id)`.
- `shop_rules.gd` (`ShopRules`): `shop_errors(world_id, e)` for WorldCatalog kind `shops` (keeper, stock, item loot for sale, buy / sell multipliers, `buys`, open flags).
- `service_catalog.gd` (`ServiceCatalog`): the base service lines in `data/dialogue/services.json`, same contract as `QuestCatalog` (`upsert`, `remove`, `save`, `use_path`, `validate_all`, `warnings_all` for missing ids, `text_of` with the `DEFAULT_LINES` fallback).

Planned (DLG1-L commit 2): `dialogue_runner.gd` (DialogueRunner), presence, services, shop prices and trading, party gold. Reputation is REP1.

## Rules

- Content goes through `WorldCatalog.upsert` (trees, shops, characters, manifest via `update_manifest`) and `ServiceCatalog.upsert`, never raw JSON.
- Dialogue conditions (`flag`, `quest`, `items`, `gold`, `stat`) exist on dialogue choices only; markers, quests, loot, routes and `gone_flags` keep the M4 flag-only grammar.
- Tests: `scripts/tests/dialogue_test_suite.gd` (sandbox content fork, quest file, service file and world root).
