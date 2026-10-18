class_name PurseTestSuite
extends RefCounted

# ==============================================================================
# PURSE1-L: encounter purse (PurseRules).
# Stage 1 (schema slice): the `purse` field and its {} default in both schemas, old entries normalising to {},
# `PurseRules.read` defaults, every validation message of spec 3 in both catalogs (one purse message per bad
# entry), valid purses clean in both catalogs (world-local and qualified loot; catalog encounters take only
# qualified loot), the two soft warnings in both catalogs, storage, and shipped encounters clean. Fixture refs
# are picked by rule (side / role, first item and passive); loot fixtures and writes go to a sandbox world fork.
# Stage 2 (runtime): roll / pay, BattleBridge hook, save state.
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
		"loot not a string": [{"gold_min": 1, "loot": 7}, "Purse loot '7' does not resolve."],
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
