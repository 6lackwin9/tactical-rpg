class_name ExploreRunner
extends RefCounted

# ==============================================================================
# Tactical RPG - Explore runner (S9, spec EXP1-L section 2): walking a location's region map over GameState.
# Stateless: everything lives in the GameState (location_id, position, explore, flags, rolls). Refs resolve from
# gs.world_id (child overrides apply). Every call returns {ok, errors, events...}; errors change nothing. Flags go
# through QuestRunner.set_flag (so quests refresh). The caller starts a battle at once on a `fight` event and calls
# after_fight with the BattleBridge outcome; it writes the save on `autosave` (the runner does no I/O).
# ==============================================================================

const DIRS: Array[Vector2i] = [Vector2i(0, -1), Vector2i(1, 0), Vector2i(0, 1), Vector2i(-1, 0)]

# ------------------------------------------------------------------------------
# Lookups
# ------------------------------------------------------------------------------

# The current location entry ({} when none), resolved from gs.world_id.
static func location(gs: GameState) -> Dictionary:
	if gs.location_id.is_empty():
		return {}
	return WorldCatalog.resolve(gs.world_id, "locations", gs.location_id)

static func template_of(loc: Dictionary) -> Dictionary:
	var map_id: String = str(loc.get("map", ""))
	return MapCatalog.get_entry(map_id) if not map_id.is_empty() else {}

static func markers_of(loc: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for m: Variant in DictUtil.array_of(loc.get("markers", [])):
		out.append(MapRules.marker_full(DictUtil.dict_of(m)))
	return out

static func _conditions_hold(gs: GameState, raw: Variant) -> bool:
	for c: Variant in DictUtil.array_of(JsonUtil.normalize(raw)):
		if not QuestRules.condition_met(gs, DictUtil.dict_of(c)):
			return false
	return true

static func _any_holds(gs: GameState, raw: Variant) -> bool:
	for c: Variant in DictUtil.array_of(JsonUtil.normalize(raw)):
		if QuestRules.condition_met(gs, DictUtil.dict_of(c)):
			return true
	return false

static func _is_done(gs: GameState, loc_qid: String, m: Dictionary) -> bool:
	return gs.get_flag(MapRules.done_flag(loc_qid, str(m["id"])), false) == true

# "present", "locked" (an exit whose requires_flags fail while its unless_flags pass) or "absent" (EXP1-L 2.1).
static func presence(gs: GameState, loc_qid: String, m: Dictionary) -> String:
	var req_ok: bool = _conditions_hold(gs, m.get("requires_flags", []))
	var unless_ok: bool = not _any_holds(gs, m.get("unless_flags", []))
	var kind: String = str(m.get("kind", ""))
	if (kind == "encounter" or kind == "roamer") and DictUtil.to_bool(m.get("once", true)) and _is_done(gs, loc_qid, m):
		return "absent"
	if kind == "exit" and not req_ok and unless_ok:
		return "locked"
	if kind == "npc" and DialogueRules.is_gone(gs, str(m.get("character", ""))):
		return "absent"   # DLG1-L 2.1: a gone character's npc markers are absent
	return "present" if req_ok and unless_ok else "absent"

# Where a marker stands for `loc_steps` (roamers walk their path; others sit on their anchor). (-1, -1) = nowhere.
static func marker_pos(t: Dictionary, m: Dictionary, loc_steps: int) -> Vector2i:
	if str(m.get("kind", "")) == "roamer":
		var pts: Array[Vector2i] = MapRules.path_points(t, str(m.get("path", "")))
		return pts[MapRules.roamer_index(pts.size(), loc_steps)] if not pts.is_empty() else GameState.NO_POSITION
	return MapRules.anchor_pos(t, str(m.get("anchor", "")))

static func _solid(state: String, m: Dictionary) -> bool:
	return state == "locked" or (state == "present" and MapRules.SOLID_KINDS.has(str(m.get("kind", ""))))

# A SAFE tile: no zone, a safe zone, or no map at all.
static func tile_safe(t: Dictionary, p: Vector2i) -> bool:
	return t.is_empty() or MapRules.zone_kind(t, MapRules.zone_at(t, p.x, p.y)) != "danger"

# The binding of the bound DANGER zone under `p` ({} when SAFE or unbound).
static func binding_at(loc: Dictionary, t: Dictionary, p: Vector2i) -> Dictionary:
	var zone: String = MapRules.zone_at(t, p.x, p.y)
	if zone.is_empty() or MapRules.zone_kind(t, zone) != "danger":
		return {}
	var bindings: Dictionary = DictUtil.dict_of(loc.get("zone_bindings", {}))
	return MapRules.binding_full(DictUtil.dict_of(bindings[zone])) if bindings.has(zone) else {}

static func _encounter_qid(gs: GameState, ref: String) -> String:
	return str(WorldCatalog.resolve(gs.world_id, "encounters", ref).get("_qid", ""))

static func _pos_array(p: Vector2i) -> Array:
	return [p.x, p.y]

static func _result(ok: bool, errors: Array[String], events: Array[Dictionary]) -> Dictionary:
	return {"ok": ok, "errors": errors, "events": events}

static func _fail(message: String) -> Dictionary:
	var errors: Array[String] = [message]
	var events: Array[Dictionary] = []
	return _result(false, errors, events)

# Applies a marker's sets_flags through QuestRunner.set_flag; returns the keys set.
static func _apply_sets(gs: GameState, m: Dictionary) -> Array[String]:
	var keys: Array[String] = []
	for c: Variant in DictUtil.array_of(JsonUtil.normalize(m.get("sets_flags", []))):
		var cond: Dictionary = DictUtil.dict_of(c)
		var key: String = str(cond.get("key", ""))
		if QuestRunner.set_flag(gs, key, cond.get("value", true)).is_empty():
			keys.append(key)
	return keys

# ------------------------------------------------------------------------------
# Entering and leaving (EXP1-L 2.5)
# ------------------------------------------------------------------------------

# The location `ref` names for this state: from gs.world_id first (overrides apply), else a qualified `w:id` in its
# own world (arrivals over a connecting world's route).
static func resolve_location(gs: GameState, ref: String) -> Dictionary:
	var hit: Dictionary = WorldCatalog.resolve(gs.world_id, "locations", ref)
	if hit.is_empty() and ref.contains(":"):
		var parts: Array[String] = WorldCatalog.split_qid(ref)
		if parts.size() == 2 and WorldCatalog.has_world(parts[0]):
			hit = WorldCatalog.resolve(parts[0], "locations", parts[1])
	return hit

static func enter(gs: GameState, location_ref: String, anchor: String = MapRules.SPAWN) -> Dictionary:
	var loc: Dictionary = resolve_location(gs, location_ref)
	if loc.is_empty():
		return _fail("Unknown location '%s'." % location_ref)
	var t: Dictionary = template_of(loc)
	if not str(loc.get("map", "")).is_empty() and t.is_empty():
		return _fail("Location '%s' names an unknown map '%s'." % [str(loc["_qid"]), str(loc.get("map", ""))])
	var pos: Vector2i = GameState.NO_POSITION
	if not t.is_empty():
		pos = MapRules.anchor_pos(t, anchor)
		if pos == GameState.NO_POSITION:
			return _fail("Map '%s' has no anchor '%s'." % [str(t["id"]), anchor])
	var loc_world: String = str(loc["_world"])
	if not WorldCatalog.lineage(gs.world_id).has(loc_world):
		gs.world_id = loc_world   # a `connects` crossing
	var qid: String = str(loc["_qid"])
	gs.location_id = qid
	gs.position = pos
	gs.explore["loc_steps"] = 0
	var _visited: String = QuestRunner.set_flag(gs, MapRules.visited_flag(qid), true)
	if t.is_empty() or tile_safe(t, MapRules.anchor_pos(t, MapRules.SPAWN)):
		gs.explore["last_safe"] = qid
	QuestRunner.refresh(gs)
	var advanced: Array[String] = QuestRunner.on_location_reached(gs, qid)
	var errors: Array[String] = []
	var events: Array[Dictionary] = [{"kind": "entered", "location": qid, "advanced": advanced}, {"kind": "autosave"}]
	return _result(true, errors, events)

# Enters the world's resolved start location at its spawn.
static func begin(gs: GameState) -> Dictionary:
	var start: String = MapRules.start_location(gs.world_id)
	if start.is_empty():
		return _fail("World '%s' has no start location." % gs.world_id)
	return enter(gs, start)

# After a load: no hooks, no autosave; a position off the map or not open goes back to spawn. An empty location_id
# falls back to begin (the start location, with its entry hooks and autosave).
static func resume(gs: GameState) -> Dictionary:
	var errors: Array[String] = []
	var events: Array[Dictionary] = []
	if gs.location_id.is_empty():
		return begin(gs)   # a save from before exploration (or a cleared location): start over at the start location
	var loc: Dictionary = location(gs)
	if loc.is_empty():
		return _fail("Unknown location '%s'." % gs.location_id)
	var t: Dictionary = template_of(loc)
	if t.is_empty():
		gs.position = GameState.NO_POSITION
	elif not MapRules.is_open(t, gs.position.x, gs.position.y):
		gs.position = MapRules.anchor_pos(t, MapRules.SPAWN)
		events.append({"kind": "moved", "from": [], "to": _pos_array(gs.position)})
	return _result(true, errors, events)

# Leaves through one of the current location's `exits` (the menu path; exit markers call it with their to_anchor).
static func take_exit(gs: GameState, to: String, anchor: String = MapRules.SPAWN) -> Dictionary:
	var loc: Dictionary = location(gs)
	if not MapRules.exit_targets(loc).has(to) and to != MapRules.WORLD_MAP:
		return _fail("No exit to '%s' here." % to)
	if to == MapRules.WORLD_MAP:
		var errors: Array[String] = []
		var events: Array[Dictionary] = [{"kind": "exit", "to": MapRules.WORLD_MAP}]
		return _result(true, errors, events)
	return enter(gs, to, anchor)

# {to, label} of the exits of a map-less location ([] with a map), plus {to: "world_map"} when the location is a world
# map node (TRAV1-L 2.6).
static func menu(gs: GameState) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var loc: Dictionary = location(gs)
	if loc.is_empty() or not str(loc.get("map", "")).is_empty():
		return out
	for x: Variant in DictUtil.array_of(loc.get("exits", [])):
		var ex: Dictionary = DictUtil.dict_of(x)
		out.append({"to": str(ex.get("to", "")), "label": str(ex.get("label", ex.get("to", "")))})
	if TravelRules.nodes(gs.world_id).has(str(loc["_qid"])):
		out.append({"to": MapRules.WORLD_MAP, "label": "World map"})
	return out

# The defeat rule (also TRAV1, DLG1): wake at `at` (or the current location) when it has no map or a SAFE spawn,
# else at explore.last_safe, else at the start location. Fully healed (no HP is stored); nothing else is lost.
static func wake(gs: GameState, at: String = "") -> Dictionary:
	var target_ref: String = at if not at.is_empty() else gs.location_id
	var loc: Dictionary = resolve_location(gs, target_ref)
	var t: Dictionary = template_of(loc)
	if not loc.is_empty() and (t.is_empty() or tile_safe(t, MapRules.anchor_pos(t, MapRules.SPAWN))):
		return enter(gs, str(loc["_qid"]))
	var last_safe: String = str(gs.explore.get("last_safe", ""))
	if not last_safe.is_empty():
		return enter(gs, last_safe)
	return begin(gs)

# ------------------------------------------------------------------------------
# Stepping (EXP1-L 2.2)
# ------------------------------------------------------------------------------

static func step(gs: GameState, dir: Vector2i) -> Dictionary:
	if not DIRS.has(dir):
		return _fail("A step is one tile up, down, left or right.")
	var loc: Dictionary = location(gs)
	var t: Dictionary = template_of(loc)
	if t.is_empty():
		return _fail("This location has no map to walk.")
	var errors: Array[String] = []
	var events: Array[Dictionary] = []
	var loc_qid: String = str(loc["_qid"])
	var markers: Array[Dictionary] = markers_of(loc)
	var loc_steps: int = DictUtil.to_int(gs.explore.get("loc_steps", 0))
	var from: Vector2i = gs.position
	var target: Vector2i = from + dir
	# 1. blocked / locked: nothing else changes
	if not MapRules.is_open(t, target.x, target.y):
		events.append({"kind": "blocked", "at": _pos_array(target)})
		return _result(true, errors, events)
	for m: Dictionary in markers:
		var state: String = presence(gs, loc_qid, m)
		if _solid(state, m) and marker_pos(t, m, loc_steps) == target:
			if state == "locked":
				events.append({"kind": "locked", "marker": str(m["id"]), "text": str(m.get("locked_text", ""))})
			else:
				events.append({"kind": "blocked", "at": _pos_array(target), "marker": str(m["id"])})
			return _result(true, errors, events)
	# A world_map exit hands over to the world map (TRAV1) without moving; only its sets_flags apply (TRAV1-L commit 3).
	for m: Dictionary in markers:
		if str(m["kind"]) == "exit" and str(m.get("to", "")) == MapRules.WORLD_MAP and presence(gs, loc_qid, m) == "present" and marker_pos(t, m, loc_steps) == target:
			var wm_keys: Array[String] = _apply_sets(gs, m)
			if not wm_keys.is_empty():
				events.append({"kind": "flags_set", "marker": str(m["id"]), "flags": wm_keys})
			events.append({"kind": "exit", "to": MapRules.WORLD_MAP, "marker": str(m["id"])})
			return _result(true, errors, events)
	# An exit on the tile whose target can't be entered refuses the step before anything changes (dry run on a copy).
	for m: Dictionary in markers:
		if str(m["kind"]) == "exit" and presence(gs, loc_qid, m) == "present" and marker_pos(t, m, loc_steps) == target:
			var probe: Dictionary = enter(GameState.from_dict(gs.to_dict()), str(m.get("to", "")), str(m.get("to_anchor", MapRules.SPAWN)))
			if not DictUtil.to_bool(probe["ok"]):
				errors.assign(DictUtil.array_of(probe["errors"]))
				return _result(false, errors, events)
	# 2. move; points and exits on the tile
	gs.position = target
	events.append({"kind": "moved", "from": _pos_array(from), "to": _pos_array(target)})
	for m: Dictionary in markers:
		if marker_pos(t, m, loc_steps) != target or presence(gs, loc_qid, m) != "present":
			continue
		match str(m["kind"]):
			"point":
				var keys: Array[String] = _apply_sets(gs, m)
				if not keys.is_empty():
					events.append({"kind": "flags_set", "marker": str(m["id"]), "flags": keys})
			"exit":
				var exit_keys: Array[String] = _apply_sets(gs, m)
				if not exit_keys.is_empty():
					events.append({"kind": "flags_set", "marker": str(m["id"]), "flags": exit_keys})
				var to: String = str(m.get("to", ""))
				var left: Dictionary = enter(gs, to, str(m.get("to_anchor", MapRules.SPAWN)))
				if not DictUtil.to_bool(left["ok"]):
					errors.assign(DictUtil.array_of(left["errors"]))
					return _result(false, errors, events)
				events.append({"kind": "exit", "to": to, "marker": str(m["id"]), "events": left["events"]})
				return _result(true, errors, events)
	# 3. counters
	loc_steps += 1
	gs.explore["loc_steps"] = loc_steps
	var binding: Dictionary = binding_at(loc, t, target)
	if not binding.is_empty():
		gs.explore["steps"] = DictUtil.to_int(gs.explore.get("steps", 0)) + 1
	# 4. roamers take their position for the new loc_steps (marker_pos below); 5. triggers, first match only
	var fight: Dictionary = _trigger(gs, loc, t, markers, target, loc_steps, binding)
	if not fight.is_empty():
		fight["from"] = _pos_array(from)
		fight["location"] = loc_qid
		events.append(fight)
	# 6. interaction hints
	for m: Dictionary in markers:
		if not ["npc", "chest", "lore"].has(str(m["kind"])) or presence(gs, loc_qid, m) != "present":
			continue
		var at: Vector2i = marker_pos(t, m, loc_steps)
		for d: Vector2i in DIRS:
			if target + d == at:
				events.append({"kind": "interact_hint", "marker": str(m["id"]), "marker_kind": str(m["kind"]), "dir": _pos_array(d)})
	return _result(true, errors, events)

static func _manhattan(a: Vector2i, b: Vector2i) -> int:
	return absi(a.x - b.x) + absi(a.y - b.y)

static func _fight(gs: GameState, source: String, ref: String, marker: String, elite: bool) -> Dictionary:
	return {"kind": "fight", "source": source, "encounter": _encounter_qid(gs, ref), "marker": marker, "elite": elite}

static func _trigger(gs: GameState, loc: Dictionary, t: Dictionary, markers: Array[Dictionary], at: Vector2i, loc_steps: int, binding: Dictionary) -> Dictionary:
	var loc_qid: String = str(loc["_qid"])
	for m: Dictionary in markers:
		if str(m["kind"]) == "encounter" and str(m.get("trigger", "")) == "touch" and presence(gs, loc_qid, m) == "present" and marker_pos(t, m, loc_steps) == at:
			return _fight(gs, "marker", str(m["encounter"]), str(m["id"]), false)
	for m: Dictionary in markers:
		if str(m["kind"]) == "roamer" and presence(gs, loc_qid, m) == "present":
			var rp: Vector2i = marker_pos(t, m, loc_steps)
			if rp != GameState.NO_POSITION and _manhattan(rp, at) <= DictUtil.to_int(m.get("radius", 0)):
				return _fight(gs, "roamer", str(m["encounter"]), str(m["id"]), false)
	for m: Dictionary in markers:
		if str(m["kind"]) == "encounter" and str(m.get("trigger", "")) == "proximity" and presence(gs, loc_qid, m) == "present":
			if _manhattan(marker_pos(t, m, loc_steps), at) <= DictUtil.to_int(m.get("radius", 1)):
				return _fight(gs, "marker", str(m["encounter"]), str(m["id"]), false)
	if binding.is_empty() or DictUtil.to_int(gs.explore.get("steps", 0)) < DictUtil.to_int(gs.explore.get("next_at", 1)):
		return {}
	return _counter_pick(gs, binding)

# EXP1-L 2.4 counter pick: both lists = chance(elite_pct) then pick; one list = pick from it.
static func _counter_pick(gs: GameState, binding: Dictionary) -> Dictionary:
	var normal: Array = DictUtil.array_of(binding.get("encounters", []))
	var elites: Array = DictUtil.array_of(binding.get("elites", []))
	var elite: bool = false
	if not elites.is_empty() and not normal.is_empty():
		elite = GameRolls.chance(gs, GameRolls.EXPLORE, DictUtil.to_int(binding.get("elite_pct", MapRules.DEFAULT_ELITE_PCT)))
	elif not elites.is_empty():
		elite = true
	var ref: String = GameRolls.pick(gs, GameRolls.EXPLORE, elites if elite else normal)
	if ref.is_empty():
		return {}
	return _fight(gs, "counter", ref, "", elite)

# Counter reset after any fight: steps = 0, next_at rolled from the bound DANGER zone under the party, else 12..20.
static func reset_counter(gs: GameState) -> void:
	var loc: Dictionary = location(gs)
	var t: Dictionary = template_of(loc)
	var binding: Dictionary = binding_at(loc, t, gs.position) if not t.is_empty() else {}
	var lo: int = DictUtil.to_int(binding.get("steps_min", MapRules.DEFAULT_STEPS_MIN))
	var hi: int = DictUtil.to_int(binding.get("steps_max", MapRules.DEFAULT_STEPS_MAX))
	gs.explore["steps"] = 0
	gs.explore["next_at"] = GameRolls.range_i(gs, GameRolls.EXPLORE, lo, hi)

# {ok, errors, events, xp, advanced}: BattleBridge.apply, then won: the marker's done flag (once) and sets_flags;
# fled: back to the tile before the step; lost: wake. Then the counter reset.
static func after_fight(gs: GameState, fight: Dictionary, result: String) -> Dictionary:
	var applied: Dictionary = BattleBridge.apply(gs, str(fight.get("encounter", "")), result)
	var errors: Array[String] = []
	var events: Array[Dictionary] = []
	if not DictUtil.to_bool(applied["ok"]):
		errors.assign(DictUtil.array_of(applied["errors"]))
		return {"ok": false, "errors": errors, "events": events, "xp": {}, "advanced": [] as Array[String]}
	var marker_id: String = str(fight.get("marker", ""))
	match result:
		"won":
			if not marker_id.is_empty():
				var loc: Dictionary = location(gs)
				for m: Dictionary in markers_of(loc):
					if str(m["id"]) != marker_id:
						continue
					if DictUtil.to_bool(m.get("once", true)):
						var _done: String = QuestRunner.set_flag(gs, MapRules.done_flag(str(loc["_qid"]), marker_id), true)
					var keys: Array[String] = _apply_sets(gs, m)
					if not keys.is_empty():
						events.append({"kind": "flags_set", "marker": marker_id, "flags": keys})
		"fled":
			var from: Array = DictUtil.array_of(fight.get("from", []))
			if from.size() == 2:
				gs.position = Vector2i(DictUtil.to_int(from[0]), DictUtil.to_int(from[1]))
		"lost":
			var woke: Dictionary = wake(gs)
			events.append_array(DictUtil.array_of(woke.get("events", [])))
	reset_counter(gs)
	return {"ok": true, "errors": errors, "events": events, "xp": applied["xp"], "advanced": applied["advanced"]}

# ------------------------------------------------------------------------------
# Interacting (EXP1-L 2.5)
# ------------------------------------------------------------------------------

# The present npc / chest / lore marker on position + dir: {ok, errors, kind ("" = nothing), ...}.
static func interact(gs: GameState, dir: Vector2i) -> Dictionary:
	var errors: Array[String] = []
	if not DIRS.has(dir):
		errors.append("Interact faces up, down, left or right.")
		return {"ok": false, "errors": errors, "kind": ""}
	var loc: Dictionary = location(gs)
	var t: Dictionary = template_of(loc)
	if t.is_empty():
		errors.append("This location has no map.")
		return {"ok": false, "errors": errors, "kind": ""}
	var loc_qid: String = str(loc["_qid"])
	var at: Vector2i = gs.position + dir
	var loc_steps: int = DictUtil.to_int(gs.explore.get("loc_steps", 0))
	for m: Dictionary in markers_of(loc):
		var kind: String = str(m["kind"])
		if not ["npc", "chest", "lore"].has(kind) or presence(gs, loc_qid, m) != "present" or marker_pos(t, m, loc_steps) != at:
			continue
		var out: Dictionary = {"ok": true, "errors": errors, "kind": kind, "marker": str(m["id"])}
		match kind:
			"npc":
				out["character"] = str(WorldCatalog.resolve(gs.world_id, "characters", str(m["character"])).get("_qid", ""))
			"lore":
				var lore: Dictionary = WorldCatalog.resolve(gs.world_id, "lore", str(m["lore"]))
				out["lore"] = str(lore.get("_qid", ""))
				out["name"] = str(lore.get("name", ""))
				out["text"] = str(lore.get("text", ""))
			"chest":
				if _is_done(gs, loc_qid, m):
					out["empty"] = true
					return out
				var opened: Dictionary = _open_chest(gs, loc_qid, m)
				if not DictUtil.to_bool(opened["ok"]):
					return opened
				out.merge(opened, true)
		out["flags"] = _apply_sets(gs, m)
		return out
	return {"ok": true, "errors": errors, "kind": ""}

# Items, gold (DLG1-L) plus the loot item (when unclaimed and its requires_flags hold) in one RewardGrant, then the
# claim and the done flag.
static func _open_chest(gs: GameState, loc_qid: String, m: Dictionary) -> Dictionary:
	var items: Dictionary = JsonUtil.ints(m.get("items", {}))
	var loot_qid: String = ""
	var loot_ref: String = str(m.get("loot", ""))
	if not loot_ref.is_empty():
		var loot: Dictionary = WorldCatalog.resolve(gs.world_id, "loot", loot_ref)
		var lqid: String = str(loot.get("_qid", ""))
		if not lqid.is_empty() and not gs.loot_claimed.has(lqid) and _conditions_hold(gs, loot.get("requires_flags", [])):
			var ref: String = str(loot.get("ref", ""))
			items[ref] = DictUtil.to_int(items.get(ref, 0)) + DictUtil.to_int(loot.get("count", 1), 1)
			loot_qid = lqid
	var gold: int = maxi(DictUtil.to_int(m.get("gold", 0)), 0)
	var granted: Dictionary = RewardGrant.apply(gs, {"items": items, "gold": gold}, "")
	if not DictUtil.to_bool(granted["ok"]):
		return {"ok": false, "errors": granted["errors"], "kind": "chest"}
	if not loot_qid.is_empty():
		var _claim: String = RewardGrant.claim_loot(gs, loot_qid)
	var _done: String = QuestRunner.set_flag(gs, MapRules.done_flag(loc_qid, str(m["id"])), true)
	return {"ok": true, "empty": false, "items": items, "loot": loot_qid, "gold": DictUtil.to_int(DictUtil.dict_of(granted["granted"]).get("gold", 0))}
