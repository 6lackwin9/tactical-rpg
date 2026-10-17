class_name DisplayTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Display Test Suite (DISPLAY1)
# Rules: the four display keys round-trip through settings.cfg (an off-list value reads as the default, save slots never touch
# them); each window mode / vsync / cap value makes the matching backend call; the window size list only has sizes that fit the
# screen and a saved size that does not fit applies the largest that does without rewriting the file; a changed mode or size is
# applied live, written only on Keep, and goes back on Revert / Esc / the 10 s timeout; a changed cap starts no countdown; the web
# build shows only a Fullscreen checkbox and writes no display key; the Settings screen, the countdown box and the main menu fit
# logical sizes 1280x720, 1280x800 and 1680x720 at every text size. Every window call goes to a recording backend, never the real window.
# Fixtures: sandbox settings file and save folder.
# ==============================================================================

const TAG: String = "Display"
const LOGICAL_SIZES: Array[Vector2] = [Vector2(1280, 720), Vector2(1280, 800), Vector2(1680, 720)]
const TEXT_SCALES: Array[float] = [0.85, 1.0, 1.15, 1.3]
static var SANDBOX: String = TestHelpers.sandbox("display")

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING DISPLAY TEST (DISPLAY1)                 ===")
	print("=======================================================")
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/saves")
	SettingsFile.use_path(SANDBOX + "/settings.cfg")
	SaveGame.set_base_dir(SANDBOX + "/saves")
	var ok: bool = _test_keys()
	ok = _test_mapping() and ok
	ok = _test_fit() and ok
	ok = _test_apply_calls() and ok
	ok = _test_countdown() and ok
	ok = _test_web_model() and ok
	if tree_parent != null:
		ok = _test_web_view(tree_parent) and ok
		for logical: Vector2 in LOGICAL_SIZES:
			ok = _test_layout(tree_parent, logical) and ok
	DisplaySettings.use_backend()
	DisplaySettings.web_override = -1
	DisplaySettings.reset_boot()
	SettingsFile.use_path()
	SaveGame.reset_base_dir()
	TestHelpers.remove_sandbox(SANDBOX)
	return ok

static func _fresh() -> DisplaySettings.Backend:
	var b := DisplaySettings.Backend.new()
	DisplaySettings.use_backend(b)
	DisplaySettings.web_override = 0
	var _gone: Error = DirAccess.remove_absolute(SettingsFile.path())
	SettingsFile.load_file()
	return b

static func _file_has(key: String) -> bool:
	var cfg := ConfigFile.new()
	var _e: Error = cfg.load(SettingsFile.path())
	var parts: PackedStringArray = key.split("/")
	return cfg.has_section_key(parts[0], parts[1])

static func _raw(key: String) -> Variant:
	var cfg := ConfigFile.new()
	var _e: Error = cfg.load(SettingsFile.path())
	var parts: PackedStringArray = key.split("/")
	return cfg.get_value(parts[0], parts[1], null)

static func _test_keys() -> bool:
	var _b: DisplaySettings.Backend = _fresh()
	var checks: Dictionary = {}
	var samples: Dictionary = {"display/window_mode": "fullscreen", "display/resolution": "1920x1080", "display/vsync": false, "display/max_fps": 144}
	for key: String in samples:
		var dflt: Variant = SettingsFile.default_of(key)
		checks[key + " has a default"] = dflt != null and typeof(dflt) == DictUtil.to_int(DictUtil.dict_of(SettingsFile.KEYS[key])["type"])
		checks[key + " differs from default in the sample"] = samples[key] != dflt
		SettingsFile.set_value(key, samples[key])
	checks["saved"] = SettingsFile.save() == OK
	SettingsFile.load_file()
	for key: String in samples:
		checks[key + " round-trips"] = SettingsFile.get_value(key) == samples[key]
	# Every listed value is accepted, a value outside the list reads as the default.
	for key: String in samples:
		var row: Dictionary = DictUtil.dict_of(SettingsFile.KEYS[key])
		for allowed: Variant in DictUtil.array_of(row.get("choices", [])):
			checks["%s accepts %s" % [key, str(allowed)]] = SettingsFile.checked(key, allowed) == allowed
	checks["mode off-list"] = SettingsFile.checked("display/window_mode", "sideways") == SettingsFile.default_of("display/window_mode")
	checks["size off-list"] = SettingsFile.checked("display/resolution", "800x600") == SettingsFile.default_of("display/resolution")
	checks["cap off-list"] = SettingsFile.checked("display/max_fps", 75) == SettingsFile.default_of("display/max_fps")
	checks["vsync wrong type"] = SettingsFile.checked("display/vsync", "yes") == SettingsFile.default_of("display/vsync")
	var cfg := ConfigFile.new()
	var _e: Error = cfg.load(SettingsFile.path())
	cfg.set_value("display", "window_mode", "sideways")
	cfg.set_value("display", "max_fps", 75)
	checks["bad file written"] = cfg.save(SettingsFile.path()) == OK
	SettingsFile.load_file()
	checks["file off-list mode reads default"] = SettingsFile.get_value("display/window_mode") == "windowed"
	checks["file off-list cap reads default"] = SettingsFile.get_value("display/max_fps") == 0
	# Save slots never touch the file.
	var before: String = FileAccess.get_file_as_string(SettingsFile.path())
	for slot: int in range(0, 6):
		var _r: Dictionary = SaveGame.read(slot)
		var _d: bool = SaveGame.delete(slot)
	checks["slot read / delete leave the settings file alone"] = FileAccess.get_file_as_string(SettingsFile.path()) == before
	return TestHelpers.report(TAG, "Display keys: defaults, round trip, off-list values", checks)

static func _test_mapping() -> bool:
	var checks: Dictionary = {}
	var expect: Dictionary = {"windowed": DisplayServer.WINDOW_MODE_WINDOWED, "borderless": DisplayServer.WINDOW_MODE_FULLSCREEN, "fullscreen": DisplayServer.WINDOW_MODE_EXCLUSIVE_FULLSCREEN}
	for mode_id: String in expect:
		var b: DisplaySettings.Backend = _fresh()
		DisplaySettings.apply_mode_size(mode_id, "1600x900")
		checks[mode_id + " -> mode call"] = b.mode == DictUtil.to_int(expect[mode_id]) and b.calls.has("mode:%d" % DictUtil.to_int(expect[mode_id]))
		checks[mode_id + ": size only when windowed"] = b.calls.has("size:1600x900") == (mode_id == "windowed")
		checks[mode_id + ": centred only when windowed"] = (b.centred == 1) == (mode_id == "windowed")
	for on: bool in [true, false]:
		var b: DisplaySettings.Backend = _fresh()
		DisplaySettings.apply_vsync_cap(on, 0)
		checks["vsync %s" % str(on)] = b.vsync == on and b.calls.has("vsync:%s" % str(on))
	for cap: int in [30, 60, 120, 144, 0]:
		var b: DisplaySettings.Backend = _fresh()
		var m := SettingsModel.new()
		var _e: Array[String] = m.set_value("display/max_fps", cap)
		var _v: Array[String] = m.set_value("display/vsync", false)
		var errs: Array[String] = m.apply()
		checks["cap %d applied on Apply" % cap] = errs.is_empty() and b.fps == cap and not b.vsync
		checks["cap %d: no countdown" % cap] = not m.confirming() and is_zero_approx(m.confirm_left)
		checks["cap %d: written" % cap] = SettingsFile.get_value("display/max_fps") == cap and _file_has("display/max_fps")
		checks["cap %d: no mode call" % cap] = not _has_prefix(b.calls, "mode:") and not _has_prefix(b.calls, "size:")
	# Boot applies all four once.
	var boot: DisplaySettings.Backend = _fresh()
	SettingsFile.set_value("display/window_mode", "borderless")
	SettingsFile.set_value("display/max_fps", 60)
	DisplaySettings.reset_boot()
	DisplaySettings.apply_saved()
	DisplaySettings.apply_saved()
	checks["boot applies mode, vsync and cap once"] = boot.mode == DisplayServer.WINDOW_MODE_FULLSCREEN and boot.fps == 60 and boot.calls.size() == 3
	var web_boot: DisplaySettings.Backend = _fresh()
	DisplaySettings.web_override = 1
	DisplaySettings.reset_boot()
	DisplaySettings.apply_saved()
	checks["boot does nothing on web"] = web_boot.calls.is_empty()
	DisplaySettings.web_override = 0
	return TestHelpers.report(TAG, "Backend calls for each mode, vsync and cap", checks)

static func _has_prefix(calls: Array[String], prefix: String) -> bool:
	for c: String in calls:
		if c.begins_with(prefix):
			return true
	return false

static func _test_fit() -> bool:
	var b: DisplaySettings.Backend = _fresh()
	var checks: Dictionary = {}
	b.usable = Vector2i(1600, 900)
	checks["1600x900 screen offers every listed size up to 1600x900"] = DisplaySettings.sizes() == (["960x540", "1024x576", "1152x648", "1280x720", "1366x768", "1600x900"] as Array[String])
	b.usable = Vector2i(1000, 600)
	checks["a small screen offers what fits plus 1280x720"] = DisplaySettings.sizes() == (["960x540", "1280x720"] as Array[String])
	b.usable = Vector2i(2880, 1800)
	b.scale = 2.0
	checks["a 200% screen lists logical sizes up to half the physical width"] = DisplaySettings.sizes() == (["960x540", "1024x576", "1152x648", "1280x720", "1366x768"] as Array[String])
	b.scale = 1.0
	b.usable = Vector2i(1600, 900)
	SettingsFile.set_value("display/resolution", "2560x1440")
	checks["saved file written"] = SettingsFile.save() == OK
	var before: String = FileAccess.get_file_as_string(SettingsFile.path())
	DisplaySettings.reset_boot()
	DisplaySettings.apply_saved()
	checks["a saved 2560x1440 applies 1600x900"] = b.size == Vector2i(1600, 900)
	checks["the file stays at 2560x1440"] = FileAccess.get_file_as_string(SettingsFile.path()) == before and SettingsFile.get_value("display/resolution") == "2560x1440"
	var m := SettingsModel.new()
	checks["the draft keeps 2560x1440"] = m.value("display/resolution") == "2560x1440"
	checks["picker list is the fitting sizes"] = SettingsModel.choices("display/resolution") == DisplaySettings.sizes()
	return TestHelpers.report(TAG, "Window size list fits the screen", checks)

# Apply reaches the backend (mode, size, vsync, cap), and the "In use" text reads the backend back.
static func _test_apply_calls() -> bool:
	var b: DisplaySettings.Backend = _fresh()
	var checks: Dictionary = {}
	var m := SettingsModel.new()
	var _a: Array[String] = m.set_value("display/window_mode", "windowed")
	var _b: Array[String] = m.set_value("display/resolution", "960x540")
	var _c: Array[String] = m.set_value("display/vsync", false)
	var _d: Array[String] = m.set_value("display/max_fps", 60)
	var _e: Array[String] = m.apply()
	checks["size goes to the backend"] = b.calls.has("size:960x540") and b.size == Vector2i(960, 540)
	checks["vsync and cap go to the backend"] = b.calls.has("vsync:false") and b.calls.has("fps:60")
	checks["a confirm is open"] = m.confirming()
	checks["In use reads the backend back"] = DisplaySettings.now_text() == "Windowed 960x540"
	m.revert_display()
	b.mode = DisplayServer.WINDOW_MODE_EXCLUSIVE_FULLSCREEN
	checks["In use shows the mode"] = DisplaySettings.now_text().begins_with("Fullscreen")
	return TestHelpers.report(TAG, "Apply calls the window backend and the settings show what is in use", checks)

static func _test_countdown() -> bool:
	var checks: Dictionary = {}
	var cases: Array[String] = ["revert", "esc", "timeout"]
	for how: String in cases:
		var b: DisplaySettings.Backend = _fresh()
		SettingsFile.set_value("display/window_mode", "windowed")
		SettingsFile.set_value("display/resolution", "1280x720")
		var _s: Error = SettingsFile.save()
		var m := SettingsModel.new()
		var _a: Array[String] = m.set_value("display/window_mode", "fullscreen")
		var errs: Array[String] = m.apply()
		checks[how + ": apply ok"] = errs.is_empty()
		checks[how + ": 10 s countdown started"] = m.confirming() and is_equal_approx(m.confirm_left, 10.0)
		checks[how + ": live"] = b.mode == DisplayServer.WINDOW_MODE_EXCLUSIVE_FULLSCREEN
		checks[how + ": not written yet"] = SettingsFile.get_value("display/window_mode") == "windowed" and _raw("display/window_mode") == "windowed"
		if how == "timeout":
			checks[how + ": 9 s is not enough"] = not m.tick(9.0) and m.confirming()
			checks[how + ": 10 s reverts"] = m.tick(1.0) and not m.confirming()
		else:
			m.revert_display()   # Esc and the Revert button both call this
			checks[how + ": reverted"] = not m.confirming()
		checks[how + ": mode restored live"] = b.mode == DisplayServer.WINDOW_MODE_WINDOWED and b.calls.back() == "centre"
		checks[how + ": draft restored"] = m.value("display/window_mode") == "windowed"
		checks[how + ": file unchanged"] = SettingsFile.get_value("display/window_mode") == "windowed"
		SettingsFile.load_file()
		checks[how + ": file still windowed after reload"] = SettingsFile.get_value("display/window_mode") == "windowed"
	# Keep writes both keys; a size change alone also counts.
	var kb: DisplaySettings.Backend = _fresh()
	var km := SettingsModel.new()
	var _k1: Array[String] = km.set_value("display/window_mode", "borderless")
	var _k2: Array[String] = km.set_value("display/resolution", "1920x1080")
	var _k3: Array[String] = km.apply()
	checks["keep: countdown started"] = km.confirming()
	checks["keep: nothing written during the countdown"] = not _file_has("display/window_mode") and not _file_has("display/resolution")
	checks["keep: no error"] = km.keep_display().is_empty()
	checks["keep: both keys written"] = _file_has("display/window_mode") and _file_has("display/resolution")
	SettingsFile.load_file()
	checks["keep: values"] = SettingsFile.get_value("display/window_mode") == "borderless" and SettingsFile.get_value("display/resolution") == "1920x1080"
	checks["keep: closed"] = not km.confirming() and kb.mode == DisplayServer.WINDOW_MODE_FULLSCREEN
	var _t: bool = km.tick(100.0)
	checks["keep: time passing changes nothing"] = kb.mode == DisplayServer.WINDOW_MODE_FULLSCREEN
	# Size only (windowed) starts the countdown, an unchanged mode and size does not.
	var sb: DisplaySettings.Backend = _fresh()
	var sm := SettingsModel.new()
	var _n1: Array[String] = sm.set_value("display/resolution", "1600x900")
	var _n2: Array[String] = sm.apply()
	checks["size change starts a countdown and resizes"] = sm.confirming() and sb.size == Vector2i(1600, 900)
	sm.revert_display()
	checks["size change reverts to the old size"] = sb.size == Vector2i(1280, 720)
	var um := SettingsModel.new()
	sb.calls.clear()
	var _u: Array[String] = um.apply()
	checks["no display change: no countdown, no window call but vsync and cap"] = not um.confirming() and sb.calls == (["vsync:true", "fps:0"] as Array[String])
	return TestHelpers.report(TAG, "Apply, Keep, Revert, Esc and timeout", checks)

static func _test_web_model() -> bool:
	var b: DisplaySettings.Backend = _fresh()
	DisplaySettings.web_override = 1
	var m := SettingsModel.new()
	var _a: Array[String] = m.set_value("display/window_mode", "fullscreen")
	var _c: Array[String] = m.set_value("display/max_fps", 60)
	var _s: Array[String] = m.set_value("audio/music", 0.5)
	var errs: Array[String] = m.apply()
	var checks: Dictionary = {
		"apply ok": errs.is_empty(),
		"no countdown": not m.confirming(),
		"no window call": b.calls.is_empty(),
		"other keys still written": _file_has("audio/music"),
	}
	for key: String in SettingsModel.DISPLAY_KEYS:
		checks["no " + key] = not _file_has(key)
	DisplaySettings.web_override = 0
	return TestHelpers.report(TAG, "Web build writes no display key", checks)

static func _host(tree_parent: Node, logical: Vector2) -> Control:
	var host := Control.new()
	host.size = logical
	host.custom_minimum_size = logical
	tree_parent.add_child(host)
	return host

static func _menu(host: Control) -> MainMenuView:
	var packed: PackedScene = load("res://scenes/main_menu.tscn") as PackedScene
	if packed == null:
		return null
	var view: MainMenuView = packed.instantiate() as MainMenuView
	host.add_child(view)
	return view

static func _test_web_view(tree_parent: Node) -> bool:
	var b: DisplaySettings.Backend = _fresh()
	DisplaySettings.web_override = 1
	CurrentGame.end()
	var host: Control = _host(tree_parent, Vector2(1280, 720))
	var view: MainMenuView = _menu(host)
	if view == null:
		return TestHelpers.report(TAG, "Web Settings screen", {"scene loads": false})
	SettingsUiTestSuite.layout(view)
	SettingsUiTestSuite.layout(view)
	SettingsUiTestSuite._open(view, MenuRouter.SETTINGS)
	var box: CheckBox = view.find_child("Check_display_fullscreen", true, false) as CheckBox
	var checks: Dictionary = {
		"Fullscreen row present": box != null and box.text == "Fullscreen",
		"no window mode row": view.find_child("Choice_display_window_mode", true, false) == null,
		"no window size row": view.find_child("Choice_display_resolution", true, false) == null,
		"no vsync row": view.find_child("Check_display_vsync", true, false) == null,
		"no frame cap row": view.find_child("Choice_display_max_fps", true, false) == null,
		"other rows stay": view.find_child("Slider_audio_master", true, false) != null and view.find_child("Check_anim_instant", true, false) != null,
		"rows line up": SettingsUiTestSuite._rows_ok(view),
	}
	if box != null:
		checks["starts unchecked in a window"] = not box.button_pressed
		box.button_pressed = true
		box.pressed.emit()
		checks["press -> fullscreen call"] = b.mode == DisplayServer.WINDOW_MODE_FULLSCREEN
		b.mode = DisplayServer.WINDOW_MODE_WINDOWED   # the browser's Esc leaves fullscreen
		view.get_tree().root.size_changed.emit()
		checks["follows the real state"] = not box.button_pressed
		checks["toggling writes no display key"] = not _file_has("display/window_mode") and not _file_has("display/max_fps")
	host.queue_free()
	DisplaySettings.web_override = 0
	return TestHelpers.report(TAG, "Web Settings screen: only a Fullscreen checkbox", checks)

# True when every visible control (outside scroll containers) is inside `area` and none has an empty rect.
static func _problems(view: Control, area: Rect2) -> Array[String]:
	var out: Array[String] = []
	for n: Node in view.find_children("*", "Control", true, false):
		var c: Control = n as Control
		if c == null or not c.is_visible_in_tree() or c is ColorRect or SettingsUiTestSuite._under_scroll(c, view):
			continue
		var r: Rect2 = c.get_global_rect()
		if r.size.x <= 0.0 or r.size.y <= 0.0:
			if c is Label or c is BaseButton:
				out.append("%s has an empty rect" % c.name)
			continue
		if not area.encloses(r):
			out.append("%s outside the viewport %s" % [c.name, r])
	return out

static func _test_layout(tree_parent: Node, logical: Vector2) -> bool:
	var b: DisplaySettings.Backend = _fresh()
	CurrentGame.end()
	var host: Control = _host(tree_parent, logical)
	var view: MainMenuView = _menu(host)
	if view == null:
		return TestHelpers.report(TAG, "Layout", {"scene loads": false})
	var area := Rect2(Vector2.ZERO, logical)
	var checks: Dictionary = {}
	SettingsUiTestSuite._open(view, MenuRouter.MAIN)   # warm-up
	for scale: float in TEXT_SCALES:
		SettingsFile.set_value("ui/text_scale", scale)
		view.apply_text_scale()
		var tag: String = "%dx%d text %.2f" % [int(logical.x), int(logical.y), scale]
		SettingsFile.set_value("display/window_mode", "windowed")
		SettingsUiTestSuite._open(view, MenuRouter.MAIN)
		checks[tag + " main menu fits"] = _problems(view, area).is_empty()
		SettingsUiTestSuite._open(view, MenuRouter.SETTINGS)
		var problems: Array[String] = _problems(view, area)
		if not problems.is_empty():
			print("  ", tag, " settings -> ", problems)
		checks[tag + " settings fits"] = problems.is_empty()
		checks[tag + " settings rows line up"] = SettingsUiTestSuite._rows_ok(view)
		checks[tag + " display rows present"] = view.find_child("Choice_display_window_mode", true, false) != null and view.find_child("Choice_display_resolution", true, false) != null and view.find_child("Check_display_vsync", true, false) != null and view.find_child("Choice_display_max_fps", true, false) != null
		var size_pick: OptionButton = view.find_child("Choice_display_resolution", true, false) as OptionButton
		checks[tag + " size row enabled when windowed"] = size_pick != null and not size_pick.disabled
		SettingsFile.set_value("display/window_mode", "fullscreen")
		SettingsUiTestSuite._open(view, MenuRouter.SETTINGS)
		size_pick = view.find_child("Choice_display_resolution", true, false) as OptionButton
		var hint: Label = view.find_child("SizeHint", true, false) as Label
		problems = _problems(view, area)
		if not problems.is_empty():
			print("  ", tag, " settings (fullscreen) -> ", problems)
		checks[tag + " fullscreen settings fits"] = problems.is_empty()
		checks[tag + " fullscreen rows line up"] = SettingsUiTestSuite._rows_ok(view)
		checks[tag + " size row disabled with a hint"] = size_pick != null and size_pick.disabled and hint != null and hint.text == "Used in windowed mode"
		# The countdown box.
		SettingsFile.set_value("display/window_mode", "windowed")
		var m := SettingsModel.new()
		var _w: Array[String] = m.set_value("display/window_mode", "fullscreen")
		var _a: Array[String] = m.apply()
		view.begin_display_confirm(m)
		SettingsUiTestSuite.layout(view)
		problems = _problems(view, area)
		if not problems.is_empty():
			print("  ", tag, " countdown -> ", problems)
		checks[tag + " countdown box fits"] = problems.is_empty() and view.is_overlay_open() and m.confirming()
		checks[tag + " Keep and Revert reachable"] = SettingsUiTestSuite._button(view, "Keep") != null and SettingsUiTestSuite._button(view, "Revert") != null
		# Esc reverts (once per size, at Normal text).
		if is_equal_approx(scale, 1.0):
			view._process(3.0)
			checks[tag + " box counts down"] = is_equal_approx(m.confirm_left, 7.0) and view.is_confirming()
			view.cancel_pressed()
			checks[tag + " Esc reverts and closes"] = not m.confirming() and not view.is_overlay_open() and not view.is_confirming()
		else:
			m.revert_display()
			view._end_confirm(false)
		SettingsFile.set_value("display/window_mode", "windowed")
	b.calls.clear()
	host.queue_free()
	return TestHelpers.report(TAG, "Settings, countdown box and main menu fit %dx%d at every text size" % [int(logical.x), int(logical.y)], checks)
