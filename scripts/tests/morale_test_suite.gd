class_name MoraleTestSuite
extends RefCounted

# ==============================================================================
# MORALE1-L: enemy morale (EncounterMorale).
# Stage 1 (schema): the five fields and their defaults in both schemas, old entries normalising to
# morale false, every validation message of spec 1.1 in both catalogs (and none while morale is off),
# valid entries clean in both catalogs, the leader rule (rule 2), the default group (rule 3) and the
# World Forge preview line (rule 16). Fixture refs are picked by rule (side / role / tier); world
# writes go to a sandbox fork only. Nothing depends on shipped numbers.
# ==============================================================================

const TAG: String = "Morale"

const FIELDS: Dictionary = {
	"morale": false, "morale_break": 50, "morale_leader": "", "morale_groups": [], "morale_text": "",
}

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING MORALE TEST (MORALE1-L)                 ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_schema_defaults() and ok
	ok = _test_leader_and_groups() and ok
	ok = _test_catalog_validation() and ok
	ok = _test_world_validation() and ok
	ok = _test_shipped_clean() and ok
	return ok

# ------------------------------------------------------------------------------
# Fixtures
# ------------------------------------------------------------------------------

# Catalog refs by rule: {party, a, b, chief} = a party unit, two enemy units (b may equal a) and an enemy
# unit of tier boss or elite ("" when the catalog has none).
static func _catalog_refs() -> Dictionary:
	var party: String = ""
	var foes: Array[String] = []
	var chief: String = ""
	for u: Dictionary in ContentCatalog.entries("units"):
		var side: String = str(u.get("side", ""))
		if side == "party" and party.is_empty():
			party = str(u["id"])
		elif side == "enemy":
			if EncounterMorale.LEADER_TIERS.has(str(u.get("tier", ""))):
				if chief.is_empty():
					chief = str(u["id"])
			elif foes.size() < 2:
				foes.append(str(u["id"]))
	if party.is_empty() or foes.is_empty():
		return {}
	return {"party": party, "a": foes[0], "b": foes[foes.size() - 1], "chief": chief}

# A world with a party and an enemy character: {world, party, a, b}.
static func _world_refs() -> Dictionary:
	for w: String in WorldCatalog.world_ids():
		var party: String = ""
		var foes: Array[String] = []
		for c: Dictionary in WorldCatalog.merged_entries(w, "characters"):
			var role: String = str(c.get("role", ""))
			if role == "party" and party.is_empty():
				party = WorldCatalog.ref_value(w, c)
			elif role == "enemy" and foes.size() < 2:
				foes.append(WorldCatalog.ref_value(w, c))
		if not party.is_empty() and not foes.is_empty():
			return {"world": w, "party": party, "a": foes[0], "b": foes[foes.size() - 1]}
	return {}

# Three enemies (a, b, a) so groups have something to split.
static func _entry(refs: Dictionary, changes: Dictionary) -> Dictionary:
	var e: Dictionary = {
		"id": "morale_probe", "name": "Morale Probe", "zone": "stone_arena",
		"party": [refs["party"]], "enemies": [refs["a"], refs["b"], refs["a"]],
	}
	e.merge(changes, true)
	return e

static func _good_cases(refs: Dictionary) -> Dictionary:
	return {
		"morale on, all defaults": {"morale": true},
		"break 0": {"morale": true, "morale_break": EncounterMorale.BREAK_MIN},
		"break 90": {"morale": true, "morale_break": EncounterMorale.BREAK_MAX},
		"explicit leader": {"morale": true, "morale_leader": refs["b"]},
		"two groups": {"morale": true, "morale_groups": [[0, 2], [1]]},
		"three groups": {"morale": true, "morale_groups": [[2], [1], [0]]},
		"text at the limit": {"morale": true, "morale_text": "x".repeat(EncounterMorale.MAX_TEXT)},
		"bad fields ignored while off": {"morale": false, "morale_break": EncounterMorale.BREAK_MAX + 1, "morale_leader": refs["party"],
			"morale_groups": [[], [7]], "morale_text": "x".repeat(EncounterMorale.MAX_TEXT + 1)},
	}

static func _bad_cases(refs: Dictionary) -> Dictionary:
	return {
		"break above 90": [{"morale": true, "morale_break": EncounterMorale.BREAK_MAX + 1}, "Morale break must be 0..90."],
		"break below 0": [{"morale": true, "morale_break": EncounterMorale.BREAK_MIN - 1}, "Morale break must be 0..90."],
		"leader not an enemy": [{"morale": true, "morale_leader": refs["party"]}, "Morale leader must be a unit from enemies."],
		"empty group": [{"morale": true, "morale_groups": [[0, 1, 2], []]}, "A morale group cannot be empty."],
		"index past the roster": [{"morale": true, "morale_groups": [[0, 1, 2, 3]]}, "Morale group index out of range."],
		"negative index": [{"morale": true, "morale_groups": [[0, 1, 2], [-1]]}, "Morale group index out of range."],
		"enemy missing": [{"morale": true, "morale_groups": [[0, 1]]}, "Every enemy must be in exactly one morale group."],
		"enemy twice": [{"morale": true, "morale_groups": [[0, 1], [1, 2]]}, "Every enemy must be in exactly one morale group."],
		"group not a list": [{"morale": true, "morale_groups": [[0, 1, 2], 3]}, "Morale groups must be a list of enemy index lists."],
		"long text": [{"morale": true, "morale_text": "x".repeat(EncounterMorale.MAX_TEXT + 1)}, "Morale text holds at most 80 characters."],
	}

static func _matrix(refs: Dictionary, validate: Callable) -> Dictionary:
	var checks: Dictionary = {}
	var good: Dictionary = _good_cases(refs)
	for label: Variant in good:
		var errors: Array[String] = validate.call(_entry(refs, good[label]))
		checks["valid: %s (%s)" % [str(label), "; ".join(errors)]] = errors.is_empty()
	var bad: Dictionary = _bad_cases(refs)
	for label: Variant in bad:
		var pair: Array = bad[label]
		var errors: Array[String] = validate.call(_entry(refs, pair[0]))
		checks["%s -> '%s' (%s)" % [str(label), str(pair[1]), "; ".join(errors)]] = TestHelpers.any_contains(errors, str(pair[1]))
		var morale_errors: Array[String] = []
		for msg: String in errors:
			if msg.containsn("morale"):
				morale_errors.append(msg)
		checks["%s: one morale message only (%s)" % [str(label), "; ".join(morale_errors)]] = morale_errors.size() == 1
	return checks

# ------------------------------------------------------------------------------
# Tests
# ------------------------------------------------------------------------------

static func _test_schema_defaults() -> bool:
	var checks: Dictionary = {}
	var old_catalog: Dictionary = ContentSchema.normalize("encounters", {"id": "old", "name": "Old"})
	var old_world: Dictionary = WorldSchema.normalize("encounters", {"id": "old", "name": "Old"})
	for key: String in FIELDS:
		var cf: Dictionary = ContentSchema.field("encounters", key)
		var wf: Dictionary = WorldSchema.field("encounters", key)
		checks["catalog field %s default %s" % [key, str(FIELDS[key])]] = not cf.is_empty() and str(cf.get("default", null)) == str(FIELDS[key])
		checks["world field %s default %s" % [key, str(FIELDS[key])]] = not wf.is_empty() and str(wf.get("default", null)) == str(FIELDS[key])
		checks["old catalog entry: %s = %s" % [key, str(old_catalog.get(key, null))]] = str(old_catalog.get(key, null)) == str(FIELDS[key])
		checks["old world entry: %s = %s" % [key, str(old_world.get(key, null))]] = str(old_world.get(key, null)) == str(FIELDS[key])
	checks["the default break is the schema default"] = DictUtil.to_int(ContentSchema.field("encounters", "morale_break").get("default", -1)) == EncounterMorale.DEFAULT_BREAK
	checks["old entry has morale off"] = not EncounterMorale.enabled(old_catalog) and not EncounterMorale.enabled(old_world)
	checks["old entry has no problems"] = EncounterMorale.problems(old_catalog).is_empty()
	var stored: Dictionary = ContentSchema.to_storage("encounters", {"id": "old", "name": "Old"})
	var hidden: Array[String] = []
	for key: String in ["morale_break", "morale_leader", "morale_groups", "morale_text"]:
		if stored.has(key):
			hidden.append(key)
	checks["catalog storage leaves morale_* out while morale is off (%s)" % ", ".join(hidden)] = hidden.is_empty()
	var on: Dictionary = ContentSchema.to_storage("encounters", {"id": "on", "name": "On", "morale": true, "morale_break": 40})
	checks["catalog storage keeps morale_* while morale is on"] = DictUtil.to_int(on.get("morale_break", -1)) == 40 and on.has("morale_groups")
	checks["states in spec order"] = EncounterMorale.STATES == (["steady", "shaken", "wavering", "broken"] as Array[String])
	return TestHelpers.report(TAG, "Five fields with spec defaults in both schemas; old entries normalise to morale off", checks)

# Rule 2 (leader) and rule 3 (default group), with a fake tier table (no catalog data involved).
static func _test_leader_and_groups() -> bool:
	var checks: Dictionary = {}
	var tiers: Dictionary = {"grunt": "minion", "brute": "standard", "captain": "elite", "chief": "boss"}
	var tier_of: Callable = func(ref: String) -> String: return str(tiers.get(ref, ""))
	var name_of: Callable = func(ref: String) -> String: return ref.capitalize()
	checks["authored leader wins"] = EncounterMorale.leader_ref({"enemies": ["chief", "grunt"], "morale_leader": "grunt"}, tier_of) == "grunt"
	checks["auto: first boss or elite in enemies order"] = EncounterMorale.leader_ref({"enemies": ["grunt", "captain", "chief"]}, tier_of) == "captain"
	checks["auto: boss found"] = EncounterMorale.leader_ref({"enemies": ["brute", "chief"]}, tier_of) == "chief"
	checks["auto: no boss or elite = no leader"] = EncounterMorale.leader_ref({"enemies": ["grunt", "brute"]}, tier_of).is_empty()
	var one: Array[Array] = EncounterMorale.groups_of({"enemies": ["grunt", "brute", "grunt"]})
	checks["[] = one group of every enemy (%s)" % str(one)] = one.size() == 1 and str(one[0]) == str([0, 1, 2])
	var two: Array[Array] = EncounterMorale.groups_of({"enemies": ["grunt", "brute", "grunt"], "morale_groups": [[0, 2], [1]]})
	checks["authored groups kept (%s)" % str(two)] = two.size() == 2 and str(two[0]) == str([0, 2]) and str(two[1]) == str([1])
	checks["summary off = empty"] = EncounterMorale.summary({"enemies": ["chief"]}, name_of, tier_of).is_empty()
	var s1: String = EncounterMorale.summary({"morale": true, "morale_break": 50, "enemies": ["grunt", "brute"]}, name_of, tier_of)
	checks["summary, one group, no leader (%s)" % s1] = s1 == "breaks at 50 or below"
	var s2: String = EncounterMorale.summary({"morale": true, "morale_break": 40, "enemies": ["grunt", "chief", "grunt"], "morale_groups": [[0], [1, 2]]}, name_of, tier_of)
	checks["summary, two groups, auto leader (%s)" % s2] = s2 == "breaks at 40 or below, 2 groups, leader Chief"
	return TestHelpers.report(TAG, "Leader rule 2, default group rule 3, viewer summary rule 16", checks)

static func _test_catalog_validation() -> bool:
	var refs: Dictionary = _catalog_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "Catalog validation", false, "no party / enemy units in the catalog")
	var checks: Dictionary = _matrix(refs, func(e: Dictionary) -> Array[String]: return ContentCatalog.validate_entry("encounters", e))
	return TestHelpers.report(TAG, "Catalog encounters: every morale message fires on a bad entry; valid entries are clean", checks)

# World: the shared matrix plus the preview line (explicit and auto leader) in a sandbox fork.
static func _test_world_validation() -> bool:
	var refs: Dictionary = _world_refs()
	var crefs: Dictionary = _catalog_refs()
	if refs.is_empty() or crefs.is_empty():
		return TestHelpers.report_ok(TAG, "World validation", false, "no world with party and enemy characters")
	var world: String = str(refs["world"])
	var path: String = TestHelpers.sandbox("morale_world")
	if not TestHelpers.fork_worlds(path):
		TestHelpers.cleanup_worlds(path)
		return TestHelpers.report_ok(TAG, "World validation", false, "world fork failed")
	var checks: Dictionary = _matrix(refs, func(e: Dictionary) -> Array[String]: return WorldCatalog.validate_entry(world, "encounters", e))
	var stored: Array[String] = WorldCatalog.upsert(world, "encounters", _entry(refs, {"morale": true, "morale_break": 40, "morale_leader": refs["b"], "morale_groups": [[0, 2], [1]], "morale_text": "They run."}))
	checks["world upsert accepts the morale fields (%s)" % "; ".join(stored)] = stored.is_empty()
	var own: Dictionary = WorldCatalog.get_own(world, "encounters", "morale_probe")
	checks["world keeps the morale fields"] = DictUtil.to_bool(own.get("morale", false)) and DictUtil.to_int(own.get("morale_break", -1)) == 40 and DictUtil.array_of(own.get("morale_groups", [])).size() == 2
	var leader_name: String = str(WorldCatalog.resolve(world, "characters", str(refs["b"])).get("name", ""))
	var preview: String = WorldPreview.describe(world, "encounters", own)
	checks["preview: Morale line with groups and leader (%s)" % leader_name] = preview.contains("Morale:[/color] breaks at 40 or below, 2 groups, leader %s" % leader_name)
	var off: String = WorldPreview.describe(world, "encounters", WorldSchema.normalize("encounters", _entry(refs, {})))
	checks["preview: no Morale line while off"] = not off.contains("Morale:")
	if str(crefs["chief"]).is_empty():
		checks["boss or elite enemy unit found for the auto leader"] = false
	else:
		var chief: Array[String] = WorldCatalog.upsert(world, "characters", {"id": "morale_probe_chief", "name": "Probe Chief", "role": "enemy", "base_unit": crefs["chief"]})
		checks["fixture chief stored (%s)" % "; ".join(chief)] = chief.is_empty()
		var auto: Dictionary = WorldSchema.normalize("encounters", _entry(refs, {"morale": true, "enemies": [refs["a"], "morale_probe_chief"]}))
		var auto_preview: String = WorldPreview.describe(world, "encounters", auto)
		checks["preview: auto leader from roster tier"] = auto_preview.contains("Morale:[/color] breaks at 50 or below, leader Probe Chief")
	TestHelpers.cleanup_worlds(path)
	return TestHelpers.report(TAG, "World encounters (%s): every morale message fires; valid entries are clean; preview line" % world, checks)

# Shipped encounters pass the morale rules (the new fields do not break existing data).
static func _test_shipped_clean() -> bool:
	var checks: Dictionary = {}
	for e: Dictionary in ContentCatalog.entries("encounters"):
		var errs: Array[String] = EncounterMorale.problems(ContentSchema.normalize("encounters", e))
		checks["catalog %s (%s)" % [str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
	for w: String in WorldCatalog.world_ids():
		for e: Dictionary in WorldCatalog.own_entries(w, "encounters"):
			var errs: Array[String] = EncounterMorale.problems(WorldSchema.normalize("encounters", e))
			checks["%s:%s (%s)" % [w, str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
	return TestHelpers.report(TAG, "Shipped encounters pass the morale rules", checks)
