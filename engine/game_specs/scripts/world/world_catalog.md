---
class: WorldCatalog
extends: RefCounted
description: "Tactical RPG - World Catalog (modular, inheritable lore worlds) Worlds live in res://data/worlds/<world_id>/: world.json (manifest) plus one <kind>.json per WorldSchema kind ({\"kind\", \"version\", \"entries\": [...]}). Inheritance A world `extends` other worlds and sees all their entries. lineage(world) is the load order: ancestors first (parents in `extends` order), the world last. Entries are namespaced: the qualified id of \"mara\" in world \"ashlands\" is \"ashlands:mara\". An unqualified ref resolves in the entry's own world first, then up the lineage (nearest wins); \"world:id\" is explicit. An entry with `override: \"ashlands:mara\"` replaces that ancestor entry for its world and every world below it (refs to the old id resolve to the override). `connects` links worlds without inheriting: qualified refs may cross into them. Entries are plain Dictionaries shaped by WorldSchema. Merged views add `_world` and `_qid`. State (loaded data, root, dirty flags) lives on the Engine singleton like ContentCatalog, so hot reloads can't reset it. Tests that write call fork_to(\"user://...\") so the shipped worlds stay untouched."
---

# Properties
* `DEFAULT_ROOT`: String = "res://data/worlds" (const)
* `VERSION`: int = 1 (const)
* `STATE_KEY`: StringName = &"world_catalog_state" (const)
* `MARKER_REF_KEYS`: Dictionary = {"encounter": "encounters", "character": "characters", "lore": "lore", "loot": "loot", "to": "locations"} (const)
* `GONE_FLAGS_NOT_LIST`: String = "Gone flags must be a list of {\"key\", \"value\"} conditions." (const)
* `errors`: Array[String] = _state()["errors"]
* `removed`: Array[String] = _state()["removed"]
* `out`: Array[String] = []
* `out`: Array[String] = []
* `arr`: Array = v
* `out`: Array[String] = []
* `stack`: Array[String] = []
* `out`: Array[String] = []
* `parts`: PackedStringArray = ref.split(":", false, 1)
* `out`: Array[String] = []
* `out`: Array[Dictionary] = []
* `d`: Dictionary = e
* `copy`: Dictionary = d.duplicate(true)
* `all`: Array[Dictionary] = []
* `overridden`: Dictionary = {}
* `ov`: String = str(e.get("override", ""))
* `out`: Array[Dictionary] = []
* `out`: Array[String] = []
* `errors`: Array[String] = []
* `m`: Dictionary = WorldSchema.normalize("world", {"id": world_id, "name": p_name, "extends": extends_from})
* `kinds`: Dictionary = {}
* `errors`: Array[String] = []
* `updated`: Dictionary = manifest(world_id)
* `previous`: Dictionary = manifest(world_id)
* `clean`: Dictionary = WorldSchema.normalize(kind, entry)
* `new_id`: String = str(clean.get("id", ""))
* `old_id`: String = replace_id if not replace_id.is_empty() else new_id
* `errors`: Array[String] = blocking_errors(world_id, kind, clean, old_id)
* `list`: Array = _own_list(world_id, kind)
* `at`: int = -1
* `item`: Dictionary = list[i]
* `rename_matches`: Array[Dictionary] = []
* `target`: Dictionary = {"id": new_id, "_world": world_id, "_qid": qid(world_id, new_id)}
* `target`: String = qid(world_id, local)
* `out`: Array[Dictionary] = []
* `key`: String = str(rf["key"])
* `value`: Variant = e.get(key, null)
* `arr`: Array = value
* `out`: Array[Dictionary] = []
* `base`: Dictionary = {"world": w, "kind": k2, "id": str(e.get("id", ""))}
* `exits`: Array = e["exits"]
* `ex`: Dictionary = exits[i] if exits[i] is Dictionary else {}
* `for_sale`: Array = DictUtil.array_of(e.get("loot", null))
* `steps`: Array = e["steps"]
* `sub`: String = "location" if kind == "locations" else "encounter"
* `step`: Dictionary = steps[i] if steps[i] is Dictionary else {}
* `rel`: Dictionary = e["relations"]
* `out`: Array[Dictionary] = []
* `rep`: Dictionary = DictUtil.dict_of(e.get("reputation", null))
* `variants`: Array = DictUtil.array_of(e.get("variants", null))
* `set_d`: Dictionary = DictUtil.dict_of(DictUtil.dict_of(variants[vi]).get("set", null))
* `at`: Array = ["variants", vi, "set"]
* `steps`: Dictionary = DictUtil.dict_of(set_d.get("steps", null))
* `sub`: String = "location" if kind == "locations" else ("encounter" if kind == "encounters" else "")
* `partial`: Dictionary = DictUtil.dict_of(steps[idx])
* `out`: Array[Dictionary] = []
* `bindings`: Dictionary = DictUtil.dict_of(e.get("zone_bindings", null))
* `b`: Dictionary = DictUtil.dict_of(bindings[zone])
* `pairs`: Array = DictUtil.array_of(b.get(list_key, null))
* `pair`: Array = DictUtil.array_of(pairs[i])
* `markers`: Array = DictUtil.array_of(e.get("markers", null))
* `m`: Dictionary = DictUtil.dict_of(markers[i])
* `ref`: String = str(m[key])
* `out`: Array[Dictionary] = []
* `pairs`: Array = DictUtil.array_of(e.get(list_key, null))
* `pair`: Array = DictUtil.array_of(pairs[i])
* `scripted`: Array = DictUtil.array_of(e.get("scripted", null))
* `out`: Array[Dictionary] = []
* `nodes`: Array = DictUtil.array_of(e.get("nodes", null))
* `node`: Dictionary = DictUtil.dict_of(nodes[i])
* `at`: Array = ["nodes", i]
* `choices`: Array = DictUtil.array_of(node.get("choices", null))
* `c`: Dictionary = DictUtil.dict_of(choices[j])
* `c_at`: Array = at + ["choices", j]
* `conds`: Array = DictUtil.array_of(c.get(list_key, null))
* `cond`: Dictionary = DictUtil.dict_of(conds[k])
* `e`: Dictionary = WorldSchema.normalize(kind, entry)
* `errors`: Array[String] = []
* `id`: String = str(e.get("id", ""))
* `value`: String = str(e.get(str(f["key"]), ""))
* `errors`: Array[String] = []
* `pair`: Array = x if x is Array else []
* `e`: Dictionary = WorldSchema.normalize(kind, entry)
* `errors`: Array[String] = []
* `id`: String = str(e.get("id", ""))
* `value`: String = str(e.get(str(f["key"]), ""))
* `out`: Array[String] = [world_id]
* `errors`: Array[String] = []
* `f`: Dictionary = rf["field"]
* `ref_kind`: String = str(rf["ref"])
* `values`: Array[String] = []
* `raw`: Variant = e.get(str(rf["key"]), null)
* `seen`: Dictionary = {}
* `errors`: Array[String] = []
* `label`: String = str(f["label"])
* `base_kind`: String = ref_kind.trim_prefix("base:")
* `target`: Dictionary = ContentCatalog.get_entry(base_kind, v)
* `parts`: Array[String] = split_qid(v)
* `hit`: Dictionary = resolve(world_id, ref_kind, v)
* `errors`: Array[String] = []
* `ov`: String = str(e.get("override", ""))
* `parts`: Array[String] = split_qid(ov)
* `errors`: Array[String] = []
* `seen`: Array[String] = [id]
* `cursor`: String = str(e.get("parent", ""))
* `hit`: Dictionary = resolve(world_id, "regions", cursor)
* `hit_id`: String = str(hit.get("id", ""))
* `errors`: Array[String] = []
* `exits`: Variant = e["exits"]
* `list`: Array = exits
* `ex`: Dictionary = x if x is Dictionary else {}
* `to`: String = str(ex.get("to", ""))
* `parts`: Array[String] = split_qid(to)
* `errors`: Array[String] = []
* `relations`: Variant = e["relations"]
* `rel`: Dictionary = relations
* `errors`: Array[String] = []
* `tuning`: Dictionary = raw
* `errors`: Array[String] = []
* `slots`: Dictionary = {}
* `named`: Dictionary = resolve(world_id, "items", ref)
* `base_item`: Dictionary = ContentCatalog.get_entry("items", str(named.get("base_item", "")))
* `type`: String = str(base_item.get("item_type", ""))
* `slot`: String = str(base_item.get("slot", ""))
* `errors`: Array[String] = []
* `base_id`: String = str(e.get("base_item", ""))
* `base`: Dictionary = ContentCatalog.get_entry("items", base_id)
* `errors`: Array[String] = []
* `steps`: Variant = e["steps"]
* `list`: Array = steps
* `templated`: bool = not str(e.get("template", "")).is_empty()
* `step`: Dictionary = list[i] if list[i] is Dictionary else {}
* `loc`: String = str(step.get("location", ""))
* `enc`: String = str(step.get("encounter", ""))
* `errors`: Array[String] = []
* `party`: Array = e["party"]
* `wave`: Array = e["wave_units"]
* `wave_field`: Dictionary = WorldSchema.field("encounters", "wave_units")
* `exits`: Variant = e["exits"]
* `pair`: Array = x if x is Array else []
* `errors`: Array[String] = []
* `m`: Dictionary = manifest(world_id)
* `start`: String = str(m.get("start_location", ""))
* `cap`: Variant = m.get("party_max", -1)
* `resolved`: Array[String] = start_party(world_id)
* `gold`: Variant = m.get("start_gold", -1)
* `supplies`: Variant = m.get("start_supplies", -1)
* `errors`: Array[String] = []
* `ids`: Array[String] = _strings(raw)
* `seen`: Dictionary = {}
* `unit`: Dictionary = ContentCatalog.get_entry("units", id)
* `character`: Dictionary = resolve(world_id, "characters", id)
* `errors`: Array[String] = []
* `c`: Dictionary = resolve(world_id, "characters", id)
* `base_id`: String = str(c.get("base_unit", ""))
* `base`: Dictionary = ContentCatalog.get_entry("units", base_id)
* `chain`: Array[String] = lineage(world_id)
* `ids`: Array[String] = _strings(manifest(w).get("start_party", []))
* `none`: Array[String] = []
* `out`: Array[String] = []

# Methods

### `static _state() -> Dictionary`
```logic
if not Engine.has_meta(STATE_KEY):
	var errors: Array[String] = []
	var removed: Array[String] = []
	Engine.set_meta(STATE_KEY, {"root": DEFAULT_ROOT, "worlds": {}, "dirty": {}, "errors": errors, "removed": removed, "loaded": false})
var state: Dictionary = Engine.get_meta(STATE_KEY)
return state
```

### `static get_root() -> String`
```logic
return str(_state()["root"])
```

### `static _worlds() -> Dictionary`
```logic
var worlds: Dictionary = _state()["worlds"]
return worlds
```

### `static _dirty() -> Dictionary`
```logic
var dirty: Dictionary = _state()["dirty"]
return dirty
```

### `static _ensure_loaded() -> void`
```logic
if not DictUtil.to_bool(_state()["loaded"]):
	reload_from_disk()
```

### `static use_root(p_root: String) -> void`
```logic
_state()["root"] = p_root
reload_from_disk()
```

### `static path_for(world_id: String, kind: String = "", p_root: String = "") -> String`
```logic
var base: String = "%s/%s" % [p_root if not p_root.is_empty() else get_root(), world_id]
return "%s/%s.json" % [base, kind if not kind.is_empty() else "world"]
```

### `static reload_from_disk() -> void`
```logic
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
```

### `static _read_json(path: String) -> Variant`
```logic
if not FileAccess.file_exists(path):
	return null
var json := JSON.new()
if json.parse(FileAccess.get_file_as_string(path)) != OK:
	_error_list().append("%s: JSON error on line %d: %s" % [path, json.get_error_line(), json.get_error_message()])
	return null
return json.data
```

### `static _read_kind(world_id: String, kind: String) -> Array`
```logic
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
```

### `static clear_cache() -> void`
```logic
if Engine.has_meta(STATE_KEY):
	Engine.remove_meta(STATE_KEY)
```

### `static has_world(world_id: String) -> bool`
```logic
_ensure_loaded()
return _worlds().has(world_id)
```

### `static manifest(world_id: String) -> Dictionary`
```logic
_ensure_loaded()
var raw: Variant = _worlds().get(world_id, null)
if raw is Dictionary:
	var w: Dictionary = raw
	var m: Dictionary = w["manifest"]
	return m.duplicate(true)
return {}
```

### `static _manifest_ref(world_id: String) -> Dictionary`
```logic
var w: Dictionary = _worlds()[world_id]
var m: Dictionary = w["manifest"]
return m
```

### `static _own_list(world_id: String, kind: String) -> Array`
```logic
var raw: Variant = _worlds().get(world_id, null)
if raw is Dictionary:
	var w: Dictionary = raw
	var kinds: Dictionary = w["kinds"]
	var list_raw: Variant = kinds.get(kind, [])
	if list_raw is Array:
		var list: Array = list_raw
		return list
return []
```

### `static _visit(world_id: String, out: Array[String], stack: Array[String]) -> void`
```logic
if out.has(world_id) or stack.has(world_id) or not _worlds().has(world_id):
	return
stack.append(world_id)
for parent: String in _strings(_manifest_ref(world_id).get("extends", [])):
	_visit(parent, out, stack)
stack.pop_back()
out.append(world_id)
```

### `static has_cycle(world_id: String) -> bool`
```logic
return _cycle_from(world_id, [])
```

### `static _cycle_from(world_id: String, path: Array[String]) -> bool`
```logic
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
```

### `static qid(world_id: String, local_id: String) -> String`
```logic
return "%s:%s" % [world_id, local_id]
```

### `static _redirects(world_id: String, kind: String) -> Dictionary`
```logic
var out: Dictionary = {}
for w: String in lineage(world_id):
	for e: Dictionary in own_entries(w, kind):
		var ov: String = str(e.get("override", ""))
		if not ov.is_empty():
			out[ov] = e
return out
```

### `static resolve(world_id: String, kind: String, ref: String) -> Dictionary`
```logic
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
```

### `static _own_by_id(world_id: String, kind: String, local: String) -> Dictionary`
```logic
for e: Dictionary in own_entries(world_id, kind):
	if str(e.get("id", "")) == local:
		return e
return {}
```

### `static ref_value(world_id: String, entry: Dictionary) -> String`
```logic
return str(entry.get("id", "")) if str(entry.get("_world", "")) == world_id else str(entry.get("_qid", ""))
```

### `static has_entry(world_id: String, kind: String, ref: String) -> bool`
```logic
return not resolve(world_id, kind, ref).is_empty()
```

### `static get_own(world_id: String, kind: String, local: String) -> Dictionary`
```logic
var e: Dictionary = _own_by_id(world_id, kind, local)
e.erase("_world")
e.erase("_qid")
return e
```

### `static entry_count(world_id: String) -> int`
```logic
var n: int = 0
for kind: String in WorldSchema.KINDS:
	n += _own_list(world_id, kind).size()
return n
```

### `static _touch(world_id: String, kind: String) -> void`
```logic
var dirty: Dictionary = _dirty()
var per_world: Dictionary = {}
var raw: Variant = dirty.get(world_id, null)
if raw is Dictionary:
	per_world = raw
per_world[kind] = true
dirty[world_id] = per_world
```

### `static is_dirty(world_id: String = "") -> bool`
```logic
if world_id.is_empty():
	return not _dirty().is_empty() or not _removed().is_empty()
return _dirty().has(world_id)
```

### `static remove_world(world_id: String) -> String`
```logic
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
```

### `static remove(world_id: String, kind: String, local: String) -> String`
```logic
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
```

### `static unique_id(world_id: String, kind: String, base: String) -> String`
```logic
var stem: String = base if ContentSchema.is_valid_id(base) else "new_entry"
var taken: Array[String] = own_ids(world_id, kind)
if not taken.has(stem):
	return stem
var n: int = 2
while taken.has("%s_%d" % [stem, n]):
	n += 1
return "%s_%d" % [stem, n]
```

### `static duplicate_entry(world_id: String, kind: String, local: String) -> String`
```logic
var src: Dictionary = get_own(world_id, kind, local)
if src.is_empty():
	return ""
src["id"] = unique_id(world_id, kind, local + "_copy")
src["name"] = str(src.get("name", local)) + " (Copy)"
src.erase("override")
return str(src["id"]) if upsert(world_id, kind, src).is_empty() else ""
```

### `static override_entry(world_id: String, kind: String, source_qid: String) -> String`
```logic
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
```

### `static _effect_refs(w: String, raw: Variant, at: Array, kind: String, target: String, base: Dictionary, out: Array[Dictionary]) -> void`
```logic
var effects: Array = DictUtil.array_of(raw)
for k: int in range(effects.size()):
	var fx: Dictionary = DictUtil.dict_of(effects[k])
	for key: Variant in DialogueRules.EFFECT_REF_KEYS:
		if str(DialogueRules.EFFECT_REF_KEYS[key]) == kind:
			_ref_at(w, fx, str(key), at + [k], kind, target, base, out)
```

### `static _ref_at(w: String, holder: Dictionary, key: String, at: Array, kind: String, target: String, base: Dictionary, out: Array[Dictionary]) -> void`
```logic
if holder.get(key, null) is String and _points_at(w, kind, str(holder[key]), target):
	out.append(base.merged({"key": str(at[0]), "index": DictUtil.to_int(at[1], -1) if at.size() > 1 else -1, "path": at + [key], "value": str(holder[key])}))
```

### `static _links_refs(w: String, links: Dictionary, at: Array, kind: String, target: String, base: Dictionary, out: Array[Dictionary]) -> void`
```logic
for key: Variant in links:
	if str(QuestRules.LINK_KINDS.get(str(key), "")) == kind:
		_list_refs(w, links[key], at + [key], kind, target, base, out)
```

### `static _list_refs(w: String, raw: Variant, at: Array, kind: String, target: String, base: Dictionary, out: Array[Dictionary]) -> void`
```logic
var list: Array = DictUtil.array_of(raw)
for i: int in range(list.size()):
	if list[i] is String and _points_at(w, kind, str(list[i]), target):
		out.append(base.merged({"key": str(at[0]), "index": i, "path": at + [i], "value": str(list[i])}))
```

### `static _points_at(world_id: String, kind: String, ref: String, target_qid: String) -> bool`
```logic
if ref.is_empty():
	return false
var hit: Dictionary = resolve(world_id, kind, ref)
return not hit.is_empty() and str(hit.get("_qid", "")) == target_qid or (ref == target_qid)
```

### `static _rewrite_ref(match_info: Dictionary, new_target: Dictionary, old_qid: String) -> void`
```logic
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
```

### `static _set_at_path(root: Dictionary, path: Array, expected: String, value: String) -> void`
```logic
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
```

### `static can_reach_world(world_id: String, other: String) -> bool`
```logic
var chain: Array[String] = lineage(world_id)
if chain.has(other):
	return true
for w: String in chain:   # connections are inherited: a child may link wherever its ancestors do
	if _strings(manifest(w).get("connects", [])).has(other):
		return true
return false
```

### `static guest_ok(world_id: String, ref: String) -> bool`
```logic
var c: Dictionary = resolve(world_id, "characters", ref)
if c.is_empty() or str(c.get("role", "")) != "npc":
	return true
return str(ContentCatalog.get_entry("units", str(c.get("base_unit", ""))).get("side", "")) == "party"
```

### `static party_max(world_id: String) -> int`
```logic
var chain: Array[String] = lineage(world_id)
chain.reverse()
for w: String in chain:
	var cap: Variant = manifest(w).get("party_max", -1)
	if cap is int and DictUtil.to_int(cap) >= 1:
		return mini(DictUtil.to_int(cap), PartyState.PARTY_MAX)
return PartyState.PARTY_MAX
```

### `static validate_world(world_id: String) -> Dictionary`
```logic
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
```

### `static warnings_all() -> Dictionary`
```logic
var out: Dictionary = {}
for w: String in world_ids():
	var warns: Array[String] = world_warnings(w)
	if not warns.is_empty():
		out[w] = warns
return out
```

### `static validate_all() -> Dictionary`
```logic
var problems: Dictionary = {}
for w: String in world_ids():
	problems.merge(validate_world(w))
if not _error_list().is_empty():
	problems["load"] = _error_list().duplicate()
return problems
```

### `static save_world(world_id: String) -> Error`
```logic
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
```

### `static _write(path: String, doc: Dictionary) -> Error`
```logic
var file := FileAccess.open(path, FileAccess.WRITE)
if file == null:
	return FileAccess.get_open_error()
file.store_string(JSON.stringify(doc, "\t", false) + "\n")
file.close()
return OK
```

### `static save_all() -> Error`
```logic
_ensure_loaded()
for gone: String in _removed():
	_delete_world_files(gone)
_removed().clear()
for w: String in world_ids():
	var err: Error = save_world(w)
	if err != OK:
		return err
return OK
```

### `static _delete_world_files(world_id: String) -> void`
```logic
var dir_path: String = "%s/%s" % [get_root(), world_id]
var dir: DirAccess = DirAccess.open(dir_path)
if dir == null:
	return
for f: String in dir.get_files():
	DirAccess.remove_absolute("%s/%s" % [dir_path, f])
DirAccess.remove_absolute(dir_path)
```

### `static fork_to(p_root: String) -> Error`
```logic
_ensure_loaded()
_state()["root"] = p_root
for w: String in world_ids():
	_touch(w, "world")
var err: Error = save_all()
if err == OK:
	reload_from_disk()
return err
```

