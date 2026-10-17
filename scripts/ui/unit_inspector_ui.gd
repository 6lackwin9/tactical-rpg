class_name UnitInspectorUI
extends PanelContainer

# ==============================================================================
# Tactical RPG - Unit Inspector & Action Setup Dock
# Left-side tactical inspector displaying unit portrait, vitals, stats,
# and hostile AI threat intel.
# ==============================================================================

signal gear_modal_requested(combatant: Combatant)

var inspector_hdr: Label
var btn_gear_modal: Button
var inspector_avatar: TextureRect
var inspector_name_lbl: Label
var inspector_team_lbl: Label
var inspector_hp_bar: ProgressBar
var inspector_hp_lbl: Label
var inspector_ap_bar: ProgressBar
var inspector_ap_lbl: Label
var inspector_stance_lbl: Label
var inspector_speed_lbl: Label
var inspector_range_lbl: Label
var inspector_pos_lbl: Label
var inspector_target_lbl: Label
var inspector_buffs_lbl: Label
var inspector_gear_lbl: Label
var ally_actions_container: VBoxContainer
var enemy_actions_container: VBoxContainer
var left_hint: Label

var current_inspected_combatant: Combatant = null

func _init() -> void:
	name = "UnitInspectorUI"
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	custom_minimum_size = Vector2(340, 468)

	var left_style := StyleBoxFlat.new()
	left_style.bg_color = Color(0.08, 0.10, 0.15, 0.95)
	left_style.border_color = Color(0.2, 0.28, 0.42, 0.8)
	left_style.set_border_width_all(1)
	left_style.set_corner_radius_all(6)
	left_style.content_margin_left = 8
	left_style.content_margin_right = 8
	left_style.content_margin_top = 6
	left_style.content_margin_bottom = 6
	add_theme_stylebox_override("panel", left_style)

	_build_ui()

func _build_ui() -> void:
	var left_vbox := VBoxContainer.new()
	left_vbox.add_theme_constant_override("separation", 5)
	add_child(left_vbox)

	var top_hdr_row := HBoxContainer.new()
	left_vbox.add_child(top_hdr_row)

	inspector_hdr = Label.new()
	inspector_hdr.text = "UNIT INSPECTOR & ACTION SETUP"
	inspector_hdr.add_theme_font_size_override("font_size", 10)
	inspector_hdr.add_theme_color_override("font_color", Color(0.65, 0.8, 1.0))
	top_hdr_row.add_child(inspector_hdr)

	top_hdr_row.add_spacer(false)

	btn_gear_modal = Button.new()
	btn_gear_modal.text = "🎒 GEAR"
	btn_gear_modal.add_theme_font_size_override("font_size", 9)
	btn_gear_modal.tooltip_text = "Open Character Gear & Attribute Screen"
	btn_gear_modal.focus_mode = Control.FOCUS_NONE
	btn_gear_modal.pressed.connect(func() -> void:
		if current_inspected_combatant != null:
			gear_modal_requested.emit(current_inspected_combatant)
	)
	top_hdr_row.add_child(btn_gear_modal)

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

	# AP Gauge (the round's action points; debt shows as a negative value)
	var ap_gauge_vbox := VBoxContainer.new()
	ap_gauge_vbox.add_theme_constant_override("separation", 1)
	left_block.add_child(ap_gauge_vbox)

	var ap_hdr_row := HBoxContainer.new()
	ap_gauge_vbox.add_child(ap_hdr_row)
	var ap_tag := Label.new()
	ap_tag.text = "AP"
	ap_tag.add_theme_font_size_override("font_size", 9)
	ap_tag.add_theme_color_override("font_color", Color(0.3, 0.75, 1.0))
	ap_hdr_row.add_child(ap_tag)
	ap_hdr_row.add_spacer(false)
	inspector_ap_lbl = Label.new()
	inspector_ap_lbl.text = "12/12"
	inspector_ap_lbl.add_theme_font_size_override("font_size", 9)
	inspector_ap_lbl.add_theme_color_override("font_color", Color(0.7, 0.85, 1.0))
	ap_hdr_row.add_child(inspector_ap_lbl)

	inspector_ap_bar = ProgressBar.new()
	inspector_ap_bar.custom_minimum_size = Vector2(110, 8)
	inspector_ap_bar.show_percentage = false
	inspector_ap_bar.max_value = 12
	inspector_ap_bar.value = 12
	var ap_bg := StyleBoxFlat.new()
	ap_bg.bg_color = Color(0.08, 0.12, 0.18, 0.9)
	ap_bg.set_corner_radius_all(3)
	var ap_fill := StyleBoxFlat.new()
	ap_fill.bg_color = Color(0.2, 0.6, 0.95, 1.0)
	ap_fill.set_corner_radius_all(3)
	inspector_ap_bar.add_theme_stylebox_override("background", ap_bg)
	inspector_ap_bar.add_theme_stylebox_override("fill", ap_fill)
	hp_gauge_vbox.add_child(inspector_ap_bar)

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
	inspector_range_lbl.text = "Reach: 1 tile"
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
	inspector_buffs_lbl.text = "Effects: None active"
	inspector_buffs_lbl.add_theme_font_size_override("font_size", 9)
	inspector_buffs_lbl.add_theme_color_override("font_color", Color(0.7, 0.9, 0.7))
	right_block.add_child(inspector_buffs_lbl)

	inspector_gear_lbl = Label.new()
	inspector_gear_lbl.text = "Gear: Default Loadout"
	inspector_gear_lbl.add_theme_font_size_override("font_size", 9)
	inspector_gear_lbl.add_theme_color_override("font_color", Color(0.9, 0.8, 0.4))
	right_block.add_child(inspector_gear_lbl)

	# Allies: planning happens in the Plan Round screen; this container stays for layout symmetry
	ally_actions_container = VBoxContainer.new()
	left_vbox.add_child(ally_actions_container)

	# Enemy Possible Actions Deck (Secret Intent Pool)
	enemy_actions_container = VBoxContainer.new()
	enemy_actions_container.add_theme_constant_override("separation", 3)
	enemy_actions_container.visible = false
	left_vbox.add_child(enemy_actions_container)

	var enemy_deck_hdr := Label.new()
	enemy_deck_hdr.text = "THREAT INTEL (AI BEHAVIOUR)"
	enemy_deck_hdr.add_theme_font_size_override("font_size", 10)
	enemy_deck_hdr.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
	enemy_actions_container.add_child(enemy_deck_hdr)

	left_hint = Label.new()
	left_hint.text = "💡 Planning: click an enemy, yourself or a tile on the grid."
	left_hint.add_theme_font_size_override("font_size", 8)
	left_hint.add_theme_color_override("font_color", Color(0.5, 0.6, 0.7))
	left_vbox.add_child(left_hint)

func update_inspector(
	selected_combatant: Combatant,
	is_ally: bool,
	pos_text: String,
	sprite_texture: Texture2D,
	snap: BlockSnapshot = null,
	intel: Array[Dictionary] = []
) -> void:
	if not selected_combatant:
		return

	var c: Combatant = selected_combatant
	var cur_hp: int = c.hp
	var max_hp: int = c.max_hp
	var stance: CombatTypes.StanceType = c.current_stance
	var target_val: Variant = c.active_target
	var boost_rds: int = c.attack_boost_rounds_remaining
	var withdrawn: bool = c.is_withdrawn
	var boost_mult: float = c.attack_boost_multiplier
	var cur_ap: int = c.ap
	if snap != null:
		cur_ap = DictUtil.int_of(snap.aps, c, cur_ap)
		boost_mult = DictUtil.float_of(snap.attack_boost_mult, c, boost_mult)
		var raw_hp: Variant = snap.hps.get(c, cur_hp)
		cur_hp = raw_hp if raw_hp is int else cur_hp
		var raw_max: Variant = snap.max_hps.get(c, max_hp)
		max_hp = raw_max if raw_max is int else max_hp
		var raw_stance: Variant = snap.stances.get(c, stance)
		stance = raw_stance if raw_stance is CombatTypes.StanceType else stance
		target_val = snap.targets.get(c, target_val)
		var raw_rds: Variant = snap.attack_boost_rds.get(c, boost_rds)
		boost_rds = raw_rds if raw_rds is int else boost_rds
		var raw_withdrawn: Variant = snap.is_withdrawn.get(c, withdrawn)
		withdrawn = raw_withdrawn if raw_withdrawn is bool else withdrawn
	var alive: bool = cur_hp > 0

	current_inspected_combatant = c
	if btn_gear_modal:
		btn_gear_modal.visible = is_ally

	inspector_name_lbl.text = c.name

	if is_ally:
		inspector_hdr.text = "UNIT INSPECTOR & ACTION SETUP"
		inspector_team_lbl.text = "Allied Hero (Party Member)"
		inspector_team_lbl.add_theme_color_override("font_color", Color(0.4, 0.75, 1.0))
		inspector_avatar.texture = sprite_texture
		ally_actions_container.visible = alive
		enemy_actions_container.visible = false
		if not alive:
			left_hint.text = "💀 Unit is defeated. Future commands cancelled."
	else:
		inspector_hdr.text = "ENEMY INTEL & BEHAVIOUR"
		inspector_team_lbl.text = "Hostile Enemy Unit"
		inspector_team_lbl.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
		inspector_avatar.texture = sprite_texture
		ally_actions_container.visible = false
		enemy_actions_container.visible = alive
		_rebuild_enemy_threat_dock(intel)
		if not alive:
			left_hint.text = "💀 Enemy hostile defeated."
		elif withdrawn:
			left_hint.text = "💨 Enemy safely withdrew through escape boundary."
		else:
			left_hint.text = "💡 Hostiles plan in secret; their commands are revealed as blocks resolve."

	inspector_hp_bar.max_value = max_hp
	inspector_hp_bar.value = cur_hp
	inspector_hp_lbl.text = "%d/%d" % [cur_hp, max_hp]

	var hp_ratio: float = float(cur_hp) / float(maxi(1, max_hp))
	var hp_fill := StyleBoxFlat.new()
	hp_fill.set_corner_radius_all(3)
	if hp_ratio > 0.5:
		hp_fill.bg_color = Color(0.25, 0.75, 0.35, 1.0)
	elif hp_ratio > 0.25:
		hp_fill.bg_color = Color(0.9, 0.75, 0.2, 1.0)
	else:
		hp_fill.bg_color = Color(0.9, 0.25, 0.25, 1.0)
	inspector_hp_bar.add_theme_stylebox_override("fill", hp_fill)

	if inspector_ap_bar and inspector_ap_lbl:
		inspector_ap_bar.max_value = maxi(1, c.max_ap)
		inspector_ap_bar.value = maxi(0, cur_ap)
		inspector_ap_lbl.text = ("%d/%d" % [cur_ap, c.max_ap]) + (" (debt)" if cur_ap < 0 else "")

	inspector_stance_lbl.text = "Stance: " + CombatTypes.stance_to_string(stance)
	inspector_speed_lbl.text = "Speed: %d" % c.speed
	var band: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(c)
	inspector_range_lbl.text = ("Reach: %d tile" % band.y) if band.x == band.y else ("Reach: %d-%d tiles" % [band.x, band.y])
	inspector_pos_lbl.text = pos_text

	if target_val is Combatant:
		var tgt: Combatant = target_val
		inspector_target_lbl.text = "Target: " + tgt.name
	else:
		inspector_target_lbl.text = "Target: None"

	var effects: Array[String] = []
	if boost_rds > 0:
		effects.append("Focus x%.2f ATK (%d rds)" % [boost_mult, boost_rds])
	for status: Variant in c.statuses:
		effects.append(StatusEffects.chip_text(c, str(status)))
	inspector_buffs_lbl.text = "Effects: " + (", ".join(effects) if not effects.is_empty() else "None active")

	if inspector_gear_lbl:
		if c.inventory != null:
			var wpn: Equipment = c.inventory.get_main_weapon()
			if wpn != null:
				inspector_gear_lbl.text = "Gear: " + wpn.name
			else:
				inspector_gear_lbl.text = "Gear: Unarmed"
		else:
			inspector_gear_lbl.text = "Gear: None"

func _rebuild_enemy_threat_dock(intel: Array[Dictionary]) -> void:
	for child in enemy_actions_container.get_children():
		if child is PanelContainer:
			child.queue_free()

	var enemy_actions: Array[Dictionary] = intel.duplicate()
	if enemy_actions.is_empty():
		enemy_actions = [{"name": "⚔ Attack", "desc": "Basic Strike | Cost: 2 Blocks", "cost": 2, "color": Color(0.9, 0.35, 0.35)}]

	for act in enemy_actions:
		var card := _create_mini_action_card(act)
		enemy_actions_container.add_child(card)

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
