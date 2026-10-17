class_name MenusTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Menus Test Suite (UI1-L, S13)
# Part A (schema): world start_party field and validation, the playable rule, NewGame.create_for_world, SaveGame slots
# 0..5 / autosave / peek / newest_slot, CurrentGame, SettingsFile. Part B (models): router, main menu, new game, save
# slots, settings and hub models. Fixtures: hand-built worlds in an EMPTY sandbox world root, saves and the settings
# file in the sandbox; the shipped catalog is only read (units picked by rule), shipped data is never written.
# ==============================================================================

const TAG: String = "Menus"
static var SANDBOX: String = TestHelpers.sandbox("menus")
const BASE: String = "mn_base"
const KID: String = "mn_kid"
const BARE: String = "mn_bare"

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func _party_units() -> Array[String]:
	var out: Array[String] = []
	for e: Dictionary in ContentCatalog.entries("units"):
		if str(e.get("side", "")) == "party" and ContentFactory.class_of(e) != null and str(e.get("stat_mode", "derived")) == "derived":
			out.append(str(e["id"]))
	return out

static func _enemy_unit() -> String:
	for e: Dictionary in ContentCatalog.entries("units"):
		if str(e.get("side", "enemy")) == "enemy":
			return str(e["id"])
	return ""

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING MENUS TEST (UI1-L)                      ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	var ok: bool = _report("Shipped worlds validate with start_party", {"valid": WorldCatalog.validate_all().is_empty()})
	ok = _test_shipped_start() and ok
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/worlds")
	WorldCatalog.use_root(SANDBOX + "/worlds")
	SaveGame.set_base_dir(SANDBOX + "/saves")
	SettingsFile.use_path(SANDBOX + "/settings.cfg")
	var sandboxed: bool = WorldCatalog.get_root().begins_with("user://") and WorldCatalog.world_ids().is_empty() and SaveGame.base_dir().begins_with("user://") and SettingsFile.path().begins_with("user://")
	ok = _report("Sandbox (empty world root, saves and settings under user://)", {"sandboxed": sandboxed}) and ok
	var units: Array[String] = _party_units()
	ok = _report("Content fixtures picked by rule", {"two party units": units.size() >= 2, "an enemy unit": not _enemy_unit().is_empty()}) and ok
	if sandboxed and units.size() >= 2 and _build_worlds(units):
		ok = _test_start_party(units) and ok
		ok = _test_create_for_world(units) and ok
		ok = _test_slots() and ok
		ok = _test_peek() and ok
		ok = _test_newest() and ok
		ok = _test_current_game() and ok
		ok = _test_settings() and ok
		ok = _test_part_b(tree_parent) and ok
	else:
		ok = false
	CurrentGame.end()
	SettingsFile.use_path()
	SaveGame.reset_base_dir()
	TestHelpers.cleanup_worlds(SANDBOX)
	return ok

# UI1-A: the shipped campaign world starts with its fixed party; a child inherits it; a world without one is refused.
static func _test_shipped_start() -> bool:
	var checks: Dictionary = {}
	var res: Dictionary = NewGame.create_for_world("ashlands")
	var gs: GameState = res["state"] as GameState
	var levels_ok: bool = gs != null and gs.party.members.size() == 4
	if gs != null:
		for m: UnitState in gs.party.members:
			levels_ok = levels_ok and XpCurve.level_for(m.total_xp) == DictUtil.to_int(ContentCatalog.get_entry("units", m.base_unit).get("level", 1), 1)
	checks["ashlands: four members at authored levels"] = levels_ok and gs.validate().is_empty()
	checks["tidewrack inherits"] = WorldCatalog.start_party("tidewrack") == WorldCatalog.start_party("ashlands") and not WorldCatalog.start_party("ashlands").is_empty()
	checks["iron_reaches not playable"] = JsonUtil.strings(NewGame.create_for_world("iron_reaches")["errors"]).has("World 'iron_reaches' is not playable (no start party).")
	return _report("UI1-A: shipped start party", checks)

static func _build_worlds(units: Array[String]) -> bool:
	var errors: Array[String] = []
	errors.append_array(WorldCatalog.create_world(BASE, "MN Base"))
	errors.append_array(WorldCatalog.create_world(KID, "MN Kid", [BASE]))
	errors.append_array(WorldCatalog.create_world(BARE, "MN Bare"))
	errors.append_array(WorldCatalog.upsert(BASE, "locations", {"id": "camp", "name": "Camp"}))
	errors.append_array(WorldCatalog.upsert(BARE, "locations", {"id": "nook", "name": "Nook"}))
	errors.append_array(WorldCatalog.update_manifest(BASE, {"start_location": "camp", "start_party": [units[0], units[1]], "tagline": "A test world."}))
	errors.append_array(WorldCatalog.update_manifest(BARE, {"start_location": "nook"}))
	if not errors.is_empty():
		print("      fixture errors: %s" % str(errors))
	return _report("Fixture worlds (base with a start party, child inheriting, bare without)", {"no errors": errors.is_empty(), "base valid": WorldCatalog.manifest_errors(BASE).is_empty()})

static func _party_errs(list: Array) -> Array[String]:
	var old: Dictionary = WorldCatalog.manifest(BARE)
	var _set: Array[String] = WorldCatalog.update_manifest(BARE, {"start_party": list})
	var errors: Array[String] = WorldCatalog.manifest_errors(BARE)
	var _back: Array[String] = WorldCatalog.update_manifest(BARE, {"start_party": old.get("start_party", [])})
	return errors

# Check 1: each start_party message; a child with [] inherits the parent's list.
static func _test_start_party(units: Array[String]) -> bool:
	var checks: Dictionary = {}
	var u: String = units[0]
	checks["unknown unit"] = _party_errs(["no_such_unit_xyz"]) == ["Start party unit 'no_such_unit_xyz' is not a known unit."]
	checks["not a party unit"] = _party_errs([_enemy_unit()]) == ["Start party unit '%s' is not a party unit." % _enemy_unit()]
	checks["duplicate"] = _party_errs([u, u]) == ["Start party lists '%s' twice." % u]
	var many: Array[String] = _party_units()
	var distinct: Array = []
	for id: String in many:
		if distinct.size() < 5:
			distinct.append(id)
	if distinct.size() == 5:
		checks["more than four"] = _party_errs(distinct) == ["A start party holds at most 4 units."]
	else:
		checks["more than four"] = TestHelpers.any_contains(_party_errs([u, u, u, u, u]), "A start party holds at most 4 units.")
	checks["valid list passes"] = _party_errs([u]).is_empty()
	checks["child inherits"] = WorldCatalog.start_party(KID) == [units[0], units[1]] and WorldCatalog.start_party(BARE).is_empty()
	checks["own list wins"] = WorldCatalog.update_manifest(KID, {"start_party": [units[1]]}).is_empty() and WorldCatalog.start_party(KID) == [units[1]]
	var _undo: Array[String] = WorldCatalog.update_manifest(KID, {"start_party": []})
	checks["playable worlds"] = NewGame.playable_worlds() == [BASE, KID]
	return _report("Check 1: start_party validation and inheritance", checks)

# Check 2: create_for_world on a playable world; the exact error on a world without a start party.
static func _test_create_for_world(units: Array[String]) -> bool:
	var checks: Dictionary = {}
	var res: Dictionary = NewGame.create_for_world(KID, 5)
	var gs: GameState = res["state"] as GameState
	checks["state valid"] = gs != null and gs.validate().is_empty() and gs.world_id == KID
	if gs != null:
		var ids: Array[String] = []
		var levels_ok: bool = gs.party.members.size() == 2
		for m: UnitState in gs.party.members:
			ids.append(m.base_unit)
			var lvl: int = DictUtil.to_int(ContentCatalog.get_entry("units", m.base_unit).get("level", 1), 1)
			levels_ok = levels_ok and XpCurve.level_for(m.total_xp) == lvl and m.id == m.base_unit
		checks["members = start party"] = ids == [units[0], units[1]] and levels_ok
	var bare: Dictionary = NewGame.create_for_world(BARE)
	checks["no start party"] = bare["state"] == null and JsonUtil.strings(bare["errors"]) == ["World '%s' is not playable (no start party)." % BARE]
	var _s: Array[String] = WorldCatalog.update_manifest(BARE, {"start_party": [units[0]], "start_location": ""})
	var no_loc: Dictionary = NewGame.create_for_world(BARE)
	checks["no start location"] = JsonUtil.strings(no_loc["errors"]) == ["World '%s' is not playable (no start location)." % BARE]
	var _r: Array[String] = WorldCatalog.update_manifest(BARE, {"start_party": [], "start_location": "nook"})
	return _report("Check 2: NewGame.create_for_world", checks)

static func _new_state() -> GameState:
	return NewGame.create_for_world(BASE, 3)["state"] as GameState

static func _write_raw(slot: int, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(SaveGame.base_dir())
	var f: FileAccess = FileAccess.open(SaveGame.path_for(slot), FileAccess.WRITE)
	f.store_string(text)
	f.close()

static func _wipe_saves() -> void:
	for slot: int in range(SaveGame.AUTOSAVE_SLOT, SaveGame.LAST_SLOT + 1):
		var _gone: bool = SaveGame.delete(slot)

# Check 3: slot range 0..5; slot 0 round trip.
static func _test_slots() -> bool:
	_wipe_saves()
	var gs: GameState = _new_state()
	var checks: Dictionary = {}
	checks["slot 6 refused"] = SaveGame.write(gs, 6) == ["Save slot must be 0-5."]
	checks["slot -1 refused"] = SaveGame.write(gs, -1) == ["Save slot must be 0-5."]
	checks["autosave writes slot 0"] = SaveGame.autosave(gs).is_empty() and SaveGame.exists(SaveGame.AUTOSAVE_SLOT)
	var back: GameState = SaveGame.read(0)["state"] as GameState
	checks["slot 0 reads back equal"] = back != null and back.equals(gs)
	for slot: int in [4, 5]:
		var written: bool = SaveGame.write(gs, slot).is_empty()
		var round_trip: GameState = SaveGame.read(slot)["state"] as GameState
		checks["slot %d round trip" % slot] = written and round_trip != null and round_trip.equals(gs) and JsonUtil.deep_equal(round_trip.to_dict(), gs.to_dict())
	checks["constants"] = SaveGame.AUTOSAVE_SLOT == 0 and SaveGame.MANUAL_SLOTS == [1, 2, 3, 4, 5]
	return _report("Check 3: save slots 0-5 and autosave", checks)

# Check 4: peek of missing, corrupt, v2 and current files; peek never migrates or changes the file.
static func _test_peek() -> bool:
	_wipe_saves()
	var checks: Dictionary = {}
	var missing: Dictionary = SaveGame.peek(1)
	checks["missing"] = not DictUtil.to_bool(missing["exists"]) and not DictUtil.to_bool(missing["ok"])
	_write_raw(2, "{not json")
	var bad: Dictionary = SaveGame.peek(2)
	checks["corrupt = unreadable"] = DictUtil.to_bool(bad["exists"]) and not DictUtil.to_bool(bad["ok"]) and str(bad["error"]) == "unreadable"
	_write_raw(2, JSON.stringify({"format": "other", "version": 1, "saved_at": 1, "game_state": {}}))
	checks["wrong format = unreadable"] = str(SaveGame.peek(2)["error"]) == "unreadable"
	var gs: GameState = _new_state()
	gs.location_id = WorldCatalog.qid(BASE, "camp")
	gs.party.members[0].total_xp = XpCurve.xp_for_level(5)
	checks["current written"] = SaveGame.write(gs, 1).is_empty()
	checks["current matches read"] = _peek_matches(1)
	var v2: Dictionary = gs.to_dict()
	for key: String in ["position", "explore", "rng_seed", "rolls", "shops"]:
		v2.erase(key)
	DictUtil.dict_of(v2["party"]).erase("gold")
	v2["location_id"] = "camp"
	var v2_text: String = JSON.stringify({"format": SaveGame.FORMAT, "version": 2, "saved_at": 5, "game_state": v2})
	_write_raw(3, v2_text)
	var head: Dictionary = SaveGame.peek(3)
	checks["v2 raw fields (no migration)"] = DictUtil.to_bool(head["ok"]) and DictUtil.to_int(head["version"]) == 2 and str(head["location_id"]) == "camp" and str(head["world_id"]) == BASE
	checks["v2 matches read"] = _peek_matches(3, true)
	checks["v2 file unchanged"] = FileAccess.get_file_as_string(SaveGame.path_for(3)) == v2_text
	_write_raw(2, JSON.stringify({"format": SaveGame.FORMAT, "version": SaveGame.VERSION + 1, "saved_at": 9, "game_state": v2}))
	var newer: Dictionary = SaveGame.peek(2)
	checks["newer version listed, not loadable"] = not DictUtil.to_bool(newer["ok"]) and str(newer["error"]) == "newer version" and str(newer["world_id"]) == BASE
	_write_raw(2, JSON.stringify({"format": SaveGame.FORMAT, "version": 2, "saved_at": 1, "game_state": {"party": {"members": [{}]}}}))
	var sparse: Dictionary = SaveGame.peek(2)
	var row: Dictionary = DictUtil.dict_of(DictUtil.array_of(sparse["party"])[0]) if not DictUtil.array_of(sparse["party"]).is_empty() else {}
	checks["missing pieces stay empty"] = DictUtil.to_bool(sparse["ok"]) and str(sparse["world_id"]).is_empty() and str(row.get("id", "x")).is_empty() and DictUtil.to_int(row.get("level", -1)) == 0
	return _report("Check 4: SaveGame.peek", checks)

# World, location (unless `local_location`: v2 keeps it local, read qualifies it) and party levels of peek equal read.
static func _peek_matches(slot: int, local_location: bool = false) -> bool:
	var head: Dictionary = SaveGame.peek(slot)
	var gs: GameState = SaveGame.read(slot)["state"] as GameState
	if gs == null or not DictUtil.to_bool(head["ok"]):
		return false
	var ok: bool = str(head["world_id"]) == gs.world_id
	ok = ok and (str(head["location_id"]) == gs.location_id or (local_location and WorldCatalog.qid(gs.world_id, str(head["location_id"])) == gs.location_id))
	var rows: Array = DictUtil.array_of(head["party"])
	ok = ok and rows.size() == gs.party.members.size()
	for i: int in range(mini(rows.size(), gs.party.members.size())):
		var row: Dictionary = DictUtil.dict_of(rows[i])
		var m: UnitState = gs.party.members[i]
		ok = ok and str(row["id"]) == m.id and str(row["base_unit"]) == m.base_unit and DictUtil.to_int(row["level"]) == XpCurve.level_for(m.total_xp)
	return ok

# Check 5: newest_slot picks the larger saved_at.
static func _test_newest() -> bool:
	_wipe_saves()
	var checks: Dictionary = {"none = -1": SaveGame.newest_slot() == -1}
	var gs: Dictionary = _new_state().to_dict()
	_write_raw(1, JSON.stringify({"format": SaveGame.FORMAT, "version": SaveGame.VERSION, "saved_at": 2000, "game_state": gs}))
	_write_raw(0, JSON.stringify({"format": SaveGame.FORMAT, "version": SaveGame.VERSION, "saved_at": 1000, "game_state": gs}))
	_write_raw(3, "{broken")
	checks["slot 1 newer than autosave"] = SaveGame.newest_slot() == 1
	_write_raw(0, JSON.stringify({"format": SaveGame.FORMAT, "version": SaveGame.VERSION, "saved_at": 3000, "game_state": gs}))
	checks["autosave counts"] = SaveGame.newest_slot() == 0
	_write_raw(2, JSON.stringify({"format": SaveGame.FORMAT, "version": SaveGame.VERSION + 1, "saved_at": 9000, "game_state": gs}))
	checks["newer version ignored"] = SaveGame.newest_slot() == 0
	_wipe_saves()
	return _report("Check 5: SaveGame.newest_slot", checks)

static func _test_current_game() -> bool:
	var checks: Dictionary = {}
	CurrentGame.end()
	checks["empty"] = not CurrentGame.has_game() and CurrentGame.slot() == -1 and not CurrentGame.is_dirty()
	var gs: GameState = _new_state()
	CurrentGame.start(gs)
	checks["started clean"] = CurrentGame.state() == gs and not CurrentGame.is_dirty()
	gs.party.gold += 5
	checks["change = dirty"] = CurrentGame.is_dirty()
	CurrentGame.mark_saved(2)
	checks["mark_saved = clean"] = not CurrentGame.is_dirty() and CurrentGame.slot() == 2
	CurrentGame.set_busy(true)
	checks["busy"] = CurrentGame.busy()
	CurrentGame.end()
	checks["ended"] = not CurrentGame.has_game() and not CurrentGame.busy() and CurrentGame.clean_copy().is_empty()
	return _report("CurrentGame start / dirty / mark_saved / end", checks)

# Check 9: SettingsFile round trip, clamps and defaults (one case per key); deleting a save leaves the file unchanged.
static func _test_settings() -> bool:
	var checks: Dictionary = {}
	var _gone: Error = DirAccess.remove_absolute(SettingsFile.path())
	SettingsFile.load()
	checks["missing file = defaults"] = SettingsFile.get_value("anim/speed") == 1.0
	var cases: Array[Array] = [
		["audio/master", 1.5, 1.0, 1.0], ["audio/music", -0.5, 0.0, 0.8], ["audio/sfx", 0.25, 0.25, 0.8], ["audio/ui", 2, 1.0, 0.8],
		["anim/speed", 10.0, 4.0, 1.0], ["anim/instant", true, true, false], ["ui/text_scale", 0.1, 0.8, 1.0],
	]
	for c: Array in cases:
		var key: String = str(c[0])
		checks["%s default" % key] = SettingsFile.get_value(key) == c[3]
		SettingsFile.set_value(key, c[1])
		checks["%s clamped" % key] = SettingsFile.get_value(key) == c[2]
	checks["saved"] = SettingsFile.save() == OK
	SettingsFile.set_value("anim/speed", 2.0)
	SettingsFile.load()
	checks["reload restores the file"] = SettingsFile.get_value("anim/speed") == 4.0 and SettingsFile.get_value("anim/instant") == true
	SettingsFile.set_value("anim/instant", "yes")
	checks["wrong type = default"] = SettingsFile.get_value("anim/instant") == false and SettingsFile.checked("anim/speed", "fast") == 1.0
	var cfg := ConfigFile.new()
	cfg.set_value("ui", "text_scale", "big")
	checks["file written"] = cfg.save(SettingsFile.path()) == OK
	SettingsFile.load()
	checks["wrong type on disk = default"] = SettingsFile.get_value("ui/text_scale") == 1.0
	var before: String = FileAccess.get_file_as_string(SettingsFile.path())
	var gs: GameState = _new_state()
	var _w: Array[String] = SaveGame.write(gs, 1)
	checks["deleting a save keeps settings"] = SaveGame.delete(1) and FileAccess.get_file_as_string(SettingsFile.path()) == before
	return _report("Check 9: SettingsFile", checks)

# ------------------------------------------------------------------------------
# Part B: router and screen models
# ------------------------------------------------------------------------------

static func _test_part_b(tree_parent: Node) -> bool:
	var ok: bool = _test_router()
	ok = _test_new_game_model() and ok
	ok = _test_save_slots_model() and ok
	ok = _test_hub_model() and ok
	ok = _test_main_menu_and_settings_model() and ok
	ok = _test_scene(tree_parent) and ok
	return ok

# Commit 3: scenes/main_menu.tscn renders every screen and drives the models (thin view).
static func _test_scene(tree_parent: Node) -> bool:
	var packed: PackedScene = load("res://scenes/main_menu.tscn") as PackedScene
	var checks: Dictionary = {"scene loads": packed != null}
	if packed == null or tree_parent == null:
		return _report("Main menu scene (headless without a tree: load only)", checks)
	_wipe_saves()
	CurrentGame.end()
	var view: MainMenuView = packed.instantiate() as MainMenuView
	tree_parent.add_child(view)
	var r: MenuRouter = view.router
	checks["main screen"] = r.top() == MenuRouter.MAIN and _has_button(view, "New game") and not _has_button(view, "Continue")
	for id: String in [MenuRouter.LOAD, MenuRouter.SETTINGS, MenuRouter.NEW_GAME]:
		r.push(id)
		checks["%s renders" % id] = view.get_child_count() > 0 and _has_button(view, "Back")
	checks["new game lists the world"] = _has_button(view, "Start in MN Base")
	view.act((r.model() as NewGameModel).choose.bind(BASE, 4))
	checks["game scene opened"] = view.game_view != null and CurrentGame.has_game() and r.top() == MenuRouter.HUB
	# REVIEW-F2: Esc on the results screen is Continue (autosave, results_done, map back, no RESULTS left)
	var done: Array[int] = [0]
	r.results_done.connect(func() -> void: done[0] += 1)
	var _gone: bool = SaveGame.delete(SaveGame.AUTOSAVE_SLOT)
	view._on_fight_settled({"xp": {}, "advanced": [] as Array[String], "outcome": "won"})
	checks["results shown over the hidden map"] = r.top() == MenuRouter.RESULTS and not view.game_view.visible
	view.cancel_pressed()
	checks["Esc on results: autosave written, map back"] = SaveGame.exists(SaveGame.AUTOSAVE_SLOT) and done[0] == 1 and view.game_view.visible and not r.stack.has(MenuRouter.RESULTS) and r.top() == MenuRouter.HUB
	checks["back to the hub"] = view.leave_game() and view.game_view == null and _has_button(view, "Quit to main menu") and _has_button(view, "Explore")
	CurrentGame.state().party.gold += 2
	view.act((r.model() as HubModel).quit_to_menu)
	checks["dirty quit shows the question"] = view.is_overlay_open() and r.is_asking()
	view.answer(true)
	checks["confirmed: main menu with Continue"] = not CurrentGame.has_game() and r.top() == MenuRouter.MAIN and _has_button(view, "Continue")
	tree_parent.remove_child(view)
	view.free()
	CurrentGame.end()
	_wipe_saves()
	return _report("Commit 3: main menu scene (screens, new game into the game scene, hub, quit question)", checks)

static func _has_button(root: Node, label: String) -> bool:
	for n: Node in root.find_children("*", "Button", true, false):
		var b: Button = n as Button
		if b != null and b.text == label and b.is_inside_tree():
			return true
	return false

# Records what the router emits.
class Probe:
	extends RefCounted
	var asked: Array[String] = []
	var notices: Array[String] = []
	var started: int = 0
	var quits: int = 0
	func watch(r: MenuRouter) -> void:
		r.confirm_needed.connect(func(m: String) -> void: asked.append(m))
		r.notice.connect(func(lines: Array[String]) -> void: notices.append_array(lines))
		r.game_started.connect(func(_gs: GameState) -> void: started += 1)
		r.quit_requested.connect(func() -> void: quits += 1)

static func _router() -> Array:
	var r := MenuRouter.new()
	var p := Probe.new()
	p.watch(r)
	return [r, p]

# Check 10: pop() on a model with a draft asks first and pops only on confirm.
static func _test_router() -> bool:
	var pair: Array = _router()
	var r: MenuRouter = pair[0]
	var p: Probe = pair[1]
	var checks: Dictionary = {"starts on main": r.stack == [MenuRouter.MAIN] and r.model() is MainMenuModel}
	r.push(MenuRouter.SETTINGS)
	var sm: SettingsModel = r.model() as SettingsModel
	checks["no draft pops at once"] = sm != null and not r.has_draft()
	r.pop()
	checks["popped"] = r.top() == MenuRouter.MAIN and p.asked.is_empty()
	r.push(MenuRouter.SETTINGS)
	sm = r.model() as SettingsModel
	var _e: Array[String] = sm.set_value("anim/speed", 2.5)
	checks["draft detected"] = r.has_draft()
	r.pop()
	checks["asks and stays"] = p.asked == ["Discard your changes?"] and r.top() == MenuRouter.SETTINGS
	r.decline()
	checks["decline keeps the screen"] = r.top() == MenuRouter.SETTINGS and not r.is_asking()
	r.pop()
	r.confirm()
	checks["confirm pops"] = r.top() == MenuRouter.MAIN and r.stack.size() == 1
	r.pop()
	checks["bottom screen stays"] = r.top() == MenuRouter.MAIN
	r.push(MenuRouter.LOAD)
	r.replace(MenuRouter.NEW_GAME)
	checks["replace"] = r.stack == [MenuRouter.MAIN, MenuRouter.NEW_GAME] and r.model() is NewGameModel
	r.clear_to(MenuRouter.HUB)
	checks["clear_to"] = r.stack == [MenuRouter.HUB] and r.models.size() == 1
	return _report("Check 10: MenuRouter stack and draft confirm", checks)

# Check 6: choose() makes the game current, writes slot 0 and opens the hub.
static func _test_new_game_model() -> bool:
	_wipe_saves()
	CurrentGame.end()
	var pair: Array = _router()
	var r: MenuRouter = pair[0]
	var p: Probe = pair[1]
	r.push(MenuRouter.NEW_GAME)
	var m: NewGameModel = r.model() as NewGameModel
	var rows: Array[Dictionary] = m.worlds()
	var checks: Dictionary = {"playable worlds listed": rows.size() == 2 and str(rows[0]["id"]) == BASE and DictUtil.array_of(rows[0]["party"]).size() == 2 and str(rows[0]["tagline"]) == "A test world."}
	checks["unplayable refused"] = m.choose(BARE) == ["World '%s' is not playable (no start party)." % BARE] and not CurrentGame.has_game()
	checks["no autosave: no question"] = not r.is_asking() and not SaveGame.exists(SaveGame.AUTOSAVE_SLOT)
	checks["choose"] = m.choose(BASE, 11).is_empty() and not r.is_asking()
	checks["current game set"] = CurrentGame.has_game() and CurrentGame.state().world_id == BASE and not CurrentGame.is_dirty()
	var back: GameState = SaveGame.read(SaveGame.AUTOSAVE_SLOT)["state"] as GameState
	checks["slot 0 written"] = back != null and back.equals(CurrentGame.state()) and CurrentGame.slot() == 0
	checks["hub on top"] = r.stack == [MenuRouter.HUB] and p.started == 1
	# Q1: a readable autosave is asked about first
	var r2: MenuRouter = MenuRouter.new(MenuRouter.NEW_GAME)
	var m2: NewGameModel = r2.model() as NewGameModel
	var old_autosave: GameState = CurrentGame.state()
	var asked_ok: bool = m2.choose(BASE, 12).is_empty() and r2.is_asking() and r2.pending_message.begins_with("Starting a new game replaces your autosave (") and r2.pending_message.ends_with("). Continue?")
	checks["Q1 asks over a readable autosave"] = asked_ok and CurrentGame.state() == old_autosave
	r2.decline()
	checks["declined: nothing replaced"] = CurrentGame.state() == old_autosave and r2.top() == MenuRouter.NEW_GAME
	checks["unplayable world: error, no question"] = m2.choose(BARE) == ["World '%s' is not playable (no start party)." % BARE] and not r2.is_asking()
	var _again: Array[String] = m2.choose(BASE, 12)
	r2.confirm()
	checks["accepted: new game replaces it"] = CurrentGame.state() != old_autosave and r2.top() == MenuRouter.HUB
	_write_raw(SaveGame.AUTOSAVE_SLOT, "not json")
	var r3: MenuRouter = MenuRouter.new(MenuRouter.NEW_GAME)
	checks["unreadable autosave: no question"] = (r3.model() as NewGameModel).choose(BASE, 13).is_empty() and not r3.is_asking() and r3.top() == MenuRouter.HUB
	return _report("Check 6: NewGameModel.choose", checks)

# Check 7: save(0) refused; overwrite asks; busy refuses; load enters the game and shows warnings once.
static func _test_save_slots_model() -> bool:
	var pair: Array = _router()
	var r: MenuRouter = pair[0]
	var p: Probe = pair[1]
	var checks: Dictionary = {}
	CurrentGame.end()
	var none := SaveSlotsModel.new(r, true)
	checks["no game, no save"] = none.save(1) == [SaveSlotsModel.NO_GAME]
	CurrentGame.start(_new_state())
	var m := SaveSlotsModel.new(r, true)
	checks["slot 0 refused"] = m.save(0) == [SaveSlotsModel.MANUAL_ONLY] and not p.asked.size() > 0
	var _gone: bool = SaveGame.delete(2)
	checks["empty slot saves at once"] = m.save(2).is_empty() and p.asked.is_empty() and SaveGame.exists(2) and CurrentGame.slot() == 2
	CurrentGame.state().party.gold += 1
	checks["existing slot asks"] = m.save(2).is_empty() and p.asked == ["Overwrite slot 2?"] and CurrentGame.is_dirty()
	r.confirm()
	checks["confirmed overwrite saved"] = m.last_errors.is_empty() and not CurrentGame.is_dirty()
	CurrentGame.set_busy(true)
	checks["busy refused"] = m.save(3) == ["You can't save during a battle or a conversation."] and not SaveGame.exists(3)
	CurrentGame.set_busy(false)
	var rows: Array[Dictionary] = m.rows()
	checks["rows 0..5"] = rows.size() == SaveGame.LAST_SLOT + 1 and str(rows[0]["label"]) == "Autosave" and str(rows[2]["status"]) == "ok" and not DictUtil.to_bool(rows[0]["can_save"]) and DictUtil.to_bool(rows[2]["can_save"])
	_write_raw(3, "{bad")
	rows = m.rows()
	checks["unreadable row: delete only"] = str(rows[3]["status"]) == "unreadable" and DictUtil.to_bool(rows[3]["can_delete"]) and not DictUtil.to_bool(SaveSlotsModel.new(r, false).rows()[3]["can_load"])
	checks["delete asks"] = m.delete(3).is_empty() and p.asked.back() == "Delete the save in slot 3?" and SaveGame.exists(3)
	r.confirm()
	checks["deleted on confirm"] = not SaveGame.exists(3)
	checks["delete empty refused"] = m.delete(3) == ["Slot 3 is empty."]
	# load: a v2 file (migration) and a corrupt file
	var v2: Dictionary = _new_state().to_dict()
	for key: String in ["position", "explore", "rng_seed", "rolls", "shops"]:
		v2.erase(key)
	v2["location_id"] = "nowhere"
	_write_raw(1, JSON.stringify({"format": SaveGame.FORMAT, "version": 2, "saved_at": 1, "game_state": v2}))
	_write_raw(3, "{bad")
	CurrentGame.end()
	var loader := SaveSlotsModel.new(r, false)
	checks["corrupt load keeps the screen"] = not loader.load(3).is_empty() and not CurrentGame.has_game() and p.started == 0
	checks["load enters the game"] = loader.load(1).is_empty() and CurrentGame.has_game() and CurrentGame.slot() == 1 and r.top() == MenuRouter.HUB and p.started == 1
	checks["warnings shown once"] = p.notices == ["cleared unresolved location_id 'nowhere'"]
	_wipe_saves()
	return _report("Check 7: SaveSlotsModel save / load / delete", checks)

# Check 8: quit_to_menu asks when dirty, not after mark_saved.
static func _test_hub_model() -> bool:
	var pair: Array = _router()
	var r: MenuRouter = pair[0]
	var p: Probe = pair[1]
	CurrentGame.start(_new_state())
	r.clear_to(MenuRouter.HUB)
	var hub: HubModel = r.model() as HubModel
	var checks: Dictionary = {"names": hub.world_name() == "MN Base" and hub.party().size() == 2}
	checks["save opens"] = hub.open(MenuRouter.SAVE).is_empty() and r.top() == MenuRouter.SAVE
	r.pop()
	checks["journal opens (UI3-L)"] = hub.open(MenuRouter.JOURNAL).is_empty() and r.top() == MenuRouter.JOURNAL
	r.pop()
	checks["unknown entry refused"] = hub.open("nope") == ["Unknown hub entry 'nope'."] and r.top() == MenuRouter.HUB
	CurrentGame.state().party.gold += 3
	checks["dirty asks"] = hub.quit_to_menu().is_empty() and p.asked == ["Quit without saving?"] and CurrentGame.has_game()
	r.decline()
	CurrentGame.mark_saved(1)
	checks["clean quits at once"] = hub.quit_to_menu().is_empty() and p.asked.size() == 1 and not CurrentGame.has_game() and r.stack == [MenuRouter.MAIN]
	return _report("Check 8: HubModel.quit_to_menu", checks)

static func _test_main_menu_and_settings_model() -> bool:
	_wipe_saves()
	CurrentGame.end()
	var pair: Array = _router()
	var r: MenuRouter = pair[0]
	var p: Probe = pair[1]
	var mm: MainMenuModel = r.model() as MainMenuModel
	var checks: Dictionary = {"no continue without saves": not mm.can_continue() and not mm.continue_game().is_empty()}
	var gs: GameState = _new_state()
	var _w: Array[String] = SaveGame.write(gs, 2)
	checks["continue loads the newest"] = mm.can_continue() and mm.continue_game().is_empty() and CurrentGame.slot() == 2 and p.started == 1
	checks["clean quit"] = mm.quit_requested().is_empty() and p.quits == 1 and p.asked.is_empty()
	CurrentGame.state().party.gold += 1
	var _q: Array[String] = mm.quit_requested()
	checks["dirty quit asks"] = p.asked == ["Quit without saving?"] and p.quits == 1
	r.confirm()
	checks["confirmed quit"] = p.quits == 2
	var sm := SettingsModel.new()
	checks["unknown key refused"] = sm.set_value("nope", 1) == ["Unknown setting 'nope'."]
	var _s: Array[String] = sm.set_value("ui/text_scale", 9.0)
	checks["draft clamped, file untouched"] = sm.value("ui/text_scale") == 1.5 and SettingsFile.get_value("ui/text_scale") != 1.5
	var applied: bool = sm.apply().is_empty() and not sm.has_draft()
	SettingsFile.load()
	checks["apply writes"] = applied and SettingsFile.get_value("ui/text_scale") == 1.5
	var _t: Array[String] = sm.set_value("ui/text_scale", 1.0)
	sm.revert()
	checks["revert"] = sm.value("ui/text_scale") == 1.5 and not sm.has_draft()
	CurrentGame.end()
	_wipe_saves()
	return _report("Main menu continue / quit, SettingsModel draft / apply / revert", checks)
