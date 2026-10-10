extends Node2D

# ==============================================================================
# Tactical RPG - Interactive Visual Combat Arena & Headless Test Suite
# ==============================================================================

# Asset Textures (loaded dynamically at runtime)
var tex_floor: Texture2D
var tex_exit: Texture2D
var tex_center: Texture2D
var tex_warrior: Texture2D
var tex_ranger: Texture2D
var tex_brawler: Texture2D
var tex_scout: Texture2D

static func _load_svg_texture(res_path: String) -> Texture2D:
	if FileAccess.file_exists(res_path):
		var file := FileAccess.open(res_path, FileAccess.READ)
		if file != null:
			var svg_text: String = file.get_as_text()
			var img := Image.new()
			var err: Error = img.load_svg_from_buffer(svg_text.to_utf8_buffer())
			if err == OK:
				return ImageTexture.create_from_image(img)
	return null

func _init_textures() -> void:
	tex_floor = _load_svg_texture("res://assets/tile_floor.svg")
	tex_exit = _load_svg_texture("res://assets/tile_exit.svg")
	tex_center = _load_svg_texture("res://assets/tile_center.svg")
	tex_warrior = _load_svg_texture("res://assets/token_warrior.svg")
	tex_ranger = _load_svg_texture("res://assets/token_ranger.svg")
	tex_brawler = _load_svg_texture("res://assets/token_brawler.svg")
	tex_scout = _load_svg_texture("res://assets/token_scout.svg")

const TILE_SIZE: int = 54
const GRID_SIZE: int = 9

# Interactive UI References
var ui_canvas: CanvasLayer
var timeline_blocks: Array[PanelContainer] = []
var timeline_labels: Array[Label] = []
var grid_tile_nodes: Dictionary = {}  # Vector2i -> TextureRect
var unit_token_nodes: Dictionary = {} # Combatant -> Control
var roster_cards: Dictionary = {}     # Combatant -> Control
var log_rich_text: RichTextLabel
var round_label: Label
var status_label: Label
var btn_step: Button
var btn_play: Button
var btn_reset: Button
var auto_play_timer: Timer

# Simulation State
var sim_grid: CombatGrid
var sim_resolver: CombatResolver
var sim_allies: Array[Combatant] = []
var sim_enemies: Array[Combatant] = []
var current_sim_block: int = 1
var is_auto_playing: bool = false
var sim_finished: bool = false

func _ready() -> void:
	print("\n=======================================================")
	print("=== STARTING TACTICAL RPG ARENA HARNESS             ===")
	print("=======================================================")

	var test1_passed: bool = _run_warrior_vs_3_goblins_battle()
	var test2_passed: bool = _run_combat_grid_and_withdrawal_test()
	var all_tests_passed: bool = test1_passed and test2_passed

	var exit_code: int = 0 if all_tests_passed else 1

	if DisplayServer.get_name() == "headless":
		print("Execution finished headlessly. Exiting with code: %d\n" % exit_code)
		get_tree().quit(exit_code)
	else:
		print("Running in windowed mode. Initializing interactive tactical UI...\n")
		_setup_interactive_ui()
		_init_interactive_simulation()


# ==============================================================================
# SECTION 1: INTERACTIVE UI CREATION & LAYOUT
# ==============================================================================

func _setup_interactive_ui() -> void:
	_init_textures()
	ui_canvas = CanvasLayer.new()
	add_child(ui_canvas)

	# Main background
	var bg := ColorRect.new()
	bg.color = Color(0.06, 0.08, 0.11, 1.0)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	ui_canvas.add_child(bg)

	# Root layout container
	var root_margin := MarginContainer.new()
	root_margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	root_margin.add_theme_constant_override("margin_left", 24)
	root_margin.add_theme_constant_override("margin_right", 24)
	root_margin.add_theme_constant_override("margin_top", 16)
	root_margin.add_theme_constant_override("margin_bottom", 16)
	ui_canvas.add_child(root_margin)

	var main_vbox := VBoxContainer.new()
	main_vbox.add_theme_constant_override("separation", 14)
	root_margin.add_child(main_vbox)

	# 1. Header Bar
	var header_hbox := HBoxContainer.new()
	main_vbox.add_child(header_hbox)

	var title_lbl := Label.new()
	title_lbl.text = "TACTICAL RPG - 10-BLOCK SIMULTANEOUS COMBAT ARENA"
	title_lbl.add_theme_font_size_override("font_size", 20)
	title_lbl.add_theme_color_override("font_color", Color(0.9, 0.95, 1.0))
	header_hbox.add_child(title_lbl)

	header_hbox.add_spacer(false)

	round_label = Label.new()
	round_label.text = "Round 1 | Planning Mode: Pre-planned Actions Loaded"
	round_label.add_theme_font_size_override("font_size", 14)
	round_label.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
	header_hbox.add_child(round_label)

	# 2. 10-Block Timeline Bar
	var timeline_panel := PanelContainer.new()
	var timeline_style := StyleBoxFlat.new()
	timeline_style.bg_color = Color(0.1, 0.13, 0.18, 0.9)
	timeline_style.set_corner_radius_all(8)
	timeline_style.content_margin_left = 12
	timeline_style.content_margin_right = 12
	timeline_style.content_margin_top = 8
	timeline_style.content_margin_bottom = 8
	timeline_panel.add_theme_stylebox_override("panel", timeline_style)
	main_vbox.add_child(timeline_panel)

	var timeline_hbox := HBoxContainer.new()
	timeline_hbox.add_theme_constant_override("separation", 8)
	timeline_panel.add_child(timeline_hbox)

	var timeline_title := Label.new()
	timeline_title.text = "TIMELINE\n(10 BLOCKS):"
	timeline_title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	timeline_title.add_theme_font_size_override("font_size", 11)
	timeline_title.add_theme_color_override("font_color", Color(0.6, 0.7, 0.8))
	timeline_hbox.add_child(timeline_title)

	timeline_blocks.clear()
	timeline_labels.clear()

	for b: int in range(1, 11):
		var block_box := PanelContainer.new()
		block_box.custom_minimum_size = Vector2(98, 48)
		var b_style := StyleBoxFlat.new()
		b_style.bg_color = Color(0.14, 0.18, 0.24, 0.8)
		b_style.border_color = Color(0.25, 0.32, 0.42, 1.0)
		b_style.set_border_width_all(1)
		b_style.set_corner_radius_all(6)
		block_box.add_theme_stylebox_override("panel", b_style)

		var b_vbox := VBoxContainer.new()
		b_vbox.alignment = BoxContainer.ALIGNMENT_CENTER
		b_vbox.add_theme_constant_override("separation", 2)
		block_box.add_child(b_vbox)

		var num_lbl := Label.new()
		num_lbl.text = "BLOCK %d" % b
		num_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		num_lbl.add_theme_font_size_override("font_size", 11)
		num_lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
		b_vbox.add_child(num_lbl)

		var desc_lbl := Label.new()
		desc_lbl.text = "--"
		desc_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		desc_lbl.add_theme_font_size_override("font_size", 9)
		desc_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		b_vbox.add_child(desc_lbl)

		timeline_hbox.add_child(block_box)
		timeline_blocks.append(block_box)
		timeline_labels.append(desc_lbl)

	# 3. Main Center Area (Grid on Left, Roster & Log on Right)
	var content_hbox := HBoxContainer.new()
	content_hbox.add_theme_constant_override("separation", 24)
	content_hbox.size_flags_vertical = Control.SIZE_EXPAND_FILL
	main_vbox.add_child(content_hbox)

	# Left Column: Combat Grid Display
	var grid_vbox := VBoxContainer.new()
	grid_vbox.add_theme_constant_override("separation", 10)
	content_hbox.add_child(grid_vbox)

	var grid_header := Label.new()
	grid_header.text = "9x9 COMBAT GRID (Closed Arena - Exit at (8, 4))"
	grid_header.add_theme_font_size_override("font_size", 14)
	grid_header.add_theme_color_override("font_color", Color(0.8, 0.9, 1.0))
	grid_vbox.add_child(grid_header)

	var grid_frame := PanelContainer.new()
	var frame_style := StyleBoxFlat.new()
	frame_style.bg_color = Color(0.08, 0.1, 0.14, 0.9)
	frame_style.border_color = Color(0.2, 0.26, 0.35, 1.0)
	frame_style.set_border_width_all(2)
	frame_style.set_corner_radius_all(8)
	frame_style.content_margin_left = 6
	frame_style.content_margin_right = 6
	frame_style.content_margin_top = 6
	frame_style.content_margin_bottom = 6
	grid_frame.add_theme_stylebox_override("panel", frame_style)
	grid_vbox.add_child(grid_frame)

	# Container holding both background tile grid and overlay units
	var grid_anchor := Control.new()
	grid_anchor.custom_minimum_size = Vector2(TILE_SIZE * GRID_SIZE, TILE_SIZE * GRID_SIZE)
	grid_frame.add_child(grid_anchor)

	var grid_container := GridContainer.new()
	grid_container.columns = GRID_SIZE
	grid_container.add_theme_constant_override("h_separation", 0)
	grid_container.add_theme_constant_override("v_separation", 0)
	grid_anchor.add_child(grid_container)

	grid_tile_nodes.clear()
	for y: int in range(GRID_SIZE):
		for x: int in range(GRID_SIZE):
			var coord := Vector2i(x, y)
			var tile_rect := TextureRect.new()
			tile_rect.custom_minimum_size = Vector2(TILE_SIZE, TILE_SIZE)
			tile_rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
			tile_rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED

			if coord == Vector2i(4, 4):
				tile_rect.texture = tex_center
			elif coord == Vector2i(8, 4):
				tile_rect.texture = tex_exit
			else:
				tile_rect.texture = tex_floor

			grid_container.add_child(tile_rect)
			grid_tile_nodes[coord] = tile_rect

	# Unit Token Container overlay
	var unit_overlay := Control.new()
	unit_overlay.name = "UnitOverlay"
	unit_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	grid_anchor.add_child(unit_overlay)

	# Control Buttons Bar below grid
	var controls_hbox := HBoxContainer.new()
	controls_hbox.add_theme_constant_override("separation", 10)
	grid_vbox.add_child(controls_hbox)

	btn_step = Button.new()
	btn_step.text = "Step Block (Next)"
	btn_step.custom_minimum_size = Vector2(140, 36)
	btn_step.pressed.connect(_on_step_block_pressed)
	controls_hbox.add_child(btn_step)

	btn_play = Button.new()
	btn_play.text = "Auto Play"
	btn_play.custom_minimum_size = Vector2(120, 36)
	btn_play.pressed.connect(_on_play_toggle_pressed)
	controls_hbox.add_child(btn_play)

	btn_reset = Button.new()
	btn_reset.text = "Reset Simulation"
	btn_reset.custom_minimum_size = Vector2(130, 36)
	btn_reset.pressed.connect(_on_reset_pressed)
	controls_hbox.add_child(btn_reset)

	status_label = Label.new()
	status_label.text = "Ready to step or play simulation."
	status_label.add_theme_font_size_override("font_size", 12)
	status_label.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
	grid_vbox.add_child(status_label)

	# Right Column: Combatant Roster & Battle Log
	var right_vbox := VBoxContainer.new()
	right_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_vbox.add_theme_constant_override("separation", 12)
	content_hbox.add_child(right_vbox)

	var roster_label := Label.new()
	roster_label.text = "ACTIVE COMBATANTS"
	roster_label.add_theme_font_size_override("font_size", 14)
	roster_label.add_theme_color_override("font_color", Color(0.8, 0.9, 1.0))
	right_vbox.add_child(roster_label)

	var roster_vbox := VBoxContainer.new()
	roster_vbox.name = "RosterCardsContainer"
	roster_vbox.add_theme_constant_override("separation", 6)
	right_vbox.add_child(roster_vbox)

	var log_label := Label.new()
	log_label.text = "TACTICAL RESOLUTION FEED"
	log_label.add_theme_font_size_override("font_size", 14)
	log_label.add_theme_color_override("font_color", Color(0.8, 0.9, 1.0))
	right_vbox.add_child(log_label)

	var log_panel := PanelContainer.new()
	log_panel.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var log_style := StyleBoxFlat.new()
	log_style.bg_color = Color(0.08, 0.1, 0.14, 0.95)
	log_style.border_color = Color(0.2, 0.26, 0.35, 1.0)
	log_style.set_border_width_all(1)
	log_style.set_corner_radius_all(6)
	log_style.content_margin_left = 12
	log_style.content_margin_right = 12
	log_style.content_margin_top = 8
	log_style.content_margin_bottom = 8
	log_panel.add_theme_stylebox_override("panel", log_style)
	right_vbox.add_child(log_panel)

	log_rich_text = RichTextLabel.new()
	log_rich_text.bbcode_enabled = true
	log_rich_text.scroll_following = true
	log_rich_text.set_anchors_preset(Control.PRESET_FULL_RECT)
	log_panel.add_child(log_rich_text)

	# Auto-play timer
	auto_play_timer = Timer.new()
	auto_play_timer.wait_time = 0.8
	auto_play_timer.one_shot = false
	auto_play_timer.timeout.connect(_on_auto_play_tick)
	add_child(auto_play_timer)


# ==============================================================================
# SECTION 2: SIMULATION SETUP WITH PRE-PLANNED COMMANDS
# ==============================================================================

func _init_interactive_simulation() -> void:
	current_sim_block = 1
	sim_finished = false
	is_auto_playing = false
	auto_play_timer.stop()
	btn_play.text = "Auto Play"

	# Clear previous tokens
	var overlay: Control = ui_canvas.find_child("UnitOverlay", true, false) as Control
	if overlay:
		for child in overlay.get_children():
			child.queue_free()
	unit_token_nodes.clear()

	var roster_container: Control = ui_canvas.find_child("RosterCardsContainer", true, false) as Control
	if roster_container:
		for child in roster_container.get_children():
			child.queue_free()
	roster_cards.clear()

	log_rich_text.clear()
	_append_log("[color=cyan]>>> New 10-Block Tactical Battle Initialized (Pre-planned Actions Loaded).[/color]")

	# 1. Initialize 9x9 Closed Arena with designated Exit at (8, 4)
	var exit_coord := Vector2i(8, 4)
	sim_grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [exit_coord])
	sim_resolver = CombatResolver.new()
	sim_resolver.grid = sim_grid

	# 2. Combatants
	var warrior := Combatant.new("Warrior", 100, 15, 1)
	var ranger := Combatant.new("Ranger", 80, 14, 2)
	var brawler := Combatant.new("Enemy Brawler", 45, 12, 1)
	var scout := Combatant.new("Fleeing Scout", 30, 16, 1)

	sim_allies = [warrior, ranger]
	sim_enemies = [brawler, scout]

	sim_resolver.register_combatant(warrior)
	sim_resolver.register_combatant(ranger)
	sim_resolver.register_combatant(brawler)
	sim_resolver.register_combatant(scout)

	# 3. Center-ring deployment
	sim_grid.deploy_combatants(sim_allies, sim_enemies)

	# 4. Schedule Pre-Planned Commands for Round 1
	# Warrior: Move Up (4, 4) on Block 1; Attack Brawler on Block 2-3
	sim_resolver.schedule_action(warrior, MoveCommand.new(warrior, sim_grid, Vector2i(0, -1), 1), 1)
	sim_resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 25, 2, 0, sim_grid), 2)

	# Ranger: Move Up to (4, 5) on Block 4; Ranged Attack Brawler on Block 5-6
	sim_resolver.schedule_action(ranger, MoveCommand.new(ranger, sim_grid, Vector2i(0, -1), 1), 4)
	sim_resolver.schedule_action(ranger, AttackCommand.new(ranger, brawler, 25, 2, 0, sim_grid), 5)

	# Fleeing Scout: Move East (6, 4) on Block 1; Sprint East (8, 4) on Block 2-3 (Reaches Exit and Withdraws!)
	sim_resolver.schedule_action(scout, MoveCommand.new(scout, sim_grid, Vector2i(1, 0), 1), 1)
	sim_resolver.schedule_action(scout, MoveCommand.new(scout, sim_grid, Vector2i(1, 0), 2, 2), 2)
	sim_resolver.schedule_action(scout, AttackCommand.new(scout, warrior, 10, 2, 0, sim_grid), 4)

	# 5. Build visual unit tokens and roster cards
	for ally in sim_allies:
		_create_unit_token(ally, tex_warrior if ally.name == "Warrior" else tex_ranger)
		_create_roster_card(ally, Color(0.2, 0.5, 0.9))
	for enemy in sim_enemies:
		_create_unit_token(enemy, tex_brawler if enemy.name == "Enemy Brawler" else tex_scout)
		_create_roster_card(enemy, Color(0.9, 0.3, 0.3))

	# Update timeline labels with pre-planned actions
	_update_timeline_labels()
	_highlight_timeline_block(1)
	status_label.text = "Block 1 ready. Click 'Step Block' or 'Auto Play' to resolve."


func _create_unit_token(combatant: Combatant, texture: Texture2D) -> void:
	var overlay: Control = ui_canvas.find_child("UnitOverlay", true, false) as Control
	if not overlay:
		return

	var token_ctrl := Control.new()
	token_ctrl.custom_minimum_size = Vector2(TILE_SIZE, TILE_SIZE)

	var icon := TextureRect.new()
	icon.texture = texture
	icon.custom_minimum_size = Vector2(TILE_SIZE - 4, TILE_SIZE - 4)
	icon.position = Vector2(2, 2)
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	token_ctrl.add_child(icon)

	# Name badge
	var name_lbl := Label.new()
	name_lbl.text = combatant.name
	name_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	name_lbl.position = Vector2(-20, -14)
	name_lbl.custom_minimum_size = Vector2(TILE_SIZE + 40, 14)
	name_lbl.add_theme_font_size_override("font_size", 9)
	name_lbl.add_theme_color_override("font_color", Color(1, 1, 1, 0.9))
	token_ctrl.add_child(name_lbl)

	overlay.add_child(token_ctrl)
	unit_token_nodes[combatant] = token_ctrl

	# Set initial grid pixel position
	var pos: Vector2i = sim_grid.get_position(combatant)
	token_ctrl.position = Vector2(pos.x * TILE_SIZE, pos.y * TILE_SIZE)


func _create_roster_card(combatant: Combatant, team_color: Color) -> void:
	var roster_container: Control = ui_canvas.find_child("RosterCardsContainer", true, false) as Control
	if not roster_container:
		return

	var card := PanelContainer.new()
	var card_style := StyleBoxFlat.new()
	card_style.bg_color = Color(0.12, 0.15, 0.2, 0.9)
	card_style.border_color = team_color
	card_style.set_border_width_all(1)
	card_style.set_corner_radius_all(4)
	card_style.content_margin_left = 8
	card_style.content_margin_right = 8
	card_style.content_margin_top = 4
	card_style.content_margin_bottom = 4
	card.add_theme_stylebox_override("panel", card_style)

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 10)
	card.add_child(hbox)

	var name_lbl := Label.new()
	name_lbl.text = combatant.name
	name_lbl.custom_minimum_size = Vector2(110, 0)
	name_lbl.add_theme_font_size_override("font_size", 12)
	name_lbl.add_theme_color_override("font_color", team_color.lightened(0.3))
	hbox.add_child(name_lbl)

	var hp_bar := ProgressBar.new()
	hp_bar.custom_minimum_size = Vector2(90, 14)
	hp_bar.max_value = combatant.max_hp
	hp_bar.value = combatant.hp
	hp_bar.show_percentage = false
	hbox.add_child(hp_bar)

	var hp_lbl := Label.new()
	hp_lbl.name = "HpLabel"
	hp_lbl.text = "%d/%d" % [combatant.hp, combatant.max_hp]
	hp_lbl.custom_minimum_size = Vector2(60, 0)
	hp_lbl.add_theme_font_size_override("font_size", 11)
	hbox.add_child(hp_lbl)

	var stance_lbl := Label.new()
	stance_lbl.name = "StanceLabel"
	stance_lbl.text = CombatTypes.stance_to_string(combatant.current_stance)
	stance_lbl.custom_minimum_size = Vector2(70, 0)
	stance_lbl.add_theme_font_size_override("font_size", 11)
	stance_lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
	hbox.add_child(stance_lbl)

	var pos_lbl := Label.new()
	pos_lbl.name = "PosLabel"
	pos_lbl.text = "Pos: %s" % str(sim_grid.get_position(combatant))
	pos_lbl.add_theme_font_size_override("font_size", 11)
	pos_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
	hbox.add_child(pos_lbl)

	roster_container.add_child(card)
	roster_cards[combatant] = card


# ==============================================================================
# SECTION 3: STEPPING & VISUAL PLAYBACK
# ==============================================================================

func _step_simulation() -> void:
	if sim_finished:
		return

	_highlight_timeline_block(current_sim_block)
	_append_log("[color=yellow]>>> Executing Block %d...[/color]" % current_sim_block)

	# Execute the block in the domain resolver
	sim_resolver.execute_block(current_sim_block)

	# Update visual representations
	_sync_visuals_after_block()

	# Check combat conclusion
	var is_over: bool = sim_resolver.is_combat_resolved(sim_allies, sim_enemies)
	if is_over:
		sim_finished = true
		auto_play_timer.stop()
		btn_play.disabled = true
		btn_step.disabled = true
		status_label.text = "BATTLE CONCLUDED! All hostiles resolved."
		_append_log("[color=green][b]>>> BATTLE CONCLUDED: All hostiles defeated or safely withdrawn![/b][/color]")
		return

	current_sim_block += 1
	if current_sim_block > CombatResolver.TOTAL_BLOCKS:
		sim_resolver.finish_round()
		sim_finished = true
		auto_play_timer.stop()
		btn_play.disabled = true
		btn_step.disabled = true
		status_label.text = "Round 1 complete."
		_append_log("[color=cyan]>>> Round 1 concluded.[/color]")
	else:
		_highlight_timeline_block(current_sim_block)
		status_label.text = "Ready for Block %d." % current_sim_block


func _sync_visuals_after_block() -> void:
	# 1. Update unit token positions with smooth Tween animations
	for combatant: Combatant in unit_token_nodes.keys():
		var token: Control = unit_token_nodes[combatant] as Control
		if not combatant.is_active():
			if combatant.is_withdrawn:
				# Fade out token
				var tw := create_tween()
				tw.tween_property(token, "modulate:a", 0.0, 0.4)
			elif not combatant.is_alive():
				token.modulate = Color(0.4, 0.4, 0.4, 0.6)
		else:
			var target_coord: Vector2i = sim_grid.get_position(combatant)
			var target_px := Vector2(target_coord.x * TILE_SIZE, target_coord.y * TILE_SIZE)
			if token.position != target_px:
				var tw := create_tween()
				tw.tween_property(token, "position", target_px, 0.25).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)

	# 2. Update roster cards
	for combatant: Combatant in roster_cards.keys():
		var card: Control = roster_cards[combatant] as Control
		var hp_bar: ProgressBar = card.find_child("ProgressBar", true, false) as ProgressBar
		var hp_lbl: Label = card.find_child("HpLabel", true, false) as Label
		var stance_lbl: Label = card.find_child("StanceLabel", true, false) as Label
		var pos_lbl: Label = card.find_child("PosLabel", true, false) as Label

		if hp_bar:
			hp_bar.value = combatant.hp
		if hp_lbl:
			hp_lbl.text = "%d/%d" % [combatant.hp, combatant.max_hp]
		if stance_lbl:
			if combatant.is_withdrawn:
				stance_lbl.text = "WITHDRAWN"
				stance_lbl.add_theme_color_override("font_color", Color(0.9, 0.7, 0.2))
			elif not combatant.is_alive():
				stance_lbl.text = "DEFEATED"
				stance_lbl.add_theme_color_override("font_color", Color(0.7, 0.3, 0.3))
			else:
				stance_lbl.text = CombatTypes.stance_to_string(combatant.current_stance)
		if pos_lbl:
			if combatant.is_active():
				pos_lbl.text = "Pos: %s" % str(sim_grid.get_position(combatant))
			else:
				pos_lbl.text = "Off Grid"


func _update_timeline_labels() -> void:
	for b: int in range(1, 11):
		var actions: Array[CombatResolver.ScheduledAction] = sim_resolver.get_actions_at_block(b)
		var label: Label = timeline_labels[b - 1]
		if actions.is_empty():
			label.text = "--"
		else:
			var names: Array[String] = []
			for a in actions:
				names.append(a.combatant.name.substr(0, 3) + ":" + a.command.command_name.substr(0, 4))
			label.text = " / ".join(names)


func _highlight_timeline_block(active_block: int) -> void:
	for i: int in range(timeline_blocks.size()):
		var panel: PanelContainer = timeline_blocks[i]
		var b_style := StyleBoxFlat.new()
		b_style.set_corner_radius_all(6)
		if i + 1 == active_block:
			b_style.bg_color = Color(0.18, 0.35, 0.55, 1.0)
			b_style.border_color = Color(0.4, 0.8, 1.0, 1.0)
			b_style.set_border_width_all(2)
		else:
			b_style.bg_color = Color(0.14, 0.18, 0.24, 0.8)
			b_style.border_color = Color(0.25, 0.32, 0.42, 1.0)
			b_style.set_border_width_all(1)
		panel.add_theme_stylebox_override("panel", b_style)


func _append_log(text: String) -> void:
	if log_rich_text:
		log_rich_text.append_text(text + "\n")


func _on_step_block_pressed() -> void:
	_step_simulation()


func _on_play_toggle_pressed() -> void:
	if is_auto_playing:
		is_auto_playing = false
		auto_play_timer.stop()
		btn_play.text = "Auto Play"
	else:
		is_auto_playing = true
		btn_play.text = "Pause"
		auto_play_timer.start()


func _on_auto_play_tick() -> void:
	if not sim_finished:
		_step_simulation()
	else:
		auto_play_timer.stop()


func _on_reset_pressed() -> void:
	btn_play.disabled = false
	btn_step.disabled = false
	_init_interactive_simulation()


# ==============================================================================
# SECTION 4: HEADLESS AUTOMATED INTEGRATION TESTS
# ==============================================================================

func _run_warrior_vs_3_goblins_battle() -> bool:
	var resolver := CombatResolver.new()

	var warrior := Combatant.new("Warrior", 120, 15)
	var scout := Combatant.new("Goblin Scout", 35, 18)
	var brawler := Combatant.new("Goblin Brawler", 45, 12)
	var skirmisher := Combatant.new("Goblin Skirmisher", 40, 16)

	resolver.register_combatant(warrior)
	resolver.register_combatant(scout)
	resolver.register_combatant(brawler)
	resolver.register_combatant(skirmisher)

	# Round 1 Scheduling
	resolver.schedule_action(warrior, TargetCommand.new(warrior, scout), 1)
	resolver.schedule_action(warrior, FocusCommand.new(warrior, 1.5, 2), 2)
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 4)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, scout, 30), 5)
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 7)
	resolver.schedule_action(warrior, DefendCommand.new(warrior, 0.5), 8)
	resolver.schedule_action(warrior, HoldStanceCommand.new(warrior), 10)

	resolver.schedule_action(scout, DefendCommand.new(scout, 0.5), 1)
	resolver.schedule_action(scout, AttackCommand.new(scout, warrior, 12), 3)
	resolver.schedule_action(scout, AttackCommand.new(scout, warrior, 12), 5)

	resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5), 1)
	resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5), 3)
	resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5), 5)
	resolver.schedule_action(brawler, EnterNeutralCommand.new(brawler), 7)
	resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 14), 8)
	resolver.schedule_action(brawler, HoldStanceCommand.new(brawler), 10)

	resolver.schedule_action(skirmisher, TargetCommand.new(skirmisher, warrior), 1)
	resolver.schedule_action(skirmisher, AttackCommand.new(skirmisher, warrior, 10), 2)
	resolver.schedule_action(skirmisher, EnterNeutralCommand.new(skirmisher), 4)
	resolver.schedule_action(skirmisher, DefendCommand.new(skirmisher, 0.5), 5)
	resolver.schedule_action(skirmisher, EnterNeutralCommand.new(skirmisher), 7)
	resolver.schedule_action(skirmisher, AttackCommand.new(skirmisher, warrior, 12), 8)
	resolver.schedule_action(skirmisher, HoldStanceCommand.new(skirmisher), 10)

	resolver.resolve_round()

	var warrior_hp_r1: int = warrior.hp
	var scout_defeated_r1: bool = scout.hp == 0
	var warrior_hp_r1_ok: bool = warrior_hp_r1 == 73
	var focus_buff_persisted: bool = warrior.attack_boost_multiplier == 1.5 and warrior.attack_boost_rounds_remaining == 1
	var stance_persisted: bool = warrior.current_stance == CombatTypes.StanceType.DEFENSIVE

	# Round 2 Scheduling
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 1)
	resolver.schedule_action(warrior, TargetCommand.new(warrior, brawler), 2)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 30), 3)
	resolver.schedule_action(warrior, EnterNeutralCommand.new(warrior), 5)
	resolver.schedule_action(warrior, TargetCommand.new(warrior, skirmisher), 6)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, skirmisher, 30), 7)
	resolver.schedule_action(warrior, DefendCommand.new(warrior, 0.5), 9)

	resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 10), 1)
	resolver.schedule_action(skirmisher, DefendCommand.new(skirmisher, 0.5), 1)

	resolver.resolve_round()

	var warrior_hp_r2: int = warrior.hp
	var brawler_defeated: bool = brawler.hp == 0
	var skirmisher_defeated: bool = skirmisher.hp == 0
	var warrior_survived: bool = warrior.is_alive() and warrior_hp_r2 == 63
	var all_goblins_down: bool = scout_defeated_r1 and brawler_defeated and skirmisher_defeated
	var buff_expired_after_r2: bool = warrior.attack_boost_multiplier == 1.0 and warrior.attack_boost_rounds_remaining == 0

	print("\n==========================================")
	print("=== FINAL INTEGRATION TEST ASSERTIONS ===")
	print("==========================================")
	print("  [R1] Goblin Scout defeated (0 HP): %s" % ("PASS" if scout_defeated_r1 else "FAIL"))
	print("  [R1] Warrior mitigated damage (HP == 73): %s (HP: %d)" % [("PASS" if warrior_hp_r1_ok else "FAIL"), warrior_hp_r1])
	print("  [R1] Defensive Stance carried to R2: %s" % ("PASS" if stance_persisted else "FAIL"))
	print("  [R1] Focus 2-round buff active into R2: %s" % ("PASS" if focus_buff_persisted else "FAIL"))
	print("  [R2] Goblin Brawler defeated (0 HP): %s" % ("PASS" if brawler_defeated else "FAIL"))
	print("  [R2] Goblin Skirmisher defeated (0 HP): %s" % ("PASS" if skirmisher_defeated else "FAIL"))
	print("  [R2] Warrior survived battle (HP == 63): %s (HP: %d/120)" % [("PASS" if warrior_survived else "FAIL"), warrior_hp_r2])
	print("  [R2] All 3 Goblins eliminated: %s" % ("PASS" if all_goblins_down else "FAIL"))
	print("  [R2] Focus 2-round buff expired cleanly: %s" % ("PASS" if buff_expired_after_r2 else "FAIL"))

	warrior.clear_target()
	scout.clear_target()
	brawler.clear_target()
	skirmisher.clear_target()
	resolver.registered_combatants.clear()

	return (scout_defeated_r1 and warrior_hp_r1_ok and stance_persisted and
		focus_buff_persisted and brawler_defeated and skirmisher_defeated and
		warrior_survived and all_goblins_down and buff_expired_after_r2)


func _run_combat_grid_and_withdrawal_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING COMBAT GRID & WITHDRAWAL TEST (9x9)    ===")
	print("=======================================================")

	var exit_coord := Vector2i(8, 4)
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [exit_coord])
	var resolver := CombatResolver.new()
	resolver.grid = grid

	var warrior := Combatant.new("Warrior", 100, 15, 1)
	var ranger := Combatant.new("Ranger", 80, 14, 2)
	var brawler := Combatant.new("Enemy Brawler", 45, 12, 1)
	var scout := Combatant.new("Fleeing Scout", 30, 16, 1)

	var allies: Array[Combatant] = [warrior, ranger]
	var enemies: Array[Combatant] = [brawler, scout]

	resolver.register_combatant(warrior)
	resolver.register_combatant(ranger)
	resolver.register_combatant(brawler)
	resolver.register_combatant(scout)

	grid.deploy_combatants(allies, enemies)

	var warrior_pos: Vector2i = grid.get_position(warrior)
	var ranger_pos: Vector2i = grid.get_position(ranger)
	var brawler_pos: Vector2i = grid.get_position(brawler)
	var scout_pos: Vector2i = grid.get_position(scout)

	var warrior_center_ok: bool = grid.get_distance(warrior_pos, CombatGrid.CENTER) <= warrior.attack_range
	var ranger_center_ok: bool = grid.get_distance(ranger_pos, CombatGrid.CENTER) <= ranger.attack_range
	var brawler_center_ok: bool = grid.get_distance(brawler_pos, CombatGrid.CENTER) <= brawler.attack_range
	var scout_center_ok: bool = grid.get_distance(scout_pos, CombatGrid.CENTER) <= scout.attack_range

	var deployment_ok: bool = warrior_center_ok and ranger_center_ok and brawler_center_ok and scout_center_ok
	var initial_out_of_range: bool = not grid.can_target(warrior, brawler, warrior.attack_range)

	resolver.schedule_action(warrior, MoveCommand.new(warrior, grid, Vector2i(0, -1), 1), 1)
	resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 25, 2, 0, grid), 2)

	resolver.schedule_action(ranger, MoveCommand.new(ranger, grid, Vector2i(0, -1), 1), 4)
	resolver.schedule_action(ranger, AttackCommand.new(ranger, brawler, 25, 2, 0, grid), 5)

	resolver.schedule_action(scout, MoveCommand.new(scout, grid, Vector2i(1, 0), 1), 1)
	resolver.schedule_action(scout, MoveCommand.new(scout, grid, Vector2i(1, 0), 2, 2), 2)
	resolver.schedule_action(scout, AttackCommand.new(scout, warrior, 10, 2, 0, grid), 4)

	resolver.resolve_round()

	var scout_withdrawn_ok: bool = scout.is_withdrawn and not scout.is_active() and grid.withdrawn_combatants.has(scout)
	var scout_removed_from_grid: bool = not grid.has_combatant(scout)
	var brawler_defeated_ok: bool = brawler.hp == 0 and not brawler.is_active()
	var combat_ended_ok: bool = resolver.is_combat_resolved(allies, enemies)
	var warrior_pos_after: Vector2i = grid.get_position(warrior)
	var warrior_moved_ok: bool = warrior_pos_after == Vector2i(4, 4)

	print("\n================================================")
	print("=== COMBAT GRID & WITHDRAWAL TEST ASSERTIONS ===")
	print("================================================")
	print("  [Grid] Center-ring deployment valid: %s" % ("PASS" if deployment_ok else "FAIL"))
	print("  [Grid] Spatial range gate verified: %s" % ("PASS" if initial_out_of_range else "FAIL"))
	print("  [Grid] Warrior stepped to (4, 4): %s (%s)" % [("PASS" if warrior_moved_ok else "FAIL"), str(warrior_pos_after)])
	print("  [Grid] Fleeing Scout partially withdrawn at (8, 4): %s" % ("PASS" if scout_withdrawn_ok else "FAIL"))
	print("  [Grid] Scout removed from active grid: %s" % ("PASS" if scout_removed_from_grid else "FAIL"))
	print("  [Grid] Brawler eliminated by ranged team: %s" % ("PASS" if brawler_defeated_ok else "FAIL"))
	print("  [Grid] Combat terminated (all hostiles resolved): %s" % ("PASS" if combat_ended_ok else "FAIL"))

	warrior.clear_target()
	ranger.clear_target()
	brawler.clear_target()
	scout.clear_target()
	resolver.registered_combatants.clear()

	return (deployment_ok and initial_out_of_range and warrior_moved_ok and 
		scout_withdrawn_ok and scout_removed_from_grid and brawler_defeated_ok and combat_ended_ok)