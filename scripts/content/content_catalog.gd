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
	for kind: String in ContentSchema.KINDS:
		_data_map()[kind] = _read_kind(kind)
	SkillCatalog.clear_cache()

static func _ensure_loaded() -> void:
	if _data_map().is_empty():
		reload_from_disk()

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

# Soft problems of one entry (never block saving): gear budget rules (GearBudget).
static func warnings_for(kind: String, entry: Dictionary) -> Array[String]:
	var e: Dictionary = ContentSchema.normalize(kind, entry)
	match kind:
		"items":
			return GearBudget.check_item(e)
		"classes":
			return GearBudget.check_class(e)
	var none: Array[String] = []
	return none

# Every warning in the catalog: {"kind:id": [warnings]}. Separate from validate_all().
static func warnings_all() -> Dictionary:
	var out: Dictionary = {}
	for kind: String in ["items", "classes"]:
		for e: Dictionary in entries(kind):
			var found: Array[String] = warnings_for(kind, e)
			if not found.is_empty():
				out["%s:%s" % [kind, str(e.get("id", ""))]] = found
	return out

# Skill family rules: a valid family id, tier within the family cap (basic 5, class 3), tiers unique
# per family, one kind per family, and basic-ness shared by the whole family.
static func _family_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var id: String = str(e.get("id", ""))
	var family: String = str(e.get("family", ""))
	if family.is_empty():
		family = id
	elif not ContentSchema.is_valid_id(family):
		errors.append("Family '%s' must be lowercase snake_case (a-z, 0-9, _)." % family)
	var tier: int = DictUtil.to_int(e.get("tier", 1), 1)
	var basic: bool = DictUtil.to_bool(e.get("basic", false))
	var cap: int = StatRules.MAX_TIER if basic else StatRules.MAX_CLASS_TIER
	if tier > cap and not DictUtil.to_bool(e.get("system", false)):
		errors.append("Tier %d is above the %d-tier limit for %s skills." % [tier, cap, "basic" if basic else "class"])
	for other: Dictionary in entries("skills"):
		var other_id: String = str(other.get("id", ""))
		if other_id == id:
			continue
		var other_family: String = str(other.get("family", ""))
		if (other_family if not other_family.is_empty() else other_id) != family:
			continue
		if DictUtil.to_int(other.get("tier", 1), 1) == tier:
			errors.append("Family '%s' already has a tier %d skill (%s)." % [family, tier, other_id])
		if str(other.get("kind", "")) != str(e.get("kind", "")):
			errors.append("Family '%s' mixes kinds (%s is %s)." % [family, other_id, str(other.get("kind", ""))])
		if DictUtil.to_bool(other.get("basic", false)) != basic:
			errors.append("Family '%s': basic must match across all tiers (%s differs)." % [family, other_id])
	return errors

static func _skill_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var kind: String = str(e["kind"])
	errors.append_array(_family_errors(e))
	if not DictUtil.to_bool(e.get("system", false)) and kind in ["MOVE", "TARGET", "STATUS"]:
		errors.append("%s skills are engine-only; author an ATTACK, DEFEND, FOCUS, REST or SUPPORT skill." % kind)
	if kind == "SUPPORT":
		var heal_stat: String = str(e["heal_stat"])
		var heal_mult: float = DictUtil.to_float(e["heal_mult"])
		var heal_base: int = DictUtil.to_int(e["heal_base"])
		var grant: String = str(e["grant_status"])
		if heal_base <= 0 and heal_mult <= 0.0 and grant.is_empty():
			errors.append("A support skill needs a heal (base or per-stat) or a granted status.")
		if heal_mult > 0.0 and not SupportRules.HEAL_STATS.has(heal_stat):
			errors.append("Heal per stat point needs a heal stat (INT or KNOW).")
		if heal_mult <= 0.0 and not heal_stat.is_empty():
			errors.append("Heal stat is set but heal per stat point is 0.")
		if not grant.is_empty() and not StatusEffects.support_buffs().has(grant):
			errors.append("Grants: '%s' is not a support buff status (%s)." % [grant, ", ".join(StatusEffects.support_buffs())])
		if DictUtil.to_int(e["reach_max"]) > 0 and DictUtil.to_int(e["reach_min"]) > DictUtil.to_int(e["reach_max"]):
			errors.append("Reach min is above reach max.")
		if grant == StatusEffects.TAUNT and str(e["target_mode"]) != "SELF":
			errors.append("Taunt is cast on the taunter itself: set Targets to SELF.")
	elif kind == "DEFEND":
		var defend_grant: String = str(e["grant_status"])
		if not defend_grant.is_empty() and not StatusEffects.defend_buffs().has(defend_grant):
			errors.append("Grants: '%s' is not a defend buff status (%s)." % [defend_grant, ", ".join(StatusEffects.defend_buffs())])
		if DictUtil.to_int(e["heal_base"]) > 0 or DictUtil.to_float(e["heal_mult"]) > 0.0:
			errors.append("Heal belongs to SUPPORT skills.")
	elif not str(e["grant_status"]).is_empty() or DictUtil.to_int(e["heal_base"]) > 0 or DictUtil.to_float(e["heal_mult"]) > 0.0:
		errors.append("Heal and Grants belong to SUPPORT skills.")
	if kind != "ATTACK" and DictUtil.to_int(e["dash_tiles"]) > 0:
		errors.append("Gap-close tiles belong to ATTACK skills.")
	if kind == "ATTACK":
		if DictUtil.to_int(e["dash_tiles"]) > 0:
			if str(e["area"]) != "NONE":
				errors.append("An area attack can't be a gap-close.")
			if DictUtil.to_bool(e["offhand"]):
				errors.append("An off-hand jab can't be a gap-close.")
		if not str(e["status_effect"]).is_empty() and not StatusEffects.debuffs().has(str(e["status_effect"])):
			errors.append("Inflicts: '%s' is not a debuff status (%s)." % [str(e["status_effect"]), ", ".join(StatusEffects.debuffs())])
		if DictUtil.to_int(e["reach_max"]) > 0 and DictUtil.to_int(e["reach_min"]) > DictUtil.to_int(e["reach_max"]):
			errors.append("Reach min is above reach max.")
		if DictUtil.to_int(e["reach_max"]) == 0 and DictUtil.to_int(e["reach_min"]) > 0:
			errors.append("Set reach max too (or leave both at 0 for the unit's default band).")
		if str(e["area"]) == "LINE" and DictUtil.to_int(e["reach_max"]) > 1:
			errors.append("LINE areas start from an adjacent tile; leave the reach at 0 or 1.")
		for gear: String in ContentFactory._strings(e["weapon_types"]):
			if not Proficiency.gear_types().has(gear):
				errors.append("Needs weapon type: unknown gear type '%s'." % gear)
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
		var has_start: bool = false
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
			if level == 1:
				has_start = true
			if skill.is_empty():
				errors.append("Skill path: unknown skill '%s'." % skill_id)
			elif DictUtil.to_bool(skill.get("system", false)):
				errors.append("Skill path: '%s' is an engine skill (always known)." % skill_id)
			elif DictUtil.to_int(skill.get("level", 1)) > StatRules.skill_cap_for_level(level):
				errors.append("Skill path: %s (skill level %d) is too advanced for unit level %d (cap %d)." % [skill_id, DictUtil.to_int(skill.get("level", 1)), level, StatRules.skill_cap_for_level(level)])
		if not has_start:
			errors.append("The skill path needs at least one level-1 skill.")
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
