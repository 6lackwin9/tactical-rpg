class_name SettingsView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Settings view (UI1-L): one slider (volumes), level picker (animation speed, text size) or checkbox per SettingsFile key on the model's draft; Apply writes
# the file (and rescales the menu text), Revert drops the draft, Back asks when the draft changed.
# ==============================================================================

const MAX_WIDTH: int = 840   # three 260 px buttons and their gaps fit
const LABEL_W: int = 240
const CONTROL_W: int = 300
const VALUE_W: int = 90

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
	add_child(grid)
	for key: String in m.keys():
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
			grid.add_child(_level_picker(m, key))
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
	var buttons := HBoxContainer.new()
	buttons.alignment = BoxContainer.ALIGNMENT_CENTER
	buttons.add_theme_constant_override("separation", 14)
	add_child(buttons)
	buttons.add_child(MainMenuView.button("Apply", func() -> void:
		host.act(m.apply)
		host.apply_text_scale()))
	buttons.add_child(MainMenuView.button("Revert", func() -> void:
		m.revert()
		host.render()))
	buttons.add_child(MainMenuView.button("Back", host.router.pop))

# A keyboard-friendly dropdown of the key's fixed levels (the draft always holds a level).
func _level_picker(m: SettingsModel, key: String) -> OptionButton:
	var pick := OptionButton.new()
	pick.name = "Levels_" + key.replace("/", "_")
	pick.focus_mode = Control.FOCUS_ALL
	pick.custom_minimum_size = Vector2(CONTROL_W, 0)
	pick.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	var lv: Array[float] = SettingsModel.levels(key)
	var current: float = SettingsFile.snap(key, DictUtil.to_float(m.value(key)))
	for i: int in lv.size():
		pick.add_item(SettingsModel.level_text(key, i), i)
		if is_equal_approx(lv[i], current):
			pick.select(i)
	pick.item_selected.connect(func(i: int) -> void:
		var _e: Array[String] = m.set_value(key, lv[i]))
	return pick

static func _gap(width: int) -> Control:
	var c := Control.new()
	c.custom_minimum_size = Vector2(width, 0)
	return c
