class_name TravelTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Travel Test Suite (S10, spec TRAV1-L)
# Schema (checks 1-4): route validation (TravelRules.route_errors through WorldCatalog.validate_entry), soft warnings,
# the world_map exit node rule in MapRules, reference scans and inheritance of the world kind `routes`.
# Runtime (checks 5-17, TravelRuntimeChecks): graph, fog, path, trips, chances, scripted fights, determinism, outcomes,
# cross-world, menu, save v5. Scenes (WorldMapChecks): the debug world map and the region map hand-over.
# TRAV2-L (supplies, schema slice; checks T1-T5): PartyState.supplies, save v6, manifest start_supplies, shop
# supply_price, chest supplies. The save_v6.json fixture load and round trip live in SaveCompatTestSuite.
# Fixtures: hand-built worlds in an EMPTY sandbox world root and a sandbox map file; the shipped catalog is only read
# (a combat zone picked by rule), shipped data is never written.
# ==============================================================================

const TAG: String = "Travel"
static var SANDBOX: String = TestHelpers.sandbox("travel")
const W: String = "tv_base"     # fixture world
const FAR: String = "tv_far"    # connected by W (not inherited)
const LONE: String = "tv_lone"  # neither inherited nor connected by W
const KID: String = "tv_kid"    # extends W

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func _errs(kind: String, entry: Dictionary, world: String = W) -> Array[String]:
	return WorldCatalog.validate_entry(world, kind, entry, str(entry.get("id", "")))

# True when `errors` is exactly one message containing `needle` (prints the list otherwise, for debugging).
static func _one(errors: Array[String], needle: String) -> bool:
	var ok: bool = errors.size() == 1 and errors[0].contains(needle)
	if not ok:
		print("      expected one error with '%s', got: %s" % [needle, str(errors)])
	return ok

# A valid route of W: every list field used, so a single change shows exactly its own error.
static func _route(changes: Dictionary = {}) -> Dictionary:
	var r: Dictionary = {
		"id": "r_valid", "name": "Valid road", "from": "camp", "to": "ford", "two_way": true, "time": 3,
		"zone": CombatZones.zone_ids()[0], "encounter_pct": 20, "repeat_pct": -1, "elite_pct": 5,
		"encounters": [["enc_a", 3], ["enc_b", 1]], "elites": [["elite_a", 1]],
		"scripted": [{"id": "ambush", "encounter": "enc_b", "requires_flags": [{"key": "tv.go"}], "unless_flags": [], "sets_flags": [{"key": "tv.ambushed"}]}],
		"requires_flags": [{"key": "tv.open"}], "unless_flags": [{"key": "tv.shut", "value": true}],
	}
	r.merge(changes, true)
	return r

static func _with_scripted(entry: Dictionary) -> Dictionary:
	return _route({"scripted": [entry]})

# 10 x 10, open inside a wall ring; no zones; anchors spawn and a_wm.
static func _map() -> Dictionary:
	var rows: Array = ["##########"]
	var zones: Array = ["----------"]
	for _y: int in range(8):
		rows.append("#........#")
		zones.append("----------")
	rows.append("##########")
	zones.append("----------")
	return {"id": "m_tv", "name": "TV map", "rows": rows, "zones": zones, "zone_legend": {}, "paths": {},
		"anchors": [{"id": "spawn", "x": 1, "y": 1}, {"id": "a_wm", "x": 2, "y": 2}]}

static func _gate() -> Dictionary:
	return {"id": "gate", "name": "Gate", "map": "m_tv", "markers": [{"id": "to_map", "kind": "exit", "anchor": "a_wm", "to": MapRules.WORLD_MAP}]}

# ------------------------------------------------------------------------------

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING TRAVEL TEST (TRAV1-L)                   ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	MapCatalog.use_path(MapCatalog.DEFAULT_PATH)
	var ok: bool = true
	ok = _test_shipped() and ok
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/worlds")
	MapCatalog.use_path(SANDBOX + "/maps.json")
	WorldCatalog.use_root(SANDBOX + "/worlds")
	var sandboxed: bool = MapCatalog.file_path().begins_with("user://") and WorldCatalog.get_root().begins_with("user://") and WorldCatalog.world_ids().is_empty()
	ok = _report("Sandbox (empty map file and world root under user://)", {"sandboxed": sandboxed}) and ok
	if sandboxed:
		if _build_worlds():
			ok = _test_route_validation() and ok
			ok = _test_warnings() and ok
			ok = _test_world_map_exit() and ok
			ok = _test_refs_and_inheritance() and ok
			QuestCatalog.use_path(SANDBOX + "/quests.json")
			SaveGame.set_base_dir(SANDBOX + "/saves")
			ok = _test_supplies_schema() and ok   # TRAV2-L checks 1-5
			ok = TravelRuntimeChecks.run() and ok   # commit 2: checks 5-17
			ok = WorldMapChecks.run(tree_parent) and ok   # commit 3: debug world map scene, region map hand-over
			SaveGame.reset_base_dir()
			QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
		else:
			ok = false
	MapCatalog.use_path(MapCatalog.DEFAULT_PATH)
	TestHelpers.cleanup_worlds(SANDBOX)   # back to the shipped worlds; removes the whole sandbox
	return ok

# The shipped worlds still validate with the new kind (a missing routes.json loads as an empty kind).
static func _test_shipped() -> bool:
	var checks: Dictionary = {}
	checks["routes is a world kind"] = WorldSchema.KINDS.has("routes") and not WorldSchema.fields("routes").is_empty()
	checks["defaults"] = JsonUtil.deep_equal(WorldSchema.defaults("routes").get("two_way", null), true) and DictUtil.to_int(WorldSchema.defaults("routes").get("time", 0)) == 3 and DictUtil.to_int(WorldSchema.defaults("routes").get("repeat_pct", 0)) == TravelRules.REPEAT_HALF
	checks["shipped worlds validate"] = WorldCatalog.validate_all().is_empty()
	return _report("Shipped worlds validate with the routes kind", checks)

static func _build_worlds() -> bool:
	var errors: Array[String] = []
	errors.append_array(MapCatalog.upsert(_map()))
	errors.append_array(WorldCatalog.create_world(FAR, "TV Far"))
	errors.append_array(WorldCatalog.create_world(LONE, "TV Lone"))
	errors.append_array(WorldCatalog.create_world(W, "TV Base"))
	errors.append_array(WorldCatalog.create_world(KID, "TV Kid", [W]))
	errors.append_array(WorldCatalog.update_manifest(W, {"connects": [FAR]}))
	for id: String in ["enc_a", "enc_b", "elite_a"]:
		errors.append_array(WorldCatalog.upsert(W, "encounters", {"id": id, "name": id}))
	for id: String in ["camp", "ford", "hill", "mill"]:
		errors.append_array(WorldCatalog.upsert(W, "locations", {"id": id, "name": id.capitalize()}))
	errors.append_array(WorldCatalog.upsert(W, "locations", _gate()))
	errors.append_array(WorldCatalog.upsert(FAR, "locations", {"id": "dock", "name": "Dock"}))
	errors.append_array(WorldCatalog.upsert(FAR, "locations", {"id": "pier", "name": "Pier"}))
	errors.append_array(WorldCatalog.upsert(LONE, "locations", {"id": "camp", "name": "Lone Camp"}))
	if not errors.is_empty():
		print("      fixture errors: %s" % str(errors))
	return _report("Fixture worlds (base, connected, unrelated, child) and a map built through upsert", {"no errors": errors.is_empty(), "lineage": WorldCatalog.lineage(KID) == [W, KID]})

# ------------------------------------------------------------------------------
# Check 1: route validation
# ------------------------------------------------------------------------------

static func _test_route_validation() -> bool:
	var checks: Dictionary = {}
	var valid: Array[String] = _errs("routes", _route())
	if not valid.is_empty():
		print("      valid route errors: %s" % str(valid))
	checks["a valid route has no error"] = valid.is_empty()
	checks["defaults + endpoints only is valid"] = _errs("routes", {"id": "r_min", "name": "Min", "from": "camp", "to": "ford"}).is_empty()
	checks["qualified to into a connected world is valid"] = _errs("routes", _route({"to": "%s:dock" % FAR})).is_empty()
	checks["unknown from"] = _one(_errs("routes", _route({"from": "nowhere"})), "unknown location 'nowhere'")
	checks["unknown to"] = _one(_errs("routes", _route({"to": "nowhere"})), "unknown location 'nowhere'")
	checks["from missing"] = _one(_errs("routes", _route({"from": ""})), "Choose where the route starts.")
	checks["from == to"] = _one(_errs("routes", _route({"to": "camp"})), "A route can't lead to its own start.")
	checks["from == to (qualified)"] = _one(_errs("routes", _route({"to": "%s:camp" % W})), "A route can't lead to its own start.")
	checks["qualified to outside connects"] = _one(_errs("routes", _route({"to": "%s:camp" % LONE})), "neither an ancestor nor connected")
	checks["time below range"] = _one(_errs("routes", _route({"time": TravelRules.TIME_MIN - 1})), "Travel time must be")
	checks["time above range"] = _one(_errs("routes", _route({"time": TravelRules.TIME_MAX + 1})), "Travel time must be")
	checks["time edges ok"] = _errs("routes", _route({"time": TravelRules.TIME_MIN})).is_empty() and _errs("routes", _route({"time": TravelRules.TIME_MAX})).is_empty()
	checks["encounter_pct above 100"] = _one(_errs("routes", _route({"encounter_pct": TravelRules.PCT_MAX + 1})), "Encounter chance must be")
	checks["repeat_pct -2"] = _one(_errs("routes", _route({"repeat_pct": TravelRules.REPEAT_HALF - 1})), "Repeat chance must be")
	checks["repeat_pct 0 and 100 ok"] = _errs("routes", _route({"repeat_pct": 0})).is_empty() and _errs("routes", _route({"repeat_pct": TravelRules.PCT_MAX})).is_empty()
	checks["elite_pct above 100"] = _one(_errs("routes", _route({"elite_pct": TravelRules.PCT_MAX + 1})), "Elite chance must be")
	checks["weight 0"] = _one(_errs("routes", _route({"encounters": [["enc_a", MapRules.WEIGHT_MIN - 1]]})), "weight of 'enc_a'")
	checks["duplicate encounter ref"] = _one(_errs("routes", _route({"elites": [["elite_a", 1], ["elite_a", 2]]})), "lists 'elite_a' twice")
	checks["unknown encounter ref"] = _one(_errs("routes", _route({"encounters": [["enc_zz", 1]]})), "unknown encounter 'enc_zz'")
	checks["unknown zone"] = _one(_errs("routes", _route({"zone": "zz_nowhere"})), "Unknown zone 'zz_nowhere'.")
	checks["empty zone ok"] = _errs("routes", _route({"zone": ""})).is_empty()
	checks["scripted without encounter"] = _one(_errs("routes", _with_scripted({"id": "s1"})), "Scripted fight 's1': choose an encounter.")
	var dup: Array = [{"id": "s1", "encounter": "enc_a"}, {"id": "s1", "encounter": "enc_b"}]
	checks["duplicate scripted id"] = _one(_errs("routes", _route({"scripted": dup})), "Scripted fight id 's1' is used twice.")
	checks["unknown scripted key"] = _one(_errs("routes", _with_scripted({"id": "s1", "encounter": "enc_a", "bogus": 1})), "unknown key 'bogus'")
	checks["unknown scripted encounter"] = _one(_errs("routes", _with_scripted({"id": "s1", "encounter": "enc_zz"})), "unknown encounter 'enc_zz'")
	checks["bad scripted flag"] = _one(_errs("routes", _with_scripted({"id": "s1", "encounter": "enc_a", "sets_flags": [{"key": "Bad Key"}]})), "flag key")
	var five: Array = []
	for i: int in range(TravelRules.MAX_SCRIPTED + 1):
		five.append({"id": "s%d" % i, "encounter": "enc_a"})
	checks["too many scripted"] = _one(_errs("routes", _route({"scripted": five})), "At most %d scripted fights." % TravelRules.MAX_SCRIPTED)
	checks["bad route condition"] = _one(_errs("routes", _route({"unless_flags": [{"key": "tv.x", "cmp": "gt"}]})), "unknown key 'cmp'")   # REP1-L made op a condition key
	checks["wrong-typed list: blocking"] = not WorldCatalog.blocking_errors(W, "routes", _route({"scripted": {}})).is_empty() and WorldCatalog.blocking_errors(W, "routes", _route()).is_empty()
	return _report("Check 1: route validation (one error per fault; a valid route has none)", checks)

# ------------------------------------------------------------------------------
# Check 2: warnings
# ------------------------------------------------------------------------------

static func _test_warnings() -> bool:
	var checks: Dictionary = {}
	var errors: Array[String] = []
	errors.append_array(WorldCatalog.upsert(W, "routes", {"id": "r_quiet", "name": "Quiet", "from": "camp", "to": "hill", "encounter_pct": 20}))
	errors.append_array(WorldCatalog.upsert(W, "routes", {"id": "r_calm", "name": "Calm", "from": "hill", "to": "mill", "encounter_pct": 0}))
	errors.append_array(WorldCatalog.upsert(W, "routes", {"id": "r_oneway", "name": "One way", "from": "camp", "to": "%s:dock" % FAR, "two_way": false, "encounter_pct": 0}))
	var warns: Array[String] = TravelRules.warnings(W)
	checks["fixtures stored"] = errors.is_empty()
	checks["chance with no encounters"] = TestHelpers.any_contains(warns, "%s:r_quiet: encounter chance 20%%" % W)
	checks["chance 0 with no encounters: quiet"] = not TestHelpers.any_contains(warns, "%s:r_calm" % W)
	checks["one-way cross-world with no way back"] = TestHelpers.any_contains(warns, "%s:r_oneway: leads into world '%s' with no route back." % [W, FAR])
	checks["exactly those two"] = warns.size() == 2
	# A route of the far world leading back (the far world connects back to W for it) clears the one-way warning.
	errors.append_array(WorldCatalog.update_manifest(FAR, {"connects": [W]}))
	errors.append_array(WorldCatalog.upsert(FAR, "routes", {"id": "r_home", "name": "Home", "from": "dock", "to": "%s:camp" % W, "two_way": false, "encounter_pct": 0}))
	checks["a far route back clears it"] = errors.is_empty() and not TestHelpers.any_contains(TravelRules.warnings(W), "r_oneway")
	var route_errs: Array[String] = []
	for w: String in [W, FAR]:
		for e: Dictionary in WorldCatalog.own_entries(w, "routes"):
			route_errs.append_array(_errs("routes", e, w))
	checks["stored routes validate"] = route_errs.is_empty()
	if not route_errs.is_empty():
		print("      %s" % str(route_errs))
	# A two-way cross-world route has no warning, and is itself a way back from the far world (world level).
	checks["far route removed"] = WorldCatalog.remove(FAR, "routes", "r_home").is_empty() and TestHelpers.any_contains(TravelRules.warnings(W), "r_oneway")
	checks["bridge stored"] = WorldCatalog.upsert(W, "routes", {"id": "r_bridge", "name": "Bridge", "from": "mill", "to": "%s:pier" % FAR, "encounter_pct": 0}).is_empty()
	var after: Array[String] = TravelRules.warnings(W)
	checks["two-way cross-world: no warning"] = not TestHelpers.any_contains(after, "%s:r_bridge" % W)
	checks["the bridge is a way back for the one-way route too"] = not TestHelpers.any_contains(after, "r_oneway") and after.size() == 1
	return _report("Check 2: warnings (chance without encounters, cross-world route without a way back)", checks)

# ------------------------------------------------------------------------------
# Check 3: a world_map exit needs a route node
# ------------------------------------------------------------------------------

static func _test_world_map_exit() -> bool:
	var checks: Dictionary = {}
	var gate: Dictionary = WorldCatalog.get_own(W, "locations", "gate")
	checks["not a node yet"] = not TravelRules.nodes(W).has("%s:gate" % W)
	checks["world_map exit without a route"] = _one(_errs("locations", gate), "'gate' has a world map exit but no route.")
	var no_exit: Dictionary = gate.duplicate(true)
	no_exit["markers"] = []
	checks["no world_map exit: no error"] = _errs("locations", no_exit).is_empty()
	checks["route added"] = WorldCatalog.upsert(W, "routes", {"id": "r_gate", "name": "Gate road", "from": "ford", "to": "gate", "encounter_pct": 0}).is_empty()
	checks["the route clears it"] = TravelRules.nodes(W).has("%s:gate" % W) and _errs("locations", gate).is_empty()
	return _report("Check 3: a location with a world_map exit must be a route node", checks)

# ------------------------------------------------------------------------------
# Check 4: reference scans, inheritance and override
# ------------------------------------------------------------------------------

static func _route_keys(matches: Array[Dictionary], id: String) -> Array[String]:
	var out: Array[String] = []
	for m: Dictionary in matches:
		if str(m["kind"]) == "routes" and str(m["id"]) == id:
			out.append(str(m["key"]))
	out.sort()
	return out

static func _test_refs_and_inheritance() -> bool:
	var checks: Dictionary = {}
	var errors: Array[String] = WorldCatalog.upsert(W, "routes", _route({"id": "r_scan", "requires_flags": [], "unless_flags": []}))
	checks["route stored"] = errors.is_empty()
	checks["encounter refs found"] = _route_keys(WorldCatalog.references_to(W, "encounters", "enc_b"), "r_scan") == ["encounters", "scripted"]
	checks["location refs found"] = _route_keys(WorldCatalog.references_to(W, "locations", "camp"), "r_scan") == ["from"]
	var enc: Dictionary = WorldCatalog.get_own(W, "encounters", "enc_b")
	enc["id"] = "enc_b2"
	checks["rename encounter"] = WorldCatalog.upsert(W, "encounters", enc, "enc_b").is_empty()
	var loc: Dictionary = WorldCatalog.get_own(W, "locations", "camp")
	loc["id"] = "camp2"
	checks["rename location"] = WorldCatalog.upsert(W, "locations", loc, "camp").is_empty()
	var r: Dictionary = WorldCatalog.get_own(W, "routes", "r_scan")
	var pairs: Array = DictUtil.array_of(r.get("encounters", []))
	checks["encounters pair rewritten"] = pairs.size() == 2 and str(DictUtil.array_of(pairs[1])[0]) == "enc_b2" and str(DictUtil.array_of(pairs[0])[0]) == "enc_a"
	checks["scripted encounter rewritten"] = str(DictUtil.dict_of(DictUtil.array_of(r.get("scripted", []))[0]).get("encounter", "")) == "enc_b2"
	checks["from rewritten"] = str(r.get("from", "")) == "camp2"
	checks["still valid"] = _errs("routes", r).is_empty()
	checks["referenced encounter can't be removed"] = not WorldCatalog.remove(W, "encounters", "enc_b2").is_empty()
	# The child inherits the parent route and can override it.
	var kid_ids: Array[String] = []
	for e: Dictionary in WorldCatalog.merged_entries(KID, "routes"):
		kid_ids.append(str(e["_qid"]))
	checks["child inherits"] = kid_ids.has("%s:r_scan" % W)
	var over: Dictionary = WorldCatalog.get_own(W, "routes", "r_scan")
	over["id"] = "r_scan_kid"
	over["override"] = "%s:r_scan" % W
	over["time"] = TravelRules.TIME_MAX
	checks["override stored"] = WorldCatalog.upsert(KID, "routes", over).is_empty() and _errs("routes", over, KID).is_empty()
	var kid_graph: Dictionary = {}
	for g: Dictionary in TravelRules.graph(KID):
		kid_graph[str(g["_qid"])] = DictUtil.to_int(g["time"])
	var base_graph: Dictionary = {}
	for g: Dictionary in TravelRules.graph(W):
		base_graph[str(g["_qid"])] = DictUtil.to_int(g["time"])
	checks["override replaces it in the child"] = not kid_graph.has("%s:r_scan" % W) and DictUtil.to_int(kid_graph.get("%s:r_scan_kid" % KID, 0)) == TravelRules.TIME_MAX
	checks["parent unchanged"] = base_graph.has("%s:r_scan" % W) and not base_graph.has("%s:r_scan_kid" % KID)
	checks["child routes never leak into the parent graph"] = not _graph_has_world(W, KID)
	checks["files written and reloaded"] = WorldCatalog.save_all() == OK and FileAccess.file_exists(WorldCatalog.path_for(W, "routes")) and _reload_equal(r)
	return _report("Check 4: ref scans (encounters, scripted, endpoints) and child inheritance / override", checks)

static func _graph_has_world(world_id: String, other: String) -> bool:
	for g: Dictionary in TravelRules.graph(world_id):
		if str(g["_world"]) == other:
			return true
	return false

static func _reload_equal(before: Dictionary) -> bool:
	WorldCatalog.reload_from_disk()
	return JsonUtil.deep_equal(WorldCatalog.get_own(W, "routes", "r_scan"), before)

# ------------------------------------------------------------------------------
# TRAV2-L checks 1-5: supplies schema (party counter, save v6, manifest, shop, chest)
# ------------------------------------------------------------------------------

static func _supplies_errors(p: PartyState) -> Array[String]:
	var out: Array[String] = []
	for e: String in p.validate():
		if e.contains("supplies"):
			out.append(e)
	return out

static func _party_at(n: int) -> PartyState:
	var p := PartyState.new()
	p.supplies = n
	return p

static func _test_supplies_schema() -> bool:
	var ok: bool = true
	var unit: String = TravelRuntimeChecks._pick_unit("party")
	ok = _report("TRAV2 fixture: a party unit picked by rule", {"picked": not unit.is_empty()}) and ok
	if unit.is_empty():
		return false
	ok = _test_supplies_party() and ok
	ok = _test_supplies_save(unit) and ok
	ok = _test_supplies_manifest(unit) and ok
	ok = _test_supplies_shop(unit) and ok
	ok = _test_supplies_chest() and ok
	return ok

static func _test_supplies_party() -> bool:
	var checks: Dictionary = {}
	checks["-1 fails"] = _one(_supplies_errors(_party_at(-1)), "supplies must be 0..%d." % PartyState.SUPPLIES_MAX)
	checks["100 fails"] = _one(_supplies_errors(_party_at(PartyState.SUPPLIES_MAX + 1)), "supplies must be 0..")
	checks["0 and 99 pass"] = _supplies_errors(_party_at(0)).is_empty() and _supplies_errors(_party_at(PartyState.SUPPLIES_MAX)).is_empty()
	var loaded: PartyState = PartyState.from_dict({"members": [], "supplies": "ten"})
	checks["a String loads as -1 and fails"] = loaded.supplies == -1 and _supplies_errors(loaded).size() == 1
	checks["missing loads as 0"] = PartyState.from_dict({"members": []}).supplies == 0
	checks["to_dict key"] = DictUtil.to_int(_party_at(12).to_dict().get("supplies", -1)) == 12 and PartyState.from_dict(_party_at(12).to_dict()).supplies == 12
	var p: PartyState = _party_at(60)
	var added: int = p.add_supplies(50)
	checks["add 50 from 60: returns 39, leaves 99"] = added == PartyState.SUPPLIES_MAX - 60 and p.supplies == PartyState.SUPPLIES_MAX
	var q: PartyState = _party_at(3)
	var spent: int = q.spend_supplies(5)
	checks["spend 5 from 3: returns 3, leaves 0"] = spent == 3 and q.supplies == 0
	var r: PartyState = _party_at(20)
	checks["negative arguments change nothing"] = r.add_supplies(-5) == 0 and r.spend_supplies(-5) == 0 and r.supplies == 20
	return _report("TRAV2 check 1: party supplies range, load rule, add / spend clamps", checks)

static func _test_supplies_save(unit: String) -> bool:
	var checks: Dictionary = {}
	checks["version 6"] = SaveGame.VERSION >= 6
	var v5_in: Dictionary = {"world_id": "w", "party": {"members": [], "gold": 7}, "shops": {}, "time": 3}
	var v5: Dictionary = SaveMigrations.migrate({"version": 5, "game_state": v5_in}, 6)
	var expect: Dictionary = v5_in.duplicate(true)
	DictUtil.dict_of(expect["party"])["supplies"] = PartyState.SUPPLIES_START
	checks["v5 -> v6 adds party.supplies only"] = DictUtil.to_bool(v5["ok"]) and JsonUtil.deep_equal(DictUtil.dict_of(v5["data"]).get("game_state", {}), expect)
	var no_party: Dictionary = SaveMigrations.migrate({"version": 5, "game_state": {"world_id": "w", "party": 3}}, 6)
	checks["a non-object party is left alone"] = DictUtil.to_bool(no_party["ok"]) and DictUtil.dict_of(DictUtil.dict_of(no_party["data"])["game_state"]).get("party", null) == 3
	var gs: GameState = GameState.of(NewGame.create(W, [unit] as Array[String], 77)["state"])
	checks["state built"] = gs != null
	if gs != null:
		gs.party.supplies = PartyState.SUPPLIES_MAX + 1
		checks["write refuses supplies 100"] = TestHelpers.any_contains(SaveGame.write(gs, 4), "supplies must be 0..")
		gs.party.supplies = 6
		var wrote: bool = SaveGame.write(gs, 4).is_empty()
		var back: GameState = GameState.of(SaveGame.read(4)["state"]) if wrote else null
		checks["supplies round-trip through a slot"] = back != null and back.party.supplies == 6 and DictUtil.to_int(SaveGame.peek(4).get("version", 0)) == SaveGame.VERSION
		var _gone: bool = SaveGame.delete(4)
	return _report("TRAV2 check 2: save v6 (v5 migrates to supplies %d, write refuses 100)" % PartyState.SUPPLIES_START, checks)

static func _test_supplies_manifest(unit: String) -> bool:
	var checks: Dictionary = {}
	checks["manifest default -1"] = DictUtil.to_int(WorldCatalog.manifest(W).get("start_supplies", 0)) == -1
	checks["fixture manifest valid"] = WorldCatalog.manifest_errors(W).is_empty()
	var msg: String = "Start supplies must be -1 (inherited) or 0..99."
	for bad: int in [-2, PartyState.SUPPLIES_MAX + 1]:
		WorldCatalog.update_manifest(W, {"start_supplies": bad})
		checks["start_supplies %d: one error" % bad] = _one(WorldCatalog.manifest_errors(W), msg)
	# A non-whole value never reaches the stored manifest: the int field coerces it (WorldSchema.normalize), so the
	# validator's type guard covers raw callers only; the stored value is a whole number again.
	WorldCatalog.update_manifest(W, {"start_supplies": "ten"})
	checks["start_supplies 'ten' is coerced to a whole number"] = WorldCatalog.manifest(W).get("start_supplies", null) is int
	checks["0 valid"] = WorldCatalog.update_manifest(W, {"start_supplies": 0}).is_empty() and WorldCatalog.manifest_errors(W).is_empty()
	checks["99 valid"] = WorldCatalog.update_manifest(W, {"start_supplies": PartyState.SUPPLIES_MAX}).is_empty() and WorldCatalog.manifest_errors(W).is_empty()
	checks["parent 4 stored"] = WorldCatalog.update_manifest(W, {"start_supplies": 4}).is_empty()
	checks["child -1 inherits the parent's 4"] = DictUtil.to_int(WorldCatalog.manifest(KID).get("start_supplies", 0)) == -1 and TravelRules.start_supplies(KID) == 4 and TravelRules.start_supplies(W) == 4
	checks["no value anywhere: SUPPLIES_START"] = TravelRules.start_supplies(LONE) == PartyState.SUPPLIES_START and PartyState.SUPPLIES_START == 10
	var kid: GameState = GameState.of(NewGame.create(KID, [unit] as Array[String], 77)["state"])
	var lone: GameState = GameState.of(NewGame.create(LONE, [unit] as Array[String], 77)["state"])
	checks["NewGame.create uses the resolved value"] = kid != null and kid.party.supplies == 4 and lone != null and lone.party.supplies == PartyState.SUPPLIES_START
	checks["reset"] = WorldCatalog.update_manifest(W, {"start_supplies": -1}).is_empty() and TravelRules.start_supplies(KID) == PartyState.SUPPLIES_START
	return _report("TRAV2 check 3: manifest start_supplies validation, lineage resolver, NewGame", checks)

static func _test_supplies_shop(unit: String) -> bool:
	var checks: Dictionary = {}
	checks["keeper stored"] = WorldCatalog.upsert(W, "characters", {"id": "tv_keeper", "name": "Keeper", "base_unit": unit}).is_empty()
	checks["schema default 0"] = DictUtil.to_int(WorldSchema.defaults("shops").get("supply_price", -1)) == 0
	var shop: Dictionary = {"id": "s_sup", "name": "Supply shop", "keeper": "tv_keeper"}
	var plain: Array[String] = _errs("shops", shop)
	if not plain.is_empty():
		print("      plain shop errors: %s" % str(plain))
	checks["a shop without the field validates"] = plain.is_empty()
	var msg: String = "Supply price must be 0 (not sold) or 1..%d." % ShopRules.SUPPLY_PRICE_MAX
	for bad: int in [-1, ShopRules.SUPPLY_PRICE_MAX + 1]:
		checks["supply_price %d: one error" % bad] = _one(_errs("shops", shop.merged({"supply_price": bad})), msg)
	# 2.5: the catalog path coerces the int field (2, valid); the raw rule refuses it.
	checks["supply_price 2.5 (raw): one error"] = _one(ShopRules.shop_errors(W, shop.merged({"supply_price": 2.5})), msg)
	checks["0 and 999 valid"] = _errs("shops", shop.merged({"supply_price": 0})).is_empty() and _errs("shops", shop.merged({"supply_price": ShopRules.SUPPLY_PRICE_MAX})).is_empty()
	checks["SUPPLY_PRICE_MAX 999"] = ShopRules.SUPPLY_PRICE_MAX == 999
	return _report("TRAV2 check 4: shop supply_price validation (0 = not sold)", checks)

static func _test_supplies_chest() -> bool:
	var checks: Dictionary = {}
	var tmpl: Dictionary = {"anchors": [{"id": MapRules.SPAWN, "x": 1, "y": 1}, {"id": "a_chest", "x": 2, "y": 2}], "paths": {}}
	var chest: Dictionary = {"id": "box", "kind": "chest", "anchor": "a_chest"}
	var msg: String = "Marker 'box': supplies must be a whole number 0..%d." % MapRules.CHEST_SUPPLIES_MAX
	checks["marker default 0"] = DictUtil.to_int(DictUtil.dict_of(MapRules.MARKER_KINDS["chest"]).get("supplies", -1)) == 0
	checks["supplies -1 errors"] = TestHelpers.any_contains(MapRules.markers_errors(W, {}, tmpl, [chest.merged({"supplies": -1})]), msg)
	checks["supplies 100 errors"] = _one(MapRules.markers_errors(W, {}, tmpl, [chest.merged({"supplies": MapRules.CHEST_SUPPLIES_MAX + 1})]), msg)
	checks["supplies-only chest valid"] = MapRules.markers_errors(W, {}, tmpl, [chest.merged({"supplies": 3})]).is_empty()
	checks["empty chest (supplies 0) still errors"] = _one(MapRules.markers_errors(W, {}, tmpl, [chest.merged({"supplies": 0})]), "Chest 'box' holds nothing.")
	return _report("TRAV2 check 5: chest supplies validation and the holds-nothing rule", checks)
