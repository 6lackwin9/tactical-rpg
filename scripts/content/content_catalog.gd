class_name ContentCatalog
extends RefCounted

# ==============================================================================
# Tactical RPG - Content Catalog (the single store of authored game content)
# Skills, items, classes, units (party members & enemies) and encounters live as
# JSON in res://data/catalog/<kind>.json:  {"kind": ..., "version": 1, "entries": [...]}
# Entries are plain Dictionaries shaped by ContentSchema; ContentFactory turns
# them into engine objects (SkillDef, Item, ClassArchetype, Combatant) and
# CatalogEncounter loads an encounter entry into the real combat engine.
#
# Everything that reads content goes through here: SkillCatalog, ItemCatalog,
# ClassArchetype presets, CatalogEncounter, the tests and the Content Forge.
# Tests that write use use_root("user://...") so the shipped catalog is untouched.
# ==============================================================================

const DEFAULT_ROOT: String = "res://data/catalog"
const VERSION: int = 1
# File format version per kind (default VERSION). skills.json is version 2: ranked entries on disk, flat rows in memory
# (SkillRanks). The loader decides by the file's own version, so a version 1 skills file still loads.
const VERSIONS: Dictionary = {"skills": 2}

static func version_of(kind: String) -> int:
	return DictUtil.to_int(VERSIONS.get(kind, VERSION), VERSION)


# ------------------------------------------------------------------------------
# Loading & saving
# ------------------------------------------------------------------------------

# ------------------------------------------------------------------------------
# State
# Kept on the Engine singleton rather than in static vars, so a hot reload of
# this script (SceneRunner) can't silently point the catalog back at res:// or
# drop unsaved edits. (Never name a static func `reload`: ContentCatalog.reload()
# would call the class's Script.reload() and recompile this file instead.)
# ------------------------------------------------------------------------------

const STATE_KEY: StringName = &"content_catalog_state"

static func _state() -> Dictionary:
	if not Engine.has_meta(STATE_KEY):
		var errors: Array[String] = []
		Engine.set_meta(STATE_KEY, {"root": DEFAULT_ROOT, "data": {}, "dirty": {}, "errors": errors, "scope": false, "allow_remove": false, "save_errors": []})
	var state: Dictionary = Engine.get_meta(STATE_KEY)
	return state

# The folder the catalog reads from and saves to.
static func get_root() -> String:
	return str(_state()["root"])

static func _data_map() -> Dictionary:   # kind -> Array of entry Dictionaries (file order)
	var data: Dictionary = _state()["data"]
	return data

static func _dirty_map() -> Dictionary:  # kind -> true when changed since the last save
	var dirty: Dictionary = _state()["dirty"]
	return dirty

static func _error_list() -> Array[String]:
	var errors: Array[String] = _state()["errors"]
	return errors

static func path_for(kind: String, p_root: String = "") -> String:
	return "%s/%s.json" % [p_root if not p_root.is_empty() else get_root(), kind]

# Points the catalog at another folder (e.g. a test sandbox) and loads it.
static func use_root(p_root: String) -> void:
	_state()["root"] = p_root
	reload_from_disk()

# Copies the loaded catalog into `p_root` (replacing its catalog files) and
# switches to it: tests and experiments edit the copy, never the shipped data.
static func fork_to(p_root: String) -> Error:
	_ensure_loaded()
	_state()["root"] = p_root
	for kind: String in ContentSchema.KINDS:
		_dirty_map()[kind] = true
	var err: Error = save_all()
	if err == OK:
		reload_from_disk()
	return err

static func reload_from_disk() -> void:
	_data_map().clear()
	_dirty_map().clear()
	_error_list().clear()
	_state()["scope"] = false
	_state()["allow_remove"] = false
	_state()["save_errors"] = []
	_blocked_map().clear()
	for kind: String in ContentSchema.KINDS:
		_data_map()[kind] = _read_kind(kind)
	SkillCatalog.clear_cache()

static func _ensure_loaded() -> void:
	if _data_map().is_empty():
		reload_from_disk()

static func load_errors() -> Array[String]:
	_ensure_loaded()
	return _error_list().duplicate()

# Kinds whose file loaded with problems that dropped or changed data (kind -> true). save() refuses them until the file
# is fixed and reloaded, so a save can never silently delete what the loader could not read.
static func _blocked_map() -> Dictionary:
	if not _state().has("blocked"):
		_state()["blocked"] = {}
	var blocked: Dictionary = _state()["blocked"]
	return blocked

static func _block_save(kind: String) -> void:
	_blocked_map()[kind] = true

# True when save(kind) is refused because the loaded file had problems (see load_errors()).
static func save_blocked(kind: String) -> bool:
	_ensure_loaded()
	return _blocked_map().has(kind)

static func _read_kind(kind: String) -> Array:
	var found: Array = []
	var path: String = path_for(kind)
	if not FileAccess.file_exists(path):
		return found
	var text: String = FileAccess.get_file_as_string(path)
	var json := JSON.new()
	if json.parse(text) != OK:
		_error_list().append("%s: JSON error on line %d: %s" % [path, json.get_error_line(), json.get_error_message()])
		_block_save(kind)
		return found
	var parsed: Variant = json.data
	var raw: Variant = parsed
	var file_version: int = 1
	if parsed is Dictionary:
		var doc: Dictionary = parsed
		raw = doc.get("entries", [])
		file_version = DictUtil.to_int(doc.get("version", 1), 1)
	if not (raw is Array):
		_error_list().append("%s: expected an \"entries\" list." % path)
		_block_save(kind)
		return found
	var list: Array = raw
	if kind == "skills" and file_version >= 2:
		return _read_ranked_skills(list, path)
	for item: Variant in list:
		if item is Dictionary:
			var dict: Dictionary = item
			found.append(ContentSchema.normalize(kind, dict))
	return found

# Version 2 skills file: ranked entries expand to flat rows, rank-major (all rank 1 rows in entry order, then rank 2, ...).
static func _read_ranked_skills(list: Array, path: String) -> Array:
	var by_rank: Array = [[], [], [], [], []]
	for item: Variant in list:
		if not (item is Dictionary):
			_error_list().append("%s: an entry is not an object; it is skipped." % path)
			_block_save("skills")
			continue
		var dict: Dictionary = item
		var id: String = str(dict.get("id", ""))
		var bad: bool = false
		for banned: String in ["family", "tier", "template"]:
			if dict.has(banned):
				_error_list().append("%s: skill '%s' stores '%s'; version 2 entries derive it." % [path, id, banned])
				bad = true
		for mod: Dictionary in SkillRanks.modifiers_of(dict):
			for banned: String in ["name", "family", "tier", "id"]:
				if mod.has(banned):
					_error_list().append("%s: skill '%s' has a rank modifier with '%s'; rank names and ids are derived." % [path, id, banned])
					bad = true
		if bad:
			_block_save("skills")   # the entry is not loaded: saving now would delete it from the file
			continue
		# Modifier keys expand() would drop or change (unknown keys, fields the kind does not use, values out of range):
		# the entry loads, but saving is refused so the file is never rewritten without them.
		var base: Dictionary = SkillRanks.effective(dict, 1)
		for p: Dictionary in _modifier_problems(str(base["kind"]), base, SkillRanks.modifiers_of(dict)):
			_error_list().append("%s: skill '%s': %s" % [path, id, str(p["message"])])
			_block_save("skills")
		var rows: Array[Dictionary] = SkillRanks.expand(dict)
		for i: int in range(rows.size()):
			var bucket: Array = by_rank[mini(i, by_rank.size() - 1)]
			bucket.append(rows[i])
	var found: Array = []
	for bucket: Variant in by_rank:
		var rows_here: Array = bucket
		found.append_array(rows_here)
	return found

static func save(kind: String) -> Error:
	_ensure_loaded()
	if _blocked_map().has(kind):
		var why: Array[String] = ["%s.json loaded with errors (entries were skipped or changed); fix the file and reload before saving: %s" % [kind, "; ".join(PackedStringArray(_error_list()))]]
		_state()["save_errors"] = why
		push_error("ContentCatalog.save(%s) refused: %s" % [kind, why[0]])
		return ERR_INVALID_DATA
	var dir_err: Error = DirAccess.make_dir_recursive_absolute(get_root())
	if dir_err != OK and not DirAccess.dir_exists_absolute(get_root()):
		return dir_err
	var stored: Array = []
	if kind == "skills":
		var folded: Dictionary = SkillRanks.collapse(entries("skills"))
		var fold_errors: Array = folded["errors"]
		_state()["save_errors"] = fold_errors.duplicate()
		if not fold_errors.is_empty():
			push_error("ContentCatalog.save(skills) refused: " + "; ".join(PackedStringArray(fold_errors)))
			return ERR_INVALID_DATA
		var ranked_list: Array = folded["entries"]
		stored.append_array(ranked_list)
	else:
		for entry: Dictionary in entries(kind):
			stored.append(ContentSchema.to_storage(kind, entry))
	var doc: Dictionary = {"kind": kind, "version": version_of(kind), "entries": stored}
	var file := FileAccess.open(path_for(kind), FileAccess.WRITE)
	if file == null:
		return FileAccess.get_open_error()
	file.store_string(JSON.stringify(doc, "\t", false) + "\n")
	file.close()
	_dirty_map().erase(kind)
	return OK

static func save_all() -> Error:
	for kind: String in ContentSchema.KINDS:
		var err: Error = save(kind)
		if err != OK:
			return err
	return OK

static func is_dirty(kind: String = "") -> bool:
	return _dirty_map().has(kind) if not kind.is_empty() else not _dirty_map().is_empty()

# Drops the loaded data and the root override (call on shutdown with the other caches).
static func clear_cache() -> void:
	if Engine.has_meta(STATE_KEY):
		Engine.remove_meta(STATE_KEY)

# ------------------------------------------------------------------------------
# Reading
# ------------------------------------------------------------------------------

# The live entries of `kind` (do not modify; use upsert()).
static func entries(kind: String) -> Array[Dictionary]:
	_ensure_loaded()
	var out: Array[Dictionary] = []
	var raw: Variant = _data_map().get(kind, [])
	if raw is Array:
		var list: Array = raw
		out.assign(list)
	return out

static func ids(kind: String) -> Array[String]:
	var out: Array[String] = []
	for e: Dictionary in entries(kind):
		out.append(str(e.get("id", "")))
	return out

static func has(kind: String, id: String) -> bool:
	return _index_of(kind, id) >= 0

# A copy of the entry (empty when missing).
static func get_entry(kind: String, id: String) -> Dictionary:
	var i: int = _index_of(kind, id)
	return entries(kind)[i].duplicate(true) if i >= 0 else {}

static func _index_of(kind: String, id: String) -> int:
	var list: Array[Dictionary] = entries(kind)
	for i: int in range(list.size()):
		if str(list[i].get("id", "")) == id:
			return i
	return -1

# ------------------------------------------------------------------------------
# Editing
# ------------------------------------------------------------------------------

static func _problem(field: String, rank: int, message: String) -> Dictionary:
	return {"field": field, "rank": rank, "message": message}

# ------------------------------------------------------------------------------
# Origin (FORGE1 section 12): system assets have locked key fields and are never deleted from the Forge;
# a new asset is a user asset. Seeds and tests that must edit key fields of system assets use begin/end_system_edit.
# ------------------------------------------------------------------------------

# Opens the system-edit scope (seeds and tests only; the Forge never calls it). `allow_remove` also lets remove() delete system
# assets (SAFE DELETE). The scope resets on reload_from_disk().
static func begin_system_edit(allow_remove: bool = false) -> void:
	_ensure_loaded()
	_state()["scope"] = true
	_state()["allow_remove"] = allow_remove

static func end_system_edit() -> void:
	_state()["scope"] = false
	_state()["allow_remove"] = false

static func in_system_edit() -> bool:
	return DictUtil.to_bool(_state().get("scope", false))

static func _can_remove_system() -> bool:
	return in_system_edit() and DictUtil.to_bool(_state().get("allow_remove", false))

static func is_system_origin(entry: Dictionary) -> bool:
	return str(entry.get("origin", ContentSchema.ORIGIN_SYSTEM)) == ContentSchema.ORIGIN_SYSTEM

# The origin a stored entry gets: new ids are user assets outside the scope; existing assets keep theirs.
static func _with_origin(entry: Dictionary, old: Dictionary) -> Dictionary:
	var raw: Dictionary = entry.duplicate()
	if old.is_empty():
		if not in_system_edit():
			raw["origin"] = ContentSchema.ORIGIN_USER
	elif not raw.has("origin"):
		raw["origin"] = old.get("origin", ContentSchema.ORIGIN_SYSTEM)
	return raw

# Key fields (schema `key: true`) of a SYSTEM asset cannot change outside the system-edit scope.
static func key_field_errors(kind: String, old: Dictionary, new_entry: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	if old.is_empty() or not is_system_origin(old) or in_system_edit():
		return errors
	for f: Dictionary in ContentSchema.fields(kind):
		if not DictUtil.to_bool(f.get("key_field", false)):
			continue
		var key: String = str(f["key"])
		if old.get(key, null) != new_entry.get(key, null):
			errors.append("'%s' is a system asset: key field '%s' is locked. Create your own copy to change it." % [str(old.get("id", "")), str(f.get("label", key))])
	return errors

# Validates and stores `entry` (replacing `replace_id`, or the entry with the same
# id, in place; otherwise appended). Returns the problems; nothing changes if any.
# Renaming an id rewrites references to it in other entries. A new id becomes a user asset; key fields of an
# existing system asset are locked (see key_field_errors).
static func upsert(kind: String, entry: Dictionary, replace_id: String = "") -> Array[String]:
	_ensure_loaded()
	var new_id: String = str(entry.get("id", ""))
	var old_id: String = replace_id if not replace_id.is_empty() else new_id
	var old: Dictionary = get_entry(kind, old_id)
	var clean: Dictionary = ContentSchema.normalize(kind, _with_origin(entry, old))
	new_id = str(clean.get("id", ""))
	var errors: Array[String] = validate_entry(kind, clean, old_id)
	errors.append_array(key_field_errors(kind, old, clean))
	if not errors.is_empty():
		return errors
	var list: Array = _data_map()[kind]
	var at: int = _index_of(kind, old_id)
	if at >= 0:
		list[at] = clean
	else:
		list.append(clean)
	if old_id != new_id and at >= 0:
		_rename_refs(kind, old_id, new_id)
	_touched(kind)
	return errors

# Atomic multi-entry upsert (all or none): a set of entries that are only valid TOGETHER, e.g. every rank row of a skill
# family whose `grade` must change on all rows at once. Each entry replaces the entry with its id (or is appended);
# every entry is then validated against the catalog as it would look with ALL of them applied. Any problem restores
# the previous state and returns the problems (each prefixed with the entry id); nothing changes.
static func upsert_many(kind: String, new_entries: Array) -> Array[String]:
	_ensure_loaded()
	var list: Array = _data_map()[kind]
	var backup: Array = list.duplicate()
	var applied: Array[Dictionary] = []
	var errors: Array[String] = []
	for raw: Variant in new_entries:
		var src: Dictionary = DictUtil.dict_of(raw)
		var id: String = str(src.get("id", ""))
		var old: Dictionary = get_entry(kind, id)
		var clean: Dictionary = ContentSchema.normalize(kind, _with_origin(src, old))
		for problem: String in key_field_errors(kind, old, clean):
			errors.append("%s: %s" % [id, problem])
		applied.append(clean)
	if not errors.is_empty():
		return errors
	for clean: Dictionary in applied:
		var at: int = _index_of(kind, str(clean.get("id", "")))
		if at >= 0:
			list[at] = clean
		else:
			list.append(clean)
	for clean: Dictionary in applied:
		for problem: String in validate_entry(kind, clean, str(clean.get("id", ""))):
			errors.append("%s: %s" % [str(clean.get("id", "")), problem])
	if not errors.is_empty():
		list.clear()
		list.append_array(backup)
		return errors
	_touched(kind)
	return errors

# Removes an entry unless something still references it. Returns "" or why not.
# System assets (origin system) are never removed here (SAFE DELETE, tools/safe_delete.gd, is the process).
static func remove(kind: String, id: String) -> String:
	var at: int = _index_of(kind, id)
	if at < 0:
		return "No %s entry '%s'." % [kind, id]
	var entry: Dictionary = entries(kind)[at]
	if DictUtil.to_bool(entry.get("system", false)):
		return "'%s' is a system entry the engine relies on." % id
	if kind == "skills" and SkillKinds.REQUIRED_IDS.has(id):
		return "'%s' is a required engine skill (system)." % id
	if is_system_origin(entry) and not _can_remove_system():
		return "'%s' is a system asset (origin system): delete is refused. Use SAFE DELETE (tools/safe_delete.gd) or make your own copy." % id
	var users: Array[String] = references_to(kind, id)
	if not users.is_empty():
		return "'%s' is still used by: %s." % [id, ", ".join(users)]
	var list: Array = _data_map()[kind]
	list.remove_at(at)
	_touched(kind)
	return ""

# Changes the origin of an asset (a whole family for skills) and saves. Returns "" or why not.
static func _set_origin(kind: String, id: String, origin: String) -> String:
	var row_ids: Array[String] = [id]
	if kind == "skills":
		row_ids.clear()
		for row: Dictionary in family_rows(id):
			row_ids.append(str(row["id"]))
	if row_ids.is_empty():
		return "No %s entry '%s'." % [kind, id]
	var list: Array = _data_map()[kind]
	for row_id: String in row_ids:
		var at: int = _index_of(kind, row_id)
		var row: Dictionary = list[at]
		row["origin"] = origin
	_touched(kind)
	var err: Error = save(kind)
	return "" if err == OK else "Could not save %s (error %d)." % [kind, err]

# Promote a user asset to system (explicit action; key fields lock, delete locks). Returns "" or why not.
static func promote(kind: String, id: String) -> String:
	_ensure_loaded()
	var entry: Dictionary = get_entry(kind, id)
	if entry.is_empty():
		return "No %s entry '%s'." % [kind, id]
	if is_system_origin(entry):
		return "'%s' is already a system asset." % id
	return _set_origin(kind, id, ContentSchema.ORIGIN_SYSTEM)

# Demote a system asset back to user (delete becomes possible, key fields unlock). Refused for basics, required and
# engine skills, and any asset a test or seed still names (SAFE DELETE scan). Returns "" or why not.
static func demote(kind: String, id: String) -> String:
	_ensure_loaded()
	var entry: Dictionary = get_entry(kind, id)
	if entry.is_empty():
		return "No %s entry '%s'." % [kind, id]
	if not is_system_origin(entry):
		return "'%s' is already a user asset." % id
	var row_ids: Array[String] = [id]
	if kind == "skills":
		row_ids.clear()
		for row: Dictionary in family_rows(id):
			row_ids.append(str(row["id"]))
			if DictUtil.to_bool(row.get("basic", false)):
				return "'%s' is a basic skill and stays system." % id
			if DictUtil.to_bool(row.get("system", false)):
				return "'%s' is an engine skill and stays system." % id
		if SkillKinds.REQUIRED_IDS.has(id):
			return "'%s' is a required engine skill and stays system." % id
	var refs: Array[String] = AssetScan.scan_files(row_ids)
	if not refs.is_empty():
		return "'%s' is named by tests or seeds, so it stays system: %s." % [id, "; ".join(refs)]
	return _set_origin(kind, id, ContentSchema.ORIGIN_USER)

# ------------------------------------------------------------------------------
# Ranked skills (FORGE1 section 2): the Forge edits ranked ENTRIES; the store keeps flat rows.
# ------------------------------------------------------------------------------

# The stored flat rows of one family (rank 1 first when the file order is rank-major).
static func family_rows(head_id: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for e: Dictionary in entries("skills"):
		if SkillRanks.head_of(e) == head_id:
			out.append(e)
	return out

# True when any stored rank row of the family fails validation (the cheap check behind the list marks and the filter).
static func family_has_problems(head_id: String) -> bool:
	for row: Dictionary in family_rows(head_id):
		if not validate_entry("skills", row, str(row["id"])).is_empty():
			return true
	return false

# Every skill as a ranked entry, in the order the file stores them.
static func skill_entries() -> Array[Dictionary]:
	var folded: Dictionary = SkillRanks.collapse(entries("skills"))
	var out: Array[Dictionary] = []
	var list: Array = folded["entries"]
	for item: Variant in list:
		out.append(DictUtil.dict_of(item))
	return out

# One family as a ranked entry (empty when unknown or when its rows cannot be folded).
static func skill_entry(head_id: String) -> Dictionary:
	var rows: Array[Dictionary] = family_rows(head_id)
	if rows.is_empty():
		return {}
	var normal: Array[Dictionary] = []
	for row: Dictionary in rows:
		normal.append(ContentSchema.normalize("skills", row))
	var folded: Dictionary = SkillRanks.fold(head_id, normal)
	return DictUtil.dict_of(folded["entry"])

# Problems of a ranked skill entry, as {"field", "rank" (0 = the whole family), "message"}. Checks the entry and every
# rank row against the catalog as it would look with this family replacing `current_head`'s (the head being edited).
static func validate_skill_entry(entry: Dictionary, current_head: String = "") -> Array[Dictionary]:
	_ensure_loaded()
	var out: Array[Dictionary] = []
	var base: Dictionary = SkillRanks.effective(entry, 1)
	var head: String = str(base["id"])
	var kind: String = str(base["kind"])
	var old_head: String = current_head if not current_head.is_empty() else head
	var mods: Array[Dictionary] = SkillRanks.modifiers_of(entry)
	var count: int = mods.size() + 1
	var single: bool = DictUtil.to_bool(base["basic"]) or DictUtil.to_bool(base["system"])
	if single and count > 1:
		out.append(_problem("ranks", 0, "Basic and engine skills have exactly one rank."))
	if count > StatRules.MAX_CLASS_TIER:
		out.append(_problem("ranks", 0, "A skill has at most %d ranks." % StatRules.MAX_CLASS_TIER))
	if ContentSchema.is_valid_id(head):
		for n: int in range(1, count + 1):
			var rid: String = SkillRanks.rank_id(head, n)
			var owner: Dictionary = get_entry("skills", rid)
			if not owner.is_empty() and SkillRanks.head_of(owner) != old_head:
				out.append(_problem("id", n, "Rank id '%s' is already used by another skill." % rid))
	var longest: String = SkillRanks.rank_name(str(base["name"]), count)
	if longest.length() > 32:
		out.append(_problem("name", count, "The derived rank name '%s' is %d characters (max 32)." % [longest, longest.length()]))
	out.append_array(_modifier_problems(kind, base, mods))
	out.append_array(SkillText.validate(str(base["description"]), entry))
	var old_entry: Dictionary = skill_entry(old_head)
	if not old_entry.is_empty():
		var old_base: Dictionary = SkillRanks.effective(old_entry, 1)
		for message: String in key_field_errors("skills", old_base, base):
			out.append(_problem("key", 0, message))
		if is_system_origin(old_base) and not in_system_edit() and SkillRanks.rank_count(old_entry) != count:
			out.append(_problem("ranks", 0, "'%s' is a system asset: its number of ranks is locked. Create your own copy to change it." % old_head))
	# Per-rank rows against the catalog as it would look after the replacement.
	var rows: Array[Dictionary] = SkillRanks.expand(entry)
	var list: Array = _data_map()["skills"]
	var backup: Array = list.duplicate()
	var keep: Array = []
	for existing: Variant in list:
		if SkillRanks.head_of(DictUtil.dict_of(existing)) != old_head:
			keep.append(existing)
	list.clear()
	list.append_array(keep)
	list.append_array(rows)
	var seen: Dictionary = {}
	for row: Dictionary in rows:
		for message: String in validate_entry("skills", row, str(row["id"])):
			var rank: int = DictUtil.to_int(row["tier"], 1)
			var tag: String = message
			if not seen.has(tag):
				seen[tag] = true
				out.append(_problem(_field_of_message(message), rank, message))
	list.clear()
	list.append_array(backup)
	return _dedupe(out)

static func _dedupe(problems: Array[Dictionary]) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var seen: Dictionary = {}
	for p: Dictionary in problems:
		var tag: String = "%s|%d|%s" % [str(p["field"]), DictUtil.to_int(p["rank"]), str(p["message"])]
		if not seen.has(tag):
			seen[tag] = true
			out.append(p)
	return out

# Best-effort field name of a row-level message (the first schema label it starts with).
static func _field_of_message(message: String) -> String:
	for f: Dictionary in ContentSchema.fields("skills"):
		if message.begins_with(str(f["label"])):
			return str(f["key"])
	return ""

# Rank modifier rules: only rank-modifiable fields the kind shows, inside their range.
static func _modifier_problems(kind: String, base: Dictionary, mods: Array[Dictionary]) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var allowed: Array[String] = SkillRanks.rank_fields(kind)
	for i: int in range(mods.size()):
		var rank: int = i + 2
		var mod: Dictionary = mods[i]
		for raw_key: Variant in mod:
			var key: String = str(raw_key)
			if key == "description":
				continue
			if key == "name":
				out.append(_problem("name", rank, "Rank %d: rank names are derived (%s); a modifier cannot set the name." % [rank, SkillRanks.rank_name(str(base["name"]), rank)]))
			elif not SkillRanks.RANK_KEYS.has(key):
				out.append(_problem(key, rank, "Rank %d: '%s' is base-only or unknown; a rank can only change %s." % [rank, key, ", ".join(SkillRanks.RANK_KEYS)]))
			elif not allowed.has(key):
				out.append(_problem(key, rank, "Rank %d: '%s' is not used by %s skills." % [rank, key, kind]))
			else:
				var f: Dictionary = ContentSchema.field("skills", key)
				var value: Variant = mod[raw_key]
				if (value is int or value is float) and (f.has("min") or f.has("max")):
					var x: float = DictUtil.to_float(value)
					if (f.has("min") and x < DictUtil.to_float(f["min"])) or (f.has("max") and x > DictUtil.to_float(f["max"])):
						out.append(_problem(key, rank, "Rank %d: %s %s is outside %s-%s." % [rank, str(f["label"]), str(value), str(f.get("min", "")), str(f.get("max", ""))]))
	return out

# Validates and stores a ranked skill entry (replacing the family `replace_id`, default the same head). All rank rows are
# validated together and replaced atomically; any problem changes nothing. A head rename renames the rank ids and rewrites
# references (class paths, passives, unit skills and loadouts, other skills' `requires`).
static func upsert_skill(entry: Dictionary, replace_id: String = "") -> Array[String]:
	_ensure_loaded()
	var raw: Dictionary = entry.duplicate(true)
	var head: String = str(raw.get("id", ""))
	var old_head: String = replace_id if not replace_id.is_empty() else head
	var old_rows: Array[Dictionary] = family_rows(old_head)
	if old_rows.is_empty():
		if not in_system_edit():
			if str(raw.get("origin", ContentSchema.ORIGIN_USER)) == ContentSchema.ORIGIN_SYSTEM:
				return ["New skills are user assets: origin system is only allowed inside the system-edit scope (seeds)."]
			raw["origin"] = ContentSchema.ORIGIN_USER
	elif not raw.has("origin"):
		raw["origin"] = old_rows[0].get("origin", ContentSchema.ORIGIN_SYSTEM)
	var problems: Array[Dictionary] = validate_skill_entry(raw, old_head)
	if not problems.is_empty():
		var messages: Array[String] = []
		for p: Dictionary in problems:
			messages.append(str(p["message"]))
		return messages
	var rows: Array[Dictionary] = SkillRanks.expand(raw)
	var list: Array = _data_map()["skills"]
	var slots: Array[int] = []
	for i: int in range(list.size()):
		if SkillRanks.head_of(DictUtil.dict_of(list[i])) == old_head:
			slots.append(i)
	var old_count: int = slots.size()
	var remove_slots: Array[int] = []
	for i: int in range(slots.size()):
		if i < rows.size():
			list[slots[i]] = rows[i]
		else:
			remove_slots.append(slots[i])
	remove_slots.reverse()
	for at: int in remove_slots:
		list.remove_at(at)
	for i: int in range(slots.size(), rows.size()):
		list.append(rows[i])
	if old_head != head and old_count > 0:
		for n: int in range(1, old_count + 1):
			_rename_refs("skills", SkillRanks.rank_id(old_head, n), SkillRanks.rank_id(head, n))
	_touched("skills")
	var none: Array[String] = []
	return none

# Removes a whole skill family (every rank). Refused for system assets, basics, the required engine skills and any
# referenced rank id. Returns "" or why not.
static func remove_skill(head_id: String) -> String:
	_ensure_loaded()
	var rows: Array[Dictionary] = family_rows(head_id)
	if rows.is_empty():
		return "No skills entry '%s'." % head_id
	for row: Dictionary in rows:
		if DictUtil.to_bool(row.get("system", false)):
			return "'%s' is a system entry the engine relies on." % head_id
		if DictUtil.to_bool(row.get("basic", false)):
			return "'%s' is a basic skill: basics are never deleted." % head_id
	if SkillKinds.REQUIRED_IDS.has(head_id):
		return "'%s' is a required engine skill (system)." % head_id
	if is_system_origin(rows[0]) and not _can_remove_system():
		return "'%s' is a system asset (origin system): delete is refused. Use SAFE DELETE (tools/safe_delete.gd) or make your own copy." % head_id
	var users: Array[String] = []
	for row: Dictionary in rows:
		for user: String in references_to("skills", str(row["id"])):
			if not users.has(user):
				users.append(user)
	if not users.is_empty():
		return "'%s' is still used by: %s." % [head_id, ", ".join(users)]
	var list: Array = _data_map()["skills"]
	var keep: Array = []
	for existing: Variant in list:
		if SkillRanks.head_of(DictUtil.dict_of(existing)) != head_id:
			keep.append(existing)
	list.clear()
	list.append_array(keep)
	_touched("skills")
	return ""

# A free id based on `base` (base, base_2, base_3...).
static func unique_id(kind: String, base: String) -> String:
	var stem: String = base if ContentSchema.is_valid_id(base) else "new_entry"
	if not has(kind, stem):
		return stem
	var n: int = 2
	while has(kind, "%s_%d" % [stem, n]):
		n += 1
	return "%s_%d" % [stem, n]

# A copy of `id` under a fresh id (and "(Copy)" name), stored right away.
static func duplicate_entry(kind: String, id: String) -> String:
	if kind == "skills":
		return _duplicate_skill(id)
	var src: Dictionary = get_entry(kind, id)
	if src.is_empty():
		return ""
	src["id"] = unique_id(kind, id + "_copy")
	src["name"] = str(src.get("name", id)) + " (Copy)"
	src.erase("system")
	src["origin"] = ContentSchema.ORIGIN_USER
	var errors: Array[String] = upsert(kind, src)
	return str(src["id"]) if errors.is_empty() else ""

# A user copy of a whole skill family (every rank) under a fresh head id.
static func _duplicate_skill(head_id: String) -> String:
	var src: Dictionary = skill_entry(head_id)
	if src.is_empty():
		return ""
	var stem: String = head_id + "_copy"
	var new_id: String = unique_id("skills", stem)
	while true:
		var clash: bool = false
		for n: int in range(1, SkillRanks.rank_count(src) + 1):
			clash = clash or has("skills", SkillRanks.rank_id(new_id, n))
		if not clash:
			break
		new_id = unique_id("skills", new_id + "_x")
	src["id"] = new_id
	src["name"] = str(src.get("name", head_id)).substr(0, 20) + " (Copy)"
	src.erase("system")
	src.erase("basic")
	src["origin"] = ContentSchema.ORIGIN_USER
	return new_id if upsert_skill(src).is_empty() else ""

static func _touched(kind: String) -> void:
	_dirty_map()[kind] = true
	if kind == "skills":
		SkillCatalog.clear_cache()

# ------------------------------------------------------------------------------
# References
# ------------------------------------------------------------------------------

# "kind:id" of every entry with a ref/refs field pointing at kind/id.
static func references_to(kind: String, id: String) -> Array[String]:
	var users: Array[String] = []
	for other_kind: String in ContentSchema.KINDS:
		for f: Dictionary in ContentSchema.fields(other_kind):
			if str(f.get("ref", "")) != kind:
				continue
			var key: String = str(f["key"])
			for e: Dictionary in entries(other_kind):
				var value: Variant = e.get(key, null)
				var hit: bool = value is String and str(value) == id
				if value is Array:
					var list: Array = value
					hit = list.has(id)
				var label: String = "%s:%s" % [other_kind, str(e.get("id", ""))]
				if hit and not users.has(label):
					users.append(label)
	if kind == "skills":
		for c: Dictionary in entries("classes"):
			for step: Variant in DictUtil.array_of(c.get("skill_path", [])):
				if str(DictUtil.dict_of(step).get("skill", "")) == id and not users.has("classes:%s" % str(c.get("id", ""))):
					users.append("classes:%s" % str(c.get("id", "")))
		for p: Dictionary in entries("passives"):
			for fx: Variant in DictUtil.array_of(p.get("effects", [])):
				if str(DictUtil.dict_of(fx).get("skill", "")) == id and not users.has("passives:%s" % str(p.get("id", ""))):
					users.append("passives:%s" % str(p.get("id", "")))
	return users

static func _rename_refs(kind: String, old_id: String, new_id: String) -> void:
	for other_kind: String in ContentSchema.KINDS:
		for f: Dictionary in ContentSchema.fields(other_kind):
			if str(f.get("ref", "")) != kind:
				continue
			var key: String = str(f["key"])
			for e: Dictionary in entries(other_kind):
				var value: Variant = e.get(key, null)
				if value is Array:
					var list: Array = value
					for i: int in range(list.size()):
						if str(list[i]) == old_id:
							list[i] = new_id
							_dirty_map()[other_kind] = true
				elif value is String and str(value) == old_id:
					e[key] = new_id
					_dirty_map()[other_kind] = true
	if kind == "skills":
		for c: Dictionary in entries("classes"):
			for step: Variant in DictUtil.array_of(c.get("skill_path", [])):
				var s: Dictionary = DictUtil.dict_of(step)
				if str(s.get("skill", "")) == old_id:
					s["skill"] = new_id
					_dirty_map()["classes"] = true
		for p: Dictionary in entries("passives"):
			for fx: Variant in DictUtil.array_of(p.get("effects", [])):
				var d: Dictionary = DictUtil.dict_of(fx)
				if str(d.get("skill", "")) == old_id:
					d["skill"] = new_id
					_dirty_map()["passives"] = true
		for s: Dictionary in entries("skills"):
			if str(s.get("family", "")) == old_id:
				s["family"] = new_id
				_dirty_map()["skills"] = true
		for u: Dictionary in entries("units"):
			var loadout: Variant = u.get("loadout", [])
			if loadout is Array:
				var fams: Array = loadout
				for i: int in range(fams.size()):
					if str(fams[i]) == old_id:
						fams[i] = new_id
						_dirty_map()["units"] = true

# ------------------------------------------------------------------------------
# Validation
# ------------------------------------------------------------------------------

# Problems with `entry` as a `kind` entry (current_id: the id it replaces, if editing).
static func validate_entry(kind: String, entry: Dictionary, current_id: String = "") -> Array[String]:
	var e: Dictionary = ContentSchema.normalize(kind, entry)
	var errors: Array[String] = ContentSchema.field_errors(kind, e)
	var id: String = str(e.get("id", ""))
	if id != current_id and has(kind, id):
		errors.append("ID '%s' is already used by another %s entry." % [id, kind.trim_suffix("s")])
	# References must resolve (and match their filters).
	for f: Dictionary in ContentSchema.fields(kind):
		var ref_kind: String = str(f.get("ref", ""))
		if ref_kind.is_empty() or not ContentSchema.is_visible(f, e):
			continue
		var key: String = str(f["key"])
		var targets: Array[String] = []
		var value: Variant = e.get(key, null)
		if value is Array:
			var value_list: Array = value
			targets.assign(value_list)
			if not DictUtil.to_bool(f.get("dupes", false)):
				var seen: Dictionary = {}
				for t: String in targets:
					if seen.has(t):
						errors.append("%s lists '%s' twice." % [str(f["label"]), t])
					seen[t] = true
		elif not str(value).is_empty():
			targets.append(str(value))
		for t: String in targets:
			if t == ContentSchema.NO_ITEM and _is_gear_field(kind, key):
				continue   # the sentinel: units accept it, classes get a specific error (_no_item_errors)
			var target: Dictionary = get_entry(ref_kind, t)
			if target.is_empty():
				errors.append("%s: unknown %s '%s'." % [str(f["label"]), ref_kind.trim_suffix("s"), t])
			elif not ContentSchema.passes_filter(f, target):
				errors.append("%s: '%s' can't be used here." % [str(f["label"]), t])
	match kind:
		"skills":
			errors.append_array(_skill_errors(e))
		"items":
			errors.append_array(_item_errors(e))
		"passives":
			errors.append_array(_passive_errors(e))
		"classes":
			errors.append_array(_class_errors(e))
		"units":
			errors.append_array(_unit_errors(e))
		"encounters":
			errors.append_array(_encounter_errors(e))
	errors.append_array(_no_item_errors(kind, e))
	return errors

# A gear slot or the pouch of a class / unit (the fields that may carry ContentSchema.NO_ITEM).
static func _is_gear_field(kind: String, key: String) -> bool:
	return (kind == "units" or kind == "classes") and (ContentSchema.LOADOUT_SLOTS.has(key) or key == "pouch")

# The NO_ITEM sentinel rules (BAL2-L 3.2): units only, alone in the pouch; no item may take the id.
static func _no_item_errors(kind: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	match kind:
		"items":
			if str(e.get("id", "")) == ContentSchema.NO_ITEM:
				errors.append("The item id '%s' is reserved." % ContentSchema.NO_ITEM)
		"units":
			var pouch: Array[String] = ContentFactory._strings(e.get("pouch", []))
			if pouch.has(ContentSchema.NO_ITEM) and pouch.size() > 1:
				errors.append("Battle pouch: '%s' must be the only entry." % ContentSchema.NO_ITEM)
		"classes":
			for f: Dictionary in ContentSchema.fields(kind):
				var key: String = str(f["key"])
				if not _is_gear_field(kind, key):
					continue
				var hit: bool = str(e.get(key, "")) == ContentSchema.NO_ITEM
				if key == "pouch":
					hit = ContentFactory._strings(e.get(key, [])).has(ContentSchema.NO_ITEM)
				if hit:
					errors.append("%s: '%s' is only for units; leave the class slot empty." % [str(f["label"]), ContentSchema.NO_ITEM])
	return errors

# Every problem in the catalog: {"kind:id": [errors]} (empty = all valid).
static func validate_all() -> Dictionary:
	var problems: Dictionary = {}
	for kind: String in ContentSchema.KINDS:
		for e: Dictionary in entries(kind):
			var id: String = str(e.get("id", ""))
			var errors: Array[String] = validate_entry(kind, e, id)
			if not errors.is_empty():
				problems["%s:%s" % [kind, id]] = errors
	for message: String in _required_skill_errors():
		problems["skills:required"] = _append_problem(problems.get("skills:required", []), message)
	# Families save() could not fold into ranked entries (or would fold with data loss): saving skills is refused.
	var folded: Dictionary = SkillRanks.collapse(entries("skills"))
	var fold_errors: Array[String] = ContentFactory._strings(folded["errors"])
	if not fold_errors.is_empty():
		problems["skills:fold"] = fold_errors
	if not _error_list().is_empty():
		problems["load"] = _error_list().duplicate()
	return problems

static func _append_problem(existing: Variant, message: String) -> Array[String]:
	var out: Array[String] = ContentFactory._strings(existing)
	out.append(message)
	return out

# The skills the engine looks up by id must exist with the right kind, and basics, required and engine skills are system assets.
static func _required_skill_errors() -> Array[String]:
	var errors: Array[String] = []
	for required: String in SkillKinds.REQUIRED_IDS:
		var row: Dictionary = get_entry("skills", required)
		if row.is_empty():
			errors.append("Required engine skill '%s' is missing." % required)
		elif not is_system_origin(row):
			errors.append("Required engine skill '%s' must be origin system." % required)
	for row: Dictionary in entries("skills"):
		if (DictUtil.to_bool(row.get("basic", false)) or DictUtil.to_bool(row.get("system", false))) and not is_system_origin(row):
			errors.append("'%s' is a basic or engine skill and must be origin system." % str(row.get("id", "")))
	return errors

# Soft problems of one entry (never block saving): gear budget rules (GearBudget), skill description / hand hints.
static func warnings_for(kind: String, entry: Dictionary) -> Array[String]:
	var e: Dictionary = ContentSchema.normalize(kind, entry)
	match kind:
		"items":
			return GearBudget.check_item(e)
		"classes":
			return GearBudget.check_class(e)
		"skills":
			return _skill_warnings(entry, e)
	var none: Array[String] = []
	return none

# `raw` is a ranked entry (no tier / family keys) or a flat row; `e` its normalized form.
static func _skill_warnings(raw: Dictionary, e: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var ranked: bool = not raw.has("tier") and not raw.has("family")
	if ranked:
		out.append_array(SkillText.warnings(str(e["description"]), raw))
		var base: Dictionary = SkillRanks.effective(raw, 1)
		var mods: Array[Dictionary] = SkillRanks.modifiers_of(raw)
		for i: int in range(mods.size()):
			for key: Variant in mods[i]:
				if SkillRanks.RANK_KEYS.has(str(key)) and base.get(str(key), null) == mods[i][key]:
					out.append("Rank %d: %s equals the base value; the modifier is dropped on save." % [i + 2, str(key)])
	elif not str(e.get("template", "")).is_empty():
		out.append_array(SkillText.warnings(str(e["template"]), e))
	if str(e["weapon_hand"]) != "main":
		for gear: String in ContentFactory._strings(e["weapon_types"]):
			if gear != SkillDef.UNARMED_TOKEN:
				out.append("Weapon hand '%s' with weapon type '%s': off-hand slots hold shields, orbs and tomes, so only 'unarmed' can satisfy this gate." % [str(e["weapon_hand"]), gear])
	return out

# Every warning in the catalog: {"kind:id": [warnings]}. Separate from validate_all(). Skills are checked once per
# family as ranked entries (keyed by the head id), so rank warnings (literal rank descriptions) fire and the template
# warnings are not repeated for every rank row.
static func warnings_all() -> Dictionary:
	var out: Dictionary = {}
	for kind: String in ["items", "classes"]:
		for e: Dictionary in entries(kind):
			var found: Array[String] = warnings_for(kind, e)
			if not found.is_empty():
				out["%s:%s" % [kind, str(e.get("id", ""))]] = found
	for ranked: Dictionary in skill_entries():
		var skill_found: Array[String] = warnings_for("skills", ranked)
		if not skill_found.is_empty():
			out["skills:%s" % str(ranked.get("id", ""))] = skill_found
	return out

# Skill family rules (FORGE1 sections 2-3): a valid family id, ids <head> / <head>_<n>, ranks 1..n without gaps (basic 1,
# class 3), one kind per family, and every base-only field (not rank-modifiable, not derived) identical across the ranks.
static func _family_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var id: String = str(e.get("id", ""))
	var family: String = str(e.get("family", ""))
	var head: String = id if family.is_empty() else family
	if not family.is_empty() and not ContentSchema.is_valid_id(family):
		errors.append("Family '%s' must be lowercase snake_case (a-z, 0-9, _)." % family)
	var tier: int = DictUtil.to_int(e.get("tier", 1), 1)
	var basic: bool = DictUtil.to_bool(e.get("basic", false))
	var system: bool = DictUtil.to_bool(e.get("system", false))
	var kind: String = str(e.get("kind", ""))
	if basic and tier > 1:
		errors.append("Basic skills have exactly one rank (tier %d is above the 1-tier limit for basic skills)." % tier)
	elif not basic and not system and tier > StatRules.MAX_CLASS_TIER:
		errors.append("Tier %d is above the %d-tier limit for class skills." % [tier, StatRules.MAX_CLASS_TIER])
	if tier <= 1:
		if not family.is_empty() and family != id:
			errors.append("A tier 1 skill is the family head: its id '%s' must equal the family '%s'." % [id, family])
	elif id != SkillRanks.rank_id(head, tier):
		errors.append("Tier %d of family '%s' must have the id '%s' (found '%s')." % [tier, head, SkillRanks.rank_id(head, tier), id])
	var tiers: Array[int] = [tier]
	for other: Dictionary in entries("skills"):
		var other_id: String = str(other.get("id", ""))
		if other_id == id or SkillRanks.head_of(other) != head:
			continue
		var other_tier: int = DictUtil.to_int(other.get("tier", 1), 1)
		if other_tier == tier:
			errors.append("Family '%s' already has a tier %d skill (%s)." % [head, tier, other_id])
		tiers.append(other_tier)
		if str(other.get("kind", "")) != kind:
			errors.append("Family '%s' mixes kinds (%s is %s)." % [head, other_id, str(other.get("kind", ""))])
		if DictUtil.to_bool(other.get("basic", false)) != basic:
			errors.append("Family '%s': basic must match across all tiers (%s differs)." % [head, other_id])
		for f: Dictionary in ContentSchema.fields("skills"):
			var key: String = str(f["key"])
			if SkillRanks.DERIVED_KEYS.has(key) or SkillRanks.RANK_KEYS.has(key) or key == "basic" or key == "kind" or not SkillKinds.has_field(kind, key):
				continue
			if other.get(key, null) != e.get(key, null):
				errors.append("Family '%s': %s must match across all tiers (%s differs)." % [head, str(f.get("label", key)).to_lower(), other_id])
	tiers.sort()
	for i: int in range(tiers.size()):
		if tiers[i] != i + 1:
			errors.append("Family '%s': tiers must run 1..n without gaps (it needs a tier %d skill)." % [head, i + 1])
			break
	return errors

static func grade_rank(grade: String) -> int:
	return maxi(0, ["novice", "advanced", "expert"].find(grade))

static func _skill_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = _family_errors(e)
	for p: Dictionary in _skill_problems(e):
		errors.append(str(p["message"]))
	# The description template of a family is validated once, from its first rank, against all the family's ranks.
	var template: String = str(e.get("template", ""))
	if not template.is_empty() and DictUtil.to_int(e.get("tier", 1), 1) <= 1:
		var rows: Array[Dictionary] = [e]
		for other: Dictionary in family_rows(str(e.get("id", ""))):
			if str(other.get("id", "")) != str(e.get("id", "")):
				rows.append(other)
		var folded: Dictionary = SkillRanks.fold(str(e.get("id", "")), rows)
		var fold_errors: Array = folded["errors"]
		if fold_errors.is_empty():
			for p: Dictionary in SkillText.validate(template, DictUtil.dict_of(folded["entry"])):
				errors.append(str(p["message"]))
	return errors

# Row-level rules of one flat skill row: [{"field", "rank", "message"}].
static func _skill_problems(e: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var kind: String = str(e["kind"])
	var id: String = str(e["id"])
	var tier: int = DictUtil.to_int(e.get("tier", 1), 1)
	var basic: bool = DictUtil.to_bool(e.get("basic", false))
	var system: bool = DictUtil.to_bool(e.get("system", false))
	var grade: String = str(e["grade"])
	var unlock_offset: int = DictUtil.to_int(e.get("unlock_offset", -1), -1)
	if unlock_offset < -1 or unlock_offset > StatRules.MAX_UNLOCK_OFFSET:
		out.append(_problem("unlock_offset", tier, "Unlock offset must be -1 (derived) or 0-%d." % StatRules.MAX_UNLOCK_OFFSET))
	elif unlock_offset > 0 and tier == 1:
		out.append(_problem("unlock_offset", tier, "A tier 1 skill opens at its family's unlock level: leave the unlock offset at -1 or 0."))
	var requires: Array[String] = ContentFactory._strings(e.get("requires", []))
	if (basic or system) and grade != "novice":
		out.append(_problem("grade", tier, "Only class skills can be advanced (not basic or engine skills)."))
	if (basic or system) and not requires.is_empty():
		out.append(_problem("requires", tier, "Basic and engine skills cannot require other skills."))
	var head: String = SkillRanks.head_of(e)
	for required: String in requires:
		var target: Dictionary = get_entry("skills", required)
		if target.is_empty():
			continue
		if required == head:
			out.append(_problem("requires", tier, "A skill cannot require itself."))
		elif not str(target.get("family", "")).is_empty() or DictUtil.to_int(target.get("tier", 1), 1) != 1:
			out.append(_problem("requires", tier, "Requires: '%s' is a higher rank; name the family head." % required))
		elif grade_rank(str(target.get("grade", "novice"))) >= grade_rank(grade):
			out.append(_problem("requires", tier, "Requires: '%s' (%s) must be a lower grade than this skill (%s)." % [required, str(target.get("grade", "novice")), grade]))
	if tier == 1:
		for other: Dictionary in entries("skills"):
			if SkillRanks.head_of(other) == head or not ContentFactory._strings(other.get("requires", [])).has(head):
				continue
			if DictUtil.to_int(other.get("tier", 1), 1) == 1 and grade_rank(str(other.get("grade", "novice"))) <= grade_rank(grade):
				out.append(_problem("grade", tier, "Grade: '%s' requires this skill and must stay a higher grade (a grade change would break it)." % str(other.get("id", ""))))
	if not system and not SkillKinds.is_authorable(kind):
		out.append(_problem("kind", tier, "%s skills are engine-only; author an %s skill." % [kind, " / ".join(SkillKinds.authorable())]))
	# One rule replaces the old per-field "belongs to" messages: a field is set but the kind has no section for it.
	var guarded: Array[String] = SkillKinds.sectioned_fields()
	guarded.append("stance")
	for key: String in guarded:
		if SkillKinds.has_field(kind, key):
			continue
		var f: Dictionary = ContentSchema.field("skills", key)
		if e.get(key, null) != f["default"]:
			out.append(_problem(key, tier, "%s belongs to %s skills, not %s." % [str(f["label"]), "/".join(SkillKinds.kinds_with(key)), kind]))
	var status_id: String = str(e["status_effect"])
	var magnitude: int = DictUtil.to_int(e["status_magnitude"])
	if status_id == StatusEffects.WEAKEN:
		if magnitude < 1 or magnitude > StatusRegistry.cap_of(StatusEffects.WEAKEN):
			out.append(_problem("status_magnitude", tier, "weaken needs a magnitude 1-%d" % StatusRegistry.cap_of(StatusEffects.WEAKEN)))
	elif magnitude != 0:
		out.append(_problem("status_magnitude", tier, "magnitude only applies to weaken"))
	var grant: String = str(e["grant_status"])
	var reach_min: int = DictUtil.to_int(e["reach_min"])
	var reach_max: int = DictUtil.to_int(e["reach_max"])
	if kind == "SUPPORT":
		var heal_stat: String = str(e["heal_stat"])
		var heal_mult: float = DictUtil.to_float(e["heal_mult"])
		var heal_base: int = DictUtil.to_int(e["heal_base"])
		var cleanse: Array[String] = ContentFactory._strings(e["cleanse_statuses"])
		if heal_base <= 0 and heal_mult <= 0.0 and grant.is_empty() and cleanse.is_empty():
			out.append(_problem("heal_base", tier, "A support skill needs a heal (base or per-stat), a cleanse or a granted status."))
		if cleanse.has(SkillDef.CLEANSE_ALL) and cleanse.size() > 1:
			out.append(_problem("cleanse_statuses", tier, "Cleanses: '%s' stands alone (it already clears every debuff)." % SkillDef.CLEANSE_ALL))
		for status: String in cleanse:
			if status != SkillDef.CLEANSE_ALL and not StatusEffects.debuffs().has(status):
				out.append(_problem("cleanse_statuses", tier, "Cleanses: '%s' is not a debuff status (%s, or '%s')." % [status, ", ".join(StatusEffects.debuffs()), SkillDef.CLEANSE_ALL]))
		if heal_mult > 0.0 and not SupportRules.HEAL_STATS.has(heal_stat):
			out.append(_problem("heal_stat", tier, "Heal per stat point needs a heal stat (INT or KNOW)."))
		if heal_mult <= 0.0 and not heal_stat.is_empty():
			out.append(_problem("heal_stat", tier, "Heal stat is set but heal per stat point is 0."))
		if not grant.is_empty() and not StatusEffects.support_buffs().has(grant):
			out.append(_problem("grant_status", tier, "Grants: '%s' is not a support buff status (%s)." % [grant, ", ".join(StatusEffects.support_buffs())]))
		if reach_max > 0 and reach_min > reach_max:
			out.append(_problem("reach_min", tier, "Reach min is above reach max."))
		if grant == StatusEffects.TAUNT and str(e["target_mode"]) != "SELF":
			out.append(_problem("target_mode", tier, "Taunt is cast on the taunter itself: set Targets to SELF."))
	elif kind == "DEFEND":
		var defend_options: Array[String] = StatusEffects.defend_buffs()
		defend_options.append_array(StatusEffects.support_buffs())
		if not grant.is_empty() and not defend_options.has(grant):
			out.append(_problem("grant_status", tier, "Grants: '%s' is not a defend or support buff status (%s)." % [grant, ", ".join(defend_options)]))
	if kind == "DISENGAGE" and DictUtil.to_int(e["jump_tiles"]) < 1:
		out.append(_problem("jump_tiles", tier, "A disengage skill needs jump tiles of at least 1."))
	if kind == "ATTACK":
		var area: String = str(e["area"])
		var dash: int = DictUtil.to_int(e["dash_tiles"])
		var hand: String = str(e["weapon_hand"])
		if DictUtil.to_float(e["power_mult"]) <= 0.0:
			if status_id.is_empty():
				out.append(_problem("power_mult", tier, "A skill with no damage needs a status"))
			if area != "NONE":
				out.append(_problem("area", tier, "A skill with no damage can't be an area attack."))
		if area == "SWEEP" and (dash > 0 or reach_max > 1):
			out.append(_problem("area", tier, "SWEEP hits the L of tiles around you; leave reach at 0 or 1 and no gap-close."))
		if dash > 0:
			if area != "NONE":
				out.append(_problem("dash_tiles", tier, "An area attack can't be a gap-close."))
			if hand != "main":
				out.append(_problem("dash_tiles", tier, "An off-hand or dual-wield strike can't be a gap-close."))
		if hand == "both" and not SkillDef.BOTH_ENABLED:
			out.append(_problem("weapon_hand", tier, "Weapon hand 'both' is not playable yet (engine pending)."))
		if not status_id.is_empty() and not StatusEffects.debuffs().has(status_id):
			out.append(_problem("status_effect", tier, "Inflicts: '%s' is not a debuff status (%s)." % [status_id, ", ".join(StatusEffects.debuffs())]))
		if reach_max > 0 and reach_min > reach_max:
			out.append(_problem("reach_min", tier, "Reach min is above reach max."))
		if reach_max == 0 and reach_min > 0:
			out.append(_problem("reach_max", tier, "Set reach max too (or leave both at 0 for the unit's default band)."))
		if area == "LINE" and reach_max > 1:
			out.append(_problem("reach_max", tier, "LINE areas start from an adjacent tile; leave the reach at 0 or 1."))
		if not status_id.is_empty() and DictUtil.to_int(e["status_potency"]) <= 0:
			out.append(_problem("status_potency", tier, "Give the status a potency above 0."))
	# The weapon gate (ATTACK, DEFEND, SUPPORT): known gear types, and a type needs a proficiency to mean anything.
	var types: Array[String] = ContentFactory._strings(e["weapon_types"])
	for gear: String in types:
		if not SkillDef.weapon_type_tokens().has(gear):
			out.append(_problem("weapon_types", tier, "Needs weapon type: unknown gear type '%s' (gear types or '%s')." % [gear, SkillDef.UNARMED_TOKEN]))
	if not types.is_empty() and DictUtil.to_int(e["min_proficiency"]) < 1:
		out.append(_problem("min_proficiency", tier, "Needs proficiency: a weapon type is listed, so give a proficiency of at least 1 (otherwise the types do nothing)."))
	return out

static func _item_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var type: String = str(e["item_type"])
	var slot: String = str(e["slot"])
	if type != "WEAPON" and Equipment.FOCUS_TYPES.has(str(e["weapon_type"])) and ContentSchema.EQUIP_TYPES.has(type):
		errors.append("Orb and tome are weapon types (item type WEAPON).")
	if Equipment.FOCUS_TYPES.has(str(e["off_hand_category"])) and str(e["weapon_type"]) != str(e["off_hand_category"]):
		errors.append("Off-hand category '%s' needs the matching gear type '%s'." % [str(e["off_hand_category"]), str(e["off_hand_category"])])
	if ContentSchema.EQUIP_TYPES.has(type):
		errors.append_array(Passives.validate_effects(e["effects"], func(skill_id: String) -> bool: return has("skills", skill_id), true, true))
	match type:
		"WEAPON":
			if slot != "MAIN_HAND" and slot != "OFF_HAND":
				errors.append("Weapons go in MAIN_HAND or OFF_HAND.")
			if DictUtil.to_bool(e["two_handed"]) and slot != "MAIN_HAND":
				errors.append("Two-handed weapons use the MAIN_HAND slot.")
			if DictUtil.to_bool(e["two_handed"]) and not str(e["off_hand_category"]).is_empty():
				errors.append("Hand conflict: a two-handed weapon (main_2h) can't also be an off-hand category weapon (off).")
			if Equipment.FOCUS_TYPES.has(str(e["weapon_type"])):
				if slot != "OFF_HAND":
					errors.append("Orb and tome weapons are off-hand only (use the OFF_HAND slot).")
				if str(e["off_hand_category"]) != str(e["weapon_type"]):
					errors.append("An orb or tome needs off-hand category '%s' (matching its gear type)." % str(e["weapon_type"]))
				if DictUtil.to_bool(e["two_handed"]):
					errors.append("Orb and tome weapons are never two-handed.")
			var tiers: Variant = e["threshold_tiers"]
			if not (tiers is Array):
				errors.append("Threshold tiers must be a list.")
			else:
				var tier_list: Array = tiers
				for t: Variant in tier_list:
					var tier: Dictionary = t if t is Dictionary else {}
					if not tier.has("stat") or not tier.has("threshold"):
						errors.append("Each threshold tier needs \"stat\" and \"threshold\".")
						break
		"SHIELD":
			if slot != "OFF_HAND":
				errors.append("Shields go in the OFF_HAND slot.")
		"ARMOR":
			if slot != "HEAD" and slot != "BODY":
				errors.append("Armor goes in HEAD or BODY.")
		"ACCESSORY":
			if slot != "ACCESSORY_1" and slot != "ACCESSORY_2":
				errors.append("Accessories go in ACCESSORY_1 or ACCESSORY_2.")
		"CONSUMABLE":
			if DictUtil.to_int(e["heal"]) == 0 and DictUtil.to_int(e["damage"]) == 0 and DictUtil.to_float(e["buff_pct"]) == 0.0:
				errors.append("A consumable should heal, damage or buff.")
	return errors

static func _passive_errors(e: Dictionary) -> Array[String]:
	return Passives.validate_effects(e["effects"], func(skill_id: String) -> bool: return has("skills", skill_id))

# Class rules: equal stat budget and growth (StatRules), valid proficiency, a skill path whose
# skills fit the level cap, and default gear the class can wield.
static func _class_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = StatRules.validate_class(ContentFactory._ints(e["base_stats"]), ContentFactory._ints(e["growth"]))
	if not UnitArt.CLASS_KEYS.has(str(e["art"])):
		errors.append("Art '%s' isn't a known token/portrait key." % str(e["art"]))
	var prof_raw: Variant = e["proficiency"]
	var weapons: int = 0
	if not (prof_raw is Dictionary) or (prof_raw as Dictionary).is_empty():
		errors.append("A class needs a gear proficiency dictionary.")
	else:
		var prof: Dictionary = prof_raw
		for key: Variant in prof:
			var gear: String = str(key)
			var entry: Dictionary = DictUtil.dict_of(prof[key])
			if not Proficiency.gear_types().has(gear):
				errors.append("Proficiency: unknown gear type '%s'." % gear)
			if not entry.has("level") or DictUtil.to_int(entry.get("level", 0)) < 1 or DictUtil.to_int(entry.get("level", 0)) > Proficiency.MAX_LEVEL:
				errors.append("Proficiency %s: level must be 1-%d." % [gear, Proficiency.MAX_LEVEL])
			var scaling: float = DictUtil.to_float(entry.get("scaling", Proficiency.DEFAULT_SCALING), -1.0)
			if scaling < Proficiency.MIN_SCALING or scaling > Proficiency.MAX_SCALING:
				errors.append("Proficiency %s: scaling must be %.1f-%.1f." % [gear, Proficiency.MIN_SCALING, Proficiency.MAX_SCALING])
			if Proficiency.WEAPON_TYPES.has(gear):
				weapons += 1
		if weapons == 0:
			errors.append("Proficiency needs at least one weapon type.")
	var path: Variant = e["skill_path"]
	if not (path is Array):
		errors.append("The skill path must be a list of {level, skill}.")
	else:
		var last_level: int = 0
		var seen_families: Array[String] = []
		var family_levels: Dictionary = {}   # family head -> lowest path level (for the `requires` rule)
		for step_raw: Variant in (path as Array):
			var raw_step: Dictionary = DictUtil.dict_of(step_raw)
			var raw_skill: String = str(raw_step.get("skill", ""))
			var raw_fam: String = SkillRanks.head_of(get_entry("skills", raw_skill)) if has("skills", raw_skill) else raw_skill
			var raw_level: int = DictUtil.to_int(raw_step.get("level", 0))
			if not family_levels.has(raw_fam) or raw_level < DictUtil.int_of(family_levels, raw_fam):
				family_levels[raw_fam] = raw_level
		for step_raw: Variant in (path as Array):
			var step: Dictionary = DictUtil.dict_of(step_raw)
			var skill_id: String = str(step.get("skill", ""))
			var level: int = DictUtil.to_int(step.get("level", 0))
			var skill: Dictionary = get_entry("skills", skill_id)
			if level < 1 or level > StatRules.MAX_LEVEL:
				errors.append("Skill path: level %d is outside 1-%d." % [level, StatRules.MAX_LEVEL])
			if level < last_level:
				errors.append("Skill path must be listed in ascending level order.")
			last_level = maxi(last_level, level)
			if skill.is_empty():
				errors.append("Skill path: unknown skill '%s'." % skill_id)
			elif DictUtil.to_bool(skill.get("system", false)):
				errors.append("Skill path: '%s' is an engine skill (always known)." % skill_id)
			elif DictUtil.to_bool(skill.get("basic", false)):
				errors.append("Skill path: '%s' is a basic skill; every unit owns the basics from level 1, so they are not valid in a class path." % skill_id)
			else:
				var fam: String = str(skill.get("family", ""))
				if fam.is_empty():
					fam = skill_id
				if DictUtil.to_int(skill.get("tier", 1), 1) != 1:
					errors.append("Skill path: '%s' is a higher tier; list the tier 1 skill of the family (tiers open with it)." % skill_id)
				if seen_families.has(fam):
					errors.append("Skill path: family '%s' is listed more than once." % fam)
				seen_families.append(fam)
				if str(skill.get("grade", "novice")) != "novice" and level < StatRules.ADVANCED_MIN_LEVEL:
					errors.append("Skill path: %s is an advanced skill and needs unit level %d or higher (listed at %d)." % [skill_id, StatRules.ADVANCED_MIN_LEVEL, level])
				for required: String in ContentFactory._strings(skill.get("requires", [])):
					if not family_levels.has(required):
						errors.append("Skill path: %s needs %s on the same path." % [skill_id, required])
					elif DictUtil.int_of(family_levels, required) > level:
						errors.append("Skill path: %s (level %d) needs %s first, but %s is only listed at level %d." % [skill_id, level, required, required, DictUtil.int_of(family_levels, required)])
	errors.append_array(_loadout_errors(e, ContentFactory.class_archetype(e).make_proficiency(), true))
	return errors

# Gear problems of a class or unit entry: every item fits its slot, is trained for by
# `prof` (when `check_training`), and a two-handed weapon isn't paired with a shield.
static func _loadout_errors(e: Dictionary, prof: Proficiency, check_training: bool) -> Array[String]:
	var errors: Array[String] = []
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		var item_id: String = str(e.get(slot, ""))
		if item_id.is_empty():
			continue
		var item: Dictionary = get_entry("items", item_id)
		if item.is_empty():
			continue
		if not ContentFactory.item_fits_slot(item, slot.to_upper()):
			errors.append("%s can't be equipped in %s." % [str(item.get("name", item_id)), slot.capitalize()])
		elif check_training and not str(item.get("weapon_type", "")).is_empty() and not prof.can_use(str(item["weapon_type"])):
			errors.append("This class has no training in '%s' (%s)." % [str(item["weapon_type"]), str(item.get("name", item_id))])
	var main: Dictionary = get_entry("items", str(e.get("main_hand", "")))
	var off: Dictionary = get_entry("items", str(e.get("off_hand", "")))
	if DictUtil.to_bool(main.get("two_handed", false)) and not off.is_empty():
		errors.append("A two-handed weapon can't be paired with an off-hand item.")
	return errors

static func _unit_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var fixed: bool = str(e["stat_mode"]) == "fixed"
	var arch: ClassArchetype = ContentFactory.class_of(e)
	if not fixed and arch == null:
		errors.append("Class-derived units need a class (or set the stat model to fixed).")
	if not fixed:
		errors.append_array(StatRules.validate_spend(ContentFactory._ints(e["spend"]), DictUtil.to_int(e["level"], 1)))
	elif not (e["spend"] as Dictionary).is_empty():
		errors.append("Fixed units don't spend points.")
	var prof: Proficiency = arch.make_proficiency() if arch != null else Proficiency.new()
	errors.append_array(_loadout_errors(e, prof, arch != null))
	if arch != null and not fixed:
		errors.append_array(_skill_loadout_errors(e, arch))
	errors.append_array(_unit_requires_errors(e, arch))
	return errors

# A skill in `extra_skills` or `loadout` needs every family of its `requires` on the class path (any level) or in extra_skills.
static func _unit_requires_errors(e: Dictionary, arch: ClassArchetype) -> Array[String]:
	var errors: Array[String] = []
	var owned: Array[String] = []
	if arch != null:
		for step: Dictionary in arch.skill_path:
			owned.append(SkillRanks.head_of(get_entry("skills", str(step.get("skill", "")))))
	var extras: Array[String] = ContentFactory._strings(e.get("extra_skills", []))
	for extra: String in extras:
		owned.append(SkillRanks.head_of(get_entry("skills", extra)))
	var checked: Array[String] = []
	checked.append_array(extras)
	checked.append_array(ContentFactory._strings(e.get("loadout", [])))
	for skill_id: String in checked:
		var skill: Dictionary = get_entry("skills", skill_id)
		for required: String in ContentFactory._strings(skill.get("requires", [])):
			if not owned.has(required):
				errors.append("Unit skill '%s' needs '%s' (on the class path or in the extra skills)." % [skill_id, required])
	return errors

# Active skill loadout of a unit: listed families must be owned (class path up to its level or extra skills),
# non-basic, unique, and fit MAX_CLASS_SLOTS class + MAX_EXTRA_SLOTS extra slots. The DEFAULT loadout
# (no `loadout` list) is trimmed to the limits, so only an explicit list can break them.
static func _skill_loadout_errors(e: Dictionary, arch: ClassArchetype) -> Array[String]:
	var errors: Array[String] = []
	var listed: Array[String] = ContentFactory._strings(e.get("loadout", []))
	if listed.is_empty():
		return errors
	var level: int = DictUtil.to_int(e.get("level", 1), 1)
	var owned_ids: Array[String] = arch.skills_at(level)
	owned_ids.append_array(ContentFactory._strings(e.get("extra_skills", [])))
	var owned: Array[String] = ContentFactory.families_of(owned_ids)
	var seen: Array[String] = []
	for fam: String in listed:
		if seen.has(fam):
			errors.append("Loadout: family '%s' is listed twice." % fam)
		seen.append(fam)
		if SkillCatalog.family_tiers(fam).is_empty():
			errors.append("Loadout: unknown skill family '%s'." % fam)
		elif SkillCatalog.is_basic_family(fam):
			errors.append("Loadout: '%s' is a basic family (always active, no slot)." % fam)
		elif not owned.has(fam):
			errors.append("Loadout: the unit does not own the skill family '%s'." % fam)
	var slots: Dictionary = ContentFactory.loadout_of(e, arch, level)
	var class_count: int = ContentFactory._strings(slots["class"]).size()
	var extra_count: int = ContentFactory._strings(slots["extra"]).size()
	if class_count > StatRules.MAX_CLASS_SLOTS:
		errors.append("Loadout: %d class families (max %d)." % [class_count, StatRules.MAX_CLASS_SLOTS])
	if extra_count > StatRules.MAX_EXTRA_SLOTS:
		errors.append("Loadout: %d extra families (max %d)." % [extra_count, StatRules.MAX_EXTRA_SLOTS])
	return errors

static func _encounter_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var party: Array = e["party"]
	var foes: Array = e["enemies"]
	if party.is_empty():
		errors.append("Add at least one party member.")
	if foes.is_empty():
		errors.append("Add at least one enemy.")
	if party.size() > 4:
		errors.append("The party holds at most 4 members.")
	var wave: Array = e["wave_units"]
	if not wave.is_empty():
		if DictUtil.to_int(e["wave_block"]) > DictUtil.to_int(e["total_blocks"]):
			errors.append("The wave block is past the end of the round (%d blocks)." % DictUtil.to_int(e["total_blocks"]))
		if DictUtil.to_int(e["wave_round"]) > DictUtil.to_int(e["max_rounds"]):
			errors.append("The wave round is after the last round.")
	var exits: Variant = e["exits"]
	if exits is Array:
		var exit_list: Array = exits
		for x: Variant in exit_list:
			var pair: Array = x if x is Array else []
			if pair.size() != 2:
				errors.append("Exit tiles are [x, y] pairs.")
				break
			var cell := Vector2i(DictUtil.to_int(pair[0]), DictUtil.to_int(pair[1]))
			if cell.x < 0 or cell.y < 0 or cell.x >= CombatGrid.GRID_SIZE or cell.y >= CombatGrid.GRID_SIZE:
				errors.append("Exit %s is off the 9x9 grid." % str(cell))
	else:
		errors.append("Exit tiles must be a list of [x, y] pairs.")
	return errors
