# scripts/dialogue

System: S11 (spec `docs/specs/DLG1_L.md`, rules `docs/specs/S11_RULES.md`). Status: in progress (commit 1 schema, commit 2 runtime; commit 3 panels next).

Purpose: dialogue trees, quest services, NPC presence, shops and gold.

## Files

- `dialogue_rules.gd` (`DialogueRules`): constants (`MAX_NODES`, `MAX_CHOICES`, node / choice keys, `CONDITION_KINDS`, `QUEST_STATUSES`, `EFFECTS`, `NODE_EFFECTS`, json ref keys, `SERVICE_IDS`, `DEFAULT_LINES`, `TOKENS`, `TEXT_TOKENS`) and pure validation: `tree_errors(world_id, e)` (WorldCatalog kind `dialogues`; authored node / choice texts allow only `{npc}` and `{gold}`), `tree_warnings(e)` / `dialogue_warnings(world_id)` (unreachable nodes; a `give` node on a dialogue cycle unless every way in is a choice guarded by `unless` + a matching `set_flag`), `conditions_errors`, `effects_errors` (choice order rules: `turn_in` first, `open_shop` / `fight` last), `line_errors` / `service_lines_errors`, the gone-NPC lint `quest_warnings(world_id)`; runtime reads `is_gone`, `condition_holds`, `choice_open`, `best_stat` (best ACTIVE member, gear and passives included), `stat_prefix`, `start_gold`, `service_line`, `fill`, `complete_from_journal`.
- `dialogue_runner.gd` (`DialogueRunner`): one conversation, never saved. `open(gs, character_ref)` (`errors` non-empty = refused, e.g. "Mara is gone."), `view()` ({speaker, speaker_name, node, text, choices [{text, locked}], end, next, shop, pending}), `choose(index)` (pre-checks every effect, all-or-nothing; returns `pending`, `shop`, `fight`, `node` or `end`), `confirm_turn_in(recipient_id, picks)` / `cancel()`, `close_shop()`, `advance()` (for `next` and end nodes), static `after_fight(gs, fight, outcome)` (BattleBridge.apply, plus ExploreRunner.wake on lost).
- `shop_rules.gd` (`ShopRules`): `shop_errors(world_id, e)` for kind `shops`; `buy_types()` (every ItemFamilies type but QUEST, the `buys` default); runtime `is_open`, integer `buy_price` / `sell_price`, `offers`, `buy`, `sell_list`, `sell`.
- `service_catalog.gd` (`ServiceCatalog`): the base service lines in `data/dialogue/services.json`, same contract as `QuestCatalog` (`upsert`, `remove`, `save`, `use_path`, `validate_all`, `warnings_all` for missing ids, `text_of` with the `DEFAULT_LINES` fallback).

## Rules

- Content goes through `WorldCatalog.upsert` (trees, shops, characters, manifest via `update_manifest`) and `ServiceCatalog.upsert`, never raw JSON.
- Dialogue conditions (`flag`, `quest`, `items`, `gold`, `stat`) exist on dialogue choices only; markers, quests, loot, routes and `gone_flags` keep the M4 flag-only grammar.
- Services are generated on the start node (unless `services: false`) after the authored choices: offers (resolved giver = the character, `available`; Accept and Decline both return to the start node), turn-ins (effective turn-in = the character, `ready`), open shops it keeps, Farewell. They may push the start node past `MAX_CHOICES` (allowed: the cap is an authoring limit).
- `start_quest` is refused unless the resolved giver is "" or the character talked to; `turn_in` likewise for the effective turn-in. `QuestRunner.start` / `complete` stay unrestricted for tests and tools.
- A gone character (any `gone_flags` holds) cannot be opened, its npc markers are absent (`ExploreRunner.presence`), its shops are closed.
- Party gold (`PartyState.gold`, 0..`GOLD_MAX`, additions capped) comes from quest `rewards.gold`, chest `gold`, selling and dialogue `give`; `NewGame` starts at the resolved `start_gold`.
- Tests: `scripts/tests/dialogue_test_suite.gd` (sandbox content fork, quest / service / map files, save folder and world root).
