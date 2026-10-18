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
# world picker starts a debug game (debug_tools only, MAPDEV1-U). REVIEW-F1: `autosave` events write slot 0 only for the main menu's game
# (CurrentGame.state()); busy() (a fight is pending) is mirrored to CurrentGame.set_busy.
# MAPVIEW1: the canvas fills the view, the top strip and the side column float over it (RegionMapView.float_panel). When
# the asset registry serves ui key "<world>_world_map" (ashlands_world_map, 1920x1080) from a file, it is drawn
# letterboxed (into the part of the canvas the floating panels leave free) behind routes and pins; a location's `map_pos` [x, y] (0..1) places its pin in that rect (the free area
# without a backdrop); locations without it keep the circle layout (layout, pure).
# MAPREAD1-L2: pins use the kind icons (ui keys pin_<kind>, MapArt placeholders; the current location is ringed gold with
# "You are here"); a gold "!" badge marks a pin a quest step points at; hovering a pin opens a card (name, kind, region,
# visited or not, route time and supply cost from here, the quest steps pointing there); a legend in the bottom-left
# corner lists the pin kinds and route styles. Text sizes follow the theme's text size.
# ==============================================================================

signal arrived(location_qid: String)
signal closed
signal fight_settled(res: Dictionary)

const PIN_RADIUS: float = 12.0   # the art contract is a 24x24 pin per kind
const PIN_SIZE: float = 24.0
const HERE_SIZE: float = 36.0   # the gold ring under the party's pin
const BADGE_SIZE: float = 16.0
const CARD_WIDTH: float = 250.0
const HERE_TEXT: String = "You are here"
const COL_UNVISITED: Color = Color(0.68, 0.74, 0.80)
const COL_LEGEND_BG: Color = Color(0.04, 0.05, 0.08, 0.78)
const LEGEND_PAD: float = 8.0
const ROUTE_SAMPLE: float = 30.0   # width of the line sample in the legend
const ROUTE_STYLES: Array[Dictionary] = [{"text": "Open road", "style": "open"}, {"text": "Closed road", "style": "closed"}, {"text": "One-way road", "style": "oneway"}]
const CLICK_RADIUS: float = 20.0
const LAYOUT_FRACTION: float = 0.38
const BACKDROP_SUFFIX: String = "_world_map"   # ui key "<world id>_world_map" (ashlands_world_map), variant "background"
const BACKDROP_VARIANT: String = "background"
const NO_POS: Vector2 = Vector2(-1.0, -1.0)
const PLAY_STATUS: String = "%s   Time %d   %s"   # MAPDEV1-U: place name, clock, RegionMapView.supplies_text
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
# MAPDEV1-U: the world picker / New game (standalone) and the debug status line (location id, time, supplies, rolls) only
# when this is on (read in _ready). world_map_debug.tscn sets it; RegionMapView passes its own flag when it embeds the
# view. Off (the main menu's game): the status shows the place name, the time and the supplies in words.
@export var debug_tools: bool = false
var launch_errors: Array[String] = []
var log_lines: Array[String] = []

var backdrop: Texture2D = null   # MAPVIEW1: the world's backdrop (backdrop_for) or null (plain look); set by show_state
var _root: Control = null   # the canvas and the floating panels (hidden while a battle runs)
var _top: PanelContainer = null
var _side: PanelContainer = null
var _world_pick: OptionButton = null
var _status: Label = null
var _canvas: Control = null
var _back: Button = null
var _fight_panel: PanelContainer = null
var _fight_label: Label = null
var _dev_box: VBoxContainer = null
var _log: Label = null
var hover_node: String = ""   # the known node under the pointer (its card shows)
var _card: PanelContainer = null
var _card_label: Label = null

func _ready() -> void:
	# Offsets too: added inside a running scene (embedded over the region map) the plain preset keeps the zero size.
	set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	_root = Control.new()
	_root.set_anchors_preset(Control.PRESET_FULL_RECT)
	_root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_root)
	_canvas = RegionMapView.map_canvas(_root)
	_canvas.texture_filter = CanvasItem.TEXTURE_FILTER_LINEAR   # the painted backdrop is scaled, not pixel art
	_canvas.draw.connect(_draw_map)
	_canvas.gui_input.connect(_on_canvas_input)
	var bar := HBoxContainer.new()
	_top = RegionMapView.float_panel(_root, bar, false)
	if embedded:
		_back = UiTheme.button("Back", "Close the world map without travelling", back)
		bar.add_child(_back)
	elif debug_tools:
		_world_pick = OptionButton.new()
		for w: String in WorldCatalog.world_ids():
			_world_pick.add_item(w)
		bar.add_child(_world_pick)
		bar.add_child(UiTheme.button("New game", "NewGame.create with the first party units, then ExploreRunner.begin", _on_new_game))
	_status = UiTheme.label("")
	bar.add_child(_status)
	var side := VBoxContainer.new()
	_side = RegionMapView.float_panel(_root, side, true)
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
	_top.resized.connect(func() -> void: RegionMapView.dock_below(_top, _side))
	_canvas.mouse_exited.connect(func() -> void: _set_hover(""))
	_card = PanelContainer.new()
	_card.add_theme_stylebox_override("panel", UiTheme.box(Color(0.06, 0.07, 0.11, 0.94), COL_CURRENT, 1, 8, 6, 0))
	_card.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_card_label = UiTheme.label("")
	_card_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_card_label.custom_minimum_size = Vector2(CARD_WIDTH, 0)
	_card_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_card.add_child(_card_label)
	_card.visible = false
	_root.add_child(_card)
	_refresh()

func _on_new_game() -> void:
	if _world_pick != null and _world_pick.item_count > 0:
		start_game(_world_pick.get_item_text(_world_pick.selected))

# Standalone: a new debug game in `world_id` (RegionMapView.new_state), entered at the world's start location.
func start_game(world_id: String, p_seed: int = RegionMapView.DEBUG_SEED) -> Array[String]:
	if not debug_tools:   # MAPDEV1-U: the debug scene only
		_note(RegionMapView.DEBUG_ONLY_REFUSAL)
		_refresh()
		return [RegionMapView.DEBUG_ONLY_REFUSAL] as Array[String]
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
	backdrop = backdrop_for(gs.world_id) if gs != null else null
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

# The backdrop texture of `world_id` (ui key "<world_id>_world_map"), or null when no manifest file serves it (the
# plain look; a missing key falls back silently).
static func backdrop_for(world_id: String) -> Texture2D:
	return RegionMapView.ui_file_texture(world_id + BACKDROP_SUFFIX, BACKDROP_VARIANT)

# The rect of a `tex_size` picture fitted into `area` with its aspect kept, centred (letterboxed).
static func letterbox(tex_size: Vector2, area: Rect2) -> Rect2:
	if tex_size.x <= 0.0 or tex_size.y <= 0.0:
		return area
	var s: float = minf(area.size.x / tex_size.x, area.size.y / tex_size.y)
	var fit: Vector2 = tex_size * s
	return Rect2(area.position + (area.size - fit) / 2.0, fit)

# A location's `map_pos` ([x, y], each 0..1, clamped) or NO_POS when missing or malformed.
static func norm_pos(raw: Variant) -> Vector2:
	var arr: Array = DictUtil.array_of(raw)
	if arr.size() != 2:
		return NO_POS
	for v: Variant in arr:
		if not (v is int or v is float):
			return NO_POS
	return Vector2(clampf(DictUtil.to_float(arr[0]), 0.0, 1.0), clampf(DictUtil.to_float(arr[1]), 0.0, 1.0))

# Pin layout (pure): known node qid -> pin centre inside `rect`. A node with a valid `map_pos` (qid -> raw value) sits
# at that point of the rect; any other gets its fixed slot on a circle (graph node order in `all`), so pins keep their
# place as the fog lifts; the circle sits in `circle` (default: `rect`). Only known nodes are returned.
static func layout(all: Array[String], known: Array[String], map_pos: Dictionary, rect: Rect2, circle: Rect2 = Rect2()) -> Dictionary:
	var out: Dictionary = {}
	var ring: Rect2 = circle if circle.has_area() else rect
	var center: Vector2 = ring.get_center()
	var radius: float = minf(ring.size.x, ring.size.y) * LAYOUT_FRACTION
	for i: int in range(all.size()):
		if not known.has(all[i]):
			continue
		var at: Vector2 = norm_pos(map_pos.get(all[i], null))
		if at != NO_POS:
			out[all[i]] = rect.position + at * rect.size
			continue
		var angle: float = TAU * float(i) / float(all.size()) - PI / 2.0
		out[all[i]] = center + Vector2(cos(angle), sin(angle)) * radius
	return out

# The rect pins are placed in: the part of the canvas the floating panels leave free (no pin sits under a panel), with the
# backdrop letterboxed into it when there is one.
func layout_rect() -> Rect2:
	var canvas: Vector2 = _canvas.size if _canvas != null and _canvas.size.x > 0.0 else Vector2(520, 360)
	var area: Rect2 = RegionMapView.free_rect(canvas, _top, _side)
	return letterbox(backdrop.get_size(), area) if backdrop != null else area

# Known node qid -> pin centre (layout over the graph nodes, their locations' `map_pos` and layout_rect(); the circle
# fallback sits in the same rect).
func pin_positions() -> Dictionary:
	if gs == null:
		return {}
	var all: Array[String] = TravelRules.nodes(gs.world_id)
	var map_pos: Dictionary = {}
	for q: String in all:
		var raw: Variant = ExploreRunner.resolve_location(gs, q).get("map_pos", null)
		if raw != null:
			map_pos[q] = raw
	return layout(all, TravelRules.known(gs), map_pos, layout_rect())

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
	var motion: InputEventMouseMotion = event as InputEventMouseMotion
	if motion != null:
		_set_hover(node_at(motion.position))
		return
	if event is InputEventMouseButton:
		var mb: InputEventMouseButton = event
		if mb.pressed and mb.button_index == MOUSE_BUTTON_LEFT:
			var q: String = node_at(mb.position)
			if not q.is_empty():
				travel_to(q)

# ------------------------------------------------------------------------------
# Hover card and text (MAPREAD1-L2)
# ------------------------------------------------------------------------------

func label_font_size() -> int:
	return RegionMapView.name_font_size(get_theme_default_font_size())

# The card lines of known node `qid`: name, "Kind, Region", visited or not, the trip from the party's location (time and
# supplies) or "You are here", and the active quest steps pointing there.
func card_lines(qid: String) -> Array[String]:
	var out: Array[String] = []
	if gs == null:
		return out
	var info: Dictionary = {}
	for n: Variant in DictUtil.array_of(TravelRules.view(gs)["nodes"]):
		if str(DictUtil.dict_of(n).get("qid", "")) == qid:
			info = DictUtil.dict_of(n)
	if info.is_empty():
		return out
	out.append(str(info["name"]))
	var where: Array[String] = []
	if not str(info["kind"]).is_empty():
		where.append(str(info["kind"]).capitalize())
	if not str(info["region"]).is_empty():
		where.append(str(info["region"]))
	if not where.is_empty():
		out.append(", ".join(PackedStringArray(where)))
	out.append("Visited" if DictUtil.to_bool(info["visited"]) else "Not visited yet")
	if qid == gs.location_id:
		out.append(HERE_TEXT)
	else:
		var trip: Dictionary = TravelRules.trip(gs, TravelRunner.current_node(gs), qid)
		out.append("Trip: time %d, supplies %d" % [DictUtil.to_int(trip["time"]), DictUtil.to_int(trip["supplies"])] if DictUtil.to_bool(trip["ok"]) else "No known way from here")
	for step: String in TravelRules.quest_steps_at(gs, qid):
		out.append("Quest: %s" % step)
	return out

func _set_hover(qid: String) -> void:
	if qid == hover_node:
		return
	hover_node = qid
	_update_card()

# Shows the card of the hovered pin next to it, kept inside the canvas.
func _update_card() -> void:
	if _card == null:
		return
	var lines: Array[String] = []
	if not hover_node.is_empty():
		lines.assign(card_lines(hover_node))
	_card.visible = not lines.is_empty() and battle == null
	if lines.is_empty():
		return
	_card_label.text = "\n".join(PackedStringArray(lines))
	_card_label.add_theme_font_size_override("font_size", label_font_size())
	_card.reset_size()
	var pin: Vector2 = pin_positions().get(hover_node, Vector2.ZERO)
	_card.position = card_origin(pin, _card.get_combined_minimum_size(), RegionMapView.free_rect(_canvas.size, _top, _side))

# Card position (pure): right of the pin, left when it would leave `bounds`, then clamped inside them.
static func card_origin(pin: Vector2, card: Vector2, bounds: Rect2) -> Vector2:
	var x: float = pin.x + PIN_RADIUS + 10.0
	if x + card.x > bounds.end.x:
		x = pin.x - PIN_RADIUS - 10.0 - card.x
	var y: float = pin.y - card.y / 2.0
	return Vector2(maxf(minf(x, bounds.end.x - card.x), bounds.position.x), maxf(minf(y, bounds.end.y - card.y), bounds.position.y))

# Baseline-left origin of a pin's name (pure): right of the pin, left of it when it would leave `bounds`.
static func name_origin(pin: Vector2, text_size: Vector2, ascent: float, bounds: Rect2) -> Vector2:
	var x: float = pin.x + PIN_RADIUS + 4.0
	if x + text_size.x > bounds.end.x:
		x = pin.x - PIN_RADIUS - 4.0 - text_size.x
	return Vector2(maxf(x, bounds.position.x + 2.0), pin.y + ascent * 0.35)

# Legend rows: pin kinds, then the route styles: [{text, pin (ui key or ""), style ("" or open / closed / oneway)}].
static func legend_entries() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for kind: String in MapArt.PIN_KINDS:
		out.append({"text": kind.capitalize(), "pin": MapArt.pin_key(kind), "style": ""})
	for r: Dictionary in ROUTE_STYLES:
		out.append({"text": str(r["text"]), "pin": "", "style": str(r["style"])})
	return out

# The legend box in the bottom-left corner of `bounds` for text of size `fs`.
static func legend_rect(font: Font, fs: int, bounds: Rect2) -> Rect2:
	var widest: float = 0.0
	for e: Dictionary in legend_entries():
		widest = maxf(widest, font.get_string_size(str(e["text"]), HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x)
	var row: float = legend_row_height(fs)
	var size := Vector2(LEGEND_PAD * 2.0 + ROUTE_SAMPLE + 8.0 + widest, LEGEND_PAD * 2.0 + row * legend_entries().size())
	return Rect2(Vector2(bounds.position.x + 8.0, bounds.end.y - size.y - 8.0), size)

static func legend_row_height(fs: int) -> float:
	return maxf(float(fs) + 6.0, PIN_SIZE + 2.0)

# ------------------------------------------------------------------------------
# Travel and fights
# ------------------------------------------------------------------------------

# A click on a pin: TravelRunner.travel towards `location_qid` (ignored while a fight is pending). A click on the
# party's own location enters it (`arrived`).
func travel_to(location_qid: String) -> void:
	if gs == null or busy():
		return
	if location_qid == gs.location_id:
		if TravelRunner.current_node(gs).is_empty():
			_note("%s is not on the world map (no route)." % location_qid)
		else:
			_note("Entering %s." % location_qid)
			arrived.emit(gs.location_id)
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
			_note("Arrived at %s (time %d, supplies -%d)." % [RegionMapView.location_name(gs, gs.location_id), gs.time, DictUtil.to_int(res.get("supplies_spent", 0))])
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
	var report: Dictionary = battle.report() if battle != null else {}   # BREPORT1 1.5: built before the arena closes
	if not keep_battle:
		_close_battle()
	var res: Dictionary = TravelRunner.after_fight(gs, fight, result)
	_note("Road fight %s: %s (quests advanced: %s)" % [str(fight["encounter"]), result, str(res.get("advanced", []))])
	_handle(res, gs)
	if DictUtil.to_bool(res.get("ok", false)):
		res["outcome"] = result
		res["report"] = report
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
				_note("Entered %s." % RegionMapView.location_name(state, str(ev["location"])))   # MAPDEV1-U: the place name
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

# The status line's first row: debug = "<qid>  time n  supplies n/max  rolls {..}" (the idle prompt without a game); play =
# "<place>  Time n  Supplies n / max" (PLAY_STATUS; "" without a game).
static func status_line(state: GameState, debug: bool) -> String:
	if state == null:
		return RegionMapView.DEBUG_IDLE if debug else ""
	if debug:
		return "%s  time %d  supplies %d/%d  rolls %s" % [state.location_id, state.time, state.party.supplies, PartyState.SUPPLIES_MAX, str(state.rolls)]
	return PLAY_STATUS % [RegionMapView.place_name(state), state.time, RegionMapView.supplies_text(state)]

# The status line as shown (tests).
func status_text() -> String:
	return _status.text if _status != null else ""

func _refresh() -> void:
	_sync_busy()
	MusicDirector.request_game(self, MusicDirector.CTX_TRAVEL, gs)   # MUSIC1: travel falls back to field
	if _status == null:
		return
	_status.text = status_line(gs, debug_tools)
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
	_update_card()
	_log.text = "\n".join(PackedStringArray(log_lines))
	_canvas.queue_redraw()

func _draw_map() -> void:
	if gs == null:
		return
	if backdrop != null:
		_canvas.draw_texture_rect(backdrop, layout_rect(), false)
	var view: Dictionary = TravelRules.view(gs)
	var pins: Dictionary = pin_positions()
	var font: Font = get_theme_default_font()
	var fs: int = label_font_size()
	var area: Rect2 = RegionMapView.free_rect(_canvas.size, _top, _side)
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
		_canvas.draw_string(font, mid + Vector2(4, -4), str(route["time"]), HORIZONTAL_ALIGNMENT_LEFT, -1, maxi(fs - 1, 9), COL_ROUTE)
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
		var here: bool = q == gs.location_id
		var visited: bool = DictUtil.to_bool(node["visited"])
		var col: Color = COL_CURRENT if here else (COL_VISITED if visited else COL_KNOWN)
		if here:
			_canvas.draw_texture_rect(AssetRegistry.texture("ui", "pin_here", "icon"), Rect2(p - Vector2(HERE_SIZE, HERE_SIZE) / 2.0, Vector2(HERE_SIZE, HERE_SIZE)), false)
		var key: String = MapArt.pin_key(str(node["kind"]))
		if key.is_empty():
			_canvas.draw_circle(p, PIN_RADIUS, col)
		else:
			_canvas.draw_texture_rect(AssetRegistry.texture("ui", key, "icon"), Rect2(p - Vector2(PIN_SIZE, PIN_SIZE) / 2.0, Vector2(PIN_SIZE, PIN_SIZE)), false, Color.WHITE if visited or here else COL_UNVISITED)
		if DictUtil.to_bool(node["quest"]):
			_canvas.draw_texture_rect(AssetRegistry.texture("ui", "badge_quest", "icon"), Rect2(p + Vector2(PIN_RADIUS - 6.0, -PIN_RADIUS - BADGE_SIZE + 6.0), Vector2(BADGE_SIZE, BADGE_SIZE)), false)
		var name_text: String = str(node["name"])
		var tsize: Vector2 = font.get_string_size(name_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
		var at: Vector2 = name_origin(p, tsize, font.get_ascent(fs), area)
		_canvas.draw_string_outline(font, at, name_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, 4, RegionMapView.COL_LABEL_EDGE)
		_canvas.draw_string(font, at, name_text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, col)
		if here:
			var hat: Vector2 = Vector2(at.x, at.y + fs + 2.0)
			_canvas.draw_string_outline(font, hat, HERE_TEXT, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, 4, RegionMapView.COL_LABEL_EDGE)
			_canvas.draw_string(font, hat, HERE_TEXT, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, COL_CURRENT)
	_draw_legend(font, fs, area)

# The pin kinds and road styles in the bottom-left corner.
func _draw_legend(font: Font, fs: int, area: Rect2) -> void:
	var box: Rect2 = legend_rect(font, fs, area)
	_canvas.draw_rect(box, COL_LEGEND_BG)
	var row: float = legend_row_height(fs)
	var y: float = box.position.y + LEGEND_PAD
	for e: Dictionary in legend_entries():
		var mid: float = y + row / 2.0
		var x0: float = box.position.x + LEGEND_PAD
		match str(e["style"]):
			"":
				_canvas.draw_texture_rect(AssetRegistry.texture("ui", str(e["pin"]), "icon"), Rect2(Vector2(x0 + (ROUTE_SAMPLE - PIN_SIZE) / 2.0, mid - PIN_SIZE / 2.0), Vector2(PIN_SIZE, PIN_SIZE)), false)
			"open":
				_canvas.draw_line(Vector2(x0, mid), Vector2(x0 + ROUTE_SAMPLE, mid), COL_ROUTE, 2.0)
			"closed":
				_canvas.draw_dashed_line(Vector2(x0, mid), Vector2(x0 + ROUTE_SAMPLE, mid), COL_CLOSED, 2.0, DASH)
			"oneway":
				var tip: Vector2 = Vector2(x0 + ROUTE_SAMPLE, mid)
				_canvas.draw_line(Vector2(x0, mid), tip, COL_ROUTE, 2.0)
				var side: Vector2 = Vector2(0.0, ARROW_SIZE * 0.5)
				_canvas.draw_colored_polygon(PackedVector2Array([tip, tip - Vector2(ARROW_SIZE, 0.0) + side, tip - Vector2(ARROW_SIZE, 0.0) - side]), COL_ROUTE)
		_canvas.draw_string(font, Vector2(x0 + ROUTE_SAMPLE + 8.0, mid + fs * 0.35), str(e["text"]), HORIZONTAL_ALIGNMENT_LEFT, -1, fs, RegionMapView.COL_LABEL)
		y += row
