class_name UnitInspectorUI
extends PanelContainer

# ==============================================================================
# Tactical RPG - Unit Inspector & Action Setup Dock
# Left-side tactical inspector displaying unit portrait, vitals, stats,
# player action palette, and hostile threat intent decks.
# ==============================================================================

signal gear_modal_requested(combatant: Combatant)

var inspector_hdr: Label
var btn_gear_modal: Button
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

	inspector_gear_lbl = Label.new()
	inspector_gear_lbl.text = "Gear: Default Loadout"
	inspector_gear_lbl.add_theme_font_size_override("font_size", 9)
	inspector_gear_lbl.add_theme_color_override("font_color", Color(0.9, 0.8, 0.4))
	right_block.add_child(inspector_gear_lbl)

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

# `snap` (optional) renders the unit as it was at a replayed block; null = live state.
func update_inspector(
	selected_combatant: Combatant,
	is_ally: bool,
	pos_text: String,
	sprite_texture: Texture2D,
	snap: BlockSnapshot = null
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
	if snap != null:
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
			left_hint.text = "💡 Actions for selected hero line up on continuous timeline below."
	else:
		inspector_hdr.text = "ENEMY INTEL & POSSIBLE ACTIONS"
		inspector_team_lbl.text = "Hostile Enemy Unit"
		inspector_team_lbl.add_theme_color_override("font_color", Color(1.0, 0.45, 0.45))
		inspector_avatar.texture = sprite_texture
		ally_actions_container.visible = false
		enemy_actions_container.visible = alive
		_rebuild_enemy_threat_dock(c)
		if not alive:
			left_hint.text = "💀 Enemy hostile defeated."
		elif withdrawn:
			left_hint.text = "💨 Enemy safely withdrew through escape boundary."
		else:
			left_hint.text = "💡 In actual combat, enemy turns resolve simultaneously in secret."

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

	if inspector_sp_bar and inspector_sp_lbl:
		var cur_sp: int = 50 if alive else 0
		inspector_sp_bar.max_value = 50
		inspector_sp_bar.value = cur_sp
		inspector_sp_lbl.text = "%d/50" % cur_sp

	inspector_stance_lbl.text = "Stance: " + CombatTypes.stance_to_string(stance)
	inspector_speed_lbl.text = "Speed: %d" % c.speed
	inspector_range_lbl.text = "Range: %d tile%s" % [c.attack_range, "s" if c.attack_range > 1 else ""]
	inspector_pos_lbl.text = pos_text

	if target_val is Combatant:
		var tgt: Combatant = target_val
		inspector_target_lbl.text = "Target: " + tgt.name
	else:
		inspector_target_lbl.text = "Target: None"

	if boost_rds > 0:
		inspector_buffs_lbl.text = "Buffs: Focus (+50%% ATK, %d rds)" % boost_rds
	else:
		inspector_buffs_lbl.text = "Buffs: None active"

	if inspector_gear_lbl:
		if c.inventory != null:
			var wpn: Equipment = c.inventory.get_main_weapon()
			if wpn != null:
				inspector_gear_lbl.text = "Gear: " + wpn.name
			else:
				inspector_gear_lbl.text = "Gear: Unarmed"
		else:
			inspector_gear_lbl.text = "Gear: None"

func _rebuild_enemy_threat_dock(selected_combatant: Combatant) -> void:
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
