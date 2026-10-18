class_name PurseTestSuite
extends RefCounted

# ==============================================================================
# PURSE1-L: encounter purse (PurseRules).
# Stage 1 (schema slice): the `purse` field and its {} default in both schemas, old entries normalising to {},
# `PurseRules.read` defaults, every validation message of spec 3 in both catalogs (one purse message per bad
# entry), valid purses clean in both catalogs (world-local and qualified loot; catalog encounters take only
# qualified loot), the two soft warnings in both catalogs, storage, and shipped encounters clean. Fixture refs
# are picked by rule (side / role, first item and passive); loot fixtures and writes go to a sandbox world fork.
# Stage 2 (runtime, spec section 1 and 5): a sandbox world fork with test-only purse encounters and loot entries, played
# from new games through BattleBridge.apply: no purse = today, win only, the roll (pure, seeded, in range), once,
# loot (claimed once, pre-claimed and gated skipped), the gold cap, qualified ids, after-fight result keys and the
# results reward lines, and a morale break win paying like a rout. Save v10 is tested in save_test_suite.
# ==============================================================================

const TAG: String = "Purse"
const ITEM_LOOT: String = "purse_probe_item"
const PASSIVE_LOOT: String = "purse_probe_passive"
const MISSING_LOOT: String = "purse_probe_missing"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING PURSE TEST (PURSE1-L)                   ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_schema_defaults() and ok
	ok = _test_shipped_clean() and ok
	ok = _test_validation() and ok
	ok = _test_runtime() and ok
	return ok

# ------------------------------------------------------------------------------
# Fixtures
# ------------------------------------------------------------------------------

# {party, enemy}: a party and an enemy unit of the catalog ({} when missing).
static func _catalog_refs() -> Dictionary:
	var party: String = ""
	var enemy: String = ""
	for u: Dictionary in ContentCatalog.entries("units"):
		var side: String = str(u.get("side", ""))
		if side == "party" and party.is_empty():
			party = str(u["id"])
		elif side == "enemy" and enemy.is_empty():
			enemy = str(u["id"])
	if party.is_empty() or enemy.is_empty():
		return {}
	return {"party": party, "enemy": enemy}

# {world, party, enemy}: the first world with a party and an enemy character ({} when none).
static func _world_refs() -> Dictionary:
	for w: String in WorldCatalog.world_ids():
		var party: String = ""
		var enemy: String = ""
		for c: Dictionary in WorldCatalog.merged_entries(w, "characters"):
			var role: String = str(c.get("role", ""))
			if role == "party" and party.is_empty():
				party = WorldCatalog.ref_value(w, c)
			elif role == "enemy" and enemy.is_empty():
				enemy = WorldCatalog.ref_value(w, c)
		if not party.is_empty() and not enemy.is_empty():
			return {"world": w, "party": party, "enemy": enemy}
	return {}

static func _entry(refs: Dictionary, purse: Variant) -> Dictionary:
	var e: Dictionary = {
		"id": "purse_probe", "name": "Purse Probe", "zone": "stone_arena",
		"party": [refs["party"]], "enemies": [refs["enemy"]],
	}
	if purse != null:
		e["purse"] = purse
	return e

# Valid purses; `loot` is the local (world) or qualified (catalog) item loot ref.
static func _good_cases(loot: String) -> Dictionary:
	return {
		"no purse key": null,
		"empty purse": {},
		"gold_min only": {"gold_min": 10},
		"range": {"gold_min": 3, "gold_max": 6},
		"full range 0..500": {"gold_min": 0, "gold_max": PurseRules.GOLD_CAP},
		"whole floats (JSON)": {"gold_min": 4.0, "gold_max": 6.0},
		"item loot": {"gold_min": 12, "gold_max": 18, "loot": loot, "once": false},
		"loot only": {"loot": loot},
		"empty loot string": {"gold_min": 1, "loot": ""},
		"once true": {"gold_min": 15, "gold_max": 20, "once": true},
	}

# [purse, message]; `prefix` qualifies the loot refs ("" in a world, "<world>:" in the catalog).
static func _bad_cases(prefix: String) -> Dictionary:
	var cap: int = PurseRules.GOLD_CAP
	var min_msg: String = "Purse gold_min must be a whole number 0..%d." % cap
	var max_msg: String = "Purse gold_max must be gold_min..%d." % cap
	return {
		"not an object": [[1, 2], "Purse must be an object"],
		"unknown key": [{"gold_min": 1, "bonus": 2}, "Purse key 'bonus' is not allowed."],
		"gold_min below 0": [{"gold_min": -1}, min_msg],
		"gold_min above the cap": [{"gold_min": cap + 1}, min_msg],
		"gold_min not whole": [{"gold_min": 2.5}, min_msg],
		"gold_min a string": [{"gold_min": "5"}, min_msg],
		"gold_max below gold_min": [{"gold_min": 5, "gold_max": 4}, max_msg],
		"gold_max above the cap": [{"gold_min": 5, "gold_max": cap + 1}, max_msg],
		"gold_max not whole": [{"gold_min": 1, "gold_max": 1.5}, max_msg],
		"loot missing": [{"gold_min": 1, "loot": prefix + MISSING_LOOT}, "Purse loot '%s' does not resolve." % (prefix + MISSING_LOOT)],
		"loot not a string": [{"gold_min": 1, "loot": 7}, "Purse loot '7' must be an item id."],
		"loot a passive reward": [{"gold_min": 1, "loot": prefix + PASSIVE_LOOT}, "Purse loot '%s' must be an item reward." % (prefix + PASSIVE_LOOT)],
		"once not a bool": [{"gold_min": 1, "once": "yes"}, "Purse once must be true or false."],
	}

static func _matrix(refs: Dictionary, loot_prefix: String, scope: String, validate: Callable) -> Dictionary:
	var checks: Dictionary = {}
	var good: Dictionary = _good_cases(loot_prefix + ITEM_LOOT)
	for label: Variant in good:
		var errors: Array[String] = validate.call(_entry(refs, good[label]))
		checks["%s valid: %s (%s)" % [scope, str(label), "; ".join(errors)]] = errors.is_empty()
	var bad: Dictionary = _bad_cases(loot_prefix)
	for label: Variant in bad:
		var pair: Array = bad[label]
		var errors: Array[String] = validate.call(_entry(refs, pair[0]))
		checks["%s %s -> '%s' (%s)" % [scope, str(label), str(pair[1]), "; ".join(errors)]] = TestHelpers.any_contains(errors, str(pair[1]))
		var purse_errors: Array[String] = []
		for msg: String in errors:
			if msg.containsn("purse"):
				purse_errors.append(msg)
		checks["%s %s: one purse message only (%s)" % [scope, str(label), "; ".join(purse_errors)]] = purse_errors.size() == 1
	return checks

# ------------------------------------------------------------------------------
# Tests
# ------------------------------------------------------------------------------

static func _test_schema_defaults() -> bool:
	var checks: Dictionary = {}
	var cf: Dictionary = ContentSchema.field("encounters", "purse")
	var wf: Dictionary = WorldSchema.field("encounters", "purse")
	checks["catalog field purse, json, default {}"] = str(cf.get("type", "")) == "json" and str(cf.get("default", null)) == str({})
	checks["world field purse, json, default {}"] = str(wf.get("type", "")) == "json" and str(wf.get("default", null)) == str({})
	var old_catalog: Dictionary = ContentSchema.normalize("encounters", {"id": "old", "name": "Old"})
	var old_world: Dictionary = WorldSchema.normalize("encounters", {"id": "old", "name": "Old"})
	checks["old catalog entry: purse = %s" % str(old_catalog.get("purse", null))] = old_catalog.get("purse", null) is Dictionary and DictUtil.dict_of(old_catalog["purse"]).is_empty()
	checks["old world entry: purse = %s" % str(old_world.get("purse", null))] = old_world.get("purse", null) is Dictionary and DictUtil.dict_of(old_world["purse"]).is_empty()
	checks["old entry reads as no purse"] = PurseRules.read(old_catalog).is_empty() and PurseRules.read(old_world).is_empty()
	checks["old entry: no problems, no warnings"] = PurseRules.problems(old_catalog, func(_r: String) -> String: return "missing").is_empty() and PurseRules.warnings(old_world).is_empty()
	var stored: Dictionary = ContentSchema.to_storage("encounters", {"id": "p", "name": "P", "purse": {"gold_min": 3, "gold_max": 6}})
	checks["catalog storage keeps the purse"] = DictUtil.to_int(DictUtil.dict_of(stored.get("purse", {})).get("gold_max", -1)) == 6
	# Spec 3 defaults: gold_min 0, gold_max = gold_min, loot "", once true.
	var lo_only: Dictionary = PurseRules.read({"purse": {"gold_min": 5}})
	checks["read gold_min only (%s)" % str(lo_only)] = DictUtil.to_int(lo_only.get("gold_min", -1)) == 5 and DictUtil.to_int(lo_only.get("gold_max", -1)) == 5 and str(lo_only.get("loot", "x")).is_empty() and lo_only.get("once", false) == true
	var loot_only: Dictionary = PurseRules.read({"purse": {"loot": "a_loot"}})
	checks["read loot only (%s)" % str(loot_only)] = DictUtil.to_int(loot_only.get("gold_min", -1)) == 0 and DictUtil.to_int(loot_only.get("gold_max", -1)) == 0 and str(loot_only.get("loot", "")) == "a_loot"
	var farm: Dictionary = PurseRules.read({"purse": {"gold_min": 2, "gold_max": 4, "once": false}})
	checks["read once false"] = farm.get("once", true) == false and DictUtil.to_int(farm.get("gold_max", -1)) == 4
	checks["read: not a dictionary = no purse"] = PurseRules.read({"purse": [1]}).is_empty()
	# Soft warnings (rule-derived thresholds).
	var soft: int = PurseRules.GOLD_SOFT_CAP
	var soft_msg: String = "Purse gold above %d: check the budget." % soft
	checks["gold_max above the soft cap warns"] = PurseRules.warnings({"purse": {"gold_min": 1, "gold_max": soft + 1}}).has(soft_msg)
	checks["gold_min above the soft cap warns (gold_max defaults to it)"] = PurseRules.warnings({"purse": {"gold_min": soft + 1}}).has(soft_msg)
	checks["gold_max at the soft cap is quiet"] = PurseRules.warnings({"purse": {"gold_min": 1, "gold_max": soft}}).is_empty()
	checks["zero gold, no loot = Empty purse."] = PurseRules.warnings({"purse": {"gold_min": 0, "gold_max": 0}}).has("Empty purse.")
	checks["once only = Empty purse."] = PurseRules.warnings({"purse": {"once": false}}).has("Empty purse.")
	checks["zero gold with loot is quiet"] = PurseRules.warnings({"purse": {"loot": "a_loot"}}).is_empty()
	checks["{} is quiet"] = PurseRules.warnings({"purse": {}}).is_empty()
	return TestHelpers.report(TAG, "Field purse with default {} in both schemas; read defaults; soft warnings", checks)

# Shipped encounters pass the purse rules in both catalogs (the new field does not break existing data).
static func _test_shipped_clean() -> bool:
	var checks: Dictionary = {}
	for e: Dictionary in ContentCatalog.entries("encounters"):
		var errs: Array[String] = PurseRules.problems(ContentSchema.normalize("encounters", e), func(ref: String) -> String: return PurseRules.world_loot_check("", ref))
		checks["catalog %s (%s)" % [str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
	for w: String in WorldCatalog.world_ids():
		for e: Dictionary in WorldCatalog.own_entries(w, "encounters"):
			var errs: Array[String] = PurseRules.problems(WorldSchema.normalize("encounters", e), func(ref: String) -> String: return PurseRules.world_loot_check(w, ref))
			checks["%s:%s (%s)" % [w, str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
	return TestHelpers.report(TAG, "Shipped encounters pass the purse rules", checks)

# Both catalogs in one sandbox world fork that holds an item loot and a passive loot fixture.
static func _test_validation() -> bool:
	var crefs: Dictionary = _catalog_refs()
	var wrefs: Dictionary = _world_refs()
	var items: Array[String] = ContentCatalog.ids("items")
	var passives: Array[String] = ContentCatalog.ids("passives")
	if crefs.is_empty() or wrefs.is_empty() or items.is_empty() or passives.is_empty():
		return TestHelpers.report_ok(TAG, "Validation fixtures", false, "need catalog party / enemy units, a world with party / enemy characters, an item and a passive")
	var world: String = str(wrefs["world"])
	var path: String = TestHelpers.sandbox("purse_world")
	if not TestHelpers.fork_worlds(path):
		TestHelpers.cleanup_worlds(path)
		return TestHelpers.report_ok(TAG, "Validation", false, "world fork failed")
	var checks: Dictionary = {}
	var setup: Array[String] = []
	setup.append_array(WorldCatalog.upsert(world, "loot", {"id": ITEM_LOOT, "name": "Purse Probe Item", "reward_kind": "item", "ref": items[0], "count": 1}))
	setup.append_array(WorldCatalog.upsert(world, "loot", {"id": PASSIVE_LOOT, "name": "Purse Probe Passive", "reward_kind": "passive", "ref": passives[0]}))
	checks["loot fixtures stored (%s)" % "; ".join(setup)] = setup.is_empty()
	var qual: String = world + ":"
	checks.merge(_matrix(wrefs, "", "world", func(e: Dictionary) -> Array[String]: return WorldCatalog.validate_entry(world, "encounters", e)))
	checks.merge(_matrix(crefs, qual, "catalog", func(e: Dictionary) -> Array[String]: return ContentCatalog.validate_entry("encounters", e)))
	# A world encounter may name its loot qualified too (rule 8: the purse resolves in the encounter's world).
	var wq: Array[String] = WorldCatalog.validate_entry(world, "encounters", _entry(wrefs, {"gold_min": 1, "loot": qual + ITEM_LOOT}))
	checks["world: qualified item loot is clean (%s)" % "; ".join(wq)] = wq.is_empty()
	# A catalog encounter has no world: a bare loot id never resolves there.
	var cbare: Array[String] = ContentCatalog.validate_entry("encounters", _entry(crefs, {"gold_min": 1, "loot": ITEM_LOOT}))
	checks["catalog: bare loot id does not resolve (%s)" % "; ".join(cbare)] = cbare.has("Purse loot '%s' does not resolve." % ITEM_LOOT)
	# Stored world purse and its warnings (world list prefixed with the qualified id; catalog per entry).
	var soft: int = PurseRules.GOLD_SOFT_CAP
	var stored: Array[String] = WorldCatalog.upsert(world, "encounters", _entry(wrefs, {"gold_min": soft, "gold_max": soft + 1, "loot": ITEM_LOOT, "once": false}))
	checks["world upsert accepts a purse (%s)" % "; ".join(stored)] = stored.is_empty()
	var own: Dictionary = DictUtil.dict_of(WorldCatalog.get_own(world, "encounters", "purse_probe").get("purse", {}))
	checks["world keeps the purse (%s)" % str(own)] = DictUtil.to_int(own.get("gold_max", -1)) == soft + 1 and str(own.get("loot", "")) == ITEM_LOOT and own.get("once", true) == false
	var soft_line: String = "%s:purse_probe: Purse gold above %d: check the budget." % [world, soft]
	checks["world warnings list the soft cap"] = WorldCatalog.encounter_warnings(world).has(soft_line)
	var empty_stored: Array[String] = WorldCatalog.upsert(world, "encounters", _entry(wrefs, {"gold_min": 0}))
	checks["world upsert accepts an empty purse (%s)" % "; ".join(empty_stored)] = empty_stored.is_empty()
	checks["world warnings list the empty purse"] = WorldCatalog.encounter_warnings(world).has("%s:purse_probe: Empty purse." % world)
	var cw: Array[String] = ContentCatalog.warnings_for("encounters", _entry(crefs, {"gold_min": 1, "gold_max": soft + 1}))
	checks["catalog warnings: soft cap (%s)" % "; ".join(cw)] = cw.has("Purse gold above %d: check the budget." % soft)
	var ce: Array[String] = ContentCatalog.warnings_for("encounters", _entry(crefs, {"gold_max": 0}))
	checks["catalog warnings: empty purse (%s)" % "; ".join(ce)] = ce.has("Empty purse.")
	var cq: Array[String] = ContentCatalog.warnings_for("encounters", _entry(crefs, {"gold_min": 3, "gold_max": 6}))
	checks["catalog warnings: none for a plain purse (%s)" % "; ".join(cq)] = not TestHelpers.any_contains(cq, "urse")
	TestHelpers.cleanup_worlds(path)
	return TestHelpers.report(TAG, "Purse validation in both catalogs (%s): every message fires once; valid purses are clean; warnings" % world, checks)

# ------------------------------------------------------------------------------
# Runtime (stage 2)
# ------------------------------------------------------------------------------

const GATE_FLAG: String = "purse_probe.gate"
const GATED_LOOT: String = "purse_probe_gated"

# Test-only purse encounters (ids) for the runtime checks.
static func _runtime_encounters() -> Dictionary:
	return {
		"purse_none": null,
		"purse_fixed": {"gold_min": 10, "gold_max": 10},
		"purse_farm": {"gold_min": 3, "gold_max": 6, "once": false},
		"purse_loot": {"gold_min": 2, "gold_max": 2, "loot": ITEM_LOOT, "once": false},
		"purse_gated": {"gold_min": 1, "loot": GATED_LOOT, "once": false},
	}

static func _new_game(world: String, unit: String) -> GameState:
	var made: Dictionary = NewGame.create(world, [unit] as Array[String], 1)
	return GameState.of(made.get("state", null))

static func _purse_of(res: Dictionary) -> Dictionary:
	return DictUtil.dict_of(res.get("purse", {}))

static func _test_runtime() -> bool:
	var crefs: Dictionary = _catalog_refs()
	var wrefs: Dictionary = _world_refs()
	var items: Array[String] = ContentCatalog.ids("items")
	if crefs.is_empty() or wrefs.is_empty() or items.is_empty():
		return TestHelpers.report_ok(TAG, "Runtime fixtures", false, "need catalog units, a world with party / enemy characters and an item")
	var world: String = str(wrefs["world"])
	var path: String = TestHelpers.sandbox("purse_runtime")
	if not TestHelpers.fork_worlds(path):
		TestHelpers.cleanup_worlds(path)
		return TestHelpers.report_ok(TAG, "Runtime", false, "world fork failed")
	var ok: bool = true
	var setup: Array[String] = []
	setup.append_array(WorldCatalog.upsert(world, "loot", {"id": ITEM_LOOT, "name": "Purse Probe Item", "reward_kind": "item", "ref": items[0], "count": 1}))
	setup.append_array(WorldCatalog.upsert(world, "loot", {"id": GATED_LOOT, "name": "Purse Probe Gated", "reward_kind": "item", "ref": items[0], "count": 1,
		"requires_flags": [{"key": GATE_FLAG, "value": true}]}))
	var encs: Dictionary = _runtime_encounters()
	for id: Variant in encs:
		var e: Dictionary = _entry(wrefs, encs[id])
		e["id"] = str(id)
		setup.append_array(WorldCatalog.upsert(world, "encounters", e))
	var gs0: GameState = _new_game(world, str(crefs["party"]))
	if not setup.is_empty() or gs0 == null:
		TestHelpers.cleanup_worlds(path)
		return TestHelpers.report_ok(TAG, "Runtime fixtures stored", false, "%s; game %s" % ["; ".join(setup), str(gs0 != null)])
	var unit: String = str(crefs["party"])
	ok = _test_no_purse(world, unit) and ok
	ok = _test_win_only(world, unit) and ok
	ok = _test_roll(world, unit) and ok
	ok = _test_loot(world, unit, items[0]) and ok
	ok = _test_results(world, unit) and ok
	ok = _test_break_pays(world, unit, wrefs) and ok
	TestHelpers.cleanup_worlds(path)
	return ok

# Acceptance "No purse": the apply result, party gold and purses are as today for won, lost and fled.
static func _test_no_purse(world: String, unit: String) -> bool:
	var checks: Dictionary = {}
	for outcome: String in BattleBridge.OUTCOMES:
		var gs: GameState = _new_game(world, unit)
		var gold: int = gs.party.gold
		var res: Dictionary = BattleBridge.apply(gs, "purse_none", outcome)
		var keys: Array = res.keys()
		keys.sort()
		checks["%s: result keys %s" % [outcome, str(keys)]] = str(keys) == str(["advanced", "errors", "ok", "xp"])
		checks["%s: gold and purses unchanged" % outcome] = gs.party.gold == gold and gs.purses.is_empty()
		checks["%s: pay of a no-purse encounter is {} and changes nothing" % outcome] = PurseRules.pay(gs, "purse_none").is_empty() and gs.purses.is_empty() and gs.party.gold == gold
	return TestHelpers.report(TAG, "No purse: apply result, gold and purses as today (won, lost, fled)", checks)

# Rules 1.1, 1.4, 1.7, 1.8: win only, once, every win counted, the gold cap, qualified ids.
static func _test_win_only(world: String, unit: String) -> bool:
	var checks: Dictionary = {}
	var qid: String = WorldCatalog.qid(world, "purse_fixed")
	for outcome: String in ["lost", "fled"]:
		var g: GameState = _new_game(world, unit)
		var gold: int = g.party.gold
		var res: Dictionary = BattleBridge.apply(g, "purse_fixed", outcome)
		checks["%s pays nothing, records nothing" % outcome] = g.party.gold == gold and g.purses.is_empty() and not res.has("purse")
	var gs: GameState = _new_game(world, unit)
	var start: int = gs.party.gold
	var first: Dictionary = BattleBridge.apply(gs, "purse_fixed", "won")
	checks["won 10-10: gold +10 (%d -> %d)" % [start, gs.party.gold]] = gs.party.gold == start + 10
	checks["won: purse.gold 10, loot empty (%s)" % str(_purse_of(first))] = DictUtil.to_int(_purse_of(first).get("gold", -1)) == 10 and str(_purse_of(first).get("loot", "x")).is_empty()
	checks["won: purses[%s] = 1 (qualified)" % qid] = DictUtil.to_int(gs.purses.get(qid, 0)) == 1 and gs.purses.size() == 1
	checks["won: still valid"] = gs.validate().is_empty()
	var second: Dictionary = BattleBridge.apply(gs, "purse_fixed", "won")
	checks["once: the second win pays 0"] = DictUtil.to_int(_purse_of(second).get("gold", -1)) == 0 and gs.party.gold == start + 10
	checks["once: no gold key for the results"] = PurseRules.result_keys(_purse_of(second)).is_empty()
	checks["once: purses[qid] becomes 2"] = DictUtil.to_int(gs.purses.get(qid, 0)) == 2
	var farm: GameState = _new_game(world, unit)
	var paid: Array[int] = []
	for i: int in range(3):
		var before: int = farm.party.gold
		var expect: int = DictUtil.to_int(PurseRules.roll(farm, "purse_farm")["gold"])
		var r: Dictionary = BattleBridge.apply(farm, "purse_farm", "won")
		paid.append(farm.party.gold - before)
		checks["once false: win %d pays the roll (%d)" % [i + 1, expect]] = farm.party.gold - before == expect and DictUtil.to_int(_purse_of(r)["gold"]) == expect and expect >= 3
	checks["once false: three wins counted"] = DictUtil.to_int(farm.purses.get(WorldCatalog.qid(world, "purse_farm"), 0)) == 3
	# Rule 1.7: the cap.
	var rich: GameState = _new_game(world, unit)
	rich.party.gold = PartyState.GOLD_MAX - 3
	var capped: Dictionary = BattleBridge.apply(rich, "purse_fixed", "won")
	checks["cap: gold stops at GOLD_MAX, 3 added (%s)" % str(_purse_of(capped))] = rich.party.gold == PartyState.GOLD_MAX and DictUtil.to_int(_purse_of(capped)["gold"]) == 3
	checks["cap: a note says so"] = TestHelpers.any_contains(JsonUtil.strings(_purse_of(capped)["notes"]), "capped")
	# Rule 1.8: a qualified ref records the qualified id, also from another world.
	var q: GameState = _new_game(world, unit)
	var _qr: Dictionary = BattleBridge.apply(q, qid, "won")
	checks["qualified ref records %s" % qid] = DictUtil.to_int(q.purses.get(qid, 0)) == 1
	var other: String = ""
	for w: String in WorldCatalog.world_ids():
		if not WorldCatalog.lineage(w).has(world):
			other = w
			break
	if not other.is_empty():
		var x: GameState = _new_game(world, unit)
		x.world_id = other
		var before_x: int = x.party.gold
		var _xr: Dictionary = BattleBridge.apply(x, qid, "won")
		checks["from world %s: records %s and pays" % [other, qid]] = DictUtil.to_int(x.purses.get(qid, 0)) == 1 and x.party.gold == before_x + 10
	return TestHelpers.report(TAG, "Win only; once; every win counted; gold cap; qualified ids (rules 1.1, 1.4, 1.7, 1.8)", checks)

# Rule 1.3: pure, seeded from (qid, time, paid), in range, both ends reachable.
static func _test_roll(world: String, unit: String) -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _new_game(world, unit)
	gs.time = 37
	var before: Dictionary = gs.to_dict()
	var a: Dictionary = PurseRules.roll(gs, "purse_farm")
	var b: Dictionary = PurseRules.roll(gs, "purse_farm")
	checks["roll is pure"] = JsonUtil.deep_equal(before, gs.to_dict())
	checks["same time and paid: same gold (%d, %d)" % [DictUtil.to_int(a["gold"]), DictUtil.to_int(b["gold"])]] = DictUtil.to_int(a["gold"]) == DictUtil.to_int(b["gold"]) and DictUtil.to_bool(a["pays"])
	var rng := RandomNumberGenerator.new()
	rng.seed = ("%s|%d|%d" % [WorldCatalog.qid(world, "purse_farm"), 37, 0]).hash()
	checks["seed is (qid|time|paid).hash()"] = DictUtil.to_int(a["gold"]) == rng.randi_range(3, 6)
	var seen: Dictionary = {}
	var in_range: bool = true
	var qid: String = WorldCatalog.qid(world, "purse_farm")
	for paid: int in range(200):
		gs.purses[qid] = paid
		var g: int = DictUtil.to_int(PurseRules.roll(gs, "purse_farm")["gold"])
		in_range = in_range and g >= 3 and g <= 6
		seen[g] = true
	checks["every roll in 3..6"] = in_range
	checks["paid 0..199 hits 3 and 6 (%s)" % str(seen.keys())] = seen.has(3) and seen.has(6)
	var once: GameState = _new_game(world, unit)
	once.purses[WorldCatalog.qid(world, "purse_fixed")] = 1
	var r: Dictionary = PurseRules.roll(once, "purse_fixed")
	checks["once, already won: does not pay"] = not DictUtil.to_bool(r["pays"]) and DictUtil.to_int(r["gold"]) == 0 and DictUtil.to_bool(r["has_purse"])
	checks["no purse: has_purse false"] = not DictUtil.to_bool(PurseRules.roll(once, "purse_none")["has_purse"])
	return TestHelpers.report(TAG, "Roll: pure, seeded, in range (rule 1.3)", checks)

# Rule 1.5: loot claimed on the first paying win; then gold only; pre-claimed and gated loot skipped, gold still paid.
static func _test_loot(world: String, unit: String, item: String) -> bool:
	var checks: Dictionary = {}
	var lqid: String = WorldCatalog.qid(world, ITEM_LOOT)
	var gs: GameState = _new_game(world, unit)
	var stash: int = DictUtil.to_int(gs.party.stash.get(item, 0))
	var first: Dictionary = BattleBridge.apply(gs, "purse_loot", "won")
	checks["first win: loot %s (%s)" % [lqid, str(_purse_of(first))]] = str(_purse_of(first).get("loot", "")) == lqid and gs.loot_claimed.has(lqid)
	checks["first win: item in the stash"] = DictUtil.to_int(gs.party.stash.get(item, 0)) == stash + 1
	var second: Dictionary = BattleBridge.apply(gs, "purse_loot", "won")
	checks["second win: gold only"] = str(_purse_of(second).get("loot", "x")).is_empty() and DictUtil.to_int(_purse_of(second)["gold"]) == 2 and DictUtil.to_int(gs.party.stash.get(item, 0)) == stash + 1
	var pre: GameState = _new_game(world, unit)
	var _c: String = RewardGrant.claim_loot(pre, lqid)
	var pre_gold: int = pre.party.gold
	var pr: Dictionary = BattleBridge.apply(pre, "purse_loot", "won")
	checks["claimed before (by a quest): skipped, gold pays"] = str(_purse_of(pr).get("loot", "x")).is_empty() and pre.party.gold == pre_gold + 2
	var gated: GameState = _new_game(world, unit)
	var gqid: String = WorldCatalog.qid(world, GATED_LOOT)
	var gr: Dictionary = BattleBridge.apply(gated, "purse_gated", "won")
	checks["unmet requires_flags: unclaimed, gold pays"] = str(_purse_of(gr).get("loot", "x")).is_empty() and not gated.loot_claimed.has(gqid) and DictUtil.to_int(_purse_of(gr)["gold"]) == 1
	var _f: String = gated.set_flag(GATE_FLAG, true)
	var gr2: Dictionary = BattleBridge.apply(gated, "purse_gated", "won")
	checks["flags met: claimed"] = str(_purse_of(gr2).get("loot", "")) == gqid and gated.loot_claimed.has(gqid)
	return TestHelpers.report(TAG, "Loot: once per save; claimed and gated entries skipped, gold still pays (rule 1.5)", checks)

# Rule 1.6: the after-fight results carry top-level gold / loot; the results popup lists them.
static func _test_results(world: String, unit: String) -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _new_game(world, unit)
	var dlg: Dictionary = DialogueRunner.after_fight(gs, {"encounter": "purse_loot"}, "won")
	checks["dialogue fight: gold 2 and loot (%s)" % str(dlg.get("gold", null))] = DictUtil.to_int(dlg.get("gold", 0)) == 2 and str(dlg.get("loot", "")) == WorldCatalog.qid(world, ITEM_LOOT)
	var map: Dictionary = ExploreRunner.after_fight(gs, {"encounter": "purse_fixed"}, "won")
	checks["map fight: gold 10"] = DictUtil.to_bool(map.get("ok", false)) and DictUtil.to_int(map.get("gold", 0)) == 10 and not map.has("loot")
	var again: Dictionary = ExploreRunner.after_fight(gs, {"encounter": "purse_fixed"}, "won")
	checks["map fight, once paid: no gold key"] = DictUtil.to_bool(again.get("ok", false)) and not again.has("gold")
	var none: Dictionary = DialogueRunner.after_fight(gs, {"encounter": "purse_none"}, "won")
	checks["no purse: no gold / loot keys"] = not none.has("gold") and not none.has("loot")
	var rewards: Dictionary = BattleResultsModel.rewards_of(dlg)
	checks["rewards_of holds gold and loot"] = DictUtil.to_int(rewards.get("gold", 0)) == 2 and not str(rewards.get("loot", "")).is_empty()
	var lines: Array[String] = BattleResultsModel.new(null, {}, [] as Array[String], "won", rewards).reward_lines()
	checks["reward lines: Gold +2 and the loot (%s)" % "; ".join(lines)] = lines.has("Gold +2") and TestHelpers.any_contains(lines, "Loot: ")
	# Road fights (TravelRunner.after_fight) merge the same PurseRules.result_keys; a leg needs a route, so the shared
	# helper is checked directly.
	checks["result_keys: gold and loot"] = JsonUtil.deep_equal(PurseRules.result_keys({"gold": 4, "loot": "w:x", "notes": []}), {"gold": 4, "loot": "w:x"})
	return TestHelpers.report(TAG, "Results: settled fights carry gold / loot; reward lines show them (rule 1.6)", checks)

# Rule 1.2: a morale break win pays the same as a full rout (both are `won`). A test-only enemy that can flee, picked
# by rule, three of them in a morale encounter with a fixed purse; two down, the third surrenders.
static func _test_break_pays(world: String, unit: String, refs: Dictionary) -> bool:
	var checks: Dictionary = {}
	var grunt_unit: String = ""
	for u: Dictionary in ContentCatalog.entries("units"):
		var n: Dictionary = ContentSchema.normalize("units", u)
		if str(n["side"]) == "enemy" and DictUtil.to_float(n["flee_ratio"]) > 0.0 and not DictUtil.to_bool(n["reckless"]) and not EncounterMorale.LEADER_TIERS.has(str(n["tier"])):
			grunt_unit = str(n["id"])
			break
	if grunt_unit.is_empty():
		return TestHelpers.report_ok(TAG, "Break win pays", false, "no catalog enemy unit that can flee")
	var stored: Array[String] = WorldCatalog.upsert(world, "characters", {"id": "purse_probe_grunt", "name": "Purse Grunt", "role": "enemy", "base_unit": grunt_unit})
	stored.append_array(WorldCatalog.upsert(world, "encounters", {"id": "purse_break", "name": "Purse Break", "zone": "stone_arena", "party": [refs["party"]],
		"enemies": ["purse_probe_grunt", "purse_probe_grunt", "purse_probe_grunt"], "morale": true, "purse": {"gold_min": 7, "gold_max": 7}}))
	checks["fixtures stored (%s)" % "; ".join(stored)] = stored.is_empty()
	var gs: GameState = _new_game(world, unit)
	var built: Dictionary = BattleBridge.encounter_for(gs, "purse_break")
	var raw_enc: Variant = built.get("encounter", null)
	if not (raw_enc is GameEncounter):
		checks["encounter built (%s)" % str(built.get("errors", []))] = false
		return TestHelpers.report(TAG, "A morale break win pays the purse (rule 1.2)", checks)
	var s := CombatSession.new(raw_enc as GameEncounter)
	s.start()
	for i: int in range(2):
		s.enemies[i].hp = 0
	# One block, as MoraleTestSuite._run_block: the break resolves at the block end.
	if s.phase != CombatSession.Phase.PLANNING or s.confirm_plan(false):
		var b: int = s.current_block
		for i: int in range(600):
			s.step_next()
			if s.phase == CombatSession.Phase.REPLAY or s.current_block != b or s.combat_over:
				break
	var outcome: String = BattleBridge.outcome(s)
	checks["break win is won (%s)" % outcome] = outcome == "won"
	var before: int = gs.party.gold
	var res: Dictionary = BattleBridge.apply(gs, "purse_break", outcome)
	checks["break win pays the full purse"] = gs.party.gold == before + 7 and DictUtil.to_int(_purse_of(res).get("gold", 0)) == 7
	s.dispose()
	return TestHelpers.report(TAG, "A morale break win pays the purse (rule 1.2)", checks)
