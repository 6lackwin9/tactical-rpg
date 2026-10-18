---
class: AssetCatalog
extends: RefCounted
description: "Tactical RPG - Asset Catalog (S14, spec S14-L1): the art / sound manifest res://data/assets/manifest.json: {\"kind\": \"asset_manifest\", \"version\": 1, \"entries\": [...]} One row per namespace/key/variant that has a FILE override (or an alias of another key); generated-only keys need no row. AssetRegistry resolves keys through these rows, then the generated template, then the namespace fallback. Same contract as QuestCatalog: upsert validates and stores (nothing changes on error), save writes the file, a file that loaded with errors refuses to save. Tests call use_path(\"user://...\", asset_root) and never write res://. State lives on the Engine singleton (hot reloads can't reset it)."
---

# Properties
* `DEFAULT_PATH`: String = "res://data/assets/manifest.json" (const)
* `PATH_PREFIX`: String = "res://assets/" (const)
* `DEFAULT_ASSET_ROOT`: String = "res://assets" (const)
* `FILE_KIND`: String = "asset_manifest" (const)
* `VERSION`: int = 1 (const)
* `STATE_KEY`: StringName = &"asset_catalog_state" (const)
* `FIELDS`: Array[String] = ["id", "namespace", "key", "variant", "path", "alias_of", "size", "frames", "origin", "note"] (const)
* `ORIGIN_SYSTEM`: String = "system" (const)
* `ORIGIN_USER`: String = "user" (const)
* `ORIGINS`: Array[String] = [ORIGIN_SYSTEM, ORIGIN_USER] (const)
* `NOTE_MAX`: int = 120 (const)
* `SIZE_MIN`: int = 8 (const)
* `SIZE_MAX`: int = 2048 (const)
* `KEY_PATTERN`: String = "^[a-z][a-z0-9_]{0,39}$" (const)
* `errors`: Array[String] = _state()["errors"]
* `out`: Array[Dictionary] = []
* `out`: Array[String] = []
* `out`: Array[String] = []
* `row`: Dictionary = DictUtil.dict_of(e)
* `key`: String = str(row.get("key", ""))
* `e`: Dictionary = normalize(entry)
* `errors`: Array[String] = []
* `ns`: String = str(e["namespace"])
* `key`: String = str(e["key"])
* `variant`: String = str(e["variant"])
* `ns_ok`: bool = AssetRegistry.namespaces().has(ns)
* `regex`: Variant
* `key_ok`: bool = regex.search(key) != null
* `allowed`: Array[String] = AssetRegistry.variants(ns)
* `variant_ok`: bool = allowed.has(variant)
* `id`: String = str(e["id"])
* `expected`: String = id_for(ns, key, variant)
* `path`: String = str(e["path"])
* `alias`: String = str(e["alias_of"])
* `errors`: Array[String] = []
* `raw`: Dictionary = entry.duplicate(true)
* `clean`: Dictionary = normalize(raw)
* `new_id`: String = str(clean["id"])
* `old_id`: String = replace_id if not replace_id.is_empty() else new_id
* `old`: Dictionary = get_entry(old_id)
* `errors`: Array[String] = []
* `blocked`: String = removal_block(old_id)
* `at`: int = _index_of(old_id)
* `e`: Dictionary = normalize(entry)
* `out`: Array[String] = []
* `ns`: String = str(e["namespace"])
* `path`: String = str(e["path"])
* `want`: Vector2i = AssetRegistry.default_size(ns, str(e["variant"]))
* `size`: Array = DictUtil.array_of(e["size"])
* `tex`: Resource = AssetRegistry.load_file(ns, path)
* `got`: Vector2i = Vector2i((tex as Texture2D).get_size())

# Methods

### `static _state() -> Dictionary`
```logic
if not Engine.has_meta(STATE_KEY):
	var errors: Array[String] = []
	Engine.set_meta(STATE_KEY, {"path": DEFAULT_PATH, "asset_root": DEFAULT_ASSET_ROOT, "entries": [], "errors": errors, "dirty": false, "blocked": false, "loaded": false, "system_edit": false})
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

### `static asset_root() -> String`
```logic
return str(_state()["asset_root"])
```

### `static use_path(p_path: String, p_asset_root: String = DEFAULT_ASSET_ROOT) -> void`
```logic
_state()["path"] = p_path
_state()["asset_root"] = p_asset_root
reload_from_disk()
```

### `static file_of(path: String) -> String`
```logic
if not path.begins_with(PATH_PREFIX) or asset_root() == DEFAULT_ASSET_ROOT:
	return path
return asset_root().path_join(path.substr(PATH_PREFIX.length()))
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
		if not FIELDS.has(str(key)):
			# to_storage keeps FIELDS only: saving now would silently drop the key
			_error_list().append("%s: asset '%s' has unknown field '%s'; fix the file before saving." % [path, str(dict.get("id", "")), str(key)])
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

### `static begin_system_edit() -> void`
```logic
_state()["system_edit"] = true
```

### `static end_system_edit() -> void`
```logic
_state()["system_edit"] = false
```

### `static in_system_edit() -> bool`
```logic
return DictUtil.to_bool(_state().get("system_edit", false))
```

### `static id_for(ns: String, key: String, variant: String) -> String`
```logic
return "%s/%s/%s" % [ns, key, variant]
```

### `static normalize(entry: Dictionary) -> Dictionary`
```logic
var e: Dictionary = DictUtil.dict_of(JsonUtil.normalize(entry))
var ns: String = str(e.get("namespace", ""))
var key: String = str(e.get("key", ""))
var variant: String = str(e.get("variant", ""))
if variant.is_empty():
	var allowed: Array[String] = AssetRegistry.variants(ns)
	variant = allowed[0] if not allowed.is_empty() else ""
var id: String = str(e.get("id", ""))
if id.is_empty():
	id = id_for(ns, key, variant)
var out: Dictionary = {
	"id": id, "namespace": ns, "key": key, "variant": variant,
	"path": str(e.get("path", "")), "alias_of": str(e.get("alias_of", "")),
	"size": e.get("size", []), "frames": e.get("frames", 1),
	"origin": str(e.get("origin", ORIGIN_USER)), "note": str(e.get("note", "")),
}
for k: Variant in e:
	if not out.has(k) and not str(k).begins_with("_"):
		out[k] = e[k]
return out
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
return _index_of(id) >= 0
```

### `static get_entry(id: String) -> Dictionary`
```logic
var i: int = _index_of(id)
return DictUtil.dict_of(_list()[i]).duplicate(true) if i >= 0 else {}
```

### `static _size_ok(raw: Variant) -> bool`
```logic
if not (raw is Array):
	return false
var list: Array = raw
if list.is_empty():
	return true
if list.size() != 2:
	return false
for v: Variant in list:
	if typeof(v) != TYPE_INT or int(v) < SIZE_MIN or int(v) > SIZE_MAX:
		return false
return true
```

### `static _is_alias(ns: String, key: String, variant: String) -> bool`
```logic
var row: Dictionary = get_entry(id_for(ns, key, variant))
return not row.is_empty() and not str(row.get("alias_of", "")).is_empty()
```

### `static _is_system(row: Dictionary) -> bool`
```logic
return str(row.get("origin", ORIGIN_USER)) == ORIGIN_SYSTEM
```

### `static removal_block(id: String) -> String`
```logic
var row: Dictionary = get_entry(id)
if row.is_empty():
	return ""
var ns: String = str(row.get("namespace", ""))
var key: String = str(row.get("key", ""))
var variant: String = str(row.get("variant", ""))
if AssetRegistry.has_template(ns, key):
	return ""
if ns == "tile" and variant != "v0" and has(id_for(ns, key, "v0")):
	return ""
var refs: Array[String] = AssetScan.asset_refs(ns, key, id, variant)
if refs.is_empty():
	return ""
return "Asset '%s' is used by %s; remove the references first." % [id, ", ".join(refs)]
```

### `static remove(id: String) -> String`
```logic
var at: int = _index_of(id)
if at < 0:
	return "No asset '%s'." % id
if _is_system(get_entry(id)) and not in_system_edit():
	return "'%s' is a system asset (origin system): delete is refused outside the system-edit scope." % id
var blocked: String = removal_block(id)
if not blocked.is_empty():
	return blocked
_list().remove_at(at)
_state()["dirty"] = true
return ""
```

### `static promote(id: String) -> String`
```logic
return _set_origin(id, ORIGIN_SYSTEM)
```

### `static demote(id: String) -> String`
```logic
var row_ids: Array[String] = [id]
var refs: Array[String] = AssetScan.scan_files(row_ids)
if has(id) and _is_system(get_entry(id)) and not refs.is_empty():
	return "'%s' is named by tests or seeds, so it stays system: %s." % [id, "; ".join(refs)]
return _set_origin(id, ORIGIN_USER)
```

### `static _set_origin(id: String, origin: String) -> String`
```logic
var at: int = _index_of(id)
if at < 0:
	return "No asset '%s'." % id
var row: Dictionary = DictUtil.dict_of(_list()[at])
if str(row.get("origin", ORIGIN_USER)) == origin:
	return "'%s' is already a %s asset." % [id, origin]
row["origin"] = origin
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
		errs.append("Asset '%s' already exists." % id)
	seen[id] = true
	if not errs.is_empty():
		problems["assets:%s" % id] = errs
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
		out["assets:%s" % str(e.get("id", ""))] = found
return out
```

### `static to_storage(entry: Dictionary) -> Dictionary`
```logic
var e: Dictionary = normalize(entry)
var out: Dictionary = {}
for key: String in FIELDS:
	out[key] = e[key]
return out
```

### `static save() -> Error`
```logic
_ensure_loaded()
if save_blocked():
	push_error("AssetCatalog.save refused: %s loaded with errors: %s" % [file_path(), "; ".join(PackedStringArray(_error_list()))])
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

