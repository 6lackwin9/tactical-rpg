class_name CombatGridPanel
extends CombatPanel

# ==============================================================================
# Tactical RPG - Combat Grid Panel
# 12x9 battlefield. Tokens are created for the session's own combatants (one
# source of truth), placed from the displayed state (live or snapshot), and
# animated by CombatPresenter from session.fx_requested. The unit hover card (UnitInspectorUI, COMBAT-UI2-L2)
# follows session.hover_changed after HOVER_DELAY, so hovering a roster card shows it too; focus opens it at once;
# a click or the Pin button pins it; Esc, a click elsewhere or a right click closes a pinned card.
# ==============================================================================

const TILE_SIZE: int = CombatGridView.TILE_SIZE
const HOVER_DELAY: float = 0.15   # seconds the pointer rests on a unit before the card opens
const CLOSE_GRACE: float = 0.2    # seconds the unpinned card waits after the pointer leaves (so the Pin button is reachable)
const CARD_GAP: float = 6.0       # space between the hovered tile and the card

var grid_view: CombatGridView
var popup: PlanActionPopup
var presenter: CombatPresenter
var hover_card: UnitInspectorUI
var card_unit: Combatant = null   # whose card is open

var _popup_key: Variant = null   # what the open popup is for: a Combatant or a Vector2i tile
var _hint_option: PlanOption = null
var _hover_timer: Timer
var _close_timer: Timer
var _pending: Combatant = null   # the unit the pointer rests on, waiting for HOVER_DELAY
var _last_head: Vector2i = Vector2i(-1, -1)   # (viewed round, playhead) at the last refresh: stepping back kills live FX

func _init() -> void:
	super._init()
	panel_id = "grid"
	panel_title = "Combat Grid"
	panel_icon = "🗺"
	default_min_size = Vector2(660, 480)
	grid_view = CombatGridView.new()
	add_child(grid_view)
	popup = PlanActionPopup.new()
	add_child(popup)
	presenter = CombatPresenter.new()
	presenter.overlay = grid_view.fx_layer
	add_child(presenter)
	hover_card = UnitInspectorUI.new()
	grid_view.grid_anchor.add_child(hover_card)
	_hover_timer = _make_timer(HOVER_DELAY, _on_hover_delay)
	_close_timer = _make_timer(CLOSE_GRACE, _on_close_grace)

func _on_bound() -> void:
	session.mounted.connect(_mount)
	session.fx_requested.connect(_on_fx)
	session.log_reset.connect(presenter.kill_all)
	session.hover_changed.connect(_on_hover_changed)
	hover_card.pin_toggled.connect(_on_pin_toggled)
	hover_card.card_clicked.connect(func() -> void:
		if card_unit != null:
			pin_card(card_unit)
	)
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
	for y: int in range(CombatGrid.GRID_H):
		for x: int in range(CombatGrid.GRID_W):
			var coord := Vector2i(x, y)
			var terrain_type: Terrain.Type = grid.get_terrain(coord)
			textures[coord] = TileArt.tile_at(terrain_type, coord)
			var tip: String = "%s  %s" % [str(coord), Terrain.describe(terrain_type)]
			if grid.is_exit_tile(coord):
				overlays[coord] = TileArt.overlay("exit")
				tip += " • EXIT (withdraw here)"
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
	if _click_pins(c):
		pin_card(c)
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

# A click on a token pins its card, unless the click opens the planning popup (the popup takes that spot).
func _click_pins(c: Combatant) -> bool:
	if not _planning_ready():
		return true
	return session.is_ally(c) and c != session.focused_hero and session.planned_position(c) == session.grid.get_position(c)

func _on_tile_clicked(coord: Vector2i) -> void:
	if not _planning_ready() or session.planned_occupant(coord) != null:
		return
	var title: String = "Exit tile" if session.grid.is_exit_tile(coord) else "Tile %s" % str(coord)
	_open_popup(coord, title, session.plan_cell_options(coord), _tile_rect(coord))

func _title_for(c: Combatant) -> String:
	return popup_title(c, c == session.focused_hero)

# The action popup title of a clicked unit (the Forge skill preview uses it too).
static func popup_title(c: Combatant, is_self: bool) -> String:
	if is_self:
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
	if not hover_card.is_pinned:
		close_card()   # the popup opens beside the unit: the unpinned card gets out of its way
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

# INTENT1-U: the enemy kind chips and arrows show while planning, when the tells are on (setting ui/intent_tells, hotkey I).
func tells_on() -> bool:
	return SettingsFile.get_bool("ui/intent_tells")

# Turns the tells on / off (and remembers it in the settings file unless `persist` is false).
func set_tells(on: bool, persist: bool = true) -> void:
	SettingsFile.set_value("ui/intent_tells", on)
	if persist:
		SettingsFile.save()
	_refresh_tells()

func toggle_tells() -> void:
	set_tells(not tells_on())

func _refresh_tells() -> void:
	if session != null and session.grid != null and session.is_planning() and tells_on():
		grid_view.intent_overlay.show_tells(session.enemy_intents(), session.grid)
	else:
		grid_view.intent_overlay.clear()

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

# Fits the board to `px` per tile (CombatHudMetrics.tile): tokens jump to their cells, FX use the new size.
func set_tile_px(px: int) -> void:
	if px == grid_view.tile_px:
		return
	presenter.kill_all()
	grid_view.set_tile_px(px)
	presenter.tile_px = px
	for c: Combatant in grid_view.unit_token_nodes:
		var token: Control = grid_view.get_token_for(c)
		if token != null and session != null and session.grid != null:
			token.position = Vector2(session.position_of(c) * px)
	refresh()

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
		var target_px := Vector2(session.position_of(c) * grid_view.tile_px)
		if not token.visible:
			token.position = target_px
		token.visible = true
		token.modulate = Color.WHITE
		presenter.slide(token, target_px)
	grid_view.update_selection_highlight(session.selected_combatant)
	_refresh_overlay()
	_refresh_tells()
	if popup.visible and not session.is_planning():
		popup.close()
	if hover_card.visible:
		_fill_card()
		_place_card()

# ------------------------------------------------------------------------------
# Unit hover card
# ------------------------------------------------------------------------------

func _make_timer(wait: float, callback: Callable) -> Timer:
	var timer := Timer.new()
	timer.one_shot = true
	timer.wait_time = wait
	timer.timeout.connect(callback)
	add_child(timer)
	return timer

func _start(timer: Timer) -> void:
	if timer.is_inside_tree():
		timer.start()

func _on_hover_changed(c: Combatant) -> void:
	if c == null:
		_hover_timer.stop()
		_pending = null
		if hover_card.visible and not hover_card.is_pinned:
			_start(_close_timer)
		return
	_close_timer.stop()
	if hover_card.is_pinned or (hover_card.visible and card_unit == c):
		return
	_pending = c
	_start(_hover_timer)

func _on_hover_delay() -> void:
	if _pending != null and session != null and not popup.visible:
		open_card(_pending)

func _on_close_grace() -> void:
	if hover_card.visible and not hover_card.is_pinned and not _pointer_on_card():
		close_card()

func _pointer_on_card() -> bool:
	return hover_card.visible and hover_card.get_global_rect().has_point(get_global_mouse_position())

# Shows the card for `c` beside its tile (replacing an unpinned card); a pinned card stays on its unit.
func open_card(c: Combatant) -> void:
	if c == null or session == null or session.grid == null or not is_instance_valid(c):
		return
	if hover_card.is_pinned and card_unit != c:
		return
	_close_timer.stop()
	card_unit = c
	_fill_card()
	hover_card.visible = true
	_place_card()
	_place_next_frame.call_deferred(c)

func pin_card(c: Combatant) -> void:
	if c == null or session == null:
		return
	hover_card.set_pinned(false)
	open_card(c)
	hover_card.set_pinned(card_unit == c)
	_fill_card()
	_place_card()

func close_card() -> void:
	_hover_timer.stop()
	_close_timer.stop()
	_pending = null
	card_unit = null
	hover_card.set_pinned(false)
	hover_card.visible = false

func card_is_open() -> bool:
	return hover_card.visible

# Keyboard / controller focus left the card of a unit: an unpinned card closes.
func focus_lost() -> void:
	if hover_card.visible and not hover_card.is_pinned:
		close_card()

func _on_pin_toggled(pinned: bool) -> void:
	if card_unit == null:
		return
	if pinned:
		pin_card(card_unit)
	else:
		hover_card.set_pinned(false)
		_fill_card()
		_place_card()

func _place_next_frame(c: Combatant) -> void:
	if is_inside_tree() and hover_card.visible and card_unit == c:
		await get_tree().process_frame
		if hover_card.visible and card_unit == c:
			_place_card()

func _input(event: InputEvent) -> void:
	var mouse_event := event as InputEventMouseButton
	if mouse_event == null or not mouse_event.pressed or not hover_card.visible or not hover_card.is_pinned:
		return
	if mouse_event.button_index == MOUSE_BUTTON_RIGHT:   # right click = back: closes a pinned card first
		close_card()
		get_viewport().set_input_as_handled()
	elif mouse_event.button_index == MOUSE_BUTTON_LEFT:
		if not hover_card.get_global_rect().has_point(mouse_event.global_position) and not _on_unit_token(mouse_event.global_position):
			close_card()

func _on_unit_token(point: Vector2) -> bool:
	var token: Control = grid_view.get_token_for(card_unit)
	return token != null and token.visible and token.get_global_rect().has_point(point)

# The hovered unit's tile in global coordinates (its cell, so the card does not follow a sliding token).
func unit_tile_rect(c: Combatant) -> Rect2:
	var px: float = float(grid_view.tile_px)
	var origin: Vector2 = grid_view.grid_anchor.global_position
	var cell: Vector2i = session.position_of(c)
	if cell == Vector2i(-1, -1):
		return Rect2(origin, Vector2.ZERO)
	return Rect2(origin + Vector2(cell) * px, Vector2(px, px))

# Beside the tile: right if the card fits, else left, else above or below; clamped to the board and the viewport.
func _place_card() -> void:
	if card_unit == null or not hover_card.visible:
		return
	hover_card.settle()
	var board: Rect2 = _board_rect()
	var bounds: Rect2 = board.intersection(get_viewport().get_visible_rect()) if is_inside_tree() else board
	var tile: Rect2 = unit_tile_rect(card_unit)
	var sz: Vector2 = hover_card.size
	var pos := Vector2.ZERO
	pos.x = tile.end.x + CARD_GAP
	if pos.x + sz.x > bounds.end.x:
		pos.x = tile.position.x - CARD_GAP - sz.x
	if pos.x < bounds.position.x:   # neither side has room: above or below the tile
		pos.x = clampf(tile.position.x, bounds.position.x, maxf(bounds.position.x, bounds.end.x - sz.x))
		pos.y = tile.end.y + CARD_GAP
		if pos.y + sz.y > bounds.end.y:
			pos.y = tile.position.y - CARD_GAP - sz.y
		pos.y = maxf(pos.y, bounds.position.y)
	else:
		pos.y = clampf(tile.position.y - 4.0, bounds.position.y, maxf(bounds.position.y, bounds.end.y - sz.y))
	hover_card.global_position = pos

func _fill_card() -> void:
	var c: Combatant = card_unit
	if c == null or not is_instance_valid(c):
		return
	var ally: bool = session.is_ally(c)
	var plan_lines: Array[String] = []
	if ally and session.plan != null:
		var steps: Array[PlanStep] = session.plan.steps_of(c)
		for i: int in range(steps.size()):
			plan_lines.append("%d. %s" % [i + 1, steps[i].title()])
	var intel: Array[Dictionary] = _threat_intel(c)
	var versus: Array[String] = _versus_lines(c)
	var intent: Dictionary = _intent_view(c)
	for level: int in range(3):   # compact (no gauge bars), then tight (a hostile's tell only) when the card is taller than the board
		hover_card.update_inspector(c, ally, session.position_text(c), session.encounter.get_sprite_texture(c), session.display_snapshot(),
			intel, plan_lines, versus, level >= 1, intent, level >= 2)
		hover_card.settle()
		if hover_card.size.y <= _board_rect().size.y:
			break

func _board_rect() -> Rect2:
	var px: float = float(grid_view.tile_px)
	return Rect2(grid_view.grid_anchor.global_position, Vector2(px * CombatGridView.GRID_W, px * CombatGridView.GRID_H))

# INTENT1-U: the hostile's tell for the hover card: {"first": the first action, "head": "Read by ..." (revealed only), "lines": the plan, one per action}. {} outside planning.
func _intent_view(c: Combatant) -> Dictionary:
	if session.is_ally(c):
		return {}
	var rec: Dictionary = session.intent_of(c)
	if rec.is_empty():
		return {}
	var lines: Array[String] = []
	var head: String = ""
	if DictUtil.to_bool(rec["revealed"]):
		head = IntentText.read_by(rec)
		for raw: Variant in DictUtil.array_of(rec["plan"]):
			lines.append(IntentText.plan_line(DictUtil.dict_of(raw)))
	return {"first": IntentText.first_line(rec), "head": head, "lines": lines}

# What the AI is known to do (role, strike power, guard and flee thresholds) for a hostile; one line each.
func _threat_intel(c: Combatant) -> Array[Dictionary]:
	var intel: Array[Dictionary] = []
	if session.is_ally(c):
		return intel
	var profile: AiProfile = session.encounter.profile_of(c)
	var role_desc: String = ""
	match profile.role:
		AiProfile.Role.AGGRESSOR:
			role_desc = "closes in on the nearest foe and trades blows"
		AiProfile.Role.SKIRMISHER:
			role_desc = "hunts the weakest foe; backs off when hurt"
		AiProfile.Role.SNIPER:
			role_desc = "holds maximum range; retreats from melee"
		AiProfile.Role.SUPPORT:
			role_desc = "opens with Focus, then fights from range"
	intel.append({"name": AiProfile.role_name(profile.role), "desc": role_desc, "color": Color(0.9, 0.6, 0.3)})
	intel.append({"name": "Strike %d power, range %d" % [DamageCalculator.power_of(c, c.attack_range > 1), c.attack_range], "desc": "", "color": Color(0.9, 0.35, 0.35)})
	if profile.defend_below > 0.0:
		intel.append({"name": "Guards at <=%d%% HP (50%% guard, once a round)" % int(profile.defend_below * 100.0), "desc": "", "color": Color(0.35, 0.65, 0.95)})
	if profile.flee_ratio > 0.0:
		intel.append({"name": "Flees at <=%d%% HP to the exit" % int(profile.flee_ratio * 100.0), "desc": "", "color": Color(0.3, 0.85, 0.5)})
	return intel

# The numbers of the planning hero against a hostile: distance and reach, stance factor, both strikes.
func _versus_lines(c: Combatant) -> Array[String]:
	var lines: Array[String] = []
	var hero: Combatant = session.focused_hero
	if session.is_ally(c) or hero == null or not hero.is_active() or not c.is_active():
		return lines
	if not session.grid.has_combatant(hero) or not session.grid.has_combatant(c):
		return lines
	var basic: SkillDef = SkillCatalog.lowest(SkillDef.Kind.ATTACK)
	var band: Vector2i = basic.reach_for(hero)
	var dist: int = session.grid.get_distance(session.grid.get_position(hero), session.grid.get_position(c))
	lines.append("%s: distance %d, reach %d-%d" % [hero.name, dist, band.x, band.y])
	lines.append("Stance %s vs %s %.2fx" % [CombatTypes.stance_to_string(hero.current_stance), CombatTypes.stance_to_string(c.current_stance),
		CombatTypes.get_matchup_multiplier(hero.current_stance, c.current_stance)])
	var out: int = maxi(1, roundi(float(DamageCalculator.expected(hero, c, basic, basic.stance())) * (1.0 - c.defense_mitigation_ratio)))
	var back: int = maxi(1, roundi(float(DamageCalculator.expected(c, hero, basic, basic.stance())) * (1.0 - hero.defense_mitigation_ratio)))
	lines.append("Strike ~%d (crit %.0f%%); it hits you ~%d" % [out, DamageCalculator.crit_chance(hero), back])
	return lines

func _on_fx(entry: ActionLogEntry) -> void:
	if entry == null or not is_inside_tree():
		return
	var actor_token: Control = grid_view.get_token_for(session.find_combatant(entry.actor_name))
	var target_token: Control = grid_view.get_token_for(session.find_combatant(entry.target_name))
	var target_px: Vector2 = target_token.position if target_token != null else Vector2.ZERO
	if entry.category == "MOVE" and entry.coord_to != Vector2i(-1, -1):
		target_px = Vector2(entry.coord_to * grid_view.tile_px)
	presenter.play(entry, actor_token, target_token, target_px)
