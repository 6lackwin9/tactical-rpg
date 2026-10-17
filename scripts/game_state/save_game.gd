class_name SaveGame
extends RefCounted

# ==============================================================================
# Tactical RPG - Save game (S13): one versioned JSON file per slot.
#   {"format": "fc_save", "version": 2, "saved_at": <unix>, "game_state": {...}}
# Files live in base_dir() (default user://saves, static override for tests / sandboxes) as
# slot_<n>.json. write() validates first and writes <file>.tmp then renames, so a crash never
# leaves a half file. read() never returns a partially loaded state.
# ==============================================================================

const VERSION: int = 4   # 2: structured quests blob + loot_claimed (step 1 -> 2); 3: exploration keys (step 2 -> 3, EXP1-L); 4: party gold + shops (step 3 -> 4, DLG1-L)
const FORMAT: String = "fc_save"
const DEFAULT_DIR: String = "user://saves"

static var _dir: String = DEFAULT_DIR

static func base_dir() -> String:
	return _dir

static func set_base_dir(path: String) -> void:
	_dir = path

static func reset_base_dir() -> void:
	_dir = DEFAULT_DIR

static func path_for(slot: int) -> String:
	return "%s/slot_%d.json" % [_dir, slot]

static func exists(slot: int) -> bool:
	return FileAccess.file_exists(path_for(slot))

static func delete(slot: int) -> bool:
	if not exists(slot):
		return false
	return DirAccess.remove_absolute(path_for(slot)) == OK

# Problem strings; empty = written.
static func write(state: GameState, slot: int = 1) -> Array[String]:
	var errors: Array[String] = []
	if slot < 1:
		errors.append("Save slot must be >= 1.")
		return errors
	if state == null:
		errors.append("No game state to save.")
		return errors
	errors = state.validate()
	if not errors.is_empty():
		return errors
	var envelope: Dictionary = {
		"format": FORMAT, "version": VERSION, "saved_at": int(Time.get_unix_time_from_system()),
		"game_state": state.to_dict(),
	}
	if DirAccess.make_dir_recursive_absolute(_dir) != OK and not DirAccess.dir_exists_absolute(_dir):
		errors.append("Can't create the save folder '%s'." % _dir)
		return errors
	var final_path: String = path_for(slot)
	var tmp_path: String = final_path + ".tmp"
	var file: FileAccess = FileAccess.open(tmp_path, FileAccess.WRITE)
	if file == null:
		errors.append("Can't open '%s' for writing." % tmp_path)
		return errors
	file.store_string(JSON.stringify(envelope, "\t"))
	file.close()
	if DirAccess.rename_absolute(tmp_path, final_path) != OK:
		# keep the previous save intact: never delete the final file before the new one is in place
		DirAccess.remove_absolute(tmp_path)
		errors.append("Can't move the save into place (the previous save was kept).")
	return errors

# {"ok": bool, "errors": Array[String], "warnings": Array[String], "state": GameState or null}
static func read(slot: int = 1) -> Dictionary:
	var fail: Dictionary = {"ok": false, "errors": [] as Array[String], "warnings": [] as Array[String], "state": null}
	var errors: Array[String] = []
	if not exists(slot):
		errors.append("No save in slot %d." % slot)
		fail["errors"] = errors
		return fail
	var json := JSON.new()
	if json.parse(FileAccess.get_file_as_string(path_for(slot))) != OK or not (json.data is Dictionary):
		errors.append("Save file is corrupt or truncated.")
		fail["errors"] = errors
		return fail
	var envelope: Dictionary = json.data
	if str(envelope.get("format", "")) != FORMAT:
		errors.append("Not a save file (format '%s')." % str(envelope.get("format", "")))
		fail["errors"] = errors
		return fail
	var migrated: Dictionary = SaveMigrations.migrate(envelope)
	var warnings: Array[String] = []
	warnings.assign(DictUtil.array_of(migrated["warnings"]))
	if not DictUtil.to_bool(migrated["ok"]):
		errors.assign(DictUtil.array_of(migrated["errors"]))
		fail["errors"] = errors
		fail["warnings"] = warnings
		return fail
	var data: Dictionary = DictUtil.dict_of(migrated["data"])
	var state: GameState = GameState.from_dict(DictUtil.dict_of(data.get("game_state", {})))
	errors = state.validate()
	if not errors.is_empty():
		fail["errors"] = errors
		fail["warnings"] = warnings
		return fail
	return {"ok": true, "errors": errors, "warnings": warnings, "state": state}
