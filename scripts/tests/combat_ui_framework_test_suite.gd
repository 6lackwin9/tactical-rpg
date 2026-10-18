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
		"party": "left_top", "timeline": "top", "enemies": "right_top",
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

	var hud_ok: bool = _hud_checks(tree_parent)

	return (hud_ok and docks_ok and tokens_ok and inspector_ok and layout_stable and forecast_ok and
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


# ------------------------------------------------------------------------------
# COMBAT-UI2-L1: layout C numbers (derived from the rules in CombatHudMetrics / the spec, not from shipped content)
# ------------------------------------------------------------------------------

const VIEWS: Array[Vector2] = [Vector2(1280, 720), Vector2(1280, 800), Vector2(1680, 720)]

static func _hud_checks(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	var saved_scale: Variant = SettingsFile.get_value("ui/text_scale", 1.0)

	# Pure metrics: the tile formula, minimum tile, fixed column heights, line dropping, at every size and text size.
	var tile_ok: bool = true
	var heights_ok: bool = true
	var rows_ok: bool = true
	for view: Vector2 in VIEWS:
		var last_party_rows: int = 99
		var last_hostile_rows: int = 99
		for scale: float in SettingsFile.TEXT_LEVELS:
			for party_n: int in [1, 3]:
				var m: CombatHudMetrics = CombatHudMetrics.compute(view, scale, party_n, 5)
				var expect: int = maxi(48, floori(minf(float(m.area_h - CombatHudMetrics.FRAME) / 9.0, float(m.area_w - CombatHudMetrics.FRAME) / 12.0)))
				tile_ok = tile_ok and m.tile == expect and m.tile >= 48
				heights_ok = heights_ok and m.party_panel_h + m.inspector_h + CombatHudMetrics.COL_GAP == m.area_h
				heights_ok = heights_ok and m.hostile_panel_h + m.forecast_h + CombatHudMetrics.COL_GAP == m.area_h and not m.hostile_scroll
				heights_ok = heights_ok and m.inspector_h > 0 and m.forecast_h > 0
				if party_n == 3:
					rows_ok = rows_ok and m.party_rows <= last_party_rows and m.hostile_rows <= last_hostile_rows
					last_party_rows = m.party_rows
					last_hostile_rows = m.hostile_rows
				rows_ok = rows_ok and m.party_rows >= CombatHudMetrics.PARTY_ROWS_MIN and m.hostile_rows >= CombatHudMetrics.HOSTILE_ROWS_MIN
	checks["tile = floor(min(h/9, w/12)), at least 48, at 3 window sizes x 4 text sizes"] = tile_ok
	checks["column panel heights are fixed and add up to the centre row, no hostile scroll"] = heights_ok
	checks["cards only drop lines as text grows (never below the minimum rows)"] = rows_ok
	checks["tile_for: a short area is height-bound, a narrow one width-bound, tiny floors at 48"] = (
		CombatHudMetrics.tile_for(Vector2(2000, 540)) == 60 and CombatHudMetrics.tile_for(Vector2(600, 2000)) == 50
		and CombatHudMetrics.tile_for(Vector2(100, 100)) == CombatHudMetrics.MIN_TILE)

	# Text scale: the HUD font never drops below 14 px at Normal and follows the setting.
	SettingsFile.set_value("ui/text_scale", 1.0)
	var normal: int = UiTheme.hud_font(9)
	SettingsFile.set_value("ui/text_scale", 1.3)
	var large: int = UiTheme.hud_font(9)
	SettingsFile.set_value("ui/text_scale", saved_scale)
	checks["HUD font: 14 px floor at Normal, scaled by the text size"] = normal == 14 and large == roundi(14.0 * 1.3)

	if tree_parent != null:
		var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
		var host := CombatDockHost.new()
		tree_parent.add_child(host)
		host.setup(session)
		session.start()
		var party: CombatRosterPanel = host.get_panel("party") as CombatRosterPanel
		var foes: CombatRosterPanel = host.get_panel("enemies") as CombatRosterPanel
		var grid: CombatGridPanel = host.get_panel("grid") as CombatGridPanel
		var in_view: int = session.enemies_in_view().size()
		checks["hero and hostile cards all show (one per unit in view, no scrolling)"] = (
			party.visible_card_count() == session.allies.size() and foes.visible_card_count() == in_view and in_view >= 1
			and not host.metrics.hostile_scroll)
		checks["empty party slots are muted outlines up to the slot count"] = party.empty_slot_count() == host.metrics.party_slots - session.allies.size()

		# Sizes at each window size: the grid tile, the fixed panel heights, nothing outside the window.
		var sizes_ok: bool = true
		var fixed_ok: bool = true
		var inside_ok: bool = true
		for view: Vector2 in VIEWS:
			host.size = view
			host._fit()
			_sort_now(host)
			sizes_ok = sizes_ok and host.metrics.tile >= 48 and grid.grid_view.tile_px == host.metrics.tile
			fixed_ok = fixed_ok and party.content_fits() and foes.content_fits() and host._slot("party").custom_minimum_size.y == host.metrics.party_panel_h and host._slot("enemies").custom_minimum_size.y == host.metrics.hostile_panel_h
			for id: String in ["timeline", "party", "grid", "enemies", "forecast", "action_log"]:
				var r: Rect2 = host.get_panel(id).get_global_rect()
				inside_ok = inside_ok and r.position.x >= -0.5 and r.end.x <= view.x + 0.5
			session.select_combatant(session.enemies[0])   # selecting a hostile keeps the fixed heights
			fixed_ok = fixed_ok and party.content_fits() and foes.content_fits() and host._slot("party").custom_minimum_size.y == host.metrics.party_panel_h
			session.select_combatant(session.allies[0])
		checks["grid tile fitted to the window (>= 48) at 1280x720, 1280x800, 1680x720"] = sizes_ok
		checks["party and hostile panels keep their fixed heights; every panel inside the window"] = fixed_ok and inside_ok

		# A hero card click selects that hero for the timeline.
		var second: Combatant = session.allies[1]
		var card: Control = party._card_for(second)
		var click := InputEventMouseButton.new()
		click.button_index = MOUSE_BUTTON_LEFT
		click.pressed = true
		card.gui_input.emit(click)
		checks["clicking a hero card selects the hero"] = session.selected_combatant == second and session.focused_hero == second

		# Rule 7: no gear button, no equipment modal path in battle.
		var gear_buttons: int = 0
		for node: Node in host.find_children("*", "Button", true, false):
			var btn: Button = node as Button
			if btn.text.contains("GEAR") or btn.text.contains("🎒") or btn.tooltip_text.contains("Gear"):
				gear_buttons += 1
		host.open_modal("gear")
		var key := InputEventKey.new()
		key.keycode = KEY_G
		key.pressed = true
		host._unhandled_input(key)
		checks["no gear button and no equipment modal path in the battle scene"] = (
			gear_buttons == 0 and host.find_children("*", "CharacterEquipmentModal", true, false).is_empty()
			and not ("btn_gear" in (host.get_panel("playback") as CombatPlaybackPanel).controls)
			and not ("btn_gear_modal" in (host.get_panel("inspector") as CombatInspectorPanel).inspector))

		# The log is a one-line ticker; Expand opens the log modal.
		var ticker: ActionLogDockUI = (host.get_panel("action_log") as CombatActionLogPanel).dock
		checks["log is a one-line ticker that expands into the log modal"] = ticker.ticker and ticker.ticker_lbl != null and not ticker.ticker_lbl.text.contains("\n")
		ticker.btn_expand_log.pressed.emit()
		checks["log is a one-line ticker that expands into the log modal"] = checks["log is a one-line ticker that expands into the log modal"] and host.log_modal.visible
		host.close_modals()

		tree_parent.remove_child(host)
		host.free()
		session.dispose()

		# One screen per case: the largest text size on its own host.
		SettingsFile.set_value("ui/text_scale", 1.3)
		var xl_session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
		var xl_host := CombatDockHost.new()
		tree_parent.add_child(xl_host)
		xl_host.size = Vector2(1280, 720)
		xl_host.setup(xl_session)
		xl_session.start()
		_sort_now(xl_host)
		var xl_party: CombatRosterPanel = xl_host.get_panel("party") as CombatRosterPanel
		var xl_foes: CombatRosterPanel = xl_host.get_panel("enemies") as CombatRosterPanel
		var xl_ok: bool = xl_host.metrics.fs == roundi(14.0 * 1.3) and xl_host.metrics.tile >= 48
		xl_ok = xl_ok and xl_party.visible_card_count() == xl_session.allies.size() and xl_foes.visible_card_count() == xl_session.enemies_in_view().size()
		xl_ok = xl_ok and xl_party.content_fits() and xl_foes.content_fits() and xl_host.metrics.party_panel_h + xl_host.metrics.inspector_h + CombatHudMetrics.COL_GAP == xl_host.metrics.area_h
		checks["Extra large text: cards still show, grid >= 48, cards fit their fixed heights"] = xl_ok
		tree_parent.remove_child(xl_host)
		xl_host.free()
		xl_session.dispose()
		SettingsFile.set_value("ui/text_scale", saved_scale)

	checks["party slots follow the cap, never fewer than the heroes, fallback 3 when unknown"] = (
		CombatHudMetrics.compute(VIEWS[0], 1.0, 2, 5, 2).party_slots == 2 and CombatHudMetrics.compute(VIEWS[0], 1.0, 2, 5, 4).party_slots == 4
		and CombatHudMetrics.compute(VIEWS[0], 1.0, 2, 5, 1).party_slots == 2 and CombatHudMetrics.compute(VIEWS[0], 1.0, 1, 5).party_slots == CombatHudMetrics.PARTY_SLOTS)
	if tree_parent != null:
		var cap_ok: bool = true
		for cap: int in [2, 4]:   # one screen per case
			var enc := CatalogEncounter.new("goblin_skirmish")
			enc.party_cap = cap
			var cap_session := CombatSession.new(enc)
			var cap_host := CombatDockHost.new()
			tree_parent.add_child(cap_host)
			cap_host.setup(cap_session)
			cap_session.start()
			var cap_party: CombatRosterPanel = cap_host.get_panel("party") as CombatRosterPanel
			cap_ok = cap_ok and cap_party.empty_slot_count() + cap_session.allies.size() == maxi(cap, cap_session.allies.size()) and cap_host.metrics.party_slots == maxi(cap, cap_session.allies.size())
			tree_parent.remove_child(cap_host)
			cap_host.free()
			cap_session.dispose()
		checks["a party cap of 2 shows 2 slots and a cap of 4 shows 4 (heroes + muted empty slots)"] = cap_ok

	print("\n=======================================================")
	print("=== COMBAT UI LAYOUT C (COMBAT-UI2-L1) ASSERTIONS   ===")
	print("=======================================================")
	var all_ok: bool = true
	for label: String in checks:
		var ok: bool = DictUtil.to_bool(checks[label])
		all_ok = all_ok and ok
		print("  [UI] %s: %s" % [label, "PASS" if ok else "FAIL"])
	return all_ok
