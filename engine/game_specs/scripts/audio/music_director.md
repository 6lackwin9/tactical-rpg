---
class: MusicDirector
extends: Node
description: "Tactical RPG - Music director (MUSIC1-L, spec docs/specs/MUSIC1.md section 2) Scenes never play music themselves: they post requests (owner, context, priority, world, location) and the director plays the one wanted track on the Music bus. A context resolves to a manifest key through a data-only fallback chain (rule 7), so new tracks need a manifest row, no code. The choice is a pure static function over the requests (rule 2), so tests run headless. One lazily created node under the tree root (survives scene changes) owns two players and crossfades between them; headless runs create no players but still track the wanted key."
---

# Properties
* `MENU`: int = 0 (const)
* `FIELD`: int = 1 (const)
* `COMBAT`: int = 2 (const)
* `CTX_MENU`: String = "menu" (const)
* `CTX_FIELD`: String = "field" (const)
* `CTX_TRAVEL`: String = "travel" (const)
* `CTX_TOWN`: String = "town" (const)
* `CTX_SHOP`: String = "shop" (const)
* `CTX_ENCOUNTER`: String = "encounter" (const)
* `CTX_DANGER`: String = "danger" (const)
* `CTX_BOSS`: String = "boss" (const)
* `CTX_COMBAT`: String = "combat" (const)
* `CONTEXTS`: Dictionary = { (const)
* `PARENT_CONTEXT`: Dictionary = { (const)
* `MENU_KEY`: String = "menu_theme"   # the menu context's last step (const)
* `MUSIC_DB`: float = -6.0 (const)
* `CROSSFADE_SEC`: float = 1.0 (const)
* `FIRST_FADE_SEC`: float = 2.0 (const)
* `BUS`: String = "Music" (const)
* `NS`: String = "music" (const)
* `_players`: Array[AudioStreamPlayer] = []
* `_tweens`: Array[Tween] = [null, null]
* `_active`: int = 0
* `_current`: String = ""   # the key playing (headless: the key that would play)
* `out`: Array[String] = []
* `sc`: Array[String] = scopes(world_id, location_id)
* `ctx`: String = context
* `seen`: Array[String] = []
* `out`: Array[String] = []
* `loc`: Dictionary = WorldCatalog.resolve(world_id, "locations", location_id) if not location_id.is_empty() else {}
* `region`: Dictionary = _region(world_id, str(loc.get("_world", world_id)), str(loc.get("region", "")))
* `guard`: int = 0
* `rid`: String = str(region.get("id", ""))
* `chain`: Array[String] = WorldCatalog.lineage(world_id)
* `KIND_CONTEXT`: Dictionary = { (const)

# Methods

### `static request(owner: Node, context: String, priority: int, world_id: String = "", location_id: String = "") -> void`
```logic
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
```

### `static release(owner: Node) -> void`
```logic
if owner == null:
	return
var id: int = owner.get_instance_id()
for i: int in range(_requests.size() - 1, -1, -1):
	if DictUtil.to_int(_requests[i]["id"]) == id:
		_requests.remove_at(i)
```

### `static request_game(owner: Node, context: String, gs: GameState) -> void`
```logic
if gs == null:
	release(owner)
	return
request(owner, context, priority_of(context), gs.world_id, gs.location_id)
```

### `static priority_of(context: String) -> int`
```logic
return DictUtil.to_int(CONTEXTS.get(context, MENU))
```

### `static clear_requests() -> void`
```logic
_requests.clear()
clear_caches()
```

### `static clear_caches() -> void`
```logic
_track_ok.clear()
_key_cache.clear()
```

### `static _prune() -> void`
```logic
for i: int in range(_requests.size() - 1, -1, -1):
	if not is_instance_valid(instance_from_id(DictUtil.to_int(_requests[i]["id"]))):
		_requests.remove_at(i)
```

### `static has_track(key: String) -> bool`
```logic
if key.is_empty():
	return false
return str(AssetRegistry.resolve(NS, key, "stream")["source"]) == "file"
```

### `static _has_track_cached(key: String) -> bool`
```logic
if not _track_ok.has(key):
	_track_ok[key] = has_track(key)
return DictUtil.to_bool(_track_ok[key])
```

### `static owner_live(owner_id: int) -> bool`
```logic
var o: Object = instance_from_id(owner_id)
if not is_instance_valid(o) or not (o is Node):
	return false
var n: Node = o as Node
if not n.is_inside_tree():
	return false
if n is CanvasItem:
	return (n as CanvasItem).is_visible_in_tree()
return true
```

### `static choose(reqs: Array[Dictionary], track: Callable = Callable()) -> String`
```logic
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
```

### `static wanted_key() -> String`
```logic
_prune()
return choose(_requests, _cached_track_for)
```

### `static playing_key() -> String`
```logic
return _node._current if is_instance_valid(_node) else ""
```

### `static track_for(context: String, world_id: String, location_id: String, resolves: Callable = Callable()) -> String`
```logic
var check: Callable = resolves if resolves.is_valid() else has_track
for key: String in candidates(context, world_id, location_id):
	if DictUtil.to_bool(check.call(key)):
		return key
return ""
```

### `static _cached_track_for(context: String, world_id: String, location_id: String) -> String`
```logic
var ck: String = "%s|%s|%s" % [context, world_id, location_id]
if not _key_cache.has(ck):
	_key_cache[ck] = track_for(context, world_id, location_id, _has_track_cached)
return str(_key_cache[ck])
```

### `static _add(out: Array[String], key: String) -> void`
```logic
if not key.is_empty() and not out.has(key):
	out.append(key)
```

### `static _region(world_id: String, home: String, ref: String) -> Dictionary`
```logic
if ref.is_empty():
	return {}
var hit: Dictionary = WorldCatalog.resolve(world_id, "regions", ref)
if hit.is_empty() and home != world_id:
	hit = WorldCatalog.resolve(home, "regions", ref)
return hit
```

### `static kind_context(kind: String) -> String`
```logic
return str(KIND_CONTEXT.get(kind, CTX_FIELD))
```

### `static field_context(gs: GameState) -> String`
```logic
if gs == null or gs.location_id.is_empty():
	return CTX_FIELD
var loc: Dictionary = WorldCatalog.resolve(gs.world_id, "locations", gs.location_id)
return kind_context(str(loc.get("kind", "")))
```

### `static region_context(gs: GameState, shop_open: bool) -> String`
```logic
return CTX_SHOP if shop_open else field_context(gs)
```

### `static combat_context(enc: CombatEncounter) -> String`
```logic
var ce: CatalogEncounter = enc as CatalogEncounter
if ce == null:
	return CTX_COMBAT
var refs: Array[String] = JsonUtil.strings(ce.definition.get("enemies", []))
refs.append_array(JsonUtil.strings(ce.definition.get("wave_units", [])))
refs.append_array(EncounterPhases.reserved_refs(ce.definition))   # BOSS1 rule 8: phase waves too
for id: String in refs:
	if str(ce._unit_entry(id).get("tier", "")) == "boss":
		return CTX_BOSS
return CTX_COMBAT
```

### `static _ensure_node() -> void`
```logic
if is_instance_valid(_node):
	return
var tree: SceneTree = Engine.get_main_loop() as SceneTree
if tree == null or tree.root == null:
	return
_node = MusicDirector.new()
tree.root.add_child.call_deferred(_node)
```

### `_init() -> void`
```logic
name = "MusicDirector"
process_mode = Node.PROCESS_MODE_ALWAYS
```

### `_ready() -> void`
```logic
if not AudioDirector.enabled():
	return
for i: int in 2:
	var p := AudioStreamPlayer.new()
	p.bus = BUS
	p.volume_db = MUSIC_DB
	add_child(p)
	_players.append(p)
```

### `player_count() -> int`
```logic
return _players.size()
```

### `_process(_delta: float) -> void`
```logic
var want: String = wanted_key()
if want != _current:
	_switch(want)
```

### `_switch(key: String) -> void`
```logic
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
```

### `_fade(index: int, to: float, seconds: float, stop_after: bool) -> void`
```logic
var p: AudioStreamPlayer = _players[index]
if _tweens[index] != null and _tweens[index].is_valid():
	_tweens[index].kill()
var from: float = clampf(db_to_linear(p.volume_db - MUSIC_DB), 0.0, 1.0)
var t: Tween = create_tween()
t.tween_method(_set_gain.bind(p), from, to, seconds)
if stop_after:
	t.tween_callback(p.stop)
_tweens[index] = t
```

### `_set_gain(gain: float, p: AudioStreamPlayer) -> void`
```logic
p.volume_db = MUSIC_DB + linear_to_db(maxf(gain, 0.0001))
```

### `static _loop(stream: AudioStream) -> void`
```logic
if stream is AudioStreamOggVorbis:
	(stream as AudioStreamOggVorbis).loop = true
elif stream is AudioStreamMP3:
	(stream as AudioStreamMP3).loop = true
elif stream is AudioStreamWAV:
	var w: AudioStreamWAV = stream as AudioStreamWAV
	if w.loop_mode == AudioStreamWAV.LOOP_DISABLED:
		w.loop_mode = AudioStreamWAV.LOOP_FORWARD
```

