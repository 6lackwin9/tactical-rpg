extends Node2D

# ==============================================================================
# Tactical RPG - Gear & Skill Benchmark Dojo
# Dedicated workbench for character gear equipping, stat allocation,
# horizontal masteries, training dummy benchmark, and damage formula verification.
# ==============================================================================

# Hero & Target State
var party_allies: Array[Combatant] = []
var active_hero: Combatant = null
var test_dummy: Combatant = null
var shared_stash: Inventory = null

# Visual Dojo Stage State
var hero_token_ctrl: Control
var dummy_token_ctrl: Control
var hero_sprite_rect: TextureRect
var dummy_sprite_rect: TextureRect
var dummy_hp_bar: ProgressBar
var dummy_hp_lbl: Label
var dummy_stance_badge: Label
var formula_log_rich: RichTextLabel

# UI Docks & Equipment Modal
var canvas_layer: CanvasLayer
var gear_modal: CharacterEquipmentModal

# Stats & Derived UI Labels
var hero_tabs_hbox: HBoxContainer
var hero_title_lbl: Label
var hero_lvl_lbl: Label
var hero_points_btn: Button
var stat_display_labels: Dictionary = {} # String -> Label
var stat_box_containers: Dictionary = {} # String -> PanelContainer
var stat_plus_buttons: Dictionary = {}   # String -> Button
var stat_minus_buttons: Dictionary = {}  # String -> Button
var stat_allocation_controls_vbox: VBoxContainer
var alloc_rem_lbl: Label
var is_allocating_stats: bool = false
var pending_allocated_stats: Dictionary = {"STR": 0, "CONS": 0, "DEX": 0, "PERC": 0, "INT": 0, "KNOW": 0}
var pending_free_points: int = 0

var derived_melee_lbl: Label
var derived_ranged_lbl: Label
var derived_speed_lbl: Label
var derived_hp_lbl: Label
var derived_mit_lbl: Label
var derived_burden_lbl: Label
var derived_crit_lbl: Label
var derived_resist_lbl: Label
var derived_rows: Dictionary = {} # String -> HBoxContainer

var loadout_cards_vbox: VBoxContainer
var battle_pouch_vbox: VBoxContainer

# Base UI Gear Picker Modal
var gear_picker_dialog: PanelContainer
var picker_items_vbox: VBoxContainer
var picker_title_lbl: Label
var picker_slot: Equipment.EquipSlot = Equipment.EquipSlot.MAIN_HAND

# Action Buttons
var btn_melee_strike: Button
var btn_offhand_strike: Button
var btn_ranged_volley: Button
var btn_switch_active_weapon: Button
var force_crit_check: CheckBox
var pouch_select_option: OptionButton
var btn_use_pouch_item: Button

# Dojo Stage Dimensions
const STAGE_WIDTH: int = 420
const STAGE_HEIGHT: int = 240
const HERO_POS: Vector2 = Vector2(90, 110)
const DUMMY_POS: Vector2 = Vector2(310, 110)

func _ready() -> void:
	print("\n=======================================================")
	print("=== TACTICAL RPG: GEAR & SKILL BENCHMARK DOJO       ===")
	print("=======================================================")

	_init_dojo_data()

	if DisplayServer.get_name() == "headless":
		print("Running headlessly. Executing Dojo verification assertions...")
		var ok: bool = _run_headless_dojo_assertions()
		var code: int = 0 if ok else 1
		print("Dojo headless test complete. Exiting with code: %d\n" % code)
		get_tree().quit(code)
	else:
		print("Initializing interactive Gear & Skill Dojo UI...\n")
		_build_dojo_ui()
		_refresh_all_views()

# ------------------------------------------------------------------------------
# 1. State & Inventory Initialization
# ------------------------------------------------------------------------------

func _init_dojo_data() -> void:
	shared_stash = Inventory.new()
	ItemCatalog.populate_party_starter_stash(shared_stash)

	# 1. Allied Heroes
	var warrior_stats := CharacterStats.new(ClassArchetype.create_warrior(), 2)
	var warrior := Combatant.new("Warrior", 100, 15, 1, warrior_stats)
	ItemCatalog.setup_default_hero_loadout(warrior, "Warrior")

	var ranger_stats := CharacterStats.new(ClassArchetype.create_ranger(), 1)
	var ranger := Combatant.new("Ranger", 80, 14, 2, ranger_stats)
	ItemCatalog.setup_default_hero_loadout(ranger, "Ranger")

	party_allies = [warrior, ranger]
	active_hero = warrior

	# 2. Training Dummy (500 HP, 0 defense mitigation by default)
	var dummy_stats := CharacterStats.new()
	test_dummy = Combatant.new("Training Dummy", 500, 10, 1, dummy_stats)
	test_dummy.defense_mitigation_ratio = 0.0

# ------------------------------------------------------------------------------
# 2. Interactive UI Construction
# ------------------------------------------------------------------------------

func _build_dojo_ui() -> void:
	canvas_layer = CanvasLayer.new()
	add_child(canvas_layer)

	# Full-screen dark background
	var bg := ColorRect.new()
	bg.color = Color(0.06, 0.08, 0.12, 1.0)
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	canvas_layer.add_child(bg)

	var root_margin := MarginContainer.new()
	root_margin.set_anchors_preset(Control.PRESET_FULL_RECT)
	root_margin.add_theme_constant_override("margin_left", 14)
	root_margin.add_theme_constant_override("margin_right", 14)
	root_margin.add_theme_constant_override("margin_top", 10)
	root_margin.add_theme_constant_override("margin_bottom", 10)

	# Compact Tooltip Theme (Requirement 7)
	var dojo_theme := Theme.new()
	dojo_theme.set_font_size("font_size", "TooltipLabel", 9)
	var tip_style := StyleBoxFlat.new()
	tip_style.bg_color = Color(0.08, 0.11, 0.16, 0.96)
	tip_style.border_color = Color(0.28, 0.38, 0.52, 0.9)
	tip_style.set_border_width_all(1)
	tip_style.set_corner_radius_all(4)
	tip_style.content_margin_left = 6
	tip_style.content_margin_right = 6
	tip_style.content_margin_top = 4
	tip_style.content_margin_bottom = 4
	dojo_theme.set_stylebox("panel", "TooltipPanel", tip_style)
	root_margin.theme = dojo_theme

	canvas_layer.add_child(root_margin)

	var main_vbox := VBoxContainer.new()
	main_vbox.add_theme_constant_override("separation", 8)
	root_margin.add_child(main_vbox)

	# Header Bar
	var hdr_row := HBoxContainer.new()
	hdr_row.add_theme_constant_override("separation", 10)
	main_vbox.add_child(hdr_row)

	var title_vbox := VBoxContainer.new()
	title_vbox.add_theme_constant_override("separation", 1)
	hdr_row.add_child(title_vbox)

	var title_lbl := Label.new()
	title_lbl.text = "🥋 GEAR & SKILL BENCHMARK DOJO"
	title_lbl.add_theme_font_size_override("font_size", 15)
	title_lbl.add_theme_color_override("font_color", Color(0.95, 0.85, 0.35))
	title_vbox.add_child(title_lbl)

	var sub_lbl := Label.new()
	sub_lbl.text = "Interactive 6-Stat Scaling, Stat-Gated Equipment, Threshold Tiers & Damage Formula Verifier"
	sub_lbl.add_theme_font_size_override("font_size", 9)
	sub_lbl.add_theme_color_override("font_color", Color(0.65, 0.78, 0.90))
	title_vbox.add_child(sub_lbl)

	hdr_row.add_spacer(false)

	# Top Toolbar Buttons
	var btn_open_gear := Button.new()
	btn_open_gear.text = "🎒 Manage Gear & Stash"
	btn_open_gear.add_theme_font_size_override("font_size", 10)
	btn_open_gear.focus_mode = Control.FOCUS_NONE
	btn_open_gear.pressed.connect(func() -> void:
		if gear_modal != null:
			gear_modal.open_modal(active_hero)
	)
	hdr_row.add_child(btn_open_gear)

	var btn_toggle_def := Button.new()
	btn_toggle_def.text = "🛡 Toggle Dummy Defend"
	btn_toggle_def.add_theme_font_size_override("font_size", 10)
	btn_toggle_def.focus_mode = Control.FOCUS_NONE
	btn_toggle_def.pressed.connect(_on_toggle_dummy_defense_pressed)
	hdr_row.add_child(btn_toggle_def)

	var btn_heal_dummy := Button.new()
	btn_heal_dummy.text = "💖 Full Heal Dummy"
	btn_heal_dummy.add_theme_font_size_override("font_size", 10)
	btn_heal_dummy.focus_mode = Control.FOCUS_NONE
	btn_heal_dummy.pressed.connect(_on_heal_dummy_pressed)
	hdr_row.add_child(btn_heal_dummy)

	var btn_reset_dojo := Button.new()
	btn_reset_dojo.text = "↺ Reset Dojo"
	btn_reset_dojo.tooltip_text = "Reset heroes, loadouts, training dummy vitals, and formula logs back to initial state"
	btn_reset_dojo.add_theme_font_size_override("font_size", 10)
	btn_reset_dojo.add_theme_color_override("font_color", Color(0.4, 0.9, 1.0))
	btn_reset_dojo.focus_mode = Control.FOCUS_NONE
	btn_reset_dojo.pressed.connect(_reset_dojo_to_baseline)
	hdr_row.add_child(btn_reset_dojo)

	var sep := HSeparator.new()
	main_vbox.add_child(sep)

	# --------------------------------------------------------------------------
	# MAIN BODY: 3-COLUMN BENCHMARK WORKBENCH
	# Left: Hero Specs & Equipped Loadout (360px)
	# Center: Visual Training Stage & Action Console (460px)
	# Right: Live Damage Formula Inspector & Party Stash (Fill)
	# --------------------------------------------------------------------------
	var body_hbox := HBoxContainer.new()
	body_hbox.add_theme_constant_override("separation", 10)
	body_hbox.size_flags_vertical = Control.SIZE_EXPAND_FILL
	main_vbox.add_child(body_hbox)

	_build_left_specs_dock(body_hbox)
	_build_center_stage_dock(body_hbox)
	_build_right_inspector_dock(body_hbox)

	# Setup Character Equipment Modal
	gear_modal = CharacterEquipmentModal.new()
	gear_modal.sprite_texture_fn = _get_combatant_sprite_texture
	gear_modal.setup_heroes(party_allies, shared_stash)
	gear_modal.equipment_updated.connect(func(_c: Combatant) -> void:
		_refresh_all_views()
	)
	canvas_layer.add_child(gear_modal)

	# Setup Base UI Gear Picker Dialog
	_build_gear_picker_dialog()

# ------------------------------------------------------------------------------
# Left Dock: Hero 6-Stat Specs & Equipped Loadout
# ------------------------------------------------------------------------------

func _build_left_specs_dock(parent: Control) -> void:
	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(350, 0)
	var p_style := StyleBoxFlat.new()
	p_style.bg_color = Color(0.08, 0.11, 0.16, 0.95)
	p_style.border_color = Color(0.22, 0.32, 0.46, 0.8)
	p_style.set_border_width_all(1)
	p_style.set_corner_radius_all(6)
	p_style.content_margin_left = 10
	p_style.content_margin_right = 10
	p_style.content_margin_top = 8
	p_style.content_margin_bottom = 8
	panel.add_theme_stylebox_override("panel", p_style)
	parent.add_child(panel)

	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	panel.add_child(scroll)

	var vbox := VBoxContainer.new()
	vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vbox.add_theme_constant_override("separation", 6)
	scroll.add_child(vbox)

	# Hero Selection Tabs
	hero_tabs_hbox = HBoxContainer.new()
	hero_tabs_hbox.add_theme_constant_override("separation", 4)
	vbox.add_child(hero_tabs_hbox)

	var sep_tabs := HSeparator.new()
	vbox.add_child(sep_tabs)

	# Identity Summary
	hero_title_lbl = Label.new()
	hero_title_lbl.text = "Warrior (Archetype: Warrior)"
	hero_title_lbl.add_theme_font_size_override("font_size", 12)
	hero_title_lbl.add_theme_color_override("font_color", Color.WHITE)
	vbox.add_child(hero_title_lbl)

	var lvl_row := HBoxContainer.new()
	vbox.add_child(lvl_row)

	hero_lvl_lbl = Label.new()
	hero_lvl_lbl.text = "Level: 2 / 10"
	hero_lvl_lbl.add_theme_font_size_override("font_size", 9)
	hero_lvl_lbl.add_theme_color_override("font_color", Color(0.9, 0.8, 0.3))
	lvl_row.add_child(hero_lvl_lbl)

	lvl_row.add_spacer(false)

	hero_points_btn = Button.new()
	hero_points_btn.text = "Unspent Points: 1"
	hero_points_btn.add_theme_font_size_override("font_size", 9)
	hero_points_btn.focus_mode = Control.FOCUS_NONE
	hero_points_btn.pressed.connect(_on_points_btn_pressed)
	lvl_row.add_child(hero_points_btn)

	# 6 Core Stats Matrix
	var stats_hdr := Label.new()
	stats_hdr.text = "CORE 6-STAT ATTRIBUTES"
	stats_hdr.add_theme_font_size_override("font_size", 10)
	stats_hdr.add_theme_color_override("font_color", Color(0.65, 0.85, 1.0))
	vbox.add_child(stats_hdr)

	var stats_grid := GridContainer.new()
	stats_grid.columns = 2
	stats_grid.add_theme_constant_override("h_separation", 6)
	stats_grid.add_theme_constant_override("v_separation", 4)
	vbox.add_child(stats_grid)

	var stat_defs: Array[Dictionary] = [
		{"k": "STR", "n": "STR (Active)", "c": Color(0.95, 0.4, 0.4)},
		{"k": "CONS", "n": "CONS (Passive)", "c": Color(0.95, 0.65, 0.3)},
		{"k": "DEX", "n": "DEX (Active)", "c": Color(0.4, 0.9, 0.5)},
		{"k": "PERC", "n": "PERC (Passive)", "c": Color(0.4, 0.8, 0.95)},
		{"k": "INT", "n": "INT (Active)", "c": Color(0.75, 0.5, 0.95)},
		{"k": "KNOW", "n": "KNOW (Passive)", "c": Color(0.5, 0.7, 1.0)}
	]

	for sd: Dictionary in stat_defs:
		var stat_box := PanelContainer.new()
		stat_box.mouse_filter = Control.MOUSE_FILTER_STOP
		var b_st := StyleBoxFlat.new()
		b_st.bg_color = Color(0.11, 0.14, 0.20, 0.8)
		b_st.set_corner_radius_all(4)
		b_st.content_margin_left = 6
		b_st.content_margin_right = 6
		b_st.content_margin_top = 3
		b_st.content_margin_bottom = 3
		stat_box.add_theme_stylebox_override("panel", b_st)
		stats_grid.add_child(stat_box)

		var s_hbox := HBoxContainer.new()
		s_hbox.mouse_filter = Control.MOUSE_FILTER_PASS
		stat_box.add_child(s_hbox)

		var s_tag := Label.new()
		s_tag.text = str(sd["n"])
		s_tag.add_theme_font_size_override("font_size", 8)
		var s_col: Color = sd.get("c", Color.WHITE)
		s_tag.add_theme_color_override("font_color", s_col)
		s_tag.mouse_filter = Control.MOUSE_FILTER_PASS
		s_hbox.add_child(s_tag)

		s_hbox.add_spacer(false)

		var key_str: String = str(sd["k"])

		var btn_minus := Button.new()
		btn_minus.text = "-"
		btn_minus.custom_minimum_size = Vector2(16, 16)
		btn_minus.add_theme_font_size_override("font_size", 8)
		btn_minus.focus_mode = Control.FOCUS_NONE
		btn_minus.visible = false
		btn_minus.pressed.connect(func() -> void: _on_stat_minus_pressed(key_str))
		s_hbox.add_child(btn_minus)
		stat_minus_buttons[key_str] = btn_minus

		var s_val := Label.new()
		s_val.text = "14"
		s_val.add_theme_font_size_override("font_size", 10)
		s_val.add_theme_color_override("font_color", Color.WHITE)
		s_val.mouse_filter = Control.MOUSE_FILTER_PASS
		s_hbox.add_child(s_val)

		var btn_plus := Button.new()
		btn_plus.text = "+"
		btn_plus.custom_minimum_size = Vector2(16, 16)
		btn_plus.add_theme_font_size_override("font_size", 8)
		btn_plus.focus_mode = Control.FOCUS_NONE
		btn_plus.visible = false
		btn_plus.pressed.connect(func() -> void: _on_stat_plus_pressed(key_str))
		s_hbox.add_child(btn_plus)
		stat_plus_buttons[key_str] = btn_plus

		stat_display_labels[key_str] = s_val
		stat_box_containers[key_str] = stat_box

	# Stat Allocation Action Controls (Requirement 8)
	stat_allocation_controls_vbox = VBoxContainer.new()
	stat_allocation_controls_vbox.add_theme_constant_override("separation", 3)
	stat_allocation_controls_vbox.visible = false
	vbox.add_child(stat_allocation_controls_vbox)

	alloc_rem_lbl = Label.new()
	alloc_rem_lbl.text = "Allocating Points: 0 remaining"
	alloc_rem_lbl.add_theme_font_size_override("font_size", 8)
	alloc_rem_lbl.add_theme_color_override("font_color", Color(0.4, 1.0, 0.6))
	stat_allocation_controls_vbox.add_child(alloc_rem_lbl)

	var alloc_btns_row := HBoxContainer.new()
	alloc_btns_row.add_theme_constant_override("separation", 4)
	stat_allocation_controls_vbox.add_child(alloc_btns_row)

	var btn_reset_alloc := Button.new()
	btn_reset_alloc.text = "↺ Reset"
	btn_reset_alloc.tooltip_text = "Reset newly allocated points back to pool"
	btn_reset_alloc.add_theme_font_size_override("font_size", 8)
	btn_reset_alloc.focus_mode = Control.FOCUS_NONE
	btn_reset_alloc.pressed.connect(_on_reset_allocation_pressed)
	alloc_btns_row.add_child(btn_reset_alloc)

	var btn_confirm_alloc := Button.new()
	btn_confirm_alloc.text = "✔ Confirm"
	btn_confirm_alloc.tooltip_text = "Commit newly allocated points to hero attributes"
	btn_confirm_alloc.add_theme_font_size_override("font_size", 8)
	btn_confirm_alloc.focus_mode = Control.FOCUS_NONE
	btn_confirm_alloc.pressed.connect(_on_confirm_allocation_pressed)
	alloc_btns_row.add_child(btn_confirm_alloc)

	var btn_cancel_alloc := Button.new()
	btn_cancel_alloc.text = "✖ Cancel"
	btn_cancel_alloc.tooltip_text = "Cancel allocation session without saving"
	btn_cancel_alloc.add_theme_font_size_override("font_size", 8)
	btn_cancel_alloc.focus_mode = Control.FOCUS_NONE
	btn_cancel_alloc.pressed.connect(_on_cancel_allocation_pressed)
	alloc_btns_row.add_child(btn_cancel_alloc)

	# Derived Combat Specs
	var derived_hdr := Label.new()
	derived_hdr.text = "DERIVED COMBAT METRICS (Hover for formula)"
	derived_hdr.add_theme_font_size_override("font_size", 10)
	derived_hdr.add_theme_color_override("font_color", Color(0.65, 0.85, 1.0))
	vbox.add_child(derived_hdr)

	derived_melee_lbl = _create_key_val_row(vbox, "Melee Power", "25", Color(0.95, 0.45, 0.4), "melee")
	derived_ranged_lbl = _create_key_val_row(vbox, "Ranged Power", "18", Color(0.4, 0.9, 0.5), "ranged")
	derived_speed_lbl = _create_key_val_row(vbox, "Initiative Speed", "15", Color(0.35, 0.85, 0.95), "speed")
	derived_hp_lbl = _create_key_val_row(vbox, "Maximum Health", "100 HP", Color(0.95, 0.4, 0.4), "hp")
	derived_mit_lbl = _create_key_val_row(vbox, "Flat Mitigation", "+0 Mit", Color(0.7, 0.85, 1.0), "mit")
	derived_burden_lbl = _create_key_val_row(vbox, "Armor Burden", "0 / 14", Color(0.9, 0.8, 0.35), "burden")
	derived_crit_lbl = _create_key_val_row(vbox, "Crit Hit Rate", "5.0%", Color(0.95, 0.75, 0.3), "crit")
	derived_resist_lbl = _create_key_val_row(vbox, "Debuff Resist", "0.0%", Color(0.7, 0.5, 0.95), "resist")

	var sep_loadout := HSeparator.new()
	vbox.add_child(sep_loadout)

	var gear_hdr := Label.new()
	gear_hdr.text = "EQUIPPED LOADOUT & GEAR SLOTS"
	gear_hdr.add_theme_font_size_override("font_size", 10)
	gear_hdr.add_theme_color_override("font_color", Color(0.65, 0.85, 1.0))
	vbox.add_child(gear_hdr)

	loadout_cards_vbox = VBoxContainer.new()
	loadout_cards_vbox.add_theme_constant_override("separation", 3)
	vbox.add_child(loadout_cards_vbox)

	var sep_pouch := HSeparator.new()
	vbox.add_child(sep_pouch)

	var pouch_hdr := Label.new()
	pouch_hdr.text = "🧪 BATTLE POUCH CONSUMABLES"
	pouch_hdr.add_theme_font_size_override("font_size", 9)
	pouch_hdr.add_theme_color_override("font_color", Color(0.5, 0.8, 0.6))
	vbox.add_child(pouch_hdr)

	battle_pouch_vbox = VBoxContainer.new()
	battle_pouch_vbox.add_theme_constant_override("separation", 2)
	vbox.add_child(battle_pouch_vbox)

func _create_key_val_row(parent: Control, tag: String, default_v: String, col: Color, row_id: String = "") -> Label:
	var row := HBoxContainer.new()
	row.mouse_filter = Control.MOUSE_FILTER_STOP
	parent.add_child(row)

	var t_lbl := Label.new()
	t_lbl.text = "• " + tag + ":"
	t_lbl.add_theme_font_size_override("font_size", 9)
	t_lbl.add_theme_color_override("font_color", col)
	t_lbl.mouse_filter = Control.MOUSE_FILTER_PASS
	row.add_child(t_lbl)

	row.add_spacer(false)

	var v_lbl := Label.new()
	v_lbl.text = default_v
	v_lbl.add_theme_font_size_override("font_size", 9)
	v_lbl.add_theme_color_override("font_color", Color.WHITE)
	v_lbl.mouse_filter = Control.MOUSE_FILTER_PASS
	row.add_child(v_lbl)

	if not row_id.is_empty():
		derived_rows[row_id] = row

	return v_lbl

# ------------------------------------------------------------------------------
# Center Dock: Visual Stage & Skill Action Console
# ------------------------------------------------------------------------------

func _build_center_stage_dock(parent: Control) -> void:
	var panel := PanelContainer.new()
	panel.custom_minimum_size = Vector2(460, 0)
	var p_style := StyleBoxFlat.new()
	p_style.bg_color = Color(0.07, 0.09, 0.14, 0.95)
	p_style.border_color = Color(0.22, 0.32, 0.46, 0.8)
	p_style.set_border_width_all(1)
	p_style.set_corner_radius_all(6)
	p_style.content_margin_left = 10
	p_style.content_margin_right = 10
	p_style.content_margin_top = 8
	p_style.content_margin_bottom = 8
	panel.add_theme_stylebox_override("panel", p_style)
	parent.add_child(panel)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 8)
	panel.add_child(vbox)

	var stage_hdr := Label.new()
	stage_hdr.text = "🎯 DOJO COMBAT ARENA & TARGET MAT"
	stage_hdr.add_theme_font_size_override("font_size", 10)
	stage_hdr.add_theme_color_override("font_color", Color(0.65, 0.85, 1.0))
	vbox.add_child(stage_hdr)

	# 2D Training Stage Mat
	var stage_mat := Panel.new()
	stage_mat.custom_minimum_size = Vector2(STAGE_WIDTH, STAGE_HEIGHT)
	var mat_style := StyleBoxFlat.new()
	mat_style.bg_color = Color(0.12, 0.16, 0.22, 0.95)
	mat_style.border_color = Color(0.25, 0.38, 0.55, 0.8)
	mat_style.set_border_width_all(2)
	mat_style.set_corner_radius_all(6)
	stage_mat.add_theme_stylebox_override("panel", mat_style)
	vbox.add_child(stage_mat)

	# Hero Token Node
	hero_token_ctrl = Control.new()
	hero_token_ctrl.position = HERO_POS
	stage_mat.add_child(hero_token_ctrl)

	var hero_icon := TextureRect.new()
	hero_icon.custom_minimum_size = Vector2(56, 56)
	hero_icon.position = Vector2(-28, -28)
	hero_icon.texture = CombatScenario.load_svg_texture("res://assets/party/tokens/token_warrior.svg")
	hero_icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	hero_icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	hero_token_ctrl.add_child(hero_icon)

	var hero_name_tag := Label.new()
	hero_name_tag.text = "Hero"
	hero_name_tag.position = Vector2(-30, 30)
	hero_name_tag.custom_minimum_size = Vector2(60, 14)
	hero_name_tag.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	hero_name_tag.add_theme_font_size_override("font_size", 8)
	hero_name_tag.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
	hero_token_ctrl.add_child(hero_name_tag)

	# Dummy Token Node
	dummy_token_ctrl = Control.new()
	dummy_token_ctrl.position = DUMMY_POS
	stage_mat.add_child(dummy_token_ctrl)

	var dummy_icon := TextureRect.new()
	dummy_icon.custom_minimum_size = Vector2(56, 56)
	dummy_icon.position = Vector2(-28, -28)
	dummy_icon.texture = CombatScenario.load_svg_texture("res://assets/enemy/tokens/token_dummy.svg")
	dummy_icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	dummy_icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	dummy_token_ctrl.add_child(dummy_icon)

	var dummy_name_tag := Label.new()
	dummy_name_tag.text = "Test Dummy"
	dummy_name_tag.position = Vector2(-35, 30)
	dummy_name_tag.custom_minimum_size = Vector2(70, 14)
	dummy_name_tag.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	dummy_name_tag.add_theme_font_size_override("font_size", 8)
	dummy_name_tag.add_theme_color_override("font_color", Color(0.95, 0.85, 0.4))
	dummy_token_ctrl.add_child(dummy_name_tag)

	# Dummy Health & Status Card
	var dummy_card := PanelContainer.new()
	var d_style := StyleBoxFlat.new()
	d_style.bg_color = Color(0.10, 0.13, 0.18, 0.9)
	d_style.set_corner_radius_all(4)
	d_style.content_margin_left = 8
	d_style.content_margin_right = 8
	d_style.content_margin_top = 4
	d_style.content_margin_bottom = 4
	dummy_card.add_theme_stylebox_override("panel", d_style)
	vbox.add_child(dummy_card)

	var d_vbox := VBoxContainer.new()
	d_vbox.add_theme_constant_override("separation", 2)
	dummy_card.add_child(d_vbox)

	var d_row := HBoxContainer.new()
	d_vbox.add_child(d_row)

	var d_lbl := Label.new()
	d_lbl.text = "🎯 Training Dummy Vitals:"
	d_lbl.add_theme_font_size_override("font_size", 9)
	d_lbl.add_theme_color_override("font_color", Color(0.9, 0.8, 0.4))
	d_row.add_child(d_lbl)

	d_row.add_spacer(false)

	dummy_hp_lbl = Label.new()
	dummy_hp_lbl.text = "500 / 500 HP"
	dummy_hp_lbl.add_theme_font_size_override("font_size", 9)
	dummy_hp_lbl.add_theme_color_override("font_color", Color.WHITE)
	d_row.add_child(dummy_hp_lbl)

	dummy_hp_bar = ProgressBar.new()
	dummy_hp_bar.custom_minimum_size = Vector2(0, 7)
	dummy_hp_bar.max_value = 500
	dummy_hp_bar.value = 500
	dummy_hp_bar.show_percentage = false
	var bar_bg := StyleBoxFlat.new()
	bar_bg.bg_color = Color(0.18, 0.08, 0.08, 0.9)
	bar_bg.set_corner_radius_all(3)
	var bar_fill := StyleBoxFlat.new()
	bar_fill.bg_color = Color(0.25, 0.75, 0.35, 1.0)
	bar_fill.set_corner_radius_all(3)
	dummy_hp_bar.add_theme_stylebox_override("background", bar_bg)
	dummy_hp_bar.add_theme_stylebox_override("fill", bar_fill)
	d_vbox.add_child(dummy_hp_bar)

	dummy_stance_badge = Label.new()
	dummy_stance_badge.text = "Stance: Neutral (0% Damage Mitigation)"
	dummy_stance_badge.add_theme_font_size_override("font_size", 8)
	dummy_stance_badge.add_theme_color_override("font_color", Color(0.5, 0.8, 0.6))
	d_vbox.add_child(dummy_stance_badge)

	# Action Test Console
	var console_hdr := Label.new()
	console_hdr.text = "BENCHMARK ACTION TESTING CONSOLE"
	console_hdr.add_theme_font_size_override("font_size", 10)
	console_hdr.add_theme_color_override("font_color", Color(0.65, 0.85, 1.0))
	vbox.add_child(console_hdr)

	var acts_grid := GridContainer.new()
	acts_grid.columns = 3
	acts_grid.add_theme_constant_override("h_separation", 6)
	acts_grid.add_theme_constant_override("v_separation", 4)
	vbox.add_child(acts_grid)

	btn_melee_strike = _add_action_btn(acts_grid, "⚔ Main Strike", "Execute main hand strike against Dummy", _on_test_melee_strike)
	btn_offhand_strike = _add_action_btn(acts_grid, "🗡 Off-Hand Strike", "Execute off-hand weapon strike against Dummy", _on_test_offhand_strike)
	btn_ranged_volley = _add_action_btn(acts_grid, "🏹 Ranged Volley", "Execute ranged attack against Dummy", _on_test_ranged_volley)
	btn_switch_active_weapon = _add_action_btn(acts_grid, "🔄 Switch Weapon", "Switch active weapon / toggle 1H off-hand or alternate weapon", _on_test_switch_active_weapon)
	_add_action_btn(acts_grid, "✦ Focus Buff", "Gain +50% ATK Power boost for 2 rounds", _on_test_focus_buff)
	_add_action_btn(acts_grid, "🛡 Shield Defend", "Enter Defensive Stance (50% Mitigation)", _on_test_defend)
	_add_action_btn(acts_grid, "🔄 Stance Shift", "Shift hero stance between Offensive/Neutral", _on_test_stance_shift)

	force_crit_check = CheckBox.new()
	force_crit_check.text = "Force Crit"
	force_crit_check.tooltip_text = "Force critical hit (2.5x damage) for formula verification"
	force_crit_check.add_theme_font_size_override("font_size", 8)
	force_crit_check.focus_mode = Control.FOCUS_NONE
	acts_grid.add_child(force_crit_check)

	# Battle Pouch Quick Selector (Requirement 5)
	var pouch_act_card := PanelContainer.new()
	var pa_st := StyleBoxFlat.new()
	pa_st.bg_color = Color(0.10, 0.13, 0.18, 0.8)
	pa_st.set_corner_radius_all(4)
	pa_st.content_margin_left = 6
	pa_st.content_margin_right = 6
	pa_st.content_margin_top = 4
	pa_st.content_margin_bottom = 4
	pouch_act_card.add_theme_stylebox_override("panel", pa_st)
	vbox.add_child(pouch_act_card)

	var pa_hbox := HBoxContainer.new()
	pa_hbox.add_theme_constant_override("separation", 6)
	pouch_act_card.add_child(pa_hbox)

	var pa_lbl := Label.new()
	pa_lbl.text = "🧪 Battle Pouch Action:"
	pa_lbl.add_theme_font_size_override("font_size", 9)
	pa_lbl.add_theme_color_override("font_color", Color(0.4, 0.9, 0.6))
	pa_hbox.add_child(pa_lbl)

	pouch_select_option = OptionButton.new()
	pouch_select_option.add_theme_font_size_override("font_size", 9)
	pouch_select_option.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pouch_select_option.focus_mode = Control.FOCUS_NONE
	pa_hbox.add_child(pouch_select_option)

	btn_use_pouch_item = Button.new()
	btn_use_pouch_item.text = "Use Item"
	btn_use_pouch_item.add_theme_font_size_override("font_size", 9)
	btn_use_pouch_item.focus_mode = Control.FOCUS_NONE
	btn_use_pouch_item.pressed.connect(_on_test_pouch_item)
	pa_hbox.add_child(btn_use_pouch_item)

func _add_action_btn(parent: Control, label_text: String, tooltip: String, callback: Callable) -> Button:
	var btn := Button.new()
	btn.text = label_text
	btn.tooltip_text = tooltip
	btn.add_theme_font_size_override("font_size", 9)
	btn.focus_mode = Control.FOCUS_NONE
	btn.pressed.connect(callback)
	parent.add_child(btn)
	return btn

# ------------------------------------------------------------------------------
# Right Dock: Live Formula Inspection Log & Party Stash Quick Browser
# ------------------------------------------------------------------------------

func _build_right_inspector_dock(parent: Control) -> void:
	var panel := PanelContainer.new()
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var p_style := StyleBoxFlat.new()
	p_style.bg_color = Color(0.08, 0.11, 0.16, 0.95)
	p_style.border_color = Color(0.22, 0.32, 0.46, 0.8)
	p_style.set_border_width_all(1)
	p_style.set_corner_radius_all(6)
	p_style.content_margin_left = 10
	p_style.content_margin_right = 10
	p_style.content_margin_top = 8
	p_style.content_margin_bottom = 8
	panel.add_theme_stylebox_override("panel", p_style)
	parent.add_child(panel)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 6)
	panel.add_child(vbox)

	var log_hdr := Label.new()
	log_hdr.text = "📊 LIVE DAMAGE FORMULA & CALCULATION INSPECTOR"
	log_hdr.add_theme_font_size_override("font_size", 10)
	log_hdr.add_theme_color_override("font_color", Color(0.4, 0.85, 1.0))
	vbox.add_child(log_hdr)

	# Rich text calculation feed
	formula_log_rich = RichTextLabel.new()
	formula_log_rich.size_flags_vertical = Control.SIZE_EXPAND_FILL
	formula_log_rich.scroll_following = true
	formula_log_rich.bbcode_enabled = true
	formula_log_rich.custom_minimum_size = Vector2(0, 360)
	var log_box_style := StyleBoxFlat.new()
	log_box_style.bg_color = Color(0.05, 0.07, 0.10, 0.95)
	log_box_style.border_color = Color(0.18, 0.25, 0.35, 0.8)
	log_box_style.set_border_width_all(1)
	log_box_style.set_corner_radius_all(4)
	log_box_style.content_margin_left = 8
	log_box_style.content_margin_right = 8
	log_box_style.content_margin_top = 6
	log_box_style.content_margin_bottom = 6
	formula_log_rich.add_theme_stylebox_override("normal", log_box_style)
	vbox.add_child(formula_log_rich)

	_append_formula_log("[color=#38bdf8][b]>>> Dojo initialized. Click actions above to verify damage formulas.[/b][/color]")
	_append_formula_log("[color=#94a3b8]• Standard Melee Formula: Base Weapon Power + STR scaling + Threshold Bonus - Deficit Penalty[/color]")
	_append_formula_log("[color=#94a3b8]• Mitigation Formula: Damage = Power * (1.0 - Target Defense Mitigation)[/color]\n")

# ------------------------------------------------------------------------------
# 3. View Refresh & Data Sync
# ------------------------------------------------------------------------------

func _refresh_all_views() -> void:
	if active_hero == null:
		return

	active_hero.sync_from_stats_and_gear()

	if hero_title_lbl == null:
		return

	# 1. Update Tabs
	_rebuild_hero_tabs()

	# 2. Update Header & Points Button (Requirement 8)
	var arch_name: String = active_hero.stats.archetype.display_name if active_hero.stats and active_hero.stats.archetype else "Hero"
	hero_title_lbl.text = "%s (Archetype: %s)" % [active_hero.name, arch_name]
	hero_lvl_lbl.text = "Level: %d / 10" % (active_hero.stats.level if active_hero.stats else 1)
	var free_pts: int = active_hero.stats.unspent_attribute_points if active_hero.stats else 0
	if is_allocating_stats:
		hero_points_btn.text = "Allocating Stats (%d unspent)" % pending_free_points
		hero_points_btn.add_theme_color_override("font_color", Color(0.95, 0.85, 0.4))
	else:
		hero_points_btn.text = "Unspent Points: %d (Click to Allocate)" % free_pts
		hero_points_btn.add_theme_color_override("font_color", Color(0.4, 1.0, 0.5) if free_pts > 0 else Color(0.6, 0.7, 0.8))

	# 3. Update 6 Stats Matrix & Tooltips (Items 1, 2 & 8)
	if active_hero.stats != null:
		var st := active_hero.stats
		var main_w: Equipment = active_hero.inventory.get_main_weapon() if active_hero.inventory else null
		var ctx: Dictionary = {"weapon": main_w.weapon_type} if main_w != null else {}

		for k: String in ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"]:
			var eff: int = st.get_effective_stat(k, ctx)
			var base_val: int = st.get_base_stat(k)
			var diff: int = eff - base_val
			var txt := "%d" % eff
			if diff > 0:
				txt += " (+%d)" % diff
			if is_allocating_stats:
				var pend: int = pending_allocated_stats.get(k, 0)
				if pend > 0:
					txt += " (+%d)" % pend

			if stat_display_labels.has(k):
				var lbl: Variant = stat_display_labels[k]
				if lbl is Label:
					lbl.text = txt

			if stat_plus_buttons.has(k):
				var btn_p: Variant = stat_plus_buttons[k]
				if btn_p is Button:
					btn_p.visible = is_allocating_stats
					btn_p.disabled = (pending_free_points <= 0)

			if stat_minus_buttons.has(k):
				var btn_m: Variant = stat_minus_buttons[k]
				if btn_m is Button:
					btn_m.visible = is_allocating_stats
					btn_m.disabled = (pending_allocated_stats.get(k, 0) <= 0)

			if stat_box_containers.has(k):
				var box: Variant = stat_box_containers[k]
				if box is PanelContainer:
					box.tooltip_text = _build_stat_breakdown_tooltip(k)

		stat_allocation_controls_vbox.visible = is_allocating_stats
		if is_allocating_stats:
			alloc_rem_lbl.text = "Allocating Points: %d unspent remaining" % pending_free_points

		# 4. Update Derived Combat Metrics (Items 3, 5, 9, 10 & 11)
		var burden: int = active_hero.inventory.get_total_armor_burden() if active_hero.inventory else 0
		var carry_max: int = st.get_carry_weight_capacity(ctx)
		var pwr_melee: int = active_hero.get_effective_power(false)
		var pwr_ranged: int = active_hero.get_effective_power(true)
		var arm_mit: int = active_hero.inventory.get_total_armor_mitigation() if active_hero.inventory else 0
		var shd_mit: int = active_hero.inventory.get_total_shield_mitigation() if active_hero.inventory else 0
		var total_gear_mit: int = active_hero.inventory.get_total_flat_mitigation() if active_hero.inventory else 0
		var str_mit: int = int(roundf(float(st.get_effective_stat("STR", ctx)) * 0.25))
		var total_mit_rating: int = total_gear_mit + str_mit

		var crit_rate: float = st.get_crit_rate(ctx)
		var crit_mult: float = st.get_crit_damage_multiplier(ctx)
		var debuff_resist: float = st.get_debuff_resistance(ctx)

		if main_w != null and main_w.is_ranged_weapon():
			derived_melee_lbl.text = "%d ATK Power (Ranged Eqpd)" % pwr_melee
		else:
			derived_melee_lbl.text = "%d ATK Power" % pwr_melee

		if main_w != null and main_w.is_melee_weapon():
			derived_ranged_lbl.text = "%d ATK Power (Melee Eqpd)" % pwr_ranged
		else:
			derived_ranged_lbl.text = "%d ATK Power" % pwr_ranged

		derived_speed_lbl.text = "%d Speed (Burden -%d)" % [st.get_initiative_speed(burden, ctx), burden]
		derived_hp_lbl.text = "%d HP" % st.get_max_hp(ctx)
		if derived_mit_lbl != null:
			derived_mit_lbl.text = "+%d Flat (Gear: %d, STR: +%d)" % [total_mit_rating, total_gear_mit, str_mit]
		derived_burden_lbl.text = "%d / %d Burden" % [burden, carry_max]
		derived_crit_lbl.text = "%.1f%% (%.1fx DMG)" % [crit_rate, crit_mult]
		derived_resist_lbl.text = "%.1f%% Resist" % debuff_resist

		_update_derived_tooltips(st, ctx, main_w, burden, carry_max, arm_mit, shd_mit, total_gear_mit, str_mit, total_mit_rating)

		# 5. Active Loadout & Quick Pouches (Items 3, 4, 6, 7, 8, 9, 10)
		_render_dojo_loadout_cards(st)

		# 6. Action Button Availabilities based on weapon type (Item 3 & 7)
		_update_action_button_states(main_w)

		# 7. Update Battle Pouch Selector Dropdown (Item 5)
		_refresh_pouch_dropdown()

	# 7. Update Dummy Vitals
	if test_dummy != null:
		dummy_hp_bar.value = test_dummy.hp
		dummy_hp_bar.max_value = test_dummy.max_hp
		dummy_hp_lbl.text = "%d / %d HP" % [test_dummy.hp, test_dummy.max_hp]
		var mit_pct: int = int(roundf(test_dummy.defense_mitigation_ratio * 100.0))
		if mit_pct > 0:
			dummy_stance_badge.text = "Stance: 🛡 Defending (%d%% Damage Mitigation)" % mit_pct
			dummy_stance_badge.add_theme_color_override("font_color", Color(0.4, 0.8, 1.0))
		else:
			dummy_stance_badge.text = "Stance: Neutral (0% Damage Mitigation)"
			dummy_stance_badge.add_theme_color_override("font_color", Color(0.5, 0.8, 0.6))

# Compact micro-card tooltips (Requirement 7)
func _build_stat_breakdown_tooltip(k: String) -> String:
	if active_hero == null or active_hero.stats == null:
		return ""
	var st := active_hero.stats
	var main_w: Equipment = active_hero.inventory.get_main_weapon() if active_hero.inventory else null
	var ctx: Dictionary = {"weapon": main_w.weapon_type} if main_w != null else {}

	var stat_full_name: String = _get_stat_full_name(k)
	var base_val: int = st.get_base_stat(k)
	var eff_val: int = st.get_effective_stat(k, ctx)
	var diff: int = eff_val - base_val

	var lines: Array[String] = []
	lines.append("%s [%s]" % [stat_full_name, "Combat" if (k in ["STR", "DEX", "INT"]) else "Passive"])
	lines.append(_get_stat_role_desc(k))
	lines.append("Base: %d (Lv %d %s) | Effective: %d" % [base_val, st.level, st.archetype.display_name if st.archetype else "Hero", eff_val])

	if diff != 0:
		var bonus_items: Array[String] = []
		if st.mastery != null and ctx.has("weapon"):
			var m_bonus: int = st.mastery.get_conditional_stat_bonus(k, ctx)
			if m_bonus > 0:
				bonus_items.append("• Mastery (%s): +%d %s" % [str(ctx["weapon"]).capitalize(), m_bonus, k])
		if active_hero.inventory != null:
			for slot_k: Variant in active_hero.inventory.equipped_gear:
				var raw_eq: Variant = active_hero.inventory.equipped_gear[slot_k]
				if raw_eq is Equipment:
					var eq: Equipment = raw_eq
					if eq.stat_bonuses.has(k):
						var val_int: int = eq.stat_bonuses[k] if eq.stat_bonuses[k] is int else 0
						if val_int != 0:
							bonus_items.append("• %s: %+d %s" % [eq.name, val_int, k])
		for mod: StatModifier in st.modifiers:
			if mod.source_id != "gear" and mod.stat_name.to_upper() == k and mod.matches_context(ctx):
				bonus_items.append("• Buff (%s): %+d" % [mod.source_id, mod.flat_bonus])

		if not bonus_items.is_empty():
			lines.append("Bonuses (%+d):\n%s" % [diff, "\n".join(bonus_items)])

	return "\n".join(lines)

func _get_stat_full_name(k: String) -> String:
	match k:
		"STR": return "STR (Strength)"
		"DEX": return "DEX (Dexterity)"
		"INT": return "INT (Intelligence)"
		"CONS": return "CONS (Constitution)"
		"PERC": return "PERC (Perception)"
		"KNOW": return "KNOW (Knowledge)"
	return k

func _get_stat_role_desc(k: String) -> String:
	match k:
		"STR": return "Melee scaling (x1.5) & Burden limit."
		"DEX": return "Ranged scaling (x1.5) & Speed."
		"INT": return "Skill Power (x1.5) & Buff potency."
		"CONS": return "Max Health (x8 HP) & Endurance."
		"PERC": return "Intent Unmasking & Sight range."
		"KNOW": return "Lore checks, danger mitigation."
	return ""

# Compact derived combat tooltips (Requirement 7)
func _update_derived_tooltips(
	st: CharacterStats,
	ctx: Dictionary,
	main_w: Equipment,
	burden: int,
	carry_max: int,
	arm_mit: int,
	shd_mit: int,
	total_gear_mit: int,
	str_mit: int,
	total_mit_rating: int
) -> void:
	if derived_rows.has("melee"):
		var lines: Array[String] = ["[MELEE POWER FORMULA]"]
		lines.append("Power = Base Wep + Tier + (STR * 1.5) - Deficit")
		var w_part := "0"
		if main_w != null:
			if main_w.is_melee_weapon():
				var t_boost: int = main_w.get_unlocked_threshold_power_boost(st, st.mastery)
				w_part = "%d (Base %d%s)" % [main_w.base_power + t_boost, main_w.base_power, " + Tier %d" % t_boost if t_boost > 0 else ""]
			else:
				w_part = "0 (Ranged Eqpd)"
		var eff_str: int = st.get_effective_stat("STR", ctx)
		lines.append("Calc: %s + (%d STR * 1.5 = %d) = %d ATK" % [w_part, eff_str, int(roundf(eff_str * 1.5)), active_hero.get_effective_power(false)])
		var row_m: Variant = derived_rows["melee"]
		if row_m is Control: row_m.tooltip_text = "\n".join(lines)

	if derived_rows.has("ranged"):
		var lines: Array[String] = ["[RANGED POWER FORMULA]"]
		lines.append("Power = Base Wep + Tier + (DEX * 1.5) - Deficit")
		var w_part := "0"
		if main_w != null:
			if main_w.is_ranged_weapon():
				var t_boost: int = main_w.get_unlocked_threshold_power_boost(st, st.mastery)
				w_part = "%d (Base %d%s)" % [main_w.base_power + t_boost, main_w.base_power, " + Tier %d" % t_boost if t_boost > 0 else ""]
			else:
				w_part = "0 (Melee Eqpd)"
		var eff_dex: int = st.get_effective_stat("DEX", ctx)
		lines.append("Calc: %s + (%d DEX * 1.5 = %d) = %d ATK" % [w_part, eff_dex, int(roundf(eff_dex * 1.5)), active_hero.get_effective_power(true)])
		var row_r: Variant = derived_rows["ranged"]
		if row_r is Control: row_r.tooltip_text = "\n".join(lines)

	if derived_rows.has("speed"):
		var eff_dex_spd: int = st.get_effective_stat("DEX", ctx)
		var spd_tip: String = "[INITIATIVE SPEED]\nFormula: 8 (Base) + DEX - Burden\nCalc: 8 + %d (DEX) - %d (Burden) = %d Speed" % [eff_dex_spd, burden, st.get_initiative_speed(burden, ctx)]
		var row_s: Variant = derived_rows["speed"]
		if row_s is Control: row_s.tooltip_text = spd_tip

	if derived_rows.has("hp"):
		var eff_cons: int = st.get_effective_stat("CONS", ctx)
		var eff_str_hp: int = st.get_effective_stat("STR", ctx)
		var hp_tip: String = "[MAX HEALTH]\nFormula: 60 (Base) + (CONS * 8) + (STR * 2)\nCalc: 60 + %d (CONS*8) + %d (STR*2) = %d HP" % [eff_cons * 8, eff_str_hp * 2, st.get_max_hp(ctx)]
		var row_h: Variant = derived_rows["hp"]
		if row_h is Control: row_h.tooltip_text = hp_tip

	if derived_rows.has("mit"):
		var mit_tip: String = "[FLAT MITIGATION]\nFormula: Gear Mit + (STR * 0.25)\nCalc: %d (Armor %d, Shield %d) + %d (STR) = +%d Flat" % [total_gear_mit, arm_mit, shd_mit, str_mit, total_mit_rating]
		var row_mit: Variant = derived_rows["mit"]
		if row_mit is Control: row_mit.tooltip_text = mit_tip

	if derived_rows.has("burden"):
		var status_str := "Within limit (Rem: %d)" % (carry_max - burden) if burden <= carry_max else "OVERENCUMBERED!"
		var bur_tip: String = "[BURDEN & CARRY CAPACITY]\nCapacity: 30 + (STR * 3) + (CONS * 1) = %d\nEquipped Burden: %d / %d (%s)" % [carry_max, burden, carry_max, status_str]
		var row_b: Variant = derived_rows["burden"]
		if row_b is Control: row_b.tooltip_text = bur_tip

	if derived_rows.has("crit"):
		var eff_dex_c: int = st.get_effective_stat("DEX", ctx)
		var c_rate: float = st.get_crit_rate(ctx)
		var c_mult: float = st.get_crit_damage_multiplier(ctx)
		var c_tip: String = "[CRITICAL HIT RATE & MULTIPLIER]\nFormula: 5.0% (Base) + (DEX * 1.5%)\nCalc: 5.0% + (%d DEX * 1.5% = +%.1f%%) = %.1f%% Crit Rate\nCritical Damage Multiplier: %.1fx" % [eff_dex_c, float(eff_dex_c) * 1.5, c_rate, c_mult]
		var row_c: Variant = derived_rows["crit"]
		if row_c is Control: row_c.tooltip_text = c_tip

	if derived_rows.has("resist"):
		var eff_int_r: int = st.get_effective_stat("INT", ctx)
		var r_rate: float = st.get_debuff_resistance(ctx)
		var r_tip: String = "[DEBUFF & STATUS RESISTANCE]\nFormula: INT * 2.5%\nCalc: %d INT * 2.5% = %.1f%% Debuff Resistance\nEffect: Reduces incoming debuff duration, status infliction rate, and penalty severity." % [eff_int_r, r_rate]
		var row_res: Variant = derived_rows["resist"]
		if row_res is Control: row_res.tooltip_text = r_tip

# Card-based equipped loadout & consumable pouches (Requirements 3 & 4)
func _render_dojo_loadout_cards(st: CharacterStats) -> void:
	if loadout_cards_vbox == null or active_hero == null or active_hero.inventory == null:
		return
	for c in loadout_cards_vbox.get_children():
		c.queue_free()

	var inv: Inventory = active_hero.inventory
	var slot_defs: Array[Dictionary] = [
		{"slot": Equipment.EquipSlot.MAIN_HAND, "name": "Main Hand", "icon": "⚔"},
		{"slot": Equipment.EquipSlot.OFF_HAND, "name": "Off Hand", "icon": "🛡"},
		{"slot": Equipment.EquipSlot.BODY, "name": "Body Armor", "icon": "🥋"},
		{"slot": Equipment.EquipSlot.ACCESSORY_1, "name": "Accessory 1", "icon": "💍"},
		{"slot": Equipment.EquipSlot.ACCESSORY_2, "name": "Accessory 2", "icon": "💍"}
	]

	for def: Dictionary in slot_defs:
		var slot_raw: Variant = def.get("slot", Equipment.EquipSlot.MAIN_HAND)
		var slot_val: Equipment.EquipSlot = slot_raw if slot_raw is Equipment.EquipSlot else Equipment.EquipSlot.MAIN_HAND
		var s_name: String = str(def.get("name", ""))
		var s_icon: String = str(def.get("icon", ""))
		var eq: Equipment = inv.get_equipped(slot_val)

		var card := _create_dojo_slot_card(s_name, s_icon, eq, slot_val, st)
		loadout_cards_vbox.add_child(card)

	_render_dojo_battle_pouches()

func _create_dojo_slot_card(slot_name: String, icon: String, eq: Equipment, slot_val: Equipment.EquipSlot, st: CharacterStats) -> PanelContainer:
	var card := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.10, 0.13, 0.19, 0.85)
	style.border_color = Color(0.20, 0.28, 0.40, 0.7)
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.content_margin_left = 6
	style.content_margin_right = 6
	style.content_margin_top = 3
	style.content_margin_bottom = 3
	card.add_theme_stylebox_override("panel", style)

	var c_vbox := VBoxContainer.new()
	c_vbox.add_theme_constant_override("separation", 2)
	card.add_child(c_vbox)

	# Row 1: Slot title, and stats summary
	var r1 := HBoxContainer.new()
	c_vbox.add_child(r1)

	var title_lbl := Label.new()
	title_lbl.text = "%s %s" % [icon, slot_name]
	title_lbl.add_theme_font_size_override("font_size", 8)
	title_lbl.add_theme_color_override("font_color", Color(0.65, 0.8, 1.0))
	r1.add_child(title_lbl)

	r1.add_spacer(false)

	if eq != null:
		var stat_desc_parts: Array[String] = []
		if eq.base_power > 0:
			var bonus_pwr: int = eq.get_unlocked_threshold_power_boost(st, st.mastery) if st else 0
			if bonus_pwr > 0:
				stat_desc_parts.append("Pwr: %d (+%d Tier)" % [eq.base_power + bonus_pwr, bonus_pwr])
			else:
				var pen := eq.get_wield_penalty(st, st.mastery) if st else {}
				if pen.get("is_penalized", false):
					var ratio_raw: Variant = pen.get("power_penalty_ratio", 0.0)
					var ratio: float = ratio_raw if ratio_raw is float else 0.0
					var pen_pwr: int = maxi(1, int(roundf(float(eq.base_power) * (1.0 - ratio))))
					stat_desc_parts.append("Pwr: %d (-%d%% Pen)" % [pen_pwr, int(roundf(ratio * 100.0))])
				else:
					stat_desc_parts.append("Pwr: %d" % eq.base_power)
		if eq.mitigation > 0:
			stat_desc_parts.append("Mit: +%d" % eq.mitigation)
		if eq.burden > 0:
			stat_desc_parts.append("Brd: %d" % eq.burden)
		var st_bonus := eq.get_stat_bonuses_display()
		if not st_bonus.is_empty():
			stat_desc_parts.append(st_bonus)

		var stats_lbl := Label.new()
		stats_lbl.text = " • ".join(stat_desc_parts)
		stats_lbl.add_theme_font_size_override("font_size", 8)
		stats_lbl.add_theme_color_override("font_color", Color(0.8, 0.9, 0.9))
		r1.add_child(stats_lbl)

	# Row 2: Item Name & Requirement [Req STAT lvl0, lvl1] / Status Badge (Requirement 4)
	var r2 := HBoxContainer.new()
	c_vbox.add_child(r2)

	if eq == null:
		var empty_lbl := Label.new()
		empty_lbl.text = "— Empty Slot —"
		empty_lbl.add_theme_font_size_override("font_size", 8)
		empty_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		r2.add_child(empty_lbl)
	else:
		var name_lbl := Label.new()
		name_lbl.text = eq.name
		name_lbl.add_theme_font_size_override("font_size", 9)
		name_lbl.add_theme_color_override("font_color", _get_rarity_color(eq.rarity))
		r2.add_child(name_lbl)

		r2.add_spacer(false)

		var summary: Dictionary = eq.get_requirement_status_summary(st, st.mastery) if st else {}
		var req_str: String = str(summary.get("req_display", ""))
		var status_tag: String = str(summary.get("status_tag", ""))
		var tag_col: Color = summary.get("tag_color", Color.WHITE)

		var badge_lbl := Label.new()
		badge_lbl.text = "%s  %s" % [req_str, status_tag]
		badge_lbl.add_theme_font_size_override("font_size", 8)
		badge_lbl.add_theme_color_override("font_color", tag_col)
		r2.add_child(badge_lbl)

	# Row 3: Slot Action Buttons (Equip / Swap & Unequip) (Requirement 4)
	var r3 := HBoxContainer.new()
	r3.add_theme_constant_override("separation", 4)
	c_vbox.add_child(r3)

	var btn_eq := Button.new()
	btn_eq.text = "Swap" if eq != null else "Equip"
	btn_eq.add_theme_font_size_override("font_size", 8)
	btn_eq.focus_mode = Control.FOCUS_NONE
	btn_eq.pressed.connect(func() -> void: _open_base_equip_picker(slot_val))
	r3.add_child(btn_eq)

	if eq != null:
		var btn_uneq := Button.new()
		btn_uneq.text = "Unequip"
		btn_uneq.add_theme_font_size_override("font_size", 8)
		btn_uneq.focus_mode = Control.FOCUS_NONE
		btn_uneq.pressed.connect(func() -> void: _unequip_slot_to_stash(slot_val))
		r3.add_child(btn_uneq)

	return card

func _render_dojo_battle_pouches() -> void:
	if battle_pouch_vbox == null:
		return
	for c in battle_pouch_vbox.get_children():
		c.queue_free()

	if active_hero == null or active_hero.inventory == null:
		return

	var pouch: Array[Consumable] = active_hero.inventory.battle_pouch
	for i in range(3):
		var con: Consumable = pouch[i] if i < pouch.size() else null
		var p_card := PanelContainer.new()
		var p_style := StyleBoxFlat.new()
		p_style.bg_color = Color(0.08, 0.11, 0.15, 0.8)
		p_style.border_color = Color(0.18, 0.25, 0.35, 0.6)
		p_style.set_border_width_all(1)
		p_style.set_corner_radius_all(3)
		p_style.content_margin_left = 6
		p_style.content_margin_right = 6
		p_style.content_margin_top = 2
		p_style.content_margin_bottom = 2
		p_card.add_theme_stylebox_override("panel", p_style)

		var p_row := HBoxContainer.new()
		p_card.add_child(p_row)

		var tag := Label.new()
		tag.text = "Slot %d:" % (i + 1)
		tag.add_theme_font_size_override("font_size", 8)
		tag.add_theme_color_override("font_color", Color(0.5, 0.7, 0.9))
		p_row.add_child(tag)

		if con == null:
			var empty := Label.new()
			empty.text = "— Empty Quick-Slot —"
			empty.add_theme_font_size_override("font_size", 8)
			empty.add_theme_color_override("font_color", Color(0.4, 0.5, 0.6))
			p_row.add_child(empty)
		else:
			var icon_str := "🧪 " if con.consumable_type == Consumable.ConsumableType.HEALTH_POTION else ("💣 " if con.consumable_type == Consumable.ConsumableType.THROWABLE_BOMB else "✦ ")
			var c_lbl := Label.new()
			c_lbl.text = icon_str + con.name
			c_lbl.add_theme_font_size_override("font_size", 8)
			c_lbl.add_theme_color_override("font_color", Color(0.4, 0.9, 0.6))
			p_row.add_child(c_lbl)

			p_row.add_spacer(false)

			var effect_lbl := Label.new()
			if con.heal_amount > 0:
				effect_lbl.text = "+%d HP" % con.heal_amount
			elif con.damage_amount > 0:
				effect_lbl.text = "%d DMG" % con.damage_amount
			else:
				effect_lbl.text = "+%d%% ATK (%d rds)" % [int(roundf(con.buff_percent * 100.0)), con.buff_duration_rounds]
			effect_lbl.add_theme_font_size_override("font_size", 8)
			effect_lbl.add_theme_color_override("font_color", Color(0.9, 0.85, 0.4))
			p_row.add_child(effect_lbl)

		battle_pouch_vbox.add_child(p_card)

func _get_rarity_color(rarity: Item.ItemRarity) -> Color:
	match rarity:
		Item.ItemRarity.COMMON:
			return Color(0.85, 0.85, 0.85)
		Item.ItemRarity.UNCOMMON:
			return Color(0.4, 0.9, 0.5)
		Item.ItemRarity.RARE:
			return Color(0.35, 0.75, 1.0)
		Item.ItemRarity.EPIC:
			return Color(0.75, 0.45, 1.0)
		Item.ItemRarity.LEGENDARY:
			return Color(1.0, 0.8, 0.2)
		_:
			return Color.WHITE

func _update_action_button_states(main_w: Equipment) -> void:
	if btn_melee_strike == null or btn_ranged_volley == null or btn_offhand_strike == null:
		return

	var off_w: Equipment = active_hero.inventory.get_offhand_weapon() if (active_hero and active_hero.inventory) else null
	var shield: Equipment = active_hero.inventory.get_shield() if (active_hero and active_hero.inventory) else null

	# Main Strike & Ranged Volley (Requirements 3 & 7)
	if main_w == null:
		btn_melee_strike.disabled = false
		btn_melee_strike.tooltip_text = "Execute unarmed melee strike against Dummy"
		btn_ranged_volley.disabled = true
		btn_ranged_volley.tooltip_text = "Disabled: Ranged Volley requires a ranged weapon (e.g. Bow)"
	elif main_w.is_ranged_weapon():
		btn_melee_strike.disabled = true
		btn_melee_strike.tooltip_text = "Disabled: Cannot perform Melee Strike with ranged weapon (%s)" % main_w.name
		btn_ranged_volley.disabled = false
		btn_ranged_volley.tooltip_text = "Execute ranged volley with %s against Dummy" % main_w.name
	else:
		btn_melee_strike.disabled = false
		btn_melee_strike.tooltip_text = "Execute main strike with %s against Dummy" % main_w.name
		btn_ranged_volley.disabled = true
		btn_ranged_volley.tooltip_text = "Disabled: Cannot perform Ranged Volley with melee weapon (%s)" % main_w.name

	# Off-Hand Strike (Requirement 7)
	if main_w != null and main_w.is_two_handed:
		btn_offhand_strike.disabled = true
		btn_offhand_strike.tooltip_text = "Disabled: Cannot use Off-Hand while wielding 2-Handed weapon (%s)" % main_w.name
	elif off_w != null:
		btn_offhand_strike.disabled = false
		btn_offhand_strike.tooltip_text = "Execute off-hand strike with %s (scales with %s * 1.5)" % [off_w.name, off_w.scaling_stat]
	elif shield != null:
		btn_offhand_strike.disabled = true
		btn_offhand_strike.tooltip_text = "Disabled: %s is a Shield (Mitigation only, not offensive)" % shield.name
	else:
		btn_offhand_strike.disabled = true
		btn_offhand_strike.tooltip_text = "Disabled: No off-hand weapon equipped"

	# Switch Weapon Action (Requirement 7)
	if btn_switch_active_weapon != null:
		if main_w != null and main_w.is_two_handed:
			btn_switch_active_weapon.disabled = false
			btn_switch_active_weapon.tooltip_text = "Use an action to switch from 2-Handed weapon to 1-Handed loadout"
		elif off_w != null:
			btn_switch_active_weapon.disabled = false
			btn_switch_active_weapon.tooltip_text = "Use an action to swap active Main Hand and Off Hand weapons"
		else:
			btn_switch_active_weapon.disabled = false
			btn_switch_active_weapon.tooltip_text = "Use an action to switch active weapon from stash"

func _rebuild_hero_tabs() -> void:
	for c in hero_tabs_hbox.get_children():
		c.queue_free()

	for hero in party_allies:
		var btn := Button.new()
		var is_active: bool = (hero == active_hero)
		btn.text = "%s (Lv %d)" % [hero.name, hero.stats.level if hero.stats else 1]
		btn.add_theme_font_size_override("font_size", 9)
		btn.focus_mode = Control.FOCUS_NONE
		if is_active:
			btn.add_theme_color_override("font_color", Color(0.4, 0.95, 0.6))
		btn.pressed.connect(func() -> void:
			active_hero = hero
			_refresh_all_views()
		)
		hero_tabs_hbox.add_child(btn)

func _append_formula_log(bbcode: String) -> void:
	if formula_log_rich != null:
		formula_log_rich.append_text(bbcode + "\n")

# ------------------------------------------------------------------------------
# 4. Interactive Skill Action Handlers & Formula Verification
# ------------------------------------------------------------------------------

# Thin wrapper – all math now lives in DamageCalculator (scripts/core/combat/damage_calculator.gd)
func _calculate_combat_damage(
	attacker: Combatant,
	target: Combatant,
	is_ranged: bool,
	use_offhand: bool,
	skill_mult: float = 1.0,
	force_crit: bool = false,
	allow_random_crit: bool = false
) -> Dictionary:
	return DamageCalculator.compute(attacker, target, is_ranged, use_offhand, skill_mult, force_crit, allow_random_crit)

func _log_combat_damage_breakdown(action_title: String, res: Dictionary, actual_dmg: int) -> void:
	var w: Equipment = res["weapon"]
	var w_name: String = w.name if w != null else "Unarmed"
	_append_formula_log("[color=#fde047][b]%s: %s strikes %s with %s![/b][/color]" % [action_title, active_hero.name, test_dummy.name, w_name])

	# Step 1: Weapon Power
	var pwr_parts: Array[String] = ["Base: %d" % res["base_w_power"]]
	if res["tier_boost"] > 0:
		pwr_parts.append("[color=#fbbf24]+Tier: %d[/color]" % res["tier_boost"])
	if res["mastery_pwr"] > 0:
		pwr_parts.append("[color=#a855f7]+Mastery: %d[/color]" % res["mastery_pwr"])
	_append_formula_log("  1. Wep Power: [color=#e2e8f0]%d[/color] (%s)" % [res["total_w_power"], ", ".join(pwr_parts)])

	if res["pen_ratio"] > 0.0:
		var pen_raw: Variant = res.get("pen_ratio", 0.0)
		var pen_val: float = pen_raw if pen_raw is float else 0.0
		var pen_pct: int = int(roundf(pen_val * 100.0))
		_append_formula_log("     [color=#f87171]⚠️ Deficit Penalty (-%d Deficit): %d * (1 - %d%%) = %.1f Wep Power[/color]" % [res["def_amount"], res["total_w_power"], pen_pct, res["penalized_w_power"]])

	# Step 2: Stat Scaling
	_append_formula_log("  2. Stat Scaling: [color=#e2e8f0]%d %s[/color] * 1.5 = [color=#38bdf8]+%.1f Power[/color]" % [res["stat_val"], res["stat_name"], res["stat_scaling_contrib"]])

	# Step 3: Base Attack Power & Buffs
	_append_formula_log("  3. Base Attack Power: (%.1f Wep + %.1f Stat) = [color=#38bdf8]%.1f[/color]" % [res["penalized_w_power"], res["stat_scaling_contrib"], res["unbuffed_attack_power"]])
	if res["buff_mult"] != 1.0:
		_append_formula_log("     [color=#c084fc]✦ Buff Multiplier: (1 + %.2f) -> Gross ATK Power: %d[/color]" % [res["buff_mult"] - 1.0, res["final_attack_power"]])
	else:
		_append_formula_log("     Final Attack Power: [color=#38bdf8]%d[/color]" % res["final_attack_power"])

	# Step 4: Combat Multipliers
	var mult_parts: Array[String] = []
	if res["skill_mult"] != 1.0:
		mult_parts.append("Skill: x%.2f" % res["skill_mult"])
	var mit_raw: Variant = res.get("mit_ratio", 0.0)
	var mit_val: float = mit_raw if mit_raw is float else 0.0
	mult_parts.append("Mitigation: x%.2f (%d%%)" % [res["mit_mult"], int(roundf(mit_val * 100.0))])
	if res["stance_mult"] != 0.0:
		var sign_str := "+" if res["stance_mult"] > 0 else ""
		mult_parts.append("[color=#fbbf24]Stance: (1 %s%.2f = x%.2f)[/color]" % [sign_str, res["stance_mult"], res["stance_factor"]])
	else:
		mult_parts.append("Stance: Neutral (x1.0)")

	if res["is_crit"]:
		mult_parts.append("[color=#f43f5e]CRIT! x%.1f (Rate: %.1f%%)[/color]" % [res["crit_dmg_mult"], res["crit_rate"]])
	else:
		mult_parts.append("Crit: No (Rate: %.1f%%)" % res["crit_rate"])

	if res["effect_factor"] != 1.0:
		mult_parts.append("Effect: x%.2f" % res["effect_factor"])

	_append_formula_log("  4. Combat Multipliers: %s" % " | ".join(mult_parts))
	_append_formula_log("  • [color=#4ade80][b]RESULT: %d Final Damage Inflicted! (Dummy HP: %d / %d)[/b][/color]\n" % [actual_dmg, test_dummy.hp, test_dummy.max_hp])

func _on_test_melee_strike() -> void:
	if active_hero == null or test_dummy == null:
		return

	var main_w: Equipment = active_hero.inventory.get_main_weapon() if active_hero.inventory else null
	if main_w != null and main_w.is_ranged_weapon():
		_append_formula_log("[color=#f87171]❌ Action Failed: Cannot execute Melee Strike while wielding ranged weapon (%s)![/color]\n" % main_w.name)
		return

	var force_crit: bool = force_crit_check != null and force_crit_check.button_pressed
	var res: Dictionary = _calculate_combat_damage(active_hero, test_dummy, false, false, 1.0, force_crit, true)
	var final_dmg_raw: Variant = res.get("final_dmg", 0)
	var final_dmg: int = final_dmg_raw if final_dmg_raw is int else 0
	var actual_dmg: int = test_dummy.take_damage(final_dmg, true)

	var mit_raw_fx: Variant = res.get("mit_ratio", 0.0)
	var mit_ratio_fx: float = mit_raw_fx if mit_raw_fx is float else 0.0
	_trigger_stage_fx(hero_token_ctrl, dummy_token_ctrl, actual_dmg, mit_ratio_fx > 0.0)
	_log_combat_damage_breakdown("⚔ MAIN STRIKE", res, actual_dmg)
	_refresh_all_views()

func _on_test_offhand_strike() -> void:
	if active_hero == null or test_dummy == null:
		return

	var main_w: Equipment = active_hero.inventory.get_main_weapon() if active_hero.inventory else null
	if main_w != null and main_w.is_two_handed:
		_append_formula_log("[color=#f87171]❌ Action Failed: Cannot use Off-Hand while wielding 2-Handed weapon (%s)![/color]\n" % main_w.name)
		return

	var off_w: Equipment = active_hero.inventory.get_offhand_weapon() if active_hero.inventory else null
	if off_w == null:
		_append_formula_log("[color=#f87171]❌ Action Failed: No off-hand weapon equipped![/color]\n")
		return
	if off_w.is_shield() or off_w.base_power == 0:
		_append_formula_log("[color=#f87171]❌ Action Failed: %s is a Shield (Mitigation only, not offensive)![/color]\n" % off_w.name)
		return

	var force_crit: bool = force_crit_check != null and force_crit_check.button_pressed
	var res: Dictionary = _calculate_combat_damage(active_hero, test_dummy, false, true, 1.0, force_crit, true)
	var final_dmg_raw: Variant = res.get("final_dmg", 0)
	var final_dmg: int = final_dmg_raw if final_dmg_raw is int else 0
	var actual_dmg: int = test_dummy.take_damage(final_dmg, true)

	var mit_raw_fx: Variant = res.get("mit_ratio", 0.0)
	var mit_ratio_fx: float = mit_raw_fx if mit_raw_fx is float else 0.0
	_trigger_stage_fx(hero_token_ctrl, dummy_token_ctrl, actual_dmg, mit_ratio_fx > 0.0)
	_log_combat_damage_breakdown("🗡 OFF-HAND STRIKE", res, actual_dmg)
	_refresh_all_views()

func _on_test_ranged_volley() -> void:
	if active_hero == null or test_dummy == null:
		return

	var main_w: Equipment = active_hero.inventory.get_main_weapon() if active_hero.inventory else null
	var w_name: String = main_w.name if main_w != null else "Unarmed"
	if main_w == null or not main_w.is_ranged_weapon():
		_append_formula_log("[color=#f87171]❌ Action Failed: Cannot execute Ranged Volley without a ranged weapon! Currently wielding: %s.[/color]\n" % w_name)
		return

	var force_crit: bool = force_crit_check != null and force_crit_check.button_pressed
	var res: Dictionary = _calculate_combat_damage(active_hero, test_dummy, true, false, 1.0, force_crit, true)
	var final_dmg_raw: Variant = res.get("final_dmg", 0)
	var final_dmg: int = final_dmg_raw if final_dmg_raw is int else 0
	var actual_dmg: int = test_dummy.take_damage(final_dmg, true)

	var mit_raw_fx: Variant = res.get("mit_ratio", 0.0)
	var mit_ratio_fx: float = mit_raw_fx if mit_raw_fx is float else 0.0
	_trigger_stage_fx(hero_token_ctrl, dummy_token_ctrl, actual_dmg, mit_ratio_fx > 0.0)
	_log_combat_damage_breakdown("🏹 RANGED VOLLEY", res, actual_dmg)
	_refresh_all_views()

func _on_test_switch_active_weapon() -> void:
	if active_hero == null or active_hero.inventory == null:
		return

	var inv: Inventory = active_hero.inventory
	var main_w: Equipment = inv.get_main_weapon()
	var off_w: Equipment = inv.get_offhand_weapon()

	if main_w != null and off_w != null:
		# 1H dual wield: Swap main hand and off hand
		inv.equipped_gear.erase(Equipment.EquipSlot.MAIN_HAND)
		inv.equipped_gear.erase(Equipment.EquipSlot.OFF_HAND)
		inv.equipped_gear[Equipment.EquipSlot.MAIN_HAND] = off_w
		inv.equipped_gear[Equipment.EquipSlot.OFF_HAND] = main_w
		_spawn_floating_text(hero_token_ctrl.position, "🔄 Swapped Hands", Color(0.4, 0.9, 1.0))
		_append_formula_log("[color=#38bdf8][b]🔄 WEAPON SWITCH: %s swapped hands! Active Main Hand is now %s (Off-Hand: %s).[/b][/color]\n" % [active_hero.name, off_w.name, main_w.name])
	elif main_w != null and main_w.is_two_handed:
		# 2-Handed weapon equipped: switch to first eligible 1H weapon in stash
		var alt_1h: Equipment = null
		if shared_stash != null:
			for item_raw: Item in shared_stash.items:
				if item_raw is Equipment:
					var item: Equipment = item_raw as Equipment
					if item.is_weapon() and not item.is_two_handed:
						if active_hero.stats == null or active_hero.stats.archetype == null or active_hero.stats.archetype.can_equip_gear(item):
							alt_1h = item
							break
		if alt_1h != null:
			inv.unequip(Equipment.EquipSlot.MAIN_HAND)
			shared_stash.remove_item(alt_1h)
			shared_stash.add_item(main_w)
			inv.equip(alt_1h, Equipment.EquipSlot.MAIN_HAND)
			_spawn_floating_text(hero_token_ctrl.position, "🔄 1H Weapon", Color(0.4, 0.9, 1.0))
			_append_formula_log("[color=#38bdf8][b]🔄 WEAPON SWITCH: %s stowed 2H %s and drew 1H %s![/b][/color]\n" % [active_hero.name, main_w.name, alt_1h.name])
		else:
			_append_formula_log("[color=#94a3b8]🔄 WEAPON SWITCH: %s adjusted grip on 2H %s (No alternative 1H weapon in stash).[/color]\n" % [active_hero.name, main_w.name])
	else:
		# Check for eligible 2H weapon in stash to switch into
		var alt_2h: Equipment = null
		if shared_stash != null:
			for item_raw: Item in shared_stash.items:
				if item_raw is Equipment:
					var item: Equipment = item_raw as Equipment
					if item.is_weapon() and item.is_two_handed:
						if active_hero.stats == null or active_hero.stats.archetype == null or active_hero.stats.archetype.can_equip_gear(item):
							alt_2h = item
							break
		if alt_2h != null:
			if main_w != null:
				inv.unequip(Equipment.EquipSlot.MAIN_HAND)
				shared_stash.add_item(main_w)
			var cur_off: Equipment = inv.unequip(Equipment.EquipSlot.OFF_HAND)
			if cur_off != null and shared_stash != null:
				shared_stash.add_item(cur_off)
			shared_stash.remove_item(alt_2h)
			inv.equip(alt_2h, Equipment.EquipSlot.MAIN_HAND)
			_spawn_floating_text(hero_token_ctrl.position, "🔄 2H Weapon", Color(0.4, 0.9, 1.0))
			_append_formula_log("[color=#38bdf8][b]🔄 WEAPON SWITCH: %s drew 2H %s![/b][/color]\n" % [active_hero.name, alt_2h.name])
		else:
			_append_formula_log("[color=#94a3b8]🔄 WEAPON SWITCH: %s shifted combat ready stance.[/color]\n" % active_hero.name)

	active_hero.sync_from_stats_and_gear()
	_refresh_all_views()

func _on_test_focus_buff() -> void:
	if active_hero == null:
		return
	active_hero.apply_attack_boost(1.5, 2)
	_animate_token_glow(hero_token_ctrl, Color(1.8, 1.5, 0.4))
	_spawn_floating_text(hero_token_ctrl.position, "✦ +50% ATK BOOST (2 rds)", Color(1.0, 0.85, 0.2))
	_append_formula_log("[color=#c084fc][b]✦ FOCUS SKILL: %s channelled Focus! Next attacks deal 1.5x damage for 2 rounds.[/b][/color]\n" % active_hero.name)
	_refresh_all_views()

func _on_test_defend() -> void:
	if active_hero == null:
		return
	active_hero.defense_mitigation_ratio = 0.5
	_animate_token_glow(hero_token_ctrl, Color(0.5, 0.8, 1.8))
	_spawn_floating_text(hero_token_ctrl.position, "🛡 DEFEND [50% MIT]", Color(0.4, 0.8, 1.0))
	_append_formula_log("[color=#60a5fa][b]🛡 DEFEND BARRIER: %s activated Defensive Guard (50%% damage mitigation active).[/b][/color]\n" % active_hero.name)
	_refresh_all_views()

func _on_test_pouch_item() -> void:
	if active_hero == null or active_hero.inventory == null or pouch_select_option == null:
		return

	var pouch := active_hero.inventory.battle_pouch
	if pouch.is_empty():
		_append_formula_log("[color=#f87171]⚠️ Battle pouch is empty. Open 'Manage Gear & Stash' to load consumables.[/color]\n")
		return

	var idx: int = pouch_select_option.get_selected_id()
	if idx < 0 or idx >= pouch.size():
		idx = 0

	var con: Consumable = pouch[idx]
	active_hero.inventory.remove_from_battle_pouch(con)

	match con.consumable_type:
		Consumable.ConsumableType.HEALTH_POTION:
			active_hero.heal(con.heal_amount)
			_spawn_floating_text(hero_token_ctrl.position, "+%d HP [HEAL]" % con.heal_amount, Color(0.3, 0.9, 0.5))
			_append_formula_log("[color=#4ade80][b]🧪 RESTORATIVE: %s consumed %s and recovered %d HP![/b][/color]\n" % [active_hero.name, con.name, con.heal_amount])
		Consumable.ConsumableType.THROWABLE_BOMB:
			var actual_dmg: int = test_dummy.take_damage(con.damage_amount)
			_trigger_stage_fx(hero_token_ctrl, dummy_token_ctrl, actual_dmg, false)
			_append_formula_log("[color=#fb923c][b]💣 EXPLOSIVE: %s hurled %s at Dummy for %d explosive damage![/b][/color]\n" % [active_hero.name, con.name, actual_dmg])
		_:
			active_hero.apply_attack_boost(1.0 + con.buff_percent, con.buff_duration_rounds)
			_spawn_floating_text(hero_token_ctrl.position, "✦ ELIXIR BOOST!", Color(1.0, 0.8, 0.2))
			_append_formula_log("[color=#c084fc][b]🧪 ELIXIR: %s drank %s (+50%% ATK for %d rounds)![/b][/color]\n" % [active_hero.name, con.name, con.buff_duration_rounds])

	_refresh_all_views()

func _refresh_pouch_dropdown() -> void:
	if pouch_select_option == null or active_hero == null or active_hero.inventory == null:
		return
	pouch_select_option.clear()
	var pouch := active_hero.inventory.battle_pouch
	if pouch.is_empty():
		pouch_select_option.add_item("— Battle Pouch Empty —", 0)
		pouch_select_option.set_item_disabled(0, true)
		if btn_use_pouch_item != null:
			btn_use_pouch_item.disabled = true
	else:
		for i in range(pouch.size()):
			var con: Consumable = pouch[i]
			var icon_str := "🧪 " if con.consumable_type == Consumable.ConsumableType.HEALTH_POTION else ("💣 " if con.consumable_type == Consumable.ConsumableType.THROWABLE_BOMB else "✦ ")
			var detail := ""
			if con.heal_amount > 0: detail = "(+%d HP)" % con.heal_amount
			elif con.damage_amount > 0: detail = "(%d DMG)" % con.damage_amount
			else: detail = "(+50% ATK)"
			pouch_select_option.add_item("%sSlot %d: %s %s" % [icon_str, i + 1, con.name, detail], i)
		if btn_use_pouch_item != null:
			btn_use_pouch_item.disabled = false

func _on_test_stance_shift() -> void:
	if active_hero == null:
		return
	if active_hero.current_stance == CombatTypes.StanceType.NEUTRAL:
		active_hero.set_stance(CombatTypes.StanceType.OFFENSIVE)
		_spawn_floating_text(hero_token_ctrl.position, "⚔ OFFENSIVE STANCE", Color(0.95, 0.4, 0.4))
		_append_formula_log("[color=#f87171][b]🔄 STANCE: %s switched to Offensive Stance (+20%% Attack Power).[/b][/color]\n" % active_hero.name)
	else:
		active_hero.set_stance(CombatTypes.StanceType.NEUTRAL)
		_spawn_floating_text(hero_token_ctrl.position, "🔄 NEUTRAL STANCE", Color(0.7, 0.8, 0.9))
		_append_formula_log("[color=#94a3b8][b]🔄 STANCE: %s shifted back to Neutral Stance.[/b][/color]\n" % active_hero.name)
	_refresh_all_views()

func _on_toggle_dummy_defense_pressed() -> void:
	if test_dummy == null:
		return
	if test_dummy.defense_mitigation_ratio > 0.0:
		test_dummy.defense_mitigation_ratio = 0.0
		_spawn_floating_text(dummy_token_ctrl.position, "Dummy: 0% Mit", Color(0.5, 0.8, 0.6))
		_append_formula_log("[color=#4ade80]🛡 DUMMY STANCE: Switched to Neutral (0%% Mitigation). Hits will deal full damage.[/color]\n")
	else:
		test_dummy.defense_mitigation_ratio = 0.5
		_spawn_floating_text(dummy_token_ctrl.position, "🛡 Dummy: 50% Mit", Color(0.4, 0.8, 1.0))
		_append_formula_log("[color=#60a5fa]🛡 DUMMY STANCE: Switched to Defending (50%% Mitigation). Hits will be halved.[/color]\n")
	_refresh_all_views()

func _on_heal_dummy_pressed() -> void:
	if test_dummy == null:
		return
	test_dummy.hp = test_dummy.max_hp
	_spawn_floating_text(dummy_token_ctrl.position, "💖 500 / 500 HP", Color(0.3, 0.9, 0.6))
	_append_formula_log("[color=#4ade80]💖 DUMMY HEAL: Training Dummy HP reset to 500/500.[/color]\n")
	_refresh_all_views()

# ------------------------------------------------------------------------------
# 5. Interactive Stat Allocation Mode (Requirement 8)
# ------------------------------------------------------------------------------

func _on_points_btn_pressed() -> void:
	if active_hero == null or active_hero.stats == null:
		return
	if is_allocating_stats:
		_on_cancel_allocation_pressed()
	else:
		if active_hero.stats.unspent_attribute_points <= 0:
			_append_formula_log("[color=#94a3b8]ℹ No unspent attribute points available to allocate.[/color]")
			return
		is_allocating_stats = true
		pending_free_points = active_hero.stats.unspent_attribute_points
		for k: String in pending_allocated_stats:
			pending_allocated_stats[k] = 0
		_refresh_all_views()
		_append_formula_log("[color=#38bdf8]⚡ Stat Allocation Mode Active: Use [+] and [-] to allocate points, then Confirm.[/color]")

func _on_stat_plus_pressed(k: String) -> void:
	if not is_allocating_stats or pending_free_points <= 0:
		return
	pending_free_points -= 1
	var current_p: int = pending_allocated_stats[k] if pending_allocated_stats.has(k) else 0
	pending_allocated_stats[k] = current_p + 1
	_refresh_all_views()

func _on_stat_minus_pressed(k: String) -> void:
	if not is_allocating_stats:
		return
	var current_p: int = pending_allocated_stats[k] if pending_allocated_stats.has(k) else 0
	if current_p <= 0:
		# Requirement 8: - should only remove newly allocated points and not give extra points back
		return
	pending_allocated_stats[k] = current_p - 1
	pending_free_points += 1
	_refresh_all_views()

func _on_reset_allocation_pressed() -> void:
	if not is_allocating_stats or active_hero == null or active_hero.stats == null:
		return
	pending_free_points = active_hero.stats.unspent_attribute_points
	for k: String in pending_allocated_stats:
		pending_allocated_stats[k] = 0
	_refresh_all_views()
	_append_formula_log("[color=#94a3b8]↺ Pending stat allocations reset to baseline.[/color]")

func _on_confirm_allocation_pressed() -> void:
	if not is_allocating_stats or active_hero == null or active_hero.stats == null:
		return
	var allocated_summary: Array[String] = []
	for k: String in pending_allocated_stats:
		var pts: int = pending_allocated_stats[k]
		if pts > 0:
			match k:
				"STR": active_hero.stats.str_stat += pts
				"DEX": active_hero.stats.dex_stat += pts
				"INT": active_hero.stats.int_stat += pts
				"CONS": active_hero.stats.cons_stat += pts
				"PERC": active_hero.stats.perc_stat += pts
				"KNOW": active_hero.stats.know_stat += pts
			allocated_summary.append("+%d %s" % [pts, k])
	active_hero.stats.unallocated_attribute_points = pending_free_points
	is_allocating_stats = false
	for k: String in pending_allocated_stats:
		pending_allocated_stats[k] = 0

	active_hero.sync_from_stats_and_gear()
	_refresh_all_views()
	var alloc_str := ", ".join(allocated_summary) if not allocated_summary.is_empty() else "No points spent"
	_append_formula_log("[color=#4ade80]✔ Stat Allocation Confirmed for %s: %s (Remaining: %d).[/color]" % [active_hero.name, alloc_str, active_hero.stats.unspent_attribute_points])

func _on_cancel_allocation_pressed() -> void:
	is_allocating_stats = false
	for k: String in pending_allocated_stats:
		pending_allocated_stats[k] = 0
	pending_free_points = 0
	_refresh_all_views()
	_append_formula_log("[color=#94a3b8]✖ Stat Allocation Cancelled without saving.[/color]")

# ------------------------------------------------------------------------------
# 6. Base UI Gear Equip & Stash Picker (Requirement 4)
# ------------------------------------------------------------------------------

func _build_gear_picker_dialog() -> void:
	gear_picker_dialog = PanelContainer.new()
	gear_picker_dialog.custom_minimum_size = Vector2(420, 340)
	gear_picker_dialog.set_anchors_preset(Control.PRESET_CENTER)
	gear_picker_dialog.grow_horizontal = Control.GROW_DIRECTION_BOTH
	gear_picker_dialog.grow_vertical = Control.GROW_DIRECTION_BOTH
	gear_picker_dialog.position = Vector2(430, 160)
	var d_style := StyleBoxFlat.new()
	d_style.bg_color = Color(0.09, 0.12, 0.18, 0.98)
	d_style.border_color = Color(0.35, 0.50, 0.75, 0.95)
	d_style.set_border_width_all(2)
	d_style.set_corner_radius_all(6)
	d_style.content_margin_left = 12
	d_style.content_margin_right = 12
	d_style.content_margin_top = 8
	d_style.content_margin_bottom = 8
	gear_picker_dialog.add_theme_stylebox_override("panel", d_style)
	gear_picker_dialog.visible = false
	canvas_layer.add_child(gear_picker_dialog)

	var v := VBoxContainer.new()
	v.add_theme_constant_override("separation", 6)
	gear_picker_dialog.add_child(v)

	picker_title_lbl = Label.new()
	picker_title_lbl.text = "Select Equipment from Stash"
	picker_title_lbl.add_theme_font_size_override("font_size", 11)
	picker_title_lbl.add_theme_color_override("font_color", Color(0.95, 0.85, 0.4))
	v.add_child(picker_title_lbl)

	var sc := ScrollContainer.new()
	sc.custom_minimum_size = Vector2(396, 250)
	sc.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	v.add_child(sc)

	picker_items_vbox = VBoxContainer.new()
	picker_items_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	picker_items_vbox.add_theme_constant_override("separation", 4)
	sc.add_child(picker_items_vbox)

	var btn_close := Button.new()
	btn_close.text = "Close"
	btn_close.add_theme_font_size_override("font_size", 9)
	btn_close.focus_mode = Control.FOCUS_NONE
	btn_close.pressed.connect(func() -> void: gear_picker_dialog.visible = false)
	v.add_child(btn_close)

func _open_base_equip_picker(slot_val: Equipment.EquipSlot) -> void:
	if active_hero == null or shared_stash == null or picker_items_vbox == null:
		return

	picker_slot = slot_val
	var slot_name: String = _get_slot_name(slot_val)
	var arch_name: String = active_hero.stats.archetype.display_name if (active_hero.stats and active_hero.stats.archetype) else "Hero"
	picker_title_lbl.text = "Equip %s (%s Eligible Only)" % [slot_name, arch_name]

	for c in picker_items_vbox.get_children():
		c.queue_free()

	var eligible_count: int = 0
	for item_raw: Item in shared_stash.items:
		if not (item_raw is Equipment):
			continue
		var item: Equipment = item_raw as Equipment

		# Slot filter
		var matches_slot: bool = false
		match slot_val:
			Equipment.EquipSlot.MAIN_HAND:
				matches_slot = item.is_weapon()
			Equipment.EquipSlot.OFF_HAND:
				matches_slot = item.is_shield() or (item.is_weapon() and not item.is_two_handed)
			Equipment.EquipSlot.BODY:
				matches_slot = (item.slot == Equipment.EquipSlot.BODY)
			Equipment.EquipSlot.ACCESSORY_1, Equipment.EquipSlot.ACCESSORY_2:
				matches_slot = (item.slot == Equipment.EquipSlot.ACCESSORY_1 or item.slot == Equipment.EquipSlot.ACCESSORY_2)

		if not matches_slot:
			continue

		# Archetype eligibility filter (Requirement 4: "show only elegible gear for the character")
		if active_hero.stats != null and active_hero.stats.archetype != null:
			if not active_hero.stats.archetype.can_equip_gear(item):
				continue

		eligible_count += 1
		var item_card := _create_picker_item_card(item, slot_val)
		picker_items_vbox.add_child(item_card)

	if eligible_count == 0:
		var empty_lbl := Label.new()
		empty_lbl.text = "No eligible %s items found in stash for %s." % [slot_name, arch_name]
		empty_lbl.add_theme_font_size_override("font_size", 9)
		empty_lbl.add_theme_color_override("font_color", Color(0.7, 0.7, 0.7))
		picker_items_vbox.add_child(empty_lbl)

	gear_picker_dialog.visible = true

func _create_picker_item_card(item: Equipment, slot_val: Equipment.EquipSlot) -> PanelContainer:
	var card := PanelContainer.new()
	var st_box := StyleBoxFlat.new()
	st_box.bg_color = Color(0.12, 0.16, 0.22, 0.9)
	st_box.border_color = Color(0.24, 0.35, 0.50, 0.8)
	st_box.set_border_width_all(1)
	st_box.set_corner_radius_all(4)
	st_box.content_margin_left = 6
	st_box.content_margin_right = 6
	st_box.content_margin_top = 4
	st_box.content_margin_bottom = 4
	card.add_theme_stylebox_override("panel", st_box)

	var hbox := HBoxContainer.new()
	card.add_child(hbox)

	var info_v := VBoxContainer.new()
	info_v.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	info_v.add_theme_constant_override("separation", 1)
	hbox.add_child(info_v)

	var name_lbl := Label.new()
	name_lbl.text = item.name
	name_lbl.add_theme_font_size_override("font_size", 9)
	name_lbl.add_theme_color_override("font_color", _get_rarity_color(item.rarity))
	info_v.add_child(name_lbl)

	var stat_parts: Array[String] = []
	if item.base_power > 0: stat_parts.append("Pwr: %d" % item.base_power)
	if item.mitigation > 0: stat_parts.append("Mit: %d" % item.mitigation)
	if item.burden > 0: stat_parts.append("Brd: %d" % item.burden)
	var b_str := item.get_stat_bonuses_display()
	if not b_str.is_empty(): stat_parts.append(b_str)

	var stats_lbl := Label.new()
	stats_lbl.text = " • ".join(stat_parts)
	stats_lbl.add_theme_font_size_override("font_size", 8)
	stats_lbl.add_theme_color_override("font_color", Color(0.8, 0.9, 0.9))
	info_v.add_child(stats_lbl)

	var btn_pick := Button.new()
	btn_pick.text = "Equip"
	btn_pick.add_theme_font_size_override("font_size", 9)
	btn_pick.focus_mode = Control.FOCUS_NONE
	btn_pick.pressed.connect(func() -> void: _equip_item_from_picker(item, slot_val))
	hbox.add_child(btn_pick)

	return card

func _equip_item_from_picker(item: Equipment, slot_val: Equipment.EquipSlot) -> void:
	if active_hero == null or active_hero.inventory == null:
		return

	# If equipping 2-handed weapon in main hand, unequip off-hand to stash
	if slot_val == Equipment.EquipSlot.MAIN_HAND and item.is_two_handed:
		var off_eq: Equipment = active_hero.inventory.unequip(Equipment.EquipSlot.OFF_HAND)
		if off_eq != null and shared_stash != null:
			shared_stash.add_item(off_eq)
			_append_formula_log("[color=#94a3b8]ℹ Unequipped off-hand (%s) to stash because %s is 2-Handed.[/color]" % [off_eq.name, item.name])

	# Unequip existing item in slot if any
	var old_eq: Equipment = active_hero.inventory.unequip(slot_val)
	if old_eq != null and shared_stash != null:
		shared_stash.add_item(old_eq)

	# Remove new item from stash and equip
	if shared_stash != null:
		shared_stash.remove_item(item)
	active_hero.inventory.equip(item, slot_val)
	_append_formula_log("[color=#4ade80]🎒 Equipped [b]%s[/b] to %s.[/color]" % [item.name, active_hero.name])

	gear_picker_dialog.visible = false
	active_hero.sync_from_stats_and_gear()
	_refresh_all_views()

func _unequip_slot_to_stash(slot_val: Equipment.EquipSlot) -> void:
	if active_hero == null or active_hero.inventory == null:
		return
	var old_eq: Equipment = active_hero.inventory.unequip(slot_val)
	if old_eq != null and shared_stash != null:
		shared_stash.add_item(old_eq)
		_append_formula_log("[color=#94a3b8]🎒 Unequipped [b]%s[/b] to shared stash.[/color]" % old_eq.name)
	active_hero.sync_from_stats_and_gear()
	_refresh_all_views()

func _get_slot_name(slot_val: Equipment.EquipSlot) -> String:
	return Equipment.slot_display_name(slot_val)

# ------------------------------------------------------------------------------
# 7. Reset Dojo (Requirement 6)
# ------------------------------------------------------------------------------

func _reset_dojo_to_baseline() -> void:
	_init_dojo_data()
	is_allocating_stats = false
	for k: String in pending_allocated_stats:
		pending_allocated_stats[k] = 0
	pending_free_points = 0
	if formula_log_rich != null:
		formula_log_rich.clear()
		_append_formula_log("[color=#38bdf8][b]>>> Dojo Reset to Initial Baseline State.[/b][/color]")
		_append_formula_log("[color=#94a3b8]• Melee/Ranged Power = (Wep Power * (1-Deficit) + Stat*Scaling) * (1 + Buffs - Debuffs)[/color]")
		_append_formula_log("[color=#94a3b8]• Final Damage = ATK * Skill * (1-Mitigation) * (1+Stance) * Crit * (1+Effect)[/color]\n")
	if gear_picker_dialog != null:
		gear_picker_dialog.visible = false
	if gear_modal != null:
		gear_modal.setup_heroes(party_allies, shared_stash)
	_refresh_all_views()
	_spawn_floating_text(HERO_POS, "↺ Dojo Reset", Color(0.4, 0.9, 1.0))
	print("Dojo reset to initial baseline state.")

# ------------------------------------------------------------------------------
# 8. Visual Animation & Floating Combat Text FX
# ------------------------------------------------------------------------------

func _trigger_stage_fx(actor_ctrl: Control, target_ctrl: Control, dmg: int, is_mit: bool) -> void:
	if actor_ctrl != null:
		var orig_pos := actor_ctrl.position
		var tw := create_tween()
		tw.tween_property(actor_ctrl, "position", orig_pos + Vector2(25, 0), 0.12).set_trans(Tween.TRANS_QUAD)
		tw.tween_property(actor_ctrl, "position", orig_pos, 0.12).set_trans(Tween.TRANS_QUAD)

	if target_ctrl != null:
		var orig_target := target_ctrl.position
		var tw2 := create_tween()
		tw2.tween_property(target_ctrl, "position", orig_target + Vector2(6, -3), 0.06)
		tw2.tween_property(target_ctrl, "position", orig_target + Vector2(-6, 3), 0.06)
		tw2.tween_property(target_ctrl, "position", orig_target, 0.06)

		var txt := "-%d" % dmg
		if is_mit:
			txt += " [MIT]"
		_spawn_floating_text(target_ctrl.position, txt, Color(0.4, 0.85, 1.0) if is_mit else Color(1.0, 0.35, 0.35))

func _animate_token_glow(target_ctrl: Control, glow_col: Color) -> void:
	if target_ctrl == null:
		return
	var tw := create_tween()
	tw.tween_property(target_ctrl, "modulate", glow_col, 0.18)
	tw.tween_property(target_ctrl, "modulate", Color.WHITE, 0.25)

func _spawn_floating_text(center_pos: Vector2, text: String, color: Color) -> void:
	if canvas_layer == null:
		return
	var lbl := Label.new()
	lbl.text = text
	lbl.position = center_pos + Vector2(-40, -45)
	lbl.add_theme_font_size_override("font_size", 11)
	lbl.add_theme_color_override("font_color", color)
	canvas_layer.add_child(lbl)

	var tw := create_tween()
	tw.parallel().tween_property(lbl, "position:y", lbl.position.y - 25.0, 0.65).set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tw.parallel().tween_property(lbl, "modulate:a", 0.0, 0.65).set_trans(Tween.TRANS_LINEAR)
	tw.tween_callback(lbl.queue_free)

func _get_combatant_sprite_texture(c: Combatant) -> Texture2D:
	if c == null:
		return null
	if c.name == "Warrior":
		return AssetUtils.load_svg_texture("res://assets/party/sprites/sprite_warrior.svg")
	elif c.name == "Ranger":
		return AssetUtils.load_svg_texture("res://assets/party/sprites/sprite_ranger.svg")
	elif c.name == "Training Dummy":
		return AssetUtils.load_svg_texture("res://assets/enemy/sprites/sprite_dummy.svg")
	return null

# ------------------------------------------------------------------------------
# 9. Headless Verification Test Suite
# ------------------------------------------------------------------------------

func _run_headless_dojo_assertions() -> bool:
	return DojoAssertions.run(self)
