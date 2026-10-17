class_name TravelRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Travel Rules (S10, spec TRAV1-L)
# Pure rules over the world kind `routes`: route validation and soft warnings, and the route graph a world sees
# (its merged routes plus the two-way routes of connected worlds that touch it). World map nodes are derived: every
# endpoint of the graph. No state; every call reads WorldCatalog.
# ==============================================================================

const TIME_MIN: int = 1
const TIME_MAX: int = 20
const PCT_MAX: int = 100
const REPEAT_HALF: int = -1   # repeat_pct -1 = encounter_pct / 2, rounded down
const MAX_SCRIPTED: int = 4
const SCRIPTED_KEYS: Array[String] = ["id", "encounter", "requires_flags", "unless_flags", "sets_flags"]
const SCRIPTED_FLAG_KEYS: Array[String] = ["requires_flags", "unless_flags", "sets_flags"]
const LIST_FIELDS: Array[String] = ["encounters", "elites", "scripted", "requires_flags", "unless_flags"]

# ------------------------------------------------------------------------------
# Validation
# ------------------------------------------------------------------------------

# Blocking shape rules of a (normalised) route: its json fields are lists.
static func shape_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	for key: String in LIST_FIELDS:
		if not (e.get(key, []) is Array):
			errors.append("%s must be a list." % str(WorldSchema.field("routes", key)["label"]))
	return errors

# Every problem of a (normalised) route entry of `world_id` beyond the generic id / name / ref checks (an unknown
# `from` / `to`, or a qualified `to` outside Connects, is reported by WorldCatalog's ref check).
static func route_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var from_ref: String = str(e.get("from", ""))
	var to_ref: String = str(e.get("to", ""))
	if from_ref.is_empty():
		errors.append("Choose where the route starts.")
	if to_ref.is_empty():
		errors.append("Choose where the route leads.")
	if not from_ref.is_empty() and not to_ref.is_empty():
		var a: Dictionary = WorldCatalog.resolve(world_id, "locations", from_ref)
		var b: Dictionary = WorldCatalog.resolve(world_id, "locations", to_ref)
		if not a.is_empty() and not b.is_empty() and str(a["_qid"]) == str(b["_qid"]):
			errors.append("A route can't lead to its own start.")
	if not _int_in(e.get("time", null), TIME_MIN, TIME_MAX):
		errors.append("Travel time must be %d..%d." % [TIME_MIN, TIME_MAX])
	var zone: Variant = e.get("zone", "")
	if not (zone is String) or (not str(zone).is_empty() and not CombatZones.zone_ids().has(str(zone))):
		errors.append("Unknown zone '%s'." % str(zone))
	if not _int_in(e.get("encounter_pct", null), 0, PCT_MAX):
		errors.append("Encounter chance must be 0..%d." % PCT_MAX)
	var repeat: Variant = e.get("repeat_pct", null)
	if not _int_in(repeat, 0, PCT_MAX) and not (repeat is int and DictUtil.to_int(repeat) == REPEAT_HALF):
		errors.append("Repeat chance must be -1 (half the encounter chance) or 0..%d." % PCT_MAX)
	if not _int_in(e.get("elite_pct", null), 0, PCT_MAX):
		errors.append("Elite chance must be 0..%d." % PCT_MAX)
	for key: String in ["encounters", "elites"]:
		errors.append_array(MapRules.pair_list_errors(world_id, "Route", key, e.get(key, [])))
	errors.append_array(_scripted_errors(world_id, e.get("scripted", [])))
	errors.append_array(QuestRules.condition_errors(e.get("requires_flags", []), "Open when"))
	errors.append_array(QuestRules.condition_errors(e.get("unless_flags", []), "Closed when"))
	return errors

static func _int_in(v: Variant, lo: int, hi: int) -> bool:
	return v is int and DictUtil.to_int(v) >= lo and DictUtil.to_int(v) <= hi

static func _scripted_errors(world_id: String, raw: Variant) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Array):
		errors.append("Scripted fights must be a list.")
		return errors
	var list: Array = raw
	if list.size() > MAX_SCRIPTED:
		errors.append("At most %d scripted fights." % MAX_SCRIPTED)
	var ids: Dictionary = {}
	for i: int in range(list.size()):
		if not (list[i] is Dictionary):
			errors.append("Scripted fight %d must be an object." % (i + 1))
			continue
		var s: Dictionary = DictUtil.dict_of(JsonUtil.normalize(list[i]))
		var sid: String = str(s.get("id", ""))
		if not (s.get("id", null) is String) or not JsonUtil.id_ok(sid):
			errors.append("Scripted fight %d: id '%s' must be lowercase [a-z0-9_]." % [i + 1, sid])
			continue
		if ids.has(sid):
			errors.append("Scripted fight id '%s' is used twice." % sid)
		ids[sid] = true
		for key: Variant in s:
			if not SCRIPTED_KEYS.has(str(key)):
				errors.append("Scripted fight '%s': unknown key '%s'." % [sid, str(key)])
		var enc: Variant = s.get("encounter", "")
		if not (enc is String) or str(enc).is_empty():
			errors.append("Scripted fight '%s': choose an encounter." % sid)
		else:
			errors.append_array(WorldCatalog.ref_errors(world_id, "Scripted fight '%s' encounter" % sid, "encounters", str(enc)))
		for fk: String in SCRIPTED_FLAG_KEYS:
			errors.append_array(QuestRules.flag_list_errors(s.get(fk, []), "Scripted fight '%s' %s" % [sid, fk], fk))
	return errors

# Soft warnings over the world's OWN routes, each prefixed with the route qid: a chance with nothing to fight, and a
# route into another world with no route of that world's graph leading back.
static func warnings(world_id: String) -> Array[String]:
	var out: Array[String] = []
	var chain: Array[String] = WorldCatalog.lineage(world_id)
	for e: Dictionary in WorldCatalog.own_entries(world_id, "routes"):
		var rq: String = str(e["_qid"])
		if DictUtil.to_int(e.get("encounter_pct", 0)) > 0 and DictUtil.array_of(e.get("encounters", [])).is_empty() and DictUtil.array_of(e.get("elites", [])).is_empty():
			out.append("%s: encounter chance %d%% but no encounters or elites (it never fights)." % [rq, DictUtil.to_int(e["encounter_pct"])])
		var to_hit: Dictionary = WorldCatalog.resolve(world_id, "locations", str(e.get("to", "")))
		if to_hit.is_empty():
			continue
		var to_world: String = str(to_hit["_world"])
		if chain.has(to_world) or _has_way_back(to_world, chain):
			continue
		out.append("%s: leads into world '%s' with no route back." % [rq, to_world])
	return out

# Does an allowed direction of some route in `to_world`'s graph lead from a location outside `home_chain` to one
# inside it?
static func _has_way_back(to_world: String, home_chain: Array[String]) -> bool:
	for r: Dictionary in graph(to_world):
		var a_home: bool = home_chain.has(_world_of(str(r["_from"])))
		var b_home: bool = home_chain.has(_world_of(str(r["_to"])))
		if not a_home and b_home:
			return true
		if DictUtil.to_bool(r.get("two_way", true)) and a_home and not b_home:
			return true
	return false

static func _world_of(location_qid: String) -> String:
	var parts: Array[String] = WorldCatalog.split_qid(location_qid)
	return parts[0] if parts.size() == 2 else ""

# ------------------------------------------------------------------------------
# Graph and nodes
# ------------------------------------------------------------------------------

# The routes `world_id` travels on: its merged routes, then the two-way routes of every other world X that reaches it
# (WorldCatalog.can_reach_world(X, world_id)) with an endpoint visible from `world_id` (X in world id order, routes in
# catalog order). Each is a route entry copy plus `_from` / `_to`: the endpoint qids as seen from `world_id`. Routes
# with an unresolved endpoint are left out. Worlds that inherit from `world_id` (or that it inherits) are not "other"
# worlds here: inheritance already shares routes downwards, and a child's routes never leak up into its parent.
static func graph(world_id: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var seen: Dictionary = {}
	var chain: Array[String] = WorldCatalog.lineage(world_id)
	for r: Dictionary in WorldCatalog.merged_entries(world_id, "routes"):
		_add_route(world_id, r, out, seen, false)
	for x: String in WorldCatalog.world_ids():
		if chain.has(x) or WorldCatalog.lineage(x).has(world_id) or not WorldCatalog.can_reach_world(x, world_id):
			continue
		for r: Dictionary in WorldCatalog.merged_entries(x, "routes"):
			if DictUtil.to_bool(r.get("two_way", true)) and not chain.has(str(r["_world"])):
				_add_route(world_id, r, out, seen, true)
	return out

static func _add_route(world_id: String, r: Dictionary, out: Array[Dictionary], seen: Dictionary, needs_visible: bool) -> void:
	var rq: String = str(r["_qid"])
	if seen.has(rq):
		return
	var a: String = endpoint(world_id, r, "from")
	var b: String = endpoint(world_id, r, "to")
	if a.is_empty() or b.is_empty():
		return
	if needs_visible and not _visible(world_id, a) and not _visible(world_id, b):
		return
	seen[rq] = true
	var copy: Dictionary = r.duplicate(true)
	copy["_from"] = a
	copy["_to"] = b
	out.append(copy)

# The qid of a route endpoint (`key` "from" / "to"): resolved where the route is written, then seen from `world_id`
# (so an override of the location applies). "" when it does not resolve.
static func endpoint(world_id: String, route: Dictionary, key: String) -> String:
	var home: String = str(route.get("_world", world_id))
	var hit: Dictionary = WorldCatalog.resolve(home, "locations", str(route.get(key, "")))
	if hit.is_empty():
		return ""
	var seen: Dictionary = WorldCatalog.resolve(world_id, "locations", str(hit["_qid"]))
	return str(seen.get("_qid", hit["_qid"]))

static func _visible(world_id: String, location_qid: String) -> bool:
	var w: String = _world_of(location_qid)
	return not w.is_empty() and WorldCatalog.can_reach_world(world_id, w) and WorldCatalog.has_entry(world_id, "locations", location_qid)

# Every endpoint qid of `world_id`'s graph, first-seen order (from before to, routes in graph order).
static func nodes(world_id: String) -> Array[String]:
	var out: Array[String] = []
	for r: Dictionary in graph(world_id):
		for key: String in ["_from", "_to"]:
			var q: String = str(r[key])
			if not out.has(q):
				out.append(q)
	return out

# ------------------------------------------------------------------------------
# Runtime reads (TRAV1-L 2.1, 2.2, 2.6): flags, open routes, known nodes, path, view
# ------------------------------------------------------------------------------

static func traversed_flag(route_qid: String) -> String:
	return "%s.traversed" % MapRules.dotted(route_qid)

static func event_done_flag(route_qid: String, event_id: String) -> String:
	return "%s.%s.done" % [MapRules.dotted(route_qid), event_id]

static func conditions_hold(gs: GameState, raw: Variant) -> bool:
	for c: Variant in DictUtil.array_of(JsonUtil.normalize(raw)):
		if not QuestRules.condition_met(gs, DictUtil.dict_of(c)):
			return false
	return true

static func any_holds(gs: GameState, raw: Variant) -> bool:
	for c: Variant in DictUtil.array_of(JsonUtil.normalize(raw)):
		if QuestRules.condition_met(gs, DictUtil.dict_of(c)):
			return true
	return false

# All requires_flags hold and no unless_flags holds.
static func is_open(gs: GameState, route: Dictionary) -> bool:
	return conditions_hold(gs, route.get("requires_flags", [])) and not any_holds(gs, route.get("unless_flags", []))

# The resolved repeat chance: repeat_pct, or encounter_pct / 2 (rounded down) for -1.
static func repeat_chance(route: Dictionary) -> int:
	var repeat: int = DictUtil.to_int(route.get("repeat_pct", REPEAT_HALF))
	return floori(DictUtil.to_int(route.get("encounter_pct", 0)) / 2.0) if repeat == REPEAT_HALF else repeat

# The road chance of the next leg on `route`: repeat once its traversed flag is set, else encounter_pct.
static func leg_chance(gs: GameState, route: Dictionary) -> int:
	if gs.get_flag(traversed_flag(str(route["_qid"])), false) == true:
		return repeat_chance(route)
	return DictUtil.to_int(route.get("encounter_pct", 0))

# The party supplies of a new game in `world_id` (TRAV2-L): the nearest manifest (the world, then up the lineage) with
# start_supplies >= 0, else PartyState.SUPPLIES_START.
static func start_supplies(world_id: String) -> int:
	var chain: Array[String] = WorldCatalog.lineage(world_id)
	chain.reverse()
	for w: String in chain:
		var v: int = DictUtil.to_int(WorldCatalog.manifest(w).get("start_supplies", -1), -1)
		if v >= 0:
			return v
	return PartyState.SUPPLIES_START

static func is_visited(gs: GameState, location_qid: String) -> bool:
	return gs.get_flag(MapRules.visited_flag(location_qid), false) == true

# Directed legs of the graph of `world_id` over OPEN routes: [{route (graph entry), from, to}] (from -> to always,
# to -> from only when two_way), graph order.
static func legs(gs: GameState, world_id: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for r: Dictionary in graph(world_id):
		if not is_open(gs, r):
			continue
		out.append({"route": r, "from": str(r["_from"]), "to": str(r["_to"])})
		if DictUtil.to_bool(r.get("two_way", true)):
			out.append({"route": r, "from": str(r["_to"]), "to": str(r["_from"])})
	return out

# Known nodes of gs.world_id's graph (node order): visited, or one open leg away from a visited node.
static func known(gs: GameState) -> Array[String]:
	var hit: Dictionary = {}
	for q: String in nodes(gs.world_id):
		if is_visited(gs, q):
			hit[q] = true
	for leg: Dictionary in legs(gs, gs.world_id):
		if is_visited(gs, str(leg["from"])):
			hit[str(leg["to"])] = true
	var out: Array[String] = []
	for q: String in nodes(gs.world_id):
		if hit.has(q):
			out.append(q)
	return out

# Route qids of the best path from `from_qid` to `to_qid` over open legs between known nodes: least total time, then
# fewest legs, then the route qid list compared element by element. [] when equal, unknown or unreachable.
static func path(gs: GameState, from_qid: String, to_qid: String) -> Array[String]:
	var none: Array[String] = []
	var known_nodes: Array[String] = known(gs)
	if from_qid == to_qid or not known_nodes.has(from_qid) or not known_nodes.has(to_qid):
		return none
	var all_legs: Array[Dictionary] = legs(gs, gs.world_id)
	# Dijkstra on the label [time, legs, route list]: every leg adds time >= 1, so labels only grow along a path.
	var best: Dictionary = {from_qid: {"time": 0, "routes": none}}
	var done: Dictionary = {}
	while true:
		var cur: String = ""
		for q: Variant in best:
			if not done.has(q) and (cur.is_empty() or _better(DictUtil.dict_of(best[q]), DictUtil.dict_of(best[cur]))):
				cur = str(q)
		if cur.is_empty() or cur == to_qid:
			break
		done[cur] = true
		var label: Dictionary = best[cur]
		for leg: Dictionary in all_legs:
			var nxt: String = str(leg["to"])
			if str(leg["from"]) != cur or done.has(nxt) or not known_nodes.has(nxt):
				continue
			var route: Dictionary = leg["route"]
			var routes: Array[String] = JsonUtil.strings(label["routes"])
			routes.append(str(route["_qid"]))
			var cand: Dictionary = {"time": DictUtil.to_int(label["time"]) + DictUtil.to_int(route.get("time", 0)), "routes": routes}
			if not best.has(nxt) or _better(cand, DictUtil.dict_of(best[nxt])):
				best[nxt] = cand
	if not best.has(to_qid):
		return none
	return JsonUtil.strings(DictUtil.dict_of(best[to_qid])["routes"])

static func _better(a: Dictionary, b: Dictionary) -> bool:
	var ta: int = DictUtil.to_int(a["time"])
	var tb: int = DictUtil.to_int(b["time"])
	if ta != tb:
		return ta < tb
	var ra: Array[String] = JsonUtil.strings(a["routes"])
	var rb: Array[String] = JsonUtil.strings(b["routes"])
	if ra.size() != rb.size():
		return ra.size() < rb.size()
	for i: int in range(ra.size()):
		if ra[i] != rb[i]:
			return ra[i] < rb[i]
	return false

# The graph entry of `route_qid` in `world_id`'s graph ({} when none).
static func route_in(world_id: String, route_qid: String) -> Dictionary:
	for r: Dictionary in graph(world_id):
		if str(r["_qid"]) == route_qid:
			return r
	return {}

# What the world map draws: {current, nodes: [{qid, name, visited}], routes: [{qid, from, to, two_way, open, time}]}
# for known nodes only; a closed route is listed only when both its endpoints are known, an open one when both are.
static func view(gs: GameState) -> Dictionary:
	var known_nodes: Array[String] = known(gs)
	var node_list: Array = []
	for q: String in known_nodes:
		var loc: Dictionary = ExploreRunner.resolve_location(gs, q)
		node_list.append({"qid": q, "name": str(loc.get("name", q)), "visited": is_visited(gs, q)})
	var route_list: Array = []
	for r: Dictionary in graph(gs.world_id):
		var a: String = str(r["_from"])
		var b: String = str(r["_to"])
		if not known_nodes.has(a) or not known_nodes.has(b):
			continue
		route_list.append({"qid": str(r["_qid"]), "from": a, "to": b, "two_way": DictUtil.to_bool(r.get("two_way", true)), "open": is_open(gs, r), "time": DictUtil.to_int(r.get("time", 0))})
	return {"current": gs.location_id, "nodes": node_list, "routes": route_list}
