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
# items.json is version 2 (FORGE1_ITEMS 8): stored entries are family + modifiers (ItemModifiers), flat resolved rows in
# memory. Until ITEM1-A migrates the shipped data, a catalog without item families keeps the version 1 flat file (legacy
# mode: items_inherit() is false, saving writes version 1); a version 1 file loaded next to families collapses on load.
const VERSIONS: Dictionary = {"skills": 2, "items": 2, "item_families": 1}

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
	_data_map()["item_families"] = _read_kind("item_families")   # first: items resolve against their family
	for kind: String in ContentSchema.KINDS:
		if kind != "item_families":
			_data_map()[kind] = _read_kind(kind)
	SkillCatalog.clear_cache()
	Terrain.clear_cache()
	AssetRegistry.clear_family_cache()   # skill family ids are art keys; a reload with the same root and row count must rebuild them

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
	if kind == "skills":
		push_warning("ContentCatalog: %s is a version %d skills file (flat rows); version 1 is deprecated, saving writes version 2 (CLN-3b removes v1 loading)" % [path, file_version])
	if kind == "items":
		return _read_items(list, path, file_version)
	for item: Variant in list:
		if item is Dictionary:
			var dict: Dictionary = item
			var row: Dictionary = ContentSchema.normalize(kind, dict)
			found.append(ItemFamilies.derive(row) if kind == "item_families" else row)
	return found

# items.json: version 2 entries resolve against their family; a version 1 (flat) file collapses on load when families
# exist, else stays flat (legacy mode until ITEM1-A). An entry whose family is unknown is kept as stored and blocks saving.
static func _read_items(list: Array, path: String, file_version: int) -> Array:
	var found: Array = []
	var families: Array = DictUtil.array_of(_data_map().get("item_families", []))
	for item: Variant in list:
		if not (item is Dictionary):
			_error_list().append("%s: an entry is not an object; it is skipped." % path)
			_block_save("items")
			continue
		var dict: Dictionary = item
		if file_version >= 2:
			var family: Dictionary = _family_in(families, str(dict.get("family", "")))
			if family.is_empty():
				_error_list().append("%s: item '%s' names the unknown family '%s'; it is kept as stored." % [path, str(dict.get("id", "")), str(dict.get("family", ""))])
				_block_save("items")
				found.append(ContentSchema.normalize("items", dict))
				continue
			found.append(ContentSchema.normalize("items", ItemModifiers.resolve(dict, family)))
			continue
		var flat: Dictionary = ContentSchema.normalize("items", dict)
		var legacy_family: Dictionary = _family_in(families, ItemFamilies.family_id_for_row(flat))
		if legacy_family.is_empty():
			found.append(flat)
		else:
			found.append(ContentSchema.normalize("items", ItemModifiers.resolve(ItemModifiers.collapse(flat, legacy_family), legacy_family)))
	return found

static func _family_in(families: Array, id: String) -> Dictionary:
	if id.is_empty():
		return {}
	for raw: Variant in families:
		var f: Dictionary = DictUtil.dict_of(raw)
		if str(f.get("id", "")) == id:
			return f
	return {}

# True once the catalog has item families (version 2 items: family + modifiers). False = legacy flat items (until ITEM1-A).
static func items_inherit() -> bool:
	return not entries("item_families").is_empty()

# The family row of an item row (empty when the item names none or an unknown one).
static func family_of(item: Dictionary) -> Dictionary:
	return _family_in(entries("item_families"), str(item.get("family", "")))

# The items that use family `family_id`.
static func items_of_family(family_id: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for row: Dictionary in entries("items"):
		if str(row.get("family", "")) == family_id:
			out.append(row)
	return out

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
	elif kind == "items" and items_inherit():
		var item_errors: Array[String] = []
		for entry: Dictionary in entries(kind):
			if family_of(entry).is_empty():
				item_errors.append("item '%s' has no known family ('%s'); a version 2 file stores family + modifiers only" % [str(entry.get("id", "")), str(entry.get("family", ""))])
				continue
			stored.append(ItemModifiers.stored(ContentSchema.to_storage(kind, entry)))
		_state()["save_errors"] = item_errors.duplicate()
		if not item_errors.is_empty():
			push_error("ContentCatalog.save(items) refused: " + "; ".join(PackedStringArray(item_errors)))
			return ERR_INVALID_DATA
	elif (kind == "item_families" or kind == "terrain") and entries(kind).is_empty() and not FileAccess.file_exists(path_for(kind)):
		_dirty_map().erase(kind)   # legacy catalog (no families / terrain rows yet): never create an empty file
		return OK
	else:
		for entry: Dictionary in entries(kind):
			stored.append(ContentSchema.to_storage(kind, entry))
	var version: int = version_of(kind)
	if kind == "items" and not items_inherit():
		version = VERSION   # legacy flat items (no families yet)
	var doc: Dictionary = {"kind": kind, "version": version, "entries": stored}
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
	Terrain.clear_cache()

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
	return SkillValidator.problem(field, rank, message)

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

# The normalised row `raw` is stored as. Families get their derived fields; items (once families exist) become the
# resolved row of their stored form (FORGE1_ITEMS 2.2: upsert accepts either shape).
static func _prepare(kind: String, raw: Dictionary, old: Dictionary) -> Dictionary:
	match kind:
		"item_families":
			return ItemFamilies.derive(ContentSchema.normalize(kind, raw))
		"items":
			return _prepare_item(raw, old)
		"terrain":
			return TerrainFields.prepare(ContentSchema.normalize(kind, raw))
	return ContentSchema.normalize(kind, raw)

# Item shapes: a STORED entry (family, no flat numbers) resolves as is. A FLAT row (legacy fixture, get_entry copy) whose
# engine keys differ from the stored row (or that is new) collapses, so the flat numbers win; a flat copy whose engine keys
# are unchanged keeps its own family / hand / modifiers / tiers (the Forge edits those on a get_entry copy).
static func _prepare_item(raw: Dictionary, old: Dictionary) -> Dictionary:
	if not items_inherit():
		return ContentSchema.normalize("items", raw)
	var source: Dictionary = raw
	var family_id: String = str(raw.get("family", ""))
	if not ItemModifiers.is_stored_shape(raw):
		var flat: Dictionary = ContentSchema.normalize("items", raw)
		# A legacy row (no family yet) always collapses; a resolved copy keeps its own lists while its engine keys hold.
		if old.is_empty() or str(old.get("family", "")).is_empty() or not ItemModifiers.engine_equal(flat, old):
			var own: Dictionary = get_entry("item_families", family_id)
			if own.is_empty() or not _family_matches(own, flat):
				family_id = ItemFamilies.family_id_for_row(flat)
			var target: Dictionary = get_entry("item_families", family_id)
			source = ItemModifiers.collapse(flat, target) if not target.is_empty() else flat
			if not old.is_empty() and source.has("modifiers"):   # keep the stored modifier order (no re-sorting)
				source["modifiers"] = ItemModifiers.stable_order(ItemModifiers.modifiers_of(source), ItemModifiers.modifiers_of(old))
		else:
			if family_id.is_empty():
				family_id = str(old.get("family", ""))
			source = ItemModifiers.stored(flat)
	var family: Dictionary = get_entry("item_families", family_id)
	if family.is_empty():
		var unresolved: Dictionary = ContentSchema.normalize("items", source)
		unresolved["family"] = family_id
		return unresolved
	return ContentSchema.normalize("items", ItemModifiers.resolve(source, family))

# A flat item copy that changes BOTH an engine key / number AND its modifiers or tiers is ambiguous (collapsing would
# silently drop the list edit): refused. Stored-shape entries, new rows and legacy rows (no family yet) never hit this.
static func item_shape_errors(raw: Dictionary, old: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	if not items_inherit() or old.is_empty() or str(old.get("family", "")).is_empty() or ItemModifiers.is_stored_shape(raw):
		return errors
	var flat: Dictionary = ContentSchema.normalize("items", raw)
	if ItemModifiers.engine_equal(flat, old):
		return errors
	var lists_changed: bool = (raw.has("modifiers") and not ItemModifiers.same_value(raw["modifiers"], old.get("modifiers", []))) \
		or (raw.has("tiers") and not ItemModifiers.same_value(raw["tiers"], old.get("tiers", [])))
	if lists_changed:
		errors.append("Item '%s': change the modifiers / tiers or the flat numbers in one upsert, not both (changed: %s)." % [str(old.get("id", "")), ", ".join(ItemModifiers.engine_diff(flat, old))])
	return errors

# True when flat row `flat` already has the type and gear token `family` derives.
static func _family_matches(family: Dictionary, flat: Dictionary) -> bool:
	var type: String = ItemFamilies.item_type_of(family)
	if str(flat.get("item_type", "")) != type:
		return false
	var token: String = str(family.get("id", "")) if ItemFamilies.is_gear_type(type) else ""
	return str(flat.get("weapon_type", "")) == token

# Validates and stores `entry` (replacing `replace_id`, or the entry with the same
# id, in place; otherwise appended). Returns the problems; nothing changes if any.
# Renaming an id rewrites references to it in other entries. A new id becomes a user asset; key fields of an
# existing system asset are locked (see key_field_errors).
static func upsert(kind: String, entry: Dictionary, replace_id: String = "") -> Array[String]:
	_ensure_loaded()
	var new_id: String = str(entry.get("id", ""))
	var old_id: String = replace_id if not replace_id.is_empty() else new_id
	var old: Dictionary = get_entry(kind, old_id)
	if kind == "items":
		var shape: Array[String] = item_shape_errors(entry, old)
		if not shape.is_empty():
			return shape
	var clean: Dictionary = _prepare(kind, _with_origin(entry, old), old)
	new_id = str(clean.get("id", ""))
	var errors: Array[String] = validate_entry(kind, clean, old_id)
	errors.append_array(key_field_errors(kind, old, clean))
	if kind == "skills" and old_id != new_id and not old.is_empty():
		errors.append_array(path_rename_conflicts(old_id, new_id))
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
		if kind == "items":
			for problem: String in item_shape_errors(src, old):
				errors.append("%s: %s" % [id, problem])
		var clean: Dictionary = _prepare(kind, _with_origin(src, old), old)
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
			elif SkillRanks.FAMILY_KEYS.has(key):
				out.append(_problem(key, rank, SkillRanks.FAMILY_KEY_MESSAGE % key))
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
	if old_head != head and not old_rows.is_empty():
		var conflicts: Array[String] = path_rename_conflicts(old_head, head)
		if not conflicts.is_empty():
			return conflicts
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
	if kind == "terrain":
		Terrain.clear_cache()
	if kind == "item_families":
		_reresolve_items()

# After a family change every item of a family resolves again (no cascade: one family, its own modifiers).
static func _reresolve_items() -> void:
	var list: Array = _data_map()["items"]
	var changed: bool = false
	for i: int in range(list.size()):
		var row: Dictionary = DictUtil.dict_of(list[i])
		var family: Dictionary = family_of(row)
		if family.is_empty():
			continue
		var fresh: Dictionary = ContentSchema.normalize("items", ItemModifiers.resolve(ItemModifiers.stored(row), family))
		if not ItemModifiers.same_value(fresh, row):
			list[i] = fresh
			changed = true
	if changed:
		_dirty_map()["items"] = true

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
			if DictUtil.dict_of(c.get("skill_path", {})).has(id) and not users.has("classes:%s" % str(c.get("id", ""))):
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
			var path: Dictionary = DictUtil.dict_of(c.get("skill_path", {}))
			if path.has(old_id) and not path.has(new_id):
				c["skill_path"] = _renamed_key(path, old_id, new_id)   # same position (FORGE1_CLASSES 3.5)
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

# Renaming skill family `old_id` to `new_id` is refused while a class path lists BOTH (a dict key cannot be renamed onto an
# existing key without losing an entry): one message per class, empty = the rename may proceed (_rename_refs rewrites keys).
static func path_rename_conflicts(old_id: String, new_id: String) -> Array[String]:
	var errors: Array[String] = []
	for c: Dictionary in entries("classes"):
		var path: Dictionary = DictUtil.dict_of(c.get("skill_path", {}))
		if path.has(old_id) and path.has(new_id):
			errors.append("Class '%s' lists both '%s' and '%s' in its skill path: remove one before renaming." % [str(c.get("id", "")), old_id, new_id])
	return errors

# A copy of ordered dict `d` with key `old_key` renamed to `new_key` at the same position.
static func _renamed_key(d: Dictionary, old_key: String, new_key: String) -> Dictionary:
	var out: Dictionary = {}
	for key: Variant in d:
		out[new_key if str(key) == old_key else key] = d[key]
	return out

# ------------------------------------------------------------------------------
# Validation
# ------------------------------------------------------------------------------

# Problems with `entry` as a `kind` entry (current_id: the id it replaces, if editing).
static func validate_entry(kind: String, entry: Dictionary, current_id: String = "") -> Array[String]:
	var e: Dictionary = ContentSchema.normalize(kind, entry)
	if kind == "items" or kind == "item_families":
		e = _prepare(kind, entry, get_entry(kind, current_id if not current_id.is_empty() else str(entry.get("id", ""))))
	# Terrain rows have no name and fixed ids (TerrainValidator owns every message).
	var errors: Array[String] = []
	if kind != "terrain":
		errors = ContentSchema.field_errors(kind, e)
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
			errors.append_array(SkillValidator.entry_errors(e))
		"items":
			errors.append_array(ItemValidator.item_errors(e, family_of(e), items_inherit()))
		"item_families":
			errors.append_array(ItemValidator.family_errors(e, current_id))
		"passives":
			errors.append_array(_passive_errors(e))
		"classes":
			errors.append_array(ClassUnitValidator.class_errors(e))
		"units":
			errors.append_array(ClassUnitValidator.unit_errors(e))
		"encounters":
			errors.append_array(_encounter_errors(e))
		"terrain":
			errors.append_array(TerrainValidator.entry_errors(e))
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
	for message: String in ItemValidator.family_set_errors(entries("item_families")):
		problems["item_families:required"] = _append_problem(problems.get("item_families:required", []), message)
	# Families save() could not fold into ranked entries (or would fold with data loss): saving skills is refused.
	var folded: Dictionary = SkillRanks.collapse(entries("skills"))
	var fold_errors: Array[String] = ContentFactory._strings(folded["errors"])
	if not fold_errors.is_empty():
		problems["skills:fold"] = fold_errors
	if terrain_file_exists():
		for id: String in TerrainValidator.missing_ids(entries("terrain")):
			problems["terrain:missing"] = _append_problem(problems.get("terrain:missing", []), "No terrain row for '%s'." % id)
	if not _error_list().is_empty():
		problems["load"] = _error_list().duplicate()
	return problems

# True once terrain.json exists (S12T-A): missing terrain rows are errors then; before that warnings_all() lists them
# and Terrain uses its code defaults.
static func terrain_file_exists() -> bool:
	return FileAccess.file_exists(path_for("terrain"))

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
			var item_found: Array[String] = GearBudget.check_item(e)
			item_found.append_array(ItemValidator.item_warnings(e, family_of(e)))
			return item_found
		"item_families":
			return ItemValidator.family_warnings(ItemFamilies.derive(e))
		"classes":
			return ClassUnitValidator.warning_texts(ClassUnitValidator.class_warnings(e))
		"units":
			return ClassUnitValidator.warning_texts(ClassUnitValidator.unit_warnings(e))
		"skills":
			return SkillValidator.warnings(entry, e)
	var none: Array[String] = []
	return none

# Every warning in the catalog: {"kind:id": [warnings]}. Separate from validate_all(). Skills are checked once per
# family as ranked entries (keyed by the head id), so rank warnings (literal rank descriptions) fire and the template
# warnings are not repeated for every rank row.
static func warnings_all() -> Dictionary:
	var out: Dictionary = {}
	for kind: String in ["items", "item_families", "classes", "units"]:
		for e: Dictionary in entries(kind):
			var found: Array[String] = warnings_for(kind, e)
			if not found.is_empty():
				out["%s:%s" % [kind, str(e.get("id", ""))]] = found
	for ranked: Dictionary in skill_entries():
		var skill_found: Array[String] = warnings_for("skills", ranked)
		if not skill_found.is_empty():
			out["skills:%s" % str(ranked.get("id", ""))] = skill_found
	if not terrain_file_exists():
		var missing: Array[String] = []
		for id: String in TerrainValidator.missing_ids(entries("terrain")):
			missing.append("No terrain row for '%s': the code defaults apply." % id)
		if not missing.is_empty():
			out["terrain:missing"] = missing
	return out

static func grade_rank(grade: String) -> int:
	return maxi(0, ["novice", "advanced", "expert"].find(grade))

static func _passive_errors(e: Dictionary) -> Array[String]:
	return Passives.validate_effects(e["effects"], func(skill_id: String) -> bool: return has("skills", skill_id))

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
	var wave_field: Dictionary = ContentSchema.field("encounters", "wave_units")
	errors.append_array(EncounterPhases.problems(e, func(ref: String) -> bool:
		var target: Dictionary = get_entry("units", ref)
		return not target.is_empty() and ContentSchema.passes_filter(wave_field, target)))
	# OBJ1: guests are party-side units; the generic ref check already refuses others, so every guest passes here.
	errors.append_array(EncounterObjectives.problems(e, func(_ref: String) -> bool: return true))
	var exits: Variant = e["exits"]
	if exits is Array:
		var exit_list: Array = exits
		for x: Variant in exit_list:
			var pair: Array = x if x is Array else []
			if pair.size() != 2:
				errors.append("Exit tiles are [x, y] pairs.")
				break
			var cell := Vector2i(DictUtil.to_int(pair[0]), DictUtil.to_int(pair[1]))
			if cell.x < 0 or cell.y < 0 or cell.x >= CombatGrid.GRID_W or cell.y >= CombatGrid.GRID_H:
				errors.append("Exit %s is off the 12x9 grid." % str(cell))
	else:
		errors.append("Exit tiles must be a list of [x, y] pairs.")
	return errors
