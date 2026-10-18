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
		"party": "left_top", "hero": "left", "timeline": "top", "enemies": "right_top",
		"grid": "center", "enemy_card": "right",
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

	# 3. COMBAT-UI3: no floating hover card; the merged enemy card shows the hovered enemy, else the selected (pinned) one
	var brawler_unit: Combatant = session.find_combatant("Enemy Brawler")
	var enemy_card: CombatEnemyCardPanel = host.get_panel("enemy_card") as CombatEnemyCardPanel
	session.select_combatant(brawler_unit)
	var pinned_shown: bool = enemy_card.shown == brawler_unit and enemy_card.name_lbl.text == brawler_unit.name
	var other_foe: Combatant = null
	for foe: Combatant in session.enemies:
		if foe != brawler_unit:
			other_foe = foe
			break
	session.set_hovered(other_foe)
	var hovered_shown: bool = enemy_card.shown == other_foe
	session.set_hovered(null)
	var back_to_pin: bool = enemy_card.shown == brawler_unit
	session.select_combatant(warrior)
	var inspector_ok: bool = pinned_shown and hovered_shown and back_to_pin and host.get_panel("inspector") == null and host.get_panel("forecast") == null \
		and host.find_children("UnitHoverCard", "", true, false).is_empty()

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

	# 4. With nothing hovered or pinned the enemy card shows the planning hero's forecast target
	var target: Combatant = session.forecast_target(warrior)
	enemy_card.refresh()
	var forecast_ok: bool = target != null and enemy_card.shown == target and enemy_card.name_lbl.text == target.name

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
	host.undock("enemy_card")
	var floating: bool = host.is_undocked("enemy_card") and host.region_of("enemy_card") == "right"
	host.dock("enemy_card")
	host.apply_layout("Mirrored")
	var mirrored: bool = host.region_of("party") == "right_top" and host.region_of("enemy_card") == "left" and host.region_of("hero") == "right"
	host.apply_layout("Standard")
	if tree_parent != null:
		var brawler: Combatant = session.find_combatant("Enemy Brawler")
		session.fx_requested.emit(ActionLogEntry.new(1, 2, "ATTACK", warrior.name, brawler.name, "-25 HP", 25, "HIT"))
		session.fx_requested.emit(ActionLogEntry.new(1, 1, "DEFEND", warrior.name, "", "50%", 50, "DEFEND"))
		session.fx_requested.emit(ActionLogEntry.new(1, 1, "BUFF", warrior.name, "", "+50%", 50, "FOCUS"))
		session.fx_requested.emit(ActionLogEntry.new(1, 1, "MOVE", warrior.name, "", "-> (4, 4)", 0, "MOVED", Vector2i(4, 5), Vector2i(4, 4)))
	var layout_ok: bool = floating and not host.is_undocked("enemy_card") and host.region_of("enemy_card") == "right" and mirrored

	print("\n=======================================================")
	print("=== COMBAT UI FRAMEWORK TEST ASSERTIONS             ===")
	print("=======================================================")
	print("  [UI] All 8 combat panels mounted in their dock regions: %s" % ("PASS" if docks_ok else "FAIL"))
	print("  [UI] Tokens & roster bound to the simulated combatants: %s" % ("PASS" if tokens_ok else "FAIL"))
	print("  [UI] Enemy card follows hover, else the pinned enemy; no floating hover card, inspector or forecast panel: %s" % ("PASS" if inspector_ok else "FAIL"))
	print("  [UI] Side panels and grid keep their size for hero and enemy selection, inside the viewport: %s" % ("PASS" if layout_stable else "FAIL"))
	print("  [UI] Enemy card defaults to the planning hero's forecast target: %s" % ("PASS" if forecast_ok else "FAIL"))
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
	for _pass: int in range(3):   # a resize changes the tile: the first pass still sees the old board's minimum width
		_sort_now(host)
	var rects: Array[Vector2] = []
	for id: String in ["party", "grid", "enemy_card"]:
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
# COMBAT-UI3: layout A numbers (derived from the rules in CombatHudMetrics / the spec, not from shipped content)
# ------------------------------------------------------------------------------

const VIEWS: Array[Vector2] = [Vector2(1280, 720), Vector2(1280, 800), Vector2(1680, 720)]
const VIEW_1080: Vector2 = Vector2(1920, 1080)

static func _hud_checks(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	var saved_scale: Variant = SettingsFile.get_value("ui/text_scale", 1.0)

	# Pure metrics: the tile formula, minimum tile, fixed column heights, the strip height, at every size and text size.
	var tile_ok: bool = true
	var heights_ok: bool = true
	var widths_ok: bool = true
	var strip_ok: bool = true
	var all_views: Array[Vector2] = []
	all_views.append_array(VIEWS)
	all_views.append(VIEW_1080)
	for view: Vector2 in all_views:
		var last_left: int = 0
		var last_right: int = 0
		for scale: float in SettingsFile.TEXT_LEVELS:
			for party_n: int in [1, 3]:
				var m: CombatHudMetrics = CombatHudMetrics.compute(view, scale, party_n, 5)
				var expect: int = maxi(48, floori(minf(float(m.area_h - CombatHudMetrics.FRAME) / 9.0, float(m.area_w - CombatHudMetrics.FRAME) / 12.0)))
				tile_ok = tile_ok and m.tile == expect and m.tile >= 48
				heights_ok = heights_ok and m.party_panel_h + m.hero_panel_h + CombatHudMetrics.COL_GAP == m.area_h and m.hero_panel_h >= CombatHudMetrics.HERO_MIN_LINES * m.lh
				heights_ok = heights_ok and m.hostile_panel_h + m.enemy_card_h + CombatHudMetrics.COL_GAP == m.area_h and not m.hostile_scroll
				heights_ok = heights_ok and m.enemy_card_h >= CombatHudMetrics.ENEMY_MIN_LINES * m.lh
				strip_ok = strip_ok and m.top_h >= CombatHudMetrics.STRIP_MIN_H and m.top_h >= m.ctrl_h + 2 * CombatHudMetrics.STRIP_PAD_V and m.ctrl_h > m.lh
				strip_ok = strip_ok and m.area_h + m.top_h + m.bottom_h + 2 * CombatHudMetrics.ROW_SEP + 2 * CombatHudMetrics.MARGIN_V == int(view.y)
			var mw: CombatHudMetrics = CombatHudMetrics.compute(view, scale, 2, 5)
			widths_ok = widths_ok and mw.left_w >= last_left and mw.right_w >= last_right and mw.left_w >= CombatHudMetrics.LEFT_W and mw.right_w >= CombatHudMetrics.RIGHT_W
			widths_ok = widths_ok and mw.left_w + mw.right_w + mw.area_w + 2 * CombatHudMetrics.COL_SEP + 2 * CombatHudMetrics.MARGIN_H == int(view.x)
			last_left = mw.left_w
			last_right = mw.right_w
	checks["tile = floor(min(h/9, w/12)), at least 48, at 4 window sizes x 4 text sizes"] = tile_ok
	checks["party cards + selected-hero panel and hostile cards + enemy card each fill the centre row; no hostile scroll at 5"] = heights_ok
	checks["side columns grow with the text size, the grid takes the rest of the width"] = widths_ok
	checks["the planning strip is one control line (>= 44 px) and the rows add up to the window height"] = strip_ok
	checks["tile_for: a short area is height-bound, a narrow one width-bound, tiny floors at 48"] = (
		CombatHudMetrics.tile_for(Vector2(2000, 540)) == 60 and CombatHudMetrics.tile_for(Vector2(600, 2000)) == 50
		and CombatHudMetrics.tile_for(Vector2(100, 100)) == CombatHudMetrics.MIN_TILE)
	var old_m: CombatHudMetrics = CombatHudMetrics.compute(VIEWS[0], 1.0, 2, 5)
	checks["layout A gives the grid a bigger tile than layout C's 96 px timeline + 220 px columns did"] = old_m.tile > CombatHudMetrics.tile_for(Vector2(1280 - 28 - 440 - 20 - CombatHudMetrics.FRAME, 720 - 8 - 96 - 36 - 8 - CombatHudMetrics.FRAME))

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
			fixed_ok = fixed_ok and host._slot("hero").custom_minimum_size.y == host.metrics.hero_panel_h and host._slot("enemy_card").custom_minimum_size.y == host.metrics.enemy_card_h
			for id: String in ["timeline", "party", "hero", "grid", "enemies", "enemy_card", "action_log", "playback"]:
				var r: Rect2 = host.get_panel(id).get_global_rect()
				inside_ok = inside_ok and r.position.x >= -0.5 and r.end.x <= view.x + 0.5
			session.select_combatant(session.enemies[0])   # selecting a hostile keeps the fixed heights
			fixed_ok = fixed_ok and party.content_fits() and foes.content_fits() and host._slot("party").custom_minimum_size.y == host.metrics.party_panel_h
			session.select_combatant(session.allies[0])
		checks["grid tile fitted to the window (>= 48) at 1280x720, 1280x800, 1680x720"] = sizes_ok
		checks["party, hero, hostile and enemy-card panels keep their fixed heights; every panel inside the window"] = fixed_ok and inside_ok

		# A hero card click selects that hero for the strip and the selected-hero panel.
		var second: Combatant = session.allies[1]
		var card: Control = party._card_for(second)
		var click := InputEventMouseButton.new()
		click.button_index = MOUSE_BUTTON_LEFT
		click.pressed = true
		card.gui_input.emit(click)
		var hero_panel: CombatHeroPanel = host.get_panel("hero") as CombatHeroPanel
		checks["clicking a hero card selects the hero (strip title and selected-hero panel follow)"] = (session.selected_combatant == second and session.focused_hero == second
			and hero_panel.shown == second and (host.get_panel("timeline") as CombatTimelinePanel).dock.title_lbl.text == "%s plan" % second.name)
		# Hovering (or focusing) another hero's card shows that hero in the panel; leaving shows the planned hero again.
		party._card_for(session.allies[0]).mouse_entered.emit()
		var hover_hero: bool = hero_panel.shown == session.allies[0]
		party._card_for(session.allies[0]).mouse_exited.emit()
		checks["the selected-hero panel shows a hovered hero, then the planned hero again"] = hover_hero and hero_panel.shown == second

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
			and not ("btn_gear" in (host.get_panel("playback") as CombatPlaybackPanel).controls))

		# The log is a one-line ticker in the status bar; a click on it or Expand opens the log modal.
		var ticker: ActionLogDockUI = (host.get_panel("action_log") as CombatActionLogPanel).dock
		var ticker_ok: bool = ticker.ticker and ticker.ticker_lbl != null and not ticker.ticker_lbl.text.contains("\n") and host.region_of("action_log") == "bottom_center"
		ticker.btn_expand_log.pressed.emit()
		ticker_ok = ticker_ok and host.log_modal.visible
		host.close_modals()
		var line_click := InputEventMouseButton.new()
		line_click.button_index = MOUSE_BUTTON_LEFT
		line_click.pressed = true
		ticker.active_cell_dock.gui_input.emit(line_click)
		checks["log is a one-line ticker in the status bar; a click on it or Expand opens the log modal"] = ticker_ok and host.log_modal.visible
		host.close_modals()

		# The status bar: "Round n · Planning" and the session's status line as the hint.
		var bar: CombatPlaybackPanel = host.get_panel("playback") as CombatPlaybackPanel
		bar.refresh()
		checks["status bar reads 'Round n · Planning' with the status line as its hint"] = (
			bar.controls.phase_badge_lbl.text == "Round %d · Planning" % session.current_round and bar.controls.status_label.text == session.status_text
			and bar.controls.status_label.visible)

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
		xl_ok = xl_ok and xl_party.content_fits() and xl_foes.content_fits() and xl_host.metrics.party_panel_h + xl_host.metrics.hero_panel_h + CombatHudMetrics.COL_GAP == xl_host.metrics.area_h
		checks["Extra large text: cards still show, grid >= 48, cards fit their fixed heights"] = xl_ok
		tree_parent.remove_child(xl_host)
		xl_host.free()
		xl_session.dispose()
		SettingsFile.set_value("ui/text_scale", saved_scale)

	_strip_checks(tree_parent, checks)
	_confirm_checks(tree_parent, checks)
	_enemy_card_checks(tree_parent, checks)
	_overlap_checks(tree_parent, checks)
	_autoplay_ui_checks(tree_parent, checks)
	_intent_ui_checks(tree_parent, checks)
	_phase_banner_checks(tree_parent, checks)
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
	print("=== COMBAT UI LAYOUT A (COMBAT-UI3) ASSERTIONS      ===")
	print("=======================================================")
	var all_ok: bool = true
	for label: String in checks:
		var ok: bool = DictUtil.to_bool(checks[label])
		all_ok = all_ok and ok
		print("  [UI] %s: %s" % [label, "PASS" if ok else "FAIL"])
	return all_ok


# ------------------------------------------------------------------------------
# COMBAT-UI3: the planning strip, Confirm round's missing-actions warning, the merged enemy card, no overlaps
# (one screen per case: many screens in one arena frame crash)
# ------------------------------------------------------------------------------

const WARN_KEY: String = "ui/warn_missing_actions"

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

# The strip of a host after a layout pass and a refresh (the blocks are rebuilt for the width they got).
static func _strip(host: CombatDockHost) -> TimelineDockUI:
	var panel: CombatTimelinePanel = host.get_panel("timeline") as CombatTimelinePanel
	_sort_now(host)
	panel.refresh()
	_sort_now(host)
	return panel.dock

# For one unit: the strip's blocks cover the round once, in order; each action block is the session's action at its
# start (kind and colour from PlanBlockText, width by blocks); the empty blocks are exactly the plan's empty blocks.
static func _blocks_match(host: CombatDockHost, dock: TimelineDockUI, unit: Combatant) -> bool:
	var session: CombatSession = host.session
	var next: int = 1
	var empty: Array[int] = []
	for block: Control in dock.blocks():
		var start: int = int(block.get_meta("start"))
		var cost: int = int(block.get_meta("blocks"))
		var kind: String = str(block.get_meta("kind"))
		var style: StyleBoxFlat = block.get_theme_stylebox("panel") as StyleBoxFlat
		if start != next or style == null or style.bg_color != PlanBlockText.bg(kind):
			return false
		if block.custom_minimum_size.x != float(cost * dock.block_w + (cost - 1) * CombatHudMetrics.BLOCK_GAP):
			return false
		var act: CombatResolver.ScheduledAction = session.find_recorded_action(unit, start)
		if kind == PlanBlockText.KIND_EMPTY:
			if act != null or cost != 1:
				return false
			empty.append(start)
		elif act == null or PlanBlockText.kind_of(act.command) != kind or act.end_block - act.start_block + 1 != cost or not (str(block.get_meta("form", "text")) == "symbol" or _has_text(block, act.command.command_name)):
			return false
		next += cost
	return next == session.total_blocks + 1 and empty == session.plan_empty_blocks(unit)

static func _strip_checks(tree_parent: Node, checks: Dictionary) -> void:
	if tree_parent == null:
		return
	var saved: Variant = SettingsFile.get_value("ui/text_scale", 1.0)
	var host: CombatDockHost = _card_host(tree_parent, VIEWS[0], 1.0)
	var session: CombatSession = host.session
	var dock: TimelineDockUI = _strip(host)
	var blank_ok: bool = true
	for block: Control in dock.blocks():
		blank_ok = blank_ok and str(block.get_meta("kind")) == PlanBlockText.KIND_EMPTY
	blank_ok = blank_ok and dock.blocks().size() == session.total_blocks and session.plan_empty_blocks(session.focused_hero).size() == session.total_blocks
	checks["strip: an unplanned hero shows one muted empty block per block of the round"] = blank_ok
	session.plan_apply_preset_all(PlanPresets.DEFAULT_TACTIC)
	var kinds_ok: bool = true
	var kinds: Dictionary = {}
	for hero: Combatant in session.allies:
		session.select_combatant(hero)
		dock = _strip(host)
		kinds_ok = kinds_ok and dock.title_lbl.text == "%s plan" % hero.name and _blocks_match(host, dock, hero)
		for block: Control in dock.blocks():
			kinds[str(block.get_meta("kind"))] = true
	checks["strip: the selected hero's plan as continuous blocks, width by blocks, colour by action kind, empty blocks muted"] = kinds_ok and kinds.size() >= 2
	# Hover or focus on a block: the card says how the action plays out.
	session.select_combatant(session.allies[0])
	dock = _strip(host)
	var first: Control = null
	for block: Control in dock.blocks():
		if str(block.get_meta("kind")) != PlanBlockText.KIND_EMPTY:
			first = block
			break
	var card_ok: bool = first != null
	if first != null:
		var act: CombatResolver.ScheduledAction = session.find_recorded_action(session.allies[0], int(first.get_meta("start")))
		first.mouse_entered.emit()
		card_ok = dock.block_card.visible and dock.block_card_title.text == PlanBlockText.title(act) and dock.block_card_body.text == PlanBlockText.body(act) \
			and dock.block_card_cost.text.begins_with(PlanBlockText.cost_line(act))
		first.mouse_exited.emit()
		card_ok = card_ok and not dock.block_card.visible
		first.focus_entered.emit()
		card_ok = card_ok and dock.block_card.visible
		first.focus_exited.emit()
	checks["strip: hover or focus on a block shows how it plays out (title, plain text, blocks and AP)"] = card_ok and not dock.block_card.visible
	# A right click on a block removes its step; the controls sit right of the blocks.
	var steps_before: int = session.plan.steps_of(session.allies[0]).size()
	var right := InputEventMouseButton.new()
	right.button_index = MOUSE_BUTTON_RIGHT
	right.pressed = true
	if first != null:
		first.gui_input.emit(right)
	checks["strip: right-click on a block removes its step"] = session.plan.steps_of(session.allies[0]).size() == steps_before - 1
	dock = _strip(host)
	checks["strip: the tactic controls sit right of the blocks (Party, Hero, Autoplay, Clear, Confirm round)"] = (
		dock.controls_row.get_global_rect().position.x >= dock.blocks_area.get_global_rect().end.x - 0.5
		and dock.controls_row.is_ancestor_of(dock.party_option) and dock.controls_row.is_ancestor_of(dock.hero_option)
		and dock.controls_row.is_ancestor_of(dock.autoplay_btn) and dock.controls_row.is_ancestor_of(dock.clear_btn) and dock.controls_row.is_ancestor_of(dock.confirm_btn)
		and dock.confirm_btn.text == "✔ Confirm round")
	_card_free(tree_parent, host, saved)

# Fills every empty block of every hero with self-targeted steps (whatever the hero may use on itself), so the
# Confirm round warning has nobody to name. False when the fixture cannot fill them (the check then fails loudly).
static func _fill_plans(session: CombatSession) -> bool:
	for hero: Combatant in session.allies:
		var guard: int = 0
		while not session.plan_empty_blocks(hero).is_empty() and guard < 3 * session.total_blocks:
			guard += 1
			var added: bool = false
			for option: PlanOption in session.plan.target_options(hero, hero):
				if option.valid and session.plan_add_step(hero, option.step).is_empty():
					added = true
					break
			if not added:
				break
	return session.heroes_with_empty_blocks().is_empty() and session.plan_is_valid()

static func _confirm_checks(tree_parent: Node, checks: Dictionary) -> void:
	if tree_parent == null:
		return
	var saved: Variant = SettingsFile.get_value("ui/text_scale", 1.0)
	var saved_warn: Variant = SettingsFile.get_value(WARN_KEY)
	checks["setting: 'Warn about missing actions' is a Gameplay checkbox key, on by default"] = (
		SettingsFile.KEYS.has(WARN_KEY) and SettingsFile.default_of(WARN_KEY) == true and str(SettingsModel.LABELS.get(WARN_KEY, "")) == "Warn about missing actions")
	SettingsFile.set_value(WARN_KEY, true)

	# No empty blocks: Confirm round confirms at once.
	var host: CombatDockHost = _card_host(tree_parent, VIEWS[0], 1.0)
	var dock: TimelineDockUI = _strip(host)
	host.session.plan_apply_preset_all(PlanPresets.DEFAULT_TACTIC)
	var filled: bool = _fill_plans(host.session)
	dock.confirm_btn.pressed.emit()
	checks["Confirm round: with no empty blocks it confirms at once (no question)"] = filled and host.session.plan_confirmed and not dock.question_open()
	_card_free(tree_parent, host, saved)

	# Empty blocks: the question names the heroes; Back keeps planning, Confirm confirms; Enter and Esc take the same path.
	host = _card_host(tree_parent, VIEWS[0], 1.0)
	dock = _strip(host)
	var session: CombatSession = host.session
	session.plan_apply_preset_all(PlanPresets.DEFAULT_TACTIC)
	session.plan_clear(session.allies[0])
	var names: Array[String] = []
	for hero: Combatant in session.heroes_with_empty_blocks():
		names.append(hero.name)
	dock.confirm_btn.pressed.emit()
	var asks: bool = dock.question_open() and dock.question.kind == TimelineDockUI.Q_CONFIRM and dock.question.text_lbl.text == TimelineDockUI.missing_text(names) \
		and names.has(session.allies[0].name) and not session.plan_confirmed and dock.confirm_yes_btn.text == "Confirm" and dock.confirm_no_btn.text == "Back"
	var q_rect: Rect2 = dock.question.get_global_rect()
	var placed: bool = Rect2(Vector2.ZERO, VIEWS[0]).encloses(q_rect) and q_rect.position.y >= dock.confirm_btn.get_global_rect().end.y - 0.5
	dock.confirm_no_btn.pressed.emit()
	var back_ok: bool = not dock.question_open() and not session.plan_confirmed and session.is_planning()
	host._unhandled_input(_key_event(KEY_ENTER))
	var enter_asks: bool = dock.question_open() and not session.plan_confirmed
	host._unhandled_input(_key_event(KEY_ESCAPE))
	var esc_closes: bool = not dock.question_open() and not session.plan_confirmed
	dock.confirm_btn.pressed.emit()
	dock.confirm_yes_btn.pressed.emit()
	checks["Confirm round: with empty blocks a small question names the heroes (Confirm / Back), inside the window under the button"] = asks and placed
	checks["Confirm round: Back keeps planning; Enter asks the same question; Esc closes it; Confirm confirms"] = back_ok and enter_asks and esc_closes and session.plan_confirmed and not dock.question_open()
	checks["missing_text: one name 'has', more names 'A and B have', 'A, B and C have'"] = (
		TimelineDockUI.missing_text(["Warrior"]) == "Warrior has empty actions. Confirm anyway?"
		and TimelineDockUI.missing_text(["Warrior", "Ranger"]) == "Warrior and Ranger have empty actions. Confirm anyway?"
		and TimelineDockUI.missing_text(["A", "B", "C"]) == "A, B and C have empty actions. Confirm anyway?")
	_card_free(tree_parent, host, saved)

	# Setting off: confirms at once even with empty blocks.
	SettingsFile.set_value(WARN_KEY, false)
	host = _card_host(tree_parent, VIEWS[0], 1.0)
	dock = _strip(host)
	host.session.plan_apply_preset_all(PlanPresets.DEFAULT_TACTIC)
	host.session.plan_clear(host.session.allies[0])
	var still_empty: bool = not host.session.heroes_with_empty_blocks().is_empty()
	dock.confirm_btn.pressed.emit()
	checks["Confirm round: with the setting off it confirms at once despite empty blocks"] = still_empty and host.session.plan_confirmed and not dock.question_open()
	_card_free(tree_parent, host, saved)
	SettingsFile.set_value(WARN_KEY, saved_warn)

	# The rule behind the warning: empty = not covered by a planned action; none for a hero with a full round.
	var s2 := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	s2.start()
	var hero2: Combatant = s2.allies[0]
	var all_empty: bool = s2.plan_empty_blocks(hero2).size() == s2.total_blocks
	s2.plan_apply_preset_all(PlanPresets.DEFAULT_TACTIC)
	var covered: Dictionary = {}
	for e: PlanEntry in s2.plan.entries_for(hero2):
		for b: int in range(e.start_block, e.end_block() + 1):
			covered[b] = true
	var rule_ok: bool = all_empty
	for b: int in range(1, s2.total_blocks + 1):
		rule_ok = rule_ok and s2.plan_empty_blocks(hero2).has(b) != covered.has(b)
	checks["plan_empty_blocks: the blocks no planned action covers (all of them before planning)"] = rule_ok
	s2.dispose()

# Lines of the merged enemy card derived from the rules (the unit's own numbers, the session's intent and forecast).
static func _enemy_card_checks(tree_parent: Node, checks: Dictionary) -> void:
	if tree_parent == null:
		return
	var saved: Variant = SettingsFile.get_value("ui/text_scale", 1.0)
	var host: CombatDockHost = _card_host(tree_parent, VIEWS[0], 1.0)
	var session: CombatSession = host.session
	var card: CombatEnemyCardPanel = host.get_panel("enemy_card") as CombatEnemyCardPanel
	var hero: Combatant = session.focused_hero
	var ok: bool = true
	var engine_free: bool = true
	var checked: int = 0
	for foe: Combatant in session.enemies_in_view():
		session.set_hovered(foe)
		var rec: Dictionary = session.intent_of(foe)
		var band: Vector2i = SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(foe)
		var reach: String = str(band.y) if band.x == band.y else "%d-%d" % [band.x, band.y]
		ok = ok and card.shown == foe and card.name_lbl.text == foe.name and card.state_lbl.text == card.state_word(foe)
		ok = ok and card.stats_lbl.text == "HP %d/%d · Speed %d · Reach %s" % [foe.hp, foe.max_hp, foe.speed, reach]
		if not rec.is_empty() and str(rec["kind"]) != IntentRules.ARRIVING and str(rec["kind"]) != IntentRules.IDLE:
			ok = ok and card.next_lbl.visible and card.next_lbl.text == "Next: " + IntentText.first_line(rec)
			ok = ok and card.threat_lbl.visible == (rec.get("target", null) == hero)
		var basic: SkillDef = SkillCatalog.lowest(SkillDef.Kind.ATTACK)
		var deals: int = maxi(1, roundi(float(DamageCalculator.expected(hero, foe, basic, basic.stance())) * (1.0 - foe.defense_mitigation_ratio)))
		var takes: int = maxi(1, roundi(float(DamageCalculator.expected(foe, hero, basic, basic.stance())) * (1.0 - hero.defense_mitigation_ratio)))
		ok = ok and card.versus_lbl.text == "Your %s: deals about %d · takes about %d" % [hero.name, deals, takes]
		var cap: int = session.encounter.profile_of(foe).skill_cap
		var names: Array[String] = card.skill_names(foe)
		ok = ok and card.skills_lbl.text.begins_with("Can use: ") and not names.is_empty() and names.size() <= CombatHudMetrics.SKILL_NAMES_MAX + 1
		for skill_name: String in names:
			if skill_name.begins_with("+"):
				continue
			var usable: bool = false
			for s: SkillDef in SkillCatalog.all():
				usable = usable or (s.display_name == skill_name and not s.system and s.level <= cap and foe.can_use(s))
			ok = ok and usable
		for line: String in card.lines():
			for word: String in ["crit", "power", "THREAT", "Guards at", "Flees at", "x1.", "vs "]:
				engine_free = engine_free and not line.contains(word)
		checked += 1
	session.set_hovered(null)
	checks["enemy card: name + state word, HP · Speed · Reach, Next: <tell>, 'Your <hero>: deals about X · takes about Y', Can use: <usable skills>"] = ok and checked >= 1
	checks["enemy card: no engine detail (no crit, power, stance multiplier, threat-intel lines)"] = engine_free
	checks["enemy card: fits its fixed height at Normal text (no scrolling)"] = card.content_fits() and card.custom_minimum_size.y == float(host.metrics.enemy_card_h)
	_card_free(tree_parent, host, saved)

# Nothing overlaps: at 1280x720 in all four text sizes (and 1920x1080) the panels are pairwise disjoint inside the
# window, the strip's controls sit inside the strip, the blocks' room and the controls do not overlap, and the
# fixed-height cards fit.
static func _overlap_checks(tree_parent: Node, checks: Dictionary) -> void:
	if tree_parent == null:
		return
	var saved: Variant = SettingsFile.get_value("ui/text_scale", 1.0)
	var cases: Array[Array] = []
	for scale: float in SettingsFile.TEXT_LEVELS:
		cases.append([VIEWS[0], scale])
	cases.append([VIEW_1080, 1.0])
	for case: Array in cases:
		var view: Vector2 = case[0]
		var scale: float = case[1]
		var host: CombatDockHost = _card_host(tree_parent, view, scale)
		host.session.plan_apply_preset_all(PlanPresets.DEFAULT_TACTIC)
		var dock: TimelineDockUI = _strip(host)
		var ids: Array[String] = ["timeline", "party", "hero", "grid", "enemies", "enemy_card", "playback", "action_log"]
		var rects: Array[Rect2] = []
		var ok: bool = true
		var bad: String = ""
		for id: String in ids:
			var r: Rect2 = host._slot(id).get_global_rect()
			rects.append(r)
			if not Rect2(Vector2.ZERO, view).grow(0.5).encloses(r):
				ok = false
				bad += " %s outside %s;" % [id, str(r)]
		for i: int in range(rects.size()):
			for j: int in range(i + 1, rects.size()):
				if rects[i].grow(-0.5).intersects(rects[j].grow(-0.5)):
					ok = false
					bad += " %s overlaps %s;" % [ids[i], ids[j]]
		var strip: Rect2 = (host.get_panel("timeline") as Control).get_global_rect()
		ok = ok and strip.size.y <= float(host.metrics.top_h) + 0.5
		for node: Node in dock.strip_row.find_children("*", "Control", true, false):
			var ctrl: Control = node as Control
			if ctrl.is_visible_in_tree() and ctrl.size.x > 0.0 and not dock.blocks_area.is_ancestor_of(ctrl):
				if not strip.grow(0.5).encloses(ctrl.get_global_rect()):
					ok = false
					bad += " strip control %s %s;" % [str(ctrl.name), str(ctrl.get_global_rect())]
		ok = ok and not dock.blocks_area.get_global_rect().grow(-0.5).intersects(dock.controls_row.get_global_rect().grow(-0.5))
		var party: CombatRosterPanel = host.get_panel("party") as CombatRosterPanel
		var foes: CombatRosterPanel = host.get_panel("enemies") as CombatRosterPanel
		ok = ok and party.content_fits() and foes.content_fits() and not host.metrics.hostile_scroll and host.metrics.tile >= 48
		if not ok:
			print("    overlap at %s x%.2f:%s" % [str(view), scale, bad])
		checks["no overlap: panels disjoint inside %dx%d at text size %.2f; strip controls inside the strip, cards fit" % [int(view.x), int(view.y), scale]] = ok
		_card_free(tree_parent, host, saved)


# ------------------------------------------------------------------------------
# COMBAT-UI2-L3 (in the COMBAT-UI3 strip): the Tactic dropdowns, Autoplay toggle and text sizes (one screen per case)
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

	# Text sizes: the strip keeps its one-line height and every control stays inside it; font >= 14 px at Normal.
	var fits_ok: bool = true
	var floor_ok: bool = true
	for scale: float in SettingsFile.TEXT_LEVELS:
		var fit_host: CombatDockHost = _card_host(tree_parent, VIEWS[0], scale)
		var fit_panel: CombatTimelinePanel = fit_host.get_panel("timeline") as CombatTimelinePanel
		var dock: TimelineDockUI = _strip(fit_host)
		var need: float = dock.strip_panel.get_combined_minimum_size().y
		var rect: Rect2 = fit_panel.get_global_rect()
		fits_ok = fits_ok and need <= float(fit_host.metrics.top_h) + 0.5 and rect.size.y <= float(fit_host.metrics.top_h) + 0.5
		for node: Node in dock.strip_row.find_children("*", "Control", true, false):
			var ctrl: Control = node as Control
			if ctrl.is_visible_in_tree() and ctrl.size.x > 0.0:
				if ctrl.get_global_rect().end.x > rect.end.x + 0.5:
					print("    outside the strip at %.2fx: %s %s" % [scale, str(ctrl.get_path()), str(ctrl.get_global_rect())])
				fits_ok = fits_ok and ctrl.get_global_rect().end.x <= rect.end.x + 0.5
		if not fits_ok:
			print("    strip at %.2fx needs %.1f px of %d (panel %s)" % [scale, need, fit_host.metrics.top_h, str(rect)])
		var want_fs: int = UiTheme.hud_font()
		var block_lbl: Label = dock.blocks()[0].get_child(0) as Label
		floor_ok = floor_ok and dock.title_lbl.get_theme_font_size("font_size") == want_fs and block_lbl.get_theme_font_size("font_size") == dock.block_font()   # block labels: the smaller block font
		floor_ok = floor_ok and dock.confirm_btn.get_theme_font_size("font_size") == want_fs and dock.hero_option.get_theme_font_size("font_size") == want_fs
		floor_ok = floor_ok and (scale != 1.0 or want_fs == 14)
		floor_ok = floor_ok and dock.party_caption.visible == (scale <= CombatHudMetrics.SHORT_ABOVE)
		_card_free(tree_parent, fit_host, saved)
	checks["strip: one-line height, nothing clipped or outside the strip at all four text sizes"] = fits_ok
	checks["strip: text uses UiTheme.text_scale() with the 14 px floor (title, blocks, buttons, dropdowns); captions drop above Normal+"] = floor_ok

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
	var hint2: Label = (host2.get_panel("playback") as CombatPlaybackPanel).controls.status_label
	var starts_off: bool = not ap2.is_on() and not dock2.autoplay_btn.button_pressed
	dock2.autoplay_btn.button_pressed = true
	var toggled_on: bool = ap2.is_on() and dock2.autoplay_btn.button_pressed
	host2._unhandled_input(_key_event(KEY_SPACE))
	var space_pauses: bool = not ap2.is_on() and not dock2.autoplay_btn.button_pressed and ap2.pause_reason() == "Autoplay paused."
	var note_shown: bool = hint2.text == ap2.pause_reason()
	host2._unhandled_input(_key_event(KEY_A))
	var a_on: bool = ap2.is_on() and dock2.autoplay_btn.button_pressed and hint2.text != "Autoplay paused."
	host2._unhandled_input(_key_event(KEY_A))
	var a_off: bool = not ap2.is_on()
	checks["autoplay toggle: starts off, the button turns it on, Space pauses it (the status bar says why), A toggles"] = starts_off and toggled_on and space_pauses and note_shown and a_on and a_off
	# Safety stop reason reaches the banner.
	ap2.set_autoplay(true)
	var hero2: Combatant = host2.session.allies[0]
	host2.session.select_combatant(hero2)
	ap2._pause("Autoplay paused: %s is at 22%% HP" % hero2.name, false)
	checks["autoplay toggle: a safety stop's reason shows in the status bar"] = hint2.text.contains("is at 22% HP") and not dock2.autoplay_btn.button_pressed
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


# ------------------------------------------------------------------------------
# INTENT1-U: enemy intent tells in the combat UI (one screen per case; the fixture is IntentTestSuite's scripted battle,
# expectations come from the scheduled actions and the spec's rules, never from shipped content)
# ------------------------------------------------------------------------------

static func _intent_host(tree_parent: Node, scale: float = 1.0, scripted: bool = true) -> CombatDockHost:
	SettingsFile.set_value("ui/text_scale", scale)
	var session: CombatSession = null
	if scripted:
		session = CombatSession.new(IntentTestSuite.ScriptedEncounter.new())
	else:
		session = CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	var host := CombatDockHost.new()
	tree_parent.add_child(host)
	host.size = VIEWS[0]
	host.setup(session)
	session.start()
	host._fit()
	_sort_now(host)
	return host

static func _unit(host: CombatDockHost, unit_name: String) -> Combatant:
	return host.session.find_combatant(unit_name)

# All Label texts under a node.
static func _texts(node: Node) -> Array[String]:
	var out: Array[String] = []
	for raw: Node in node.find_children("*", "Label", true, false):
		var lbl: Label = raw as Label
		out.append(lbl.text)
	return out

static func _has_text(node: Node, part: String) -> bool:
	for text: String in _texts(node):
		if text.contains(part):
			return true
	return false

# Arrows the records ask for: a target on the grid or an aim cell, other than the enemy's own tile.
static func _expected_arrows(session: CombatSession) -> int:
	var n: int = 0
	for rec: Dictionary in session.enemy_intents():
		var unit: Combatant = rec["unit"]
		var aim: Vector2i = rec["cell"]
		var target: Variant = rec["target"]
		if target is Combatant:
			var foe: Combatant = target
			if session.grid.has_combatant(foe) and session.grid.get_position(foe) != session.grid.get_position(unit):
				n += 1
		elif aim != IntentRules.NO_CELL and aim != session.grid.get_position(unit):
			n += 1
	return n

# "B4" or "B4-5": the blocks of a scheduled action.
static func _span(a: CombatResolver.ScheduledAction) -> String:
	return "B%d" % a.start_block if a.end_block == a.start_block else "B%d-%d" % [a.start_block, a.end_block]

static func _first_scheduled(session: CombatSession, unit: Combatant) -> CombatResolver.ScheduledAction:
	return IntentRules.actions_of(unit, session.resolver.scheduled_actions)[0]

# The merged enemy card's lines after showing `foe` on it (pinned: selected).
static func _enemy_lines(host: CombatDockHost, foe: Combatant) -> Array[String]:
	var card: CombatEnemyCardPanel = host.get_panel("enemy_card") as CombatEnemyCardPanel
	host.session.select_combatant(foe)
	var out: Array[String] = []
	for line: String in card.lines():
		for part: String in line.split("\n"):
			out.append(part)
	return out

static func _lines_have(lines: Array[String], part: String) -> bool:
	for line: String in lines:
		if line.contains(part):
			return true
	return false

static func _intent_ui_checks(tree_parent: Node, checks: Dictionary) -> void:
	if tree_parent == null:
		return
	var saved: Variant = SettingsFile.get_value("ui/text_scale", 1.0)
	var old_path: String = SettingsFile.path()
	var sandbox: String = "user://intent_ui_settings.cfg"
	SettingsFile.use_path(sandbox)   # the I hotkey saves the setting: keep it out of the player's file

	# Planning: a chip per enemy on the grid (its kind's ui texture) and the arrows the records ask for; the setting defaults on.
	var host: CombatDockHost = _intent_host(tree_parent)
	var grid: CombatGridPanel = host.get_panel("grid") as CombatGridPanel
	var overlay: IntentOverlay = grid.grid_view.intent_overlay
	var records: Array[Dictionary] = host.session.enemy_intents()
	var chips_ok: bool = records.size() > 0 and overlay.chip_count() == records.size()
	for rec: Dictionary in records:
		var unit: Combatant = rec["unit"]
		var chip_raw: Variant = overlay.chips.get(unit, null)
		if chip_raw is TextureRect:
			var chip: TextureRect = chip_raw
			chips_ok = chips_ok and chip.texture == AssetRegistry.texture("ui", "intent_" + str(rec["kind"]), "icon")
		else:
			chips_ok = false
	checks["intent tells: planning shows a kind chip on every enemy token (ui key intent_<kind>)"] = chips_ok and SettingsFile.get_bool("ui/intent_tells") and grid.tells_on()
	checks["intent tells: an arrow per target or aim cell, dashed for move and flee"] = _expected_arrows(host.session) > 0 and overlay.arrow_count() == _expected_arrows(host.session)
	var move_dashed: int = 0
	var move_recs: int = 0
	for rec: Dictionary in records:
		if str(rec["kind"]) == "move" and (rec["target"] is Combatant or rec["cell"] != IntentRules.NO_CELL):
			move_recs += 1
	for arrow: Dictionary in overlay.arrows:
		if DictUtil.to_bool(arrow["dashed"]):
			move_dashed += 1
	checks["intent tells: dashed arrows are exactly the move tells"] = move_dashed == move_recs and move_recs > 0

	# I toggles the tells and remembers it; the second press brings them back.
	host._unhandled_input(_key_event(KEY_I))
	var hidden: bool = overlay.chip_count() == 0 and overlay.arrow_count() == 0 and not SettingsFile.get_bool("ui/intent_tells")
	host._unhandled_input(_key_event(KEY_I))
	var shown: bool = overlay.chip_count() == records.size() and overlay.arrow_count() > 0 and SettingsFile.get_bool("ui/intent_tells")
	checks["intent tells: I hides and shows the chips and arrows (remembered in the settings)"] = hidden and shown

	# Resolution: the confirmed round hides them.
	host.session.plan_apply_preset_all("balanced")
	host.session.confirm_plan(false)
	checks["intent tells: hidden once the round is confirmed (resolution)"] = not host.session.is_planning() and overlay.chip_count() == 0 and overlay.arrow_count() == 0
	_card_free(tree_parent, host, saved)

	# Hostile cards: the "Next:" line and the eye badge of a read foe (a standard battle; every hero holds the foresight perk).
	var host2: CombatDockHost = _intent_host(tree_parent, 1.0, false)
	for ally: Combatant in host2.session.allies:
		Passives.apply(ally, IntentTestSuite.EYE)
	var foes: CombatRosterPanel = host2.get_panel("enemies") as CombatRosterPanel
	foes.refresh()
	var next_ok: bool = CombatHudMetrics.HOSTILE_ROWS >= 2   # the "Next:" line is the card's second row
	var read_foes: int = 0
	var attack_named: bool = false
	for rec: Dictionary in host2.session.enemy_intents():
		var unit2: Combatant = rec["unit"]
		var card: Control = foes._card_for(unit2)
		var beh: Label = card.find_child("BehaviourLabel", true, false) as Label
		var badge: Control = card.find_child("ForesightBadge", true, false) as Control
		var blocks: Vector2i = rec["blocks"]
		var bare: String = "Arriving" if str(rec["kind"]) == IntentRules.ARRIVING else ("Holding" if str(rec["kind"]) == IntentRules.IDLE else "")
		var first: String = bare if not bare.is_empty() else "%s (%s)" % [IntentText.core(rec), IntentText.block_text(blocks)]
		next_ok = next_ok and beh != null and beh.text == (bare if not bare.is_empty() else "Next: " + first) and badge != null and badge.visible == DictUtil.to_bool(rec["revealed"])
		if DictUtil.to_bool(rec["revealed"]):
			read_foes += 1
		var target: Variant = rec["target"]
		if target is Combatant:   # the line names the target and the block
			var tgt: Combatant = target
			attack_named = attack_named or (beh.text.begins_with("Next: ") and beh.text.contains(tgt.name) and beh.text.contains("(B%d" % blocks.x))
	checks["hostile cards: the behaviour line reads 'Next: <skill> on <target> (B<n>)', a read foe wears the eye badge"] = next_ok and read_foes > 0 and attack_named
	_card_free(tree_parent, host2, saved)

	# Hostile card name row: HP right-aligned at its right end, readable (not clipped) at all four text sizes, "Next:" below.
	var hp_ok: bool = true
	for scale: float in SettingsFile.TEXT_LEVELS:
		var hp_host: CombatDockHost = _intent_host(tree_parent, scale, false)
		var hp_foes: CombatRosterPanel = hp_host.get_panel("enemies") as CombatRosterPanel
		hp_foes.refresh()
		_sort_now(hp_host)
		var seen: int = 0
		for foe: Combatant in hp_host.session.enemies_in_view():
			var foe_card: Control = hp_foes._card_for(foe)
			var hp_lbl: Label = foe_card.find_child("HpLabel", true, false) as Label
			var name_lbl: Label = foe_card.find_child("NameLabel", true, false) as Label
			var next_lbl: Label = foe_card.find_child("BehaviourLabel", true, false) as Label
			if hp_lbl == null or name_lbl == null or next_lbl == null:
				hp_ok = false
				continue
			seen += 1
			var font: Font = hp_lbl.get_theme_font("font")
			var text_w: float = font.get_string_size(hp_lbl.text, HORIZONTAL_ALIGNMENT_LEFT, -1, hp_lbl.get_theme_font_size("font_size")).x
			hp_ok = hp_ok and hp_lbl.text == "%d/%d" % [foe.hp, foe.max_hp] and hp_lbl.size.x >= text_w - 0.5 and hp_lbl.size.x > 0.0
			hp_ok = hp_ok and hp_lbl.get_global_rect().end.x <= foe_card.get_global_rect().end.x + 0.5 and hp_lbl.get_global_rect().position.x >= name_lbl.get_global_rect().end.x - 0.5
			hp_ok = hp_ok and absf(hp_lbl.get_global_rect().position.y - name_lbl.get_global_rect().position.y) < 1.0 and next_lbl.get_global_rect().position.y >= name_lbl.get_global_rect().end.y - 0.5
		hp_ok = hp_ok and seen > 0 and hp_foes.content_fits()
		_card_free(tree_parent, hp_host, saved)
	checks["hostile cards: HP 'x/y' sits right-aligned on the name row, unclipped, with the Next line below, at all four text sizes"] = hp_ok

	# Tactic drafts fill the leftover blocks (the strip's pickers and autoplay pass fill_blocks; sims and enemies do not).
	var fill_host: CombatDockHost = _intent_host(tree_parent, 1.0, false)
	var fs: CombatSession = fill_host.session
	fs.plan_apply_preset_all("balanced")
	var bare_empty: int = fs.heroes_with_empty_blocks().size()
	var bare_steps: Dictionary = {}
	for hero: Combatant in fs.allies:
		bare_steps[hero] = fs.plan.steps_of(hero).size()
	var fill_ok: bool = true
	for id: String in PlanPresets.AUTO_IDS:
		fs.plan_apply_preset_all(id, true)
		fill_ok = fill_ok and fs.plan_is_valid()
		if id == "balanced":
			fill_ok = fill_ok and fs.heroes_with_empty_blocks().is_empty()
			for hero: Combatant in fs.allies:
				fill_ok = fill_ok and fs.plan.steps_of(hero).size() >= int(bare_steps[hero])
	fs.plan_apply_preset_all("balanced", true)
	var hero_pick: Combatant = fs.allies[fs.allies.size() - 1]
	fs.plan_clear(hero_pick)
	fs.plan_apply_preset(hero_pick, "balanced", true)
	fill_ok = fill_ok and fs.plan_empty_blocks(hero_pick).is_empty() and fs.plan.is_valid(hero_pick)
	checks["tactic drafts with fill_blocks leave no empty block under Balanced (party and one hero), stay valid under every tactic, never lose steps (bare draft left %d hero(s) empty)" % bare_empty] = fill_ok
	_card_free(tree_parent, fill_host, saved)

	# Strip blocks: a smaller label font; a name that does not fit shows the kind symbol instead (never an ellipsis),
	# every block keeps the same height; the symbol form has the name on the hover card.
	var blk_ok: bool = true
	var forms: Array[String] = []
	for scale: float in SettingsFile.TEXT_LEVELS:
		var bh: CombatDockHost = _intent_host(tree_parent, scale, false)
		bh.session.plan_apply_preset_all("balanced", true)
		var bdock: TimelineDockUI = _strip(bh)
		var n_text: int = 0
		var n_sym: int = 0
		var h0: float = -1.0
		for blk: Control in bdock.blocks():
			var lbl: Label = blk.get_child(0) as Label
			if lbl == null:
				continue
			blk_ok = blk_ok and lbl.get_theme_font_size("font_size") == bdock.block_font() and bdock.block_font() < bdock.fs()
			h0 = blk.size.y if h0 < 0.0 else h0
			blk_ok = blk_ok and is_equal_approx(blk.size.y, h0)
			var form: String = str(blk.get_meta("form", "text"))
			if form == "symbol":
				n_sym += 1
				blk_ok = blk_ok and lbl.text.length() <= 4
			else:
				n_text += 1
				var font: Font = lbl.get_theme_font("font")
				blk_ok = blk_ok and font.get_string_size(lbl.text, HORIZONTAL_ALIGNMENT_LEFT, -1, lbl.get_theme_font_size("font_size")).x <= lbl.size.x + 0.5
		forms.append("%.2f: %d text / %d symbol" % [scale, n_text, n_sym])
		_card_free(tree_parent, bh, saved)
	print("STRIP FORMS ", forms)
	checks["strip blocks: small label font, uniform height, full text fits or the kind symbol replaces it (all four text sizes)"] = blk_ok

	# Enemy card: "Next:" with the first action; a read foe lists its whole plan under "Read by ...".
	var host3: CombatDockHost = _intent_host(tree_parent)
	var walker: Combatant = _unit(host3, "Walker")
	var walker_lines: Array[String] = _enemy_lines(host3, walker)
	var acts: Array[CombatResolver.ScheduledAction] = IntentRules.actions_of(walker, host3.session.resolver.scheduled_actions)
	var plan_re := RegEx.new()
	plan_re.compile("^B\\d+(-\\d+)? ")
	var plan_labels: int = 0
	for text: String in walker_lines:
		if plan_re.search(text) != null:
			plan_labels += 1
	var plan_ok: bool = acts.size() >= 2 and plan_labels == acts.size()   # one line per scheduled action
	for a: CombatResolver.ScheduledAction in acts:
		if a.command is AttackCommand:
			var atk: AttackCommand = a.command
			plan_ok = plan_ok and walker_lines.has("%s %s on %s" % [_span(a), atk.command_name, atk.target.name])
	var read_ok: bool = _lines_have(walker_lines, "Next: ") and _lines_have(walker_lines, "Read by Bryn")
	var first: Dictionary = host3.session.intent_of(walker)
	var first_blocks: Vector2i = first["blocks"]
	var first_ok: bool = _lines_have(walker_lines, "(B%d" % first_blocks.x)
	var cutter_lines: Array[String] = _enemy_lines(host3, _unit(host3, "Cutter"))
	var plain_ok: bool = _lines_have(cutter_lines, "Next: ") and _lines_have(cutter_lines, "on Sella (%s)" % _span(_first_scheduled(host3.session, _unit(host3, "Cutter")))) and not _lines_have(cutter_lines, "Read by")
	checks["enemy card: a hostile shows 'Next:' with its first action; a read foe lists its whole plan under 'Read by Bryn (...)'"] = read_ok and plan_ok and first_ok and plain_ok
	_card_free(tree_parent, host3, saved)

	# Timeline: a foe that is not read keeps the masked banner with its block 1 tell; a read foe's track shows unmasked and read-only.
	var host4: CombatDockHost = _intent_host(tree_parent)
	var dock: TimelineDockUI = (host4.get_panel("timeline") as CombatTimelinePanel).dock
	var cutter4: Combatant = _unit(host4, "Cutter")
	host4.session.select_combatant(cutter4)
	var rec4: Dictionary = host4.session.intent_of(cutter4)
	var masked_ok: bool = dock.enemy_classified_banner.visible and not dock.track_clips_hbox.visible and dock.banner_lbl.text.contains("plan in secret") and dock.banner_lbl.text.contains(IntentText.core(rec4)) and not DictUtil.to_bool(rec4["revealed"])
	var stalker4: Combatant = _unit(host4, "Stalker")
	host4.session.select_combatant(stalker4)
	var action_clips: int = 0
	for clip: Node in dock.track_clips_hbox.get_children():
		if _has_text(clip, "Attack"):
			action_clips += 1
	checks["timeline: a foe's masked banner shows its block 1 tell; a read foe's track is unmasked and read-only"] = (
		masked_ok and not dock.enemy_classified_banner.visible and dock.track_clips_hbox.visible and dock.read_only and action_clips == 1)
	host4.session.select_combatant(host4.session.allies[0])
	checks["timeline: selecting a hero again restores the editable track"] = not dock.read_only and not dock.enemy_classified_banner.visible
	_card_free(tree_parent, host4, saved)

	# Forecast: the planning hero is told when a hovered foe's tell aims at it.
	var host5: CombatDockHost = _intent_host(tree_parent)
	var fc: CombatEnemyCardPanel = host5.get_panel("enemy_card") as CombatEnemyCardPanel
	var bryn: Combatant = _unit(host5, "Bryn")
	host5.session.select_combatant(bryn)
	host5.session.set_hovered(_unit(host5, "Stalker"))
	var told: bool = fc.threat_lbl.visible and fc.threat_lbl.text == "Targets you in %s" % _span(_first_scheduled(host5.session, _unit(host5, "Stalker")))
	host5.session.set_hovered(_unit(host5, "Cutter"))   # aims at Sella
	var quiet: bool = not fc.threat_lbl.visible
	checks["forecast: 'Targets you in B<n>' only when the hovered foe's tell aims at the planning hero"] = told and quiet
	_card_free(tree_parent, host5, saved)

	# Display only: the battle resolves the same with the tells on and off.
	var outcomes: Array[Array] = []
	for on: bool in [true, false]:
		var host6: CombatDockHost = _intent_host(tree_parent, 1.0, false)
		var grid6: CombatGridPanel = host6.get_panel("grid") as CombatGridPanel
		grid6.set_tells(on, false)
		host6.session.plan_apply_preset_all("balanced")
		host6.session.confirm_plan(false)
		TestHelpers.resolve_until_replay(host6.session)
		outcomes.append(IntentTestSuite._outcome(host6.session))
		_card_free(tree_parent, host6, saved)
	SettingsFile.set_value("ui/intent_tells", true)
	checks["intent tells: combat results are identical with the tells on and off"] = outcomes[0] == outcomes[1] and outcomes[0].size() > 6

	# Layout: at 1280x720 in all four text sizes the chips and arrows sit under the hover card and inside the board.
	var layers_ok: bool = true
	for scale: float in SettingsFile.TEXT_LEVELS:
		var host7: CombatDockHost = _intent_host(tree_parent, scale, false)
		var grid7: CombatGridPanel = host7.get_panel("grid") as CombatGridPanel
		var ov: IntentOverlay = grid7.grid_view.intent_overlay
		var board: Rect2 = Rect2(Vector2.ZERO, Vector2(grid7.grid_view.tile_px * CombatGridView.GRID_W, grid7.grid_view.tile_px * CombatGridView.GRID_H))
		layers_ok = layers_ok and ov.chip_count() > 0
		for chip_raw: Variant in ov.chips.values():
			var chip7: TextureRect = chip_raw
			layers_ok = layers_ok and board.encloses(Rect2(chip7.position, chip7.size)) and chip7.mouse_filter == Control.MOUSE_FILTER_IGNORE
		for arrow7: Dictionary in ov.arrows:
			var from7: Vector2i = arrow7["from"]
			var to7: Vector2i = arrow7["to"]
			layers_ok = layers_ok and board.has_point(ov._center(from7)) and board.has_point(ov._center(to7))
		layers_ok = layers_ok and ov.mouse_filter == Control.MOUSE_FILTER_IGNORE
		for ally7: Combatant in host7.session.allies:   # a read foe's enemy card lists its whole plan
			Passives.apply(ally7, IntentTestSuite.EYE)
		host7.session.changed.emit()
		var read7: int = 0
		for rec7: Dictionary in host7.session.enemy_intents():
			if DictUtil.to_bool(rec7["revealed"]):
				read7 += 1
				var foe7: Combatant = rec7["unit"]
				layers_ok = layers_ok and _lines_have(_enemy_lines(host7, foe7), "Read by")
		host7.session.select_combatant(host7.session.allies[0])
		layers_ok = layers_ok and read7 > 0
		var tile_before: int = host7.metrics.tile
		grid7.set_tells(false, false)
		layers_ok = layers_ok and host7.metrics.tile == tile_before   # the tells never change the layout
		SettingsFile.set_value("ui/intent_tells", true)
		_card_free(tree_parent, host7, saved)
	checks["intent tells: chips and arrows stay on the board and off the layout; a read foe's card lists its plan, at all four text sizes"] = layers_ok

	SettingsFile.use_path(old_path)
	DirAccess.remove_absolute(sandbox)

# ------------------------------------------------------------------------------
# BOSS1-U: the phase banner (one screen per case; the phase info is a test dictionary, never shipped content)
# ------------------------------------------------------------------------------

const PHASE_LINE: String = "The ground shakes as the warlord roars: you will not leave this hall alive, every last one of you, not while my banner stands tall above the broken field!"

static func _fire_phase(host: CombatDockHost) -> void:
	var entry := ActionLogEntry.new(1, 0, "EVENT", "Test Boss", "", PHASE_LINE, 0, "")
	host.session.phase_fired.emit(entry, {"id": "rage", "name": "Warlord's Rage", "actor": "Test Boss", "line": PHASE_LINE, "pause_autoplay": true})

static func _phase_banner_checks(tree_parent: Node, checks: Dictionary) -> void:
	if tree_parent == null:
		return
	var saved: Variant = SettingsFile.get_value("ui/text_scale", 1.0)

	# Appears with the name and the line; hides when its timer ends.
	var host: CombatDockHost = _card_host(tree_parent, VIEWS[0], 1.0)
	var none_before: bool = not host.is_phase_banner_visible()
	_fire_phase(host)
	var texts: Array[String] = _texts(host.get_node("PhaseBanner"))
	checks["phase banner: shows the phase name and line when phase_fired emits"] = none_before and host.is_phase_banner_visible() and texts.has("Warlord's Rage") and texts.has(PHASE_LINE)
	var timer: Timer = null
	for child: Node in host.get_children():
		if child is Timer and (child as Timer).one_shot:
			timer = child
	checks["phase banner: a one-shot timer of CombatDockHost.PHASE_BANNER_SECONDS runs"] = timer != null and is_equal_approx(timer.wait_time, CombatDockHost.PHASE_BANNER_SECONDS)
	if timer != null:
		timer.timeout.emit()
	checks["phase banner: hides when its timer ends"] = not host.is_phase_banner_visible()
	_card_free(tree_parent, host, saved)

	# Any key skips it (and does nothing else).
	host = _card_host(tree_parent, VIEWS[0], 1.0)
	_fire_phase(host)
	var round_before: int = host.session.viewed_round
	host._unhandled_input(_key_event(KEY_X))
	checks["phase banner: any key skips it"] = not host.is_phase_banner_visible() and host.session.viewed_round == round_before
	_card_free(tree_parent, host, saved)

	# Never during replay scrubbing.
	host = _card_host(tree_parent, VIEWS[0], 1.0)
	var live_phase: CombatSession.Phase = host.session.phase
	host.session.phase = CombatSession.Phase.REPLAY
	_fire_phase(host)
	var replay_hidden: bool = not host.is_phase_banner_visible()
	host.session.phase = live_phase
	checks["phase banner: not shown while a replay plays"] = replay_hidden
	_card_free(tree_parent, host, saved)

	# Fits 1280x720 at every text size: inside the viewport, the panel's minimum height under the view.
	for scale: float in SettingsFile.TEXT_LEVELS:
		host = _card_host(tree_parent, VIEWS[0], scale)
		_fire_phase(host)
		_sort_now(host)
		var panel: PanelContainer = host.get_node("PhaseBanner/PhasePanel")
		var name_label: Label = host.get_node("PhaseBanner/PhasePanel/PhaseColumn/PhaseName")
		var line_label: Label = host.get_node("PhaseBanner/PhasePanel/PhaseColumn/PhaseLine")
		# The wrapped height from the font (a one-shot headless layout pass leaves wrapped labels at a transient height).
		var text_w: float = line_label.custom_minimum_size.x
		var name_h: float = name_label.get_theme_font("font").get_multiline_string_size(name_label.text, HORIZONTAL_ALIGNMENT_CENTER, text_w, name_label.get_theme_font_size("font_size")).y
		var line_h: float = line_label.get_theme_font("font").get_multiline_string_size(line_label.text, HORIZONTAL_ALIGNMENT_CENTER, text_w, line_label.get_theme_font_size("font_size")).y
		var need_h: float = name_h + line_h + 6.0 + 28.0 + 4.0   # label gap, the style's vertical margins, the 2 px borders
		var rect: Rect2 = panel.get_global_rect()
		var view_rect := Rect2(Vector2.ZERO, VIEWS[0])
		checks["phase banner: fits 1280x720 at text size %.2f" % scale] = panel.custom_minimum_size.x <= VIEWS[0].x - 64.0 and need_h <= VIEWS[0].y - 64.0 and rect.position.x >= 0.0 and rect.end.x <= view_rect.end.x and line_h > name_h
		_card_free(tree_parent, host, saved)
	SettingsFile.set_value("ui/text_scale", saved)
