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
var tex_skirmisher: Texture2D
var tex_archer: Texture2D
var tex_berserker: Texture2D
var tex_shaman: Texture2D

# Character Full Sprite Textures
var tex_sprite_warrior: Texture2D
var tex_sprite_ranger: Texture2D
var tex_sprite_brawler: Texture2D
var tex_sprite_scout: Texture2D
var tex_sprite_skirmisher: Texture2D
var tex_sprite_archer: Texture2D
var tex_sprite_berserker: Texture2D
var tex_sprite_shaman: Texture2D

# Active Combatant Unit Instances & Scenario State
var unit_warrior: Combatant
var unit_ranger: Combatant
var unit_brawler: Combatant
var unit_scout: Combatant
var unit_skirmisher: Combatant
var unit_archer: Combatant
var unit_berserker: Combatant
var unit_shaman: Combatant
var reinforcements_spawned: bool = false
var current_combat_round: int = 1
var btn_next_round: Button

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
	tex_skirmisher = _load_svg_texture("res://assets/token_skirmisher.svg")
	tex_archer = _load_svg_texture("res://assets/token_archer.svg")
	tex_berserker = _load_svg_texture("res://assets/token_berserker.svg")
	tex_shaman = _load_svg_texture("res://assets/token_shaman.svg")

	# Load sprite assets
	tex_sprite_warrior = _load_svg_texture("res://assets/sprite_warrior.svg")
	tex_sprite_ranger = _load_svg_texture("res://assets/sprite_ranger.svg")
	tex_sprite_brawler = _load_svg_texture("res://assets/sprite_brawler.svg")
	tex_sprite_scout = _load_svg_texture("res://assets/sprite_scout.svg")
	tex_sprite_skirmisher = _load_svg_texture("res://assets/sprite_skirmisher.svg")
	tex_sprite_archer = _load_svg_texture("res://assets/sprite_archer.svg")
	tex_sprite_berserker = _load_svg_texture("res://assets/sprite_berserker.svg")
	tex_sprite_shaman = _load_svg_texture("res://assets/sprite_shaman.svg")

func _get_combatant_token_texture(c: Combatant) -> Texture2D:
	if c == null:
		return null
	match c.name:
		"Warrior": return tex_warrior
		"Ranger": return tex_ranger
		"Enemy Brawler": return tex_brawler
		"Fleeing Scout", "Goblin Scout": return tex_scout
		"Goblin Skirmisher": return tex_skirmisher
		"Goblin Archer": return tex_archer
		"Goblin Berserker": return tex_berserker
		"Goblin Shaman": return tex_shaman
		_: return tex_brawler

func _get_combatant_sprite_texture(c: Combatant) -> Texture2D:
	if c == null:
		return null
	match c.name:
		"Warrior": return tex_sprite_warrior
		"Ranger": return tex_sprite_ranger
		"Enemy Brawler": return tex_sprite_brawler
		"Fleeing Scout", "Goblin Scout": return tex_sprite_scout
		"Goblin Skirmisher": return tex_sprite_skirmisher
		"Goblin Archer": return tex_sprite_archer
		"Goblin Berserker": return tex_sprite_berserker
		"Goblin Shaman": return tex_sprite_shaman
		_: return tex_sprite_brawler


const TILE_SIZE: int = 54
const GRID_SIZE: int = 9
const UNIT_BLOCK_W: int = 80
const BLOCK_GAP: int = 4
const TOTAL_BLOCKS: int = 10

enum TimelineViewMode {
	TRACK_LAYERS,
	COMBINED_BLOCK
}

enum CombatPhase {
	PLANNING,
	RESOLVING,
	REPLAY
}

class BlockSnapshot:
	extends RefCounted
	var block_index: int = 0
	var round_num: int = 1
	var reinforcements_spawned: bool = false
	var positions: Dictionary = {}         # Combatant -> Vector2i
	var hps: Dictionary = {}               # Combatant -> int
	var max_hps: Dictionary = {}           # Combatant -> int
	var stances: Dictionary = {}           # Combatant -> CombatTypes.StanceType
	var targets: Dictionary = {}           # Combatant -> Variant
	var is_active: Dictionary = {}         # Combatant -> bool
	var is_withdrawn: Dictionary = {}      # Combatant -> bool
	var attack_boost_mult: Dictionary = {} # Combatant -> float
	var attack_boost_rds: Dictionary = {}  # Combatant -> int
	var defense_mit: Dictionary = {}       # Combatant -> float

class ActionLogEntry:
	extends RefCounted
	var round_num: int = 1
	var block_num: int = 1
	var ref_tag: String = "R1-B01"   # R<X>-B<Y>
	var actor_name: String = ""
	var actor_is_ally: bool = true
	var target_name: String = ""
	var target_is_ally: bool = false
	var category: String = "ATTACK"  # ATTACK, DEFEND, BUFF, MOVE, TARGET, EVENT
	var skill_name: String = ""
	var skill_subtype: String = ""
	var value_num: int = 0
	var outcome_tag: String = "HIT"
	var detail_text: String = ""
	var status_text: String = ""
	var coord_from: Vector2i = Vector2i(-1, -1)
	var coord_to: Vector2i = Vector2i(-1, -1)

# Interactive UI References
var ui_canvas: CanvasLayer
var grid_tile_nodes: Dictionary = {}  # Vector2i -> TextureRect
var unit_token_nodes: Dictionary = {} # Combatant -> Control
var roster_cards: Dictionary = {}     # Combatant -> PanelContainer
var party_cards_container: GridContainer
var enemy_cards_container: GridContainer
var party_title_lbl: Label
var enemy_title_lbl: Label
var log_rich_text: RichTextLabel
var active_cell_dock: PanelContainer
var log_step_badge: Label
var btn_log_prev: Button
var btn_log_next: Button
var btn_expand_log: Button
var log_modal_overlay: PanelContainer
var modal_entries_vbox: VBoxContainer
var modal_filter_btns: Dictionary = {} # String -> Button
var current_log_filter: String = "ALL"
var all_action_log_entries: Array[ActionLogEntry] = []
var active_log_entry_index: int = -1
var pending_block_actions: Array[CombatResolver.ScheduledAction] = []
var status_label: Label
var phase_badge_lbl: Label
var btn_step: Button
var btn_play: Button
var btn_reset: Button
var btn_rewind: Button
var btn_toggle_view: Button
var auto_play_timer: Timer

# Selection and Inspector State
var selected_combatant: Combatant = null
var timeline_view_mode: TimelineViewMode = TimelineViewMode.TRACK_LAYERS
var combat_phase: CombatPhase = CombatPhase.PLANNING
var filter_active_only: bool = false

# Left Popup (Unit Inspector & Action Setup Dock)
var inspector_hdr: Label
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
var ally_actions_container: VBoxContainer
var enemy_actions_container: VBoxContainer
var left_hint: Label

# Right Popup (Tactical Forecast & Arena Intel)
var forecast_target_lbl: Label
var forecast_dist_lbl: Label
var forecast_range_badge: Label
var forecast_stance_lbl: Label
var forecast_dmg_lbl: Label
var intel_center_lbl: Label
var intel_exit_lbl: Label
var intel_turn_lbl: Label

# 2x2 Party Select Box UI Components
var selected_party_hero: Combatant = null
var party_select_grid: GridContainer
var party_slot_panels: Array[PanelContainer] = []
var btn_party_all: Button

# Timeline Deck UI Components
var tracks_container: VBoxContainer
var tracks_ruler_row: HBoxContainer
var ruler_tick_nodes: Array[PanelContainer] = []
var ruler_tick_labels: Array[Label] = []
var single_track_row: HBoxContainer
var track_avatar_icon: TextureRect
var track_unit_name_lbl: Label
var track_unit_stance_lbl: Label
var track_clips_hbox: HBoxContainer
var enemy_classified_banner: PanelContainer

# Popout Modal Timeline Suite UI Components
const UNIT_MODAL_BLOCK_W: int = 94
var btn_popout: Button
var timeline_modal_overlay: PanelContainer
var modal_tracks_list: VBoxContainer
var modal_ruler_tick_nodes: Array[PanelContainer] = []
var modal_ruler_tick_labels: Array[Label] = []
var modal_phase_lbl: Label
var btn_modal_play: Button
var btn_modal_step: Button
var btn_modal_rewind: Button
var btn_modal_reset: Button
var btn_modal_close: Button

# Combined Block Inspector UI Components
var combined_container: VBoxContainer
var combined_block_title_lbl: Label
var combined_chip_buttons: Array[Button] = []
var btn_combined_prev: Button
var btn_combined_next: Button
var btn_filter_active: Button
var combined_party_vbox: VBoxContainer
var combined_enemy_vbox: VBoxContainer
var combined_party_hdr: Label
var combined_enemy_hdr: Label

# Simulation & Replay State
var sim_grid: CombatGrid
var sim_resolver: CombatResolver
var sim_allies: Array[Combatant] = []
var sim_enemies: Array[Combatant] = []
var current_sim_block: int = 1
var replay_block: int = 1
var combined_selected_block: int = 1
var is_auto_playing: bool = false
var sim_finished: bool = false
var snapshots: Array[BlockSnapshot] = []
var recorded_actions: Array[CombatResolver.ScheduledAction] = []

# Multi-Round Timeline Traversal
var viewed_timeline_round: int = 1
var round_recorded_actions: Dictionary = {} # int -> Array[CombatResolver.ScheduledAction]
var round_snapshots: Dictionary = {}        # int -> Array[BlockSnapshot]
var btn_rnd_1: Button
var btn_rnd_2: Button
var btn_modal_rnd_1: Button
var btn_modal_rnd_2: Button

# Combat Grid Unit Hover Card (Replaces persistent overlapping labels)
var grid_hover_tooltip: PanelContainer
var hover_name_lbl: Label
var hover_hp_lbl: Label
var hover_status_lbl: Label

func _ready() -> void:
	print("\n=======================================================")
	print("=== STARTING TACTICAL RPG ARENA HARNESS             ===")
	print("=======================================================")

	var test1_passed: bool = _run_warrior_vs_3_goblins_battle()
	var test2_passed: bool = _run_combat_grid_and_withdrawal_test()
	var test3_passed: bool = _run_resolution_feed_logging_test()
	var test4_passed: bool = _run_goblin_ambush_and_casualty_test()
	var all_tests_passed: bool = test1_passed and test2_passed and test3_passed and test4_passed

	var exit_code: int = 0 if all_tests_passed else 1

	if DisplayServer.get_name() == "headless":
		print("Execution finished headlessly. Exiting with code: %d\n" % exit_code)
		get_tree().quit(exit_code)
	else:
		print("Running in windowed mode. Initializing interactive tactical UI...\n")
		_setup_interactive_ui()
		_init_interactive_simulation()


# ==============================================================================
# SECTION 1: INTERACTIVE UI CREATION & ZERO-CLIPPING LAYOUT
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
	root_margin.add_theme_constant_override("margin_top", 4)
	root_margin.add_theme_constant_override("margin_bottom", 4)
	ui_canvas.add_child(root_margin)

	var main_vbox := VBoxContainer.new()
	main_vbox.add_theme_constant_override("separation", 4)
	root_margin.add_child(main_vbox)

	# --------------------------------------------------------------------------
	# TOP ROW: PARTY STATUS (2x2) | ACTION LOG DOCK (Center) | ENEMY STATUS (5x2)
	# --------------------------------------------------------------------------
	var top_hbox := HBoxContainer.new()
	top_hbox.custom_minimum_size = Vector2(0, 84)
	top_hbox.add_theme_constant_override("separation", 8)
	main_vbox.add_child(top_hbox)

	# Top-Left: Allied Party Status Panel (Static 2x2 Grid for 4 Heroes)
	var party_panel := PanelContainer.new()
	party_panel.custom_minimum_size = Vector2(280, 84)
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
	party_vbox.add_theme_constant_override("separation", 2)
	party_panel.add_child(party_vbox)

	var party_hdr_hbox := HBoxContainer.new()
	party_vbox.add_child(party_hdr_hbox)
	party_title_lbl = Label.new()
	party_title_lbl.text = "⚔ PARTY MEMBERS (2/4)"
	party_title_lbl.add_theme_font_size_override("font_size", 10)
	party_title_lbl.add_theme_color_override("font_color", Color(0.38, 0.72, 1.0))
	party_hdr_hbox.add_child(party_title_lbl)

	var party_hint := Label.new()
	party_hint.text = "(2x2 Formation)"
	party_hint.add_theme_font_size_override("font_size", 8)
	party_hint.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
	party_hdr_hbox.add_spacer(false)
	party_hdr_hbox.add_child(party_hint)

	party_cards_container = GridContainer.new()
	party_cards_container.columns = 2
	party_cards_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	party_cards_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	party_cards_container.add_theme_constant_override("h_separation", 4)
	party_cards_container.add_theme_constant_override("v_separation", 3)
	party_vbox.add_child(party_cards_container)

	# Top-Center: Single-Action Resolution Log Dock with Navigation & Expand Button
	var log_panel := PanelContainer.new()
	log_panel.custom_minimum_size = Vector2(430, 84)
	log_panel.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	var log_style := StyleBoxFlat.new()
	log_style.bg_color = Color(0.06, 0.08, 0.12, 0.95)
	log_style.border_color = Color(0.25, 0.32, 0.44, 0.8)
	log_style.set_border_width_all(1)
	log_style.set_corner_radius_all(6)
	log_style.content_margin_left = 6
	log_style.content_margin_right = 6
	log_style.content_margin_top = 4
	log_style.content_margin_bottom = 4
	log_panel.add_theme_stylebox_override("panel", log_style)
	top_hbox.add_child(log_panel)

	var log_vbox := VBoxContainer.new()
	log_vbox.add_theme_constant_override("separation", 2)
	log_panel.add_child(log_vbox)

	var log_hdr_hbox := HBoxContainer.new()
	log_hdr_hbox.add_theme_constant_override("separation", 6)
	log_vbox.add_child(log_hdr_hbox)

	var log_title := Label.new()
	log_title.text = "📜 RESOLUTION LOG"
	log_title.add_theme_font_size_override("font_size", 10)
	log_title.add_theme_color_override("font_color", Color(0.96, 0.82, 0.35))
	log_hdr_hbox.add_child(log_title)

	log_step_badge = Label.new()
	log_step_badge.text = "[Awaiting Action]"
	log_step_badge.add_theme_font_size_override("font_size", 8)
	log_step_badge.add_theme_color_override("font_color", Color(0.55, 0.75, 0.95))
	log_hdr_hbox.add_child(log_step_badge)

	btn_log_prev = Button.new()
	btn_log_prev.text = "◀"
	btn_log_prev.custom_minimum_size = Vector2(20, 16)
	btn_log_prev.add_theme_font_size_override("font_size", 8)
	btn_log_prev.tooltip_text = "Previous Action Cell"
	btn_log_prev.pressed.connect(_on_log_prev_pressed)
	log_hdr_hbox.add_child(btn_log_prev)

	btn_log_next = Button.new()
	btn_log_next.text = "▶"
	btn_log_next.custom_minimum_size = Vector2(20, 16)
	btn_log_next.add_theme_font_size_override("font_size", 8)
	btn_log_next.tooltip_text = "Next Action Cell"
	btn_log_next.pressed.connect(_on_log_next_pressed)
	log_hdr_hbox.add_child(btn_log_next)

	log_hdr_hbox.add_spacer(false)

	btn_expand_log = Button.new()
	btn_expand_log.text = "⛶ EXPAND"
	btn_expand_log.custom_minimum_size = Vector2(66, 18)
	btn_expand_log.add_theme_font_size_override("font_size", 8)
	var exp_btn_style := StyleBoxFlat.new()
	exp_btn_style.bg_color = Color(0.16, 0.22, 0.32, 0.9)
	exp_btn_style.border_color = Color(0.38, 0.6, 0.9, 0.8)
	exp_btn_style.set_border_width_all(1)
	exp_btn_style.set_corner_radius_all(3)
	exp_btn_style.content_margin_left = 4
	exp_btn_style.content_margin_right = 4
	btn_expand_log.add_theme_stylebox_override("normal", exp_btn_style)
	btn_expand_log.tooltip_text = "Expand Full Chronological Action Log Modal (Shortcut: L)"
	btn_expand_log.pressed.connect(func() -> void: _toggle_log_modal(true))
	log_hdr_hbox.add_child(btn_expand_log)

	# Active single action cell dock
	active_cell_dock = PanelContainer.new()
	active_cell_dock.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	active_cell_dock.size_flags_vertical = Control.SIZE_EXPAND_FILL
	var dock_style := StyleBoxFlat.new()
	dock_style.bg_color = Color(0.04, 0.06, 0.09, 0.7)
	dock_style.border_color = Color(0.18, 0.24, 0.34, 0.5)
	dock_style.set_border_width_all(1)
	dock_style.set_corner_radius_all(4)
	dock_style.content_margin_left = 4
	dock_style.content_margin_right = 4
	dock_style.content_margin_top = 2
	dock_style.content_margin_bottom = 2
	active_cell_dock.add_theme_stylebox_override("panel", dock_style)
	log_vbox.add_child(active_cell_dock)

	# Maintain hidden RichTextLabel for headless test / logging compatibility
	log_rich_text = RichTextLabel.new()
	log_rich_text.visible = false
	log_panel.add_child(log_rich_text)

	# Top-Right: Enemy Hostiles Status Panel (Active Roster Grid)
	var enemy_panel := PanelContainer.new()
	enemy_panel.custom_minimum_size = Vector2(516, 84)
	enemy_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
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
	enemy_vbox.add_theme_constant_override("separation", 2)
	enemy_panel.add_child(enemy_vbox)

	var enemy_hdr_hbox := HBoxContainer.new()
	enemy_vbox.add_child(enemy_hdr_hbox)
	enemy_title_lbl = Label.new()
	enemy_title_lbl.text = "💀 ENEMY HOSTILES (2 Active)"
	enemy_title_lbl.add_theme_font_size_override("font_size", 10)
	enemy_title_lbl.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
	enemy_hdr_hbox.add_child(enemy_title_lbl)

	var enemy_hint := Label.new()
	enemy_hint.text = "(Active Roster • Click to inspect)"
	enemy_hint.add_theme_font_size_override("font_size", 8)
	enemy_hint.add_theme_color_override("font_color", Color(0.6, 0.5, 0.5))
	enemy_hdr_hbox.add_spacer(false)
	enemy_hdr_hbox.add_child(enemy_hint)

	enemy_cards_container = GridContainer.new()
	enemy_cards_container.columns = 5
	enemy_cards_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	enemy_cards_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	enemy_cards_container.add_theme_constant_override("h_separation", 4)
	enemy_cards_container.add_theme_constant_override("v_separation", 3)
	enemy_vbox.add_child(enemy_cards_container)

	# --------------------------------------------------------------------------
	# CENTER ROW: LEFT POPUP (Inspector) | COMBAT GRID (54px) | RIGHT POPUP (Forecast)
	# --------------------------------------------------------------------------
	var center_hbox := HBoxContainer.new()
	center_hbox.add_theme_constant_override("separation", 10)
	center_hbox.size_flags_vertical = Control.SIZE_EXPAND_FILL
	main_vbox.add_child(center_hbox)

	# 1. Left Popup Area: Unit Inspector & Dynamic Actions/Threat Dock
	var left_popup_panel := PanelContainer.new()
	left_popup_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left_popup_panel.custom_minimum_size = Vector2(340, 468)
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
	left_vbox.add_theme_constant_override("separation", 5)
	left_popup_panel.add_child(left_vbox)

	inspector_hdr = Label.new()
	inspector_hdr.text = "UNIT INSPECTOR & ACTION SETUP"
	inspector_hdr.add_theme_font_size_override("font_size", 11)
	inspector_hdr.add_theme_color_override("font_color", Color(0.65, 0.8, 1.0))
	left_vbox.add_child(inspector_hdr)

	# Selected Unit Profile Card (2-Block: Left=Sprite+Vitals, Right=Stats)
	var profile_card := PanelContainer.new()
	var prof_style := StyleBoxFlat.new()
	prof_style.bg_color = Color(0.12, 0.15, 0.22, 0.8)
	prof_style.border_color = Color(0.3, 0.4, 0.55, 0.6)
	prof_style.set_border_width_all(1)
	prof_style.set_corner_radius_all(6)
	prof_style.content_margin_left = 8
	prof_style.content_margin_right = 8
	prof_style.content_margin_top = 5
	prof_style.content_margin_bottom = 5
	profile_card.add_theme_stylebox_override("panel", prof_style)
	left_vbox.add_child(profile_card)

	var prof_hbox := HBoxContainer.new()
	prof_hbox.add_theme_constant_override("separation", 10)
	profile_card.add_child(prof_hbox)

	# Left Block: Avatar Sprite + Vitals
	var left_block := VBoxContainer.new()
	left_block.custom_minimum_size = Vector2(115, 0)
	left_block.add_theme_constant_override("separation", 4)
	prof_hbox.add_child(left_block)

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

	# HP Gauge
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

	# SP/MP Gauge
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
	hp_gauge_vbox.add_child(inspector_sp_bar)

	# Right Block: Identity & Stats
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

	var stats_row2 := HBoxContainer.new()
	right_block.add_child(stats_row2)
	inspector_pos_lbl = Label.new()
	inspector_pos_lbl.text = "Pos: (4, 4)"
	inspector_pos_lbl.add_theme_font_size_override("font_size", 9)
	inspector_pos_lbl.add_theme_color_override("font_color", Color(0.5, 0.7, 0.9))
	stats_row2.add_child(inspector_pos_lbl)

	stats_row2.add_spacer(false)
	inspector_target_lbl = Label.new()
	inspector_target_lbl.text = "Target: Enemy Brawler"
	inspector_target_lbl.add_theme_font_size_override("font_size", 9)
	inspector_target_lbl.add_theme_color_override("font_color", Color(0.9, 0.75, 0.4))
	stats_row2.add_child(inspector_target_lbl)

	inspector_buffs_lbl = Label.new()
	inspector_buffs_lbl.text = "Buffs: None active"
	inspector_buffs_lbl.add_theme_font_size_override("font_size", 9)
	inspector_buffs_lbl.add_theme_color_override("font_color", Color(0.7, 0.9, 0.7))
	right_block.add_child(inspector_buffs_lbl)

	# Ally Action Setup Palette
	ally_actions_container = VBoxContainer.new()
	ally_actions_container.add_theme_constant_override("separation", 3)
	left_vbox.add_child(ally_actions_container)

	var ally_pal_hdr := Label.new()
	ally_pal_hdr.text = "ACTION PALETTE (ALLIED HERO)"
	ally_pal_hdr.add_theme_font_size_override("font_size", 10)
	ally_pal_hdr.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
	ally_actions_container.add_child(ally_pal_hdr)

	var ally_actions: Array[Dictionary] = [
		{"name": "⚔ Attack", "desc": "Standard Strike | Cost: 2 Blocks", "cost": 2, "color": Color(0.9, 0.3, 0.3)},
		{"name": "🛡 Defend", "desc": "Mitigation Barrier | Cost: 2 Blocks", "cost": 2, "color": Color(0.3, 0.6, 0.95)},
		{"name": "✦ Focus", "desc": "+50% ATK Buff (2 rds) | Cost: 2 Blocks", "cost": 2, "color": Color(0.7, 0.4, 0.95)},
		{"name": "➜ Move", "desc": "1-Tile Cardinal Step | Cost: 1 Block", "cost": 1, "color": Color(0.2, 0.8, 0.9)},
		{"name": "🎯 Target Lock", "desc": "Assign Combat Focus | Cost: 1 Block", "cost": 1, "color": Color(0.3, 0.85, 0.5)},
		{"name": "🔄 Stance Shift", "desc": "Drop to Neutral | Cost: 1 Block", "cost": 1, "color": Color(0.7, 0.7, 0.8)}
	]

	for act in ally_actions:
		var act_card := _create_mini_action_card(act)
		ally_actions_container.add_child(act_card)

	# Enemy Possible Actions Deck (Secret Intent Pool)
	enemy_actions_container = VBoxContainer.new()
	enemy_actions_container.add_theme_constant_override("separation", 3)
	enemy_actions_container.visible = false
	left_vbox.add_child(enemy_actions_container)

	var enemy_deck_hdr := Label.new()
	enemy_deck_hdr.text = "POSSIBLE ACTIONS POOL (THREAT INTEL)"
	enemy_deck_hdr.add_theme_font_size_override("font_size", 10)
	enemy_deck_hdr.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
	enemy_actions_container.add_child(enemy_deck_hdr)

	left_hint = Label.new()
	left_hint.text = "💡 Actions for selected hero line up on continuous timeline below."
	left_hint.add_theme_font_size_override("font_size", 8)
	left_hint.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
	left_vbox.add_child(left_hint)

	# 2. Center Column: Prominent Combat Grid (54px Tiles)
	var grid_vbox := VBoxContainer.new()
	grid_vbox.custom_minimum_size = Vector2(496, 480)
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
	grid_frame.custom_minimum_size = Vector2(490, 490)
	var frame_style := StyleBoxFlat.new()
	frame_style.bg_color = Color(0.05, 0.07, 0.1, 1.0)
	frame_style.border_color = Color(0.22, 0.32, 0.46, 1.0)
	frame_style.set_border_width_all(1)
	frame_style.set_corner_radius_all(6)
	frame_style.content_margin_left = 2
	frame_style.content_margin_right = 2
	frame_style.content_margin_top = 2
	frame_style.content_margin_bottom = 2
	grid_frame.add_theme_stylebox_override("panel", frame_style)
	grid_vbox.add_child(grid_frame)

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

	var unit_overlay := Control.new()
	unit_overlay.name = "UnitOverlay"
	unit_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	grid_anchor.add_child(unit_overlay)

	# Floating Unit Inspector / Hover Tooltip (replacing overlapping persistent text labels)
	grid_hover_tooltip = PanelContainer.new()
	grid_hover_tooltip.name = "GridHoverTooltip"
	grid_hover_tooltip.visible = false
	grid_hover_tooltip.mouse_filter = Control.MOUSE_FILTER_IGNORE
	grid_hover_tooltip.custom_minimum_size = Vector2(130, 46)
	var hov_style := StyleBoxFlat.new()
	hov_style.bg_color = Color(0.06, 0.08, 0.12, 0.96)
	hov_style.border_color = Color(0.35, 0.55, 0.85, 0.9)
	hov_style.set_border_width_all(1)
	hov_style.set_corner_radius_all(4)
	hov_style.content_margin_left = 6
	hov_style.content_margin_right = 6
	hov_style.content_margin_top = 4
	hov_style.content_margin_bottom = 4
	grid_hover_tooltip.add_theme_stylebox_override("panel", hov_style)
	grid_anchor.add_child(grid_hover_tooltip)

	var hov_vbox := VBoxContainer.new()
	hov_vbox.add_theme_constant_override("separation", 2)
	grid_hover_tooltip.add_child(hov_vbox)

	hover_name_lbl = Label.new()
	hover_name_lbl.add_theme_font_size_override("font_size", 9)
	hov_vbox.add_child(hover_name_lbl)

	hover_hp_lbl = Label.new()
	hover_hp_lbl.add_theme_font_size_override("font_size", 8)
	hover_hp_lbl.add_theme_color_override("font_color", Color(0.85, 0.92, 1.0))
	hov_vbox.add_child(hover_hp_lbl)

	hover_status_lbl = Label.new()
	hover_status_lbl.add_theme_font_size_override("font_size", 7)
	hover_status_lbl.add_theme_color_override("font_color", Color(0.7, 0.75, 0.85))
	hov_vbox.add_child(hover_status_lbl)

	# 3. Right Popup Area: Tactical Forecast & Arena Intel
	var right_popup_panel := PanelContainer.new()
	right_popup_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_popup_panel.custom_minimum_size = Vector2(340, 468)
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
	# BOTTOM ROW: PLAYBACK CONTROLS + ZERO-CLIPPING CONTINUOUS ACTIVE TRACK
	# --------------------------------------------------------------------------
	var bottom_panel := PanelContainer.new()
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

	var bottom_hbox := HBoxContainer.new()
	bottom_hbox.add_theme_constant_override("separation", 10)
	bottom_panel.add_child(bottom_hbox)

	# Controls Column
	var controls_vbox := VBoxContainer.new()
	controls_vbox.custom_minimum_size = Vector2(165, 0)
	controls_vbox.add_theme_constant_override("separation", 2)
	bottom_hbox.add_child(controls_vbox)

	var btns_row := HBoxContainer.new()
	btns_row.add_theme_constant_override("separation", 3)
	controls_vbox.add_child(btns_row)

	btn_rewind = Button.new()
	btn_rewind.text = "⏮"
	btn_rewind.tooltip_text = "Rewind to Block 1 (Replay)"
	btn_rewind.custom_minimum_size = Vector2(24, 22)
	btn_rewind.add_theme_font_size_override("font_size", 10)
	btn_rewind.pressed.connect(_on_rewind_pressed)
	btns_row.add_child(btn_rewind)

	btn_play = Button.new()
	btn_play.text = "▶"
	btn_play.tooltip_text = "Auto-Resolve Round / Play Replay"
	btn_play.custom_minimum_size = Vector2(26, 22)
	btn_play.add_theme_font_size_override("font_size", 10)
	btn_play.pressed.connect(_on_play_toggle_pressed)
	btns_row.add_child(btn_play)

	btn_step = Button.new()
	btn_step.text = "⏭"
	btn_step.tooltip_text = "Step 1 Block Forward"
	btn_step.custom_minimum_size = Vector2(24, 22)
	btn_step.add_theme_font_size_override("font_size", 10)
	btn_step.pressed.connect(_on_step_block_pressed)
	btns_row.add_child(btn_step)

	btn_reset = Button.new()
	btn_reset.text = "↺"
	btn_reset.tooltip_text = "Reset Round / Return to Planning Phase"
	btn_reset.custom_minimum_size = Vector2(24, 22)
	btn_reset.add_theme_font_size_override("font_size", 10)
	btn_reset.pressed.connect(_on_reset_pressed)
	btns_row.add_child(btn_reset)

	# Round Traversal Buttons (R1 / R2) in quick view
	btn_rnd_1 = Button.new()
	btn_rnd_1.text = "R1"
	btn_rnd_1.tooltip_text = "View Round 1 Timeline"
	btn_rnd_1.custom_minimum_size = Vector2(24, 22)
	btn_rnd_1.add_theme_font_size_override("font_size", 9)
	btn_rnd_1.pressed.connect(func() -> void: _select_timeline_round(1))
	btns_row.add_child(btn_rnd_1)

	btn_rnd_2 = Button.new()
	btn_rnd_2.text = "R2"
	btn_rnd_2.tooltip_text = "View Round 2 Timeline"
	btn_rnd_2.custom_minimum_size = Vector2(24, 22)
	btn_rnd_2.add_theme_font_size_override("font_size", 9)
	btn_rnd_2.pressed.connect(func() -> void: _select_timeline_round(2))
	btns_row.add_child(btn_rnd_2)

	btn_popout = Button.new()
	btn_popout.text = "⛶"
	btn_popout.tooltip_text = "Popout Detailed Full-Screen Timeline Suite [F / ESC]"
	btn_popout.custom_minimum_size = Vector2(24, 22)
	btn_popout.add_theme_font_size_override("font_size", 10)
	btn_popout.pressed.connect(func() -> void: _toggle_timeline_modal(true))
	btns_row.add_child(btn_popout)

	phase_badge_lbl = Label.new()
	phase_badge_lbl.text = "PLANNING PHASE"
	phase_badge_lbl.add_theme_font_size_override("font_size", 8)
	phase_badge_lbl.add_theme_color_override("font_color", Color(0.95, 0.8, 0.35))
	controls_vbox.add_child(phase_badge_lbl)

	status_label = Label.new()
	status_label.text = "Set party actions, then Auto-Resolve."
	status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	status_label.add_theme_font_size_override("font_size", 8)
	status_label.add_theme_color_override("font_color", Color(0.7, 0.85, 0.95))
	controls_vbox.add_child(status_label)

	btn_next_round = Button.new()
	btn_next_round.text = "⚔ ADVANCE TO ROUND 2"
	btn_next_round.tooltip_text = "Conclude Round 1 and advance to Round 2 (Warrior Last Stand)"
	btn_next_round.custom_minimum_size = Vector2(0, 22)
	btn_next_round.add_theme_font_size_override("font_size", 9)
	var next_btn_style := StyleBoxFlat.new()
	next_btn_style.bg_color = Color(0.7, 0.45, 0.1, 0.95)
	next_btn_style.border_color = Color(1.0, 0.85, 0.3, 1.0)
	next_btn_style.set_border_width_all(1)
	next_btn_style.set_corner_radius_all(4)
	next_btn_style.content_margin_left = 6
	next_btn_style.content_margin_right = 6
	next_btn_style.content_margin_top = 1
	next_btn_style.content_margin_bottom = 1
	btn_next_round.add_theme_stylebox_override("normal", next_btn_style)
	btn_next_round.visible = false
	btn_next_round.pressed.connect(_on_next_round_pressed)
	controls_vbox.add_child(btn_next_round)

	# Timeline Display Deck: Holds Active Track Layers View
	var timeline_deck := Control.new()
	timeline_deck.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	timeline_deck.size_flags_vertical = Control.SIZE_EXPAND_FILL
	bottom_hbox.add_child(timeline_deck)

	_build_timeline_tracks_ui(timeline_deck)
	_build_timeline_modal_ui()
	_build_log_modal_ui()

	# Auto-play timer
	auto_play_timer = Timer.new()
	auto_play_timer.wait_time = 0.55
	auto_play_timer.one_shot = false
	auto_play_timer.timeout.connect(_on_auto_play_tick)
	add_child(auto_play_timer)


func _create_mini_action_card(act: Dictionary) -> PanelContainer:
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
	act_name_lbl.text = str(act["name"])
	var act_col: Color = act.get("color", Color.WHITE)
	act_name_lbl.add_theme_color_override("font_color", act_col)
	act_hbox.add_child(act_name_lbl)

	act_hbox.add_spacer(false)

	var act_desc_lbl := Label.new()
	act_desc_lbl.text = str(act["desc"])
	act_desc_lbl.add_theme_font_size_override("font_size", 8)
	act_desc_lbl.add_theme_color_override("font_color", Color(0.6, 0.7, 0.8))
	act_hbox.add_child(act_desc_lbl)

	return act_card


# ------------------------------------------------------------------------------
# SECTION 1B: ZERO-CLIPPING ACTIVE TRACK WITH 1-CLICK LAYER SELECTOR TABS
# ------------------------------------------------------------------------------

func _build_timeline_tracks_ui(parent: Control) -> void:
	tracks_container = VBoxContainer.new()
	tracks_container.set_anchors_preset(Control.PRESET_FULL_RECT)
	tracks_container.add_theme_constant_override("separation", 2)
	parent.add_child(tracks_container)

	# Row 1: Time Ruler Ticks (Starts at X=0, exactly aligned with action clips in Row 2)
	tracks_ruler_row = HBoxContainer.new()
	tracks_ruler_row.add_theme_constant_override("separation", BLOCK_GAP)
	tracks_container.add_child(tracks_ruler_row)

	ruler_tick_nodes.clear()
	ruler_tick_labels.clear()

	for b: int in range(1, TOTAL_BLOCKS + 1):
		var tick_panel := PanelContainer.new()
		tick_panel.custom_minimum_size = Vector2(UNIT_BLOCK_W, 16)
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
				_on_ruler_block_clicked(cur_b)
		)

		tracks_ruler_row.add_child(tick_panel)
		ruler_tick_nodes.append(tick_panel)
		ruler_tick_labels.append(tick_lbl)

	# Row 2: Selected Hero's Continuous Action Track Row (Starts at X=0, exact 1:1 match)
	single_track_row = HBoxContainer.new()
	single_track_row.add_theme_constant_override("separation", BLOCK_GAP)
	tracks_container.add_child(single_track_row)

	track_clips_hbox = HBoxContainer.new()
	track_clips_hbox.name = "ClipsHBox"
	track_clips_hbox.add_theme_constant_override("separation", BLOCK_GAP)
	single_track_row.add_child(track_clips_hbox)

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
	btn_view_party.pressed.connect(func() -> void:
		if not sim_allies.is_empty():
			_select_combatant(sim_allies[0])
	)
	banner_hbox.add_child(btn_view_party)


# ------------------------------------------------------------------------------
# SECTION 1C: COMBINED BLOCK INSPECTOR (SPLIT COLUMNS WITH ACTIVE FILTER)
# ------------------------------------------------------------------------------

func _build_combined_block_ui(parent: Control) -> void:
	combined_container = VBoxContainer.new()
	combined_container.set_anchors_preset(Control.PRESET_FULL_RECT)
	combined_container.add_theme_constant_override("separation", 2)
	combined_container.visible = false
	parent.add_child(combined_container)

	# Nav Bar Row: Prev | Title & Chips | Next | Active Filter Toggle | Tracks View
	var nav_hbox := HBoxContainer.new()
	nav_hbox.add_theme_constant_override("separation", 6)
	combined_container.add_child(nav_hbox)

	btn_combined_prev = Button.new()
	btn_combined_prev.text = "◀"
	btn_combined_prev.tooltip_text = "Previous Block (Shortcut: Q)"
	btn_combined_prev.custom_minimum_size = Vector2(26, 20)
	btn_combined_prev.add_theme_font_size_override("font_size", 10)
	btn_combined_prev.pressed.connect(func() -> void: _navigate_combined_block(-1))
	nav_hbox.add_child(btn_combined_prev)

	combined_block_title_lbl = Label.new()
	combined_block_title_lbl.text = "COMBINED BLOCK 01 / 10"
	combined_block_title_lbl.add_theme_font_size_override("font_size", 9)
	combined_block_title_lbl.add_theme_color_override("font_color", Color(1.0, 0.85, 0.35))
	nav_hbox.add_child(combined_block_title_lbl)

	var chip_hbox := HBoxContainer.new()
	chip_hbox.add_theme_constant_override("separation", 2)
	nav_hbox.add_child(chip_hbox)

	combined_chip_buttons.clear()
	for b: int in range(1, TOTAL_BLOCKS + 1):
		var chip_btn := Button.new()
		chip_btn.text = str(b)
		chip_btn.custom_minimum_size = Vector2(20, 20)
		chip_btn.add_theme_font_size_override("font_size", 8)
		var target_b: int = b
		chip_btn.pressed.connect(func() -> void:
			combined_selected_block = target_b
			_update_combined_block_display(combined_selected_block)
		)
		chip_hbox.add_child(chip_btn)
		combined_chip_buttons.append(chip_btn)

	btn_combined_next = Button.new()
	btn_combined_next.text = "▶"
	btn_combined_next.tooltip_text = "Next Block (Shortcut: E)"
	btn_combined_next.custom_minimum_size = Vector2(26, 20)
	btn_combined_next.add_theme_font_size_override("font_size", 10)
	btn_combined_next.pressed.connect(func() -> void: _navigate_combined_block(1))
	nav_hbox.add_child(btn_combined_next)

	nav_hbox.add_spacer(false)

	btn_filter_active = Button.new()
	btn_filter_active.text = "⚡ Active Only"
	btn_filter_active.tooltip_text = "Toggle filtering to hide idle units"
	btn_filter_active.custom_minimum_size = Vector2(85, 20)
	btn_filter_active.add_theme_font_size_override("font_size", 8)
	btn_filter_active.pressed.connect(func() -> void:
		filter_active_only = not filter_active_only
		btn_filter_active.text = "👥 Show All" if filter_active_only else "⚡ Active Only"
		_update_combined_block_display(combined_selected_block)
	)
	nav_hbox.add_child(btn_filter_active)

	var btn_tracks_return := Button.new()
	btn_tracks_return.text = "🎬 Hero Track"
	btn_tracks_return.tooltip_text = "Return to Single Hero Track View"
	btn_tracks_return.custom_minimum_size = Vector2(85, 20)
	btn_tracks_return.add_theme_font_size_override("font_size", 8)
	btn_tracks_return.pressed.connect(_on_toggle_view_mode_pressed)
	nav_hbox.add_child(btn_tracks_return)

	# Split Columns Row: Left=Allied Party, Right=Enemies
	var split_hbox := HBoxContainer.new()
	split_hbox.size_flags_vertical = Control.SIZE_EXPAND_FILL
	split_hbox.add_theme_constant_override("separation", 8)
	combined_container.add_child(split_hbox)

	# Left Column: Party Members (Heroes)
	var party_col_box := VBoxContainer.new()
	party_col_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	party_col_box.add_theme_constant_override("separation", 2)
	split_hbox.add_child(party_col_box)

	combined_party_hdr = Label.new()
	combined_party_hdr.text = "HERO ACTIONS"
	combined_party_hdr.add_theme_font_size_override("font_size", 8)
	combined_party_hdr.add_theme_color_override("font_color", Color(0.4, 0.75, 1.0))
	party_col_box.add_child(combined_party_hdr)

	var party_scroll := ScrollContainer.new()
	party_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	party_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	party_col_box.add_child(party_scroll)

	combined_party_vbox = VBoxContainer.new()
	combined_party_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	combined_party_vbox.add_theme_constant_override("separation", 2)
	party_scroll.add_child(combined_party_vbox)

	# Right Column: Enemies (Hostiles)
	var enemy_col_box := VBoxContainer.new()
	enemy_col_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	enemy_col_box.add_theme_constant_override("separation", 2)
	split_hbox.add_child(enemy_col_box)

	combined_enemy_hdr = Label.new()
	combined_enemy_hdr.text = "HOSTILE ACTIONS"
	combined_enemy_hdr.add_theme_font_size_override("font_size", 8)
	combined_enemy_hdr.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
	enemy_col_box.add_child(combined_enemy_hdr)

	var enemy_scroll := ScrollContainer.new()
	enemy_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	enemy_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	enemy_col_box.add_child(enemy_scroll)

	combined_enemy_vbox = VBoxContainer.new()
	combined_enemy_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	combined_enemy_vbox.add_theme_constant_override("separation", 2)
	enemy_scroll.add_child(combined_enemy_vbox)


# ==============================================================================
# SECTION 2: SIMULATION SETUP WITH PRE-PLANNED COMMANDS & RECORDED ARCHIVE
# ==============================================================================

func _init_interactive_simulation() -> void:
	current_combat_round = 1
	viewed_timeline_round = 1
	round_recorded_actions.clear()
	round_snapshots.clear()
	current_sim_block = 1
	replay_block = 1
	combined_selected_block = 1
	sim_finished = false
	is_auto_playing = false
	reinforcements_spawned = false
	combat_phase = CombatPhase.PLANNING
	auto_play_timer.stop()
	btn_play.text = "▶"
	btn_play.tooltip_text = "Auto-Resolve Round 1"
	if btn_next_round:
		btn_next_round.visible = false
	phase_badge_lbl.text = "PLANNING PHASE"
	phase_badge_lbl.add_theme_color_override("font_color", Color(0.95, 0.8, 0.35))
	status_label.text = "Round 1: Set party actions, then Auto-Resolve."

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

	all_action_log_entries.clear()
	active_log_entry_index = -1
	pending_block_actions.clear()
	_update_active_log_cell()

	log_rich_text.clear()
	_append_log("[color=cyan]>>> New 10-Block Tactical Battle Initialized (Round 1 Pre-planned Actions Loaded).[/color]")

	# 1. Initialize 9x9 Closed Arena with designated Exit at (8, 4)
	var exit_coord := Vector2i(8, 4)
	sim_grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [exit_coord])
	sim_resolver = CombatResolver.new()
	sim_resolver.grid = sim_grid
	sim_resolver.log_emitted.connect(_on_resolver_log_emitted)

	# 2. Combatants (Allied Party & Enemy Forces)
	unit_warrior = Combatant.new("Warrior", 100, 15, 1)
	unit_ranger = Combatant.new("Ranger", 80, 14, 2)
	unit_brawler = Combatant.new("Enemy Brawler", 45, 12, 1)
	unit_scout = Combatant.new("Fleeing Scout", 30, 16, 1)

	# Goblin Ambush Reinforcements (Arriving Block 4 after Scout exit)
	unit_skirmisher = Combatant.new("Goblin Skirmisher", 40, 15, 1)
	unit_archer = Combatant.new("Goblin Archer", 35, 13, 3)
	unit_berserker = Combatant.new("Goblin Berserker", 55, 11, 1)
	unit_shaman = Combatant.new("Goblin Shaman", 35, 12, 2)

	sim_allies = [unit_warrior, unit_ranger]
	sim_enemies = [unit_brawler, unit_scout, unit_skirmisher, unit_archer, unit_berserker, unit_shaman]

	var initial_combatants: Array[Combatant] = [
		unit_warrior, unit_ranger, unit_brawler, unit_scout,
		unit_skirmisher, unit_archer, unit_berserker, unit_shaman
	]
	for c: Combatant in initial_combatants:
		sim_resolver.register_combatant(c)

	# 3. Center-ring deployment for initial combatants only
	sim_grid.deploy_combatants(sim_allies, [unit_brawler, unit_scout])

	# 4. Schedule Pre-Planned Commands for Round 1
	# Warrior:
	#   Block 1: Move Up to Center (4, 4) [1b]
	#   Block 2-3: Melee Attack on Brawler [2b]
	#   Block 4: Target Goblin Berserker [1b]
	#   Block 5-6: Attack Goblin Berserker (25 power) [2b]
	#   Block 7-8: Defend (50% mitigation against ambush) [2b]
	#   Block 9-10: Counter-Attack Goblin Skirmisher [2b]
	sim_resolver.schedule_action(unit_warrior, MoveCommand.new(unit_warrior, sim_grid, Vector2i(0, -1), 1), 1)
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_brawler, 25, 2, 0, sim_grid), 2)
	sim_resolver.schedule_action(unit_warrior, TargetCommand.new(unit_warrior, unit_skirmisher, 1), 4)
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_skirmisher, 25, 2, 0, sim_grid), 5)
	sim_resolver.schedule_action(unit_warrior, DefendCommand.new(unit_warrior, 0.5, 2, 1), 7)
	sim_resolver.schedule_action(unit_warrior, HoldStanceCommand.new(unit_warrior, 0), 9)

	# Ranger:
	#   Block 1-2: Skill - Focus (+50% ATK for 2 rds) [2b]
	#   Block 4: Move Up to (4, 5) [1b]
	#   Block 5-6: Boosted Ranged Attack on Brawler (resolves Block 6 -> Brawler defeated!) [2b]
	#   Block 7-8: Attack Archer (cancelled when Ranger falls to ambush at Block 6) [2b]
	sim_resolver.schedule_action(unit_ranger, FocusCommand.new(unit_ranger, 1.5, 2, 2, 0), 1)
	sim_resolver.schedule_action(unit_ranger, MoveCommand.new(unit_ranger, sim_grid, Vector2i(0, -1), 1), 4)
	sim_resolver.schedule_action(unit_ranger, AttackCommand.new(unit_ranger, unit_brawler, 30, 2, 0, sim_grid), 5)
	sim_resolver.schedule_action(unit_ranger, AttackCommand.new(unit_ranger, unit_archer, 25, 2, 0, sim_grid), 7)

	# Enemy Brawler (Scheduled in secret):
	#   Block 1: Target Warrior [1b]
	#   Block 2-3: Defend (50% mitigation) [2b]
	#   Block 4-5: Counter-Attack Warrior (15 power) [2b]
	sim_resolver.schedule_action(unit_brawler, TargetCommand.new(unit_brawler, unit_warrior, 2), 1)
	sim_resolver.schedule_action(unit_brawler, DefendCommand.new(unit_brawler, 0.5, 2, 1), 2)
	sim_resolver.schedule_action(unit_brawler, AttackCommand.new(unit_brawler, unit_warrior, 15, 2, 0, sim_grid), 4)

	# Fleeing Scout (Scheduled in secret):
	#   Block 1: Move East to (6, 4) [1b]
	#   Block 2-3: Sprint East 2 steps to (8, 4) Exit Tile -> Safely Withdraws! [2b]
	sim_resolver.schedule_action(unit_scout, MoveCommand.new(unit_scout, sim_grid, Vector2i(1, 0), 1), 1)
	sim_resolver.schedule_action(unit_scout, MoveCommand.new(unit_scout, sim_grid, Vector2i(1, 0), 2, 2), 2)

	# Ambush Reinforcements (Scheduled for Blocks 4-10):
	# Berserker (spawns (5, 5)):
	sim_resolver.schedule_action(unit_berserker, TargetCommand.new(unit_berserker, unit_ranger, 1), 4)
	sim_resolver.schedule_action(unit_berserker, AttackCommand.new(unit_berserker, unit_ranger, 30, 2, 0, sim_grid), 5)

	# Archer (spawns (7, 5)):
	sim_resolver.schedule_action(unit_archer, TargetCommand.new(unit_archer, unit_ranger, 1), 4)
	sim_resolver.schedule_action(unit_archer, AttackCommand.new(unit_archer, unit_ranger, 25, 2, 0, sim_grid), 5)

	# Shaman (spawns (6, 5)):
	sim_resolver.schedule_action(unit_shaman, TargetCommand.new(unit_shaman, unit_ranger, 1), 4)
	sim_resolver.schedule_action(unit_shaman, AttackCommand.new(unit_shaman, unit_ranger, 25, 2, 0, sim_grid), 5)

	# Skirmisher (spawns (6, 4)):
	sim_resolver.schedule_action(unit_skirmisher, MoveCommand.new(unit_skirmisher, sim_grid, Vector2i(-1, 0), 1), 4)
	sim_resolver.schedule_action(unit_skirmisher, TargetCommand.new(unit_skirmisher, unit_warrior, 1), 5)
	sim_resolver.schedule_action(unit_skirmisher, AttackCommand.new(unit_skirmisher, unit_warrior, 12, 2, 0, sim_grid), 7)

	# 5. Clone All Scheduled Actions into Permanent Archive
	recorded_actions.clear()
	for a in sim_resolver.scheduled_actions:
		recorded_actions.append(CombatResolver.ScheduledAction.new(a.combatant, a.command, a.start_block, a.end_block))
	round_recorded_actions[1] = recorded_actions.duplicate()

	# 6. Build visual unit tokens and roster cards
	for ally in sim_allies:
		_create_unit_token(ally, _get_combatant_token_texture(ally))
		_create_roster_card(ally, Color(0.2, 0.5, 0.9))
	_add_party_placeholders()

	for enemy in sim_enemies:
		_create_unit_token(enemy, _get_combatant_token_texture(enemy))
		_create_roster_card(enemy, Color(0.9, 0.3, 0.3))
		# Goblin ambush reinforcements are invisible until they appear
		if enemy in [unit_berserker, unit_skirmisher, unit_archer, unit_shaman]:
			if roster_cards.has(enemy):
				@warning_ignore("unsafe_cast")
				var enemy_card := roster_cards[enemy] as Control
				if is_instance_valid(enemy_card):
					enemy_card.visible = false
	_add_enemy_placeholders()

	_update_all_roster_cards()

	# 7. Capture Initial Snapshot (State 0 before block 1 executes)
	snapshots.clear()
	_capture_snapshot(0)
	round_snapshots[1] = snapshots.duplicate()
	_update_round_traversal_ui()

	# 8. Setup 2x2 Party Focus Box & Default Selection
	selected_party_hero = unit_warrior
	_update_party_select_box()

	# Set default selected unit to Warrior
	_select_combatant(unit_warrior)
	_update_timeline_tracks_display()
	_update_combined_block_display(1)


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

	token_ctrl.mouse_entered.connect(func() -> void:
		_show_unit_hover_card(combatant, token_ctrl)
	)
	token_ctrl.mouse_exited.connect(func() -> void:
		_hide_unit_hover_card()
	)

	overlay.add_child(token_ctrl)
	unit_token_nodes[combatant] = token_ctrl

	var pos: Vector2i = sim_grid.get_position(combatant)
	if pos == Vector2i(-1, -1) or not combatant.is_alive() or combatant.is_withdrawn:
		token_ctrl.visible = false
		token_ctrl.modulate.a = 0.0
		token_ctrl.position = Vector2(0, 0)
	else:
		token_ctrl.visible = true
		token_ctrl.modulate.a = 1.0
		token_ctrl.position = Vector2(pos.x * TILE_SIZE, pos.y * TILE_SIZE)


func _show_unit_hover_card(c: Combatant, token: Control) -> void:
	if grid_hover_tooltip == null or not is_instance_valid(c):
		return
	var is_ally: bool = sim_allies.has(c)
	var col: Color = Color(0.4, 0.75, 1.0) if is_ally else Color(1.0, 0.45, 0.45)
	hover_name_lbl.text = c.name
	hover_name_lbl.add_theme_color_override("font_color", col)

	var cur_snap: BlockSnapshot = null
	var snaps: Array = round_snapshots.get(viewed_timeline_round, snapshots)
	var cur_b: int = replay_block if combat_phase == CombatPhase.REPLAY else current_sim_block
	if cur_b >= 0 and cur_b < snaps.size():
		@warning_ignore("unsafe_cast")
		cur_snap = snaps[cur_b] as BlockSnapshot

	var cur_hp: int = cur_snap.hps.get(c, c.hp) if cur_snap != null else c.hp
	var max_hp: int = cur_snap.max_hps.get(c, c.max_hp) if cur_snap != null else c.max_hp
	var is_withdrawn: bool = cur_snap.is_withdrawn.get(c, c.is_withdrawn) if cur_snap != null else c.is_withdrawn
	var is_alive: bool = cur_hp > 0
	var cur_stance: CombatTypes.StanceType = cur_snap.stances.get(c, c.current_stance) if cur_snap != null else c.current_stance
	var pos: Vector2i = cur_snap.positions.get(c, sim_grid.get_position(c)) if cur_snap != null else sim_grid.get_position(c)

	if not is_alive:
		hover_hp_lbl.text = "HP: 0/%d [DEFEATED]" % max_hp
		hover_status_lbl.text = "Status: Fallen in battle"
	elif is_withdrawn:
		hover_hp_lbl.text = "HP: %d/%d [ESCAPED]" % [cur_hp, max_hp]
		hover_status_lbl.text = "Status: Withdrawn via Exit Tile"
	else:
		hover_hp_lbl.text = "HP: %d/%d | Spd: %d | Rng: %d" % [cur_hp, max_hp, c.speed, c.attack_range]
		hover_status_lbl.text = "Stance: %s | Pos: %s" % [CombatTypes.stance_to_string(cur_stance), str(pos)]

	# Position adjacent to token within grid boundaries
	var target_pos := token.position + Vector2(TILE_SIZE + 4, -4)
	if target_pos.x + 140 > GRID_SIZE * TILE_SIZE:
		target_pos.x = maxf(0.0, token.position.x - 142)
	if target_pos.y + 55 > GRID_SIZE * TILE_SIZE:
		target_pos.y = maxf(0.0, token.position.y - 45)
	grid_hover_tooltip.position = target_pos
	grid_hover_tooltip.z_index = 50
	grid_hover_tooltip.visible = true


func _hide_unit_hover_card() -> void:
	if grid_hover_tooltip != null:
		grid_hover_tooltip.visible = false


func _create_roster_card(combatant: Combatant, team_color: Color) -> void:
	var is_ally: bool = sim_allies.has(combatant)
	var target_container: GridContainer = party_cards_container if is_ally else enemy_cards_container
	if not target_container:
		return

	var card := PanelContainer.new()
	card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	card.size_flags_horizontal = Control.SIZE_EXPAND_FILL

	var card_style := StyleBoxFlat.new()
	card_style.bg_color = Color(0.10, 0.13, 0.18, 0.92)
	card_style.border_color = team_color
	card_style.set_border_width_all(1)
	card_style.set_corner_radius_all(3)
	card_style.content_margin_left = 3
	card_style.content_margin_right = 3
	card_style.content_margin_top = 1
	card_style.content_margin_bottom = 1
	card.add_theme_stylebox_override("panel", card_style)

	if is_ally:
		card.custom_minimum_size = Vector2(130, 26)
		var main_vbox := VBoxContainer.new()
		main_vbox.add_theme_constant_override("separation", 1)
		card.add_child(main_vbox)

		# Top row: Name & HP fraction
		var top_row := HBoxContainer.new()
		top_row.add_theme_constant_override("separation", 2)
		main_vbox.add_child(top_row)

		var name_lbl := Label.new()
		name_lbl.text = combatant.name
		name_lbl.clip_text = true
		name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		name_lbl.add_theme_font_size_override("font_size", 8)
		name_lbl.add_theme_color_override("font_color", team_color.lightened(0.35))
		top_row.add_child(name_lbl)

		var hp_lbl := Label.new()
		hp_lbl.name = "HpLabel"
		hp_lbl.text = "%d/%d" % [combatant.hp, combatant.max_hp]
		hp_lbl.add_theme_font_size_override("font_size", 7)
		hp_lbl.add_theme_color_override("font_color", Color(0.85, 0.9, 0.95))
		top_row.add_child(hp_lbl)

		# Bottom row: HP bar, Stance, Pos
		var bot_row := HBoxContainer.new()
		bot_row.add_theme_constant_override("separation", 2)
		main_vbox.add_child(bot_row)

		var hp_bar := ProgressBar.new()
		hp_bar.custom_minimum_size = Vector2(34, 4)
		hp_bar.max_value = combatant.max_hp
		hp_bar.value = combatant.hp
		hp_bar.show_percentage = false
		bot_row.add_child(hp_bar)

		var stance_lbl := Label.new()
		stance_lbl.name = "StanceLabel"
		stance_lbl.clip_text = true
		stance_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		stance_lbl.text = CombatTypes.stance_to_string(combatant.current_stance)
		stance_lbl.add_theme_font_size_override("font_size", 7)
		stance_lbl.add_theme_color_override("font_color", Color(0.65, 0.78, 0.9))
		bot_row.add_child(stance_lbl)

		var pos_lbl := Label.new()
		pos_lbl.name = "PosLabel"
		pos_lbl.text = "%s" % str(sim_grid.get_position(combatant))
		pos_lbl.add_theme_font_size_override("font_size", 7)
		pos_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		bot_row.add_child(pos_lbl)
	else:
		card.custom_minimum_size = Vector2(94, 26)
		var main_vbox := VBoxContainer.new()
		main_vbox.add_theme_constant_override("separation", 1)
		card.add_child(main_vbox)

		# Top row: Truncated name & HP
		var top_row := HBoxContainer.new()
		top_row.add_theme_constant_override("separation", 2)
		main_vbox.add_child(top_row)

		var short_name: String = combatant.name
		if short_name.begins_with("Goblin "):
			short_name = short_name.substr(7)
		elif short_name.begins_with("Enemy "):
			short_name = short_name.substr(6)
		elif short_name.begins_with("Fleeing "):
			short_name = short_name.substr(8)

		var name_lbl := Label.new()
		name_lbl.text = short_name
		name_lbl.clip_text = true
		name_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		name_lbl.add_theme_font_size_override("font_size", 8)
		name_lbl.add_theme_color_override("font_color", team_color.lightened(0.35))
		top_row.add_child(name_lbl)

		var hp_lbl := Label.new()
		hp_lbl.name = "HpLabel"
		hp_lbl.text = "%d" % combatant.hp
		hp_lbl.add_theme_font_size_override("font_size", 7)
		hp_lbl.add_theme_color_override("font_color", Color(0.9, 0.75, 0.75))
		top_row.add_child(hp_lbl)

		# Bottom row: Mini HP bar, status
		var bot_row := HBoxContainer.new()
		bot_row.add_theme_constant_override("separation", 2)
		main_vbox.add_child(bot_row)

		var hp_bar := ProgressBar.new()
		hp_bar.custom_minimum_size = Vector2(26, 4)
		hp_bar.max_value = combatant.max_hp
		hp_bar.value = combatant.hp
		hp_bar.show_percentage = false
		bot_row.add_child(hp_bar)

		var stance_lbl := Label.new()
		stance_lbl.name = "StanceLabel"
		stance_lbl.clip_text = true
		stance_lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		var st_str: String = CombatTypes.stance_to_string(combatant.current_stance)
		stance_lbl.text = st_str.left(3)
		stance_lbl.add_theme_font_size_override("font_size", 7)
		stance_lbl.add_theme_color_override("font_color", Color(0.7, 0.65, 0.65))
		bot_row.add_child(stance_lbl)

		var pos_lbl := Label.new()
		pos_lbl.name = "PosLabel"
		pos_lbl.visible = false
		bot_row.add_child(pos_lbl)

	card.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
			_select_combatant(combatant)
	)

	target_container.add_child(card)
	roster_cards[combatant] = card


func _add_party_placeholders() -> void:
	if not party_cards_container:
		return
	while party_cards_container.get_child_count() < 4:
		var slot_idx: int = party_cards_container.get_child_count() + 1
		var ph := PanelContainer.new()
		ph.custom_minimum_size = Vector2(130, 26)
		var ph_style := StyleBoxFlat.new()
		ph_style.bg_color = Color(0.06, 0.08, 0.12, 0.35)
		ph_style.border_color = Color(0.2, 0.28, 0.4, 0.35)
		ph_style.set_border_width_all(1)
		ph_style.set_corner_radius_all(3)
		ph.add_theme_stylebox_override("panel", ph_style)

		var lbl := Label.new()
		lbl.text = "[+ Hero Slot %d]" % slot_idx
		lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		lbl.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		lbl.add_theme_font_size_override("font_size", 8)
		lbl.add_theme_color_override("font_color", Color(0.35, 0.45, 0.55))
		ph.add_child(lbl)
		party_cards_container.add_child(ph)


func _add_enemy_placeholders() -> void:
	# Active roster only: reserve placeholders removed so only active enemies appear
	pass


# ==============================================================================
# SECTION 3: SNAPSHOTS, REPLAY RESTORATION & UNIFIED ROSTER UPDATES
# ==============================================================================

func _capture_snapshot(block_idx: int) -> void:
	var snap := BlockSnapshot.new()
	snap.block_index = block_idx
	snap.round_num = current_combat_round
	snap.reinforcements_spawned = reinforcements_spawned
	for c: Combatant in sim_resolver.registered_combatants:
		snap.positions[c] = sim_grid.get_position(c)
		snap.hps[c] = c.hp
		snap.max_hps[c] = c.max_hp
		snap.stances[c] = c.current_stance
		snap.targets[c] = c.active_target
		snap.is_active[c] = c.is_active()
		snap.is_withdrawn[c] = c.is_withdrawn
		snap.attack_boost_mult[c] = c.attack_boost_multiplier
		snap.attack_boost_rds[c] = c.attack_boost_rounds_remaining
		snap.defense_mit[c] = c.defense_mitigation_ratio
	snapshots.append(snap)
	round_snapshots[current_combat_round] = snapshots.duplicate()


func _restore_snapshot(snap: BlockSnapshot) -> void:
	reinforcements_spawned = snap.reinforcements_spawned
	for c: Combatant in unit_token_nodes:
		@warning_ignore("unsafe_cast")
		var token := unit_token_nodes[c] as Control
		if snap.positions.has(c):
			var pos: Vector2i = snap.positions[c]
			var active: bool = snap.is_active.get(c, false)
			var withdrawn: bool = snap.is_withdrawn.get(c, false)
			var hp_val: int = snap.hps.get(c, c.hp)
			var is_alive: bool = hp_val > 0

			# Requirement 7: Remove dead or withdrawn units from board completely to eliminate token overlap
			if pos == Vector2i(-1, -1) or not is_alive or withdrawn or not active:
				token.visible = false
				token.modulate.a = 0.0
			else:
				token.visible = true
				token.modulate = Color.WHITE
				var target_px := Vector2(pos.x * TILE_SIZE, pos.y * TILE_SIZE)
				var tw := create_tween()
				tw.tween_property(token, "position", target_px, 0.2).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)

	_update_all_roster_cards(snap)
	_update_selected_unit_panel()
	_update_tactical_forecast_panel()


func _update_all_roster_cards(snap: BlockSnapshot = null) -> void:
	var alive_enemies: int = 0
	var is_r1: bool = (snap.round_num == 1 if snap != null else current_combat_round == 1)
	var reinforcements_active: bool = (snap.reinforcements_spawned if snap != null else reinforcements_spawned)

	for combatant: Combatant in roster_cards:
		@warning_ignore("unsafe_cast")
		var card := roster_cards[combatant] as PanelContainer
		if not is_instance_valid(card):
			continue

		var is_ally: bool = sim_allies.has(combatant)
		var cur_hp: int = snap.hps.get(combatant, combatant.hp) if snap != null else combatant.hp
		var max_hp: int = snap.max_hps.get(combatant, combatant.max_hp) if snap != null else combatant.max_hp
		var is_withdrawn: bool = snap.is_withdrawn.get(combatant, combatant.is_withdrawn) if snap != null else combatant.is_withdrawn
		var is_alive: bool = (cur_hp > 0)
		var is_act: bool = is_alive and not is_withdrawn
		var cur_stance: CombatTypes.StanceType = snap.stances.get(combatant, combatant.current_stance) if snap != null else combatant.current_stance
		var pos: Vector2i = snap.positions.get(combatant, sim_grid.get_position(combatant)) if snap != null else sim_grid.get_position(combatant)

		# In Round 1, ambush goblins are only visible after they appear
		if not is_ally and combatant in [unit_berserker, unit_skirmisher, unit_archer, unit_shaman]:
			if is_r1:
				card.visible = reinforcements_active
			else:
				card.visible = true

		if not is_ally and card.visible and is_act:
			alive_enemies += 1

		var hp_bar := card.find_child("ProgressBar", true, false) as ProgressBar
		var hp_lbl := card.find_child("HpLabel", true, false) as Label
		var stance_lbl := card.find_child("StanceLabel", true, false) as Label
		var pos_lbl := card.find_child("PosLabel", true, false) as Label

		if hp_bar:
			hp_bar.max_value = max_hp
			hp_bar.value = cur_hp
		if hp_lbl:
			if is_ally:
				hp_lbl.text = "%d/%d" % [cur_hp, max_hp]
			else:
				hp_lbl.text = "%d" % cur_hp
		if stance_lbl:
			if not is_alive:
				stance_lbl.text = "DEFEATED"
				stance_lbl.add_theme_color_override("font_color", Color(0.85, 0.35, 0.35))
			elif is_withdrawn:
				stance_lbl.text = "ESCAPED"
				stance_lbl.add_theme_color_override("font_color", Color(0.95, 0.75, 0.25))
			else:
				stance_lbl.text = CombatTypes.stance_to_string(cur_stance) if is_ally else CombatTypes.stance_to_string(cur_stance).left(3)
				stance_lbl.add_theme_color_override("font_color", Color(0.65, 0.78, 0.9) if is_ally else Color(0.7, 0.65, 0.65))

		if pos_lbl:
			if is_act:
				pos_lbl.text = "%s" % str(pos)
			else:
				pos_lbl.text = "Off"

		# Dimmer appearance for dead or escaped units (both party and enemy)
		if not is_alive:
			card.modulate = Color(0.55, 0.55, 0.55, 0.45)
		elif is_withdrawn:
			card.modulate = Color(0.65, 0.65, 0.65, 0.45)
		else:
			card.modulate = Color.WHITE

	if enemy_title_lbl:
		enemy_title_lbl.text = "💀 ENEMY HOSTILES (%d Active)" % alive_enemies
	if party_title_lbl:
		party_title_lbl.text = "⚔ PARTY MEMBERS (%d/4)" % sim_allies.size()


# ==============================================================================
# SECTION 4: SELECTION & DYNAMIC INSPECTOR UPDATES
# ==============================================================================

func _select_combatant(combatant: Combatant) -> void:
	selected_combatant = combatant
	if sim_allies.has(combatant):
		selected_party_hero = combatant
	_update_roster_card_highlights()
	_update_unit_token_highlights()
	_update_selected_unit_panel()
	_update_tactical_forecast_panel()
	_update_party_select_box()
	_update_timeline_tracks_display()
	_update_combined_block_display(combined_selected_block)


func _update_roster_card_highlights() -> void:
	for c: Combatant in roster_cards:
		@warning_ignore("unsafe_cast")
		var card := roster_cards[c] as PanelContainer
		if not is_instance_valid(card):
			continue
		var style := StyleBoxFlat.new()
		style.set_corner_radius_all(3)
		style.content_margin_left = 4
		style.content_margin_right = 4
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

		# Preserve dimming for dead / escaped units
		if not c.is_alive():
			card.modulate = Color(0.55, 0.55, 0.55, 0.45)
		elif c.is_withdrawn:
			card.modulate = Color(0.65, 0.65, 0.65, 0.45)
		else:
			card.modulate = Color.WHITE


func _update_unit_token_highlights() -> void:
	for c: Combatant in unit_token_nodes:
		@warning_ignore("unsafe_cast")
		var token := unit_token_nodes[c] as Control
		var border := token.find_child("SelectionBorder", true, false) as ReferenceRect
		if border:
			border.visible = (c == selected_combatant)


func _update_selected_unit_panel() -> void:
	if not selected_combatant:
		return

	inspector_name_lbl.text = selected_combatant.name
	var is_ally: bool = sim_allies.has(selected_combatant)

	if is_ally:
		inspector_hdr.text = "UNIT INSPECTOR & ACTION SETUP"
		inspector_team_lbl.text = "Allied Hero (Party Member)"
		inspector_team_lbl.add_theme_color_override("font_color", Color(0.4, 0.75, 1.0))
		inspector_avatar.texture = _get_combatant_sprite_texture(selected_combatant)
		ally_actions_container.visible = selected_combatant.is_alive()
		enemy_actions_container.visible = false
		if not selected_combatant.is_alive():
			left_hint.text = "💀 Unit is defeated. Future commands cancelled."
		else:
			left_hint.text = "💡 Actions for selected hero line up on continuous timeline below."
	else:
		inspector_hdr.text = "ENEMY INTEL & POSSIBLE ACTIONS"
		inspector_team_lbl.text = "Hostile Enemy Unit"
		inspector_team_lbl.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
		inspector_avatar.texture = _get_combatant_sprite_texture(selected_combatant)
		ally_actions_container.visible = false
		enemy_actions_container.visible = selected_combatant.is_alive()
		_rebuild_enemy_threat_dock()
		if not selected_combatant.is_alive():
			left_hint.text = "💀 Enemy hostile defeated."
		elif selected_combatant.is_withdrawn:
			left_hint.text = "💨 Enemy safely withdrew through escape boundary."
		else:
			left_hint.text = "💡 In actual combat, enemy turns resolve simultaneously in secret."

	inspector_hp_bar.max_value = selected_combatant.max_hp
	inspector_hp_bar.value = selected_combatant.hp
	inspector_hp_lbl.text = "%d/%d" % [selected_combatant.hp, selected_combatant.max_hp]

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
		var tgt: Combatant = selected_combatant.active_target as Combatant
		inspector_target_lbl.text = "Target: " + tgt.name
	else:
		inspector_target_lbl.text = "Target: None"

	if selected_combatant.attack_boost_rounds_remaining > 0:
		inspector_buffs_lbl.text = "Buffs: Focus (+50%% ATK, %d rds)" % selected_combatant.attack_boost_rounds_remaining
	else:
		inspector_buffs_lbl.text = "Buffs: None active"


func _rebuild_enemy_threat_dock() -> void:
	for child in enemy_actions_container.get_children():
		if child is PanelContainer:
			child.queue_free()

	var enemy_actions: Array[Dictionary] = []
	match selected_combatant.name:
		"Enemy Brawler":
			enemy_actions = [
				{"name": "⚔ Heavy Strike", "desc": "20 Power Melee Crush | Cost: 2 Blocks", "cost": 2, "color": Color(0.9, 0.35, 0.35)},
				{"name": "🛡 Iron Guard", "desc": "50% Mitigation Barrier | Cost: 2 Blocks", "cost": 2, "color": Color(0.35, 0.65, 0.95)},
				{"name": "🎯 Target Lock", "desc": "Melee Focus Engagement | Cost: 1 Block", "cost": 1, "color": Color(0.35, 0.85, 0.5)},
				{"name": "➜ Tactical Stride", "desc": "1-Tile Repositioning | Cost: 1 Block", "cost": 1, "color": Color(0.2, 0.8, 0.9)}
			]
		"Fleeing Scout", "Goblin Scout":
			enemy_actions = [
				{"name": "➜ Swift Retreat", "desc": "1-Tile Move toward Exit | Cost: 1 Block", "cost": 1, "color": Color(0.2, 0.8, 0.9)},
				{"name": "⚡ Sprint / Dash", "desc": "2-Tile Burst toward Boundary | Cost: 2 Blocks", "cost": 2, "color": Color(0.9, 0.8, 0.3)},
				{"name": "🗡 Quick Shank", "desc": "10 Power Puncture | Cost: 1 Block", "cost": 1, "color": Color(0.9, 0.35, 0.35)}
			]
		"Goblin Skirmisher":
			enemy_actions = [
				{"name": "🗡 Twin Dagger Rush", "desc": "15 Power Rapid Flank | Cost: 1 Block", "cost": 1, "color": Color(0.95, 0.6, 0.2)},
				{"name": "➜ Shadow Step", "desc": "Agile 1-Tile Advance | Cost: 1 Block", "cost": 1, "color": Color(0.2, 0.8, 0.9)},
				{"name": "⚔ Flank Strike", "desc": "20 Power Heavy Puncture | Cost: 2 Blocks", "cost": 2, "color": Color(0.9, 0.35, 0.35)}
			]
		"Goblin Archer":
			enemy_actions = [
				{"name": "🏹 Piercing Volley", "desc": "25 Power Longshot (Rng 3) | Cost: 2 Blocks", "cost": 2, "color": Color(0.2, 0.85, 0.5)},
				{"name": "🎯 Eagle Eye", "desc": "Target Lock & Ranged Focus | Cost: 1 Block", "cost": 1, "color": Color(0.35, 0.85, 0.5)},
				{"name": "➜ Reposition", "desc": "Kiting 1-Tile Step | Cost: 1 Block", "cost": 1, "color": Color(0.2, 0.8, 0.9)}
			]
		"Goblin Berserker":
			enemy_actions = [
				{"name": "🪓 Brutal Cleave", "desc": "30 Power Devastating Blow | Cost: 2 Blocks", "cost": 2, "color": Color(0.95, 0.25, 0.25)},
				{"name": "🔥 Blood Frenzy", "desc": "+25% Attack Boost Buff | Cost: 2 Blocks", "cost": 2, "color": Color(0.95, 0.5, 0.2)},
				{"name": "⚔ Axe Slam", "desc": "20 Power Heavy Chop | Cost: 2 Blocks", "cost": 2, "color": Color(0.9, 0.35, 0.35)}
			]
		"Goblin Shaman":
			enemy_actions = [
				{"name": "🔮 Shadow Hex", "desc": "25 Power Dark Curse (Rng 2) | Cost: 2 Blocks", "cost": 2, "color": Color(0.7, 0.4, 0.95)},
				{"name": "💀 Tribal Ward", "desc": "50% Mystic Barrier | Cost: 2 Blocks", "cost": 2, "color": Color(0.35, 0.65, 0.95)},
				{"name": "✨ Mending Chant", "desc": "Restores 15 HP to Ally | Cost: 2 Blocks", "cost": 2, "color": Color(0.3, 0.9, 0.6)}
			]
		_:
			enemy_actions = [
				{"name": "⚔ Attack", "desc": "Basic Strike | Cost: 2 Blocks", "cost": 2, "color": Color(0.9, 0.35, 0.35)}
			]

	for act in enemy_actions:
		var card := _create_mini_action_card(act)
		enemy_actions_container.add_child(card)


func _update_tactical_forecast_panel() -> void:
	if not selected_combatant:
		return

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


# ==============================================================================
# SECTION 5: 2x2 PARTY SELECT BOX & CONTINUOUS ACTIVE TRACK RENDERING
# ==============================================================================

func _build_party_select_box() -> PanelContainer:
	var pnl := PanelContainer.new()
	pnl.custom_minimum_size = Vector2(124, 72)
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.08, 0.11, 0.16, 0.95)
	style.border_color = Color(0.2, 0.35, 0.55, 0.8)
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.content_margin_left = 4
	style.content_margin_right = 4
	style.content_margin_top = 3
	style.content_margin_bottom = 3
	pnl.add_theme_stylebox_override("panel", style)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 2)
	pnl.add_child(vbox)

	var hdr_hbox := HBoxContainer.new()
	vbox.add_child(hdr_hbox)

	var hdr_lbl := Label.new()
	hdr_lbl.text = "HERO FOCUS"
	hdr_lbl.add_theme_font_size_override("font_size", 8)
	hdr_lbl.add_theme_color_override("font_color", Color(0.4, 0.75, 1.0))
	hdr_hbox.add_child(hdr_lbl)

	hdr_hbox.add_spacer(false)

	btn_party_all = Button.new()
	btn_party_all.text = "ALL"
	btn_party_all.tooltip_text = "Show All Combatants in Combined View"
	btn_party_all.custom_minimum_size = Vector2(28, 14)
	btn_party_all.add_theme_font_size_override("font_size", 7)
	btn_party_all.pressed.connect(_on_party_all_clicked)
	hdr_hbox.add_child(btn_party_all)

	party_select_grid = GridContainer.new()
	party_select_grid.columns = 2
	party_select_grid.add_theme_constant_override("h_separation", 3)
	party_select_grid.add_theme_constant_override("v_separation", 3)
	vbox.add_child(party_select_grid)

	party_slot_panels.clear()
	for i in range(4):
		var slot_pnl := PanelContainer.new()
		slot_pnl.custom_minimum_size = Vector2(56, 23)
		slot_pnl.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
		var cur_i: int = i
		slot_pnl.gui_input.connect(func(event: InputEvent) -> void:
			var me := event as InputEventMouseButton
			if me != null and me.pressed and me.button_index == MOUSE_BUTTON_LEFT:
				_on_party_slot_clicked(cur_i)
		)
		party_select_grid.add_child(slot_pnl)
		party_slot_panels.append(slot_pnl)

	return pnl


func _update_party_select_box() -> void:
	if party_slot_panels.is_empty():
		return

	# Style for "ALL" button
	if selected_party_hero == null:
		btn_party_all.add_theme_color_override("font_color", Color(1.0, 0.85, 0.25))
	else:
		btn_party_all.remove_theme_color_override("font_color")

	for i in range(4):
		var slot_pnl: PanelContainer = party_slot_panels[i]
		for child in slot_pnl.get_children():
			child.queue_free()

		var slot_style := StyleBoxFlat.new()
		slot_style.set_corner_radius_all(3)
		slot_style.content_margin_left = 3
		slot_style.content_margin_right = 3
		slot_style.content_margin_top = 2
		slot_style.content_margin_bottom = 2

		var ally: Combatant = sim_allies[i] if i < sim_allies.size() else null

		if ally != null:
			var is_selected: bool = (ally == selected_party_hero)
			var is_alive: bool = ally.is_alive()
			if is_selected:
				slot_style.bg_color = Color(0.18, 0.26, 0.38, 0.98)
				slot_style.border_color = Color(1.0, 0.85, 0.25, 1.0)
				slot_style.set_border_width_all(2)
			elif not is_alive:
				slot_style.bg_color = Color(0.14, 0.08, 0.08, 0.9)
				slot_style.border_color = Color(0.7, 0.25, 0.25, 0.8)
				slot_style.set_border_width_all(1)
			else:
				slot_style.bg_color = Color(0.09, 0.12, 0.17, 0.9)
				slot_style.border_color = Color(0.2, 0.4, 0.65, 0.7)
				slot_style.set_border_width_all(1)

			slot_pnl.modulate = Color(0.6, 0.6, 0.6, 0.5) if not is_alive else Color.WHITE

			var hbox := HBoxContainer.new()
			hbox.add_theme_constant_override("separation", 3)
			slot_pnl.add_child(hbox)

			var icon := TextureRect.new()
			icon.texture = _get_combatant_token_texture(ally)
			icon.custom_minimum_size = Vector2(16, 16)
			icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
			icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
			if not is_alive:
				icon.modulate = Color(0.5, 0.5, 0.5, 0.6)
			hbox.add_child(icon)

			var lbl := Label.new()
			if not is_alive:
				lbl.text = "%s 💀" % ally.name
				lbl.add_theme_color_override("font_color", Color(0.95, 0.35, 0.35))
			else:
				lbl.text = ally.name
				lbl.add_theme_color_override("font_color", Color(1.0, 0.85, 0.25) if is_selected else Color(0.85, 0.92, 1.0))
			lbl.add_theme_font_size_override("font_size", 7)
			hbox.add_child(lbl)
		else:
			slot_pnl.modulate = Color.WHITE
			slot_style.bg_color = Color(0.06, 0.08, 0.11, 0.6)
			slot_style.border_color = Color(0.18, 0.24, 0.32, 0.5)
			slot_style.set_border_width_all(1)

			var lbl := Label.new()
			lbl.text = "+ Slot %d" % (i + 1)
			lbl.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
			lbl.add_theme_font_size_override("font_size", 7)
			lbl.add_theme_color_override("font_color", Color(0.4, 0.48, 0.58))
			slot_pnl.add_child(lbl)

		slot_pnl.add_theme_stylebox_override("panel", slot_style)


func _on_party_slot_clicked(slot_idx: int) -> void:
	if slot_idx >= sim_allies.size():
		return

	var ally: Combatant = sim_allies[slot_idx]
	if selected_party_hero == ally:
		selected_party_hero = null
	else:
		selected_party_hero = ally
		selected_combatant = ally

	_update_party_select_box()
	_update_timeline_tracks_display()
	_update_combined_block_display(combined_selected_block)
	_update_selected_unit_panel()
	_update_tactical_forecast_panel()
	_update_roster_card_highlights()
	_update_unit_token_highlights()


func _on_party_all_clicked() -> void:
	selected_party_hero = null
	_update_party_select_box()
	_update_combined_block_display(combined_selected_block)


func _update_timeline_tracks_display() -> void:
	# Update Ruler highlights
	var active_playhead: int = replay_block if combat_phase == CombatPhase.REPLAY else current_sim_block
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

	_update_party_select_box()

	# Check if selected unit is enemy during planning phase
	var is_enemy_selected: bool = selected_combatant != null and not sim_allies.has(selected_combatant)
	if is_enemy_selected and combat_phase != CombatPhase.REPLAY:
		enemy_classified_banner.visible = true
		single_track_row.visible = false
		return
	else:
		enemy_classified_banner.visible = false
		single_track_row.visible = true

	# Render active track for target unit
	var target_unit: Combatant = null
	if is_enemy_selected and combat_phase == CombatPhase.REPLAY:
		target_unit = selected_combatant
	elif selected_party_hero != null:
		target_unit = selected_party_hero
	elif selected_combatant != null and sim_allies.has(selected_combatant):
		target_unit = selected_combatant
	elif not sim_allies.is_empty():
		target_unit = sim_allies[0]

	if target_unit == null:
		return

	# Rebuild continuous clips (Starts at X=0, exactly under Ruler BLK 01..10)
	for child in track_clips_hbox.get_children():
		child.queue_free()

	var b: int = 1
	while b <= TOTAL_BLOCKS:
		var act: CombatResolver.ScheduledAction = _find_recorded_action_starting_at(target_unit, b)
		if act != null:
			var cost: int = act.end_block - act.start_block + 1
			var clip_w: int = cost * UNIT_BLOCK_W + (cost - 1) * BLOCK_GAP
			var clip := _create_action_clip_widget(act, clip_w, cost)
			track_clips_hbox.add_child(clip)
			b += cost
		else:
			var idle_slot := _create_idle_clip_widget(b == active_playhead)
			track_clips_hbox.add_child(idle_slot)
			b += 1


func _find_recorded_action_starting_at(combatant: Combatant, block: int) -> CombatResolver.ScheduledAction:
	for a: CombatResolver.ScheduledAction in recorded_actions:
		if a.combatant == combatant and a.start_block == block:
			return a
	return null


func _get_recorded_actions_at_block(block: int) -> Array[CombatResolver.ScheduledAction]:
	var res: Array[CombatResolver.ScheduledAction] = []
	for a: CombatResolver.ScheduledAction in recorded_actions:
		if a.start_block <= block and a.end_block >= block:
			res.append(a)
	return res


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
			_select_combatant(cur_c)
	)

	return clip


func _create_idle_clip_widget(is_playhead: bool, width: int = UNIT_BLOCK_W) -> PanelContainer:
	var slot := PanelContainer.new()
	slot.custom_minimum_size = Vector2(width, 36)
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


# ==============================================================================
# SECTION 6: COMBINED BLOCK INSPECTOR (SPLIT COLUMNS WITH REPLAY VISIBILITY)
# ==============================================================================

func _update_combined_block_display(block: int) -> void:
	# Deprecated: Full screen log modal and master timeline modal replace old combined block inspector
	if combined_party_vbox == null or combined_enemy_vbox == null or combined_block_title_lbl == null:
		return

	combined_block_title_lbl.text = "COMBINED BLOCK %02d / 10" % block

	for i in range(combined_chip_buttons.size()):
		var btn: Button = combined_chip_buttons[i]
		var b_idx: int = i + 1
		if b_idx == block:
			btn.add_theme_color_override("font_color", Color(1.0, 0.85, 0.25))
		else:
			btn.remove_theme_color_override("font_color")

	# Clear previous cards in both split columns
	for child in combined_party_vbox.get_children():
		child.queue_free()
	for child in combined_enemy_vbox.get_children():
		child.queue_free()

	# Query recorded actions archive (100% fidelity even after round finish)
	var actions_at_b: Array[CombatResolver.ScheduledAction] = _get_recorded_actions_at_block(block)

	# ----------------------------------------------------------------------
	# CASE 1: HERO FOCUSED MODE (Party Hero selected in 2x2 box)
	# ----------------------------------------------------------------------
	if selected_party_hero != null:
		var hero: Combatant = selected_party_hero
		var hero_act: CombatResolver.ScheduledAction = null
		for a in actions_at_b:
			if a.combatant == hero:
				hero_act = a
				break

		# 1. Left Column: Only the focused hero
		var hero_card := _create_combined_unit_card(hero, hero_act, block, true)
		combined_party_vbox.add_child(hero_card)
		combined_party_hdr.text = "HERO FOCUS: %s" % hero.name

		# 2. Right Column: Enemies targeting or targeted by this hero
		var targeted_by_hero: Array[Combatant] = []
		var targeting_hero: Array[Combatant] = []

		for enemy in sim_enemies:
			var is_targeted_by_hero: bool = false
			var is_targeting_hero: bool = false

			# Check if hero targets this enemy
			if hero.active_target == enemy:
				is_targeted_by_hero = true
			for a in recorded_actions:
				if a.combatant == hero:
					if a.command is AttackCommand and (a.command as AttackCommand).target == enemy:
						is_targeted_by_hero = true
					elif a.command is TargetCommand and (a.command as TargetCommand).target_value == enemy:
						is_targeted_by_hero = true

			# Check if enemy targets this hero
			if enemy.active_target == hero:
				is_targeting_hero = true
			for a in recorded_actions:
				if a.combatant == enemy:
					if a.command is AttackCommand and (a.command as AttackCommand).target == hero:
						is_targeting_hero = true
					elif a.command is TargetCommand and (a.command as TargetCommand).target_value == hero:
						is_targeting_hero = true

			if is_targeted_by_hero:
				targeted_by_hero.append(enemy)
			elif is_targeting_hero:
				targeting_hero.append(enemy)

		# Targeted enemy appears FIRST on enemy side!
		var engaged_enemies: Array[Combatant] = []
		engaged_enemies.append_array(targeted_by_hero)
		for e in targeting_hero:
			if not engaged_enemies.has(e):
				engaged_enemies.append(e)

		for enemy in engaged_enemies:
			var enemy_act: CombatResolver.ScheduledAction = null
			for a in actions_at_b:
				if a.combatant == enemy:
					enemy_act = a
					break

			if filter_active_only and enemy_act == null:
				continue

			var tag_str: String = "🎯 TARGET" if targeted_by_hero.has(enemy) else "⚡ THREAT"
			var enemy_card := _create_combined_unit_card(enemy, enemy_act, block, false, tag_str)
			combined_enemy_vbox.add_child(enemy_card)

		if engaged_enemies.is_empty():
			var empty_panel := PanelContainer.new()
			var empty_lbl := Label.new()
			empty_lbl.text = "No hostiles engaged with %s." % hero.name
			empty_lbl.add_theme_font_size_override("font_size", 8)
			empty_lbl.add_theme_color_override("font_color", Color(0.6, 0.7, 0.8))
			empty_panel.add_child(empty_lbl)
			combined_enemy_vbox.add_child(empty_panel)

		combined_enemy_hdr.text = "ENGAGED HOSTILES (%d Targeted / %d Total)" % [targeted_by_hero.size(), engaged_enemies.size()]

	# ----------------------------------------------------------------------
	# CASE 2: "ALL" MODE (No hero selected in 2x2 box — show all units)
	# ----------------------------------------------------------------------
	else:
		var active_hero_count: int = 0
		var active_enemy_count: int = 0

		# 1. Populate Allied Party Column
		for ally in sim_allies:
			var act: CombatResolver.ScheduledAction = null
			for a in actions_at_b:
				if a.combatant == ally:
					act = a
					break

			var is_active_act: bool = (act != null)
			if is_active_act:
				active_hero_count += 1

			if filter_active_only and not is_active_act:
				continue

			var card := _create_combined_unit_card(ally, act, block, true)
			combined_party_vbox.add_child(card)

		# 2. Populate Enemy Hostiles Column
		for enemy in sim_enemies:
			var act: CombatResolver.ScheduledAction = null
			for a in actions_at_b:
				if a.combatant == enemy:
					act = a
					break

			var is_active_act: bool = (act != null)
			if is_active_act:
				active_enemy_count += 1

			if filter_active_only and not is_active_act:
				continue

			var card := _create_combined_unit_card(enemy, act, block, false)
			combined_enemy_vbox.add_child(card)

		combined_party_hdr.text = "HERO ACTIONS (%d Active / %d Total)" % [active_hero_count, sim_allies.size()]
		combined_enemy_hdr.text = "HOSTILE ACTIONS (%d Active / %d Total)" % [active_enemy_count, sim_enemies.size()]

	# In replay mode, scrub board state to match inspected block
	if combat_phase == CombatPhase.REPLAY:
		if block >= 0 and block < snapshots.size():
			_restore_snapshot(snapshots[block])


func _create_combined_unit_card(combatant: Combatant, act: CombatResolver.ScheduledAction, block: int, is_ally: bool, engagement_tag: String = "") -> PanelContainer:
	var card := PanelContainer.new()
	card.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	card.custom_minimum_size = Vector2(0, 32)
	card.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	var card_style := StyleBoxFlat.new()
	card_style.set_corner_radius_all(3)
	card_style.content_margin_left = 6
	card_style.content_margin_right = 6
	card_style.content_margin_top = 2
	card_style.content_margin_bottom = 2
	card_style.set_border_width_all(1)

	if combatant == selected_combatant:
		card_style.bg_color = Color(0.14, 0.2, 0.3, 0.98)
		card_style.border_color = Color(1.0, 0.85, 0.3, 1.0)
		card_style.set_border_width_all(2)
	else:
		card_style.bg_color = Color(0.09, 0.12, 0.17, 0.95)
		card_style.border_color = Color(0.2, 0.3, 0.45, 0.8)

	card.add_theme_stylebox_override("panel", card_style)

	var cur_c: Combatant = combatant
	card.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
			_select_combatant(cur_c)
	)

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 6)
	card.add_child(hbox)

	# Avatar icon
	var icon := TextureRect.new()
	if combatant.name == "Warrior":
		icon.texture = tex_warrior
	elif combatant.name == "Ranger":
		icon.texture = tex_ranger
	elif combatant.name == "Enemy Brawler":
		icon.texture = tex_brawler
	else:
		icon.texture = tex_scout
	icon.custom_minimum_size = Vector2(18, 18)
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	hbox.add_child(icon)

	# Identity Info
	var id_vbox := VBoxContainer.new()
	id_vbox.custom_minimum_size = Vector2(75, 0)
	id_vbox.add_theme_constant_override("separation", 0)
	hbox.add_child(id_vbox)

	var name_lbl := Label.new()
	name_lbl.text = combatant.name
	name_lbl.add_theme_font_size_override("font_size", 8)
	name_lbl.add_theme_color_override("font_color", Color.WHITE)
	id_vbox.add_child(name_lbl)

	var stance_lbl := Label.new()
	stance_lbl.text = CombatTypes.stance_to_string(combatant.current_stance)
	stance_lbl.add_theme_font_size_override("font_size", 7)
	stance_lbl.add_theme_color_override("font_color", Color(0.7, 0.85, 0.95) if is_ally else Color(1.0, 0.7, 0.7))
	id_vbox.add_child(stance_lbl)

	if engagement_tag != "":
		var tag_lbl := Label.new()
		tag_lbl.text = engagement_tag
		tag_lbl.add_theme_font_size_override("font_size", 7)
		tag_lbl.add_theme_color_override("font_color", Color(1.0, 0.85, 0.25) if engagement_tag.begins_with("🎯") else Color(1.0, 0.55, 0.35))
		id_vbox.add_child(tag_lbl)

	# Action Block Box
	var act_box := PanelContainer.new()
	act_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var box_style := StyleBoxFlat.new()
	box_style.set_corner_radius_all(3)
	box_style.set_border_width_all(1)
	box_style.content_margin_left = 6
	box_style.content_margin_right = 6
	box_style.content_margin_top = 2
	box_style.content_margin_bottom = 2

	var act_vbox := VBoxContainer.new()
	act_vbox.add_theme_constant_override("separation", 0)
	act_box.add_child(act_vbox)

	var act_title := Label.new()
	act_title.add_theme_font_size_override("font_size", 8)
	act_vbox.add_child(act_title)

	var act_sub := Label.new()
	act_sub.add_theme_font_size_override("font_size", 7)
	act_sub.add_theme_color_override("font_color", Color(0.7, 0.8, 0.85))
	act_vbox.add_child(act_sub)

	var is_enemy_in_planning: bool = not is_ally and combat_phase != CombatPhase.REPLAY

	if is_enemy_in_planning:
		act_title.text = "🔒 Secret Action"
		act_title.add_theme_color_override("font_color", Color(1.0, 0.55, 0.55))
		act_sub.text = "Intent masked during planning"
		box_style.bg_color = Color(0.18, 0.09, 0.11, 0.8)
		box_style.border_color = Color(0.55, 0.25, 0.25, 0.8)
	elif act != null:
		var cmd := act.command
		var step_in_act: int = block - act.start_block + 1
		var total_steps: int = act.end_block - act.start_block + 1
		var phase_desc: String = "Resolving Impact" if block == act.end_block else "Windup / Channeling"

		act_title.text = "%s [%d/%d]" % [cmd.command_name, step_in_act, total_steps]
		act_title.add_theme_color_override("font_color", Color.WHITE)
		act_sub.text = phase_desc

		if cmd is AttackCommand:
			box_style.bg_color = Color(0.4, 0.12, 0.14, 0.9)
			box_style.border_color = Color(0.85, 0.3, 0.3, 1.0)
		elif cmd is DefendCommand:
			box_style.bg_color = Color(0.12, 0.25, 0.48, 0.9)
			box_style.border_color = Color(0.3, 0.6, 0.95, 1.0)
		elif cmd is FocusCommand:
			box_style.bg_color = Color(0.32, 0.14, 0.48, 0.9)
			box_style.border_color = Color(0.75, 0.4, 0.95, 1.0)
		elif cmd is MoveCommand:
			box_style.bg_color = Color(0.08, 0.32, 0.44, 0.9)
			box_style.border_color = Color(0.2, 0.8, 0.95, 1.0)
		else:
			box_style.bg_color = Color(0.08, 0.38, 0.24, 0.9)
			box_style.border_color = Color(0.25, 0.85, 0.5, 1.0)
	else:
		act_title.text = "· Idle Guard"
		act_title.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		act_sub.text = "Evasion / Stance bonus"
		box_style.bg_color = Color(0.11, 0.14, 0.19, 0.8)
		box_style.border_color = Color(0.2, 0.26, 0.35, 0.6)

	act_box.add_theme_stylebox_override("panel", box_style)
	hbox.add_child(act_box)

	return card


func _navigate_combined_block(delta: int) -> void:
	var target_b: int = clampi(replay_block + delta, 1, TOTAL_BLOCKS)
	_on_ruler_block_clicked(target_b)


# ==============================================================================
# SECTION 7: STEPPING, AUTO-RESOLVE, REPLAY & INPUT CONTROLS
# ==============================================================================

func _step_simulation() -> void:
	if sim_finished and combat_phase != CombatPhase.REPLAY:
		return

	if combat_phase == CombatPhase.PLANNING:
		combat_phase = CombatPhase.RESOLVING
		phase_badge_lbl.text = "RESOLVING COMBAT"
		phase_badge_lbl.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
		current_sim_block = 1
		pending_block_actions.clear()

	if combat_phase == CombatPhase.RESOLVING:
		# If no pending actions are queued for this block, prepare the block!
		if pending_block_actions.is_empty():
			# Mid-round ambush trigger: When Block 4 is reached in Round 1, 4 new goblins appear!
			if current_sim_block == 4 and not reinforcements_spawned and current_combat_round == 1:
				_spawn_goblin_reinforcements()
				var ambush_entry := ActionLogEntry.new()
				ambush_entry.round_num = current_combat_round
				ambush_entry.block_num = current_sim_block
				ambush_entry.ref_tag = "R%d-B%02d" % [current_combat_round, current_sim_block]
				ambush_entry.actor_name = "Goblin Pack"
				ambush_entry.actor_is_ally = false
				ambush_entry.category = "EVENT"
				ambush_entry.skill_name = "Ambush Reinforcements"
				ambush_entry.skill_subtype = "Perimeter Breach"
				ambush_entry.outcome_tag = "AMBUSH!"
				ambush_entry.detail_text = "4 Goblin reinforcements arrive from eastern ridge!"
				ambush_entry.status_text = "Hostiles: 6 total"
				all_action_log_entries.append(ambush_entry)
				active_log_entry_index = all_action_log_entries.size() - 1
				_update_active_log_cell()
				_trigger_action_impact_fx(ambush_entry)
				return

			_append_log("[color=#fde047][b]━━━ BLOCK %d RESOLUTION (ROUND %d) ━━━[/b][/color]" % [current_sim_block, current_combat_round])
			pending_block_actions = sim_resolver.prepare_block(current_sim_block)

		# Execute next single action if available
		if not pending_block_actions.is_empty():
			var act: CombatResolver.ScheduledAction = pending_block_actions.pop_front()
			sim_resolver.execute_single_action(act)
			var entry := _build_entry_from_action(act, current_combat_round, current_sim_block)
			all_action_log_entries.append(entry)
			active_log_entry_index = all_action_log_entries.size() - 1
			_update_active_log_cell()
			_trigger_action_impact_fx(entry)
			_sync_visuals_after_block()

		# If all actions in this block have finished:
		if pending_block_actions.is_empty():
			sim_resolver.block_completed.emit(current_sim_block)
			_capture_snapshot(current_sim_block)
			_sync_visuals_after_block()

			var is_over: bool = sim_resolver.is_combat_resolved(sim_allies, sim_enemies)
			if is_over or current_sim_block >= TOTAL_BLOCKS:
				sim_finished = true
				combat_phase = CombatPhase.REPLAY
				replay_block = current_sim_block
				auto_play_timer.stop()
				is_auto_playing = false
				btn_play.text = "▶"
				btn_play.tooltip_text = "Play Replay"
				phase_badge_lbl.text = "ROUND %d CONCLUDED - REPLAY MODE" % current_combat_round
				phase_badge_lbl.add_theme_color_override("font_color", Color(0.3, 0.9, 0.5))

				if btn_next_round:
					btn_next_round.visible = (current_combat_round == 1 and not is_over)

				if current_combat_round == 1:
					status_label.text = "Round 1 complete! Click Advance to Round 2."
					_append_log("\n[color=#4ade80][b]🎉 ROUND 1 RESOLVED: Ranger fell in the ambush, Warrior holds at 73 HP against 4 goblins! Click ADVANCE TO ROUND 2 to begin the Last Stand.[/b][/color]")
				else:
					status_label.text = "Round 2 resolved! Warrior's last stand concluded. Review replay."
					_append_log("\n[color=#4ade80][b]🎉 ROUND 2 RESOLVED: Combat concluded! Review replay or Reset.[/b][/color]")

				_update_timeline_tracks_display()
				_update_combined_block_display(replay_block)
				return

			current_sim_block += 1
			status_label.text = "Ready for Block %d." % current_sim_block
			_update_timeline_tracks_display()
			_update_combined_block_display(current_sim_block)

	elif combat_phase == CombatPhase.REPLAY:
		# Step Replay through recorded action entries
		if not all_action_log_entries.is_empty():
			active_log_entry_index = (active_log_entry_index + 1) % all_action_log_entries.size()
			var replay_entry := all_action_log_entries[active_log_entry_index]
			_update_active_log_cell()
			_trigger_action_impact_fx(replay_entry)
			replay_block = replay_entry.block_num
			if replay_block >= 0 and replay_block < snapshots.size():
				_restore_snapshot(snapshots[replay_block])
			status_label.text = "Replaying: %s [%s]" % [replay_entry.skill_name, replay_entry.ref_tag]
			_update_timeline_tracks_display()
			_update_combined_block_display(replay_block)
		else:
			replay_block = (replay_block % TOTAL_BLOCKS) + 1
			status_label.text = "Replay Block %d of 10." % replay_block
			_update_timeline_tracks_display()
			_update_combined_block_display(replay_block)

		if timeline_modal_overlay != null and timeline_modal_overlay.visible:
			_update_modal_tracks_display()


func _sync_visuals_after_block() -> void:
	for combatant: Combatant in unit_token_nodes:
		@warning_ignore("unsafe_cast")
		var token := unit_token_nodes[combatant] as Control
		var target_coord: Vector2i = sim_grid.get_position(combatant)
		var is_alive: bool = combatant.is_alive()
		var is_withdrawn: bool = combatant.is_withdrawn
		var is_act: bool = combatant.is_active() and is_alive and not is_withdrawn

		# Requirement 7: Remove dead or withdrawn unit from board completely
		if target_coord == Vector2i(-1, -1) or not is_act:
			token.visible = false
			token.modulate.a = 0.0
		else:
			token.visible = true
			token.modulate = Color.WHITE
			var target_px := Vector2(target_coord.x * TILE_SIZE, target_coord.y * TILE_SIZE)
			if token.position != target_px:
				var tw := create_tween()
				tw.tween_property(token, "position", target_px, 0.25).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)

	_update_all_roster_cards()


func _spawn_goblin_reinforcements() -> void:
	reinforcements_spawned = true
	_append_log("\n[color=#ef4444][b]⚠️ AMBUSH! Scout's escape alerted the pack! 4 Goblin Reinforcements enter the arena![/b][/color]")
	_append_log("[color=#f87171]  • Goblin Berserker (55 HP) deployed at (5, 5)[/color]")
	_append_log("[color=#f87171]  • Goblin Skirmisher (40 HP) deployed at (6, 4)[/color]")
	_append_log("[color=#f87171]  • Goblin Archer (35 HP) deployed at (7, 5)[/color]")
	_append_log("[color=#f87171]  • Goblin Shaman (35 HP) deployed at (6, 5)[/color]")

	sim_grid.place_combatant(unit_berserker, Vector2i(5, 5))
	sim_grid.place_combatant(unit_skirmisher, Vector2i(6, 4))
	sim_grid.place_combatant(unit_archer, Vector2i(7, 5))
	sim_grid.place_combatant(unit_shaman, Vector2i(6, 5))

	var new_goblins: Array[Combatant] = [unit_berserker, unit_skirmisher, unit_archer, unit_shaman]
	for c: Combatant in new_goblins:
		if unit_token_nodes.has(c):
			@warning_ignore("unsafe_cast")
			var token := unit_token_nodes[c] as Control
			var coord: Vector2i = sim_grid.get_position(c)
			token.position = Vector2(coord.x * TILE_SIZE, coord.y * TILE_SIZE)
			var tw := create_tween()
			tw.tween_property(token, "modulate:a", 1.0, 0.35)
		if roster_cards.has(c):
			@warning_ignore("unsafe_cast")
			var card := roster_cards[c] as Control
			if is_instance_valid(card):
				card.visible = true

	_update_all_roster_cards()


func _on_next_round_pressed() -> void:
	_start_round_2()


func _start_round_2() -> void:
	current_combat_round = 2
	sim_resolver.finish_round()

	current_sim_block = 1
	replay_block = 1
	combined_selected_block = 1
	sim_finished = false
	is_auto_playing = false
	combat_phase = CombatPhase.PLANNING
	auto_play_timer.stop()
	btn_play.text = "▶"
	btn_play.tooltip_text = "Auto-Resolve Round 2"
	if btn_next_round:
		btn_next_round.visible = false

	# Prune any defeated or withdrawn enemies from roster cards for the new round
	var enemies_to_remove: Array[Combatant] = []
	for c: Combatant in roster_cards:
		if not sim_allies.has(c):
			if not c.is_alive() or c.is_withdrawn:
				enemies_to_remove.append(c)

	for c: Combatant in enemies_to_remove:
		@warning_ignore("unsafe_cast")
		var old_card := roster_cards[c] as Control
		roster_cards.erase(c)
		if is_instance_valid(old_card):
			old_card.queue_free()

	# Ensure all active surviving enemy cards are visible
	for c: Combatant in roster_cards:
		if not sim_allies.has(c):
			@warning_ignore("unsafe_cast")
			var rem_card := roster_cards[c] as Control
			if is_instance_valid(rem_card):
				rem_card.visible = true

	pending_block_actions.clear()
	var r2_entry := ActionLogEntry.new()
	r2_entry.round_num = 2
	r2_entry.block_num = 1
	r2_entry.ref_tag = "R2-B01"
	r2_entry.actor_name = "Warrior"
	r2_entry.actor_is_ally = true
	r2_entry.category = "EVENT"
	r2_entry.skill_name = "Round 2 Commenced"
	r2_entry.skill_subtype = "Last Stand"
	r2_entry.outcome_tag = "ROUND 2"
	r2_entry.detail_text = "Warrior stands alone at (4, 4) against 4 goblins"
	r2_entry.status_text = "Round 2 Active"
	all_action_log_entries.append(r2_entry)
	active_log_entry_index = all_action_log_entries.size() - 1
	_update_active_log_cell()

	phase_badge_lbl.text = "ROUND 2: PLANNING PHASE"
	phase_badge_lbl.add_theme_color_override("font_color", Color(0.95, 0.8, 0.35))
	status_label.text = "Round 2: Warrior (73 HP) stands alone against 4 goblins! Set actions or Auto-Resolve."

	_append_log("\n[color=#38bdf8][b]⚔⚔⚔ COMMENCING ROUND 2: WARRIOR'S LAST STAND ⚔⚔⚔[/b][/color]")
	_append_log("[color=#93c5fd]Surviving hero Warrior holds center at (4, 4). Pre-planned Round 2 actions loaded.[/color]")

	# Schedule Pre-Planned Commands for Round 2
	# Warrior:
	#   Block 1-2: Cleaving Strike on Skirmisher at (4, 3) (power 25) [2b] -> Skirmisher DEFEATED!
	#   Block 3-4: Defend (50% mitigation) [2b]
	#   Block 5-6: Heavy Strike on Berserker at (5, 4) (power 30) [2b] -> Berserker DEFEATED!
	#   Block 7: Move East to (5, 4) [1b]
	#   Block 8-9: Strike on Shaman at (6, 5) (power 35) [2b] -> Shaman DEFEATED!
	#   Block 10: Hold Stance [1b]
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_skirmisher, 25, 2, 0, sim_grid), 1)
	sim_resolver.schedule_action(unit_warrior, DefendCommand.new(unit_warrior, 0.5, 2, 1), 3)
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_berserker, 30, 2, 0, sim_grid), 5)
	sim_resolver.schedule_action(unit_warrior, MoveCommand.new(unit_warrior, sim_grid, Vector2i(1, 0), 1), 7)
	sim_resolver.schedule_action(unit_warrior, AttackCommand.new(unit_warrior, unit_shaman, 35, 2, 0, sim_grid), 8)
	sim_resolver.schedule_action(unit_warrior, HoldStanceCommand.new(unit_warrior, 0), 10)

	# Goblin Skirmisher (15 HP at (4, 3)):
	sim_resolver.schedule_action(unit_skirmisher, TargetCommand.new(unit_skirmisher, unit_warrior, 1), 1)
	sim_resolver.schedule_action(unit_skirmisher, AttackCommand.new(unit_skirmisher, unit_warrior, 10, 1, 0, sim_grid), 2)

	# Goblin Berserker (30 HP at (5, 4)):
	sim_resolver.schedule_action(unit_berserker, TargetCommand.new(unit_berserker, unit_warrior, 1), 1)
	sim_resolver.schedule_action(unit_berserker, AttackCommand.new(unit_berserker, unit_warrior, 20, 2, 0, sim_grid), 2)
	sim_resolver.schedule_action(unit_berserker, TargetCommand.new(unit_berserker, unit_warrior, 1), 4)
	sim_resolver.schedule_action(unit_berserker, AttackCommand.new(unit_berserker, unit_warrior, 20, 2, 0, sim_grid), 5)

	# Goblin Archer (35 HP at (7, 5)):
	sim_resolver.schedule_action(unit_archer, TargetCommand.new(unit_archer, unit_warrior, 1), 1)
	sim_resolver.schedule_action(unit_archer, AttackCommand.new(unit_archer, unit_warrior, 14, 2, 0, sim_grid), 2)
	sim_resolver.schedule_action(unit_archer, AttackCommand.new(unit_archer, unit_warrior, 15, 2, 0, sim_grid), 4)
	sim_resolver.schedule_action(unit_archer, AttackCommand.new(unit_archer, unit_warrior, 15, 2, 0, sim_grid), 6)

	# Goblin Shaman (35 HP at (6, 5)):
	sim_resolver.schedule_action(unit_shaman, TargetCommand.new(unit_shaman, unit_warrior, 1), 1)
	sim_resolver.schedule_action(unit_shaman, AttackCommand.new(unit_shaman, unit_warrior, 10, 2, 0, sim_grid), 2)
	sim_resolver.schedule_action(unit_shaman, DefendCommand.new(unit_shaman, 0.5, 2, 1), 4)

	# Record Round 2 actions
	recorded_actions.clear()
	for a in sim_resolver.scheduled_actions:
		recorded_actions.append(CombatResolver.ScheduledAction.new(a.combatant, a.command, a.start_block, a.end_block))
	round_recorded_actions[2] = recorded_actions.duplicate()

	# Snapshot 0 for Round 2
	snapshots.clear()
	_capture_snapshot(0)
	round_snapshots[2] = snapshots.duplicate()
	viewed_timeline_round = 2
	_update_round_traversal_ui()

	_update_party_select_box()
	_select_combatant(unit_warrior)
	_update_timeline_tracks_display()
	_update_combined_block_display(1)
	_sync_visuals_after_block()
	_update_all_roster_cards()

	_update_selected_unit_panel()
	_update_tactical_forecast_panel()
	if timeline_modal_overlay != null and timeline_modal_overlay.visible:
		_update_modal_tracks_display()


# ==============================================================================
# RULE-DRIVEN ACTION LOG STREAM & DYNAMIC CELL TEMPLATES
# ==============================================================================

func _build_entry_from_action(act: CombatResolver.ScheduledAction, round_num: int, block_num: int) -> ActionLogEntry:
	var entry := ActionLogEntry.new()
	entry.round_num = round_num
	entry.block_num = block_num
	entry.ref_tag = "R%d-B%02d" % [round_num, block_num]
	entry.actor_name = act.combatant.name
	entry.actor_is_ally = sim_allies.has(act.combatant)

	if act.command is AttackCommand:
		var cmd := act.command as AttackCommand
		entry.category = "ATTACK"
		var target_c: Combatant = cmd.target
		if target_c:
			entry.target_name = target_c.name
			entry.target_is_ally = sim_allies.has(target_c)
		else:
			entry.target_name = "Target"

		var atk_range: int = cmd.source.attack_range
		if atk_range > 1:
			entry.skill_subtype = "Ranged Projectile"
			entry.skill_name = "Quick Snipe" if entry.actor_is_ally else "Longbow Volley"
		else:
			entry.skill_subtype = "Melee Physical"
			entry.skill_name = "Heavy Slash" if entry.actor_is_ally else "Power Strike"

		entry.value_num = cmd.power
		if target_c:
			if not target_c.is_alive():
				entry.outcome_tag = "DEFEATED"
			elif target_c.defense_mitigation_ratio > 0.0:
				entry.outcome_tag = "MITIGATED"
			else:
				entry.outcome_tag = "HIT"
			entry.status_text = "Target HP: %d/%d" % [target_c.hp, target_c.max_hp]
		else:
			entry.outcome_tag = "HIT"
		entry.detail_text = "Power: %d | Range: %d" % [cmd.power, atk_range]

	elif act.command is MoveCommand:
		var cmd := act.command as MoveCommand
		entry.category = "MOVE"
		entry.skill_name = "Sprint" if cmd.steps > 1 else "Tactical Move"
		entry.skill_subtype = "Sprint Relocation" if cmd.steps > 1 else "Standard Step"
		entry.value_num = cmd.steps
		entry.coord_to = sim_grid.get_position(act.combatant)
		if act.combatant.is_withdrawn:
			entry.outcome_tag = "WITHDRAWN"
			entry.status_text = "Safely exited battle grid"
		else:
			entry.outcome_tag = "MOVED"
			entry.status_text = "Position: %s" % str(entry.coord_to)
		entry.detail_text = "Stepped %d tiles (%s)" % [cmd.steps, str(cmd.direction)]

	elif act.command is DefendCommand:
		var cmd := act.command as DefendCommand
		entry.category = "DEFEND"
		entry.skill_name = "Defensive Stance"
		entry.skill_subtype = "Shield Guard"
		entry.value_num = int(cmd.mitigation_ratio * 100.0)
		entry.outcome_tag = "%d%% MITIGATION" % entry.value_num
		entry.detail_text = "Damage mitigation posture active"
		entry.status_text = "Duration: %d Blocks" % cmd.block_cost

	elif act.command is TargetCommand:
		var cmd := act.command as TargetCommand
		entry.category = "TARGET"
		entry.skill_name = "Target Lock"
		entry.skill_subtype = "Engagement Focus"
		var target_c: Combatant = null
		if cmd.target_value is Combatant:
			@warning_ignore("unsafe_cast")
			target_c = cmd.target_value as Combatant
		if target_c != null:
			entry.target_name = target_c.name
			entry.target_is_ally = sim_allies.has(target_c)
		elif cmd.target_value != null:
			entry.target_name = str(cmd.target_value)
		else:
			entry.target_name = "None"
		entry.outcome_tag = "LOCKED"
		entry.detail_text = "Assigned tactical target focus"
		entry.status_text = "Target Locked"

	elif act.command is FocusCommand:
		var cmd := act.command as FocusCommand
		entry.category = "BUFF"
		entry.skill_name = "Focus Skill"
		entry.skill_subtype = "Attack Multiplier"
		entry.value_num = int((cmd.boost_multiplier - 1.0) * 100.0)
		entry.outcome_tag = "+%d%% ATK" % entry.value_num
		entry.detail_text = "+%d%% Attack Power for %d Round(s)" % [entry.value_num, cmd.duration_rounds]
		entry.status_text = "Empowered"

	elif act.command is HoldStanceCommand:
		entry.category = "DEFEND"
		entry.skill_name = "Hold Stance"
		entry.skill_subtype = "Posture Maintenance"
		entry.outcome_tag = "MAINTAINED"
		entry.detail_text = "Current stance maintained"
		entry.status_text = "Active"

	else:
		entry.category = "EVENT"
		entry.skill_name = act.command.command_name
		entry.skill_subtype = "Standard Action"
		entry.outcome_tag = "EXECUTED"
		entry.detail_text = act.command.command_name

	return entry


func _find_combatant_by_name(p_name: String) -> Combatant:
	for c: Combatant in sim_allies:
		if c.name == p_name:
			return c
	for c: Combatant in sim_enemies:
		if c.name == p_name:
			return c
	return null


func _format_action_cell(entry: ActionLogEntry, is_modal: bool = false) -> PanelContainer:
	var cell := PanelContainer.new()
	cell.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cell.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND if is_modal else Control.CURSOR_ARROW

	var style := StyleBoxFlat.new()
	var cat_color: Color
	match entry.category:
		"ATTACK": cat_color = Color(0.92, 0.32, 0.32)
		"DEFEND": cat_color = Color(0.35, 0.68, 1.0)
		"BUFF": cat_color = Color(0.98, 0.78, 0.25)
		"MOVE": cat_color = Color(0.3, 0.8, 0.95)
		"TARGET": cat_color = Color(0.3, 0.85, 0.55)
		"EVENT": cat_color = Color(0.75, 0.5, 0.98)
		_: cat_color = Color(0.7, 0.75, 0.85)

	style.bg_color = Color(cat_color.r * 0.12, cat_color.g * 0.12, cat_color.b * 0.15, 0.95)
	style.border_color = cat_color.lerp(Color.WHITE, 0.15) if is_modal else cat_color
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.content_margin_left = 6
	style.content_margin_right = 6
	style.content_margin_top = 2
	style.content_margin_bottom = 2
	cell.add_theme_stylebox_override("panel", style)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 1)
	cell.add_child(vbox)

	# Row 1: Badges & Tags
	var row1 := HBoxContainer.new()
	row1.add_theme_constant_override("separation", 5)
	vbox.add_child(row1)

	# Reference Tag Badge [R1-B04]
	var ref_lbl := Label.new()
	ref_lbl.text = "[%s]" % entry.ref_tag
	ref_lbl.add_theme_font_size_override("font_size", 9)
	ref_lbl.add_theme_color_override("font_color", Color(0.95, 0.85, 0.35))
	row1.add_child(ref_lbl)

	# Category + Subtype Badge
	var type_lbl := Label.new()
	type_lbl.text = "• %s" % entry.category
	if not entry.skill_subtype.is_empty():
		type_lbl.text += " (%s)" % entry.skill_subtype
	type_lbl.add_theme_font_size_override("font_size", 8)
	type_lbl.add_theme_color_override("font_color", cat_color)
	row1.add_child(type_lbl)

	row1.add_spacer(false)

	# Outcome Badge
	if not entry.outcome_tag.is_empty():
		var out_lbl := Label.new()
		out_lbl.text = "[%s]" % entry.outcome_tag
		out_lbl.add_theme_font_size_override("font_size", 8)
		if entry.outcome_tag == "DEFEATED":
			out_lbl.add_theme_color_override("font_color", Color(1.0, 0.25, 0.25))
		elif "MITIGAT" in entry.outcome_tag:
			out_lbl.add_theme_color_override("font_color", Color(0.4, 0.85, 1.0))
		elif "BUFF" in entry.outcome_tag or "+" in entry.outcome_tag:
			out_lbl.add_theme_color_override("font_color", Color(0.98, 0.82, 0.3))
		elif entry.outcome_tag == "WITHDRAWN":
			out_lbl.add_theme_color_override("font_color", Color(0.35, 0.95, 0.55))
		else:
			out_lbl.add_theme_color_override("font_color", Color(0.9, 0.9, 0.95))
		row1.add_child(out_lbl)

	# Row 2: Narrative & Target Details
	var row2 := HBoxContainer.new()
	row2.add_theme_constant_override("separation", 4)
	vbox.add_child(row2)

	# Actor
	var actor_lbl := Label.new()
	actor_lbl.text = entry.actor_name
	actor_lbl.add_theme_font_size_override("font_size", 9)
	var actor_col := Color(0.4, 0.75, 1.0) if entry.actor_is_ally else Color(1.0, 0.45, 0.45)
	actor_lbl.add_theme_color_override("font_color", actor_col)
	row2.add_child(actor_lbl)

	# Verb / Skill
	var skill_lbl := Label.new()
	var icon_sym := "⚔"
	match entry.category:
		"MOVE": icon_sym = "➜"
		"DEFEND": icon_sym = "🛡"
		"BUFF": icon_sym = "✦"
		"TARGET": icon_sym = "🎯"
		"EVENT": icon_sym = "⚡"
	skill_lbl.text = "%s %s" % [icon_sym, entry.skill_name]
	skill_lbl.add_theme_font_size_override("font_size", 8)
	skill_lbl.add_theme_color_override("font_color", Color(0.85, 0.85, 0.9))
	row2.add_child(skill_lbl)

	# Target (if any)
	if not entry.target_name.is_empty():
		var arrow_lbl := Label.new()
		arrow_lbl.text = "➔"
		arrow_lbl.add_theme_font_size_override("font_size", 8)
		arrow_lbl.add_theme_color_override("font_color", Color(0.6, 0.65, 0.75))
		row2.add_child(arrow_lbl)

		var target_lbl := Label.new()
		target_lbl.text = entry.target_name
		target_lbl.add_theme_font_size_override("font_size", 9)
		var target_col := Color(0.4, 0.75, 1.0) if entry.target_is_ally else Color(1.0, 0.45, 0.45)
		target_lbl.add_theme_color_override("font_color", target_col)
		row2.add_child(target_lbl)

	row2.add_spacer(false)

	# Status / Detail Text
	var detail_msg: String = entry.status_text if not entry.status_text.is_empty() else entry.detail_text
	if not detail_msg.is_empty():
		var stat_lbl := Label.new()
		stat_lbl.text = detail_msg
		stat_lbl.add_theme_font_size_override("font_size", 8)
		stat_lbl.add_theme_color_override("font_color", Color(0.65, 0.75, 0.85))
		row2.add_child(stat_lbl)

	if is_modal:
		cell.gui_input.connect(func(ev: InputEvent) -> void:
			var mb := ev as InputEventMouseButton
			if mb != null and mb.pressed and mb.button_index == MOUSE_BUTTON_LEFT:
				_on_log_modal_cell_clicked(entry)
		)

	return cell


func _update_active_log_cell() -> void:
	if not active_cell_dock:
		return
	for child in active_cell_dock.get_children():
		child.queue_free()

	if all_action_log_entries.is_empty() or active_log_entry_index < 0:
		var empty_lbl := Label.new()
		empty_lbl.text = "Awaiting Action Resolution... Click Step [F6] or Play [Space] to begin."
		empty_lbl.add_theme_font_size_override("font_size", 9)
		empty_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		active_cell_dock.add_child(empty_lbl)
		if log_step_badge:
			log_step_badge.text = "[Awaiting Action]"
		if btn_log_prev:
			btn_log_prev.disabled = true
		if btn_log_next:
			btn_log_next.disabled = true
	else:
		var entry := all_action_log_entries[active_log_entry_index]
		active_cell_dock.add_child(_format_action_cell(entry, false))
		if log_step_badge:
			log_step_badge.text = "[%s] (%d/%d)" % [entry.ref_tag, active_log_entry_index + 1, all_action_log_entries.size()]
		if btn_log_prev:
			btn_log_prev.disabled = (active_log_entry_index <= 0)
		if btn_log_next:
			btn_log_next.disabled = (active_log_entry_index >= all_action_log_entries.size() - 1)


func _on_log_prev_pressed() -> void:
	if active_log_entry_index > 0:
		active_log_entry_index -= 1
		_update_active_log_cell()
		var entry := all_action_log_entries[active_log_entry_index]
		_trigger_action_impact_fx(entry)
		if combat_phase == CombatPhase.REPLAY and entry.block_num < snapshots.size():
			replay_block = entry.block_num
			_restore_snapshot(snapshots[replay_block])
			_update_timeline_tracks_display()
			_update_combined_block_display(replay_block)


func _on_log_next_pressed() -> void:
	if active_log_entry_index < all_action_log_entries.size() - 1:
		active_log_entry_index += 1
		_update_active_log_cell()
		var entry := all_action_log_entries[active_log_entry_index]
		_trigger_action_impact_fx(entry)
		if combat_phase == CombatPhase.REPLAY and entry.block_num < snapshots.size():
			replay_block = entry.block_num
			_restore_snapshot(snapshots[replay_block])
			_update_timeline_tracks_display()
			_update_combined_block_display(replay_block)


func _build_log_modal_ui() -> void:
	log_modal_overlay = PanelContainer.new()
	log_modal_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	log_modal_overlay.visible = false
	log_modal_overlay.mouse_filter = Control.MOUSE_FILTER_STOP

	var backdrop_style := StyleBoxFlat.new()
	backdrop_style.bg_color = Color(0.02, 0.04, 0.07, 0.9)
	log_modal_overlay.add_theme_stylebox_override("panel", backdrop_style)
	ui_canvas.add_child(log_modal_overlay)

	var center_margin := MarginContainer.new()
	center_margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	center_margin.add_theme_constant_override("margin_left", 180)
	center_margin.add_theme_constant_override("margin_right", 180)
	center_margin.add_theme_constant_override("margin_top", 70)
	center_margin.add_theme_constant_override("margin_bottom", 70)
	log_modal_overlay.add_child(center_margin)

	var modal_box := PanelContainer.new()
	var box_style := StyleBoxFlat.new()
	box_style.bg_color = Color(0.07, 0.09, 0.14, 0.98)
	box_style.border_color = Color(0.28, 0.46, 0.75, 0.9)
	box_style.set_border_width_all(2)
	box_style.set_corner_radius_all(6)
	box_style.content_margin_left = 12
	box_style.content_margin_right = 12
	box_style.content_margin_top = 10
	box_style.content_margin_bottom = 10
	modal_box.add_theme_stylebox_override("panel", box_style)
	center_margin.add_child(modal_box)

	var modal_vbox := VBoxContainer.new()
	modal_vbox.add_theme_constant_override("separation", 6)
	modal_box.add_child(modal_vbox)

	# Top header bar
	var hdr := HBoxContainer.new()
	modal_vbox.add_child(hdr)

	var title := Label.new()
	title.text = "📜 EXPANDED ACTION STREAM LOG"
	title.add_theme_font_size_override("font_size", 12)
	title.add_theme_color_override("font_color", Color(0.96, 0.85, 0.35))
	hdr.add_child(title)

	var sub := Label.new()
	sub.text = "Chronological Action Cells (R<X>-B<Y> Reference Format)"
	sub.add_theme_font_size_override("font_size", 8)
	sub.add_theme_color_override("font_color", Color(0.55, 0.65, 0.78))
	hdr.add_child(sub)

	hdr.add_spacer(false)

	var close_btn := Button.new()
	close_btn.text = "✕ CLOSE (ESC / L)"
	close_btn.custom_minimum_size = Vector2(95, 22)
	close_btn.add_theme_font_size_override("font_size", 9)
	var cls_style := StyleBoxFlat.new()
	cls_style.bg_color = Color(0.35, 0.12, 0.15, 0.9)
	cls_style.border_color = Color(0.85, 0.35, 0.35, 1.0)
	cls_style.set_border_width_all(1)
	cls_style.set_corner_radius_all(3)
	close_btn.add_theme_stylebox_override("normal", cls_style)
	close_btn.pressed.connect(func() -> void: _toggle_log_modal(false))
	hdr.add_child(close_btn)

	# Filter bar
	var filter_bar := HBoxContainer.new()
	filter_bar.add_theme_constant_override("separation", 4)
	modal_vbox.add_child(filter_bar)

	var filter_hdr := Label.new()
	filter_hdr.text = "Filter Stream:"
	filter_hdr.add_theme_font_size_override("font_size", 8)
	filter_hdr.add_theme_color_override("font_color", Color(0.6, 0.7, 0.8))
	filter_bar.add_child(filter_hdr)

	var categories: Array[String] = ["ALL", "ATTACK", "DEFEND", "BUFF", "MOVE", "TARGET", "EVENT"]
	for cat in categories:
		var btn := Button.new()
		btn.text = cat
		btn.custom_minimum_size = Vector2(50, 18)
		btn.add_theme_font_size_override("font_size", 8)
		btn.pressed.connect(func() -> void:
			current_log_filter = cat
			_refresh_log_modal_content()
		)
		filter_bar.add_child(btn)
		modal_filter_btns[cat] = btn

	var hsep := HSeparator.new()
	modal_vbox.add_child(hsep)

	# Scrollable entries stream
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	modal_vbox.add_child(scroll)

	modal_entries_vbox = VBoxContainer.new()
	modal_entries_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	modal_entries_vbox.add_theme_constant_override("separation", 4)
	scroll.add_child(modal_entries_vbox)


func _toggle_log_modal(open: bool) -> void:
	if not log_modal_overlay:
		return
	log_modal_overlay.visible = open
	if open:
		log_modal_overlay.move_to_front()
		_refresh_log_modal_content()


func _refresh_log_modal_content() -> void:
	if not modal_entries_vbox:
		return
	for child in modal_entries_vbox.get_children():
		child.queue_free()

	# Highlight active filter button
	for cat: String in modal_filter_btns:
		var btn: Button = modal_filter_btns[cat]
		if cat == current_log_filter:
			btn.add_theme_color_override("font_color", Color(0.95, 0.85, 0.35))
		else:
			btn.remove_theme_color_override("font_color")

	if all_action_log_entries.is_empty():
		var empty_lbl := Label.new()
		empty_lbl.text = "No action cells resolved yet. Run combat blocks to populate stream."
		empty_lbl.add_theme_font_size_override("font_size", 10)
		empty_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		modal_entries_vbox.add_child(empty_lbl)
		return

	var last_round: int = -1
	var last_block: int = -1

	for entry in all_action_log_entries:
		if current_log_filter != "ALL" and entry.category != current_log_filter:
			continue

		# Round Divider
		if entry.round_num != last_round:
			last_round = entry.round_num
			last_block = -1
			var rd_panel := PanelContainer.new()
			var rd_style := StyleBoxFlat.new()
			rd_style.bg_color = Color(0.12, 0.16, 0.24, 0.95)
			rd_style.border_color = Color(0.35, 0.55, 0.85, 0.9)
			rd_style.set_border_width_all(1)
			rd_style.set_corner_radius_all(3)
			rd_style.content_margin_left = 8
			rd_style.content_margin_right = 8
			rd_style.content_margin_top = 2
			rd_style.content_margin_bottom = 2
			rd_panel.add_theme_stylebox_override("panel", rd_style)

			var rd_lbl := Label.new()
			rd_lbl.text = "━━━ ROUND %d RESOLUTION STREAM ━━━" % entry.round_num
			rd_lbl.add_theme_font_size_override("font_size", 9)
			rd_lbl.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
			rd_panel.add_child(rd_lbl)
			modal_entries_vbox.add_child(rd_panel)

		# Block Divider
		if entry.block_num != last_block:
			last_block = entry.block_num
			var blk_lbl := Label.new()
			blk_lbl.text = "── Block %02d Actions ──" % entry.block_num
			blk_lbl.add_theme_font_size_override("font_size", 8)
			blk_lbl.add_theme_color_override("font_color", Color(0.6, 0.7, 0.8))
			modal_entries_vbox.add_child(blk_lbl)

		var cell := _format_action_cell(entry, true)
		modal_entries_vbox.add_child(cell)


func _on_log_modal_cell_clicked(entry: ActionLogEntry) -> void:
	var idx: int = all_action_log_entries.find(entry)
	if idx != -1:
		active_log_entry_index = idx
		_update_active_log_cell()
		_trigger_action_impact_fx(entry)
		var actor := _find_combatant_by_name(entry.actor_name)
		if actor != null:
			_select_combatant(actor)
		if combat_phase == CombatPhase.REPLAY and entry.block_num < snapshots.size():
			replay_block = entry.block_num
			_restore_snapshot(snapshots[replay_block])
			_update_timeline_tracks_display()
			_update_combined_block_display(replay_block)


func _spawn_floating_combat_text(pos: Vector2, text: String, color: Color) -> void:
	var overlay: Control = ui_canvas.find_child("UnitOverlay", true, false) as Control
	if not overlay:
		return
	var lbl := Label.new()
	lbl.text = text
	lbl.add_theme_font_size_override("font_size", 10)
	lbl.add_theme_color_override("font_color", color)
	lbl.add_theme_color_override("font_shadow_color", Color(0, 0, 0, 0.9))
	lbl.add_theme_constant_override("shadow_offset_x", 1)
	lbl.add_theme_constant_override("shadow_offset_y", 1)
	lbl.position = pos + Vector2(8, -6)
	lbl.scale = Vector2(1.2, 1.2)
	overlay.add_child(lbl)

	var tw := create_tween()
	tw.set_parallel(true)
	tw.tween_property(lbl, "position", pos + Vector2(8, -28), 0.55).set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tw.tween_property(lbl, "scale", Vector2(1.0, 1.0), 0.25)
	tw.tween_property(lbl, "modulate:a", 0.0, 0.55).set_delay(0.2)
	tw.chain().tween_callback(lbl.queue_free)


func _animate_attacker_lunge(actor_token: Control, target_pos: Vector2) -> void:
	if not is_instance_valid(actor_token):
		return
	var orig_pos: Vector2 = actor_token.position
	var diff := target_pos - orig_pos
	if diff.length() < 0.1:
		return
	var dir := diff.normalized() * 10.0
	var tw := create_tween()
	tw.tween_property(actor_token, "position", orig_pos + dir, 0.08).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	tw.tween_property(actor_token, "position", orig_pos, 0.12).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_IN)


func _animate_token_impact(target_token: Control, is_damage: bool, is_mitigated: bool) -> void:
	if not is_instance_valid(target_token):
		return
	var base_pos: Vector2 = target_token.position
	var shake_tw := create_tween()
	shake_tw.tween_property(target_token, "position", base_pos + Vector2(4, 0), 0.03)
	shake_tw.tween_property(target_token, "position", base_pos + Vector2(-4, 0), 0.03)
	shake_tw.tween_property(target_token, "position", base_pos + Vector2(2, 0), 0.03)
	shake_tw.tween_property(target_token, "position", base_pos, 0.04)

	var flash_col: Color
	if is_mitigated:
		flash_col = Color(0.4, 0.8, 1.8)
	elif is_damage:
		flash_col = Color(2.0, 0.35, 0.35)
	else:
		flash_col = Color(1.8, 1.6, 0.4)

	var flash_tw := create_tween()
	target_token.modulate = flash_col
	flash_tw.tween_property(target_token, "modulate", Color.WHITE, 0.25)


func _animate_token_pulse(token: Control, pulse_col: Color) -> void:
	if not is_instance_valid(token):
		return
	var tw := create_tween()
	token.modulate = pulse_col
	tw.tween_property(token, "modulate", Color.WHITE, 0.3)


func _trigger_action_impact_fx(entry: ActionLogEntry) -> void:
	var actor := _find_combatant_by_name(entry.actor_name)
	var target := _find_combatant_by_name(entry.target_name)
	var actor_token: Control = null
	if actor != null and unit_token_nodes.has(actor):
		@warning_ignore("unsafe_cast")
		actor_token = unit_token_nodes[actor] as Control
	var target_token: Control = null
	if target != null and unit_token_nodes.has(target):
		@warning_ignore("unsafe_cast")
		target_token = unit_token_nodes[target] as Control

	match entry.category:
		"ATTACK":
			if actor_token != null and target_token != null:
				_animate_attacker_lunge(actor_token, target_token.position)
			if target_token != null:
				var is_mitigated: bool = ("MITIGAT" in entry.outcome_tag)
				_animate_token_impact(target_token, true, is_mitigated)
				var txt := "-%d" % entry.value_num
				if is_mitigated:
					txt += " [MIT]"
				_spawn_floating_combat_text(target_token.position, txt, Color(0.4, 0.85, 1.0) if is_mitigated else Color(1.0, 0.35, 0.35))
				if entry.outcome_tag == "DEFEATED":
					_spawn_floating_combat_text(target_token.position + Vector2(0, -14), "💀 DEFEATED!", Color(1.0, 0.25, 0.25))

		"DEFEND":
			if actor_token != null:
				_animate_token_pulse(actor_token, Color(0.5, 0.8, 1.8))
				_spawn_floating_combat_text(actor_token.position, "🛡 DEFEND [%d%%]" % entry.value_num, Color(0.4, 0.8, 1.0))

		"BUFF":
			if actor_token != null:
				_animate_token_pulse(actor_token, Color(1.8, 1.5, 0.4))
				_spawn_floating_combat_text(actor_token.position, "✦ +%d%% ATK" % entry.value_num, Color(1.0, 0.85, 0.25))

		"MOVE":
			if actor_token != null and entry.coord_to != Vector2i(-1, -1):
				var target_px := Vector2(entry.coord_to.x * TILE_SIZE, entry.coord_to.y * TILE_SIZE)
				var tw := create_tween()
				tw.tween_property(actor_token, "position", target_px, 0.25).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)

		"EVENT":
			if "WITHDRAWN" in entry.outcome_tag and actor_token != null:
				_spawn_floating_combat_text(actor_token.position, "⚑ WITHDRAWN!", Color(0.35, 0.95, 0.55))
			elif "AMBUSH" in entry.outcome_tag:
				var overlay: Control = ui_canvas.find_child("UnitOverlay", true, false) as Control
				if overlay:
					_spawn_floating_combat_text(Vector2(200, 180), "⚠️ AMBUSH SPAWN!", Color(1.0, 0.4, 0.4))


func _on_step_block_pressed() -> void:
	_step_simulation()


func _on_play_toggle_pressed() -> void:
	if is_auto_playing:
		is_auto_playing = false
		auto_play_timer.stop()
		btn_play.text = "▶"
		if btn_modal_play: btn_modal_play.text = "▶"
	else:
		is_auto_playing = true
		btn_play.text = "⏸"
		if btn_modal_play: btn_modal_play.text = "⏸"
		auto_play_timer.start()


func _on_auto_play_tick() -> void:
	_step_simulation()


func _on_reset_pressed() -> void:
	_init_interactive_simulation()


func _on_rewind_pressed() -> void:
	replay_block = 1
	current_sim_block = 1
	var snaps: Array = round_snapshots.get(viewed_timeline_round, snapshots)
	if snaps.size() > 1:
		@warning_ignore("unsafe_cast")
		_restore_snapshot(snaps[1] as BlockSnapshot)
	elif not snaps.is_empty():
		@warning_ignore("unsafe_cast")
		_restore_snapshot(snaps[0] as BlockSnapshot)
	status_label.text = "Rewound to Block 1."
	_update_timeline_tracks_display()
	if timeline_modal_overlay != null and timeline_modal_overlay.visible:
		_update_modal_tracks_display()


func _on_toggle_view_mode_pressed() -> void:
	_toggle_timeline_modal(not timeline_modal_overlay.visible)


func _on_ruler_block_clicked(b: int) -> void:
	replay_block = b
	current_sim_block = b
	# Directly traverse through battle state for clicked block
	var snaps: Array = round_snapshots.get(viewed_timeline_round, snapshots)
	if b >= 0 and b < snaps.size():
		@warning_ignore("unsafe_cast")
		_restore_snapshot(snaps[b] as BlockSnapshot)
	elif not snaps.is_empty():
		@warning_ignore("unsafe_cast")
		_restore_snapshot(snaps[0] as BlockSnapshot)

	status_label.text = "Round %d • Block %02d State inspected." % [viewed_timeline_round, b]
	_update_timeline_tracks_display()
	if timeline_modal_overlay != null and timeline_modal_overlay.visible:
		_update_modal_tracks_display()


func _update_round_traversal_ui() -> void:
	var r1_btns: Array[Button] = []
	if btn_rnd_1 != null: r1_btns.append(btn_rnd_1)
	if btn_modal_rnd_1 != null: r1_btns.append(btn_modal_rnd_1)
	for b in r1_btns:
		var st := StyleBoxFlat.new()
		st.set_corner_radius_all(3)
		if viewed_timeline_round == 1:
			st.bg_color = Color(0.18, 0.32, 0.54, 0.95)
			st.border_color = Color(1.0, 0.85, 0.3, 1.0)
			st.set_border_width_all(2)
			b.add_theme_color_override("font_color", Color(1.0, 0.9, 0.35))
		else:
			st.bg_color = Color(0.1, 0.13, 0.18, 0.8)
			st.border_color = Color(0.25, 0.35, 0.48, 0.6)
			st.set_border_width_all(1)
			b.add_theme_color_override("font_color", Color(0.65, 0.75, 0.85))
		b.add_theme_stylebox_override("normal", st)

	var r2_btns: Array[Button] = []
	if btn_rnd_2 != null: r2_btns.append(btn_rnd_2)
	if btn_modal_rnd_2 != null: r2_btns.append(btn_modal_rnd_2)
	for b in r2_btns:
		var st := StyleBoxFlat.new()
		st.set_corner_radius_all(3)
		if viewed_timeline_round == 2:
			st.bg_color = Color(0.18, 0.32, 0.54, 0.95)
			st.border_color = Color(1.0, 0.85, 0.3, 1.0)
			st.set_border_width_all(2)
			b.add_theme_color_override("font_color", Color(1.0, 0.9, 0.35))
		else:
			st.bg_color = Color(0.1, 0.13, 0.18, 0.8)
			st.border_color = Color(0.25, 0.35, 0.48, 0.6)
			st.set_border_width_all(1)
			b.add_theme_color_override("font_color", Color(0.65, 0.75, 0.85))
		b.add_theme_stylebox_override("normal", st)


func _select_timeline_round(rd: int) -> void:
	viewed_timeline_round = rd
	@warning_ignore("unsafe_cast")
	recorded_actions = round_recorded_actions.get(rd, []) as Array[CombatResolver.ScheduledAction]
	@warning_ignore("unsafe_cast")
	snapshots = round_snapshots.get(rd, []) as Array[BlockSnapshot]
	_update_round_traversal_ui()
	_update_timeline_tracks_display()
	if timeline_modal_overlay != null and timeline_modal_overlay.visible:
		_update_modal_tracks_display()

	var snaps: Array = round_snapshots.get(rd, [])
	var target_b: int = replay_block if combat_phase == CombatPhase.REPLAY else 1
	if target_b >= 0 and target_b < snaps.size():
		@warning_ignore("unsafe_cast")
		_restore_snapshot(snaps[target_b] as BlockSnapshot)
	elif not snaps.is_empty():
		@warning_ignore("unsafe_cast")
		_restore_snapshot(snaps[0] as BlockSnapshot)
	status_label.text = "Viewing Round %d Timeline." % rd


# ==============================================================================
# SECTION 8: FULL-SCREEN POPOUT MODAL TIMELINE SUITE
# ==============================================================================

func _build_timeline_modal_ui() -> void:
	timeline_modal_overlay = PanelContainer.new()
	timeline_modal_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	timeline_modal_overlay.visible = false
	timeline_modal_overlay.mouse_filter = Control.MOUSE_FILTER_STOP

	var overlay_style := StyleBoxFlat.new()
	overlay_style.bg_color = Color(0.04, 0.06, 0.09, 0.97)
	overlay_style.border_color = Color(0.25, 0.45, 0.75, 0.95)
	overlay_style.set_border_width_all(2)
	overlay_style.set_corner_radius_all(8)
	overlay_style.content_margin_left = 16
	overlay_style.content_margin_right = 16
	overlay_style.content_margin_top = 12
	overlay_style.content_margin_bottom = 12
	timeline_modal_overlay.add_theme_stylebox_override("panel", overlay_style)
	ui_canvas.add_child(timeline_modal_overlay)

	var modal_vbox := VBoxContainer.new()
	modal_vbox.add_theme_constant_override("separation", 8)
	timeline_modal_overlay.add_child(modal_vbox)

	# 1. Header Bar: Title | Playback Controls | Phase Badge | Close Button
	var hdr_row := HBoxContainer.new()
	hdr_row.add_theme_constant_override("separation", 10)
	modal_vbox.add_child(hdr_row)

	var title_lbl := Label.new()
	title_lbl.text = "🗂 TACTICAL TIMELINE MASTER SUITE (DETAILED EXPANDED VIEW)"
	title_lbl.add_theme_font_size_override("font_size", 12)
	title_lbl.add_theme_color_override("font_color", Color(1.0, 0.85, 0.35))
	hdr_row.add_child(title_lbl)

	hdr_row.add_spacer(false)

	var modal_ctrls := HBoxContainer.new()
	modal_ctrls.add_theme_constant_override("separation", 4)
	hdr_row.add_child(modal_ctrls)

	btn_modal_rewind = Button.new()
	btn_modal_rewind.text = "⏮"
	btn_modal_rewind.tooltip_text = "Rewind to Block 1"
	btn_modal_rewind.custom_minimum_size = Vector2(28, 22)
	btn_modal_rewind.add_theme_font_size_override("font_size", 10)
	btn_modal_rewind.pressed.connect(_on_rewind_pressed)
	modal_ctrls.add_child(btn_modal_rewind)

	btn_modal_play = Button.new()
	btn_modal_play.text = "▶"
	btn_modal_play.tooltip_text = "Play Auto-Resolve / Play Replay"
	btn_modal_play.custom_minimum_size = Vector2(30, 22)
	btn_modal_play.add_theme_font_size_override("font_size", 10)
	btn_modal_play.pressed.connect(_on_play_toggle_pressed)
	modal_ctrls.add_child(btn_modal_play)

	btn_modal_step = Button.new()
	btn_modal_step.text = "⏭"
	btn_modal_step.tooltip_text = "Step 1 Block Forward"
	btn_modal_step.custom_minimum_size = Vector2(28, 22)
	btn_modal_step.add_theme_font_size_override("font_size", 10)
	btn_modal_step.pressed.connect(_on_step_block_pressed)
	modal_ctrls.add_child(btn_modal_step)

	btn_modal_reset = Button.new()
	btn_modal_reset.text = "↺"
	btn_modal_reset.tooltip_text = "Reset Round"
	btn_modal_reset.custom_minimum_size = Vector2(28, 22)
	btn_modal_reset.add_theme_font_size_override("font_size", 10)
	btn_modal_reset.pressed.connect(_on_reset_pressed)
	modal_ctrls.add_child(btn_modal_reset)

	# Round Traversal Buttons (R1 / R2) in modal suite
	btn_modal_rnd_1 = Button.new()
	btn_modal_rnd_1.text = "R1"
	btn_modal_rnd_1.tooltip_text = "Switch Modal to Round 1 Timeline"
	btn_modal_rnd_1.custom_minimum_size = Vector2(26, 22)
	btn_modal_rnd_1.add_theme_font_size_override("font_size", 9)
	btn_modal_rnd_1.pressed.connect(func() -> void: _select_timeline_round(1))
	modal_ctrls.add_child(btn_modal_rnd_1)

	btn_modal_rnd_2 = Button.new()
	btn_modal_rnd_2.text = "R2"
	btn_modal_rnd_2.tooltip_text = "Switch Modal to Round 2 Timeline"
	btn_modal_rnd_2.custom_minimum_size = Vector2(26, 22)
	btn_modal_rnd_2.add_theme_font_size_override("font_size", 9)
	btn_modal_rnd_2.pressed.connect(func() -> void: _select_timeline_round(2))
	modal_ctrls.add_child(btn_modal_rnd_2)

	modal_phase_lbl = Label.new()
	modal_phase_lbl.text = "PLANNING PHASE"
	modal_phase_lbl.add_theme_font_size_override("font_size", 9)
	modal_phase_lbl.add_theme_color_override("font_color", Color(0.95, 0.8, 0.35))
	modal_ctrls.add_child(modal_phase_lbl)

	hdr_row.add_spacer(false)

	btn_modal_close = Button.new()
	btn_modal_close.text = "✕ CLOSE [ESC]"
	btn_modal_close.tooltip_text = "Close expanded timeline [ESC / F / Tab]"
	btn_modal_close.custom_minimum_size = Vector2(90, 22)
	btn_modal_close.add_theme_font_size_override("font_size", 9)
	var close_style := StyleBoxFlat.new()
	close_style.bg_color = Color(0.35, 0.12, 0.14, 0.9)
	close_style.border_color = Color(0.85, 0.35, 0.35, 1.0)
	close_style.set_border_width_all(1)
	close_style.set_corner_radius_all(3)
	btn_modal_close.add_theme_stylebox_override("normal", close_style)
	btn_modal_close.pressed.connect(func() -> void: _toggle_timeline_modal(false))
	hdr_row.add_child(btn_modal_close)

	var hsep1 := HSeparator.new()
	modal_vbox.add_child(hsep1)

	# 2. Synchronous Time Ruler Row
	var ruler_row := HBoxContainer.new()
	ruler_row.add_theme_constant_override("separation", BLOCK_GAP)
	modal_vbox.add_child(ruler_row)

	var track_col_lbl := Label.new()
	track_col_lbl.text = "COMBATANT"
	track_col_lbl.custom_minimum_size = Vector2(130, 18)
	track_col_lbl.add_theme_font_size_override("font_size", 9)
	track_col_lbl.add_theme_color_override("font_color", Color(0.6, 0.75, 0.9))
	ruler_row.add_child(track_col_lbl)

	modal_ruler_tick_nodes.clear()
	modal_ruler_tick_labels.clear()

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
				_on_ruler_block_clicked(cur_b)
				if timeline_modal_overlay.visible:
					_update_modal_tracks_display()
		)

		ruler_row.add_child(tick_panel)
		modal_ruler_tick_nodes.append(tick_panel)
		modal_ruler_tick_labels.append(tick_lbl)

	# 3. Scrollable Tracks Matrix
	var scroll := ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	modal_vbox.add_child(scroll)

	modal_tracks_list = VBoxContainer.new()
	modal_tracks_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	modal_tracks_list.add_theme_constant_override("separation", 6)
	scroll.add_child(modal_tracks_list)


func _update_modal_tracks_display() -> void:
	if not timeline_modal_overlay.visible:
		return

	var active_playhead: int = replay_block if combat_phase == CombatPhase.REPLAY else current_sim_block
	modal_phase_lbl.text = "REPLAY BLOCK %02d / 10" % active_playhead if combat_phase == CombatPhase.REPLAY else "PLANNING BLOCK %02d / 10" % active_playhead
	btn_modal_play.text = "⏸" if is_auto_playing else "▶"

	# Update modal ruler tick highlights
	for i in range(modal_ruler_tick_nodes.size()):
		var pnl: PanelContainer = modal_ruler_tick_nodes[i]
		var lbl: Label = modal_ruler_tick_labels[i]
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

	# Rebuild tracks list in modal
	for child in modal_tracks_list.get_children():
		child.queue_free()

	# Query snapshot for active playhead in viewed round
	var snap: BlockSnapshot = null
	var snaps: Array = round_snapshots.get(viewed_timeline_round, snapshots)
	if active_playhead >= 0 and active_playhead < snaps.size():
		@warning_ignore("unsafe_cast")
		snap = snaps[active_playhead] as BlockSnapshot

	# Section 1: Hero Battlegroup Tracks (All 4 slots)
	var ally_hdr := Label.new()
	ally_hdr.text = "⚔ ALLIED HERO BATTLEGROUP (4 SLOTS)"
	ally_hdr.add_theme_font_size_override("font_size", 9)
	ally_hdr.add_theme_color_override("font_color", Color(0.4, 0.75, 1.0))
	modal_tracks_list.add_child(ally_hdr)

	for i in range(4):
		var ally: Combatant = sim_allies[i] if i < sim_allies.size() else null
		var row := _create_modal_unit_track_row(ally, i + 1, true, active_playhead, snap)
		modal_tracks_list.add_child(row)

	var sep := HSeparator.new()
	modal_tracks_list.add_child(sep)

	# Section 2: Hostile Opposition Tracks (Requirement 5: Only show active enemies for the round)
	var active_enemies_for_view: Array[Combatant] = []
	var is_r1: bool = (viewed_timeline_round == 1)
	var reinforcements_active: bool = (snap.reinforcements_spawned if snap != null else reinforcements_spawned)

	for enemy in sim_enemies:
		if is_r1:
			if enemy in [unit_berserker, unit_skirmisher, unit_archer, unit_shaman]:
				if reinforcements_active:
					active_enemies_for_view.append(enemy)
			else:
				active_enemies_for_view.append(enemy)
		else:
			# Round 2: Brawler & Scout pruned, only surviving goblins shown
			if not (enemy in [unit_brawler, unit_scout]):
				active_enemies_for_view.append(enemy)

	var enemy_hdr := Label.new()
	enemy_hdr.text = "💀 HOSTILE OPPOSITION (%d Active Hostiles)" % active_enemies_for_view.size()
	enemy_hdr.add_theme_font_size_override("font_size", 9)
	enemy_hdr.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
	modal_tracks_list.add_child(enemy_hdr)

	for i in range(active_enemies_for_view.size()):
		var enemy: Combatant = active_enemies_for_view[i]
		var row := _create_modal_unit_track_row(enemy, i + 1, false, active_playhead, snap)
		modal_tracks_list.add_child(row)


func _create_modal_unit_track_row(combatant: Combatant, slot_idx: int, is_ally: bool, active_playhead: int, snap: BlockSnapshot = null) -> HBoxContainer:
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
		icon.texture = _get_combatant_token_texture(combatant)
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
	if not is_ally and combat_phase != CombatPhase.REPLAY:
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

	# Otherwise render action clips from recorded_actions
	var b: int = 1
	while b <= TOTAL_BLOCKS:
		var act: CombatResolver.ScheduledAction = _find_recorded_action_starting_at(combatant, b)
		if act != null:
			var cost: int = act.end_block - act.start_block + 1
			var clip_w: int = cost * UNIT_MODAL_BLOCK_W + (cost - 1) * BLOCK_GAP
			var clip := _create_action_clip_widget(act, clip_w, cost)
			clips_hbox.add_child(clip)
			b += cost
		else:
			var idle := _create_idle_clip_widget(b == active_playhead, UNIT_MODAL_BLOCK_W)
			clips_hbox.add_child(idle)
			b += 1

	return row


func _toggle_timeline_modal(open: bool) -> void:
	if timeline_modal_overlay == null:
		return
	timeline_modal_overlay.visible = open
	if open:
		timeline_modal_overlay.move_to_front()
		_update_modal_tracks_display()
	else:
		_update_timeline_tracks_display()
		_update_combined_block_display(combined_selected_block)


func _unhandled_input(event: InputEvent) -> void:
	var key_event := event as InputEventKey
	if key_event != null and key_event.pressed and not key_event.echo:
		if key_event.keycode == KEY_ESCAPE:
			if log_modal_overlay != null and log_modal_overlay.visible:
				_toggle_log_modal(false)
				get_viewport().set_input_as_handled()
				return
			elif timeline_modal_overlay != null and timeline_modal_overlay.visible:
				_toggle_timeline_modal(false)
				get_viewport().set_input_as_handled()
				return
		elif key_event.keycode == KEY_L:
			if log_modal_overlay != null:
				_toggle_log_modal(not log_modal_overlay.visible)
				get_viewport().set_input_as_handled()
				return
		elif key_event.keycode == KEY_F:
			if timeline_modal_overlay != null:
				_toggle_timeline_modal(not timeline_modal_overlay.visible)
				get_viewport().set_input_as_handled()
				return
		elif key_event.keycode == KEY_BRACKETLEFT:
			_on_log_prev_pressed()
			get_viewport().set_input_as_handled()
			return
		elif key_event.keycode == KEY_BRACKETRIGHT:
			_on_log_next_pressed()
			get_viewport().set_input_as_handled()
			return
		elif key_event.keycode == KEY_Q or key_event.keycode == KEY_LEFT:
			_navigate_combined_block(-1)
		elif key_event.keycode == KEY_E or key_event.keycode == KEY_RIGHT:
			_navigate_combined_block(1)
		elif key_event.keycode == KEY_SPACE:
			_on_play_toggle_pressed()
		elif key_event.keycode == KEY_TAB:
			_on_toggle_view_mode_pressed()


func _append_log(text: String) -> void:
	if log_rich_text:
		log_rich_text.append_text(text + "\n")


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


func _run_goblin_ambush_and_casualty_test() -> bool:
	print("\n=======================================================")
	print("=== STARTING GOBLIN AMBUSH & CASUALTY TEST (MULTI-RD) ===")
	print("=======================================================")

	var test_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var test_resolver := CombatResolver.new()
	test_resolver.grid = test_grid

	var warrior := Combatant.new("Warrior", 100, 15, 1)
	var ranger := Combatant.new("Ranger", 80, 14, 2)
	var brawler := Combatant.new("Enemy Brawler", 45, 12, 1)
	var scout := Combatant.new("Fleeing Scout", 30, 16, 1)

	var skirmisher := Combatant.new("Goblin Skirmisher", 40, 15, 1)
	var archer := Combatant.new("Goblin Archer", 35, 13, 3)
	var berserker := Combatant.new("Goblin Berserker", 55, 11, 1)
	var shaman := Combatant.new("Goblin Shaman", 35, 12, 2)

	var allies: Array[Combatant] = [warrior, ranger]
	var initial_enemies: Array[Combatant] = [brawler, scout]
	var all_enemies: Array[Combatant] = [brawler, scout, skirmisher, archer, berserker, shaman]

	for c: Combatant in [warrior, ranger, brawler, scout, skirmisher, archer, berserker, shaman]:
		test_resolver.register_combatant(c)

	test_grid.deploy_combatants(allies, initial_enemies)

	var init_placement_ok: bool = (test_grid.has_combatant(warrior) and test_grid.has_combatant(ranger) and
		test_grid.has_combatant(brawler) and test_grid.has_combatant(scout) and
		not test_grid.has_combatant(skirmisher) and not test_grid.has_combatant(archer))

	# Schedule Round 1 actions:
	test_resolver.schedule_action(warrior, MoveCommand.new(warrior, test_grid, Vector2i(0, -1), 1), 1)
	test_resolver.schedule_action(warrior, AttackCommand.new(warrior, brawler, 25, 2, 0, test_grid), 2)
	test_resolver.schedule_action(warrior, TargetCommand.new(warrior, skirmisher, 1), 4)
	test_resolver.schedule_action(warrior, AttackCommand.new(warrior, skirmisher, 25, 2, 0, test_grid), 5)
	test_resolver.schedule_action(warrior, DefendCommand.new(warrior, 0.5, 2, 1), 7)
	test_resolver.schedule_action(warrior, HoldStanceCommand.new(warrior, 0), 9)

	test_resolver.schedule_action(ranger, FocusCommand.new(ranger, 1.5, 2, 2, 0), 1)
	test_resolver.schedule_action(ranger, MoveCommand.new(ranger, test_grid, Vector2i(0, -1), 1), 4)
	test_resolver.schedule_action(ranger, AttackCommand.new(ranger, brawler, 30, 2, 0, test_grid), 5)
	var ranger_future_act := AttackCommand.new(ranger, archer, 25, 2, 0, test_grid)
	test_resolver.schedule_action(ranger, ranger_future_act, 7)

	test_resolver.schedule_action(brawler, TargetCommand.new(brawler, warrior, 2), 1)
	test_resolver.schedule_action(brawler, DefendCommand.new(brawler, 0.5, 2, 1), 2)
	test_resolver.schedule_action(brawler, AttackCommand.new(brawler, warrior, 15, 2, 0, test_grid), 4)

	test_resolver.schedule_action(scout, MoveCommand.new(scout, test_grid, Vector2i(1, 0), 1), 1)
	test_resolver.schedule_action(scout, MoveCommand.new(scout, test_grid, Vector2i(1, 0), 2, 2), 2)

	# Execute Block 1..3
	for b: int in range(1, 4):
		test_resolver.execute_block(b)

	var scout_escaped_ok: bool = scout.is_withdrawn and not scout.is_active() and not test_grid.has_combatant(scout)

	# Block 4 Ambush: Reinforcements arrive on grid
	test_grid.place_combatant(berserker, Vector2i(5, 5))
	test_grid.place_combatant(skirmisher, Vector2i(6, 4))
	test_grid.place_combatant(archer, Vector2i(7, 5))
	test_grid.place_combatant(shaman, Vector2i(6, 5))

	test_resolver.schedule_action(berserker, TargetCommand.new(berserker, ranger, 1), 4)
	test_resolver.schedule_action(berserker, AttackCommand.new(berserker, ranger, 30, 2, 0, test_grid), 5)
	test_resolver.schedule_action(archer, TargetCommand.new(archer, ranger, 1), 4)
	test_resolver.schedule_action(archer, AttackCommand.new(archer, ranger, 25, 2, 0, test_grid), 5)
	test_resolver.schedule_action(shaman, TargetCommand.new(shaman, ranger, 1), 4)
	test_resolver.schedule_action(shaman, AttackCommand.new(shaman, ranger, 25, 2, 0, test_grid), 5)

	test_resolver.schedule_action(skirmisher, MoveCommand.new(skirmisher, test_grid, Vector2i(-1, 0), 1), 4)
	test_resolver.schedule_action(skirmisher, TargetCommand.new(skirmisher, warrior, 1), 5)
	test_resolver.schedule_action(skirmisher, AttackCommand.new(skirmisher, warrior, 12, 2, 0, test_grid), 7)

	# Execute Block 4..6
	for b: int in range(4, 7):
		test_resolver.execute_block(b)

	var ranger_defeated_ok: bool = not ranger.is_alive() and ranger.hp == 0
	var brawler_defeated_ok: bool = not brawler.is_alive() and brawler.hp == 0

	var ranger_future_cancelled_ok: bool = false
	for sa: CombatResolver.ScheduledAction in test_resolver.scheduled_actions:
		if sa.combatant == ranger and sa.start_block == 7:
			ranger_future_cancelled_ok = sa.is_cancelled
			break

	# Execute remaining Blocks 7..10 of Round 1
	for b: int in range(7, 11):
		test_resolver.execute_block(b)

	var warrior_survived_r1_ok: bool = warrior.is_alive() and warrior.hp == 73
	var r1_not_prematurely_ended: bool = not test_resolver.is_combat_resolved(allies, all_enemies)

	# Rollover to Round 2
	test_resolver.finish_round()
	var round_2_started_ok: bool = test_resolver.current_round == 2

	# Schedule Round 2 action for Warrior: Finish Skirmisher at (5, 4)
	test_resolver.schedule_action(warrior, AttackCommand.new(warrior, skirmisher, 25, 2, 0, test_grid), 1)
	test_resolver.execute_block(1)
	test_resolver.execute_block(2)

	var skirmisher_defeated_r2_ok: bool = not skirmisher.is_alive() and skirmisher.hp == 0

	print("\n=======================================================")
	print("=== GOBLIN AMBUSH & CASUALTY TEST ASSERTIONS        ===")
	print("=======================================================")
	print("  [Ambush] Initial grid deployment (4 units): %s" % ("PASS" if init_placement_ok else "FAIL"))
	print("  [Ambush] Scout withdrawal at Block 3: %s" % ("PASS" if scout_escaped_ok else "FAIL"))
	print("  [Casualty] Ranger defeated at Block 6: %s" % ("PASS" if ranger_defeated_ok else "FAIL"))
	print("  [Casualty] Ranger future actions cancelled: %s" % ("PASS" if ranger_future_cancelled_ok else "FAIL"))
	print("  [Casualty] Brawler defeated at Block 6: %s" % ("PASS" if brawler_defeated_ok else "FAIL"))
	print("  [Multi-Round] Warrior survived Round 1 (73 HP): %s" % ("PASS" if warrior_survived_r1_ok else "FAIL"))
	print("  [Multi-Round] Combat active with 1 ally & 4 goblins: %s" % ("PASS" if r1_not_prematurely_ended else "FAIL"))
	print("  [Multi-Round] Rollover to Round 2 succeeded: %s" % ("PASS" if round_2_started_ok else "FAIL"))
	print("  [Multi-Round] Round 2 combat resolved Skirmisher: %s" % ("PASS" if skirmisher_defeated_r2_ok else "FAIL"))

	warrior.clear_target()
	ranger.clear_target()
	brawler.clear_target()
	scout.clear_target()
	skirmisher.clear_target()
	archer.clear_target()
	berserker.clear_target()
	shaman.clear_target()
	test_resolver.registered_combatants.clear()

	return (init_placement_ok and scout_escaped_ok and ranger_defeated_ok and
		ranger_future_cancelled_ok and brawler_defeated_ok and warrior_survived_r1_ok and
		r1_not_prematurely_ended and round_2_started_ok and skirmisher_defeated_r2_ok)