class_name MapKeysChecks
extends RefCounted

# ==============================================================================
# Tactical RPG - Region map keys, glide and name placement checks (MAPKEYS1-U). Pure (run_pure): crowded name labels never
# overlap, stay next to their own tile (also at the view edges), a tile outside the view has no name. Scene (run_region): WASD
# steps like the arrow keys, a held key repeats after REPEAT_DELAY every REPEAT_INTERVAL and stops on release, holding stops
# at a fight and while a conversation is open, the drawn party position glides to the party tile without lag. Expectations are
# derived from the view's constants and the fixture's rules (ExplorationTestSuite `field`, an open safe column at x = 1).
# ==============================================================================

const TAG: String = "MapKeys"

static func _report(label: String, checks: Dictionary) -> bool:
	var ok: bool = true
	print("   %s" % label)
	for k: Variant in checks:
		var pass_one: bool = bool(checks[k])
		ok = ok and pass_one
		print("      [%s] %s" % ["PASS" if pass_one else "FAIL", str(k)])
	return ok

static func run_pure() -> bool:
	var checks: Dictionary = {}
	var font: Font = ThemeDB.fallback_font
	var fs: int = RegionMapView.name_font_size(MainMenuView.BASE_FONT)
	var asc: float = font.get_ascent(fs)
	var bounds: Rect2 = Rect2(Vector2(0, 40), Vector2(1000, 680))
	var px: float = 32.0
	var names: Array[String] = ["Tev Quill", "Brask Toller", "Hest the Quartermaster", "Mara Voss"]
	# a crowd: four neighbouring tiles at the right edge of the view, the user's screenshot case
	var items: Array[Dictionary] = []
	for i: int in range(names.size()):
		items.append({"tile_pos": Vector2(bounds.end.x - px - 4.0, 300.0 + 20.0 * i), "size": font.get_string_size(names[i], HORIZONTAL_ALIGNMENT_LEFT, -1, fs)})
	var spots: Array[Vector2] = RegionMapView.place_labels(items, px, asc, bounds)
	var rects: Array[Rect2] = []
	var attached: bool = true
	var inside: bool = true
	for i: int in range(items.size()):
		if spots[i] == RegionMapView.NO_LABEL:
			continue
		var item_size: Vector2 = items[i]["size"]
		var r: Rect2 = RegionMapView.label_rect(spots[i], item_size, asc)
		rects.append(r)
		var tile_pos: Vector2 = items[i]["tile_pos"]
		attached = attached and r.position.x <= tile_pos.x + px and r.end.x >= tile_pos.x and absf(r.get_center().y - (tile_pos.y + px / 2.0)) <= px / 2.0 + 3.0 * (r.size.y)
		inside = inside and bounds.encloses(r)
	var overlap: bool = false
	for i: int in range(rects.size()):
		for j: int in range(i + 1, rects.size()):
			overlap = overlap or rects[i].intersects(rects[j])
	checks["a crowded edge fixture places at least the first name"] = spots[0] != RegionMapView.NO_LABEL and rects.size() >= 1
	checks["placed names do not overlap each other"] = not overlap
	checks["placed names stay next to their own tile (hang off the tile edge, not clamped away)"] = attached
	checks["placed names stay inside the free view"] = inside
	# the same tile twice (two markers on neighbouring tiles of one row): both fit by stacking or the second hides, never overlap
	var row_items: Array[Dictionary] = []
	for i: int in range(3):
		row_items.append({"tile_pos": Vector2(400.0 + px * i, 300.0), "size": font.get_string_size(names[2], HORIZONTAL_ALIGNMENT_LEFT, -1, fs)})
	var row_spots: Array[Vector2] = RegionMapView.place_labels(row_items, px, asc, bounds)
	var row_rects: Array[Rect2] = []
	for i: int in range(row_items.size()):
		if row_spots[i] != RegionMapView.NO_LABEL:
			var row_size: Vector2 = row_items[i]["size"]
			row_rects.append(RegionMapView.label_rect(row_spots[i], row_size, asc))
	var row_overlap: bool = false
	for i: int in range(row_rects.size()):
		for j: int in range(i + 1, row_rects.size()):
			row_overlap = row_overlap or row_rects[i].intersects(row_rects[j])
	checks["three wide names on neighbouring tiles never overlap"] = not row_overlap and row_rects.size() >= 2
	# a lone name keeps the classic spot (centred above its tile)
	var lone: Array[Vector2] = RegionMapView.place_labels([{"tile_pos": Vector2(500, 300), "size": items[0]["size"]}], px, asc, bounds)
	var lone_size: Vector2 = items[0]["size"]
	checks["a lone name is centred above its tile"] = lone[0] == Vector2(500.0 + (px - lone_size.x) / 2.0, 297.0)
	# a tile under the top panel (outside the free view) has no name
	var hidden: Array[Vector2] = RegionMapView.place_labels([{"tile_pos": Vector2(500, 0), "size": items[0]["size"]}], px, asc, bounds)
	checks["a tile outside the free view draws no name"] = hidden[0] == RegionMapView.NO_LABEL
	return _report("Region map name placement (pure)", checks)

static func _key(code: Key, pressed: bool, echo: bool = false) -> InputEventKey:
	var ev := InputEventKey.new()
	ev.keycode = code
	ev.physical_keycode = code
	ev.pressed = pressed
	ev.echo = echo
	return ev

static func _play(view: RegionMapView, make_state: Callable, x: int, y: int) -> void:
	var state: GameState = make_state.call(x, y)
	view.play_state(state)

static func _press(view: RegionMapView, code: Key) -> void:
	view._unhandled_input(_key(code, true))

static func _release(view: RegionMapView, code: Key) -> void:
	view._input(_key(code, false))

# `view` plays the empty `field` (x = 1 .. 3 safe, wilds x = 4 .. 7 danger bound to encounters); `make_state` builds a state at (x, y).
# `talk_with`: a character ref that opens a conversation in `view`'s fixture (may be "").
static func run_region(view: RegionMapView, make_state: Callable) -> bool:
	var checks: Dictionary = {}
	view.size = MapViewChecks.VIEW
	# WASD steps like the arrows (one tile in the key's direction from an open tile)
	var pairs: Array = [[KEY_W, KEY_UP, Vector2i(0, -1)], [KEY_S, KEY_DOWN, Vector2i(0, 1)], [KEY_A, KEY_LEFT, Vector2i(-1, 0)], [KEY_D, KEY_RIGHT, Vector2i(1, 0)]]
	var same: bool = true
	for p: Array in pairs:
		var by_arrow: Vector2i = Vector2i.ZERO
		for code: Key in [p[1], p[0]]:
			_play(view, make_state, 2, 4)
			var start: Vector2i = view.gs.position
			_press(view, code)
			_release(view, code)
			var moved: Vector2i = view.gs.position - start
			same = same and moved == p[2] and view.facing == p[2]
			if code == p[1]:
				by_arrow = moved
			else:
				same = same and moved == by_arrow
	checks["W A S D step the party exactly like the arrow keys and face that way"] = same
	# press steps once at once; OS key echo does not step; the repeat starts after REPEAT_DELAY
	_play(view, make_state, 1, 1)
	_press(view, KEY_S)
	var after_press: Vector2i = view.gs.position
	view._unhandled_input(_key(KEY_S, true, true))
	view._process(RegionMapView.REPEAT_DELAY * 0.5)
	var before_delay: Vector2i = view.gs.position
	view._process(RegionMapView.REPEAT_DELAY * 0.5 + 0.001)
	var after_delay: Vector2i = view.gs.position
	checks["the first step happens on the press; an OS echo and the time before the delay add none"] = after_press == Vector2i(1, 2) and before_delay == after_press
	checks["a held key steps again once REPEAT_DELAY has passed"] = after_delay == Vector2i(1, 3)
	view._process(RegionMapView.REPEAT_INTERVAL * 0.5)
	var mid: Vector2i = view.gs.position
	view._process(RegionMapView.REPEAT_INTERVAL * 0.5 + 0.001)
	checks["then one step per REPEAT_INTERVAL, steady"] = mid == after_delay and view.gs.position == Vector2i(1, 4)
	_release(view, KEY_S)
	var at_release: Vector2i = view.gs.position
	view._process(RegionMapView.REPEAT_DELAY + RegionMapView.REPEAT_INTERVAL * 4.0)
	checks["releasing the key stops the walk"] = view.gs.position == at_release
	# two keys: the newest held direction wins; releasing it falls back to the other
	_press(view, KEY_D)
	_press(view, KEY_S)
	var two_start: Vector2i = view.gs.position
	_release(view, KEY_S)
	view._process(RegionMapView.REPEAT_DELAY + 0.001)
	var fell_back: bool = view.gs.position == two_start + Vector2i(1, 0)   # D is still held: one step right
	_release(view, KEY_D)
	checks["with two keys held, releasing the newest falls back to the key still held"] = fell_back
	# the step counter, glide and camera: every held step counts exactly as a single press (compare with the same steps by calls)
	_play(view, make_state, 1, 1)
	var steps_before: int = DictUtil.to_int(view.gs.explore.get("loc_steps", 0))
	_press(view, KEY_S)
	view._process(RegionMapView.REPEAT_DELAY + 0.001)
	view._process(RegionMapView.REPEAT_INTERVAL + 0.001)
	_release(view, KEY_S)
	checks["each held step counts once in the location step counter"] = DictUtil.to_int(view.gs.explore.get("loc_steps", 0)) - steps_before == view.gs.position.y - 1
	# glide: the drawn position trails the party tile, gets there within a tile / GLIDE speed, and never overshoots
	_play(view, make_state, 1, 1)
	_press(view, KEY_S)
	_release(view, KEY_S)
	var target: Vector2 = Vector2(view.gs.position) + Vector2(0.5, 0.5)
	var lagging: bool = view.party_focus().distance_to(target) > 0.0 and view.party_focus().y < target.y
	var frame: float = 1.0 / 60.0
	var ticks: int = 0
	var monotone: bool = true
	var last_y: float = view.party_focus().y
	while view.party_focus() != target and ticks < 60:
		view.glide(frame)
		monotone = monotone and view.party_focus().y >= last_y and view.party_focus().y <= target.y
		last_y = view.party_focus().y
		ticks += 1
	var need: int = ceili(1.0 / (RegionMapView.GLIDE_TILES_PER_SEC * frame))
	checks["the drawn party position glides to the new tile (no jump, no overshoot)"] = lagging and monotone and ticks <= need + 1
	checks["the glide is faster than the repeat rate, so no lag builds up while a key is held"] = RegionMapView.GLIDE_TILES_PER_SEC * RegionMapView.REPEAT_INTERVAL > 1.0
	_press(view, KEY_S)   # mid-glide again: the drawn position trails the new tile
	_release(view, KEY_S)
	var trailing: bool = view.party_focus() != Vector2(view.gs.position) + Vector2(0.5, 0.5)
	view.snap_focus()
	checks["snap_focus ends a glide in progress at the party tile"] = trailing and view.party_focus() == Vector2(view.gs.position) + Vector2(0.5, 0.5) and not view._gliding
	# loading a game: no glide from the old spot, no held key carried over
	_press(view, KEY_S)
	_play(view, make_state, 3, 3)
	checks["play_state snaps the view to the new game and drops held keys"] = view.party_focus() == Vector2(3.5, 3.5) and view._held_keys.is_empty() and not view._gliding
	# a fight stops the hold: step into the wilds with the counter one step short
	var fight_view_state: GameState = make_state.call(3, 2)
	view.play_state(fight_view_state)
	view.gs.explore["next_at"] = DictUtil.to_int(view.gs.explore.get("steps", 0)) + 2
	_press(view, KEY_D)
	var fight_before: bool = view.pending_fight.is_empty()
	view._process(RegionMapView.REPEAT_DELAY + 0.001)
	var fought: bool = not view.pending_fight.is_empty()
	var stop_at: Vector2i = view.gs.position
	view._process(RegionMapView.REPEAT_DELAY + RegionMapView.REPEAT_INTERVAL * 6.0)
	checks["a held key walks into a counter fight like single presses and stops there"] = fight_before and fought and view.gs.position == stop_at and view._held_keys.is_empty()
	view.settle("won")
	_release(view, KEY_D)
	return _report("Region map WASD, held-key walking and glide", checks)

# A conversation open (or opening) stops the hold; keys are ignored while it is open. `character`: a ref that opens one.
static func run_dialogue(view: RegionMapView, make_state: Callable, character: String) -> bool:
	var checks: Dictionary = {}
	view.size = MapViewChecks.VIEW
	_play(view, make_state, 1, 1)
	_press(view, KEY_S)
	view.talk(character)
	var open: bool = view.busy()
	var held_at: Vector2i = view.gs.position
	view._process(RegionMapView.REPEAT_DELAY + RegionMapView.REPEAT_INTERVAL * 4.0)
	checks["a conversation opens (fixture)"] = open
	checks["holding stops while a conversation is open"] = view.gs.position == held_at and view._held_keys.is_empty()
	_press(view, KEY_D)
	checks["a direction key pressed during a conversation does not move or start a hold"] = view.gs.position == held_at and view._held_keys.is_empty()
	_release(view, KEY_S)
	return _report("Region map holding stops at a conversation", checks)
