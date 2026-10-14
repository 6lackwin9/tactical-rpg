class_name ForgeForm
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Forge Form (schema-driven editor for one catalog entry)
# Builds one row per visible ContentSchema field of `kind` and edits a draft
# Dictionary in place. Fields whose value controls another field's visibility
# ("when") rebuild the form when they change. Emits `edited` after every change.
# ==============================================================================

signal edited(key: String)

const LABEL_W: int = 150

var source: ForgeSource = ContentForgeSource.new()   # where fields and reference choices come from
var kind: String = ""
var draft: Dictionary = {}
var json_errors: Dictionary = {}   # key -> parse error of a json field (draft keeps the last good value)
var _controllers: Dictionary = {}  # keys other fields depend on
var _grid: GridContainer = null

func _init() -> void:
	name = "ForgeForm"
	add_theme_constant_override("separation", 6)

func edit(p_kind: String, entry: Dictionary) -> void:
	kind = p_kind
	draft = source.normalize(kind, entry)
	json_errors.clear()
	_controllers.clear()
	for f: Dictionary in source.fields(kind):
		var cond: Variant = f.get("when", null)
		if cond is Dictionary:
			var cond_dict: Dictionary = cond
			_controllers[str(cond_dict.get("key", ""))] = true
	rebuild()

func rebuild() -> void:
	UiTheme.clear_children(self)
	_grid = GridContainer.new()
	_grid.columns = 2
	_grid.add_theme_constant_override("h_separation", 10)
	_grid.add_theme_constant_override("v_separation", 6)
	_grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_child(_grid)
	for f: Dictionary in source.fields(kind):
		if not ContentSchema.is_visible(f, draft):
			continue
		if str(f["key"]) == "system" and not DictUtil.to_bool(draft.get("system", false)):
			continue
		_add_row(f)

func _add_row(f: Dictionary) -> void:
	var key: String = str(f["key"])
	var help: String = str(f.get("help", ""))
	var lbl: Label = UiTheme.label(str(f["label"]), UiTheme.FONT_MD, UiTheme.COL_MUTED)
	lbl.custom_minimum_size.x = LABEL_W
	lbl.tooltip_text = help
	lbl.mouse_filter = Control.MOUSE_FILTER_STOP
	lbl.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	_grid.add_child(lbl)
	var widget: Control = _widget(f)
	widget.name = "Field_" + key
	widget.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if not help.is_empty() and widget.tooltip_text.is_empty():
		widget.tooltip_text = help
	_grid.add_child(widget)

# The editor control for field `key` (tests drive the form through these).
func field_control(key: String) -> Control:
	if _grid == null:
		return null
	return _grid.get_node_or_null("Field_" + key) as Control

func _put(key: String, value: Variant) -> void:
	draft[key] = value
	edited.emit(key)
	if _controllers.has(key):
		rebuild.call_deferred()

# Sets a field programmatically (coerced like form input) and refreshes the form.
func set_value(key: String, value: Variant) -> void:
	var f: Dictionary = source.field(kind, key)
	if f.is_empty():
		return
	draft[key] = ContentSchema.coerce(f, value)
	edited.emit(key)
	rebuild()

# ------------------------------------------------------------------------------
# Widgets
# ------------------------------------------------------------------------------

func _widget(f: Dictionary) -> Control:
	var key: String = str(f["key"])
	var value: Variant = draft.get(key, f["default"])
	var readonly: bool = DictUtil.to_bool(f.get("readonly", false))
	match str(f["type"]):
		"id", "string":
			var line := LineEdit.new()
			line.text = str(value)
			line.editable = not readonly
			line.text_changed.connect(func(t: String) -> void: _put(key, t.strip_edges()))
			return line
		"text":
			var text := TextEdit.new()
			text.text = str(value)
			text.custom_minimum_size.y = 54
			text.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
			text.text_changed.connect(func() -> void: _put(key, text.text))
			return text
		"int", "float":
			var spin := SpinBox.new()
			var is_float: bool = str(f["type"]) == "float"
			spin.min_value = DictUtil.to_float(f.get("min", -99999))
			spin.max_value = DictUtil.to_float(f.get("max", 99999))
			spin.step = DictUtil.to_float(f.get("step", 0.01 if is_float else 1.0))
			spin.value = DictUtil.to_float(value)
			spin.editable = not readonly
			spin.value_changed.connect(func(v: float) -> void: _put(key, v if is_float else roundi(v)))
			return spin
		"bool":
			var check := CheckBox.new()
			check.button_pressed = DictUtil.to_bool(value)
			check.disabled = readonly
			check.toggled.connect(func(on: bool) -> void: _put(key, on))
			return check
		"enum":
			return _enum_widget(key, source.options_of(f), str(value), readonly)
		"ref":
			var ids: Array[String] = [""]
			for choice: Dictionary in source.ref_choices(f):
				ids.append(str(choice["value"]))
			return _enum_widget(key, ids, str(value), readonly, f)
		"refs":
			return _refs_widget(f, value)
		"stats":
			return _stats_widget(f, value)
		"strings":
			var tags := LineEdit.new()
			var parts: Array[String] = []
			if value is Array:
				var arr: Array = value
				for v: Variant in arr:
					parts.append(str(v))
			tags.text = ", ".join(parts)
			tags.placeholder_text = "comma, separated, values"
			tags.text_changed.connect(func(t: String) -> void:
				var out: Array[String] = []
				for part: String in t.split(",", false):
					if not part.strip_edges().is_empty():
						out.append(part.strip_edges())
				_put(key, out)
			)
			return tags
		"json":
			var json_edit := TextEdit.new()
			json_edit.text = JSON.stringify(value)
			json_edit.custom_minimum_size.y = 54
			json_edit.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
			json_edit.text_changed.connect(func() -> void:
				var parsed: JSON = JSON.new()
				if parsed.parse(json_edit.text) == OK and (parsed.data is Array or parsed.data is Dictionary):
					json_errors.erase(key)
					json_edit.remove_theme_color_override("font_color")
					_put(key, ContentSchema.coerce(f, parsed.data))
				else:
					json_errors[key] = "%s: invalid JSON (%s)" % [str(f["label"]), parsed.get_error_message() if parsed.get_error_message() != "" else "expected a list or object"]
					json_edit.add_theme_color_override("font_color", UiTheme.COL_BAD)
					edited.emit(key)
			)
			return json_edit
	return UiTheme.label(str(value))

func _enum_widget(key: String, options: Array[String], value: String, readonly: bool, ref_field: Dictionary = {}) -> OptionButton:
	var opt := OptionButton.new()
	opt.fit_to_longest_item = false
	opt.clip_text = true
	opt.custom_minimum_size.x = 60
	var list: Array[String] = options.duplicate()
	if not list.has(value):
		list.append(value)   # keep an unknown value visible (validation reports it)
	for i: int in range(list.size()):
		var shown: String = source.ref_label(ref_field, list[i]) if not ref_field.is_empty() else (list[i] if not list[i].is_empty() else "(none)")
		opt.add_item(shown, i)
		opt.set_item_metadata(i, list[i])
		if list[i] == value:
			opt.select(i)
	opt.disabled = readonly
	opt.item_selected.connect(func(i: int) -> void: _put(key, str(opt.get_item_metadata(i))))
	return opt

func _refs_widget(f: Dictionary, value: Variant) -> Control:
	var key: String = str(f["key"])
	var current: Array[String] = []
	if value is Array:
		var arr: Array = value
		for v: Variant in arr:
			current.append(str(v))
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 2)
	for i: int in range(current.size()):
		var row := HBoxContainer.new()
		var tag: Label = UiTheme.label("• " + source.ref_label(f, current[i]), UiTheme.FONT_MD)
		tag.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		row.add_child(tag)
		var index: int = i
		var up: Button = UiTheme.button("▲", "Move up", func() -> void:
			if index > 0:
				var moved: Array[String] = current.duplicate()
				var item: String = moved[index]
				moved.remove_at(index)
				moved.insert(index - 1, item)
				_put(key, moved)
				rebuild()
		)
		up.disabled = index == 0
		row.add_child(up)
		row.add_child(UiTheme.button("✕", "Remove", func() -> void:
			var trimmed: Array[String] = current.duplicate()
			trimmed.remove_at(index)
			_put(key, trimmed)
			rebuild()
		))
		box.add_child(row)
	var add_row := HBoxContainer.new()
	var picker := OptionButton.new()
	picker.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	picker.fit_to_longest_item = false
	picker.clip_text = true
	picker.custom_minimum_size.x = 60
	var dupes: bool = DictUtil.to_bool(f.get("dupes", false))
	for choice: Dictionary in source.ref_choices(f):
		var id: String = str(choice["value"])
		if dupes or not current.has(id):
			picker.add_item(str(choice["label"]))
			picker.set_item_metadata(picker.item_count - 1, id)
	add_row.add_child(picker)
	var add_btn: Button = UiTheme.button("+ Add", "Add the selected entry", func() -> void:
		if picker.item_count == 0 or picker.selected < 0:
			return
		var grown: Array[String] = current.duplicate()
		grown.append(str(picker.get_item_metadata(picker.selected)))
		_put(key, grown)
		rebuild()
	)
	add_btn.disabled = picker.item_count == 0
	add_row.add_child(add_btn)
	box.add_child(add_row)
	if current.is_empty():
		var hint: Label = UiTheme.label(str(f.get("help", "(empty)")), UiTheme.FONT_SM, UiTheme.COL_MUTED)
		hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART   # a long hint must never widen the form
		hint.custom_minimum_size.x = 60
		box.add_child(hint)
	return box

func _stats_widget(f: Dictionary, value: Variant) -> Control:
	var key: String = str(f["key"])
	var stats: Dictionary = {}
	if value is Dictionary:
		var given: Dictionary = value
		stats = given.duplicate()
	var grid := GridContainer.new()
	grid.columns = 6
	grid.add_theme_constant_override("h_separation", 6)
	var keys: Array = f.get("keys", ContentSchema.STAT_KEYS)
	for k: Variant in keys:
		var stat: String = str(k)
		grid.add_child(UiTheme.label(stat.to_upper(), UiTheme.FONT_SM, UiTheme.COL_MUTED))
		var spin := SpinBox.new()
		spin.min_value = -99
		spin.max_value = 999
		spin.value = DictUtil.int_of(stats, stat)
		spin.custom_minimum_size.x = 64
		spin.tooltip_text = "0 = not set"
		spin.value_changed.connect(func(v: float) -> void:
			if roundi(v) == 0:
				stats.erase(stat)
			else:
				stats[stat] = roundi(v)
			_put(key, stats.duplicate())
		)
		grid.add_child(spin)
	return grid
