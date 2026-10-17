class_name ForgeForm
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Forge Form (schema-driven READ-ONLY view of one catalog entry; TOOLS-RO)
# Builds one row per visible ContentSchema field of `kind` from the entry it shows. Every control is disabled / not
# editable and the action buttons of the tables (add, remove, move, token palette) are not shown: the Forge tools are
# asset viewers, content is edited through ContentCatalog.upsert / WorldCatalog seed scripts.
#
# Typed controls (FORGE1 appendix G): enum dropdowns show an unknown stored value as "(invalid: x)", spin boxes carry the
# dynamic limits, checklists show `multi_pick` values, and the description (`template`) shows its token problems and a
# per-rank preview.
# A hook set decides the rest: `state_fn(key) -> Dictionary` (SkillFormModel.state), `layout_fn() -> [{id, title, fields}]`
# (sections instead of one flat grid), `footer_fn() -> Control` (e.g. the ranks panel) and `problems_fn() -> [{field, rank,
# message}]` (red outline + tooltip). Without hooks the form is the flat grid every other kind uses.
# Class tab (CLASS1): `table` is the skill path table (family picker, standard / custom level, ladder, default-loadout preview),
# `dict_pick` the proficiency rows (type, level, scaling, guarded Remove), class stats show a "Remaining" chip, gear refs with
# `choices` records are grouped by item family, and length-capped text fields show a live counter in their label.
# Item tabs (ITEM1 commit 3): `modifier_table`, `tier_table` and `hand_grid` are built by ItemTables (read-only; their add /
# move / remove buttons are hidden by _hide_actions); a layout block may name read-only panels ("pre" / "post" ids) that
# `panel_fn(id) -> Control` builds; a string field whose state lists `options` is a dropdown; checklist options show their
# `option_help` / disabled reason as tooltip (engine rules).
# ==============================================================================

signal edited(key: String)

const LABEL_W: int = 150

var source: ForgeSource = ContentForgeSource.new()   # where fields and reference choices come from
var kind: String = ""
var draft: Dictionary = {}
var state_fn: Callable = Callable()
var layout_fn: Callable = Callable()
var footer_fn: Callable = Callable()
var problems_fn: Callable = Callable()
var panel_fn: Callable = Callable()   # panel id -> Control (read-only panels of a layout block)
var _controllers: Dictionary = {}  # keys other fields depend on
var _grid: GridContainer = null
var _widgets: Dictionary = {}      # key -> control (hooked forms refresh these in place)
var _labels: Dictionary = {}       # key -> row label (live counters of length-capped fields)
var _live: Array[Callable] = []    # refreshers of live widgets (the description preview)
var _signature: String = ""        # visible field keys at the last rebuild (hooked forms)

func _init() -> void:
	name = "ForgeForm"
	add_theme_constant_override("separation", 6)

func edit(p_kind: String, entry: Dictionary) -> void:
	kind = p_kind
	draft = source.normalize(kind, entry)
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
	_labels.clear()
	_live.clear()
	_grid = null
	if _hooked():
		var blocks: Array = layout_fn.call()
		var keys: Array[String] = []
		for raw: Variant in blocks:
			var block: Dictionary = DictUtil.dict_of(raw)
			add_child(UiTheme.header(str(block.get("title", ""))))
			_add_panels(DictUtil.array_of(block.get("pre", [])))
			var grid: GridContainer = _new_grid()
			add_child(grid)
			if _grid == null:
				_grid = grid
			for key: Variant in block.get("fields", []):
				var f: Dictionary = source.field(kind, str(key))
				if not f.is_empty():
					_add_row(f, grid)
					keys.append(str(key))
			_add_panels(DictUtil.array_of(block.get("post", [])))
		_signature = ",".join(keys)
		if footer_fn.is_valid():
			var footer: Variant = footer_fn.call()
			if footer is Control:
				add_child(footer as Control)
		refresh_states()
		_hide_actions(self)
		return
	_grid = _new_grid()
	add_child(_grid)
	for f: Dictionary in source.fields(kind):
		if not ContentSchema.is_visible(f, draft):
			continue
		if str(f["key"]) == "system" and not DictUtil.to_bool(draft.get("system", false)):
			continue
		_add_row(f, _grid)

# The viewer shows values only: plain action buttons (add / remove / move / insert) and add rows of the ItemTables
# widgets are hidden (value controls such as CheckBox, OptionButton and MenuButton stay, disabled).
func _hide_actions(node: Node) -> void:
	for child: Node in node.get_children():
		if child.get_class() == "Button" or str(child.name) == "AddRow":
			(child as Control).visible = false
			continue
		_hide_actions(child)

# Read-only panels of a layout block (built by panel_fn; the panel registers its own live refresher with add_live).
func _add_panels(ids: Array) -> void:
	if not panel_fn.is_valid():
		return
	for id: Variant in ids:
		var panel: Variant = panel_fn.call(str(id))
		if panel is Control:
			var control: Control = panel
			control.name = "Panel_" + str(id)
			add_child(control)

# A panel of the current form ("" panels are not built; tests read them).
func panel_control(id: String) -> Control:
	return get_node_or_null("Panel_" + id) as Control

# Sets `key` in the shown draft and refreshes states / live widgets / problem marks (ItemTables widget callbacks; the
# controls are disabled in the viewer, so nothing calls it from the UI).
func put(key: String, value: Variant) -> void:
	_put(key, value)

# Registers a refresher called after every edit (live labels of table rows and panels).
func add_live(refresh: Callable) -> void:
	_live.append(refresh)

# The current problems of the hook ({field, index | rank, message} rows; empty without a problems hook).
func current_problems() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if problems_fn.is_valid():
		for raw: Variant in DictUtil.array_of(problems_fn.call()):
			out.append(DictUtil.dict_of(raw))
	return out

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
	var lbl: Label = UiTheme.label(label_text, UiTheme.FONT_MD, UiTheme.COL_MUTED)
	lbl.custom_minimum_size.x = LABEL_W
	lbl.tooltip_text = help
	lbl.mouse_filter = Control.MOUSE_FILTER_STOP
	lbl.size_flags_vertical = Control.SIZE_SHRINK_BEGIN
	grid.add_child(lbl)
	_labels[key] = lbl
	if f.has("max_len") and _hooked() and ["id", "string", "text"].has(str(f["type"])):
		var base_text: String = label_text
		var cap: int = DictUtil.to_int(f["max_len"])
		_live.append(func() -> void:
			if is_instance_valid(lbl):
				var n: int = str(draft.get(key, "")).length()
				lbl.text = "%s (%d/%d)" % [base_text, n, cap]
				lbl.add_theme_color_override("font_color", UiTheme.COL_BAD if n > cap else UiTheme.COL_MUTED)
		)
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
	var enabled: bool = false   # read-only viewer: the state only brings limits, options and reasons
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
			var value: String = str(box.get_meta("value", ""))
			box.disabled = not enabled or off.has(value)
			var help: String = str(DictUtil.dict_of(info.get("option_help", {})).get(value, ""))
			box.tooltip_text = str(off[value]) if off.has(value) else help

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
	var readonly: bool = true   # read-only viewer (TOOLS-RO)
	match str(f["type"]):
		"id", "string":
			var choices: Array = DictUtil.array_of(info.get("options", []))
			if str(f["type"]) == "string" and not choices.is_empty():
				var listed: Array[String] = []
				for c: Variant in choices:
					listed.append(str(c))
				return _enum_widget(key, listed, str(value), readonly, {}, true)
			var line := LineEdit.new()
			line.text = str(value)
			line.editable = false
			return line
		"text":
			var text := TextEdit.new()
			text.text = str(value)
			text.custom_minimum_size.y = 54
			text.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
			text.editable = false
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
			spin.editable = false
			return spin
		"bool":
			var check := CheckBox.new()
			check.button_pressed = DictUtil.to_bool(value)
			check.disabled = true
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
			if info.get("choices", null) is Array:
				return _grouped_ref_widget(key, DictUtil.array_of(info["choices"]), str(value), readonly)
			var ids: Array[String] = [""]
			for choice: Dictionary in source.ref_choices(f):
				ids.append(str(choice["value"]))
			return _enum_widget(key, ids, str(value), readonly, f)
		"refs":
			return _refs_widget(f, value, info, readonly)
		"stats":
			return _stats_widget(f, value, readonly)
		"table":
			return _path_widget(f, readonly)
		"dict_pick":
			return _proficiency_widget(f, readonly)
		"multi_pick":
			return _checklist_widget(f, value, info, readonly)
		"modifier_table":
			return ItemTables.modifier_table(self, key, info, readonly)
		"tier_table":
			return ItemTables.tier_table(self, key, info, readonly)
		"hand_grid":
			return ItemTables.hand_grid(self, key, str(f.get("grid", "base")), readonly)
		"strings":
			var tags := LineEdit.new()
			var parts: Array[String] = []
			if value is Array:
				var arr: Array = value
				for v: Variant in arr:
					parts.append(str(v))
			tags.text = ", ".join(parts)
			tags.editable = false
			return tags
		"json":
			var json_edit := TextEdit.new()
			json_edit.text = JSON.stringify(value)
			json_edit.custom_minimum_size.y = 54
			json_edit.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
			json_edit.editable = false
			return json_edit
	return UiTheme.label(str(value))

# An enum / ref dropdown (disabled). With `strict` an unknown stored value is shown as a red "(invalid: x)" item.
func _enum_widget(key: String, options: Array[String], value: String, readonly: bool, ref_field: Dictionary = {}, strict: bool = false) -> OptionButton:
	var opt := OptionButton.new()
	opt.fit_to_longest_item = false
	opt.clip_text = true
	opt.custom_minimum_size.x = 60
	var list: Array[String] = options.duplicate()
	var invalid: bool = not list.has(value)
	if invalid:
		list.append(value)   # keep an unknown value visible (validation reports it)
	var empty_label: String = str(_state(key).get("empty_label", "(none)")) if state_fn.is_valid() else "(none)"
	for i: int in range(list.size()):
		var shown: String = source.ref_label(ref_field, list[i]) if not ref_field.is_empty() else (list[i] if not list[i].is_empty() else empty_label)
		if strict and invalid and i == list.size() - 1:
			shown = "(invalid: %s)" % list[i]
		opt.add_item(shown, i)
		opt.set_item_metadata(i, list[i])
		if list[i] == value:
			opt.select(i)
	opt.disabled = readonly
	if strict and invalid:
		opt.modulate = Color(1.0, 0.5, 0.5)
	return opt

# A checklist (multi_pick): one disabled checkbox per option, the stored values checked.
func _checklist_widget(f: Dictionary, value: Variant, info: Dictionary, _readonly: bool) -> Control:
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
	for option: String in options:
		var check := CheckBox.new()
		check.text = option
		check.name = "Option_" + option
		check.set_meta("value", option)
		check.tooltip_text = str(DictUtil.dict_of(info.get("option_help", {})).get(option, ""))
		check.button_pressed = current.has(option)
		check.disabled = true
		box.add_child(check)
	return box

# The description template: the text (read-only) with its length, token problems and warnings, and one preview line per rank.
func _template_widget(f: Dictionary, value: String, _readonly: bool) -> Control:
	var key: String = str(f["key"])
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 4)
	var editor := TextEdit.new()
	editor.name = "Editor"
	editor.text = value
	editor.editable = false
	editor.custom_minimum_size.y = 64
	editor.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	box.add_child(editor)
	var counter: Label = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	counter.name = "Counter"
	box.add_child(counter)
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

# A reference list: one line per referenced entry (its label), or the field help when empty.
func _refs_widget(f: Dictionary, value: Variant, info: Dictionary = {}, _readonly: bool = true) -> Control:
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 2)
	var current: Array[String] = ContentFactory._strings(value)
	for id: String in current:
		box.add_child(UiTheme.label("• " + source.ref_label(f, id), UiTheme.FONT_MD))
	if current.is_empty():
		var hint: Label = UiTheme.label(str(info.get("reason", "")) if not str(info.get("reason", "")).is_empty() else str(f.get("help", "(empty)")), UiTheme.FONT_SM, UiTheme.COL_MUTED)
		hint.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART   # a long hint must never widen the form
		hint.custom_minimum_size.x = 60
		box.add_child(hint)
	return box

# Stat spinners (not editable). With schema min / max (class base stats and growth) the spinners use those ranges, an unset
# base stat shows its floor, and classes show a "Remaining: N" chip (budget minus total, ClassForgeModel.remaining).
func _stats_widget(f: Dictionary, value: Variant, _readonly: bool = true) -> Control:
	var key: String = str(f["key"])
	var stats: Dictionary = DictUtil.dict_of(value)
	var ranged: bool = f.has("min") and f.has("max")
	var grid := GridContainer.new()
	grid.columns = 6
	grid.add_theme_constant_override("h_separation", 6)
	var keys: Array = f.get("keys", ContentSchema.STAT_KEYS)
	for k: Variant in keys:
		var stat: String = str(k)
		grid.add_child(UiTheme.label(stat.to_upper(), UiTheme.FONT_SM, UiTheme.COL_MUTED))
		var spin := SpinBox.new()
		spin.name = "Stat_" + stat
		spin.min_value = DictUtil.to_float(f["min"]) if ranged else -99.0
		spin.max_value = DictUtil.to_float(f["max"]) if ranged else 999.0
		spin.value = DictUtil.int_of(stats, stat, DictUtil.to_int(f["min"]) if ranged and key == "base_stats" else 0)
		spin.custom_minimum_size.x = 64
		spin.tooltip_text = "%d-%d" % [DictUtil.to_int(f["min"]), DictUtil.to_int(f["max"])] if ranged else "0 = not set"
		spin.editable = false
		grid.add_child(spin)
	if not (ranged and kind == "classes"):
		return grid
	var box := VBoxContainer.new()
	box.add_child(grid)
	var left: int = ClassForgeModel.remaining(draft, key)
	var chip: Label = UiTheme.label("Remaining: %d" % left, UiTheme.FONT_SM, UiTheme.COL_GOOD if left == 0 else UiTheme.COL_BAD)
	chip.name = "Remaining"
	box.add_child(chip)
	return box

# A gear slot dropdown (disabled) from ClassForgeModel.slot_choices records: one separator per item family; "" (empty slot)
# first; an unknown stored value stays visible as "(invalid: x)".
func _grouped_ref_widget(_key: String, choices: Array, value: String, readonly: bool) -> OptionButton:
	var opt := OptionButton.new()
	opt.fit_to_longest_item = false
	opt.clip_text = true
	opt.custom_minimum_size.x = 60
	opt.add_item("(empty)")
	opt.set_item_metadata(0, "")
	var found: bool = value.is_empty()
	var last_group: String = "\u0000"
	for raw: Variant in choices:
		var c: Dictionary = DictUtil.dict_of(raw)
		var group: String = str(c.get("group", ""))
		if group != last_group and not group.is_empty():
			opt.add_separator(group)
		last_group = group
		opt.add_item(str(c.get("label", c.get("id", ""))))
		var i: int = opt.item_count - 1
		opt.set_item_metadata(i, str(c.get("id", "")))
		if str(c.get("id", "")) == value:
			opt.select(i)
			found = true
	if not found:
		opt.add_item("(invalid: %s)" % value)
		opt.set_item_metadata(opt.item_count - 1, value)
		opt.select(opt.item_count - 1)
		opt.modulate = Color(1.0, 0.5, 0.5)
	elif value.is_empty():
		opt.select(0)
	opt.disabled = readonly
	return opt

# The class skill path table (FORGE1_CLASSES 3.5): one row per entry in ENTRY order (family, level, tier ladder, a
# "custom" tag for a non-standard level, "★ active" for the default loadout), the counter and the default-loadout preview.
func _path_widget(_f: Dictionary, _readonly: bool) -> Control:
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 3)
	var counter: Label = UiTheme.label(ClassForgeModel.path_counter(draft), UiTheme.FONT_SM, UiTheme.COL_MUTED)
	counter.name = "Counter"
	box.add_child(counter)
	var rows_box := VBoxContainer.new()
	rows_box.name = "Rows"
	box.add_child(rows_box)
	for row: Dictionary in ClassForgeModel.path_rows(draft):
		rows_box.add_child(_path_row(row))
	var preview: Label = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	preview.name = "DefaultLoadout"
	preview.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	preview.custom_minimum_size.x = 60
	var families: Array[String] = ClassForgeModel.default_loadout(draft, StatRules.MAX_LEVEL)
	var names: Array[String] = []
	for fam: String in families:
		names.append(SkillCatalog.family_name(fam))
	preview.text = "Default active class skills at level %d (units without a loadout): %s" % [StatRules.MAX_LEVEL, ", ".join(names) if not names.is_empty() else "none"]
	box.add_child(preview)
	return box

static func _family_label(choice: Dictionary) -> String:
	var grade: String = str(choice.get("grade", "novice"))
	var requires: Array = DictUtil.array_of(choice.get("requires", []))
	return "%s  [%s%s]%s" % [str(choice["name"]), str(choice.get("kind", "")).to_lower(), "" if grade == "novice" else ", " + grade, ("  needs " + ", ".join(PackedStringArray(requires))) if not requires.is_empty() else ""]

func _path_row(row: Dictionary) -> Control:
	var fam: String = str(row["family"])
	var level: int = DictUtil.to_int(row["level"])
	var line := HBoxContainer.new()
	line.name = "Row_" + fam
	line.add_theme_constant_override("separation", 8)
	var label: String = fam
	var known: bool = false
	for choice: Dictionary in ClassForgeModel.family_choices(draft, fam):
		if str(choice["id"]) == fam:
			label = _family_label(choice)
			known = true
	var fam_label: Label = UiTheme.label(label if known else "(invalid: %s)" % fam, UiTheme.FONT_MD, UiTheme.COL_TEXT if known else UiTheme.COL_BAD)
	fam_label.name = "Family"
	fam_label.custom_minimum_size.x = 150
	fam_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	line.add_child(fam_label)
	var lvl: Label = UiTheme.label("Lv %d" % level, UiTheme.FONT_MD)
	lvl.name = "Level"
	line.add_child(lvl)
	var tag: String = "  custom" if DictUtil.to_bool(row["custom"]) else ""
	var ladder: Label = UiTheme.label("%s%s%s" % [str(row["ladder"]), tag, "  ★ active" if DictUtil.to_bool(row["active_by_default"]) else ""], UiTheme.FONT_SM, UiTheme.COL_MUTED if DictUtil.to_bool(row["requires_ok"]) else UiTheme.COL_BAD)
	ladder.name = "Ladder"
	if not DictUtil.to_bool(row["requires_ok"]):
		ladder.tooltip_text = "Needs %s on the path at level %d or lower." % [", ".join(PackedStringArray(DictUtil.array_of(row["requires"]))), level]
	line.add_child(ladder)
	return line

# The proficiency rows (dict_pick, FORGE1_CLASSES 2.3): gear type, level and scaling (spinners, not editable).
func _proficiency_widget(f: Dictionary, _readonly: bool) -> Control:
	var key: String = str(f["key"])
	var ranges: Dictionary = ClassForgeModel.ranges()
	var lvl_r: Dictionary = ranges["proficiency_level"]
	var sc_r: Dictionary = ranges["proficiency_scaling"]
	var box := VBoxContainer.new()
	box.add_theme_constant_override("separation", 2)
	var prof: Dictionary = DictUtil.dict_of(draft.get(key, {}))
	for gear_key: Variant in prof:
		var gear: String = str(gear_key)
		var entry: Dictionary = DictUtil.dict_of(prof[gear_key])
		var line := HBoxContainer.new()
		line.name = "Prof_" + gear
		var lbl: Label = UiTheme.label(gear, UiTheme.FONT_MD)
		lbl.custom_minimum_size.x = 70
		line.add_child(lbl)
		var lvl := SpinBox.new()
		lvl.name = "Level"
		lvl.min_value = DictUtil.to_float(lvl_r["min"])
		lvl.max_value = DictUtil.to_float(lvl_r["max"])
		lvl.value = DictUtil.to_int(entry.get("level", 1), 1)
		lvl.prefix = "Lv"
		lvl.editable = false
		line.add_child(lvl)
		var sc := SpinBox.new()
		sc.name = "Scaling"
		sc.min_value = DictUtil.to_float(sc_r["min"])
		sc.max_value = DictUtil.to_float(sc_r["max"])
		sc.step = DictUtil.to_float(sc_r["step"])
		sc.value = DictUtil.to_float(entry.get("scaling", sc_r["default"]))
		sc.prefix = "x"
		sc.editable = false
		line.add_child(sc)
		box.add_child(line)
	return box
