class_name DojoConsolePanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Dojo Component 4: Test Console
# Attack / skill / item actions plus modifiers (force crit, skill multiplier)
# and damage benchmarks. Buttons enable/disable from the hero's actual gear.
# ==============================================================================

const SKILL_MULTS: Array[Dictionary] = [
	{"label": "×1.0 Strike", "value": 1.0},
	{"label": "×0.5 Quick", "value": 0.5},
	{"label": "×1.5 Heavy", "value": 1.5},
	{"label": "×2.0 Finisher", "value": 2.0},
]

var _action_btns: Dictionary = {}  # String -> Button
var _force_crit: CheckBox
var _mult_option: OptionButton
var _pouch_option: OptionButton
var _pouch_use_btn: Button

func _init() -> void:
	super._init()
	panel_id = "console"
	panel_title = "Test Console"
	panel_short_title = "Console"
	panel_icon = "🧪"
	default_min_size = Vector2(260, 150)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	add_child(scroll)

	var vbox := VBoxContainer.new()
	vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_theme_constant_override("separation", 6)
	scroll.add_child(vbox)

	vbox.add_child(UiTheme.header("ATTACKS"))
	var attacks := _flow(vbox)
	_action("main", "⚔ Main Strike", attacks, func() -> void: session.perform_strike("main"))
	_action("offhand", "🗡 Off-Hand Strike", attacks, func() -> void: session.perform_strike("offhand"))
	_action("dual", "⚔⚔ Dual Strike", attacks, func() -> void: session.perform_strike("dual"))
	_action("ranged", "🏹 Ranged Volley", attacks, func() -> void: session.perform_strike("ranged"))

	vbox.add_child(UiTheme.header("SKILLS & STANCES"))
	var skills := _flow(vbox)
	_action("focus", "✦ Focus Buff", skills, func() -> void: session.use_focus(), "Gain +50% ATK for 2 rounds")
	_action("defend", "🛡 Shield Defend", skills, func() -> void: session.toggle_defend(), "Toggle 50% damage mitigation guard")
	_action("stance", "⚡ Stance Shift", skills, func() -> void: session.shift_stance(), "Toggle Neutral / Offensive stance (+20% ATK)")
	_action("switch", "🔄 Switch Weapon (1b)", skills, func() -> void: session.switch_weapon())

	vbox.add_child(UiTheme.header("MODIFIERS"))
	var mods := HBoxContainer.new()
	mods.add_theme_constant_override("separation", 8)
	vbox.add_child(mods)

	_force_crit = CheckBox.new()
	_force_crit.text = "Force Crit"
	_force_crit.tooltip_text = "Force a critical hit (×2.5) for formula verification"
	_force_crit.focus_mode = Control.FOCUS_NONE
	_force_crit.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	_force_crit.toggled.connect(func(pressed: bool) -> void: session.set_force_crit(pressed))
	mods.add_child(_force_crit)

	mods.add_child(UiTheme.label("Skill:", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	_mult_option = OptionButton.new()
	_mult_option.focus_mode = Control.FOCUS_NONE
	_mult_option.fit_to_longest_item = false
	_mult_option.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	_mult_option.tooltip_text = "Skill damage multiplier applied to the next attacks"
	for def: Dictionary in SKILL_MULTS:
		_mult_option.add_item(str(def.get("label", "")))
	_mult_option.item_selected.connect(_on_mult_selected)
	mods.add_child(_mult_option)

	vbox.add_child(UiTheme.header("BATTLE POUCH", Color(0.5, 0.85, 0.65)))
	var pouch_row := HBoxContainer.new()
	pouch_row.add_theme_constant_override("separation", 4)
	vbox.add_child(pouch_row)
	_pouch_option = OptionButton.new()
	_pouch_option.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_pouch_option.fit_to_longest_item = false
	_pouch_option.clip_text = true
	_pouch_option.custom_minimum_size = Vector2(60, 0)
	_pouch_option.focus_mode = Control.FOCUS_NONE
	_pouch_option.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	pouch_row.add_child(_pouch_option)
	_pouch_use_btn = UiTheme.button("Use Item", "Consume the selected pouch item", _on_use_pouch)
	pouch_row.add_child(_pouch_use_btn)

	vbox.add_child(UiTheme.header("SIMULATION (does not change dummy HP)", UiTheme.COL_WARN))
	var sims := _flow(vbox)
	sims.add_child(UiTheme.button("📊 Benchmark ×10", "Simulate 10 swings with the current gear & modifiers", func() -> void: session.run_benchmark(10)))
	sims.add_child(UiTheme.button("📊 Benchmark ×100", "Simulate 100 swings: min / avg / max, crits, expected damage", func() -> void: session.run_benchmark(100)))

func _flow(parent: Control) -> HFlowContainer:
	var flow := HFlowContainer.new()
	flow.add_theme_constant_override("h_separation", 4)
	flow.add_theme_constant_override("v_separation", 4)
	parent.add_child(flow)
	return flow

func _action(key: String, text: String, parent: Control, callback: Callable, tip: String = "") -> void:
	var btn: Button = UiTheme.button(text, tip, callback)
	parent.add_child(btn)
	_action_btns[key] = btn

func _on_mult_selected(index: int) -> void:
	if index < 0 or index >= SKILL_MULTS.size():
		return
	session.set_skill_mult(UiTheme.dict_float(SKILL_MULTS[index], "value", 1.0))

func _on_use_pouch() -> void:
	session.use_pouch_item(maxi(0, _pouch_option.selected))

func refresh() -> void:
	if session == null or session.active_hero == null or session.active_hero.inventory == null:
		return

	_force_crit.set_pressed_no_signal(session.force_crit)
	for i: int in range(SKILL_MULTS.size()):
		if is_equal_approx(UiTheme.dict_float(SKILL_MULTS[i], "value", 1.0), session.skill_mult):
			_mult_option.select(i)

	var availability: Dictionary = session.get_action_availability()
	for key: String in availability:
		var info_raw: Variant = availability[key]
		var btn_raw: Variant = _action_btns.get(key, null)
		if info_raw is Dictionary and btn_raw is Button:
			var info: Dictionary = info_raw
			var btn: Button = btn_raw
			btn.disabled = not UiTheme.dict_bool(info, "enabled", true)
			btn.tooltip_text = str(info.get("tip", ""))

	var previous: int = _pouch_option.selected
	_pouch_option.clear()
	var pouch: Array[Consumable] = session.active_hero.inventory.battle_pouch
	if pouch.is_empty():
		_pouch_option.add_item("— Battle pouch empty (load in Unit Gear) —")
		_pouch_option.set_item_disabled(0, true)
		_pouch_use_btn.disabled = true
	else:
		for i: int in range(pouch.size()):
			var con: Consumable = pouch[i]
			_pouch_option.add_item("%s Slot %d: %s (%s)" % [UiTheme.consumable_icon(con), i + 1, con.name, UiTheme.consumable_effect_text(con)])
		_pouch_option.select(clampi(previous, 0, pouch.size() - 1))
		_pouch_use_btn.disabled = false
