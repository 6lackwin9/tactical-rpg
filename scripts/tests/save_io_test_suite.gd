class_name SaveIoTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Save export / import test suite (SAVE-IO, S13)
# SaveGame.export_to / import_from / decode and the SaveSlotsModel export and import actions. Fixtures: new games of
# the shipped Ashlands and Tidewrack (read only), saves and exported files in a sandbox under user://.
# ==============================================================================

const TAG: String = "SaveIO"
static var SANDBOX: String = TestHelpers.sandbox("save_io")

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING SAVE EXPORT / IMPORT TEST (SAVE-IO)     ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/files")
	SaveGame.set_base_dir(SANDBOX + "/saves")
	var ash: GameState = NewGame.create_for_world("ashlands", 3)["state"] as GameState
	var tide: GameState = NewGame.create_for_world("tidewrack", 4)["state"] as GameState
	var ok: bool = _report("Fixtures", {"sandboxed": SaveGame.base_dir().begins_with("user://"), "two worlds": ash != null and tide != null})
	if ok:
		ok = _test_export(ash) and ok
		ok = _test_import(ash, tide) and ok
		ok = _test_refusals(ash) and ok
		ok = _test_model(ash) and ok
		ok = _test_safety(ash) and ok
	CurrentGame.end()
	SaveGame.reset_base_dir()
	TestHelpers.remove_sandbox(SANDBOX)
	return ok

static func _file(name: String) -> String:
	return SANDBOX + "/files/" + name

static func _test_export(gs: GameState) -> bool:
	var checks: Dictionary = {}
	checks["written"] = SaveGame.write(gs, 2).is_empty()
	var path: String = _file("ash.json")
	checks["export"] = SaveGame.export_to(2, path).is_empty() and FileAccess.get_file_as_string(path) == FileAccess.get_file_as_string(SaveGame.path_for(2))
	checks["no tmp left"] = not FileAccess.file_exists(path + SaveGame.TMP_EXT) and not FileAccess.file_exists(path + SaveGame.BAK_EXT)
	checks["re-export overwrites"] = SaveGame.export_to(2, path).is_empty()
	checks["empty slot refused"] = SaveGame.export_to(4, _file("none.json")) == ["No save in slot 4."] and not FileAccess.file_exists(_file("none.json"))
	checks["no path refused"] = SaveGame.export_to(2, " ") == ["Pick a file to export to."]
	return _report("Export copies a readable slot", checks)

static func _test_import(ash: GameState, tide: GameState) -> bool:
	var checks: Dictionary = {}
	var _w: Array[String] = SaveGame.write(tide, 1)
	var tide_file: String = _file("tide.json")
	var _e: Array[String] = SaveGame.export_to(1, tide_file)
	var old_at: int = DictUtil.to_int(SaveGame.peek(1)["saved_at"])
	var _d: bool = SaveGame.delete(1)
	var res: Dictionary = SaveGame.import_from(tide_file, 5)
	var back: Dictionary = SaveGame.read(5)
	checks["imported"] = DictUtil.to_bool(res["ok"]) and DictUtil.to_bool(back["ok"])
	checks["same state"] = DictUtil.to_bool(back["ok"]) and JsonUtil.deep_equal((back["state"] as GameState).to_dict(), tide.to_dict())
	checks["other world kept"] = str(SaveGame.peek(5)["world_id"]) == "tidewrack"
	checks["save time kept"] = DictUtil.to_int(SaveGame.peek(5)["saved_at"]) == old_at
	# the autosave written after the export stays the newest save (Continue resumes it)
	var auto: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(tide_file))
	auto["saved_at"] = old_at - 100
	var older: String = _file("older.json")
	var f: FileAccess = FileAccess.open(older, FileAccess.WRITE)
	f.store_string(JSON.stringify(auto))
	f.close()
	var _a: Array[String] = SaveGame.write(ash, SaveGame.AUTOSAVE_SLOT, old_at)
	checks["older import below autosave"] = DictUtil.to_bool(SaveGame.import_from(older, 3)["ok"]) and SaveGame.newest_slot() != 3
	return _report("Import reads any world into a manual slot", checks)

static func _test_refusals(gs: GameState) -> bool:
	var checks: Dictionary = {}
	var path: String = _file("ash.json")
	checks["autosave slot refused"] = JsonUtil.strings(SaveGame.import_from(path, SaveGame.AUTOSAVE_SLOT)["errors"]) == ["Import into a manual slot (1-%d)." % SaveGame.LAST_SLOT]
	checks["missing file"] = not DictUtil.to_bool(SaveGame.import_from(_file("nope.json"), 4)["ok"]) and not SaveGame.exists(4)
	var bad: String = _file("bad.json")
	var f: FileAccess = FileAccess.open(bad, FileAccess.WRITE)
	f.store_string("{ not json")
	f.close()
	var _w: Array[String] = SaveGame.write(gs, 4)
	var before: String = FileAccess.get_file_as_string(SaveGame.path_for(4))
	checks["corrupt refused, slot kept"] = JsonUtil.strings(SaveGame.import_from(bad, 4)["errors"]) == ["Save file is corrupt or truncated."] and FileAccess.get_file_as_string(SaveGame.path_for(4)) == before
	var other: String = _file("other.json")
	f = FileAccess.open(other, FileAccess.WRITE)
	f.store_string(JSON.stringify({"format": "something_else"}))
	f.close()
	checks["wrong format refused"] = JsonUtil.strings(SaveGame.import_from(other, 4)["errors"]) == ["Not a save file (format 'something_else')."]
	var broken: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(path))
	DictUtil.dict_of(broken["game_state"])["world_id"] = ""
	var invalid: String = _file("invalid.json")
	f = FileAccess.open(invalid, FileAccess.WRITE)
	f.store_string(JSON.stringify(broken))
	f.close()
	checks["invalid state refused"] = not DictUtil.to_bool(SaveGame.import_from(invalid, 4)["ok"]) and FileAccess.get_file_as_string(SaveGame.path_for(4)) == before
	return _report("Import refusals leave slots untouched", checks)

static func _test_model(gs: GameState) -> bool:
	var checks: Dictionary = {}
	CurrentGame.start(gs)
	var router := MenuRouter.new(MenuRouter.LOAD)
	var m: SaveSlotsModel = router.model() as SaveSlotsModel
	var rows: Array[Dictionary] = m.rows()
	checks["row flags"] = not DictUtil.to_bool(rows[0]["can_import"]) and DictUtil.to_bool(rows[1]["can_import"]) and DictUtil.to_bool(rows[2]["can_export"])
	checks["export name"] = SaveSlotsModel.export_name(2, "ashlands", 0) == "ashlands_slot2_save.json" and str(rows[2]["export_name"]).begins_with("ashlands_slot2_") and str(rows[2]["export_name"]).ends_with(".json")
	var notices: Array[int] = [0]
	router.notice.connect(func(_l: Array[String]) -> void: notices[0] += 1)
	checks["export action"] = m.export_slot(2, _file("model.json")).is_empty() and FileAccess.file_exists(_file("model.json")) and notices[0] == 1
	var _d: bool = SaveGame.delete(1)
	checks["import into empty"] = m.import_file(_file("tide.json"), 1).is_empty() and not router.is_asking() and str(SaveGame.peek(1)["world_id"]) == "tidewrack"
	checks["occupied asks"] = m.import_file(_file("model.json"), 1).is_empty() and router.is_asking() and str(SaveGame.peek(1)["world_id"]) == "tidewrack"
	router.confirm()
	checks["confirm replaces"] = str(SaveGame.peek(1)["world_id"]) == "ashlands"
	checks["autosave refused"] = m.import_file(_file("model.json"), 0) == [SaveSlotsModel.MANUAL_ONLY]
	return _report("SaveSlotsModel export and import", checks)

static func _put(path: String, text: String) -> void:
	var f: FileAccess = FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()

# REVIEW-F1: export keeps the previous file, import clamps the save time and refuses huge files.
static func _test_safety(gs: GameState) -> bool:
	var checks: Dictionary = {}
	var _w: Array[String] = SaveGame.write(gs, 2)
	var slot_text: String = FileAccess.get_file_as_string(SaveGame.path_for(2))
	# a normal export over an existing file replaces it and leaves no helper files
	var path: String = _file("over.json")
	_put(path, "old export")
	checks["replaces"] = SaveGame.export_to(2, path).is_empty() and FileAccess.get_file_as_string(path) == slot_text
	checks["no .bak / .fc_tmp left"] = not FileAccess.file_exists(path + SaveGame.BAK_EXT) and not FileAccess.file_exists(path + SaveGame.TMP_EXT)
	# a user file named like the old temp name is never touched
	var mine: String = path + ".tmp"
	_put(mine, "mine")
	checks["user .tmp file kept"] = SaveGame.export_to(2, path).is_empty() and FileAccess.get_file_as_string(mine) == "mine"
	# a free temp name is chosen when the first one is taken
	_put(path + SaveGame.TMP_EXT, "taken")
	checks["taken tmp name skipped"] = SaveGame.export_to(2, path).is_empty() and FileAccess.get_file_as_string(path + SaveGame.TMP_EXT) == "taken"
	# a user <target>.bak file survives an export over an existing target, byte-identical
	var guarded: String = _file("guarded.json")
	_put(guarded, "old")
	_put(guarded + SaveGame.BAK_EXT, "user bak")
	checks["user .bak kept"] = SaveGame.export_to(2, guarded).is_empty() and FileAccess.get_file_as_string(guarded + SaveGame.BAK_EXT) == "user bak" and FileAccess.get_file_as_string(guarded) == slot_text
	checks["numbered bak removed"] = not FileAccess.file_exists(guarded + SaveGame.BAK_EXT + "1")
	# forced rename failure (root ignores read-only folders, so the target is a folder): refused, nothing left behind
	var held: String = _file("held_dir")
	DirAccess.make_dir_recursive_absolute(held)
	var errs: Array[String] = SaveGame.export_to(2, held)
	checks["failed move reports and keeps the target"] = errs == ["Can't move the export into place at '%s' (the previous file was kept)." % held] and DirAccess.dir_exists_absolute(held)
	checks["failed move leaves no tmp"] = not FileAccess.file_exists(held + SaveGame.TMP_EXT)
	# import: future saved_at is clamped
	var env: Dictionary = DictUtil.dict_of(JSON.parse_string(slot_text))
	var now: int = int(Time.get_unix_time_from_system())
	env["saved_at"] = now + 86400
	var future: String = _file("future.json")
	_put(future, JSON.stringify(env))
	checks["future import ok"] = DictUtil.to_bool(SaveGame.import_from(future, 3)["ok"])
	checks["future saved_at clamped"] = DictUtil.to_int(SaveGame.peek(3)["saved_at"]) <= int(Time.get_unix_time_from_system())
	# import: files over 8 MiB are refused before reading
	var _w4: Array[String] = SaveGame.write(gs, 4)
	var before: String = FileAccess.get_file_as_string(SaveGame.path_for(4))
	var big: String = _file("big.json")
	_put(big, slot_text + " ".repeat(SaveGame.MAX_IMPORT_BYTES + 1 - slot_text.length() + 1024 * 1024))
	checks["big file refused"] = JsonUtil.strings(SaveGame.import_from(big, 4)["errors"]) == ["File is too large to be a save."]
	checks["slot untouched"] = FileAccess.get_file_as_string(SaveGame.path_for(4)) == before
	return _report("Export keeps the previous file; import clamps time and size", checks)
