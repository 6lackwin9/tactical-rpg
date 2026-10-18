class_name CombatGridView
extends VBoxContainer

# ==============================================================================
# Tactical RPG - Combat Grid View Component
# Manages the 12x9 tactical combat grid, spatial tile rendering, unit token nodes,
# token selection highlights (the unit hover card lives in CombatGridPanel). Combat animation and floating text
# live in CombatPresenter (S14-L2), which draws its temporary nodes on fx_layer.
# ==============================================================================

signal combatant_clicked(combatant: Combatant)
signal combatant_hovered(combatant: Combatant, token: Control)
signal combatant_unhovered()
signal tile_clicked(coord: Vector2i)
signal tile_hovered(coord: Vector2i)
signal tile_unhovered()
signal grid_right_clicked()      # anywhere on the board (planning: undo the last step)
signal ghost_clicked(combatant: Combatant)   # a planned-move shadow

const TILE_SIZE: int = 54   # the default tile; the combat screen fits `tile_px` to the window (CombatHudMetrics)
const FRAME_PX: int = 4
const GRID_W: int = CombatGrid.GRID_W
const GRID_H: int = CombatGrid.GRID_H

var tile_px: int = TILE_SIZE
var grid_frame: PanelContainer
var grid_anchor: Control
var grid_container: GridContainer
var grid_tile_nodes: Dictionary = {}  # Vector2i -> TextureRect
var unit_overlay: Control
var intent_overlay: IntentOverlay   # INTENT1-U: enemy kind chips and arrows (planning only), above the tokens
var fx_layer: Control   # CombatPresenter's temporary nodes (labels, status pops); clear_tokens leaves it alone
var plan_overlay: PlanOverlay
var ghost_layer: Control
var ghost_nodes: Dictionary = {}  # Combatant -> Control (planned-move shadows)
var unit_token_nodes: Dictionary = {} # Combatant -> Control

func _init() -> void:
	name = "CombatGridView"
	custom_minimum_size = Vector2(tile_px * GRID_W + FRAME_PX, tile_px * GRID_H + FRAME_PX)
	size_flags_horizontal = Control.SIZE_SHRINK_CENTER
	alignment = BoxContainer.ALIGNMENT_CENTER
	add_theme_constant_override("separation", 4)
	_build_ui()

func _build_ui() -> void:
	grid_frame = PanelContainer.new()
	grid_frame.custom_minimum_size = Vector2(tile_px * GRID_W + FRAME_PX, tile_px * GRID_H + FRAME_PX)
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
	grid_anchor.custom_minimum_size = Vector2(tile_px * GRID_W, tile_px * GRID_H)
	grid_frame.add_child(grid_anchor)

	grid_container = GridContainer.new()
	grid_container.columns = GRID_W
	grid_container.add_theme_constant_override("h_separation", 0)
	grid_container.add_theme_constant_override("v_separation", 0)
	grid_anchor.add_child(grid_container)

	# Planning ghosts / paths sit above the tiles and below the unit tokens
	plan_overlay = PlanOverlay.new()
	grid_anchor.add_child(plan_overlay)

	unit_overlay = Control.new()
	unit_overlay.name = "UnitOverlay"
	unit_overlay.set_anchors_preset(Control.PRESET_FULL_RECT)
	unit_overlay.mouse_filter = Control.MOUSE_FILTER_IGNORE   # tokens (children) take clicks; empty tiles underneath must too
	grid_anchor.add_child(unit_overlay)

	intent_overlay = IntentOverlay.new()
	grid_anchor.add_child(intent_overlay)

	fx_layer = Control.new()
	fx_layer.name = "FxLayer"
	fx_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	fx_layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	grid_anchor.add_child(fx_layer)

	# Planned-move shadows sit above the tokens so a shadow on an occupied tile stays clickable
	ghost_layer = Control.new()
	ghost_layer.name = "GhostLayer"
	ghost_layer.set_anchors_preset(Control.PRESET_FULL_RECT)
	ghost_layer.mouse_filter = Control.MOUSE_FILTER_IGNORE
	grid_anchor.add_child(ghost_layer)

# Resizes the board to `px` per tile (COMBAT-UI2: the biggest tile the window allows): tiles, frame, tokens and
# ghosts. Token positions follow the grid cells they stand on; the owner redraws overlays and FX afterwards.
func set_tile_px(px: int) -> void:
	if px == tile_px:
		return
	tile_px = px
	var board := Vector2(px * GRID_W, px * GRID_H)
	custom_minimum_size = board + Vector2(FRAME_PX, FRAME_PX)
	grid_frame.custom_minimum_size = board + Vector2(FRAME_PX, FRAME_PX)
	grid_anchor.custom_minimum_size = board
	plan_overlay.tile = px
	plan_overlay.queue_redraw()
	intent_overlay.set_tile(px)
	for child: Node in grid_container.get_children():
		var tile_rect: Control = child as Control
		if tile_rect != null:
			tile_rect.custom_minimum_size = Vector2(px, px)
	for token: Variant in unit_token_nodes.values():
		var ctrl: Control = token as Control
		if ctrl == null:
			continue
		ctrl.custom_minimum_size = Vector2(px, px)
		ctrl.size = Vector2(px, px)
		for part: Node in ctrl.get_children():
			var part_ctrl: Control = part as Control
			if part_ctrl == null:
				continue
			if part_ctrl is TextureRect:
				part_ctrl.custom_minimum_size = Vector2(px - 6, px - 6)
				part_ctrl.size = Vector2(px - 6, px - 6)
			else:
				part_ctrl.custom_minimum_size = Vector2(px, px)
				part_ctrl.size = Vector2(px, px)

# Builds the 12x9 board from per-tile textures (Vector2i -> Texture2D), optional
# overlays drawn on top (exits, landmarks) and hover tooltips (terrain info).
func setup_tile_textures(textures: Dictionary, overlays: Dictionary = {}, tooltips: Dictionary = {}) -> void:
	for child: Node in grid_container.get_children():
		grid_container.remove_child(child)
		child.queue_free()
	grid_tile_nodes.clear()

	for y: int in range(GRID_H):
		for x: int in range(GRID_W):
			var coord := Vector2i(x, y)
			var tile_rect := TextureRect.new()
			tile_rect.custom_minimum_size = Vector2(tile_px, tile_px)
			tile_rect.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
			tile_rect.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
			tile_rect.mouse_filter = Control.MOUSE_FILTER_PASS
			var tex_raw: Variant = textures.get(coord, null)
			if tex_raw is Texture2D:
				var tex: Texture2D = tex_raw
				tile_rect.texture = tex
			tile_rect.tooltip_text = str(tooltips.get(coord, ""))
			tile_rect.gui_input.connect(func(event: InputEvent) -> void:
				var mouse_event := event as InputEventMouseButton
				if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
					tile_clicked.emit(coord)
				elif mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_RIGHT:
					grid_right_clicked.emit()
			)
			tile_rect.mouse_entered.connect(func() -> void: tile_hovered.emit(coord))
			tile_rect.mouse_exited.connect(func() -> void: tile_unhovered.emit())

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
	for ghost: Node in ghost_layer.get_children():
		ghost.queue_free()
	ghost_nodes.clear()

func create_unit_token(combatant: Combatant, texture: Texture2D, grid_pos: Vector2i) -> Control:
	var token_ctrl := Control.new()
	token_ctrl.custom_minimum_size = Vector2(tile_px, tile_px)
	token_ctrl.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	var icon := TextureRect.new()
	icon.texture = texture
	icon.custom_minimum_size = Vector2(tile_px - 6, tile_px - 6)
	icon.position = Vector2(3, 3)
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
	token_ctrl.add_child(icon)

	var sel_border := ReferenceRect.new()
	sel_border.name = "SelectionBorder"
	sel_border.custom_minimum_size = Vector2(tile_px, tile_px)
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
		elif mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_RIGHT:
			grid_right_clicked.emit()
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
		token_ctrl.position = Vector2(grid_pos.x * tile_px, grid_pos.y * tile_px)

	return token_ctrl

# Shows a shadow per planned move: `ghosts` holds {"unit": Combatant, "cell": Vector2i,
# "texture": Texture2D, "small": bool}. Small shadows (their tile is still occupied)
# sit in the tile's top-right corner so the unit standing there stays visible.
func update_ghosts(ghosts: Array[Dictionary]) -> void:
	var keep: Dictionary = {}
	for g: Dictionary in ghosts:
		var unit: Combatant = g["unit"]
		var cell: Vector2i = g["cell"]
		var small: bool = DictUtil.to_bool(g.get("small", false))
		keep[unit] = true
		var node: Control = _ghost_for(unit, g.get("texture", null))
		var size_px: float = 28.0 if small else float(tile_px - 8)
		node.size = Vector2(size_px, size_px)
		node.position = Vector2(cell * tile_px) + (Vector2(tile_px - size_px - 2, 2) if small else Vector2(4, 4))
		node.visible = true
	for unit_key: Variant in ghost_nodes:
		if not keep.has(unit_key):
			var stale: Control = ghost_nodes[unit_key]
			stale.visible = false

func get_ghost_for(combatant: Combatant) -> Control:
	var raw: Variant = ghost_nodes.get(combatant, null)
	return raw if raw is Control else null

func _ghost_for(unit: Combatant, tex_raw: Variant) -> Control:
	var existing: Control = get_ghost_for(unit)
	if existing != null:
		return existing
	var node := Control.new()
	node.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	node.tooltip_text = "%s's planned spot (click for self actions)" % unit.name
	var icon := TextureRect.new()
	icon.set_anchors_preset(Control.PRESET_FULL_RECT)
	icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
	icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
	icon.modulate = Color(1, 1, 1, 0.6)
	icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
	if tex_raw is Texture2D:
		var tex: Texture2D = tex_raw
		icon.texture = tex
	node.add_child(icon)
	var frame := ReferenceRect.new()
	frame.set_anchors_preset(Control.PRESET_FULL_RECT)
	frame.border_color = Color(0.4, 0.9, 1.0, 0.95)
	frame.border_width = 1.5
	frame.editor_only = false
	frame.mouse_filter = Control.MOUSE_FILTER_IGNORE
	node.add_child(frame)
	node.gui_input.connect(func(event: InputEvent) -> void:
		var mouse_event := event as InputEventMouseButton
		if mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_LEFT:
			ghost_clicked.emit(unit)
		elif mouse_event != null and mouse_event.pressed and mouse_event.button_index == MOUSE_BUTTON_RIGHT:
			grid_right_clicked.emit()
	)
	ghost_layer.add_child(node)
	ghost_nodes[unit] = node
	return node

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
