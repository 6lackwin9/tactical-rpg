class_name MapDevChecks
extends RefCounted

# ==============================================================================
# Tactical RPG - Region map debug bar vs play bar checks (MAPDEV1-U). run_play: a region map in real play (debug_tools off,
# the main menu route) has no world picker, no New game button and no debug status line, its bar shows the location's
# name and the party's supplies, and start_game is refused with the game untouched. run_debug: the debug scene keeps its
# controls. run_world_play / run_world_debug: the same rule for the world map (embedded over a play region map: place, time
# and supplies in words, no qid / rolls, no picker; the standalone debug scene keeps its debug line). run_chest_note (pure): the chest note names the items by display name ("xN" above one), adds supplies and gold,
# and prints no raw dictionary. Expectations come from the view's constants and the catalog entries, not shipped numbers.
# ==============================================================================

static func _report(label: String, checks: Dictionary) -> bool:
	var ok: bool = true
	print("   %s" % label)
	for k: Variant in checks:
		var pass_one: bool = bool(checks[k])
		ok = ok and pass_one
		print("      [%s] %s" % ["PASS" if pass_one else "FAIL", str(k)])
	return ok

static func _nodes_of(root: Node, out: Array[Node]) -> void:
	for c: Node in root.get_children():
		out.append(c)
		_nodes_of(c, out)

# Visible-in-tree controls under `view._root` (the bar and the side column; the embedded world map is not part of it).
static func _shown(view: RegionMapView) -> Array[Control]:
	var all: Array[Node] = []
	_nodes_of(view._root, all)
	var out: Array[Control] = []
	for n: Node in all:
		var c: Control = n as Control
		if c != null and c.is_visible_in_tree():
			out.append(c)
	return out

static func run_play(view: RegionMapView, label: String) -> bool:
	var checks: Dictionary = {}
	var gs: GameState = view.gs
	view._refresh()
	checks["a game is shown, debug_tools off"] = gs != null and not view.debug_tools
	var pickers: int = 0
	var new_game: int = 0
	var debug_text: int = 0
	for c: Control in _shown(view):
		if c is OptionButton:
			pickers += 1
		var b: Button = c as Button
		if b != null and b.text == "New game":
			new_game += 1
		var l: Label = c as Label
		if l != null and gs != null and (l.text.contains("rolls") or l.text == RegionMapView.DEBUG_IDLE or (not gs.location_id.is_empty() and l.text.contains(gs.location_id))):
			debug_text += 1
			print("      debug-looking text: %s" % l.text.left(120))
	checks["no world picker shown"] = pickers == 0 and view._world_pick == null
	checks["no New game button shown"] = new_game == 0 and view.new_game_button == null
	checks["no debug status line, no location ids in the bar or the log (position, steps, rolls)"] = debug_text == 0 and view._status == null
	var loc_name: String = str(ExploreRunner.location(gs).get("name", "")) if gs != null else ""
	checks["the bar shows the place name"] = not loc_name.is_empty() and view.place_label != null and view.place_label.is_visible_in_tree() and view.place_label.text == loc_name
	var want_supplies: String = RegionMapView.SUPPLIES_TEXT % [gs.party.supplies, PartyState.SUPPLIES_MAX] if gs != null else "-"
	checks["the bar shows the party's supplies"] = view.supplies_label != null and view.supplies_label.is_visible_in_tree() and view.supplies_label.text == want_supplies
	var before: String = JSON.stringify(gs.to_dict()) if gs != null else ""
	var refused: Array[String] = view.start_game(gs.world_id if gs != null else "")
	checks["start_game is refused in play, the game untouched"] = not refused.is_empty() and view.gs == gs and gs != null and JSON.stringify(gs.to_dict()) == before
	return _report("Region map in play (%s): place and supplies, no debug controls (MAPDEV1-U)" % label, checks)

static func run_debug(view: RegionMapView) -> bool:
	var checks: Dictionary = {}
	view._refresh()
	checks["the debug scene sets debug_tools"] = view.debug_tools
	checks["world picker shown"] = view._world_pick != null and view._world_pick.is_visible_in_tree()
	checks["New game button shown"] = view.new_game_button != null and view.new_game_button.is_visible_in_tree()
	checks["debug status line shown"] = view._status != null and view._status.is_visible_in_tree() and not view._status.text.is_empty()
	return _report("Region map debug scene keeps its debug controls (MAPDEV1-U)", checks)

# `item_a` and `item_b`: two distinct catalog item ids (picked by rule by the caller).
static func run_chest_note(item_a: String, item_b: String) -> bool:
	var checks: Dictionary = {}
	var name_a: String = str(ContentCatalog.get_entry("items", item_a).get("name", ""))
	var name_b: String = str(ContentCatalog.get_entry("items", item_b).get("name", ""))
	checks["fixture items have display names"] = item_a != item_b and not name_a.is_empty() and not name_b.is_empty()
	var note: String = RegionMapView.chest_note({"items": {item_a: 2, item_b: 1}, "supplies": 4, "gold": 0})
	checks["names both items, xN only above one"] = note.begins_with("Found: ") and note.contains(name_a + " x2") and note.contains(name_b) and not note.contains(name_b + " x1")
	checks["supplies in words"] = note.contains("4 supplies")
	checks["no raw ids or dictionaries"] = not note.contains("{") and not note.contains("}") and (name_a == item_a or not note.contains(item_a))
	var gold_only: String = RegionMapView.chest_note({"items": {}, "supplies": 0, "gold": 5})
	checks["gold only: 'Found: 5 gold'"] = gold_only == "Found: 5 gold"
	checks["nothing: the nothing note"] = RegionMapView.chest_note({"items": {}, "supplies": 0}) == RegionMapView.CHEST_NOTHING_NOTE
	return _report("Chest note in words (MAPDEV1-U)", checks)

static func _no_debug_text(text: String, gs: GameState) -> bool:
	return not text.contains("rolls") and not text.contains("{") and not text.contains(RegionMapView.DEBUG_IDLE) and (gs.location_id.is_empty() or not text.contains(gs.location_id))

# `region`: a play-mode region map (debug_tools off) whose game is on a world-map node; `dest`: a node one road away. Opens
# the world map over it, travels to `dest` and closes it again (the region map is left on `dest`).
static func run_world_play(region: RegionMapView, dest: String) -> bool:
	var checks: Dictionary = {}
	var gs: GameState = region.gs
	region.open_world_map()
	var wm: WorldMapView = region.world_map
	checks["the world map opens over the play region map, debug_tools off"] = wm != null and gs != null and not wm.debug_tools
	if wm == null or gs == null:
		return _report("World map in play (MAPDEV1-U)", checks)
	var all: Array[Node] = []
	_nodes_of(wm, all)
	var pickers: int = 0
	var new_game: int = 0
	for n: Node in all:
		if n is OptionButton:
			pickers += 1
		var b: Button = n as Button
		if b != null and b.text == "New game":
			new_game += 1
	checks["no world picker, no New game"] = pickers == 0 and new_game == 0
	var status: String = wm.status_text()
	checks["status: place name, time and supplies in words"] = status.begins_with(WorldMapView.PLAY_STATUS % [RegionMapView.place_name(gs), gs.time, RegionMapView.supplies_text(gs)])
	checks["status: no location id, no rolls"] = _no_debug_text(status, gs)
	var supplies0: int = gs.party.supplies
	gs.party.supplies = 0
	wm._refresh()
	checks["out of supplies: the OUT_OF_SUPPLIES warning is kept"] = wm.status_text().contains(WorldMapView.OUT_OF_SUPPLIES) and _no_debug_text(wm.status_text(), gs)
	gs.party.supplies = supplies0
	wm.travel_to(dest)
	var log_text: String = "\n".join(PackedStringArray(wm.log_lines + region.log_lines))
	var dest_name: String = RegionMapView.location_name(gs, dest)
	if not log_text.contains(dest_name) or log_text.contains(dest):
		print("      log after travel (%s / %s): %s" % [dest, dest_name, log_text.replace("\n", " | ")])
	checks["travelled: the log names the place, never its id"] = gs.location_id == dest and dest_name != dest and log_text.contains(dest_name) and not log_text.contains(dest)
	if region.world_map != null:
		region.close_world_map()
	return _report("World map in play (MAPDEV1-U)", checks)

# `view`: the standalone world map debug scene with a game.
static func run_world_debug(view: WorldMapView) -> bool:
	var checks: Dictionary = {}
	view._refresh()
	var gs: GameState = view.gs
	checks["the debug scene sets debug_tools"] = view.debug_tools
	checks["world picker shown"] = view._world_pick != null and view._world_pick.is_visible_in_tree()
	checks["debug status line: location id and rolls"] = gs != null and view.status_text().contains(gs.location_id) and view.status_text().contains("rolls")
	return _report("World map debug scene keeps its debug line (MAPDEV1-U)", checks)
