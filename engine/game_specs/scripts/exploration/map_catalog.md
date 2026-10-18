---
class: MapCatalog
extends: RefCounted
description: "Tactical RPG - Map Catalog (S9, spec EXP1-L 1.1): lore-free region map templates res://data/maps/maps.json: {\"kind\": \"map_templates\", \"version\": 1, \"entries\": [...]} A template is {id, name, rows, zones, zone_legend, anchors, paths, tags} (MapRules validates it). World locations bind a template by id (WorldSchema location `map`, `zone_bindings`, `markers`). Same contract as QuestCatalog: upsert validates and stores (nothing changes on error), save writes the file, a file that loaded with errors refuses to save. Tests call use_path(\"user://...\") and never write res://. State lives on the Engine singleton (hot reloads can't reset it)."
---

# Properties
* `DEFAULT_PATH`: String = "res://data/maps/maps.json" (const)
* `FILE_KIND`: String = "map_templates" (const)
* `VERSION`: int = 1 (const)
* `STATE_KEY`: StringName = &"map_catalog_state" (const)
* `errors`: Array[String] = _state()["errors"]
* `out`: Array[Dictionary] = []
* `out`: Array[String] = []
* `e`: Dictionary = normalize(entry)
* `errors`: Array[String] = []
* `id`: String = str(e["id"])
* `clean`: Dictionary = normalize(entry)
* `new_id`: String = str(clean["id"])
* `old_id`: String = replace_id if not replace_id.is_empty() else new_id
* `errors`: Array[String] = validate_entry(clean, old_id)
* `users`: Array[String] = used_by(old_id)
* `at`: int = _index_of(old_id)
* `out`: Array[String] = []

# Methods

### `static _state() -> Dictionary`
```logic
if not Engine.has_meta(STATE_KEY):
	var errors: Array[String] = []
	Engine.set_meta(STATE_KEY, {"path": DEFAULT_PATH, "entries": [], "errors": errors, "dirty": false, "blocked": false, "loaded": false})
var state: Dictionary = Engine.get_meta(STATE_KEY)
return state
```

### `static _list() -> Array`
```logic
var list: Array = _state()["entries"]
return list
```

### `static _ensure_loaded() -> void`
```logic
if not DictUtil.to_bool(_state()["loaded"]):
	reload_from_disk()
```

### `static file_path() -> String`
```logic
return str(_state()["path"])
```

### `static use_path(p_path: String) -> void`
```logic
_state()["path"] = p_path
reload_from_disk()
```

### `static clear_cache() -> void`
```logic
if Engine.has_meta(STATE_KEY):
	Engine.remove_meta(STATE_KEY)
```

### `static reload_from_disk() -> void`
```logic
_list().clear()
_error_list().clear()
_state()["dirty"] = false
_state()["blocked"] = false
_state()["loaded"] = true
var path: String = file_path()
if not FileAccess.file_exists(path):
	return
var json := JSON.new()
if json.parse(FileAccess.get_file_as_string(path)) != OK:
	_error_list().append("%s: JSON error on line %d: %s" % [path, json.get_error_line(), json.get_error_message()])
	_state()["blocked"] = true
	return
var doc: Dictionary = DictUtil.dict_of(json.data)
var raw: Variant = doc.get("entries", null)
if not (json.data is Dictionary) or not (raw is Array):
	_error_list().append("%s: expected {\"kind\": \"%s\", \"entries\": [...]}." % [path, FILE_KIND])
	_state()["blocked"] = true
	return
for item: Variant in DictUtil.array_of(raw):
	if not (item is Dictionary):
		_error_list().append("%s: an entry is not an object; it is skipped." % path)
		_state()["blocked"] = true
		continue
	var dict: Dictionary = item
	for key: Variant in dict:
		if not MapRules.TEMPLATE_FIELDS.has(str(key)):
			# to_storage keeps TEMPLATE_FIELDS only: saving now would silently drop the key
			_error_list().append("%s: map '%s' has unknown field '%s'; fix the file before saving." % [path, str(dict.get("id", "")), str(key)])
			_state()["blocked"] = true
	_list().append(normalize(dict))
```

### `static save_blocked() -> bool`
```logic
_ensure_loaded()
return DictUtil.to_bool(_state()["blocked"])
```

### `static is_dirty() -> bool`
```logic
return DictUtil.to_bool(_state()["dirty"])
```

### `static normalize(entry: Dictionary) -> Dictionary`
```logic
return MapRules.normalize_template(entry)
```

### `static _index_of(id: String) -> int`
```logic
_ensure_loaded()
var list: Array = _list()
for i: int in range(list.size()):
	if str(DictUtil.dict_of(list[i]).get("id", "")) == id:
		return i
return -1
```

### `static has(id: String) -> bool`
```logic
return not id.is_empty() and _index_of(id) >= 0
```

### `static get_entry(id: String) -> Dictionary`
```logic
var i: int = _index_of(id) if not id.is_empty() else -1
return DictUtil.dict_of(_list()[i]).duplicate(true) if i >= 0 else {}
```

### `static remove(id: String) -> String`
```logic
var at: int = _index_of(id)
if at < 0:
	return "No map template '%s'." % id
var users: Array[String] = used_by(id)
if not users.is_empty():
	return "Map '%s' is still used by: %s." % [id, ", ".join(users)]
_list().remove_at(at)
_state()["dirty"] = true
return ""
```

### `static validate_all() -> Dictionary`
```logic
_ensure_loaded()
var problems: Dictionary = {}
var seen: Dictionary = {}
for e: Dictionary in entries():
	var id: String = str(e.get("id", ""))
	var errs: Array[String] = validate_entry(e, id)
	if seen.has(id):
		errs.append("Map id '%s' is used twice in the file." % id)
	seen[id] = true
	if not errs.is_empty():
		problems["maps:%s" % id] = errs
if not _error_list().is_empty():
	problems["load"] = _error_list().duplicate()
return problems
```

### `static warnings_all() -> Dictionary`
```logic
var out: Dictionary = {}
for e: Dictionary in entries():
	var found: Array[String] = warnings_for(e)
	if not found.is_empty():
		out["maps:%s" % str(e.get("id", ""))] = found
return out
```

### `static to_storage(entry: Dictionary) -> Dictionary`
```logic
var e: Dictionary = normalize(entry)
var out: Dictionary = {}
for key: String in MapRules.TEMPLATE_FIELDS:
	out[key] = e[key]
return out
```

### `static save() -> Error`
```logic
_ensure_loaded()
if save_blocked():
	push_error("MapCatalog.save refused: %s loaded with errors: %s" % [file_path(), "; ".join(PackedStringArray(_error_list()))])
	return ERR_INVALID_DATA
var dir: String = file_path().get_base_dir()
if DirAccess.make_dir_recursive_absolute(dir) != OK and not DirAccess.dir_exists_absolute(dir):
	return ERR_CANT_CREATE
var stored: Array = []
for e: Dictionary in entries():
	stored.append(to_storage(e))
var file := FileAccess.open(file_path(), FileAccess.WRITE)
if file == null:
	return FileAccess.get_open_error()
file.store_string(JSON.stringify({"kind": FILE_KIND, "version": VERSION, "entries": stored}, "\t", false) + "\n")
file.close()
_state()["dirty"] = false
return OK
```

