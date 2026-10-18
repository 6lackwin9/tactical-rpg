---
class: ControlsHelpTestSuite
extends: RefCounted
description: "Tactical RPG - Controls help tests (CONTROLS-HELP) Rules: the Settings popup has a read-only Controls tab listing every InputMap action in ControlsList and every hard-coded key (each hard-coded row's tokens must appear in the source of the script that handles it, so the help cannot drift); during play Esc (when nothing else took it) or the Menu button opens the popup over the region map and over a mounted fight; the cross and Esc close it; while it is open the game underneath is paused and its keys are blocked (BattleScreen keys_blocked mechanism). Shipped Ashlands data, read only; settings and saves go to a sandbox. One game screen per case (message queue)."
---

# Properties
* `TAG`: String = "CONTROLS-HELP" (const)
* `WORLD`: String = "ashlands" (const)
* `ENCOUNTER`: String = "ash_scouts" (const)
* `VIEW`: Vector2 = Vector2(1280, 720) (const)

# Methods

### `static run(tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING CONTROLS HELP TEST (CONTROLS-HELP)      ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
TestHelpers.remove_sandbox(SANDBOX)
DirAccess.make_dir_recursive_absolute(SANDBOX + "/saves")
SaveGame.set_base_dir(SANDBOX + "/saves")
SettingsFile.use_path(SANDBOX + "/settings.cfg")
var ok: bool = _test_table_sources()
var built: Dictionary = RegionMapView.new_state(WORLD)
if not (built["state"] is GameState):
	ok = TestHelpers.report_ok(TAG, "A new Ashlands game for the fixtures", false)
elif tree_parent != null:
	CurrentGame.start(built["state"] as GameState)
	ok = _test_tab_actions(tree_parent) and ok
	ok = _test_tab_hard_coded(tree_parent) and ok
	ok = _test_region(tree_parent) and ok
	ok = _test_combat(tree_parent) and ok
	ok = _test_blocked_region(tree_parent) and ok
	ok = _test_blocked_combat(tree_parent) and ok
CurrentGame.end()
SettingsFile.use_path()
SaveGame.reset_base_dir()
TestHelpers.remove_sandbox(SANDBOX)
return ok
```

### `static _test_table_sources() -> bool`
```logic
var checks: Dictionary = {}
for i: int in ControlsList.HARD_CODED.size():
	var h: Dictionary = ControlsList.HARD_CODED[i]
	var src: String = str(h["src"])
	var text: String = FileAccess.get_file_as_string(src) if FileAccess.file_exists(src) else ""
	var tag: String = "%s (%s)" % [str(h["action"]), str(h["keys"])]
	checks[tag + ": group known"] = ControlsList.GROUPS.has(str(h["group"]))
	checks[tag + ": script exists"] = not text.is_empty()
	for token: Variant in DictUtil.array_of(h["tokens"]):
		checks["%s: %s in %s" % [tag, str(token), src.get_file()]] = text.contains(str(token))
return TestHelpers.report(TAG, "Hard-coded keys: each is in the source that handles it", checks)
```

### `static _menu(tree_parent: Node) -> Array`
```logic
var packed: PackedScene = load("res://scenes/main_menu.tscn") as PackedScene
var holder := SubViewport.new()
holder.size = Vector2i(VIEW)
tree_parent.add_child(holder)
var host: MainMenuView = packed.instantiate() as MainMenuView
holder.add_child(host)
return [holder, host]
```

### `static _free(holder: Node) -> void`
```logic
holder.get_parent().remove_child(holder)
holder.free()
```

### `static _controls_tab(host: MainMenuView) -> Control`
```logic
host.router.clear_to(MenuRouter.SETTINGS)
SettingsUiTestSuite.layout(host)
var tabs: TabContainer = SettingsUiTestSuite.tabs_of(host)
if tabs == null:
	return null
for i: int in tabs.get_tab_count():
	if tabs.get_tab_title(i) == SettingsView.TAB_CONTROLS:
		return tabs.get_tab_control(i)
return null
```

### `static _test_tab_actions(tree_parent: Node) -> bool`
```logic
var checks: Dictionary = {}
var pair: Array = _menu(tree_parent)
var tab: Control = _controls_tab(pair[1] as MainMenuView)
checks["the Controls tab exists"] = tab != null
if tab != null:
	for group: String in ControlsList.GROUPS:
		checks["group header " + group] = tab.find_child("ControlsGroup_" + group, true, false) != null
	var listed: int = 0
	for a: Array in ControlsList.INPUT_ACTIONS:
		var action: String = str(a[0])
		if not InputMap.has_action(action):
			continue
		listed += 1
		var name_label: Label = tab.find_child("ControlRow_action_" + action, true, false) as Label
		var keys_label: Label = tab.find_child("ControlKeys_action_" + action, true, false) as Label
		checks[action + ": row"] = name_label != null and name_label.text == str(a[1])
		checks[action + ": keys match InputMap"] = keys_label != null and keys_label.text == ControlsList.action_keys(action)
	checks["at least one InputMap action is listed"] = listed > 0
	checks["ui_cancel shows a key"] = ControlsList.action_keys("ui_cancel") != ""
_free(pair[0] as Node)
return TestHelpers.report(TAG, "Controls tab lists the InputMap actions", checks)
```

### `static _test_tab_hard_coded(tree_parent: Node) -> bool`
```logic
var checks: Dictionary = {}
var pair: Array = _menu(tree_parent)
var tab: Control = _controls_tab(pair[1] as MainMenuView)
checks["the Controls tab exists"] = tab != null
if tab != null:
	for i: int in ControlsList.HARD_CODED.size():
		var h: Dictionary = ControlsList.HARD_CODED[i]
		var name_label: Label = tab.find_child("ControlRow_key_%d" % i, true, false) as Label
		var keys_label: Label = tab.find_child("ControlKeys_key_%d" % i, true, false) as Label
		var tag: String = str(h["action"])
		checks[tag + ": row"] = name_label != null and name_label.text == tag
		checks[tag + ": keys"] = keys_label != null and keys_label.text == str(h["keys"])
_free(pair[0] as Node)
return TestHelpers.report(TAG, "Controls tab lists every hard-coded key", checks)
```

### `static _key(code: Key) -> InputEventKey`
```logic
var ev := InputEventKey.new()
ev.keycode = code
ev.physical_keycode = code
ev.pressed = true
return ev
```

### `static _game(tree_parent: Node, fight: bool) -> Dictionary`
```logic
var pair: Array = _menu(tree_parent)
var host: MainMenuView = pair[1] as MainMenuView
var gs: GameState = CurrentGame.state()
host.open_game(gs)
var view: RegionMapView = host.game_view as RegionMapView
if fight and view != null:
	view.pending_fight = {"kind": "fight", "source": "encounter", "marker": "", "encounter": ENCOUNTER, "elite": false, "from": [gs.position.x, gs.position.y]}
	view.launch_battle()
SettingsUiTestSuite.layout(host)
return {"holder": pair[0], "host": host, "view": view}
```

### `static _popup(host: MainMenuView) -> Control`
```logic
return host.find_child("PopupLayer", true, false) as Control
```

### `static _is_open(host: MainMenuView) -> bool`
```logic
var p: Control = _popup(host)
return host.router.top() == MenuRouter.SETTINGS and p != null and p.visible
```

### `static _menu_open(host: MainMenuView) -> bool`
```logic
var p: Control = _popup(host)
return host.router.top() == MenuRouter.HUB and p != null and p.visible
```

### `static _esc(host: MainMenuView) -> void`
```logic
host._unhandled_input(_key(KEY_ESCAPE))
```

### `static _open_close_checks(f: Dictionary, checks: Dictionary) -> void`
```logic
var host: MainMenuView = f["host"] as MainMenuView
checks["starts closed"] = not _menu_open(host) and host.router.top() == MenuRouter.PLAY
var pill: Button = host.find_child("MenuPill", true, false) as Button
checks["the Menu button shows during play"] = pill != null and pill.visible and pill.text.contains("Menu")
_esc(host)
checks["Esc opens the game menu"] = _menu_open(host)
checks["the Menu button hides while open"] = pill != null and not pill.visible
var entry: Button = host.find_child("Hub_" + MenuRouter.SETTINGS, true, false) as Button
checks["Settings is a menu entry"] = entry != null and not entry.disabled
if entry != null:
	entry.pressed.emit()
checks["the entry opens Settings"] = _is_open(host)
var tabs: TabContainer = SettingsUiTestSuite.tabs_of(host)
checks["all four tabs"] = tabs != null and tabs.get_tab_count() == 4 and tabs.get_tab_title(3) == SettingsView.TAB_CONTROLS
var close: Button = host.find_child("SettingsClose", true, false) as Button
checks["the cross exists"] = close != null
if close != null:
	close.pressed.emit()
checks["the cross goes back to the menu"] = not _is_open(host) and _menu_open(host)
_esc(host)
checks["Esc closes the menu"] = not _menu_open(host) and host.router.top() == MenuRouter.PLAY
checks["the Menu button is back"] = pill != null and pill.visible
pill.pressed.emit()
checks["the Menu button opens it"] = _menu_open(host)
_esc(host)
checks["Esc closes it"] = not _menu_open(host)
var ev := InputEventMouseButton.new()
ev.button_index = MOUSE_BUTTON_RIGHT
ev.pressed = true
_esc(host)
host._input(ev)
checks["right click closes it"] = not _menu_open(host) and host.router.top() == MenuRouter.PLAY
```

### `static _test_region(tree_parent: Node) -> bool`
```logic
var checks: Dictionary = {}
var f: Dictionary = _game(tree_parent, false)
checks["the region map is showing"] = f["view"] is RegionMapView and (f["host"] as MainMenuView).in_game()
_open_close_checks(f, checks)
_free(f["holder"] as Node)
return TestHelpers.report(TAG, "The game menu opens with Esc / Menu on the region map; Settings from it; the cross, Esc and right click close", checks)
```

### `static _test_combat(tree_parent: Node) -> bool`
```logic
var checks: Dictionary = {}
var f: Dictionary = _game(tree_parent, true)
var view: RegionMapView = f["view"] as RegionMapView
checks["a fight is mounted"] = view != null and view.battle != null
if view != null and view.battle != null:
	_open_close_checks(f, checks)
	checks["no results popup was created"] = not (f["host"] as MainMenuView).router.stack.has(MenuRouter.RESULTS)
	var dock: Variant = view.battle.arena.get("host") if view.battle.arena != null else null
	checks["combat keys are free again after closing"] = dock is CombatDockHost and not (dock as CombatDockHost).keys_blocked
_free(f["holder"] as Node)
return TestHelpers.report(TAG, "The game menu opens over a fight with Esc; Settings from it; the cross and Esc close", checks)
```

### `static _test_blocked_region(tree_parent: Node) -> bool`
```logic
var checks: Dictionary = {}
var f: Dictionary = _game(tree_parent, false)
var host: MainMenuView = f["host"] as MainMenuView
var view: RegionMapView = f["view"] as RegionMapView
checks["running before"] = view != null and view.can_process()
_esc(host)
checks["open"] = _menu_open(host)
checks["the map is paused (no key or mouse input reaches it)"] = view != null and not view.can_process()
_esc(host)
checks["closed: the map runs again"] = view != null and view.can_process()
_free(f["holder"] as Node)
return TestHelpers.report(TAG, "Input underneath is blocked on the region map while the game menu is open", checks)
```

### `static _test_blocked_combat(tree_parent: Node) -> bool`
```logic
var checks: Dictionary = {}
var f: Dictionary = _game(tree_parent, true)
var host: MainMenuView = f["host"] as MainMenuView
var view: RegionMapView = f["view"] as RegionMapView
var battle: BattleScreen = view.battle if view != null else null
checks["a fight is mounted"] = battle != null
if battle != null:
	var dock: CombatDockHost = battle.arena.get("host") as CombatDockHost
	checks["the arena has a dock host"] = dock != null
	if dock != null:
		checks["keys free before"] = not dock.keys_blocked
		_esc(host)
		checks["open"] = _menu_open(host)
		checks["keys_blocked while open"] = dock.keys_blocked
		checks["the fight is paused"] = not battle.can_process()
		var playing: bool = battle.session.is_auto_playing
		dock._unhandled_input(_key(KEY_SPACE))
		checks["Space does nothing under the popup"] = battle.session.is_auto_playing == playing
		_esc(host)
		checks["closed: keys_blocked cleared"] = not dock.keys_blocked and battle.can_process()
_free(f["holder"] as Node)
return TestHelpers.report(TAG, "Input underneath is blocked in combat while the game menu is open", checks)
```

