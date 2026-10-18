class_name MusicDirector
extends Node

# ==============================================================================
# Tactical RPG - Music director (MUSIC1-L, spec docs/specs/MUSIC1.md section 2)
# Scenes never play music themselves: they post requests (owner, context, priority,
# world, location) and the director plays the one wanted track on the Music bus.
# A context resolves to a manifest key through a data-only fallback chain (rule 7),
# so new tracks need a manifest row, no code. The choice is a pure static function
# over the requests (rule 2), so tests run headless. One lazily created node under
# the tree root (survives scene changes) owns two players and crossfades between
# them; headless runs create no players but still track the wanted key.
# ==============================================================================

const MENU: int = 0
const FIELD: int = 1
const COMBAT: int = 2

const CTX_MENU: String = "menu"
const CTX_FIELD: String = "field"
const CTX_TRAVEL: String = "travel"
const CTX_TOWN: String = "town"
const CTX_SHOP: String = "shop"
const CTX_ENCOUNTER: String = "encounter"
const CTX_DANGER: String = "danger"
const CTX_BOSS: String = "boss"
const CTX_COMBAT: String = "combat"
# Rule 7: context -> priority, and the context tried next when none of a context's keys resolve.
const CONTEXTS: Dictionary = {
	CTX_MENU: MENU, CTX_FIELD: FIELD, CTX_TRAVEL: FIELD, CTX_TOWN: FIELD, CTX_SHOP: FIELD, CTX_DANGER: FIELD,
	CTX_COMBAT: COMBAT, CTX_ENCOUNTER: COMBAT, CTX_BOSS: COMBAT,
}
const PARENT_CONTEXT: Dictionary = {
	CTX_TRAVEL: CTX_FIELD, CTX_TOWN: CTX_FIELD, CTX_SHOP: CTX_TOWN, CTX_DANGER: CTX_FIELD,
	CTX_ENCOUNTER: CTX_COMBAT, CTX_BOSS: CTX_COMBAT,
}
const MENU_KEY: String = "menu_theme"   # the menu context's last step

const MUSIC_DB: float = -6.0
const CROSSFADE_SEC: float = 1.0
const FIRST_FADE_SEC: float = 2.0
const BUS: String = "Music"
const NS: String = "music"

# {"id": owner instance id, "context", "priority", "world", "location", "seq"}, one per owner (rule 1).
static var _requests: Array[Dictionary] = []
static var _seq: int = 0
static var _node: MusicDirector = null
static var _track_ok: Dictionary = {}   # key -> bool (has_track, cached for the per-frame check)
static var _key_cache: Dictionary = {}  # "context|world|location" -> resolved key

var _players: Array[AudioStreamPlayer] = []
var _tweens: Array[Tween] = [null, null]
var _active: int = 0
var _current: String = ""   # the key playing (headless: the key that would play)

# ------------------------------------------------------------------------------
# Requests (rule 1)
# ------------------------------------------------------------------------------

# Posts or replaces `owner`'s request. The newest request wins ties; an unchanged repeat keeps its place, so views
# may call this on every refresh.
static func request(owner: Node, context: String, priority: int, world_id: String = "", location_id: String = "") -> void:
	if owner == null:
		return
	var id: int = owner.get_instance_id()
	var want: Dictionary = {"id": id, "context": context, "priority": priority, "world": world_id, "location": location_id}
	_prune()
	for r: Dictionary in _requests:
		if DictUtil.to_int(r["id"]) != id:
			continue
		for k: String in want:
			if r[k] != want[k]:
				_seq += 1
				r.merge(want, true)
				r["seq"] = _seq
				break
		_ensure_node()
		return
	_seq += 1
	want["seq"] = _seq
	_requests.append(want)
	_ensure_node()

static func release(owner: Node) -> void:
	if owner == null:
		return
	var id: int = owner.get_instance_id()
	for i: int in range(_requests.size() - 1, -1, -1):
		if DictUtil.to_int(_requests[i]["id"]) == id:
			_requests.remove_at(i)

# Requests `context` at its table priority for `gs`'s world and location; releases when there is no game.
static func request_game(owner: Node, context: String, gs: GameState) -> void:
	if gs == null:
		release(owner)
		return
	request(owner, context, priority_of(context), gs.world_id, gs.location_id)

static func priority_of(context: String) -> int:
	return DictUtil.to_int(CONTEXTS.get(context, MENU))

# A copy of the live requests (tests, debugging).
static func requests() -> Array[Dictionary]:
	_prune()
	return _requests.duplicate(true)

# Test hook: drops every request and the caches.
static func clear_requests() -> void:
	_requests.clear()
	clear_caches()

# Forgets cached resolutions (after manifest or world edits).
static func clear_caches() -> void:
	_track_ok.clear()
	_key_cache.clear()

static func _prune() -> void:
	for i: int in range(_requests.size() - 1, -1, -1):
		if not is_instance_valid(instance_from_id(DictUtil.to_int(_requests[i]["id"]))):
			_requests.remove_at(i)

# ------------------------------------------------------------------------------
# Choice (rules 2, 3 and 7)
# ------------------------------------------------------------------------------

# True when `key` resolves to a music file in the manifest (uncached).
static func has_track(key: String) -> bool:
	if key.is_empty():
		return false
	return str(AssetRegistry.resolve(NS, key, "stream")["source"]) == "file"

static func _has_track_cached(key: String) -> bool:
	if not _track_ok.has(key):
		_track_ok[key] = has_track(key)
	return DictUtil.to_bool(_track_ok[key])

# An owner counts while it is a valid Node inside the tree and, for a CanvasItem, visible in the tree.
static func owner_live(owner_id: int) -> bool:
	var o: Object = instance_from_id(owner_id)
	if not is_instance_valid(o) or not (o is Node):
		return false
	var n: Node = o as Node
	if not n.is_inside_tree():
		return false
	if n is CanvasItem:
		return (n as CanvasItem).is_visible_in_tree()
	return true

# The wanted key of `reqs` ("" = silence): the highest priority live request whose context resolves to a key, ties
# to the newest. `track(context, world, location) -> String` defaults to track_for.
static func choose(reqs: Array[Dictionary], track: Callable = Callable()) -> String:
	var key_of: Callable = track if track.is_valid() else track_for
	var best_key: String = ""
	var best_p: int = -1
	var best_seq: int = -1
	for r: Dictionary in reqs:
		if not owner_live(DictUtil.to_int(r.get("id", 0))):
			continue
		var key: String = str(key_of.call(str(r.get("context", "")), str(r.get("world", "")), str(r.get("location", ""))))
		if key.is_empty():
			continue
		var p: int = DictUtil.to_int(r.get("priority", 0))
		var s: int = DictUtil.to_int(r.get("seq", 0))
		if p > best_p or (p == best_p and s > best_seq):
			best_key = key
			best_p = p
			best_seq = s
	return best_key

# The track the director wants now (tests read this headless).
static func wanted_key() -> String:
	_prune()
	return choose(_requests, _cached_track_for)

# The key the player node is on ("" before the node exists or in silence).
static func playing_key() -> String:
	return _node._current if is_instance_valid(_node) else ""

# The first resolving key of the context chain ("" when none resolves).
static func track_for(context: String, world_id: String, location_id: String, resolves: Callable = Callable()) -> String:
	var check: Callable = resolves if resolves.is_valid() else has_track
	for key: String in candidates(context, world_id, location_id):
		if DictUtil.to_bool(check.call(key)):
			return key
	return ""

# wanted_key's resolver: track_for memoised per context + world + location (cleared by clear_caches).
static func _cached_track_for(context: String, world_id: String, location_id: String) -> String:
	var ck: String = "%s|%s|%s" % [context, world_id, location_id]
	if not _key_cache.has(ck):
		_key_cache[ck] = track_for(context, world_id, location_id, _has_track_cached)
	return str(_key_cache[ck])

# Rule 7: the keys a context tries, in order, most specific first. Per context step: "<scope>_<context>" for every
# scope, (field only) the plain scope keys, "<context>" alone, (menu only) menu_theme; then the parent context.
static func candidates(context: String, world_id: String, location_id: String) -> Array[String]:
	var out: Array[String] = []
	var sc: Array[String] = scopes(world_id, location_id)
	var ctx: String = context
	var seen: Array[String] = []
	while not ctx.is_empty() and not seen.has(ctx):
		seen.append(ctx)
		for s: String in sc:
			_add(out, "%s_%s" % [s, ctx])
		if ctx == CTX_FIELD:
			for s: String in sc:
				_add(out, s)
		_add(out, ctx)
		if ctx == CTX_MENU:
			_add(out, MENU_KEY)
		ctx = str(PARENT_CONTEXT.get(ctx, ""))
	return out

static func _add(out: Array[String], key: String) -> void:
	if not key.is_empty() and not out.has(key):
		out.append(key)

# Rule 3 scopes, most specific first: the location's region id, its parent chain, the world id, then the world's
# lineage (nearest ancestor first). Empty world -> [].
static func scopes(world_id: String, location_id: String) -> Array[String]:
	var out: Array[String] = []
	if world_id.is_empty():
		return out
	var loc: Dictionary = WorldCatalog.resolve(world_id, "locations", location_id) if not location_id.is_empty() else {}
	var region: Dictionary = _region(world_id, str(loc.get("_world", world_id)), str(loc.get("region", "")))
	var guard: int = 0
	while not region.is_empty() and guard < 32:
		var rid: String = str(region.get("id", ""))
		if rid.is_empty() or out.has(rid):
			break
		out.append(rid)
		region = _region(world_id, str(region.get("_world", world_id)), str(region.get("parent", "")))
		guard += 1
	_add(out, world_id)
	var chain: Array[String] = WorldCatalog.lineage(world_id)
	chain.reverse()
	for w: String in chain:
		_add(out, w)
	return out

# A region ref seen from the game world, else from the world that wrote it (a connected world's entry).
static func _region(world_id: String, home: String, ref: String) -> Dictionary:
	if ref.is_empty():
		return {}
	var hit: Dictionary = WorldCatalog.resolve(world_id, "regions", ref)
	if hit.is_empty() and home != world_id:
		hit = WorldCatalog.resolve(home, "regions", ref)
	return hit

# MUSIC2-L: location kind -> region map context; any other kind (gate, wild, none) plays field.
const KIND_CONTEXT: Dictionary = {
	"town": CTX_TOWN, "fortress": CTX_TOWN,
	"ruin": CTX_DANGER, "camp": CTX_DANGER, "dungeon": CTX_DANGER,
}

# The context for a location kind through KIND_CONTEXT (field when the kind is not listed).
static func kind_context(kind: String) -> String:
	return str(KIND_CONTEXT.get(kind, CTX_FIELD))

# The region map's context for `gs`'s location: its kind through KIND_CONTEXT, else field.
static func field_context(gs: GameState) -> String:
	if gs == null or gs.location_id.is_empty():
		return CTX_FIELD
	var loc: Dictionary = WorldCatalog.resolve(gs.world_id, "locations", gs.location_id)
	return kind_context(str(loc.get("kind", "")))

# The region map's request: shop while its shop panel is open, else the location's field context.
static func region_context(gs: GameState, shop_open: bool) -> String:
	return CTX_SHOP if shop_open else field_context(gs)

# The battle context for `enc`: boss when any enemy (or wave) unit has the boss tier, else combat.
static func combat_context(enc: CombatEncounter) -> String:
	var ce: CatalogEncounter = enc as CatalogEncounter
	if ce == null:
		return CTX_COMBAT
	for list_key: String in ["enemies", "wave_units"]:
		for id: String in JsonUtil.strings(ce.definition.get(list_key, [])):
			if str(ce._unit_entry(id).get("tier", "")) == "boss":
				return CTX_BOSS
	return CTX_COMBAT

# ------------------------------------------------------------------------------
# Player node (rule 5)
# ------------------------------------------------------------------------------

static func _ensure_node() -> void:
	if is_instance_valid(_node):
		return
	var tree: SceneTree = Engine.get_main_loop() as SceneTree
	if tree == null or tree.root == null:
		return
	_node = MusicDirector.new()
	tree.root.add_child.call_deferred(_node)

func _init() -> void:
	name = "MusicDirector"
	process_mode = Node.PROCESS_MODE_ALWAYS

func _ready() -> void:
	if not AudioDirector.enabled():
		return
	for i: int in 2:
		var p := AudioStreamPlayer.new()
		p.bus = BUS
		p.volume_db = MUSIC_DB
		add_child(p)
		_players.append(p)

func player_count() -> int:
	return _players.size()

func _process(_delta: float) -> void:
	var want: String = wanted_key()
	if want != _current:
		_switch(want)

func _switch(key: String) -> void:
	var was_silent: bool = _current.is_empty()
	_current = key
	if _players.is_empty():
		return
	var old: int = _active
	if _players[old].playing:
		_fade(old, 0.0, CROSSFADE_SEC, true)
	if key.is_empty():
		return
	var stream: AudioStream = AssetRegistry.stream(NS, key)
	if stream == null:
		return
	_loop(stream)
	_active = 1 - old
	var p: AudioStreamPlayer = _players[_active]
	p.stream = stream
	_set_gain(0.0, p)
	p.play()
	_fade(_active, 1.0, FIRST_FADE_SEC if was_silent else CROSSFADE_SEC, false)

func _fade(index: int, to: float, seconds: float, stop_after: bool) -> void:
	var p: AudioStreamPlayer = _players[index]
	if _tweens[index] != null and _tweens[index].is_valid():
		_tweens[index].kill()
	var from: float = clampf(db_to_linear(p.volume_db - MUSIC_DB), 0.0, 1.0)
	var t: Tween = create_tween()
	t.tween_method(_set_gain.bind(p), from, to, seconds)
	if stop_after:
		t.tween_callback(p.stop)
	_tweens[index] = t

func _set_gain(gain: float, p: AudioStreamPlayer) -> void:
	p.volume_db = MUSIC_DB + linear_to_db(maxf(gain, 0.0001))

static func _loop(stream: AudioStream) -> void:
	if stream is AudioStreamOggVorbis:
		(stream as AudioStreamOggVorbis).loop = true
	elif stream is AudioStreamMP3:
		(stream as AudioStreamMP3).loop = true
	elif stream is AudioStreamWAV:
		var w: AudioStreamWAV = stream as AudioStreamWAV
		if w.loop_mode == AudioStreamWAV.LOOP_DISABLED:
			w.loop_mode = AudioStreamWAV.LOOP_FORWARD
