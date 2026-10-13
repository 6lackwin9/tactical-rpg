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
# ClassArchetype presets, SkirmishEncounter, the tests and the Content Forge.
# Tests that write use use_root("user://...") so the shipped catalog is untouched.
# ==============================================================================

const DEFAULT_ROOT: String = "res://data/catalog"
const VERSION: int = 1


# ------------------------------------------------------------------------------
# Loading & saving
# ------------------------------------------------------------------------------

# ------------------------------------------------------------------------------
# State
# Kept on the Engine singleton rather than in static vars: Godot may reload this
# class's script mid-run (re-running static initialisers), which would silently
# point the catalog back at res:// and drop unsaved edits.
# ------------------------------------------------------------------------------

const STATE_KEY: StringName = &"content_catalog_state"

static func _state() -> Dictionary:
	if not Engine.has_meta(STATE_KEY):
		var errors: Array[String] = []
		Engine.set_meta(STATE_KEY, {"root": DEFAULT_ROOT, "data": {}, "dirty": {}, "errors": errors})
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
	reload()

# Copies the loaded catalog into `p_root` (replacing its catalog files) and
# switches to it: tests and experiments edit the copy, never the shipped data.
static func fork_to(p_root: String) -> Error:
	_ensure_loaded()
	_state()["root"] = p_root
	for kind: String in ContentSchema.KINDS:
		_dirty_map()[kind] = true
	var err: Error = save_all()
	if err == OK:
		reload()
	return err

static func reload() -> void:
	_data_map().clear()
	_dirty_map().clear()
	_error_list().clear()
	for kind: String in ContentSchema.KINDS:
		_data_map()[kind] = _read_kind(kind)
	SkillCatalog.clear_cache()

static func _ensure_loaded() -> void:
	if _data_map().is_empty():
		reload()

static func load_errors() -> Array[String]:
	_ensure_loaded()
	return _error_list().duplicate()

static func _read_kind(kind: String) -> Array:
	var found: Array = []
	var path: String = path_for(kind)
	if not FileAccess.file_exists(path):
		return found
	var text: String = FileAccess.get_file_as_string(path)
	var json := JSON.new()
	if json.parse(text) != OK:
		_error_list().append("%s: JSON error on line %d: %s" % [path, json.get_error_line(), json.get_error_message()])
		return found
	var parsed: Variant = json.data
	var raw: Variant = parsed
	if parsed is Dictionary:
		var doc: Dictionary = parsed
		raw = doc.get("entries", [])
	if not (raw is Array):
		_error_list().append("%s: expected an \"entries\" list." % path)
		return found
	var list: Array = raw
	for item: Variant in list:
		if item is Dictionary:
			var dict: Dictionary = item
			found.append(ContentSchema.normalize(kind, dict))
	return found

static func save(kind: String) -> Error:
	_ensure_loaded()
	var dir_err: Error = DirAccess.make_dir_recursive_absolute(get_root())
	if dir_err != OK and not DirAccess.dir_exists_absolute(get_root()):
		return dir_err
	var stored: Array = []
	for entry: Dictionary in entries(kind):
		stored.append(ContentSchema.to_storage(kind, entry))
	var doc: Dictionary = {"kind": kind, "version": VERSION, "entries": stored}
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

# Validates and stores `entry` (replacing `replace_id`, or the entry with the same
# id, in place; otherwise appended). Returns the problems; nothing changes if any.
# Renaming an id rewrites references to it in other entries.
static func upsert(kind: String, entry: Dictionary, replace_id: String = "") -> Array[String]:
	_ensure_loaded()
	var clean: Dictionary = ContentSchema.normalize(kind, entry)
	var new_id: String = str(clean.get("id", ""))
	var old_id: String = replace_id if not replace_id.is_empty() else new_id
	var errors: Array[String] = validate_entry(kind, clean, old_id)
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

# Removes an entry unless something still references it. Returns "" or why not.
static func remove(kind: String, id: String) -> String:
	var at: int = _index_of(kind, id)
	if at < 0:
		return "No %s entry '%s'." % [kind, id]
	var entry: Dictionary = entries(kind)[at]
	if DictUtil.to_bool(entry.get("system", false)):
		return "'%s' is a system entry the engine relies on." % id
	var users: Array[String] = references_to(kind, id)
	if not users.is_empty():
		return "'%s' is still used by: %s." % [id, ", ".join(users)]
	var list: Array = _data_map()[kind]
	list.remove_at(at)
	_touched(kind)
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
	var src: Dictionary = get_entry(kind, id)
	if src.is_empty():
		return ""
	src["id"] = unique_id(kind, id + "_copy")
	src["name"] = str(src.get("name", id)) + " (Copy)"
	src.erase("system")
	var errors: Array[String] = upsert(kind, src)
	return str(src["id"]) if errors.is_empty() else ""

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
		"units":
			errors.append_array(_unit_errors(e))
		"encounters":
			errors.append_array(_encounter_errors(e))
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
	if not _error_list().is_empty():
		problems["load"] = _error_list().duplicate()
	return problems

static func _skill_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var kind: String = str(e["kind"])
	if not DictUtil.to_bool(e.get("system", false)) and kind in ["MOVE", "TARGET", "STATUS"]:
		errors.append("%s skills are engine-only; author an ATTACK, DEFEND, FOCUS or REST skill." % kind)
	if kind == "ATTACK":
		if DictUtil.to_int(e["reach_max"]) > 0 and DictUtil.to_int(e["reach_min"]) > DictUtil.to_int(e["reach_max"]):
			errors.append("Reach min is above reach max.")
		if DictUtil.to_int(e["reach_max"]) == 0 and DictUtil.to_int(e["reach_min"]) > 0:
			errors.append("Set reach max too (or leave both at 0 for the unit's default band).")
		if str(e["area"]) == "LINE" and DictUtil.to_int(e["reach_max"]) > 1:
			errors.append("LINE areas start from an adjacent tile; leave the reach at 0 or 1.")
		if not str(e["status_effect"]).is_empty() and DictUtil.to_int(e["status_potency"]) <= 0:
			errors.append("Give the status a potency above 0.")
	return errors

static func _item_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var type: String = str(e["item_type"])
	var slot: String = str(e["slot"])
	match type:
		"WEAPON":
			if slot != "MAIN_HAND" and slot != "OFF_HAND":
				errors.append("Weapons go in MAIN_HAND or OFF_HAND.")
			if DictUtil.to_bool(e["two_handed"]) and slot != "MAIN_HAND":
				errors.append("Two-handed weapons use the MAIN_HAND slot.")
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

static func _unit_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	if str(e["stat_mode"]) == "derived" and str(e["class_id"]).is_empty():
		errors.append("Derived vitals need a class.")
	# Each loadout item must fit its slot (an off-hand slot takes shields, off-hand and 1H weapons).
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		var item_id: String = str(e.get(slot, ""))
		if item_id.is_empty():
			continue
		var item: Dictionary = get_entry("items", item_id)
		if item.is_empty():
			continue
		var engine_slot: String = slot.to_upper()
		if not ContentFactory.item_fits_slot(item, engine_slot):
			errors.append("%s can't be equipped in %s." % [str(item.get("name", item_id)), slot.capitalize()])
	var main_id: String = str(e.get("main_hand", ""))
	if not main_id.is_empty() and not str(e.get("off_hand", "")).is_empty():
		var main: Dictionary = get_entry("items", main_id)
		var off: Dictionary = get_entry("items", str(e["off_hand"]))
		if DictUtil.to_bool(main.get("two_handed", false)) and str(off.get("item_type", "")) == "SHIELD":
			errors.append("A two-handed weapon can't be paired with a shield.")
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
