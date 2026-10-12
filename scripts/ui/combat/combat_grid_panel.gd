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

func _init() -> void:
	super._init()
	panel_id = "grid"
	panel_title = "Combat Grid"
	panel_icon = "🗺"
	default_min_size = Vector2(496, 480)
	grid_view = CombatGridView.new()
	add_child(grid_view)

func _on_bound() -> void:
	session.mounted.connect(_mount)
	session.fx_requested.connect(_on_fx)
	session.hover_changed.connect(_on_hover_changed)
	grid_view.combatant_clicked.connect(func(c: Combatant) -> void: session.select_combatant(c))
	grid_view.combatant_hovered.connect(func(c: Combatant, _token: Control) -> void: session.set_hovered(c))
	grid_view.combatant_unhovered.connect(func() -> void: session.set_hovered(null))
	_mount()

func _mount() -> void:
	grid_view.clear_tokens()
	if session == null or session.grid == null:
		return
	var enc: CombatEncounter = session.encounter
	grid_view.setup_grid_tiles(enc.get_tile_texture("floor"), enc.get_tile_texture("exit"), enc.get_tile_texture("center"))
	for c: Combatant in session.allies:
		grid_view.create_unit_token(c, enc.get_token_texture(c), session.grid.get_position(c))
	for c: Combatant in session.enemies:
		grid_view.create_unit_token(c, enc.get_token_texture(c), session.grid.get_position(c))

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
	if session.hovered_combatant != null:
		_on_hover_changed(session.hovered_combatant)

func _on_hover_changed(c: Combatant) -> void:
	if c == null:
		grid_view.hide_hover_card()
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
