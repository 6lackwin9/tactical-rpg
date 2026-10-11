class_name CharacterEquipmentModal
extends PanelContainer

# ==============================================================================
# Tactical RPG - Character Equipment & Gear Inspection Modal
# Interactive management of 6-stat attributes, horizontal masteries,
# stat-gated gear loadout, requirement checks, and party inventory stash.
# ==============================================================================

signal modal_closed()
signal equipment_updated(combatant: Combatant)
signal stat_allocated(combatant: Combatant, stat_name: String)

# State References
var party_allies: Array[Combatant] = []
var active_hero: Combatant = null
var current_filter: String = "ALL" # ALL, WEAPON, ARMOR, ACCESSORY, CONSUMABLE
var shared_inventory: Inventory = null
var sprite_texture_fn: Callable = Callable()

# Core UI Component Handles
var party_tabs_hbox: HBoxContainer
var avatar_rect: TextureRect
var hero_name_lbl: Label
var hero_class_lbl: Label
var hero_level_lbl: Label
var free_points_lbl: Label
var free_points_hbox: HBoxContainer

# 6 Stats Value Labels
var stat_labels: Dictionary = {}        # String -> Label
var derived_hp_lbl: Label
var derived_speed_lbl: Label
var derived_melee_lbl: Label
var derived_ranged_lbl: Label
var derived_burden_lbl: Label
var derived_range_lbl: Label
var derived_shield_lbl: Label
var masteries_container: VBoxContainer

# Equipped Slots Container
var equipped_slots_vbox: VBoxContainer
var battle_pouch_vbox: VBoxContainer
var burden_progress: ProgressBar
var burden_text_lbl: Label

# Party Stash Container
var stash_items_vbox: VBoxContainer
var filter_btn_group: Dictionary = {} # String -> Button

func _init() -> void:
	name = "CharacterEquipmentModal"
	visible = false
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_STOP
	_build_ui()

func _build_ui() -> void:
	# Full-screen dark translucent backdrop
	var modal_style := StyleBoxFlat.new()
	modal_style.bg_color = Color(0.04, 0.06, 0.09, 0.88)
	add_theme_stylebox_override("panel", modal_style)

	var center_box := CenterContainer.new()
	center_box.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(center_box)

	# Main Dialog Box
	var dialog_card := PanelContainer.new()
	dialog_card.custom_minimum_size = Vector2(1080, 580)
	var dlg_style := StyleBoxFlat.new()
	dlg_style.bg_color = Color(0.08, 0.10, 0.15, 0.98)
	dlg_style.border_color = Color(0.25, 0.40, 0.62, 0.9)
	dlg_style.set_border_width_all(2)
	dlg_style.set_corner_radius_all(8)
	dlg_style.content_margin_left = 14
	dlg_style.content_margin_right = 14
	dlg_style.content_margin_top = 10
	dlg_style.content_margin_bottom = 10
	dialog_card.add_theme_stylebox_override("panel", dlg_style)

	# Compact Tooltip Theme (Requirement 7)
	var modal_theme := Theme.new()
	modal_theme.set_font_size("font_size", "TooltipLabel", 9)
	var tip_style := StyleBoxFlat.new()
	tip_style.bg_color = Color(0.08, 0.11, 0.16, 0.96)
	tip_style.border_color = Color(0.28, 0.38, 0.52, 0.9)
	tip_style.set_border_width_all(1)
	tip_style.set_corner_radius_all(4)
	tip_style.content_margin_left = 6
	tip_style.content_margin_right = 6
	tip_style.content_margin_top = 4
	tip_style.content_margin_bottom = 4
	modal_theme.set_stylebox("panel", "TooltipPanel", tip_style)
	dialog_card.theme = modal_theme

	center_box.add_child(dialog_card)

	var main_vbox := VBoxContainer.new()
	main_vbox.add_theme_constant_override("separation", 8)
	dialog_card.add_child(main_vbox)

	# --------------------------------------------------------------------------
	# HEADER ROW: TITLE & TABS & CLOSE BUTTON
	# --------------------------------------------------------------------------
	var header_hbox := HBoxContainer.new()
	header_hbox.add_theme_constant_override("separation", 10)
	main_vbox.add_child(header_hbox)

	var title_vbox := VBoxContainer.new()
	title_vbox.add_theme_constant_override("separation", 1)
	header_hbox.add_child(title_vbox)

	var title_lbl := Label.new()
	title_lbl.text = "🎒 CHARACTER GEAR & ATTRIBUTE BENCHMARK"
	title_lbl.add_theme_font_size_override("font_size", 14)
	title_lbl.add_theme_color_override("font_color", Color(0.4, 0.85, 1.0))
	title_vbox.add_child(title_lbl)

	var subtitle_lbl := Label.new()
	subtitle_lbl.text = "Equip Weapons, Shields & Armor | Stat Requirements, Threshold Tiers & Masteries"
	subtitle_lbl.add_theme_font_size_override("font_size", 9)
	subtitle_lbl.add_theme_color_override("font_color", Color(0.6, 0.75, 0.85))
	title_vbox.add_child(subtitle_lbl)

	header_hbox.add_spacer(false)

	# Party Switcher Buttons
	party_tabs_hbox = HBoxContainer.new()
	party_tabs_hbox.add_theme_constant_override("separation", 6)
	header_hbox.add_child(party_tabs_hbox)

	var btn_close := Button.new()
	btn_close.text = " ✕ "
	btn_close.tooltip_text = "Close Gear & Equipment Screen"
	btn_close.focus_mode = Control.FOCUS_NONE
	btn_close.pressed.connect(close_modal)
	header_hbox.add_child(btn_close)

	var sep := HSeparator.new()
	main_vbox.add_child(sep)

	# --------------------------------------------------------------------------
	# BODY ROW: 3-COLUMN LAYOUT
	# Col 1: Character Stats & Attributes (320px)
	# Col 2: Equipped Loadout & Battle Pouches (350px)
	# Col 3: Shared Party Stash Browser (Fill)
	# --------------------------------------------------------------------------
	var body_hbox := HBoxContainer.new()
	body_hbox.add_theme_constant_override("separation", 12)
	body_hbox.size_flags_vertical = Control.SIZE_EXPAND_FILL
	main_vbox.add_child(body_hbox)

	# -----------------------------
	# COLUMN 1: CHARACTER STATS & SPECS
	# -----------------------------
	var col1_panel := PanelContainer.new()
	col1_panel.custom_minimum_size = Vector2(320, 0)
	var col1_style := StyleBoxFlat.new()
	col1_style.bg_color = Color(0.06, 0.08, 0.12, 0.85)
	col1_style.border_color = Color(0.18, 0.26, 0.38, 0.7)
	col1_style.set_border_width_all(1)
	col1_style.set_corner_radius_all(6)
	col1_style.content_margin_left = 8
	col1_style.content_margin_right = 8
	col1_style.content_margin_top = 8
	col1_style.content_margin_bottom = 8
	col1_panel.add_theme_stylebox_override("panel", col1_style)
	body_hbox.add_child(col1_panel)

	var col1_scroll := ScrollContainer.new()
	col1_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	col1_panel.add_child(col1_scroll)

	var col1_vbox := VBoxContainer.new()
	col1_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col1_vbox.add_theme_constant_override("separation", 6)
	col1_scroll.add_child(col1_vbox)

	# Hero Identity Header
	var id_hbox := HBoxContainer.new()
	id_hbox.add_theme_constant_override("separation", 8)
	col1_vbox.add_child(id_hbox)

	avatar_rect = TextureRect.new()
	avatar_rect.custom_minimum_size = Vector2(64, 64)
	avatar_rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	avatar_rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	id_hbox.add_child(avatar_rect)

	var id_info := VBoxContainer.new()
	id_info.add_theme_constant_override("separation", 1)
	id_hbox.add_child(id_info)

	hero_name_lbl = Label.new()
	hero_name_lbl.text = "Warrior"
	hero_name_lbl.add_theme_font_size_override("font_size", 13)
	hero_name_lbl.add_theme_color_override("font_color", Color.WHITE)
	id_info.add_child(hero_name_lbl)

	hero_class_lbl = Label.new()
	hero_class_lbl.text = "Archetype: Warrior"
	hero_class_lbl.add_theme_font_size_override("font_size", 9)
	hero_class_lbl.add_theme_color_override("font_color", Color(0.4, 0.75, 1.0))
	id_info.add_child(hero_class_lbl)

	hero_level_lbl = Label.new()
	hero_level_lbl.text = "Level: 2 / 10"
	hero_level_lbl.add_theme_font_size_override("font_size", 9)
	hero_level_lbl.add_theme_color_override("font_color", Color(0.9, 0.8, 0.3))
	id_info.add_child(hero_level_lbl)

	# Free Attribute Points Section
	free_points_hbox = HBoxContainer.new()
	free_points_hbox.add_theme_constant_override("separation", 4)
	col1_vbox.add_child(free_points_hbox)

	free_points_lbl = Label.new()
	free_points_lbl.text = "Unspent Points: 1"
	free_points_lbl.add_theme_font_size_override("font_size", 9)
	free_points_lbl.add_theme_color_override("font_color", Color(0.4, 1.0, 0.5))
	free_points_hbox.add_child(free_points_lbl)

	# 6 Core Stats Cards
	var stats_hdr := Label.new()
	stats_hdr.text = "6 CORE ATTRIBUTES"
	stats_hdr.add_theme_font_size_override("font_size", 10)
	stats_hdr.add_theme_color_override("font_color", Color(0.7, 0.85, 1.0))
	col1_vbox.add_child(stats_hdr)

	var stats_grid := GridContainer.new()
	stats_grid.columns = 2
	stats_grid.add_theme_constant_override("h_separation", 6)
	stats_grid.add_theme_constant_override("v_separation", 4)
	col1_vbox.add_child(stats_grid)

	var stat_keys: Array[Dictionary] = [
		{"key": "STR", "name": "STR (Active)", "color": Color(0.95, 0.4, 0.4), "tip": "Melee Power & Armor Burden Mitigation"},
		{"key": "CONS", "name": "CONS (Passive)", "color": Color(0.95, 0.65, 0.3), "tip": "Max HP & Travel Endurance"},
		{"key": "DEX", "name": "DEX (Active)", "color": Color(0.4, 0.9, 0.5), "tip": "Initiative Speed & Ranged Power"},
		{"key": "PERC", "name": "PERC (Passive)", "color": Color(0.4, 0.8, 0.95), "tip": "Secret Spotting & Intent Unmasking"},
		{"key": "INT", "name": "INT (Active)", "color": Color(0.75, 0.5, 0.95), "tip": "Tactical/Skill Power & Buff Potency"},
		{"key": "KNOW", "name": "KNOW (Passive)", "color": Color(0.5, 0.7, 1.0), "tip": "Lore Dialogue & Danger Reduction"}
	]

	for s: Dictionary in stat_keys:
		var s_key: String = str(s.get("key", ""))
		var s_name: String = str(s.get("name", ""))
		var s_col: Color = s.get("color", Color.WHITE)
		var s_tip: String = str(s.get("tip", ""))
		var stat_card := _create_stat_badge(s_key, s_name, s_col, s_tip)
		stats_grid.add_child(stat_card)

	# Derived Combat Metrics
	var derived_hdr := Label.new()
	derived_hdr.text = "DERIVED COMBAT SPECS"
	derived_hdr.add_theme_font_size_override("font_size", 10)
	derived_hdr.add_theme_color_override("font_color", Color(0.7, 0.85, 1.0))
	col1_vbox.add_child(derived_hdr)

	var derived_vbox := VBoxContainer.new()
	derived_vbox.add_theme_constant_override("separation", 2)
	col1_vbox.add_child(derived_vbox)

	derived_hp_lbl = _create_derived_spec_row(derived_vbox, "Max Health", "100 HP", Color(0.9, 0.4, 0.4))
	derived_speed_lbl = _create_derived_spec_row(derived_vbox, "Initiative Speed", "15 Speed", Color(0.4, 0.9, 0.5))
	derived_melee_lbl = _create_derived_spec_row(derived_vbox, "Melee Power", "25 ATK", Color(0.95, 0.5, 0.4))
	derived_ranged_lbl = _create_derived_spec_row(derived_vbox, "Ranged Power", "18 ATK", Color(0.4, 0.8, 0.95))
	derived_range_lbl = _create_derived_spec_row(derived_vbox, "Attack Range", "1 Tile", Color(0.8, 0.8, 0.8))
	derived_shield_lbl = _create_derived_spec_row(derived_vbox, "Shield Mitigation", "+0 Flat", Color(0.4, 0.7, 1.0))
	derived_burden_lbl = _create_derived_spec_row(derived_vbox, "Armor Burden", "0 / 14", Color(0.9, 0.75, 0.3))

	# Horizontal Masteries
	var masteries_hdr := Label.new()
	masteries_hdr.text = "HORIZONTAL MASTERIES"
	masteries_hdr.add_theme_font_size_override("font_size", 10)
	masteries_hdr.add_theme_color_override("font_color", Color(0.7, 0.85, 1.0))
	col1_vbox.add_child(masteries_hdr)

	masteries_container = VBoxContainer.new()
	masteries_container.add_theme_constant_override("separation", 2)
	col1_vbox.add_child(masteries_container)

	# -----------------------------
	# COLUMN 2: EQUIPPED LOADOUT & BATTLE POUCHES
	# -----------------------------
	var col2_panel := PanelContainer.new()
	col2_panel.custom_minimum_size = Vector2(340, 0)
	var col2_style := StyleBoxFlat.new()
	col2_style.bg_color = Color(0.06, 0.08, 0.12, 0.85)
	col2_style.border_color = Color(0.18, 0.26, 0.38, 0.7)
	col2_style.set_border_width_all(1)
	col2_style.set_corner_radius_all(6)
	col2_style.content_margin_left = 8
	col2_style.content_margin_right = 8
	col2_style.content_margin_top = 8
	col2_style.content_margin_bottom = 8
	col2_panel.add_theme_stylebox_override("panel", col2_style)
	body_hbox.add_child(col2_panel)

	var col2_scroll := ScrollContainer.new()
	col2_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	col2_panel.add_child(col2_scroll)

	var col2_vbox := VBoxContainer.new()
	col2_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col2_vbox.add_theme_constant_override("separation", 6)
	col2_scroll.add_child(col2_vbox)

	var equip_hdr := Label.new()
	equip_hdr.text = "EQUIPPED GEAR LOADOUT"
	equip_hdr.add_theme_font_size_override("font_size", 10)
	equip_hdr.add_theme_color_override("font_color", Color(0.7, 0.85, 1.0))
	col2_vbox.add_child(equip_hdr)

	# Armor Burden Meter
	var burden_card := PanelContainer.new()
	var b_style := StyleBoxFlat.new()
	b_style.bg_color = Color(0.11, 0.14, 0.20, 0.7)
	b_style.set_corner_radius_all(4)
	b_style.content_margin_left = 6
	b_style.content_margin_right = 6
	b_style.content_margin_top = 4
	b_style.content_margin_bottom = 4
	burden_card.add_theme_stylebox_override("panel", b_style)
	col2_vbox.add_child(burden_card)

	var b_vbox := VBoxContainer.new()
	b_vbox.add_theme_constant_override("separation", 2)
	burden_card.add_child(b_vbox)

	var b_row := HBoxContainer.new()
	b_vbox.add_child(b_row)

	var b_tag := Label.new()
	b_tag.text = "⚖ Armor Burden:"
	b_tag.add_theme_font_size_override("font_size", 9)
	b_tag.add_theme_color_override("font_color", Color(0.9, 0.8, 0.4))
	b_row.add_child(b_tag)
	b_row.add_spacer(false)

	burden_text_lbl = Label.new()
	burden_text_lbl.text = "0 / 14 (No Penalty)"
	burden_text_lbl.add_theme_font_size_override("font_size", 9)
	burden_text_lbl.add_theme_color_override("font_color", Color.WHITE)
	b_row.add_child(burden_text_lbl)

	burden_progress = ProgressBar.new()
	burden_progress.custom_minimum_size = Vector2(0, 6)
	burden_progress.show_percentage = false
	var bp_bg := StyleBoxFlat.new()
	bp_bg.bg_color = Color(0.15, 0.18, 0.25, 0.9)
	bp_bg.set_corner_radius_all(3)
	var bp_fill := StyleBoxFlat.new()
	bp_fill.bg_color = Color(0.95, 0.75, 0.2, 1.0)
	bp_fill.set_corner_radius_all(3)
	burden_progress.add_theme_stylebox_override("background", bp_bg)
	burden_progress.add_theme_stylebox_override("fill", bp_fill)
	b_vbox.add_child(burden_progress)

	equipped_slots_vbox = VBoxContainer.new()
	equipped_slots_vbox.add_theme_constant_override("separation", 4)
	col2_vbox.add_child(equipped_slots_vbox)

	var pouch_sep := HSeparator.new()
	col2_vbox.add_child(pouch_sep)

	var pouch_hdr := Label.new()
	pouch_hdr.text = "COMBAT BATTLE POUCH (3 SLOTS)"
	pouch_hdr.add_theme_font_size_override("font_size", 10)
	pouch_hdr.add_theme_color_override("font_color", Color(0.4, 0.85, 0.6))
	col2_vbox.add_child(pouch_hdr)

	battle_pouch_vbox = VBoxContainer.new()
	battle_pouch_vbox.add_theme_constant_override("separation", 3)
	col2_vbox.add_child(battle_pouch_vbox)

	# -----------------------------
	# COLUMN 3: SHARED PARTY STASH & ITEM BROWSER
	# -----------------------------
	var col3_panel := PanelContainer.new()
	col3_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var col3_style := StyleBoxFlat.new()
	col3_style.bg_color = Color(0.06, 0.08, 0.12, 0.85)
	col3_style.border_color = Color(0.18, 0.26, 0.38, 0.7)
	col3_style.set_border_width_all(1)
	col3_style.set_corner_radius_all(6)
	col3_style.content_margin_left = 8
	col3_style.content_margin_right = 8
	col3_style.content_margin_top = 8
	col3_style.content_margin_bottom = 8
	col3_panel.add_theme_stylebox_override("panel", col3_style)
	body_hbox.add_child(col3_panel)

	var col3_vbox := VBoxContainer.new()
	col3_vbox.add_theme_constant_override("separation", 6)
	col3_panel.add_child(col3_vbox)

	var stash_top_row := HBoxContainer.new()
	stash_top_row.add_theme_constant_override("separation", 6)
	col3_vbox.add_child(stash_top_row)

	var stash_hdr := Label.new()
	stash_hdr.text = "PARTY STASH INVENTORY"
	stash_hdr.add_theme_font_size_override("font_size", 10)
	stash_hdr.add_theme_color_override("font_color", Color(0.7, 0.85, 1.0))
	stash_top_row.add_child(stash_hdr)

	stash_top_row.add_spacer(false)

	# Filter Buttons
	var filters: Array[String] = ["ALL", "WEAPON", "ARMOR", "ACCESSORY", "CONSUMABLE"]
	for f in filters:
		var fb := Button.new()
		fb.text = f
		fb.add_theme_font_size_override("font_size", 9)
		fb.focus_mode = Control.FOCUS_NONE
		fb.pressed.connect(func() -> void: _set_stash_filter(f))
		stash_top_row.add_child(fb)
		filter_btn_group[f] = fb

	var stash_scroll := ScrollContainer.new()
	stash_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col3_vbox.add_child(stash_scroll)

	stash_items_vbox = VBoxContainer.new()
	stash_items_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	stash_items_vbox.add_theme_constant_override("separation", 4)
	stash_scroll.add_child(stash_items_vbox)

# ------------------------------------------------------------------------------
# Helpers for Column 1 (Stat Badges & Derived Rows)
# ------------------------------------------------------------------------------

func _create_stat_badge(key: String, label_text: String, col: Color, tooltip: String) -> PanelContainer:
	var card := PanelContainer.new()
	card.custom_minimum_size = Vector2(144, 44)
	card.tooltip_text = tooltip
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.10, 0.13, 0.19, 0.85)
	style.border_color = Color(col.r * 0.7, col.g * 0.7, col.b * 0.7, 0.6)
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.content_margin_left = 6
	style.content_margin_right = 6
	style.content_margin_top = 4
	style.content_margin_bottom = 4
	card.add_theme_stylebox_override("panel", style)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 1)
	card.add_child(vbox)

	var row1 := HBoxContainer.new()
	vbox.add_child(row1)

	var name_lbl := Label.new()
	name_lbl.text = label_text
	name_lbl.add_theme_font_size_override("font_size", 8)
	name_lbl.add_theme_color_override("font_color", col)
	row1.add_child(name_lbl)

	row1.add_spacer(false)

	var btn_add := Button.new()
	btn_add.text = "+"
	btn_add.custom_minimum_size = Vector2(18, 16)
	btn_add.add_theme_font_size_override("font_size", 9)
	btn_add.tooltip_text = "Allocate 1 free attribute point to %s" % key
	btn_add.focus_mode = Control.FOCUS_NONE
	btn_add.pressed.connect(func() -> void: _on_allocate_stat(key))
	row1.add_child(btn_add)

	var val_lbl := Label.new()
	val_lbl.text = "14"
	val_lbl.add_theme_font_size_override("font_size", 12)
	val_lbl.add_theme_color_override("font_color", Color.WHITE)
	vbox.add_child(val_lbl)

	stat_labels[key] = val_lbl
	return card

func _create_derived_spec_row(parent: Control, tag: String, default_val: String, col: Color) -> Label:
	var row := HBoxContainer.new()
	parent.add_child(row)

	var tag_lbl := Label.new()
	tag_lbl.text = "• " + tag + ":"
	tag_lbl.add_theme_font_size_override("font_size", 9)
	tag_lbl.add_theme_color_override("font_color", col)
	row.add_child(tag_lbl)

	row.add_spacer(false)

	var val_lbl := Label.new()
	val_lbl.text = default_val
	val_lbl.add_theme_font_size_override("font_size", 9)
	val_lbl.add_theme_color_override("font_color", Color.WHITE)
	row.add_child(val_lbl)

	return val_lbl

# ------------------------------------------------------------------------------
# Modal Lifecycle & Data Binding
# ------------------------------------------------------------------------------

func setup_heroes(allies: Array[Combatant], shared_stash: Inventory = null) -> void:
	party_allies = allies
	shared_inventory = shared_stash
	if not party_allies.is_empty():
		select_hero(party_allies[0])
	_rebuild_hero_tabs()

func select_hero(hero: Combatant) -> void:
	if hero == null:
		return
	active_hero = hero
	_rebuild_hero_tabs()
	refresh_display()

func open_modal(hero: Combatant = null) -> void:
	if hero != null:
		active_hero = hero
	elif active_hero == null and not party_allies.is_empty():
		active_hero = party_allies[0]

	visible = true
	_rebuild_hero_tabs()
	refresh_display()

func close_modal() -> void:
	visible = false
	modal_closed.emit()

func _set_stash_filter(filter_name: String) -> void:
	current_filter = filter_name
	for f: String in filter_btn_group:
		var btn_raw: Variant = filter_btn_group[f]
		if btn_raw is Button:
			var btn: Button = btn_raw
			if f == current_filter:
				btn.add_theme_color_override("font_color", Color(0.4, 0.9, 1.0))
			else:
				btn.remove_theme_color_override("font_color")
	_render_stash_items()

func _rebuild_hero_tabs() -> void:
	for c in party_tabs_hbox.get_children():
		c.queue_free()

	for ally in party_allies:
		var btn := Button.new()
		var is_active: bool = (ally == active_hero)
		var lvl: int = ally.stats.level if ally.stats != null else 1
		btn.text = "%s (Lv %d)" % [ally.name, lvl]
		btn.add_theme_font_size_override("font_size", 10)
		btn.focus_mode = Control.FOCUS_NONE
		if is_active:
			btn.add_theme_color_override("font_color", Color(0.4, 0.95, 0.6))
		btn.pressed.connect(func() -> void: select_hero(ally))
		party_tabs_hbox.add_child(btn)

func refresh_display() -> void:
	if active_hero == null:
		return

	# Synchronize combatant model from specs and equipment
	active_hero.sync_from_stats_and_gear()

	# 1. Update Identity & Avatar
	hero_name_lbl.text = active_hero.name
	var arch_name: String = active_hero.stats.archetype.display_name if active_hero.stats and active_hero.stats.archetype else "Adventurer"
	hero_class_lbl.text = "Archetype: %s" % arch_name
	var cur_lvl: int = active_hero.stats.level if active_hero.stats else 1
	hero_level_lbl.text = "Level: %d / %d" % [cur_lvl, CharacterStats.MAX_LEVEL]

	if sprite_texture_fn.is_valid():
		var tex_variant: Variant = sprite_texture_fn.call(active_hero)
		if tex_variant is Texture2D:
			avatar_rect.texture = tex_variant

	var free_pts: int = active_hero.stats.unspent_attribute_points if active_hero.stats else 0
	free_points_lbl.text = "Unspent Points: %d" % free_pts
	free_points_lbl.add_theme_color_override("font_color", Color(0.4, 1.0, 0.5) if free_pts > 0 else Color(0.6, 0.7, 0.8))

	# 2. Update 6 Stats Values
	if active_hero.stats != null:
		var st := active_hero.stats
		var main_wpn: Equipment = active_hero.inventory.get_main_weapon() if active_hero.inventory else null
		var ctx: Dictionary = {"weapon": main_wpn.weapon_type} if main_wpn != null else {}

		for k: String in ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"]:
			var eff: int = st.get_effective_stat(k, ctx)
			var base_val: int = st.get_base_stat(k)
			var diff: int = eff - base_val
			var diff_str: String = ""
			if diff > 0:
				diff_str = " (+%d)" % diff
			elif diff < 0:
				diff_str = " (%d)" % diff
			if stat_labels.has(k):
				var lbl: Variant = stat_labels[k]
				if lbl is Label:
					lbl.text = "%d%s" % [eff, diff_str]

		# 3. Derived Metrics
		var burden: int = active_hero.inventory.get_total_armor_burden() if active_hero.inventory else 0
		var max_carry: int = st.get_carry_weight_capacity()
		var shield_mit: int = active_hero.inventory.get_total_shield_mitigation() if active_hero.inventory else 0

		derived_hp_lbl.text = "%d HP" % st.get_max_hp()
		derived_speed_lbl.text = "%d Speed (Burden -%d)" % [st.get_initiative_speed(burden), burden]
		derived_melee_lbl.text = "%d Power" % active_hero.get_effective_power(false)
		derived_ranged_lbl.text = "%d Power" % active_hero.get_effective_power(true)
		derived_range_lbl.text = "%d Tile%s" % [active_hero.attack_range, "s" if active_hero.attack_range > 1 else ""]
		derived_shield_lbl.text = "+%d Flat Mit" % shield_mit
		derived_burden_lbl.text = "%d / %d Burden" % [burden, max_carry]

		# Armor Burden Meter Update
		burden_progress.max_value = maxi(1, max_carry)
		burden_progress.value = burden
		if burden > 0:
			burden_text_lbl.text = "%d / %d (-%d Speed Penalty)" % [burden, max_carry, burden]
			burden_text_lbl.add_theme_color_override("font_color", Color(1.0, 0.7, 0.3))
		else:
			burden_text_lbl.text = "%d / %d (No Burden Penalty)" % [burden, max_carry]
			burden_text_lbl.add_theme_color_override("font_color", Color(0.4, 0.9, 0.5))

		# Masteries
		_render_masteries_list(st.mastery)

	# 4. Render Equipped Slots & Battle Pouches
	_render_equipped_slots()
	_render_battle_pouches()

	# 5. Render Stash
	_render_stash_items()

func _render_masteries_list(m: MasterySystem) -> void:
	for c in masteries_container.get_children():
		c.queue_free()

	if m == null:
		return

	for w_type: String in m.weapon_masteries:
		var raw_lvl: Variant = m.weapon_masteries[w_type]
		var lvl: int = raw_lvl if raw_lvl is int else 0
		if lvl > 0:
			var discount: int = m.get_weapon_requirement_discount(w_type)
			var row := HBoxContainer.new()
			var tag := Label.new()
			tag.text = "⚔ %s Mastery: Lv %d" % [w_type.capitalize(), lvl]
			tag.add_theme_font_size_override("font_size", 8)
			tag.add_theme_color_override("font_color", Color(0.9, 0.8, 0.4))
			row.add_child(tag)
			row.add_spacer(false)
			var disc_lbl := Label.new()
			disc_lbl.text = "(-%d REQ discount)" % discount
			disc_lbl.add_theme_font_size_override("font_size", 8)
			disc_lbl.add_theme_color_override("font_color", Color(0.4, 0.9, 0.6))
			row.add_child(disc_lbl)
			masteries_container.add_child(row)

# ------------------------------------------------------------------------------
# Column 2: Equipped Slot Renderers
# ------------------------------------------------------------------------------

func _render_equipped_slots() -> void:
	for c in equipped_slots_vbox.get_children():
		c.queue_free()

	if active_hero == null or active_hero.inventory == null:
		return

	var inv: Inventory = active_hero.inventory
	var slot_defs: Array[Dictionary] = [
		{"slot": Equipment.EquipSlot.MAIN_HAND, "name": "⚔ Main Hand Weapon", "icon": "⚔"},
		{"slot": Equipment.EquipSlot.OFF_HAND, "name": "🛡 Off Hand / Shield", "icon": "🛡"},
		{"slot": Equipment.EquipSlot.BODY, "name": "🥋 Body Armor", "icon": "🥋"},
		{"slot": Equipment.EquipSlot.ACCESSORY_1, "name": "💍 Accessory 1", "icon": "💍"},
		{"slot": Equipment.EquipSlot.ACCESSORY_2, "name": "💍 Accessory 2", "icon": "💍"}
	]

	for def: Dictionary in slot_defs:
		var slot_raw: Variant = def.get("slot", Equipment.EquipSlot.MAIN_HAND)
		var slot_val: Equipment.EquipSlot = slot_raw if slot_raw is Equipment.EquipSlot else Equipment.EquipSlot.MAIN_HAND
		var eq: Equipment = inv.get_equipped(slot_val)
		var s_name: String = str(def.get("name", ""))
		var s_icon: String = str(def.get("icon", ""))
		var card := _create_slot_card(s_name, s_icon, eq, slot_val)
		equipped_slots_vbox.add_child(card)

func _create_slot_card(slot_name: String, _icon: String, eq: Equipment, slot_val: Equipment.EquipSlot) -> PanelContainer:
	var card := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.10, 0.13, 0.19, 0.8)
	style.border_color = Color(0.22, 0.32, 0.46, 0.7)
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.content_margin_left = 6
	style.content_margin_right = 6
	style.content_margin_top = 4
	style.content_margin_bottom = 4
	card.add_theme_stylebox_override("panel", style)

	var vbox := VBoxContainer.new()
	vbox.add_theme_constant_override("separation", 2)
	card.add_child(vbox)

	var row1 := HBoxContainer.new()
	vbox.add_child(row1)

	var title_lbl := Label.new()
	title_lbl.text = slot_name
	title_lbl.add_theme_font_size_override("font_size", 9)
	title_lbl.add_theme_color_override("font_color", Color(0.65, 0.8, 1.0))
	row1.add_child(title_lbl)

	row1.add_spacer(false)

	if eq != null:
		var btn_unequip := Button.new()
		btn_unequip.text = "Unequip"
		btn_unequip.add_theme_font_size_override("font_size", 8)
		btn_unequip.focus_mode = Control.FOCUS_NONE
		btn_unequip.pressed.connect(func() -> void: _unequip_item(slot_val))
		row1.add_child(btn_unequip)

	var row2 := HBoxContainer.new()
	vbox.add_child(row2)

	if eq == null:
		var empty_lbl := Label.new()
		empty_lbl.text = "— Empty Slot —"
		empty_lbl.add_theme_font_size_override("font_size", 9)
		empty_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		row2.add_child(empty_lbl)
	else:
		var name_lbl := Label.new()
		name_lbl.text = eq.name
		name_lbl.add_theme_font_size_override("font_size", 10)
		name_lbl.add_theme_color_override("font_color", _get_rarity_color(eq.rarity))
		row2.add_child(name_lbl)

		row2.add_spacer(false)

		var stats_desc := ""
		if eq.base_power > 0:
			stats_desc += "Pwr: %d " % eq.base_power
		if eq.mitigation > 0:
			stats_desc += "Mit: %d " % eq.mitigation
		if eq.burden > 0:
			stats_desc += "Brd: %d " % eq.burden

		var stats_lbl := Label.new()
		stats_lbl.text = stats_desc
		stats_lbl.add_theme_font_size_override("font_size", 8)
		stats_lbl.add_theme_color_override("font_color", Color(0.8, 0.9, 0.9))
		row2.add_child(stats_lbl)

		# Threshold / Deficit Status Line
		var status_text := _get_equipment_status_text(eq)
		if not status_text.is_empty():
			var status_lbl := Label.new()
			status_lbl.text = status_text
			status_lbl.add_theme_font_size_override("font_size", 8)
			if "TIER" in status_text:
				status_lbl.add_theme_color_override("font_color", Color(1.0, 0.85, 0.3))
			elif "DEFICIT" in status_text or "PENALTY" in status_text:
				status_lbl.add_theme_color_override("font_color", Color(1.0, 0.4, 0.4))
			else:
				status_lbl.add_theme_color_override("font_color", Color(0.4, 0.9, 0.5))
			vbox.add_child(status_lbl)

	return card

func _render_battle_pouches() -> void:
	for c in battle_pouch_vbox.get_children():
		c.queue_free()

	if active_hero == null or active_hero.inventory == null:
		return

	var pouch: Array[Consumable] = active_hero.inventory.battle_pouch
	for i in range(3):
		var con: Consumable = pouch[i] if i < pouch.size() else null
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)

		var slot_tag := Label.new()
		slot_tag.text = "Pouch Slot %d:" % (i + 1)
		slot_tag.add_theme_font_size_override("font_size", 9)
		slot_tag.add_theme_color_override("font_color", Color(0.5, 0.7, 0.9))
		row.add_child(slot_tag)

		if con == null:
			var empty := Label.new()
			empty.text = "— Empty Quick-Slot —"
			empty.add_theme_font_size_override("font_size", 8)
			empty.add_theme_color_override("font_color", Color(0.4, 0.5, 0.6))
			row.add_child(empty)
		else:
			var c_name := Label.new()
			c_name.text = "🧪 " + con.name
			c_name.add_theme_font_size_override("font_size", 9)
			c_name.add_theme_color_override("font_color", Color(0.4, 0.9, 0.6))
			row.add_child(c_name)

			row.add_spacer(false)

			var btn_rem := Button.new()
			btn_rem.text = "Remove"
			btn_rem.add_theme_font_size_override("font_size", 8)
			btn_rem.focus_mode = Control.FOCUS_NONE
			btn_rem.pressed.connect(func() -> void:
				active_hero.inventory.remove_from_battle_pouch(con)
				if shared_inventory != null:
					shared_inventory.add_item(con)
				refresh_display()
			)
			row.add_child(btn_rem)

		battle_pouch_vbox.add_child(row)

# ------------------------------------------------------------------------------
# Column 3: Party Stash Item Browser
# ------------------------------------------------------------------------------

func _render_stash_items() -> void:
	for c in stash_items_vbox.get_children():
		c.queue_free()

	# Use shared inventory or active hero's bag
	var inv: Inventory = shared_inventory if shared_inventory != null else (active_hero.inventory if active_hero != null else null)
	if inv == null:
		return

	var visible_items: Array[Item] = []
	for it in inv.items:
		if _matches_filter(it):
			visible_items.append(it)

	if visible_items.is_empty():
		var empty_lbl := Label.new()
		empty_lbl.text = "No items match current filter '%s'." % current_filter
		empty_lbl.add_theme_font_size_override("font_size", 10)
		empty_lbl.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
		stash_items_vbox.add_child(empty_lbl)
		return

	for item in visible_items:
		var item_card := _create_stash_item_card(item)
		stash_items_vbox.add_child(item_card)

func _matches_filter(it: Item) -> bool:
	if current_filter == "ALL":
		return true
	if current_filter == "WEAPON" and it.item_type == Item.ItemType.WEAPON:
		return true
	if current_filter == "ARMOR" and (it.item_type == Item.ItemType.ARMOR or it.item_type == Item.ItemType.SHIELD):
		return true
	if current_filter == "ACCESSORY" and it.item_type == Item.ItemType.ACCESSORY:
		return true
	if current_filter == "CONSUMABLE" and it.item_type == Item.ItemType.CONSUMABLE:
		return true
	return false

func _create_stash_item_card(item: Item) -> PanelContainer:
	var card := PanelContainer.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.10, 0.13, 0.19, 0.75)
	style.border_color = Color(0.2, 0.3, 0.44, 0.6)
	style.set_border_width_all(1)
	style.set_corner_radius_all(4)
	style.content_margin_left = 8
	style.content_margin_right = 8
	style.content_margin_top = 4
	style.content_margin_bottom = 4
	card.add_theme_stylebox_override("panel", style)

	var hbox := HBoxContainer.new()
	hbox.add_theme_constant_override("separation", 8)
	card.add_child(hbox)

	var info_vbox := VBoxContainer.new()
	info_vbox.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	info_vbox.add_theme_constant_override("separation", 2)
	hbox.add_child(info_vbox)

	var row1 := HBoxContainer.new()
	info_vbox.add_child(row1)

	var name_lbl := Label.new()
	name_lbl.text = item.name
	name_lbl.add_theme_font_size_override("font_size", 10)
	name_lbl.add_theme_color_override("font_color", _get_rarity_color(item.rarity))
	row1.add_child(name_lbl)

	row1.add_spacer(false)

	var type_lbl := Label.new()
	type_lbl.text = Item.type_to_string(item.item_type)
	type_lbl.add_theme_font_size_override("font_size", 8)
	type_lbl.add_theme_color_override("font_color", Color(0.6, 0.7, 0.8))
	row1.add_child(type_lbl)

	var row2 := HBoxContainer.new()
	info_vbox.add_child(row2)

	var stats_info := ""
	if item is Equipment:
		var eq := item as Equipment
		if eq.base_power > 0:
			stats_info += "Power: %d  " % eq.base_power
		if eq.mitigation > 0:
			stats_info += "Mitigation: %d  " % eq.mitigation
		if eq.burden > 0:
			stats_info += "Burden: %d  " % eq.burden
		if eq.base_range > 1:
			stats_info += "Range: %d  " % eq.base_range
	elif item is Consumable:
		var con := item as Consumable
		if con.heal_amount > 0:
			stats_info = "Heals %d HP" % con.heal_amount
		elif con.damage_amount > 0:
			stats_info = "Deals %d Damage" % con.damage_amount
		else:
			stats_info = "Buffs %s (+%d%%)" % [con.buff_stat, int(roundf(con.buff_percent * 100.0))]

	var stats_lbl := Label.new()
	stats_lbl.text = stats_info
	stats_lbl.add_theme_font_size_override("font_size", 8)
	stats_lbl.add_theme_color_override("font_color", Color(0.8, 0.85, 0.9))
	row2.add_child(stats_lbl)

	# Requirements badge & Action Buttons
	if item is Equipment:
		var eq := item as Equipment
		var can_class_equip: bool = true
		if active_hero != null and active_hero.stats != null and active_hero.stats.archetype != null:
			can_class_equip = active_hero.stats.archetype.can_equip_gear(eq)

		if not can_class_equip:
			var rest_lbl := Label.new()
			var arch_name: String = active_hero.stats.archetype.display_name if active_hero.stats.archetype else "Hero"
			rest_lbl.text = "⚠️ Class Restricted (%s cannot equip %s)" % [arch_name, eq.weapon_type.capitalize()]
			rest_lbl.add_theme_font_size_override("font_size", 8)
			rest_lbl.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
			info_vbox.add_child(rest_lbl)

			var btn_equip := Button.new()
			btn_equip.text = "Equip"
			btn_equip.custom_minimum_size = Vector2(60, 26)
			btn_equip.add_theme_font_size_override("font_size", 9)
			btn_equip.disabled = true
			btn_equip.tooltip_text = "Restricted: %s cannot equip %s." % [arch_name, eq.weapon_type]
			btn_equip.focus_mode = Control.FOCUS_NONE
			hbox.add_child(btn_equip)
		else:
			var req_text := _get_equipment_requirement_badge(eq)
			var req_lbl := Label.new()
			req_lbl.text = req_text
			req_lbl.add_theme_font_size_override("font_size", 8)
			if "TIER" in req_text:
				req_lbl.add_theme_color_override("font_color", Color(1.0, 0.85, 0.3))
			elif "DEFICIT" in req_text:
				req_lbl.add_theme_color_override("font_color", Color(1.0, 0.4, 0.4))
			else:
				req_lbl.add_theme_color_override("font_color", Color(0.4, 0.9, 0.5))
			info_vbox.add_child(req_lbl)

			var btn_equip := Button.new()
			btn_equip.text = "Equip"
			btn_equip.custom_minimum_size = Vector2(60, 26)
			btn_equip.add_theme_font_size_override("font_size", 9)
			btn_equip.focus_mode = Control.FOCUS_NONE
			btn_equip.pressed.connect(func() -> void: _equip_item(eq))
			hbox.add_child(btn_equip)

	elif item is Consumable:
		var con := item as Consumable
		var btn_pouch := Button.new()
		btn_pouch.text = "+ Pouch"
		btn_pouch.custom_minimum_size = Vector2(60, 26)
		btn_pouch.add_theme_font_size_override("font_size", 9)
		btn_pouch.focus_mode = Control.FOCUS_NONE
		btn_pouch.pressed.connect(func() -> void: _add_to_pouch(con))
		hbox.add_child(btn_pouch)

	return card

# ------------------------------------------------------------------------------
# Equipment Interaction Actions
# ------------------------------------------------------------------------------

func _equip_item(eq: Equipment) -> void:
	if active_hero == null or active_hero.inventory == null or eq == null:
		return

	if active_hero.stats != null and active_hero.stats.archetype != null:
		if not active_hero.stats.archetype.can_equip_gear(eq):
			return

	# Remove from shared stash if shared
	if shared_inventory != null and shared_inventory.items.has(eq):
		shared_inventory.remove_item(eq)

	var prev: Equipment = active_hero.inventory.equip(eq)
	if prev != null and shared_inventory != null:
		shared_inventory.add_item(prev)

	active_hero.sync_from_stats_and_gear()
	equipment_updated.emit(active_hero)
	refresh_display()

func _unequip_item(slot_val: Equipment.EquipSlot) -> void:
	if active_hero == null or active_hero.inventory == null:
		return

	var eq: Equipment = active_hero.inventory.unequip(slot_val)
	if eq != null and shared_inventory != null:
		shared_inventory.add_item(eq)

	active_hero.sync_from_stats_and_gear()
	equipment_updated.emit(active_hero)
	refresh_display()

func _add_to_pouch(con: Consumable) -> void:
	if active_hero == null or active_hero.inventory == null or con == null:
		return

	if active_hero.inventory.battle_pouch.size() >= 3:
		print("Battle pouch is full (3/3 slots occupied).")
		return

	if shared_inventory != null and shared_inventory.items.has(con):
		shared_inventory.remove_item(con)

	active_hero.inventory.add_to_battle_pouch(con)
	refresh_display()

func _on_allocate_stat(stat_name: String) -> void:
	if active_hero == null or active_hero.stats == null:
		return

	if active_hero.stats.allocate_attribute_point(stat_name):
		active_hero.sync_from_stats_and_gear()
		stat_allocated.emit(active_hero, stat_name)
		refresh_display()
	else:
		print("No free attribute points available.")

# ------------------------------------------------------------------------------
# Formatting & Requirement Text Utilities
# ------------------------------------------------------------------------------

func _get_equipment_requirement_badge(eq: Equipment) -> String:
	if active_hero == null or active_hero.stats == null or eq == null:
		return ""

	var st := active_hero.stats
	var m := st.mastery
	var summary: Dictionary = eq.get_requirement_status_summary(st, m)
	var req_disp: String = str(summary.get("req_display", ""))
	var status: String = str(summary.get("status_tag", ""))
	return "%s  %s" % [req_disp, status]

func _get_equipment_status_text(eq: Equipment) -> String:
	if active_hero == null or active_hero.stats == null or eq == null:
		return ""

	var st := active_hero.stats
	var m := st.mastery
	var summary: Dictionary = eq.get_requirement_status_summary(st, m)
	return str(summary.get("status_tag", ""))

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
