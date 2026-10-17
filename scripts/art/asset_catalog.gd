class_name AssetCatalog
extends RefCounted

# ==============================================================================
# Tactical RPG - Asset Catalog (S14, spec S14-L1): the art / sound manifest
# res://data/assets/manifest.json: {"kind": "asset_manifest", "version": 1, "entries": [...]}
# One row per namespace/key/variant that has a FILE override (or an alias of another key); generated-only keys need no
# row. AssetRegistry resolves keys through these rows, then the generated template, then the namespace fallback.
# Same contract as QuestCatalog: upsert validates and stores (nothing changes on error), save writes the file, a file
# that loaded with errors refuses to save. Tests call use_path("user://...", asset_root) and never write res://.
# State lives on the Engine singleton (hot reloads can't reset it).
# ==============================================================================

const DEFAULT_PATH: String = "res://data/assets/manifest.json"
# Every row `path` starts with this prefix; the files live below the asset root (res://assets, or a test sandbox).
const PATH_PREFIX: String = "res://assets/"
const DEFAULT_ASSET_ROOT: String = "res://assets"
const FILE_KIND: String = "asset_manifest"
const VERSION: int = 1
const STATE_KEY: StringName = &"asset_catalog_state"
const FIELDS: Array[String] = ["id", "namespace", "key", "variant", "path", "alias_of", "size", "frames", "origin", "note"]
const ORIGIN_SYSTEM: String = "system"
const ORIGIN_USER: String = "user"
const ORIGINS: Array[String] = [ORIGIN_SYSTEM, ORIGIN_USER]
const NOTE_MAX: int = 120
const SIZE_MIN: int = 8
const SIZE_MAX: int = 2048
const KEY_PATTERN: String = "^[a-z][a-z0-9_]{0,39}$"

static func _state() -> Dictionary:
	if not Engine.has_meta(STATE_KEY):
		var errors: Array[String] = []
		Engine.set_meta(STATE_KEY, {"path": DEFAULT_PATH, "asset_root": DEFAULT_ASSET_ROOT, "entries": [], "errors": errors, "dirty": false, "blocked": false, "loaded": false, "system_edit": false})
	var state: Dictionary = Engine.get_meta(STATE_KEY)
	return state

static func _list() -> Array:
	var list: Array = _state()["entries"]
	return list

static func _error_list() -> Array[String]:
	var errors: Array[String] = _state()["errors"]
	return errors

static func _ensure_loaded() -> void:
	if not DictUtil.to_bool(_state()["loaded"]):
		reload_from_disk()

static func file_path() -> String:
	return str(_state()["path"])

# The folder that stands for res://assets (a test sandbox points it at user://...).
static func asset_root() -> String:
	return str(_state()["asset_root"])

# Points the catalog at another manifest (a test sandbox) and loads it. `p_asset_root` stands in for res://assets, so
# sandbox rows keep res://assets/... paths while their files live under user://.
static func use_path(p_path: String, p_asset_root: String = DEFAULT_ASSET_ROOT) -> void:
	_state()["path"] = p_path
	_state()["asset_root"] = p_asset_root
	reload_from_disk()

# The real file behind a row path (res://assets/x -> <asset_root>/x).
static func file_of(path: String) -> String:
	if not path.begins_with(PATH_PREFIX) or asset_root() == DEFAULT_ASSET_ROOT:
		return path
	return asset_root().path_join(path.substr(PATH_PREFIX.length()))

static func clear_cache() -> void:
	if Engine.has_meta(STATE_KEY):
		Engine.remove_meta(STATE_KEY)

static func reload_from_disk() -> void:
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

static func load_errors() -> Array[String]:
	_ensure_loaded()
	return _error_list().duplicate()

# True when save() is refused because the file loaded with problems (see load_errors()).
static func save_blocked() -> bool:
	_ensure_loaded()
	return DictUtil.to_bool(_state()["blocked"])

static func is_dirty() -> bool:
	return DictUtil.to_bool(_state()["dirty"])

# ------------------------------------------------------------------------------
# System-edit scope (same meaning as ContentCatalog.begin_system_edit): inside it new rows may be origin system and
# system rows may be removed. Seeds use it; nothing else does.
# ------------------------------------------------------------------------------

static func begin_system_edit() -> void:
	_state()["system_edit"] = true

static func end_system_edit() -> void:
	_state()["system_edit"] = false

static func in_system_edit() -> bool:
	return DictUtil.to_bool(_state().get("system_edit", false))

# ------------------------------------------------------------------------------
# Entries
# ------------------------------------------------------------------------------

static func id_for(ns: String, key: String, variant: String) -> String:
	return "%s/%s/%s" % [ns, key, variant]

# Defaults filled (variant = the namespace's first, id derived when empty), JSON numbers normalised; unknown keys are
# kept for validation.
static func normalize(entry: Dictionary) -> Dictionary:
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

static func entries() -> Array[Dictionary]:
	_ensure_loaded()
	var out: Array[Dictionary] = []
	for e: Variant in _list():
		out.append(DictUtil.dict_of(e).duplicate(true))
	return out

static func ids() -> Array[String]:
	var out: Array[String] = []
	for e: Dictionary in entries():
		out.append(str(e.get("id", "")))
	return out

static func _index_of(id: String) -> int:
	_ensure_loaded()
	var list: Array = _list()
	for i: int in range(list.size()):
		if str(DictUtil.dict_of(list[i]).get("id", "")) == id:
			return i
	return -1

static func has(id: String) -> bool:
	return _index_of(id) >= 0

# A copy of the row ({} when missing).
static func get_entry(id: String) -> Dictionary:
	var i: int = _index_of(id)
	return DictUtil.dict_of(_list()[i]).duplicate(true) if i >= 0 else {}

# The manifest keys of a namespace (every row, alias rows included), unique, in file order.
static func keys_of(ns: String) -> Array[String]:
	_ensure_loaded()
	var out: Array[String] = []
	for e: Variant in _list():
		var row: Dictionary = DictUtil.dict_of(e)
		var key: String = str(row.get("key", ""))
		if str(row.get("namespace", "")) == ns and not out.has(key):
			out.append(key)
	return out

static func _size_ok(raw: Variant) -> bool:
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

# Every problem with `entry` as a manifest row (current_id: the id it replaces). Messages are the spec's field table.
static func validate_entry(entry: Dictionary, current_id: String = "") -> Array[String]:
	_ensure_loaded()
	var e: Dictionary = normalize(entry)
	var errors: Array[String] = []
	for k: Variant in e:
		if not FIELDS.has(str(k)):
			errors.append("Unknown asset field '%s' (allowed: %s)." % [str(k), ", ".join(FIELDS)])
	var ns: String = str(e["namespace"])
	var key: String = str(e["key"])
	var variant: String = str(e["variant"])
	var ns_ok: bool = AssetRegistry.namespaces().has(ns)
	if not ns_ok:
		errors.append("Unknown namespace '%s'." % ns)
	var regex := RegEx.create_from_string(KEY_PATTERN)
	var key_ok: bool = regex.search(key) != null
	if not key_ok:
		errors.append("Key '%s' must be snake_case (letters, digits, _; max 40)." % key)
	var allowed: Array[String] = AssetRegistry.variants(ns)
	var variant_ok: bool = allowed.has(variant)
	if ns_ok and not variant_ok:
		errors.append("Variant '%s' is not valid for namespace %s (allowed: %s)." % [variant, ns, ", ".join(allowed)])
	var id: String = str(e["id"])
	var expected: String = id_for(ns, key, variant)
	if id != expected:
		errors.append("Id must be namespace/key/variant ('%s')." % expected)
	elif id != current_id and has(id):
		errors.append("Asset '%s' already exists." % id)
	var path: String = str(e["path"])
	var alias: String = str(e["alias_of"])
	if not path.is_empty() and not alias.is_empty():
		errors.append("Set a path or an alias, not both.")
	elif path.is_empty() and alias.is_empty():
		errors.append("Set a path or an alias.")
	if not path.is_empty():
		errors.append_array(_path_errors(ns, key, variant, path, ns_ok and key_ok and variant_ok))
	if not alias.is_empty() and ns_ok:
		if not AssetRegistry.has_key(ns, alias) or alias == key:
			if alias == key:
				errors.append("Alias target '%s' is an alias (one hop only)." % alias)
			else:
				errors.append("Alias target '%s' is not a known %s key." % [alias, ns])
		elif _is_alias(ns, alias, variant):
			errors.append("Alias target '%s' is an alias (one hop only)." % alias)
	if not _size_ok(e["size"]):
		errors.append("Size must be two whole numbers 8-2048.")
	if typeof(e["frames"]) != TYPE_INT or int(e["frames"]) != 1:
		errors.append("frames must be 1 until sprite strips are built.")
	if not ORIGINS.has(str(e["origin"])):
		errors.append("Origin '%s' must be system or user." % str(e["origin"]))
	if str(e["note"]).length() > NOTE_MAX:
		errors.append("Note must be at most %d characters." % NOTE_MAX)
	return errors

# True when the manifest row ns/key/variant is an alias row.
static func _is_alias(ns: String, key: String, variant: String) -> bool:
	var row: Dictionary = get_entry(id_for(ns, key, variant))
	return not row.is_empty() and not str(row.get("alias_of", "")).is_empty()

static func _path_errors(ns: String, key: String, variant: String, path: String, pattern_known: bool) -> Array[String]:
	var errors: Array[String] = []
	if not path.begins_with(PATH_PREFIX):
		errors.append("Path must start with res://assets/.")
		return errors
	if pattern_known and not AssetRegistry.pattern_paths(ns, key, variant).has(path):
		errors.append("Path must be %s (folder pattern)." % AssetRegistry.pattern_text(ns, key, variant))
		return errors
	if not AssetRegistry.file_exists(path):
		errors.append("File '%s' does not exist." % path)
		return errors
	if pattern_known and AssetRegistry.load_file(ns, path) == null:
		errors.append("File '%s' does not load as %s." % [path, AssetRegistry.resource_type(ns)])
	return errors

# Validates and stores (replacing `replace_id`, or the row with the same id, in place; otherwise appended). The id is
# always derived from namespace/key/variant. Outside the system-edit scope a new row is a user row.
# Returns the problems; nothing changes if any.
static func upsert(entry: Dictionary, replace_id: String = "") -> Array[String]:
	_ensure_loaded()
	var raw: Dictionary = entry.duplicate(true)
	raw.erase("id")
	var clean: Dictionary = normalize(raw)
	var new_id: String = str(clean["id"])
	var old_id: String = replace_id if not replace_id.is_empty() else new_id
	var old: Dictionary = get_entry(old_id)
	if old.is_empty() and not in_system_edit():
		clean["origin"] = ORIGIN_USER
	elif not old.is_empty() and not entry.has("origin"):
		clean["origin"] = str(old.get("origin", ORIGIN_USER))
	var errors: Array[String] = validate_entry(clean, old_id)
	if not errors.is_empty():
		return errors
	var at: int = _index_of(old_id)
	if at >= 0:
		_list()[at] = clean
	else:
		_list().append(clean)
	_state()["dirty"] = true
	return errors

# Removes a row. Refused (nothing changes) for system rows outside the system-edit scope.
static func remove(id: String) -> String:
	var at: int = _index_of(id)
	if at < 0:
		return "No asset '%s'." % id
	var row: Dictionary = get_entry(id)
	if str(row.get("origin", ORIGIN_USER)) == ORIGIN_SYSTEM and not in_system_edit():
		return "'%s' is a system asset (origin system): delete is refused outside the system-edit scope." % id
	_list().remove_at(at)
	_state()["dirty"] = true
	return ""

# {"assets:<id>": [errors]} plus "load" for file problems; {} when clean.
static func validate_all() -> Dictionary:
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

# Soft problems of one row (never block saving): a texture file whose size differs from the row `size` or the
# namespace default.
static func warnings_for(entry: Dictionary) -> Array[String]:
	var e: Dictionary = normalize(entry)
	var out: Array[String] = []
	var ns: String = str(e["namespace"])
	var path: String = str(e["path"])
	if path.is_empty() or not AssetRegistry.is_texture_namespace(ns) or not AssetRegistry.file_exists(path):
		return out
	var want: Vector2i = AssetRegistry.default_size(ns, str(e["variant"]))
	var size: Array = DictUtil.array_of(e["size"])
	if size.size() == 2:
		want = Vector2i(DictUtil.to_int(size[0]), DictUtil.to_int(size[1]))
	if want == Vector2i.ZERO:
		return out
	var tex: Resource = AssetRegistry.load_file(ns, path)
	if tex is Texture2D:
		var got: Vector2i = Vector2i((tex as Texture2D).get_size())
		if got != want:
			out.append("File '%s' is %dx%d; expected %dx%d (it is scaled)." % [path, got.x, got.y, want.x, want.y])
	return out

static func warnings_all() -> Dictionary:
	var out: Dictionary = {}
	for e: Dictionary in entries():
		var found: Array[String] = warnings_for(e)
		if not found.is_empty():
			out["assets:%s" % str(e.get("id", ""))] = found
	return out

# ------------------------------------------------------------------------------
# Saving
# ------------------------------------------------------------------------------

static func to_storage(entry: Dictionary) -> Dictionary:
	var e: Dictionary = normalize(entry)
	var out: Dictionary = {}
	for key: String in FIELDS:
		out[key] = e[key]
	return out

static func save() -> Error:
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
