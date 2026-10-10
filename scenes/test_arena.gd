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

# Character Placeholder Full Sprite Textures
var tex_sprite_warrior: Texture2D
var tex_sprite_ranger: Texture2D
var tex_sprite_brawler: Texture2D
var tex_sprite_scout: Texture2D

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

	# Load placeholder sprite assets
	tex_sprite_warrior = _load_svg_texture("res://assets/sprite_warrior.svg")
	tex_sprite_ranger = _load_svg_texture("res://assets/sprite_ranger.svg")
	tex_sprite_brawler = _load_svg_texture("res://assets/sprite_brawler.svg")
	tex_sprite_scout = _load_svg_texture("res://assets/sprite_scout.svg")

const TILE_SIZE: int = 54
const GRID_SIZE: int = 9

# Interactive UI References
var ui_canvas: CanvasLayer
var timeline_blocks: Array[PanelContainer] = []
var timeline_labels: Array[Label] = []
var grid_tile_nodes: Dictionary = {}  # Vector2i -> TextureRect
var unit_token_nodes: Dictionary = {} # Combatant -> Control
var roster_cards: Dictionary = {}     # Combatant -> PanelContainer
var party_cards_container: VBoxContainer
var enemy_cards_container: VBoxContainer
var log_rich_text: RichTextLabel
var round_label: Label
var status_label: Label
var btn_step: Button
var btn_play: Button
var btn_reset: Button
var btn_toggle_timeline: Button
var auto_play_timer: Timer

# Selection and Inspector State
var selected_combatant: Combatant = null
var timeline_mode_all: bool = false
var timeline_header_lbl: Label

# Left Popup (Unit Inspector & Action Setup Dock)
var inspector_avatar: TextureRect
var inspector_name_lbl: Label
var inspector_team_lbl: Label
var inspector_hp_bar: ProgressBar
var inspector_hp_lbl: Label
var inspector_sp_bar: ProgressBar
var inspector_sp_lbl: Label
var inspector_stance_lbl: Label
var inspector_speed_lbl: Label
var inspector_range_lbl: Label
var inspector_pos_lbl: Label
var inspector_target_lbl: Label
var inspector_buffs_lbl: Label

# Right Popup (Tactical Forecast & Arena Intel)
var forecast_target_lbl: Label
var forecast_dist_lbl: Label
var forecast_range_badge: Label
var forecast_stance_lbl: Label
var forecast_dmg_lbl: Label
var intel_center_lbl: Label
var intel_exit_lbl: Label
var intel_turn_lbl: Label

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
	var test3_passed: bool = _run_resolution_feed_logging_test()
	var all_tests_passed: bool = test1_passed and test2_passed and test3_passed

	var exit_code: int = 0 if all_tests_passed else 1

	if DisplayServer.get_name() == "headless":
		print("Execution finished headlessly. Exiting with code: %d\n" % exit_code)
		get_tree().quit(exit_code)
	else:
		print("Running in windowed mode. Initializing interactive tactical UI...\n")
		_setup_interactive_ui()
		_init_interactive_simulation()
		get_tree().create_timer(1.2).timeout.connect(func() -> void:
			if not sim_finished and not is_auto_playing:
				_on_play_toggle_pressed()
		)


# ==============================================================================
# SECTION 1: INTERACTIVE UI CREATION & REPOSITIONED LAYOUT
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
	root_margin.add_theme_constant_override("margin_left", 14)
	root_margin.add_theme_constant_override("margin_right", 14)
	root_margin.add_theme_constant_override("margin_top", 8)
	root_margin.add_theme_constant_override("margin_bottom", 8)
	ui_canvas.add_child(root_margin)

	var main_vbox := VBoxContainer.new()
	main_vbox.add_theme_constant_override("separation", 8)
	root_margin.add_child(main_vbox)

	# --------------------------------------------------------------------------
	# TOP ROW: PARTY STATUS (Left) | BATTLE LOG (Center) | ENEMY STATUS (Right)
	# --------------------------------------------------------------------------
	var top_hbox := HBoxContainer.new()
	top_hbox.custom_minimum_size = Vector2(0, 115)
	top_hbox.add_theme_constant_override("separation", 10)
	main_vbox.add_child(top_hbox)

	# Top-Left: Allied Party Status Panel (Expands to match left column)
	var party_panel := PanelContainer.new()
	party_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	party_panel.custom_minimum_size = Vector2(340, 115)
	var party_box_style := StyleBoxFlat.new()
	party_box_style.bg_color = Color(0.08, 0.11, 0.16, 0.95)
	party_box_style.border_color = Color(0.2, 0.38, 0.65, 0.8)
	party_box_style.set_border_width_all(1)
	party_box_style.set_corner_radius_all(6)
	party_box_style.content_margin_left = 6
	party_box_style.content_margin_right = 6
	party_box_style.content_margin_top = 4
	party_box_style.content_margin_bottom = 4
	party_panel.add_theme_stylebox_override("panel", party_box_style)
	top_hbox.add_child(party_panel)

	var party_vbox := VBoxContainer.new()
	party_vbox.add_theme_constant_override("separation", 3)
	party_panel.add_child(party_vbox)

	var party_hdr_hbox := HBoxContainer.new()
	party_vbox.add_child(party_hdr_hbox)
	var party_title := Label.new()
	party_title.text = "⚔ PARTY MEMBER STATUS"
	party_title.add_theme_font_size_override("font_size", 11)
	party_title.add_theme_color_override("font_color", Color(0.38, 0.72, 1.0))
	party_hdr_hbox.add_child(party_title)

	var party_hint := Label.new()
	party_hint.text = "(Click to select)"
	party_hint.add_theme_font_size_override("font_size", 9)
	party_hint.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
	party_hdr_hbox.add_spacer(false)
	party_hdr_hbox.add_child(party_hint)

	party_cards_container = VBoxContainer.new()
	party_cards_container.name = "PartyCardsContainer"
	party_cards_container.add_theme_constant_override("separation", 3)
	party_vbox.add_child(party_cards_container)

	# Top-Center: Battle Log (Matches center grid width of 496px)
	var log_panel := PanelContainer.new()
	log_panel.custom_minimum_size = Vector2(496, 115)
	log_panel.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	var log_style := StyleBoxFlat.new()
	log_style.bg_color = Color(0.06, 0.08, 0.12, 0.95)
	log_style.border_color = Color(0.25, 0.32, 0.44, 0.8)
	log_style.set_border_width_all(1)
	log_style.set_corner_radius_all(6)
	log_style.content_margin_left = 8
	log_style.content_margin_right = 8
	log_style.content_margin_top = 4
	log_style.content_margin_bottom = 4
	log_panel.add_theme_stylebox_override("panel", log_style)
	top_hbox.add_child(log_panel)

	var log_vbox := VBoxContainer.new()
	log_vbox.add_theme_constant_override("separation", 2)
	log_panel.add_child(log_vbox)

	var log_hdr_hbox := HBoxContainer.new()
	log_vbox.add_child(log_hdr_hbox)
	var log_title := Label.new()
	log_title.text = "📜 BATTLE LOG"
	log_title.add_theme_font_size_override("font_size", 11)
	log_title.add_theme_color_override("font_color", Color(0.96, 0.82, 0.35))
	log_hdr_hbox.add_child(log_title)

	var log_sub := Label.new()
	log_sub.text = "Live Block Feed | Scroll to review history"
	log_sub.add_theme_font_size_override("font_size", 9)
	log_sub.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
	log_hdr_hbox.add_spacer(false)
	log_hdr_hbox.add_child(log_sub)

	log_rich_text = RichTextLabel.new()
	log_rich_text.bbcode_enabled = true
	log_rich_text.scroll_following = true
	log_rich_text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	log_rich_text.scroll_active = true
	log_rich_text.size_flags_vertical = Control.SIZE_EXPAND_FILL
	log_vbox.add_child(log_rich_text)

	# Top-Right: Enemy Hostiles Status Panel (Expands flush to right edge)
	var enemy_panel := PanelContainer.new()
	enemy_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	enemy_panel.custom_minimum_size = Vector2(340, 115)
	var enemy_box_style := StyleBoxFlat.new()
	enemy_box_style.bg_color = Color(0.12, 0.08, 0.10, 0.95)
	enemy_box_style.border_color = Color(0.65, 0.25, 0.25, 0.8)
	enemy_box_style.set_border_width_all(1)
	enemy_box_style.set_corner_radius_all(6)
	enemy_box_style.content_margin_left = 6
	enemy_box_style.content_margin_right = 6
	enemy_box_style.content_margin_top = 4
	enemy_box_style.content_margin_bottom = 4
	enemy_panel.add_theme_stylebox_override("panel", enemy_box_style)
	top_hbox.add_child(enemy_panel)

	var enemy_vbox := VBoxContainer.new()
	enemy_vbox.add_theme_constant_override("separation", 3)
	enemy_panel.add_child(enemy_vbox)

	var enemy_hdr_hbox := HBoxContainer.new()
	enemy_vbox.add_child(enemy_hdr_hbox)
	var enemy_title := Label.new()
	enemy_title.text = "💀 ENEMY STATUS"
	enemy_title.add_theme_font_size_override("font_size", 11)
	enemy_title.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
	enemy_hdr_hbox.add_child(enemy_title)

	var enemy_hint := Label.new()
	enemy_hint.text = "(Click to inspect)"
	enemy_hint.add_theme_font_size_override("font_size", 9)
	enemy_hint.add_theme_color_override("font_color", Color(0.6, 0.5, 0.5))
	enemy_hdr_hbox.add_spacer(false)
	enemy_hdr_hbox.add_child(enemy_hint)

	enemy_cards_container = VBoxContainer.new()
	enemy_cards_container.name = "EnemyCardsContainer"
	enemy_cards_container.add_theme_constant_override("separation", 3)
	enemy_vbox.add_child(enemy_cards_container)

	# --------------------------------------------------------------------------
	# CENTER ROW: LEFT POPUP (Inspector) | COMBAT GRID (54px) | RIGHT POPUP (Forecast)
	# --------------------------------------------------------------------------
	var center_hbox := HBoxContainer.new()
	center_hbox.add_theme_constant_override("separation", 10)
	center_hbox.size_flags_vertical = Control.SIZE_EXPAND_FILL
	main_vbox.add_child(center_hbox)

	# 1. Left Popup Area: Unit Inspector & Action Setup Dock (Expands to fill left wing)
	var left_popup_panel := PanelContainer.new()
	left_popup_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left_popup_panel.custom_minimum_size = Vector2(340, 488)
	var left_style := StyleBoxFlat.new()
	left_style.bg_color = Color(0.08, 0.10, 0.15, 0.95)
	left_style.border_color = Color(0.2, 0.28, 0.42, 0.8)
	left_style.set_border_width_all(1)
	left_style.set_corner_radius_all(6)
	left_style.content_margin_left = 8
	left_style.content_margin_right = 8
	left_style.content_margin_top = 6
	left_style.content_margin_bottom = 6
	left_popup_panel.add_theme_stylebox_override("panel", left_style)
	center_hbox.add_child(left_popup_panel)

	var left_vbox := VBoxContainer.new()
	left_vbox.add_theme_constant_override("separation", 6)
	left_popup_panel.add_child(left_vbox)

	var inspector_hdr := Label.new()
	inspector_hdr.text = "UNIT INSPECTOR & ACTION SETUP"
	inspector_hdr.add_theme_font_size_override("font_size", 11)
	inspector_hdr.add_theme_color_override("font_color", Color(0.65, 0.8, 1.0))
	left_vbox.add_child(inspector_hdr)

	# Selected Unit Profile Card (2-Block Structure: Left=Sprite+Vitals, Right=Information)
	var profile_card := PanelContainer.new()
	var prof_style := StyleBoxFlat.new()
	prof_style.bg_color = Color(0.12, 0.15, 0.22, 0.8)
	prof_style.border_color = Color(0.3, 0.4, 0.55, 0.6)
	prof_style.set_border_width_all(1)
	prof_style.set_corner_radius_all(6)
	prof_style.content_margin_left = 8
	prof_style.content_margin_right = 8
	prof_style.content_margin_top = 6
	prof_style.content_margin_bottom = 6
	profile_card.add_theme_stylebox_override("panel", prof_style)
	left_vbox.add_child(profile_card)

	var prof_hbox := HBoxContainer.new()
	prof_hbox.add_theme_constant_override("separation", 10)
	profile_card.add_child(prof_hbox)

	# --- Left Block: Character Sprite & Vitals Gauges ---
	var left_block := VBoxContainer.new()
	left_block.custom_minimum_size = Vector2(115, 0)
	left_block.add_theme_constant_override("separation", 4)
	prof_hbox.add_child(left_block)

	# Framed Character Sprite Container
	var sprite_frame := PanelContainer.new()
	var spr_style := StyleBoxFlat.new()
	spr_style.bg_color = Color(0.07, 0.09, 0.13, 0.95)
	spr_style.border_color = Color(0.22, 0.32, 0.46, 0.8)
	spr_style.set_border_width_all(1)
	spr_style.set_corner_radius_all(4)
	spr_style.content_margin_left = 2
	spr_style.content_margin_right = 2
	spr_style.content_margin_top = 2
	spr_style.content_margin_bottom = 2
	sprite_frame.add_theme_stylebox_override("panel", spr_style)
	left_block.add_child(sprite_frame)

	inspector_avatar = TextureRect.new()
	inspector_avatar.custom_minimum_size = Vector2(110, 110)
	inspector_avatar.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	inspector_avatar.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	sprite_frame.add_child(inspector_avatar)

	# HP Bar Gauge
	var hp_gauge_vbox := VBoxContainer.new()
	hp_gauge_vbox.add_theme_constant_override("separation", 1)
	left_block.add_child(hp_gauge_vbox)

	var hp_hdr_row := HBoxContainer.new()
	hp_gauge_vbox.add_child(hp_hdr_row)

	var hp_tag := Label.new()
	hp_tag.text = "HP"
	hp_tag.add_theme_font_size_override("font_size", 9)
	hp_tag.add_theme_color_override("font_color", Color(0.9, 0.4, 0.4))
	hp_hdr_row.add_child(hp_tag)

	hp_hdr_row.add_spacer(false)

	inspector_hp_lbl = Label.new()
	inspector_hp_lbl.text = "100/100"
	inspector_hp_lbl.add_theme_font_size_override("font_size", 9)
	inspector_hp_lbl.add_theme_color_override("font_color", Color.WHITE)
	hp_hdr_row.add_child(inspector_hp_lbl)

	inspector_hp_bar = ProgressBar.new()
	inspector_hp_bar.custom_minimum_size = Vector2(110, 8)
	inspector_hp_bar.show_percentage = false
	var hp_bg := StyleBoxFlat.new()
	hp_bg.bg_color = Color(0.18, 0.08, 0.08, 0.9)
	hp_bg.set_corner_radius_all(3)
	var hp_fill := StyleBoxFlat.new()
	hp_fill.bg_color = Color(0.25, 0.75, 0.35, 1.0)
	hp_fill.set_corner_radius_all(3)
	inspector_hp_bar.add_theme_stylebox_override("background", hp_bg)
	inspector_hp_bar.add_theme_stylebox_override("fill", hp_fill)
	hp_gauge_vbox.add_child(inspector_hp_bar)

	# SP/MP Bar Gauge (Placeholder)
	var sp_gauge_vbox := VBoxContainer.new()
	sp_gauge_vbox.add_theme_constant_override("separation", 1)
	left_block.add_child(sp_gauge_vbox)

	var sp_hdr_row := HBoxContainer.new()
	sp_gauge_vbox.add_child(sp_hdr_row)

	var sp_tag := Label.new()
	sp_tag.text = "SP/MP"
	sp_tag.add_theme_font_size_override("font_size", 9)
	sp_tag.add_theme_color_override("font_color", Color(0.3, 0.75, 1.0))
	sp_hdr_row.add_child(sp_tag)

	sp_hdr_row.add_spacer(false)

	inspector_sp_lbl = Label.new()
	inspector_sp_lbl.text = "50/50"
	inspector_sp_lbl.add_theme_font_size_override("font_size", 9)
	inspector_sp_lbl.add_theme_color_override("font_color", Color(0.7, 0.85, 1.0))
	sp_hdr_row.add_child(inspector_sp_lbl)

	inspector_sp_bar = ProgressBar.new()
	inspector_sp_bar.custom_minimum_size = Vector2(110, 8)
	inspector_sp_bar.show_percentage = false
	inspector_sp_bar.max_value = 50
	inspector_sp_bar.value = 50
	var sp_bg := StyleBoxFlat.new()
	sp_bg.bg_color = Color(0.08, 0.12, 0.18, 0.9)
	sp_bg.set_corner_radius_all(3)
	var sp_fill := StyleBoxFlat.new()
	sp_fill.bg_color = Color(0.2, 0.6, 0.95, 1.0)
	sp_fill.set_corner_radius_all(3)
	inspector_sp_bar.add_theme_stylebox_override("background", sp_bg)
	inspector_sp_bar.add_theme_stylebox_override("fill", sp_fill)
	sp_gauge_vbox.add_child(inspector_sp_bar)

	# --- Right Block: Unit Identity, Stance & Combat Stats ---
	var right_block := VBoxContainer.new()
	right_block.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_block.add_theme_constant_override("separation", 3)
	prof_hbox.add_child(right_block)

	inspector_name_lbl = Label.new()
	inspector_name_lbl.text = "Warrior"
	inspector_name_lbl.add_theme_font_size_override("font_size", 12)
	inspector_name_lbl.add_theme_color_override("font_color", Color.WHITE)
	right_block.add_child(inspector_name_lbl)

	inspector_team_lbl = Label.new()
	inspector_team_lbl.text = "Allied Hero (Melee Frontline)"
	inspector_team_lbl.add_theme_font_size_override("font_size", 9)
	inspector_team_lbl.add_theme_color_override("font_color", Color(0.4, 0.75, 1.0))
	right_block.add_child(inspector_team_lbl)

	var info_sep := HSeparator.new()
	right_block.add_child(info_sep)

	inspector_stance_lbl = Label.new()
	inspector_stance_lbl.text = "Stance: Neutral"
	inspector_stance_lbl.add_theme_font_size_override("font_size", 10)
	inspector_stance_lbl.add_theme_color_override("font_color", Color(0.95, 0.85, 0.4))
	right_block.add_child(inspector_stance_lbl)

	var stats_row1 := HBoxContainer.new()
	right_block.add_child(stats_row1)
	inspector_speed_lbl = Label.new()
	inspector_speed_lbl.text = "Speed: 15"
	inspector_speed_lbl.add_theme_font_size_override("font_size", 9)
	inspector_speed_lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
	stats_row1.add_child(inspector_speed_lbl)

	stats_row1.add_spacer(false)
	inspector_range_lbl = Label.new()
	inspector_range_lbl.text = "Range: 1 (Melee)"
	inspector_range_lbl.add_theme_font_size_override("font_size", 9)
	inspector_range_lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
	stats_row1.add_child(inspector_range_lbl)

	inspector_pos_lbl = Label.new()
	inspector_pos_lbl.text = "Pos: (4, 5)"
	inspector_pos_lbl.add_theme_font_size_override("font_size", 9)
	inspector_pos_lbl.add_theme_color_override("font_color", Color(0.5, 0.7, 0.9))
	right_block.add_child(inspector_pos_lbl)

	inspector_target_lbl = Label.new()
	inspector_target_lbl.text = "Target: None"
	inspector_target_lbl.add_theme_font_size_override("font_size", 9)
	inspector_target_lbl.add_theme_color_override("font_color", Color(0.4, 0.9, 0.7))
	right_block.add_child(inspector_target_lbl)

	inspector_buffs_lbl = Label.new()
	inspector_buffs_lbl.text = "Buffs: None active"
	inspector_buffs_lbl.add_theme_font_size_override("font_size", 9)
	inspector_buffs_lbl.add_theme_color_override("font_color", Color(0.8, 0.6, 1.0))
	right_block.add_child(inspector_buffs_lbl)

	# Action Setup Dock / Palette Header
	var action_dock_hdr := Label.new()
	action_dock_hdr.text = "AVAILABLE ACTIONS / STANCE DECK"
	action_dock_hdr.add_theme_font_size_override("font_size", 10)
	action_dock_hdr.add_theme_color_override("font_color", Color(0.85, 0.8, 0.55))
	left_vbox.add_child(action_dock_hdr)

	# Action Palette List
	var palette_vbox := VBoxContainer.new()
	palette_vbox.add_theme_constant_override("separation", 3)
	left_vbox.add_child(palette_vbox)

	var actions_info: Array[Dictionary] = [
		{"name": "🏃 Move", "desc": "1 Tile Cardinal | Cost: 1 Block"},
		{"name": "⚡ Sprint", "desc": "2 Tiles Forward Dash | Cost: 2 Blocks"},
		{"name": "⚔ Strike", "desc": "Direct Physical Attack | Cost: 2 Blocks"},
		{"name": "🛡 Defend", "desc": "-50% Damage Mitigation | Cost: 2 Blocks"},
		{"name": "✦ Focus Skill", "desc": "+50% ATK Boost for 2 Rds | Cost: 2 Blocks"},
		{"name": "🎯 Target Lock", "desc": "Assign Combat Focus | Cost: 1 Block"},
		{"name": "🔄 Stance Shift", "desc": "Aggressive / Defensive / Tactical | 1 Block"}
	]

	for act_item in actions_info:
		var act_card := PanelContainer.new()
		var act_style := StyleBoxFlat.new()
		act_style.bg_color = Color(0.11, 0.14, 0.19, 0.7)
		act_style.border_color = Color(0.22, 0.3, 0.42, 0.6)
		act_style.set_border_width_all(1)
		act_style.set_corner_radius_all(4)
		act_style.content_margin_left = 6
		act_style.content_margin_right = 6
		act_style.content_margin_top = 2
		act_style.content_margin_bottom = 2
		act_card.add_theme_stylebox_override("panel", act_style)

		var act_hbox := HBoxContainer.new()
		act_card.add_child(act_hbox)

		var act_name_lbl := Label.new()
		act_name_lbl.text = act_item["name"]
		act_name_lbl.add_theme_font_size_override("font_size", 9)
		act_name_lbl.add_theme_color_override("font_color", Color(0.9, 0.95, 1.0))
		act_hbox.add_child(act_name_lbl)

		act_hbox.add_spacer(false)

		var act_desc_lbl := Label.new()
		act_desc_lbl.text = act_item["desc"]
		act_desc_lbl.add_theme_font_size_override("font_size", 8)
		act_desc_lbl.add_theme_color_override("font_color", Color(0.6, 0.7, 0.8))
		act_hbox.add_child(act_desc_lbl)

		palette_vbox.add_child(act_card)

	var left_hint := Label.new()
	left_hint.text = "💡 Actions for selected unit lineup in bottom timeline."
	left_hint.add_theme_font_size_override("font_size", 8)
	left_hint.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
	left_vbox.add_child(left_hint)

	# 2. Center Column: Prominent Combat Grid (54px Tiles, Centered at 496px)
	var grid_vbox := VBoxContainer.new()
	grid_vbox.custom_minimum_size = Vector2(496, 488)
	grid_vbox.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	grid_vbox.alignment = BoxContainer.ALIGNMENT_CENTER
	grid_vbox.add_theme_constant_override("separation", 4)
	center_hbox.add_child(grid_vbox)

	var grid_header := Label.new()
	grid_header.text = "9x9 COMBAT GRID"
	grid_header.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	grid_header.add_theme_font_size_override("font_size", 11)
	grid_header.add_theme_color_override("font_color", Color(0.8, 0.9, 1.0))
	grid_vbox.add_child(grid_header)

	var grid_frame := PanelContainer.new()
	grid_frame.custom_minimum_size = Vector2(494, 494)
	var frame_style := StyleBoxFlat.new()
	frame_style.bg_color = Color(0.05, 0.07, 0.1, 1.0)
	frame_style.border_color = Color(0.22, 0.32, 0.46, 1.0)
	frame_style.set_border_width_all(2)
	frame_style.set_corner_radius_all(6)
	frame_style.content_margin_left = 4
	frame_style.content_margin_right = 4
	frame_style.content_margin_top = 4
	frame_style.content_margin_bottom = 4
	grid_frame.add_theme_stylebox_override("panel", frame_style)
	grid_vbox.add_child(grid_frame)

	# Container holding both background tile grid and overlay units (486x486 px)
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

	# 3. Right Popup Area: Tactical Forecast & Arena Intel (Expands flush to right edge)
	var right_popup_panel := PanelContainer.new()
	right_popup_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_popup_panel.custom_minimum_size = Vector2(340, 488)
	var right_style := StyleBoxFlat.new()
	right_style.bg_color = Color(0.08, 0.10, 0.15, 0.95)
	right_style.border_color = Color(0.2, 0.28, 0.42, 0.8)
	right_style.set_border_width_all(1)
	right_style.set_corner_radius_all(6)
	right_style.content_margin_left = 8
	right_style.content_margin_right = 8
	right_style.content_margin_top = 6
	right_style.content_margin_bottom = 6
	right_popup_panel.add_theme_stylebox_override("panel", right_style)
	center_hbox.add_child(right_popup_panel)

	var right_vbox := VBoxContainer.new()
	right_vbox.add_theme_constant_override("separation", 6)
	right_popup_panel.add_child(right_vbox)

	var forecast_hdr := Label.new()
	forecast_hdr.text = "TACTICAL FORECAST & ARENA INTEL"
	forecast_hdr.add_theme_font_size_override("font_size", 11)
	forecast_hdr.add_theme_color_override("font_color", Color(0.65, 0.8, 1.0))
	right_vbox.add_child(forecast_hdr)

	# Tactical Forecast Box
	var forecast_box := PanelContainer.new()
	var fc_style := StyleBoxFlat.new()
	fc_style.bg_color = Color(0.12, 0.15, 0.22, 0.8)
	fc_style.border_color = Color(0.3, 0.4, 0.55, 0.6)
	fc_style.set_border_width_all(1)
	fc_style.set_corner_radius_all(6)
	fc_style.content_margin_left = 8
	fc_style.content_margin_right = 8
	fc_style.content_margin_top = 6
	fc_style.content_margin_bottom = 6
	forecast_box.add_theme_stylebox_override("panel", fc_style)
	right_vbox.add_child(forecast_box)

	var fc_vbox := VBoxContainer.new()
	fc_vbox.add_theme_constant_override("separation", 4)
	forecast_box.add_child(fc_vbox)

	forecast_target_lbl = Label.new()
	forecast_target_lbl.text = "Target: Enemy Brawler"
	forecast_target_lbl.add_theme_font_size_override("font_size", 11)
	forecast_target_lbl.add_theme_color_override("font_color", Color(1.0, 0.85, 0.4))
	fc_vbox.add_child(forecast_target_lbl)

	forecast_dist_lbl = Label.new()
	forecast_dist_lbl.text = "Distance: 1 Tile"
	forecast_dist_lbl.add_theme_font_size_override("font_size", 10)
	forecast_dist_lbl.add_theme_color_override("font_color", Color(0.8, 0.9, 1.0))
	fc_vbox.add_child(forecast_dist_lbl)

	forecast_range_badge = Label.new()
	forecast_range_badge.text = "✓ IN ATTACK RANGE"
	forecast_range_badge.add_theme_font_size_override("font_size", 10)
	forecast_range_badge.add_theme_color_override("font_color", Color(0.4, 0.9, 0.4))
	fc_vbox.add_child(forecast_range_badge)

	forecast_stance_lbl = Label.new()
	forecast_stance_lbl.text = "Stance Matchup: Neutral vs Defensive"
	forecast_stance_lbl.add_theme_font_size_override("font_size", 10)
	forecast_stance_lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
	fc_vbox.add_child(forecast_stance_lbl)

	forecast_dmg_lbl = Label.new()
	forecast_dmg_lbl.text = "Estimated Strike: ~12 dmg (Defending)"
	forecast_dmg_lbl.add_theme_font_size_override("font_size", 10)
	forecast_dmg_lbl.add_theme_color_override("font_color", Color(0.95, 0.5, 0.5))
	fc_vbox.add_child(forecast_dmg_lbl)

	# Arena Rules / Objectives
	var arena_hdr := Label.new()
	arena_hdr.text = "ARENA RULES & OBJECTIVES"
	arena_hdr.add_theme_font_size_override("font_size", 10)
	arena_hdr.add_theme_color_override("font_color", Color(0.85, 0.8, 0.55))
	right_vbox.add_child(arena_hdr)

	var arena_card := PanelContainer.new()
	var ar_style := StyleBoxFlat.new()
	ar_style.bg_color = Color(0.10, 0.13, 0.18, 0.8)
	ar_style.border_color = Color(0.22, 0.3, 0.42, 0.6)
	ar_style.set_border_width_all(1)
	ar_style.set_corner_radius_all(6)
	ar_style.content_margin_left = 8
	ar_style.content_margin_right = 8
	ar_style.content_margin_top = 6
	ar_style.content_margin_bottom = 6
	arena_card.add_theme_stylebox_override("panel", ar_style)
	right_vbox.add_child(arena_card)

	var ar_vbox := VBoxContainer.new()
	ar_vbox.add_theme_constant_override("separation", 3)
	arena_card.add_child(ar_vbox)

	intel_center_lbl = Label.new()
	intel_center_lbl.text = "🚩 Center Ring (4, 4): Core arena landmark"
	intel_center_lbl.add_theme_font_size_override("font_size", 9)
	intel_center_lbl.add_theme_color_override("font_color", Color(0.8, 0.9, 1.0))
	ar_vbox.add_child(intel_center_lbl)

	intel_exit_lbl = Label.new()
	intel_exit_lbl.text = "🚪 Exit Tile (8, 4): Allows safe withdrawal"
	intel_exit_lbl.add_theme_font_size_override("font_size", 9)
	intel_exit_lbl.add_theme_color_override("font_color", Color(0.9, 0.8, 0.4))
	ar_vbox.add_child(intel_exit_lbl)

	intel_turn_lbl = Label.new()
	intel_turn_lbl.text = "⏱ 10 Blocks / Round: Simultaneous Execution"
	intel_turn_lbl.add_theme_font_size_override("font_size", 9)
	intel_turn_lbl.add_theme_color_override("font_color", Color(0.6, 0.8, 1.0))
	ar_vbox.add_child(intel_turn_lbl)

	var right_hint := Label.new()
	right_hint.text = "💡 Click any enemy unit to inspect their threat level."
	right_hint.add_theme_font_size_override("font_size", 8)
	right_hint.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
	right_vbox.add_child(right_hint)

	# --------------------------------------------------------------------------
	# BOTTOM ROW: PLAYBACK CONTROLS + 10-BLOCK TIMELINE (Selected Unit Lineup)
	# --------------------------------------------------------------------------
	var bottom_panel := PanelContainer.new()
	bottom_panel.custom_minimum_size = Vector2(0, 68)
	var bottom_style := StyleBoxFlat.new()
	bottom_style.bg_color = Color(0.08, 0.11, 0.16, 0.95)
	bottom_style.border_color = Color(0.22, 0.32, 0.46, 0.9)
	bottom_style.set_border_width_all(1)
	bottom_style.set_corner_radius_all(6)
	bottom_style.content_margin_left = 8
	bottom_style.content_margin_right = 8
	bottom_style.content_margin_top = 4
	bottom_style.content_margin_bottom = 4
	bottom_panel.add_theme_stylebox_override("panel", bottom_style)
	main_vbox.add_child(bottom_panel)

	var bottom_hbox := HBoxContainer.new()
	bottom_hbox.add_theme_constant_override("separation", 10)
	bottom_panel.add_child(bottom_hbox)

	# Controls sub-column: Icon Buttons with Tooltips + Status Message
	var controls_vbox := VBoxContainer.new()
	controls_vbox.custom_minimum_size = Vector2(210, 0)
	controls_vbox.add_theme_constant_override("separation", 2)
	bottom_hbox.add_child(controls_vbox)

	var btns_row := HBoxContainer.new()
	btns_row.add_theme_constant_override("separation", 4)
	controls_vbox.add_child(btns_row)

	btn_step = Button.new()
	btn_step.text = "⏭"
	btn_step.tooltip_text = "Step Simulation (Advance 1 Block)"
	btn_step.custom_minimum_size = Vector2(34, 28)
	btn_step.add_theme_font_size_override("font_size", 13)
	btn_step.pressed.connect(_on_step_block_pressed)
	btns_row.add_child(btn_step)

	btn_play = Button.new()
	btn_play.text = "▶"
	btn_play.tooltip_text = "Auto Play Simulation"
	btn_play.custom_minimum_size = Vector2(34, 28)
	btn_play.add_theme_font_size_override("font_size", 13)
	btn_play.pressed.connect(_on_play_toggle_pressed)
	btns_row.add_child(btn_play)

	btn_reset = Button.new()
	btn_reset.text = "↺"
	btn_reset.tooltip_text = "Reset Combat Simulation"
	btn_reset.custom_minimum_size = Vector2(34, 28)
	btn_reset.add_theme_font_size_override("font_size", 13)
	btn_reset.pressed.connect(_on_reset_pressed)
	btns_row.add_child(btn_reset)

	btn_toggle_timeline = Button.new()
	btn_toggle_timeline.text = "👤"
	btn_toggle_timeline.tooltip_text = "Timeline View: Selected Unit (Click to toggle All Units)"
	btn_toggle_timeline.custom_minimum_size = Vector2(34, 28)
	btn_toggle_timeline.add_theme_font_size_override("font_size", 13)
	btn_toggle_timeline.pressed.connect(_on_toggle_timeline_pressed)
	btns_row.add_child(btn_toggle_timeline)

	status_label = Label.new()
	status_label.text = "Block 1 ready."
	status_label.add_theme_font_size_override("font_size", 10)
	status_label.add_theme_color_override("font_color", Color(0.7, 0.85, 0.95))
	controls_vbox.add_child(status_label)

	var vsep := VSeparator.new()
	bottom_hbox.add_child(vsep)

	# Timeline sub-column
	var timeline_vbox := VBoxContainer.new()
	timeline_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	timeline_vbox.add_theme_constant_override("separation", 2)
	bottom_hbox.add_child(timeline_vbox)

	timeline_header_lbl = Label.new()
	timeline_header_lbl.text = "10-BLOCK TIMELINE — Selected: Warrior"
	timeline_header_lbl.add_theme_font_size_override("font_size", 10)
	timeline_header_lbl.add_theme_color_override("font_color", Color(0.9, 0.85, 0.4))
	timeline_vbox.add_child(timeline_header_lbl)

	var timeline_hbox := HBoxContainer.new()
	timeline_hbox.alignment = BoxContainer.ALIGNMENT_CENTER
	timeline_hbox.add_theme_constant_override("separation", 4)
	timeline_vbox.add_child(timeline_hbox)

	timeline_blocks.clear()
	timeline_labels.clear()

	for b: int in range(1, 11):
		var block_box := PanelContainer.new()
		block_box.custom_minimum_size = Vector2(76, 36)
		var b_style := StyleBoxFlat.new()
		b_style.bg_color = Color(0.14, 0.18, 0.24, 0.8)
		b_style.border_color = Color(0.25, 0.32, 0.42, 1.0)
		b_style.set_border_width_all(1)
		b_style.set_corner_radius_all(4)
		block_box.add_theme_stylebox_override("panel", b_style)

		var b_vbox := VBoxContainer.new()
		b_vbox.alignment = BoxContainer.ALIGNMENT_CENTER
		b_vbox.add_theme_constant_override("separation", 1)
		block_box.add_child(b_vbox)

		var num_lbl := Label.new()
		num_lbl.text = "BLK %d" % b
		num_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		num_lbl.add_theme_font_size_override("font_size", 9)
		num_lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
		b_vbox.add_child(num_lbl)

		var desc_lbl := Label.new()
		desc_lbl.text = "--"
		desc_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		desc_lbl.add_theme_font_size_override("font_size", 8)
		desc_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		b_vbox.add_child(desc_lbl)

		timeline_hbox.add_child(block_box)
		timeline_blocks.append(block_box)
		timeline_labels.append(desc_lbl)

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
	btn_play.text = "▶"
	btn_play.tooltip_text = "Auto Play Simulation"
	btn_toggle_timeline.text = "👥" if timeline_mode_all else "👤"

	# Clear previous tokens
	var overlay: Control = ui_canvas.find_child("UnitOverlay", true, false) as Control
	if overlay:
		for child in overlay.get_children():
			child.queue_free()
	unit_token_nodes.clear()

	# Clear roster cards in both party and enemy containers
	if party_cards_container:
		for child in party_cards_container.get_children():
			child.queue_free()
	if enemy_cards_container:
		for child in enemy_cards_container.get_children():
			child.queue_free()
	roster_cards.clear()

	log_rich_text.clear()
	_append_log("[color=cyan]>>> New 10-Block Tactical Battle Initialized (Pre-planned Actions Loaded).[/color]")

	# 1. Initialize 9x9 Closed Arena with designated Exit at (8, 4)
	var exit_coord := Vector2i(8, 4)
	sim_grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [exit_coord])
	sim_resolver = CombatResolver.new()
	sim_resolver.grid = sim_grid
	sim_resolver.log_emitted.connect(_on_resolver_log_emitted)

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
	# Warrior:
	#   Block 1: Move Up to Center (4, 4)
	#   Block 2-3: Melee Attack on Brawler (25 power)
	sim_resolver.schedule_action(warrior, MoveCommand.new(warrior, sim_grid, Vector2i(0, -1), 1), 1)
	sim_resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 25, 2, 0, sim_grid), 2)

	# Ranger:
	#   Block 1-2: Skill - Focus (+50% Attack Boost for 2 rounds in Tactical Stance)
	#   Block 4: Move Up to (4, 5) (Distance 2 to Brawler at 4, 3)
	#   Block 5-6: Boosted Ranged Attack on Brawler (25 power x 1.5 = 38 dmg)
	sim_resolver.schedule_action(ranger, FocusCommand.new(ranger, 1.5, 2, 2, 0), 1)
	sim_resolver.schedule_action(ranger, MoveCommand.new(ranger, sim_grid, Vector2i(0, -1), 1), 4)
	sim_resolver.schedule_action(ranger, AttackCommand.new(ranger, brawler, 25, 2, 0, sim_grid), 5)

	# Enemy Brawler:
	#   Block 1: Target Warrior
	#   Block 2-3: Defend (Defensive Stance, mitigates 50% damage against Warrior's strike)
	#   Block 4-5: Counter-Attack Warrior (20 power)
	sim_resolver.schedule_action(brawler, TargetCommand.new(brawler, warrior, 2), 1)
	sim_resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5, 2, 1), 2)
	sim_resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 20, 2, 0, sim_grid), 4)

	# Fleeing Scout:
	#   Block 1: Move East to (6, 4)
	#   Block 2-3: Sprint East 2 steps to (8, 4) Exit Tile -> Safely Withdraws!
	sim_resolver.schedule_action(scout, MoveCommand.new(scout, sim_grid, Vector2i(1, 0), 1), 1)
	sim_resolver.schedule_action(scout, MoveCommand.new(scout, sim_grid, Vector2i(1, 0), 2, 2), 2)

	# 5. Build visual unit tokens and roster cards
	for ally in sim_allies:
		_create_unit_token(ally, tex_warrior if ally.name == "Warrior" else tex_ranger)
		_create_roster_card(ally, Color(0.2, 0.5, 0.9))
	for enemy in sim_enemies:
		_create_unit_token(enemy, tex_brawler if enemy.name == "Enemy Brawler" else tex_scout)
		_create_roster_card(enemy, Color(0.9, 0.3, 0.3))

	# Set default selected unit
	_select_combatant(warrior)
	_highlight_timeline_block(1)
	status_label.text = "Block 1 ready. Click 'Step Block' or 'Auto Play' to resolve."


func _create_unit_token(combatant: Combatant, texture: Texture2D) -> void:
	var overlay: Control = ui_canvas.find_child("UnitOverlay", true, false) as Control
	if not overlay:
		return

	var token_ctrl := Control.new()
	token_ctrl.custom_minimum_size = Vector2(TILE_SIZE, TILE_SIZE)
	token_ctrl.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	var icon := TextureRect.new()
	icon.texture = texture
	icon.custom_minimum_size = Vector2(TILE_SIZE - 6, TILE_SIZE - 6)
	icon.position = Vector2(3, 3)
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	token_ctrl.add_child(icon)

	# Name badge
	var name_lbl := Label.new()
	name_lbl.text = combatant.name
	name_lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	name_lbl.position = Vector2(-15, -13)
	name_lbl.custom_minimum_size = Vector2(TILE_SIZE + 30, 14)
	name_lbl.add_theme_font_size_override("font_size", 9)
	name_lbl.add_theme_color_override("font_color", Color(1, 1, 1, 0.9))
	token_ctrl.add_child(name_lbl)

	# Selection border
	var sel_border := ReferenceRect.new()
	sel_border.name = "SelectionBorder"
	sel_border.custom_minimum_size = Vector2(TILE_SIZE, TILE_SIZE)
	sel_border.border_color = Color(1.0, 0.85, 0.25, 0.95)
	sel_border.border_width = 2.0
	sel_border.editor_only = false
	sel_border.visible = false
	token_ctrl.add_child(sel_border)

	token_ctrl.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
			_select_combatant(combatant)
	)

	overlay.add_child(token_ctrl)
	unit_token_nodes[combatant] = token_ctrl

	# Set initial grid pixel position
	var pos: Vector2i = sim_grid.get_position(combatant)
	token_ctrl.position = Vector2(pos.x * TILE_SIZE, pos.y * TILE_SIZE)


func _create_roster_card(combatant: Combatant, team_color: Color) -> void:
	var target_container: VBoxContainer = party_cards_container if sim_allies.has(combatant) else enemy_cards_container
	if not target_container:
		return

	var card := PanelContainer.new()
	card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	var card_style := StyleBoxFlat.new()
	card_style.bg_color = Color(0.11, 0.14, 0.19, 0.9)
	card_style.border_color = team_color
	card_style.set_border_width_all(1)
	card_style.set_corner_radius_all(4)
	card_style.content_margin_left = 6
	card_style.content_margin_right = 6
	card_style.content_margin_top = 2
	card_style.content_margin_bottom = 2
	card.add_theme_stylebox_override("panel", card_style)

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 6)
	card.add_child(hbox)

	var name_lbl := Label.new()
	name_lbl.text = combatant.name
	name_lbl.custom_minimum_size = Vector2(85, 0)
	name_lbl.add_theme_font_size_override("font_size", 10)
	name_lbl.add_theme_color_override("font_color", team_color.lightened(0.3))
	hbox.add_child(name_lbl)

	var hp_bar := ProgressBar.new()
	hp_bar.custom_minimum_size = Vector2(65, 10)
	hp_bar.max_value = combatant.max_hp
	hp_bar.value = combatant.hp
	hp_bar.show_percentage = false
	hbox.add_child(hp_bar)

	var hp_lbl := Label.new()
	hp_lbl.name = "HpLabel"
	hp_lbl.text = "%d/%d" % [combatant.hp, combatant.max_hp]
	hp_lbl.custom_minimum_size = Vector2(45, 0)
	hp_lbl.add_theme_font_size_override("font_size", 9)
	hbox.add_child(hp_lbl)

	var stance_lbl := Label.new()
	stance_lbl.name = "StanceLabel"
	stance_lbl.text = CombatTypes.stance_to_string(combatant.current_stance)
	stance_lbl.custom_minimum_size = Vector2(55, 0)
	stance_lbl.add_theme_font_size_override("font_size", 9)
	stance_lbl.add_theme_color_override("font_color", Color(0.7, 0.8, 0.9))
	hbox.add_child(stance_lbl)

	var pos_lbl := Label.new()
	pos_lbl.name = "PosLabel"
	pos_lbl.text = "Pos: %s" % str(sim_grid.get_position(combatant))
	pos_lbl.custom_minimum_size = Vector2(55, 0)
	pos_lbl.add_theme_font_size_override("font_size", 9)
	pos_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
	hbox.add_child(pos_lbl)

	card.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
			_select_combatant(combatant)
	)

	target_container.add_child(card)
	roster_cards[combatant] = card


# ==============================================================================
# SECTION 3: STEPPING, VISUAL PLAYBACK, INSPECTOR & FORECAST UPDATES
# ==============================================================================

func _select_combatant(combatant: Combatant) -> void:
	selected_combatant = combatant
	_update_roster_card_highlights()
	_update_unit_token_highlights()
	_update_selected_unit_panel()
	_update_tactical_forecast_panel()
	_update_timeline_labels()


func _update_roster_card_highlights() -> void:
	for c: Combatant in roster_cards.keys():
		@warning_ignore("unsafe_cast")
		var card: PanelContainer = roster_cards[c] as PanelContainer
		var style := StyleBoxFlat.new()
		style.set_corner_radius_all(4)
		style.content_margin_left = 6
		style.content_margin_right = 6
		style.content_margin_top = 2
		style.content_margin_bottom = 2
		var is_ally: bool = sim_allies.has(c)
		var base_color: Color = Color(0.2, 0.5, 0.9) if is_ally else Color(0.9, 0.3, 0.3)

		if c == selected_combatant:
			style.bg_color = Color(0.18, 0.22, 0.32, 0.95)
			style.border_color = Color(1.0, 0.85, 0.25, 1.0)
			style.set_border_width_all(2)
		else:
			style.bg_color = Color(0.11, 0.14, 0.19, 0.9)
			style.border_color = base_color
			style.set_border_width_all(1)
		card.add_theme_stylebox_override("panel", style)


func _update_unit_token_highlights() -> void:
	for c: Combatant in unit_token_nodes.keys():
		@warning_ignore("unsafe_cast")
		var token: Control = unit_token_nodes[c] as Control
		var border: ReferenceRect = token.find_child("SelectionBorder", true, false) as ReferenceRect
		if border:
			border.visible = (c == selected_combatant)


func _update_selected_unit_panel() -> void:
	if not selected_combatant:
		return

	inspector_name_lbl.text = selected_combatant.name
	var is_ally: bool = sim_allies.has(selected_combatant)
	if is_ally:
		inspector_team_lbl.text = "Allied Hero (Party Member)"
		inspector_team_lbl.add_theme_color_override("font_color", Color(0.4, 0.75, 1.0))
		if selected_combatant.name == "Warrior":
			inspector_avatar.texture = tex_sprite_warrior if tex_sprite_warrior != null else tex_warrior
		else:
			inspector_avatar.texture = tex_sprite_ranger if tex_sprite_ranger != null else tex_ranger
	else:
		inspector_team_lbl.text = "Hostile Enemy Unit"
		inspector_team_lbl.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
		if selected_combatant.name == "Enemy Brawler":
			inspector_avatar.texture = tex_sprite_brawler if tex_sprite_brawler != null else tex_brawler
		else:
			inspector_avatar.texture = tex_sprite_scout if tex_sprite_scout != null else tex_scout

	inspector_hp_bar.max_value = selected_combatant.max_hp
	inspector_hp_bar.value = selected_combatant.hp
	inspector_hp_lbl.text = "%d/%d" % [selected_combatant.hp, selected_combatant.max_hp]

	# Dynamic HP Bar Fill Color
	var hp_ratio: float = float(selected_combatant.hp) / float(maxi(1, selected_combatant.max_hp))
	var hp_fill := StyleBoxFlat.new()
	hp_fill.set_corner_radius_all(3)
	if hp_ratio > 0.5:
		hp_fill.bg_color = Color(0.25, 0.75, 0.35, 1.0)
	elif hp_ratio > 0.25:
		hp_fill.bg_color = Color(0.9, 0.75, 0.2, 1.0)
	else:
		hp_fill.bg_color = Color(0.9, 0.25, 0.25, 1.0)
	inspector_hp_bar.add_theme_stylebox_override("fill", hp_fill)

	# SP/MP Bar (Placeholder Gauge)
	if inspector_sp_bar and inspector_sp_lbl:
		var cur_sp: int = 50 if selected_combatant.is_alive() else 0
		inspector_sp_bar.max_value = 50
		inspector_sp_bar.value = cur_sp
		inspector_sp_lbl.text = "%d/50" % cur_sp

	inspector_stance_lbl.text = "Stance: " + CombatTypes.stance_to_string(selected_combatant.current_stance)
	inspector_speed_lbl.text = "Speed: %d" % selected_combatant.speed
	inspector_range_lbl.text = "Range: %d tile%s" % [selected_combatant.attack_range, "s" if selected_combatant.attack_range > 1 else ""]

	if selected_combatant.is_active():
		inspector_pos_lbl.text = "Pos: %s" % str(sim_grid.get_position(selected_combatant))
	elif selected_combatant.is_withdrawn:
		inspector_pos_lbl.text = "Pos: Withdrawn (Safe)"
	else:
		inspector_pos_lbl.text = "Pos: Defeated"

	if selected_combatant.active_target is Combatant:
		@warning_ignore("unsafe_cast")
		inspector_target_lbl.text = "Target: " + (selected_combatant.active_target as Combatant).name
	else:
		inspector_target_lbl.text = "Target: None"

	if selected_combatant.attack_boost_rounds_remaining > 0:
		inspector_buffs_lbl.text = "Buffs: Focus (+50%% ATK, %d rds)" % selected_combatant.attack_boost_rounds_remaining
	else:
		inspector_buffs_lbl.text = "Buffs: None active"


func _update_tactical_forecast_panel() -> void:
	if not selected_combatant:
		return

	# Determine target to compare against
	var target: Combatant = null
	if selected_combatant.active_target is Combatant:
		@warning_ignore("unsafe_cast")
		target = selected_combatant.active_target as Combatant
	elif sim_allies.has(selected_combatant):
		target = sim_enemies[0]
	else:
		target = sim_allies[0]

	if target:
		forecast_target_lbl.text = "Target: %s" % target.name
		if selected_combatant.is_active() and target.is_active():
			var p1: Vector2i = sim_grid.get_position(selected_combatant)
			var p2: Vector2i = sim_grid.get_position(target)
			var dist: int = sim_grid.get_distance(p1, p2)
			forecast_dist_lbl.text = "Distance: %d tile%s (Manhattan)" % [dist, "s" if dist != 1 else ""]

			var in_range: bool = dist <= selected_combatant.attack_range
			if in_range:
				forecast_range_badge.text = "✓ IN ATTACK RANGE (%d/%d)" % [dist, selected_combatant.attack_range]
				forecast_range_badge.add_theme_color_override("font_color", Color(0.4, 0.9, 0.4))
			else:
				var needed: int = dist - selected_combatant.attack_range
				forecast_range_badge.text = "✗ OUT OF RANGE (Need +%d steps)" % needed
				forecast_range_badge.add_theme_color_override("font_color", Color(0.95, 0.4, 0.4))

			var mult: float = CombatTypes.get_matchup_multiplier(selected_combatant.current_stance, target.current_stance)
			forecast_stance_lbl.text = "Stance: %s vs %s (%.2fx)" % [
				CombatTypes.stance_to_string(selected_combatant.current_stance),
				CombatTypes.stance_to_string(target.current_stance),
				mult
			]

			var base_power: int = 25
			var est_dmg: int = int(roundf(float(base_power) * mult * selected_combatant.attack_boost_multiplier * (1.0 - target.defense_mitigation_ratio)))
			var mit_note: String = " (-%d%% def)" % int(target.defense_mitigation_ratio * 100) if target.defense_mitigation_ratio > 0 else ""
			forecast_dmg_lbl.text = "Est. Strike Impact: ~%d dmg%s" % [est_dmg, mit_note]
		else:
			forecast_dist_lbl.text = "Distance: Inactive combatant"
			forecast_range_badge.text = "Status: Off Active Grid"
			forecast_range_badge.add_theme_color_override("font_color", Color(0.6, 0.6, 0.6))
			forecast_stance_lbl.text = "Stance: N/A"
			forecast_dmg_lbl.text = "Est. Strike: N/A"


func _step_simulation() -> void:
	if sim_finished:
		return

	_highlight_timeline_block(current_sim_block)
	_append_log("[color=#fde047][b]━━━ BLOCK %d RESOLUTION ━━━[/b][/color]" % current_sim_block)

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
		btn_play.text = "▶"
		status_label.text = "BATTLE CONCLUDED! All hostiles resolved."
		_append_log("\n[color=#4ade80][b]🎉 BATTLE CONCLUDED: All hostiles defeated or safely withdrawn![/b][/color]")
		return

	current_sim_block += 1
	if current_sim_block > CombatResolver.TOTAL_BLOCKS:
		sim_resolver.finish_round()
		sim_finished = true
		auto_play_timer.stop()
		btn_play.disabled = true
		btn_step.disabled = true
		btn_play.text = "▶"
		status_label.text = "Round 1 complete."
		_append_log("[color=#38bdf8]>>> Round 1 concluded.[/color]")
	else:
		_highlight_timeline_block(current_sim_block)
		status_label.text = "Ready for Block %d." % current_sim_block


func _sync_visuals_after_block() -> void:
	# 1. Update unit token positions with smooth Tween animations (54px tile grid)
	for combatant: Combatant in unit_token_nodes.keys():
		@warning_ignore("unsafe_cast")
		var token: Control = unit_token_nodes[combatant] as Control
		if not combatant.is_active():
			if combatant.is_withdrawn:
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
		@warning_ignore("unsafe_cast")
		var card: PanelContainer = roster_cards[combatant] as PanelContainer
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

	# 3. Refresh Inspector & Tactical Forecast
	_update_selected_unit_panel()
	_update_tactical_forecast_panel()


func _update_timeline_labels() -> void:
	if timeline_mode_all:
		timeline_header_lbl.text = "10-BLOCK TIMELINE — Mode: ALL UNITS (Click 'View' to toggle)"
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
	else:
		var unit_name: String = selected_combatant.name if selected_combatant != null else "None"
		timeline_header_lbl.text = "10-BLOCK TIMELINE — Lineup for: %s (Selected)" % unit_name
		for b: int in range(1, 11):
			var label: Label = timeline_labels[b - 1]
			if selected_combatant == null:
				label.text = "--"
				continue

			var found_text: String = ""
			var actions_at_b: Array[CombatResolver.ScheduledAction] = sim_resolver.get_actions_at_block(b)
			for a in actions_at_b:
				if a.combatant == selected_combatant:
					if a.start_block == a.end_block:
						found_text = a.command.command_name
					elif b == a.start_block:
						found_text = a.command.command_name + " (Start)"
					elif b == a.end_block:
						found_text = a.command.command_name + " (Resolve)"
					else:
						found_text = a.command.command_name + " (Hold)"
					break

			label.text = found_text if not found_text.is_empty() else "--"


func _on_toggle_timeline_pressed() -> void:
	timeline_mode_all = not timeline_mode_all
	btn_toggle_timeline.text = "👥" if timeline_mode_all else "👤"
	btn_toggle_timeline.tooltip_text = "Timeline View: All Units (Click for Selected)" if timeline_mode_all else "Timeline View: Selected Unit (Click for All)"
	_update_timeline_labels()


func _highlight_timeline_block(active_block: int) -> void:
	for i: int in range(timeline_blocks.size()):
		var panel: PanelContainer = timeline_blocks[i]
		var b_style := StyleBoxFlat.new()
		b_style.set_corner_radius_all(4)
		if i + 1 == active_block:
			b_style.bg_color = Color(0.18, 0.35, 0.55, 1.0)
			b_style.border_color = Color(0.4, 0.8, 1.0, 1.0)
			b_style.set_border_width_all(2)
		else:
			b_style.bg_color = Color(0.14, 0.18, 0.24, 0.8)
			b_style.border_color = Color(0.25, 0.32, 0.42, 1.0)
			b_style.set_border_width_all(1)
		panel.add_theme_stylebox_override("panel", b_style)


func _on_resolver_log_emitted(msg: String) -> void:
	var formatted: String = msg
	if msg.begins_with("[Attack]"):
		if "DEFEATED" in msg:
			formatted = "[color=#f87171]⚔ [/color][b][color=#ef4444]" + msg.substr(9) + "[/color][/b]"
		else:
			formatted = "[color=#f87171]⚔ " + msg.substr(9) + "[/color]"
	elif msg.begins_with("[Attack Out of Range]") or msg.begins_with("[Attack Miss]"):
		var tag_len: int = 22 if msg.begins_with("[Attack Out of Range]") else 14
		formatted = "[color=#fb7185]❌ [i]" + msg.substr(tag_len) + "[/i][/color]"
	elif msg.begins_with("[Move]") or msg.begins_with("[Sprint]"):
		var is_sprint: bool = msg.begins_with("[Sprint]")
		var tag_len: int = 9 if is_sprint else 7
		formatted = "[color=#38bdf8]➜ [b]" + (msg.left(tag_len - 1).strip_edges()) + "[/b]: " + msg.substr(tag_len) + "[/color]"
	elif msg.begins_with("[Move Blocked]") or msg.begins_with("[Sprint Blocked]"):
		formatted = "[color=#f59e0b]⚠ " + msg + "[/color]"
	elif msg.begins_with("[Withdrawal]"):
		formatted = "[b][color=#fbbf24]⚑ " + msg.substr(13) + "[/color][/b]"
	elif msg.begins_with("[Skill:"):
		formatted = "[color=#c084fc]✦ [b]" + msg + "[/b][/color]"
	elif msg.begins_with("[Defend]"):
		formatted = "[color=#60a5fa]🛡 [b]" + msg + "[/b][/color]"
	elif msg.begins_with("[Target]"):
		formatted = "[color=#34d399]🎯 " + msg.substr(9) + "[/color]"
	elif msg.begins_with("[Action Started]"):
		formatted = "[color=#94a3b8]⏳ " + msg.substr(17) + "[/color]"
	elif msg.begins_with("[Cancelled]"):
		formatted = "[color=#ef4444]⛔ " + msg.substr(12) + "[/color]"
	elif msg.strip_edges().begins_with("[") and "Status:" in msg:
		formatted = "[color=#64748b]" + msg + "[/color]"
	else:
		formatted = "[color=#e2e8f0]" + msg + "[/color]"

	_append_log("  " + formatted)


func _append_log(text: String) -> void:
	if log_rich_text:
		log_rich_text.append_text(text + "\n")


func _on_step_block_pressed() -> void:
	_step_simulation()


func _on_play_toggle_pressed() -> void:
	if is_auto_playing:
		is_auto_playing = false
		auto_play_timer.stop()
		btn_play.text = "▶"
		btn_play.tooltip_text = "Auto Play Simulation"
	else:
		is_auto_playing = true
		btn_play.text = "⏸"
		btn_play.tooltip_text = "Pause Simulation"
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


func _run_resolution_feed_logging_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING RESOLUTION FEED & ACTION LOGGING TEST  ===")
	print("=======================================================")
	var captured_logs: Array[String] = []

	var exit_coord := Vector2i(8, 4)
	var test_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [exit_coord])
	var test_resolver := CombatResolver.new()
	test_resolver.grid = test_grid
	test_resolver.log_emitted.connect(func(msg: String) -> void:
		captured_logs.append(msg)
	)

	var warrior := Combatant.new("Warrior", 100, 15, 1)
	var ranger := Combatant.new("Ranger", 80, 14, 2)
	var brawler := Combatant.new("Enemy Brawler", 45, 12, 1)
	var scout := Combatant.new("Fleeing Scout", 30, 16, 1)

	var allies: Array[Combatant] = [warrior, ranger]
	var enemies: Array[Combatant] = [brawler, scout]

	test_resolver.register_combatant(warrior)
	test_resolver.register_combatant(ranger)
	test_resolver.register_combatant(brawler)
	test_resolver.register_combatant(scout)

	test_grid.deploy_combatants(allies, enemies)

	test_resolver.schedule_action(warrior, MoveCommand.new(warrior, test_grid, Vector2i(0, -1), 1), 1)
	test_resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 25, 2, 0, test_grid), 2)
	test_resolver.schedule_action(ranger, FocusCommand.new(ranger, 1.5, 2, 2, 0), 1)
	test_resolver.schedule_action(ranger, MoveCommand.new(ranger, test_grid, Vector2i(0, -1), 1), 4)
	test_resolver.schedule_action(ranger, AttackCommand.new(ranger, brawler, 25, 2, 0, test_grid), 5)
	test_resolver.schedule_action(brawler, TargetCommand.new(brawler, warrior, 2), 1)
	test_resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5, 2, 1), 2)
	test_resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 20, 2, 0, test_grid), 4)
	test_resolver.schedule_action(scout, MoveCommand.new(scout, test_grid, Vector2i(1, 0), 1), 1)
	test_resolver.schedule_action(scout, MoveCommand.new(scout, test_grid, Vector2i(1, 0), 2, 2), 2)

	for b: int in range(1, 7):
		test_resolver.execute_block(b)

	var has_move_log: bool = false
	var has_skill_log: bool = false
	var has_defend_log: bool = false
	var has_damage_log: bool = false
	var has_withdrawal_log: bool = false
	var has_defeat_log: bool = false

	for log_msg in captured_logs:
		if "[Move]" in log_msg and "moved from" in log_msg:
			has_move_log = true
		if "[Skill: Focus]" in log_msg:
			has_skill_log = true
		if "[Defend]" in log_msg and "Defensive Stance" in log_msg:
			has_defend_log = true
		if "[Attack]" in log_msg and "damage" in log_msg:
			has_damage_log = true
		if "[Withdrawal]" in log_msg and "WITHDREW" in log_msg:
			has_withdrawal_log = true
		if "DEFEATED" in log_msg:
			has_defeat_log = true

	var battle_resolved: bool = test_resolver.is_combat_resolved(allies, enemies)

	print("\n=======================================================")
	print("=== RESOLUTION FEED LOGGING TEST ASSERTIONS        ===")
	print("=======================================================")
	print("  [Feed] Movement coordinates logged: %s" % ("PASS" if has_move_log else "FAIL"))
	print("  [Feed] Focus skill usage logged: %s" % ("PASS" if has_skill_log else "FAIL"))
	print("  [Feed] Defense stance mitigation logged: %s" % ("PASS" if has_defend_log else "FAIL"))
	print("  [Feed] Attack damage and target HP logged: %s" % ("PASS" if has_damage_log else "FAIL"))
	print("  [Feed] Exit tile withdrawal logged: %s" % ("PASS" if has_withdrawal_log else "FAIL"))
	print("  [Feed] Enemy defeat logged: %s" % ("PASS" if has_defeat_log else "FAIL"))
	print("  [Feed] Combat resolution state reached: %s" % ("PASS" if battle_resolved else "FAIL"))

	warrior.clear_target()
	ranger.clear_target()
	brawler.clear_target()
	scout.clear_target()
	test_resolver.registered_combatants.clear()

	return (has_move_log and has_skill_log and has_defend_log and 
		has_damage_log and has_withdrawal_log and has_defeat_log and battle_resolved)