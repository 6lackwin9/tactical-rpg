class_name SettingsView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Settings view (UI1-L): one slider (volumes), level picker (animation speed, text size) or checkbox per SettingsFile key on the model's draft; Apply writes
# the file (and rescales the menu text), Restore defaults sets the draft to the defaults, the close cross (also Esc, right click) asks when the draft changed.
# MENU-POLISH: animation speed and text size are notched sliders (text size previews live, with a sample), the Skip animations box sits on the speed row
# (the speed slider greys out while it is ticked).
# SETTINGS-POPUP: the host shows this view inside a centred popup panel over a dimmed backdrop: header (title left, close cross right), the scrolling
# rows, then small Apply / Restore defaults buttons at the bottom right (no Back button).
# ==============================================================================

const LABEL_W: int = 240
const CONTROL_W: int = 300
const VALUE_W: int = 230   # fits "Extra large (1.30x)" and "Skip animations" at the largest text

const TAB_SOUND: String = "Sound"
const TAB_DISPLAY: String = "Display"
const TAB_GAMEPLAY: String = "Gameplay"
const TAB_CONTROLS: String = "Controls"   # CONTROLS-HELP: read-only key list
const CONTROLS_ACTION_W: int = 340
const SOUND_KEYS: Array[String] = ["audio/master", "audio/music", "audio/sfx", "audio/ui"]
const MUTE_KEY: String = "audio/mute"   # a checkbox row on top of the Sound tab, above Master
const DISPLAY_ORDER: Array[String] = ["display/window_mode", "display/resolution", "display/vsync", "display/max_fps", "ui/text_scale"]

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: SettingsModel = model as SettingsModel
	add_theme_constant_override("separation", 12)
	var header := HBoxContainer.new()
	header.name = "SettingsHeader"
	add_child(header)
	var head: Label = MainMenuView.title("Settings")
	head.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	header.add_child(head)
	var close := Button.new()   # same path as Esc and right click: asks first when the draft changed
	close.name = "SettingsClose"
	close.text = "✕"
	close.tooltip_text = "Close (Esc)"
	close.focus_mode = Control.FOCUS_ALL
	close.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	close.pressed.connect(host.cancel_pressed)
	header.add_child(close)
	var tabs := TabContainer.new()   # the panel keeps one size whichever tab shows; each tab scrolls on its own
	tabs.name = "SettingsTabs"
	tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	tabs.focus_mode = Control.FOCUS_ALL
	add_child(tabs)
	var refs: Dictionary = {}   # the window size row's controls, which follow the window mode picker
	var web: bool = DisplaySettings.is_web()
	var sound: Array[String] = []
	var display: Array[String] = []
	var gameplay: Array[String] = []
	for key: String in m.keys():
		if key == "anim/instant" or (web and SettingsModel.DISPLAY_KEYS.has(key)):
			continue   # the Skip box lives on the animation speed row; the browser owns the canvas size and the frame rate
		if key == MUTE_KEY:
			sound.push_front(key)
		elif SOUND_KEYS.has(key):
			sound.append(key)
		elif DISPLAY_ORDER.has(key):
			display.append(key)
		else:
			gameplay.append(key)
	display.sort_custom(func(x: String, y: String) -> bool: return DISPLAY_ORDER.find(x) < DISPLAY_ORDER.find(y))
	var sound_grid: GridContainer = _tab(tabs, TAB_SOUND)
	for key: String in sound:
		_add_row(sound_grid, host, m, key, refs)
	var display_grid: GridContainer = _tab(tabs, TAB_DISPLAY)
	if web:
		display_grid.add_child(_fullscreen_row(display_grid))
		display_grid.add_child(_gap(CONTROL_W))
		display_grid.add_child(_gap(VALUE_W))
	for key: String in display:
		_add_row(display_grid, host, m, key, refs)
		if key == DisplaySettings.KEY_SIZE and not web:
			_in_use_row(display_grid)
	var play_grid: GridContainer = _tab(tabs, TAB_GAMEPLAY)
	for key: String in gameplay:
		_add_row(play_grid, host, m, key, refs)
	_controls_tab(tabs)
	tabs.current_tab = clampi(m.tab, 0, tabs.get_tab_count() - 1)
	tabs.tab_changed.connect(func(i: int) -> void:
		m.tab = i)
	# The text size shown is the draft's (an unsaved preview); every other render shows the saved one.
	host.apply_text_scale(SettingsFile.snap("ui/text_scale", DictUtil.to_float(m.value("ui/text_scale"))))
	var footer := HBoxContainer.new()   # outside the scroll, bottom right
	footer.name = "SettingsFooter"
	footer.alignment = BoxContainer.ALIGNMENT_END
	footer.add_theme_constant_override("separation", 10)
	add_child(footer)
	if host.in_game():   # CONTROLS-HELP: opened during play; the way back to the hub lives here (Esc now opens this popup)
		var hub: Button = MainMenuView.small(MainMenuView.button("Back to hub", host.leave_from_settings))
		hub.name = "SettingsLeaveGame"
		hub.tooltip_text = "Leave the game scene (refused while a fight, conversation or shop is open)"
		footer.add_child(hub)
		var spacer := Control.new()
		spacer.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		footer.add_child(spacer)
	var restore: Button = MainMenuView.small(MainMenuView.button("Restore defaults", func() -> void:
		m.restore_defaults()
		host.render()))
	restore.flat = true   # low emphasis: text only, no frame until hovered
	restore.tooltip_text = "Resets every tab (Sound, Display, Gameplay) to the defaults"
	restore.add_theme_color_override("font_color", UiTheme.COL_MUTED)
	footer.add_child(restore)
	footer.add_child(MainMenuView.small(MainMenuView.button("Apply", func() -> void:
		host.act(m.apply)
		host.apply_text_scale()
		if m.confirming():
			host.begin_display_confirm(m))))
	for key: String in SOUND_KEYS:   # a re-render (Restore defaults) keeps the buses on the draft levels
		host.sync_volume(key, DictUtil.to_float(m.value(key)))
	if DictUtil.to_bool(m.value(MUTE_KEY)) != SettingsFile.get_bool(MUTE_KEY) or AudioDirector.mute_now != SettingsFile.get_bool(MUTE_KEY):
		host.preview_mute(DictUtil.to_bool(m.value(MUTE_KEY)))   # a re-render keeps the draft Mute on the bus
	var cur: Control = tabs.get_current_tab_control()
	var first: Array[Node] = cur.find_children("*", "HSlider", true, false) if cur != null else []
	if not first.is_empty():
		set_meta(MainMenuView.FOCUS_META, first[0])   # focus a setting, not the close cross

# CONTROLS-HELP: the Controls tab, grouped by context; the rows come from ControlsList (InputMap actions + the hard-coded keys).
func _controls_tab(tabs: TabContainer) -> void:
	var grid: GridContainer = _tab(tabs, TAB_CONTROLS)
	for group: String in ControlsList.GROUPS:
		var head: Label = MainMenuView.text(group, UiTheme.COL_TITLE, false)
		head.name = "ControlsGroup_" + group
		head.add_theme_font_size_override("font_size", 24)
		grid.add_child(head)
		grid.add_child(_gap(0))
		grid.add_child(_gap(0))
		for row: Dictionary in ControlsList.rows(group):
			var id: String = str(row["id"])
			var action: Label = MainMenuView.text(str(row["action"]), UiTheme.COL_TEXT, false)
			action.name = "ControlRow_" + id
			action.custom_minimum_size = Vector2(CONTROLS_ACTION_W, 0)
			grid.add_child(action)
			var keys: Label = MainMenuView.text(str(row["keys"]), UiTheme.COL_MUTED, false)
			keys.name = "ControlKeys_" + id
			keys.custom_minimum_size = Vector2(CONTROL_W, 0)
			grid.add_child(keys)
			grid.add_child(_gap(0))

# One tab page: a scroll area (named like the tab) holding a label | control | value grid.
func _tab(tabs: TabContainer, title: String) -> GridContainer:
	var scroll := ScrollContainer.new()
	scroll.name = title
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	tabs.add_child(scroll)
	var grid := GridContainer.new()
	grid.name = "SettingsGrid_" + title
	grid.columns = 3   # label | control | value
	grid.add_theme_constant_override("h_separation", 20)
	grid.add_theme_constant_override("v_separation", 10)
	grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(grid)
	return grid

func _in_use_row(grid: GridContainer) -> void:
	grid.add_child(_now_label("In use"))
	var now: Label = MainMenuView.text(DisplaySettings.now_text(), UiTheme.COL_MUTED, false)   # read back from the window, so a change visibly worked
	now.name = "DisplayNow"
	now.size_flags_vertical = Control.SIZE_FILL
	now.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	grid.add_child(now)
	grid.add_child(_gap(VALUE_W))
	var root: Window = grid.get_tree().root if grid.is_inside_tree() else null
	if root != null:
		var refresh: Callable = func() -> void:
			if is_instance_valid(now):
				now.text = DisplaySettings.now_text()
		root.size_changed.connect(refresh)
		now.tree_exiting.connect(func() -> void:
			if root.size_changed.is_connected(refresh):
				root.size_changed.disconnect(refresh))

func _add_row(grid: GridContainer, host: MainMenuView, m: SettingsModel, key: String, refs: Dictionary) -> void:
	var spec: Dictionary = DictUtil.dict_of(SettingsFile.KEYS[key])
	if DictUtil.to_int(spec["type"]) == TYPE_BOOL:
		var box := CheckBox.new()
		box.name = "Check_" + key.replace("/", "_")
		box.text = str(SettingsModel.LABELS.get(key, key))
		box.button_pressed = DictUtil.to_bool(m.value(key))
		box.focus_mode = Control.FOCUS_ALL
		box.custom_minimum_size = Vector2(LABEL_W, 0)
		box.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		box.toggled.connect(func(on: bool) -> void:
			var _e: Array[String] = m.set_value(key, on)
			if key == MUTE_KEY:
				host.preview_mute(on))   # Master follows the draft at once; closing without Apply restores the saved state
		grid.add_child(box)
		grid.add_child(_gap(CONTROL_W))
		grid.add_child(_gap(VALUE_W))
		return
	var label: Label = MainMenuView.text(str(SettingsModel.LABELS.get(key, key)), UiTheme.COL_TEXT, false)
	label.custom_minimum_size = Vector2(LABEL_W, 0)
	label.size_flags_vertical = Control.SIZE_FILL
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	grid.add_child(label)
	if not SettingsModel.levels(key).is_empty():
		_level_row(grid, host, m, key)
		return
	if not SettingsModel.choices(key).is_empty():
		var pick: OptionButton = _choice_picker(host, m, key, refs)
		grid.add_child(pick)
		if key == DisplaySettings.KEY_SIZE:
			var hint: Label = MainMenuView.text("", UiTheme.COL_MUTED, false)
			hint.name = "SizeHint"
			hint.size_flags_vertical = Control.SIZE_FILL
			hint.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
			refs["hint"] = hint
			refs["size_pick"] = pick
			grid.add_child(hint)
			_sync_size_row(m, refs)
		else:
			grid.add_child(_gap(VALUE_W))
		return
	var slider := HSlider.new()
	slider.name = "Slider_" + key.replace("/", "_")
	slider.min_value = DictUtil.to_float(spec["min"])
	slider.max_value = DictUtil.to_float(spec["max"])
	slider.step = 0.05
	slider.value = DictUtil.to_float(m.value(key))
	slider.custom_minimum_size = Vector2(CONTROL_W, 0)
	slider.size_flags_vertical = Control.SIZE_FILL   # fills the row, the grabber centres in it
	slider.focus_mode = Control.FOCUS_ALL
	var shown: Label = MainMenuView.text("%d%%" % roundi(slider.value * 100.0), UiTheme.COL_MUTED, false)
	shown.custom_minimum_size = Vector2(VALUE_W, 0)
	shown.size_flags_vertical = Control.SIZE_FILL
	shown.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	slider.value_changed.connect(func(v: float) -> void:
		var _e: Array[String] = m.set_value(key, v)
		shown.text = "%d%%" % roundi(v * 100.0)
		if SOUND_KEYS.has(key):
			host.preview_volume(key, v))   # the bus takes the draft level and a short sample plays (throttled)
	grid.add_child(slider)
	grid.add_child(shown)

# A notched slider over the key's fixed levels (value = level index, one notch per level, the draft always holds a level).
# Row cells: label | slider with the level name under it | the speed row's Skip box, (the text size row has none: the whole menu is its preview).
func _level_row(grid: GridContainer, host: MainMenuView, m: SettingsModel, key: String) -> void:
	var lv: Array[float] = SettingsModel.levels(key)
	var current: float = SettingsFile.snap(key, DictUtil.to_float(m.value(key)))
	var index: int = 0
	for i: int in lv.size():
		if is_equal_approx(lv[i], current):
			index = i
	var tag: String = key.replace("/", "_")
	var cell := VBoxContainer.new()
	cell.name = "LevelCell_" + tag
	cell.custom_minimum_size = Vector2(CONTROL_W, 0)
	cell.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	var slider := HSlider.new()
	slider.name = "Levels_" + tag
	slider.min_value = 0.0
	slider.max_value = float(lv.size() - 1)
	slider.step = 1.0
	slider.tick_count = lv.size()
	slider.ticks_on_borders = true
	slider.scrollable = false   # the wheel scrolls the rows, not the notch
	slider.focus_mode = Control.FOCUS_ALL
	slider.value = float(index)
	slider.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cell.add_child(slider)
	var shown: Label = MainMenuView.text(SettingsModel.level_text(key, index), UiTheme.COL_MUTED, false)
	shown.name = "LevelName_" + tag
	cell.add_child(shown)
	grid.add_child(cell)
	var extra: Control = null
	if key == "anim/speed":
		extra = _skip_box(m, cell)
	elif key == "ui/text_scale":
		extra = _gap(VALUE_W)
		slider.value_changed.connect(func(v: float) -> void:
			host.apply_text_scale(lv[clampi(roundi(v), 0, lv.size() - 1)]))   # the whole menu rescales at once; Apply saves it, closing without Apply / a re-render restores the saved size
	else:
		extra = _gap(VALUE_W)
	grid.add_child(extra)
	slider.value_changed.connect(func(v: float) -> void:
		var i: int = clampi(roundi(v), 0, lv.size() - 1)
		var _e: Array[String] = m.set_value(key, lv[i])
		shown.text = SettingsModel.level_text(key, i))

# "Skip animations" (FxSettings.instant: every effect jumps to its end state) on the speed row; the speed slider is greyed out and locked while it is ticked.
func _skip_box(m: SettingsModel, speed_cell: Control) -> CheckBox:
	var box := CheckBox.new()
	box.name = "Check_anim_instant"
	box.text = str(SettingsModel.LABELS.get("anim/instant", "Skip animations"))
	box.focus_mode = Control.FOCUS_ALL
	box.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	box.size_flags_horizontal = Control.SIZE_SHRINK_BEGIN
	var sync: Callable = func(on: bool) -> void:
		var slider: HSlider = speed_cell.get_child(0) as HSlider
		if slider != null:
			slider.editable = not on
		speed_cell.modulate = Color(1, 1, 1, 0.45 if on else 1.0)
	box.toggled.connect(func(on: bool) -> void:
		var _e: Array[String] = m.set_value("anim/instant", on)
		sync.call(on))
	box.button_pressed = DictUtil.to_bool(m.value("anim/instant"))
	sync.call(box.button_pressed)
	return box

# A dropdown of a key's listed values (window mode, window size, frame-rate cap).
func _choice_picker(host: MainMenuView, m: SettingsModel, key: String, refs: Dictionary) -> OptionButton:
	var pick := OptionButton.new()
	pick.name = "Choice_" + key.replace("/", "_")
	pick.focus_mode = Control.FOCUS_ALL
	pick.custom_minimum_size = Vector2(CONTROL_W, 0)
	pick.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	var list: Array = SettingsModel.choices(key)
	var current: Variant = m.value(key)
	if key == DisplaySettings.KEY_SIZE:
		current = DisplaySettings.fit_size(str(current))   # a size that does not fit the screen shows the one in use
	for i: int in list.size():
		pick.add_item(SettingsModel.choice_text(key, list[i]), i)
		if list[i] == current:
			pick.select(i)
	pick.item_selected.connect(func(i: int) -> void:
		var _e: Array[String] = m.set_value(key, list[i])
		if key == DisplaySettings.KEY_MODE:
			_sync_size_row(m, refs)
		if SettingsModel.CONFIRM_KEYS.has(key) and m.start_display_change():
			host.begin_display_confirm(m))   # a window mode / size applies on select and waits for Keep (10 s, then it reverts)
	return pick

# Fullscreen / borderless leave the window size unused: the row is disabled and says so (the saved size is kept).
static func _sync_size_row(m: SettingsModel, refs: Dictionary) -> void:
	var pick_raw: Variant = refs.get("size_pick")
	var hint_raw: Variant = refs.get("hint")
	if not (pick_raw is OptionButton and hint_raw is Label):
		return
	var pick: OptionButton = pick_raw
	var hint: Label = hint_raw
	var windowed: bool = str(m.value(DisplaySettings.KEY_MODE)) == DisplaySettings.MODE_WINDOWED
	pick.disabled = not windowed
	hint.text = "" if windowed else SettingsModel.SIZE_HINT

# Web: one Fullscreen checkbox. The mode changes inside the pressed handler (browsers allow it only during a click) and the
# box follows the root's size_changed, because the browser's Esc leaves fullscreen without telling the game.
func _fullscreen_row(grid: Control) -> CheckBox:
	var box := CheckBox.new()
	box.name = "Check_display_fullscreen"
	box.text = "Fullscreen"
	box.focus_mode = Control.FOCUS_ALL
	box.custom_minimum_size = Vector2(LABEL_W, 0)
	box.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	box.button_pressed = _is_fullscreen()
	box.pressed.connect(func() -> void:
		var target: int = DisplayServer.WINDOW_MODE_FULLSCREEN if box.button_pressed else DisplayServer.WINDOW_MODE_WINDOWED
		DisplaySettings.backend().set_mode(target))
	var sync: Callable = func() -> void:
		box.set_pressed_no_signal(_is_fullscreen())
	var root: Window = grid.get_tree().root if grid.is_inside_tree() else null
	if root != null:
		root.size_changed.connect(sync)
		box.tree_exiting.connect(func() -> void:
			if root.size_changed.is_connected(sync):
				root.size_changed.disconnect(sync))
	return box

static func _is_fullscreen() -> bool:
	return DisplaySettings.backend().current_mode() != DisplayServer.WINDOW_MODE_WINDOWED

static func _now_label(text: String) -> Label:
	var l: Label = MainMenuView.text(text, UiTheme.COL_TEXT, false)
	l.custom_minimum_size = Vector2(LABEL_W, 0)
	l.size_flags_vertical = Control.SIZE_FILL
	l.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	return l

static func _gap(width: int) -> Control:
	var c := Control.new()
	c.custom_minimum_size = Vector2(width, 0)
	return c
