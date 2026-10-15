class_name SaveTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Save Test Suite (M3 / S13)
# GameState round trip, flags, SaveGame write / read (sandbox directory only), corrupt and newer
# files, the migration hook, unknown-key warnings, and NewGame. The real user://saves is never touched.
# ==============================================================================

const SANDBOX: String = "user://m3_save_test"

static func _report(label: String, checks: Dictionary) -> bool:
	var failed: Array[String] = []
	for key: Variant in checks:
		if not DictUtil.to_bool(checks[key]):
			failed.append(str(key))
	var ok: bool = failed.is_empty()
	print("    [Save] %s: %s%s" % [label, "PASS" if ok else "FAIL", "" if ok else " (%s)" % ", ".join(failed)])
	return ok

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING SAVE TEST (M3 / S13)                    ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var real_before: PackedStringArray = DirAccess.get_files_at(SaveGame.DEFAULT_DIR)
	SaveGame.set_base_dir(SANDBOX)
	_wipe()
	var ok: bool = true
	ok = _test_game_state() and ok
	ok = _test_new_game() and ok
	ok = _test_write_read() and ok
	ok = _test_bad_files() and ok
	ok = _test_migrations() and ok
	_wipe()
	SaveGame.reset_base_dir()
	SaveMigrations.reset()
	var real_after: PackedStringArray = DirAccess.get_files_at(SaveGame.DEFAULT_DIR)
	ok = _report("real saves untouched", {"listing_unchanged": real_before == real_after, "dir_restored": SaveGame.base_dir() == SaveGame.DEFAULT_DIR}) and ok
	return ok

static func _joined(v: Variant) -> String:
	var parts: PackedStringArray = PackedStringArray()
	for x: Variant in DictUtil.array_of(v):
		parts.append(str(x))
	return ", ".join(parts)

static func _wipe() -> void:
	if DirAccess.dir_exists_absolute(SANDBOX):
		for f: String in DirAccess.get_files_at(SANDBOX):
			DirAccess.remove_absolute(SANDBOX + "/" + f)
		DirAccess.remove_absolute(SANDBOX)

static func _world() -> String:
	return WorldCatalog.world_ids()[0]

static func _party_unit() -> String:
	for e: Dictionary in ContentCatalog.entries("units"):
		if str(e.get("side", "")) == "party" and ContentFactory.class_of(e) != null and str(e.get("stat_mode", "derived")) == "derived":
			return str(e["id"])
	return ""

# A state with every field family used.
static func _rich_state() -> GameState:
	var uid: String = _party_unit()
	var res: Dictionary = NewGame.create(_world(), [uid, uid] as Array[String])
	var g: GameState = res["state"] as GameState
	var m: UnitState = g.party.members[0]
	m.total_xp = XpCurve.xp_for_level(4) + 7
	LevelUp.confirm(m, {"STR": 1, "KNOW": 2})
	var pids: Array[String] = ContentCatalog.ids("passives")
	if not pids.is_empty():
		m.passives = [pids[0]] as Array[String]
	g.party.add_to_stash(str(ContentCatalog.entries("items")[0]["id"]), 2)
	g.location_id = "somewhere"
	g.set_flag("intro.done", true)
	g.set_flag("count", 7)
	g.set_flag("name", "x")
	g.quests = {"q1": {"stage": 2, "log": ["a", "b"], "ratio": 0.5}}
	return g

static func _test_game_state() -> bool:
	var g: GameState = _rich_state()
	var checks: Dictionary = {"rich_valid": g.validate().is_empty()}
	checks["flag_roundtrip"] = g.has_flag("count") and g.get_flag("count") == 7 and g.get_flag("missing", "dflt") == "dflt"
	g.clear_flag("count")
	checks["flag_cleared"] = not g.has_flag("count")
	checks["bad_key_refused"] = not g.set_flag("Bad Key", true).is_empty() and not g.has_flag("Bad Key")
	checks["bad_value_refused"] = not g.set_flag("ok_key", 1.5).is_empty() and not g.set_flag("ok_key2", [1]).is_empty()
	var bad_flags: GameState = _rich_state()
	bad_flags.flags["BAD"] = true
	checks["validate_bad_flag_key"] = not bad_flags.validate().is_empty()
	var bad_quest: GameState = _rich_state()
	bad_quest.quests = {"q": Vector2(1, 2)}
	checks["validate_unsafe_quests"] = not bad_quest.validate().is_empty()
	var no_world: GameState = _rich_state()
	no_world.world_id = ""
	checks["validate_world_required"] = not no_world.validate().is_empty()
	var text: String = JSON.stringify(g.to_dict())
	var back: GameState = GameState.from_dict(DictUtil.dict_of(JSON.parse_string(text)))
	checks["json_roundtrip_equal"] = back.equals(g)
	checks["ints_preserved"] = back.party.members[0].total_xp is int and back.flags["intro.done"] == true and back.quests["q1"]["stage"] is int
	var changed: GameState = GameState.from_dict(g.to_dict())
	changed.party.members[0].total_xp += 1
	checks["equals_detects_difference"] = not changed.equals(g)
	return _report("game state", checks)

static func _test_new_game() -> bool:
	var uid: String = _party_unit()
	var good: Dictionary = NewGame.create(_world(), [uid, uid] as Array[String])
	var state: GameState = good["state"] as GameState
	var checks: Dictionary = {"valid_input": state != null and DictUtil.array_of(good["errors"]).is_empty() and state.validate().is_empty()}
	if state != null:
		var entry: Dictionary = ContentCatalog.get_entry("units", uid)
		var m: UnitState = state.party.members[0]
		checks["authored_level_and_threshold"] = m.level() == DictUtil.to_int(entry["level"]) and m.total_xp == XpCurve.xp_for_level(m.level())
		checks["spend_empty_auto"] = m.spend.is_empty() and m.unspent() == StatRules.spend_pool(m.level())
		checks["duplicate_units_get_unique_ids"] = state.party.members.size() == 2 and state.party.members[0].id != state.party.members[1].id
		checks["world_set"] = state.world_id == _world()
	var bad_unit: Dictionary = NewGame.create(_world(), ["no_such_unit_xyz"] as Array[String])
	var bad_world: Dictionary = NewGame.create("no_such_world_xyz", [uid] as Array[String])
	var none: Dictionary = NewGame.create(_world(), [] as Array[String])
	checks["unknown_unit_refused"] = bad_unit["state"] == null and not DictUtil.array_of(bad_unit["errors"]).is_empty()
	checks["unknown_world_refused"] = bad_world["state"] == null and not DictUtil.array_of(bad_world["errors"]).is_empty()
	checks["empty_party_refused"] = none["state"] == null
	return _report("new game", checks)

static func _test_write_read() -> bool:
	var g: GameState = _rich_state()
	var checks: Dictionary = {"no_file_yet": not SaveGame.exists(1)}
	checks["write_ok"] = SaveGame.write(g, 1).is_empty()
	checks["exists"] = SaveGame.exists(1)
	checks["no_tmp_left"] = not FileAccess.file_exists(SaveGame.path_for(1) + ".tmp")
	checks["inside_sandbox"] = SaveGame.path_for(1).begins_with(SANDBOX)
	var res: Dictionary = SaveGame.read(1)
	checks["read_ok"] = DictUtil.to_bool(res["ok"]) and res["state"] != null
	var back: GameState = res["state"] as GameState
	checks["equal_after_read"] = back != null and back.equals(g)
	g.party.members[0].total_xp += 5
	checks["overwrite_ok"] = SaveGame.write(g, 1).is_empty() and not FileAccess.file_exists(SaveGame.path_for(1) + ".tmp")
	var back2: GameState = SaveGame.read(1)["state"] as GameState
	checks["overwrite_visible"] = back2 != null and back2.equals(g)
	# Refuse invalid states without touching the existing file.
	var broken: GameState = _rich_state()
	broken.party.members[0].base_unit = "no_such_unit_xyz"
	checks["invalid_refused"] = not SaveGame.write(broken, 1).is_empty()
	var still: GameState = SaveGame.read(1)["state"] as GameState
	checks["invalid_keeps_old_file"] = still != null and still.equals(g)
	var dup: GameState = _rich_state()
	dup.party.members[1].id = dup.party.members[0].id
	var over: GameState = _rich_state()
	over.party.members[0].spend = {"STR": 999}
	var lo: GameState = _rich_state()
	lo.party.members[0].loadout = {"class": ["family_nobody_owns"] as Array[String], "extra": [] as Array[String]}
	var gear: GameState = _rich_state()
	gear.party.members[0].equipment = {"main_hand": "no_such_item_xyz"}
	var flag: GameState = _rich_state()
	flag.flags["Bad Key"] = true
	checks["dup_id_refused"] = not SaveGame.write(dup, 2).is_empty()
	checks["spend_over_pool_refused"] = not SaveGame.write(over, 2).is_empty()
	checks["unowned_family_refused"] = not SaveGame.write(lo, 2).is_empty()
	checks["missing_item_refused"] = not SaveGame.write(gear, 2).is_empty()
	checks["bad_flag_refused"] = not SaveGame.write(flag, 2).is_empty()
	checks["nothing_written_for_refusals"] = not SaveGame.exists(2)
	checks["slot_zero_refused"] = not SaveGame.write(g, 0).is_empty()
	checks["delete"] = SaveGame.delete(1) and not SaveGame.exists(1) and not SaveGame.delete(1)
	checks["read_missing"] = not DictUtil.to_bool(SaveGame.read(1)["ok"])
	return _report("write and read", checks)

static func _write_raw(slot: int, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(SANDBOX)
	var f: FileAccess = FileAccess.open(SaveGame.path_for(slot), FileAccess.WRITE)
	f.store_string(text)
	f.close()

static func _test_bad_files() -> bool:
	var g: GameState = _rich_state()
	SaveGame.write(g, 1)
	var good_text: String = FileAccess.get_file_as_string(SaveGame.path_for(1))
	_write_raw(3, good_text.substr(0, good_text.length() / 2))
	var truncated: Dictionary = SaveGame.read(3)
	_write_raw(3, "not json at all")
	var garbage: Dictionary = SaveGame.read(3)
	_write_raw(3, JSON.stringify({"format": "other", "version": 1, "game_state": {}}))
	var wrong_format: Dictionary = SaveGame.read(3)
	var env: Dictionary = DictUtil.dict_of(JSON.parse_string(good_text))
	env["version"] = SaveGame.VERSION + 1
	_write_raw(3, JSON.stringify(env))
	var newer: Dictionary = SaveGame.read(3)
	env["version"] = SaveGame.VERSION
	DictUtil.dict_of(env["game_state"])["party"] = {"members": []}
	_write_raw(3, JSON.stringify(env))
	var empty_party: Dictionary = SaveGame.read(3)
	env = DictUtil.dict_of(JSON.parse_string(good_text))
	DictUtil.dict_of(env["game_state"])["future_field"] = 1
	_write_raw(3, JSON.stringify(env))
	var extra: Dictionary = SaveGame.read(3)
	var checks: Dictionary = {
		"truncated": not DictUtil.to_bool(truncated["ok"]) and truncated["state"] == null and not DictUtil.array_of(truncated["errors"]).is_empty(),
		"garbage": not DictUtil.to_bool(garbage["ok"]) and garbage["state"] == null,
		"wrong_format": not DictUtil.to_bool(wrong_format["ok"]) and wrong_format["state"] == null,
		"newer_version": not DictUtil.to_bool(newer["ok"]) and newer["state"] == null and _joined(newer["errors"]).contains("newer version"),
		"invalid_state_rejected": not DictUtil.to_bool(empty_party["ok"]) and empty_party["state"] == null,
		"unknown_key_dropped_with_warning": DictUtil.to_bool(extra["ok"]) and not DictUtil.array_of(extra["warnings"]).is_empty() and (extra["state"] as GameState).equals(g),
	}
	return _report("corrupt, newer, unknown keys", checks)

static func _test_migrations() -> bool:
	SaveMigrations.reset()
	var base: Dictionary = {"format": SaveGame.FORMAT, "version": 1, "saved_at": 0, "game_state": {"world_id": "w", "location_id": "old"}}
	var no_step: Dictionary = SaveMigrations.migrate(base, 2)
	SaveMigrations.register(1, func(state: Dictionary) -> Dictionary:
		var out: Dictionary = state.duplicate(true)
		out["location_id"] = "migrated_" + str(state.get("location_id", ""))
		return out
	)
	var up: Dictionary = SaveMigrations.migrate(base, 2)
	var data: Dictionary = DictUtil.dict_of(up["data"])
	var at_current: Dictionary = SaveMigrations.migrate(base)
	var future: Dictionary = base.duplicate(true)
	future["version"] = SaveGame.VERSION + 1
	var refused: Dictionary = SaveMigrations.migrate(future)
	var chain_base: Dictionary = base.duplicate(true)
	SaveMigrations.register(2, func(state: Dictionary) -> Dictionary:
		var out: Dictionary = state.duplicate(true)
		out["location_id"] = str(state.get("location_id", "")) + "_v3"
		return out
	)
	var chain: Dictionary = SaveMigrations.migrate(chain_base, 3)
	var checks: Dictionary = {
		"missing_step_fails": not DictUtil.to_bool(no_step["ok"]),
		"step_applied": DictUtil.to_bool(up["ok"]) and DictUtil.dict_of(data["game_state"])["location_id"] == "migrated_old" and DictUtil.to_int(data["version"]) == 2,
		"input_not_mutated": DictUtil.dict_of(base["game_state"])["location_id"] == "old" and DictUtil.to_int(base["version"]) == 1,
		"current_version_untouched": DictUtil.to_bool(at_current["ok"]) and DictUtil.to_int(DictUtil.dict_of(at_current["data"])["version"]) == SaveGame.VERSION,
		"newer_refused": not DictUtil.to_bool(refused["ok"]) and _joined(refused["errors"]).contains("newer version"),
		"steps_chain_in_order": DictUtil.to_bool(chain["ok"]) and DictUtil.dict_of(DictUtil.dict_of(chain["data"])["game_state"])["location_id"] == "migrated_old_v3",
		"unknown_keys_warned": not DictUtil.array_of(SaveMigrations.migrate({"version": 1, "game_state": {"bogus": 1}})["warnings"]).is_empty(),
		"missing_version_fails": not DictUtil.to_bool(SaveMigrations.migrate({"game_state": {}})["ok"]),
	}
	SaveMigrations.reset()
	return _report("migrations", checks)
