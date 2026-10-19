class_name QuestCatalog
extends RefCounted

# ==============================================================================
# Tactical RPG - Quest Catalog (S8, spec M4-L): lore-free quest templates
# res://data/quests/quests.json: {"kind": "quest_templates", "version": 1, "entries": [...]}
# A template is {id, name, description, objectives: [{"kind": K}], rewards: {xp, items, passives, skills}, tags}.
# World quests bind a template to world targets (WorldSchema quests `template` + `steps`).
# Same contract as ContentCatalog: upsert validates and stores (nothing changes on error), save writes the file,
# a file that loaded with errors refuses to save. Tests call use_path("user://...") and never write res://.
# State lives on the Engine singleton (hot reloads can't reset it).
# ==============================================================================

const DEFAULT_PATH: String = "res://data/quests/quests.json"
const FILE_KIND: String = "quest_templates"
const VERSION: int = 1
const STATE_KEY: StringName = &"quest_catalog_state"
const FIELDS: Array[String] = ["id", "name", "description", "objectives", "rewards", "tags"]

static func _state() -> Dictionary:
	if not Engine.has_meta(STATE_KEY):
		var errors: Array[String] = []
		Engine.set_meta(STATE_KEY, {"path": DEFAULT_PATH, "entries": [], "errors": errors, "dirty": false, "blocked": false, "loaded": false})
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

# Points the catalog at another file (a test sandbox) and loads it.
static func use_path(p_path: String) -> void:
	_state()["path"] = p_path
	reload_from_disk()

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
				_error_list().append("%s: template '%s' has unknown field '%s'; fix the file before saving." % [path, str(dict.get("id", "")), str(key)])
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
# Entries
# ------------------------------------------------------------------------------

# Defaults filled, JSON numbers normalised (whole floats become ints); unknown keys are kept for validation.
static func normalize(entry: Dictionary) -> Dictionary:
	var e: Dictionary = DictUtil.dict_of(JsonUtil.normalize(entry))
	var out: Dictionary = {
		"id": str(e.get("id", "")), "name": e.get("name", ""), "description": e.get("description", ""),
		"objectives": e.get("objectives", []), "rewards": e.get("rewards", {}), "tags": e.get("tags", []),
	}
	for key: Variant in e:
		if not out.has(key) and not str(key).begins_with("_"):
			out[key] = e[key]
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

# A copy of the template ({} when missing).
static func get_entry(id: String) -> Dictionary:
	var i: int = _index_of(id)
	return DictUtil.dict_of(_list()[i]).duplicate(true) if i >= 0 else {}

# Every problem with `entry` as a template (current_id: the id it replaces).
static func validate_entry(entry: Dictionary, current_id: String = "") -> Array[String]:
	_ensure_loaded()
	var e: Dictionary = normalize(entry)
	var errors: Array[String] = []
	var id: String = str(e["id"])
	if id.is_empty():
		errors.append("ID is required.")
	elif id != current_id and has(id):
		errors.append("ID '%s' is already used by another quest template." % id)
	for key: Variant in e:
		if not FIELDS.has(str(key)):
			errors.append("Unknown template field '%s' (allowed: %s)." % [str(key), ", ".join(FIELDS)])
	for err: String in QuestRules.template_errors(e):
		if not (id.is_empty() and err.begins_with("ID ")):
			errors.append(err)
	return errors

# Validates and stores (replacing `replace_id`, or the entry with the same id, in place; otherwise appended).
# Returns the problems; nothing changes if any.
static func upsert(entry: Dictionary, replace_id: String = "") -> Array[String]:
	_ensure_loaded()
	var clean: Dictionary = normalize(entry)
	var new_id: String = str(clean["id"])
	var old_id: String = replace_id if not replace_id.is_empty() else new_id
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

# Removes a template. World quests naming it then fail validation (WorldCatalog.validate_all reports them).
static func remove(id: String) -> String:
	var at: int = _index_of(id)
	if at < 0:
		return "No quest template '%s'." % id
	_list().remove_at(at)
	_state()["dirty"] = true
	return ""

# {"quests:<id>": [errors]} plus "load" for file problems; {} when clean.
static func validate_all() -> Dictionary:
	_ensure_loaded()
	var problems: Dictionary = {}
	var seen: Dictionary = {}
	for e: Dictionary in entries():
		var id: String = str(e.get("id", ""))
		var errs: Array[String] = validate_entry(e, id)
		if seen.has(id):
			errs.append("ID '%s' is used twice in the file." % id)
		seen[id] = true
		if not errs.is_empty():
			problems["quests:%s" % id] = errs
	if not _error_list().is_empty():
		problems["load"] = _error_list().duplicate()
	return problems

# Soft problems of one template (never block saving): the QuestRules soft reward caps.
static func warnings_for(entry: Dictionary) -> Array[String]:
	return QuestRules.rewards_warnings(normalize(entry).get("rewards", {}))

static func warnings_all() -> Dictionary:
	var out: Dictionary = {}
	for e: Dictionary in entries():
		var found: Array[String] = warnings_for(e)
		if not found.is_empty():
			out["quests:%s" % str(e.get("id", ""))] = found
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
		push_error("QuestCatalog.save refused: %s loaded with errors: %s" % [file_path(), "; ".join(PackedStringArray(_error_list()))])
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
