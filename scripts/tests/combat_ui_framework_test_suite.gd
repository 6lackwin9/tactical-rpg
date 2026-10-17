class_name CombatUiFrameworkTestSuite
extends RefCounted

# ==============================================================================
# TEST 1: COMBAT UI FRAMEWORK & BEHAVIOUR TEST (CombatSession + CombatDockHost)
# ==============================================================================

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING COMBAT UI FRAMEWORK & BEHAVIOUR TEST   ===")
	print("=======================================================")

	var session := CombatSession.new(CatalogEncounter.new("goblin_ambush"))
	var host := CombatDockHost.new()
	if tree_parent != null:
		tree_parent.add_child(host)
	host.setup(session)
	session.start()

	# 1. Every panel is mounted in its dock region of the default layout
	var expected_regions: Dictionary = {
		"party": "top_left", "timeline": "top_center", "enemies": "top_right",
		"inspector": "left", "grid": "center", "forecast": "right",
		"playback": "bottom_left", "action_log": "bottom_center",
	}
	var docks_ok: bool = host.log_modal != null and host.timeline_modal != null
	for id: String in expected_regions:
		if host.get_panel(id) == null or host.region_of(id) != str(expected_regions[id]):
			docks_ok = false

	# 2. Tokens and roster cards are bound to the SAME combatants the session simulates
	var grid_panel: CombatGridPanel = host.get_panel("grid") as CombatGridPanel
	var enemy_panel: CombatRosterPanel = host.get_panel("enemies") as CombatRosterPanel
	var warrior: Combatant = session.allies[0]
	var tokens_ok: bool = (
		grid_panel.token_count() == session.allies.size() + session.enemies.size()
		and grid_panel.grid_view.get_token_for(warrior) != null
		and grid_panel.visible_token_count() == 4  # 2 heroes + Brawler + Scout; the ambush is not on the field yet
		and enemy_panel.visible_card_count() == 2
	)

	# 3. Selecting a unit updates the Unit Inspector
	var inspector_panel: CombatInspectorPanel = host.get_panel("inspector") as CombatInspectorPanel
	session.select_combatant(session.find_combatant("Enemy Brawler"))
	var enemy_shown: bool = inspector_panel.inspector.inspector_name_lbl.text == "Enemy Brawler"
	session.select_combatant(warrior)
	var inspector_ok: bool = enemy_shown and inspector_panel.inspector.inspector_name_lbl.text == "Warrior"

	# 3b. The side panels and the grid keep their rects whichever unit is selected (hero or enemy)
	var layout_stable: bool = true
	if tree_parent != null:
		var view_size: Vector2 = Vector2(1280, 720)   # the default / web window size
		host.size = view_size
		var hero_rects: Array[Vector2] = _dock_rects(host)
		session.select_combatant(session.find_combatant("Enemy Brawler"))
		var enemy_rects: Array[Vector2] = _dock_rects(host)
		session.select_combatant(warrior)
		layout_stable = hero_rects == enemy_rects and hero_rects[2].x + hero_rects[2].y <= view_size.x and hero_rects[0].y > 0.0

	# 4. Tactical forecast targets the selected unit's opponent
	var forecast_panel: CombatForecastPanel = host.get_panel("forecast") as CombatForecastPanel
	var target: Combatant = session.forecast_target(warrior)
	var forecast_ok: bool = target != null and target.name in forecast_panel.forecast.forecast_target_lbl.text

	# 5. Log modal opens with the session's entries and closes
	for i: int in range(30):
		session.step_next()
		if session.current_block >= 3:
			break
	host.open_modal("log")
	var log_open: bool = host.log_modal.visible and host.log_modal.all_entries.size() == session.log_entries.size() and not session.log_entries.is_empty()
	host.close_modals()
	var log_modal_ok: bool = log_open and not host.log_modal.visible

	# 6. Timeline modal opens and closes
	host.open_modal("timeline")
	var timeline_open: bool = host.timeline_modal.visible
	host.close_modals()
	var timeline_modal_ok: bool = timeline_open and not host.timeline_modal.visible

	# 7. Timeline dock block click inspects a resolved block of the live round
	var timeline_panel: CombatTimelinePanel = host.get_panel("timeline") as CombatTimelinePanel
	timeline_panel.dock.block_selected.emit(1)
	var inspected: bool = session.inspect_block == 1 and session.playhead() == 1 and session.display_snapshot() != null
	session.step_next()
	var signals_ok: bool = inspected and session.inspect_block == -1

	# 8. Undock floats a panel (its region keeps a placeholder), dock returns it; FX dispatch is safe
	host.undock("forecast")
	var floating: bool = host.is_undocked("forecast") and host.region_of("forecast") == "right"
	host.dock("forecast")
	host.apply_layout("Mirrored")
	var mirrored: bool = host.region_of("inspector") == "right" and host.region_of("forecast") == "left"
	host.apply_layout("Standard")
	if tree_parent != null:
		var brawler: Combatant = session.find_combatant("Enemy Brawler")
		session.fx_requested.emit(ActionLogEntry.new(1, 2, "ATTACK", warrior.name, brawler.name, "-25 HP", 25, "HIT"))
		session.fx_requested.emit(ActionLogEntry.new(1, 1, "DEFEND", warrior.name, "", "50%", 50, "DEFEND"))
		session.fx_requested.emit(ActionLogEntry.new(1, 1, "BUFF", warrior.name, "", "+50%", 50, "FOCUS"))
		session.fx_requested.emit(ActionLogEntry.new(1, 1, "MOVE", warrior.name, "", "-> (4, 4)", 0, "MOVED", Vector2i(4, 5), Vector2i(4, 4)))
	var layout_ok: bool = floating and not host.is_undocked("forecast") and host.region_of("forecast") == "right" and mirrored

	print("\n=======================================================")
	print("=== COMBAT UI FRAMEWORK TEST ASSERTIONS             ===")
	print("=======================================================")
	print("  [UI] All 8 combat panels mounted in their dock regions: %s" % ("PASS" if docks_ok else "FAIL"))
	print("  [UI] Tokens & roster bound to the simulated combatants: %s" % ("PASS" if tokens_ok else "FAIL"))
	print("  [UI] Combatant selection & Unit Inspector synced: %s" % ("PASS" if inspector_ok else "FAIL"))
	print("  [UI] Side panels and grid keep their size for hero and enemy selection, inside the viewport: %s" % ("PASS" if layout_stable else "FAIL"))
	print("  [UI] Tactical forecast computed & rendered: %s" % ("PASS" if forecast_ok else "FAIL"))
	print("  [UI] Action Log Modal toggle & entry view lifecycle: %s" % ("PASS" if log_modal_ok else "FAIL"))
	print("  [UI] Timeline Modal toggle lifecycle: %s" % ("PASS" if timeline_modal_ok else "FAIL"))
	print("  [UI] Timeline dock block selection inspects resolved block: %s" % ("PASS" if signals_ok else "FAIL"))
	print("  [UI] Undock/dock, layout presets & action FX dispatch: %s" % ("PASS" if layout_ok else "FAIL"))

	if tree_parent != null:
		tree_parent.remove_child(host)
	host.free()
	session.dispose()

	return (docks_ok and tokens_ok and inspector_ok and layout_stable and forecast_ok and
		log_modal_ok and timeline_modal_ok and signals_ok and layout_ok)


# Left panel, grid and right panel horizontal extents (x, width) after a synchronous layout pass;
# heights are not compared: a one-shot pass leaves wrapped labels at a transient height.
static func _dock_rects(host: CombatDockHost) -> Array[Vector2]:
	_sort_now(host)
	var rects: Array[Vector2] = []
	for id: String in ["inspector", "grid", "forecast"]:
		var r: Rect2 = host.get_panel(id).get_global_rect()
		rects.append(Vector2(r.position.x, r.size.x))
	return rects

# Containers re-sort on the next frame; run the pass now, top-down.
static func _sort_now(node: Node) -> void:
	if node is Container:
		(node as Container).notification(Container.NOTIFICATION_SORT_CHILDREN)
	for child: Node in node.get_children():
		_sort_now(child)
