class_name ItemTables
extends RefCounted

# ==============================================================================
# Tactical RPG - Item Tables (FORGE1_ITEMS 6, matrix 6; ITEM1-L commit 3)
# The typed controls of the item and item-family tabs, rendered from ItemFormModel (no rule lives here):
#   modifier_table  one row per modifier; the parameter widgets swap by kind (ItemModifiers.KINDS; effects from
#                   Passives.effect_specs), dynamic delta limits, add disabled at the cap with the reason
#   tier_table      one row per tier: label, criterion (stat / proficiency / unit level), bonuses allowed for the type
#                   (power, mitigation, stat bonuses, effects), description (empty = auto)
#   hand_grid       the family base numbers or soft bands per hand
#   panel           read-only panels: rendered description, build chips, base values, preview, used by (ForgeForm.panel_fn)
# The Forge is a read-only viewer (TOOLS-RO): every control is built with readonly = true and ForgeForm hides the action
# buttons and add rows; the per-control callbacks (ForgeForm.put) are inert (cleanup candidate).
# ==============================================================================

const BAD_TINT: Color = Color(1.0, 0.6, 0.6)

# ------------------------------------------------------------------------------
# Small widgets
# ------------------------------------------------------------------------------

static func _spin(lo: float, hi: float, step: float, value: float, editable: bool, node_name: String) -> SpinBox:
	var spin := SpinBox.new()
	spin.name = node_name
	spin.min_value = lo
	spin.max_value = maxf(lo, hi)
	spin.step = step
	spin.value = value
	spin.editable = editable
	spin.custom_minimum_size.x = 64
	return spin

# A dropdown of `names` (metadata = the value); disabled options keep their reason as tooltip; an unknown `selected`
# value stays visible as a red "(invalid: x)" item.
static func _options(names: Array, selected: String, disabled: Dictionary, readonly: bool, node_name: String, labels: Dictionary = {}) -> OptionButton:
	var opt := OptionButton.new()
	opt.name = node_name
	opt.fit_to_longest_item = false
	opt.clip_text = true
	opt.custom_minimum_size.x = 60
	var found: bool = false
	for raw: Variant in names:
		var value: String = str(raw)
		opt.add_item(str(labels.get(value, value if not value.is_empty() else "(none)")))
		var i: int = opt.item_count - 1
		opt.set_item_metadata(i, value)
		if disabled.has(value) and value != selected:
			opt.set_item_disabled(i, true)
			opt.set_item_tooltip(i, str(disabled[value]))
		if value == selected:
			opt.select(i)
			found = true
	if not found:
		opt.add_item("(invalid: %s)" % selected)
		opt.set_item_metadata(opt.item_count - 1, selected)
		opt.select(opt.item_count - 1)
		opt.modulate = BAD_TINT
	opt.disabled = readonly
	return opt

static func _picked(opt: OptionButton, index: int) -> String:
	return str(opt.get_item_metadata(index))

static func _mark(control: Control, problems: Array) -> void:
	if not is_instance_valid(control):
		return
	control.modulate = BAD_TINT if not problems.is_empty() else Color.WHITE
	control.tooltip_text = "\n".join(PackedStringArray(problems))

# ------------------------------------------------------------------------------
# Modifier table
# ------------------------------------------------------------------------------

static func modifier_table(form: ForgeForm, key: String, info: Dictionary, readonly: bool) -> Control:
	var box := VBoxContainer.new()
	box.name = "ModifierTable"
	box.add_theme_constant_override("separation", 3)
	var counter: Label = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	counter.name = "Counter"
	box.add_child(counter)
	var rows_box := VBoxContainer.new()
	rows_box.name = "Rows"
	box.add_child(rows_box)
	var lines: Array[Control] = []
	var infos: Array[Label] = []
	for i: int in range(ItemModifiers.modifiers_of(form.draft).size()):
		var line: HBoxContainer = _modifier_row(form, key, i, readonly)
		rows_box.add_child(line)
		lines.append(line)
		infos.append(line.get_node("Info") as Label)
	var add_row := HBoxContainer.new()
	add_row.name = "AddRow"
	var kind_state: Dictionary = ItemFormModel.modifier_kind_state(form.draft)
	var first_ok: String = ""
	for kind: String in ContentFactory._strings(kind_state["options"]):
		if not DictUtil.dict_of(kind_state["disabled"]).has(kind):
			first_ok = kind
			break
	var picker: OptionButton = _options(DictUtil.array_of(kind_state["options"]), first_ok, DictUtil.dict_of(kind_state["disabled"]), readonly, "AddKind", _kind_labels())
	add_row.add_child(picker)
	var add: Dictionary = DictUtil.dict_of(info.get("add", {"allowed": true, "reason": ""}))
	var add_btn: Button = UiTheme.button("+ Add modifier", "Append a modifier of the selected kind", func() -> void:
		if picker.selected < 0:
			return
		form.put(key, ItemFormModel.with_modifier_added(form.draft, _picked(picker, picker.selected)))
		form.rebuild()
	)
	add_btn.name = "AddButton"
	add_btn.disabled = readonly or not DictUtil.to_bool(add.get("allowed", true)) or first_ok.is_empty()
	if not str(add.get("reason", "")).is_empty():
		add_btn.tooltip_text = str(add["reason"])
	picker.disabled = add_btn.disabled
	add_row.add_child(add_btn)
	var reason: Label = UiTheme.label(str(add.get("reason", "")), UiTheme.FONT_SM, UiTheme.COL_MUTED)
	reason.name = "AddReason"
	reason.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	reason.custom_minimum_size.x = 60
	reason.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	add_row.add_child(reason)
	box.add_child(add_row)
	var refresh: Callable = func() -> void:
		if not is_instance_valid(box):
			return
		var rows: Array[Dictionary] = ItemFormModel.modifier_rows(form.draft, form.current_problems())
		var total: float = 0.0
		for row: Dictionary in rows:
			total += DictUtil.to_float(row["value"])
		counter.text = "%d of %d modifiers (family max); modifier value %+.1f" % [rows.size(), DictUtil.to_int(info.get("max", 0)), total]
		for i: int in range(mini(rows.size(), lines.size())):
			var row: Dictionary = rows[i]
			var result: String = ("  = %d" % DictUtil.to_int(row["result"])) if row["result"] != null else ""
			if is_instance_valid(infos[i]):
				infos[i].text = "%s%s  (value %+.1f)" % [str(row["text"]), result, DictUtil.to_float(row["value"])]
			_mark(lines[i], DictUtil.array_of(row["problems"]))
	form.add_live(refresh)
	refresh.call()
	return box

static func _kind_labels() -> Dictionary:
	var out: Dictionary = {}
	for kind: Variant in ItemModifiers.KINDS:
		out[str(kind)] = str(DictUtil.dict_of(ItemModifiers.KINDS[kind]).get("label", kind))
	return out

static func _current_mod(form: ForgeForm, index: int) -> Dictionary:
	var mods: Array[Dictionary] = ItemModifiers.modifiers_of(form.draft)
	return mods[index].duplicate(true) if index >= 0 and index < mods.size() else {}

static func _set_mod(form: ForgeForm, key: String, index: int, field: String, value: Variant, structural: bool) -> void:
	var mod: Dictionary = _current_mod(form, index)
	if mod.is_empty():
		return
	mod[field] = value
	form.put(key, ItemFormModel.with_modifier_set(form.draft, index, mod))
	if structural:
		form.rebuild()

static func _modifier_row(form: ForgeForm, key: String, index: int, readonly: bool) -> HBoxContainer:
	var mod: Dictionary = _current_mod(form, index)
	var kind: String = str(mod.get("kind", ""))
	var line := HBoxContainer.new()
	line.name = "Mod_%d" % (index + 1)
	line.add_theme_constant_override("separation", 4)
	var kind_state: Dictionary = ItemFormModel.modifier_kind_state(form.draft)
	var kind_opt: OptionButton = _options(DictUtil.array_of(kind_state["options"]), kind, DictUtil.dict_of(kind_state["disabled"]), readonly, "Kind", _kind_labels())
	kind_opt.item_selected.connect(func(i: int) -> void:
		form.put(key, ItemFormModel.with_modifier_kind(form.draft, index, _picked(kind_opt, i)))
		form.rebuild()
	)
	line.add_child(kind_opt)
	match kind:
		ItemModifiers.ADJUST:
			var fields: Dictionary = ItemFormModel.adjust_field_state(form.draft, index)
			var field_opt: OptionButton = _options(DictUtil.array_of(fields["options"]), str(mod.get("field", "")), DictUtil.dict_of(fields["disabled"]), readonly, "Field")
			field_opt.item_selected.connect(func(i: int) -> void: _set_mod(form, key, index, "field", _picked(field_opt, i), true))
			line.add_child(field_opt)
			var limits: Dictionary = ItemFormModel.delta_limits(form.draft, index)
			var delta: SpinBox = _spin(DictUtil.to_float(limits["min"]), DictUtil.to_float(limits["max"]), 1.0, DictUtil.to_float(mod.get("delta", 0)), not readonly, "Delta")
			delta.tooltip_text = "Delta %d to %d keeps the resolved value inside its range (0 changes nothing)." % [DictUtil.to_int(limits["min"]), DictUtil.to_int(limits["max"])]
			delta.value_changed.connect(func(v: float) -> void: _set_mod(form, key, index, "delta", roundi(v), false))
			line.add_child(delta)
		ItemModifiers.STAT_BONUS, ItemModifiers.REQUIREMENT:
			var stats: Dictionary = ItemFormModel.stat_state(form.draft, index, kind)
			var stat_opt: OptionButton = _options(DictUtil.array_of(stats["options"]), str(mod.get("stat", "")), DictUtil.dict_of(stats["disabled"]), readonly, "Stat")
			stat_opt.item_selected.connect(func(i: int) -> void: _set_mod(form, key, index, "stat", _picked(stat_opt, i), true))
			line.add_child(stat_opt)
			var value: SpinBox = _spin(DictUtil.to_float(stats["min"]), DictUtil.to_float(stats["max"]), 1.0, DictUtil.to_float(mod.get("value", stats["min"])), not readonly, "Value")
			value.value_changed.connect(func(v: float) -> void: _set_mod(form, key, index, "value", roundi(v), false))
			line.add_child(value)
		ItemModifiers.EFFECT:
			line.add_child(effect_editor(DictUtil.dict_of(mod.get("effect", {})), readonly, func(next: Dictionary, structural: bool) -> void:
				_set_mod(form, key, index, "effect", next, structural)
			))
	var info: Label = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	info.name = "Info"
	info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	info.clip_text = true
	info.custom_minimum_size.x = 60
	line.add_child(info)
	var up: Button = UiTheme.button("▲", "Move up", func() -> void:
		form.put(key, ItemFormModel.with_modifier_moved(form.draft, index, -1))
		form.rebuild()
	)
	up.name = "Up"
	up.disabled = readonly or index == 0
	line.add_child(up)
	var remove: Button = UiTheme.button("✕", "Remove this modifier", func() -> void:
		form.put(key, ItemFormModel.with_modifier_removed(form.draft, index))
		form.rebuild()
	)
	remove.name = "Remove"
	remove.disabled = readonly
	line.add_child(remove)
	return line

# ------------------------------------------------------------------------------
# Effect editor (item grammar; the param widgets come from Passives.effect_specs)
# ------------------------------------------------------------------------------

# `on_change(effect: Dictionary, structural: bool)`: structural changes (type, a choice switch) rebuild the form.
static func effect_editor(fx: Dictionary, readonly: bool, on_change: Callable) -> HBoxContainer:
	var cur: Dictionary = fx.duplicate(true)
	var box := HBoxContainer.new()
	box.name = "Effect"
	box.add_theme_constant_override("separation", 3)
	var type_opt: OptionButton = _options(ItemFormModel.effect_types(), str(cur.get("type", "")), {}, readonly, "EffectType")
	type_opt.item_selected.connect(func(i: int) -> void: on_change.call(ItemFormModel.effect_default(_picked(type_opt, i)), true))
	box.add_child(type_opt)
	var rows: Array[Dictionary] = ItemFormModel.effect_param_rows(cur)
	var groups_done: Dictionary = {}
	for p: Dictionary in rows:
		var key: String = str(p["key"])
		var choice: String = str(p["choice"])
		if not choice.is_empty():
			if not groups_done.has(choice):
				groups_done[choice] = true
				var keys: Array[String] = []
				var active: String = ""
				for q: Dictionary in rows:
					if str(q["choice"]) == choice:
						keys.append(str(q["key"]))
						if DictUtil.to_bool(q["active"]):
							active = str(q["key"])
				var choice_opt: OptionButton = _options(keys, active, {}, readonly, "Choice_" + choice)
				choice_opt.item_selected.connect(func(i: int) -> void: on_change.call(ItemFormModel.effect_with_choice(cur, _picked(choice_opt, i)), true))
				box.add_child(choice_opt)
			if not DictUtil.to_bool(p["active"]):
				continue
		var setter: Callable = func(v: Variant) -> void:
			var next: Dictionary = ItemFormModel.effect_with_param(cur, key, v)
			cur.clear()
			cur.merge(next)
			on_change.call(cur.duplicate(true), false)
		match str(p["kind"]):
			"enum", "skill":
				var opt: OptionButton = _options(DictUtil.array_of(p["options"]), str(p["value"]) if p["value"] != null else "", {}, readonly, "Param_" + key)
				opt.item_selected.connect(func(i: int) -> void: setter.call(_picked(opt, i)))
				box.add_child(opt)
			"int", "float":
				var is_float: bool = str(p["kind"]) == "float"
				var step: float = DictUtil.to_float(p["step"], 0.05 if is_float else 1.0)
				var spin: SpinBox = _spin(DictUtil.to_float(p["min"]), DictUtil.to_float(p["max"]), step, DictUtil.to_float(p["value"]), not readonly, "Param_" + key)
				spin.prefix = key
				spin.custom_minimum_size.x = 96
				spin.value_changed.connect(func(v: float) -> void: setter.call(snappedf(v, step) if is_float else roundi(v)))
				box.add_child(spin)
	return box

# ------------------------------------------------------------------------------
# Tier table
# ------------------------------------------------------------------------------

static func tier_table(form: ForgeForm, key: String, info: Dictionary, readonly: bool) -> Control:
	var box := VBoxContainer.new()
	box.name = "TierTable"
	box.add_theme_constant_override("separation", 4)
	var counter: Label = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	counter.name = "Counter"
	box.add_child(counter)
	var rows_box := VBoxContainer.new()
	rows_box.name = "Rows"
	box.add_child(rows_box)
	var limits: Dictionary = ItemFormModel.tier_limits(form.draft)
	var panels: Array[Control] = []
	var summaries: Array[Label] = []
	var descs: Array[LineEdit] = []
	for i: int in range(ItemModifiers.tiers_of(form.draft).size()):
		var row: VBoxContainer = _tier_row(form, key, i, limits, readonly)
		rows_box.add_child(row)
		panels.append(row)
		summaries.append(row.find_child("Summary", true, false) as Label)
		descs.append(row.find_child("Desc", true, false) as LineEdit)
	var add: Dictionary = DictUtil.dict_of(info.get("add", {"allowed": true, "reason": ""}))
	var add_row := HBoxContainer.new()
	add_row.name = "AddRow"
	var add_btn: Button = UiTheme.button("+ Add tier", "Append a tier (criterion and bonus default to the item)", func() -> void:
		form.put(key, ItemFormModel.with_tier_added(form.draft))
		form.rebuild()
	)
	add_btn.name = "AddButton"
	add_btn.disabled = readonly or not DictUtil.to_bool(add.get("allowed", true))
	add_btn.tooltip_text = str(add.get("reason", ""))
	add_row.add_child(add_btn)
	var reason: Label = UiTheme.label(str(add.get("reason", "")), UiTheme.FONT_SM, UiTheme.COL_MUTED)
	reason.name = "AddReason"
	add_row.add_child(reason)
	box.add_child(add_row)
	var refresh: Callable = func() -> void:
		if not is_instance_valid(box):
			return
		var rows: Array[Dictionary] = ItemFormModel.tier_rows(form.draft, form.current_problems())
		var potential: float = 0.0
		for row: Dictionary in rows:
			potential += DictUtil.to_float(row["value"])
		counter.text = "%d of %d tiers (family max); tier potential %.1f (informational, never in the gear value)" % [rows.size(), DictUtil.to_int(limits["max_tiers"]), potential]
		for i: int in range(mini(rows.size(), panels.size())):
			var row: Dictionary = rows[i]
			if is_instance_valid(summaries[i]):
				summaries[i].text = "%s -> %s  (preview %+.1f)" % [str(row["criterion_text"]), str(row["bonus_text"]), DictUtil.to_float(row["value"])]
			if is_instance_valid(descs[i]):
				descs[i].placeholder_text = "auto: %s" % str(row["auto_desc"])
			_mark(panels[i], DictUtil.array_of(row["problems"]))
	form.add_live(refresh)
	refresh.call()
	return box

static func _current_tier(form: ForgeForm, index: int) -> Dictionary:
	var tiers: Array[Dictionary] = ItemModifiers.tiers_of(form.draft)
	return tiers[index].duplicate(true) if index >= 0 and index < tiers.size() else {}

# Applies `mutate(tier)` to a copy of tier `index` and puts the list back.
static func _edit_tier(form: ForgeForm, key: String, index: int, mutate: Callable, structural: bool) -> void:
	var tier: Dictionary = _current_tier(form, index)
	if tier.is_empty():
		return
	mutate.call(tier)
	form.put(key, ItemFormModel.with_tier_set(form.draft, index, tier))
	if structural:
		form.rebuild()

static func _set_when(form: ForgeForm, key: String, index: int, field: String, value: Variant) -> void:
	_edit_tier(form, key, index, func(t: Dictionary) -> void:
		var when: Dictionary = DictUtil.dict_of(t.get("when", {})).duplicate()
		when[field] = value
		t["when"] = when
	, false)

static func _set_bonus(form: ForgeForm, key: String, index: int, bonus_key: String, value: Variant, structural: bool) -> void:
	_edit_tier(form, key, index, func(t: Dictionary) -> void:
		var bonus: Dictionary = DictUtil.dict_of(t.get("bonus", {})).duplicate(true)
		var empty: bool = (value is int and DictUtil.to_int(value) == 0) or (value is Dictionary and DictUtil.dict_of(value).is_empty()) or (value is Array and DictUtil.array_of(value).is_empty())
		if empty:
			bonus.erase(bonus_key)
		else:
			bonus[bonus_key] = value
		t["bonus"] = bonus
	, structural)

static func _tier_row(form: ForgeForm, key: String, index: int, limits: Dictionary, readonly: bool) -> VBoxContainer:
	var tier: Dictionary = _current_tier(form, index)
	var when: Dictionary = DictUtil.dict_of(tier.get("when", {}))
	var bonus: Dictionary = DictUtil.dict_of(tier.get("bonus", {}))
	var tier_box := VBoxContainer.new()
	tier_box.name = "Tier_%d" % (index + 1)
	tier_box.add_theme_constant_override("separation", 2)
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 4)
	tier_box.add_child(head)
	head.add_child(UiTheme.label("T%d" % (index + 1), UiTheme.FONT_MD, UiTheme.COL_HEADER))
	var label_edit := LineEdit.new()
	label_edit.name = "Label"
	label_edit.text = str(tier.get("label", ""))
	label_edit.placeholder_text = "Label (max %d)" % DictUtil.to_int(limits["label_max"])
	label_edit.editable = not readonly
	label_edit.custom_minimum_size.x = 110
	label_edit.text_changed.connect(func(t: String) -> void: _edit_tier(form, key, index, func(d: Dictionary) -> void: d["label"] = t, false))
	head.add_child(label_edit)
	var when_type: String = str(when.get("type", ""))
	var when_opt: OptionButton = _options(DictUtil.array_of(limits["when_types"]), when_type, {}, readonly, "WhenType")
	when_opt.item_selected.connect(func(i: int) -> void:
		form.put(key, ItemFormModel.with_tier_when_type(form.draft, index, _picked(when_opt, i)))
		form.rebuild()
	)
	head.add_child(when_opt)
	match when_type:
		ItemModifiers.WHEN_STAT:
			var stat_opt: OptionButton = _options(ContentSchema.STAT_KEYS, str(when.get("stat", "")), {}, readonly, "WhenStat")
			stat_opt.item_selected.connect(func(i: int) -> void: _set_when(form, key, index, "stat", _picked(stat_opt, i)))
			head.add_child(stat_opt)
			var v: SpinBox = _spin(DictUtil.to_float(limits["stat_min"]), DictUtil.to_float(limits["stat_max"]), 1.0, DictUtil.to_float(when.get("value", 0)), not readonly, "WhenValue")
			v.value_changed.connect(func(x: float) -> void: _set_when(form, key, index, "value", roundi(x)))
			head.add_child(v)
		ItemModifiers.WHEN_PROFICIENCY:
			var gear_opt: OptionButton = _options(DictUtil.array_of(limits["gear_options"]), str(when.get("gear", limits["gear_default"])), {}, readonly, "WhenGear")
			gear_opt.item_selected.connect(func(i: int) -> void: _set_when(form, key, index, "gear", _picked(gear_opt, i)))
			head.add_child(gear_opt)
			var lv: SpinBox = _spin(DictUtil.to_float(limits["proficiency_min"]), DictUtil.to_float(limits["proficiency_max"]), 1.0, DictUtil.to_float(when.get("level", 1)), not readonly, "WhenLevel")
			lv.prefix = "Lv"
			lv.value_changed.connect(func(x: float) -> void: _set_when(form, key, index, "level", roundi(x)))
			head.add_child(lv)
		ItemModifiers.WHEN_LEVEL:
			var ul: SpinBox = _spin(DictUtil.to_float(limits["level_min"]), DictUtil.to_float(limits["level_max"]), 1.0, DictUtil.to_float(when.get("value", 1)), not readonly, "WhenValue")
			ul.prefix = "Lv"
			ul.value_changed.connect(func(x: float) -> void: _set_when(form, key, index, "value", roundi(x)))
			head.add_child(ul)
	var up: Button = UiTheme.button("▲", "Move up", func() -> void:
		form.put(key, ItemFormModel.with_tier_moved(form.draft, index, -1))
		form.rebuild()
	)
	up.name = "Up"
	up.disabled = readonly or index == 0
	head.add_child(up)
	var remove: Button = UiTheme.button("✕", "Remove this tier", func() -> void:
		form.put(key, ItemFormModel.with_tier_removed(form.draft, index))
		form.rebuild()
	)
	remove.name = "Remove"
	remove.disabled = readonly
	head.add_child(remove)
	tier_box.add_child(_bonus_editor(form, key, index, bonus, limits, readonly))
	var tail := HBoxContainer.new()
	tail.add_theme_constant_override("separation", 4)
	var desc := LineEdit.new()
	desc.name = "Desc"
	desc.text = str(tier.get("desc", ""))
	desc.editable = not readonly
	desc.custom_minimum_size.x = 160
	desc.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	desc.tooltip_text = "Empty = the auto text (label: bonus); max %d characters." % DictUtil.to_int(limits["desc_max"])
	desc.text_changed.connect(func(t: String) -> void: _edit_tier(form, key, index, func(d: Dictionary) -> void: d["desc"] = t, false))
	tail.add_child(desc)
	var summary: Label = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	summary.name = "Summary"
	tail.add_child(summary)
	tier_box.add_child(tail)
	return tier_box

# The bonuses a tier of this item type may carry: power / mitigation spinners, stat bonus rows, effect editors.
static func _bonus_editor(form: ForgeForm, key: String, index: int, bonus: Dictionary, limits: Dictionary, readonly: bool) -> VBoxContainer:
	var box := VBoxContainer.new()
	box.name = "Bonus"
	var ranges: Dictionary = limits["bonus_ranges"]
	var flat := HBoxContainer.new()
	flat.add_theme_constant_override("separation", 4)
	box.add_child(flat)
	flat.add_child(UiTheme.label("Bonus:", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	for bonus_key: String in ContentFactory._strings(limits["bonus_keys"]):
		match bonus_key:
			"power", "mitigation":
				var rng: Array = ranges[bonus_key]
				var spin: SpinBox = _spin(DictUtil.to_float(rng[0]), DictUtil.to_float(rng[1]), 1.0, DictUtil.to_float(bonus.get(bonus_key, 0)), not readonly, "Bonus_" + bonus_key)
				spin.prefix = bonus_key
				spin.custom_minimum_size.x = 120
				var bk: String = bonus_key
				spin.value_changed.connect(func(v: float) -> void: _set_bonus(form, key, index, bk, roundi(v), false))
				flat.add_child(spin)
			"stat_bonuses":
				var stats: Dictionary = DictUtil.dict_of(bonus.get("stat_bonuses", {}))
				var s_rng: Array = ranges["stat_bonuses"]
				for stat_key: Variant in stats:
					var stat: String = str(stat_key)
					var disabled: Dictionary = {}
					for other: Variant in stats:
						if str(other) != stat:
							disabled[str(other)] = "Already a bonus of this tier."
					var stat_opt: OptionButton = _options(ContentSchema.STAT_KEYS, stat, disabled, readonly, "BonusStat_" + stat)
					stat_opt.item_selected.connect(func(i: int) -> void:
						var next: Dictionary = {}
						for s: Variant in stats:
							next[_picked(stat_opt, i) if str(s) == stat else str(s)] = stats[s]
						_set_bonus(form, key, index, "stat_bonuses", next, true)
					)
					flat.add_child(stat_opt)
					var sv: SpinBox = _spin(DictUtil.to_float(s_rng[0]), DictUtil.to_float(s_rng[1]), 1.0, DictUtil.to_float(stats[stat_key]), not readonly, "BonusStatValue_" + stat)
					sv.prefix = "+"
					sv.value_changed.connect(func(v: float) -> void:
						var current: Dictionary = DictUtil.dict_of(DictUtil.dict_of(_current_tier(form, index).get("bonus", {})).get("stat_bonuses", {})).duplicate()
						current[stat] = roundi(v)
						_set_bonus(form, key, index, "stat_bonuses", current, false)
					)
					flat.add_child(sv)
					var drop: Button = UiTheme.button("✕", "Remove the %s bonus" % stat, func() -> void:
						var current: Dictionary = DictUtil.dict_of(DictUtil.dict_of(_current_tier(form, index).get("bonus", {})).get("stat_bonuses", {})).duplicate()
						current.erase(stat)
						_set_bonus(form, key, index, "stat_bonuses", current, true)
					)
					drop.disabled = readonly
					flat.add_child(drop)
				var add_stat: Button = UiTheme.button("+ stat", "Add a stat bonus (max %d)" % DictUtil.to_int(limits["max_stats"]), func() -> void:
					var current: Dictionary = DictUtil.dict_of(DictUtil.dict_of(_current_tier(form, index).get("bonus", {})).get("stat_bonuses", {})).duplicate()
					for s: String in ContentSchema.STAT_KEYS:
						if not current.has(s):
							current[s] = DictUtil.to_int(s_rng[0])
							break
					_set_bonus(form, key, index, "stat_bonuses", current, true)
				)
				add_stat.name = "AddStat"
				add_stat.disabled = readonly or stats.size() >= DictUtil.to_int(limits["max_stats"])
				flat.add_child(add_stat)
			"effects":
				var fx_list: Array = DictUtil.array_of(bonus.get("effects", []))
				for j: int in range(fx_list.size()):
					var row := HBoxContainer.new()
					row.name = "BonusEffect_%d" % (j + 1)
					var fx_index: int = j
					row.add_child(effect_editor(DictUtil.dict_of(fx_list[j]), readonly, func(next: Dictionary, structural: bool) -> void:
						var current: Array = DictUtil.array_of(DictUtil.dict_of(_current_tier(form, index).get("bonus", {})).get("effects", [])).duplicate(true)
						if fx_index < current.size():
							current[fx_index] = next
						_set_bonus(form, key, index, "effects", current, structural)
					))
					var drop_fx: Button = UiTheme.button("✕", "Remove this effect", func() -> void:
						var current: Array = DictUtil.array_of(DictUtil.dict_of(_current_tier(form, index).get("bonus", {})).get("effects", [])).duplicate(true)
						if fx_index < current.size():
							current.remove_at(fx_index)
						_set_bonus(form, key, index, "effects", current, true)
					)
					drop_fx.disabled = readonly
					row.add_child(drop_fx)
					box.add_child(row)
				var add_fx: Button = UiTheme.button("+ effect", "Add an effect (max %d)" % DictUtil.to_int(limits["max_effects"]), func() -> void:
					var current: Array = DictUtil.array_of(DictUtil.dict_of(_current_tier(form, index).get("bonus", {})).get("effects", [])).duplicate(true)
					var types: Array[String] = ItemFormModel.effect_types()
					if not types.is_empty():
						current.append(ItemFormModel.effect_default(types[0]))
					_set_bonus(form, key, index, "effects", current, true)
				)
				add_fx.name = "AddEffect"
				add_fx.disabled = readonly or fx_list.size() >= DictUtil.to_int(limits["max_effects"])
				flat.add_child(add_fx)
	return box

# ------------------------------------------------------------------------------
# Family base / band grid
# ------------------------------------------------------------------------------

static func hand_grid(form: ForgeForm, key: String, grid_kind: String, readonly: bool) -> Control:
	var bands: bool = grid_kind == "bands"
	var keys: Array[String] = ItemFormModel.grid_keys(form.draft)
	var fields: Array[String] = ItemFormModel.grid_fields(form.draft, bands)
	var grid := GridContainer.new()
	grid.name = "HandGrid_" + grid_kind
	grid.columns = fields.size() + 1
	grid.add_theme_constant_override("h_separation", 6)
	grid.add_child(UiTheme.label("hand", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	for field: String in fields:
		grid.add_child(UiTheme.label(field, UiTheme.FONT_SM, UiTheme.COL_MUTED))
	for hand: String in keys:
		grid.add_child(UiTheme.label(hand, UiTheme.FONT_SM))
		for field: String in fields:
			var rng: Dictionary = ItemFormModel.grid_range(field)
			if not bands:
				var spin: SpinBox = _spin(DictUtil.to_float(rng["min"]), DictUtil.to_float(rng["max"]), DictUtil.to_float(rng["step"]), float(ItemFormModel.base_cell(form.draft, hand, field)), not readonly, "Base_%s_%s" % [hand, field])
				spin.value_changed.connect(func(v: float) -> void: form.put(key, ItemFormModel.with_base(form.draft, hand, field, roundi(v))))
				grid.add_child(spin)
				continue
			var cell: Dictionary = ItemFormModel.band_cell(form.draft, hand, field)
			var on: bool = DictUtil.to_bool(cell["set"])
			var cell_box := HBoxContainer.new()
			cell_box.name = "Band_%s_%s" % [hand, field]
			var check := CheckBox.new()
			check.name = "Set"
			check.button_pressed = on
			check.disabled = readonly
			check.tooltip_text = "Soft band (warning only) for %s %s" % [hand, field]
			check.toggled.connect(func(pressed: bool) -> void:
				if pressed:
					form.put(key, ItemFormModel.with_band(form.draft, hand, field, "min", DictUtil.to_float(ItemFormModel.base_cell(form.draft, hand, field)) if field != ItemFamilies.BONUS_BAND else 0.0))
				else:
					form.put(key, ItemFormModel.without_band(form.draft, hand, field))
				form.rebuild()
			)
			cell_box.add_child(check)
			for bound: String in ["min", "max"]:
				var b_spin: SpinBox = _spin(DictUtil.to_float(rng["min"]), DictUtil.to_float(rng["max"]), DictUtil.to_float(rng["step"]), DictUtil.to_float(cell[bound]) if cell[bound] != null else DictUtil.to_float(rng["min"]), on and not readonly, bound.capitalize())
				b_spin.custom_minimum_size.x = 56
				var which: String = bound
				b_spin.value_changed.connect(func(v: float) -> void: form.put(key, ItemFormModel.with_band(form.draft, hand, field, which, v)))
				cell_box.add_child(b_spin)
			grid.add_child(cell_box)
	return grid

# ------------------------------------------------------------------------------
# Read-only panels
# ------------------------------------------------------------------------------

# `stored_id` = the stored entry shown.
static func panel(form: ForgeForm, id: String, stored_id: String) -> Control:
	match id:
		"tokens":
			return _tokens_panel(form)
		"chips":
			return _rich(form, func() -> String: return chips_text(form.draft))
		"base":
			return _rich(form, func() -> String: return base_text(form.draft))
		"preview":
			return _rich(form, func() -> String: return preview_text(form.draft))
		"used_by":
			return _rich(form, func() -> String: return used_by_text(stored_id if not stored_id.is_empty() else str(form.draft.get("id", ""))))
	return null

static func _rich(form: ForgeForm, text_fn: Callable) -> RichTextLabel:
	var rich := RichTextLabel.new()
	rich.bbcode_enabled = true
	rich.fit_content = true
	rich.scroll_active = false
	rich.add_theme_font_size_override("normal_font_size", UiTheme.FONT_SM)
	rich.add_theme_font_size_override("bold_font_size", UiTheme.FONT_MD)
	var refresh: Callable = func() -> void:
		if is_instance_valid(rich):
			rich.text = str(text_fn.call())
	form.add_live(refresh)
	refresh.call()
	return rich

# The rendered description of the item and its token problems and warnings.
static func _tokens_panel(form: ForgeForm) -> Control:
	var box := VBoxContainer.new()
	var rendered: RichTextLabel = RichTextLabel.new()
	rendered.name = "Rendered"
	rendered.bbcode_enabled = true
	rendered.fit_content = true
	rendered.scroll_active = false
	rendered.add_theme_font_size_override("normal_font_size", UiTheme.FONT_SM)
	box.add_child(rendered)
	var refresh: Callable = func() -> void:
		if not is_instance_valid(box):
			return
		var d: Dictionary = ItemFormModel.description_preview(form.draft)
		var lines: Array[String] = ["[color=#8ba3b8]Rendered:[/color] %s" % str(d["text"])]
		for p: Variant in DictUtil.array_of(d["problems"]):
			lines.append("[color=#f87171]%s[/color]" % str(p))
		for w: Variant in DictUtil.array_of(d["warnings"]):
			lines.append("[color=#fbbf24]%s[/color]" % str(w))
		rendered.text = "\n".join(lines)
	form.add_live(refresh)
	refresh.call()
	return box

static func chips_text(draft: Dictionary) -> String:
	var parts: Array[String] = []
	for chip: Dictionary in ItemFormModel.chips(draft):
		parts.append("[color=#8ba3b8]%s[/color] [b]%s[/b]" % [str(chip["label"]), str(chip["value"])])
	return "   ".join(parts)

static func base_text(draft: Dictionary) -> String:
	var lines: Array[String] = ["[color=#8ba3b8]family base + adjustments = result[/color]"]
	for row: Dictionary in ItemFormModel.base_rows(draft):
		var colour: String = "#f87171" if not DictUtil.to_bool(row["in_range"]) else ("#fbbf24" if not DictUtil.to_bool(row["in_band"]) else "#e2e8f0")
		lines.append("%s: [color=%s]%s[/color]" % [str(row["label"]), colour, str(row["text"])])
	return "\n".join(lines)

static func preview_text(draft: Dictionary) -> String:
	var p: Dictionary = ItemFormModel.preview(draft)
	var lines: Array[String] = []
	if not str(p["hand"]).is_empty():
		lines.append("[color=#8ba3b8]Hand:[/color] %s" % str(p["hand"]))
	lines.append("[color=#8ba3b8]Modifiers:[/color] %s" % (str(p["summary"]) if not str(p["summary"]).is_empty() else "-"))
	for t: Variant in DictUtil.array_of(p["tiers"]):
		lines.append("[color=#8ba3b8]Tier:[/color] %s" % str(t))
	var budget: Dictionary = p["budget"]
	for row: Variant in DictUtil.array_of(budget["rows"]):
		var r: Dictionary = DictUtil.dict_of(row)
		lines.append("  %s  %+.1f" % [str(r["label"]), DictUtil.to_float(r["value"])])
	lines.append("[color=#8ba3b8]Gear value:[/color] [b]%.1f[/b]  (tier potential %.1f, informational)" % [DictUtil.to_float(budget["item_value"]), DictUtil.to_float(budget["tier_potential"])])
	var users: Array = DictUtil.array_of(p["users"])
	lines.append("[color=#8ba3b8]Loadouts:[/color] %s" % (", ".join(PackedStringArray(users)) if not users.is_empty() else "none"))
	var named: Array = DictUtil.array_of(p["named"])
	lines.append("[color=#8ba3b8]Named items:[/color] %s" % (", ".join(PackedStringArray(named)) if not named.is_empty() else "none"))
	return "\n".join(lines)

static func used_by_text(family_id: String) -> String:
	var rows: Array[Dictionary] = ItemFormModel.used_by(family_id)
	if rows.is_empty():
		return "No items use this family."
	var parts: Array[String] = []
	for row: Dictionary in rows:
		parts.append("%s [%s]%s" % [str(row["name"]), str(row["id"]), (" " + str(row["hand"])) if not str(row["hand"]).is_empty() else ""])
	return "%d item%s: %s" % [rows.size(), "" if rows.size() == 1 else "s", ", ".join(parts)]
