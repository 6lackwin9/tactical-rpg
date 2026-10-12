class_name CombatGridView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Combat Grid View Component
# Manages the 9x9 tactical combat grid, spatial tile rendering, unit token nodes,
# token selection highlights, hover tooltip cards, combat animations, and floating text.
# ==============================================================================

signal combatant_clicked(combatant: Combatant)
signal combatant_hovered(combatant: Combatant, token: Control)
signal combatant_unhovered()

const TILE_SIZE: int = 54
const GRID_SIZE: int = 9

var grid_frame: PanelContainer
var grid_anchor: Control
var grid_container: GridContainer
var grid_tile_nodes: Dictionary = {}  # Vector2i -> TextureRect
var unit_overlay: Control
var unit_token_nodes: Dictionary = {} # Combatant -> Control

var grid_hover_tooltip: PanelContainer
var hover_name_lbl: Label
var hover_hp_lbl: Label
var hover_status_lbl: Label

func _init() -> void:
	name = "CombatGridView"
	custom_minimum_size = Vector2(496, 480)
	size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	alignment = BoxContainer.ALIGNMENT_CENTER
	add_theme_constant_override("separation", 4)
	_build_ui()

func _build_ui() -> void:
	var grid_header := Label.new()
	grid_header.text = "9x9 COMBAT GRID"
	grid_header.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	grid_header.add_theme_font_size_override("font_size", 11)
	grid_header.add_theme_color_override("font_color", Color(0.8, 0.9, 1.0))
	add_child(grid_header)

	grid_frame = PanelContainer.new()
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
	add_child(grid_frame)

	grid_anchor = Control.new()
	grid_anchor.custom_minimum_size = Vector2(TILE_SIZE * GRID_SIZE, TILE_SIZE * GRID_SIZE)
	grid_frame.add_child(grid_anchor)

	grid_container = GridContainer.new()
	grid_container.columns = GRID_SIZE
	grid_container.add_theme_constant_override("h_separation", 0)
	grid_container.add_theme_constant_override("v_separation", 0)
	grid_anchor.add_child(grid_container)

	unit_overlay = Control.new()
	unit_overlay.name = "UnitOverlay"
	unit_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	grid_anchor.add_child(unit_overlay)

	# Floating Unit Inspector / Hover Tooltip
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

# Builds the 9x9 board from per-tile textures (Vector2i -> Texture2D), optional
# overlays drawn on top (exits, landmarks) and hover tooltips (terrain info).
func setup_tile_textures(textures: Dictionary, overlays: Dictionary = {}, tooltips: Dictionary = {}) -> void:
	for child: Node in grid_container.get_children():
		grid_container.remove_child(child)
		child.queue_free()
	grid_tile_nodes.clear()

	for y: int in range(GRID_SIZE):
		for x: int in range(GRID_SIZE):
			var coord := Vector2i(x, y)
			var tile_rect := TextureRect.new()
			tile_rect.custom_minimum_size = Vector2(TILE_SIZE, TILE_SIZE)
			tile_rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
			tile_rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
			tile_rect.mouse_filter = Control.MOUSE_FILTER_PASS
			var tex_raw: Variant = textures.get(coord, null)
			if tex_raw is Texture2D:
				var tex: Texture2D = tex_raw
				tile_rect.texture = tex
			tile_rect.tooltip_text = str(tooltips.get(coord, ""))

			var overlay_raw: Variant = overlays.get(coord, null)
			if overlay_raw is Texture2D:
				var overlay_tex: Texture2D = overlay_raw
				var overlay := TextureRect.new()
				overlay.texture = overlay_tex
				overlay.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
				overlay.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
				overlay.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
				overlay.mouse_filter = Control.MOUSE_FILTER_IGNORE
				tile_rect.add_child(overlay)

			grid_container.add_child(tile_rect)
			grid_tile_nodes[coord] = tile_rect

func clear_tokens() -> void:
	for child in unit_overlay.get_children():
		child.queue_free()
	unit_token_nodes.clear()

func create_unit_token(combatant: Combatant, texture: Texture2D, grid_pos: Vector2i) -> Control:
	var token_ctrl := Control.new()
	token_ctrl.custom_minimum_size = Vector2(TILE_SIZE, TILE_SIZE)
	token_ctrl.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	var icon := TextureRect.new()
	icon.texture = texture
	icon.custom_minimum_size = Vector2(TILE_SIZE - 6, TILE_SIZE - 6)
	icon.position = Vector2(3, 3)
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
	token_ctrl.add_child(icon)

	var sel_border := ReferenceRect.new()
	sel_border.name = "SelectionBorder"
	sel_border.custom_minimum_size = Vector2(TILE_SIZE, TILE_SIZE)
	sel_border.border_color = Color(1.0, 0.85, 0.25, 0.95)
	sel_border.border_width = 2.0
	sel_border.editor_only = false
	sel_border.visible = false
	sel_border.mouse_filter = Control.MOUSE_FILTER_IGNORE
	token_ctrl.add_child(sel_border)

	token_ctrl.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
			combatant_clicked.emit(combatant)
	)

	token_ctrl.mouse_entered.connect(func() -> void:
		combatant_hovered.emit(combatant, token_ctrl)
	)
	token_ctrl.mouse_exited.connect(func() -> void:
		combatant_unhovered.emit()
	)

	unit_overlay.add_child(token_ctrl)
	unit_token_nodes[combatant] = token_ctrl

	if grid_pos == Vector2i(-1, -1) or not combatant.is_alive() or combatant.is_withdrawn:
		token_ctrl.visible = false
		token_ctrl.modulate.a = 0.0
		token_ctrl.position = Vector2(0, 0)
	else:
		token_ctrl.visible = true
		token_ctrl.modulate.a = 1.0
		token_ctrl.position = Vector2(grid_pos.x * TILE_SIZE, grid_pos.y * TILE_SIZE)

	return token_ctrl

func get_token_for(combatant: Combatant) -> Control:
	if combatant == null or not unit_token_nodes.has(combatant):
		return null
	var node: Variant = unit_token_nodes[combatant]
	if node is Control:
		return node
	return null

func update_selection_highlight(selected_combatant: Combatant) -> void:
	for c: Combatant in unit_token_nodes:
		var token := get_token_for(c)
		if token:
			var border := token.find_child("SelectionBorder", true, false) as ReferenceRect
			if border:
				border.visible = (c == selected_combatant)

func show_hover_card(
	c: Combatant,
	token: Control,
	cur_snap: BlockSnapshot,
	is_ally: bool,
	grid_pos: Vector2i
) -> void:
	if grid_hover_tooltip == null or not is_instance_valid(c):
		return
	var col: Color = Color(0.4, 0.75, 1.0) if is_ally else Color(1.0, 0.45, 0.45)
	hover_name_lbl.text = c.name
	hover_name_lbl.add_theme_color_override("font_color", col)

	var cur_hp: int = cur_snap.hps.get(c, c.hp) if cur_snap != null else c.hp
	var max_hp: int = cur_snap.max_hps.get(c, c.max_hp) if cur_snap != null else c.max_hp
	var is_withdrawn: bool = cur_snap.is_withdrawn.get(c, c.is_withdrawn) if cur_snap != null else c.is_withdrawn
	var is_alive: bool = cur_hp > 0
	var cur_stance: CombatTypes.StanceType = cur_snap.stances.get(c, c.current_stance) if cur_snap != null else c.current_stance
	var pos: Vector2i = cur_snap.positions.get(c, grid_pos) if cur_snap != null else grid_pos

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
	var target_pos: Vector2
	if token != null and token.visible and pos != Vector2i(-1, -1):
		target_pos = token.position + Vector2(TILE_SIZE + 4, -4)
	elif pos != Vector2i(-1, -1):
		target_pos = Vector2(pos.x * TILE_SIZE + TILE_SIZE + 4, pos.y * TILE_SIZE - 4)
	else:
		target_pos = Vector2(8, 8)

	if target_pos.x + 140 > GRID_SIZE * TILE_SIZE:
		target_pos.x = maxf(0.0, target_pos.x - 146)
	if target_pos.y + 55 > GRID_SIZE * TILE_SIZE:
		target_pos.y = maxf(0.0, target_pos.y - 45)
	grid_hover_tooltip.position = target_pos
	grid_hover_tooltip.z_index = 50
	grid_hover_tooltip.visible = true

func hide_hover_card() -> void:
	if grid_hover_tooltip != null:
		grid_hover_tooltip.visible = false

func animate_attacker_lunge(actor: Combatant, target_pos: Vector2) -> void:
	var actor_token := get_token_for(actor)
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

func animate_token_impact(target: Combatant, is_damage: bool, is_mitigated: bool) -> void:
	var target_token := get_token_for(target)
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

func animate_token_pulse(token_actor: Combatant, pulse_col: Color) -> void:
	var token := get_token_for(token_actor)
	if not is_instance_valid(token):
		return
	var tw := create_tween()
	token.modulate = pulse_col
	tw.tween_property(token, "modulate", Color.WHITE, 0.3)

func animate_token_move(token_actor: Combatant, target_coord: Vector2i, duration: float = 0.25) -> void:
	var token := get_token_for(token_actor)
	if not is_instance_valid(token):
		return
	var target_px := Vector2(target_coord.x * TILE_SIZE, target_coord.y * TILE_SIZE)
	var tw := create_tween()
	tw.tween_property(token, "position", target_px, duration).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)

func spawn_floating_combat_text(pos: Vector2, text: String, color: Color) -> void:
	var lbl := Label.new()
	lbl.text = text
	lbl.add_theme_font_size_override("font_size", 10)
	lbl.add_theme_color_override("font_color", color)
	lbl.add_theme_color_override("font_shadow_color", Color(0, 0, 0, 0.9))
	lbl.add_theme_constant_override("shadow_offset_x", 1)
	lbl.add_theme_constant_override("shadow_offset_y", 1)
	lbl.position = pos + Vector2(8, -6)
	lbl.scale = Vector2(1.2, 1.2)
	unit_overlay.add_child(lbl)

	var tw := create_tween()
	tw.set_parallel(true)
	tw.tween_property(lbl, "position", pos + Vector2(8, -28), 0.55).set_trans(Tween.TRANS_CUBIC).set_ease(Tween.EASE_OUT)
	tw.tween_property(lbl, "scale", Vector2(1.0, 1.0), 0.25)
	tw.tween_property(lbl, "modulate:a", 0.0, 0.55).set_delay(0.2)
	tw.chain().tween_callback(lbl.queue_free)
