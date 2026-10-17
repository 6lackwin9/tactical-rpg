class_name RegionMapView
extends Control

# ==============================================================================
# Tactical RPG - Region map debug view (S9, spec EXP1-L commit 3)
# Thin: renders the GameState's location map (tiles, zone tint, markers, roamers, token) and forwards input to
# ExploreRunner / BattleBridge. Arrow keys = step, E = interact in the facing direction. A fight shows a panel with
# three debug outcomes (won / fled / lost -> ExploreRunner.after_fight); launching the real battle is out of scope.
# `autosave` events write debug slot 9 (refused while a fight is pending). A world picker starts a new game.
# ==============================================================================

const DEBUG_SLOT: int = 9
const TILE: int = 22
const DEBUG_SEED: int = 7
const COL_TILES: Dictionary = {".": Color(0.22, 0.22, 0.24), "#": Color(0.45, 0.38, 0.30), "~": Color(0.18, 0.32, 0.55)}
const COL_DANGER: Color = Color(0.85, 0.20, 0.15, 0.28)
const COL_SAFE: Color = Color(0.20, 0.75, 0.30, 0.18)
const MARKER_GLYPHS: Dictionary = {"exit": "⇨", "encounter": "!", "roamer": "R", "npc": "N", "chest": "C", "lore": "L", "point": "•"}

var gs: GameState = null
var pending_fight: Dictionary = {}
var facing: Vector2i = Vector2i(0, 1)
var log_lines: Array[String] = []

var _world_pick: OptionButton = null
var _status: Label = null
var _canvas: Control = null
var _menu_box: VBoxContainer = null
var _fight_panel: PanelContainer = null
var _fight_label: Label = null
var _log: Label = null

func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(root)
	var bar := HBoxContainer.new()
	root.add_child(bar)
	_world_pick = OptionButton.new()
	for w: String in WorldCatalog.world_ids():
		_world_pick.add_item(w)
	bar.add_child(_world_pick)
	bar.add_child(UiTheme.button("New game", "NewGame.create with the first party units, then ExploreRunner.begin", _on_new_game))
	_status = UiTheme.label("Pick a world and start a new game.")
	bar.add_child(_status)
	var body := HBoxContainer.new()
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(body)
	_canvas = Control.new()
	_canvas.custom_minimum_size = Vector2(TILE * 24, TILE * 16)
	_canvas.draw.connect(_draw_map)
	body.add_child(_canvas)
	var side := VBoxContainer.new()
	side.custom_minimum_size = Vector2(260, 0)
	body.add_child(side)
	_menu_box = VBoxContainer.new()
	side.add_child(_menu_box)
	_fight_panel = PanelContainer.new()
	var fight_box := VBoxContainer.new()
	_fight_panel.add_child(fight_box)
	_fight_label = UiTheme.label("")
	fight_box.add_child(_fight_label)
	for result: String in BattleBridge.OUTCOMES:
		fight_box.add_child(UiTheme.button(result.capitalize(), "Settle the fight as %s (ExploreRunner.after_fight)" % result, settle.bind(result)))
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

# A new game in `world_id` with the first party units of the catalog, entered at the world's start location.
func start_game(world_id: String, p_seed: int = DEBUG_SEED) -> Array[String]:
	var units: Array[String] = []
	for e: Dictionary in ContentCatalog.entries("units"):
		if str(e.get("side", "")) == "party" and ContentFactory.class_of(e) != null and units.size() < 2:
			units.append(str(e["id"]))
	var res: Dictionary = NewGame.create(world_id, units, p_seed)
	var errors: Array[String] = []
	errors.assign(DictUtil.array_of(res["errors"]))
	if res["state"] == null:
		_note("New game refused: %s" % "; ".join(PackedStringArray(errors)))
		return errors
	var state: Variant = res["state"]
	if not (state is GameState):
		_note("New game refused: no game state returned")
		return errors
	gs = state
	pending_fight = {}
	_handle(ExploreRunner.begin(gs))
	return errors

func _unhandled_input(event: InputEvent) -> void:
	if not (event is InputEventKey) or not event.is_pressed() or event.is_echo():
		return
	var key: InputEventKey = event
	var dirs: Dictionary = {KEY_UP: Vector2i(0, -1), KEY_DOWN: Vector2i(0, 1), KEY_LEFT: Vector2i(-1, 0), KEY_RIGHT: Vector2i(1, 0)}
	if dirs.has(key.keycode):
		var dir: Vector2i = dirs[key.keycode]
		do_step(dir)
		accept_event()
	elif key.keycode == KEY_E:
		do_interact()
		accept_event()

func do_step(dir: Vector2i) -> void:
	if gs == null or not pending_fight.is_empty():
		return
	facing = dir
	_handle(ExploreRunner.step(gs, dir))

func do_interact() -> void:
	if gs == null or not pending_fight.is_empty():
		return
	var res: Dictionary = ExploreRunner.interact(gs, facing)
	match str(res.get("kind", "")):
		"npc":
			_note("You speak with %s." % str(WorldCatalog.resolve(gs.world_id, "characters", str(res["character"])).get("name", res["character"])))
		"lore":
			_note("%s: %s" % [str(res["name"]), str(res["text"])])
		"chest":
			_note("The chest is empty." if DictUtil.to_bool(res.get("empty", false)) else "Opened: %s %s" % [str(res.get("items", {})), str(res.get("loot", ""))])
		_:
			if not DictUtil.to_bool(res.get("ok", true)):
				_note("Interact refused: %s" % str(res["errors"]))
	_refresh()

func take_exit(to: String) -> void:
	if gs != null and pending_fight.is_empty():
		_handle(ExploreRunner.take_exit(gs, to))

# Debug outcome button: settles the pending fight (no real battle is launched).
func settle(result: String) -> void:
	if gs == null or pending_fight.is_empty():
		return
	var fight: Dictionary = pending_fight
	pending_fight = {}
	var res: Dictionary = ExploreRunner.after_fight(gs, fight, result)
	_note("Fight %s: %s (quests advanced: %s)" % [str(fight["encounter"]), result, str(res.get("advanced", []))])
	_handle(res)

func _handle(res: Dictionary) -> void:
	if not DictUtil.to_bool(res.get("ok", false)):
		_note("Refused: %s" % str(res.get("errors", [])))
	for e: Variant in DictUtil.array_of(res.get("events", [])):
		_event(DictUtil.dict_of(e))
	_refresh()

func _event(e: Dictionary) -> void:
	match str(e.get("kind", "")):
		"entered":
			_note("Entered %s." % str(e["location"]))
		"autosave":
			_autosave()
		"locked":
			_note(str(e["text"]))
		"flags_set":
			_note("Flags set: %s" % str(e["flags"]))
		"exit":
			if str(e["to"]) == MapRules.WORLD_MAP:
				_note("The road leads to the world map (TRAV1).")
			for inner: Variant in DictUtil.array_of(e.get("events", [])):
				_event(DictUtil.dict_of(inner))
		"fight":
			pending_fight = e
			_note("Fight! %s (%s%s)" % [str(e["encounter"]), str(e["source"]), ", elite" if DictUtil.to_bool(e["elite"]) else ""])

func _autosave() -> void:
	if gs == null or not pending_fight.is_empty():
		return
	var errors: Array[String] = SaveGame.write(gs, DEBUG_SLOT)
	if not errors.is_empty():
		_note("Autosave refused: %s" % "; ".join(PackedStringArray(errors)))

func _note(text: String) -> void:
	log_lines.append(text)
	while log_lines.size() > 12:
		log_lines.pop_front()

func _refresh() -> void:
	if _status == null:
		return
	if gs == null:
		_status.text = "Pick a world and start a new game."
	else:
		_status.text = "%s  %s  steps %d / %d  rolls %s" % [gs.location_id, str(gs.position), DictUtil.to_int(gs.explore["steps"]), DictUtil.to_int(gs.explore["next_at"]), str(gs.rolls)]
	for c: Node in _menu_box.get_children():
		c.queue_free()
	if gs != null and pending_fight.is_empty():
		for item: Dictionary in ExploreRunner.menu(gs):
			_menu_box.add_child(UiTheme.button(str(item["label"]), "ExploreRunner.take_exit", take_exit.bind(str(item["to"]))))
	_fight_panel.visible = not pending_fight.is_empty()
	_fight_label.text = "Fight: %s" % str(pending_fight.get("encounter", ""))
	_log.text = "\n".join(PackedStringArray(log_lines))
	_canvas.queue_redraw()

func _draw_map() -> void:
	if gs == null:
		return
	var loc: Dictionary = ExploreRunner.location(gs)
	var t: Dictionary = ExploreRunner.template_of(loc)
	if t.is_empty():
		_canvas.draw_string(get_theme_default_font(), Vector2(8, 24), "%s (no map: use the exits on the right)" % str(loc.get("name", gs.location_id)))
		return
	var size: Vector2i = MapRules.size_of(t)
	for y: int in range(size.y):
		for x: int in range(size.x):
			var r := Rect2(Vector2(x * TILE, y * TILE), Vector2(TILE - 1, TILE - 1))
			var tile_col: Color = COL_TILES.get(MapRules.tile(t, x, y), Color.BLACK)
			_canvas.draw_rect(r, tile_col)
			var zone: String = MapRules.zone_at(t, x, y)
			if not zone.is_empty():
				_canvas.draw_rect(r, COL_DANGER if MapRules.zone_kind(t, zone) == "danger" else COL_SAFE)
	var font: Font = get_theme_default_font()
	var loc_steps: int = DictUtil.to_int(gs.explore["loc_steps"])
	for m: Dictionary in ExploreRunner.markers_of(loc):
		var state: String = ExploreRunner.presence(gs, str(loc["_qid"]), m)
		if state == "absent":
			continue
		var p: Vector2i = ExploreRunner.marker_pos(t, m, loc_steps)
		var glyph: String = str(MARKER_GLYPHS.get(str(m["kind"]), "?"))
		var col: Color = Color.ORANGE if state == "locked" else Color.WHITE
		_canvas.draw_string(font, Vector2(p.x * TILE + 6, p.y * TILE + 16), glyph, HORIZONTAL_ALIGNMENT_LEFT, -1, 14, col)
	var c: Vector2 = Vector2(gs.position.x * TILE + TILE / 2.0, gs.position.y * TILE + TILE / 2.0)
	_canvas.draw_circle(c, TILE * 0.35, Color(0.95, 0.85, 0.30))
	_canvas.draw_line(c, c + Vector2(facing) * TILE * 0.45, Color.BLACK, 2.0)
