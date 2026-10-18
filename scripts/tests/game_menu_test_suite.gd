class_name GameMenuTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Game menu tests (UIR-2, UI-REVIEW1 Q1 B + the 2026-10-18 playtest asks)
# Rules: during play the router stack is [PLAY]; Esc / the Menu button push the game menu (HUB) as a popup over the
# paused map; it offers Party, Journal, Save game, Load game, Settings and Quit to main menu, and Return to map closes it;
# every screen opened from it shows in the same popup; saving and party changes are refused (with the reason) while a
# battle, conversation or shop is open; loading over unsaved progress asks (No focused); quitting frees the game scene.
# New game: selectable world cards and one Start; save / load: fixed-height rows with the actions in fixed slots.
# Everything fits 1280x720 at the four text sizes. Shipped Ashlands data is only read; saves and settings go to a sandbox.
# ==============================================================================

const TAG: String = "GAME-MENU"
const WORLD: String = "ashlands"
const VIEW: Vector2 = Vector2(1280, 720)
static var SANDBOX: String = TestHelpers.sandbox("game_menu")

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING GAME MENU TEST (UIR-2)                  ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/saves")
	SaveGame.set_base_dir(SANDBOX + "/saves")
	SettingsFile.use_path(SANDBOX + "/settings.cfg")
	var ok: bool = _test_hub_model()
	ok = _test_new_game_model() and ok
	ok = _test_load_question() and ok
	if tree_parent != null:
		ok = _test_in_play(tree_parent) and ok
		ok = _test_fit(tree_parent) and ok
	CurrentGame.end()
	SettingsFile.use_path()
	SaveGame.reset_base_dir()
	TestHelpers.remove_sandbox(SANDBOX)
	return ok

static func _state() -> GameState:
	return GameState.of(RegionMapView.new_state(WORLD)["state"])

static func _wipe_saves() -> void:
	for slot: int in range(SaveGame.LAST_SLOT + 1):
		var _gone: bool = SaveGame.delete(slot)

# ------------------------------------------------------------------------------
# Models
# ------------------------------------------------------------------------------

static func _test_hub_model() -> bool:
	var checks: Dictionary = {}
	_wipe_saves()
	var gs: GameState = _state()
	checks["fixture game"] = gs != null
	if gs == null:
		return TestHelpers.report(TAG, "HubModel entries, reasons, labels", checks)
	CurrentGame.start(gs)
	var r := MenuRouter.new(MenuRouter.PLAY)
	r.push(MenuRouter.HUB)
	var hub: HubModel = r.model() as HubModel
	checks["entries: party, journal, save, load, settings"] = HubModel.ENTRIES == [MenuRouter.PARTY, MenuRouter.JOURNAL, MenuRouter.SAVE, MenuRouter.LOAD, MenuRouter.SETTINGS]
	checks["over the map: Return to map"] = hub.can_return()
	checks["party opens"] = hub.open(MenuRouter.PARTY).is_empty() and r.top() == MenuRouter.PARTY
	r.pop()
	checks["save opens"] = hub.open(MenuRouter.SAVE).is_empty() and r.top() == MenuRouter.SAVE
	r.pop()
	checks["settings opens"] = hub.open(MenuRouter.SETTINGS).is_empty() and r.top() == MenuRouter.SETTINGS
	r.pop()
	checks["load without saves says why"] = hub.reason(MenuRouter.LOAD) == HubModel.NO_SAVES and hub.open(MenuRouter.LOAD) == [HubModel.NO_SAVES] and r.top() == MenuRouter.HUB
	var _w: Array[String] = SaveGame.write(gs, 1)
	checks["load opens once a save exists"] = hub.open(MenuRouter.LOAD).is_empty() and r.top() == MenuRouter.LOAD
	r.pop()
	CurrentGame.set_busy(true)
	checks["busy: save refused with the reason"] = hub.reason(MenuRouter.SAVE) == SaveSlotsModel.BUSY and hub.open(MenuRouter.SAVE) == [SaveSlotsModel.BUSY] and r.top() == MenuRouter.HUB
	checks["busy: party refused with the reason"] = hub.open(MenuRouter.PARTY) == [HubModel.BUSY_PARTY] and r.top() == MenuRouter.HUB
	checks["busy: settings still opens"] = hub.can_open(MenuRouter.SETTINGS)
	checks["busy: load refused with the reason"] = hub.open(MenuRouter.LOAD) == [HubModel.BUSY_LEAVE] and r.top() == MenuRouter.HUB
	checks["busy: quit refused with the reason, game kept"] = hub.quit_reason() == HubModel.BUSY_LEAVE and hub.quit_to_menu() == [HubModel.BUSY_LEAVE] and CurrentGame.has_game() and r.top() == MenuRouter.HUB and not r.is_asking()
	checks["busy: loading a slot refused in the model too"] = SaveSlotsModel.new(r, false).load(1) == [HubModel.BUSY_LEAVE] and CurrentGame.state() == gs
	CurrentGame.set_busy(false)
	checks["not busy: load and quit allowed again"] = hub.can_open(MenuRouter.LOAD) and hub.quit_reason().is_empty()
	checks["journal check matches the journal"] = HubModel.has_quests() == not JournalModel.new().shown().is_empty()
	checks["journal: open iff there are quests"] = hub.can_open(MenuRouter.JOURNAL) == not JournalModel.new().shown().is_empty()
	checks["unknown entry refused"] = hub.open("nope") == ["Unknown hub entry 'nope'."]
	# Party label names the unspent points (derived from the roster, not a fixed number).
	var pts: int = hub.unspent_points()
	checks["party label follows the points"] = hub.label(MenuRouter.PARTY) == ("Party" if pts == 0 else "Party (%d point%s to spend)" % [pts, "" if pts == 1 else "s"])
	var m: UnitState = gs.party.members[0]
	m.total_xp = XpCurve.xp_for_level(m.level() + 2)
	var more: int = hub.unspent_points()
	checks["more XP, more points in the label"] = more > pts and hub.label(MenuRouter.PARTY) == "Party (%d point%s to spend)" % [more, "" if more == 1 else "s"]
	var rows: Array[Dictionary] = hub.party()
	var hp_ok: bool = rows.size() == gs.party.members.size()
	var seen_reserve: bool = false
	for row: Dictionary in rows:
		hp_ok = hp_ok and DictUtil.to_int(row["hp"]) <= DictUtil.to_int(row["max_hp"]) and DictUtil.to_int(row["max_hp"]) > 0
		if not DictUtil.to_bool(row["active"]):
			seen_reserve = true
		elif seen_reserve:
			hp_ok = false   # an active member after a reserve one
	checks["tiles: every member, HP within max, active first"] = hp_ok
	checks["return to map pops to the game"] = hub.return_to_map().is_empty() and r.stack == [MenuRouter.PLAY]
	var alone := MenuRouter.new(MenuRouter.HUB)
	checks["full-screen fallback: no Return to map"] = not (alone.model() as HubModel).can_return() and not (alone.model() as HubModel).return_to_map().is_empty()
	CurrentGame.end()
	_wipe_saves()
	return TestHelpers.report(TAG, "HubModel entries, reasons, labels, tiles, Return to map", checks)

static func _test_new_game_model() -> bool:
	var checks: Dictionary = {}
	var r := MenuRouter.new(MenuRouter.NEW_GAME)
	var m: NewGameModel = r.model() as NewGameModel
	var playable: Array[String] = NewGame.playable_worlds()
	checks["first playable world selected"] = not playable.is_empty() and m.selected == playable[0]
	checks["unknown world not selectable"] = not m.select("no_such_world").is_empty() and m.selected == playable[0]
	checks["select a playable world"] = m.select(playable[-1]).is_empty() and m.selected == playable[-1]
	var same: Array = [{"name": "A", "class": "Warrior", "level": 2}, {"name": "B", "class": "B", "level": 2}]
	checks["shared level said once"] = NewGameModel.party_line(same) == "Party, all Lv 2: A (Warrior), B"
	var mixed: Array = [{"name": "A", "class": "Warrior", "level": 1}, {"name": "B", "class": "Mage", "level": 3}]
	checks["mixed levels per member"] = NewGameModel.party_line(mixed) == "Party: A (Warrior, Lv 1), B (Mage, Lv 3)"
	m.selected = ""
	checks["start without a selection refused"] = m.start() == [NewGameModel.NONE_SELECTED]
	return TestHelpers.report(TAG, "NewGameModel selection, Start, party line", checks)

static func _test_load_question() -> bool:
	var checks: Dictionary = {}
	_wipe_saves()
	var gs: GameState = _state()
	if gs == null:
		return TestHelpers.report_ok(TAG, "Load over a game in progress", false, "no fixture game")
	var _w: Array[String] = SaveGame.write(gs, 2)
	CurrentGame.start(gs)
	CurrentGame.mark_saved(2)
	var r := MenuRouter.new(MenuRouter.PLAY)
	var loader := SaveSlotsModel.new(r, false)
	CurrentGame.state().party.gold += 1
	checks["dirty: load asks, destructive"] = loader.load(2).is_empty() and r.is_asking() and r.pending_destructive and r.pending_message.contains(SaveSlotsModel.describe(2)) and CurrentGame.state() == gs
	r.decline()
	checks["declined: the game stays"] = CurrentGame.state() == gs
	var _l: Array[String] = loader.load(2)
	r.confirm()
	checks["confirmed: loaded"] = loader.last_errors.is_empty() and CurrentGame.state() != gs and not CurrentGame.is_dirty() and r.stack == [MenuRouter.PLAY]
	var r2 := MenuRouter.new(MenuRouter.PLAY)
	checks["clean: loads at once"] = SaveSlotsModel.new(r2, false).load(2).is_empty() and not r2.is_asking() and CurrentGame.slot() == 2
	checks["describe names world and time"] = SaveSlotsModel.describe(2).begins_with(str(WorldCatalog.manifest(WORLD).get("name", WORLD))) and SaveSlotsModel.describe(4).is_empty()
	CurrentGame.end()
	_wipe_saves()
	return TestHelpers.report(TAG, "Load over unsaved progress asks first (No focused)", checks)

# ------------------------------------------------------------------------------
# Scene
# ------------------------------------------------------------------------------

# [holder, host]: the main menu in a 1280x720 viewport.
static func _menu(tree_parent: Node) -> Array:
	var holder := SubViewport.new()
	holder.size = Vector2i(VIEW)
	tree_parent.add_child(holder)
	var host: MainMenuView = (load("res://scenes/main_menu.tscn") as PackedScene).instantiate() as MainMenuView
	holder.add_child(host)
	return [holder, host]

static func _free(holder: Node) -> void:
	holder.get_parent().remove_child(holder)
	holder.free()

static func _find(host: Node, name: String) -> Button:
	return host.find_child(name, true, false) as Button

static func _panel(host: MainMenuView) -> Control:
	return host.find_child("SettingsPanel", true, false) as Control

static func _popup_shown(host: MainMenuView) -> bool:
	var layer: Control = host.find_child("PopupLayer", true, false) as Control
	return layer != null and layer.is_visible_in_tree()

static func _press(host: MainMenuView, name: String) -> bool:
	var b: Button = _find(host, name)
	if b == null or b.disabled:
		return false
	b.pressed.emit()
	SettingsUiTestSuite.layout(host)
	return true

static func _test_in_play(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_wipe_saves()
	SettingsFile.set_value("ui/text_scale", 1.0)
	var gs: GameState = _state()
	var pair: Array = _menu(tree_parent)
	var host: MainMenuView = pair[1] as MainMenuView
	var r: MenuRouter = host.router
	CurrentGame.start(gs)
	r.enter_game(gs)
	SettingsUiTestSuite.layout(host)
	var first: Node = host.game_view
	checks["playing: the stack is the game scene"] = r.stack == [MenuRouter.PLAY] and host.in_game() and not _popup_shown(host)
	var play_ok: bool = MapDevChecks.run_play(first as RegionMapView, "main menu route") if first is RegionMapView else false   # MAPDEV1-U
	checks["the main menu's game is a region map without debug controls"] = play_ok
	host.cancel_pressed()
	SettingsUiTestSuite.layout(host)
	checks["Esc: game menu popup over the visible, paused map"] = r.top() == MenuRouter.HUB and _popup_shown(host) and host.game_view.visible and not host.game_view.can_process()
	var labels: Array[String] = []
	for id: String in HubModel.ENTRIES:
		var b: Button = _find(host, "Hub_" + id)
		labels.append(b.text if b != null else "?")
	checks["entries shown"] = labels.size() == 5 and labels[0].begins_with("Party") and labels[2] == "Save game" and labels[3] == "Load game" and labels[4] == "Settings" and _find(host, "HubQuit") != null and _find(host, "BackButton") != null and _find(host, "BackButton").text == "Return to map"
	checks["a tile per member"] = host.find_children("HubTile_*", "", true, false).size() == gs.party.members.size()
	checks["the menu is the small popup"] = _panel(host).size.x <= MainMenuView.POPUP_MAX.x + 0.5
	# Party, over the paused map in the wide popup; Esc back to the menu.
	checks["Party opens in the popup"] = _press(host, "Hub_" + MenuRouter.PARTY) and r.top() == MenuRouter.PARTY and _popup_shown(host) and host.game_view.visible and not host.game_view.can_process()
	checks["party screens get the wide popup"] = _panel(host).size.x > MainMenuView.POPUP_MAX.x
	r.ask("Question over the popup?", func() -> void: pass)
	SettingsUiTestSuite.layout(host)
	var popup_layer: CanvasLayer = host._popup.get_parent() as CanvasLayer
	var question_layer: CanvasLayer = host._overlay.get_parent() as CanvasLayer
	checks["the question overlay sits in a layer above the in-play popup"] = host.is_overlay_open() and popup_layer != null and question_layer != null and question_layer.layer > popup_layer.layer and host._mlayer.layer > popup_layer.layer and host._mlayer.layer < question_layer.layer
	host.answer(false)
	host.cancel_pressed()
	checks["Esc: back to the menu"] = r.top() == MenuRouter.HUB and _popup_shown(host)
	# Save into slot 1, back, then the busy rule.
	checks["Save game opens in the popup"] = _press(host, "Hub_" + MenuRouter.SAVE) and r.top() == MenuRouter.SAVE and _popup_shown(host)
	var row1: Node = host.find_child("SlotRow1", true, false)
	var save_here: Button = row1.find_child("SlotMain", true, false) as Button if row1 != null else null
	if save_here != null:
		save_here.pressed.emit()
	checks["saved to slot 1"] = SaveGame.exists(1) and CurrentGame.slot() == 1
	host.cancel_pressed()
	CurrentGame.set_busy(true)
	host.render()
	var save: Button = _find(host, "Hub_" + MenuRouter.SAVE)
	var detail: Label = host.find_child("HubDetail", true, false) as Label
	checks["busy: Save disabled, says why"] = save != null and save.disabled and save.tooltip_text == SaveSlotsModel.BUSY
	if save != null:
		save.focus_entered.emit()
	checks["busy: the detail line shows the reason on focus"] = detail != null and detail.text == SaveSlotsModel.BUSY
	checks["busy: Party disabled, says why"] = _find(host, "Hub_" + MenuRouter.PARTY).disabled and _find(host, "Hub_" + MenuRouter.PARTY).tooltip_text == HubModel.BUSY_PARTY
	var quit_b: Button = _find(host, "HubQuit")
	var load_e: Button = _find(host, "Hub_" + MenuRouter.LOAD)
	checks["busy: Quit and Load disabled, say why"] = quit_b != null and quit_b.disabled and quit_b.tooltip_text == HubModel.BUSY_LEAVE and load_e != null and load_e.disabled and load_e.tooltip_text == HubModel.BUSY_LEAVE
	if quit_b != null:
		quit_b.focus_entered.emit()
	checks["busy: the detail line shows the quit reason on focus"] = detail != null and detail.text == HubModel.BUSY_LEAVE
	CurrentGame.set_busy(false)
	host.render()
	checks["not busy: Quit and Load enabled again"] = not _find(host, "HubQuit").disabled and not _find(host, "Hub_" + MenuRouter.LOAD).disabled
	# Settings, then the cross returns to the menu.
	checks["Settings opens from the menu"] = _press(host, "Hub_" + MenuRouter.SETTINGS) and r.top() == MenuRouter.SETTINGS and _popup_shown(host)
	var close: Button = _find(host, "SettingsClose")
	if close != null:
		close.pressed.emit()
	checks["closing Settings returns to the menu"] = r.top() == MenuRouter.HUB
	# Return to map.
	checks["Return to map closes the popup, the map runs"] = _press(host, "BackButton") and r.stack == [MenuRouter.PLAY] and not _popup_shown(host) and host.game_view.can_process()
	# Load slot 1 from the menu: a clean game loads at once and the scene is rebuilt.
	host.cancel_pressed()
	checks["Load game opens in the popup"] = _press(host, "Hub_" + MenuRouter.LOAD) and r.top() == MenuRouter.LOAD
	var lrow: Node = host.find_child("SlotRow1", true, false)
	var load_b: Button = lrow.find_child("SlotMain", true, false) as Button if lrow != null else null
	if load_b != null:
		load_b.pressed.emit()
	checks["loaded: a new game scene, no popup"] = r.stack == [MenuRouter.PLAY] and host.game_view != null and host.game_view != first and host.in_game() and not _popup_shown(host) and host.game_view.can_process()
	# Quit to main menu with unsaved progress: asks (No focused), then the scene goes.
	CurrentGame.state().party.gold += 1
	host.cancel_pressed()
	checks["quit asks, No focused"] = _press(host, "HubQuit") and host.is_overlay_open() and host.overlay_default_button() != null and host.overlay_default_button().text == "No"
	host.answer(true)
	checks["quit: main menu, game scene freed"] = r.stack == [MenuRouter.MAIN] and host.game_view == null and not CurrentGame.has_game() and not _popup_shown(host)
	_free(pair[0] as Node)
	CurrentGame.end()
	_wipe_saves()
	return TestHelpers.report(TAG, "In play: Esc opens the game menu over the paused map; Party, Save, Load, Settings, Return, Quit", checks)

# 1280x720 at the four text sizes: the in-play menu and save popup keep their action bar inside the panel; the full-screen
# load, new game and main screens keep theirs (and the version label) on screen; save rows keep one height and aligned slots.
static func _test_fit(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	_wipe_saves()
	var gs: GameState = _state()
	var _w: Array[String] = SaveGame.write(gs, 2)
	var scales: Array[float] = SettingsModel.levels("ui/text_scale")
	checks["four text sizes"] = scales.size() == 4
	var screen := Rect2(Vector2.ZERO, VIEW)
	for sc: float in scales:
		SettingsFile.set_value("ui/text_scale", sc)
		var tag: String = "x%s" % str(sc)
		var pair: Array = _menu(tree_parent)
		var host: MainMenuView = pair[1] as MainMenuView
		var r: MenuRouter = host.router
		SettingsUiTestSuite.layout(host)
		var version: Label = host.find_child("VersionLabel", true, false) as Label
		checks["%s: version label bottom right, on screen" % tag] = version != null and screen.encloses(version.get_global_rect()) and version.get_global_rect().position.x > VIEW.x / 2.0 and version.get_global_rect().position.y > VIEW.y / 2.0
		r.push(MenuRouter.NEW_GAME)
		SettingsUiTestSuite.layout(host)
		checks["%s: new game bar on screen, Start last" % tag] = _bar_ok(host, screen) and _find(host, "PrimaryButton") != null and _find(host, "PrimaryButton").text == "Start"
		r.pop()
		r.push(MenuRouter.LOAD)
		SettingsUiTestSuite.layout(host)
		checks["%s: load bar on screen" % tag] = _bar_ok(host, screen)
		checks["%s: load rows one height, slots aligned" % tag] = _rows_ok(host)
		CurrentGame.start(gs)
		r.enter_game(gs)
		host.cancel_pressed()
		SettingsUiTestSuite.layout(host)
		var panel: Rect2 = _panel(host).get_global_rect()
		var entries: Control = host.find_child("HubEntries", true, false) as Control
		var tiles: Control = host.find_child("HubParty", true, false) as Control
		checks["%s: menu popup on screen" % tag] = screen.encloses(panel)
		checks["%s: menu bar inside the popup" % tag] = _bar_ok(host, panel)
		checks["%s: entries and tiles inside the popup" % tag] = entries != null and tiles != null and panel.grow(0.5).encloses(entries.get_global_rect()) and panel.grow(0.5).encloses(tiles.get_global_rect())
		var scroll: ScrollContainer = host.find_child("FrameScroll", true, false) as ScrollContainer
		var column: Control = host.find_child("FrameColumn", true, false) as Control
		checks["%s: the whole menu shows without scrolling" % tag] = scroll != null and column != null and column.get_combined_minimum_size().y <= scroll.size.y + 0.5
		var _s: bool = _press(host, "Hub_" + MenuRouter.SAVE)
		var wide: Rect2 = _panel(host).get_global_rect()
		checks["%s: save popup on screen, bar inside" % tag] = screen.encloses(wide) and _bar_ok(host, wide)
		checks["%s: save rows one height, slots aligned" % tag] = _rows_ok(host)
		host.cancel_pressed()
		var _p: bool = _press(host, "Hub_" + MenuRouter.PARTY)
		var party_panel: Rect2 = _panel(host).get_global_rect()
		checks["%s: party popup stays on screen, X and detail inside" % tag] = r.top() == MenuRouter.PARTY and screen.encloses(party_panel) and party_panel.grow(0.5).encloses((host.find_child("CloseButton", true, false) as Control).get_global_rect()) and party_panel.grow(0.5).encloses((host.find_child("DetailPane", true, false) as Control).get_global_rect())
		_free(pair[0] as Node)
		CurrentGame.end()
	SettingsFile.set_value("ui/text_scale", 1.0)
	_wipe_saves()
	return TestHelpers.report(TAG, "1280x720 at four text sizes: menu popup, save / load, new game, main menu", checks)

static func _bar_ok(host: MainMenuView, area: Rect2) -> bool:
	var bar: Control = host.find_child("ActionBar", true, false) as Control
	if bar == null or not bar.is_visible_in_tree():
		return false
	var rect: Rect2 = bar.get_global_rect()
	return rect.size.y > 0.0 and area.grow(0.5).encloses(rect)

# Every slot row has the same height and each action slot starts at the same x in every row.
static func _rows_ok(host: MainMenuView) -> bool:
	var heights: Dictionary = {}
	var xs: Dictionary = {}
	var rows: int = 0
	for slot: int in range(SaveGame.LAST_SLOT + 1):
		var row: Control = host.find_child("SlotRow%d" % slot, true, false) as Control
		if row == null:
			return false
		rows += 1
		heights[roundi(row.size.y)] = true
		for n: Node in row.find_children("Slot*", "Button", true, false):
			var key: String = str(n.name).trim_suffix("Empty")
			xs[key + "@" + str(roundi((n as Control).get_global_rect().position.x))] = true
	return rows == SaveGame.LAST_SLOT + 1 and heights.size() == 1 and xs.size() <= SaveSlotsView.SLOT_NAMES.size()
