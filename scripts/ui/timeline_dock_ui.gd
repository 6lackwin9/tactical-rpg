class_name TimelineDockUI
extends Control

# ==============================================================================
# Tactical RPG - Timeline Dock UI
# Bottom timeline dock featuring multi-round navigation arrows, 10-block ruler,
# active combatant action track, and hostile planning mask banner.
# ==============================================================================

signal round_nav_requested(delta: int)
signal block_selected(block: int)
signal combatant_selected(c: Combatant)
signal hero_view_requested()
signal draft_requested(preset_id: String, whole_party: bool)  # planning: AI-drafted sequence
signal clear_requested()                                       # planning: clear the hero's sequence
signal confirm_requested()                                     # planning: lock the round
signal step_remove_requested(step_index: int)
signal step_reorder_requested(from_step: int, to_step: int)

const BLOCK_GAP: int = 4
const TRACK_WIDTH: int = 830   # block widths scale so a round always fills this width

var btn_prev_round: Button
var btn_next_round_arrow: Button
var tracks_container: VBoxContainer
var tracks_ruler_row: HBoxContainer
var total_blocks: int = 10
var block_w: int = 80
var ruler_tick_nodes: Array[PanelContainer] = []
var ruler_tick_labels: Array[Label] = []
var single_track_row: HBoxContainer
var plan_row: HBoxContainer
var plan_info_lbl: Label
var draft_btn: MenuButton
var clear_btn: Button
var confirm_btn: Button

var planning: bool = false
var plan_step_fn: Callable   # func(block: int) -> int: index of the step behind the command at a block
var track_clips_hbox: HBoxContainer
var enemy_classified_banner: PanelContainer

# Callable: func(combatant: Combatant, block: int) -> CombatResolver.ScheduledAction
var action_lookup_fn: Callable

func _init() -> void:
	name = "TimelineDockUI"
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	_build_ui()

func _build_ui() -> void:
	var timeline_flank_hbox := HBoxContainer.new()
	timeline_flank_hbox.set_anchors_preset(Control.PRESET_FULL_RECT)
	timeline_flank_hbox.add_theme_constant_override("separation", 4)
	add_child(timeline_flank_hbox)

	# 1. Left Round Traversal Arrow [ ◀ ] (Spans vertical height of ruler + action track)
	btn_prev_round = Button.new()
	btn_prev_round.text = "◀"
	btn_prev_round.tooltip_text = "Previous Round Timeline"
	btn_prev_round.custom_minimum_size = Vector2(22, 54)
	btn_prev_round.size_flags_vertical = Control.SIZE_EXPAND_FILL
	btn_prev_round.add_theme_font_size_override("font_size", 12)
	btn_prev_round.pressed.connect(func() -> void: round_nav_requested.emit(-1))
	timeline_flank_hbox.add_child(btn_prev_round)

	# 2. Center Timeline Container: BLK Button tray on top, Action Timeline on bottom
	tracks_container = VBoxContainer.new()
	tracks_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	tracks_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	tracks_container.add_theme_constant_override("separation", 2)
	timeline_flank_hbox.add_child(tracks_container)

	# Row 1: BLK Button Tray (Time Ruler Ticks BLK 01 .. BLK 10)
	tracks_ruler_row = HBoxContainer.new()
	tracks_ruler_row.add_theme_constant_override("separation", BLOCK_GAP)
	tracks_container.add_child(tracks_ruler_row)

	_build_ruler()

	# Row 2: Action Timeline (Selected Unit's Continuous Action Track Row)
	single_track_row = HBoxContainer.new()
	single_track_row.add_theme_constant_override("separation", BLOCK_GAP)
	tracks_container.add_child(single_track_row)

	track_clips_hbox = HBoxContainer.new()
	track_clips_hbox.name = "ClipsHBox"
	track_clips_hbox.add_theme_constant_override("separation", BLOCK_GAP)
	single_track_row.add_child(track_clips_hbox)

	# Planning bar: status of the hero's sequence plus Draft / Clear / Confirm
	plan_row = HBoxContainer.new()
	plan_row.add_theme_constant_override("separation", 6)
	plan_row.visible = false
	tracks_container.add_child(plan_row)
	plan_info_lbl = Label.new()
	plan_info_lbl.add_theme_font_size_override("font_size", 9)
	plan_info_lbl.add_theme_color_override("font_color", Color(0.98, 0.78, 0.3))
	plan_info_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	plan_info_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	plan_row.add_child(plan_info_lbl)
	draft_btn = MenuButton.new()
	draft_btn.text = "🤖 Draft ▾"
	draft_btn.tooltip_text = "Let the AI draft a sequence for this hero or the whole party (you can edit it)"
	draft_btn.flat = false
	draft_btn.add_theme_font_size_override("font_size", 9)
	var draft_popup: PopupMenu = draft_btn.get_popup()
	draft_popup.add_theme_font_size_override("font_size", 10)
	draft_popup.add_separator("This hero")
	for i: int in range(PlanPresets.IDS.size()):
		draft_popup.add_item(PlanPresets.label(PlanPresets.IDS[i]), i)
	draft_popup.add_separator("Whole party")
	for i: int in range(PlanPresets.IDS.size()):
		draft_popup.add_item(PlanPresets.label(PlanPresets.IDS[i]), 100 + i)
	draft_popup.id_pressed.connect(func(id: int) -> void:
		if id >= 100:
			draft_requested.emit(PlanPresets.IDS[id - 100], true)
		else:
			draft_requested.emit(PlanPresets.IDS[id], false)
	)
	plan_row.add_child(draft_btn)
	clear_btn = Button.new()
	clear_btn.text = "🗑 Clear"
	clear_btn.tooltip_text = "Remove this hero's whole sequence"
	clear_btn.add_theme_font_size_override("font_size", 9)
	clear_btn.pressed.connect(func() -> void: clear_requested.emit())
	plan_row.add_child(clear_btn)
	confirm_btn = Button.new()
	confirm_btn.text = "✔ CONFIRM ROUND"
	confirm_btn.tooltip_text = "Lock every hero's sequence and start the round [Enter]"
	confirm_btn.add_theme_font_size_override("font_size", 10)
	confirm_btn.pressed.connect(func() -> void: confirm_requested.emit())
	plan_row.add_child(confirm_btn)

	# 3. Right Round Traversal Arrow [ ▶ ] (Spans vertical height of ruler + action track)
	btn_next_round_arrow = Button.new()
	btn_next_round_arrow.text = "▶"
	btn_next_round_arrow.tooltip_text = "Next Round Timeline"
	btn_next_round_arrow.custom_minimum_size = Vector2(22, 54)
	btn_next_round_arrow.size_flags_vertical = Control.SIZE_EXPAND_FILL
	btn_next_round_arrow.add_theme_font_size_override("font_size", 12)
	btn_next_round_arrow.pressed.connect(func() -> void: round_nav_requested.emit(1))
	timeline_flank_hbox.add_child(btn_next_round_arrow)

	# Enemy Classified Banner (Shown in Planning Phase when an enemy is selected)
	enemy_classified_banner = PanelContainer.new()
	enemy_classified_banner.visible = false
	var banner_style := StyleBoxFlat.new()
	banner_style.bg_color = Color(0.12, 0.08, 0.10, 0.95)
	banner_style.border_color = Color(0.65, 0.25, 0.25, 0.8)
	banner_style.set_border_width_all(1)
	banner_style.set_corner_radius_all(6)
	banner_style.content_margin_left = 12
	banner_style.content_margin_right = 12
	banner_style.content_margin_top = 8
	banner_style.content_margin_bottom = 8
	enemy_classified_banner.add_theme_stylebox_override("panel", banner_style)
	tracks_container.add_child(enemy_classified_banner)

	var banner_hbox := HBoxContainer.new()
	banner_hbox.add_theme_constant_override("separation", 10)
	enemy_classified_banner.add_child(banner_hbox)

	var lock_icon := Label.new()
	lock_icon.text = "🔒"
	lock_icon.add_theme_font_size_override("font_size", 16)
	banner_hbox.add_child(lock_icon)

	var banner_vbox := VBoxContainer.new()
	banner_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	banner_hbox.add_child(banner_vbox)

	var banner_t1 := Label.new()
	banner_t1.text = "HOSTILE TIMELINE MASKED — SIMULTANEOUS PLANNING PHASE"
	banner_t1.add_theme_font_size_override("font_size", 9)
	banner_t1.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
	banner_vbox.add_child(banner_t1)

	var banner_t2 := Label.new()
	banner_t2.text = "Hostiles plan in secret. Check Threat Intel on the left to anticipate their actions."
	banner_t2.add_theme_font_size_override("font_size", 8)
	banner_t2.add_theme_color_override("font_color", Color(0.7, 0.6, 0.6))
	banner_vbox.add_child(banner_t2)

	var btn_view_party := Button.new()
	btn_view_party.text = "View Hero Track"
	btn_view_party.add_theme_font_size_override("font_size", 8)
	btn_view_party.pressed.connect(func() -> void: hero_view_requested.emit())
	banner_hbox.add_child(btn_view_party)

# (Re)builds the BLK ruler for the current round length.
func _build_ruler() -> void:
	for child: Node in tracks_ruler_row.get_children():
		tracks_ruler_row.remove_child(child)
		child.queue_free()
	ruler_tick_nodes.clear()
	ruler_tick_labels.clear()
	for b: int in range(1, total_blocks + 1):
		var tick_panel := PanelContainer.new()
		tick_panel.custom_minimum_size = Vector2(block_w, 16)
		tick_panel.clip_contents = true
		var tick_style := StyleBoxFlat.new()
		tick_style.bg_color = Color(0.10, 0.13, 0.18, 0.8)
		tick_style.border_color = Color(0.2, 0.28, 0.38, 0.8)
		tick_style.set_border_width_all(1)
		tick_style.set_corner_radius_all(3)
		tick_style.content_margin_left = 2
		tick_style.content_margin_right = 2
		tick_style.content_margin_top = 1
		tick_style.content_margin_bottom = 1
		tick_panel.add_theme_stylebox_override("panel", tick_style)

		var tick_lbl := Label.new()
		tick_lbl.text = "BLK %02d" % b
		tick_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		tick_lbl.add_theme_font_size_override("font_size", 8)
		tick_lbl.add_theme_color_override("font_color", Color(0.65, 0.75, 0.85))
		tick_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		tick_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		tick_panel.add_child(tick_lbl)

		var cur_b: int = b
		tick_panel.gui_input.connect(func(event: InputEvent) -> void:
			var mouse_event := event as InputEventMouseButton
			if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
				block_selected.emit(cur_b)
		)

		tracks_ruler_row.add_child(tick_panel)
		ruler_tick_nodes.append(tick_panel)
		ruler_tick_labels.append(tick_lbl)

func set_total_blocks(n: int) -> void:
	if n == total_blocks:
		return
	total_blocks = n
	block_w = int(float(TRACK_WIDTH - (n - 1) * BLOCK_GAP) / float(n))
	_build_ruler()

# Planning bar state: info is CombatSession.plan_info(); step_fn maps a block to its step index.
func set_planning(info: Dictionary, step_fn: Callable) -> void:
	planning = bool(info.get("planning", false))
	plan_step_fn = step_fn
	plan_row.visible = planning
	if planning:
		plan_info_lbl.text = str(info.get("text", ""))
		var warn: bool = bool(info.get("warn", false))
		plan_info_lbl.add_theme_color_override("font_color", Color(1.0, 0.5, 0.4) if warn else Color(0.98, 0.78, 0.3))
		confirm_btn.disabled = not bool(info.get("can_confirm", false))

func update_round_arrows(can_go_prev: bool, can_go_next: bool) -> void:
	if btn_prev_round != null:
		btn_prev_round.disabled = not can_go_prev
	if btn_next_round_arrow != null:
		btn_next_round_arrow.disabled = not can_go_next

func update_view(
	target_unit: Combatant,
	active_playhead: int,
	is_enemy_masked: bool,
	can_go_prev: bool,
	can_go_next: bool
) -> void:
	update_round_arrows(can_go_prev, can_go_next)

	# Update Ruler highlights
	for i: int in range(ruler_tick_nodes.size()):
		var panel: PanelContainer = ruler_tick_nodes[i]
		var lbl: Label = ruler_tick_labels[i]
		var b_num: int = i + 1
		var style := StyleBoxFlat.new()
		style.set_corner_radius_all(3)
		style.content_margin_left = 2
		style.content_margin_right = 2
		style.content_margin_top = 1
		style.content_margin_bottom = 1
		if b_num == active_playhead:
			style.bg_color = Color(0.2, 0.45, 0.75, 1.0)
			style.border_color = Color(0.5, 0.9, 1.0, 1.0)
			style.set_border_width_all(2)
			lbl.text = "▼ %02d" % b_num
			lbl.add_theme_color_override("font_color", Color(1.0, 0.95, 0.4))
		else:
			style.bg_color = Color(0.10, 0.13, 0.18, 0.8)
			style.border_color = Color(0.2, 0.28, 0.38, 0.8)
			style.set_border_width_all(1)
			lbl.text = "BLK %02d" % b_num
			lbl.add_theme_color_override("font_color", Color(0.65, 0.75, 0.85))
		panel.add_theme_stylebox_override("panel", style)

	# Handle enemy masked planning phase banner vs single track row
	if is_enemy_masked:
		enemy_classified_banner.visible = true
		single_track_row.visible = false
		return
	else:
		enemy_classified_banner.visible = false
		single_track_row.visible = true

	# Clear previous clips
	for child in track_clips_hbox.get_children():
		child.queue_free()

	if target_unit == null:
		return

	# Rebuild continuous clips (Starts at X=0, exactly under Ruler BLK 01..10)
	var b: int = 1
	while b <= total_blocks:
		var act: CombatResolver.ScheduledAction = null
		if action_lookup_fn.is_valid():
			act = action_lookup_fn.call(target_unit, b) as CombatResolver.ScheduledAction

		if act != null:
			var cost: int = act.end_block - act.start_block + 1
			var clip_w: int = cost * block_w + (cost - 1) * BLOCK_GAP
			var clip := _create_action_clip_widget(act, clip_w, cost)
			track_clips_hbox.add_child(clip)
			b += cost
		else:
			var idle_slot := _create_idle_clip_widget(b == active_playhead)
			track_clips_hbox.add_child(idle_slot)
			b += 1

func _create_action_clip_widget(act: CombatResolver.ScheduledAction, width: int, cost: int) -> PanelContainer:
	var clip := PanelContainer.new()
	clip.custom_minimum_size = Vector2(width, 36)
	clip.clip_contents = true
	clip.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	var clip_style := StyleBoxFlat.new()
	clip_style.set_corner_radius_all(4)
	clip_style.content_margin_left = 4
	clip_style.content_margin_right = 4
	clip_style.content_margin_top = 2
	clip_style.content_margin_bottom = 2
	clip_style.set_border_width_all(1)

	var icon_str: String = "⚔"
	var subtitle: String = ""
	var cmd_display_name: String = act.command.command_name

	if act.command is AttackCommand:
		clip_style.bg_color = Color(0.45, 0.12, 0.14, 0.95)
		clip_style.border_color = Color(0.9, 0.3, 0.3, 1.0)
		icon_str = "⚔"
		var atk: AttackCommand = act.command as AttackCommand
		subtitle = "🎯 %s" % atk.target.name if atk != null and atk.target != null else "🎯 Strike"
		if atk != null and act.command.est_damage > 0:
			subtitle += " ~%d dmg" % act.command.est_damage
	elif act.command is DefendCommand:
		clip_style.bg_color = Color(0.12, 0.25, 0.48, 0.95)
		clip_style.border_color = Color(0.3, 0.6, 0.95, 1.0)
		icon_str = "🛡"
		subtitle = "Mitigate 50%"
	elif act.command is FocusCommand:
		clip_style.bg_color = Color(0.32, 0.14, 0.48, 0.95)
		clip_style.border_color = Color(0.75, 0.4, 0.95, 1.0)
		icon_str = "✦"
		subtitle = "+50% ATK (2 rds)"
	elif act.command is MoveCommand:
		clip_style.bg_color = Color(0.08, 0.32, 0.44, 0.95)
		clip_style.border_color = Color(0.2, 0.8, 0.95, 1.0)
		icon_str = "➜"
		var mv: MoveCommand = act.command as MoveCommand
		var chase: ApproachCommand = act.command as ApproachCommand
		if chase != null and chase.target != null:
			subtitle = "→ %s (%d)" % [chase.target.name, mv.steps]
		else:
			subtitle = "%s %s" % [PlanEntry.dir_arrow(mv.direction), "2 tiles" if mv.steps > 1 else "1 tile"] if mv != null else "Grid Step"
	elif act.command is RestCommand:
		clip_style.bg_color = Color(0.1, 0.3, 0.2, 0.95)
		clip_style.border_color = Color(0.3, 0.85, 0.5, 1.0)
		icon_str = "💤"
		subtitle = "+AP, Neutral"
	elif act.command is TargetCommand:
		clip_style.bg_color = Color(0.08, 0.38, 0.24, 0.95)
		clip_style.border_color = Color(0.25, 0.85, 0.5, 1.0)
		icon_str = "🎯"
		subtitle = "Focus Lock"
	elif act.command is HoldStanceCommand:
		clip_style.bg_color = Color(0.2, 0.28, 0.38, 0.95)
		clip_style.border_color = Color(0.4, 0.65, 0.85, 1.0)
		icon_str = "⏱"
		cmd_display_name = "Hold" if cost == 1 else "Hold Stance"
		subtitle = "Maintain Stance"
	else:
		clip_style.bg_color = Color(0.22, 0.26, 0.34, 0.95)
		clip_style.border_color = Color(0.5, 0.6, 0.7, 1.0)
		icon_str = "·"
		subtitle = "Action"

	clip.add_theme_stylebox_override("panel", clip_style)

	var vbox := VBoxContainer.new()
	vbox.alignment = BoxContainer.ALIGNMENT_CENTER
	vbox.add_theme_constant_override("separation", 0)
	vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	clip.add_child(vbox)

	var title_hbox := HBoxContainer.new()
	title_hbox.add_theme_constant_override("separation", 2)
	title_hbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_child(title_hbox)

	var name_lbl := Label.new()
	name_lbl.text = "%s %s" % [icon_str, cmd_display_name]
	name_lbl.add_theme_font_size_override("font_size", 9)
	name_lbl.add_theme_color_override("font_color", Color.WHITE)
	name_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	title_hbox.add_child(name_lbl)

	var cost_lbl := Label.new()
	cost_lbl.text = "[%db]" % cost
	if act.command.planner_controlled:
		cost_lbl.text = "[%db · %dAP]" % [cost, act.command.ap_cost + act.command.stance_switch_cost]
	cost_lbl.add_theme_font_size_override("font_size", 8)
	cost_lbl.add_theme_color_override("font_color", Color(1.0, 0.9, 0.5))
	title_hbox.add_child(cost_lbl)

	var sub_lbl := Label.new()
	sub_lbl.text = subtitle
	sub_lbl.add_theme_font_size_override("font_size", 7)
	sub_lbl.add_theme_color_override("font_color", Color(0.8, 0.85, 0.9))
	sub_lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	sub_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_child(sub_lbl)

	var cur_c: Combatant = act.combatant
	var start_b: int = act.start_block
	clip.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event == null or not mouse_event.pressed:
			return
		if planning and mouse_event.button_index == MOUSE_BUTTON_RIGHT:
			var step_idx: int = _step_at(start_b)
			if step_idx >= 0:
				step_remove_requested.emit(step_idx)
		elif mouse_event.button_index == MOUSE_BUTTON_LEFT:
			combatant_selected.emit(cur_c)
	)
	if planning:
		clip.tooltip_text = "%s%s\nAP %d • drag onto another command to reorder • right-click to remove" % [cmd_display_name, ("\nExpected hit ~%d damage" % act.command.est_damage) if act.command.est_damage > 0 else "", act.command.ap_cost + act.command.stance_switch_cost]
		clip.set_drag_forwarding(
			func(_pos: Vector2) -> Variant:
				var from_step: int = _step_at(start_b)
				if from_step < 0:
					return null
				var preview := Label.new()
				preview.text = "  ↕ %s" % cmd_display_name
				clip.set_drag_preview(preview)
				return {"timeline_step": from_step},
			func(_pos: Vector2, data: Variant) -> bool:
				if not (data is Dictionary):
					return false
				var d: Dictionary = data
				return DictUtil.int_of(d, "timeline_step", -1) >= 0 and DictUtil.int_of(d, "timeline_step", -1) != _step_at(start_b),
			func(_pos: Vector2, data: Variant) -> void:
				if data is Dictionary:
					var d: Dictionary = data
					step_reorder_requested.emit(DictUtil.int_of(d, "timeline_step", -1), _step_at(start_b))
		)

	return clip

func _create_idle_clip_widget(is_playhead: bool, width: int = -1) -> PanelContainer:
	var slot_w: int = width if width > 0 else block_w
	var slot := PanelContainer.new()
	slot.custom_minimum_size = Vector2(slot_w, 36)
	slot.clip_contents = true

	var slot_style := StyleBoxFlat.new()
	slot_style.set_corner_radius_all(3)
	if is_playhead:
		slot_style.bg_color = Color(0.15, 0.22, 0.32, 0.85)
		slot_style.border_color = Color(0.4, 0.7, 0.95, 0.9)
		slot_style.set_border_width_all(1)
	else:
		slot_style.bg_color = Color(0.08, 0.10, 0.14, 0.6)
		slot_style.border_color = Color(0.18, 0.24, 0.32, 0.5)
		slot_style.set_border_width_all(1)
	slot.add_theme_stylebox_override("panel", slot_style)

	var lbl := Label.new()
	lbl.text = "· Idle"
	lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	lbl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	lbl.add_theme_font_size_override("font_size", 8)
	lbl.add_theme_color_override("font_color", Color(0.4, 0.5, 0.6))
	lbl.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	slot.add_child(lbl)

	return slot

func _step_at(block: int) -> int:
	if not plan_step_fn.is_valid():
		return -1
	var raw: Variant = plan_step_fn.call(block)
	return raw if raw is int else -1
