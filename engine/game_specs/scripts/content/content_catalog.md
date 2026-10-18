---
class: ContentCatalog
extends: RefCounted
description: "Tactical RPG - Content Catalog (the single store of authored game content) Skills, items, classes, units (party members & enemies) and encounters live as JSON in res://data/catalog/<kind>.json:  {\"kind\": ..., \"version\": 1, \"entries\": [...]} Entries are plain Dictionaries shaped by ContentSchema; ContentFactory turns them into engine objects (SkillDef, Item, ClassArchetype, Combatant) and CatalogEncounter loads an encounter entry into the real combat engine. Everything that reads content goes through here: SkillCatalog, ItemCatalog, ClassArchetype presets, CatalogEncounter, the tests and the Content Forge. Tests that write use use_root(\"user://...\") so the shipped catalog is untouched."
---

# Properties
* `DEFAULT_ROOT`: String = "res://data/catalog" (const)
* `VERSION`: int = 1 (const)
* `VERSIONS`: Dictionary = {"skills": 2, "items": 2, "item_families": 1} (const)
* `STATE_KEY`: StringName = &"content_catalog_state" (const)
* `errors`: Array[String] = _state()["errors"]
* `out`: Array[Dictionary] = []
* `out`: Array[Dictionary] = []
* `raw`: Variant = _data_map().get(kind, [])
* `list`: Array = raw
* `out`: Array[String] = []
* `errors`: Array[String] = []
* `key`: String = str(f["key"])
* `errors`: Array[String] = []
* `flat`: Dictionary = ContentSchema.normalize("items", raw)
* `lists_changed`: bool = (raw.has("modifiers") and not ItemModifiers.same_value(raw["modifiers"], old.get("modifiers", []))) \
* `new_id`: String = str(entry.get("id", ""))
* `old_id`: String = replace_id if not replace_id.is_empty() else new_id
* `old`: Dictionary = get_entry(kind, old_id)
* `shape`: Array[String] = item_shape_errors(entry, old)
* `clean`: Dictionary = _prepare(kind, _with_origin(entry, old), old)
* `errors`: Array[String] = validate_entry(kind, clean, old_id)
* `list`: Array = _data_map()[kind]
* `at`: int = _index_of(kind, old_id)
* `list`: Array = _data_map()[kind]
* `backup`: Array = list.duplicate()
* `applied`: Array[Dictionary] = []
* `errors`: Array[String] = []
* `src`: Dictionary = DictUtil.dict_of(raw)
* `id`: String = str(src.get("id", ""))
* `old`: Dictionary = get_entry(kind, id)
* `clean`: Dictionary = _prepare(kind, _with_origin(src, old), old)
* `at`: int = _index_of(kind, str(clean.get("id", "")))
* `out`: Array[Dictionary] = []
* `folded`: Dictionary = SkillRanks.collapse(entries("skills"))
* `out`: Array[Dictionary] = []
* `list`: Array = folded["entries"]
* `out`: Array[Dictionary] = []
* `base`: Dictionary = SkillRanks.effective(entry, 1)
* `head`: String = str(base["id"])
* `kind`: String = str(base["kind"])
* `old_head`: String = current_head if not current_head.is_empty() else head
* `mods`: Array[Dictionary] = SkillRanks.modifiers_of(entry)
* `count`: int = mods.size() + 1
* `single`: bool = DictUtil.to_bool(base["basic"]) or DictUtil.to_bool(base["system"])
* `rid`: String = SkillRanks.rank_id(head, n)
* `owner`: Dictionary = get_entry("skills", rid)
* `longest`: String = SkillRanks.rank_name(str(base["name"]), count)
* `old_entry`: Dictionary = skill_entry(old_head)
* `old_base`: Dictionary = SkillRanks.effective(old_entry, 1)
* `rows`: Array[Dictionary] = SkillRanks.expand(entry)
* `list`: Array = _data_map()["skills"]
* `backup`: Array = list.duplicate()
* `keep`: Array = []
* `seen`: Dictionary = {}
* `rank`: int = DictUtil.to_int(row["tier"], 1)
* `tag`: String = message
* `out`: Array[Dictionary] = []
* `seen`: Dictionary = {}
* `tag`: String = "%s|%d|%s" % [str(p["field"]), DictUtil.to_int(p["rank"]), str(p["message"])]
* `out`: Array[Dictionary] = []
* `allowed`: Array[String] = SkillRanks.rank_fields(kind)
* `rank`: int = i + 2
* `mod`: Dictionary = mods[i]
* `key`: String = str(raw_key)
* `f`: Dictionary = ContentSchema.field("skills", key)
* `value`: Variant = mod[raw_key]
* `x`: float = DictUtil.to_float(value)
* `raw`: Dictionary = entry.duplicate(true)
* `head`: String = str(raw.get("id", ""))
* `old_head`: String = replace_id if not replace_id.is_empty() else head
* `old_rows`: Array[Dictionary] = family_rows(old_head)
* `problems`: Array[Dictionary] = validate_skill_entry(raw, old_head)
* `messages`: Array[String] = []
* `conflicts`: Array[String] = path_rename_conflicts(old_head, head)
* `rows`: Array[Dictionary] = SkillRanks.expand(raw)
* `list`: Array = _data_map()["skills"]
* `slots`: Array[int] = []
* `old_count`: int = slots.size()
* `remove_slots`: Array[int] = []
* `none`: Array[String] = []
* `users`: Array[String] = []
* `key`: String = str(f["key"])
* `value`: Variant = e.get(key, null)
* `hit`: bool = value is String and str(value) == id
* `list`: Array = value
* `label`: String = "%s:%s" % [other_kind, str(e.get("id", ""))]
* `errors`: Array[String] = []
* `path`: Dictionary = DictUtil.dict_of(c.get("skill_path", {}))
* `e`: Dictionary = ContentSchema.normalize(kind, entry)
* `errors`: Array[String] = []
* `id`: String = str(e.get("id", ""))
* `ref_kind`: String = str(f.get("ref", ""))
* `key`: String = str(f["key"])
* `targets`: Array[String] = []
* `value`: Variant = e.get(key, null)
* `value_list`: Array = value
* `seen`: Dictionary = {}
* `target`: Dictionary = get_entry(ref_kind, t)
* `errors`: Array[String] = []
* `pouch`: Array[String] = ContentFactory._strings(e.get("pouch", []))
* `key`: String = str(f["key"])
* `hit`: bool = str(e.get(key, "")) == ContentSchema.NO_ITEM
* `out`: Array[String] = ContentFactory._strings(existing)
* `errors`: Array[String] = []
* `row`: Dictionary = get_entry("skills", required)
* `e`: Dictionary = ContentSchema.normalize(kind, entry)
* `item_found`: Array[String] = GearBudget.check_item(e)
* `none`: Array[String] = []
* `errors`: Array[String] = []
* `party`: Array = e["party"]
* `foes`: Array = e["enemies"]
* `wave`: Array = e["wave_units"]
* `wave_field`: Dictionary = ContentSchema.field("encounters", "wave_units")
* `target`: Dictionary = get_entry("units", ref)
* `exits`: Variant = e["exits"]
* `exit_list`: Array = exits
* `pair`: Array = x if x is Array else []
* `cell`: Variant

# Methods

### `static version_of(kind: String) -> int`
```logic
return DictUtil.to_int(VERSIONS.get(kind, VERSION), VERSION)
```

### `static _state() -> Dictionary`
```logic
if not Engine.has_meta(STATE_KEY):
	var errors: Array[String] = []
	Engine.set_meta(STATE_KEY, {"root": DEFAULT_ROOT, "data": {}, "dirty": {}, "errors": errors, "scope": false, "allow_remove": false, "save_errors": []})
var state: Dictionary = Engine.get_meta(STATE_KEY)
return state
```

### `static get_root() -> String`
```logic
return str(_state()["root"])
```

### `static _data_map() -> Dictionary`
```logic
var data: Dictionary = _state()["data"]
return data
```

### `static _dirty_map() -> Dictionary`
```logic
var dirty: Dictionary = _state()["dirty"]
return dirty
```

### `static path_for(kind: String, p_root: String = "") -> String`
```logic
return "%s/%s.json" % [p_root if not p_root.is_empty() else get_root(), kind]
```

### `static use_root(p_root: String) -> void`
```logic
_state()["root"] = p_root
reload_from_disk()
```

### `static fork_to(p_root: String) -> Error`
```logic
_ensure_loaded()
_state()["root"] = p_root
for kind: String in ContentSchema.KINDS:
	_dirty_map()[kind] = true
var err: Error = save_all()
if err == OK:
	reload_from_disk()
return err
```

### `static reload_from_disk() -> void`
```logic
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
```

### `static _ensure_loaded() -> void`
```logic
if _data_map().is_empty():
	reload_from_disk()
```

### `static _blocked_map() -> Dictionary`
```logic
if not _state().has("blocked"):
	_state()["blocked"] = {}
var blocked: Dictionary = _state()["blocked"]
return blocked
```

### `static _block_save(kind: String) -> void`
```logic
_blocked_map()[kind] = true
```

### `static save_blocked(kind: String) -> bool`
```logic
_ensure_loaded()
return _blocked_map().has(kind)
```

### `static _read_kind(kind: String) -> Array`
```logic
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
	_error_list().append("%s: version %d skills files (flat rows) are no longer read; write version 2 (ranked entries)." % [path, file_version])
	_block_save(kind)
	return found
if kind == "items":
	return _read_items(list, path, file_version)
for item: Variant in list:
	if item is Dictionary:
		var dict: Dictionary = item
		var row: Dictionary = ContentSchema.normalize(kind, dict)
		found.append(ItemFamilies.derive(row) if kind == "item_families" else row)
return found
```

### `static _read_items(list: Array, path: String, file_version: int) -> Array`
```logic
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
```

### `static _family_in(families: Array, id: String) -> Dictionary`
```logic
if id.is_empty():
	return {}
for raw: Variant in families:
	var f: Dictionary = DictUtil.dict_of(raw)
	if str(f.get("id", "")) == id:
		return f
return {}
```

### `static items_inherit() -> bool`
```logic
return not entries("item_families").is_empty()
```

### `static family_of(item: Dictionary) -> Dictionary`
```logic
return _family_in(entries("item_families"), str(item.get("family", "")))
```

### `static _read_ranked_skills(list: Array, path: String) -> Array`
```logic
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
```

### `static save(kind: String) -> Error`
```logic
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
```

### `static save_all() -> Error`
```logic
for kind: String in ContentSchema.KINDS:
	var err: Error = save(kind)
	if err != OK:
		return err
return OK
```

### `static is_dirty(kind: String = "") -> bool`
```logic
return _dirty_map().has(kind) if not kind.is_empty() else not _dirty_map().is_empty()
```

### `static clear_cache() -> void`
```logic
if Engine.has_meta(STATE_KEY):
	Engine.remove_meta(STATE_KEY)
Terrain.clear_cache()
```

### `static has(kind: String, id: String) -> bool`
```logic
return _index_of(kind, id) >= 0
```

### `static get_entry(kind: String, id: String) -> Dictionary`
```logic
var i: int = _index_of(kind, id)
return entries(kind)[i].duplicate(true) if i >= 0 else {}
```

### `static _index_of(kind: String, id: String) -> int`
```logic
var list: Array[Dictionary] = entries(kind)
for i: int in range(list.size()):
	if str(list[i].get("id", "")) == id:
		return i
return -1
```

### `static _problem(field: String, rank: int, message: String) -> Dictionary`
```logic
return SkillValidator.problem(field, rank, message)
```

### `static begin_system_edit(allow_remove: bool = false) -> void`
```logic
_ensure_loaded()
_state()["scope"] = true
_state()["allow_remove"] = allow_remove
```

### `static end_system_edit() -> void`
```logic
_state()["scope"] = false
_state()["allow_remove"] = false
```

### `static in_system_edit() -> bool`
```logic
return DictUtil.to_bool(_state().get("scope", false))
```

### `static _can_remove_system() -> bool`
```logic
return in_system_edit() and DictUtil.to_bool(_state().get("allow_remove", false))
```

### `static is_system_origin(entry: Dictionary) -> bool`
```logic
return str(entry.get("origin", ContentSchema.ORIGIN_SYSTEM)) == ContentSchema.ORIGIN_SYSTEM
```

### `static _with_origin(entry: Dictionary, old: Dictionary) -> Dictionary`
```logic
var raw: Dictionary = entry.duplicate()
if old.is_empty():
	if not in_system_edit():
		raw["origin"] = ContentSchema.ORIGIN_USER
elif not raw.has("origin"):
	raw["origin"] = old.get("origin", ContentSchema.ORIGIN_SYSTEM)
return raw
```

### `static _prepare(kind: String, raw: Dictionary, old: Dictionary) -> Dictionary`
```logic
match kind:
	"item_families":
		return ItemFamilies.derive(ContentSchema.normalize(kind, raw))
	"items":
		return _prepare_item(raw, old)
	"terrain":
		return TerrainFields.prepare(ContentSchema.normalize(kind, raw))
return ContentSchema.normalize(kind, raw)
```

### `static _prepare_item(raw: Dictionary, old: Dictionary) -> Dictionary`
```logic
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
```

### `static _family_matches(family: Dictionary, flat: Dictionary) -> bool`
```logic
var type: String = ItemFamilies.item_type_of(family)
if str(flat.get("item_type", "")) != type:
	return false
var token: String = str(family.get("id", "")) if ItemFamilies.is_gear_type(type) else ""
return str(flat.get("weapon_type", "")) == token
```

### `static remove(kind: String, id: String) -> String`
```logic
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
```

### `static _set_origin(kind: String, id: String, origin: String) -> String`
```logic
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
```

### `static promote(kind: String, id: String) -> String`
```logic
_ensure_loaded()
var entry: Dictionary = get_entry(kind, id)
if entry.is_empty():
	return "No %s entry '%s'." % [kind, id]
if is_system_origin(entry):
	return "'%s' is already a system asset." % id
return _set_origin(kind, id, ContentSchema.ORIGIN_SYSTEM)
```

### `static demote(kind: String, id: String) -> String`
```logic
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
```

### `static family_has_problems(head_id: String) -> bool`
```logic
for row: Dictionary in family_rows(head_id):
	if not validate_entry("skills", row, str(row["id"])).is_empty():
		return true
return false
```

### `static skill_entry(head_id: String) -> Dictionary`
```logic
var rows: Array[Dictionary] = family_rows(head_id)
if rows.is_empty():
	return {}
var normal: Array[Dictionary] = []
for row: Dictionary in rows:
	normal.append(ContentSchema.normalize("skills", row))
var folded: Dictionary = SkillRanks.fold(head_id, normal)
return DictUtil.dict_of(folded["entry"])
```

### `static _field_of_message(message: String) -> String`
```logic
for f: Dictionary in ContentSchema.fields("skills"):
	if message.begins_with(str(f["label"])):
		return str(f["key"])
return ""
```

### `static remove_skill(head_id: String) -> String`
```logic
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
```

### `static unique_id(kind: String, base: String) -> String`
```logic
var stem: String = base if ContentSchema.is_valid_id(base) else "new_entry"
if not has(kind, stem):
	return stem
var n: int = 2
while has(kind, "%s_%d" % [stem, n]):
	n += 1
return "%s_%d" % [stem, n]
```

### `static duplicate_entry(kind: String, id: String) -> String`
```logic
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
```

### `static _duplicate_skill(head_id: String) -> String`
```logic
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
```

### `static _touched(kind: String) -> void`
```logic
_dirty_map()[kind] = true
if kind == "skills":
	SkillCatalog.clear_cache()
if kind == "terrain":
	Terrain.clear_cache()
if kind == "item_families":
	_reresolve_items()
```

### `static _reresolve_items() -> void`
```logic
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
```

### `static _rename_refs(kind: String, old_id: String, new_id: String) -> void`
```logic
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
```

### `static _renamed_key(d: Dictionary, old_key: String, new_key: String) -> Dictionary`
```logic
var out: Dictionary = {}
for key: Variant in d:
	out[new_key if str(key) == old_key else key] = d[key]
return out
```

### `static _is_gear_field(kind: String, key: String) -> bool`
```logic
return (kind == "units" or kind == "classes") and (ContentSchema.LOADOUT_SLOTS.has(key) or key == "pouch")
```

### `static validate_all() -> Dictionary`
```logic
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
```

### `static terrain_file_exists() -> bool`
```logic
return FileAccess.file_exists(path_for("terrain"))
```

### `static warnings_all() -> Dictionary`
```logic
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
```

### `static grade_rank(grade: String) -> int`
```logic
return maxi(0, ["novice", "advanced", "expert"].find(grade))
```

