class_name CombatGridPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Combat Grid Panel
# 9x9 battlefield. Tokens are created for the session's own combatants (one
# source of truth), placed from the displayed state (live or snapshot), and
# animated by session.fx_requested. Hover cards follow session.hover_changed so
# hovering a roster card also highlights the unit here.
# ==============================================================================

const TILE_SIZE: int = 54

var grid_view: CombatGridView
var popup: PlanActionPopup

var _popup_key: Variant = null   # what the open popup is for: a Combatant or a Vector2i tile
var _hint_option: PlanOption = null

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

func _on_bound() -> void:
	session.mounted.connect(_mount)
	session.fx_requested.connect(_on_fx)
	session.hover_changed.connect(_on_hover_changed)
	grid_view.combatant_clicked.connect(_on_token_clicked)
	grid_view.combatant_hovered.connect(_on_token_hovered)
	grid_view.combatant_unhovered.connect(_on_token_unhovered)
	grid_view.tile_clicked.connect(_on_tile_clicked)
	grid_view.tile_hovered.connect(_on_tile_hovered)
	grid_view.tile_unhovered.connect(_on_tile_unhovered)
	session.plan_preview_changed.connect(_refresh_overlay)
	popup.option_hovered.connect(func(step: PlanStep) -> void: session.set_plan_preview(step))
	popup.option_chosen.connect(_on_option_chosen)
	popup.closed.connect(func() -> void:
		_popup_key = null
		session.set_plan_preview(null)
	)
	_mount()

func _mount() -> void:
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
		session.select_combatant(c)   # switch the hero being planned
		return
	_open_popup(c, _title_for(c), session.plan_target_options(c), _token_rect(c))

func _on_tile_clicked(coord: Vector2i) -> void:
	if not _planning_ready() or session.grid.is_occupied(coord) or session.grid.get_position(session.focused_hero) == coord:
		return
	var title: String = "Exit tile" if session.grid.is_exit_tile(coord) else "Move to %s" % str(coord)
	_open_popup(coord, title, session.plan_cell_options(coord), _tile_rect(coord))

func _title_for(c: Combatant) -> String:
	if c == session.focused_hero:
		return "%s - yourself" % c.name
	return "%s  HP %d/%d" % [c.name, c.hp, c.max_hp]

func _open_popup(key: Variant, title: String, options: Array[PlanOption], target_rect: Rect2) -> void:
	if popup.visible and _popup_key == key:
		var step: PlanStep = popup.default_step()   # clicking the same target again takes the first action
		popup.close()
		if step != null:
			_on_option_chosen(step)
		return
	popup.close()
	_popup_key = key
	session.set_plan_preview(null)
	popup.open_for(title, options, target_rect.end + Vector2(6, -target_rect.size.y), target_rect)

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
	if _planning_ready() and not popup.visible and not session.grid.is_occupied(coord):
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
	if session == null or not _planning_ready():
		return
	var hero: Combatant = session.focused_hero
	var start: Vector2i = session.grid.get_position(hero)
	var st: CombatPlan.SimState = session.plan.compile(hero)
	overlay.ghost_texture = session.encounter.get_token_texture(hero)
	overlay.committed_marks = []
	overlay.committed_trail = _trail(start, st.entries, 0, overlay.committed_marks)
	overlay.committed_ghost = st.pos if st.pos != start else PlanOverlay.NONE
	var pst: CombatPlan.SimState = session.plan_preview_state()
	if pst != null and pst.error_index < 0:
		overlay.preview_marks = []
		overlay.preview_trail = _trail(st.pos, pst.entries, st.entries.size(), overlay.preview_marks)
		overlay.preview_ghost = pst.pos if pst.pos != st.pos else PlanOverlay.NONE
		var step: PlanStep = session.preview_step
		if step != null and step.mode == PlanStep.Mode.ENGAGE and step.target != null:
			var tpos: Vector2i = session.grid.get_position(step.target)
			overlay.preview_target = tpos
			overlay.reach_tiles = _reach_around(pst.pos, hero.attack_range)
		if _hint_option != null and _hint_option.step == step:
			overlay.hint_text = _hint_option.cost_text().get_slice("\n", 0)
			overlay.hint_cell = tpos_for_hint(step, pst.pos)
	overlay.refresh_hint()

func tpos_for_hint(step: PlanStep, end_pos: Vector2i) -> Vector2i:
	if step.mode == PlanStep.Mode.ENGAGE and step.target != null:
		return session.grid.get_position(step.target)
	return end_pos

# Every tile the unit walks through: rom, then each step of each move entry from skip on.
# marks receives the tile each move block ends on (numbered on the grid).
func _trail(from: Vector2i, entries: Array[PlanEntry], skip: int, marks: Array[Vector2i]) -> Array[Vector2i]:
	var trail: Array[Vector2i] = [from]
	var pos: Vector2i = from
	for i: int in range(skip, entries.size()):
		var e: PlanEntry = entries[i]
		if e.skill.kind != SkillDef.Kind.MOVE or e.pos_after == e.pos_before:
			continue
		for d: Vector2i in e.path:
			pos += d
			trail.append(pos)
		marks.append(pos)
	return trail
func _reach_around(center: Vector2i, reach: int) -> Array[Vector2i]:
	var tiles: Array[Vector2i] = []
	for dx: int in range(-reach, reach + 1):
		for dy: int in range(-reach, reach + 1):
			var cell: Vector2i = center + Vector2i(dx, dy)
			if absi(dx) + absi(dy) <= reach and absi(dx) + absi(dy) > 0 and session.grid.is_valid_coord(cell):
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
		if token.position != target_px and is_inside_tree():
			var tw := create_tween()
			tw.tween_property(token, "position", target_px, 0.22).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
		elif not is_inside_tree():
			token.position = target_px
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
	var actor: Combatant = session.find_combatant(entry.actor_name)
	var target: Combatant = session.find_combatant(entry.target_name)
	var actor_token: Control = grid_view.get_token_for(actor)
	var target_token: Control = grid_view.get_token_for(target)
	match entry.category:
		"ATTACK":
			if actor != null and target_token != null:
				grid_view.animate_attacker_lunge(actor, target_token.position)
			if target != null and target_token != null:
				var is_mitigated: bool = "MITIGAT" in entry.outcome_tag
				grid_view.animate_token_impact(target, true, is_mitigated)
				var txt: String = "-%d%s" % [entry.value_num, " [MIT]" if is_mitigated else ""]
				grid_view.spawn_floating_combat_text(target_token.position, txt, Color(0.4, 0.85, 1.0) if is_mitigated else Color(1.0, 0.35, 0.35))
				if entry.outcome_tag == "DEFEATED":
					grid_view.spawn_floating_combat_text(target_token.position + Vector2(0, -14), "💀 DEFEATED!", Color(1.0, 0.25, 0.25))
		"DEFEND":
			if actor != null and actor_token != null:
				grid_view.animate_token_pulse(actor, Color(0.5, 0.8, 1.8))
				grid_view.spawn_floating_combat_text(actor_token.position, "🛡 DEFEND [%d%%]" % entry.value_num, Color(0.4, 0.8, 1.0))
		"BUFF":
			if actor != null and actor_token != null:
				grid_view.animate_token_pulse(actor, Color(1.8, 1.5, 0.4))
				grid_view.spawn_floating_combat_text(actor_token.position, "✦ +%d%% ATK" % entry.value_num, Color(1.0, 0.85, 0.25))
		"MOVE":
			if actor != null and entry.coord_to != Vector2i(-1, -1):
				grid_view.animate_token_move(actor, entry.coord_to)
			if "WITHDRAWN" in entry.outcome_tag and actor_token != null:
				grid_view.spawn_floating_combat_text(actor_token.position, "⚑ WITHDRAWN!", Color(0.35, 0.95, 0.55))
		"EVENT":
			if "AMBUSH" in entry.outcome_tag:
				grid_view.spawn_floating_combat_text(Vector2(200, 180), "⚠️ AMBUSH!", Color(1.0, 0.4, 0.4))
