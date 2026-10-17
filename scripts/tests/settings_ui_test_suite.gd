class_name SettingsUiTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Settings UI Test Suite (SETTINGS-UI)
# Rules: animation speed and text size are fixed level lists (any value snaps to the nearest level, a tie to the lower);
# for every text-size level (and every speed level on the settings screen) the main menu, Load game, Save game and
# Settings screens build their controls, visible, inside a 1280x720 viewport, with no label/control overlap in a settings row.
# Layout is forced by hand (containers sorted top-down three times, wrapped labels re-measured between passes, one warm-up screen) because a one-shot headless pass leaves it unsorted;
# what is checked is the control rects after that pass, not pixels. Fixtures: sandbox settings file and save folder.
# ==============================================================================

const TAG: String = "SettingsUi"
const VIEW: Vector2 = Vector2(1280, 720)
const EXPECT_SPEEDS: Array[float] = [0.5, 1.0, 1.5, 2.0]
const EXPECT_TEXT: Array[float] = [0.85, 1.0, 1.15, 1.3]
static var SANDBOX: String = TestHelpers.sandbox("settings_ui")

static func run(tree_parent: Node = null) -> bool:
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
	CurrentGame.end()
	SettingsFile.use_path()
	SaveGame.reset_base_dir()
	FxSettings.set_speed(1.0)
	TestHelpers.remove_sandbox(SANDBOX)
	return ok

static func _test_levels() -> bool:
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

static func _test_snapping() -> bool:
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

static func _test_screens(tree_parent: Node) -> bool:
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
			var problems: Array[String] = _problems(view)
			checks[tag2 + ": fits"] = problems.is_empty()
			if not problems.is_empty():
				print("  ", tag2, " -> ", problems)
			checks[tag2 + ": controls"] = _settings_controls_ok(view, speed, scale)
			checks[tag2 + ": row cells do not overlap"] = _rows_ok(view)
	_open(view, MenuRouter.MAIN)
	host.queue_free()
	return TestHelpers.report(TAG, "Menu screens at every text-size and speed level (1280x720)", checks)

static func _open(view: MainMenuView, id: String) -> void:
	view.router.clear_to(id)   # one render per screen: the whole arena runs in one frame, so the message queue must stay small
	layout(view)

# Sorts every container top-down three times, re-measuring wrapped labels between passes, so rects are final in a headless one-shot pass.
static func layout(root: Control) -> void:
	for _pass: int in 3:
		_sort(root)
		for n: Node in root.find_children("*", "Label", true, false):
			var l: Label = n as Label
			if l != null and l.autowrap_mode != TextServer.AUTOWRAP_OFF:
				l.update_minimum_size()   # a wrapped label measured at width 1 is stale once its real width is known

static func _sort(n: Node) -> void:
	var ct: Container = n as Container
	if ct != null:
		ct.notification(Container.NOTIFICATION_SORT_CHILDREN)
	for c: Node in n.get_children():
		_sort(c)

static func _button(view: Node, text: String) -> Button:
	for n: Node in view.find_children("*", "Button", true, false):
		var b: Button = n as Button
		if b != null and b.text == text and b.is_visible_in_tree():
			return b
	return null

# Every visible control of the screen sits inside the viewport (controls under a ScrollContainer scroll by design and
# are skipped; the scroll itself is checked) and has a non-empty rect.
static func _problems(view: MainMenuView) -> Array[String]:
	var out: Array[String] = []
	var area := Rect2(Vector2.ZERO, VIEW)
	for n: Node in view.find_children("*", "Control", true, false):
		var c: Control = n as Control
		if c == null or not c.is_visible_in_tree() or c is ColorRect or _under_scroll(c, view):
			continue
		var r: Rect2 = c.get_global_rect()
		if r.size.x <= 0.0 or r.size.y <= 0.0:
			if c is Label or c is BaseButton:
				out.append("%s has an empty rect" % c.name)
			continue
		if not area.encloses(r):
			out.append("%s outside the viewport %s" % [c.name, r])
	return out

static func _under_scroll(c: Control, view: Node) -> bool:
	var p: Node = c.get_parent()
	while p != null and p != view:
		if p is ScrollContainer:
			return true
		p = p.get_parent()
	return false

static func _settings_controls_ok(view: MainMenuView, speed: float, scale: float) -> bool:
	var speed_pick: OptionButton = view.find_child("Levels_anim_speed", true, false) as OptionButton
	var text_pick: OptionButton = view.find_child("Levels_ui_text_scale", true, false) as OptionButton
	var check: CheckBox = view.find_child("Check_anim_instant", true, false) as CheckBox
	if speed_pick == null or text_pick == null or check == null:
		return false
	var volumes: int = 0
	for n: Node in view.find_children("Slider_audio_*", "HSlider", true, false):
		volumes += 1
	var speed_ok: bool = speed_pick.item_count == 4 and is_equal_approx(EXPECT_SPEEDS[speed_pick.selected], speed)
	var text_ok: bool = text_pick.item_count == 4 and is_equal_approx(EXPECT_TEXT[text_pick.selected], scale)
	var check_ok: bool = check.text == "Instant animations" and check.is_visible_in_tree() and check.get_theme_icon("unchecked").get_width() > 0
	var buttons_ok: bool = _button(view, "Apply") != null and _button(view, "Revert") != null and _button(view, "Back") != null
	# Centred: the grid's middle sits at the viewport's middle (within a pixel or two).
	var grid: Control = view.find_child("SettingsScroll", true, false) as Control   # DISPLAY1: the rows scroll; the scroll area is the centred column
	var centred: bool = grid != null and absf(grid.get_global_rect().get_center().x - VIEW.x / 2.0) <= 2.0
	return volumes == 4 and speed_ok and text_ok and check_ok and buttons_ok and centred

# Cells of the settings grid, three per row: label | control | value must not overlap and must share a vertical centre line
# (rows may differ in height, so the centres are compared within the row's tallest cell).
static func _rows_ok(view: MainMenuView) -> bool:
	var grid: GridContainer = view.find_child("SettingsGrid", true, false) as GridContainer
	if grid == null:
		return false
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
