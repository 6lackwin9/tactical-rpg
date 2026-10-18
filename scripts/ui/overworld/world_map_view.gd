class_name WorldMapView
extends Control

# ==============================================================================
# Tactical RPG - World map debug view (S10, spec TRAV1-L commit 3)
# Thin: draws TravelRules.view(gs) (known nodes as pins on an automatic circle layout in graph node order; routes as
# lines: closed dashed, one-way with an arrow) and forwards a click on a pin to TravelRunner.travel. A road fight opens
# the real combat arena (BattleScreen) and settles through TravelRunner.after_fight; if the battle cannot start, the
# fight panel offers Start battle again and the debug outcomes (`dev_outcomes` always shows them). The clock shows
# GameState.time. Embedded by RegionMapView on a world_map exit: `arrived` (trip done, or woken after a lost fight)
# hands back to the region map, Back closes without travelling. Standalone (scenes/overworld/world_map_debug.tscn): a
# world picker starts a debug game. REVIEW-F1: `autosave` events write slot 0 only for the main menu's game
# (CurrentGame.state()); busy() (a fight is pending) is mirrored to CurrentGame.set_busy.
# ==============================================================================

signal arrived(location_qid: String)
signal closed
signal fight_settled(res: Dictionary)

const PIN_RADIUS: float = 9.0
const CLICK_RADIUS: float = 16.0
const LAYOUT_FRACTION: float = 0.38
const OUT_OF_SUPPLIES: String = "Out of supplies: road fights are more likely."   # TRAV2-L status warning at 0
const ARROW_SIZE: float = 10.0
const DASH: float = 6.0
const COL_ROUTE: Color = Color(0.75, 0.70, 0.55)
const COL_CLOSED: Color = Color(0.55, 0.30, 0.30)
const COL_CURRENT: Color = Color(0.95, 0.85, 0.30)
const COL_VISITED: Color = Color(0.90, 0.92, 0.96)
const COL_KNOWN: Color = Color(0.50, 0.55, 0.60)

var gs: GameState = null
var embedded: bool = false   # set before entering the tree: true = opened by the region map (Back, no world picker)
var pending_fight: Dictionary = {}
var battle: BattleScreen = null
var results_popup: bool = false   # BATTLE-FLOW: see RegionMapView
var dev_outcomes: bool = false
var launch_errors: Array[String] = []
var log_lines: Array[String] = []

var _root: VBoxContainer = null
var _world_pick: OptionButton = null
var _status: Label = null
var _canvas: Control = null
var _back: Button = null
var _fight_panel: PanelContainer = null
var _fight_label: Label = null
var _dev_box: VBoxContainer = null
var _log: Label = null

func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	_root = VBoxContainer.new()
	_root.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(_root)
	var bar := HBoxContainer.new()
	_root.add_child(bar)
	if embedded:
		_back = UiTheme.button("Back", "Close the world map without travelling", back)
		bar.add_child(_back)
	else:
		_world_pick = OptionButton.new()
		for w: String in WorldCatalog.world_ids():
			_world_pick.add_item(w)
		bar.add_child(_world_pick)
		bar.add_child(UiTheme.button("New game", "NewGame.create with the first party units, then ExploreRunner.begin", _on_new_game))
	_status = UiTheme.label("")
	bar.add_child(_status)
	var body := HBoxContainer.new()
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_root.add_child(body)
	_canvas = Control.new()
	_canvas.custom_minimum_size = Vector2(520, 360)
	_canvas.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_canvas.mouse_filter = Control.MOUSE_FILTER_STOP
	_canvas.draw.connect(_draw_map)
	_canvas.gui_input.connect(_on_canvas_input)
	body.add_child(_canvas)
	var side := VBoxContainer.new()
	side.custom_minimum_size = Vector2(260, 0)
	body.add_child(side)
	_fight_panel = PanelContainer.new()
	var fight_box := VBoxContainer.new()
	_fight_panel.add_child(fight_box)
	_fight_label = UiTheme.label("")
	fight_box.add_child(_fight_label)
	fight_box.add_child(UiTheme.button("Start battle", "Open the combat arena with the active party (BattleScreen)", launch_battle))
	_dev_box = VBoxContainer.new()
	fight_box.add_child(_dev_box)
	for result: String in BattleBridge.OUTCOMES:
		_dev_box.add_child(UiTheme.button("Dev: " + result, "Settle the road fight as %s without a battle (TravelRunner.after_fight)" % result, settle.bind(result)))
	_fight_panel.visible = false
	side.add_child(_fight_panel)
	_log = UiTheme.label("")
	_log.autowrap_mode = TextServer.AUTOWRAP_WORD
	_log.custom_minimum_size = Vector2(260, 0)
	side.add_child(_log)
	_refresh()

func _on_new_game() -> void:
	if _world_pick != null and _world_pick.item_count > 0:
		start_game(_world_pick.get_item_text(_world_pick.selected))

# Standalone: a new debug game in `world_id` (RegionMapView.new_state), entered at the world's start location.
func start_game(world_id: String, p_seed: int = RegionMapView.DEBUG_SEED) -> Array[String]:
	var res: Dictionary = RegionMapView.new_state(world_id, p_seed)
	var errors: Array[String] = []
	errors.assign(DictUtil.array_of(res["errors"]))
	var state: GameState = GameState.of(res["state"])
	if state == null:
		_note("New game refused: %s" % "; ".join(PackedStringArray(errors)))
		return errors
	_handle(ExploreRunner.begin(state), state)
	show_state(state)
	return errors

# Shows the world map of `state` (the region map's game when embedded).
func show_state(state: GameState) -> void:
	gs = state
	pending_fight = {}
	_close_battle()
	if gs != null and TravelRunner.current_node(gs).is_empty():
		_note("%s is not on the world map (no route)." % gs.location_id)
	_refresh()

func back() -> void:
	if busy():
		return
	closed.emit()

# True while a road fight waits (clicks are ignored, saving is refused).
func busy() -> bool:
	return not pending_fight.is_empty()

# ------------------------------------------------------------------------------
# Layout
# ------------------------------------------------------------------------------

# Known node qid -> pin centre: every graph node gets a fixed slot on a circle (graph node order), so pins keep their
# place as the fog lifts; only known nodes are returned.
func pin_positions() -> Dictionary:
	var out: Dictionary = {}
	if gs == null:
		return out
	var all: Array[String] = TravelRules.nodes(gs.world_id)
	var known: Array[String] = TravelRules.known(gs)
	var area: Vector2 = _canvas.size if _canvas != null and _canvas.size.x > 0.0 else Vector2(520, 360)
	var center: Vector2 = area / 2.0
	var radius: float = minf(area.x, area.y) * LAYOUT_FRACTION
	for i: int in range(all.size()):
		if not known.has(all[i]):
			continue
		var angle: float = TAU * float(i) / float(all.size()) - PI / 2.0
		out[all[i]] = center + Vector2(cos(angle), sin(angle)) * radius
	return out

# The known node under canvas point `p` ("" = none).
func node_at(p: Vector2) -> String:
	var best: String = ""
	var best_d: float = CLICK_RADIUS
	var pins: Dictionary = pin_positions()
	for q: Variant in pins:
		var pin: Vector2 = pins[q]
		var d: float = pin.distance_to(p)
		if d <= best_d:
			best = str(q)
			best_d = d
	return best

func _on_canvas_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		var mb: InputEventMouseButton = event
		if mb.pressed and mb.button_index == MOUSE_BUTTON_LEFT:
			var q: String = node_at(mb.position)
			if not q.is_empty():
				travel_to(q)

# ------------------------------------------------------------------------------
# Travel and fights
# ------------------------------------------------------------------------------

# A click on a pin: TravelRunner.travel towards `location_qid` (ignored while a fight is pending).
func travel_to(location_qid: String) -> void:
	if gs == null or busy():
		return
	if location_qid == gs.location_id:
		_note("The party is already at %s." % location_qid)
		_refresh()
		return
	var res: Dictionary = TravelRunner.travel(gs, location_qid)
	_handle(res, gs)
	match str(res.get("status", "")):
		"fight":
			pending_fight = DictUtil.dict_of(res["fight"])
			_sync_busy()
			_note("Road fight! %s (%s%s) after %d leg(s)" % [str(pending_fight["encounter"]), str(pending_fight["source"]), ", elite" if DictUtil.to_bool(pending_fight["elite"]) else "", DictUtil.to_int(res["legs_done"])])
			launch_battle.call_deferred()
		"arrived":
			_note("Arrived at %s (time %d, supplies -%d)." % [gs.location_id, gs.time, DictUtil.to_int(res.get("supplies_spent", 0))])
			arrived.emit(gs.location_id)
	_refresh()

func launch_battle() -> void:
	launch_errors.clear()
	if gs == null or pending_fight.is_empty() or battle != null:
		_refresh()
		return
	var res: Dictionary = BattleScreen.open(self, gs, pending_fight)
	launch_errors.assign(DictUtil.array_of(res["errors"]))
	var screen: BattleScreen = res["screen"] as BattleScreen
	if screen == null:
		_note("Battle refused: %s" % "; ".join(PackedStringArray(launch_errors)))
	else:
		battle = screen
		battle.auto_results = results_popup
		battle.finished.connect(_on_battle_finished)
	_refresh()

func _on_battle_finished(result: String) -> void:
	settle(result, results_popup)

func close_settled_battle() -> void:
	_close_battle()
	_refresh()

func _close_battle() -> void:
	if battle != null:
		battle.close()
		battle = null
	launch_errors.clear()

# Settles the pending road fight through TravelRunner.after_fight. won: the leg completes and the map stays open (click
# again to go on); fled: the party stays at the leg's start; lost: the party wakes in a location, so `arrived` hands
# back. `fight_settled` carries the result plus {outcome}.
func settle(result: String, keep_battle: bool = false) -> void:
	if gs == null or pending_fight.is_empty():
		return
	var fight: Dictionary = pending_fight
	pending_fight = {}
	if not keep_battle:
		_close_battle()
	var res: Dictionary = TravelRunner.after_fight(gs, fight, result)
	_note("Road fight %s: %s (quests advanced: %s)" % [str(fight["encounter"]), result, str(res.get("advanced", []))])
	_handle(res, gs)
	if DictUtil.to_bool(res.get("ok", false)):
		res["outcome"] = result
		fight_settled.emit(res)
		if result == "lost":
			arrived.emit(gs.location_id)
	_refresh()

func _handle(res: Dictionary, state: GameState) -> void:
	if not DictUtil.to_bool(res.get("ok", false)):
		_note("Refused: %s" % "; ".join(PackedStringArray(JsonUtil.strings(res.get("errors", [])))))
	for e: Variant in DictUtil.array_of(res.get("events", [])):
		var ev: Dictionary = DictUtil.dict_of(e)
		match str(ev.get("kind", "")):
			"entered":
				_note("Entered %s." % str(ev["location"]))
			"flags_set":
				_note("Flags set: %s" % str(ev["flags"]))
			"autosave":
				_autosave(state)

func _autosave(state: GameState) -> void:
	if state == null or not pending_fight.is_empty():
		return
	if state != CurrentGame.state():
		_note("Autosave skipped (debug game).")
		return
	var errors: Array[String] = SaveGame.autosave(state)
	if errors.is_empty():
		CurrentGame.mark_saved(SaveGame.AUTOSAVE_SLOT)
	else:
		_note("Autosave refused: %s" % "; ".join(PackedStringArray(errors)))

func _note(text: String) -> void:
	log_lines.append(text)
	while log_lines.size() > 12:
		log_lines.pop_front()

# Mirrors busy() into CurrentGame for the main menu's game, so SaveSlotsModel refuses saves mid-fight.
func _sync_busy() -> void:
	if gs != null and gs == CurrentGame.state():
		CurrentGame.set_busy(busy())

func _exit_tree() -> void:
	if gs != null and gs == CurrentGame.state():
		CurrentGame.set_busy(false)

# ------------------------------------------------------------------------------
# Drawing
# ------------------------------------------------------------------------------

# The status line as shown (tests).
func status_text() -> String:
	return _status.text if _status != null else ""

func _refresh() -> void:
	_sync_busy()
	if _status == null:
		return
	_status.text = "Pick a world and start a new game." if gs == null else "%s  time %d  supplies %d/%d  rolls %s" % [gs.location_id, gs.time, gs.party.supplies, PartyState.SUPPLIES_MAX, str(gs.rolls)]
	if gs != null and gs.party.supplies == 0:
		_status.text += "\n" + OUT_OF_SUPPLIES   # TRAV2-L: the starved warning
	if _back != null:
		_back.disabled = busy()
	_fight_panel.visible = not pending_fight.is_empty() and battle == null
	_fight_label.text = "Road fight: %s" % str(pending_fight.get("encounter", ""))
	if not launch_errors.is_empty():
		_fight_label.text += "\nBattle refused: %s" % "; ".join(PackedStringArray(launch_errors))
	_dev_box.visible = dev_outcomes or not launch_errors.is_empty()
	_root.visible = battle == null
	_log.text = "\n".join(PackedStringArray(log_lines))
	_canvas.queue_redraw()

func _draw_map() -> void:
	if gs == null:
		return
	var view: Dictionary = TravelRules.view(gs)
	var pins: Dictionary = pin_positions()
	var font: Font = get_theme_default_font()
	for r: Variant in DictUtil.array_of(view["routes"]):
		var route: Dictionary = DictUtil.dict_of(r)
		if not pins.has(route["from"]) or not pins.has(route["to"]):
			continue
		var a: Vector2 = pins[route["from"]]
		var b: Vector2 = pins[route["to"]]
		if not DictUtil.to_bool(route["open"]):
			_canvas.draw_dashed_line(a, b, COL_CLOSED, 2.0, DASH)
			continue
		_canvas.draw_line(a, b, COL_ROUTE, 2.0)
		var mid: Vector2 = (a + b) / 2.0
		_canvas.draw_string(font, mid + Vector2(4, -4), str(route["time"]), HORIZONTAL_ALIGNMENT_LEFT, -1, UiTheme.FONT_SM, COL_ROUTE)
		if not DictUtil.to_bool(route["two_way"]):
			var dir: Vector2 = (b - a).normalized()
			var tip: Vector2 = b - dir * PIN_RADIUS
			var side: Vector2 = Vector2(-dir.y, dir.x) * ARROW_SIZE * 0.5
			_canvas.draw_colored_polygon(PackedVector2Array([tip, tip - dir * ARROW_SIZE + side, tip - dir * ARROW_SIZE - side]), COL_ROUTE)
	for n: Variant in DictUtil.array_of(view["nodes"]):
		var node: Dictionary = DictUtil.dict_of(n)
		var q: String = str(node["qid"])
		if not pins.has(q):
			continue
		var p: Vector2 = pins[q]
		var col: Color = COL_CURRENT if q == gs.location_id else (COL_VISITED if DictUtil.to_bool(node["visited"]) else COL_KNOWN)
		_canvas.draw_circle(p, PIN_RADIUS, col)
		_canvas.draw_string(font, p + Vector2(PIN_RADIUS + 3, 4), str(node["name"]), HORIZONTAL_ALIGNMENT_LEFT, -1, UiTheme.FONT_MD, col)
