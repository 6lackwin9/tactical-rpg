class_name TravelRuntimeChecks
extends RefCounted

# ==============================================================================
# Tactical RPG - Travel runtime checks (S10, spec TRAV1-L checks 5-17), run by TravelTestSuite inside its sandbox
# (empty world root, sandbox map / quest files and save folder). Hand-built worlds: R (runtime) and RF, connected both
# ways; every location is map-less (menu stops). Party and enemy units are picked from the catalog by rule.
# ==============================================================================

const TAG: String = "Travel"
const R: String = "tr_run"
const RF: String = "tr_far"
const SEED: int = 5151
const M01_TIME: int = 2
const M12_TIME: int = 3
const L_TIME: int = 2
static var _units: Dictionary = {}

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func _q(id: String, world: String = R) -> String:
	return WorldCatalog.qid(world, id)

static func _snap(gs: GameState) -> String:
	return JSON.stringify(gs.to_dict(), "", true)

static func _new_gs() -> GameState:
	var res: Dictionary = NewGame.create(R, [str(_units["party"]), str(_units["party"])] as Array[String], SEED)
	return GameState.of(res["state"])

# A fresh state entered at `loc` of R (visited, hooks ran).
static func _at(loc: String) -> GameState:
	var gs: GameState = _new_gs()
	var _entered: Dictionary = ExploreRunner.enter(gs, loc)
	return gs

static func _visit(gs: GameState, ids: Array[String]) -> void:
	for id: String in ids:
		var _e: String = gs.set_flag(MapRules.visited_flag(_q(id)), true)

static func _path(gs: GameState, a: String, b: String) -> Array[String]:
	return TravelRules.path(gs, _q(a), _q(b))

static func _qids(ids: Array[String]) -> Array[String]:
	var out: Array[String] = []
	for id: String in ids:
		out.append(_q(id))
	return out

static func _rolls(gs: GameState, stream: String) -> int:
	return DictUtil.to_int(gs.rolls.get(stream, 0))

static func _fight(res: Dictionary) -> Dictionary:
	return DictUtil.dict_of(res.get("fight", {}))

static func _count(res: Dictionary, kind: String) -> int:
	var n: int = 0
	for e: Variant in DictUtil.array_of(res.get("events", [])):
		if str(DictUtil.dict_of(e).get("kind", "")) == kind:
			n += 1
	return n

static func _write_raw(slot: int, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(SaveGame.base_dir())
	var f: FileAccess = FileAccess.open(SaveGame.path_for(slot), FileAccess.WRITE)
	f.store_string(text)
	f.close()

static func _pick_unit(side: String) -> String:
	for e: Dictionary in ContentCatalog.entries("units"):
		if str(e.get("side", "")) == side and ContentFactory.class_of(e) != null and (side != "party" or str(e.get("stat_mode", "derived")) == "derived"):
			return str(e["id"])
	return ""

# ------------------------------------------------------------------------------

static func run() -> bool:
	if not _build():
		return false
	var ok: bool = true
	ok = _test_graph() and ok
	ok = _test_fog() and ok
	ok = _test_path() and ok
	ok = _test_time_and_intermediate() and ok
	ok = _test_chances() and ok
	ok = _test_scripted() and ok
	ok = _test_determinism() and ok
	ok = _test_fled_lost() and ok
	ok = _test_won_mid_trip() and ok
	ok = _test_cross_world() and ok
	ok = _test_menu() and ok
	ok = _test_save_v5() and ok
	return ok

static func _route(id: String, from: String, to: String, extra: Dictionary = {}) -> Dictionary:
	var r: Dictionary = {"id": id, "name": id, "from": from, "to": to, "time": 1, "encounter_pct": 0}
	r.merge(extra, true)
	return r

static func _build() -> bool:
	_units = {"party": _pick_unit("party"), "enemy": _pick_unit("enemy")}
	var errors: Array[String] = []
	for key: Variant in _units:
		if str(_units[key]).is_empty():
			errors.append("no %s unit picked" % str(key))
	errors.append_array(QuestCatalog.upsert({"id": "t_reach", "name": "Reach", "objectives": [{"kind": "reach_location"}]}))
	errors.append_array(WorldCatalog.create_world(R, "TR Run"))
	errors.append_array(WorldCatalog.create_world(RF, "TR Far"))
	errors.append_array(WorldCatalog.update_manifest(R, {"connects": [RF]}))
	errors.append_array(WorldCatalog.update_manifest(RF, {"connects": [R]}))
	errors.append_array(WorldCatalog.upsert(R, "characters", {"id": "hero", "name": "Hero TR", "role": "party", "base_unit": str(_units["party"])}))
	errors.append_array(WorldCatalog.upsert(R, "characters", {"id": "foe", "name": "Foe TR", "role": "enemy", "base_unit": str(_units["enemy"])}))
	for id: String in ["enc_a", "enc_b", "enc_s", "elite_a"]:
		errors.append_array(WorldCatalog.upsert(R, "encounters", {"id": id, "name": id, "party": ["hero"], "enemies": ["foe"], "max_rounds": 2}))
	var locs: Array[String] = ["p0", "p1", "p2", "p3", "p4", "p5", "p6", "p7", "k1", "u2", "p8", "p9", "p10", "f0", "f1", "f2", "f3",
		"m0", "m1", "m2", "n0", "n1", "q0", "q1", "s0", "s1", "d0", "d1", "l0", "l1", "w0", "w1", "w2", "c0", "c1", "lonely"]
	for id: String in locs:
		errors.append_array(WorldCatalog.upsert(R, "locations", {"id": id, "name": id.capitalize()}))
	for id: String in ["dock", "pier"]:
		errors.append_array(WorldCatalog.upsert(RF, "locations", {"id": id, "name": id.capitalize()}))
	errors.append_array(WorldCatalog.upsert(R, "quests", {"id": "q_m1", "name": "Reach M1", "template": "t_reach", "auto_start": true, "steps": [{"text": "Go", "location": "m1"}]}))
	var zone: String = CombatZones.zone_ids()[0]
	var road: Dictionary = {"encounter_pct": 100, "encounters": [["enc_a", 1]]}
	var routes: Array[Dictionary] = [
		_route("r_p01", "p0", "p1", {"time": 2}), _route("r_p12", "p1", "p2", {"time": 2}), _route("r_p02", "p0", "p2", {"time": 5}),
		_route("r_p03", "p0", "p3", {"time": 4}), _route("r_p13", "p1", "p3", {"time": 2}),
		_route("r_p14", "p1", "p4", {"time": 2}), _route("r_p05", "p0", "p5", {"time": 2}), _route("r_p54", "p5", "p4", {"time": 2}),
		_route("r_p60", "p6", "p0", {"two_way": false}), _route("r_p07", "p0", "p7", {"requires_flags": [{"key": "tr.open7"}]}), _route("r_p17", "p1", "p7"),
		_route("r_pk", "p0", "k1"), _route("r_ku", "k1", "u2"), _route("r_u8", "u2", "p8", {"two_way": false}), _route("r_p08", "p0", "p8", {"time": 10}),
		_route("r_p910", "p9", "p10"),
		_route("r_f01", "f0", "f1"), _route("r_f02", "f0", "f2", {"requires_flags": [{"key": "tr.f2"}]}), _route("r_f13", "f1", "f3"),
		_route("r_m01", "m0", "m1", {"time": M01_TIME}), _route("r_m12", "m1", "m2", {"time": M12_TIME}),
		_route("r_n", "n0", "n1", road.merged({"repeat_pct": 0, "zone": zone})),
		_route("r_q", "q0", "q1", {"encounter_pct": 41, "repeat_pct": -1, "encounters": [["enc_a", 1]]}),
		_route("r_s", "s0", "s1", road.merged({"repeat_pct": 100, "scripted": [
			{"id": "never", "encounter": "enc_b", "requires_flags": [{"key": "tr.never"}]},
			{"id": "amb", "encounter": "enc_s", "requires_flags": [{"key": "tr.go"}], "sets_flags": [{"key": "tr.ambushed"}]},
		]})),
		_route("r_d", "d0", "d1", {"encounter_pct": 50, "encounters": [["enc_a", 2], ["enc_b", 1]], "elites": [["elite_a", 1]], "elite_pct": 30}),
		_route("r_l", "l0", "l1", road.merged({"time": L_TIME})),
		_route("r_w01", "w0", "w1", road.merged({"repeat_pct": 0})), _route("r_w12", "w1", "w2"),
		_route("r_cross", "c0", "%s:dock" % RF),
	]
	for r: Dictionary in routes:
		errors.append_array(WorldCatalog.upsert(R, "routes", r))
	errors.append_array(WorldCatalog.upsert(RF, "routes", _route("rf_two", "dock", "%s:c1" % R)))
	errors.append_array(WorldCatalog.upsert(RF, "routes", _route("rf_one", "pier", "%s:c1" % R, {"two_way": false})))
	var problems: Dictionary = WorldCatalog.validate_world(R)
	problems.merge(WorldCatalog.validate_world(RF))
	if not errors.is_empty() or not problems.is_empty():
		print("      runtime fixture errors: %s %s" % [str(errors), str(problems)])
	return _report("Runtime fixtures (worlds R and RF connected both ways, routes, quest, units picked by rule)", {"no errors": errors.is_empty(), "worlds validate": problems.is_empty()})

static func _graph_qids(world_id: String) -> Array[String]:
	var out: Array[String] = []
	for r: Dictionary in TravelRules.graph(world_id):
		out.append(str(r["_qid"]))
	return out

# ------------------------------------------------------------------------------
# Check 5: nodes and graph
# ------------------------------------------------------------------------------

static func _test_graph() -> bool:
	var checks: Dictionary = {}
	var endpoints: Array[String] = []
	for r: Dictionary in TravelRules.graph(R):
		for key: String in ["_from", "_to"]:
			if not endpoints.has(str(r[key])):
				endpoints.append(str(r[key]))
	checks["nodes are the graph endpoints, first-seen order"] = TravelRules.nodes(R) == endpoints
	checks["a location without a route is no node"] = not TravelRules.nodes(R).has(_q("lonely"))
	var g_r: Array[String] = _graph_qids(R)
	var g_f: Array[String] = _graph_qids(RF)
	checks["own routes in the owner's graph"] = g_r.has(_q("r_cross")) and g_f.has(_q("rf_two", RF)) and g_f.has(_q("rf_one", RF))
	checks["a two-way route owned by a connected world is in both graphs"] = g_r.has(_q("rf_two", RF)) and g_f.has(_q("r_cross"))
	checks["a one-way one only in its owner's"] = not g_r.has(_q("rf_one", RF))
	checks["nodes cross worlds"] = TravelRules.nodes(R).has(_q("dock", RF)) and TravelRules.nodes(RF).has(_q("c0")) and not TravelRules.nodes(R).has(_q("pier", RF))
	return _report("Check 5: nodes and graph (endpoints only, connected two-way routes in both graphs)", checks)

# ------------------------------------------------------------------------------
# Check 6: fog
# ------------------------------------------------------------------------------

static func _known_of(gs: GameState, ids: Array[String]) -> Array[String]:
	var out: Array[String] = []
	for q: String in TravelRules.known(gs):
		if _qids(ids).has(q):
			out.append(q)
	return out

static func _view_route(gs: GameState, id: String) -> Dictionary:
	for r: Variant in DictUtil.array_of(TravelRules.view(gs)["routes"]):
		if str(DictUtil.dict_of(r).get("qid", "")) == _q(id):
			return DictUtil.dict_of(r)
	return {}

static func _test_fog() -> bool:
	var checks: Dictionary = {}
	var fs: Array[String] = ["f0", "f1", "f2", "f3"]
	var gs: GameState = _at("f0")
	checks["at start: the visited node and its open neighbours"] = _known_of(gs, fs) == _qids(["f0", "f1"] as Array[String])
	checks["a closed route reveals nothing (view skips it)"] = _view_route(gs, "r_f02").is_empty() and not _view_route(gs, "r_f01").is_empty()
	var view: Dictionary = TravelRules.view(gs)
	var view_nodes: Array[String] = []
	for n: Variant in DictUtil.array_of(view["nodes"]):
		view_nodes.append(str(DictUtil.dict_of(n)["qid"]))
	checks["view: current and known nodes only"] = str(view["current"]) == _q("f0") and view_nodes == TravelRules.known(gs)
	var _f: String = gs.set_flag("tr.f2", true)
	checks["its flag reveals the neighbour"] = _known_of(gs, fs) == _qids(["f0", "f1", "f2"] as Array[String])
	var closed: GameState = _at("f0")
	_visit(closed, ["f2"] as Array[String])
	var drawn: Dictionary = _view_route(closed, "r_f02")
	checks["a closed route between known nodes is drawn closed"] = not drawn.is_empty() and not DictUtil.to_bool(drawn["open"])
	return _report("Check 6: fog (visited or one open route away; closed routes reveal nothing)", checks)

# ------------------------------------------------------------------------------
# Check 7: path
# ------------------------------------------------------------------------------

static func _test_path() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _at("p0")
	_visit(gs, ["p4", "p6", "p7", "p8", "p9"] as Array[String])
	checks["shortest time beats fewer legs"] = _path(gs, "p0", "p2") == _qids(["r_p01", "r_p12"] as Array[String])
	checks["a time tie picks fewer legs"] = _path(gs, "p0", "p3") == _qids(["r_p03"] as Array[String])
	checks["a full tie picks the lower route qid list"] = _path(gs, "p0", "p4") == _qids(["r_p01", "r_p14"] as Array[String])
	checks["one-way not used backwards"] = _path(gs, "p0", "p6").is_empty() and _path(gs, "p6", "p0") == _qids(["r_p60"] as Array[String])
	checks["closed routes are skipped"] = _path(gs, "p0", "p7") == _qids(["r_p01", "r_p17"] as Array[String])
	var _o: String = gs.set_flag("tr.open7", true)
	checks["an opened route is used"] = _path(gs, "p0", "p7") == _qids(["r_p07"] as Array[String])
	checks["unknown intermediate nodes are skipped"] = _path(gs, "p0", "p8") == _qids(["r_p08"] as Array[String])
	_visit(gs, ["k1"] as Array[String])
	checks["once known they are used"] = _path(gs, "p0", "p8") == _qids(["r_pk", "r_ku", "r_u8"] as Array[String])
	checks["unreachable = []"] = _path(gs, "p0", "p9").is_empty()
	checks["start = destination = []"] = _path(gs, "p0", "p0").is_empty()
	checks["unknown destination = []"] = _path(gs, "p0", "p10").is_empty()
	return _report("Check 7: path (time, then legs, then qid list; one-way, closed and unknown honoured)", checks)

# ------------------------------------------------------------------------------
# Check 8 + 12: time, refusals, intermediate nodes
# ------------------------------------------------------------------------------

static func _test_time_and_intermediate() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _at("m0")
	_visit(gs, ["m2"] as Array[String])
	var t0: int = gs.time
	var before: String = _snap(gs)
	var refused: Dictionary = TravelRunner.travel(gs, "lonely")
	checks["refused trip: nothing changes"] = not DictUtil.to_bool(refused["ok"]) and _snap(gs) == before
	var off: GameState = _at("lonely")
	var off_before: String = _snap(off)
	checks["refused off the graph"] = not DictUtil.to_bool(TravelRunner.travel(off, "m0")["ok"]) and _snap(off) == off_before
	checks["m1 not visited before"] = not TravelRules.is_visited(gs, _q("m1")) and QuestRunner.status(gs, "q_m1") != "ready"
	var res: Dictionary = TravelRunner.travel(gs, "m2")
	checks["arrived after two legs"] = DictUtil.to_bool(res["ok"]) and str(res["status"]) == "arrived" and DictUtil.to_int(res["legs_done"]) == 2 and gs.location_id == _q("m2") and str(res["location"]) == _q("m2")
	checks["time adds both legs (Check 8)"] = gs.time == t0 + M01_TIME + M12_TIME and DictUtil.to_int(res["time"]) == M01_TIME + M12_TIME
	checks["one autosave at the end"] = _count(res, "autosave") == 1 and str(DictUtil.dict_of(DictUtil.array_of(res["events"]).back()).get("kind", "")) == "autosave"
	checks["intermediate node visited (Check 12)"] = TravelRules.is_visited(gs, _q("m1")) and _count(res, "entered") == 2
	checks["quest waiting at the intermediate node advanced"] = QuestRunner.status(gs, "q_m1") == "ready"
	checks["legs traversed"] = gs.get_flag(TravelRules.traversed_flag(_q("r_m01")), false) == true and gs.get_flag(TravelRules.traversed_flag(_q("r_m12")), false) == true
	return _report("Checks 8 + 12: the clock, refused trips, intermediate nodes count as visits", checks)

# ------------------------------------------------------------------------------
# Check 9: chances
# ------------------------------------------------------------------------------

static func _test_chances() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _at("n0")
	var first: Dictionary = TravelRunner.travel(gs, "n1")
	var fight: Dictionary = _fight(first)
	checks["encounter_pct 100 fights on the first traversal"] = str(first.get("status", "")) == "fight" and str(fight.get("source", "")) == "road" and str(fight.get("encounter", "")) == _q("enc_a")
	checks["fight fields"] = str(fight.get("route", "")) == _q("r_n") and str(fight.get("from", "")) == _q("n0") and str(fight.get("to", "")) == _q("n1") and str(fight.get("zone", "")) == CombatZones.zone_ids()[0] and DictUtil.to_int(first["legs_done"]) == 0
	checks["no autosave while a fight is pending"] = _count(first, "autosave") == 0
	checks["BattleBridge builds the fight"] = DictUtil.to_bool(BattleBridge.encounter_for(gs, str(fight["encounter"]), str(fight["zone"]))["ok"])
	var won: Dictionary = TravelRunner.after_fight(gs, fight, "won")
	checks["won: leg completes"] = DictUtil.to_bool(won["ok"]) and gs.location_id == _q("n1")
	var rolls0: int = _rolls(gs, GameRolls.TRAVEL)
	var second: Dictionary = TravelRunner.travel(gs, "n0")
	checks["repeat_pct 0: the second traversal never fights (no roll)"] = str(second.get("status", "")) == "arrived" and _rolls(gs, GameRolls.TRAVEL) == rolls0
	var q: Dictionary = TravelRules.route_in(R, _q("r_q"))
	var fresh: GameState = _at("q0")
	var first_pct: int = TravelRules.leg_chance(fresh, q)
	var _t: String = fresh.set_flag(TravelRules.traversed_flag(_q("r_q")), true)
	checks["repeat_pct -1 with encounter_pct 41 uses 20"] = first_pct == 41 and TravelRules.leg_chance(fresh, q) == floori(first_pct / 2.0) and TravelRules.repeat_chance(q) == floori(first_pct / 2.0)
	return _report("Check 9: chances (first traversal, repeat_pct 0, repeat_pct -1 = half rounded down)", checks)

# ------------------------------------------------------------------------------
# Check 10: scripted
# ------------------------------------------------------------------------------

static func _test_scripted() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _at("s0")
	var _g: String = gs.set_flag("tr.go", true)
	var rolls0: int = _rolls(gs, GameRolls.TRAVEL)
	var res: Dictionary = TravelRunner.travel(gs, "s1")
	var fight: Dictionary = _fight(res)
	checks["first matching entry fires, before the roll"] = str(fight.get("source", "")) == "scripted" and str(fight.get("event", "")) == "amb" and str(fight.get("encounter", "")) == _q("enc_s") and _rolls(gs, GameRolls.TRAVEL) == rolls0
	var lost: Dictionary = TravelRunner.after_fight(gs, fight, "lost")
	var done_key: String = TravelRules.event_done_flag(_q("r_s"), "amb")
	checks["a loss leaves it armed"] = DictUtil.to_bool(lost["ok"]) and gs.get_flag(done_key, false) == false and gs.location_id == _q("s0")
	var again: Dictionary = TravelRunner.travel(gs, "s1")
	checks["armed: fires again"] = str(_fight(again).get("event", "")) == "amb"
	var won: Dictionary = TravelRunner.after_fight(gs, _fight(again), "won")
	checks["won: done flag and sets_flags set, leg completed"] = DictUtil.to_bool(won["ok"]) and gs.get_flag(done_key, false) == true and gs.get_flag("tr.ambushed", false) == true and gs.location_id == _q("s1") and gs.get_flag(TravelRules.traversed_flag(_q("r_s")), false) == true
	var next: Dictionary = TravelRunner.travel(gs, "s0")
	checks["the next traversal rolls normally (once)"] = str(_fight(next).get("source", "")) == "road" and str(_fight(next).get("encounter", "")) == _q("enc_a") and _rolls(gs, GameRolls.TRAVEL) > rolls0
	return _report("Check 10: scripted road fights (before the roll, once, a loss keeps them armed)", checks)

# ------------------------------------------------------------------------------
# Check 11: determinism
# ------------------------------------------------------------------------------

const SHUTTLE_LEGS: int = 8

# Travels d0 <-> d1 SHUTTLE_LEGS times (every fight won): [{status, fight}] per leg.
static func _shuttle(gs: GameState) -> Array:
	var out: Array = []
	for i: int in range(SHUTTLE_LEGS):
		var target: String = "d1" if gs.location_id == _q("d0") else "d0"
		var res: Dictionary = TravelRunner.travel(gs, target)
		out.append({"status": str(res.get("status", "")), "fight": _fight(res)})
		if str(res.get("status", "")) == "fight":
			var _w: Dictionary = TravelRunner.after_fight(gs, _fight(res), "won")
	return out

static func _test_determinism() -> bool:
	var checks: Dictionary = {}
	var a: GameState = _at("d0")
	var b: GameState = _at("d0")
	var saved: bool = SaveGame.write(a, 1).is_empty()
	var travel0: int = _rolls(a, GameRolls.TRAVEL)
	var explore0: int = _rolls(a, GameRolls.EXPLORE)
	var run_a: Array = _shuttle(a)
	var run_b: Array = _shuttle(b)
	checks["equal states give equal trips"] = JsonUtil.deep_equal(run_a, run_b) and a.equals(b)
	var loaded: GameState = GameState.of(SaveGame.read(1)["state"])
	checks["save, load and repeat give the same fights"] = saved and loaded != null and JsonUtil.deep_equal(_shuttle(loaded), run_a) and loaded.equals(a)
	var fights: int = 0
	for x: Variant in run_a:
		if str(DictUtil.dict_of(x)["status"]) == "fight":
			fights += 1
	print("      shuttle: %d legs, %d fights" % [SHUTTLE_LEGS, fights])
	# Every leg draws the chance roll; a road fight with both lists draws the elite chance and the pick on top.
	checks["rolls.travel rises by exactly the rolls drawn"] = _rolls(a, GameRolls.TRAVEL) - travel0 == SHUTTLE_LEGS + 2 * fights
	checks["rolls.explore unchanged"] = _rolls(a, GameRolls.EXPLORE) == explore0
	return _report("Check 11: determinism (equal states, save / load, roll counts per stream)", checks)

# ------------------------------------------------------------------------------
# Check 13 + 14: fled, lost, won mid-trip
# ------------------------------------------------------------------------------

static func _test_fled_lost() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _at("l0")
	var t0: int = gs.time
	var fight: Dictionary = _fight(TravelRunner.travel(gs, "l1"))
	var fled: Dictionary = TravelRunner.after_fight(gs, fight, "fled")
	checks["fled: at the leg's start, the time stays spent"] = DictUtil.to_bool(fled["ok"]) and gs.location_id == _q("l0") and gs.time == t0 + L_TIME
	checks["fled: the leg is not traversed"] = gs.get_flag(TravelRules.traversed_flag(_q("r_l")), false) == false
	var lost_gs: GameState = _at("l0")
	var lost_fight: Dictionary = _fight(TravelRunner.travel(lost_gs, "l1"))
	var copy: GameState = GameState.from_dict(lost_gs.to_dict())
	var lost: Dictionary = TravelRunner.after_fight(lost_gs, lost_fight, "lost")
	var _woke: Dictionary = ExploreRunner.wake(copy, str(lost_fight["from"]))
	checks["lost: the EXP1 wake rule from the leg's start"] = DictUtil.to_bool(lost["ok"]) and _snap(lost_gs) == _snap(copy) and lost_gs.location_id == _q("l0")
	checks["bad outcome refused"] = not DictUtil.to_bool(TravelRunner.after_fight(lost_gs, lost_fight, "draw")["ok"])
	return _report("Check 13: fled (stay, time spent) and lost (wake at the leg's start)", checks)

static func _test_won_mid_trip() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _at("w0")
	_visit(gs, ["w2"] as Array[String])
	var res: Dictionary = TravelRunner.travel(gs, "w2")
	checks["the first leg fights"] = str(res.get("status", "")) == "fight" and DictUtil.to_int(res["legs_done"]) == 0
	var won: Dictionary = TravelRunner.after_fight(gs, _fight(res), "won")
	checks["won: traversed and the end node entered"] = DictUtil.to_bool(won["ok"]) and gs.location_id == _q("w1") and gs.get_flag(TravelRules.traversed_flag(_q("r_w01")), false) == true and _count(won, "entered") == 1
	var rest: Dictionary = TravelRunner.travel(gs, "w2")
	checks["travel again finishes the trip"] = str(rest.get("status", "")) == "arrived" and DictUtil.to_int(rest["legs_done"]) == 1 and gs.location_id == _q("w2")
	return _report("Check 14: won mid-trip (the leg completes; travel again finishes)", checks)

# ------------------------------------------------------------------------------
# Check 15 - 17: cross-world, menu, save v5
# ------------------------------------------------------------------------------

static func _test_cross_world() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _at("c0")
	var there: Dictionary = TravelRunner.travel(gs, "%s:dock" % RF)
	checks["arriving in a connected world sets world_id"] = str(there.get("status", "")) == "arrived" and gs.world_id == RF and gs.location_id == _q("dock", RF)
	var back: Dictionary = TravelRunner.travel(gs, "%s:c0" % R)
	checks["back through the same two-way route restores it"] = str(back.get("status", "")) == "arrived" and gs.world_id == R and gs.location_id == _q("c0")
	return _report("Check 15: cross-world travel (world_id follows the arrival)", checks)

static func _test_menu() -> bool:
	var checks: Dictionary = {}
	var node: GameState = _at("m0")
	var entries: Array[String] = []
	for m: Dictionary in ExploreRunner.menu(node):
		entries.append(str(m["to"]))
	checks["a map-less node lists world_map"] = entries.has(MapRules.WORLD_MAP)
	checks["world_map entry hands over"] = str(DictUtil.dict_of(DictUtil.array_of(ExploreRunner.take_exit(node, MapRules.WORLD_MAP)["events"])[0]).get("to", "")) == MapRules.WORLD_MAP
	var lone: GameState = _at("lonely")
	checks["a map-less non-node does not"] = ExploreRunner.menu(lone).is_empty()
	return _report("Check 16: menu (world_map entry on map-less nodes only)", checks)

static func _test_save_v5() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _at("m0")
	var v4: Dictionary = gs.to_dict()
	v4.erase("time")
	_write_raw(2, JSON.stringify({"format": SaveGame.FORMAT, "version": 4, "saved_at": 0, "game_state": v4}))
	var read: Dictionary = SaveGame.read(2)
	var st: GameState = GameState.of(read["state"])
	checks["v4 migrates to time 0 and nothing else changes"] = DictUtil.to_bool(read["ok"]) and st != null and st.time == 0 and st.equals(gs) and JsonUtil.strings(read["warnings"]).is_empty()
	gs.time = 7
	checks["time round-trips"] = SaveGame.write(gs, 3).is_empty() and GameState.of(SaveGame.read(3)["state"]).time == 7 and SaveGame.VERSION == 5
	var neg: GameState = GameState.from_dict(gs.to_dict())
	neg.time = -1
	checks["negative time fails validate and write"] = TestHelpers.any_contains(neg.validate(), "time must be a whole number >= 0.") and not SaveGame.write(neg, 3).is_empty()
	var bad: Dictionary = gs.to_dict()
	bad["time"] = "noon"
	checks["a non-int time is a load problem"] = TestHelpers.any_contains(GameState.from_dict(bad).validate(), "time must be a whole number")
	return _report("Check 17: save v5 (v4 migrates to time 0; negative time refused)", checks)
