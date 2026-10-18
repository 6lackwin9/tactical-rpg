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
		"grid": "center", "forecast": "right",
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

	# 3. The unit hover card replaces the Unit Inspector: pinning a unit shows that unit's card
	var brawler_unit: Combatant = session.find_combatant("Enemy Brawler")
	grid_panel.pin_card(brawler_unit)
	var enemy_shown: bool = grid_panel.hover_card.visible and grid_panel.hover_card.name_lbl.text == brawler_unit.name
	grid_panel.pin_card(warrior)
	var inspector_ok: bool = enemy_shown and grid_panel.hover_card.name_lbl.text == warrior.name and host.get_panel("inspector") == null
	grid_panel.close_card()

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
	var mirrored: bool = host.region_of("party") == "right_top" and host.region_of("forecast") == "left"
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
	print("  [UI] All 7 combat panels mounted in their dock regions: %s" % ("PASS" if docks_ok else "FAIL"))
	print("  [UI] Tokens & roster bound to the simulated combatants: %s" % ("PASS" if tokens_ok else "FAIL"))
	print("  [UI] Unit hover card shows the pinned unit (the inspector panel is gone): %s" % ("PASS" if inspector_ok else "FAIL"))
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
	for id: String in ["party", "grid", "forecast"]:
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
				heights_ok = heights_ok and m.party_panel_h == m.area_h
				heights_ok = heights_ok and m.hostile_panel_h + m.forecast_h + CombatHudMetrics.COL_GAP == m.area_h and not m.hostile_scroll
				heights_ok = heights_ok and m.forecast_h > 0
				if party_n == 3:
					rows_ok = rows_ok and m.party_rows <= last_party_rows and m.hostile_rows <= last_hostile_rows
					last_party_rows = m.party_rows
					last_hostile_rows = m.hostile_rows
				rows_ok = rows_ok and m.party_rows >= CombatHudMetrics.PARTY_ROWS_MIN and m.hostile_rows >= CombatHudMetrics.HOSTILE_ROWS_MIN
	checks["tile = floor(min(h/9, w/12)), at least 48, at 3 window sizes x 4 text sizes"] = tile_ok
	checks["party panel fills the centre row (inspector space handed back); hostile + forecast add up to it, no hostile scroll"] = heights_ok
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
			and not ("btn_gear_modal" in grid.hover_card) and _card_buttons_ok(grid))

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
		xl_ok = xl_ok and xl_party.content_fits() and xl_foes.content_fits() and xl_host.metrics.party_panel_h == xl_host.metrics.area_h
		checks["Extra large text: cards still show, grid >= 48, cards fit their fixed heights"] = xl_ok
		tree_parent.remove_child(xl_host)
		xl_host.free()
		xl_session.dispose()
		SettingsFile.set_value("ui/text_scale", saved_scale)

	_hover_card_checks(tree_parent, checks)
	_autoplay_ui_checks(tree_parent, checks)
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


# ------------------------------------------------------------------------------
# COMBAT-UI2-L2: the unit hover card (one screen per case: many screens in one arena frame crash)
# ------------------------------------------------------------------------------

# The only button on a card is Pin / Unpin: nothing on it opens the equipment modal.
static func _card_buttons_ok(grid: CombatGridPanel) -> bool:
	grid.pin_card(grid.session.allies[0])
	var ok: bool = true
	for node: Node in grid.hover_card.find_children("*", "Button", true, false):
		var btn: Button = node as Button
		ok = ok and btn.name == "PinButton" and (btn.text == "Pin" or btn.text == "Unpin")
	grid.close_card()
	return ok

static func _card_host(tree_parent: Node, view: Vector2, scale: float) -> CombatDockHost:
	SettingsFile.set_value("ui/text_scale", scale)
	var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	var host := CombatDockHost.new()
	tree_parent.add_child(host)
	host.size = view
	host.setup(session)
	session.start()
	host._fit()
	_sort_now(host)
	return host

static func _card_free(tree_parent: Node, host: CombatDockHost, saved_scale: Variant) -> void:
	var session: CombatSession = host.session
	tree_parent.remove_child(host)
	host.free()
	session.dispose()
	SettingsFile.set_value("ui/text_scale", saved_scale)

static func _units_in_view(session: CombatSession) -> Array[Combatant]:
	var units: Array[Combatant] = []
	units.append_array(session.allies)
	units.append_array(session.enemies_in_view())
	return units

# For every unit in view: the card opens, stays inside the viewport, never overlaps the unit's tile or token and
# holds all its content (nothing clipped).
static func _card_geometry_ok(host: CombatDockHost, view: Vector2) -> bool:
	var grid: CombatGridPanel = host.get_panel("grid") as CombatGridPanel
	var session: CombatSession = host.session
	var ok: bool = true
	var viewport := Rect2(Vector2.ZERO, view)
	for c: Combatant in _units_in_view(session):
		grid.pin_card(c)
		_sort_now(host)
		grid._place_card()
		var rect: Rect2 = grid.hover_card.get_global_rect()
		var token: Control = grid.grid_view.get_token_for(c)
		ok = ok and grid.hover_card.visible and grid.card_unit == c and grid.hover_card.name_lbl.text == c.name
		ok = ok and viewport.encloses(rect) and not rect.intersects(grid.unit_tile_rect(c)) and token != null and not rect.intersects(token.get_global_rect())
		ok = ok and grid.hover_card.content_fits()
		if not ok:
			print("    card geometry failed for %s: card %s tile %s view %s fits %s" % [c.name, str(rect), str(grid.unit_tile_rect(c)), str(view), str(grid.hover_card.content_fits())])
			break
	grid.close_card()
	return ok

static func _hover_card_checks(tree_parent: Node, checks: Dictionary) -> void:
	if tree_parent == null:
		return
	var saved: Variant = SettingsFile.get_value("ui/text_scale", 1.0)

	# Delay: hovering only schedules the card; it appears when the delay is over, and shows the hovered unit.
	var host: CombatDockHost = _card_host(tree_parent, VIEWS[0], 1.0)
	var grid: CombatGridPanel = host.get_panel("grid") as CombatGridPanel
	var foe: Combatant = host.session.enemies_in_view()[0]
	host.session.set_hovered(foe)
	var waiting: bool = not grid.card_is_open() and grid._pending == foe and is_equal_approx(grid._hover_timer.wait_time, CombatGridPanel.HOVER_DELAY) and CombatGridPanel.HOVER_DELAY == 0.15
	grid._on_hover_delay()
	checks["hover card: appears only after the 150 ms hover delay, for the hovered unit"] = waiting and grid.card_is_open() and grid.card_unit == foe
	# Unpinned: leaving closes it (after the grace), pinned: leaving keeps it, Esc closes it.
	host.session.set_hovered(null)
	grid._on_close_grace()
	var closed_on_exit: bool = not grid.card_is_open()
	host.session.set_hovered(foe)
	grid._on_hover_delay()
	grid.hover_card.pin_btn.pressed.emit()
	host.session.set_hovered(null)
	grid._on_close_grace()
	var kept: bool = grid.card_is_open() and grid.hover_card.is_pinned
	var esc := InputEventKey.new()
	esc.keycode = KEY_ESCAPE
	esc.pressed = true
	host._unhandled_input(esc)
	checks["hover card: an unpinned card closes when the pointer leaves; Pin keeps it on exit; Esc closes it"] = closed_on_exit and kept and not grid.card_is_open() and not grid.hover_card.is_pinned
	# Click on the card pins it; a click elsewhere closes it.
	grid.open_card(foe)
	var card_click := InputEventMouseButton.new()
	card_click.button_index = MOUSE_BUTTON_LEFT
	card_click.pressed = true
	grid.hover_card.gui_input.emit(card_click)
	var pinned_by_click: bool = grid.hover_card.is_pinned
	var away := InputEventMouseButton.new()
	away.button_index = MOUSE_BUTTON_LEFT
	away.pressed = true
	away.global_position = Vector2(-50, -50)
	grid._input(away)
	checks["hover card: a click on the card pins it; a click elsewhere closes it"] = pinned_by_click and not grid.card_is_open()
	# Right click (back, read in _input) closes a pinned card; an unpinned one lets the click through.
	grid.open_card(foe)
	var right := InputEventMouseButton.new()
	right.button_index = MOUSE_BUTTON_RIGHT
	right.pressed = true
	grid._input(right)
	var unpinned_stays: bool = grid.card_is_open()
	grid.pin_card(foe)
	grid._input(right)
	checks["hover card: right click closes a pinned card only"] = unpinned_stays and not grid.card_is_open()
	# Focus on a card opens it at once, losing focus closes an unpinned one.
	var foes: CombatRosterPanel = host.get_panel("enemies") as CombatRosterPanel
	foes._card_for(foe).focus_entered.emit()
	var focus_open: bool = grid.card_is_open() and grid.card_unit == foe
	foes._card_for(foe).focus_exited.emit()
	checks["hover card: keyboard / controller focus on a card opens it, blur closes it"] = focus_open and not grid.card_is_open()
	# A card click pins that unit's card.
	foes._card_for(foe).gui_input.emit(card_click)
	checks["hover card: clicking a hostile card pins its hover card"] = grid.card_is_open() and grid.hover_card.is_pinned and grid.card_unit == foe
	grid.close_card()
	checks["hover card: only a Pin button, no gear / equipment modal path"] = _card_buttons_ok(grid) and host.find_children("*", "CharacterEquipmentModal", true, false).is_empty()
	_card_free(tree_parent, host, saved)

	# Geometry at the three window sizes (Normal text), then at all four text sizes (1280x720).
	var views_ok: bool = true
	for view: Vector2 in VIEWS:
		var view_host: CombatDockHost = _card_host(tree_parent, view, 1.0)
		views_ok = views_ok and _card_geometry_ok(view_host, view)
		_card_free(tree_parent, view_host, saved)
	checks["hover card: never overlaps the hovered tile, stays in the viewport at 1280x720, 1280x800, 1680x720"] = views_ok
	var scales_ok: bool = true
	for scale: float in SettingsFile.TEXT_LEVELS:
		var scale_host: CombatDockHost = _card_host(tree_parent, VIEWS[0], scale)
		scales_ok = scales_ok and _card_geometry_ok(scale_host, VIEWS[0])
		var scale_grid: CombatGridPanel = scale_host.get_panel("grid") as CombatGridPanel
		scale_grid.pin_card(scale_host.session.allies[0])
		scales_ok = scales_ok and scale_grid.hover_card.size.y <= float(scale_host.metrics.tile * CombatHudMetrics.GRID_ROWS)
		_card_free(tree_parent, scale_host, saved)
	checks["hover card: fits its content (no clipping) inside the board at all four text sizes"] = scales_ok


# ------------------------------------------------------------------------------
# COMBAT-UI2-L3: the timeline row's Tactic dropdowns, Autoplay toggle and text sizes (one screen per case)
# ------------------------------------------------------------------------------

static func _key_event(code: Key) -> InputEventKey:
	var key := InputEventKey.new()
	key.keycode = code
	key.pressed = true
	return key

static func _pick(opt: OptionButton, id: String) -> void:
	for i: int in range(opt.item_count):
		if str(opt.get_item_metadata(i)) == id:
			opt.select(i)
			opt.item_selected.emit(i)
			return

static func _autoplay_ui_checks(tree_parent: Node, checks: Dictionary) -> void:
	if tree_parent == null:
		return
	var saved: Variant = SettingsFile.get_value("ui/text_scale", 1.0)

	# Text sizes: the row keeps its fixed height and every control stays inside the dock; font >= 14 px at Normal.
	var fits_ok: bool = true
	var floor_ok: bool = true
	for scale: float in SettingsFile.TEXT_LEVELS:
		var fit_host: CombatDockHost = _card_host(tree_parent, VIEWS[0], scale)
		var fit_panel: CombatTimelinePanel = fit_host.get_panel("timeline") as CombatTimelinePanel
		var dock: TimelineDockUI = fit_panel.dock
		_sort_now(fit_host)
		var need: float = dock.tracks_container.get_combined_minimum_size().y
		var rect: Rect2 = fit_panel.get_global_rect()
		fits_ok = fits_ok and need <= float(CombatHudMetrics.TOP_H) + 0.5 and rect.size.y <= float(CombatHudMetrics.TOP_H) + 0.5
		for node: Node in dock.find_children("*", "Control", true, false):
			var ctrl: Control = node as Control
			if ctrl.is_visible_in_tree() and ctrl.size.x > 0.0:
				if ctrl.get_global_rect().end.x > rect.end.x + 0.5:
					print("    outside the dock at %.2fx: %s %s" % [scale, str(ctrl.get_path()), str(ctrl.get_global_rect())])
				fits_ok = fits_ok and ctrl.get_global_rect().end.x <= rect.end.x + 0.5
		if not fits_ok:
			print("    timeline row at %.2fx needs %.1f px of %d (panel %s)" % [scale, need, CombatHudMetrics.TOP_H, str(rect)])
		var want_fs: int = UiTheme.hud_font()
		for lbl: Label in [dock.plan_info_lbl, dock.ruler_tick_labels[0]]:
			floor_ok = floor_ok and lbl.get_theme_font_size("font_size") == want_fs
		floor_ok = floor_ok and dock.confirm_btn.get_theme_font_size("font_size") == want_fs and dock.hero_option.get_theme_font_size("font_size") == want_fs
		floor_ok = floor_ok and (scale != 1.0 or want_fs == 14)
		_card_free(tree_parent, fit_host, saved)
	checks["timeline row: fixed height, nothing clipped or outside the dock at all four text sizes"] = fits_ok
	checks["timeline row: text uses UiTheme.text_scale() with the 14 px floor (ruler, info, buttons, dropdowns)"] = floor_ok

	# Dropdown contents, Draft menu gone, Tactic picks drive CombatAutoplay.
	var host: CombatDockHost = _card_host(tree_parent, VIEWS[0], 1.0)
	var panel: CombatTimelinePanel = host.get_panel("timeline") as CombatTimelinePanel
	var dock: TimelineDockUI = panel.dock
	var ap: CombatAutoplay = host.autoplay
	var session: CombatSession = host.session
	var hero: Combatant = session.focused_hero
	var entries: Array[Dictionary] = CombatAutoplay.tactic_entries(true)
	var party_ids: Array[String] = []
	for i: int in range(dock.party_option.item_count):
		party_ids.append(str(dock.party_option.get_item_metadata(i)))
	var hero_ids: Array[String] = []
	for i: int in range(dock.hero_option.item_count):
		hero_ids.append(str(dock.hero_option.get_item_metadata(i)))
	var expect_party: Array[String] = []
	var expect_hero: Array[String] = []
	for e: Dictionary in entries:
		expect_hero.append(str(e["id"]))
		if str(e["id"]) != PlanPresets.CUSTOM:
			expect_party.append(str(e["id"]))
	checks["timeline row: party dropdown lists the tactics, hero dropdown the tactics plus Custom; the Draft menu is gone"] = (
		party_ids == expect_party and hero_ids == expect_hero and dock.find_children("*", "MenuButton", true, false).is_empty()
		and dock.hero_option.get_item_tooltip(0) == str(entries[0]["description"]))
	# The party dropdown sets every hero (and clears overrides); the hero dropdown overrides one row.
	var other: String = expect_party[1]
	_pick(dock.party_option, other)
	var party_ok: bool = ap.get_party_tactic() == other and not ap.has_override(hero) and dock.hero_option.get_item_metadata(dock.hero_option.selected) == other
	var row_pick: String = expect_party[2]
	_pick(dock.hero_option, row_pick)
	checks["timeline row: the party dropdown sets every hero, the hero dropdown overrides the shown hero"] = party_ok and ap.has_override(hero) and ap.tactic_of(hero) == row_pick and ap.get_party_tactic() == other
	# One manual edit shows Custom (edited); picking a tactic asks first, Keep leaves it, Redraft restores the row.
	dock.clear_btn.pressed.emit()
	var edited_text: String = dock.hero_option.get_item_text(dock.hero_option.selected)
	var edited_ok: bool = ap.is_edited(hero) and str(dock.hero_option.get_item_metadata(dock.hero_option.selected)) == PlanPresets.CUSTOM and edited_text.ends_with("(edited)")
	_pick(dock.hero_option, expect_party[0])
	var asks: bool = dock.redraft_pending() and ap.is_edited(hero)
	dock.confirm_no_btn.pressed.emit()
	var kept: bool = not dock.redraft_pending() and ap.is_edited(hero) and dock.hero_option.get_item_text(dock.hero_option.selected).ends_with("(edited)")
	_pick(dock.hero_option, expect_party[0])
	dock.confirm_yes_btn.pressed.emit()
	var restored_text: String = dock.hero_option.get_item_text(dock.hero_option.selected)
	checks["timeline row: one manual edit shows Custom (edited); a tactic pick asks before redrafting, Keep keeps, Redraft restores"] = (
		edited_ok and asks and kept and not ap.is_edited(hero) and ap.tactic_of(hero) == expect_party[0] and not restored_text.contains("edited")
		and restored_text == PlanPresets.label(expect_party[0]))
	# Deliberate Custom (no edits yet) applies at once and stays Custom without the edited tag.
	_pick(dock.hero_option, PlanPresets.CUSTOM)
	checks["timeline row: choosing Custom on purpose applies at once and is not tagged edited"] = (
		not dock.redraft_pending() and ap.tactic_of(hero) == PlanPresets.CUSTOM and not ap.is_edited(hero)
		and dock.hero_option.get_item_text(dock.hero_option.selected) == PlanPresets.label(PlanPresets.CUSTOM))
	# Guests: no row picker.
	session.encounter.guests.append(hero)
	panel.refresh()
	var guest_ok: bool = dock.hero_option.disabled
	session.encounter.guests.erase(hero)
	panel.refresh()
	checks["timeline row: a guest's hero dropdown is disabled (guests always play Balanced)"] = guest_ok and not dock.hero_option.disabled
	_card_free(tree_parent, host, saved)

	# Autoplay toggle, A and Space, banner.
	var host2: CombatDockHost = _card_host(tree_parent, VIEWS[0], 1.0)
	var panel2: CombatTimelinePanel = host2.get_panel("timeline") as CombatTimelinePanel
	var dock2: TimelineDockUI = panel2.dock
	var ap2: CombatAutoplay = host2.autoplay
	var starts_off: bool = not ap2.is_on() and not dock2.autoplay_btn.button_pressed
	dock2.autoplay_btn.button_pressed = true
	var toggled_on: bool = ap2.is_on() and dock2.autoplay_btn.button_pressed
	host2._unhandled_input(_key_event(KEY_SPACE))
	var space_pauses: bool = not ap2.is_on() and not dock2.autoplay_btn.button_pressed and ap2.pause_reason() == "Autoplay paused."
	var note_shown: bool = dock2.plan_info_lbl.text == ap2.pause_reason()
	host2._unhandled_input(_key_event(KEY_A))
	var a_on: bool = ap2.is_on() and dock2.autoplay_btn.button_pressed and dock2.plan_info_lbl.text != "Autoplay paused."
	host2._unhandled_input(_key_event(KEY_A))
	var a_off: bool = not ap2.is_on()
	checks["autoplay toggle: starts off, the button turns it on, Space pauses it (banner shows why), A toggles"] = starts_off and toggled_on and space_pauses and note_shown and a_on and a_off
	# Safety stop reason reaches the banner.
	ap2.set_autoplay(true)
	var hero2: Combatant = host2.session.allies[0]
	host2.session.select_combatant(hero2)
	ap2._pause("Autoplay paused: %s is at 22%% HP" % hero2.name, false)
	checks["autoplay toggle: a safety stop's reason shows in the row"] = dock2.plan_info_lbl.text.contains("is at 22% HP") and not dock2.autoplay_btn.button_pressed
	_card_free(tree_parent, host2, saved)

	# Replay / playback hosts (setup without autoplay): A, Space and the toggle never draft or confirm a live round.
	SettingsFile.set_value("ui/text_scale", 1.0)
	var rsession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	var rhost := CombatDockHost.new()
	tree_parent.add_child(rhost)
	rhost.size = VIEWS[0]
	rhost.setup(rsession, false)
	rsession.start()
	var rdock: TimelineDockUI = (rhost.get_panel("timeline") as CombatTimelinePanel).dock
	var round_before: int = rsession.current_round
	rhost._unhandled_input(_key_event(KEY_A))
	rdock.autoplay_btn.button_pressed = true
	rhost._unhandled_input(_key_event(KEY_SPACE))   # plain session play / pause path, not autoplay
	rsession.is_auto_playing = false
	var no_steps: bool = true
	for ally: Combatant in rsession.allies:
		no_steps = no_steps and rsession.plan.steps_of(ally).is_empty()
	checks["replay host: A and the Autoplay toggle are no-ops (no autoplay, no tactic dropdowns, nothing drafted)"] = (
		rhost.autoplay == null and not rdock.autoplay_btn.visible and not rdock.party_option.visible and not rdock.hero_option.visible
		and rsession.is_planning() and not rsession.plan_confirmed and rsession.current_round == round_before and no_steps and rhost._autoplay_timer == null)
	_card_free(tree_parent, rhost, saved)
