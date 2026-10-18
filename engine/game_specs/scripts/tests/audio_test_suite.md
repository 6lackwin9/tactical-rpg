---
class: AudioTestSuite
extends: RefCounted
description: "Tactical RPG - Audio Test Suite (S14, spec S14-L3) Bus layout, SoundBank silent fallback and warn-once, the headless AudioDirector (no voices), SettingsFile audio keys (defaults, clamp, round trip, untouched by save slot deletes), a sandbox manifest row resolving to a generated wav, and the preset -> sound key table; MUSIC1-L: the MusicDirector chooser (priority, ties, live owners, unresolved contexts, release), the context fallback chain (region, parent, world, lineage, parent context) and the shipped music rows. Fixtures: sandbox settings file, save dir, manifest and worlds."
---

# Properties
* `TAG`: String = "Audio" (const)
* `folder`: String = AssetRegistry.folder_of("music")
* `src`: String = ""
* `out`: Array[String] = []
* `row_path`: String = "%s/%s.ogg" % [folder, key]
* `real`: String = AssetCatalog.file_of(row_path)
* `fa`: Variant

# Methods

### `static _report(label: String, checks: Dictionary) -> bool`
```logic
return TestHelpers.report(TAG, label, checks)
```

### `static run(tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING AUDIO TEST (S14-L3)                     ===")
print("=======================================================")
TestHelpers.remove_sandbox(SANDBOX)
DirAccess.make_dir_recursive_absolute(SANDBOX + "/assets")
var ok: bool = true
ok = _test_buses() and ok
ok = _test_bank() and ok
ok = _test_director(tree_parent) and ok
ok = _test_settings() and ok
ok = _test_manifest_stream() and ok
ok = _test_event_table() and ok
ok = _test_music_choice(tree_parent) and ok
ok = _test_music_api(tree_parent) and ok
ok = _test_music_chain() and ok
ok = _test_music_shipped() and ok
MusicDirector.clear_requests()
WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
SettingsFile.use_path()
SaveGame.reset_base_dir()
AssetCatalog.use_path(AssetCatalog.DEFAULT_PATH)
AssetRegistry.clear_warnings()
SoundBank.clear_warned()
SoundBank.set_quiet(false)
TestHelpers.remove_sandbox(SANDBOX)
print("  [Audio] Audio suite: %s" % ("PASS" if ok else "FAIL"))
return ok
```

### `static _test_buses() -> bool`
```logic
var checks: Dictionary = {"Master": AudioServer.get_bus_index("Master") == 0}
for bus: String in ["Music", "SFX", "UI"]:
	var idx: int = AudioServer.get_bus_index(bus)
	checks[bus] = idx > 0 and AudioServer.get_bus_send(idx) == &"Master"
return _report("Bus layout: Master, Music, SFX, UI send to Master", checks)
```

### `static _test_bank() -> bool`
```logic
SoundBank.clear_warned()
SoundBank.set_quiet(true)
var first: AudioStream = SoundBank.stream("no_such_key")
var second: AudioStream = SoundBank.stream("no_such_key")
var checks: Dictionary = {
	"unknown key is silent": first == null and second == null and not SoundBank.has("no_such_key"),
	"warns once": SoundBank.warning_count() == 1,
}
SoundBank.clear_warned()
checks["clear_warned resets"] = SoundBank.warning_count() == 0
SoundBank.stream("no_such_key")
checks["warns again after clear"] = SoundBank.warning_count() == 1
SoundBank.clear_warned()
return _report("SoundBank: missing key is null and warns once", checks)
```

### `static _test_director(tree_parent: Node) -> bool`
```logic
var baseline: AudioDirector = AudioDirector.active   # the host scene may own a director already
var director := AudioDirector.new()
if tree_parent != null:
	tree_parent.add_child(director)
director.play_sfx("hit")
director.ui("click")
AudioDirector.ui_event("click")
var presenter := CombatPresenter.new()
director.add_child(presenter)
director.connect_presenter(presenter)
presenter.fx_played.emit("hit", ActionLogEntry.new())
var checks: Dictionary = {
	"disabled headless": not AudioDirector.enabled(),
	"no voices": director.voice_count() == 0,
	"presenter connected": presenter.fx_played.get_connections().size() == 1,
	"nothing looked up": SoundBank.warning_count() == 0,
	"active while in tree": tree_parent == null or AudioDirector.active == director,
}
if director.is_inside_tree():
	tree_parent.remove_child(director)
checks["active cleared on exit"] = tree_parent == null or AudioDirector.active == baseline
director.free()
if tree_parent != null:
	var a := AudioDirector.new()
	var b := AudioDirector.new()
	tree_parent.add_child(a)
	tree_parent.add_child(b)
	checks["newest is active"] = AudioDirector.active == b
	tree_parent.remove_child(b)
	checks["older director active again after the newer exits"] = AudioDirector.active == a
	tree_parent.remove_child(a)
	checks["none of ours left gives the baseline"] = AudioDirector.active == baseline
	a.free()
	b.free()
return _report("AudioDirector headless: no voices, play is a no-op", checks)
```

### `static _test_settings() -> bool`
```logic
var cfg_path: String = SANDBOX + "/settings.cfg"
SettingsFile.use_path(cfg_path)
var checks: Dictionary = {}
checks["defaults when missing"] = (
	is_equal_approx(SettingsFile.get_float("audio/master"), 1.0) and is_equal_approx(SettingsFile.get_float("audio/music"), 0.8)
	and is_equal_approx(SettingsFile.get_float("audio/sfx"), 0.8) and is_equal_approx(SettingsFile.get_float("audio/ui"), 0.8)
)
SettingsFile.set_value("audio/sfx", 1.7)
checks["clamps to 1.0"] = is_equal_approx(SettingsFile.get_float("audio/sfx"), 1.0)
SettingsFile.set_value("audio/music", -2)
checks["clamps to 0.0"] = is_equal_approx(SettingsFile.get_float("audio/music"), 0.0)
SettingsFile.set_value("audio/ui", 0.35)
checks["saved"] = SettingsFile.save() == OK and FileAccess.file_exists(cfg_path)
SettingsFile.use_path(cfg_path)   # drops the in-memory file
SettingsFile.load_file()
checks["round trip"] = is_equal_approx(SettingsFile.get_float("audio/ui"), 0.35) and is_equal_approx(SettingsFile.get_float("audio/sfx"), 1.0)
var bad := ConfigFile.new()
bad.set_value("audio", "master", "loud")
bad.save(cfg_path)
SettingsFile.use_path(cfg_path)
checks["wrong type gives the default"] = is_equal_approx(SettingsFile.get_float("audio/master"), 1.0)
SettingsFile.set_value("audio/ui", 0.35)
SettingsFile.save()
var before: String = FileAccess.get_file_as_string(cfg_path)
SaveGame.set_base_dir(SANDBOX + "/saves")
var slot_file: String = SaveGame.path_for(1)
DirAccess.make_dir_recursive_absolute(slot_file.get_base_dir())
var f := FileAccess.open(slot_file, FileAccess.WRITE)
if f != null:
	f.store_string("{}")
	f.close()
SaveGame.delete(1)
checks["save slot delete leaves settings"] = FileAccess.get_file_as_string(cfg_path) == before and not FileAccess.file_exists(slot_file)
SaveGame.reset_base_dir()
SettingsFile.use_path()
return _report("SettingsFile audio keys: defaults, clamp, round trip", checks)
```

### `static _test_manifest_stream() -> bool`
```logic
AssetCatalog.use_path(SANDBOX + "/manifest.json", SANDBOX + "/assets")
var row_path: String = "%s/hit.wav" % AssetRegistry.folder_of("sfx")
var write_err: String = PlaceholderSfx.write(AssetCatalog.file_of(row_path), "noise", 0.0, 0.0, 80, "decay", 0.0)
var errs: Array[String] = AssetCatalog.upsert({"namespace": "sfx", "key": "hit", "variant": "stream", "path": row_path, "alias_of": ""})
var s: AudioStream = SoundBank.stream("hit")
var wav: AudioStreamWAV = s as AudioStreamWAV
var checks: Dictionary = {
	"tool wrote the file": write_err.is_empty(),
	"row stored": errs.is_empty(),
	"stream is a wav": wav != null,
	"80 ms long": wav != null and absf(wav.get_length() - 0.08) < 0.005,
	"bad wave refused": not PlaceholderSfx.write(AssetCatalog.file_of(row_path), "saw", 1.0, 1.0, 10, "decay", 0.0).is_empty(),
}
AssetCatalog.use_path(AssetCatalog.DEFAULT_PATH)
return _report("Manifest sfx row resolves to the generated wav", checks)
```

### `static _test_event_table() -> bool`
```logic
var checks: Dictionary = {}
for id: Variant in FxPresets.PRESETS:
	checks["preset %s mapped" % id] = SoundEvents.FOR_PRESET.has(id)
for id: Variant in SoundEvents.FOR_PRESET:
	var key: String = str(SoundEvents.FOR_PRESET[id])
	if not key.is_empty():
		checks["%s -> %s is an event key" % [id, key]] = SoundEvents.KEYS.has(key)
var crit := ActionLogEntry.new(1, 1, "ATTACK", "A", "B", "", 9, "CRIT")
checks["crit replaces hit"] = SoundEvents.key_for("hit", crit) == "crit"
var stun := ActionLogEntry.new(1, 1, "ATTACK", "A", "B", "", 0, StatusEffects.display_name("stun").to_upper())
var haste := ActionLogEntry.new(1, 1, "BUFF", "A", "B", "", 0, StatusEffects.display_name("haste").to_upper())
checks["debuff status plays debuff"] = SoundEvents.key_for("status", stun) == "debuff"
checks["buff status is silent"] = SoundEvents.key_for("status", haste).is_empty()
return _report("Preset -> sound key table covers every preset", checks)
```

### `static _req(owner: Node, context: String, priority: int, seq: int) -> Dictionary`
```logic
return {"id": owner.get_instance_id(), "context": context, "priority": priority, "world": "", "location": "", "seq": seq}
```

### `static _fake_track(context: String, _world: String, _location: String) -> String`
```logic
return "" if context == "nope" else context
```

### `static _test_music_choice(tree_parent: Node) -> bool`
```logic
if tree_parent == null:
	return TestHelpers.report_ok(TAG, "MusicDirector chooser (skipped: no tree)", true)
var fake: Callable = _fake_track
var menu := Control.new()
var field := Control.new()
var field2 := Control.new()
var hidden_map := Control.new()
var battle := Node.new()   # a BattleScreen is a plain Node
var hidden_ui := Control.new()
var loose := Node.new()    # never enters the tree
for n: Node in [menu, field, field2, hidden_map, hidden_ui]:
	tree_parent.add_child(n)
hidden_map.add_child(battle)
hidden_map.visible = false
hidden_ui.visible = false
var base: Array[Dictionary] = [_req(menu, "m", MusicDirector.MENU, 1), _req(field, "f", MusicDirector.FIELD, 2)]
var checks: Dictionary = {"priority: field over menu": MusicDirector.choose(base, fake) == "f"}
var with_battle: Array[Dictionary] = base.duplicate()
with_battle.append(_req(battle, "c", MusicDirector.COMBAT, 3))
checks["plain Node under a hidden map counts"] = MusicDirector.choose(with_battle, fake) == "c"
var tie: Array[Dictionary] = base.duplicate()
tie.append(_req(field2, "f2", MusicDirector.FIELD, 4))
checks["tie: newest wins"] = MusicDirector.choose(tie, fake) == "f2"
tie[2]["seq"] = 0
checks["tie: older loses"] = MusicDirector.choose(tie, fake) == "f"
var hid: Array[Dictionary] = base.duplicate()
hid.append(_req(hidden_ui, "h", MusicDirector.COMBAT, 5))
checks["invisible CanvasItem ignored"] = MusicDirector.choose(hid, fake) == "f"
var out: Array[Dictionary] = base.duplicate()
out.append(_req(loose, "o", MusicDirector.COMBAT, 6))
checks["owner outside the tree ignored"] = MusicDirector.choose(out, fake) == "f"
var unres: Array[Dictionary] = base.duplicate()
unres.append(_req(field2, "nope", MusicDirector.COMBAT, 7))
checks["unresolved context ignored"] = MusicDirector.choose(unres, fake) == "f"
var empty: Array[Dictionary] = []
checks["no requests: silence"] = MusicDirector.choose(empty, fake).is_empty()
var gone := Control.new()
tree_parent.add_child(gone)
var freed: Array[Dictionary] = base.duplicate()
freed.append(_req(gone, "g", MusicDirector.COMBAT, 8))
checks["live before free"] = MusicDirector.choose(freed, fake) == "g"
tree_parent.remove_child(gone)
gone.free()
checks["freed owner ignored"] = MusicDirector.choose(freed, fake) == "f"
for n: Node in [menu, field, field2, hidden_map, hidden_ui]:
	tree_parent.remove_child(n)
	n.free()
checks["all owners gone: silence"] = MusicDirector.choose(with_battle, fake).is_empty()
loose.free()
return _report("MusicDirector chooser: priority, ties, live owners, unresolved", checks)
```

### `static _test_music_api(tree_parent: Node) -> bool`
```logic
if tree_parent == null:
	return TestHelpers.report_ok(TAG, "MusicDirector request / release (skipped: no tree)", true)
MusicDirector.clear_requests()
var menu := Control.new()
var battle := Node.new()
tree_parent.add_child(menu)
tree_parent.add_child(battle)
MusicDirector.request(menu, MusicDirector.CTX_MENU, MusicDirector.MENU)
var checks: Dictionary = {"menu context plays the menu theme": MusicDirector.wanted_key() == MusicDirector.MENU_KEY}
MusicDirector.request(battle, MusicDirector.CTX_COMBAT, MusicDirector.COMBAT)
checks["combat over menu"] = MusicDirector.wanted_key() == "combat"
MusicDirector.request(battle, MusicDirector.CTX_BOSS, MusicDirector.COMBAT)
checks["same owner replaces"] = MusicDirector.requests().size() == 2
checks["boss plays boss, else combat"] = MusicDirector.wanted_key() == ("boss" if MusicDirector.has_track("boss") else "combat")
MusicDirector.release(battle)
checks["release falls back"] = MusicDirector.wanted_key() == MusicDirector.MENU_KEY and MusicDirector.requests().size() == 1
MusicDirector.request(battle, "no_such_context", MusicDirector.COMBAT)
checks["unknown context ignored"] = MusicDirector.wanted_key() == MusicDirector.MENU_KEY
tree_parent.remove_child(battle)
battle.free()
checks["freed owner pruned"] = MusicDirector.requests().size() == 1
tree_parent.remove_child(menu)
checks["no live owner: silence"] = MusicDirector.wanted_key().is_empty()
menu.free()
MusicDirector.clear_requests()
checks["clear_requests"] = MusicDirector.requests().is_empty()
checks["headless: no players"] = not AudioDirector.enabled()
return _report("MusicDirector request / release / wanted key", checks)
```

### `static _test_music_chain() -> bool`
```logic
DirAccess.make_dir_recursive_absolute(SANDBOX + "/worlds")
WorldCatalog.use_root(SANDBOX + "/worlds")
var errors: Array[String] = []
errors.append_array(WorldCatalog.create_world("mu_base", "Music Base"))
errors.append_array(WorldCatalog.create_world("mu_kid", "Music Kid", ["mu_base"]))
errors.append_array(WorldCatalog.upsert("mu_base", "regions", {"id": "mu_top", "name": "Top"}))
errors.append_array(WorldCatalog.upsert("mu_kid", "regions", {"id": "mu_sub", "name": "Sub", "parent": "mu_top"}))
errors.append_array(WorldCatalog.upsert("mu_kid", "locations", {"id": "mu_loc", "name": "Loc", "region": "mu_sub"}))
errors.append_array(WorldCatalog.upsert("mu_kid", "locations", {"id": "mu_bare", "name": "Bare", "kind": "town"}))
var loc: String = "mu_kid:mu_loc"
var sc: Array[String] = MusicDirector.scopes("mu_kid", loc)
var checks: Dictionary = {
	"fixtures": errors.is_empty(),
	"scopes: region, parent, world, lineage": sc == ["mu_sub", "mu_top", "mu_kid", "mu_base"],
	"scopes without a region": MusicDirector.scopes("mu_kid", "mu_kid:mu_bare") == ["mu_kid", "mu_base"],
	"scopes without a world": MusicDirector.scopes("", "").is_empty(),
}
var g := GameState.new()
g.world_id = "mu_kid"
g.location_id = "mu_kid:mu_bare"
checks["town location: town context"] = MusicDirector.field_context(g) == MusicDirector.CTX_TOWN
g.location_id = loc
checks["other location: field context"] = MusicDirector.field_context(g) == MusicDirector.CTX_FIELD
# MUSIC2-L: every listed kind maps through the table on a sandbox location; unlisted kinds play field
var kinds: Array[String] = WorldSchema.options_of(WorldSchema.field("locations", "kind"))
checks["schema kinds listed"] = not kinds.is_empty() and MusicDirector.KIND_CONTEXT.keys().all(func(k: Variant) -> bool: return kinds.has(str(k)))
checks["unknown kind -> field"] = MusicDirector.kind_context("mu_no_such_kind") == MusicDirector.CTX_FIELD
for kind: String in kinds:
	var want_ctx: String = str(MusicDirector.KIND_CONTEXT.get(kind, MusicDirector.CTX_FIELD))
	errors.append_array(WorldCatalog.upsert("mu_kid", "locations", {"id": "mu_k_" + kind, "name": kind, "kind": kind}))
	g.location_id = "mu_kid:mu_k_" + kind
	checks["kind %s -> %s" % [kind, want_ctx]] = MusicDirector.field_context(g) == want_ctx and MusicDirector.kind_context(kind) == want_ctx
checks["kind fixtures"] = errors.is_empty()
checks["kind table: town and fortress -> town; ruin and camp -> danger"] = MusicDirector.kind_context("town") == MusicDirector.CTX_TOWN \
	and MusicDirector.kind_context("fortress") == MusicDirector.CTX_TOWN and MusicDirector.kind_context("ruin") == MusicDirector.CTX_DANGER \
	and MusicDirector.kind_context("camp") == MusicDirector.CTX_DANGER and MusicDirector.kind_context("") == MusicDirector.CTX_FIELD
checks["table contexts are known contexts"] = MusicDirector.KIND_CONTEXT.values().all(func(c: Variant) -> bool: return MusicDirector.CONTEXTS.has(c))
# shop override: the open shop panel requests shop, closed falls back to the location's context
for loc_id: String in ["mu_kid:mu_bare", loc, "mu_kid:mu_k_ruin"]:
	g.location_id = loc_id
	checks["shop open at %s -> shop" % loc_id] = MusicDirector.region_context(g, true) == MusicDirector.CTX_SHOP
	checks["shop closed at %s -> field context" % loc_id] = MusicDirector.region_context(g, false) == MusicDirector.field_context(g)
checks["no game, shop closed -> field"] = MusicDirector.region_context(null, false) == MusicDirector.CTX_FIELD
g.location_id = loc
var field: Array[String] = MusicDirector.candidates(MusicDirector.CTX_FIELD, "mu_kid", loc)
var want_field: Array[String] = []
for s: String in sc:
	want_field.append(s + "_field")
want_field.append_array(sc)
want_field.append("field")
checks["field chain: scoped, plain scope keys, then field alone"] = field == want_field
var generic: Callable = func(k: String) -> bool: return k == "field" or k == "mu_top"
checks["a generic field row does not override a scope key"] = MusicDirector.track_for(MusicDirector.CTX_FIELD, "mu_kid", loc, generic) == "mu_top"
var town: Array[String] = MusicDirector.candidates(MusicDirector.CTX_TOWN, "mu_kid", loc)
checks["town falls back to the field chain"] = town[sc.size()] == "town" and town.slice(sc.size() + 1) == field
var boss: Array[String] = MusicDirector.candidates(MusicDirector.CTX_BOSS, "mu_kid", loc)
checks["boss chain falls back to combat"] = boss.size() == 2 * (sc.size() + 1) and boss[sc.size()] == "boss" and boss[boss.size() - 1] == "combat" and boss[0] == "mu_sub_boss"
var shop: Array[String] = MusicDirector.candidates(MusicDirector.CTX_SHOP, "mu_kid", loc)
checks["shop -> town -> field"] = shop.find("shop") < shop.find("town") and shop.find("town") < shop.find("field") and shop[shop.size() - 1] == "field"
checks["menu chain"] = MusicDirector.candidates(MusicDirector.CTX_MENU, "", "") == ["menu", MusicDirector.MENU_KEY]
for ctx: String in MusicDirector.PARENT_CONTEXT:
	var parent: String = str(MusicDirector.PARENT_CONTEXT[ctx])
	checks["%s keeps %s's priority band" % [ctx, parent]] = MusicDirector.priority_of(ctx) == MusicDirector.priority_of(parent)
# rule 3 fallback, one resolving key at a time
for key: String in sc:
	var only: Callable = func(k: String) -> bool: return k == key
	checks["field resolves %s alone" % key] = MusicDirector.track_for(MusicDirector.CTX_FIELD, "mu_kid", loc, only) == key
var two: Callable = func(k: String) -> bool: return k == "mu_top" or k == "mu_base"
checks["nearest scope first"] = MusicDirector.track_for(MusicDirector.CTX_FIELD, "mu_kid", loc, two) == "mu_top"
var none: Callable = func(_k: String) -> bool: return false
checks["nothing resolves: empty"] = MusicDirector.track_for(MusicDirector.CTX_FIELD, "mu_kid", loc, none).is_empty()
# real resolution through a sandbox manifest: combat only, then a world boss row wins
AssetCatalog.use_path(SANDBOX + "/music_manifest.json", SANDBOX + "/assets")
var row_errs: Array[String] = _sandbox_music_row("combat")
checks["sandbox combat row"] = row_errs.is_empty()
checks["boss with only combat -> combat"] = MusicDirector.track_for(MusicDirector.CTX_BOSS, "mu_kid", loc) == "combat"
row_errs = _sandbox_music_row("mu_kid_boss")
checks["sandbox world boss row"] = row_errs.is_empty()
checks["world boss row wins"] = MusicDirector.track_for(MusicDirector.CTX_BOSS, "mu_kid", loc) == "mu_kid_boss"
checks["encounter still -> combat"] = MusicDirector.track_for(MusicDirector.CTX_ENCOUNTER, "mu_kid", loc) == "combat"
checks["field with no field rows: silence"] = MusicDirector.track_for(MusicDirector.CTX_FIELD, "mu_kid", loc).is_empty()
checks["shop with no town or field rows: silence"] = MusicDirector.track_for(MusicDirector.CTX_SHOP, "mu_kid", loc).is_empty()
row_errs.append_array(_sandbox_music_row("town"))
checks["sandbox town row"] = row_errs.is_empty()
checks["shop falls back to town"] = MusicDirector.track_for(MusicDirector.CTX_SHOP, "mu_kid", loc) == "town"
checks["danger with only town: silence"] = MusicDirector.track_for(MusicDirector.CTX_DANGER, "mu_kid", loc).is_empty()
if not row_errs.is_empty() or not errors.is_empty():
	print("      fixture errors: %s %s" % [str(errors), str(row_errs)])
AssetCatalog.use_path(AssetCatalog.DEFAULT_PATH)
WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
MusicDirector.clear_caches()
return _report("Music context chain: region, parent, world, lineage, parent context", checks)
```

### `static _test_music_shipped() -> bool`
```logic
MusicDirector.clear_caches()
var checks: Dictionary = {}
for key: String in ["menu_theme", "ashlands", "iron_reaches", "tidewrack", "combat", "town", "danger", "shop", "travel", "boss"]:
	checks["%s resolves to a file" % key] = MusicDirector.has_track(key) and AssetRegistry.stream("music", key) != null
for w: String in WorldCatalog.world_ids():
	if not MusicDirector.has_track(w):
		continue
	for id: String in WorldCatalog.own_ids(w, "locations"):
		var key: String = MusicDirector.track_for(MusicDirector.CTX_FIELD, w, WorldCatalog.qid(w, id))
		checks["%s:%s has a field track" % [w, id]] = not key.is_empty() and MusicDirector.scopes(w, WorldCatalog.qid(w, id)).has(key)
var boss_id: String = ""
var plain_id: String = ""
for e: Dictionary in ContentCatalog.entries("units"):
	var tier: String = str(e.get("tier", ""))
	if tier == "boss" and boss_id.is_empty():
		boss_id = str(e.get("id", ""))
	elif tier != "boss" and tier != "hero" and plain_id.is_empty():
		plain_id = str(e.get("id", ""))
if not plain_id.is_empty():
	checks["non-boss enemies: combat"] = MusicDirector.combat_context(CatalogEncounter.from_entry({"id": "mu_plain", "enemies": [plain_id]})) == MusicDirector.CTX_COMBAT
if not boss_id.is_empty():
	checks["a boss-tier enemy: boss"] = MusicDirector.combat_context(CatalogEncounter.from_entry({"id": "mu_boss", "enemies": [plain_id, boss_id]})) == MusicDirector.CTX_BOSS
return _report("Shipped music rows and field / boss contexts", checks)
```

