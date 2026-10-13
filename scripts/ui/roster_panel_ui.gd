class_name RosterPanelUI
extends RefCounted

# ==============================================================================
# Tactical RPG - Roster Panels UI Component
# Top status headers & active combatant rosters for allied heroes (2x2 grid)
# and hostile enemies (5x2 active roster grid).
# ==============================================================================

signal combatant_selected(combatant: Combatant)
signal combatant_hovered(combatant: Combatant, card_node: Control)
signal combatant_unhovered()

var party_panel: PanelContainer
var enemy_panel: PanelContainer
var party_title_lbl: Label
var enemy_title_lbl: Label
var party_cards_container: GridContainer
var enemy_cards_container: GridContainer
var roster_cards: Dictionary = {} # Combatant -> PanelContainer

func _init() -> void:
	_build_party_panel()
	_build_enemy_panel()

func _build_party_panel() -> void:
	party_panel = PanelContainer.new()
	party_panel.name = "PartyPanel"
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
	party_hint.text = "(click a hero to plan)"
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

func _build_enemy_panel() -> void:
	enemy_panel = PanelContainer.new()
	enemy_panel.name = "EnemyPanel"
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

func clear_roster() -> void:
	if party_cards_container:
		for child in party_cards_container.get_children():
			child.queue_free()
	if enemy_cards_container:
		for child in enemy_cards_container.get_children():
			child.queue_free()
	roster_cards.clear()

func add_party_placeholders() -> void:
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

func create_roster_card(
	combatant: Combatant,
	team_color: Color,
	is_ally: bool,
	grid_pos: Vector2i = Vector2i(-1, -1)
) -> PanelContainer:
	var target_container: GridContainer = party_cards_container if is_ally else enemy_cards_container
	if not target_container:
		return null

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
		pos_lbl.text = "%s" % str(grid_pos)
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

	card.mouse_filter = Control.MOUSE_FILTER_STOP
	for child in card.find_children("*", "Control", true, false):
		(child as Control).mouse_filter = Control.MOUSE_FILTER_IGNORE

	card.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
			combatant_selected.emit(combatant)
	)

	card.mouse_entered.connect(func() -> void:
		combatant_hovered.emit(combatant, card)
	)
	card.mouse_exited.connect(func() -> void:
		combatant_unhovered.emit()
	)

	target_container.add_child(card)
	roster_cards[combatant] = card
	return card

func update_all_roster_cards(
	allies: Array[Combatant],
	grid: CombatGrid,
	snap: BlockSnapshot = null
) -> void:
	var alive_enemies: int = 0

	for combatant: Combatant in roster_cards:
		@warning_ignore("unsafe_cast")
		var card := roster_cards[combatant] as PanelContainer
		if not is_instance_valid(card):
			continue

		var is_ally: bool = allies.has(combatant)
		var cur_hp: int = snap.hps.get(combatant, combatant.hp) if snap != null else combatant.hp
		var max_hp: int = snap.max_hps.get(combatant, combatant.max_hp) if snap != null else combatant.max_hp
		var is_withdrawn: bool = snap.is_withdrawn.get(combatant, combatant.is_withdrawn) if snap != null else combatant.is_withdrawn
		var is_alive: bool = (cur_hp > 0)
		var is_act: bool = is_alive and not is_withdrawn
		var cur_stance: CombatTypes.StanceType = snap.stances.get(combatant, combatant.current_stance) if snap != null else combatant.current_stance
		var pos: Vector2i = snap.positions.get(combatant, grid.get_position(combatant) if grid else Vector2i(-1, -1)) if snap != null else (grid.get_position(combatant) if grid else Vector2i(-1, -1))

		# Card visibility (who is on the field) is decided by the owning panel/session.
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
		party_title_lbl.text = "⚔ PARTY MEMBERS (%d/4)" % allies.size()

func update_highlights(selected_combatant: Combatant, allies: Array[Combatant]) -> void:
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
		var is_ally: bool = allies.has(c)
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
