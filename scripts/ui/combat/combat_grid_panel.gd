class_name CombatGridPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Combat Grid Panel
# 9x9 battlefield. Tokens are created for the session's own combatants (one
# source of truth), placed from the displayed state (live or snapshot), and
# animated by CombatPresenter from session.fx_requested. Hover cards follow session.hover_changed so
# hovering a roster card also highlights the unit here.
# ==============================================================================

const TILE_SIZE: int = 54

var grid_view: CombatGridView
var popup: PlanActionPopup
var presenter: CombatPresenter

var _popup_key: Variant = null   # what the open popup is for: a Combatant or a Vector2i tile
var _hint_option: PlanOption = null
var _last_head: Vector2i = Vector2i(-1, -1)   # (viewed round, playhead) at the last refresh: stepping back kills live FX

func _init() -> void:
	super._init()
	panel_id = "grid"
	panel_title = "Combat Grid"
	panel_icon = "🗺"
	default_min_size = Vector2(496, 480)
	grid_view = CombatGridView.new()
	add_child(grid_view)
	popup = PlanActionPopup.new()
	add_child(popup)
	presenter = CombatPresenter.new()
	presenter.overlay = grid_view.fx_layer
	add_child(presenter)

func _on_bound() -> void:
	session.mounted.connect(_mount)
	session.fx_requested.connect(_on_fx)
	session.log_reset.connect(presenter.kill_all)
	session.hover_changed.connect(_on_hover_changed)
	grid_view.combatant_clicked.connect(_on_token_clicked)
	grid_view.combatant_hovered.connect(_on_token_hovered)
	grid_view.combatant_unhovered.connect(_on_token_unhovered)
	grid_view.tile_clicked.connect(_on_tile_clicked)
	grid_view.tile_hovered.connect(_on_tile_hovered)
	grid_view.tile_unhovered.connect(_on_tile_unhovered)
	grid_view.ghost_clicked.connect(_on_ghost_clicked)
	grid_view.grid_right_clicked.connect(func() -> void:
		if popup.visible:
			popup.close()
		elif _planning_ready():
			session.plan_undo_last()
	)
	session.plan_preview_changed.connect(_refresh_overlay)
	popup.option_hovered.connect(func(step: PlanStep) -> void: session.set_plan_preview(step))
	popup.option_chosen.connect(_on_option_chosen)
	popup.closed.connect(func() -> void:
		_popup_key = null
		session.set_plan_preview(null)
	)
	_mount()

func _mount() -> void:
	presenter.kill_all()
	_last_head = Vector2i(-1, -1)
	grid_view.clear_tokens()
	if session == null or session.grid == null:
		return
	var enc: CombatEncounter = session.encounter
	var grid: CombatGrid = session.grid
	var textures: Dictionary = {}
	var overlays: Dictionary = {}
	var tooltips: Dictionary = {}
	for y: int in range(CombatGrid.GRID_SIZE):
		for x: int in range(CombatGrid.GRID_SIZE):
			var coord := Vector2i(x, y)
			var terrain_type: Terrain.Type = grid.get_terrain(coord)
			textures[coord] = TileArt.tile_at(terrain_type, coord)
			var tip: String = "%s  %s" % [str(coord), Terrain.describe(terrain_type)]
			if grid.is_exit_tile(coord):
				overlays[coord] = TileArt.overlay("exit")
				tip += " • EXIT (withdraw here)"
			elif coord == CombatGrid.CENTER:
				overlays[coord] = TileArt.overlay("center")
				tip += " • Center ring"
			tooltips[coord] = tip
	grid_view.setup_tile_textures(textures, overlays, tooltips)
	for c: Combatant in session.allies:
		grid_view.create_unit_token(c, enc.get_token_texture(c), session.grid.get_position(c))
	for c: Combatant in session.enemies:
		grid_view.create_unit_token(c, enc.get_token_texture(c), session.grid.get_position(c))

# ------------------------------------------------------------------------------
# Planning input: select a hero, click a target (enemy / yourself / tile), pick an action
# ------------------------------------------------------------------------------

func _planning_ready() -> bool:
	return session.is_planning() and session.focused_hero != null and session.focused_hero.is_active()

func _on_token_clicked(c: Combatant) -> void:
	if not _planning_ready():
		session.select_combatant(c)
		return
	if session.is_ally(c) and c != session.focused_hero:
		var cell: Vector2i = session.grid.get_position(c)
		if session.planned_position(c) != cell:
			# This ally is leaving its tile: offer moving the planned hero in, or switching to the ally.
			var extras: Array[Dictionary] = [{"text": "👤 Plan for %s" % c.name, "callback": func() -> void: session.select_combatant(c)}]
			_open_popup(c, "%s leaves %s" % [c.name, str(cell)], session.plan_cell_options(cell), _token_rect(c), extras)
			return
		session.select_combatant(c)   # switch the hero being planned
		return
	_open_popup(c, _title_for(c), session.plan_target_options(c), _token_rect(c))

func _on_tile_clicked(coord: Vector2i) -> void:
	if not _planning_ready() or session.planned_occupant(coord) != null:
		return
	var title: String = "Exit tile" if session.grid.is_exit_tile(coord) else "Tile %s" % str(coord)
	_open_popup(coord, title, session.plan_cell_options(coord), _tile_rect(coord))

func _title_for(c: Combatant) -> String:
	if c == session.focused_hero:
		return "%s - yourself" % c.name
	return "%s  HP %d/%d" % [c.name, c.hp, c.max_hp]

func _on_ghost_clicked(c: Combatant) -> void:
	if not _planning_ready():
		return
	if c != session.focused_hero:
		session.select_combatant(c)
		return
	var ghost: Control = grid_view.get_ghost_for(c)
	_open_popup("ghost", _title_for(c), session.plan_target_options(c), ghost.get_global_rect() if ghost != null else _token_rect(c))

func _open_popup(key: Variant, title: String, options: Array[PlanOption], target_rect: Rect2, extras: Array[Dictionary] = []) -> void:
	if popup.visible and _popup_key == key:
		var step: PlanStep = popup.default_step()   # clicking the same target again takes the first action
		popup.close()
		if step != null:
			_on_option_chosen(step)
		return
	popup.close()
	_popup_key = key
	session.set_plan_preview(null)
	popup.open_for(title, options, target_rect.end + Vector2(6, -target_rect.size.y), target_rect, extras)

func _on_option_chosen(step: PlanStep) -> void:
	if session.focused_hero != null:
		session.plan_add_step(session.focused_hero, step)

func _on_token_hovered(c: Combatant, _token: Control) -> void:
	session.set_hovered(c)
	if _planning_ready() and not popup.visible and c != session.focused_hero and not session.is_ally(c):
		_preview_default(session.plan_target_options(c))

func _on_token_unhovered() -> void:
	session.set_hovered(null)
	if not popup.visible:
		session.set_plan_preview(null)

func _on_tile_hovered(coord: Vector2i) -> void:
	if _planning_ready() and not popup.visible and session.planned_occupant(coord) == null:
		_preview_default(session.plan_cell_options(coord))

func _on_tile_unhovered() -> void:
	if not popup.visible:
		session.set_plan_preview(null)

# Previews the first valid option (and shows its cost) while hovering a target.
func _preview_default(options: Array[PlanOption]) -> void:
	for o: PlanOption in options:
		if o.valid:
			_hint_option = o
			session.set_plan_preview(o.step)
			return
	_hint_option = null
	session.set_plan_preview(null)

func _token_rect(c: Combatant) -> Rect2:
	var token: Control = grid_view.get_token_for(c)
	return token.get_global_rect() if token != null else Rect2()

func _tile_rect(coord: Vector2i) -> Rect2:
	var raw: Variant = grid_view.grid_tile_nodes.get(coord, null)
	if raw is Control:
		var tile: Control = raw
		return tile.get_global_rect()
	return Rect2()

# Draws the committed route/ghost and the hovered option's preview.
func _refresh_overlay() -> void:
	var overlay: PlanOverlay = grid_view.plan_overlay
	overlay.clear()
	_refresh_ghosts()
	if session == null or not _planning_ready():
		return
	var hero: Combatant = session.focused_hero
	var start: Vector2i = session.grid.get_position(hero)
	var st: CombatPlan.SimState = session.plan.compile(hero)
	overlay.ghost_texture = session.encounter.get_token_texture(hero)
	overlay.committed_marks = []
	overlay.committed_trail = _trail(start, st.entries, 0, overlay.committed_marks)
	overlay.committed_ghost = PlanOverlay.NONE   # planned spots are clickable shadows (grid_view ghosts)
	var pst: CombatPlan.SimState = session.plan_preview_state()
	if pst != null and pst.error_index < 0:
		overlay.preview_marks = []
		overlay.preview_trail = _trail(st.pos, pst.entries, st.entries.size(), overlay.preview_marks)
		overlay.preview_ghost = pst.pos if pst.pos != st.pos else PlanOverlay.NONE
		var step: PlanStep = session.preview_step
		if step != null and (step.mode == PlanStep.Mode.AREA or (step.mode == PlanStep.Mode.ENGAGE and step.skill != null and step.skill.is_area())):
			for i: int in range(st.entries.size(), pst.entries.size()):
				if pst.entries[i].skill.is_area():
					overlay.reach_tiles = pst.entries[i].cells
			for friend: Combatant in session.allies:
				if overlay.reach_tiles.has(session.planned_position(friend)) and friend != hero:
					overlay.ally_hit_tiles.append(session.planned_position(friend))
		elif step != null and step.mode == PlanStep.Mode.ENGAGE and step.target != null:
			var tpos: Vector2i = session.grid.get_position(step.target)
			overlay.preview_target = tpos
			var band: Vector2i = CombatPlan.reach(hero, step.skill)
			overlay.reach_tiles = _reach_around(pst.pos, band.x, band.y)
		if _hint_option != null and _hint_option.step == step:
			overlay.hint_text = _hint_option.cost_text().get_slice("\n", 0)
			overlay.hint_cell = tpos_for_hint(step, pst.pos)
	overlay.refresh_hint()

# A shadow on each ally's planned destination; small when someone still stands there.
func _refresh_ghosts() -> void:
	var ghosts: Array[Dictionary] = []
	if session != null and session.is_planning():
		for ally: Combatant in session.allies:
			if not ally.is_active() or not session.grid.has_combatant(ally):
				continue
			var dest: Vector2i = session.planned_position(ally)
			if dest == CombatPlan.NO_CELL or dest == session.grid.get_position(ally):
				continue
			ghosts.append({"unit": ally, "cell": dest, "texture": session.encounter.get_token_texture(ally), "small": session.grid.is_occupied(dest)})
	grid_view.update_ghosts(ghosts)

func tpos_for_hint(step: PlanStep, end_pos: Vector2i) -> Vector2i:
	if step.mode == PlanStep.Mode.ENGAGE and step.target != null:
		return session.grid.get_position(step.target)
	return end_pos

# Every tile the unit walks through: `from`, then each step of each move entry from `skip` on.
# marks receives the tile each move block ends on (numbered on the grid).
func _trail(from: Vector2i, entries: Array[PlanEntry], skip: int, marks: Array[Vector2i]) -> Array[Vector2i]:
	var trail: Array[Vector2i] = [from]
	var pos: Vector2i = from
	for i: int in range(skip, entries.size()):
		var e: PlanEntry = entries[i]
		if e.skill.kind == SkillDef.Kind.DISENGAGE and e.pos_after != e.pos_before:
			pos = e.pos_after   # a jump: the landing tile only, no walked trail
			trail.append(pos)
			marks.append(pos)
			continue
		if e.skill.kind != SkillDef.Kind.MOVE or e.pos_after == e.pos_before:
			continue
		for d: Vector2i in e.path:
			pos += d
			trail.append(pos)
		marks.append(pos)
	return trail
func _reach_around(center: Vector2i, near: int, far: int) -> Array[Vector2i]:
	var tiles: Array[Vector2i] = []
	for dx: int in range(-far, far + 1):
		for dy: int in range(-far, far + 1):
			var cell: Vector2i = center + Vector2i(dx, dy)
			var d: int = absi(dx) + absi(dy)
			if d <= far and d >= maxi(1, near) and session.grid.is_valid_coord(cell):
				tiles.append(cell)
	return tiles

func token_count() -> int:
	return grid_view.unit_token_nodes.size()

func visible_token_count() -> int:
	var count: int = 0
	for c: Combatant in grid_view.unit_token_nodes:
		var token: Control = grid_view.get_token_for(c)
		if token != null and token.visible:
			count += 1
	return count

func refresh() -> void:
	if session == null or session.grid == null:
		return
	var head := Vector2i(session.viewed_round, session.playhead())
	if head.x < _last_head.x or (head.x == _last_head.x and head.y < _last_head.y):
		presenter.kill_all()   # step_prev / rewind: the timeline went back, so no FX may lag behind it
	_last_head = head
	for c: Combatant in grid_view.unit_token_nodes:
		var token: Control = grid_view.get_token_for(c)
		if token == null:
			continue
		if not session.is_on_field(c):
			token.visible = false
			token.modulate.a = 0.0
			continue
		var target_px := Vector2(session.position_of(c) * TILE_SIZE)
		if not token.visible:
			token.position = target_px
		token.visible = true
		token.modulate = Color.WHITE
		presenter.slide(token, target_px)
	grid_view.update_selection_highlight(session.selected_combatant)
	_refresh_overlay()
	if popup.visible and not session.is_planning():
		popup.close()
	if session.hovered_combatant != null:
		_on_hover_changed(session.hovered_combatant)

func _on_hover_changed(c: Combatant) -> void:
	if c == null or session.is_planning():
		grid_view.hide_hover_card()   # while planning the inspector shows the hovered unit instead
		return
	grid_view.show_hover_card(c, grid_view.get_token_for(c), session.display_snapshot(), session.is_ally(c), session.position_of(c))

func _on_fx(entry: ActionLogEntry) -> void:
	if entry == null or not is_inside_tree():
		return
	var actor_token: Control = grid_view.get_token_for(session.find_combatant(entry.actor_name))
	var target_token: Control = grid_view.get_token_for(session.find_combatant(entry.target_name))
	var target_px: Vector2 = target_token.position if target_token != null else Vector2.ZERO
	if entry.category == "MOVE" and entry.coord_to != Vector2i(-1, -1):
		target_px = Vector2(entry.coord_to * TILE_SIZE)
	presenter.play(entry, actor_token, target_token, target_px)
