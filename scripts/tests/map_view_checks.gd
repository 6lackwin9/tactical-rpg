class_name MapViewChecks
extends RefCounted

# ==============================================================================
# Tactical RPG - Full-screen map view checks (MAPVIEW1). Pure: the region map camera (RegionMapView.camera_offset:
# centred, followed, clamped at each edge, a map smaller than the view centred) and the world map layout
# (WorldMapView.letterbox / norm_pos / layout: map_pos pins with and without a backdrop rect, the circle fallback).
# Scene: run_region(view) on a region view whose game stands on a mapped location (ExplorationTestSuite debug scene):
# tile <-> canvas point with the camera offset and zoom, clamping and centring in the live view, a click on a
# neighbour tile faces it. Expectations are derived from the rules, not from fixture numbers.
# ==============================================================================

const TAG: String = "MapView"
const VIEW: Vector2 = Vector2(1280, 720)   # the base window size (project.godot)

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func _near(a: Vector2, b: Vector2) -> bool:
	return a.distance_to(b) < 0.01

static func run_pure() -> bool:
	var ok: bool = _test_camera()
	ok = _test_world_layout() and ok
	return ok

static func _test_camera() -> bool:
	var checks: Dictionary = {}
	var view: Vector2 = Vector2(400, 300)
	var big: Vector2 = Vector2(1000, 800)
	var mid: Vector2 = big / 2.0
	checks["party mid-map: centred on the party"] = _near(RegionMapView.camera_offset(big, view, mid), view / 2.0 - mid)
	checks["party at the top-left corner: clamped to the map edge (offset 0)"] = _near(RegionMapView.camera_offset(big, view, Vector2(5, 5)), Vector2.ZERO)
	checks["party at the bottom-right corner: clamped (map edge on the view edge)"] = _near(RegionMapView.camera_offset(big, view, big - Vector2(5, 5)), view - big)
	var small: Vector2 = Vector2(100, 60)
	checks["map smaller than the view: centred whatever the focus"] = _near(RegionMapView.camera_offset(small, view, Vector2.ZERO), (view - small) / 2.0) and _near(RegionMapView.camera_offset(small, view, small), (view - small) / 2.0)
	var wide: Vector2 = Vector2(1000, 100)
	var off: Vector2 = RegionMapView.camera_offset(wide, view, Vector2(990, 50))
	checks["wide map: x clamped at the right edge, y centred"] = _near(off, Vector2(view.x - wide.x, (view.y - wide.y) / 2.0))
	checks["map exactly the view size: no offset"] = _near(RegionMapView.camera_offset(view, view, Vector2(1, 1)), Vector2.ZERO)
	var keys_ok: bool = true
	for ch: Variant in RegionMapView.REGION_TILE_KEYS:
		var base: String = str(RegionMapView.REGION_TILE_KEYS[ch])
		for xy: Vector2i in [Vector2i(0, 0), Vector2i(3, 7), Vector2i(47, 29)]:
			var key: String = RegionMapView.region_tile_key(str(ch), xy.x, xy.y)
			var n: int = key.trim_prefix(base + "_").to_int()
			keys_ok = keys_ok and key.begins_with(base + "_") and n >= 0 and n < RegionMapView.REGION_TILE_VARIANTS and key == RegionMapView.region_tile_key(str(ch), xy.x, xy.y)
	checks["tile art keys: base_<variant>, variant in range and stable per coordinate"] = keys_ok and RegionMapView.region_tile_key("?", 0, 0).is_empty()
	checks["a ui key without a file gives no texture (placeholder kept)"] = RegionMapView.ui_file_texture("mapview1_no_such_key", "icon") == null and WorldMapView.backdrop_for("mapview1_no_such_world") == null
	return _report("Region map camera (follow, clamp at edges, centre a smaller map)", checks)

static func _test_world_layout() -> bool:
	var checks: Dictionary = {}
	var area: Rect2 = Rect2(Vector2(0, 40), Vector2(1000, 1000))
	var tex: Vector2 = Vector2(1920, 1080)
	var box: Rect2 = WorldMapView.letterbox(tex, area)
	var s: float = minf(area.size.x / tex.x, area.size.y / tex.y)
	checks["letterbox keeps the aspect"] = absf(box.size.x / box.size.y - tex.x / tex.y) < 0.001 and _near(box.size, tex * s)
	checks["letterbox is centred in the area"] = _near(box.get_center(), area.get_center())
	checks["letterbox of an empty size = the area"] = WorldMapView.letterbox(Vector2.ZERO, area) == area
	checks["map_pos reads [x, y]"] = _near(WorldMapView.norm_pos([0.25, 0.75]), Vector2(0.25, 0.75)) and _near(WorldMapView.norm_pos([1, 0]), Vector2(1, 0))
	checks["map_pos out of range is clamped"] = _near(WorldMapView.norm_pos([1.5, -0.2]), Vector2(1, 0))
	checks["malformed map_pos is ignored"] = WorldMapView.norm_pos(null) == WorldMapView.NO_POS and WorldMapView.norm_pos([0.5]) == WorldMapView.NO_POS and WorldMapView.norm_pos(["a", 0.5]) == WorldMapView.NO_POS and WorldMapView.norm_pos({"x": 1}) == WorldMapView.NO_POS
	var all: Array[String] = ["w:a", "w:b", "w:c", "w:d"]
	var known: Array[String] = ["w:a", "w:b", "w:d"]
	var pos: Dictionary = {"w:a": [0.25, 0.5], "w:d": ["bad", 1]}
	var canvas: Rect2 = Rect2(Vector2.ZERO, Vector2(800, 600))
	var pins: Dictionary = WorldMapView.layout(all, known, pos, canvas)
	checks["only known nodes get pins"] = pins.size() == known.size() and not pins.has("w:c")
	checks["no backdrop: map_pos is a point of the canvas rect"] = _near(pins.get("w:a", Vector2.INF), canvas.position + Vector2(0.25, 0.5) * canvas.size)
	checks["without map_pos: the circle slot of its graph order"] = _near(pins.get("w:b", Vector2.INF), _slot(1, all.size(), canvas))
	checks["malformed map_pos: the circle slot"] = _near(pins.get("w:d", Vector2.INF), _slot(3, all.size(), canvas))
	var free: Rect2 = Rect2(Vector2(0, 40), Vector2(520, 560))
	var ringed: Dictionary = WorldMapView.layout(all, known, pos, canvas, free)
	checks["a circle rect moves only the circle slots (panels kept clear)"] = _near(ringed.get("w:b", Vector2.INF), _slot(1, all.size(), free)) and _near(ringed.get("w:a", Vector2.INF), pins.get("w:a", Vector2.ZERO))
	var lb: Dictionary = WorldMapView.layout(all, known, pos, box)
	checks["with a backdrop: map_pos is a point of the letterboxed rect"] = _near(lb.get("w:a", Vector2.INF), box.position + Vector2(0.25, 0.5) * box.size)
	checks["with a backdrop: the circle is centred on the backdrop"] = _near(lb.get("w:b", Vector2.INF), _slot(1, all.size(), box))
	return _report("World map layout (letterbox, map_pos with and without backdrop, circle fallback)", checks)

static func _slot(i: int, n: int, rect: Rect2) -> Vector2:
	var angle: float = TAU * float(i) / float(n) - PI / 2.0
	return rect.get_center() + Vector2(cos(angle), sin(angle)) * minf(rect.size.x, rect.size.y) * WorldMapView.LAYOUT_FRACTION

# `view` plays a game on a mapped location, nothing pending. Leaves zoom, facing and position as they were.
static func run_region(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	var gs: GameState = view.gs
	var t: Dictionary = ExploreRunner.template_of(ExploreRunner.location(gs)) if gs != null else {}
	checks["the game stands on a mapped location"] = not t.is_empty() and not view.busy()
	var canvas: Control = view._canvas
	view.size = VIEW   # the arena parent may have no size; the full-rect layout follows the view
	var free: Rect2 = RegionMapView.free_rect(canvas.size, view._top, view._side)
	checks["the canvas fills the view, the panels float over it"] = canvas.size == view.size and free.size.x > 0.0 and free.size.y > 0.0 and view._side.get_parent() == canvas.get_parent()
	if t.is_empty() or free.size.x <= 0.0 or free.size.y <= 0.0:
		print("      map view sizes: view %s canvas %s free %s" % [str(view.size), str(canvas.size), str(free)])
		return _report("Region map view (camera, click with offset)", checks)
	var zoom0: float = view.zoom
	var pos0: Vector2i = gs.position
	var facing0: Vector2i = view.facing
	var map_tiles: Vector2 = Vector2(MapRules.size_of(t))
	# a map larger than the view: the camera follows the party and clamps at the edges
	var fit: Vector2 = free.size / (map_tiles * RegionMapView.TILE)
	view.zoom = ceilf(maxf(fit.x, fit.y)) * 3.0
	var map_px: Vector2 = map_tiles * view.tile_px()
	gs.position = Vector2i.ZERO
	checks["party at the map corner: the map edge sits on the free view edge"] = _near(view.map_origin(), free.position)
	gs.position = Vector2i(map_tiles) - Vector2i.ONE
	checks["party at the far corner: clamped to the other edges"] = _near(view.map_origin(), free.position + free.size - map_px)
	var mid: Vector2i = Vector2i(map_tiles / 2.0)
	gs.position = mid
	checks["party mid-map: its tile is inside the free view"] = free.has_point(view.tile_centre(mid))
	var probes: Array[Vector2i] = [Vector2i.ZERO, mid, mid + Vector2i(1, 0), Vector2i(map_tiles) - Vector2i.ONE]
	var round_trip: bool = true
	for p: Vector2i in probes:
		round_trip = round_trip and view.tile_at_point(view.tile_centre(p)) == p
	checks["click point -> tile with the camera offset and zoom"] = round_trip
	# a map smaller than the view: centred
	view.zoom = minf(fit.x, fit.y) * 0.5
	checks["map smaller than the view: centred in the free view"] = _near(view.map_origin(), free.position + (free.size - map_tiles * view.tile_px()) / 2.0)
	# a click through the canvas input on the tile left of the party faces it
	view.zoom = zoom0
	gs.position = pos0
	view.facing = Vector2i(0, 1)
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = true
	ev.position = view.tile_centre(pos0 + Vector2i(-1, 0))
	view._on_canvas_input(ev)
	checks["a click on the neighbour tile (offset applied) faces it"] = view.facing == Vector2i(-1, 0) and gs.position == pos0
	view.facing = facing0
	view._refresh()
	return _report("Region map view (full-screen canvas, camera follow and clamp, centring, click with offset)", checks)
