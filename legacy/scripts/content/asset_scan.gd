class_name AssetScan
extends RefCounted

# ==============================================================================
# Tactical RPG - Asset scan and SAFE DELETE (FORGE1 section 12.5)
# System assets are tuning targets that tests, seeds and balance sims name, so deleting one is a process:
# scan every place that can name it, refuse while any reference exists, and only then remove it inside the
# system-edit scope. tools/safe_delete.gd is the command-line wrapper; the Forge only points at it.
# ==============================================================================

const SCAN_DIRS: Array[String] = ["res://scripts/tests", "res://scenes/dojo", "res://data/seeds"]
const WORLD_DIR: String = "res://data/worlds"
const ASSETS_KIND: String = "assets"
# Content fields that hold art keys, per asset namespace (S14-L1 schema contract).
const KEY_FIELDS: Dictionary = {"unit": {"classes": "art", "units": "art"}, "item": {"items": "icon_id", "item_families": "art"}}
const WORLD_KEY_FIELDS: Dictionary = {"unit": {"characters": "art"}, "item": {"items": "icon_id"}}

# Everything that names art key `key` of namespace `ns`: content entries ("classes:<id>"), world entries
# ("<world>:characters:<id>") and other manifest rows aliasing it ("assets:<row id>"; `exclude_id` is skipped; with a
# `variant` only alias rows of that variant count). Content fields name a key for every variant.
static func asset_refs(ns: String, key: String, exclude_id: String = "", variant: String = "") -> Array[String]:
	var refs: Array[String] = []
	if key.is_empty():
		return refs
	var fields: Dictionary = DictUtil.dict_of(KEY_FIELDS.get(ns, {}))
	for kind: Variant in fields:
		for e: Dictionary in ContentCatalog.entries(str(kind)):
			if str(e.get(str(fields[kind]), "")) == key:
				refs.append("%s:%s" % [str(kind), str(e.get("id", ""))])
	var world_fields: Dictionary = DictUtil.dict_of(WORLD_KEY_FIELDS.get(ns, {}))
	for world_id: String in WorldCatalog.world_ids():
		for kind: Variant in world_fields:
			for e: Dictionary in WorldCatalog.own_entries(world_id, str(kind)):
				if str(e.get(str(world_fields[kind]), "")) == key:
					refs.append("%s:%s:%s" % [world_id, str(kind), str(e.get("id", ""))])
	for row: Dictionary in AssetCatalog.entries():
		if variant.is_empty() or str(row.get("variant", "")) == variant:
			if str(row.get("id", "")) != exclude_id and str(row.get("namespace", "")) == ns and str(row.get("alias_of", "")) == key:
				refs.append("%s:%s" % [ASSETS_KIND, str(row.get("id", ""))])
	return refs

# "file:line" of every quoted occurrence of any id in tests, dojo scripts and seeds.
static func scan_files(ids: Array[String]) -> Array[String]:
	var hits: Array[String] = []
	for dir: String in SCAN_DIRS:
		for file: String in DirAccess.get_files_at(dir):
			if file.ends_with(".gd"):
				hits.append_array(_scan_text("%s/%s" % [dir, file], ids))
	return hits

static func _scan_text(path: String, ids: Array[String]) -> Array[String]:
	var hits: Array[String] = []
	var text: String = FileAccess.get_file_as_string(path)
	if text.is_empty():
		return hits
	var lines: PackedStringArray = text.split("\n")
	for i: int in range(lines.size()):
		for id: String in ids:
			if lines[i].contains("\"%s\"" % id):
				hits.append("%s:%d" % [path, i + 1])
				break
	return hits

# The ids a delete has to account for: the asset itself, or every rank id of a skill family.
static func ids_of(kind: String, id: String) -> Array[String]:
	var ids: Array[String] = [id]
	if kind == "skills":
		ids.clear()
		for row: Dictionary in ContentCatalog.family_rows(id):
			ids.append(str(row["id"]))
		if ids.is_empty():
			ids.append(id)
	return ids

# Every reference to `kind`/`id`: catalog entries (ContentCatalog.references_to), tests, dojo scripts, seeds, world files.
# Kind "assets" (an AssetCatalog row id "<ns>/<key>/<variant>") counts the content fields holding the key (asset_refs).
static func scan(kind: String, id: String) -> Array[String]:
	if kind == ASSETS_KIND:
		var row: Dictionary = AssetCatalog.get_entry(id)
		return asset_refs(str(row.get("namespace", "")), str(row.get("key", "")), id)
	var ids: Array[String] = ids_of(kind, id)
	var refs: Array[String] = []
	for each_id: String in ids:
		for user: String in ContentCatalog.references_to(kind, each_id):
			if not refs.has(user):
				refs.append(user)
	refs.append_array(scan_files(ids))
	if DirAccess.dir_exists_absolute(WORLD_DIR):
		for file: String in DirAccess.get_files_at(WORLD_DIR):
			if file.ends_with(".json"):
				refs.append_array(_scan_text("%s/%s" % [WORLD_DIR, file], ids))
	return refs

# The SAFE DELETE process. Returns {"ok": bool, "removed": bool, "refs": [...], "message": text}.
# Refuses (nothing changes) for basics, required and engine skills and while any reference exists; with `apply` and no
# reference it removes the asset inside the system-edit scope, saves and reports validate_all.
static func safe_delete(kind: String, id: String, apply: bool) -> Dictionary:
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

static func _result(ok: bool, removed: bool, refs: Array[String], message: String) -> Dictionary:
	return {"ok": ok, "removed": removed, "refs": refs, "message": message}
