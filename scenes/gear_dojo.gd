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
var hero_points_lbl: Label
var stat_display_labels: Dictionary = {} # String -> Label
var stat_box_containers: Dictionary = {} # String -> PanelContainer
var derived_melee_lbl: Label
var derived_ranged_lbl: Label
var derived_speed_lbl: Label
var derived_hp_lbl: Label
var derived_mit_lbl: Label
var derived_burden_lbl: Label
var derived_rows: Dictionary = {} # String -> HBoxContainer
var equipped_weapon_lbl: Label
var equipped_shield_lbl: Label
var equipped_armor_lbl: Label
var equipped_acc1_lbl: Label
var equipped_acc2_lbl: Label
var equipped_status_lbl: Label
var loadout_cards_vbox: VBoxContainer
var battle_pouch_vbox: VBoxContainer

# Action Buttons
var btn_melee_strike: Button
var btn_ranged_volley: Button

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

	hero_points_lbl = Label.new()
	hero_points_lbl.text = "Unspent Points: 1"
	hero_points_lbl.add_theme_font_size_override("font_size", 9)
	hero_points_lbl.add_theme_color_override("font_color", Color(0.4, 1.0, 0.5))
	lvl_row.add_child(hero_points_lbl)

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

		var s_val := Label.new()
		s_val.text = "14"
		s_val.add_theme_font_size_override("font_size", 10)
		s_val.add_theme_color_override("font_color", Color.WHITE)
		s_val.mouse_filter = Control.MOUSE_FILTER_PASS
		s_hbox.add_child(s_val)

		var key_str: String = str(sd["k"])
		stat_display_labels[key_str] = s_val
		stat_box_containers[key_str] = stat_box

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

	btn_melee_strike = _add_action_btn(acts_grid, "⚔ Melee Strike", "Execute weapon melee strike against Dummy", _on_test_melee_strike)
	btn_ranged_volley = _add_action_btn(acts_grid, "🏹 Ranged Volley", "Execute ranged attack against Dummy", _on_test_ranged_volley)
	_add_action_btn(acts_grid, "✦ Focus Buff", "Gain +50% ATK Power boost for 2 rounds", _on_test_focus_buff)
	_add_action_btn(acts_grid, "🛡 Shield Defend", "Enter Defensive Stance (50% Mitigation)", _on_test_defend)
	_add_action_btn(acts_grid, "🧪 Pouch Item", "Consume battle pouch item (Heal or Bomb)", _on_test_pouch_item)
	_add_action_btn(acts_grid, "🔄 Stance Shift", "Shift hero stance between Offensive/Neutral", _on_test_stance_shift)

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

	# 1. Update Tabs
	_rebuild_hero_tabs()

	# 2. Update Header
	var arch_name: String = active_hero.stats.archetype.display_name if active_hero.stats and active_hero.stats.archetype else "Hero"
	hero_title_lbl.text = "%s (Archetype: %s)" % [active_hero.name, arch_name]
	hero_lvl_lbl.text = "Level: %d / 10" % (active_hero.stats.level if active_hero.stats else 1)
	var free_pts: int = active_hero.stats.unspent_attribute_points if active_hero.stats else 0
	hero_points_lbl.text = "Unspent Points: %d" % free_pts
	hero_points_lbl.add_theme_color_override("font_color", Color(0.4, 1.0, 0.5) if free_pts > 0 else Color(0.6, 0.7, 0.8))

	# 3. Update 6 Stats Matrix & Tooltips (Items 1 & 2)
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
			if stat_display_labels.has(k):
				var lbl: Variant = stat_display_labels[k]
				if lbl is Label:
					lbl.text = txt
			if stat_box_containers.has(k):
				var box: Variant = stat_box_containers[k]
				if box is PanelContainer:
					box.tooltip_text = _build_stat_breakdown_tooltip(k)

		# 4. Update Derived Combat Metrics (Items 3, 5 & 11)
		var burden: int = active_hero.inventory.get_total_armor_burden() if active_hero.inventory else 0
		var carry_max: int = st.get_carry_weight_capacity(ctx)
		var pwr_melee: int = active_hero.get_effective_power(false)
		var pwr_ranged: int = active_hero.get_effective_power(true)
		var arm_mit: int = active_hero.inventory.get_total_armor_mitigation() if active_hero.inventory else 0
		var shd_mit: int = active_hero.inventory.get_total_shield_mitigation() if active_hero.inventory else 0
		var total_gear_mit: int = active_hero.inventory.get_total_flat_mitigation() if active_hero.inventory else 0
		var str_mit: int = int(roundf(float(st.get_effective_stat("STR", ctx)) * 0.25))
		var total_mit_rating: int = total_gear_mit + str_mit

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

		_update_derived_tooltips(st, ctx, main_w, burden, carry_max, arm_mit, shd_mit, total_gear_mit, str_mit, total_mit_rating)

		# 5. Active Loadout & Quick Pouches (Items 3, 4, 6, 7, 8, 9, 10)
		_render_dojo_loadout_cards(st)

		# 6. Action Button Availabilities based on weapon type (Item 3)
		_update_action_button_states(main_w)

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

func _create_dojo_slot_card(slot_name: String, icon: String, eq: Equipment, _slot_val: Equipment.EquipSlot, st: CharacterStats) -> PanelContainer:
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
					var ratio: float = ratio_raw as float if ratio_raw is float else 0.0
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
	if btn_melee_strike == null or btn_ranged_volley == null:
		return
	if main_w == null:
		btn_melee_strike.disabled = false
		btn_melee_strike.tooltip_text = "Execute unarmed melee strike against Dummy"
		btn_ranged_volley.disabled = true
		btn_ranged_volley.tooltip_text = "Disabled: Ranged Volley requires a ranged weapon (e.g. Bow, Crossbow)"
	elif main_w.is_ranged_weapon():
		btn_melee_strike.disabled = true
		btn_melee_strike.tooltip_text = "Disabled: Cannot perform Melee Strike with ranged weapon (%s)" % main_w.name
		btn_ranged_volley.disabled = false
		btn_ranged_volley.tooltip_text = "Execute ranged attack with %s against Dummy" % main_w.name
	else:
		btn_melee_strike.disabled = false
		btn_melee_strike.tooltip_text = "Execute melee strike with %s against Dummy" % main_w.name
		btn_ranged_volley.disabled = true
		btn_ranged_volley.tooltip_text = "Disabled: Cannot perform Ranged Volley with melee weapon (%s)" % main_w.name

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

func _on_test_melee_strike() -> void:
	if active_hero == null or test_dummy == null:
		return

	var main_w: Equipment = active_hero.inventory.get_main_weapon() if active_hero.inventory else null
	if main_w != null and main_w.is_ranged_weapon():
		_append_formula_log("[color=#f87171]❌ Action Failed: Cannot execute Melee Strike while wielding ranged weapon (%s)![/color]\n" % main_w.name)
		return

	var w_name: String = main_w.name if main_w != null else "Unarmed Strike"
	var base_w_power: int = main_w.base_power if main_w != null else 10
	var eff_power: int = active_hero.get_effective_power(false)

	# Focus buff bonus
	var focus_mult: float = active_hero.attack_boost_multiplier
	var gross_power: int = int(roundf(float(eff_power) * focus_mult))

	# Mitigation calculation
	var mit_ratio: float = test_dummy.defense_mitigation_ratio
	var actual_dmg: int = test_dummy.take_damage(gross_power)

	_trigger_stage_fx(hero_token_ctrl, dummy_token_ctrl, actual_dmg, mit_ratio > 0.0)

	# Live formula logging
	var ctx: Dictionary = {"weapon": main_w.weapon_type} if main_w != null else {}
	_append_formula_log("[color=#fde047][b]⚔ MELEE STRIKE: %s strikes %s with %s![/b][/color]" % [active_hero.name, test_dummy.name, w_name])
	_append_formula_log("  • Weapon Base Power: [color=#e2e8f0]%d[/color]" % base_w_power)
	var str_val: int = active_hero.stats.get_effective_stat("STR", ctx) if active_hero.stats else 10
	var str_scaling_bonus: int = int(roundf(float(str_val) * 1.5))
	_append_formula_log("  • Effective STR: [color=#e2e8f0]%d[/color] (x1.5 scaling -> +%d Power)" % [str_val, str_scaling_bonus])

	if main_w != null:
		var bonus_tier: int = main_w.get_unlocked_threshold_power_boost(active_hero.stats, active_hero.stats.mastery)
		if bonus_tier > 0:
			_append_formula_log("  • [color=#fbbf24]★ Threshold Tier Bonus: +%d Power[/color]" % bonus_tier)
		var pen := main_w.get_wield_penalty(active_hero.stats, active_hero.stats.mastery)
		var is_pen: bool = pen.get("is_penalized", false)
		if is_pen:
			var pen_ratio_raw: Variant = pen.get("power_penalty_ratio", 0.0)
			var pen_ratio: float = pen_ratio_raw as float if pen_ratio_raw is float else 0.0
			var pen_pct: int = int(roundf(pen_ratio * 100.0))
			_append_formula_log("  • [color=#f87171]⚠️ Deficit Penalty: -%d%% Power[/color]" % pen_pct)

	if focus_mult > 1.0:
		_append_formula_log("  • [color=#c084fc]✦ Focus Buff Multiplier: x%.1f[/color]" % focus_mult)

	_append_formula_log("  • Gross Power: [color=#38bdf8]%d[/color] | Dummy Mitigation: [color=#94a3b8]%d%%[/color]" % [gross_power, int(roundf(mit_ratio * 100.0))])
	_append_formula_log("  • [color=#4ade80][b]RESULT: %d Damage Inflicted! (Dummy HP: %d / %d)[/b][/color]\n" % [actual_dmg, test_dummy.hp, test_dummy.max_hp])

	_refresh_all_views()

func _on_test_ranged_volley() -> void:
	if active_hero == null or test_dummy == null:
		return

	var main_w: Equipment = active_hero.inventory.get_main_weapon() if active_hero.inventory else null
	var w_name: String = main_w.name if main_w != null else "Unarmed"
	if main_w == null or not main_w.is_ranged_weapon():
		_append_formula_log("[color=#f87171]❌ Action Failed: Cannot execute Ranged Volley without a ranged weapon! Currently wielding: %s.[/color]\n" % w_name)
		return

	var eff_power: int = active_hero.get_effective_power(true)
	var focus_mult: float = active_hero.attack_boost_multiplier
	var gross_power: int = int(roundf(float(eff_power) * focus_mult))
	var mit_ratio: float = test_dummy.defense_mitigation_ratio
	var actual_dmg: int = test_dummy.take_damage(gross_power)

	_trigger_stage_fx(hero_token_ctrl, dummy_token_ctrl, actual_dmg, mit_ratio > 0.0)

	var ctx: Dictionary = {"weapon": main_w.weapon_type} if main_w != null else {}
	_append_formula_log("[color=#38bdf8][b]🏹 RANGED VOLLEY: %s fires %s at %s![/b][/color]" % [active_hero.name, w_name, test_dummy.name])
	var dex_val: int = active_hero.stats.get_effective_stat("DEX", ctx) if active_hero.stats else 10
	var dex_scaling_bonus: int = int(roundf(float(dex_val) * 1.5))
	_append_formula_log("  • Effective DEX: [color=#e2e8f0]%d[/color] (x1.5 scaling -> +%d Power)" % [dex_val, dex_scaling_bonus])
	_append_formula_log("  • Weapon Type: %s (Range: %d tiles)" % [main_w.weapon_type.capitalize(), main_w.base_range])

	if main_w != null:
		var bonus_tier: int = main_w.get_unlocked_threshold_power_boost(active_hero.stats, active_hero.stats.mastery)
		if bonus_tier > 0:
			_append_formula_log("  • [color=#fbbf24]★ Threshold Tier Bonus: +%d Power[/color]" % bonus_tier)
		var pen := main_w.get_wield_penalty(active_hero.stats, active_hero.stats.mastery)
		var is_pen: bool = pen.get("is_penalized", false)
		if is_pen:
			var pen_ratio_raw_r: Variant = pen.get("power_penalty_ratio", 0.0)
			var pen_ratio_r: float = pen_ratio_raw_r as float if pen_ratio_raw_r is float else 0.0
			var pen_pct: int = int(roundf(pen_ratio_r * 100.0))
			_append_formula_log("  • [color=#f87171]⚠️ Deficit Penalty: -%d%% Power[/color]" % pen_pct)

	_append_formula_log("  • [color=#4ade80][b]RESULT: %d Ranged Damage! (Dummy HP: %d / %d)[/b][/color]\n" % [actual_dmg, test_dummy.hp, test_dummy.max_hp])

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
	if active_hero == null or active_hero.inventory == null:
		return

	if active_hero.inventory.battle_pouch.is_empty():
		_append_formula_log("[color=#f87171]⚠️ Battle pouch is empty. Open 'Manage Gear & Stash' to load consumables.[/color]\n")
		return

	var con: Consumable = active_hero.inventory.battle_pouch[0]
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
# 5. Visual Animation & Floating Combat Text FX
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
		return CombatScenario.load_svg_texture("res://assets/party/sprites/sprite_warrior.svg")
	elif c.name == "Ranger":
		return CombatScenario.load_svg_texture("res://assets/party/sprites/sprite_ranger.svg")
	elif c.name == "Training Dummy":
		return CombatScenario.load_svg_texture("res://assets/enemy/sprites/sprite_dummy.svg")
	return null

# ------------------------------------------------------------------------------
# 6. Headless Verification Test Suite
# ------------------------------------------------------------------------------

func _run_headless_dojo_assertions() -> bool:
	var passed: bool = true

	# Assert 1: Default Hero & Dummy setup
	var w: Combatant = active_hero
	var d: Combatant = test_dummy
	if w == null or d == null or w.name != "Warrior" or d.hp != 500:
		print("  [Dojo-1] Baseline hero and dummy initialization: FAIL")
		passed = false
	else:
		print("  [Dojo-1] Baseline hero and dummy initialization: PASS")

	# Assert 2: Base Attack Calculation
	var initial_power: int = w.get_effective_power(false)
	var dmg: int = d.take_damage(initial_power)
	if dmg <= 0 or d.hp != (500 - dmg):
		print("  [Dojo-2] Base melee attack calculation: FAIL")
		passed = false
	else:
		print("  [Dojo-2] Base melee attack calculation: PASS (%d DMG)" % dmg)

	# Assert 3: Dummy Mitigation Toggle
	d.defense_mitigation_ratio = 0.5
	var half_dmg: int = d.take_damage(initial_power)
	var expected_half: int = maxi(1, int(roundf(float(initial_power) * 0.5)))
	if half_dmg != expected_half:
		print("  [Dojo-3] 50%% mitigation defense barrier calculation: FAIL (%d != %d)" % [half_dmg, expected_half])
		passed = false
	else:
		print("  [Dojo-3] 50%% mitigation defense barrier calculation: PASS (%d DMG)" % half_dmg)

	# Assert 4: Focus Buff Multiplier (x1.5)
	w.apply_attack_boost(1.5, 2)
	var boosted_power: int = int(roundf(float(w.get_effective_power(false)) * 1.5))
	d.defense_mitigation_ratio = 0.0
	var boosted_dmg: int = d.take_damage(boosted_power)
	if boosted_dmg != boosted_power:
		print("  [Dojo-4] Focus buff 1.5x power multiplier calculation: FAIL")
		passed = false
	else:
		print("  [Dojo-4] Focus buff 1.5x power multiplier calculation: PASS (%d DMG)" % boosted_dmg)

	# Assert 5: Claymore Threshold Tier Boost (18 STR -> +4 Power)
	var claymore := ItemCatalog.create_iron_claymore()
	w.inventory.equip(claymore)
	w.sync_from_stats_and_gear()
	var tier_boost: int = claymore.get_unlocked_threshold_power_boost(w.stats, w.stats.mastery)
	if tier_boost != 4:
		print("  [Dojo-5] Claymore threshold tier bonus at 18 STR: FAIL (got %d)" % tier_boost)
		passed = false
	else:
		print("  [Dojo-5] Claymore threshold tier bonus at 18 STR: PASS (+4 Power unlocked)")

	# Assert 6: Titan Warhammer Deficit Penalty
	var titan := ItemCatalog.create_titan_warhammer()
	w.inventory.equip(titan)
	w.sync_from_stats_and_gear()
	var pen := titan.get_wield_penalty(w.stats, w.stats.mastery)
	var is_pen: bool = pen.get("is_penalized", false)
	if not is_pen:
		print("  [Dojo-6] Titan Warhammer deficit penalty check: FAIL")
		passed = false
	else:
		print("  [Dojo-6] Titan Warhammer deficit penalty check: PASS (Deficit penalty applied)")

	# Assert 7: Consumable Pouch Usage
	var pouch_con := ItemCatalog.create_healing_draught()
	w.hp = 50
	pouch_con.apply_to_combatant(w)
	if w.hp != 75:
		print("  [Dojo-7] Consumable healing draught effect: FAIL (HP: %d)" % w.hp)
		passed = false
	else:
		print("  [Dojo-7] Consumable healing draught effect: PASS (HP: 50 -> 75)")

	# Assert 8: Reset Dummy Vitals
	d.hp = d.max_hp
	d.defense_mitigation_ratio = 0.0
	if d.hp != 500 or d.defense_mitigation_ratio != 0.0:
		print("  [Dojo-8] Dummy vitals full reset: FAIL")
		passed = false
	else:
		print("  [Dojo-8] Dummy vitals full reset: PASS (500 / 500 HP)")

	# Assert 9: Weapon Power Melee vs Ranged Gating (Requirement 3)
	var claymore_melee: int = w.get_effective_power(false)
	var claymore_ranged: int = w.get_effective_power(true)
	if claymore_ranged >= claymore_melee:
		print("  [Dojo-9] Melee weapon power gating on ranged attack: FAIL (ranged: %d >= melee: %d)" % [claymore_ranged, claymore_melee])
		passed = false
	else:
		print("  [Dojo-9] Melee weapon power gating on ranged attack: PASS (Melee: %d, Ranged: %d)" % [claymore_melee, claymore_ranged])

	# Assert 10: Ranged Bow Power Gating on Melee Attack (Requirement 3)
	var bow := ItemCatalog.create_oak_shortbow()
	w.inventory.equip(bow)
	w.sync_from_stats_and_gear()
	var bow_melee: int = w.get_effective_power(false)
	var expected_unarmed_melee: int = w.stats.get_melee_power(0, {"weapon": "bow"})
	if bow_melee != expected_unarmed_melee:
		print("  [Dojo-10] Ranged weapon power excluded from melee attack: FAIL (%d != %d)" % [bow_melee, expected_unarmed_melee])
		passed = false
	else:
		print("  [Dojo-10] Ranged weapon power excluded from melee attack: PASS (Bow grants 0 Melee Weapon Power)")

	# Assert 11: Accessory Stat Bonus to Character Stats & Attack Power (Requirement 1)
	w.inventory.unequip(Equipment.EquipSlot.ACCESSORY_1)
	w.sync_from_stats_and_gear()
	var pre_ring_str: int = w.stats.get_effective_stat("STR")
	var pre_ring_hp: int = w.stats.get_max_hp()

	var bear_ring := ItemCatalog.create_ring_of_the_bear() # +2 STR, +2 CONS
	w.inventory.equip(bear_ring, Equipment.EquipSlot.ACCESSORY_1)
	w.sync_from_stats_and_gear()
	var post_ring_str: int = w.stats.get_effective_stat("STR")
	var post_ring_hp: int = w.stats.get_max_hp()
	if post_ring_str != (pre_ring_str + 2) or post_ring_hp <= pre_ring_hp:
		print("  [Dojo-11] Accessory stat bonuses propagated to effective stats: FAIL (STR: %d -> %d, HP: %d -> %d)" % [pre_ring_str, post_ring_str, pre_ring_hp, post_ring_hp])
		passed = false
	else:
		print("  [Dojo-11] Accessory stat bonuses propagated to effective stats: PASS (+2 STR, +2 CONS -> +%d Max HP)" % (post_ring_hp - pre_ring_hp))

	# Assert 12: Class Archetype Gear Proficiency Restrictions (Requirement 5)
	var sword := ItemCatalog.create_steel_broadsword()
	var war_arch: ClassArchetype = w.stats.archetype
	var ranger_arch: ClassArchetype = party_allies[1].stats.archetype if party_allies.size() > 1 else ClassArchetype.create_ranger()
	var war_can_bow: bool = war_arch.can_equip_gear(bow)
	var war_can_sword: bool = war_arch.can_equip_gear(sword)
	var ranger_can_bow: bool = ranger_arch.can_equip_gear(bow)
	var ranger_can_plate: bool = ranger_arch.can_equip_gear(ItemCatalog.create_steel_plate_cuirass())
	var war_can_ring: bool = war_arch.can_equip_gear(bear_ring)
	var ranger_can_ring: bool = ranger_arch.can_equip_gear(bear_ring)
	if war_can_bow != false or war_can_sword != true or ranger_can_bow != true or ranger_can_plate != false or war_can_ring != true or ranger_can_ring != true:
		print("  [Dojo-12] Class archetype gear equip restrictions: FAIL")
		passed = false
	else:
		print("  [Dojo-12] Class archetype gear equip restrictions: PASS (Archetype weapon/armor proficiencies enforced; Accessories unrestricted)")

	return passed
