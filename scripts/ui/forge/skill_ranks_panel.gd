class_name SkillRanksPanel
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Skill ranks panel (FORGE1 section 3)
# The ranks section of the skill form: one column per rank (rank 1 = the template values, read-only here), editable
# modifier cells for ranks 2+ (absolute values, modified cells highlighted with a reset button), add / remove / copy-previous
# actions, and one resolved read-only panel per rank (name, cost, numbers, rendered description). All rules come from
# SkillFormModel; the panel only renders them and emits `changed(entry)` with the edited copy of the draft.
# ==============================================================================

signal changed(entry: Dictionary)

const MODIFIED_COLOR: Color = Color(1.0, 0.85, 0.4)

var entry: Dictionary = {}
var head: String = ""

func _init() -> void:
	name = "RanksPanel"
	add_theme_constant_override("separation", 6)

func show_entry(p_entry: Dictionary, p_head: String = "") -> void:
	entry = p_entry
	head = p_head
	UiTheme.clear_children(self)
	add_child(UiTheme.header("Ranks"))
	var view: Dictionary = SkillFormModel.ranks_view(entry, head)
	var rows: Array = view["rows"]
	var count: int = rows.size()
	var bar := HBoxContainer.new()
	bar.name = "RankActions"
	var add_btn: Button = UiTheme.button("+ Add rank", "Add a rank (up to %d); it starts equal to the template" % StatRules.MAX_CLASS_TIER, func() -> void: changed.emit(SkillFormModel.add_rank(entry)), UiTheme.FONT_SM)
	add_btn.name = "AddRank"
	add_btn.disabled = not DictUtil.to_bool(view["can_add"])
	bar.add_child(add_btn)
	var remove_btn: Button = UiTheme.button("- Remove top rank", "Remove the highest rank", func() -> void: changed.emit(SkillFormModel.remove_top_rank(entry)), UiTheme.FONT_SM)
	remove_btn.name = "RemoveRank"
	remove_btn.disabled = not DictUtil.to_bool(view["can_remove"])
	bar.add_child(remove_btn)
	if not str(view["reason"]).is_empty():
		bar.add_child(UiTheme.label("  " + str(view["reason"]), UiTheme.FONT_SM, UiTheme.COL_MUTED))
		add_btn.tooltip_text = str(view["reason"])
		remove_btn.tooltip_text = str(view["reason"])
	add_child(bar)
	if count > 1 or DictUtil.to_bool(view["can_add"]):
		add_child(_table(view))
	_add_resolved(rows)

func _table(view: Dictionary) -> Control:
	var rows: Array = view["rows"]
	var fields: Array = view["fields"]
	var grid := GridContainer.new()
	grid.name = "RankTable"
	grid.columns = rows.size() + 1
	grid.add_theme_constant_override("h_separation", 12)
	grid.add_theme_constant_override("v_separation", 4)
	grid.add_child(UiTheme.label("Field", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	for raw: Variant in rows:
		var r: Dictionary = DictUtil.dict_of(raw)
		var rank: int = DictUtil.to_int(r["rank"])
		grid.add_child(UiTheme.label("Rank %d%s" % [rank, " (template)" if rank == 1 else ""], UiTheme.FONT_SM, UiTheme.COL_HEADER))
	var keys: Array[String] = []
	for key: Variant in fields:
		keys.append(str(key))
	keys.append("unlock_offset")
	for key: String in keys:
		var f: Dictionary = ContentSchema.field("skills", key)
		var label: Label = UiTheme.label(str(f.get("label", key)), UiTheme.FONT_SM, UiTheme.COL_MUTED)
		label.tooltip_text = str(f.get("help", ""))
		label.mouse_filter = Control.MOUSE_FILTER_STOP
		grid.add_child(label)
		for raw: Variant in rows:
			var r: Dictionary = DictUtil.dict_of(raw)
			var rank: int = DictUtil.to_int(r["rank"])
			grid.add_child(_cell(key, f, r, rank))
	grid.add_child(UiTheme.label("", UiTheme.FONT_SM))
	for raw: Variant in rows:
		var rank: int = DictUtil.to_int(DictUtil.dict_of(raw)["rank"])
		if rank >= 3:
			var copy_btn: Button = UiTheme.button("Copy previous", "Copy rank %d's modifiers into this rank (values are copied; no link remains)" % (rank - 1), func() -> void: changed.emit(SkillFormModel.copy_previous(entry, rank)), UiTheme.FONT_SM)
			copy_btn.name = "CopyPrevious%d" % rank
			grid.add_child(copy_btn)
		else:
			grid.add_child(Control.new())
	return grid

# One cell: rank 1 shows the template value read-only; ranks 2+ are editable absolute values.
func _cell(key: String, f: Dictionary, row: Dictionary, rank: int) -> Control:
	var values: Dictionary = DictUtil.dict_of(row["values"])
	var value: Variant = values.get(key, f.get("default", 0))
	if rank == 1:
		var shown: String = _show(value)
		if key == "unlock_offset":
			shown = "+0"
		var base_label: Label = UiTheme.label(shown, UiTheme.FONT_SM)
		base_label.name = "Cell_%s_1" % key
		return base_label
	var modified: Array = DictUtil.array_of(row["modified"])
	var is_modified: bool = modified.has(key)
	var resolved: Dictionary = SkillRanks.effective(entry, rank)
	var info: Dictionary = SkillFormModel.state(resolved, key, head)
	var box := HBoxContainer.new()
	var control: Control = _editor(key, f, value, info, rank)
	control.name = "Cell_%s_%d" % [key, rank]
	control.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if is_modified:
		control.modulate = MODIFIED_COLOR
	box.add_child(control)
	if is_modified:
		var reset: Button = UiTheme.button("↺", "Reset to the template value", func() -> void: changed.emit(SkillFormModel.reset_modifier(entry, rank, key)), UiTheme.FONT_SM)
		reset.name = "Reset_%s_%d" % [key, rank]
		box.add_child(reset)
	return box

func _show(value: Variant) -> String:
	if value is Array:
		var parts: Array[String] = []
		for v: Variant in value:
			parts.append(str(v))
		return ", ".join(parts) if not parts.is_empty() else "(none)"
	if value is float:
		return "%s" % snappedf(DictUtil.to_float(value), 0.01)
	var text: String = str(value)
	return text if not text.is_empty() else "(none)"

func _editor(key: String, f: Dictionary, value: Variant, info: Dictionary, rank: int) -> Control:
	var locked: bool = DictUtil.to_bool(info.get("locked", false))
	match str(f.get("type", "int")):
		"int", "float":
			var spin := SpinBox.new()
			var is_float: bool = str(f["type"]) == "float"
			var lo: float = DictUtil.to_float(info["min"]) if info.get("min", null) != null else DictUtil.to_float(f.get("min", -99999))
			var hi: float = DictUtil.to_float(info["max"]) if info.get("max", null) != null else DictUtil.to_float(f.get("max", 99999))
			spin.min_value = lo
			spin.max_value = maxf(lo, hi)
			spin.step = DictUtil.to_float(f.get("step", 0.01 if is_float else 1.0))
			spin.value = DictUtil.to_float(value)
			spin.editable = not locked
			if key == "unlock_offset":
				spin.tooltip_text = "-1 = auto (%d for this rank)" % (StatRules.TIER_UNLOCK_STEP * (rank - 1))
			spin.value_changed.connect(func(v: float) -> void: changed.emit(SkillFormModel.set_modifier(entry, rank, key, v if is_float else roundi(v))))
			return spin
		"enum":
			var opt := OptionButton.new()
			var options: Array[String] = []
			var allowed: Array = DictUtil.array_of(info.get("options", []))
			if allowed.is_empty():
				options = ContentSchema.options_of(f)
			else:
				for a: Variant in allowed:
					options.append(str(a))
			if not options.has(str(value)):
				options.append(str(value))
			for i: int in range(options.size()):
				opt.add_item(options[i] if not options[i].is_empty() else "(none)", i)
				opt.set_item_metadata(i, options[i])
				if options[i] == str(value):
					opt.select(i)
			opt.disabled = locked
			opt.item_selected.connect(func(i: int) -> void: changed.emit(SkillFormModel.set_modifier(entry, rank, key, str(opt.get_item_metadata(i)))))
			return opt
		"multi_pick":
			var menu := MenuButton.new()
			menu.text = _show(value)
			menu.flat = false
			var popup: PopupMenu = menu.get_popup()
			var options: Array[String] = ContentSchema.options_of(f)
			var current: Array[String] = ContentFactory._strings(value)
			for i: int in range(options.size()):
				popup.add_check_item(options[i], i)
				popup.set_item_checked(i, current.has(options[i]))
			popup.hide_on_checkable_item_selection = false
			popup.id_pressed.connect(func(id: int) -> void:
				var next: Array[String] = ContentFactory._strings(value)
				var option: String = options[id]
				if next.has(option):
					next.erase(option)
				else:
					next.append(option)
				if option == SkillDef.CLEANSE_ALL and next.has(option):
					next = [option]
				elif next.has(SkillDef.CLEANSE_ALL):
					next.erase(SkillDef.CLEANSE_ALL)
				var ordered: Array[String] = []
				for candidate: String in options:
					if next.has(candidate):
						ordered.append(candidate)
				changed.emit(SkillFormModel.set_modifier(entry, rank, key, ordered))
			)
			menu.disabled = locked
			return menu
	return UiTheme.label(_show(value), UiTheme.FONT_SM)

# Read-only resolved panel per rank.
func _add_resolved(rows: Array) -> void:
	for raw: Variant in rows:
		var r: Dictionary = DictUtil.dict_of(raw)
		var rank: int = DictUtil.to_int(r["rank"])
		var values: Dictionary = DictUtil.dict_of(r["values"])
		var parts: Array[String] = ["%d AP" % DictUtil.to_int(values.get("level", 1), 1), SkillText.plural(DictUtil.to_int(values.get("blocks", 1), 1), "block")]
		for key: Variant in values:
			var k: String = str(key)
			if ["level", "blocks", "unlock_offset"].has(k):
				continue
			var f: Dictionary = ContentSchema.field("skills", k)
			var value: Variant = values[key]
			if value == f.get("default", null) and not ["power_mult", "mitigation", "boost"].has(k):
				continue
			parts.append("%s %s" % [str(f.get("label", k)), _show(value)])
		var panel := PanelContainer.new()
		panel.name = "Resolved_%d" % rank
		panel.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG, UiTheme.COL_BORDER, 4, 6, 6))
		var col := VBoxContainer.new()
		panel.add_child(col)
		col.add_child(UiTheme.label("%s  (rank %d%s)" % [str(r["name"]), rank, ", template" if rank == 1 else ""], UiTheme.FONT_MD, UiTheme.COL_HEADER))
		var detail: Label = UiTheme.label(" · ".join(parts), UiTheme.FONT_SM, UiTheme.COL_MUTED)
		detail.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		detail.custom_minimum_size.x = 60
		col.add_child(detail)
		var text: Label = UiTheme.label(str(r["text"]), UiTheme.FONT_SM)
		text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		text.custom_minimum_size.x = 60
		col.add_child(text)
		add_child(panel)
