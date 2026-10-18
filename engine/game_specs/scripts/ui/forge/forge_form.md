---
class: ForgeForm
extends: VBoxContainer
description: "Tactical RPG - Forge Form (schema-driven READ-ONLY view of one catalog entry; TOOLS-RO) Builds one \"label: value\" row per visible ContentSchema field of `kind` from the entry it shows. FORGE-VIEW: the form is a display, not a disabled editor: every value widget (LineEdit, TextEdit, SpinBox, OptionButton, CheckBox, ItemTables cells) is replaced by a plain Label showing its value (display_of), and the action buttons of the tables (add, remove, move, token palette) are not shown: the Forge tools are asset viewers, content is edited through ContentCatalog.upsert / WorldCatalog seed scripts. Typed controls (FORGE1 appendix G): enum dropdowns show an unknown stored value as \"(invalid: x)\", spin boxes carry the dynamic limits, checklists show `multi_pick` values, and the description (`template`) shows its token problems and a per-rank preview. A hook set decides the rest: `state_fn(key) -> Dictionary` (SkillFormModel.state), `layout_fn() -> [{id, title, fields}]` (sections instead of one flat grid), `footer_fn() -> Control` (e.g. the ranks panel) and `problems_fn() -> [{field, rank, message}]` (red outline + tooltip). Without hooks the form is the flat grid every other kind uses. Class tab (CLASS1): `table` is the skill path table (family picker, standard / custom level, ladder, default-loadout preview), `dict_pick` the proficiency rows (type, level, scaling, guarded Remove), class stats show a \"Remaining\" chip, gear refs with `choices` records are grouped by item family, and length-capped text fields show a live counter in their label. Item tabs (ITEM1 commit 3): `modifier_table`, `tier_table` and `hand_grid` are built by ItemTables (read-only; their add / move / remove buttons are hidden by _hide_actions); a layout block may name read-only panels (\"pre\" / \"post\" ids) that `panel_fn(id) -> Control` builds; a string field whose state lists `options` is a dropdown; checklist options show their `option_help` / disabled reason as tooltip (engine rules)."
---

# Properties
* `LABEL_W`: int = 130 (const)
* `source`: ForgeSource = ContentForgeSource.new()   # where fields and reference choices come from
* `kind`: String = ""
* `draft`: Dictionary = {}
* `state_fn`: Callable = Callable()
* `layout_fn`: Callable = Callable()
* `footer_fn`: Callable = Callable()
* `problems_fn`: Callable = Callable()
* `panel_fn`: Callable = Callable()   # panel id -> Control (read-only panels of a layout block)
* `_controllers`: Dictionary = {}  # keys other fields depend on
* `_grid`: GridContainer = null
* `_widgets`: Dictionary = {}      # key -> control (hooked forms refresh these in place)
* `_labels`: Dictionary = {}       # key -> row label (live counters of length-capped fields)
* `_live`: Array[Callable] = []    # refreshers of live widgets (the description preview)
* `_signature`: String = ""        # visible field keys at the last rebuild (hooked forms)
* `out`: Array[Dictionary] = []

# Signals
* `edited(key: String)`

# Methods

### `_init() -> void`
```logic
name = "ForgeForm"
add_theme_constant_override("separation", 6)
```

### `edit(p_kind: String, entry: Dictionary) -> void`
```logic
kind = p_kind
draft = source.normalize(kind, entry)
_controllers.clear()
for f: Dictionary in source.fields(kind):
	var cond: Variant = f.get("when", null)
	if cond is Dictionary:
		var cond_dict: Dictionary = cond
		_controllers[str(cond_dict.get("key", ""))] = true
rebuild()
```

### `_hooked() -> bool`
```logic
return state_fn.is_valid() and layout_fn.is_valid()
```

### `rebuild() -> void`
```logic
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
```

### `_hide_actions(node: Node) -> void`
```logic
for child: Node in node.get_children():
	if child.get_class() == "Button" or str(child.name) == "AddRow":
		(child as Control).visible = false
		continue
	_hide_actions(child)
```

### `_add_panels(ids: Array) -> void`
```logic
if not panel_fn.is_valid():
	return
for id: Variant in ids:
	var panel: Variant = panel_fn.call(str(id))
	if panel is Control:
		var control: Control = panel
		control.name = "Panel_" + str(id)
		add_child(control)
```

### `panel_control(id: String) -> Control`
```logic
return get_node_or_null("Panel_" + id) as Control
```

### `put(key: String, value: Variant) -> void`
```logic
_put(key, value)
```

### `add_live(refresh: Callable) -> void`
```logic
_live.append(refresh)
```

### `_new_grid() -> GridContainer`
```logic
var grid := GridContainer.new()
grid.columns = 2
grid.add_theme_constant_override("h_separation", 10)
grid.add_theme_constant_override("v_separation", 6)
grid.size_flags_horizontal = Control.SIZE_EXPAND_FILL
return grid
```

### `_add_row(f: Dictionary, grid: GridContainer) -> void`
```logic
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
var widget: Control = display_of(_widget(f))
widget.name = "Field_" + key
widget.size_flags_horizontal = Control.SIZE_EXPAND_FILL
if not help.is_empty() and widget.tooltip_text.is_empty():
	widget.tooltip_text = help
grid.add_child(widget)
_widgets[key] = widget
```

### `field_control(key: String) -> Control`
```logic
if _widgets.has(key):
	var control: Control = _widgets[key]
	return control
if _grid == null:
	return null
return _grid.get_node_or_null("Field_" + key) as Control
```

### `_put(key: String, value: Variant) -> void`
```logic
draft[key] = value
edited.emit(key)
if _hooked():
	_after_change(key)
	return
if _controllers.has(key):
	rebuild.call_deferred()
```

### `_after_change(key: String) -> void`
```logic
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
```

### `_state(key: String) -> Dictionary`
```logic
if state_fn.is_valid():
	var info: Variant = state_fn.call(key)
	if info is Dictionary:
		var d: Dictionary = info
		return d
return {}
```

### `refresh_states() -> void`
```logic
for key: Variant in _widgets:
	var control: Control = _widgets[key]
	var f: Dictionary = source.field(kind, str(key))
	_apply_state(control, f, _state(str(key)))
for live: Callable in _live:
	live.call()
_mark_problems()
```

### `_apply_state(control: Control, f: Dictionary, info: Dictionary) -> void`
```logic
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
```

### `_set_checklist_state(control: Control, info: Dictionary, enabled: bool) -> void`
```logic
for child: Node in control.get_children():
	if child is CheckBox:
		var box: CheckBox = child
		var off: Dictionary = DictUtil.dict_of(info.get("disabled_options", {}))
		var value: String = str(box.get_meta("value", ""))
		box.disabled = not enabled or off.has(value)
		var help: String = str(DictUtil.dict_of(info.get("option_help", {})).get(value, ""))
		box.tooltip_text = str(off[value]) if off.has(value) else help
```

### `_mark_problems() -> void`
```logic
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
```

### `_widget(f: Dictionary) -> Control`
```logic
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
```

### `_enum_widget(key: String, options: Array[String], value: String, readonly: bool, ref_field: Dictionary = {}, strict: bool = false) -> OptionButton`
```logic
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
```

### `_checklist_widget(_f: Dictionary, value: Variant, _info: Dictionary, _readonly: bool) -> Control`
```logic
var current: Array[String] = ContentFactory._strings(value)
return _value_label(", ".join(current) if not current.is_empty() else "(none)", UiTheme.COL_TEXT if not current.is_empty() else UiTheme.COL_MUTED)
```

### `_template_widget(_f: Dictionary, value: String, _readonly: bool) -> Control`
```logic
var box := VBoxContainer.new()
box.add_theme_constant_override("separation", 2)
var text: Label = _value_label(value if not value.is_empty() else "(empty)", UiTheme.COL_TEXT if not value.is_empty() else UiTheme.COL_MUTED)
text.name = "Template"
box.add_child(text)
var lines: Array[String] = []
for p: Dictionary in SkillText.validate(value, draft):
	lines.append("[color=#f87171]%s[/color]" % str(p["message"]))
for w: String in SkillText.warnings(value, draft):
	lines.append("[color=#fbbf24]%s[/color]" % w)
if not lines.is_empty():
	var issues: RichTextLabel = RichTextLabel.new()
	issues.name = "Issues"
	issues.bbcode_enabled = true
	issues.fit_content = true
	issues.scroll_active = false
	issues.add_theme_font_size_override("normal_font_size", UiTheme.FONT_SM)
	issues.text = "\n".join(lines)
	box.add_child(issues)
return box
```

### `_refs_widget(f: Dictionary, value: Variant, _info: Dictionary = {}, _readonly: bool = true) -> Control`
```logic
var box := VBoxContainer.new()
box.add_theme_constant_override("separation", 2)
var current: Array[String] = ContentFactory._strings(value)
for id: String in current:
	box.add_child(_value_label("• " + source.ref_label(f, id)))
if current.is_empty():
	box.add_child(_value_label("(none)", UiTheme.COL_MUTED))
return box
```

### `_stats_widget(f: Dictionary, value: Variant, _readonly: bool = true) -> Control`
```logic
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
```

### `_grouped_ref_widget(_key: String, choices: Array, value: String, readonly: bool) -> OptionButton`
```logic
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
```

### `_path_widget(_f: Dictionary, _readonly: bool) -> Control`
```logic
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
```

### `static _family_label(choice: Dictionary) -> String`
```logic
var grade: String = str(choice.get("grade", "novice"))
var requires: Array = DictUtil.array_of(choice.get("requires", []))
return "%s  [%s%s]%s" % [str(choice["name"]), str(choice.get("kind", "")).to_lower(), "" if grade == "novice" else ", " + grade, ("  needs " + ", ".join(PackedStringArray(requires))) if not requires.is_empty() else ""]
```

### `_path_row(row: Dictionary) -> Control`
```logic
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
```

### `_proficiency_widget(f: Dictionary, _readonly: bool) -> Control`
```logic
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
```

### `static _value_label(text: String, color: Color = UiTheme.COL_TEXT) -> Label`
```logic
var lbl: Label = UiTheme.label(text, UiTheme.FONT_MD, color)
if text.length() > 24 or text.contains("\n"):
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	lbl.custom_minimum_size.x = 40
lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
return lbl
```

### `static display_of(control: Control) -> Control`
```logic
var shown: Control = _display_value(control)
if shown == null:
	_flatten_children(control)
	return control
_adopt(control, shown)
control.free()
return shown
```

### `static _flatten_children(node: Node) -> void`
```logic
for child: Node in node.get_children():
	var control: Control = child as Control
	if control == null:
		continue
	var plain: Label = control as Label
	if plain != null and plain.autowrap_mode == TextServer.AUTOWRAP_OFF and not plain.clip_text and plain.text.length() > 32:
		plain.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART   # long counters and notes wrap instead of widening the form
		plain.custom_minimum_size.x = maxf(plain.custom_minimum_size.x, 40.0)
		plain.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		continue
	var shown: Control = _display_value(control)
	if shown == null:
		_flatten_children(control)
		continue
	var at: int = control.get_index()
	node.remove_child(control)   # first, so the label can take over the widget's name
	_adopt(control, shown)
	node.add_child(shown)
	node.move_child(shown, at)
	control.queue_free()
```

### `static _adopt(from: Control, to: Control) -> void`
```logic
if not str(from.name).is_empty():
	to.name = from.name
to.tooltip_text = from.tooltip_text
if not to.tooltip_text.is_empty():
	to.mouse_filter = Control.MOUSE_FILTER_STOP
to.modulate = from.modulate
to.visible = from.visible
to.size_flags_horizontal = from.size_flags_horizontal
```

### `static _display_value(control: Control) -> Control`
```logic
var info: Label = control.get_node_or_null("Info") as Label
if str(control.name).begins_with("Mod_") and info != null:
	return _value_label(info.text)   # the modifier row's own summary ("Req STR 14  (value +0.0)")
var summary: Label = control.find_child("Summary", true, false) as Label
if str(control.name).begins_with("Tier_") and summary != null:
	var label_edit: LineEdit = control.find_child("Label", true, false) as LineEdit
	var desc: LineEdit = control.find_child("Desc", true, false) as LineEdit
	var head: String = str(control.name).trim_prefix("Tier_")
	var title: String = "T%s %s" % [head, label_edit.text if label_edit != null else ""]
	var text: String = desc.text if desc != null and not desc.text.is_empty() else (desc.placeholder_text.trim_prefix("auto: ") if desc != null else "")
	return _value_label("%s: %s\n%s" % [title.strip_edges(), summary.text, text])
if str(control.name).begins_with("Band_") and control.get_node_or_null("Set") is CheckBox:
	var on: bool = (control.get_node("Set") as CheckBox).button_pressed
	var spins: Array[String] = []
	for bound: String in ["Min", "Max"]:
		var spin: SpinBox = control.get_node_or_null(bound) as SpinBox
		if spin != null:
			spins.append(_number(spin))
	return _value_label(" to ".join(spins) if on else "-", UiTheme.COL_TEXT if on else UiTheme.COL_MUTED)
if control is LineEdit:
	var line: LineEdit = control
	if not line.text.is_empty():
		return _value_label(line.text)
	return _value_label(line.placeholder_text if not line.placeholder_text.is_empty() else "(empty)", UiTheme.COL_MUTED)
if control is TextEdit:
	var text: TextEdit = control
	return _value_label(text.text if not text.text.is_empty() else "(empty)", UiTheme.COL_TEXT if not text.text.is_empty() else UiTheme.COL_MUTED)
if control is SpinBox:
	var lbl: Label = _value_label(_number(control as SpinBox))
	lbl.size_flags_horizontal = Control.SIZE_FILL
	return lbl
if control is OptionButton:
	var opt: OptionButton = control
	var picked: String = opt.get_item_text(opt.selected) if opt.selected >= 0 else ""
	return _value_label(picked if not picked.is_empty() else "(none)", UiTheme.COL_TEXT if not picked.is_empty() else UiTheme.COL_MUTED)
if control is CheckBox:
	var check: CheckBox = control
	if check.text.is_empty():
		return _value_label("Yes" if check.button_pressed else "No", UiTheme.COL_TEXT if check.button_pressed else UiTheme.COL_MUTED)
	return _value_label(("✔ " if check.button_pressed else "✘ ") + check.text, UiTheme.COL_TEXT if check.button_pressed else UiTheme.COL_MUTED)
return null
```

### `static _number(spin: SpinBox) -> String`
```logic
var v: float = spin.value
var shown: String = str(roundi(v)) if is_equal_approx(v, roundf(v)) else str(snappedf(v, 0.01))
return "%s%s%s" % [spin.prefix + " " if not spin.prefix.is_empty() else "", shown, " " + spin.suffix if not spin.suffix.is_empty() else ""]
```

