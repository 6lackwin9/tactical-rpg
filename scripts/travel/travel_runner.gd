class_name TravelRunner
extends RefCounted

# ==============================================================================
# Tactical RPG - Travel runner (S10, spec TRAV1-L 2.3-2.5): auto-pathed trips over the world map, leg by leg.
# Stateless: everything lives in the GameState (location_id = the current node, time, flags, rolls). Per leg: the clock
# advances and the leg's supplies are paid (TRAV2-L: `route.time`, clamped at 0, travel never refused), then a scripted road fight (first matching entry) or the road roll (GameRolls stream "travel"); a fight
# stops the trip and the caller plays it (BattleBridge.encounter_for(gs, fight.encounter, fight.zone)) and then calls
# after_fight. The remaining path is never stored: after a won fight the caller calls travel() again. Entry autosaves
# of one trip collapse into one `autosave` event at the end; none while a fight is pending (the runner does no I/O).
# ==============================================================================

const SOURCE_SCRIPTED: String = "scripted"
const SOURCE_ROAD: String = "road"

static func _refuse(message: String) -> Dictionary:
	var errors: Array[String] = [message]
	return {"ok": false, "errors": errors}

# The current location qid when it is a node of gs.world_id's graph, else "".
static func current_node(gs: GameState) -> String:
	return gs.location_id if TravelRules.nodes(gs.world_id).has(gs.location_id) else ""

# Travels towards `to_ref` (a location ref seen from gs.world_id). Refused (state unchanged) unless the party stands on
# a node and a path exists. Returns {ok, status: "fight" | "arrived", legs_done, time (spent by this call), events,
# fight (status fight) | location (status arrived), supplies_spent (this call), starved_legs (legs rolled at 0 supplies)}.
static func travel(gs: GameState, to_ref: String) -> Dictionary:
	var start: String = current_node(gs)
	if start.is_empty():
		return _refuse("The party is not at a world map node.")
	var target: Dictionary = ExploreRunner.resolve_location(gs, to_ref)
	if target.is_empty():
		return _refuse("Unknown location '%s'." % to_ref)
	var to_qid: String = str(target["_qid"])
	var route_qids: Array[String] = TravelRules.path(gs, start, to_qid)
	if route_qids.is_empty():
		return _refuse("No known way from '%s' to '%s'." % [start, to_qid])
	# The legs are fixed up front from the start world's graph (a leg may cross into a connected world).
	var plan: Array[Dictionary] = []
	var at: String = start
	for rq: String in route_qids:
		var route: Dictionary = TravelRules.route_in(gs.world_id, rq)
		var leg_to: String = str(route["_to"]) if str(route["_from"]) == at else str(route["_from"])
		plan.append({"route": route, "from": at, "to": leg_to})
		at = leg_to
	var events: Array[Dictionary] = []
	var spent: int = 0
	var supplies_spent: int = 0
	var starved_legs: int = 0
	var legs_done: int = 0
	for leg: Dictionary in plan:
		var route: Dictionary = leg["route"]
		var leg_time: int = DictUtil.to_int(route.get("time", 0))
		var starved: bool = gs.party.supplies == 0   # TRAV2-L 2.1: read before this leg is paid
		gs.time += leg_time
		spent += leg_time
		supplies_spent += gs.party.spend_supplies(leg_time)
		if starved:
			starved_legs += 1
		var fight: Dictionary = _leg_fight(gs, route, str(leg["from"]), str(leg["to"]), starved)
		if not fight.is_empty():
			var errors: Array[String] = []
			return {"ok": true, "errors": errors, "status": "fight", "fight": fight, "legs_done": legs_done, "time": spent, "events": events, "supplies_spent": supplies_spent, "starved_legs": starved_legs}
		var entered: Dictionary = complete_leg(gs, str(route["_qid"]), str(leg["to"]))
		if not DictUtil.to_bool(entered["ok"]):
			return {"ok": false, "errors": entered["errors"], "status": "stopped", "legs_done": legs_done, "time": spent, "events": events, "supplies_spent": supplies_spent, "starved_legs": starved_legs}
		events.append_array(_without_autosave(entered["events"]))
		legs_done += 1
	events.append({"kind": "autosave"})
	var ok_errors: Array[String] = []
	return {"ok": true, "errors": ok_errors, "status": "arrived", "location": gs.location_id, "legs_done": legs_done, "time": spent, "events": events, "supplies_spent": supplies_spent, "starved_legs": starved_legs}

static func _without_autosave(raw: Variant) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for e: Variant in DictUtil.array_of(raw):
		var d: Dictionary = DictUtil.dict_of(e)
		if str(d.get("kind", "")) != "autosave":
			out.append(d)
	return out

# The fight of this leg ({} = none): the first armed scripted entry (supplies ignored), else the road roll (x1.5 when
# `starved`, TRAV2-L 2.2).
static func _leg_fight(gs: GameState, route: Dictionary, from_qid: String, to_qid: String, starved: bool = false) -> Dictionary:
	var rq: String = str(route["_qid"])
	for x: Variant in DictUtil.array_of(route.get("scripted", [])):
		var s: Dictionary = DictUtil.dict_of(JsonUtil.normalize(x))
		var sid: String = str(s.get("id", ""))
		if gs.get_flag(TravelRules.event_done_flag(rq, sid), false) == true:
			continue
		if TravelRules.conditions_hold(gs, s.get("requires_flags", [])) and not TravelRules.any_holds(gs, s.get("unless_flags", [])):
			return _fight(gs, route, SOURCE_SCRIPTED, str(s.get("encounter", "")), sid, false, from_qid, to_qid)
	var normal: Array = DictUtil.array_of(route.get("encounters", []))
	var elites: Array = DictUtil.array_of(route.get("elites", []))
	var chance: int = TravelRules.leg_chance(gs, route, starved)
	if chance <= 0 or (normal.is_empty() and elites.is_empty()):
		return {}
	if not GameRolls.chance(gs, GameRolls.TRAVEL, chance):
		return {}
	# The EXP1 pick rule (EXP1-L 2.4) on the travel stream: both lists = chance(elite_pct) then pick; one = pick it.
	var elite: bool = false
	if not elites.is_empty() and not normal.is_empty():
		elite = GameRolls.chance(gs, GameRolls.TRAVEL, DictUtil.to_int(route.get("elite_pct", 0)))
	elif not elites.is_empty():
		elite = true
	var ref: String = GameRolls.pick(gs, GameRolls.TRAVEL, elites if elite else normal)
	if ref.is_empty():
		return {}
	return _fight(gs, route, SOURCE_ROAD, ref, "", elite, from_qid, to_qid)

# The encounter ref resolved where the route is written, then seen from gs.world_id (overrides apply).
static func _fight(gs: GameState, route: Dictionary, source: String, ref: String, event: String, elite: bool, from_qid: String, to_qid: String) -> Dictionary:
	var hit: Dictionary = WorldCatalog.resolve(str(route.get("_world", gs.world_id)), "encounters", ref)
	var enc_qid: String = str(hit.get("_qid", ref))
	var seen: Dictionary = WorldCatalog.resolve(gs.world_id, "encounters", enc_qid)
	return {
		"source": source, "encounter": str(seen.get("_qid", enc_qid)), "route": str(route["_qid"]), "event": event,
		"elite": elite, "from": from_qid, "to": to_qid, "zone": str(route.get("zone", "")),
	}

# TRAV1-L 2.4: the route's traversed flag, then entry at the leg's end node (spawn; world switch, visited, last_safe,
# quest refresh and reach hook). Returns the ExploreRunner.enter result.
static func complete_leg(gs: GameState, route_qid: String, to_qid: String) -> Dictionary:
	var _t: String = QuestRunner.set_flag(gs, TravelRules.traversed_flag(route_qid), true)
	return ExploreRunner.enter(gs, to_qid, MapRules.SPAWN)

# TRAV1-L 2.5: {ok, errors, events, xp, advanced}. won: BattleBridge.apply, a scripted event's done flag then its
# sets_flags, then the leg completes; fled: nothing (the party is still at the leg's start, the time stays spent);
# lost: the EXP1 wake rule at the leg's start node.
static func after_fight(gs: GameState, fight: Dictionary, result: String) -> Dictionary:
	var applied: Dictionary = BattleBridge.apply(gs, str(fight.get("encounter", "")), result)
	var errors: Array[String] = []
	var events: Array[Dictionary] = []
	var none: Array[String] = []
	if not DictUtil.to_bool(applied["ok"]):
		errors.assign(DictUtil.array_of(applied["errors"]))
		return {"ok": false, "errors": errors, "events": events, "xp": {}, "advanced": none}
	var rq: String = str(fight.get("route", ""))
	match result:
		"won":
			var event_id: String = str(fight.get("event", ""))
			if str(fight.get("source", "")) == SOURCE_SCRIPTED and not event_id.is_empty():
				var _done: String = QuestRunner.set_flag(gs, TravelRules.event_done_flag(rq, event_id), true)
				var route: Dictionary = TravelRules.route_in(gs.world_id, rq)
				for x: Variant in DictUtil.array_of(route.get("scripted", [])):
					var s: Dictionary = DictUtil.dict_of(JsonUtil.normalize(x))
					if str(s.get("id", "")) != event_id:
						continue
					var keys: Array[String] = []
					for c: Variant in DictUtil.array_of(s.get("sets_flags", [])):
						var cond: Dictionary = DictUtil.dict_of(c)
						if QuestRunner.set_flag(gs, str(cond.get("key", "")), cond.get("value", true)).is_empty():
							keys.append(str(cond.get("key", "")))
					if not keys.is_empty():
						events.append({"kind": "flags_set", "route": rq, "event": event_id, "flags": keys})
			var entered: Dictionary = complete_leg(gs, rq, str(fight.get("to", "")))
			if not DictUtil.to_bool(entered["ok"]):
				errors.assign(DictUtil.array_of(entered["errors"]))
				return {"ok": false, "errors": errors, "events": events, "xp": applied["xp"], "advanced": applied["advanced"]}
			events.append_array(DictUtil.array_of(entered["events"]))
		"lost":
			var woke: Dictionary = ExploreRunner.wake(gs, str(fight.get("from", "")))
			events.append_array(DictUtil.array_of(woke.get("events", [])))
	return {"ok": true, "errors": errors, "events": events, "xp": applied["xp"], "advanced": applied["advanced"]}
