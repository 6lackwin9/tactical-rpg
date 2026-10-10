class_name TimelineModal
extends PanelContainer

# ==============================================================================
# Tactical RPG - Tactical Timeline Master Suite Modal
# Full-screen multi-track timeline suite for deep synchronous block inspection
# ==============================================================================

signal round_nav_requested(delta: int)
signal block_selected(block: int)
signal combatant_selected(c: Combatant)
signal modal_closed()

const UNIT_MODAL_BLOCK_W: int = 94
const TOTAL_BLOCKS: int = 10
const BLOCK_GAP: int = 4

var btn_prev_round: Button
var btn_next_round: Button
var round_badge: Label
var phase_lbl: Label
var ruler_tick_nodes: Array[PanelContainer] = []
var ruler_tick_labels: Array[Label] = []
var tracks_list: VBoxContainer

var action_lookup_fn: Callable      # func(c: Combatant, b: int) -> CombatResolver.ScheduledAction
var token_texture_fn: Callable     # func(c: Combatant) -> Texture2D

func _init() -> void:
	name = "TimelineModal"
	set_anchors_preset(Control.PRESET_FULL_RECT)
	visible = false
	mouse_filter = Control.MOUSE_FILTER_STOP
	_build_ui()

func _build_ui() -> void:
	var overlay_style := StyleBoxFlat.new()
	overlay_style.bg_color = Color(0.04, 0.06, 0.09, 0.97)
	overlay_style.border_color = Color(0.25, 0.45, 0.75, 0.95)
	overlay_style.set_border_width_all(2)
	overlay_style.set_corner_radius_all(8)
	overlay_style.content_margin_left = 16
	overlay_style.content_margin_right = 16
	overlay_style.content_margin_top = 12
	overlay_style.content_margin_bottom = 12
	add_theme_stylebox_override("panel", overlay_style)

	var modal_vbox := VBoxContainer.new()
	modal_vbox.add_theme_constant_override("separation", 8)
	add_child(modal_vbox)

	# 1. Header Bar: Title | Round Navigation Panel | Close Button
	var hdr_row := HBoxContainer.new()
	hdr_row.add_theme_constant_override("separation", 10)
	modal_vbox.add_child(hdr_row)

	var title_lbl := Label.new()
	title_lbl.text = "🗂 TACTICAL TIMELINE MASTER SUITE (DETAILED EXPANDED VIEW)"
	title_lbl.add_theme_font_size_override("font_size", 12)
	title_lbl.add_theme_color_override("font_color", Color(1.0, 0.85, 0.35))
	hdr_row.add_child(title_lbl)

	# Round Navigation Panel in Modal Suite
	var modal_round_nav_panel := HBoxContainer.new()
	modal_round_nav_panel.add_theme_constant_override("separation", 6)
	hdr_row.add_child(modal_round_nav_panel)

	btn_prev_round = Button.new()
	btn_prev_round.text = "◀ PREV ROUND"
	btn_prev_round.tooltip_text = "Navigate to Previous Round Timeline"
	btn_prev_round.custom_minimum_size = Vector2(95, 24)
	btn_prev_round.add_theme_font_size_override("font_size", 9)
	btn_prev_round.pressed.connect(func() -> void: round_nav_requested.emit(-1))
	modal_round_nav_panel.add_child(btn_prev_round)

	round_badge = Label.new()
	round_badge.text = "ROUND 1 TIMELINE"
	round_badge.custom_minimum_size = Vector2(130, 24)
	round_badge.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	round_badge.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	round_badge.add_theme_font_size_override("font_size", 10)
	round_badge.add_theme_color_override("font_color", Color(1.0, 0.85, 0.35))
	modal_round_nav_panel.add_child(round_badge)

	btn_next_round = Button.new()
	btn_next_round.text = "NEXT ROUND ▶"
	btn_next_round.tooltip_text = "Navigate to Next Round Timeline"
	btn_next_round.custom_minimum_size = Vector2(95, 24)
	btn_next_round.add_theme_font_size_override("font_size", 9)
	btn_next_round.pressed.connect(func() -> void: round_nav_requested.emit(1))
	modal_round_nav_panel.add_child(btn_next_round)

	phase_lbl = Label.new()
	phase_lbl.text = "PLANNING PHASE"
	phase_lbl.add_theme_font_size_override("font_size", 9)
	phase_lbl.add_theme_color_override("font_color", Color(0.95, 0.8, 0.35))
	modal_round_nav_panel.add_child(phase_lbl)

	hdr_row.add_spacer(false)

	var btn_close := Button.new()
	btn_close.text = "✕ CLOSE [ESC]"
	btn_close.tooltip_text = "Close expanded timeline [ESC / F / Tab]"
	btn_close.custom_minimum_size = Vector2(90, 22)
	btn_close.add_theme_font_size_override("font_size", 9)
	var close_style := StyleBoxFlat.new()
	close_style.bg_color = Color(0.35, 0.12, 0.14, 0.9)
	close_style.border_color = Color(0.85, 0.35, 0.35, 1.0)
	close_style.set_border_width_all(1)
	close_style.set_corner_radius_all(3)
	btn_close.add_theme_stylebox_override("normal", close_style)
	btn_close.pressed.connect(func() -> void: close_modal())
	hdr_row.add_child(btn_close)

	var hsep1 := HSeparator.new()
	modal_vbox.add_child(hsep1)

	# 2. Synchronous Time Ruler Row (Aligned exactly with Tracks below)
	var ruler_row := HBoxContainer.new()
	ruler_row.add_theme_constant_override("separation", BLOCK_GAP)
	modal_vbox.add_child(ruler_row)

	var track_col_lbl := Label.new()
	track_col_lbl.text = "COMBATANT"
	track_col_lbl.custom_minimum_size = Vector2(130, 18)
	track_col_lbl.add_theme_font_size_override("font_size", 9)
	track_col_lbl.add_theme_color_override("font_color", Color(0.6, 0.75, 0.9))
	ruler_row.add_child(track_col_lbl)

	ruler_tick_nodes.clear()
	ruler_tick_labels.clear()

	for b: int in range(1, TOTAL_BLOCKS + 1):
		var tick_panel := PanelContainer.new()
		tick_panel.custom_minimum_size = Vector2(UNIT_MODAL_BLOCK_W, 18)
		tick_panel.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
		var tick_style := StyleBoxFlat.new()
		tick_style.bg_color = Color(0.10, 0.13, 0.18, 0.85)
		tick_style.border_color = Color(0.2, 0.3, 0.42, 0.8)
		tick_style.set_border_width_all(1)
		tick_style.set_corner_radius_all(3)
		tick_panel.add_theme_stylebox_override("panel", tick_style)

		var tick_lbl := Label.new()
		tick_lbl.text = "BLK %02d" % b
		tick_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		tick_lbl.add_theme_font_size_override("font_size", 8)
		tick_lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
		tick_panel.add_child(tick_lbl)

		var cur_b: int = b
		tick_panel.gui_input.connect(func(event: InputEvent) -> void:
			var mouse_event := event as InputEventMouseButton
			if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
				block_selected.emit(cur_b)
		)

		ruler_row.add_child(tick_panel)
		ruler_tick_nodes.append(tick_panel)
		ruler_tick_labels.append(tick_lbl)

	# 3. Scrollable Tracks Matrix
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	modal_vbox.add_child(scroll)

	tracks_list = VBoxContainer.new()
	tracks_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	tracks_list.add_theme_constant_override("separation", 6)
	scroll.add_child(tracks_list)

func open_modal() -> void:
	visible = true
	move_to_front()

func close_modal() -> void:
	visible = false
	modal_closed.emit()

func toggle_modal() -> void:
	if visible:
		close_modal()
	else:
		open_modal()

func is_modal_open() -> bool:
	return visible

func update_view(viewed_round: int, active_playhead: int, is_replay: bool, allies: Array, active_enemies: Array, snap: BlockSnapshot, can_go_prev: bool, can_go_next: bool) -> void:
	if round_badge != null:
		round_badge.text = "ROUND %d TIMELINE" % viewed_round
	if btn_prev_round != null:
		btn_prev_round.disabled = not can_go_prev
	if btn_next_round != null:
		btn_next_round.disabled = not can_go_next
	if phase_lbl != null:
		phase_lbl.text = "REPLAY BLOCK %02d / 10" % active_playhead if is_replay else "PLANNING BLOCK %02d / 10" % active_playhead

	# Update modal ruler tick highlights
	for i in range(ruler_tick_nodes.size()):
		var pnl: PanelContainer = ruler_tick_nodes[i]
		var lbl: Label = ruler_tick_labels[i]
		var b_num: int = i + 1
		var style := StyleBoxFlat.new()
		style.set_corner_radius_all(3)
		if b_num == active_playhead:
			style.bg_color = Color(0.2, 0.45, 0.75, 1.0)
			style.border_color = Color(0.5, 0.9, 1.0, 1.0)
			style.set_border_width_all(2)
			lbl.text = "▼ %02d" % b_num
			lbl.add_theme_color_override("font_color", Color(1.0, 0.95, 0.4))
		else:
			style.bg_color = Color(0.10, 0.13, 0.18, 0.85)
			style.border_color = Color(0.2, 0.3, 0.42, 0.8)
			style.set_border_width_all(1)
			lbl.text = "BLK %02d" % b_num
			lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
		pnl.add_theme_stylebox_override("panel", style)

	# Rebuild tracks list
	for child in tracks_list.get_children():
		child.queue_free()

	# Section 1: Hero Battlegroup Tracks (4 slots)
	var ally_hdr := Label.new()
	ally_hdr.text = "⚔ ALLIED HERO BATTLEGROUP (4 SLOTS)"
	ally_hdr.add_theme_font_size_override("font_size", 9)
	ally_hdr.add_theme_color_override("font_color", Color(0.4, 0.75, 1.0))
	tracks_list.add_child(ally_hdr)

	for i in range(4):
		var ally: Combatant = allies[i] if i < allies.size() else null
		var row := _create_unit_track_row(ally, i + 1, true, active_playhead, snap, is_replay)
		tracks_list.add_child(row)

	var sep := HSeparator.new()
	tracks_list.add_child(sep)

	# Section 2: Hostile Opposition Tracks
	var enemy_hdr := Label.new()
	enemy_hdr.text = "💀 HOSTILE OPPOSITION (%d Active Hostiles)" % active_enemies.size()
	enemy_hdr.add_theme_font_size_override("font_size", 9)
	enemy_hdr.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
	tracks_list.add_child(enemy_hdr)

	for i in range(active_enemies.size()):
		var enemy: Combatant = active_enemies[i]
		var row := _create_unit_track_row(enemy, i + 1, false, active_playhead, snap, is_replay)
		tracks_list.add_child(row)

func _create_unit_track_row(combatant: Combatant, slot_idx: int, is_ally: bool, active_playhead: int, snap: BlockSnapshot, is_replay: bool) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", BLOCK_GAP)

	# Left Header box (130px)
	var hdr := PanelContainer.new()
	hdr.custom_minimum_size = Vector2(130, 32)
	var hdr_style := StyleBoxFlat.new()
	hdr_style.set_corner_radius_all(3)
	hdr_style.content_margin_left = 4
	hdr_style.content_margin_right = 4
	hdr_style.content_margin_top = 2
	hdr_style.content_margin_bottom = 2
	hdr_style.set_border_width_all(1)

	if combatant != null:
		var cur_hp: int = snap.hps.get(combatant, combatant.hp) if snap != null else combatant.hp
		var max_hp: int = snap.max_hps.get(combatant, combatant.max_hp) if snap != null else combatant.max_hp
		var is_withdrawn: bool = snap.is_withdrawn.get(combatant, combatant.is_withdrawn) if snap != null else combatant.is_withdrawn
		var is_alive: bool = cur_hp > 0
		var cur_stance: CombatTypes.StanceType = snap.stances.get(combatant, combatant.current_stance) if snap != null else combatant.current_stance

		var team_col: Color = Color(0.2, 0.5, 0.9) if is_ally else Color(0.9, 0.35, 0.35)
		hdr_style.bg_color = Color(0.10, 0.13, 0.19, 0.95)
		hdr_style.border_color = team_col

		var hbox := HBoxContainer.new()
		hbox.add_theme_constant_override("separation", 4)
		hdr.add_child(hbox)

		var icon := TextureRect.new()
		if token_texture_fn.is_valid():
			icon.texture = token_texture_fn.call(combatant)
		icon.custom_minimum_size = Vector2(18, 18)
		icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		if not is_alive or is_withdrawn:
			icon.modulate = Color(0.5, 0.5, 0.5, 0.6)
		hbox.add_child(icon)

		var vinfo := VBoxContainer.new()
		vinfo.add_theme_constant_override("separation", 0)
		hbox.add_child(vinfo)

		var name_lbl := Label.new()
		if not is_alive:
			name_lbl.text = "%s 💀" % combatant.name
			name_lbl.add_theme_color_override("font_color", Color(0.95, 0.35, 0.35))
			hdr.modulate = Color(0.6, 0.6, 0.6, 0.6)
		elif is_withdrawn:
			name_lbl.text = "%s 💨" % combatant.name
			name_lbl.add_theme_color_override("font_color", Color(0.9, 0.7, 0.2))
			hdr.modulate = Color(0.65, 0.65, 0.65, 0.6)
		else:
			name_lbl.text = combatant.name
			name_lbl.add_theme_color_override("font_color", Color.WHITE)
			hdr.modulate = Color.WHITE
		name_lbl.add_theme_font_size_override("font_size", 8)
		vinfo.add_child(name_lbl)

		var hp_lbl := Label.new()
		if not is_alive:
			hp_lbl.text = "0/%d HP [DEFEATED]" % max_hp
		elif is_withdrawn:
			hp_lbl.text = "%d/%d HP [ESCAPED]" % [cur_hp, max_hp]
		else:
			hp_lbl.text = "%d/%d HP | %s" % [cur_hp, max_hp, CombatTypes.stance_to_string(cur_stance)]
		hp_lbl.add_theme_font_size_override("font_size", 7)
		hp_lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
		vinfo.add_child(hp_lbl)
	else:
		hdr_style.bg_color = Color(0.06, 0.08, 0.11, 0.6)
		hdr_style.border_color = Color(0.2, 0.25, 0.35, 0.5)
		var empty_lbl := Label.new()
		empty_lbl.text = "— Slot %d (Empty) —" % slot_idx
		empty_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		empty_lbl.add_theme_font_size_override("font_size", 8)
		empty_lbl.add_theme_color_override("font_color", Color(0.4, 0.48, 0.58))
		hdr.add_child(empty_lbl)

	hdr.add_theme_stylebox_override("panel", hdr_style)
	row.add_child(hdr)

	# Right Action Clips (10 blocks)
	var clips_hbox := HBoxContainer.new()
	clips_hbox.add_theme_constant_override("separation", BLOCK_GAP)
	row.add_child(clips_hbox)

	if combatant == null:
		# Empty placeholder row
		for b in range(1, TOTAL_BLOCKS + 1):
			var slot := PanelContainer.new()
			slot.custom_minimum_size = Vector2(UNIT_MODAL_BLOCK_W, 32)
			var s_style := StyleBoxFlat.new()
			s_style.bg_color = Color(0.06, 0.08, 0.11, 0.4)
			s_style.border_color = Color(0.15, 0.2, 0.28, 0.4)
			s_style.set_border_width_all(1)
			s_style.set_corner_radius_all(3)
			slot.add_theme_stylebox_override("panel", s_style)
			clips_hbox.add_child(slot)
		return row

	# If enemy during planning phase -> Secret intent banner across the 10 blocks
	if not is_ally and not is_replay:
		var secret_panel := PanelContainer.new()
		var total_w: int = TOTAL_BLOCKS * UNIT_MODAL_BLOCK_W + (TOTAL_BLOCKS - 1) * BLOCK_GAP
		secret_panel.custom_minimum_size = Vector2(total_w, 32)
		var sec_style := StyleBoxFlat.new()
		sec_style.bg_color = Color(0.14, 0.08, 0.10, 0.8)
		sec_style.border_color = Color(0.55, 0.2, 0.2, 0.8)
		sec_style.set_border_width_all(1)
		sec_style.set_corner_radius_all(3)
		secret_panel.add_theme_stylebox_override("panel", sec_style)

		var sec_lbl := Label.new()
		sec_lbl.text = "🔒 SECRET ENEMY INTENT POOL — SIMULTANEOUS PLANNING (MASKED)"
		sec_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		sec_lbl.add_theme_font_size_override("font_size", 8)
		sec_lbl.add_theme_color_override("font_color", Color(1.0, 0.5, 0.5))
		secret_panel.add_child(sec_lbl)
		clips_hbox.add_child(secret_panel)
		return row

	# Otherwise render action clips from recorded actions
	var b: int = 1
	while b <= TOTAL_BLOCKS:
		var act: CombatResolver.ScheduledAction = null
		if action_lookup_fn.is_valid():
			act = action_lookup_fn.call(combatant, b)
		if act != null:
			var cost: int = act.end_block - act.start_block + 1
			var clip_w: int = cost * UNIT_MODAL_BLOCK_W + (cost - 1) * BLOCK_GAP
			var clip := _create_action_clip(act, clip_w, cost)
			clips_hbox.add_child(clip)
			b += cost
		else:
			var idle := _create_idle_clip(b == active_playhead, UNIT_MODAL_BLOCK_W)
			clips_hbox.add_child(idle)
			b += 1

	return row

func _create_action_clip(act: CombatResolver.ScheduledAction, width: int, cost: int) -> PanelContainer:
	var clip := PanelContainer.new()
	clip.custom_minimum_size = Vector2(width, 32)
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
		subtitle = "🎯 Strike"
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
		subtitle = "Grid Step"
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
	clip.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
			combatant_selected.emit(cur_c)
	)

	return clip

func _create_idle_clip(is_playhead: bool, width: int = UNIT_MODAL_BLOCK_W) -> PanelContainer:
	var slot := PanelContainer.new()
	slot.custom_minimum_size = Vector2(width, 32)
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
	slot.add_child(lbl)
	return slot
