class_name CombatArenaController
extends CanvasLayer

# ==============================================================================
# Tactical RPG - Combat Arena Controller (Pure View / UI Coordinator)
# Manages visual arena layout, dock arrangement, modal suites, hover tooltips,
# and bidirectional signal dispatching between the UI and combat simulations.
# ==============================================================================

signal combatant_selected(combatant: Combatant)
signal block_selected(block_num: int)
signal step_prev_pressed()
signal step_next_pressed()
signal play_pressed()
signal reset_round_pressed()
signal reset_game_pressed()
signal next_round_pressed()
signal round_nav_requested(target_round: int)
signal log_entry_clicked(entry: ActionLogEntry)
signal combatant_hovered(combatant: Combatant, token: Control)
signal combatant_unhovered()

# Core UI Component References
var roster_panel: RosterPanelUI
var action_log_dock: ActionLogDockUI
var unit_inspector: UnitInspectorUI
var grid_view: CombatGridView
var tactical_forecast: TacticalForecastUI
var playback_controls: PlaybackControlsUI
var timeline_dock: TimelineDockUI
var log_modal: CombatLogModal
var timeline_modal: TimelineModal
var log_rich_text: RichTextLabel

# Convenience property accessor for unit tokens
var unit_token_nodes: Dictionary:
	get:
		return grid_view.unit_token_nodes if grid_view else {}

# Layout Controls
var root_margin: MarginContainer
var main_vbox: VBoxContainer
var top_hbox: HBoxContainer
var center_hbox: HBoxContainer
var bottom_panel: PanelContainer
var bottom_hbox: HBoxContainer

func _init() -> void:
	name = "CombatArenaController"
	_build_ui_layout()

func _build_ui_layout() -> void:
	# Main dark background
	var bg := ColorRect.new()
	bg.color = Color(0.06, 0.08, 0.11, 1.0)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(bg)

	# Root layout container
	root_margin = MarginContainer.new()
	root_margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	root_margin.add_theme_constant_override("margin_left", 14)
	root_margin.add_theme_constant_override("margin_right", 14)
	root_margin.add_theme_constant_override("margin_top", 4)
	root_margin.add_theme_constant_override("margin_bottom", 4)
	add_child(root_margin)

	main_vbox = VBoxContainer.new()
	main_vbox.add_theme_constant_override("separation", 4)
	root_margin.add_child(main_vbox)

	# --------------------------------------------------------------------------
	# TOP ROW: PARTY STATUS (2x2) | ACTION LOG DOCK (Center) | ENEMY STATUS (5x2)
	# --------------------------------------------------------------------------
	top_hbox = HBoxContainer.new()
	top_hbox.custom_minimum_size = Vector2(0, 84)
	top_hbox.add_theme_constant_override("separation", 8)
	main_vbox.add_child(top_hbox)

	roster_panel = RosterPanelUI.new()
	top_hbox.add_child(roster_panel.party_panel)

	action_log_dock = ActionLogDockUI.new()
	top_hbox.add_child(action_log_dock)
	action_log_dock.prev_pressed.connect(func() -> void: step_prev_pressed.emit())
	action_log_dock.next_pressed.connect(func() -> void: step_next_pressed.emit())
	action_log_dock.expand_requested.connect(func() -> void: toggle_log_modal(true))
	action_log_dock.entry_clicked.connect(func(e: ActionLogEntry) -> void: log_entry_clicked.emit(e))

	# Maintain hidden RichTextLabel for headless test / logging compatibility
	log_rich_text = RichTextLabel.new()
	log_rich_text.visible = false
	action_log_dock.add_child(log_rich_text)

	top_hbox.add_child(roster_panel.enemy_panel)

	roster_panel.combatant_selected.connect(func(c: Combatant) -> void: combatant_selected.emit(c))
	roster_panel.combatant_hovered.connect(func(c: Combatant, _card: Control) -> void:
		var token: Control = grid_view.get_token_for(c) if grid_view else null
		combatant_hovered.emit(c, token)
	)
	roster_panel.combatant_unhovered.connect(func() -> void: combatant_unhovered.emit())

	# --------------------------------------------------------------------------
	# CENTER ROW: LEFT POPUP (Inspector) | COMBAT GRID (54px) | RIGHT POPUP (Forecast)
	# --------------------------------------------------------------------------
	center_hbox = HBoxContainer.new()
	center_hbox.add_theme_constant_override("separation", 10)
	center_hbox.size_flags_vertical = Control.SIZE_EXPAND_FILL
	main_vbox.add_child(center_hbox)

	unit_inspector = UnitInspectorUI.new()
	center_hbox.add_child(unit_inspector)

	grid_view = CombatGridView.new()
	center_hbox.add_child(grid_view)
	grid_view.combatant_clicked.connect(func(c: Combatant) -> void: combatant_selected.emit(c))
	grid_view.combatant_hovered.connect(func(c: Combatant, token: Control) -> void:
		combatant_hovered.emit(c, token)
	)
	grid_view.combatant_unhovered.connect(func() -> void: combatant_unhovered.emit())

	tactical_forecast = TacticalForecastUI.new()
	center_hbox.add_child(tactical_forecast)

	# --------------------------------------------------------------------------
	# BOTTOM ROW: PLAYBACK CONTROLS + TIMELINE RULER & DOCK
	# --------------------------------------------------------------------------
	bottom_panel = PanelContainer.new()
	bottom_panel.custom_minimum_size = Vector2(0, 96)
	var bottom_style := StyleBoxFlat.new()
	bottom_style.bg_color = Color(0.07, 0.09, 0.14, 0.98)
	bottom_style.border_color = Color(0.22, 0.32, 0.46, 0.9)
	bottom_style.set_border_width_all(1)
	bottom_style.set_corner_radius_all(6)
	bottom_style.content_margin_left = 8
	bottom_style.content_margin_right = 8
	bottom_style.content_margin_top = 3
	bottom_style.content_margin_bottom = 3
	bottom_panel.add_theme_stylebox_override("panel", bottom_style)
	main_vbox.add_child(bottom_panel)

	bottom_hbox = HBoxContainer.new()
	bottom_hbox.add_theme_constant_override("separation", 10)
	bottom_panel.add_child(bottom_hbox)

	playback_controls = PlaybackControlsUI.new()
	playback_controls.step_prev_pressed.connect(func() -> void: step_prev_pressed.emit())
	playback_controls.play_pressed.connect(func() -> void: play_pressed.emit())
	playback_controls.step_next_pressed.connect(func() -> void: step_next_pressed.emit())
	playback_controls.reset_round_pressed.connect(func() -> void: reset_round_pressed.emit())
	playback_controls.reset_game_pressed.connect(func() -> void: reset_game_pressed.emit())
	playback_controls.popout_pressed.connect(func() -> void: toggle_timeline_modal(true))
	playback_controls.next_round_pressed.connect(func() -> void: next_round_pressed.emit())
	bottom_hbox.add_child(playback_controls)

	timeline_dock = TimelineDockUI.new()
	timeline_dock.round_nav_requested.connect(func(rd: int) -> void: round_nav_requested.emit(rd))
	timeline_dock.block_selected.connect(func(b: int) -> void: block_selected.emit(b))
	timeline_dock.combatant_selected.connect(func(c: Combatant) -> void: combatant_selected.emit(c))
	bottom_hbox.add_child(timeline_dock)

	# Modals setup
	log_modal = CombatLogModal.new()
	log_modal.cell_clicked.connect(func(e: ActionLogEntry) -> void: log_entry_clicked.emit(e))
	add_child(log_modal)

	timeline_modal = TimelineModal.new()
	timeline_modal.round_nav_requested.connect(func(rd: int) -> void: round_nav_requested.emit(rd))
	timeline_modal.block_selected.connect(func(b: int) -> void: block_selected.emit(b))
	timeline_modal.combatant_selected.connect(func(c: Combatant) -> void: combatant_selected.emit(c))
	add_child(timeline_modal)

func mount_scenario(scenario: CombatScenario, grid: CombatGrid) -> void:
	if scenario == null or grid_view == null:
		return

	@warning_ignore("unsafe_cast")
	var tex_floor: Texture2D = scenario.tile_textures.get("floor", null) as Texture2D
	@warning_ignore("unsafe_cast")
	var tex_exit: Texture2D = scenario.tile_textures.get("exit", null) as Texture2D
	@warning_ignore("unsafe_cast")
	var tex_center: Texture2D = scenario.tile_textures.get("center", null) as Texture2D
	grid_view.setup_grid_tiles(tex_floor, tex_exit, tex_center)
	grid_view.clear_tokens()

	if roster_panel:
		roster_panel.clear_roster()

	for ally: Combatant in scenario.allies:
		var pos: Vector2i = grid.get_position(ally) if grid else scenario.get_initial_position(ally)
		grid_view.create_unit_token(ally, scenario.get_token_texture(ally), pos)
		if roster_panel:
			roster_panel.create_roster_card(ally, Color(0.2, 0.6, 1.0), true, pos)

	for enemy: Combatant in scenario.enemies:
		var pos: Vector2i = grid.get_position(enemy) if grid else scenario.get_initial_position(enemy)
		grid_view.create_unit_token(enemy, scenario.get_token_texture(enemy), pos)
		if roster_panel:
			roster_panel.create_roster_card(enemy, Color(0.9, 0.3, 0.3), false, pos)

	# Also initialize placeholder reinforcement tokens if present
	for enemy: Combatant in scenario.reinforcements:
		var pos: Vector2i = grid.get_position(enemy) if grid else scenario.get_initial_position(enemy)
		var token := grid_view.create_unit_token(enemy, scenario.get_token_texture(enemy), pos)
		token.visible = false
		if roster_panel:
			var card := roster_panel.create_roster_card(enemy, Color(0.9, 0.3, 0.3), false, pos)
			if card:
				card.visible = false

	if roster_panel:
		roster_panel.add_party_placeholders()

func select_combatant(selected: Combatant, allies: Array[Combatant]) -> void:
	if roster_panel:
		roster_panel.update_highlights(selected, allies)
	if grid_view:
		grid_view.update_selection_highlight(selected)

func show_hover_card(c: Combatant, token: Control, cur_snap: BlockSnapshot, is_ally: bool, grid_pos: Vector2i) -> void:
	if grid_view:
		grid_view.show_hover_card(c, token, cur_snap, is_ally, grid_pos)

func hide_hover_card() -> void:
	if grid_view:
		grid_view.hide_hover_card()

func update_selected_unit_panel(selected: Combatant, is_ally: bool, pos_text: String, sprite_tex: Texture2D) -> void:
	if unit_inspector and is_instance_valid(selected):
		unit_inspector.update_inspector(selected, is_ally, pos_text, sprite_tex)

func update_tactical_forecast_panel(selected: Combatant, target: Combatant, grid: CombatGrid) -> void:
	if tactical_forecast and is_instance_valid(selected):
		tactical_forecast.update_forecast(selected, target, grid)

func update_active_log_cell(entries: Array[ActionLogEntry], active_idx: int) -> void:
	if action_log_dock:
		action_log_dock.update_active_cell(entries, active_idx)

func trigger_action_fx(entry: ActionLogEntry, actor: Combatant, target: Combatant) -> void:
	if not grid_view or entry == null:
		return

	var actor_token: Control = grid_view.get_token_for(actor)
	var target_token: Control = grid_view.get_token_for(target)

	match entry.category:
		"ATTACK":
			if actor != null and target_token != null:
				grid_view.animate_attacker_lunge(actor, target_token.position)
			if target != null and target_token != null:
				var is_mitigated: bool = ("MITIGAT" in entry.outcome_tag)
				grid_view.animate_token_impact(target, true, is_mitigated)
				var txt := "-%d" % entry.value_num
				if is_mitigated:
					txt += " [MIT]"
				grid_view.spawn_floating_combat_text(target_token.position, txt, Color(0.4, 0.85, 1.0) if is_mitigated else Color(1.0, 0.35, 0.35))
				if entry.outcome_tag == "DEFEATED":
					grid_view.spawn_floating_combat_text(target_token.position + Vector2(0, -14), "💀 DEFEATED!", Color(1.0, 0.25, 0.25))

		"DEFEND":
			if actor != null and actor_token != null:
				grid_view.animate_token_pulse(actor, Color(0.5, 0.8, 1.8))
				grid_view.spawn_floating_combat_text(actor_token.position, "🛡 DEFEND [%d%%]" % entry.value_num, Color(0.4, 0.8, 1.0))

		"BUFF":
			if actor != null and actor_token != null:
				grid_view.animate_token_pulse(actor, Color(1.8, 1.5, 0.4))
				grid_view.spawn_floating_combat_text(actor_token.position, "✦ +%d%% ATK" % entry.value_num, Color(1.0, 0.85, 0.25))

		"MOVE":
			if actor != null and entry.coord_to != Vector2i(-1, -1):
				grid_view.animate_token_move(actor, entry.coord_to)

		"EVENT":
			if "WITHDRAWN" in entry.outcome_tag and actor_token != null:
				grid_view.spawn_floating_combat_text(actor_token.position, "⚑ WITHDRAWN!", Color(0.35, 0.95, 0.55))
			elif "AMBUSH" in entry.outcome_tag:
				grid_view.spawn_floating_combat_text(Vector2(200, 180), "⚠️ AMBUSH SPAWN!", Color(1.0, 0.4, 0.4))

func toggle_log_modal(open: bool, entries: Array[ActionLogEntry] = []) -> void:
	if log_modal == null:
		return
	if open:
		if not entries.is_empty():
			log_modal.set_entries(entries)
		log_modal.open_modal()
	else:
		log_modal.close_modal()

func toggle_timeline_modal(open: bool) -> void:
	if timeline_modal == null:
		return
	if open:
		timeline_modal.open_modal()
	else:
		timeline_modal.close_modal()

func append_log(text: String) -> void:
	if log_rich_text:
		log_rich_text.append_text(text + "\n")

func clear_log() -> void:
	if log_rich_text:
		log_rich_text.clear()
