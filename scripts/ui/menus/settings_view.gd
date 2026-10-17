class_name SettingsView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Settings view (UI1-L): one slider or checkbox per SettingsFile key on the model's draft; Apply writes
# the file (and rescales the menu text), Revert drops the draft, Back asks when the draft changed.
# ==============================================================================

func build(host: MainMenuView, model: RefCounted) -> void:
	var m: SettingsModel = model as SettingsModel
	add_theme_constant_override("separation", 10)
	add_child(MainMenuView.title("Settings"))
	var grid := GridContainer.new()
	grid.columns = 3
	grid.add_theme_constant_override("h_separation", 16)
	add_child(grid)
	for key: String in m.keys():
		var spec: Dictionary = DictUtil.dict_of(SettingsFile.KEYS[key])
		grid.add_child(MainMenuView.text(str(SettingsModel.LABELS.get(key, key)), UiTheme.COL_TEXT, false))
		if DictUtil.to_int(spec["type"]) == TYPE_BOOL:
			var box := CheckBox.new()
			box.button_pressed = DictUtil.to_bool(m.value(key))
			box.toggled.connect(func(on: bool) -> void:
				var _e: Array[String] = m.set_value(key, on))
			grid.add_child(box)
			grid.add_child(Control.new())
			continue
		var slider := HSlider.new()
		slider.min_value = DictUtil.to_float(spec["min"])
		slider.max_value = DictUtil.to_float(spec["max"])
		slider.step = 0.05
		slider.value = DictUtil.to_float(m.value(key))
		slider.custom_minimum_size = Vector2(260, 0)
		slider.focus_mode = Control.FOCUS_ALL
		var shown: Label = MainMenuView.text(_fmt(key, slider.value), UiTheme.COL_MUTED, false)
		slider.value_changed.connect(func(v: float) -> void:
			var _e: Array[String] = m.set_value(key, v)
			shown.text = _fmt(key, v))
		grid.add_child(slider)
		grid.add_child(shown)
	var buttons := HBoxContainer.new()
	add_child(buttons)
	buttons.add_child(MainMenuView.button("Apply", func() -> void:
		host.act(m.apply)
		host.apply_text_scale()))
	buttons.add_child(MainMenuView.button("Revert", func() -> void:
		m.revert()
		host.render()))
	buttons.add_child(MainMenuView.button("Back", host.router.pop))

static func _fmt(key: String, v: float) -> String:
	return "%.2fx" % v if key.begins_with("anim/") or key.begins_with("ui/") else "%d%%" % roundi(v * 100.0)
