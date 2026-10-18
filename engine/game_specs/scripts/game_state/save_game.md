---
class: SaveGame
extends: RefCounted
description: "Tactical RPG - Save game (S13): one versioned JSON file per slot. {\"format\": \"fc_save\", \"version\": 2, \"saved_at\": <unix>, \"game_state\": {...}} Files live in base_dir() (default user://saves, static override for tests / sandboxes) as slot_<n>.json. write() validates first and writes <file>.tmp then renames, so a crash never leaves a half file. read() never returns a partially loaded state. Slots (UI1-L): 0 is the autosave, 1..5 the manual slots (one list shared by all worlds); peek() reads a slot header without migrating or loading. A VERSION bump needs a new scripts/tests/fixtures/saves/save_v<N>.json in the same task (tools/gen_save_fixtures.gd); a format-breaking change updates the fixtures too. SaveCompatTestSuite fails otherwise."
---

# Properties
* `VERSION`: int = 9   # 2: structured quests blob + loot_claimed (step 1 -> 2); 3: exploration keys (step 2 -> 3, EXP1-L); 4: party gold + shops (step 3 -> 4, DLG1-L); 5: world clock time (step 4 -> 5, TRAV1-L); 6: party supplies (step 5 -> 6, TRAV2-L); 7: battle autoplay choice (step 6 -> 7, AUTO1-L); 8: per-hero battle pouch (step 7 -> 8, ITEMUSE1-L); 9: stored hero HP hp_lost + explore.last_rest (step 8 -> 9, HEAL1-L) (const)
* `FORMAT`: String = "fc_save" (const)
* `DEFAULT_DIR`: String = "user://saves" (const)
* `AUTOSAVE_SLOT`: int = 0 (const)
* `MANUAL_SLOTS`: Array[int] = [1, 2, 3, 4, 5] (const)
* `LAST_SLOT`: int = 5 (const)
* `TMP_EXT`: String = ".fc_tmp" (const)
* `BAK_EXT`: String = ".bak" (const)
* `MAX_IMPORT_BYTES`: int = 8 * 1024 * 1024 (const)
* `errors`: Array[String] = []
* `envelope`: Dictionary = {
* `final_path`: String = path_for(slot)
* `tmp_path`: String = _free_path(final_path, TMP_EXT)
* `file`: FileAccess = FileAccess.open(tmp_path, FileAccess.WRITE)
* `wrote`: bool = file.store_string(JSON.stringify(envelope, "\t"))
* `write_ok`: bool = wrote and file.get_error() == OK
* `errors`: Array[String] = []
* `res`: Dictionary = read(slot)
* `tmp_path`: String = _free_path(path, TMP_EXT)
* `file`: FileAccess = FileAccess.open(tmp_path, FileAccess.WRITE)
* `wrote`: bool = file.store_string(FileAccess.get_file_as_string(path_for(slot)))
* `write_ok`: bool = wrote and file.get_error() == OK
* `had_old`: bool = FileAccess.file_exists(path)
* `bak_path`: String = _free_path(path, BAK_EXT) if had_old else ""

# Methods

### `static base_dir() -> String`
```logic
return _dir
```

### `static set_base_dir(path: String) -> void`
```logic
_dir = path
```

### `static reset_base_dir() -> void`
```logic
_dir = DEFAULT_DIR
```

### `static path_for(slot: int) -> String`
```logic
return "%s/slot_%d.json" % [_dir, slot]
```

### `static exists(slot: int) -> bool`
```logic
return FileAccess.file_exists(path_for(slot))
```

### `static delete(slot: int) -> bool`
```logic
if not exists(slot):
	return false
return DirAccess.remove_absolute(path_for(slot)) == OK
```

### `static _free_path(path: String, suffix: String) -> String`
```logic
var tmp: String = path + suffix
var n: int = 1
while FileAccess.file_exists(tmp) or DirAccess.dir_exists_absolute(tmp):
	tmp = "%s%s%d" % [path, suffix, n]
	n += 1
return tmp
```

### `static read(slot: int = 1) -> Dictionary`
```logic
if not exists(slot):
	var errors: Array[String] = ["No save in slot %d." % slot]
	return {"ok": false, "errors": errors, "warnings": [] as Array[String], "state": null}
return decode(FileAccess.get_file_as_string(path_for(slot)))
```

### `static decode(text: String) -> Dictionary`
```logic
var fail: Dictionary = {"ok": false, "errors": [] as Array[String], "warnings": [] as Array[String], "state": null, "saved_at": 0}
var errors: Array[String] = []
var json := JSON.new()
if json.parse(text) != OK or not (json.data is Dictionary):
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
return {"ok": true, "errors": errors, "warnings": warnings, "state": state, "saved_at": DictUtil.to_int(envelope.get("saved_at", 0))}
```

### `static import_from(path: String, slot: int) -> Dictionary`
```logic
var errors: Array[String] = []
var out: Dictionary = {"ok": false, "errors": errors, "warnings": [] as Array[String]}
if not MANUAL_SLOTS.has(slot):
	errors.append("Import into a manual slot (1-%d)." % LAST_SLOT)
	return out
if not FileAccess.file_exists(path):
	errors.append("No file at '%s'." % path)
	return out
var size_file: FileAccess = FileAccess.open(path, FileAccess.READ)
if size_file == null:
	errors.append("Can't open '%s'." % path)
	return out
var too_big: bool = size_file.get_length() > MAX_IMPORT_BYTES
size_file.close()
if too_big:
	errors.append("File is too large to be a save.")
	return out
var res: Dictionary = decode(FileAccess.get_file_as_string(path))
out["warnings"] = res["warnings"]
if not DictUtil.to_bool(res["ok"]):
	errors.assign(DictUtil.array_of(res["errors"]))
	out["errors"] = errors
	return out
var stamp: int = mini(DictUtil.to_int(res["saved_at"]), int(Time.get_unix_time_from_system()))
errors = write(GameState.of(res["state"]), slot, stamp)
out["errors"] = errors
out["ok"] = errors.is_empty()
return out
```

### `static peek(slot: int) -> Dictionary`
```logic
var party: Array[Dictionary] = []
var out: Dictionary = {"exists": exists(slot), "ok": false, "slot": slot, "saved_at": 0, "version": 0, "world_id": "", "location_id": "", "party": party, "error": ""}
if not DictUtil.to_bool(out["exists"]):
	return out
var json := JSON.new()
if json.parse(FileAccess.get_file_as_string(path_for(slot))) != OK or not (json.data is Dictionary):
	out["error"] = "unreadable"
	return out
var envelope: Dictionary = json.data
if str(envelope.get("format", "")) != FORMAT:
	out["error"] = "unreadable"
	return out
var raw_version: Variant = envelope.get("version", null)
if not (raw_version is int or raw_version is float) or DictUtil.to_int(raw_version) < 1:
	out["error"] = "unreadable"
	return out
out["saved_at"] = DictUtil.to_int(envelope.get("saved_at", 0))
out["version"] = DictUtil.to_int(envelope.get("version", 0))
var gs: Dictionary = DictUtil.dict_of(envelope.get("game_state", {}))
out["world_id"] = str(gs.get("world_id", ""))
out["location_id"] = str(gs.get("location_id", ""))
for raw: Variant in DictUtil.array_of(DictUtil.dict_of(gs.get("party", {})).get("members", [])):
	var m: Dictionary = DictUtil.dict_of(raw)
	var level: int = XpCurve.level_for(DictUtil.to_int(m["total_xp"])) if m.has("total_xp") else 0
	party.append({"id": str(m.get("id", "")), "base_unit": str(m.get("base_unit", "")), "level": level})
if DictUtil.to_int(out["version"]) > VERSION:
	out["error"] = "newer version"
	return out
out["ok"] = true
return out
```

### `static newest_slot() -> int`
```logic
var best: int = -1
var best_at: int = -1
for slot: int in range(AUTOSAVE_SLOT, LAST_SLOT + 1):
	var head: Dictionary = peek(slot)
	if DictUtil.to_bool(head["ok"]) and DictUtil.to_int(head["saved_at"]) > best_at:
		best = slot
		best_at = DictUtil.to_int(head["saved_at"])
return best
```

