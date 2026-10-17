class_name AssetPipelineTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Asset Pipeline Test Suite (S14, spec S14-L1)
# Part A (schema): AssetCatalog contract, manifest row validation (one bad row per message), key lists and the content
# option lists / art validator reading the registry, resolve order with a missing file, the export tool.
# Fixtures: AssetCatalog on a sandbox manifest whose asset root is a sandbox folder (files written there by the export
# function); the shipped manifest and catalog are only read.
# ==============================================================================

const TAG: String = "Assets"
static var SANDBOX: String = TestHelpers.sandbox("asset")

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func _has(errors: Array[String], needle: String) -> bool:
	return TestHelpers.any_contains(errors, needle)

# Points the catalog at an empty sandbox manifest whose asset root is SANDBOX/assets.
static func _use_sandbox() -> bool:
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/assets")
	AssetCatalog.use_path(SANDBOX + "/manifest.json", SANDBOX + "/assets")
	return AssetCatalog.file_path().begins_with("user://") and AssetCatalog.asset_root().begins_with("user://") and AssetCatalog.entries().is_empty()

static func _restore() -> void:
	AssetCatalog.use_path(AssetCatalog.DEFAULT_PATH)
	AssetRegistry.clear_warnings()
	AssetRegistry.set_quiet(false)
	TestHelpers.remove_sandbox(SANDBOX)

static func _row(ns: String, key: String, variant: String, path: String = "", alias: String = "") -> Dictionary:
	return {"namespace": ns, "key": key, "variant": variant, "path": path, "alias_of": alias}

# A file in the sandbox asset root holding `text` (a broken image).
static func _write_text(row_path: String, text: String) -> void:
	var real: String = AssetCatalog.file_of(row_path)
	DirAccess.make_dir_recursive_absolute(real.get_base_dir())
	var f := FileAccess.open(real, FileAccess.WRITE)
	if f != null:
		f.store_string(text)
		f.close()

# ------------------------------------------------------------------------------

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING ASSET PIPELINE TEST (S14-L1)            ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	AssetCatalog.use_path(AssetCatalog.DEFAULT_PATH)
	AssetRegistry.set_quiet(true)   # the broken-file checks count warnings; printing them would read as failures
	var ok: bool = true
	ok = _test_shipped() and ok
	ok = _test_key_lists() and ok
	var sandboxed: bool = _use_sandbox()
	ok = _report("Sandbox manifest and asset root under user://", {"sandboxed": sandboxed}) and ok
	if sandboxed:
		ok = _test_export() and ok
		ok = _test_contract() and ok
		ok = _test_messages() and ok
		ok = _test_alias_keys() and ok
		ok = _test_resolve_missing_file() and ok
	_restore()
	return ok

# Acceptance 10: the shipped manifest and the content catalog validate clean.
static func _test_shipped() -> bool:
	var manifest_ok: bool = AssetCatalog.validate_all().is_empty()
	var content_ok: bool = ContentCatalog.validate_all().is_empty()
	return _report("Shipped manifest validate_all {} and ContentCatalog.validate_all {}", {"manifest clean": manifest_ok, "content clean": content_ok, "never res:// in tests": AssetCatalog.file_path() == AssetCatalog.DEFAULT_PATH})

# Namespaces, variants, template keys and the content pickers / art validator reading the registry.
static func _test_key_lists() -> bool:
	var c: Dictionary = {}
	c["8 namespaces"] = AssetRegistry.namespaces().size() == 8
	var every_ns_has_variants: bool = true
	for ns: String in AssetRegistry.namespaces():
		every_ns_has_variants = every_ns_has_variants and not AssetRegistry.variants(ns).is_empty()
	c["every namespace has variants"] = every_ns_has_variants
	var class_keys_ok: bool = true
	for key: String in UnitArt.CLASS_KEYS:
		class_keys_ok = class_keys_ok and AssetRegistry.keys("unit").has(key)
	c["every CLASS_KEYS key in keys(unit)"] = class_keys_ok
	var tiles_ok: bool = true
	for name: String in Terrain.Type.keys():
		tiles_ok = tiles_ok and AssetRegistry.has_key("tile", name.to_lower())
	c["every terrain type is a tile key"] = tiles_ok and AssetRegistry.has_key("tile", "overlay_exit") and AssetRegistry.has_key("tile", "overlay_center")
	var status_ok: bool = true
	for id: String in StatusRegistry.ids():
		status_ok = status_ok and AssetRegistry.has_key("status", id)
	c["every status id is a status key"] = status_ok
	var family_ok: bool = not AssetRegistry.skill_family_ids().is_empty()
	for fam: String in AssetRegistry.skill_family_ids():
		family_ok = family_ok and AssetRegistry.has_key("skill", fam)
	c["every skill family is a skill key"] = family_ok
	var items_ok: bool = true
	for key: String in ItemArt.UNIQUE_ICONS.keys():
		items_ok = items_ok and AssetRegistry.has_key("item", key)
	c["unique icons are item keys"] = items_ok and AssetRegistry.has_key("item", "weapon")
	c["no templates for sound and ui"] = AssetRegistry.template_keys("sfx").is_empty() and AssetRegistry.template_keys("music").is_empty() and AssetRegistry.template_keys("ui").is_empty()
	c["keys sorted"] = AssetRegistry.keys("unit") == _sorted(AssetRegistry.keys("unit"))
	c["art options == keys(unit)"] = ContentSchema.options_of({"options_from": "art"}) == AssetRegistry.keys("unit")
	var inherit: Array[String] = [""]
	inherit.append_array(AssetRegistry.keys("unit"))
	c["art_inherit options == '' + keys(unit)"] = ContentSchema.options_of({"options_from": "art_inherit"}) == inherit
	c["world art_inherit options == '' + keys(unit)"] = WorldSchema.options_of({"options_from": "art_inherit"}) == inherit
	c["default sizes"] = AssetRegistry.default_size("unit", "token") == AssetRegistry.TOKEN_INNER.size and AssetRegistry.default_size("tile", "v3") == AssetRegistry.default_size("tile", "v0") and AssetRegistry.default_size("ui", "icon") == Vector2i.ZERO
	return _report("Key lists, option lists and default sizes read the registry", c)

static func _sorted(list: Array[String]) -> Array[String]:
	var out: Array[String] = list.duplicate()
	out.sort()
	return out

# Rule 8: the export writes the template PNG to the folder-pattern path at the default size.
static func _test_export() -> bool:
	var c: Dictionary = {}
	var cases: Array[Array] = [["unit", "warrior", "token"], ["unit", "warrior", "portrait"], ["item", "sword", "icon"], ["tile", "grass", "v1"], ["tile", "overlay_exit", "v0"]]
	for each: Array in cases:
		var ns: String = str(each[0])
		var key: String = str(each[1])
		var variant: String = str(each[2])
		var r: Dictionary = AssetRegistry.export_placeholder(ns, key, variant)
		var size: Vector2i = r["size"]
		var label: String = "%s/%s/%s" % [ns, key, variant]
		c[label + " written"] = str(r["error"]).is_empty() and FileAccess.file_exists(str(r["file"])) and str(r["file"]).begins_with("user://")
		c[label + " at its pattern path"] = AssetRegistry.pattern_paths(ns, key, variant).has(str(r["path"]))
		c[label + " default size"] = size == AssetRegistry.default_size(ns, variant)
	c["sound has no template"] = not str(AssetRegistry.export_placeholder("sfx", "hit", "stream")["error"]).is_empty()
	return _report("Export tool writes placeholders to the folder pattern (sandbox)", c)

# The QuestCatalog-style contract: upsert derives the id, nothing changes on error, save/reload, blocked saves, origin.
static func _test_contract() -> bool:
	var c: Dictionary = {}
	var path: String = AssetRegistry.pattern_paths("unit", "warrior", "token")[0]
	var given: Dictionary = _row("unit", "warrior", "", path)
	given["id"] = "something/else"
	var errs: Array[String] = AssetCatalog.upsert(given)
	var id: String = AssetCatalog.id_for("unit", "warrior", AssetRegistry.variants("unit")[0])
	c["upsert ok"] = errs.is_empty()
	c["id derived, variant defaults to the first"] = AssetCatalog.has(id) and str(AssetCatalog.get_entry(id)["variant"]) == AssetRegistry.variants("unit")[0]
	var stored: Dictionary = AssetCatalog.get_entry(id)
	c["defaults filled"] = DictUtil.array_of(stored["size"]).is_empty() and DictUtil.to_int(stored["frames"]) == 1 and str(stored["origin"]) == AssetCatalog.ORIGIN_USER and str(stored["note"]).is_empty()
	var before: int = AssetCatalog.entries().size()
	var bad: Array[String] = AssetCatalog.upsert(_row("unit", "Bad Key", "token", path))
	c["bad row refused, nothing changes"] = not bad.is_empty() and AssetCatalog.entries().size() == before
	c["dirty after upsert"] = AssetCatalog.is_dirty()
	c["save ok"] = AssetCatalog.save() == OK and not AssetCatalog.is_dirty()
	AssetCatalog.reload_from_disk()
	c["round trip"] = AssetCatalog.has(id) and AssetCatalog.ids().size() == before and AssetCatalog.validate_all().is_empty()
	# origin: new rows are user rows outside the system-edit scope; a system row refuses remove outside it
	var sys_row: Dictionary = _row("unit", "warrior", "portrait", AssetRegistry.pattern_paths("unit", "warrior", "portrait")[0])
	sys_row["origin"] = AssetCatalog.ORIGIN_SYSTEM
	AssetCatalog.upsert(sys_row)
	var sys_id: String = AssetCatalog.id_for("unit", "warrior", "portrait")
	c["new row outside the scope is user"] = str(AssetCatalog.get_entry(sys_id).get("origin", "")) == AssetCatalog.ORIGIN_USER
	AssetCatalog.begin_system_edit()
	AssetCatalog.upsert(sys_row)
	AssetCatalog.end_system_edit()
	c["inside the scope it keeps system"] = str(AssetCatalog.get_entry(sys_id).get("origin", "")) == AssetCatalog.ORIGIN_SYSTEM
	c["system row refuses remove"] = not AssetCatalog.remove(sys_id).is_empty() and AssetCatalog.has(sys_id)
	AssetCatalog.begin_system_edit()
	var why: String = AssetCatalog.remove(sys_id)
	AssetCatalog.end_system_edit()
	c["system row removed inside the scope"] = why.is_empty() and not AssetCatalog.has(sys_id)
	c["unknown id"] = not AssetCatalog.remove("unit/nope/token").is_empty()
	# a file with an unknown field loads with errors and refuses to save
	AssetCatalog.save()
	var raw: String = FileAccess.get_file_as_string(AssetCatalog.file_path()).replace("\"note\"", "\"bogus\"")
	var f := FileAccess.open(AssetCatalog.file_path(), FileAccess.WRITE)
	f.store_string(raw)
	f.close()
	AssetCatalog.reload_from_disk()
	c["unknown field -> load error, save blocked"] = not AssetCatalog.load_errors().is_empty() and AssetCatalog.save_blocked() and AssetCatalog.validate_all().has("load")
	_use_sandbox()
	return _report("AssetCatalog contract (id, defaults, save, origin, blocked file)", c)

# Acceptance 2: one bad row per field-table message.
static func _test_messages() -> bool:
	var c: Dictionary = {}
	var good_path: String = str(AssetRegistry.export_placeholder("item", "sword", "icon")["path"])
	AssetCatalog.upsert(_row("item", "sword", "icon", good_path))
	var bad_png: String = AssetRegistry.pattern_paths("item", "broken", "icon")[0]
	_write_text(bad_png, "not an image")
	var long_note: Dictionary = _row("item", "sword2", "icon", "", "sword")
	long_note["note"] = "x".repeat(AssetCatalog.NOTE_MAX + 1)
	var wrong_id: Dictionary = _row("item", "dagger", "icon", "", "sword")
	wrong_id["id"] = "item/other/icon"
	var big: Dictionary = _row("item", "dagger", "icon", "", "sword")
	big["size"] = [4, 4096]
	var frames: Dictionary = _row("item", "dagger", "icon", "", "sword")
	frames["frames"] = 2
	var cases: Dictionary = {
		"Unknown namespace 'fx'.": _row("fx", "slash", ""),
		"Key 'Bad-Key' must be snake_case (letters, digits, _; max 40).": _row("item", "Bad-Key", "icon", "", "sword"),
		"Variant 'portrait' is not valid for namespace item (allowed: icon).": _row("item", "dagger", "portrait", "", "sword"),
		"Path must start with res://assets/.": _row("item", "dagger", "icon", "user://dagger_icon.png"),
		"Path must be %s (folder pattern)." % AssetRegistry.pattern_text("item", "dagger", "icon"): _row("item", "dagger", "icon", "res://assets/art/item/other.png"),
		"File '%s' does not exist." % AssetRegistry.pattern_paths("item", "dagger", "icon")[0]: _row("item", "dagger", "icon", AssetRegistry.pattern_paths("item", "dagger", "icon")[0]),
		"File '%s' does not load as Texture2D." % bad_png: _row("item", "broken", "icon", bad_png),
		"Alias target 'nope' is not a known item key.": _row("item", "dagger", "icon", "", "nope"),
		"Set a path or an alias, not both.": _row("item", "dagger", "icon", good_path, "sword"),
		"Set a path or an alias.": _row("item", "dagger", "icon"),
		"Size must be two whole numbers 8-2048.": big,
		"frames must be 1 until sprite strips are built.": frames,
		"Asset 'item/sword/icon' already exists.": _row("item", "sword", "icon", "", "dagger"),
		"Id must be namespace/key/variant ('item/dagger/icon').": wrong_id,
		"Note must be at most %d characters." % AssetCatalog.NOTE_MAX: long_note,
	}
	for msg: String in cases:
		var errors: Array[String] = AssetCatalog.validate_entry(DictUtil.dict_of(cases[msg]))
		c[msg] = errors.has(msg)
	# alias chain: an alias row whose target is itself an alias row
	AssetCatalog.upsert(_row("item", "blade", "icon", "", "sword"))
	var chain: Array[String] = AssetCatalog.validate_entry(_row("item", "blade2", "icon", "", "blade"))
	c["Alias target 'blade' is an alias (one hop only)."] = chain.has("Alias target 'blade' is an alias (one hop only).")
	c["a good alias row is clean"] = AssetCatalog.validate_entry(_row("item", "dagger2", "icon", "", "sword")).is_empty()
	c["the file row is clean"] = AssetCatalog.validate_entry(AssetCatalog.get_entry("item/sword/icon"), "item/sword/icon").is_empty()
	_use_sandbox()
	return _report("validate_entry gives the exact message per rule", c)

# A manifest alias key becomes a unit key: pickers list it and the class art check accepts it (read-only on content).
static func _test_alias_keys() -> bool:
	var c: Dictionary = {}
	var errs: Array[String] = AssetCatalog.upsert(_row("unit", "qa_alias_unit", "token", "", UnitArt.CLASS_KEYS[0]))
	c["alias row stored"] = errs.is_empty()
	c["alias key in keys(unit)"] = AssetRegistry.keys("unit").has("qa_alias_unit") and AssetRegistry.has_key("unit", "qa_alias_unit")
	c["alias key in the art picker"] = ContentSchema.options_of({"options_from": "art"}).has("qa_alias_unit")
	var cls: Dictionary = ContentCatalog.entries("classes")[0].duplicate(true)
	cls["art"] = "qa_alias_unit"
	c["class art check accepts the alias key"] = not _has(ContentCatalog.validate_entry("classes", cls, str(cls["id"])), "Art '")
	cls["art"] = "qa_no_such_art"
	c["class art check refuses an unknown key"] = ContentCatalog.validate_entry("classes", cls, str(cls["id"])).has("Art 'qa_no_such_art' isn't a known token/portrait key.")
	var r: Dictionary = AssetRegistry.resolve("unit", "qa_alias_unit", "token")
	c["alias resolves to its target's template"] = str(r["source"]) == "template" and str(r["key"]) == UnitArt.CLASS_KEYS[0]
	_use_sandbox()
	return _report("Manifest keys feed the pickers and the art validator", c)

# Acceptance 3 (resolve part): a file row resolves to the file; with the file gone it resolves to the template and
# logs exactly one warning across two lookups.
static func _test_resolve_missing_file() -> bool:
	var c: Dictionary = {}
	var r0: Dictionary = AssetRegistry.export_placeholder("tile", "grass", "v0")
	AssetCatalog.upsert(_row("tile", "grass", "v0", str(r0["path"])))
	var hit: Dictionary = AssetRegistry.resolve("tile", "grass", "v0")
	c["row with a file -> file"] = str(hit["source"]) == "file" and str(hit["path"]) == str(r0["path"])
	c["tile v0 file serves v2"] = str(AssetRegistry.resolve("tile", "grass", "v2")["source"]) == "file"
	c["unknown key -> fallback key"] = str(AssetRegistry.resolve("tile", "qa_nope", "v0")["key"]) == str(AssetRegistry.FALLBACK_KEYS["tile"])
	AssetRegistry.clear_warnings()
	DirAccess.remove_absolute(str(r0["file"]))
	var a: Dictionary = AssetRegistry.resolve("tile", "grass", "v0")
	var b: Dictionary = AssetRegistry.resolve("tile", "grass", "v1")
	c["missing file -> template"] = str(a["source"]) == "template" and str(b["source"]) == "template"
	c["exactly one warning across two lookups"] = AssetRegistry.warning_count() == 1
	AssetRegistry.clear_warnings()
	_use_sandbox()
	return _report("Resolve order: file, template, fallback; a missing file warns once", c)
