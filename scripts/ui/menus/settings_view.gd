class_name SettingsView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Settings view (UI1-L): one slider (volumes), level picker (animation speed, text size) or checkbox per SettingsFile key on the model's draft; Apply writes
# the file (and rescales the menu text), Restore defaults sets the draft to the defaults, Back asks when the draft changed.
# MENU-POLISH: animation speed and text size are notched sliders (text size previews live, with a sample), the Skip animations box sits on the speed row
# (the speed slider greys out while it is ticked), and Apply / Restore defaults / Back are a bar below the scrolling rows.
# ==============================================================================

const MAX_WIDTH: int = 840   # three 260 px buttons and their gaps fit
const LABEL_W: int = 240
const CONTROL_W: int = 300
const VALUE_W: int = 230   # fits "Extra large (1.30x)" and "Skip animations" at the largest text

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: SettingsModel = model as SettingsModel
	add_theme_constant_override("separation", 14)
	size_flags_horizontal = Control.SIZE_SHRINK_CENTER   # a centred column (SETTINGS-UI)
	custom_minimum_size = Vector2(MAX_WIDTH, 0)
	var head: Label = MainMenuView.title("Settings")
	head.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	add_child(head)
	var grid := GridContainer.new()
	grid.name = "SettingsGrid"
	grid.columns = 3   # label | control | value
	grid.add_theme_constant_override("h_separation", 20)
	grid.add_theme_constant_override("v_separation", 10)
	var scroll := ScrollContainer.new()   # the rows scroll when the text is large and the window short; Apply / Revert / Back stay in view
	scroll.name = "SettingsScroll"
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_child(scroll)
	scroll.add_child(grid)
	grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var refs: Dictionary = {}   # the window size row's controls, which follow the window mode picker
	var web: bool = DisplaySettings.is_web()
	for key: String in m.keys():
		if web and SettingsModel.DISPLAY_KEYS.has(key):
			continue   # the browser owns the canvas size and the frame rate
		if key == "anim/instant":
			continue   # lives on the animation speed row
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
				var _e: Array[String] = m.set_value(key, on))
			grid.add_child(box)
			grid.add_child(_gap(CONTROL_W))
			grid.add_child(_gap(VALUE_W))
			continue
		var label: Label = MainMenuView.text(str(SettingsModel.LABELS.get(key, key)), UiTheme.COL_TEXT, false)
		label.custom_minimum_size = Vector2(LABEL_W, 0)
		label.size_flags_vertical = Control.SIZE_FILL
		label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		grid.add_child(label)
		if not SettingsModel.levels(key).is_empty():
			_level_row(grid, host, m, key)
			continue
		if not SettingsModel.choices(key).is_empty():
			var pick: OptionButton = _choice_picker(m, key, refs)
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
			continue
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
			shown.text = "%d%%" % roundi(v * 100.0))
		grid.add_child(slider)
		grid.add_child(shown)
	if not web:
		grid.add_child(_now_label("In use"))
		var now: Label = MainMenuView.text(DisplaySettings.now_text(), UiTheme.COL_MUTED, false)   # read back from the window, so Apply visibly worked
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
	if web:
		grid.add_child(_fullscreen_row(grid))
		grid.add_child(_gap(CONTROL_W))
		grid.add_child(_gap(VALUE_W))
	# The text size shown is the draft's (an unsaved preview); every other render shows the saved one.
	host.apply_text_scale(SettingsFile.snap("ui/text_scale", DictUtil.to_float(m.value("ui/text_scale"))))
	var bar := PanelContainer.new()   # fixed below the scrolling rows
	bar.name = "SettingsBar"
	bar.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG, UiTheme.COL_BORDER, 6, 14, 10, 2))
	add_child(bar)
	var buttons := HBoxContainer.new()
	buttons.alignment = BoxContainer.ALIGNMENT_CENTER
	buttons.add_theme_constant_override("separation", 14)
	bar.add_child(buttons)
	buttons.add_child(MainMenuView.small(MainMenuView.button("Apply", func() -> void:
		host.act(m.apply)
		host.apply_text_scale()
		if m.confirming():
			host.begin_display_confirm(m)), 180))
	buttons.add_child(MainMenuView.small(MainMenuView.button("Restore defaults", func() -> void:
		m.restore_defaults()
		host.render()), 180))
	buttons.add_child(MainMenuView.small(MainMenuView.button("Back", host.router.pop), 180))

# A notched slider over the key's fixed levels (value = level index, one notch per level, the draft always holds a level).
# Row cells: label | slider with the level name under it | the speed row's Skip box, or the text size row's live sample.
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
		var sample: Label = MainMenuView.text("Sample text", UiTheme.COL_TEXT, false)
		sample.name = "TextSample"
		sample.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		sample.add_theme_font_size_override("font_size", roundi(MainMenuView.BASE_FONT * lv[index]))
		extra = sample
		slider.value_changed.connect(func(v: float) -> void:
			var text_scale: float = lv[clampi(roundi(v), 0, lv.size() - 1)]
			sample.add_theme_font_size_override("font_size", roundi(MainMenuView.BASE_FONT * text_scale))
			host.apply_text_scale(text_scale))   # the menu rescales at once; Apply saves it, Back / a re-render restores the saved size
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
func _choice_picker(m: SettingsModel, key: String, refs: Dictionary) -> OptionButton:
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
			_sync_size_row(m, refs))
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
