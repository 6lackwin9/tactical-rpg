---
class: AssetScan
extends: RefCounted
description: "Tactical RPG - Asset scan and SAFE DELETE (FORGE1 section 12.5) System assets are tuning targets that tests, seeds and balance sims name, so deleting one is a process: scan every place that can name it, refuse while any reference exists, and only then remove it inside the system-edit scope. tools/safe_delete.gd is the command-line wrapper; the Forge only points at it."
---

# Properties
* `SCAN_DIRS`: Array[String] = ["res://scripts/tests", "res://scenes/dojo", "res://data/seeds"] (const)
* `WORLD_DIR`: String = "res://data/worlds" (const)
* `ASSETS_KIND`: String = "assets" (const)
* `KEY_FIELDS`: Dictionary = {"unit": {"classes": "art", "units": "art"}, "item": {"items": "icon_id", "item_families": "art"}} (const)
* `WORLD_KEY_FIELDS`: Dictionary = {"unit": {"characters": "art"}, "item": {"items": "icon_id"}} (const)
* `refs`: Array[String] = []
* `fields`: Dictionary = DictUtil.dict_of(KEY_FIELDS.get(ns, {}))
* `world_fields`: Dictionary = DictUtil.dict_of(WORLD_KEY_FIELDS.get(ns, {}))
* `hits`: Array[String] = []
* `hits`: Array[String] = []
* `text`: String = FileAccess.get_file_as_string(path)
* `lines`: PackedStringArray = text.split("\n")
* `ids`: Array[String] = [id]
* `row`: Dictionary = AssetCatalog.get_entry(id)
* `ids`: Array[String] = ids_of(kind, id)
* `refs`: Array[String] = []

# Methods

### `static safe_delete(kind: String, id: String, apply: bool) -> Dictionary`
```logic
var entry: Dictionary = ContentCatalog.get_entry(kind, id)
var refs: Array[String] = []
if entry.is_empty():
	return _result(false, false, refs, "No %s entry '%s'." % [kind, id])
if kind == "skills":
	for row: Dictionary in ContentCatalog.family_rows(id):
		if DictUtil.to_bool(row.get("basic", false)):
			return _result(false, false, refs, "'%s' is a basic skill and is never deleted." % id)
		if DictUtil.to_bool(row.get("system", false)):
			return _result(false, false, refs, "'%s' is an engine skill and is never deleted." % id)
	if SkillKinds.REQUIRED_IDS.has(id):
		return _result(false, false, refs, "'%s' is a required engine skill and is never deleted." % id)
elif DictUtil.to_bool(entry.get("system", false)):
	return _result(false, false, refs, "'%s' is an engine entry and is never deleted." % id)
refs = scan(kind, id)
if not refs.is_empty():
	return _result(false, false, refs, "Refused: '%s' is still referenced (%d places). Nothing changed." % [id, refs.size()])
if not apply:
	return _result(true, false, refs, "'%s' has no references; run with --apply to delete it." % id)
ContentCatalog.begin_system_edit(true)
var why: String = ContentCatalog.remove_skill(id) if kind == "skills" else ContentCatalog.remove(kind, id)
ContentCatalog.end_system_edit()
if not why.is_empty():
	return _result(false, false, refs, why)
var err: Error = ContentCatalog.save(kind)
if err != OK:
	return _result(false, true, refs, "Removed but could not save %s (error %d)." % [kind, err])
var problems: Dictionary = ContentCatalog.validate_all()
return _result(problems.is_empty(), true, refs, "Deleted '%s'. validate_all: %s" % [id, "clean" if problems.is_empty() else str(problems.keys())])
```

### `static _result(ok: bool, removed: bool, refs: Array[String], message: String) -> Dictionary`
```logic
return {"ok": ok, "removed": removed, "refs": refs, "message": message}
```

