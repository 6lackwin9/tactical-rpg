class_name DojoConsolePanel
extends DojoPanel

# ==============================================================================
# Tactical RPG - Dojo Component 4: Test Console
# Catalog skills (the same skills, AP costs, stance switches and damage formula
# as in battles), weapon-hand tests, the dummy's attack controls, rounds, the
# battle pouch and damage benchmarks.
# ==============================================================================

var _action_btns: Dictionary = {}  # String -> Button
var _force_crit: CheckBox
var _skill_option: OptionButton
var _skill_info: RichTextLabel
var _round_lbl: Label
var _dummy_skill_option: OptionButton
var _dummy_power: SpinBox
var _dummy_counter: CheckBox
var _pouch_option: OptionButton
var _pouch_use_btn: Button
var _skill_ids: Array[String] = []
var _dummy_skill_ids: Array[String] = []

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

	# --- Catalog skills
	vbox.add_child(UiTheme.header("SKILLS (content catalog)"))
	var skill_row := HBoxContainer.new()
	skill_row.add_theme_constant_override("separation", 4)
	vbox.add_child(skill_row)
	_skill_option = _option(skill_row, "Any catalog skill: same AP, stance and damage rules as in battle")
	_skill_option.item_selected.connect(func(i: int) -> void:
		if i >= 0 and i < _skill_ids.size():
			session.select_skill(_skill_ids[i])
	)
	_skill_info = RichTextLabel.new()
	_skill_info.bbcode_enabled = true
	_skill_info.fit_content = true
	_skill_info.add_theme_font_size_override("normal_font_size", UiTheme.FONT_SM)
	vbox.add_child(_skill_info)
	var round_row := HBoxContainer.new()
	round_row.add_theme_constant_override("separation", 6)
	vbox.add_child(round_row)
	_action("use", "▶ Use Skill", round_row, func() -> void: session.use_skill(), "Use the selected skill on the dummy (attacks) or yourself")
	_round_lbl = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_HEADER)
	_round_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	round_row.add_child(_round_lbl)
	round_row.add_child(UiTheme.button("⏭ Next Round", "End the round: Focus ticks, guards drop, statuses roll over, +4 AP", func() -> void: session.next_round()))

	# --- Weapon-hand tests (dojo-only rules not yet in the catalog)
	vbox.add_child(UiTheme.header("WEAPON TESTS"))
	var weapons := _flow(vbox)
	_action("main", "⚔ Main Strike", weapons, func() -> void: session.perform_strike("main"))
	_action("offhand", "🗡 Off-Hand Strike", weapons, func() -> void: session.perform_strike("offhand"))
	_action("dual", "⚔⚔ Dual Strike", weapons, func() -> void: session.perform_strike("dual"))
	_action("ranged", "🏹 Ranged Shot", weapons, func() -> void: session.perform_strike("ranged"))
	_action("switch", "🔄 Switch Weapon (1b)", weapons, func() -> void: session.switch_weapon())
	_action("stance", "⚡ Stance Shift", weapons, func() -> void: session.shift_stance(), "Cycle stance: Neutral > Offensive > Tactical > Defensive (matchup vs the dummy: x1.25 / x0.75)")

	# --- The dummy fights back
	vbox.add_child(UiTheme.header("DUMMY ATTACKS", Color(0.97, 0.55, 0.45)))
	var dummy_row := HBoxContainer.new()
	dummy_row.add_theme_constant_override("separation", 4)
	vbox.add_child(dummy_row)
	_dummy_skill_option = _option(dummy_row, "The attack skill the dummy uses")
	_dummy_skill_option.item_selected.connect(func(i: int) -> void:
		if i >= 0 and i < _dummy_skill_ids.size():
			session.select_dummy_skill(_dummy_skill_ids[i])
	)
	dummy_row.add_child(UiTheme.label("Power", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	_dummy_power = SpinBox.new()
	_dummy_power.min_value = 0
	_dummy_power.max_value = 500
	_dummy_power.custom_minimum_size.x = 70
	_dummy_power.tooltip_text = "Dummy strike power (0 = from its stats: unarmed STR + DEX)"
	_dummy_power.value_changed.connect(func(v: float) -> void: session.set_dummy_power(roundi(v)))
	dummy_row.add_child(_dummy_power)
	var dummy_btns := _flow(vbox)
	dummy_btns.add_child(UiTheme.button("⚔ Dummy Attacks", "The dummy strikes the active hero (hero guard, armor and stance apply)", func() -> void: session.dummy_attack()))
	_dummy_counter = CheckBox.new()
	_dummy_counter.text = "Counterattack"
	_dummy_counter.tooltip_text = "The dummy strikes back after every hero skill"
	_dummy_counter.focus_mode = Control.FOCUS_NONE
	_dummy_counter.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	_dummy_counter.toggled.connect(func(on: bool) -> void: session.set_dummy_counter(on))
	dummy_btns.add_child(_dummy_counter)
	dummy_btns.add_child(UiTheme.button("💖 Heal Hero", "Restore the active hero to full HP", func() -> void: session.heal_hero()))

	# --- Modifiers
	vbox.add_child(UiTheme.header("MODIFIERS"))
	_force_crit = CheckBox.new()
	_force_crit.text = "Force Crit"
	_force_crit.tooltip_text = "Force the hero's next hits to crit (×%.1f) for formula checks" % DamageCalculator.CRIT_MULT
	_force_crit.focus_mode = Control.FOCUS_NONE
	_force_crit.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	_force_crit.toggled.connect(func(pressed: bool) -> void: session.set_force_crit(pressed))
	vbox.add_child(_force_crit)

	# --- Battle pouch
	vbox.add_child(UiTheme.header("BATTLE POUCH", Color(0.5, 0.85, 0.65)))
	var pouch_row := HBoxContainer.new()
	pouch_row.add_theme_constant_override("separation", 4)
	vbox.add_child(pouch_row)
	_pouch_option = _option(pouch_row, "")
	_pouch_use_btn = UiTheme.button("Use Item", "Consume the selected pouch item", _on_use_pouch)
	pouch_row.add_child(_pouch_use_btn)

	# --- Benchmarks
	vbox.add_child(UiTheme.header("SIMULATION (does not change dummy HP)", UiTheme.COL_WARN))
	var sims := _flow(vbox)
	sims.add_child(UiTheme.button("📊 Benchmark ×10", "Simulate 10 swings of the selected attack skill (Strike if it isn't an attack)", func() -> void: session.run_benchmark(10)))
	sims.add_child(UiTheme.button("📊 Benchmark ×100", "Simulate 100 swings: min / avg / max, crits, expected damage", func() -> void: session.run_benchmark(100)))

func _flow(parent: Control) -> HFlowContainer:
	var flow := HFlowContainer.new()
	flow.add_theme_constant_override("h_separation", 4)
	flow.add_theme_constant_override("v_separation", 4)
	parent.add_child(flow)
	return flow

func _option(parent: Control, tip: String) -> OptionButton:
	var opt := OptionButton.new()
	opt.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	opt.fit_to_longest_item = false
	opt.clip_text = true
	opt.custom_minimum_size = Vector2(60, 0)
	opt.focus_mode = Control.FOCUS_NONE
	opt.tooltip_text = tip
	opt.add_theme_font_size_override("font_size", UiTheme.FONT_SM)
	parent.add_child(opt)
	return opt

func _action(key: String, text: String, parent: Control, callback: Callable, tip: String = "") -> void:
	var btn: Button = UiTheme.button(text, tip, callback)
	parent.add_child(btn)
	_action_btns[key] = btn

func _on_use_pouch() -> void:
	session.use_pouch_item(maxi(0, _pouch_option.selected))

static func skill_row_text(s: SkillDef) -> String:
	var extra: String = ""
	match s.kind:
		SkillDef.Kind.ATTACK:
			extra = " x%.2f%s" % [s.power_mult, " area" if s.is_area() else ""]
		SkillDef.Kind.DEFEND:
			extra = " -%d%%" % roundi(s.mitigation * 100.0)
		SkillDef.Kind.FOCUS:
			extra = " x%.2f" % s.boost
	return "%s · L%d · %db%s" % [s.label(), s.level, s.blocks, extra]

func _fill(opt: OptionButton, ids: Array[String], skills: Array[SkillDef], selected_id: String) -> void:
	opt.clear()
	ids.clear()
	for s: SkillDef in skills:
		opt.add_item(skill_row_text(s))
		ids.append(s.id)
		if s.id == selected_id:
			opt.select(opt.item_count - 1)

func refresh() -> void:
	if session == null or session.active_hero == null or session.active_hero.inventory == null:
		return
	var hero: Combatant = session.active_hero
	_force_crit.set_pressed_no_signal(session.force_crit)
	_dummy_counter.set_pressed_no_signal(session.dummy_counter)
	if session.test_dummy != null:
		_dummy_power.set_value_no_signal(session.test_dummy.power_override)

	_fill(_skill_option, _skill_ids, session.usable_skills(), session.selected_skill_id)
	_fill(_dummy_skill_option, _dummy_skill_ids, session.attack_skills(), session.dummy_skill_id)

	var skill: SkillDef = session.selected_skill()
	var info: String = "[color=#94a3b8]%s[/color]\n%d AP%s • %s stance" % [skill.description, session.skill_ap(skill), " (incl. +1 switch)" if skill.switch_cost(hero.current_stance) > 0 else "", CombatTypes.stance_to_string(skill.stance_after(hero.current_stance))]
	if skill.kind == SkillDef.Kind.ATTACK and session.test_dummy != null:
		info += " • ~%d dmg on the dummy" % DamageCalculator.expected(hero, session.test_dummy, skill, skill.stance())
	_skill_info.text = info
	_round_lbl.text = "Round %d • AP %d / %d%s" % [session.round_num, hero.ap, hero.max_ap, " (debt)" if hero.ap < 0 else ""]

	var availability: Dictionary = session.get_action_availability()
	for key: String in availability:
		var info_raw: Variant = availability[key]
		var btn_raw: Variant = _action_btns.get(key, null)
		if info_raw is Dictionary and btn_raw is Button:
			var avail: Dictionary = info_raw
			var btn: Button = btn_raw
			btn.disabled = not UiTheme.dict_bool(avail, "enabled", true)
			btn.tooltip_text = str(avail.get("tip", ""))

	var previous: int = _pouch_option.selected
	_pouch_option.clear()
	var pouch: Array[Consumable] = hero.inventory.battle_pouch
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
