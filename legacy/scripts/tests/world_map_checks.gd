class_name WorldMapChecks
extends RefCounted

# ==============================================================================
# Tactical RPG - World map scene checks (S10, spec TRAV1-L commit 3), run by TravelTestSuite after the runtime checks
# (same sandbox; world R of TravelRuntimeChecks plus a start location, a mapped gate with a world_map exit and a road
# that always fights). Debug outcomes settle the fights; saves go to the sandbox save folder.
# ==============================================================================

const TAG: String = "Travel"
const WORLD_SCENE: String = "res://scenes/overworld/world_map_debug.tscn"
const REGION_SCENE: String = "res://scenes/overworld/region_map_debug.tscn"
const R: String = TravelRuntimeChecks.R
const SEED: int = 6262
const FX_POS: Array = [0.19, 0.75]   # MAPVIEW1: Fox Hollow's pin point (normalized); other nodes keep the circle

static func _q(id: String) -> String:
	return WorldCatalog.qid(R, id)

static func _build() -> bool:
	var errors: Array[String] = []
	errors.append_array(WorldCatalog.update_manifest(R, {"start_location": "m0"}))
	errors.append_array(WorldCatalog.upsert(R, "locations", {"id": "fx", "name": "Fox Hollow", "map_pos": FX_POS}))
	errors.append_array(WorldCatalog.upsert(R, "locations", {"id": "gate", "name": "Gate", "map": "m_tv",
		"markers": [{"id": "to_map", "kind": "exit", "anchor": "a_wm", "to": MapRules.WORLD_MAP, "sets_flags": [{"key": "tr.mapped"}]}]}))
	errors.append_array(WorldCatalog.upsert(R, "routes", {"id": "r_gate", "name": "Gate road", "from": "m0", "to": "gate", "time": 1, "encounter_pct": 0}))
	errors.append_array(WorldCatalog.upsert(R, "routes", {"id": "r_fx", "name": "Fox road", "from": "m0", "to": "fx", "time": 1, "encounter_pct": 100, "repeat_pct": 0, "encounters": [["enc_a", 1]]}))
	var problems: Dictionary = WorldCatalog.validate_world(R)
	if not errors.is_empty() or not problems.is_empty():
		print("      world map fixture errors: %s %s" % [str(errors), str(problems)])
	return TestHelpers.report(TAG, "World map fixtures (start location, mapped gate with a world_map exit, a fighting road)", {"no errors": errors.is_empty(), "valid": problems.is_empty()})

static func run(tree_parent: Node) -> bool:
	var packed_world: PackedScene = load(WORLD_SCENE) as PackedScene
	var packed_region: PackedScene = load(REGION_SCENE) as PackedScene
	var loads: Dictionary = {"world map scene loads": packed_world != null, "region map scene loads": packed_region != null}
	if tree_parent == null or packed_world == null or packed_region == null:
		return TestHelpers.report(TAG, "Debug scenes (headless without a tree: load only)", loads)
	if not _build():
		return false
	var ok: bool = TestHelpers.report(TAG, "Debug scenes load", loads)
	ok = _test_standalone(tree_parent, packed_world) and ok
	ok = _test_region_hand_over(tree_parent, packed_region) and ok
	ok = _test_play_mode(tree_parent, packed_region) and ok   # MAPDEV1-U
	return ok

static func _test_standalone(tree_parent: Node, packed: PackedScene) -> bool:
	var checks: Dictionary = {}
	var view: WorldMapView = packed.instantiate() as WorldMapView
	tree_parent.add_child(view)
	checks["new game at the start node"] = view.start_game(R, SEED).is_empty() and view.gs != null and view.gs.location_id == _q("m0")
	if view.gs == null:
		tree_parent.remove_child(view)
		view.free()
		return TestHelpers.report(TAG, "World map scene (standalone)", checks)
	var dev_ok: bool = MapDevChecks.run_world_debug(view)   # MAPDEV1-U
	var pins: Dictionary = view.pin_positions()
	var pin_keys: Array[String] = []
	for k: Variant in pins:
		pin_keys.append(str(k))
	pin_keys.sort()
	var known: Array[String] = TravelRules.known(view.gs)
	known.sort()
	checks["one pin per known node"] = pin_keys == known and not pin_keys.has(_q("m2"))
	var m1_pin: Vector2 = pins.get(_q("m1"), Vector2(-1000, -1000))
	checks["a click on a pin finds its node"] = view.node_at(m1_pin) == _q("m1") and view.node_at(Vector2(-500, -500)).is_empty()
	# MAPVIEW1: map_pos places the pin in the canvas rect (no backdrop in the sandbox) or in the letterboxed backdrop
	view.size = MapViewChecks.VIEW   # the arena parent may have no size; the full-rect canvas follows the view
	pins = view.pin_positions()
	var fx_at: Vector2 = Vector2(float(FX_POS[0]), float(FX_POS[1]))
	var plain: Rect2 = view.layout_rect()
	var free: Rect2 = RegionMapView.free_rect(view._canvas.size, view._top, view._side)
	checks["no backdrop: the layout rect is the free area the panels leave"] = view.backdrop == null and plain == free and free.size.x < view._canvas.size.x
	checks["map_pos pin in the layout rect"] = pins.has(_q("fx")) and MapViewChecks._near(pins[_q("fx")], plain.position + fx_at * plain.size)
	var img: Image = Image.create(192, 108, false, Image.FORMAT_RGB8)
	view.backdrop = ImageTexture.create_from_image(img)
	var box: Rect2 = WorldMapView.letterbox(Vector2(192, 108), plain)
	var lb: Dictionary = view.pin_positions()
	checks["backdrop: letterboxed rect, map_pos pin inside it"] = view.layout_rect() == box and MapViewChecks._near(lb.get(_q("fx"), Vector2.INF), box.position + fx_at * box.size)
	checks["backdrop: a pin without map_pos keeps its circle slot around the backdrop"] = MapViewChecks._near(lb.get(_q("m1"), Vector2.INF), WorldMapView.layout(TravelRules.nodes(R), TravelRules.known(view.gs), {}, box).get(_q("m1"), Vector2.ZERO))
	var clear: bool = true
	for k: Variant in lb:
		clear = clear and free.has_point(lb[k])
	checks["with a backdrop no pin sits under a panel"] = clear and not lb.is_empty()
	view.backdrop = null
	# MAPREAD1-L2: hovering a pin opens its card (name, trip cost from here); leaving closes it
	var hover := InputEventMouseMotion.new()
	hover.position = pins.get(_q("m1"), Vector2.ZERO)
	view._on_canvas_input(hover)
	var trip: Dictionary = TravelRules.trip(view.gs, view.gs.location_id, _q("m1"))
	var lines: Array[String] = view.card_lines(_q("m1"))
	var card_rect: Rect2 = Rect2(view._card.position, view._card.get_combined_minimum_size())
	checks["hover card: name, trip time and supplies from the party's node"] = view.hover_node == _q("m1") and view._card.visible and lines.size() >= 3 and lines.has("Trip: time %d, supplies %d" % [DictUtil.to_int(trip["time"]), DictUtil.to_int(trip["supplies"])]) and trip["ok"] == true
	checks["hover card stays inside the canvas"] = Rect2(Vector2.ZERO, view._canvas.size).encloses(card_rect)
	checks["the party's own node says 'You are here'"] = view.card_lines(view.gs.location_id).has(WorldMapView.HERE_TEXT)
	view._canvas.mouse_exited.emit()
	checks["leaving the pin hides the card"] = view.hover_node.is_empty() and not view._card.visible
	var t0: int = view.gs.time
	var s0: int = view.gs.party.supplies
	view.travel_to(_q("m1"))
	checks["click = travel; the clock advances"] = view.gs.location_id == _q("m1") and view.gs.time == t0 + TravelRuntimeChecks.M01_TIME
	checks["trip() matches the real travel: same time and supplies spent"] = view.gs.time - t0 == DictUtil.to_int(trip["time"]) and s0 - view.gs.party.supplies == DictUtil.to_int(trip["supplies"])
	checks["debug game never writes the autosave"] = not SaveGame.exists(SaveGame.AUTOSAVE_SLOT)
	tree_parent.remove_child(view)
	view.free()
	return TestHelpers.report(TAG, "World map scene (standalone: pins, click to travel, clock)", checks) and dev_ok

# MAPDEV1-U: a region map with debug_tools off (as the main menu opens it) and the world map over it.
static func _test_play_mode(tree_parent: Node, packed: PackedScene) -> bool:
	var view: RegionMapView = packed.instantiate() as RegionMapView
	view.debug_tools = false
	tree_parent.add_child(view)
	var gs: GameState = GameState.of(RegionMapView.new_state(R, SEED)["state"])
	var ok: bool = gs != null
	if ok:
		view.play_state(gs)
		ok = MapDevChecks.run_world_play(view, _q("m1"))
	tree_parent.remove_child(view)
	view.free()
	return TestHelpers.report(TAG, "Play-mode region map with the world map over it", {"state built": gs != null}) and ok

static func _test_region_hand_over(tree_parent: Node, packed: PackedScene) -> bool:
	var checks: Dictionary = {}
	var view: RegionMapView = packed.instantiate() as RegionMapView
	tree_parent.add_child(view)
	checks["region game at the start node"] = view.start_game(R, SEED).is_empty() and view.gs != null and view.gs.location_id == _q("m0")
	if view.gs == null:
		tree_parent.remove_child(view)
		view.free()
		return TestHelpers.report(TAG, "Region map hand-over", checks)
	CurrentGame.start(view.gs)
	var settled: Array[Dictionary] = []
	var on_settled: Callable = func(res: Dictionary) -> void: settled.append(res)
	view.fight_settled.connect(on_settled)
	view.take_exit(MapRules.WORLD_MAP)
	checks["the menu entry opens the world map"] = view.world_map != null and view.world_map.gs == view.gs
	if view.world_map == null:
		CurrentGame.end()
		tree_parent.remove_child(view)
		view.free()
		return TestHelpers.report(TAG, "Region map hand-over", checks)
	var wm: WorldMapView = view.world_map
	wm.travel_to(_q("fx"))
	checks["a road fight is pending: both views busy, saving refused"] = wm.busy() and view.busy() and CurrentGame.busy()
	wm.settle("fled")
	checks["fled: still on the world map at the start, not busy"] = view.world_map == wm and view.gs.location_id == _q("m0") and not CurrentGame.busy()
	wm.travel_to(_q("fx"))
	wm.settle("won")
	checks["won: the leg completes, the map stays open"] = view.world_map == wm and view.gs.location_id == _q("fx")
	checks["fight_settled is forwarded"] = settled.size() == 2 and str(settled[1].get("outcome", "")) == "won"
	wm.travel_to(_q("m0"))
	checks["arrival hands back to the region map"] = view.world_map == null and view.gs.location_id == _q("m0")
	checks["main-menu game: arrival autosaves slot 0"] = SaveGame.exists(SaveGame.AUTOSAVE_SLOT) and SaveGame.path_for(SaveGame.AUTOSAVE_SLOT).begins_with(TravelTestSuite.SANDBOX)
	# A world_map exit marker: its sets_flags apply, the token stays, the world map opens; Back closes it.
	view.take_exit(MapRules.WORLD_MAP)
	if view.world_map != null:
		view.world_map.travel_to(_q("gate"))
	var t: Dictionary = MapCatalog.get_entry("m_tv")
	var wm_pos: Vector2i = MapRules.anchor_pos(t, "a_wm")
	checks["arrived at the mapped gate"] = view.world_map == null and view.gs.location_id == _q("gate")
	view.gs.position = wm_pos + Vector2i(0, -1)
	view.do_step(Vector2i(0, 1))
	checks["world_map exit marker: flags set, token stays, map open"] = view.world_map != null and view.gs.get_flag("tr.mapped", false) == true and view.gs.position == wm_pos + Vector2i(0, -1)
	var before: Vector2i = view.gs.position
	view.do_step(Vector2i(-1, 0))
	checks["region input ignored while the world map is open"] = view.gs.position == before
	if view.world_map != null:
		view.world_map.back()
	checks["Back closes it"] = view.world_map == null and view.gs.location_id == _q("gate")
	# Clicking the pin of the current location enters it (no travel needed).
	view.take_exit(MapRules.WORLD_MAP)
	if view.world_map != null:
		view.world_map.travel_to(view.gs.location_id)
	checks["clicking the current node enters it and closes the world map"] = view.world_map == null and view.gs.location_id == _q("gate")
	view.fight_settled.disconnect(on_settled)
	CurrentGame.end()
	tree_parent.remove_child(view)
	view.free()
	return TestHelpers.report(TAG, "Region map hand-over (menu and marker exits, road fights, arrival, autosave, Back)", checks)
