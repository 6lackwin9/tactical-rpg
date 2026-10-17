class_name WorldCatalog
extends RefCounted

# ==============================================================================
# Tactical RPG - World Catalog (modular, inheritable lore worlds)
# Worlds live in res://data/worlds/<world_id>/: world.json (manifest) plus one
# <kind>.json per WorldSchema kind ({"kind", "version", "entries": [...]}).
#
# Inheritance
#   A world `extends` other worlds and sees all their entries. lineage(world) is
#   the load order: ancestors first (parents in `extends` order), the world last.
#   Entries are namespaced: the qualified id of "mara" in world "ashlands" is
#   "ashlands:mara". An unqualified ref resolves in the entry's own world first,
#   then up the lineage (nearest wins); "world:id" is explicit.
#   An entry with `override: "ashlands:mara"` replaces that ancestor entry for its
#   world and every world below it (refs to the old id resolve to the override).
#   `connects` links worlds without inheriting: qualified refs may cross into them.
#
# Entries are plain Dictionaries shaped by WorldSchema. Merged views add `_world`
# and `_qid`. State (loaded data, root, dirty flags) lives on the Engine singleton
# like ContentCatalog, so hot reloads can't reset it. Tests that write call
# fork_to("user://...") so the shipped worlds stay untouched.
# ==============================================================================

const DEFAULT_ROOT: String = "res://data/worlds"
const VERSION: int = 1
const STATE_KEY: StringName = &"world_catalog_state"
# Marker key -> the world kind it references (location map markers, EXP1-L).
const MARKER_REF_KEYS: Dictionary = {"encounter": "encounters", "character": "characters", "lore": "lore", "loot": "loot", "to": "locations"}
const GONE_FLAGS_NOT_LIST: String = "Gone flags must be a list of {\"key\", \"value\"} conditions."

# ------------------------------------------------------------------------------
# State & loading
# ------------------------------------------------------------------------------

static func _state() -> Dictionary:
	if not Engine.has_meta(STATE_KEY):
		var errors: Array[String] = []
		var removed: Array[String] = []
		Engine.set_meta(STATE_KEY, {"root": DEFAULT_ROOT, "worlds": {}, "dirty": {}, "errors": errors, "removed": removed, "loaded": false})
	var state: Dictionary = Engine.get_meta(STATE_KEY)
	return state

static func get_root() -> String:
	return str(_state()["root"])

static func _worlds() -> Dictionary:
	var worlds: Dictionary = _state()["worlds"]
	return worlds

static func _dirty() -> Dictionary:
	var dirty: Dictionary = _state()["dirty"]
	return dirty

static func _error_list() -> Array[String]:
	var errors: Array[String] = _state()["errors"]
	return errors

static func _removed() -> Array[String]:
	var removed: Array[String] = _state()["removed"]
	return removed

static func _ensure_loaded() -> void:
	if not DictUtil.to_bool(_state()["loaded"]):
		reload_from_disk()

static func use_root(p_root: String) -> void:
	_state()["root"] = p_root
	reload_from_disk()

static func path_for(world_id: String, kind: String = "", p_root: String = "") -> String:
	var base: String = "%s/%s" % [p_root if not p_root.is_empty() else get_root(), world_id]
	return "%s/%s.json" % [base, kind if not kind.is_empty() else "world"]

static func reload_from_disk() -> void:
	_worlds().clear()
	_dirty().clear()
	_error_list().clear()
	_removed().clear()
	_state()["loaded"] = true
	var dir: DirAccess = DirAccess.open(get_root())
	if dir == null:
		return
	var names: PackedStringArray = dir.get_directories()
	for dir_name: String in names:
		if dir_name.begins_with(".") or not FileAccess.file_exists(path_for(dir_name)):
			continue
		var header_raw: Variant = _read_json(path_for(dir_name))
		var header: Dictionary = header_raw if header_raw is Dictionary else {}
		header = WorldSchema.normalize("world", header)
		header["id"] = dir_name   # the folder is the namespace
		var kinds: Dictionary = {}
		for kind: String in WorldSchema.KINDS:
			kinds[kind] = _read_kind(dir_name, kind)
		_worlds()[dir_name] = {"manifest": header, "kinds": kinds}

static func _read_json(path: String) -> Variant:
	if not FileAccess.file_exists(path):
		return null
	var json := JSON.new()
	if json.parse(FileAccess.get_file_as_string(path)) != OK:
		_error_list().append("%s: JSON error on line %d: %s" % [path, json.get_error_line(), json.get_error_message()])
		return null
	return json.data

static func _read_kind(world_id: String, kind: String) -> Array:
	var found: Array = []
	var parsed: Variant = _read_json(path_for(world_id, kind))
	var raw: Variant = parsed
	if parsed is Dictionary:
		var doc: Dictionary = parsed
		raw = doc.get("entries", [])
	if raw is Array:
		var list: Array = raw
		for item: Variant in list:
			if item is Dictionary:
				var dict: Dictionary = item
				found.append(WorldSchema.normalize(kind, dict))
	return found

static func load_errors() -> Array[String]:
	_ensure_loaded()
	return _error_list().duplicate()

# Drops loaded data and the root override (call on shutdown with the other caches).
static func clear_cache() -> void:
	if Engine.has_meta(STATE_KEY):
		Engine.remove_meta(STATE_KEY)

# ------------------------------------------------------------------------------
# Worlds & lineage
# ------------------------------------------------------------------------------

static func world_ids() -> Array[String]:
	_ensure_loaded()
	var out: Array[String] = []
	for id: Variant in _worlds():
		out.append(str(id))
	out.sort()
	return out

static func has_world(world_id: String) -> bool:
	_ensure_loaded()
	return _worlds().has(world_id)

static func manifest(world_id: String) -> Dictionary:
	_ensure_loaded()
	var raw: Variant = _worlds().get(world_id, null)
	if raw is Dictionary:
		var w: Dictionary = raw
		var m: Dictionary = w["manifest"]
		return m.duplicate(true)
	return {}

static func _manifest_ref(world_id: String) -> Dictionary:
	var w: Dictionary = _worlds()[world_id]
	var m: Dictionary = w["manifest"]
	return m

static func _own_list(world_id: String, kind: String) -> Array:
	var raw: Variant = _worlds().get(world_id, null)
	if raw is Dictionary:
		var w: Dictionary = raw
		var kinds: Dictionary = w["kinds"]
		var list_raw: Variant = kinds.get(kind, [])
		if list_raw is Array:
			var list: Array = list_raw
			return list
	return []

static func _strings(v: Variant) -> Array[String]:
	var out: Array[String] = []
	if v is Array:
		var arr: Array = v
		for x: Variant in arr:
			out.append(str(x))
	return out

# Load order of `world_id`: ancestors first (parents in `extends` order), the world last.
static func lineage(world_id: String) -> Array[String]:
	_ensure_loaded()
	var out: Array[String] = []
	var stack: Array[String] = []
	_visit(world_id, out, stack)
	return out

static func _visit(world_id: String, out: Array[String], stack: Array[String]) -> void:
	if out.has(world_id) or stack.has(world_id) or not _worlds().has(world_id):
		return
	stack.append(world_id)
	for parent: String in _strings(_manifest_ref(world_id).get("extends", [])):
		_visit(parent, out, stack)
	stack.pop_back()
	out.append(world_id)

static func has_cycle(world_id: String) -> bool:
	return _cycle_from(world_id, [])

static func _cycle_from(world_id: String, path: Array[String]) -> bool:
	if path.has(world_id):
		return true
	if not _worlds().has(world_id):
		return false
	var next_path: Array[String] = path.duplicate()
	next_path.append(world_id)
	for parent: String in _strings(_manifest_ref(world_id).get("extends", [])):
		if _cycle_from(parent, next_path):
			return true
	return false

# Worlds that inherit from `world_id` (directly or not), excluding itself.
static func descendants(world_id: String) -> Array[String]:
	var out: Array[String] = []
	for other: String in world_ids():
		if other != world_id and lineage(other).has(world_id):
			out.append(other)
	return out

# ------------------------------------------------------------------------------
# Entries: own, merged, resolved
# ------------------------------------------------------------------------------

static func qid(world_id: String, local_id: String) -> String:
	return "%s:%s" % [world_id, local_id]

static func split_qid(ref: String) -> Array[String]:
	var parts: PackedStringArray = ref.split(":", false, 1)
	var out: Array[String] = []
	if parts.size() == 2:
		out.append(parts[0])
		out.append(parts[1])
	return out

# The world's OWN entries of `kind` (copies; file order).
static func own_entries(world_id: String, kind: String) -> Array[Dictionary]:
	_ensure_loaded()
	var out: Array[Dictionary] = []
	for e: Variant in _own_list(world_id, kind):
		if e is Dictionary:
			var d: Dictionary = e
			var copy: Dictionary = d.duplicate(true)
			copy["_world"] = world_id
			copy["_qid"] = qid(world_id, str(copy.get("id", "")))
			out.append(copy)
	return out

# What `world_id` sees of `kind`: ancestors' entries then its own, minus entries
# an override replaced. Each entry carries `_world` and `_qid`.
static func merged_entries(world_id: String, kind: String) -> Array[Dictionary]:
	_ensure_loaded()
	var all: Array[Dictionary] = []
	var overridden: Dictionary = {}
	for w: String in lineage(world_id):
		for e: Dictionary in own_entries(w, kind):
			all.append(e)
			var ov: String = str(e.get("override", ""))
			if not ov.is_empty():
				overridden[ov] = true
	var out: Array[Dictionary] = []
	for e: Dictionary in all:
		if not overridden.has(str(e["_qid"])):
			out.append(e)
	return out

# override target (qualified) -> the entry replacing it, as seen from `world_id`.
static func _redirects(world_id: String, kind: String) -> Dictionary:
	var out: Dictionary = {}
	for w: String in lineage(world_id):
		for e: Dictionary in own_entries(w, kind):
			var ov: String = str(e.get("override", ""))
			if not ov.is_empty():
				out[ov] = e
	return out

# The entry `ref` names as seen from `world_id` ({} if none). Unqualified refs
# resolve in the world, then up the lineage; overrides are followed.
static func resolve(world_id: String, kind: String, ref: String) -> Dictionary:
	_ensure_loaded()
	if ref.is_empty():
		return {}
	var chain: Array[String] = lineage(world_id)
	var target_world: String = ""
	var local: String = ref
	if ref.contains(":"):
		var parts: Array[String] = split_qid(ref)
		if parts.size() != 2:
			return {}
		target_world = parts[0]
		local = parts[1]
	var found: Dictionary = {}
	if target_world.is_empty():
		var rev: Array[String] = chain.duplicate()
		rev.reverse()
		for w: String in rev:
			found = _own_by_id(w, kind, local)
			if not found.is_empty():
				break
	else:
		found = _own_by_id(target_world, kind, local)
	if found.is_empty():
		return {}
	if not chain.has(str(found["_world"])):
		return found   # a connected world's entry: no redirects from here
	var redirects: Dictionary = _redirects(world_id, kind)
	var guard: int = 0
	while redirects.has(str(found["_qid"])) and guard < 16:
		var next_raw: Variant = redirects[str(found["_qid"])]
		if not (next_raw is Dictionary):
			break
		found = next_raw
		guard += 1
	return found

static func _own_by_id(world_id: String, kind: String, local: String) -> Dictionary:
	for e: Dictionary in own_entries(world_id, kind):
		if str(e.get("id", "")) == local:
			return e
	return {}

# `entry` as a ref value written in `world_id` (local id if it lives there, else qualified).
static func ref_value(world_id: String, entry: Dictionary) -> String:
	return str(entry.get("id", "")) if str(entry.get("_world", "")) == world_id else str(entry.get("_qid", ""))

static func has_entry(world_id: String, kind: String, ref: String) -> bool:
	return not resolve(world_id, kind, ref).is_empty()

# A copy of the world's OWN entry (empty if it only exists in an ancestor).
static func get_own(world_id: String, kind: String, local: String) -> Dictionary:
	var e: Dictionary = _own_by_id(world_id, kind, local)
	e.erase("_world")
	e.erase("_qid")
	return e

static func own_ids(world_id: String, kind: String) -> Array[String]:
	var out: Array[String] = []
	for e: Dictionary in own_entries(world_id, kind):
		out.append(str(e.get("id", "")))
	return out

static func entry_count(world_id: String) -> int:
	var n: int = 0
	for kind: String in WorldSchema.KINDS:
		n += _own_list(world_id, kind).size()
	return n

# ------------------------------------------------------------------------------
# Editing worlds
# ------------------------------------------------------------------------------

static func _touch(world_id: String, kind: String) -> void:
	var dirty: Dictionary = _dirty()
	var per_world: Dictionary = {}
	var raw: Variant = dirty.get(world_id, null)
	if raw is Dictionary:
		per_world = raw
	per_world[kind] = true
	dirty[world_id] = per_world

static func is_dirty(world_id: String = "") -> bool:
	if world_id.is_empty():
		return not _dirty().is_empty() or not _removed().is_empty()
	return _dirty().has(world_id)

# Creates an empty world; returns the problems (empty = created).
static func create_world(world_id: String, p_name: String, extends_from: Array[String] = []) -> Array[String]:
	_ensure_loaded()
	var errors: Array[String] = []
	if not ContentSchema.is_valid_id(world_id):
		errors.append("World id '%s' must be lowercase snake_case." % world_id)
	elif _worlds().has(world_id):
		errors.append("World '%s' already exists." % world_id)
	for parent: String in extends_from:
		if not _worlds().has(parent):
			errors.append("Parent world '%s' doesn't exist." % parent)
	if not errors.is_empty():
		return errors
	var m: Dictionary = WorldSchema.normalize("world", {"id": world_id, "name": p_name, "extends": extends_from})
	var kinds: Dictionary = {}
	for kind: String in WorldSchema.KINDS:
		kinds[kind] = []
	_worlds()[world_id] = {"manifest": m, "kinds": kinds}
	_touch(world_id, "world")
	return errors

# Stores a changed manifest (name, description, extends, connects); the id never changes.
static func update_manifest(world_id: String, changes: Dictionary) -> Array[String]:
	_ensure_loaded()
	var errors: Array[String] = []
	if not _worlds().has(world_id):
		errors.append("No world '%s'." % world_id)
		return errors
	var updated: Dictionary = manifest(world_id)
	updated.merge(changes, true)
	updated["id"] = world_id
	updated = WorldSchema.normalize("world", updated)
	for key: String in ["extends", "connects"]:
		for other: String in _strings(updated[key]):
			if other == world_id:
				errors.append("A world can't %s itself." % ("extend" if key == "extends" else "connect to"))
			elif not _worlds().has(other):
				errors.append("%s: unknown world '%s'." % [key.capitalize(), other])
	if errors.is_empty():
		var previous: Dictionary = manifest(world_id)
		_worlds()[world_id]["manifest"] = updated
		if has_cycle(world_id):
			_worlds()[world_id]["manifest"] = previous
			errors.append("That would make the inheritance circular.")
			return errors
		_touch(world_id, "world")
	return errors

# Removes a world (refused while another world extends or connects to it).
static func remove_world(world_id: String) -> String:
	_ensure_loaded()
	if not _worlds().has(world_id):
		return "No world '%s'." % world_id
	for other: String in world_ids():
		var m: Dictionary = manifest(other)
		if other != world_id and (_strings(m.get("extends", [])).has(world_id) or _strings(m.get("connects", [])).has(world_id)):
			return "'%s' is still extended or connected by world '%s'." % [world_id, other]
	_worlds().erase(world_id)
	_dirty().erase(world_id)
	_removed().append(world_id)
	return ""

# ------------------------------------------------------------------------------
# Editing entries
# ------------------------------------------------------------------------------

# Validates (blocking_errors: unresolved references are allowed while authoring, the
# worlds' validate_* report them) and stores `entry` in the world's OWN entries
# (replacing `replace_id`, or the entry with the same id, in place). Renaming an id rewrites every reference to
# it, in this world and all worlds below it. Returns the problems (nothing changes if any).
static func upsert(world_id: String, kind: String, entry: Dictionary, replace_id: String = "") -> Array[String]:
	_ensure_loaded()
	var clean: Dictionary = WorldSchema.normalize(kind, entry)
	var new_id: String = str(clean.get("id", ""))
	var old_id: String = replace_id if not replace_id.is_empty() else new_id
	var errors: Array[String] = blocking_errors(world_id, kind, clean, old_id)
	if not errors.is_empty():
		return errors
	var list: Array = _own_list(world_id, kind)
	var at: int = -1
	for i: int in range(list.size()):
		var item: Dictionary = list[i]
		if str(item.get("id", "")) == old_id:
			at = i
			break
	var rename_matches: Array[Dictionary] = []
	if old_id != new_id and at >= 0:
		rename_matches = references_to(world_id, kind, old_id)
	if at >= 0:
		list[at] = clean
	else:
		list.append(clean)
	_touch(world_id, kind)
	if not rename_matches.is_empty():
		var target: Dictionary = {"id": new_id, "_world": world_id, "_qid": qid(world_id, new_id)}
		for m: Dictionary in rename_matches:
			_rewrite_ref(m, target, qid(world_id, old_id))
	return errors

# Removes an own entry unless something still references it. Returns "" or why not.
static func remove(world_id: String, kind: String, local: String) -> String:
	_ensure_loaded()
	var list: Array = _own_list(world_id, kind)
	for i: int in range(list.size()):
		var item: Dictionary = list[i]
		if str(item.get("id", "")) == local:
			var users: Array[Dictionary] = references_to(world_id, kind, local)
			if not users.is_empty():
				var labels: Array[String] = []
				for u: Dictionary in users:
					if str(u["kind"]) == "world":
						labels.append("world %s %s" % [str(u["world"]), str(u["key"])])
					else:
						labels.append("%s %s:%s" % [str(u["kind"]), str(u["world"]), str(u["id"])])
				return "'%s' is still used by: %s." % [local, ", ".join(labels)]
			list.remove_at(i)
			_touch(world_id, kind)
			return ""
	return "World '%s' has no %s entry '%s'." % [world_id, kind, local]

static func unique_id(world_id: String, kind: String, base: String) -> String:
	var stem: String = base if ContentSchema.is_valid_id(base) else "new_entry"
	var taken: Array[String] = own_ids(world_id, kind)
	if not taken.has(stem):
		return stem
	var n: int = 2
	while taken.has("%s_%d" % [stem, n]):
		n += 1
	return "%s_%d" % [stem, n]

# Copies one of the world's own entries under a fresh id (a template); returns the new id.
static func duplicate_entry(world_id: String, kind: String, local: String) -> String:
	var src: Dictionary = get_own(world_id, kind, local)
	if src.is_empty():
		return ""
	src["id"] = unique_id(world_id, kind, local + "_copy")
	src["name"] = str(src.get("name", local)) + " (Copy)"
	src.erase("override")
	return str(src["id"]) if upsert(world_id, kind, src).is_empty() else ""

# Copies an ANCESTOR's entry into this world as an override (it replaces the original
# for this world and below). Returns the new local id ("" if it can't).
static func override_entry(world_id: String, kind: String, source_qid: String) -> String:
	var src: Dictionary = resolve(world_id, kind, source_qid)
	if src.is_empty() or str(src["_world"]) == world_id:
		return ""
	var copy: Dictionary = src.duplicate(true)
	copy.erase("_world")
	var original_qid: String = str(copy.get("_qid", source_qid))
	copy.erase("_qid")
	copy["override"] = original_qid
	var local: String = str(copy.get("id", ""))
	copy["id"] = local if not own_ids(world_id, kind).has(local) else unique_id(world_id, kind, local)
	return str(copy["id"]) if upsert(world_id, kind, copy).is_empty() else ""

# ------------------------------------------------------------------------------
# References
# ------------------------------------------------------------------------------

# Every reference to the entry `kind`/`local` of `world_id`, from the world and the
# worlds below it: [{"world", "kind", "id", "key", "index"}] (index -1 = single value;
# key "override" is an override pointer).
static func references_to(world_id: String, kind: String, local: String) -> Array[Dictionary]:
	_ensure_loaded()
	var target: String = qid(world_id, local)
	var out: Array[Dictionary] = []
	for w: String in dependents(world_id):
		if kind == "locations" and _points_at(w, kind, str(manifest(w).get("start_location", "")), target):
			out.append({"world": w, "kind": "world", "id": w, "key": "start_location", "index": -1})   # the manifest (EXP1-L)
		for k2: String in WorldSchema.KINDS:
			for e: Dictionary in own_entries(w, k2):
				if k2 == kind and w == world_id and str(e.get("id", "")) == local:
					continue   # not a reference to itself
				if k2 == kind and str(e.get("override", "")) == target:
					out.append({"world": w, "kind": k2, "id": str(e.get("id", "")), "key": "override", "index": -1})
				for rf: Dictionary in WorldSchema.ref_fields(k2):
					if str(rf["ref"]) != kind:
						continue
					var key: String = str(rf["key"])
					var value: Variant = e.get(key, null)
					if value is Array:
						var arr: Array = value
						for i: int in range(arr.size()):
							if _points_at(w, kind, str(arr[i]), target):
								out.append({"world": w, "kind": k2, "id": str(e.get("id", "")), "key": key, "index": i})
					elif value is String and _points_at(w, kind, str(value), target):
						out.append({"world": w, "kind": k2, "id": str(e.get("id", "")), "key": key, "index": -1})
				out.append_array(_json_refs_to(w, k2, e, kind, target))
	return out

# References held inside json fields: location exits, quest steps, faction relations, route encounters, and the quest runtime fields
# (links, loot_pool, variant `set` values). The quest ones carry a "path" (keys/indexes from the entry root) that
# _rewrite_ref follows.
static func _json_refs_to(w: String, k2: String, e: Dictionary, kind: String, target: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var base: Dictionary = {"world": w, "kind": k2, "id": str(e.get("id", ""))}
	if k2 == "quests":
		out.append_array(_quest_json_refs(w, e, kind, target, base))
	if k2 == "locations" and kind == "locations" and e.get("exits", null) is Array:
		var exits: Array = e["exits"]
		for i: int in range(exits.size()):
			var ex: Dictionary = exits[i] if exits[i] is Dictionary else {}
			if _points_at(w, "locations", str(ex.get("to", "")), target):
				out.append(base.merged({"key": "exits", "index": i, "sub": "to"}))
	if k2 == "locations":
		out.append_array(_map_json_refs(w, e, kind, target, base))
	if k2 == "dialogues":
		out.append_array(_dialogue_json_refs(w, e, kind, target, base))
	if k2 == "shops" and kind == "loot":
		var for_sale: Array = DictUtil.array_of(e.get("loot", null))
		for i: int in range(for_sale.size()):
			_ref_at(w, DictUtil.dict_of(for_sale[i]), "loot", ["loot", i], kind, target, base, out)
	if k2 == "routes" and kind == "encounters":
		out.append_array(_route_json_refs(w, e, kind, target, base))
	if k2 == "quests" and e.get("steps", null) is Array and (kind == "locations" or kind == "encounters"):
		var steps: Array = e["steps"]
		var sub: String = "location" if kind == "locations" else "encounter"
		for i: int in range(steps.size()):
			var step: Dictionary = steps[i] if steps[i] is Dictionary else {}
			if _points_at(w, kind, str(step.get(sub, "")), target):
				out.append(base.merged({"key": "steps", "index": i, "sub": sub}))
	if k2 == "factions" and kind == "factions" and e.get("relations", null) is Dictionary:
		var rel: Dictionary = e["relations"]
		for other: Variant in rel:
			if _points_at(w, "factions", str(other), target):
				out.append(base.merged({"key": "relations", "index": -1, "sub": str(other)}))
	return out

# Quest links, loot pool and variant set refs to `kind`/`target` (path-addressed matches), and the faction keys of
# `reputation` (REP1-L; keyed matches renamed like relation keys).
static func _quest_json_refs(w: String, e: Dictionary, kind: String, target: String, base: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	_links_refs(w, DictUtil.dict_of(e.get("links", null)), ["links"], kind, target, base, out)
	if kind == "factions":   # REP1-L: reputation keys are faction refs (renamed like relation keys)
		var rep: Dictionary = DictUtil.dict_of(e.get("reputation", null))
		for ref: Variant in rep:
			if _points_at(w, "factions", str(ref), target):
				out.append(base.merged({"key": "reputation", "index": -1, "sub": str(ref)}))
	if kind == "loot":
		_list_refs(w, e.get("loot_pool", null), ["loot_pool"], "loot", target, base, out)
	var variants: Array = DictUtil.array_of(e.get("variants", null))
	for vi: int in range(variants.size()):
		var set_d: Dictionary = DictUtil.dict_of(DictUtil.dict_of(variants[vi]).get("set", null))
		var at: Array = ["variants", vi, "set"]
		for key: String in ["giver", "turn_in"]:
			if kind == "characters" and set_d.get(key, null) is String and _points_at(w, kind, str(set_d[key]), target):
				out.append(base.merged({"key": "variants", "index": vi, "path": at + [key], "value": str(set_d[key])}))
		_links_refs(w, DictUtil.dict_of(set_d.get("links", null)), at + ["links"], kind, target, base, out)
		if kind == "loot":
			_list_refs(w, set_d.get("loot_pool", null), at + ["loot_pool"], "loot", target, base, out)
		var steps: Dictionary = DictUtil.dict_of(set_d.get("steps", null))
		var sub: String = "location" if kind == "locations" else ("encounter" if kind == "encounters" else "")
		if sub.is_empty():
			continue
		for idx: Variant in steps:
			var partial: Dictionary = DictUtil.dict_of(steps[idx])
			if partial.get(sub, null) is String and _points_at(w, kind, str(partial[sub]), target):
				out.append(base.merged({"key": "variants", "index": vi, "path": at + ["steps", idx, sub], "value": str(partial[sub])}))
	return out

# Location map refs (EXP1-L): encounter refs in zone bindings ([ref, weight] pairs) and markers, marker character,
# lore and loot, and exit marker `to` (except world_map). Path-addressed like the quest json refs.
static func _map_json_refs(w: String, e: Dictionary, kind: String, target: String, base: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if kind == "encounters":
		var bindings: Dictionary = DictUtil.dict_of(e.get("zone_bindings", null))
		for zone: Variant in bindings:
			var b: Dictionary = DictUtil.dict_of(bindings[zone])
			for list_key: String in ["encounters", "elites"]:
				var pairs: Array = DictUtil.array_of(b.get(list_key, null))
				for i: int in range(pairs.size()):
					var pair: Array = DictUtil.array_of(pairs[i])
					if not pair.is_empty() and pair[0] is String and _points_at(w, kind, str(pair[0]), target):
						out.append(base.merged({"key": "zone_bindings", "index": -1, "path": ["zone_bindings", zone, list_key, i, 0], "value": str(pair[0])}))
	var markers: Array = DictUtil.array_of(e.get("markers", null))
	for i: int in range(markers.size()):
		var m: Dictionary = DictUtil.dict_of(markers[i])
		for key: Variant in MARKER_REF_KEYS:
			if str(MARKER_REF_KEYS[key]) != kind or not (m.get(key, null) is String):
				continue
			var ref: String = str(m[key])
			if key == "to" and ref == MapRules.WORLD_MAP:
				continue
			if _points_at(w, kind, ref, target):
				out.append(base.merged({"key": "markers", "index": i, "path": ["markers", i, key], "value": ref}))
	return out

# Route encounter refs (TRAV1-L): [ref, weight] pairs in `encounters` / `elites` and each `scripted` entry's encounter.
static func _route_json_refs(w: String, e: Dictionary, kind: String, target: String, base: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for list_key: String in ["encounters", "elites"]:
		var pairs: Array = DictUtil.array_of(e.get(list_key, null))
		for i: int in range(pairs.size()):
			var pair: Array = DictUtil.array_of(pairs[i])
			if not pair.is_empty() and pair[0] is String and _points_at(w, kind, str(pair[0]), target):
				out.append(base.merged({"key": list_key, "index": i, "path": [list_key, i, 0], "value": str(pair[0])}))
	var scripted: Array = DictUtil.array_of(e.get("scripted", null))
	for i: int in range(scripted.size()):
		_ref_at(w, DictUtil.dict_of(scripted[i]), "encounter", ["scripted", i], kind, target, base, out)
	return out

# Dialogue tree refs (DLG1-L): node speakers, quest refs of choice conditions, and the quest / shop / encounter refs of
# node and choice effects. Path-addressed like the quest json refs.
static func _dialogue_json_refs(w: String, e: Dictionary, kind: String, target: String, base: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var nodes: Array = DictUtil.array_of(e.get("nodes", null))
	for i: int in range(nodes.size()):
		var node: Dictionary = DictUtil.dict_of(nodes[i])
		var at: Array = ["nodes", i]
		if kind == "characters":
			_ref_at(w, node, "speaker", at, kind, target, base, out)
		_effect_refs(w, node.get("effects", null), at + ["effects"], kind, target, base, out)
		var choices: Array = DictUtil.array_of(node.get("choices", null))
		for j: int in range(choices.size()):
			var c: Dictionary = DictUtil.dict_of(choices[j])
			var c_at: Array = at + ["choices", j]
			for list_key: String in ["requires", "unless"]:
				var conds: Array = DictUtil.array_of(c.get(list_key, null))
				for k: int in range(conds.size()):
					var cond: Dictionary = DictUtil.dict_of(conds[k])
					for key: Variant in DialogueRules.CONDITION_REF_KEYS:
						if str(DialogueRules.CONDITION_REF_KEYS[key]) == kind and str(cond.get("kind", "flag")) == str(key):
							_ref_at(w, cond, str(key), c_at + [list_key, k], kind, target, base, out)
			_effect_refs(w, c.get("effects", null), c_at + ["effects"], kind, target, base, out)
	return out

static func _effect_refs(w: String, raw: Variant, at: Array, kind: String, target: String, base: Dictionary, out: Array[Dictionary]) -> void:
	var effects: Array = DictUtil.array_of(raw)
	for k: int in range(effects.size()):
		var fx: Dictionary = DictUtil.dict_of(effects[k])
		for key: Variant in DialogueRules.EFFECT_REF_KEYS:
			if str(DialogueRules.EFFECT_REF_KEYS[key]) == kind:
				_ref_at(w, fx, str(key), at + [k], kind, target, base, out)

# One path-addressed match when `holder[key]` is a String ref to `target` (`at` = the holder's path from the entry root).
static func _ref_at(w: String, holder: Dictionary, key: String, at: Array, kind: String, target: String, base: Dictionary, out: Array[Dictionary]) -> void:
	if holder.get(key, null) is String and _points_at(w, kind, str(holder[key]), target):
		out.append(base.merged({"key": str(at[0]), "index": DictUtil.to_int(at[1], -1) if at.size() > 1 else -1, "path": at + [key], "value": str(holder[key])}))

static func _links_refs(w: String, links: Dictionary, at: Array, kind: String, target: String, base: Dictionary, out: Array[Dictionary]) -> void:
	for key: Variant in links:
		if str(QuestRules.LINK_KINDS.get(str(key), "")) == kind:
			_list_refs(w, links[key], at + [key], kind, target, base, out)

static func _list_refs(w: String, raw: Variant, at: Array, kind: String, target: String, base: Dictionary, out: Array[Dictionary]) -> void:
	var list: Array = DictUtil.array_of(raw)
	for i: int in range(list.size()):
		if list[i] is String and _points_at(w, kind, str(list[i]), target):
			out.append(base.merged({"key": str(at[0]), "index": i, "path": at + [i], "value": str(list[i])}))

static func _points_at(world_id: String, kind: String, ref: String, target_qid: String) -> bool:
	if ref.is_empty():
		return false
	var hit: Dictionary = resolve(world_id, kind, ref)
	return not hit.is_empty() and str(hit.get("_qid", "")) == target_qid or (ref == target_qid)

static func _rewrite_ref(match_info: Dictionary, new_target: Dictionary, old_qid: String) -> void:
	var w: String = str(match_info["world"])
	if str(match_info["kind"]) == "world":   # a manifest ref (start_location)
		# Written in place: update_manifest re-validates extends / connects and could refuse the rewrite for an
		# unrelated manifest problem, leaving a stale ref behind a successful rename; this field is a plain ref.
		if _worlds().has(w):
			_manifest_ref(w)[str(match_info["key"])] = ref_value(w, new_target)
			_touch(w, "world")
		return
	var list: Array = _own_list(w, str(match_info["kind"]))
	for i: int in range(list.size()):
		var e: Dictionary = list[i]
		if str(e.get("id", "")) != str(match_info["id"]):
			continue
		var key: String = str(match_info["key"])
		var replacement: String = old_qid
		if key == "override":
			replacement = str(new_target["_qid"])
		else:
			replacement = ref_value(w, new_target)
		var idx: int = DictUtil.to_int(match_info["index"], -1)
		var sub: String = str(match_info.get("sub", ""))
		if match_info.has("path"):
			_set_at_path(e, DictUtil.array_of(match_info["path"]), str(match_info.get("value", "")), replacement)
		elif key == "relations" or key == "reputation":   # a dictionary keyed by faction refs
			var rel: Dictionary = e[key]
			if rel.has(sub):
				var held: Variant = rel[sub]
				rel.erase(sub)
				rel[replacement] = held
		elif not sub.is_empty():
			var items: Array = e[key]
			var item: Dictionary = items[idx]
			item[sub] = replacement
		elif idx >= 0:
			var arr: Array = e[key]
			arr[idx] = replacement
		else:
			e[key] = replacement
		_touch(w, str(match_info["kind"]))

# Walks `path` (Dictionary keys / Array indexes) from `root` and sets its last step to `value`, only while it still
# holds `expected` (the ref the scan found), so a stale match never overwrites another ref.
static func _set_at_path(root: Dictionary, path: Array, expected: String, value: String) -> void:
	var node: Variant = root
	for i: int in range(path.size()):
		var step: Variant = path[i]
		var last: bool = i == path.size() - 1
		if node is Dictionary:
			var d: Dictionary = node
			if last:
				if d.get(step, null) is String and str(d[step]) == expected:
					d[step] = value
			else:
				node = d.get(step, null)
		elif node is Array:
			var a: Array = node
			var idx: int = DictUtil.to_int(step, -1)
			if idx < 0 or idx >= a.size():
				return
			if last:
				if a[idx] is String and str(a[idx]) == expected:
					a[idx] = value
			else:
				node = a[idx]
		else:
			return

# ------------------------------------------------------------------------------
# Validation
# ------------------------------------------------------------------------------

# Errors that stop an entry from being stored: id, name, enums, override validity and
# the shape of json fields. Unresolved references and incomplete entries are NOT
# blocking (worlds are full of mutual references), they show up in validate_entry.
static func blocking_errors(world_id: String, kind: String, entry: Dictionary, current_id: String = "") -> Array[String]:
	_ensure_loaded()
	var e: Dictionary = WorldSchema.normalize(kind, entry)
	var errors: Array[String] = []
	var id: String = str(e.get("id", ""))
	if id.is_empty():
		errors.append("ID is required.")
	elif not ContentSchema.is_valid_id(id):
		errors.append("ID '%s' must be lowercase snake_case (a-z, 0-9, _)." % id)
	elif id != current_id and own_ids(world_id, kind).has(id):
		errors.append("ID '%s' is already used by another %s entry in this world." % [id, kind.trim_suffix("s")])
	if str(e.get("name", "")).strip_edges().is_empty():
		errors.append("Name is required.")
	for f: Dictionary in WorldSchema.fields(kind):
		if str(f.get("type", "")) == "enum":
			var value: String = str(e.get(str(f["key"]), ""))
			if not WorldSchema.options_of(f).has(value):
				errors.append("%s: '%s' is not one of %s." % [str(f["label"]), value, ", ".join(WorldSchema.options_of(f))])
	errors.append_array(_override_errors(world_id, kind, e))
	errors.append_array(_shape_errors(kind, e))
	return errors

static func _shape_errors(kind: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	match kind:
		"characters":
			errors.append_array(_tuning_errors(e["tuning"], WorldSchema.TUNING_KEYS, "Character"))
			if not (e["gone_flags"] is Array):
				errors.append(GONE_FLAGS_NOT_LIST)
		"items":
			errors.append_array(_tuning_errors(e["tuning"], WorldSchema.ITEM_TUNING_KEYS, "Item"))
		"dialogues":
			if not (e["nodes"] is Array):
				errors.append("Nodes must be a list of node objects.")
		"shops":
			for key: String in ["stock", "loot", "buys", "requires_flags"]:
				if not (e[key] is Array):
					errors.append("%s must be a list." % str(WorldSchema.field("shops", key)["label"]))
		"locations":
			if not (e["exits"] is Array):
				errors.append("Exits must be a list of {\"to\", \"label\"} objects.")
		"routes":
			errors.append_array(TravelRules.shape_errors(e))
		"factions":
			if not (e["relations"] is Dictionary):
				errors.append("Relations must be an object {faction: stance}.")
			else:
				for other: Variant in DictUtil.dict_of(e["relations"]):
					if not ["ally", "rival", "enemy", "neutral"].has(str(DictUtil.dict_of(e["relations"])[other])):
						errors.append("Relation stance '%s' must be ally, rival, enemy or neutral." % str(DictUtil.dict_of(e["relations"])[other]))
		"quests":
			if not (e["steps"] is Array):
				errors.append("Steps must be a list.")
			for key: String in ["requires_flags", "complete_flags", "variants", "loot_pool"]:
				if not (e[key] is Array):
					errors.append("%s must be a list." % str(WorldSchema.field("quests", key)["label"]))
			if not (e["links"] is Dictionary):
				errors.append("Links must be an object.")
			if not (e["reputation"] is Dictionary):
				errors.append("Reputation must be an object {faction: delta}.")
		"loot":
			if not (e["requires_flags"] is Array):
				errors.append("Requires flags must be a list.")
		"encounters":
			if DictUtil.array_of(e["party"]).size() > 4:
				errors.append("The party holds at most 4 members.")
			if not (e["exits"] is Array):
				errors.append("Exit tiles must be a list of [x, y] pairs.")
			else:
				for x: Variant in DictUtil.array_of(e["exits"]):
					var pair: Array = x if x is Array else []
					if pair.size() != 2 or DictUtil.to_int(pair[0]) < 0 or DictUtil.to_int(pair[1]) < 0 or DictUtil.to_int(pair[0]) >= CombatGrid.GRID_W or DictUtil.to_int(pair[1]) >= CombatGrid.GRID_H:
						errors.append("Exit tiles are [x, y] pairs on the 12x9 grid.")
						break
	return errors

# EVERY problem with `entry` as a `kind` entry of `world_id`: blocking errors plus
# unresolved references and incomplete content (current_id: the id it replaces).
static func validate_entry(world_id: String, kind: String, entry: Dictionary, current_id: String = "") -> Array[String]:
	_ensure_loaded()
	var e: Dictionary = WorldSchema.normalize(kind, entry)
	var errors: Array[String] = []
	var id: String = str(e.get("id", ""))
	if id.is_empty():
		errors.append("ID is required.")
	elif not ContentSchema.is_valid_id(id):
		errors.append("ID '%s' must be lowercase snake_case (a-z, 0-9, _)." % id)
	elif id != current_id and own_ids(world_id, kind).has(id):
		errors.append("ID '%s' is already used by another %s entry in this world." % [id, kind.trim_suffix("s")])
	if str(e.get("name", "")).strip_edges().is_empty():
		errors.append("Name is required.")
	for f: Dictionary in WorldSchema.fields(kind):
		if str(f.get("type", "")) == "enum":
			var value: String = str(e.get(str(f["key"]), ""))
			if not WorldSchema.options_of(f).has(value):
				errors.append("%s: '%s' is not one of %s." % [str(f["label"]), value, ", ".join(WorldSchema.options_of(f))])
	errors.append_array(_ref_errors(world_id, kind, e))
	errors.append_array(_override_errors(world_id, kind, e))
	match kind:
		"regions":
			errors.append_array(_region_errors(world_id, e, id))
		"locations":
			errors.append_array(_location_errors(world_id, e))
		"factions":
			errors.append_array(_faction_errors(world_id, e))
		"characters":
			errors.append_array(_character_errors(world_id, e))
		"items":
			errors.append_array(_item_errors(e))
		"quests":
			errors.append_array(_quest_errors(world_id, e))
		"encounters":
			errors.append_array(_encounter_errors(world_id, e))
		"loot":
			errors.append_array(_loot_errors(e))
		"dialogues":
			errors.append_array(DialogueRules.tree_errors(world_id, e))
		"shops":
			errors.append_array(ShopRules.shop_errors(world_id, e))
		"routes":
			errors.append_array(TravelRules.route_errors(world_id, e))
	return errors

# Is a qualified ref's world visible from `world_id` (itself, an ancestor, or connected)?
static func can_reach_world(world_id: String, other: String) -> bool:
	var chain: Array[String] = lineage(world_id)
	if chain.has(other):
		return true
	for w: String in chain:   # connections are inherited: a child may link wherever its ancestors do
		if _strings(manifest(w).get("connects", [])).has(other):
			return true
	return false

# Worlds whose entries can refer to `world_id`'s: itself, everything below it, and
# every world that reaches it through a connection.
static func dependents(world_id: String) -> Array[String]:
	var out: Array[String] = [world_id]
	for other: String in world_ids():
		if other != world_id and can_reach_world(other, world_id):
			out.append(other)
	return out

static func _ref_errors(world_id: String, kind: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	for rf: Dictionary in WorldSchema.ref_fields(kind):
		var f: Dictionary = rf["field"]
		var ref_kind: String = str(rf["ref"])
		var values: Array[String] = []
		var raw: Variant = e.get(str(rf["key"]), null)
		if raw is Array:
			values = _strings(raw)
			if not DictUtil.to_bool(f.get("dupes", false)):
				var seen: Dictionary = {}
				for v: String in values:
					if seen.has(v):
						errors.append("%s lists '%s' twice." % [str(f["label"]), v])
					seen[v] = true
		elif not str(raw).is_empty():
			values.append(str(raw))
		for v: String in values:
			errors.append_array(_check_ref(world_id, f, ref_kind, v))
	return errors

# Problems with `v` as a ref to `ref_kind` written in `world_id` (refs inside json fields: quest links, loot pools...).
static func ref_errors(world_id: String, label: String, ref_kind: String, v: String) -> Array[String]:
	return _check_ref(world_id, {"label": label}, ref_kind, v)

static func _check_ref(world_id: String, f: Dictionary, ref_kind: String, v: String) -> Array[String]:
	var errors: Array[String] = []
	var label: String = str(f["label"])
	if ref_kind.begins_with("base:"):
		var base_kind: String = ref_kind.trim_prefix("base:")
		var target: Dictionary = ContentCatalog.get_entry(base_kind, v)
		if target.is_empty():
			errors.append("%s: unknown base %s '%s'." % [label, base_kind.trim_suffix("s"), v])
		elif not ContentSchema.passes_filter(f, target):
			errors.append("%s: '%s' can't be used here." % [label, v])
		return errors
	if ref_kind == "worlds":
		if not _worlds().has(v):
			errors.append("%s: unknown world '%s'." % [label, v])
		return errors
	if v.contains(":"):
		var parts: Array[String] = split_qid(v)
		if parts.size() != 2 or not _worlds().has(parts[0]):
			errors.append("%s: unknown world in '%s'." % [label, v])
			return errors
		if not can_reach_world(world_id, parts[0]):
			errors.append("%s: world '%s' is neither an ancestor nor connected (add it to Extends or Connects)." % [label, parts[0]])
			return errors
	var hit: Dictionary = resolve(world_id, ref_kind, v)
	if hit.is_empty():
		errors.append("%s: unknown %s '%s'." % [label, ref_kind.trim_suffix("s"), v])
	elif not ContentSchema.passes_filter(f, hit):
		errors.append("%s: '%s' can't be used here." % [label, v])
	return errors

static func _override_errors(world_id: String, kind: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var ov: String = str(e.get("override", ""))
	if ov.is_empty():
		return errors
	var parts: Array[String] = split_qid(ov)
	if parts.size() != 2:
		errors.append("Overrides must be a qualified id (world:id).")
	elif parts[0] == world_id:
		errors.append("An entry can only override an entry of an ancestor world.")
	elif not lineage(world_id).has(parts[0]):
		errors.append("Overrides: '%s' is not an ancestor world." % parts[0])
	elif _own_by_id(parts[0], kind, parts[1]).is_empty():
		errors.append("Overrides: no %s '%s' in world '%s'." % [kind.trim_suffix("s"), parts[1], parts[0]])
	return errors

static func _region_errors(world_id: String, e: Dictionary, id: String) -> Array[String]:
	var errors: Array[String] = []
	var seen: Array[String] = [id]
	var cursor: String = str(e.get("parent", ""))
	while not cursor.is_empty():
		var hit: Dictionary = resolve(world_id, "regions", cursor)
		if hit.is_empty():
			break
		var hit_id: String = str(hit.get("id", ""))
		if seen.has(hit_id):
			errors.append("The region hierarchy is circular.")
			break
		seen.append(hit_id)
		cursor = str(hit.get("parent", ""))
	return errors

static func _location_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var exits: Variant = e["exits"]
	if not (exits is Array):
		errors.append("Exits must be a list of {\"to\", \"label\"} objects.")
		return errors
	var list: Array = exits
	for x: Variant in list:
		var ex: Dictionary = x if x is Dictionary else {}
		var to: String = str(ex.get("to", ""))
		if to.is_empty():
			errors.append("Every exit needs a \"to\" location.")
			continue
		if to.contains(":"):
			var parts: Array[String] = split_qid(to)
			if parts.size() == 2 and _worlds().has(parts[0]) and not can_reach_world(world_id, parts[0]):
				errors.append("Exit '%s': world '%s' is neither an ancestor nor connected." % [to, parts[0]])
				continue
		if resolve(world_id, "locations", to).is_empty():
			errors.append("Exit leads to unknown location '%s'." % to)
	errors.append_array(MapRules.location_errors(world_id, e))
	return errors

static func _faction_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var relations: Variant = e["relations"]
	if not (relations is Dictionary):
		errors.append("Relations must be an object {faction: stance}.")
		return errors
	var rel: Dictionary = relations
	for other: Variant in rel:
		if resolve(world_id, "factions", str(other)).is_empty():
			errors.append("Relation to unknown faction '%s'." % str(other))
		if not ["ally", "rival", "enemy", "neutral"].has(str(rel[other])):
			errors.append("Relation stance '%s' must be ally, rival, enemy or neutral." % str(rel[other]))
	return errors

static func _tuning_errors(raw: Variant, allowed: Array[String], what: String) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Dictionary):
		errors.append("%s tuning must be an object." % what)
		return errors
	var tuning: Dictionary = raw
	for key: Variant in tuning:
		if not allowed.has(str(key)):
			errors.append("Unknown %s tuning '%s' (allowed: %s)." % [what.to_lower(), str(key), ", ".join(allowed)])
		elif not (tuning[key] is int or tuning[key] is float):
			errors.append("Tuning '%s' must be a number." % str(key))
	return errors

static func _character_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	if str(e.get("base_unit", "")).is_empty():
		errors.append("Choose the base unit this character plays as.")
	errors.append_array(_tuning_errors(e["tuning"], WorldSchema.TUNING_KEYS, "Character"))
	if e.get("gone_flags", []) is Array:
		errors.append_array(QuestRules.condition_errors(e.get("gone_flags", []), "Gone flags"))
	else:
		errors.append(GONE_FLAGS_NOT_LIST)   # one message, the same one blocking_errors shows
	var slots: Dictionary = {}
	for ref: String in _strings(e["named_items"]):
		var named: Dictionary = resolve(world_id, "items", ref)
		if named.is_empty():
			continue
		var base_item: Dictionary = ContentCatalog.get_entry("items", str(named.get("base_item", "")))
		if base_item.is_empty():
			continue
		var type: String = str(base_item.get("item_type", ""))
		if not ContentSchema.EQUIP_TYPES.has(type):
			errors.append("Named item '%s' isn't equipment, so it can't be worn." % ref)
			continue
		var slot: String = str(base_item.get("slot", ""))
		if slots.has(slot):
			errors.append("'%s' and '%s' compete for the %s slot." % [str(slots[slot]), ref, slot.to_lower().replace("_", " ")])
		slots[slot] = ref
	return errors

static func _item_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var base_id: String = str(e.get("base_item", ""))
	if base_id.is_empty():
		errors.append("Choose the base item this is a named version of.")
	elif ContentCatalog.items_inherit():
		# FORGE1_ITEMS 5: the base must resolve (its item family exists), else the named item has no numbers.
		var base: Dictionary = ContentCatalog.get_entry("items", base_id)
		if not base.is_empty() and ContentCatalog.family_of(base).is_empty():
			errors.append("Base item '%s' does not resolve: its item family '%s' is unknown." % [base_id, str(base.get("family", ""))])
	errors.append_array(_tuning_errors(e["tuning"], WorldSchema.ITEM_TUNING_KEYS, "Item"))
	return errors

static func _quest_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var steps: Variant = e["steps"]
	if not (steps is Array):
		errors.append("Steps must be a list.")
		return errors
	var list: Array = steps
	var templated: bool = not str(e.get("template", "")).is_empty()
	for i: int in range(list.size()):
		var step: Dictionary = list[i] if list[i] is Dictionary else {}
		if str(step.get("text", "")).strip_edges().is_empty():
			errors.append("Quest step %d needs a \"text\"." % (i + 1))
		if templated:
			continue   # bindings are checked per objective kind (QuestRules); unused keys are ignored
		var loc: String = str(step.get("location", ""))
		if not loc.is_empty() and resolve(world_id, "locations", loc).is_empty():
			errors.append("Quest step %d: unknown location '%s'." % [i + 1, loc])
		var enc: String = str(step.get("encounter", ""))
		if not enc.is_empty() and resolve(world_id, "encounters", enc).is_empty():
			errors.append("Quest step %d: unknown encounter '%s'." % [i + 1, enc])
	errors.append_array(QuestRules.world_quest_errors(world_id, e))
	return errors

# A loot entry: grantable ref for its reward kind, count rules, flag conditions (enums are checked generically).
static func _loot_errors(e: Dictionary) -> Array[String]:
	return QuestRules.loot_entry_errors(e)

static func _encounter_errors(_world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var party: Array = e["party"]
	if party.is_empty():
		errors.append("Add at least one party member.")
	if party.size() > 4:
		errors.append("The party holds at most 4 members.")
	if DictUtil.array_of(e["enemies"]).is_empty():
		errors.append("Add at least one enemy.")
	var wave: Array = e["wave_units"]
	if not wave.is_empty():
		if DictUtil.to_int(e["wave_block"]) > DictUtil.to_int(e["total_blocks"]):
			errors.append("The wave block is past the end of the round (%d blocks)." % DictUtil.to_int(e["total_blocks"]))
		if DictUtil.to_int(e["wave_round"]) > DictUtil.to_int(e["max_rounds"]):
			errors.append("The wave round is after the last round.")
	var exits: Variant = e["exits"]
	if exits is Array:
		for x: Variant in DictUtil.array_of(exits):
			var pair: Array = x if x is Array else []
			if pair.size() != 2:
				errors.append("Exit tiles are [x, y] pairs.")
				break
			if DictUtil.to_int(pair[0]) < 0 or DictUtil.to_int(pair[1]) < 0 or DictUtil.to_int(pair[0]) >= CombatGrid.GRID_W or DictUtil.to_int(pair[1]) >= CombatGrid.GRID_H:
				errors.append("Exit %s is off the 12x9 grid." % str(pair))
	else:
		errors.append("Exit tiles must be a list of [x, y] pairs.")
	return errors

# Problems with a world's manifest.
static func manifest_errors(world_id: String) -> Array[String]:
	var errors: Array[String] = []
	var m: Dictionary = manifest(world_id)
	if m.is_empty():
		errors.append("No world '%s'." % world_id)
		return errors
	if str(m.get("name", "")).strip_edges().is_empty():
		errors.append("The world needs a name.")
	if has_cycle(world_id):
		errors.append("The inheritance is circular.")
	for key: String in ["extends", "connects"]:
		for other: String in _strings(m.get(key, [])):
			if other == world_id:
				errors.append("%s itself." % ("Extends" if key == "extends" else "Connects to"))
			elif not _worlds().has(other):
				errors.append("%s unknown world '%s'." % ["Extends" if key == "extends" else "Connects to", other])
	var start: String = str(m.get("start_location", ""))
	if not start.is_empty() and resolve(world_id, "locations", start).is_empty():
		errors.append("Start location '%s' is unknown." % start)
	errors.append_array(start_party_errors(m.get("start_party", [])))
	var gold: Variant = m.get("start_gold", -1)
	if not (gold is int) or DictUtil.to_int(gold) < -1 or DictUtil.to_int(gold) > 99999:
		errors.append("Start gold must be -1 (inherited) or 0..99999.")
	var supplies: Variant = m.get("start_supplies", -1)
	if not (supplies is int) or DictUtil.to_int(supplies) < -1 or DictUtil.to_int(supplies) > PartyState.SUPPLIES_MAX:
		errors.append("Start supplies must be -1 (inherited) or 0..99.")
	errors.append_array(DialogueRules.service_lines_errors(m.get("service_lines", {}), "Service lines"))
	return errors

# Problems with a manifest `start_party` value (UI1-L): known party-side catalog units, at most PartyState.PARTY_MAX,
# no duplicates.
static func start_party_errors(raw: Variant) -> Array[String]:
	var errors: Array[String] = []
	var ids: Array[String] = _strings(raw)
	var seen: Dictionary = {}
	for id: String in ids:
		if seen.has(id):
			errors.append("Start party lists '%s' twice." % id)
			continue
		seen[id] = true
		var unit: Dictionary = ContentCatalog.get_entry("units", id)
		if unit.is_empty():
			errors.append("Start party unit '%s' is not a known unit." % id)
		elif str(unit.get("side", "enemy")) != "party":
			errors.append("Start party unit '%s' is not a party unit." % id)
	if ids.size() > PartyState.PARTY_MAX:
		errors.append("A start party holds at most 4 units.")
	return errors

# The resolved start party of `world_id`: its own non-empty `start_party`, else the nearest ancestor's (lineage order,
# the same rule as `start_location`). [] when none.
static func start_party(world_id: String) -> Array[String]:
	var chain: Array[String] = lineage(world_id)
	chain.reverse()
	for w: String in chain:
		var ids: Array[String] = _strings(manifest(w).get("start_party", []))
		if not ids.is_empty():
			return ids
	var none: Array[String] = []
	return none

# Every problem in one world's OWN entries: {"world/kind:id": [errors]}.
static func validate_world(world_id: String) -> Dictionary:
	var problems: Dictionary = {}
	var m_errors: Array[String] = manifest_errors(world_id)
	if not m_errors.is_empty():
		problems["%s/world" % world_id] = m_errors
	for kind: String in WorldSchema.KINDS:
		for e: Dictionary in own_entries(world_id, kind):
			var id: String = str(e.get("id", ""))
			var errs: Array[String] = validate_entry(world_id, kind, e, id)
			if not errs.is_empty():
				problems["%s/%s:%s" % [world_id, kind, id]] = errs
	return problems

# Soft warnings of one world (never blocking): map, route, dialogue, gone-NPC, quest link / reward budget and
# reputation key lints, each line prefixed with the entry qid. Lints over merged entries also report inherited ones.
static func world_warnings(world_id: String) -> Array[String]:
	var out: Array[String] = []
	out.append_array(MapRules.warnings(world_id))
	out.append_array(TravelRules.warnings(world_id))
	out.append_array(DialogueRules.dialogue_warnings(world_id))
	out.append_array(DialogueRules.quest_warnings(world_id))
	out.append_array(QuestRules.link_warnings(world_id))
	out.append_array(QuestRules.reward_warnings(world_id))
	out.append_array(RepRules.key_warnings(world_id))
	return out

# {world id: [warnings]} for every world with at least one warning.
static func warnings_all() -> Dictionary:
	var out: Dictionary = {}
	for w: String in world_ids():
		var warns: Array[String] = world_warnings(w)
		if not warns.is_empty():
			out[w] = warns
	return out

static func validate_all() -> Dictionary:
	var problems: Dictionary = {}
	for w: String in world_ids():
		problems.merge(validate_world(w))
	if not _error_list().is_empty():
		problems["load"] = _error_list().duplicate()
	return problems

# ------------------------------------------------------------------------------
# Saving, forking
# ------------------------------------------------------------------------------

static func save_world(world_id: String) -> Error:
	_ensure_loaded()
	if not _worlds().has(world_id):
		return ERR_DOES_NOT_EXIST
	var dir_path: String = "%s/%s" % [get_root(), world_id]
	if DirAccess.make_dir_recursive_absolute(dir_path) != OK and not DirAccess.dir_exists_absolute(dir_path):
		return ERR_CANT_CREATE
	var manifest_doc: Dictionary = WorldSchema.to_storage("world", manifest(world_id))
	manifest_doc["version"] = VERSION
	var err: Error = _write(path_for(world_id), manifest_doc)
	if err != OK:
		return err
	for kind: String in WorldSchema.KINDS:
		var stored: Array = []
		for e: Dictionary in own_entries(world_id, kind):
			stored.append(WorldSchema.to_storage(kind, e))
		err = _write(path_for(world_id, kind), {"kind": kind, "version": VERSION, "entries": stored})
		if err != OK:
			return err
	_dirty().erase(world_id)
	return OK

static func _write(path: String, doc: Dictionary) -> Error:
	var file := FileAccess.open(path, FileAccess.WRITE)
	if file == null:
		return FileAccess.get_open_error()
	file.store_string(JSON.stringify(doc, "\t", false) + "\n")
	file.close()
	return OK

static func save_all() -> Error:
	_ensure_loaded()
	for gone: String in _removed():
		_delete_world_files(gone)
	_removed().clear()
	for w: String in world_ids():
		var err: Error = save_world(w)
		if err != OK:
			return err
	return OK

static func _delete_world_files(world_id: String) -> void:
	var dir_path: String = "%s/%s" % [get_root(), world_id]
	var dir: DirAccess = DirAccess.open(dir_path)
	if dir == null:
		return
	for f: String in dir.get_files():
		DirAccess.remove_absolute("%s/%s" % [dir_path, f])
	DirAccess.remove_absolute(dir_path)

# Copies every loaded world into `p_root` (replacing its world folders) and switches
# to it: tests and experiments edit the copy, never the shipped worlds.
static func fork_to(p_root: String) -> Error:
	_ensure_loaded()
	_state()["root"] = p_root
	for w: String in world_ids():
		_touch(w, "world")
	var err: Error = save_all()
	if err == OK:
		reload_from_disk()
	return err
