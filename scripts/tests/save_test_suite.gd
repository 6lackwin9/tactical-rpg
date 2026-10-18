class_name SaveTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Save Test Suite (M3 / S13)
# GameState round trip, flags, SaveGame write / read (sandbox directory only), corrupt and newer
# files, the migration hook, unknown-key warnings, and NewGame. The real user://saves is never touched.
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("m3_save_test")

const TAG: String = "Save"

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
	ok = _test_peek_versions() and ok
	ok = _test_heal1_schema() and ok
	ok = _test_purse1_save() and ok
	_wipe()
	SaveGame.reset_base_dir()
	SaveMigrations.reset()
	var real_after: PackedStringArray = DirAccess.get_files_at(SaveGame.DEFAULT_DIR)
	ok = TestHelpers.report(TAG, "real saves untouched", {"listing_unchanged": real_before == real_after, "dir_restored": SaveGame.base_dir() == SaveGame.DEFAULT_DIR}) and ok
	return ok

static func _joined(v: Variant) -> String:
	var parts: PackedStringArray = PackedStringArray()
	for x: Variant in DictUtil.array_of(v):
		parts.append(str(x))
	return ", ".join(parts)

static func _wipe() -> void:
	TestHelpers.remove_sandbox(SANDBOX)

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
	var g: GameState = GameState.of(res["state"])
	var m: UnitState = g.party.members[0]
	m.total_xp = XpCurve.xp_for_level(4) + 7
	LevelUp.confirm(m, {"STR": 1, "KNOW": 2})
	var pids: Array[String] = ContentCatalog.ids("passives")
	if not pids.is_empty():
		m.passives = [pids[0]] as Array[String]
	g.party.add_to_stash(str(ContentCatalog.entries("items")[0]["id"]), 2)
	g.location_id = "qa:somewhere"   # qualified (save v3)
	g.set_flag("intro.done", true)
	g.set_flag("count", 7)
	g.set_flag("name", "x")
	g.quests = {"qa:q1": {"state": "active", "objective": 2}, "qa:q2": {"state": "completed", "objective": 1}}
	g.loot_claimed = ["qa:zeta", "qa:alpha"] as Array[String]
	g.party.gold = 42   # save v4 (DLG1-L)
	g.shops = {"qa:stall": {str(ContentCatalog.entries("items")[0]["id"]): 1}}
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
	bad_quest.quests = {"qa:q": {"state": "done", "objective": 0}}
	checks["validate_bad_quest_shape"] = not bad_quest.validate().is_empty()
	var no_world: GameState = _rich_state()
	no_world.world_id = ""
	checks["validate_world_required"] = not no_world.validate().is_empty()
	var text: String = JSON.stringify(g.to_dict())
	var back: GameState = GameState.from_dict(DictUtil.dict_of(JSON.parse_string(text)))
	checks["json_roundtrip_equal"] = back.equals(g)
	checks["ints_preserved"] = back.party.members[0].total_xp is int and back.flags["intro.done"] == true and back.quests["qa:q1"]["objective"] is int
	checks["loot_claimed_saved_sorted"] = JsonUtil.strings(g.to_dict()["loot_claimed"]) == ["qa:alpha", "qa:zeta"]
	var changed: GameState = GameState.from_dict(g.to_dict())
	changed.party.members[0].total_xp += 1
	checks["equals_detects_difference"] = not changed.equals(g)
	return TestHelpers.report(TAG, "game state", checks)

static func _test_new_game() -> bool:
	var uid: String = _party_unit()
	var good: Dictionary = NewGame.create(_world(), [uid, uid] as Array[String])
	var state: GameState = GameState.of(good["state"])
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
	return TestHelpers.report(TAG, "new game", checks)

static func _test_write_read() -> bool:
	var g: GameState = _rich_state()
	var checks: Dictionary = {"no_file_yet": not SaveGame.exists(1)}
	checks["write_ok"] = SaveGame.write(g, 1).is_empty()
	checks["exists"] = SaveGame.exists(1)
	checks["no_tmp_left"] = not FileAccess.file_exists(SaveGame.path_for(1) + ".tmp")
	checks["inside_sandbox"] = SaveGame.path_for(1).begins_with(SANDBOX)
	var res: Dictionary = SaveGame.read(1)
	checks["read_ok"] = DictUtil.to_bool(res["ok"]) and res["state"] != null
	var back: GameState = GameState.of(res["state"])
	checks["equal_after_read"] = back != null and back.equals(g)
	g.party.members[0].total_xp += 5
	checks["overwrite_ok"] = SaveGame.write(g, 1).is_empty() and not FileAccess.file_exists(SaveGame.path_for(1) + ".tmp")
	var back2: GameState = GameState.of(SaveGame.read(1)["state"])
	checks["overwrite_visible"] = back2 != null and back2.equals(g)
	# Refuse invalid states without touching the existing file.
	var broken: GameState = _rich_state()
	broken.party.members[0].base_unit = "no_such_unit_xyz"
	checks["invalid_refused"] = not SaveGame.write(broken, 1).is_empty()
	var still: GameState = GameState.of(SaveGame.read(1)["state"])
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
	checks["slot_out_of_range_refused"] = SaveGame.write(g, 6) == ["Save slot must be 0-5."] and SaveGame.write(g, -1) == ["Save slot must be 0-5."]   # UI1-L: 0 = autosave
	checks["delete"] = SaveGame.delete(1) and not SaveGame.exists(1) and not SaveGame.delete(1)
	checks["read_missing"] = not DictUtil.to_bool(SaveGame.read(1)["ok"])
	return TestHelpers.report(TAG, "write and read", checks)

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
		"unknown_key_dropped_with_warning": DictUtil.to_bool(extra["ok"]) and not DictUtil.array_of(extra["warnings"]).is_empty() and GameState.of(extra["state"]).equals(g),
		"GameState.of: null for null / int / Dictionary, a GameState returns itself": GameState.of(null) == null and GameState.of(42) == null and GameState.of({}) == null and GameState.of(g) == g,
	}
	return TestHelpers.report(TAG, "corrupt, newer, unknown keys", checks)

static func _test_migrations() -> bool:
	SaveMigrations.reset()
	var base: Dictionary = {"format": SaveGame.FORMAT, "version": 1, "saved_at": 0, "game_state": {"world_id": "w", "location_id": "old"}}
	var no_step: Dictionary = SaveMigrations.migrate(base, SaveGame.VERSION + 1)   # the built-in table stops at VERSION
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
	SaveMigrations.reset()
	var v3: Dictionary = SaveMigrations.migrate({"version": 3, "game_state": {"world_id": "w", "party": {"members": []}}}, 4)
	var v3_state: Dictionary = DictUtil.dict_of(DictUtil.dict_of(v3["data"]).get("game_state", {}))
	var v4_in: Dictionary = {"world_id": "w", "party": {"members": [], "gold": 7}, "shops": {}}
	var v4: Dictionary = SaveMigrations.migrate({"version": 4, "game_state": v4_in}, 5)
	var v4_expect: Dictionary = v4_in.duplicate(true)
	v4_expect["time"] = 0
	var checks: Dictionary = {
		"v4_to_v5_adds_time_only": DictUtil.to_bool(v4["ok"]) and JsonUtil.deep_equal(DictUtil.dict_of(v4["data"]).get("game_state", {}), v4_expect) and SaveGame.VERSION >= 5,   # 6 since TRAV2-L
		"v3_to_v4_adds_gold_and_shops": DictUtil.to_bool(v3["ok"]) and DictUtil.dict_of(v3_state.get("party", {})).get("gold", -1) == 0 and JsonUtil.deep_equal(v3_state.get("shops", null), {}) and SaveGame.VERSION >= 4,   # 5 since TRAV1-L
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
	return TestHelpers.report(TAG, "migrations", checks)

# REVIEW-F1: version-less / zero / negative files are unreadable for peek, newest_slot and can_continue.
static func _test_peek_versions() -> bool:
	_wipe()
	var good: GameState = _rich_state()
	var _w: Array[String] = SaveGame.write(good, 1)
	var env: Dictionary = DictUtil.dict_of(JSON.parse_string(FileAccess.get_file_as_string(SaveGame.path_for(1))))
	var _d: bool = SaveGame.delete(1)
	var checks: Dictionary = {}
	var bad_versions: Array[Variant] = [null, 0, -1, "x"]
	for v: Variant in bad_versions:
		var e: Dictionary = env.duplicate(true)
		if v == null:
			e.erase("version")
		else:
			e["version"] = v
		e["saved_at"] = 4102444800
		_write_raw(2, JSON.stringify(e))
		var head: Dictionary = SaveGame.peek(2)
		checks["version %s unreadable" % str(v)] = not DictUtil.to_bool(head["ok"]) and str(head["error"]) == "unreadable" and SaveGame.newest_slot() == -1
	checks["can_continue false with only such a file"] = SaveGame.newest_slot() == -1
	var e1: Dictionary = env.duplicate(true)
	e1["version"] = 1
	_write_raw(3, JSON.stringify(e1))
	checks["version 1 still peeks ok"] = DictUtil.to_bool(SaveGame.peek(3)["ok"]) and SaveGame.newest_slot() == 3
	_wipe()
	return TestHelpers.report(TAG, "peek refuses version-less, 0 and negative files", checks)

# HEAL1-L 4.1-4.2 (acceptance check 9): UnitState.hp_lost, explore.last_rest, the v8 -> v9 step and the location `rest` field.
static func _test_heal1_schema() -> bool:
	SaveMigrations.reset()
	var g: GameState = _rich_state()
	var checks: Dictionary = {"new state: hp_lost 0, last_rest empty": g.party.members[0].hp_lost == 0 and str(g.explore.get("last_rest", "x")).is_empty()}
	g.party.members[0].hp_lost = 37
	g.explore["last_rest"] = "qa:keep"
	var back: GameState = GameState.from_dict(DictUtil.dict_of(JSON.parse_string(JSON.stringify(g.to_dict()))))
	checks["hp_lost and last_rest round-trip"] = g.validate().is_empty() and back.equals(g) and back.party.members[0].hp_lost == 37 and back.party.members[1].hp_lost == 0
	checks["duplicate_state keeps hp_lost"] = g.party.members[0].duplicate_state().hp_lost == 37
	var other: UnitState = g.party.members[0].duplicate_state()
	other.hp_lost = 36
	checks["equals sees hp_lost"] = not other.equals(g.party.members[0])
	var raw: Dictionary = g.party.members[0].to_dict()
	raw.erase("hp_lost")
	checks["missing hp_lost reads 0"] = UnitState.from_dict(raw).hp_lost == 0
	raw["hp_lost"] = 12.0
	checks["whole float reads as int"] = UnitState.from_dict(raw).hp_lost == 12
	var refused: Array[Variant] = [1.5, "x", -3]
	for bad: Variant in refused:
		raw["hp_lost"] = bad
		var u: UnitState = UnitState.from_dict(raw)
		checks["hp_lost %s refused by validate" % str(bad)] = u.hp_lost < 0 and _joined(u.validate()).contains("hp_lost must be 0 or more.")
	var bad_rest: GameState = GameState.from_dict(g.to_dict())
	bad_rest.explore["last_rest"] = "not qualified"
	checks["unqualified last_rest refused"] = not bad_rest.validate().is_empty()
	var no_rest: GameState = GameState.from_dict(g.to_dict())
	no_rest.explore.erase("last_rest")
	checks["explore without last_rest refused"] = not no_rest.validate().is_empty()
	var home: GameState = GameState.from_dict(g.to_dict())
	home.explore[GameState.HOME_WORLD_KEY] = _world()
	checks["home_world stays optional"] = home.validate().is_empty()
	# v8 -> v9: hp_lost 0 on every member and explore.last_rest "", nothing else.
	var v8_state: Dictionary = g.to_dict()
	for m: Variant in DictUtil.array_of(DictUtil.dict_of(v8_state["party"]).get("members", [])):
		DictUtil.dict_of(m).erase("hp_lost")
	DictUtil.dict_of(v8_state["explore"]).erase("last_rest")
	var up: Dictionary = SaveMigrations.migrate({"version": 8, "game_state": v8_state}, 9)
	var expect: Dictionary = v8_state.duplicate(true)
	for m: Variant in DictUtil.array_of(DictUtil.dict_of(expect["party"]).get("members", [])):
		DictUtil.dict_of(m)["hp_lost"] = 0
	DictUtil.dict_of(expect["explore"])["last_rest"] = ""
	var got: Dictionary = DictUtil.dict_of(DictUtil.dict_of(up["data"]).get("game_state", {}))
	checks["v8_to_v9 adds hp_lost 0 and last_rest only"] = DictUtil.to_bool(up["ok"]) and JsonUtil.deep_equal(got, expect) and DictUtil.array_of(up["warnings"]).is_empty()
	var loaded: GameState = GameState.from_dict(got)
	checks["v8 save loads valid at full HP"] = loaded.validate().is_empty() and loaded.party.members[0].hp_lost == 0 and str(loaded.explore["last_rest"]).is_empty()
	var odd: Dictionary = SaveMigrations.migrate({"version": 8, "game_state": {"world_id": "w", "party": "x", "explore": 3}}, 9)
	checks["v8_to_v9 skips non-object party / explore"] = DictUtil.to_bool(odd["ok"]) and str(DictUtil.dict_of(DictUtil.dict_of(odd["data"])["game_state"]).get("party", "")) == "x"
	checks["SaveGame.VERSION >= 9"] = SaveGame.VERSION >= 9
	# Location field `rest` (4.2): bool, default false, kept through normalize.
	var rest_field: Dictionary = WorldSchema.field("locations", "rest")
	checks["location rest is a bool field, default false"] = str(rest_field.get("type", "")) == "bool" and rest_field.get("default", null) == false
	checks["normalize: missing rest = false, true kept"] = WorldSchema.normalize("locations", {"id": "a"}).get("rest", null) == false and WorldSchema.normalize("locations", {"id": "a", "rest": true}).get("rest", null) == true
	return TestHelpers.report(TAG, "HEAL1 save v9 and location rest", checks)

# PURSE1-L (spec 3, acceptance "Save"): GameState.purses round-trips through a slot, bad blobs are refused, and the
# v9 -> v10 step adds purses {} and nothing else.
static func _test_purse1_save() -> bool:
	var checks: Dictionary = {}
	var g: GameState = _rich_state()
	checks["new state has no purse wins"] = g.purses.is_empty() and g.validate().is_empty()
	g.purses = {"qa:fight_a": 2, "qa:fight_b": 0}
	checks["purses valid"] = g.validate().is_empty()
	var written: bool = SaveGame.write(g, 2).is_empty()
	var read: Dictionary = SaveGame.read(2)
	var back: GameState = GameState.of(read.get("state", null))
	checks["purses round-trip through a slot"] = written and back != null and back.equals(g) and DictUtil.to_int(back.purses.get("qa:fight_a", 0)) == 2
	for bad: Variant in [{"fight_a": 1}, {"qa:fight_a": -1}, {"qa:fight_a": "1"}, {"qa:fight_a": 1.5}]:
		var b: GameState = GameState.from_dict(g.to_dict())
		b.purses = DictUtil.dict_of(bad)
		checks["purses %s refused" % str(bad)] = _joined(b.validate()).contains("purses: ")
	var raw: Dictionary = g.to_dict()
	raw["purses"] = [1]
	checks["purses not an object refused"] = _joined(GameState.from_dict(raw).validate()).contains("purses must be an object")
	var v9_state: Dictionary = g.to_dict()
	v9_state.erase("purses")
	var up: Dictionary = SaveMigrations.migrate({"version": 9, "game_state": v9_state}, 10)
	var expect: Dictionary = v9_state.duplicate(true)
	expect["purses"] = {}
	var got: Dictionary = DictUtil.dict_of(DictUtil.dict_of(up["data"]).get("game_state", {}))
	checks["v9_to_v10 adds purses {} only"] = DictUtil.to_bool(up["ok"]) and JsonUtil.deep_equal(got, expect) and DictUtil.array_of(up["warnings"]).is_empty()
	checks["v9 save loads valid with no purse wins"] = GameState.from_dict(got).validate().is_empty() and GameState.from_dict(got).purses.is_empty()
	checks["SaveGame.VERSION >= 10"] = SaveGame.VERSION >= 10
	return TestHelpers.report(TAG, "PURSE1 save v10 (purses)", checks)
