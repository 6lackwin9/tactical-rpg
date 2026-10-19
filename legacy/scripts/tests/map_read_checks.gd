class_name MapReadChecks
extends RefCounted

# ==============================================================================
# Tactical RPG - Readable maps checks (MAPREAD1-L2). Pure (run_pure): every marker overlay, pin and badge key resolves
# through AssetRegistry without a fallback, icon key / opacity / hint / legend-order / name-placement rules, the world
# map's legend, card and name placement stay inside their bounds at the four text sizes. Scene (run_region): a region
# view on a location with a marker of every kind (ExplorationTestSuite fixture): legend rows and order, hover and facing
# show `label . verb`, the letter glyphs are gone, names / hint / legend fit at 1280x720 at all four text sizes, a legend
# row hover pulses its tile, point notes replace the raw flag line, a used-up chest says it is empty. Expectations are
# derived from the rules (the fixture's own markers), not from shipped content.
# ==============================================================================

const TAG: String = "MapRead"

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func _theme_for(scale: float) -> Theme:
	var t := Theme.new()
	t.default_font_size = roundi(MainMenuView.BASE_FONT * scale)
	return t

static func _resolves(ns: String, key: String, variant: String) -> bool:
	var r: Dictionary = AssetRegistry.resolve(ns, key, variant)
	var tex: Texture2D = AssetRegistry.texture(ns, key, variant)
	return str(r["source"]) != "fallback" and tex != null and tex.get_width() > 16 and tex.get_height() > 16

static func run_pure() -> bool:
	var checks: Dictionary = {}
	var tile_ok: bool = true
	for key: String in AssetRegistry.TILE_OVERLAY_KEYS:
		tile_ok = tile_ok and _resolves("tile", key, "v0")
	checks["every marker overlay key resolves (template or file), no fallback"] = tile_ok and AssetRegistry.TILE_OVERLAY_KEYS.size() >= 9
	var wanted: Array[String] = ["overlay_npc", "overlay_chest", "overlay_chest_open", "overlay_lore", "overlay_exit", "overlay_exit_locked", "overlay_encounter", "overlay_roamer", "overlay_poi"]
	checks["the art contract's nine overlay keys exist"] = wanted.all(func(k: String) -> bool: return AssetRegistry.has_key("tile", k))
	var ui_ok: bool = true
	var contract: Array[String] = ["badge_quest", "badge_ready", "pin_fortress", "pin_town", "pin_camp", "pin_ruin", "pin_gate", "pin_wild", "pin_here"]
	for key: String in contract:
		ui_ok = ui_ok and AssetRegistry.has_key("ui", key) and _resolves("ui", key, "icon")
	checks["every pin and badge key resolves through the ui namespace"] = ui_ok
	var pins_ok: bool = true
	for kind: String in MapArt.PIN_KINDS:
		pins_ok = pins_ok and MapArt.pin_key(kind) == "pin_" + kind and AssetRegistry.has_key("ui", MapArt.pin_key(kind))
	checks["a pin key per location kind; an unknown kind has none"] = pins_ok and MapArt.pin_key("moon").is_empty()
	var distinct: Dictionary = {}
	for key: String in AssetRegistry.TILE_OVERLAY_KEYS:
		distinct[AssetRegistry.template_svg("tile", key, "v0")] = true
	checks["each overlay has its own picture"] = distinct.size() == AssetRegistry.TILE_OVERLAY_KEYS.size()
	var glyphs: Dictionary = RegionMapView.new().get_script().get_script_constant_map()
	checks["the letter glyphs are gone"] = not glyphs.has("MARKER_GLYPHS") and not glyphs.has("GLYPH_SIZE")
	var spent_chest: Dictionary = {"kind": "chest", "state": "spent", "icon": "overlay_chest"}
	var locked_exit: Dictionary = {"kind": "exit", "state": "locked", "icon": "overlay_exit"}
	checks["icon key: spent chest opens, locked exit shows the padlock, others keep marker_view's"] = RegionMapView.icon_key(spent_chest) == "overlay_chest_open" and RegionMapView.icon_key(locked_exit) == "overlay_exit_locked" and RegionMapView.icon_key({"kind": "npc", "state": "present", "icon": "overlay_npc"}) == "overlay_npc"
	checks["spent = 40% opacity"] = is_equal_approx(RegionMapView.icon_alpha(spent_chest, 0.3), RegionMapView.SPENT_ALPHA)
	var beat: bool = is_equal_approx(RegionMapView.pulse_alpha(0.0), RegionMapView.pulse_alpha(RegionMapView.PULSE_PERIOD)) and RegionMapView.pulse_alpha(0.0) > RegionMapView.pulse_alpha(RegionMapView.PULSE_PERIOD / 2.0)
	checks["roamer icon pulses once per period; other present icons stay opaque"] = beat and RegionMapView.icon_alpha({"kind": "roamer", "state": "present"}, 0.5) < 1.0 and RegionMapView.icon_alpha({"kind": "npc", "state": "present"}, 0.5) == 1.0
	checks["hint line = label . verb"] = RegionMapView.hint_line({"label": "Mara", "verb": "Talk"}) == "Mara · Talk"
	var rows: Array[Dictionary] = []
	for kind: String in ["roamer", "point", "exit", "npc", "chest", "lore", "encounter"]:
		rows.append({"kind": kind, "label": "b"})
	rows.append({"kind": "npc", "label": "a"})
	rows.sort_custom(RegionMapView.legend_less)
	var order: Array[String] = []
	for r: Dictionary in rows:
		order.append(str(r["kind"]))
	checks["legend order: npcs first (by name), exits, chests, lore, points, fights"] = order == ["npc", "npc", "exit", "chest", "lore", "point", "encounter", "roamer"] and str(rows[0]["label"]) == "a"
	checks["legend text names the badge and the state"] = RegionMapView.legend_text({"label": "Mara", "badge": "quest", "state": "present"}).contains("quest") and RegionMapView.legend_text({"label": "Chest", "badge": "", "state": "spent"}).contains("used up") and RegionMapView.legend_text({"label": "Door", "badge": "", "state": "locked"}).contains("locked") and RegionMapView.legend_text({"label": "Plain", "badge": "", "state": "present"}) == "Plain"
	var note: Dictionary = {"note": "A note."}
	checks["point note: its note, the default without one, used-up line once spent"] = RegionMapView.point_note({"state": "present"}, note) == "A note." and RegionMapView.point_note({"state": "present"}, {}) == RegionMapView.POINT_LOOK_NOTE and RegionMapView.point_note({"state": "spent"}, note) == RegionMapView.SPENT_NOTE
	var ok: bool = _report("Map icons, keys and pure rules (overlays, pins, badges, hint, legend order, notes)", checks)
	return _test_pure_layout() and ok

# Names, legend and cards stay inside their bounds at every text size (pure placement).
static func _test_pure_layout() -> bool:
	var checks: Dictionary = {}
	var font: Font = ThemeDB.fallback_font
	var bounds: Rect2 = Rect2(Vector2(0, 40), Vector2(1000, 680))
	var all_in: bool = true
	var legend_in: bool = true
	var card_in: bool = true
	var names_in: bool = true
	for scale: float in SettingsFile.TEXT_LEVELS:
		var fs: int = RegionMapView.name_font_size(roundi(MainMenuView.BASE_FONT * scale))
		var legend: Rect2 = WorldMapView.legend_rect(font, fs, bounds)
		legend_in = legend_in and bounds.encloses(legend)
		var size: Vector2 = font.get_string_size("A marker name of about thirty-two ch", HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
		for corner: Vector2 in [bounds.position, bounds.end, Vector2(bounds.end.x, bounds.position.y), Vector2(bounds.position.x, bounds.end.y), bounds.get_center()]:
			var tile_pos: Vector2 = Vector2(clampf(corner.x - 16.0, bounds.position.x, bounds.end.x - 32.0), clampf(corner.y - 16.0, bounds.position.y, bounds.end.y - 32.0))   # a tile inside the view
			var at: Vector2 = RegionMapView.label_origin(tile_pos, 32.0, size, font.get_ascent(fs), bounds)
			names_in = names_in and bounds.encloses(Rect2(at - Vector2(0, font.get_ascent(fs)), size))
			var pin_at: Vector2 = WorldMapView.name_origin(corner, size, font.get_ascent(fs), bounds)
			all_in = all_in and pin_at.x >= bounds.position.x and pin_at.x + size.x <= bounds.end.x + 0.5
			var card: Vector2 = Vector2(WorldMapView.CARD_WIDTH + 12.0, 150.0)
			card_in = card_in and bounds.encloses(Rect2(WorldMapView.card_origin(corner, card, bounds), card))
	checks["map names stay inside the free view at all four text sizes"] = names_in
	checks["world pin names stay inside the free view"] = all_in
	checks["world legend fits the free view at all four text sizes"] = legend_in
	checks["world hover card is kept inside the free view"] = card_in
	checks["legend lists every pin kind and the three road styles"] = WorldMapView.legend_entries().size() == MapArt.PIN_KINDS.size() + WorldMapView.ROUTE_STYLES.size()
	return _report("Readable maps: placement at the four text sizes (pure)", checks)

# `dir` such that the tile before `pos` is plain floor (the party stands there facing the marker); (0, 0) when none.
static func _approach(t: Dictionary, others: Array[Vector2i], pos: Vector2i) -> Vector2i:
	for d: Vector2i in [Vector2i(0, 1), Vector2i(1, 0), Vector2i(-1, 0), Vector2i(0, -1)]:
		var from: Vector2i = pos - d
		if MapRules.tile(t, from.x, from.y) == "." and not others.has(from):
			return d
	return Vector2i.ZERO

# `view` plays a game on a location holding at least a chest, a point with a note and a point without one (ExplorationTestSuite fixture).
static func run_region(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	var gs: GameState = view.gs
	var loc: Dictionary = ExploreRunner.location(gs)
	var t: Dictionary = ExploreRunner.template_of(loc)
	view.size = MapViewChecks.VIEW
	view._refresh()
	var shown: Array[Dictionary] = view.visible_markers()
	var rows: Array[Dictionary] = view.legend_rows()
	checks["a legend row per visible marker"] = not t.is_empty() and rows.size() == shown.size() and not rows.is_empty()
	var sorted_ok: bool = true
	for i: int in range(1, rows.size()):
		sorted_ok = sorted_ok and not RegionMapView.legend_less(rows[i], rows[i - 1])
	checks["legend rows are in kind order"] = sorted_ok
	checks["every row has a label, an icon that resolves and a tile"] = rows.all(func(r: Dictionary) -> bool: return not str(r["label"]).is_empty() and AssetRegistry.has_key("tile", str(r["icon"])) and view.marker_at(r["pos"]).size() > 0)
	checks["the legend panel is built from the rows (title + one line each)"] = view._legend_box.visible and view._legend_box.get_child_count() == mini(rows.size(), RegionMapView.LEGEND_MAX) + 1 + (1 if rows.size() > RegionMapView.LEGEND_MAX else 0)
	# hover / facing show `label . verb`
	var ev := InputEventMouseMotion.new()
	var hover_ok: bool = true
	var any_hint: bool = false
	for row: Dictionary in shown:
		ev.position = view.tile_centre(row["pos"])
		view._on_canvas_input(ev)
		var v: Dictionary = DictUtil.dict_of(row["view"])
		any_hint = true
		hover_ok = hover_ok and view.hint.text == RegionMapView.hint_line(v) and view.hint.text.contains(str(v["label"])) and view.hint.text.contains(str(v["verb"]))
	checks["hovering a marker tile shows its label and verb in the hint line"] = any_hint and hover_ok
	view._on_canvas_exit()
	var far: Vector2i = Vector2i(-5, -5)
	view.gs.position = far
	view.facing = Vector2i(0, 1)
	view._refresh()
	checks["no hover, nothing faced: the key hint"] = view.hint.text == RegionMapView.HINT
	var faced: Dictionary = shown[0]
	view.gs.position = Vector2i(faced["pos"]) - view.facing
	view._refresh()
	checks["facing a marker from the next tile shows it too"] = view.hint.text == RegionMapView.hint_line(DictUtil.dict_of(faced["view"]))
	# legend hover pulses the tile
	view._on_legend_enter(rows[0]["pos"])
	var pulsing: bool = view._legend_tile == rows[0]["pos"] and view._animated
	view._on_legend_exit()
	checks["a legend row hover pulses its tile; leaving clears it"] = pulsing and view._legend_tile == RegionMapView.NO_TILE
	# text sizes
	var fit: bool = true
	var names_fit: bool = true
	var hint_fit: bool = true
	var panel_fit: bool = true
	for scale: float in SettingsFile.TEXT_LEVELS:
		view.theme = _theme_for(scale)
		view._refresh()
		var font: Font = view.get_theme_default_font()
		var fs: int = view.label_font_size()
		var free: Rect2 = RegionMapView.free_rect(view._canvas.size, view._top, view._side)
		var origin: Vector2 = view.map_origin()
		for row: Dictionary in view.visible_markers():
			var v: Dictionary = DictUtil.dict_of(row["view"])
			var label_size: Vector2 = font.get_string_size(str(v["label"]), HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
			var tile_pos: Vector2 = origin + Vector2(row["pos"]) * view.tile_px()
			# MAPKEYS1-U: every named tile, also partly visible ones: a name that is placed lies inside the free view; a tile fully in view always gets one
			var alone: Array[Dictionary] = [{"tile_pos": tile_pos, "size": label_size}]
			var spot: Vector2 = RegionMapView.place_labels(alone, view.tile_px(), font.get_ascent(fs), free)[0]
			var whole: bool = free.encloses(Rect2(tile_pos, Vector2(view.tile_px(), view.tile_px())))
			if spot != RegionMapView.NO_LABEL:
				names_fit = names_fit and free.encloses(Rect2(spot - Vector2(0, font.get_ascent(fs)), label_size))
			elif whole:
				names_fit = false
			hint_fit = hint_fit and font.get_string_size(RegionMapView.hint_line(v), HORIZONTAL_ALIGNMENT_LEFT, -1, UiTheme.FONT_MD).x <= view.size.x - 16.0
		# the legend lines wrap inside the side column: their total height stays inside the column
		var avail: float = RegionMapView.SIDE_WIDTH - 16.0 - (fs + 8.0) - 8.0
		var height: float = fs + 6.0
		for r: Dictionary in view.legend_rows():
			var w: float = font.get_string_size(str(r["text"]), HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
			height += (ceilf(w / avail) + 0.5) * (fs + 4.0)
		panel_fit = panel_fit and height <= view.size.y - view._top.size.y - 140.0
		fit = fit and view._legend_box.visible
	view.theme = null
	view._refresh()
	checks["npc / exit names stay inside the free map view at all four text sizes (1280x720)"] = names_fit
	checks["the hint line fits the width at all four text sizes"] = hint_fit
	checks["the legend lines fit the side column at all four text sizes"] = panel_fit and fit
	return _report("Region map readability (legend, hover and facing hint, pulse, text sizes at 1280x720)", checks)

# Notes: walking onto a point logs its note instead of the raw flag line; a used-up point and an opened chest say so.
# `with_note`: the id of a point with a `note` and `sets_flags`; `plain`: a point without; `chest`: an unopened chest.
static func run_notes(view: RegionMapView, with_note: String, plain: String, chest: String) -> bool:
	var checks: Dictionary = {}
	var gs: GameState = view.gs
	var t: Dictionary = ExploreRunner.template_of(ExploreRunner.location(gs))
	var by_id: Dictionary = {}
	var taken: Array[Vector2i] = []
	for row: Dictionary in view.visible_markers():
		by_id[str(row["id"])] = row
		taken.append(row["pos"])
	if t.is_empty() or not by_id.has(with_note) or not by_id.has(plain) or not by_id.has(chest):
		return _report("Region map notes", {"fixture markers present": false})
	var marker: Dictionary = DictUtil.dict_of(DictUtil.dict_of(by_id[with_note])["marker"])
	var pos: Vector2i = DictUtil.dict_of(by_id[with_note])["pos"]
	var d: Vector2i = _approach(t, taken, pos)
	checks["a floor tile leads to the point"] = d != Vector2i.ZERO
	if d == Vector2i.ZERO:
		return _report("Region map notes", checks)
	gs.position = pos - d
	view.log_lines.clear()
	view.do_step(d)
	var text: String = "\n".join(PackedStringArray(view.log_lines))
	checks["walking onto a point logs its note"] = gs.position == pos and text.contains(str(marker["note"]))
	checks["the raw 'Flags set' line is gone"] = not text.contains("Flags set")
	view.do_step(-d)
	view.log_lines.clear()
	view.do_step(d)
	checks["walking onto a used-up point logs the used-up line"] = view.log_lines.has(RegionMapView.SPENT_NOTE)
	var plain_pos: Vector2i = DictUtil.dict_of(by_id[plain])["pos"]
	var pd: Vector2i = _approach(t, taken, plain_pos)
	if pd != Vector2i.ZERO:
		gs.position = plain_pos - pd
		view.log_lines.clear()
		view.do_step(pd)
		checks["a point without a note logs the default"] = view.log_lines.has(RegionMapView.POINT_LOOK_NOTE)
	else:
		checks["a floor tile leads to the plain point"] = false
	var chest_pos: Vector2i = DictUtil.dict_of(by_id[chest])["pos"]
	var cd: Vector2i = _approach(t, taken, chest_pos)
	if cd != Vector2i.ZERO:
		gs.position = chest_pos - cd
		view.facing = cd
		view.do_interact()
		view.log_lines.clear()
		view.do_interact()
		checks["E on an opened chest: 'The chest is empty.'"] = view.log_lines.has("The chest is empty.")
		var spent_row: Dictionary = view.marker_at(chest_pos)
		checks["the opened chest is spent in the legend and shows the open icon"] = DictUtil.dict_of(spent_row["view"])["state"] == "spent" and RegionMapView.icon_key(DictUtil.dict_of(spent_row["view"])) == "overlay_chest_open"
	else:
		checks["a floor tile leads to the chest"] = false
	return _report("Region map notes (point notes, used-up point, empty chest)", checks)
