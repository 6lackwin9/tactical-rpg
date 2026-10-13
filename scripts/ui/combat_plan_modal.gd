class_name CombatPlanModal
extends PanelContainer

# ==============================================================================
# Tactical RPG - Plan Round Screen
# One screen to plan the round's command sequence for every hero:
#   - ONE input area: the valid next actions for the selected hero (Engage a
#     target with auto-moves, self skills, manual moves, withdraw). Each pick is
#     appended at the lowest free block and the screen immediately offers the
#     next input, chained against the sequence so far (stance flow, AP, blocks).
#   - AI presets draft a whole sequence in one click.
#   - A preview timeline per hero shows the sequence; chips can be dragged to
#     reorder or removed. A sequence that breaks a rule is flagged and blocks Confirm.
# Confirm locks the round; afterwards the timeline is only a visualisation.
# ==============================================================================

signal confirmed()
signal closed()

const BLOCK_W: int = 62

var session: CombatSession = null
var token_texture_fn: Callable   # func(c: Combatant) -> Texture2D

var _title_lbl: Label
var _status_lbl: Label
var _hero_list: VBoxContainer
var _ruler: HBoxContainer
var _chooser: VBoxContainer
var _preset_option: OptionButton
var _confirm_btn: Button
var _confirm_hint: Label

func _init() -> void:
	name = "CombatPlanModal"
	set_anchors_preset(Control.PRESET_FULL_RECT)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	add_theme_stylebox_override("panel", UiTheme.box(Color(0.04, 0.06, 0.09, 1.0), Color(0.25, 0.45, 0.75, 0.95), 8, 16, 12, 2))
	_build()

func _build() -> void:
	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 8)
	add_child(root)

	var head := HBoxContainer.new()
	root.add_child(head)
	_title_lbl = UiTheme.label("📝 PLAN ROUND", 13, UiTheme.COL_TITLE)
	head.add_child(_title_lbl)
	head.add_spacer(false)
	head.add_child(UiTheme.button("✕ Close (keeps your plan) [Esc]", "Close without confirming", close_modal, 10))

	_status_lbl = UiTheme.label("", 10, UiTheme.COL_INFO)
	_status_lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	root.add_child(_status_lbl)

	var body := HBoxContainer.new()
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 14)
	root.add_child(body)

	# Left: sequences preview
	var left := VBoxContainer.new()
	left.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left.add_theme_constant_override("separation", 6)
	body.add_child(left)
	left.add_child(UiTheme.header("ROUND SEQUENCE PREVIEW  (click a hero to plan for them • drag chips to reorder)"))
	_ruler = HBoxContainer.new()
	_ruler.add_theme_constant_override("separation", 4)
	left.add_child(_ruler)
	_hero_list = VBoxContainer.new()
	_hero_list.add_theme_constant_override("separation", 6)
	left.add_child(_hero_list)

	# Right: the single input area
	var right := PanelContainer.new()
	right.custom_minimum_size = Vector2(430, 0)
	right.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG, UiTheme.COL_BORDER, 6, 8, 6))
	body.add_child(right)
	var right_v := VBoxContainer.new()
	right_v.add_theme_constant_override("separation", 6)
	right.add_child(right_v)
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	right_v.add_child(scroll)
	_chooser = VBoxContainer.new()
	_chooser.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_chooser.add_theme_constant_override("separation", 5)
	scroll.add_child(_chooser)

	var preset_row := HBoxContainer.new()
	preset_row.add_theme_constant_override("separation", 4)
	right_v.add_child(preset_row)
	preset_row.add_child(UiTheme.label("🤖 AI draft:", UiTheme.FONT_SM, UiTheme.COL_MUTED))
	_preset_option = OptionButton.new()
	_preset_option.add_theme_font_size_override("font_size", 10)
	_preset_option.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	for id: String in PlanPresets.IDS:
		_preset_option.add_item(PlanPresets.label(id))
	_preset_option.item_selected.connect(func(i: int) -> void: _preset_option.tooltip_text = PlanPresets.description(PlanPresets.IDS[i]))
	_preset_option.tooltip_text = PlanPresets.description(PlanPresets.IDS[0])
	preset_row.add_child(_preset_option)
	preset_row.add_child(UiTheme.button("Hero", "Draft the selected hero's sequence with this preset", func() -> void: _apply_preset(false), 10))
	preset_row.add_child(UiTheme.button("Party", "Draft every hero's sequence with this preset", func() -> void: _apply_preset(true), 10))

	var foot := HBoxContainer.new()
	foot.add_theme_constant_override("separation", 8)
	root.add_child(foot)
	foot.add_child(UiTheme.button("🗑 Clear hero", "Remove the selected hero's whole sequence", func() -> void: session.plan_clear(session.focused_hero), 10))
	_confirm_hint = UiTheme.label("", 10, UiTheme.COL_BAD)
	_confirm_hint.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_confirm_hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	foot.add_child(_confirm_hint)
	_confirm_btn = UiTheme.button("✔ CONFIRM ROUND", "Lock the plan. The timeline becomes a read-only preview.", func() -> void: _on_confirm(), 12)
	_confirm_btn.custom_minimum_size = Vector2(180, 30)
	foot.add_child(_confirm_btn)

func setup(p_session: CombatSession) -> void:
	session = p_session
	session.changed.connect(_on_session_changed)

func open_modal() -> void:
	visible = true
	move_to_front()
	_refresh()

func close_modal() -> void:
	visible = false
	closed.emit()

func _on_session_changed() -> void:
	if visible:
		_refresh()

func _on_confirm() -> void:
	if session.confirm_plan():
		close_modal()
		confirmed.emit()

func _apply_preset(party: bool) -> void:
	var id: String = PlanPresets.IDS[maxi(0, _preset_option.selected)]
	if party:
		session.plan_apply_preset_all(id)
	else:
		session.plan_apply_preset(session.focused_hero, id)

# ------------------------------------------------------------------------------
# Rendering
# ------------------------------------------------------------------------------

func _refresh() -> void:
	if session == null:
		return
	var planning: bool = session.is_planning()
	_title_lbl.text = "📝 PLAN ROUND %d  •  %s" % [session.current_round, "choose each hero's sequence" if planning else "plan locked"]
	_status_lbl.text = session.status_text
	_build_ruler()
	UiTheme.clear_children(_hero_list)
	for hero: Combatant in session.allies:
		_build_hero_row(hero, planning)
	_build_chooser(planning)
	var valid: bool = session.plan_is_valid()
	_confirm_btn.disabled = not planning or not valid
	_confirm_hint.text = "" if valid or not planning else "⚠ Fix the red steps first"

func _build_ruler() -> void:
	UiTheme.clear_children(_ruler)
	var spacer := Control.new()
	spacer.custom_minimum_size = Vector2(130, 0)
	_ruler.add_child(spacer)
	for b: int in range(1, CombatPlan.TOTAL_BLOCKS + 1):
		var tick: Label = UiTheme.label("%02d" % b, 8, UiTheme.COL_MUTED)
		tick.custom_minimum_size = Vector2(BLOCK_W, 0)
		tick.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		_ruler.add_child(tick)

func _build_hero_row(hero: Combatant, planning: bool) -> void:
	var selected: bool = hero == session.focused_hero
	var card := PanelContainer.new()
	card.add_theme_stylebox_override("panel", UiTheme.box(Color(0.1, 0.16, 0.24, 0.95) if selected else UiTheme.COL_CARD_BG, UiTheme.COL_INFO if selected else UiTheme.COL_BORDER, 5, 6, 4, 2 if selected else 1))
	_hero_list.add_child(card)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	card.add_child(row)

	var info := VBoxContainer.new()
	info.custom_minimum_size = Vector2(126, 0)
	info.add_theme_constant_override("separation", 1)
	row.add_child(info)
	var name_row := HBoxContainer.new()
	info.add_child(name_row)
	if token_texture_fn.is_valid():
		var tex_raw: Variant = token_texture_fn.call(hero)
		if tex_raw is Texture2D:
			var tex: Texture2D = tex_raw
			name_row.add_child(UiTheme.icon_rect(tex, 22))
	name_row.add_child(UiTheme.label(hero.name, UiTheme.FONT_MD, UiTheme.COL_TEXT))
	var st: CombatPlan.SimState = session.plan.compile(hero)
	info.add_child(UiTheme.label("AP %d/%d  •  %s" % [st.ap_left, CombatPlan.ROUND_AP, CombatTypes.stance_to_string(st.stance)], 9, UiTheme.COL_WARN))
	info.add_child(UiTheme.label("HP %d/%d" % [hero.hp, hero.max_hp], 9, UiTheme.COL_MUTED))
	if not hero.is_active():
		info.add_child(UiTheme.label("out of action", 9, UiTheme.COL_BAD))

	# Sequence track: chips sized by their blocks, in order; invalid ones follow.
	var track := HBoxContainer.new()
	track.add_theme_constant_override("separation", 4)
	track.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(track)
	var steps: Array[PlanStep] = session.plan.steps_of(hero)
	var blocks_per_step: Dictionary = {}
	var ap_per_step: Dictionary = {}
	for e: PlanEntry in st.entries:
		blocks_per_step[e.step_index] = DictUtil.int_of(blocks_per_step, e.step_index) + e.blocks()
		ap_per_step[e.step_index] = DictUtil.int_of(ap_per_step, e.step_index) + e.ap
	var used: int = 0
	for i: int in range(steps.size()):
		var failed: bool = st.error_index >= 0 and i >= st.error_index
		var blocks: int = DictUtil.int_of(blocks_per_step, i)
		var reason: String = ""
		if failed:
			reason = st.error_text if i == st.error_index else "after an invalid step"
		var chip: PlanChipUI = PlanChipUI.create(hero, i, steps[i], blocks, DictUtil.int_of(ap_per_step, i), reason, BLOCK_W)
		chip.remove_requested.connect(func(a: Combatant, idx: int) -> void: session.plan_remove_step(a, idx))
		chip.reorder_requested.connect(func(a: Combatant, from_i: int, to_i: int) -> void: session.plan_move_step(a, from_i, to_i))
		track.add_child(chip)
		used += blocks
	if used < CombatPlan.TOTAL_BLOCKS and st.error_index < 0:
		var idle: Label = UiTheme.label("· %d idle block%s" % [CombatPlan.TOTAL_BLOCKS - used, "" if CombatPlan.TOTAL_BLOCKS - used == 1 else "s"], 9, UiTheme.COL_MUTED)
		idle.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		track.add_child(idle)

	if planning and hero.is_active() and not selected:
		card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
		card.gui_input.connect(func(event: InputEvent) -> void:
			var mouse_event := event as InputEventMouseButton
			if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
				session.select_combatant(hero)
		)

func _build_chooser(planning: bool) -> void:
	UiTheme.clear_children(_chooser)
	var hero: Combatant = session.focused_hero
	if hero == null:
		return
	var st: CombatPlan.SimState = session.plan.compile(hero)
	_chooser.add_child(UiTheme.label("ADD NEXT ACTION  —  %s" % hero.name, UiTheme.FONT_LG, UiTheme.COL_HEADER))
	_chooser.add_child(UiTheme.label("Next block %d  •  AP left %d  •  Stance %s" % [st.next_block, st.ap_left, CombatTypes.stance_to_string(st.stance)], 10, UiTheme.COL_WARN))
	if not planning:
		_chooser.add_child(UiTheme.label("The plan is locked. Step or play to resolve the round.", 10, UiTheme.COL_MUTED))
		return
	if st.error_index >= 0:
		_chooser.add_child(UiTheme.label("⚠ Step %d is invalid: %s\nRemove or reorder it to continue." % [st.error_index + 1, st.error_text], 10, UiTheme.COL_BAD))
		return
	if st.withdrawn:
		_chooser.add_child(UiTheme.label("%s leaves the battle this round." % hero.name, 10, UiTheme.COL_MUTED))
		return

	var options: Array[PlanOption] = session.plan_options(hero)
	_add_engage_section(options)
	_add_self_section(options)
	_add_move_section(options)
	for o: PlanOption in options:
		if o.step.mode == PlanStep.Mode.WITHDRAW:
			_chooser.add_child(UiTheme.header("EXIT"))
			var wd_row := HBoxContainer.new()
			_chooser.add_child(wd_row)
			wd_row.add_child(_option_button(o, "🏃 Withdraw through the exit"))

func _add_engage_section(options: Array[PlanOption]) -> void:
	_chooser.add_child(UiTheme.header("⚔ ENGAGE A TARGET  (moves into range automatically)", Color(0.95, 0.5, 0.5)))
	var by_foe: Dictionary = {}
	var foe_order: Array[Combatant] = []
	for o: PlanOption in options:
		if o.step.mode != PlanStep.Mode.ENGAGE:
			continue
		if not by_foe.has(o.step.target):
			by_foe[o.step.target] = []
			foe_order.append(o.step.target)
		var list: Array = by_foe[o.step.target]
		list.append(o)
	var hero: Combatant = session.focused_hero
	for foe: Combatant in foe_order:
		var card: VBoxContainer = UiTheme.card(_chooser)
		var dist: int = session.grid.get_distance(session.grid.get_position(hero), session.grid.get_position(foe))
		var head := HBoxContainer.new()
		card.add_child(head)
		head.add_child(UiTheme.label(foe.name, UiTheme.FONT_MD, Color(1.0, 0.7, 0.7)))
		head.add_spacer(false)
		var tag: String = "in range" if dist <= hero.attack_range else "%d tiles away" % dist
		head.add_child(UiTheme.label("HP %d/%d • %s" % [foe.hp, foe.max_hp, tag], 9, UiTheme.COL_MUTED))
		var btns := HBoxContainer.new()
		btns.add_theme_constant_override("separation", 3)
		card.add_child(btns)
		var list: Array = by_foe[foe]
		for raw: Variant in list:
			if raw is PlanOption:
				var o: PlanOption = raw
				btns.add_child(_option_button(o, "%s %dAP" % [o.step.skill.display_name, o.step.skill.level]))

func _add_self_section(options: Array[PlanOption]) -> void:
	_chooser.add_child(UiTheme.header("🛡 SELF", Color(0.5, 0.75, 1.0)))
	var row := HFlowContainer.new()
	row.add_theme_constant_override("h_separation", 3)
	row.add_theme_constant_override("v_separation", 3)
	_chooser.add_child(row)
	for o: PlanOption in options:
		if o.step.mode == PlanStep.Mode.SELF:
			row.add_child(_option_button(o, "%s %dAP" % [o.step.skill.label(), o.step.skill.level]))

func _add_move_section(options: Array[PlanOption]) -> void:
	_chooser.add_child(UiTheme.header("➜ REPOSITION  (manual move)", Color(0.4, 0.85, 0.95)))
	var skills: Array[SkillDef] = []
	var by_skill: Dictionary = {}
	for o: PlanOption in options:
		if o.step.mode != PlanStep.Mode.MOVE:
			continue
		if not by_skill.has(o.step.skill):
			by_skill[o.step.skill] = []
			skills.append(o.step.skill)
		var list: Array = by_skill[o.step.skill]
		list.append(o)
	for skill: SkillDef in skills:
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 3)
		_chooser.add_child(row)
		var name_lbl: Label = UiTheme.label("%s  %dAP • %d tile%s" % [skill.label(), skill.level, skill.steps, "" if skill.steps == 1 else "s"], 10, UiTheme.COL_TEXT)
		name_lbl.custom_minimum_size = Vector2(190, 0)
		row.add_child(name_lbl)
		var list: Array = by_skill[skill]
		for raw: Variant in list:
			if raw is PlanOption:
				var o: PlanOption = raw
				row.add_child(_option_button(o, PlanEntry.dir_arrow(o.step.direction)))

# A button for one option: enabled when valid, otherwise greyed with the reason as tooltip.
func _option_button(option: PlanOption, text: String) -> Button:
	var step: PlanStep = option.step
	var hero: Combatant = session.focused_hero
	var btn: Button = UiTheme.button(text, option.cost_text(), func() -> void: session.plan_add_step(hero, step), 10)
	btn.disabled = not option.valid
	if not option.valid:
		btn.tooltip_text = "Unavailable: %s" % option.reason
	return btn
