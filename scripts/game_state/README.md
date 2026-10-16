# scripts/game_state

System: S13 (save/load schema, session state). Status: done (M3; save v2 in M4-L). Tests: `scripts/tests/save_test_suite.gd`, `scripts/tests/quest_runtime_test_suite.gd` (quest blob and migration).

- `game_state.gd` (`GameState`): world, location, party, flags (`[a-z0-9_.]` to bool/int/String), the structured quests blob (qualified quest id to `{state, objective}`, checked by `QuestRules.validate_blob`) and `loot_claimed` (unique qualified loot ids, saved sorted); `to_dict` / `from_dict` / `validate` / `equals`.
- `save_game.gd` (`SaveGame`): versioned envelope `{format, version: 2, saved_at, game_state}`, `user://saves/slot_<n>.json`, `set_base_dir` override for tests, atomic write (tmp then rename), `read` never returns a partial state.
- `save_migrations.gd` (`SaveMigrations`): from_version to from_version+1 table with the built-in step 1 to 2 (keeps valid quest entries, drops the rest with one warning each, adds `loot_claimed: []`); `reset()` restores the built-in table; newer saves refused, unknown `game_state` keys dropped with warnings.
- `new_game.gd` (`NewGame`): `create(world_id, base_unit_ids)` returns `{state, errors}`.
- `json_util.gd` (`JsonUtil`): JSON-safety check, float-to-int normalise, deep equality.

Tests and sandboxes must call `SaveGame.set_base_dir` and never touch the real `user://saves`.

`from_dict` records raw type problems in `load_problems`; `validate()` reports them, so damaged `quests` / `loot_claimed` blobs are refused, never coerced.
