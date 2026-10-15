# scripts/game_state

System: S13 (save/load schema, session state). Status: done (M3). Tests: `scripts/tests/save_test_suite.gd`.

- `game_state.gd` (`GameState`): world, location, party, flags (`[a-z0-9_.]` to bool/int/String), opaque JSON-safe quests blob; `to_dict` / `from_dict` / `validate` / `equals`.
- `save_game.gd` (`SaveGame`): versioned envelope `{format, version, saved_at, game_state}`, `user://saves/slot_<n>.json`, `set_base_dir` override for tests, atomic write (tmp then rename), `read` never returns a partial state.
- `save_migrations.gd` (`SaveMigrations`): from_version to from_version+1 table (empty at version 1), newer saves refused, unknown `game_state` keys dropped with warnings.
- `new_game.gd` (`NewGame`): `create(world_id, base_unit_ids)` returns `{state, errors}`.
- `json_util.gd` (`JsonUtil`): JSON-safety check, float-to-int normalise, deep equality.

Tests and sandboxes must call `SaveGame.set_base_dir` and never touch the real `user://saves`.
