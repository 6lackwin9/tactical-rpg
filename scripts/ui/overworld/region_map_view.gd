class_name RegionMapView
extends Control

# ==============================================================================
# Tactical RPG - Region map debug view (S9, spec EXP1-L commit 3)
# Thin: renders the GameState's location map (tiles, zone tint, markers, roamers, token) and forwards input to
# ExploreRunner / BattleBridge. Arrow keys = step, E = interact in the facing direction. A fight opens the real combat
# arena with the saved party (BattleScreen, BATTLE1-L); its outcome (won / fled / lost) settles through
# ExploreRunner.after_fight. If the battle cannot start, the fight panel offers Start battle again and the debug
# outcomes; `dev_outcomes` (off by default) always shows the debug outcome buttons.
# E on an npc opens its conversation (DialoguePanel over DialogueRunner); an open_shop choice swaps in the ShopPanel and
# its Close returns to the dialogue; a dialogue fight uses the same battle (DialogueRunner.after_fight).
# Moving and interacting are ignored while a conversation, shop or fight is open.
# POI-FIX (discoverability): a hint line under the bar names the keys (HINT); bumping a solid npc / chest / lore marker
# notes "press E"; E on an empty tile notes NOTHING_NOTE; a left click on a marker tile next to the party faces it and
# interacts (click_tile), a click on one further away notes FAR_NOTE.
# MAPVIEW1: the map canvas fills the view behind the top strip and the side column (semi-transparent floating panels);
# the map scrolls: a camera keeps the party centred in the free part of the view, clamped at the map edges, and a map
# smaller than the view is centred (camera_offset, pure). TILE px per tile times `zoom`; on a high-DPI screen the
# project's canvas_items stretch (DisplaySettings sizes the window in physical pixels) already scales the whole view.
# MAPREAD1-L2: every present / locked / spent marker draws its icon (MapArt overlay keys through AssetRegistry; an npc
# with an `art` key shows its unit token; spent = 40% opacity, a roamer pulses every second) and npc / exit names are
# always drawn above their tile; hovering a marker tile or facing one shows `label . verb` in the hint line; quest and
# ready badges sit on the npc icon; the side column lists the location's markers ("Here" legend: icon, label, state; hovering
# a row pulses its tile); walking onto a point logs its note (POINT_LOOK_NOTE without one, SPENT_NOTE once used up).
# `autosave` events write the autosave slot 0 (SaveGame.autosave, UI1-L) only for the main menu's game (CurrentGame.state()); the standalone debug game writes nothing
# (refused while a fight is pending). A world picker starts a new game. busy() is mirrored to CurrentGame.set_busy so saving is refused mid-fight / mid-talk.
# A world_map exit (marker or menu entry, TRAV1-L commit 3) opens the WorldMapView over the map; it hands back on
# arrival (or after a lost road fight) and on Back; its road fights count as busy() and re-emit `fight_settled`.
# ==============================================================================

signal fight_settled(res: Dictionary)   # UI2-L: the main menu host shows battle results on it

const TILE: int = 32   # MAPVIEW1: 1x pixel art, drawn with the nearest filter
const SIDE_WIDTH: float = 280.0
const NO_TILE: Vector2i = Vector2i(-1, -1)
const COL_FLOAT_BG: Color = Color(0.04, 0.05, 0.08, 0.72)   # the floating panels over the map
const COL_MAP_BG: Color = Color(0.07, 0.08, 0.10)   # around a map smaller than the view
# Region tile art (ui namespace, variant "icon", 32x32; MAPVIEW1 contract with Assets): template char -> key base;
# drawn when a manifest file serves it, else the COL_TILES colour.
const REGION_TILE_KEYS: Dictionary = {".": "region_ash_floor", "#": "region_basalt_wall", "~": "region_water"}
const REGION_TILE_VARIANTS: int = 3
const TILE_HASH_SALT: int = 11
const DEBUG_SEED: int = 7
const COL_TILES: Dictionary = {".": Color(0.22, 0.22, 0.24), "#": Color(0.45, 0.38, 0.30), "~": Color(0.18, 0.32, 0.55)}
const COL_DANGER: Color = Color(0.85, 0.20, 0.15, 0.28)
const COL_SAFE: Color = Color(0.20, 0.75, 0.30, 0.18)
const SPENT_ALPHA: float = 0.4   # a used-up marker keeps its icon at 40% opacity
const PULSE_PERIOD: float = 1.0   # seconds: the roamer icon and a legend-hovered tile
const BADGE_KEYS: Dictionary = {"quest": "badge_quest", "ready": "badge_ready"}   # ui icon keys (gold "!" / "?")
const ALWAYS_NAMED: Array[String] = ["npc", "exit"]   # kinds whose label is drawn above the tile at all times
const LEGEND_ORDER: Array[String] = ["npc", "exit", "chest", "lore", "point", "encounter", "roamer"]
const LEGEND_MAX: int = 12
const LEGEND_TITLE: String = "Here"
const STATE_WORDS: Dictionary = {"locked": "locked", "spent": "used up"}
const BADGE_WORDS: Dictionary = {"quest": "has a quest", "ready": "quest done"}
const POINT_LOOK_NOTE: String = "You look around."
const SPENT_NOTE: String = "Nothing more here."
const COL_LABEL: Color = Color(0.96, 0.97, 1.0)
const COL_LABEL_EDGE: Color = Color(0.02, 0.03, 0.06, 0.95)
const COL_PULSE: Color = Color(1.0, 0.92, 0.45)
const HINT: String = "Arrows: move   E: interact   Click: interact with a marker next to you"
const NOTHING_NOTE: String = "Nothing to interact with there."
const FAR_NOTE: String = "Walk next to it first (arrow keys), then press E or click it."
const BUMP_NOTES: Dictionary = {"npc": "Someone is in the way: press E to talk.", "chest": "A chest: press E to open it.", "lore": "Something to read: press E."}

var gs: GameState = null
var pending_fight: Dictionary = {}
var battle: BattleScreen = null
var results_popup: bool = false   # BATTLE-FLOW: the main menu host shows the results over the arena, so a battle settles without closing it
var dev_outcomes: bool = false
var launch_errors: Array[String] = []
var facing: Vector2i = Vector2i(0, 1)
var log_lines: Array[String] = []

var _world_pick: OptionButton = null
var _status: Label = null
var hint: Label = null
var _canvas: Control = null
var _menu_box: VBoxContainer = null
var _fight_panel: PanelContainer = null
var _fight_label: Label = null
var _dev_box: VBoxContainer = null
var _log: Label = null
var dialogue_panel: DialoguePanel = null
var shop_panel: ShopPanel = null
var world_map: WorldMapView = null   # open while the party is on the world map (TRAV1-L)
var zoom: float = 1.0   # extra map scale (1 = TILE px); high-DPI already scales the whole canvas (canvas_items stretch)
var _root: Control = null   # the canvas and the floating panels (hidden while the world map is open)
var _top: PanelContainer = null
var _side: PanelContainer = null
var _tile_cache: Dictionary = {}   # region tile key -> Texture2D or null (no file)
var hover_tile: Vector2i = NO_TILE   # the map tile under the pointer
var _legend_tile: Vector2i = NO_TILE   # the tile of the legend row under the pointer (pulses)
var _legend_box: VBoxContainer = null
var _clock: float = 0.0
var _animated: bool = false   # a roamer is drawn or a legend row is hovered: redraw every frame

func _ready() -> void:
	set_anchors_preset(Control.PRESET_FULL_RECT)
	_root = Control.new()
	_root.set_anchors_preset(Control.PRESET_FULL_RECT)
	_root.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_root)
	_canvas = map_canvas(_root)
	_canvas.draw.connect(_draw_map)
	_canvas.gui_input.connect(_on_canvas_input)
	_canvas.mouse_exited.connect(_on_canvas_exit)
	var top_box := VBoxContainer.new()
	_top = float_panel(_root, top_box, false)
	var bar := HBoxContainer.new()
	top_box.add_child(bar)
	_world_pick = OptionButton.new()
	for w: String in WorldCatalog.world_ids():
		_world_pick.add_item(w)
	bar.add_child(_world_pick)
	bar.add_child(UiTheme.button("New game", "NewGame.create with the first party units, then ExploreRunner.begin", _on_new_game))
	_status = UiTheme.label("Pick a world and start a new game.")
	bar.add_child(_status)
	hint = UiTheme.label(HINT, UiTheme.FONT_MD, UiTheme.COL_MUTED)
	top_box.add_child(hint)
	var side := VBoxContainer.new()
	_side = float_panel(_root, side, true)
	_menu_box = VBoxContainer.new()
	side.add_child(_menu_box)
	_legend_box = VBoxContainer.new()
	side.add_child(_legend_box)
	_fight_panel = PanelContainer.new()
	var fight_box := VBoxContainer.new()
	_fight_panel.add_child(fight_box)
	_fight_label = UiTheme.label("")
	fight_box.add_child(_fight_label)
	fight_box.add_child(UiTheme.button("Start battle", "Open the combat arena with the active party (BattleScreen)", launch_battle))
	_dev_box = VBoxContainer.new()
	fight_box.add_child(_dev_box)
	for result: String in BattleBridge.OUTCOMES:
		_dev_box.add_child(UiTheme.button("Dev: " + result, "Settle the fight as %s without a battle (ExploreRunner.after_fight)" % result, settle.bind(result)))
	_fight_panel.visible = false
	side.add_child(_fight_panel)
	dialogue_panel = DialoguePanel.new()
	dialogue_panel.shop_opened.connect(_on_shop_opened)
	dialogue_panel.fight_started.connect(_on_dialogue_fight)
	dialogue_panel.finished.connect(_refresh)
	dialogue_panel.noted.connect(_on_noted)
	side.add_child(dialogue_panel)
	shop_panel = ShopPanel.new()
	shop_panel.closed.connect(_on_shop_closed)
	shop_panel.noted.connect(_on_noted)
	side.add_child(shop_panel)
	_log = UiTheme.label("")
	_log.autowrap_mode = TextServer.AUTOWRAP_WORD
	_log.custom_minimum_size = Vector2(260, 0)
	side.add_child(_log)
	_top.resized.connect(func() -> void: dock_below(_top, _side))
	_refresh()

# ------------------------------------------------------------------------------
# Full-screen layout (MAPVIEW1): the map canvas fills the view; the top strip and the side column float over it
# ------------------------------------------------------------------------------

# A full-rect canvas under `parent` (first child, so it draws behind the panels); textures use the nearest filter (1x pixel art).
static func map_canvas(parent: Control) -> Control:
	var canvas := Control.new()
	canvas.set_anchors_preset(Control.PRESET_FULL_RECT)
	canvas.mouse_filter = Control.MOUSE_FILTER_STOP
	canvas.clip_contents = true
	canvas.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	parent.add_child(canvas)
	return canvas

# A semi-transparent dark panel over the map holding `content`: the top strip across the top (`side` false) or the side
# column anchored right (`side` true, SIDE_WIDTH wide, from under the top strip to the bottom; see dock_below).
static func float_panel(parent: Control, content: Control, side: bool) -> PanelContainer:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", UiTheme.box(COL_FLOAT_BG, Color(0, 0, 0, 0), 0, 8, 4, 0))
	panel.add_child(content)
	parent.add_child(panel)
	if side:
		panel.set_anchors_preset(Control.PRESET_RIGHT_WIDE)
		panel.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	else:
		panel.set_anchors_preset(Control.PRESET_TOP_WIDE)
	# explicit offsets: the preset keeps the old rect when the parent is not laid out yet
	panel.offset_left = -SIDE_WIDTH if side else 0.0
	panel.offset_right = 0.0
	panel.offset_top = 0.0
	panel.offset_bottom = 0.0
	return panel

# Starts the side column just under the top strip.
static func dock_below(top: Control, side: Control) -> void:
	if top != null and side != null:
		side.offset_top = top.position.y + top.size.y

# The part of `canvas_size` not covered by the floating panels (the camera keeps the party and the map edges in it).
static func free_rect(canvas_size: Vector2, top: Control, side: Control) -> Rect2:
	var top_h: float = top.size.y if top != null and top.visible else 0.0
	var side_w: float = side.size.x if side != null and side.visible else 0.0
	return Rect2(Vector2(0.0, top_h), Vector2(maxf(canvas_size.x - side_w, 0.0), maxf(canvas_size.y - top_h, 0.0)))

# Camera (pure): where the map's top-left corner goes inside a view of size `view` so that `focus_px` (a point in map
# pixels, the party) is centred, clamped so no space shows past a map edge; a map axis smaller than the view is centred.
static func camera_offset(map_px: Vector2, view: Vector2, focus_px: Vector2) -> Vector2:
	var out: Vector2 = Vector2.ZERO
	for axis: int in [Vector2.AXIS_X, Vector2.AXIS_Y]:
		if map_px[axis] <= view[axis]:
			out[axis] = (view[axis] - map_px[axis]) / 2.0
		else:
			out[axis] = clampf(view[axis] / 2.0 - focus_px[axis], view[axis] - map_px[axis], 0.0)
	return out

# The ui texture key/variant when a manifest file serves it, else null (callers keep their placeholder; no warning
# for a key that has no row).
static func ui_file_texture(key: String, variant: String) -> Texture2D:
	if not AssetRegistry.has_key("ui", key) or str(AssetRegistry.resolve("ui", key, variant).get("source", "")) != "file":
		return null
	return AssetRegistry.texture("ui", key, variant)

# The ui icon key of template tile `ch` at (x, y): REGION_TILE_KEYS base + "_" + a variant 0..REGION_TILE_VARIANTS-1
# picked by a stable hash of the coordinate ("" for a tile without art keys).
static func region_tile_key(ch: String, x: int, y: int) -> String:
	var base: String = str(REGION_TILE_KEYS.get(ch, ""))
	if base.is_empty():
		return ""
	return "%s_%d" % [base, int(SvgRaster.hash01(x, y, TILE_HASH_SALT) * REGION_TILE_VARIANTS) % REGION_TILE_VARIANTS]

# The region tile texture of `key` (cached per view; null = draw the placeholder colour).
func _tile_texture(key: String) -> Texture2D:
	if key.is_empty():
		return null
	if not _tile_cache.has(key):
		_tile_cache[key] = ui_file_texture(key, "icon")
	return _tile_cache[key] as Texture2D

# Pixels per tile on screen: TILE times `zoom` (high-DPI scaling is the project's canvas_items stretch, see the header).
func tile_px() -> float:
	return TILE * zoom

# Canvas position of the map's tile (0, 0) for the current game (Vector2.ZERO without a map).
func map_origin() -> Vector2:
	if gs == null or _canvas == null:
		return Vector2.ZERO
	var t: Dictionary = ExploreRunner.template_of(ExploreRunner.location(gs))
	if t.is_empty():
		return Vector2.ZERO
	var px: float = tile_px()
	var view: Rect2 = free_rect(_canvas.size, _top, _side)
	var focus: Vector2 = (Vector2(gs.position) + Vector2(0.5, 0.5)) * px
	return view.position + camera_offset(Vector2(MapRules.size_of(t)) * px, view.size, focus)

# The map tile under canvas point `p` (camera offset and zoom applied).
func tile_at_point(p: Vector2) -> Vector2i:
	var local: Vector2 = (p - map_origin()) / tile_px()
	return Vector2i(floori(local.x), floori(local.y))

# The canvas point at the centre of map tile `tile` (tests click there).
func tile_centre(tile: Vector2i) -> Vector2:
	return map_origin() + (Vector2(tile) + Vector2(0.5, 0.5)) * tile_px()

# ------------------------------------------------------------------------------
# Marker read-out (MAPREAD1-L2): icons, names, hint line, legend
# ------------------------------------------------------------------------------

# The tile asset key of a marker view (ExploreRunner.marker_view): a used-up chest shows the open one, a locked exit the padlock.
static func icon_key(v: Dictionary) -> String:
	if str(v.get("kind", "")) == "chest" and str(v.get("state", "")) == "spent":
		return "overlay_chest_open"
	if str(v.get("kind", "")) == "exit" and str(v.get("state", "")) == "locked":
		return "overlay_exit_locked"
	return str(v.get("icon", "overlay_poi"))

# Opacity of a marker icon: SPENT_ALPHA once used up; a roamer pulses with `clock` seconds.
static func icon_alpha(v: Dictionary, clock: float) -> float:
	if str(v.get("state", "")) == "spent":
		return SPENT_ALPHA
	return pulse_alpha(clock) if str(v.get("kind", "")) == "roamer" else 1.0

# 0.55..1.0, one full beat per PULSE_PERIOD.
static func pulse_alpha(clock: float) -> float:
	return 0.775 + 0.225 * cos(TAU * clock / PULSE_PERIOD)

# "label . verb" (the hint line while a marker is hovered or faced).
static func hint_line(v: Dictionary) -> String:
	return "%s · %s" % [str(v.get("label", "")), str(v.get("verb", ""))]

# Where the baseline-left corner of a name of `text_size` goes so it stands centred above a tile at `tile_pos` (`tile_px`
# wide) and stays inside `bounds`; below the tile when there is no room above.
static func label_origin(tile_pos: Vector2, tile_size: float, text_size: Vector2, ascent: float, bounds: Rect2) -> Vector2:
	var x: float = tile_pos.x + (tile_size - text_size.x) / 2.0
	x = maxf(minf(x, bounds.end.x - text_size.x - 2.0), bounds.position.x + 2.0)
	var y: float = tile_pos.y - 3.0
	if y - ascent < bounds.position.y:
		y = tile_pos.y + tile_size + ascent + 1.0
	return Vector2(x, y)

# Legend sort key: kind order, then label.
static func legend_less(a: Dictionary, b: Dictionary) -> bool:
	var ra: int = LEGEND_ORDER.find(str(a["kind"]))
	var rb: int = LEGEND_ORDER.find(str(b["kind"]))
	if ra != rb:
		return ra < rb
	return str(a["label"]) < str(b["label"])

# "Mara Voss: has a quest" / "Chest: used up" / "To Keep" (the legend row text).
static func legend_text(v: Dictionary) -> String:
	var extras: Array[String] = []
	if BADGE_WORDS.has(str(v.get("badge", ""))):
		extras.append(str(BADGE_WORDS[str(v["badge"])]))
	if STATE_WORDS.has(str(v.get("state", ""))):
		extras.append(str(STATE_WORDS[str(v["state"])]))
	return str(v.get("label", "")) if extras.is_empty() else "%s (%s)" % [str(v["label"]), ", ".join(PackedStringArray(extras))]

# Font size of names and legend rows: follows the theme's text size (the Text size setting).
func label_font_size() -> int:
	return name_font_size(get_theme_default_font_size())

# Map name size for a theme text size (also the world map).
static func name_font_size(theme_size: int) -> int:
	return clampi(roundi(theme_size * 0.75), 10, 30)

# Every visible (present, locked or spent) marker of the current location: [{id, pos (tile), view (marker_view)}].
func visible_markers() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if gs == null:
		return out
	var loc: Dictionary = ExploreRunner.location(gs)
	var t: Dictionary = ExploreRunner.template_of(loc)
	if t.is_empty():
		return out
	var loc_steps: int = DictUtil.to_int(gs.explore.get("loc_steps", 0))
	for m: Dictionary in ExploreRunner.markers_of(loc):
		var v: Dictionary = ExploreRunner.marker_view(gs, loc, m)
		if str(v["state"]) == "absent":
			continue
		out.append({"id": str(m.get("id", "")), "pos": ExploreRunner.marker_pos(t, m, loc_steps), "view": v, "marker": m})
	return out

# The visible marker standing on `tile` ({} = none; the first one when several share it).
func marker_at(tile: Vector2i) -> Dictionary:
	for row: Dictionary in visible_markers():
		if row["pos"] == tile:
			return row
	return {}

# Legend rows (npcs first, then exits, chests, lore, points, fights): [{kind, label, state, badge, icon, pos, text}].
func legend_rows() -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	for row: Dictionary in visible_markers():
		var v: Dictionary = DictUtil.dict_of(row["view"])
		rows.append({"kind": v["kind"], "label": v["label"], "state": v["state"], "badge": v["badge"], "icon": icon_key(v), "pos": row["pos"], "text": legend_text(v), "marker": row["marker"]})
	rows.sort_custom(legend_less)
	return rows

# The hint line under the top bar: the hovered marker's `label . verb`, else the faced neighbour's, else HINT.
func hint_text() -> String:
	var row: Dictionary = marker_at(hover_tile) if hover_tile != NO_TILE else {}
	if row.is_empty() and gs != null:
		row = marker_at(gs.position + facing)
	return hint_line(DictUtil.dict_of(row["view"])) if not row.is_empty() else HINT

# The texture drawn for a marker row: an npc with a known `art` key shows its unit token, else the kind icon.
func marker_texture(row: Dictionary) -> Texture2D:
	var v: Dictionary = DictUtil.dict_of(row["view"])
	if str(v["kind"]) == "npc" and gs != null:
		var who: Dictionary = WorldCatalog.resolve(gs.world_id, "characters", str(DictUtil.dict_of(row["marker"]).get("character", "")))
		var art: String = str(who.get("art", ""))
		if not art.is_empty() and AssetRegistry.has_key("unit", art):
			return UnitArt.token(art, UnitArt.Team.NEUTRAL)
	return AssetRegistry.texture("tile", icon_key(v), "v0")

func _on_canvas_exit() -> void:
	_set_hover(NO_TILE)

func _set_hover(tile: Vector2i) -> void:
	if tile == hover_tile:
		return
	hover_tile = tile
	if hint != null:
		hint.text = hint_text()

func _on_legend_enter(tile: Vector2i) -> void:
	_legend_tile = tile
	_animated = true
	_canvas.queue_redraw()

func _on_legend_exit() -> void:
	_legend_tile = NO_TILE
	_canvas.queue_redraw()

# Rebuilds the "Here" legend (hidden while a conversation, shop or fight is open).
func _rebuild_legend() -> void:
	_legend_tile = NO_TILE
	for c: Node in _legend_box.get_children():
		_legend_box.remove_child(c)
		c.queue_free()
	var rows: Array[Dictionary] = []
	if not _input_blocked():
		rows.assign(legend_rows())
	_legend_box.visible = not rows.is_empty()
	if rows.is_empty():
		return
	var fs: int = label_font_size()
	var title: Label = UiTheme.label(LEGEND_TITLE, fs, UiTheme.COL_HEADER)
	_legend_box.add_child(title)
	for i: int in range(mini(rows.size(), LEGEND_MAX)):
		var r: Dictionary = rows[i]
		var line := HBoxContainer.new()
		line.mouse_filter = Control.MOUSE_FILTER_STOP
		var icon := TextureRect.new()
		icon.texture = AssetRegistry.texture("tile", str(r["icon"]), "v0")
		icon.custom_minimum_size = Vector2(fs + 8, fs + 8)
		icon.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		icon.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_CENTERED
		icon.modulate.a = SPENT_ALPHA if str(r["state"]) == "spent" else 1.0
		icon.mouse_filter = Control.MOUSE_FILTER_IGNORE
		line.add_child(icon)
		var text: Label = UiTheme.label(str(r["text"]), fs, UiTheme.COL_TEXT if str(r["state"]) != "spent" else UiTheme.COL_MUTED)
		text.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		text.custom_minimum_size = Vector2(120, 0)
		text.mouse_filter = Control.MOUSE_FILTER_IGNORE
		line.add_child(text)
		line.mouse_entered.connect(_on_legend_enter.bind(r["pos"]))
		line.mouse_exited.connect(_on_legend_exit)
		_legend_box.add_child(line)
	if rows.size() > LEGEND_MAX:
		_legend_box.add_child(UiTheme.label("+%d more" % (rows.size() - LEGEND_MAX), fs, UiTheme.COL_MUTED))

func _process(delta: float) -> void:
	_clock += delta
	if _animated and _canvas != null and _canvas.visible:
		_canvas.queue_redraw()

func _on_new_game() -> void:
	if _world_pick != null and _world_pick.item_count > 0:
		start_game(_world_pick.get_item_text(_world_pick.selected))

# NewGame.create in `world_id` with the first two party units of the catalog: {state, errors} (also WorldMapView).
static func new_state(world_id: String, p_seed: int = DEBUG_SEED) -> Dictionary:
	var units: Array[String] = []
	for e: Dictionary in ContentCatalog.entries("units"):
		if str(e.get("side", "")) == "party" and ContentFactory.class_of(e) != null and units.size() < 2:
			units.append(str(e["id"]))
	return NewGame.create(world_id, units, p_seed)

# A new game in `world_id` with the first party units of the catalog, entered at the world's start location.
func start_game(world_id: String, p_seed: int = DEBUG_SEED) -> Array[String]:
	var res: Dictionary = new_state(world_id, p_seed)
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
	_close_battle()
	_close_panels()
	close_world_map()
	_handle(ExploreRunner.begin(gs))
	return errors

# Plays an existing game (the main menu's new or loaded game, UI1-L): resumes at its location, or enters the start
# location when it has none yet.
func play_state(state: GameState) -> void:
	gs = state
	pending_fight = {}
	_close_battle()
	_close_panels()
	close_world_map()
	_handle(ExploreRunner.begin(gs) if gs.location_id.is_empty() else ExploreRunner.resume(gs))

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

# True while a fight (map, dialogue or road), conversation or shop waits for the player (saving is refused).
func busy() -> bool:
	return not pending_fight.is_empty() or (dialogue_panel != null and dialogue_panel.is_open()) or (shop_panel != null and shop_panel.is_open()) or (world_map != null and world_map.busy())

# Map input is ignored while busy or while the world map is open.
func _input_blocked() -> bool:
	return gs == null or busy() or world_map != null

func do_step(dir: Vector2i) -> void:
	if _input_blocked():
		return
	facing = dir
	var target: Vector2i = gs.position + dir
	var from_loc: String = gs.location_id
	var onto: Dictionary = marker_at(target)
	_handle(ExploreRunner.step(gs, dir))
	if not onto.is_empty() and str(DictUtil.dict_of(onto["view"])["kind"]) == "point" and gs.location_id == from_loc and gs.position == target:
		_note(point_note(DictUtil.dict_of(onto["view"]), DictUtil.dict_of(onto["marker"])))
		_refresh()

# What walking onto a point logs: its `note` (POINT_LOOK_NOTE without one), SPENT_NOTE once it is used up.
static func point_note(v: Dictionary, marker: Dictionary) -> String:
	if str(v.get("state", "")) == "spent":
		return SPENT_NOTE
	var note: String = str(marker.get("note", "")).strip_edges()
	return note if not note.is_empty() else POINT_LOOK_NOTE

func do_interact() -> void:
	if _input_blocked():
		return
	var res: Dictionary = ExploreRunner.interact(gs, facing)
	match str(res.get("kind", "")):
		"npc":
			talk(str(res["character"]))
		"lore":
			_note("%s: %s" % [str(res["name"]), str(res["text"])])
		"chest":
			if DictUtil.to_bool(res.get("empty", false)):
				_note("The chest is empty.")
			else:
				var got: int = DictUtil.to_int(res.get("supplies", 0))
				_note("Opened: %s %s%s" % [str(res.get("items", {})), str(res.get("loot", "")), " +%d supplies" % got if got > 0 else ""])
				for n: Variant in DictUtil.array_of(res.get("notes", [])):
					_note(str(n))   # TRAV2-L: supplies cap note
		_:
			if not DictUtil.to_bool(res.get("ok", true)):
				_note("Interact refused: %s" % str(res["errors"]))
			else:
				_note(NOTHING_NOTE)
	_refresh()

# A left click on the map: the tile under the pointer (click_tile).
func _on_canvas_input(event: InputEvent) -> void:
	var motion: InputEventMouseMotion = event as InputEventMouseMotion
	if motion != null:
		_set_hover(tile_at_point(motion.position))
		return
	var mb: InputEventMouseButton = event as InputEventMouseButton
	if mb == null or not mb.pressed or mb.button_index != MOUSE_BUTTON_LEFT:
		return
	if click_tile(tile_at_point(mb.position)):
		_canvas.accept_event()

# A click on `tile`: next to the party it faces the tile and interacts (true); a present npc / chest / lore marker
# further away gets FAR_NOTE (true); anything else is ignored (false).
func click_tile(tile: Vector2i) -> bool:
	if _input_blocked():
		return false
	var dir: Vector2i = tile - gs.position
	if ExploreRunner.DIRS.has(dir):
		facing = dir
		do_interact()
		return true
	if MapRules.SOLID_KINDS.has(_marker_kind_at(tile)):
		_note(FAR_NOTE)
		_refresh()
		return true
	return false

# The kind of the present marker drawn on `tile` ("" = none).
func _marker_kind_at(tile: Vector2i) -> String:
	if gs == null:
		return ""
	var loc: Dictionary = ExploreRunner.location(gs)
	var t: Dictionary = ExploreRunner.template_of(loc)
	if t.is_empty():
		return ""
	var loc_steps: int = DictUtil.to_int(gs.explore.get("loc_steps", 0))
	for m: Dictionary in ExploreRunner.markers_of(loc):
		if ExploreRunner.presence(gs, str(loc["_qid"]), m) == "present" and ExploreRunner.marker_pos(t, m, loc_steps) == tile:
			return str(m["kind"])
	return ""

# Opens the conversation with world character `character_ref` (DialogueRunner.open) in the dialogue panel.
func talk(character_ref: String) -> void:
	if gs == null or busy():
		return
	var r: DialogueRunner = DialogueRunner.open(gs, character_ref)
	if not r.errors.is_empty():
		_note("Talk refused: %s" % "; ".join(PackedStringArray(r.errors)))
	else:
		_note("You speak with %s." % r.character_name)
		dialogue_panel.show_runner(r)
	_refresh()

func _on_shop_opened(shop_qid: String) -> void:
	shop_panel.show_shop(gs, shop_qid)
	_refresh()

func _on_shop_closed() -> void:
	dialogue_panel.shop_closed()
	_refresh()

func _on_dialogue_fight(fight: Dictionary) -> void:
	pending_fight = fight
	_sync_busy()
	_note("Fight! %s (dialogue with %s)" % [str(fight.get("encounter", "")), str(fight.get("character", ""))])
	launch_battle.call_deferred()

func _on_noted(text: String) -> void:
	_note(text)
	_refresh()

func _close_panels() -> void:
	if dialogue_panel != null:
		dialogue_panel.visible = false
		dialogue_panel.runner = null
	if shop_panel != null:
		shop_panel.visible = false
		shop_panel.gs = null

func take_exit(to: String) -> void:
	if not _input_blocked():
		_handle(ExploreRunner.take_exit(gs, to))

# Opens the world map over the region map (TRAV1-L): a world_map exit marker or menu entry.
func open_world_map() -> void:
	if gs == null or world_map != null:
		return
	world_map = WorldMapView.new()
	world_map.embedded = true
	world_map.results_popup = results_popup
	add_child(world_map)
	world_map.show_state(gs)
	world_map.arrived.connect(_on_world_map_arrived)
	world_map.closed.connect(close_world_map)
	world_map.fight_settled.connect(_on_world_map_fight)
	_note("The road leads to the world map.")
	_refresh()

func close_world_map() -> void:
	if world_map == null:
		return
	var view: WorldMapView = world_map
	world_map = null
	remove_child(view)
	view.queue_free()
	_refresh()

func _on_world_map_arrived(location_qid: String) -> void:
	_note("Arrived at %s." % location_qid)
	close_world_map()

func _on_world_map_fight(res: Dictionary) -> void:
	fight_settled.emit(res)

# Opens the pending fight in the combat arena (BattleScreen); the map hides until the battle is settled. A refused
# start keeps the fight panel with its reasons, Start battle and the debug outcomes.
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

# The arena that is still mounted after its fight settled (results popup mode), in this view or the world map; else null.
func active_battle() -> BattleScreen:
	if battle != null:
		return battle
	return world_map.battle if world_map != null else null

# Closes the arena kept for the results popup and shows the map again.
func close_settled_battle() -> void:
	_close_battle()
	if world_map != null:
		world_map.close_settled_battle()
	_refresh()

func _close_battle() -> void:
	if battle != null:
		battle.close()
		battle = null
	launch_errors.clear()

# Settles the pending fight with `result` (the battle's outcome, or a debug outcome button) and closes the battle.
# A dialogue fight settles through DialogueRunner.after_fight, an exploration fight through ExploreRunner.after_fight.
# `fight_settled` carries the result plus {outcome}.
func settle(result: String, keep_battle: bool = false) -> void:
	if gs == null or pending_fight.is_empty():
		return
	var fight: Dictionary = pending_fight
	pending_fight = {}
	if not keep_battle:
		_close_battle()
	var res: Dictionary = DialogueRunner.after_fight(gs, fight, result) if str(fight.get("source", "")) == "dialogue" else ExploreRunner.after_fight(gs, fight, result)
	_note("Fight %s: %s (quests advanced: %s)" % [str(fight["encounter"]), result, str(res.get("advanced", []))])
	_handle(res)
	if DictUtil.to_bool(res.get("ok", false)):
		res["outcome"] = result
		fight_settled.emit(res)

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
		"blocked":
			var at: Array = DictUtil.array_of(e.get("at", []))
			var kind: String = _marker_kind_at(Vector2i(DictUtil.to_int(at[0]), DictUtil.to_int(at[1]))) if e.has("marker") and at.size() == 2 else ""
			var bump: String = str(BUMP_NOTES.get(kind, ""))
			if not bump.is_empty() and (log_lines.is_empty() or log_lines[-1] != bump):
				_note(bump)
		"flags_set":
			if _is_point(str(e.get("marker", ""))):
				return   # MAPREAD1-L2: a point logs its note instead of the raw flag list
			_note("Flags set: %s" % str(e["flags"]))
		"exit":
			if str(e["to"]) == MapRules.WORLD_MAP:
				open_world_map()
			for inner: Variant in DictUtil.array_of(e.get("events", [])):
				_event(DictUtil.dict_of(inner))
		"fight":
			pending_fight = e
			_sync_busy()
			_note("Fight! %s (%s%s)" % [str(e["encounter"]), str(e["source"]), ", elite" if DictUtil.to_bool(e["elite"]) else ""])
			launch_battle.call_deferred()

# True when `marker_id` is a point marker of the current location.
func _is_point(marker_id: String) -> bool:
	if gs == null or marker_id.is_empty():
		return false
	for m: Dictionary in ExploreRunner.markers_of(ExploreRunner.location(gs)):
		if str(m.get("id", "")) == marker_id:
			return str(m.get("kind", "")) == "point"
	return false

func _autosave() -> void:
	if gs == null or not pending_fight.is_empty():
		return
	if gs != CurrentGame.state():
		_note("Autosave skipped (debug game).")
		return
	var errors: Array[String] = SaveGame.autosave(gs)
	if errors.is_empty():
		CurrentGame.mark_saved(SaveGame.AUTOSAVE_SLOT)
	if not errors.is_empty():
		_note("Autosave refused: %s" % "; ".join(PackedStringArray(errors)))

func _note(text: String) -> void:
	log_lines.append(text)
	while log_lines.size() > 12:
		log_lines.pop_front()

# Mirrors busy() into CurrentGame for the main menu's game, so SaveSlotsModel refuses saves mid-fight / mid-talk.
func _sync_busy() -> void:
	if gs != null and gs == CurrentGame.state():
		CurrentGame.set_busy(busy())

func _exit_tree() -> void:
	# a view freed mid-fight / mid-talk must not leave the save lock stuck
	if gs != null and gs == CurrentGame.state():
		CurrentGame.set_busy(false)

func _refresh() -> void:
	_sync_busy()
	if world_map != null:
		MusicDirector.release(self)   # MUSIC1: the embedded world map's travel request plays
	else:
		var shop_open: bool = shop_panel != null and shop_panel.is_open()
		MusicDirector.request_game(self, MusicDirector.region_context(gs, shop_open), gs)   # unchanged requests are no-ops
	if _status == null:
		return
	if gs == null:
		_status.text = "Pick a world and start a new game."
	else:
		_status.text = "%s  %s  steps %d / %d  rolls %s" % [gs.location_id, str(gs.position), DictUtil.to_int(gs.explore["steps"]), DictUtil.to_int(gs.explore["next_at"]), str(gs.rolls)]
	for c: Node in _menu_box.get_children():
		c.queue_free()
	if not _input_blocked():
		for item: Dictionary in ExploreRunner.menu(gs):
			_menu_box.add_child(UiTheme.button(str(item["label"]), "ExploreRunner.take_exit", take_exit.bind(str(item["to"]))))
	_fight_panel.visible = not pending_fight.is_empty() and battle == null
	_fight_label.text = "Fight: %s" % str(pending_fight.get("encounter", ""))
	if not launch_errors.is_empty():
		_fight_label.text += "\nBattle refused: %s" % "; ".join(PackedStringArray(launch_errors))
	_dev_box.visible = dev_outcomes or not launch_errors.is_empty()
	_canvas.visible = battle == null
	_rebuild_legend()
	if hint != null:
		hint.text = hint_text()
	if _root != null:
		_root.visible = world_map == null
	_log.text = "\n".join(PackedStringArray(log_lines))
	_canvas.queue_redraw()

func _draw_map() -> void:
	_canvas.draw_rect(Rect2(Vector2.ZERO, _canvas.size), COL_MAP_BG)
	if gs == null:
		return
	var loc: Dictionary = ExploreRunner.location(gs)
	var t: Dictionary = ExploreRunner.template_of(loc)
	if t.is_empty():
		var at: Vector2 = free_rect(_canvas.size, _top, _side).position + Vector2(8, 24)
		_canvas.draw_string(get_theme_default_font(), at, "%s (no map: use the exits on the right)" % str(loc.get("name", gs.location_id)))
		return
	_canvas.draw_set_transform(map_origin(), 0.0, Vector2(zoom, zoom))   # map pixels from here on (camera + zoom)
	var size: Vector2i = MapRules.size_of(t)
	for y: int in range(size.y):
		for x: int in range(size.x):
			var ch: String = MapRules.tile(t, x, y)
			var tex: Texture2D = _tile_texture(region_tile_key(ch, x, y))
			var r := Rect2(Vector2(x * TILE, y * TILE), Vector2(TILE - 1, TILE - 1))
			if tex != null:
				r.size = Vector2(TILE, TILE)   # art tiles butt together; placeholder colours keep the 1 px grid
				_canvas.draw_texture_rect(tex, r, false)
			else:
				_canvas.draw_rect(r, COL_TILES.get(ch, Color.BLACK))
			var zone: String = MapRules.zone_at(t, x, y)
			if not zone.is_empty():
				_canvas.draw_rect(r, COL_DANGER if MapRules.zone_kind(t, zone) == "danger" else COL_SAFE)
	var rows: Array[Dictionary] = visible_markers()
	var animated: bool = _legend_tile != NO_TILE
	for row: Dictionary in rows:
		var v: Dictionary = DictUtil.dict_of(row["view"])
		var p: Vector2i = row["pos"]
		var at := Rect2(Vector2(p.x * TILE, p.y * TILE), Vector2(TILE, TILE))
		var tex: Texture2D = marker_texture(row)
		_canvas.draw_texture_rect(tex, at, false, Color(1, 1, 1, icon_alpha(v, _clock)))
		animated = animated or str(v["kind"]) == "roamer"
		var badge_key: String = str(BADGE_KEYS.get(str(v["badge"]), ""))
		if not badge_key.is_empty():
			_canvas.draw_texture_rect(AssetRegistry.texture("ui", badge_key, "icon"), Rect2(at.position + Vector2(TILE * 0.5, -TILE * 0.2), Vector2(TILE, TILE) * 0.6), false)
	if _legend_tile != NO_TILE:
		var ring := Rect2(Vector2(_legend_tile.x * TILE, _legend_tile.y * TILE), Vector2(TILE, TILE)).grow(2.0)
		_canvas.draw_rect(ring, Color(COL_PULSE, pulse_alpha(_clock)), false, 3.0)
	_animated = animated
	var c: Vector2 = Vector2(gs.position.x * TILE + TILE / 2.0, gs.position.y * TILE + TILE / 2.0)
	_canvas.draw_circle(c, TILE * 0.35, Color(0.95, 0.85, 0.30))
	_canvas.draw_line(c, c + Vector2(facing) * TILE * 0.45, Color.BLACK, 2.0)
	_canvas.draw_set_transform(Vector2.ZERO)
	_draw_names(rows)

# Names of the npc and exit markers above their tiles (screen scale, clear of the floating panels and the map view edges).
func _draw_names(rows: Array[Dictionary]) -> void:
	var font: Font = get_theme_default_font()
	var fs: int = label_font_size()
	var bounds: Rect2 = free_rect(_canvas.size, _top, _side)
	var origin: Vector2 = map_origin()
	var px: float = tile_px()
	for row: Dictionary in rows:
		var v: Dictionary = DictUtil.dict_of(row["view"])
		if not ALWAYS_NAMED.has(str(v["kind"])):
			continue
		var text: String = str(v["label"])
		var size: Vector2 = font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs)
		var tile: Vector2i = row["pos"]
		var at: Vector2 = label_origin(origin + Vector2(tile) * px, px, size, font.get_ascent(fs), bounds)
		_canvas.draw_string_outline(font, at, text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, 4, COL_LABEL_EDGE)
		_canvas.draw_string(font, at, text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs, COL_LABEL)
