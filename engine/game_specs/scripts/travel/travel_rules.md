---
class: TravelRules
extends: RefCounted
description: "Tactical RPG - Travel Rules (S10, spec TRAV1-L) Pure rules over the world kind `routes`: route validation and soft warnings, and the route graph a world sees (its merged routes plus the two-way routes of connected worlds that touch it). World map nodes are derived: every endpoint of the graph. No state; every call reads WorldCatalog."
---

# Properties
* `TIME_MIN`: int = 1 (const)
* `TIME_MAX`: int = 20 (const)
* `PCT_MAX`: int = 100 (const)
* `REPEAT_HALF`: int = -1   # repeat_pct -1 = encounter_pct / 2, rounded down (const)
* `STARVED_PCT`: int = 150  # TRAV2-L: road chance x1.5 (rounded down, capped at PCT_MAX) while supplies are 0 (const)
* `MAX_SCRIPTED`: int = 4 (const)
* `SCRIPTED_KEYS`: Array[String] = ["id", "encounter", "requires_flags", "unless_flags", "sets_flags"] (const)
* `SCRIPTED_FLAG_KEYS`: Array[String] = ["requires_flags", "unless_flags", "sets_flags"] (const)
* `LIST_FIELDS`: Array[String] = ["encounters", "elites", "scripted", "requires_flags", "unless_flags"] (const)
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `from_ref`: String = str(e.get("from", ""))
* `to_ref`: String = str(e.get("to", ""))
* `a`: Dictionary = WorldCatalog.resolve(world_id, "locations", from_ref)
* `b`: Dictionary = WorldCatalog.resolve(world_id, "locations", to_ref)
* `zone`: Variant = e.get("zone", "")
* `repeat`: Variant = e.get("repeat_pct", null)
* `errors`: Array[String] = []
* `list`: Array = raw
* `ids`: Dictionary = {}
* `s`: Dictionary = DictUtil.dict_of(JsonUtil.normalize(list[i]))
* `sid`: String = str(s.get("id", ""))
* `enc`: Variant = s.get("encounter", "")
* `out`: Array[String] = []
* `chain`: Array[String] = WorldCatalog.lineage(world_id)
* `rq`: String = str(e["_qid"])
* `to_hit`: Dictionary = WorldCatalog.resolve(world_id, "locations", str(e.get("to", "")))
* `to_world`: String = str(to_hit["_world"])
* `out`: Array[Dictionary] = []
* `seen`: Dictionary = {}
* `chain`: Array[String] = WorldCatalog.lineage(world_id)
* `out`: Array[String] = []
* `q`: String = str(r[key])
* `out`: Array[Dictionary] = []
* `hit`: Dictionary = {}
* `out`: Array[String] = []
* `none`: Array[String] = []
* `known_nodes`: Array[String] = known(gs)
* `all_legs`: Array[Dictionary] = legs(gs, gs.world_id)
* `best`: Dictionary = {from_qid: {"time": 0, "routes": none}}
* `done`: Dictionary = {}
* `cur`: String = ""
* `label`: Dictionary = best[cur]
* `nxt`: String = str(leg["to"])
* `route`: Dictionary = leg["route"]
* `routes`: Array[String] = JsonUtil.strings(label["routes"])
* `cand`: Dictionary = {"time": DictUtil.to_int(label["time"]) + DictUtil.to_int(route.get("time", 0)), "routes": routes}
* `out`: Array[String] = []
* `node`: String = str(row["node"])
* `out`: Array[String] = []
* `text`: String = str(row["text"])
* `out`: Array[Dictionary] = []
* `obj`: Dictionary = QuestRunner.current_objective(gs, qid)
* `steps`: Array = DictUtil.array_of(QuestRunner.resolved(gs, qid).get("steps", []))
* `index`: int = DictUtil.to_int(obj.get("index", -1))
* `step`: Dictionary = DictUtil.dict_of(steps[index]) if index >= 0 and index < steps.size() else {}
* `refs`: Array[String] = []
* `enc`: Dictionary = WorldCatalog.resolve(gs.world_id, "encounters", str(step.get("encounter", "")))
* `hit`: String = str(ExploreRunner.resolve_location(gs, ref).get("_qid", ""))

# Methods

### `static _int_in(v: Variant, lo: int, hi: int) -> bool`
```logic
return v is int and DictUtil.to_int(v) >= lo and DictUtil.to_int(v) <= hi
```

### `static _has_way_back(to_world: String, home_chain: Array[String]) -> bool`
```logic
for r: Dictionary in graph(to_world):
	var a_home: bool = home_chain.has(_world_of(str(r["_from"])))
	var b_home: bool = home_chain.has(_world_of(str(r["_to"])))
	if not a_home and b_home:
		return true
	if DictUtil.to_bool(r.get("two_way", true)) and a_home and not b_home:
		return true
return false
```

### `static _world_of(location_qid: String) -> String`
```logic
var parts: Array[String] = WorldCatalog.split_qid(location_qid)
return parts[0] if parts.size() == 2 else ""
```

### `static _add_route(world_id: String, r: Dictionary, out: Array[Dictionary], seen: Dictionary, needs_visible: bool) -> void`
```logic
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
```

### `static endpoint(world_id: String, route: Dictionary, key: String) -> String`
```logic
var home: String = str(route.get("_world", world_id))
var hit: Dictionary = WorldCatalog.resolve(home, "locations", str(route.get(key, "")))
if hit.is_empty():
	return ""
var seen: Dictionary = WorldCatalog.resolve(world_id, "locations", str(hit["_qid"]))
return str(seen.get("_qid", hit["_qid"]))
```

### `static _visible(world_id: String, location_qid: String) -> bool`
```logic
var w: String = _world_of(location_qid)
return not w.is_empty() and WorldCatalog.can_reach_world(world_id, w) and WorldCatalog.has_entry(world_id, "locations", location_qid)
```

### `static traversed_flag(route_qid: String) -> String`
```logic
return "%s.traversed" % MapRules.dotted(route_qid)
```

### `static event_done_flag(route_qid: String, event_id: String) -> String`
```logic
return "%s.%s.done" % [MapRules.dotted(route_qid), event_id]
```

### `static conditions_hold(gs: GameState, raw: Variant) -> bool`
```logic
for c: Variant in DictUtil.array_of(JsonUtil.normalize(raw)):
	if not QuestRules.condition_met(gs, DictUtil.dict_of(c)):
		return false
return true
```

### `static any_holds(gs: GameState, raw: Variant) -> bool`
```logic
for c: Variant in DictUtil.array_of(JsonUtil.normalize(raw)):
	if QuestRules.condition_met(gs, DictUtil.dict_of(c)):
		return true
return false
```

### `static is_open(gs: GameState, route: Dictionary) -> bool`
```logic
return conditions_hold(gs, route.get("requires_flags", [])) and not any_holds(gs, route.get("unless_flags", []))
```

### `static repeat_chance(route: Dictionary) -> int`
```logic
var repeat: int = DictUtil.to_int(route.get("repeat_pct", REPEAT_HALF))
return floori(DictUtil.to_int(route.get("encounter_pct", 0)) / 2.0) if repeat == REPEAT_HALF else repeat
```

### `static leg_chance(gs: GameState, route: Dictionary, starved: bool = false) -> int`
```logic
var chance: int = DictUtil.to_int(route.get("encounter_pct", 0))
if gs.get_flag(traversed_flag(str(route["_qid"])), false) == true:
	chance = repeat_chance(route)
return starved_chance(chance) if starved else chance
```

### `static starved_chance(chance: int) -> int`
```logic
return mini(PCT_MAX, maxi(chance, 0) * STARVED_PCT / 100)
```

### `static start_supplies(world_id: String) -> int`
```logic
var chain: Array[String] = WorldCatalog.lineage(world_id)
chain.reverse()
for w: String in chain:
	var v: int = DictUtil.to_int(WorldCatalog.manifest(w).get("start_supplies", -1), -1)
	if v >= 0:
		return v
return PartyState.SUPPLIES_START
```

### `static is_visited(gs: GameState, location_qid: String) -> bool`
```logic
return gs.get_flag(MapRules.visited_flag(location_qid), false) == true
```

### `static _better(a: Dictionary, b: Dictionary) -> bool`
```logic
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
```

### `static route_in(world_id: String, route_qid: String) -> Dictionary`
```logic
for r: Dictionary in graph(world_id):
	if str(r["_qid"]) == route_qid:
		return r
return {}
```

### `static trip(gs: GameState, from_qid: String, to_qid: String) -> Dictionary`
```logic
var routes: Array[String] = path(gs, from_qid, to_qid)
if routes.is_empty():
	return {"ok": false, "time": 0, "supplies": 0}
var time: int = 0
var spent: int = 0
var have: int = gs.party.supplies
for rq: String in routes:   # leg by leg, like TravelRunner.travel (PartyState.supplies_cost is the shared rule)
	var leg: int = DictUtil.to_int(route_in(gs.world_id, rq).get("time", 0))
	var paid: int = PartyState.supplies_cost(have, leg)
	have -= paid
	spent += paid
	time += leg
return {"ok": true, "time": time, "supplies": spent}
```

### `static view(gs: GameState) -> Dictionary`
```logic
var known_nodes: Array[String] = known(gs)
var node_list: Array = []
var quest_at: Array[String] = quest_nodes(gs)
for q: String in known_nodes:
	var loc: Dictionary = ExploreRunner.resolve_location(gs, q)
	var region_ref: String = str(loc.get("region", ""))
	var region: Dictionary = WorldCatalog.resolve(str(loc.get("_world", gs.world_id)), "regions", region_ref) if not region_ref.is_empty() else {}
	var pos: Array = DictUtil.array_of(loc.get("map_pos", []))
	node_list.append({"qid": q, "name": str(loc.get("name", q)), "visited": is_visited(gs, q), "kind": str(loc.get("kind", "")), "region": str(region.get("name", "")), "quest": quest_at.has(q), "map_pos": pos.duplicate()})
var route_list: Array = []
for r: Dictionary in graph(gs.world_id):
	var a: String = str(r["_from"])
	var b: String = str(r["_to"])
	if not known_nodes.has(a) or not known_nodes.has(b):
		continue
	route_list.append({"qid": str(r["_qid"]), "from": a, "to": b, "two_way": DictUtil.to_bool(r.get("two_way", true)), "open": is_open(gs, r), "time": DictUtil.to_int(r.get("time", 0))})
return {"current": gs.location_id, "supplies": gs.party.supplies, "nodes": node_list, "routes": route_list}
```

