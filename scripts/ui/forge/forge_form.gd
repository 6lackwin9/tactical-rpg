class_name ForgeForm
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Forge Form (schema-driven editor for one catalog entry)
# Builds one row per visible ContentSchema field of `kind` and edits a draft
# Dictionary in place. Fields whose value controls another field's visibility
# ("when") rebuild the form when they change. Emits `edited` after every change.
#
# Typed controls (FORGE1 appendix G): enum dropdowns never offer an unknown stored value as a normal choice, spin boxes take
# dynamic limits, checklists replace comma text boxes (`multi_pick`), the description is a template editor with a token palette
# and a live per-rank preview (`template`), and locked key fields of system assets show why they are locked.
# A hook set decides the rest: `state_fn(key) -> Dictionary` (SkillFormModel.state), `layout_fn() -> [{id, title, fields}]`
# (sections instead of one flat grid), `footer_fn() -> Control` (e.g. the ranks panel) and `problems_fn() -> [{field, rank,
# message}]` (red outline + tooltip). Without hooks the form is the flat grid every other kind uses.
# ==============================================================================

signal edited(key: String)

const LABEL_W: int = 150

var source: ForgeSource = ContentForgeSource.new()   # where fields and reference choices come from
var kind: String = ""
var draft: Dictionary = {}
var json_errors: Dictionary = {}   # key -> parse error of a json field (draft keeps the last good value)
var state_fn: Callable = Callable()
var layout_fn: Callable = Callable()
var footer_fn: Callable = Callable()
var problems_fn: Callable = Callable()
var _controllers: Dictionary = {}  # keys other fields depend on
var _grid: GridContainer = null
var _widgets: Dictionary = {}      # key -> control (hooked forms refresh these in place)
var _live: Array[Callable] = []    # refreshers of live widgets (the description preview)
var _signature: String = ""        # visible field keys at the last rebuild (hooked forms)

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

func _hooked() -> bool:
	return state_fn.is_valid() and layout_fn.is_valid()

func rebuild() -> void:
	UiTheme.clear_children(self)
	_widgets.clear()
	_live.clear()
	_grid = null
	if _hooked():
		var blocks: Array = layout_fn.call()
		var keys: Array[String] = []
		for raw: Variant in blocks:
			var block: Dictionary = DictUtil.dict_of(raw)
			add_child(UiTheme.header(str(block.get("title", ""))))
			var grid: GridContainer = _new_grid()
			add_child(grid)
			if _grid == null:
				_grid = grid
			for key: Variant in block.get("fields", []):
				var f: Dictionary = source.field(kind, str(key))
				if not f.is_empty():
					_add_row(f, grid)
					keys.append(str(key))
		_signature = ",".join(keys)
		if footer_fn.is_valid():
			var footer: Variant = footer_fn.call()
			if footer is Control:
				add_child(footer as Control)
		refresh_states()
		return
	_grid = _new_grid()
	add_child(_grid)
	for f: Dictionary in source.fields(kind):
		if not ContentSchema.is_visible(f, draft):
			continue
		if str(f["key"]) == "system" and not DictUtil.to_bool(draft.get("system", false)):
			continue
		_add_row(f, _grid)

func _new_grid() -> GridContainer:
	var grid := GridContainer.new()
	grid.columns = 2
	grid.add_theme_constant_override("h_separation", 10)
	grid.add_theme_constant_override("v_separation", 6)
	grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	return grid

func _add_row(f: Dictionary, grid: GridContainer) -> void:
	var key: String = str(f["key"])
	var help: String = str(f.get("help", ""))
	var info: Dictionary = _state(key)
	var label_text: String = str(f["label"])
	if DictUtil.to_bool(info.get("locked", false)):
		label_text = "🔒 " + label_text
	var lbl: Label = UiTheme.label(label_text, UiTheme.FONT_MD, UiTheme.COL_MUTED)
	lbl.custom_minimum_size.x = LABEL_W
	lbl.tooltip_text = help
	lbl.mouse_filter = Control.MOUSE_FILTER_STOP
	lbl.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	grid.add_child(lbl)
	var widget: Control = _widget(f)
	widget.name = "Field_" + key
	widget.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if not help.is_empty() and widget.tooltip_text.is_empty():
		widget.tooltip_text = help
	grid.add_child(widget)
	_widgets[key] = widget

# The editor control for field `key` (tests drive the form through these).
func field_control(key: String) -> Control:
	if _widgets.has(key):
		var control: Control = _widgets[key]
		return control
	if _grid == null:
		return null
	return _grid.get_node_or_null("Field_" + key) as Control

func _put(key: String, value: Variant) -> void:
	draft[key] = value
	edited.emit(key)
	if _hooked():
		_after_change(key)
		return
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

# Replaces the whole draft (a kind switch, a rank edit) and rebuilds.
func replace_draft(entry: Dictionary) -> void:
	draft = source.normalize(kind, entry)
	edited.emit("")
	rebuild()

# Hooked forms: a changed visible-field list rebuilds, otherwise the controls refresh in place (limits, enabled, options).
func _after_change(key: String) -> void:
	var type: String = str(source.field(kind, key).get("type", ""))
	var blocks: Array = layout_fn.call()
	var keys: Array[String] = []
	for raw: Variant in blocks:
		for k: Variant in DictUtil.dict_of(raw).get("fields", []):
			keys.append(str(k))
	if ",".join(keys) != _signature:
		rebuild.call_deferred()
		return
	if not ["string", "id", "template", "text"].has(type) or key == "name":
		refresh_states()
	for live: Callable in _live:
		live.call()
	_mark_problems()

func _state(key: String) -> Dictionary:
	if state_fn.is_valid():
		var info: Variant = state_fn.call(key)
		if info is Dictionary:
			var d: Dictionary = info
			return d
	return {}

# Pushes the current limits / enabled flags / allowed options into the existing controls.
func refresh_states() -> void:
	for key: Variant in _widgets:
		var control: Control = _widgets[key]
		var f: Dictionary = source.field(kind, str(key))
		_apply_state(control, f, _state(str(key)))
	for live: Callable in _live:
		live.call()
	_mark_problems()

func _apply_state(control: Control, f: Dictionary, info: Dictionary) -> void:
	if info.is_empty() or not is_instance_valid(control):
		return
	var enabled: bool = DictUtil.to_bool(info.get("enabled", true))
	var reason: String = str(info.get("reason", ""))
	if control is SpinBox:
		var spin: SpinBox = control
		var lo: float = DictUtil.to_float(info["min"]) if info.get("min", null) != null else DictUtil.to_float(f.get("min", -99999))
		var hi: float = DictUtil.to_float(info["max"]) if info.get("max", null) != null else DictUtil.to_float(f.get("max", 99999))
		spin.min_value = lo
		spin.max_value = maxf(lo, hi)
		if spin.value < spin.min_value or spin.value > spin.max_value:
			spin.value = clampf(spin.value, spin.min_value, spin.max_value)   # value_changed writes the clamped value into the draft
		spin.editable = enabled
	elif control is OptionButton:
		var opt: OptionButton = control
		opt.disabled = not enabled
		var off: Dictionary = DictUtil.dict_of(info.get("disabled_options", {}))
		for i: int in range(opt.item_count):
			var value: String = str(opt.get_item_metadata(i))
			var blocked: bool = off.has(value)
			opt.set_item_disabled(i, blocked)
			opt.set_item_tooltip(i, str(off.get(value, "")) if blocked else "")
	elif control is CheckBox:
		(control as CheckBox).disabled = not enabled
	elif control is LineEdit:
		(control as LineEdit).editable = enabled
	if not reason.is_empty():
		control.tooltip_text = reason
	elif not str(f.get("help", "")).is_empty():
		control.tooltip_text = str(f.get("help", ""))
	_set_checklist_state(control, info, enabled)

func _set_checklist_state(control: Control, info: Dictionary, enabled: bool) -> void:
	for child: Node in control.get_children():
		if child is CheckBox:
			var box: CheckBox = child
			var off: Dictionary = DictUtil.dict_of(info.get("disabled_options", {}))
			box.disabled = not enabled or off.has(str(box.get_meta("value", "")))

func _mark_problems() -> void:
	if not problems_fn.is_valid():
		return
	var list: Array = problems_fn.call()
	var by_field: Dictionary = {}
	for raw: Variant in list:
		var p: Dictionary = DictUtil.dict_of(raw)
		var key: String = str(p.get("field", ""))
		var texts: Array = by_field.get(key, [])
		texts.append(str(p.get("message", "")))
		by_field[key] = texts
	for key: Variant in _widgets:
		var control: Control = _widgets[key]
		if not is_instance_valid(control):
			continue
		if by_field.has(key):
			var texts: Array = by_field[key]
			control.modulate = Color(1.0, 0.6, 0.6)
			control.tooltip_text = "\n".join(PackedStringArray(texts))
		else:
			control.modulate = Color.WHITE

# ------------------------------------------------------------------------------
# Widgets
# ------------------------------------------------------------------------------

func _widget(f: Dictionary) -> Control:
	var key: String = str(f["key"])
	var value: Variant = draft.get(key, f["default"])
	var info: Dictionary = _state(key)
	var readonly: bool = DictUtil.to_bool(f.get("readonly", false)) or not DictUtil.to_bool(info.get("enabled", true))
	match str(f["type"]):
		"id", "string":
			var line := LineEdit.new()
			line.text = str(value)
			line.editable = not readonly
			if f.has("max_len"):
				line.max_length = 0   # the validator reports an over-long value; do not truncate silently
			line.text_changed.connect(func(t: String) -> void: _put(key, t.strip_edges()))
			return line
		"text":
			var text := TextEdit.new()
			text.text = str(value)
			text.custom_minimum_size.y = 54
			text.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
			text.text_changed.connect(func() -> void: _put(key, text.text))
			return text
		"template":
			return _template_widget(f, str(value), readonly)
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
			var options: Array[String] = source.options_of(f)
			var allowed: Array = DictUtil.array_of(info.get("options", []))
			if not allowed.is_empty():
				options = []
				for a: Variant in allowed:
					options.append(str(a))
			return _enum_widget(key, options, str(value), readonly, {}, state_fn.is_valid())
		"ref":
			var ids: Array[String] = [""]
			for choice: Dictionary in source.ref_choices(f):
				ids.append(str(choice["value"]))
			return _enum_widget(key, ids, str(value), readonly, f)
		"refs":
			return _refs_widget(f, value, info, readonly)
		"stats":
			return _stats_widget(f, value)
		"multi_pick":
			return _checklist_widget(f, value, info, readonly)
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

# An enum / ref dropdown. With `strict` an unknown stored value is shown as a red "(invalid: x)" item, never as a normal choice.
func _enum_widget(key: String, options: Array[String], value: String, readonly: bool, ref_field: Dictionary = {}, strict: bool = false) -> OptionButton:
	var opt := OptionButton.new()
	opt.fit_to_longest_item = false
	opt.clip_text = true
	opt.custom_minimum_size.x = 60
	var list: Array[String] = options.duplicate()
	var invalid: bool = not list.has(value)
	if invalid:
		list.append(value)   # keep an unknown value visible (validation reports it)
	for i: int in range(list.size()):
		var shown: String = source.ref_label(ref_field, list[i]) if not ref_field.is_empty() else (list[i] if not list[i].is_empty() else "(none)")
		if strict and invalid and i == list.size() - 1:
			shown = "(invalid: %s)" % list[i]
		opt.add_item(shown, i)
		opt.set_item_metadata(i, list[i])
		if strict and invalid and i == list.size() - 1:
			opt.set_item_icon(i, null)
		if list[i] == value:
			opt.select(i)
	opt.disabled = readonly
	opt.item_selected.connect(func(i: int) -> void: _put(key, str(opt.get_item_metadata(i))))
	if strict and invalid:
		opt.modulate = Color(1.0, 0.5, 0.5)
	return opt

# A checklist (multi_pick): one checkbox per option, no free text. "all" in a cleanse list is exclusive.
func _checklist_widget(f: Dictionary, value: Variant, info: Dictionary, readonly: bool) -> Control:
	var key: String = str(f["key"])
	var options: Array[String] = source.options_of(f)
	var allowed: Array = DictUtil.array_of(info.get("options", []))
	if not allowed.is_empty():
		options = []
		for a: Variant in allowed:
			options.append(str(a))
	var current: Array[String] = ContentFactory._strings(value)
	var box := HFlowContainer.new()
	box.add_theme_constant_override("h_separation", 10)
	box.add_theme_constant_override("v_separation", 2)
	var exclusive: String = SkillDef.CLEANSE_ALL if key == "cleanse_statuses" else ""
	for option: String in options:
		var check := CheckBox.new()
		check.text = option
		check.set_meta("value", option)
		check.button_pressed = current.has(option)
		check.disabled = readonly
		check.toggled.connect(func(on: bool) -> void:
			var next: Array[String] = ContentFactory._strings(draft.get(key, []))
			if on and not next.has(option):
				next.append(option)
			elif not on:
				next.erase(option)
			if on and not exclusive.is_empty():
				if option == exclusive:
					next = [exclusive]
				else:
					next.erase(exclusive)
			var ordered: Array[String] = []
			for candidate: String in options:
				if next.has(candidate):
					ordered.append(candidate)
			_put(key, ordered)
			if not exclusive.is_empty():
				for child: Node in box.get_children():
					if child is CheckBox:
						var other: CheckBox = child
						other.set_pressed_no_signal(ordered.has(str(other.get_meta("value", ""))))
		)
		box.add_child(check)
	return box

# The description template editor: text box with counter, a palette of valid tokens, problems, and one preview line per rank.
func _template_widget(f: Dictionary, value: String, readonly: bool) -> Control:
	var key: String = str(f["key"])
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 4)
	var editor := TextEdit.new()
	editor.name = "Editor"
	editor.text = value
	editor.editable = not readonly
	editor.custom_minimum_size.y = 64
	editor.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	editor.text_changed.connect(func() -> void: _put(key, editor.text))
	box.add_child(editor)
	var counter: Label = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	counter.name = "Counter"
	box.add_child(counter)
	var palette := HFlowContainer.new()
	palette.name = "Palette"
	box.add_child(palette)
	if state_fn.is_valid():
		for token: String in SkillFormModel.token_palette(draft):
			var btn: Button = UiTheme.button("{%s}" % token, "Insert {%s} at the caret" % token, func() -> void:
				editor.insert_text_at_caret("{%s}" % token)
				editor.grab_focus()
			, UiTheme.FONT_SM)
			btn.disabled = readonly
			palette.add_child(btn)
	var issues: RichTextLabel = RichTextLabel.new()
	issues.name = "Issues"
	issues.bbcode_enabled = true
	issues.fit_content = true
	issues.scroll_active = false
	box.add_child(issues)
	var previews := VBoxContainer.new()
	previews.name = "Previews"
	box.add_child(previews)
	var refresh: Callable = func() -> void:
		if not is_instance_valid(box):
			return
		var template: String = str(draft.get(key, ""))
		counter.text = "%d / %d characters" % [template.length(), SkillText.MAX_LENGTH]
		var lines: Array[String] = []
		for p: Dictionary in SkillText.validate(template, draft):
			lines.append("[color=#f87171]%s[/color]" % str(p["message"]))
		for w: String in SkillText.warnings(template, draft):
			lines.append("[color=#fbbf24]%s[/color]" % w)
		issues.text = "\n".join(lines)
		issues.visible = not lines.is_empty()
		UiTheme.clear_children(previews)
		for line: Dictionary in SkillFormModel.description_preview(draft):
			var over: bool = DictUtil.to_bool(line["over"])
			var row: Label = UiTheme.label("%s: %s  (%d)" % [str(line["name"]), str(line["text"]), DictUtil.to_int(line["length"])], UiTheme.FONT_SM, UiTheme.COL_BAD if over else UiTheme.COL_MUTED)
			row.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
			row.custom_minimum_size.x = 60
			previews.add_child(row)
	_live.append(refresh)
	refresh.call()
	return box

func _refs_widget(f: Dictionary, value: Variant, info: Dictionary = {}, readonly: bool = false) -> Control:
	var key: String = str(f["key"])
	var current: Array[String] = []
	if value is Array:
		var arr: Array = value
		for v: Variant in arr:
			current.append(str(v))
	var choices: Variant = info.get("choices", null)
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
		up.disabled = index == 0 or readonly
		row.add_child(up)
		var remove_btn: Button = UiTheme.button("✕", "Remove", func() -> void:
			var trimmed: Array[String] = current.duplicate()
			trimmed.remove_at(index)
			_put(key, trimmed)
			rebuild()
		)
		remove_btn.disabled = readonly
		row.add_child(remove_btn)
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
		if choices is Array and not (choices as Array).has(id):
			continue
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
	add_btn.disabled = picker.item_count == 0 or readonly
	picker.disabled = readonly
	add_row.add_child(add_btn)
	box.add_child(add_row)
	if current.is_empty():
		var hint: Label = UiTheme.label(str(info.get("reason", "")) if not str(info.get("reason", "")).is_empty() else str(f.get("help", "(empty)")), UiTheme.FONT_SM, UiTheme.COL_MUTED)
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
