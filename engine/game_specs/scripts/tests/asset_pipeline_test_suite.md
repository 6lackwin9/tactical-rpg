---
class: AssetPipelineTestSuite
extends: RefCounted
description: "Tactical RPG - Asset Pipeline Test Suite (S14, spec S14-L1) Part A (schema): AssetCatalog contract, manifest row validation (one bad row per message), key lists and the content option lists / art validator reading the registry, resolve order with a missing file, the export tool. Part B (resolution and art): every shipped key resolves at its size, UnitArt/TileArt delegation, skill and status icons, file textures (cache, alias, broken file, size warning), the team overlay of unit files, safe remove. Fixtures: AssetCatalog on a sandbox manifest whose asset root is a sandbox folder (files written there by the export function); the shipped manifest and catalog are only read."
---

# Properties
* `TAG`: String = "Assets" (const)
* `out`: Array[String] = []
* `head`: String = SkillRanks.head_of(row)
* `out`: Array[String] = list.duplicate()

# Methods

### `static _report(label: String, checks: Dictionary) -> bool`
```logic
return TestHelpers.report(TAG, label, checks)
```

### `static _has(errors: Array[String], needle: String) -> bool`
```logic
return TestHelpers.any_contains(errors, needle)
```

### `static _use_sandbox() -> bool`
```logic
TestHelpers.remove_sandbox(SANDBOX)
DirAccess.make_dir_recursive_absolute(SANDBOX + "/assets")
AssetCatalog.use_path(SANDBOX + "/manifest.json", SANDBOX + "/assets")
return AssetCatalog.file_path().begins_with("user://") and AssetCatalog.asset_root().begins_with("user://") and AssetCatalog.entries().is_empty()
```

### `static _restore() -> void`
```logic
AssetCatalog.use_path(AssetCatalog.DEFAULT_PATH)
AssetRegistry.clear_warnings()
AssetRegistry.set_quiet(false)
TestHelpers.remove_sandbox(SANDBOX)
```

### `static _row(ns: String, key: String, variant: String, path: String = "", alias: String = "") -> Dictionary`
```logic
return {"namespace": ns, "key": key, "variant": variant, "path": path, "alias_of": alias}
```

### `static _write_text(row_path: String, text: String) -> void`
```logic
var real: String = AssetCatalog.file_of(row_path)
DirAccess.make_dir_recursive_absolute(real.get_base_dir())
var f := FileAccess.open(real, FileAccess.WRITE)
if f != null:
	f.store_string(text)
	f.close()
```

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING ASSET PIPELINE TEST (S14-L1)            ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
AssetCatalog.use_path(AssetCatalog.DEFAULT_PATH)
AssetRegistry.set_quiet(true)   # the broken-file checks count warnings; printing them would read as failures
var ok: bool = true
ok = _test_shipped() and ok
ok = _test_key_lists() and ok
ok = _test_shipped_keys_resolve() and ok
ok = _test_delegation() and ok
ok = _test_icons() and ok
var sandboxed: bool = _use_sandbox()
ok = _report("Sandbox manifest and asset root under user://", {"sandboxed": sandboxed}) and ok
if sandboxed:
	ok = _test_export() and ok
	ok = _test_contract() and ok
	ok = _test_messages() and ok
	ok = _test_alias_keys() and ok
	ok = _test_resolve_missing_file() and ok
	ok = _test_file_textures() and ok
	ok = _test_team_overlay() and ok
	ok = _test_safe_remove() and ok
_restore()
return ok
```

### `static _test_shipped() -> bool`
```logic
var manifest_ok: bool = AssetCatalog.validate_all().is_empty()
var content_ok: bool = ContentCatalog.validate_all().is_empty()
return _report("Shipped manifest validate_all {} and ContentCatalog.validate_all {}", {"manifest clean": manifest_ok, "content clean": content_ok, "never res:// in tests": AssetCatalog.file_path() == AssetCatalog.DEFAULT_PATH})
```

### `static _test_key_lists() -> bool`
```logic
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
c["every terrain type is a tile key"] = tiles_ok and AssetRegistry.has_key("tile", "overlay_exit") and AssetRegistry.has_key("tile", "overlay_poi")
var status_ok: bool = true
for id: String in StatusRegistry.ids():
	status_ok = status_ok and AssetRegistry.has_key("status", id)
c["every status id is a status key"] = status_ok
var family_ok: bool = not AssetRegistry.skill_family_ids().is_empty()
for fam: String in AssetRegistry.skill_family_ids():
	family_ok = family_ok and AssetRegistry.has_key("skill", fam)
c["every skill family is a skill key"] = family_ok
_check_family_reload(c)
var items_ok: bool = true
for key: String in ItemArt.UNIQUE_ICONS.keys():
	items_ok = items_ok and AssetRegistry.has_key("item", key)
c["unique icons are item keys"] = items_ok and AssetRegistry.has_key("item", "weapon")
c["no templates for sound; ui templates are the map pins and badges (MapArt)"] = AssetRegistry.template_keys("sfx").is_empty() and AssetRegistry.template_keys("music").is_empty() and AssetRegistry.template_keys("ui") == MapArt.UI_KEYS
c["keys sorted"] = AssetRegistry.keys("unit") == _sorted(AssetRegistry.keys("unit"))
c["art options == keys(unit)"] = ContentSchema.options_of({"options_from": "art"}) == AssetRegistry.keys("unit")
var inherit: Array[String] = [""]
inherit.append_array(AssetRegistry.keys("unit"))
c["art_inherit options == '' + keys(unit)"] = ContentSchema.options_of({"options_from": "art_inherit"}) == inherit
c["world art_inherit options == '' + keys(unit)"] = WorldSchema.options_of({"options_from": "art_inherit"}) == inherit
c["default sizes"] = AssetRegistry.default_size("unit", "token") == AssetRegistry.TOKEN_INNER.size and AssetRegistry.default_size("tile", "v3") == AssetRegistry.default_size("tile", "v0") and AssetRegistry.default_size("ui", "icon") == Vector2i.ZERO
return _report("Key lists, option lists and default sizes read the registry", c)
```

### `static _check_family_reload(c: Dictionary) -> void`
```logic
var before: Array[String] = AssetRegistry.skill_family_ids()
AssetRegistry._family_list.append("__stale_family__")   # a stale cache would keep this
ContentCatalog.reload_from_disk()
var cleared: bool = AssetRegistry._family_stamp.is_empty()
var rebuilt: Array[String] = AssetRegistry.skill_family_ids()
c["content reload drops the skill family cache (stamp cleared)"] = cleared
c["rebuilt family list == fresh catalog heads == list before"] = rebuilt == _catalog_families() and rebuilt == before and not rebuilt.has("__stale_family__")
var stamped: bool = not AssetRegistry._family_stamp.is_empty()
ContentCatalog.reload_from_disk()
var cleared_again: bool = AssetRegistry._family_stamp.is_empty()
var again: Array[String] = AssetRegistry.skill_family_ids()
c["second reload, same root: cache cleared then rebuilt (stamp set again)"] = stamped and cleared_again and not AssetRegistry._family_stamp.is_empty() and again == _catalog_families()
```

### `static _test_export() -> bool`
```logic
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
```

### `static _test_contract() -> bool`
```logic
var c: Dictionary = {}
var path: String = AssetRegistry.pattern_paths("unit", "warrior", "token")[0]
var given: Dictionary = _row("unit", "warrior", "", path)
given["id"] = "something/else"
var errs: Array[String] = AssetCatalog.upsert(given)
var id: String = AssetCatalog.id_for("unit", "warrior", AssetRegistry.variants("unit")[0])
c["upsert ok"] = errs.is_empty()
c["id derived, variant defaults to the first"] = AssetCatalog.has(id) and str(AssetCatalog.get_entry(id)["variant"]) == AssetRegistry.variants("unit")[0]
var stored: Dictionary = AssetCatalog.get_entry(id)
c["defaults filled"] = DictUtil.array_of(stored["size"]).is_empty() and DictUtil.to_int(stored["frames"]) == 1 and str(stored.get("origin", "")) == AssetCatalog.ORIGIN_USER and str(stored.get("note", "")).is_empty()
var before: int = AssetCatalog.entries().size()
var bad: Array[String] = AssetCatalog.upsert(_row("unit", "Bad Key", "token", path))
c["bad row refused, nothing changes"] = not bad.is_empty() and AssetCatalog.entries().size() == before
c["dirty after upsert"] = AssetCatalog.is_dirty()
c["save ok"] = AssetCatalog.save() == OK and not AssetCatalog.is_dirty()
AssetCatalog.reload_from_disk()
c["round trip"] = AssetCatalog.has(id) and AssetCatalog.ids().size() == before and AssetCatalog.validate_all().is_empty()
# origin: new rows are user rows outside the system-edit scope; a system row refuses remove outside it
# (a key no content names, so only the origin rule decides the remove)
var sys_row: Dictionary = _row("unit", "qa_sys", "portrait", str(AssetRegistry.export_placeholder("unit", "qa_sys", "portrait")["path"]))
sys_row["origin"] = AssetCatalog.ORIGIN_SYSTEM
var sys_id: String = AssetCatalog.id_for("unit", "qa_sys", "portrait")
c["new row stored"] = AssetCatalog.upsert(sys_row).is_empty()
c["new row outside the scope is user"] = str(AssetCatalog.get_entry(sys_id).get("origin", "")) == AssetCatalog.ORIGIN_USER
c["explicit origin change outside the scope refused"] = TestHelpers.any_contains(AssetCatalog.upsert(sys_row), "promote / demote") and str(AssetCatalog.get_entry(sys_id).get("origin", "")) == AssetCatalog.ORIGIN_USER
AssetCatalog.begin_system_edit()
var scoped: Array[String] = AssetCatalog.upsert(sys_row)
AssetCatalog.end_system_edit()
c["inside the scope it becomes system"] = scoped.is_empty() and str(AssetCatalog.get_entry(sys_id).get("origin", "")) == AssetCatalog.ORIGIN_SYSTEM
var edit: Dictionary = sys_row.duplicate()
edit.erase("origin")
edit["note"] = "edited"
c["system row edit outside the scope refused"] = TestHelpers.any_contains(AssetCatalog.upsert(edit), "system-edit scope") and str(AssetCatalog.get_entry(sys_id).get("note", "")).is_empty()
var demoting: Dictionary = sys_row.duplicate()
demoting["origin"] = AssetCatalog.ORIGIN_USER
c["origin 'user' on a system row refused"] = not AssetCatalog.upsert(demoting).is_empty() and str(AssetCatalog.get_entry(sys_id).get("origin", "")) == AssetCatalog.ORIGIN_SYSTEM
var moved: Dictionary = _row("unit", "qa_sys", "token", "", UnitArt.CLASS_KEYS[0])
c["replace_id over a system row refused"] = not AssetCatalog.upsert(moved, sys_id).is_empty() and AssetCatalog.has(sys_id) and not AssetCatalog.has(AssetCatalog.id_for("unit", "qa_sys", "token"))
c["system row refuses remove"] = not AssetCatalog.remove(sys_id).is_empty() and AssetCatalog.has(sys_id)
c["demote / promote change the origin"] = AssetCatalog.demote(sys_id).is_empty() and str(AssetCatalog.get_entry(sys_id).get("origin", "")) == AssetCatalog.ORIGIN_USER and AssetCatalog.promote(sys_id).is_empty() and str(AssetCatalog.get_entry(sys_id).get("origin", "")) == AssetCatalog.ORIGIN_SYSTEM and not AssetCatalog.promote(sys_id).is_empty()
AssetCatalog.begin_system_edit()
var why: String = AssetCatalog.remove(sys_id)
AssetCatalog.end_system_edit()
c["system row removed inside the scope"] = why.is_empty() and not AssetCatalog.has(sys_id)
c["unknown id"] = not AssetCatalog.remove("unit/nope/token").is_empty()
# a file with an unknown field loads with errors and refuses to save
AssetCatalog.save()
var raw: String = FileAccess.get_file_as_string(AssetCatalog.file_path()).replace("\"note\"", "\"bogus\"")
var f := FileAccess.open(AssetCatalog.file_path(), FileAccess.WRITE)
c["manifest file reopened for writing"] = f != null
if f != null:
	f.store_string(raw)
	f.close()
AssetCatalog.reload_from_disk()
c["unknown field -> load error, save blocked"] = not AssetCatalog.load_errors().is_empty() and AssetCatalog.save_blocked() and AssetCatalog.validate_all().has("load")
_use_sandbox()
return _report("AssetCatalog contract (id, defaults, save, origin, blocked file)", c)
```

### `static _test_messages() -> bool`
```logic
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
	"An alias can't point at its own key ('dagger').": _row("item", "dagger", "icon", "", "dagger"),
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
```

### `static _test_alias_keys() -> bool`
```logic
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
```

### `static _test_resolve_missing_file() -> bool`
```logic
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
```

### `static _want_size(ns: String, variant: String) -> Vector2i`
```logic
if ns == "unit" and variant == "token":
	return AssetRegistry.TOKEN_SIZE
return AssetRegistry.default_size(ns, variant)
```

### `static _resolves(ns: String, key: String, variant: String, team: int = -1) -> bool`
```logic
var tex: Texture2D = AssetRegistry.texture(ns, key, variant, team)
return tex != null and Vector2i(tex.get_size()) == _want_size(ns, variant)
```

### `static _test_shipped_keys_resolve() -> bool`
```logic
var bad: Array[String] = []
var unit_keys: Array[String] = []
for e: Dictionary in ContentCatalog.entries("classes"):
	unit_keys.append(str(e.get("art", "")))
for e: Dictionary in ContentCatalog.entries("units"):
	unit_keys.append(ContentFactory.art_of(e))
for world_id: String in WorldCatalog.world_ids():
	for e: Dictionary in WorldCatalog.own_entries(world_id, "characters"):
		if not str(e.get("art", "")).is_empty():
			unit_keys.append(str(e["art"]))
for key: String in unit_keys:
	for variant: String in AssetRegistry.variants("unit"):
		for team: int in range(UnitArt.Team.size()):
			if not _resolves("unit", key, variant, team):
				bad.append("unit/%s/%s/%d" % [key, variant, team])
for e: Dictionary in ContentCatalog.entries("items"):
	var icon: String = str(e.get("icon_id", ""))
	if not icon.is_empty() and not _resolves("item", icon, "icon"):
		bad.append("item/" + icon)
for fam: String in AssetRegistry.skill_family_ids():
	if not _resolves("skill", fam, "icon"):
		bad.append("skill/" + fam)
for id: String in StatusRegistry.ids():
	if not _resolves("status", id, "icon"):
		bad.append("status/" + id)
for name: String in Terrain.Type.keys():
	for variant: String in AssetRegistry.variants("tile"):
		if not _resolves("tile", name.to_lower(), variant):
			bad.append("tile/%s/%s" % [name.to_lower(), variant])
var c: Dictionary = {"every shipped key resolves at its size (%s)" % ", ".join(bad.slice(0, 5)): bad.is_empty(), "keys collected": unit_keys.size() >= ContentCatalog.entries("classes").size()}
c["unknown key -> fallback texture, never null"] = _resolves("unit", "qa_no_such_key", "token", 0) and _resolves("item", "qa_no_such_key", "icon") and _resolves("tile", "qa_no_such_key", "v0") and _resolves("skill", "qa_no_such_key", "icon") and _resolves("status", "qa_no_such_key", "icon")
c["ui blank texture"] = AssetRegistry.texture("ui", "qa_menu", "background") != null
c["sound unresolved -> null stream, no texture"] = AssetRegistry.stream("sfx", "qa_hit") == null and AssetRegistry.texture("sfx", "qa_hit", "stream") == null
return _report("Every shipped art key resolves (shipped manifest)", c)
```

### `static _test_delegation() -> bool`
```logic
var c: Dictionary = {}
var same_tokens: bool = true
for key: String in UnitArt.CLASS_KEYS:
	if not AssetCatalog.keys_of("unit").has(key):
		same_tokens = same_tokens and UnitArt.token(key, UnitArt.Team.HOSTILE) == UnitArt.template_token(key, UnitArt.Team.HOSTILE) and UnitArt.portrait(key, UnitArt.Team.FRIENDLY) == UnitArt.template_portrait(key, UnitArt.Team.FRIENDLY)
c["token / portrait == template for template-only keys"] = same_tokens
c["unknown class draws the unit template"] = UnitArt.token("qa_no_such_key", UnitArt.Team.NEUTRAL) == UnitArt.template_token("unit", UnitArt.Team.NEUTRAL)
c["normalize_class unknown -> unit"] = UnitArt.normalize_class("no_such_key") == "unit"
c["normalize_class keeps every class key"] = UnitArt.CLASS_KEYS.all(func(k: String) -> bool: return UnitArt.normalize_class(k) == k)
c["tile == template"] = TileArt.tile(Terrain.Type.GRASS, 1) == TileArt.template_tile(Terrain.Type.GRASS, 1) or AssetCatalog.keys_of("tile").has("grass")
var tile_keys: Array = AssetCatalog.keys_of("tile")
for kind: String in ["exit", "poi"]:
	if tile_keys.has("overlay_" + kind):
		var res: Dictionary = AssetRegistry.resolve("tile", "overlay_" + kind, "v0")
		c["overlay_%s row: file, not the template" % kind] = str(res["source"]) == "file" and TileArt.overlay(kind) != TileArt.template_overlay(kind) and AssetRegistry.file_exists(str(res["path"]))
c["overlays == templates"] = (TileArt.overlay("exit") == TileArt.template_overlay("exit") or tile_keys.has("overlay_exit")) and (TileArt.overlay("poi") == TileArt.template_overlay("poi") or tile_keys.has("overlay_poi"))
var t1: Texture2D = UnitArt.token("warrior", UnitArt.Team.FRIENDLY)
var t2: Texture2D = UnitArt.token("warrior", UnitArt.Team.FRIENDLY)
SvgRaster.clear_cache()
var t3: Texture2D = UnitArt.token("warrior", UnitArt.Team.FRIENDLY)
c["cached: same instance; clear_cache rebuilds"] = t1 == t2 and t3 != t1 and t3 != null
return _report("UnitArt / TileArt delegate to the registry (identical without rows)", c)
```

### `static _test_icons() -> bool`
```logic
var c: Dictionary = {}
var glyphs_ok: bool = true
for id: String in StatusRegistry.ids():
	glyphs_ok = glyphs_ok and StatusArt.GLYPHS.has(id)
c["every status id has its own glyph"] = glyphs_ok
var distinct: Dictionary = {}
for id: String in StatusArt.GLYPHS:
	distinct[str(StatusArt.GLYPHS[id])] = true
c["status glyphs are distinct"] = distinct.size() == StatusArt.GLYPHS.size()
c["status disc by kind"] = StatusArt.icon_svg(StatusRegistry.ids_where(StatusRegistry.KIND_BUFF)[0]).contains(StatusArt.BUFF_COLOR) and StatusArt.icon_svg(StatusRegistry.ids_where(StatusRegistry.KIND_DEBUFF)[0]).contains(StatusArt.DEBUFF_COLOR)
c["status without a glyph -> kind arrow"] = StatusArt.glyph_of("qa_new_status") == StatusArt.ARROW_DOWN
var skills_ok: bool = true
for fam: String in AssetRegistry.skill_family_ids():
	var tex: Texture2D = SkillArt.icon(fam)
	skills_ok = skills_ok and tex != null and Vector2i(tex.get_size()) == AssetRegistry.default_size("skill", "icon")
c["every shipped family icon is 64x64"] = skills_ok
var every_kind_coloured: bool = true
for fam: String in AssetRegistry.skill_family_ids():
	var kind: String = str(ContentCatalog.get_entry("skills", fam).get("kind", ""))
	every_kind_coloured = every_kind_coloured and SkillArt.KIND_COLORS.has(kind) and SkillArt.icon_svg(fam).contains(str(SkillArt.KIND_COLORS[kind]))
c["family disc coloured by its kind"] = every_kind_coloured
c["every kind has a glyph path"] = SkillArt.KIND_COLORS.keys().all(func(k: Variant) -> bool: return SkillArt.GLYPH_PATHS.has(str(SkillArt.KIND_GLYPHS.get(k, ""))))
c["unknown family -> the skill fallback look"] = SkillArt.icon_svg("qa_no_such_family") == SkillArt.icon_svg(str(AssetRegistry.FALLBACK_KEYS["skill"]))
c["no text in icon SVG"] = not SkillArt.icon_svg(AssetRegistry.skill_family_ids()[0]).contains("<text") and not StatusArt.icon_svg(StatusRegistry.ids()[0]).contains("<text")
return _report("Skill and status placeholder icons", c)
```

### `static _test_file_textures() -> bool`
```logic
var c: Dictionary = {}
var sword: Dictionary = AssetRegistry.export_placeholder("item", "sword", "icon")
AssetCatalog.upsert(_row("item", "sword", "icon", str(sword["path"])))
AssetCatalog.upsert(_row("item", "qa_blade", "icon", "", "sword"))
var file_tex: Texture2D = AssetRegistry.texture("item", "sword", "icon")
c["row with a file -> file texture"] = str(AssetRegistry.resolve("item", "sword", "icon")["source"]) == "file" and file_tex != null and file_tex != AssetRegistry.texture("item", "dagger", "icon")
var alias_tex: Texture2D = AssetRegistry.texture("item", "qa_blade", "icon")
c["alias -> the target's image data"] = alias_tex != null and alias_tex.get_image().get_data() == file_tex.get_image().get_data()
c["same arguments -> same instance"] = AssetRegistry.texture("item", "sword", "icon") == file_tex
SvgRaster.clear_cache()
var rebuilt: Texture2D = AssetRegistry.texture("item", "sword", "icon")
c["clear_cache -> a new instance"] = rebuilt != null and rebuilt != file_tex
# a file that exists but does not load: template texture, exactly one warning across two lookups
_write_text(str(sword["path"]), "not an image")
SvgRaster.clear_cache()
var a: Texture2D = AssetRegistry.texture("item", "sword", "icon")
var b: Texture2D = AssetRegistry.texture("item", "sword", "icon")
c["broken file -> template texture"] = a != null and a == b and Vector2i(a.get_size()) == AssetRegistry.default_size("item", "icon")
c["broken file -> one warning"] = AssetRegistry.warning_count() == 1
SvgRaster.clear_cache()
c["clear_cache forgets the warnings"] = AssetRegistry.warning_count() == 0
# a file whose size differs from the row size: a warning, never an error
var grass: Dictionary = AssetRegistry.export_placeholder("tile", "grass", "v0")
var sized: Dictionary = _row("tile", "grass", "v0", str(grass["path"]))
sized["size"] = [64, 64]
c["size mismatch is valid"] = AssetCatalog.validate_entry(sized).is_empty()
c["size mismatch warns"] = not AssetCatalog.warnings_for(sized).is_empty() and AssetCatalog.warnings_for(_row("tile", "grass", "v0", str(grass["path"]))).is_empty()
_use_sandbox()
return _report("File textures: cache, alias, broken file, size warning", c)
```

### `static _solid_png(row_path: String, size: Vector2i, color: Color) -> void`
```logic
var img := Image.create(size.x, size.y, false, Image.FORMAT_RGBA8)
img.fill(color)
var real: String = AssetCatalog.file_of(row_path)
DirAccess.make_dir_recursive_absolute(real.get_base_dir())
img.save_png(real)
```

### `static _test_team_overlay() -> bool`
```logic
var c: Dictionary = {}
var token_path: String = AssetRegistry.pattern_paths("unit", "qa_solid", "token")[0]
var portrait_path: String = AssetRegistry.pattern_paths("unit", "qa_solid", "portrait")[0]
_solid_png(token_path, Vector2i(52, 52), Color(0.2, 0.8, 0.4))   # half size: resized to the inner square
_solid_png(portrait_path, AssetRegistry.default_size("unit", "portrait"), Color(0.2, 0.8, 0.4, 0.5))
c["rows stored"] = AssetCatalog.upsert(_row("unit", "qa_solid", "token", token_path)).is_empty() and AssetCatalog.upsert(_row("unit", "qa_solid", "portrait", portrait_path)).is_empty()
var imgs: Array[Image] = []
var portraits: Array[Image] = []
for team: int in range(UnitArt.Team.size()):
	var tex: Texture2D = UnitArt.token("qa_solid", team as UnitArt.Team)
	var por: Texture2D = UnitArt.portrait("qa_solid", team as UnitArt.Team)
	c["team %d token composed at 128" % team] = tex != null and Vector2i(tex.get_size()) == AssetRegistry.TOKEN_SIZE
	c["team %d portrait at 240" % team] = por != null and Vector2i(por.get_size()) == AssetRegistry.default_size("unit", "portrait")
	imgs.append(tex.get_image() if tex != null else Image.create(1, 1, false, Image.FORMAT_RGBA8))
	portraits.append(por.get_image() if por != null else Image.create(1, 1, false, Image.FORMAT_RGBA8))
if imgs.size() == 3 and imgs[0].get_size() == AssetRegistry.TOKEN_SIZE:
	var inner: Rect2i = AssetRegistry.TOKEN_INNER
	var inside: Array[Vector2i] = [Vector2i(64, 64), inner.position, inner.end - Vector2i.ONE]
	var outside: Array[Vector2i] = [Vector2i(64, 10), Vector2i(10, 64), Vector2i(2, 64), Vector2i(64, 2)]
	var equal_inside: bool = true
	for p: Vector2i in inside:
		equal_inside = equal_inside and imgs[0].get_pixelv(p) == imgs[1].get_pixelv(p) and imgs[1].get_pixelv(p) == imgs[2].get_pixelv(p) and imgs[0].get_pixelv(p).is_equal_approx(Color(0.2, 0.8, 0.4))
	c["inside the inner square: the file, equal for all teams"] = equal_inside
	var pairs: Array[Vector2i] = [Vector2i(0, 1), Vector2i(1, 2), Vector2i(0, 2)]
	for pair: Vector2i in pairs:
		var differs: bool = false
		for p: Vector2i in outside:
			differs = differs or imgs[pair.x].get_pixelv(p) != imgs[pair.y].get_pixelv(p)
		c["teams %d/%d differ on the rim" % [pair.x, pair.y]] = differs
	c["portrait backdrop shows the team"] = portraits[0].get_pixel(12, 120) != portraits[1].get_pixel(12, 120)
c["one row, one key"] = AssetRegistry.keys("unit").has("qa_solid") and UnitArt.normalize_class("qa_solid") == "qa_solid"
_use_sandbox()
return _report("Team overlay: one unit file serves every team", c)
```

### `static _rule7(id: String, refs: Array[String]) -> String`
```logic
return "Asset '%s' is used by %s; remove the references first." % [id, ", ".join(refs)]
```

### `static _test_safe_remove() -> bool`
```logic
var c: Dictionary = {}
# template key named by content: removal succeeds, the art falls back to the template
var first_class: Dictionary = ContentCatalog.entries("classes")[0]
var used: String = str(first_class.get("art", ""))
var exported: Dictionary = AssetRegistry.export_placeholder("unit", used, "token")
c["template-key row stored"] = AssetCatalog.upsert(_row("unit", used, "token", str(exported["path"]))).is_empty()
var used_id: String = AssetCatalog.id_for("unit", used, "token")
var refs: Array[String] = AssetScan.asset_refs("unit", used, used_id)
c["content reference found"] = refs.has("classes:%s" % str(first_class["id"]))
c["scan('assets', id) == asset_refs"] = AssetScan.scan(AssetScan.ASSETS_KIND, used_id) == refs
c["template key: removed although named"] = AssetRegistry.has_template("unit", used) and AssetCatalog.remove(used_id).is_empty() and not AssetCatalog.has(used_id)
c["template key: back to the template"] = str(AssetRegistry.resolve("unit", used, "token")["source"]) == "template"
# manifest-only key, per variant: a portrait alias row blocks the portrait row only
for variant: String in AssetRegistry.variants("unit"):
	var path: String = str(AssetRegistry.export_placeholder("unit", "qa_only", variant)["path"])
	c["qa_only %s stored" % variant] = AssetCatalog.upsert(_row("unit", "qa_only", variant, path)).is_empty()
c["alias row stored"] = AssetCatalog.upsert(_row("unit", "qa_only_alias", "portrait", "", "qa_only")).is_empty()
var portrait_id: String = AssetCatalog.id_for("unit", "qa_only", "portrait")
var alias_id: String = AssetCatalog.id_for("unit", "qa_only_alias", "portrait")
var alias_refs: Array[String] = ["%s:%s" % [AssetScan.ASSETS_KIND, alias_id]]
c["aliased variant refused with the rule 7 message"] = AssetCatalog.remove(portrait_id) == _rule7(portrait_id, alias_refs) and AssetCatalog.has(portrait_id)
c["the other variant is free"] = AssetCatalog.remove(AssetCatalog.id_for("unit", "qa_only", "token")).is_empty()
c["moving the aliased row to another key refused"] = AssetCatalog.upsert(_row("unit", "qa_moved", "portrait", "", UnitArt.CLASS_KEYS[0]), portrait_id).has(_rule7(portrait_id, alias_refs)) and AssetCatalog.has(portrait_id)
c["unreferenced alias row removed"] = AssetCatalog.remove(alias_id).is_empty() and not AssetCatalog.has(alias_id)
c["then the target row is removed"] = AssetCatalog.remove(portrait_id).is_empty() and not AssetCatalog.has(portrait_id)
# manifest-only key named by content (an item family's `art`, on a content fork)
var fork_path: String = TestHelpers.sandbox("asset_content")
TestHelpers.fork_migrated(fork_path)
var families: Array[Dictionary] = ContentCatalog.entries("item_families")
c["content fork has item families"] = ContentCatalog.get_root() == fork_path and not families.is_empty()
if not families.is_empty():
	var icon_path: String = str(AssetRegistry.export_placeholder("item", "sword", "icon")["path"])
	var only_path: String = AssetRegistry.pattern_paths("item", "qa_family_icon", "icon")[0]
	c["family icon file copied"] = DirAccess.copy_absolute(AssetCatalog.file_of(icon_path), AssetCatalog.file_of(only_path)) == OK
	c["family icon row stored"] = AssetCatalog.upsert(_row("item", "qa_family_icon", "icon", only_path)).is_empty()
	var fam: Dictionary = families[0].duplicate(true)
	fam["art"] = "qa_family_icon"
	c["family art set"] = ContentCatalog.upsert("item_families", fam).is_empty()
	var only_id: String = AssetCatalog.id_for("item", "qa_family_icon", "icon")
	var fam_refs: Array[String] = AssetScan.asset_refs("item", "qa_family_icon", only_id)
	c["item_families art counts as a reference"] = fam_refs.has("item_families:%s" % str(fam["id"]))
	c["named manifest-only key refused"] = AssetCatalog.remove(only_id) == _rule7(only_id, fam_refs) and AssetCatalog.has(only_id)
TestHelpers.cleanup(fork_path)
_use_sandbox()
return _report("Safe remove of manifest rows (rule 7 option A, per variant)", c)
```

