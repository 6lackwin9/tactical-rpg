---
class: SettingsUiTestSuite
extends: RefCounted
description: "Tactical RPG - Settings UI Test Suite (SETTINGS-UI) Rules: animation speed and text size are fixed level lists (any value snaps to the nearest level, a tie to the lower); for every text-size level (and every speed level on the settings screen) the main menu, Load game, Save game and Settings screens build their controls, visible, inside a 1280x720 viewport, with no label/control overlap in a settings row. Layout is forced by hand (containers sorted top-down three times, wrapped labels re-measured between passes, one warm-up screen) because a one-shot headless pass leaves it unsorted; what is checked is the control rects after that pass, not pixels. Fixtures: sandbox settings file and save folder."
---

# Properties
* `TAG`: String = "SettingsUi" (const)
* `VIEW`: Vector2 = Vector2(1280, 720) (const)
* `EXPECT_SPEEDS`: Array[float] = [0.5, 1.0, 1.5, 2.0] (const)
* `EXPECT_TEXT`: Array[float] = [0.85, 1.0, 1.15, 1.3] (const)
* `out`: Array[String] = []
* `area`: Variant
* `c`: Control = n as Control
* `r`: Rect2 = c.get_global_rect()

# Methods

### `static run(tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING SETTINGS UI TEST (SETTINGS-UI)          ===")
print("=======================================================")
TestHelpers.remove_sandbox(SANDBOX)
DirAccess.make_dir_recursive_absolute(SANDBOX + "/saves")
SettingsFile.use_path(SANDBOX + "/settings.cfg")
SaveGame.set_base_dir(SANDBOX + "/saves")
var ok: bool = _test_levels()
ok = _test_snapping() and ok
if tree_parent != null:
	ok = _test_screens(tree_parent) and ok
	ok = _test_restore_defaults() and ok
	ok = _test_skip_row(tree_parent) and ok
	ok = _test_text_preview(tree_parent) and ok
	ok = _test_right_click(tree_parent) and ok
	ok = _test_popup(tree_parent) and ok
	ok = _test_close_cross(tree_parent) and ok
	ok = _test_tabs(tree_parent) and ok
CurrentGame.end()
SettingsFile.use_path()
SaveGame.reset_base_dir()
FxSettings.set_speed(1.0)
TestHelpers.remove_sandbox(SANDBOX)
return ok
```

### `static _test_levels() -> bool`
```logic
var checks: Dictionary = {
	"speed levels": SettingsFile.SPEED_LEVELS == EXPECT_SPEEDS,
	"text levels": SettingsFile.TEXT_LEVELS == EXPECT_TEXT,
	"model lists speed": SettingsModel.levels("anim/speed") == EXPECT_SPEEDS,
	"model lists text": SettingsModel.levels("ui/text_scale") == EXPECT_TEXT,
	"four names each": DictUtil.array_of(SettingsModel.LEVEL_NAMES["anim/speed"]).size() == 4 and DictUtil.array_of(SettingsModel.LEVEL_NAMES["ui/text_scale"]).size() == 4,
	"names": SettingsModel.level_text("anim/speed", 0) == "Slow (0.50x)" and SettingsModel.level_text("ui/text_scale", 3) == "Extra large (1.30x)",
	"fx bounds are the end levels": is_equal_approx(FxSettings.SPEED_MIN, EXPECT_SPEEDS[0]) and is_equal_approx(FxSettings.SPEED_MAX, EXPECT_SPEEDS[3]),
}
return TestHelpers.report(TAG, "Fixed level lists", checks)
```

### `static _test_snapping() -> bool`
```logic
var checks: Dictionary = {}
var cases: Array[Array] = [
	["anim/speed", 0.1, 0.5], ["anim/speed", 0.74, 0.5], ["anim/speed", 0.75, 0.5], ["anim/speed", 0.76, 1.0], ["anim/speed", 1.3, 1.5], ["anim/speed", 3.7, 2.0],
	["ui/text_scale", 0.8, 0.85], ["ui/text_scale", 1.07, 1.0], ["ui/text_scale", 1.12, 1.15], ["ui/text_scale", 1.5, 1.3], ["ui/text_scale", 2, 1.3],
]
for c: Array in cases:
	var key: String = str(c[0])
	var want: float = DictUtil.to_float(c[2])
	checks["%s %s -> %s (checked)" % [key, c[1], c[2]]] = is_equal_approx(DictUtil.to_float(SettingsFile.checked(key, c[1])), want)
# A saved off-list value loads as the nearest level.
var cfg := ConfigFile.new()
cfg.set_value("anim", "speed", 1.37)
cfg.set_value("ui", "text_scale", 0.9)
checks["file written"] = cfg.save(SettingsFile.path()) == OK
SettingsFile.load_file()
checks["file 1.37 loads as 1.5"] = is_equal_approx(SettingsFile.get_float("anim/speed"), 1.5)
checks["file 0.9 loads as 0.85"] = is_equal_approx(SettingsFile.get_float("ui/text_scale"), 0.85)
FxSettings.load_from(cfg)
checks["FxSettings snaps the file value"] = is_equal_approx(FxSettings.speed, 1.5)
FxSettings.set_speed(0.1)
checks["FxSettings keeps only levels"] = EXPECT_SPEEDS.has(FxSettings.speed) and is_equal_approx(FxSettings.speed, 0.5)
var _gone: Error = DirAccess.remove_absolute(SettingsFile.path())
SettingsFile.load_file()
return TestHelpers.report(TAG, "Off-list values snap to the nearest level", checks)
```

### `static _test_screens(tree_parent: Node) -> bool`
```logic
var packed: PackedScene = load("res://scenes/main_menu.tscn") as PackedScene
if packed == null:
	return TestHelpers.report(TAG, "Screens at every level", {"scene loads": false})
CurrentGame.end()
var host := Control.new()
host.size = VIEW
host.custom_minimum_size = VIEW
tree_parent.add_child(host)
var view: MainMenuView = packed.instantiate() as MainMenuView
host.add_child(view)
var checks: Dictionary = {}
_open(view, MenuRouter.MAIN)   # warm-up: the very first wrapped-label measurement in a process is transient
for scale: float in EXPECT_TEXT:
	SettingsFile.set_value("ui/text_scale", scale)
	view.apply_text_scale()
	for id: String in [MenuRouter.MAIN, MenuRouter.LOAD, MenuRouter.SAVE]:
		_open(view, id)
		var tag: String = "text %.2f %s" % [scale, id]
		checks[tag + ": fits"] = _problems(view).is_empty()
		checks[tag + ": Back/exit reachable"] = id == MenuRouter.MAIN or _button(view, "Back") != null
		if id == MenuRouter.MAIN:
			checks[tag + ": buttons"] = _button(view, "New game") != null and _button(view, "Settings") != null
	# The speed only changes which picker entry is selected (same entries, same widths): every speed at Normal text,
	# and Normal speed at every other text size.
	for speed: float in EXPECT_SPEEDS:
		if not is_equal_approx(scale, 1.0) and not is_equal_approx(speed, 1.0):
			continue
		SettingsFile.set_value("anim/speed", speed)
		_open(view, MenuRouter.SETTINGS)
		var tag2: String = "text %.2f speed %.1f settings" % [scale, speed]
		var problems: Array[String] = []
		for i: int in tabs_of(view).get_tab_count():
			select_tab(view, i)
			problems.append_array(_problems(view))
		checks[tag2 + ": fits"] = problems.is_empty()
		if not problems.is_empty():
			print("  ", tag2, " -> ", problems)
		checks[tag2 + ": controls"] = _settings_controls_ok(view, speed, scale)
		checks[tag2 + ": row cells do not overlap"] = _rows_ok(view)
_open(view, MenuRouter.MAIN)
host.queue_free()
return TestHelpers.report(TAG, "Menu screens at every text-size and speed level (1280x720)", checks)
```

### `static _test_restore_defaults() -> bool`
```logic
SettingsFile.set_value("audio/master", 0.2)
SettingsFile.set_value("anim/speed", 2.0)
SettingsFile.set_value("display/vsync", false)
var m := SettingsModel.new()
m.restore_defaults()
var all_default: bool = true
for key: String in SettingsFile.KEYS:
	all_default = all_default and m.value(key) == SettingsFile.default_of(key)
var checks: Dictionary = {
	"every key (display too) is the default": all_default,
	"it is a draft: the file keeps its values": is_equal_approx(SettingsFile.get_float("audio/master"), 0.2) and m.has_draft(),
}
m.revert()
checks["revert still drops the draft"] = not m.has_draft()
SettingsFile.use_path(SANDBOX + "/settings.cfg")   # forget the values set above
var _gone: Error = DirAccess.remove_absolute(SettingsFile.path())
SettingsFile.load_file()
return TestHelpers.report(TAG, "Restore defaults sets the draft to the defaults", checks)
```

### `static _screen(tree_parent: Node, id: String) -> Array`
```logic
var host := Control.new()
host.size = VIEW
host.custom_minimum_size = VIEW
tree_parent.add_child(host)
var view: MainMenuView = (load("res://scenes/main_menu.tscn") as PackedScene).instantiate() as MainMenuView
host.add_child(view)
_open(view, MenuRouter.MAIN)
view.router.push(id)
layout(view)
return [host, view]
```

### `static _test_skip_row(tree_parent: Node) -> bool`
```logic
var made: Array = _screen(tree_parent, MenuRouter.SETTINGS)
var host: Control = made[0] as Control
var view: MainMenuView = made[1] as MainMenuView
var slider: HSlider = view.find_child("Levels_anim_speed", true, false) as HSlider
var box: CheckBox = view.find_child("Check_anim_instant", true, false) as CheckBox
var checks: Dictionary = {"controls exist": slider != null and box != null}
if slider != null and box != null:
	var m: SettingsModel = view.router.model() as SettingsModel
	checks["speed slider enabled while unticked"] = slider.editable
	box.button_pressed = true
	checks["ticking disables the speed slider"] = not slider.editable and DictUtil.to_bool(m.value("anim/instant"))
	box.button_pressed = false
	checks["unticking enables it again"] = slider.editable and not DictUtil.to_bool(m.value("anim/instant"))
	slider.value = 3.0
	checks["slider writes the level to the draft"] = is_equal_approx(DictUtil.to_float(m.value("anim/speed")), EXPECT_SPEEDS[3])
	var name_label: Label = view.find_child("LevelName_anim_speed", true, false) as Label
	checks["value label names the level"] = name_label != null and name_label.text == SettingsModel.level_text("anim/speed", 3)
host.queue_free()
return TestHelpers.report(TAG, "Skip animations sits on the speed row and locks the speed slider", checks)
```

### `static _test_text_preview(tree_parent: Node) -> bool`
```logic
SettingsFile.set_value("ui/text_scale", 1.0)
var made: Array = _screen(tree_parent, MenuRouter.SETTINGS)
var host: Control = made[0] as Control
var view: MainMenuView = made[1] as MainMenuView
var slider: HSlider = view.find_child("Levels_ui_text_scale", true, false) as HSlider
var checks: Dictionary = {"slider exists": slider != null}
if slider != null:
	var base: int = roundi(MainMenuView.BASE_FONT * 1.0)
	slider.value = 3.0
	var big: int = roundi(MainMenuView.BASE_FONT * EXPECT_TEXT[3])
	checks["the menu rescales at once"] = view.theme.default_font_size == big and big > base
	checks["no sample label"] = view.find_child("TextSample", true, false) == null
	checks["nothing is saved yet"] = is_equal_approx(SettingsFile.get_float("ui/text_scale"), 1.0) and view.router.has_draft()
	view.cancel_pressed()   # closing asks about the unapplied change
	checks["Back asks"] = view.is_overlay_open() and view.router.is_asking()
	view.answer(true)
	checks["Back without Apply restores the saved size"] = view.router.top() == MenuRouter.MAIN and view.theme.default_font_size == base and is_equal_approx(SettingsFile.get_float("ui/text_scale"), 1.0)
host.queue_free()
return TestHelpers.report(TAG, "Text size previews live and Back without Apply restores it", checks)
```

### `static _push(view: Node, button: MouseButton, at: Vector2) -> void`
```logic
var ev := InputEventMouseButton.new()
ev.button_index = button
ev.pressed = true
ev.position = at
ev.global_position = at
view.get_viewport().push_input(ev)
```

### `static _test_right_click(tree_parent: Node) -> bool`
```logic
var made: Array = _screen(tree_parent, MenuRouter.LOAD)
var host: Control = made[0] as Control
var view: MainMenuView = made[1] as MainMenuView
var checks: Dictionary = {}
_push(view, MOUSE_BUTTON_LEFT, Vector2(2, 2))   # the empty margin
checks["left click does nothing"] = view.router.top() == MenuRouter.LOAD
var back: Button = view.find_child("BackButton", true, false) as Button
checks["Back button found"] = back != null
if back != null:
	_push(view, MOUSE_BUTTON_RIGHT, back.get_global_rect().get_center())   # over a button (a STOP control)
	checks["right click over a button goes back like Esc"] = view.router.top() == MenuRouter.MAIN
# The settings popup: right click over its panel (and over the backdrop) closes it; a question box takes it as No.
view.router.push(MenuRouter.SETTINGS)
layout(view)
var panel: Control = view.find_child("SettingsPanel", true, false) as Control
var m: SettingsModel = view.router.model() as SettingsModel
var _e: Array[String] = m.set_value("audio/master", 0.1)
_push(view, MOUSE_BUTTON_RIGHT, panel.get_global_rect().get_center())
checks["unapplied draft: right click over the panel asks"] = view.is_overlay_open() and view.router.is_asking() and view.router.top() == MenuRouter.SETTINGS
_push(view, MOUSE_BUTTON_RIGHT, Vector2(2, 2))
checks["with the question open it answers No (stays)"] = not view.is_overlay_open() and not view.router.is_asking() and view.router.top() == MenuRouter.SETTINGS
m.revert()
_push(view, MOUSE_BUTTON_RIGHT, Vector2(2, 2))   # over the dimmed backdrop
checks["right click on the backdrop closes the popup"] = view.router.top() == MenuRouter.MAIN
# The game view (map / fight) keeps the right button.
var fake := Control.new()
view.game_view = fake
_push(view, MOUSE_BUTTON_RIGHT, Vector2(2, 2))
checks["nothing happens while the game view shows"] = view.router.top() == MenuRouter.MAIN and not view.is_overlay_open()
view.game_view = null
fake.free()
host.queue_free()
return TestHelpers.report(TAG, "Right click is Back, like Esc (real viewport input)", checks)
```

### `static _test_popup(tree_parent: Node) -> bool`
```logic
var checks: Dictionary = {}
SettingsFile.set_value("ui/text_scale", 1.0)
var made: Array = _screen(tree_parent, MenuRouter.SETTINGS)
var host: Control = made[0] as Control
var view: MainMenuView = made[1] as MainMenuView
var panel: Control = view.find_child("SettingsPanel", true, false) as Control
var below: Button = _button(view, "New game")
checks["popup layout"] = popup_ok(view, VIEW)
checks["screen below still in the tree"] = below != null and below.focus_mode == Control.FOCUS_NONE and panel != null and not panel.is_ancestor_of(below)
_push(view, MOUSE_BUTTON_LEFT, Vector2(2, 2))   # on the dimmed backdrop
_push(view, MOUSE_BUTTON_LEFT, below.get_global_rect().get_center())   # on the dimmed screen below
checks["backdrop click does nothing"] = view.router.top() == MenuRouter.SETTINGS and not view.is_overlay_open()
view.cancel_pressed()
var gone: Control = view.find_child("PopupBackdrop", true, false) as Control
checks["popup hidden after close"] = view.router.top() == MenuRouter.MAIN and gone != null and not gone.is_visible_in_tree() and _button(view, "New game") != null
host.queue_free()
return TestHelpers.report(TAG, "Settings is a popup over the screen it came from", checks)
```

### `static _test_close_cross(tree_parent: Node) -> bool`
```logic
var made: Array = _screen(tree_parent, MenuRouter.SETTINGS)
var host: Control = made[0] as Control
var view: MainMenuView = made[1] as MainMenuView
var close: Button = view.find_child("SettingsClose", true, false) as Button
var checks: Dictionary = {"cross exists": close != null}
if close != null:
	var m: SettingsModel = view.router.model() as SettingsModel
	var _e: Array[String] = m.set_value("audio/master", 0.15)
	close.pressed.emit()
	checks["draft: cross asks"] = view.is_overlay_open() and view.router.is_asking() and view.router.top() == MenuRouter.SETTINGS
	view.answer(false)
	checks["No keeps the popup"] = view.router.top() == MenuRouter.SETTINGS
	close.pressed.emit()
	view.answer(true)
	checks["Yes closes it"] = view.router.top() == MenuRouter.MAIN
host.queue_free()
return TestHelpers.report(TAG, "The close cross is Esc", checks)
```

### `static _test_tabs(tree_parent: Node) -> bool`
```logic
var made: Array = _screen(tree_parent, MenuRouter.SETTINGS)
var host: Control = made[0] as Control
var view: MainMenuView = made[1] as MainMenuView
var tabs: TabContainer = tabs_of(view)
var checks: Dictionary = {"tabs exist": tabs != null}
if tabs != null:
	var panel: Control = view.find_child("SettingsPanel", true, false) as Control
	var rect: Rect2 = panel.get_global_rect()
	var sound_ok: bool = tab_node(view, 0, "Slider_audio_master") != null and tab_node(view, 0, "Slider_audio_ui") != null and tab_node(view, 0, "Levels_anim_speed") == null
	var display_ok: bool = tab_node(view, 1, "Choice_display_window_mode") != null and tab_node(view, 1, "Levels_ui_text_scale") != null and tab_node(view, 1, "DisplayNow") != null and tab_node(view, 1, "Slider_audio_master") == null
	var play_ok: bool = tab_node(view, 2, "Levels_anim_speed") != null and tab_node(view, 2, "Check_anim_instant") != null and tab_node(view, 2, "Levels_ui_text_scale") == null
	checks["rows by tab"] = sound_ok and display_ok and play_ok
	var same: bool = true
	for i: int in [1, 2, 0, 2]:
		select_tab(view, i)
		same = same and panel.get_global_rect() == rect and tabs.get_current_tab_control().is_visible_in_tree()
	checks["switching tabs keeps the panel rect"] = same
	checks["tab bar is keyboard focusable"] = tabs.focus_mode == Control.FOCUS_ALL and tabs.get_tab_bar().focus_mode != Control.FOCUS_NONE
	select_tab(view, 1)
	var restore: Button = _button(view, "Restore defaults")
	checks["Restore defaults says it resets all tabs"] = restore != null and restore.tooltip_text.contains("every tab")
	var m: SettingsModel = view.router.model() as SettingsModel
	var _e: Array[String] = m.set_value("audio/master", 0.1)
	restore.pressed.emit()
	checks["Restore defaults resets every tab, stays on the tab"] = is_equal_approx(DictUtil.to_float(m.value("audio/master")), DictUtil.to_float(SettingsFile.default_of("audio/master"))) and tabs_of(view).current_tab == 1
host.queue_free()
return TestHelpers.report(TAG, "Settings tabs", checks)
```

### `static _open(view: MainMenuView, id: String) -> void`
```logic
view.router.clear_to(id)   # one render per screen: the whole arena runs in one frame, so the message queue must stay small
layout(view)
```

### `static layout(root: Control) -> void`
```logic
for _pass: int in 3:
	_sort(root)
	for n: Node in root.find_children("*", "Label", true, false):
		var l: Label = n as Label
		if l != null and l.autowrap_mode != TextServer.AUTOWRAP_OFF:
			l.update_minimum_size()   # a wrapped label measured at width 1 is stale once its real width is known
```

### `static _sort(n: Node) -> void`
```logic
var ct: Container = n as Container
if ct != null:
	ct.notification(Container.NOTIFICATION_SORT_CHILDREN)
for c: Node in n.get_children():
	_sort(c)
```

### `static _button(view: Node, text: String) -> Button`
```logic
for n: Node in view.find_children("*", "Button", true, false):
	var b: Button = n as Button
	if b != null and b.text == text and b.is_visible_in_tree():
		return b
return null
```

### `static _under_scroll(c: Control, view: Node) -> bool`
```logic
var p: Node = c.get_parent()
while p != null and p != view:
	if p is ScrollContainer:
		return true
	p = p.get_parent()
return false
```

### `static tabs_of(view: Node) -> TabContainer`
```logic
return view.find_child("SettingsTabs", true, false) as TabContainer
```

### `static select_tab(view: MainMenuView, i: int) -> void`
```logic
var tabs: TabContainer = tabs_of(view)
if tabs != null:
	tabs.current_tab = i
layout(view)
```

### `static tab_node(view: Node, i: int, child: String) -> Control`
```logic
var tabs: TabContainer = tabs_of(view)
return tabs.get_tab_control(i).find_child(child, true, false) as Control if tabs != null else null
```

### `static _settings_controls_ok(view: MainMenuView, speed: float, scale: float) -> bool`
```logic
var tabs: TabContainer = tabs_of(view)
if tabs == null or tabs.get_tab_count() != 4 or tabs.get_tab_title(0) != "Sound" or tabs.get_tab_title(1) != "Display" or tabs.get_tab_title(2) != "Gameplay" or tabs.get_tab_title(3) != "Controls":
	return false
var speed_pick: HSlider = tab_node(view, 2, "Levels_anim_speed") as HSlider
var text_pick: HSlider = tab_node(view, 1, "Levels_ui_text_scale") as HSlider
var check: CheckBox = tab_node(view, 2, "Check_anim_instant") as CheckBox
if speed_pick == null or text_pick == null or check == null:
	return false
var volumes: int = tabs.get_tab_control(0).find_children("Slider_audio_*", "HSlider", true, false).size()
var display_rows: bool = tab_node(view, 1, "Choice_display_window_mode") != null and tab_node(view, 1, "Choice_display_resolution") != null and tab_node(view, 1, "DisplayNow") != null and tab_node(view, 1, "Check_display_vsync") != null and tab_node(view, 1, "Choice_display_max_fps") != null
var speed_ok: bool = _notches_ok(speed_pick, EXPECT_SPEEDS) and is_equal_approx(EXPECT_SPEEDS[roundi(speed_pick.value)], speed)
var text_ok: bool = _notches_ok(text_pick, EXPECT_TEXT) and is_equal_approx(EXPECT_TEXT[roundi(text_pick.value)], scale)
var check_ok: bool = check.text == "Skip animations" and check.get_theme_icon("unchecked").get_width() > 0
var buttons_ok: bool = _button(view, "Apply") != null and _button(view, "Restore defaults") != null and _button(view, "Back") == null and _button(view, "Revert") == null
var no_sample: bool = view.find_child("TextSample", true, false) == null
var panel: Control = view.find_child("SettingsPanel", true, false) as Control
var rect: Rect2 = panel.get_global_rect()
var layout_ok: bool = true
for i: int in 3:
	select_tab(view, i)
	var scroll: Control = tabs.get_current_tab_control()
	# Centred: the tab's scroll area sits at the middle of the viewport; the panel keeps its rect on every tab; the footer is outside the scroll.
	layout_ok = layout_ok and absf(scroll.get_global_rect().get_center().x - VIEW.x / 2.0) <= 2.0 and panel.get_global_rect() == rect and _footer_ok(view, tabs) and popup_ok(view, VIEW)
select_tab(view, 2)
# MENU-POLISH: the Skip box sits to the right of the speed slider on its row.
var cell: Control = speed_pick.get_parent() as Control
var on_row: bool = cell != null and check.get_global_rect().position.x >= cell.get_global_rect().end.x and absf(check.get_global_rect().get_center().y - speed_pick.get_global_rect().get_center().y) <= cell.get_global_rect().size.y
select_tab(view, 0)
return volumes == 4 and display_rows and speed_ok and text_ok and check_ok and buttons_ok and no_sample and layout_ok and on_row
```

### `static _footer_ok(view: MainMenuView, scroll: Control) -> bool`
```logic
var panel: Control = view.find_child("SettingsPanel", true, false) as Control
var footer: Control = view.find_child("SettingsFooter", true, false) as Control
var apply: Button = _button(view, "Apply")
var restore: Button = _button(view, "Restore defaults")
if panel == null or footer == null or scroll == null or apply == null or restore == null:
	return false
var pr: Rect2 = panel.get_global_rect()
var ar: Rect2 = apply.get_global_rect()
var rr: Rect2 = restore.get_global_rect()
return not _under_scroll(footer, view) and footer.get_global_rect().position.y >= scroll.get_global_rect().end.y - 1.0 and footer.is_ancestor_of(apply) and footer.is_ancestor_of(restore) \
	and ar.end.x > pr.position.x + pr.size.x * 0.75 and ar.end.y > pr.position.y + pr.size.y * 0.75 and rr.end.x <= ar.position.x + 1.0 and pr.encloses(ar) and pr.encloses(rr) and restore.flat
```

### `static popup_ok(view: MainMenuView, logical: Vector2) -> bool`
```logic
var panel: Control = view.find_child("SettingsPanel", true, false) as Control
var dim: Control = view.find_child("PopupBackdrop", true, false) as Control
var close: Button = view.find_child("SettingsClose", true, false) as Button
var header: Control = view.find_child("SettingsHeader", true, false) as Control
if panel == null or dim == null or close == null or header == null:
	return false
var pr: Rect2 = panel.get_global_rect()
var area := Rect2(Vector2.ZERO, logical)
var centred: bool = absf(pr.get_center().x - logical.x / 2.0) <= 2.0 and absf(pr.get_center().y - logical.y / 2.0) <= 2.0
var margin: bool = area.grow(-MainMenuView.POPUP_MARGIN + 1.0).encloses(pr) and pr.size.x <= MainMenuView.POPUP_MAX.x + 1.0 and pr.size.y <= MainMenuView.POPUP_MAX.y + 1.0
var cr: Rect2 = close.get_global_rect()
var cross: bool = header.is_ancestor_of(close) and pr.encloses(cr) and cr.end.x >= pr.end.x - 40.0 and cr.position.y <= pr.position.y + 60.0 and close.is_visible_in_tree()
var dimmed: bool = dim.is_visible_in_tree() and dim.get_global_rect().encloses(area) and dim.mouse_filter == Control.MOUSE_FILTER_STOP
return centred and margin and cross and dimmed and _button(view, "Back") == null
```

### `static _notches_ok(s: HSlider, levels: Array[float]) -> bool`
```logic
return s.tick_count == levels.size() and s.min_value == 0.0 and s.max_value == float(levels.size() - 1) and s.step == 1.0 and not s.scrollable
```

### `static _main_menu_even(view: MainMenuView) -> bool`
```logic
var width: float = -1.0
var count: int = 0
for n: Node in view.find_children("*", "Button", true, false):
	var b: Button = n as Button
	if b == null or not b.is_visible_in_tree():
		continue
	var r: Rect2 = b.get_global_rect()
	count += 1
	if width < 0.0:
		width = r.size.x
	if absf(r.size.x - width) > 1.0 or absf(r.get_center().x - VIEW.x / 2.0) > 2.0 or b.alignment != HORIZONTAL_ALIGNMENT_CENTER:
		return false
for n: Node in view.find_children("*", "Label", true, false):
	var l: Label = n as Label
	if l != null and l.is_visible_in_tree() and (l.text == "Tactical RPG" or l.text.begins_with("A small campaign")):
		if l.horizontal_alignment != HORIZONTAL_ALIGNMENT_CENTER or absf(l.get_global_rect().get_center().x - VIEW.x / 2.0) > 2.0:
			return false
return count >= 3
```

### `static _rows_ok(view: MainMenuView) -> bool`
```logic
var tabs: TabContainer = tabs_of(view)
if tabs == null:
	return false
for t: int in tabs.get_tab_count():
	select_tab(view, t)
	var grid: GridContainer = tabs.get_current_tab_control().get_child(0) as GridContainer
	if grid == null or not _grid_rows_ok(grid):
		return false
select_tab(view, 0)
return true
```

### `static _grid_rows_ok(grid: GridContainer) -> bool`
```logic
var cells: Array[Control] = []
for n: Node in grid.get_children():
	cells.append(n as Control)
if cells.size() % 3 != 0 or cells.is_empty():
	return false
for i: int in range(0, cells.size(), 3):
	for a: int in 3:
		for b: int in range(a + 1, 3):
			if cells[i + a].get_global_rect().intersects(cells[i + b].get_global_rect()):
				return false
	var first: Rect2 = cells[i].get_global_rect()
	var second: Rect2 = cells[i + 1].get_global_rect()
	if first.size.x > 0.0 and second.size.x > 0.0 and absf(first.get_center().y - second.get_center().y) > 3.0 and not (cells[i] is CheckBox):
		return false
return true
```

