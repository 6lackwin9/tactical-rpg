class_name BossPhaseTestSuite
extends RefCounted

# ==============================================================================
# BOSS1-L: encounter `phases` (EncounterPhases). Step 1 = the schema contract: the field and
# its defaults in both schemas, every validation message of spec sections 1.2 to 1.4 in both
# catalogs, parsing defaults, and a valid `phases` field being accepted and ignored by the
# battle build. Fixture refs are picked by rule (side / role) from the catalogs; writes go
# to sandbox forks only. Nothing depends on shipped numbers.
# ==============================================================================

const TAG: String = "BossPhase"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING BOSS PHASE TEST (BOSS1-L)               ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_schema_and_parse() and ok
	ok = _test_catalog_validation() and ok
	ok = _test_world_validation() and ok
	ok = _test_shipped_clean() and ok
	ok = _test_accepted_and_ignored() and ok
	return ok

# ------------------------------------------------------------------------------
# Fixtures
# ------------------------------------------------------------------------------

# Refs by rule: {party, a, b} = a party unit and two enemy units (a != b when possible).
static func _catalog_refs() -> Dictionary:
	var party: String = ""
	var foes: Array[String] = []
	for u: Dictionary in ContentCatalog.entries("units"):
		var side: String = str(u.get("side", ""))
		if side == "party" and party.is_empty():
			party = str(u["id"])
		elif side == "enemy" and foes.size() < 2:
			foes.append(str(u["id"]))
	if foes.size() == 1:
		foes.append(foes[0])
	if party.is_empty() or foes.is_empty():
		return {}
	return {"party": party, "a": foes[0], "b": foes[1]}

# A world with a party character and an enemy character; {world, party, a, b}.
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
		if party.is_empty() or foes.is_empty():
			continue
		if foes.size() == 1:
			foes.append(foes[0])
		return {"world": w, "party": party, "a": foes[0], "b": foes[1]}
	return {}

# The first buff with a magnitude range and the first without one (registry rule).
static func _ranged_buff() -> String:
	for s: String in StatusEffects.buffs():
		if StatusEffects.has_magnitude(s):
			return s
	return ""

static func _plain_buff() -> String:
	for s: String in StatusEffects.buffs():
		if not StatusEffects.has_magnitude(s):
			return s
	return ""

static func _debuff() -> String:
	var list: Array[String] = StatusRegistry.ids_where(StatusRegistry.KIND_DEBUFF)
	return list[0] if not list.is_empty() else ""

const MAX_ROUNDS: int = 6

# A valid two-phase list over refs a (trigger, buffed) and b (phase wave, AI change).
static func _good_phases(refs: Dictionary) -> Array:
	var ranged: String = _ranged_buff()
	return [
		{
			"id": "crown", "name": "Crown", "pause_autoplay": true,
			"trigger": {"kind": "hp_below", "unit": refs["a"], "pct": 50},
			"effects": [
				{"type": "line", "text": "The crown falls."},
				{"type": "status", "unit": refs["a"], "status": ranged, "rounds": 3, "magnitude": StatusRegistry.magnitude_min(ranged)},
				{"type": "wave", "units": [refs["b"]], "label": "More come!"},
				{"type": "ai", "unit": refs["b"], "reckless": true, "target_pref": "WEAKEST", "heal_below": 0.9},
			],
		},
		{
			"id": "late_call", "name": "Late Call", "pause_autoplay": false,
			"trigger": {"kind": "round", "round": MAX_ROUNDS},
			"effects": [{"type": "status", "unit": refs["a"], "status": _plain_buff()}],
		},
	]

static func _entry(refs: Dictionary, phases: Variant, enemies: Array = []) -> Dictionary:
	return {
		"id": "phase_probe", "name": "Phase Probe", "max_rounds": MAX_ROUNDS,
		"party": [refs["party"]],
		"enemies": enemies if not enemies.is_empty() else [refs["a"]],
		"phases": phases,
	}

static func _phase(refs: Dictionary, changes: Dictionary) -> Dictionary:
	var p: Dictionary = {"id": "p", "name": "P", "trigger": {"kind": "hp_below", "unit": refs["a"], "pct": 50}, "effects": [{"type": "line", "text": "Hi."}]}
	p.merge(changes, true)
	return p

static func _fx(refs: Dictionary, effects: Array) -> Array:
	return [_phase(refs, {"effects": effects})]

# Bad cases: label -> [phases, needle]. Every message of spec 1.2 to 1.4 (and the shape guards).
static func _bad_cases(refs: Dictionary) -> Dictionary:
	var ranged: String = _ranged_buff()
	var plain: String = _plain_buff()
	var a: String = str(refs["a"])
	var lo: int = StatusRegistry.magnitude_min(ranged)
	var hi: int = StatusRegistry.magnitude_max(ranged)
	var five: Array = []
	for i: int in range(EncounterPhases.MAX_PHASES + 1):
		five.append(_phase(refs, {"id": "p%d" % i}))
	var six_fx: Array = []
	for i: int in range(EncounterPhases.MAX_EFFECTS + 1):
		six_fx.append({"type": "status", "unit": a, "status": plain})
	return {
		"not a list": [{"id": "x"}, "Phases must be a list"],
		"too many phases": [five, "at most %d phases" % EncounterPhases.MAX_PHASES],
		"duplicate id": [[_phase(refs, {}), _phase(refs, {})], "Phase ids must be unique: 'p'"],
		"bad id": [[_phase(refs, {"id": "Bad Id"})], "snake_case"],
		"unknown phase key": [[_phase(refs, {"music": "x"})], "unknown key 'music'"],
		"no name": [[_phase(refs, {"name": ""})], "Phase 'p' needs a name."],
		"long name": [[_phase(refs, {"name": "x".repeat(EncounterPhases.MAX_NAME + 1)})], "at most %d characters" % EncounterPhases.MAX_NAME],
		"pause not bool": [[_phase(refs, {"pause_autoplay": "yes"})], "pause_autoplay must be true or false"],
		"no trigger": [[_phase(refs, {"trigger": null})], "needs a trigger"],
		"unknown trigger": [[_phase(refs, {"trigger": {"kind": "death"}})], "unknown trigger 'death'"],
		"trigger unit not enemy": [[_phase(refs, {"trigger": {"kind": "hp_below", "unit": refs["party"], "pct": 50}})], "trigger unit '%s' is not in enemies" % str(refs["party"])],
		"pct low": [[_phase(refs, {"trigger": {"kind": "hp_below", "unit": a, "pct": 0}})], "pct must be 1..99"],
		"pct high": [[_phase(refs, {"trigger": {"kind": "hp_below", "unit": a, "pct": 100}})], "pct must be 1..99"],
		"round low": [[_phase(refs, {"trigger": {"kind": "round", "round": 1}})], "round must be 2..%d" % MAX_ROUNDS],
		"round high": [[_phase(refs, {"trigger": {"kind": "round", "round": MAX_ROUNDS + 1}})], "round must be 2..%d" % MAX_ROUNDS],
		"no effects": [_fx(refs, []), "needs 1 to %d effects" % EncounterPhases.MAX_EFFECTS],
		"too many effects": [_fx(refs, six_fx), "needs 1 to %d effects" % EncounterPhases.MAX_EFFECTS],
		"two lines": [_fx(refs, [{"type": "line", "text": "a"}, {"type": "line", "text": "b"}]), "one line per phase"],
		"long line": [_fx(refs, [{"type": "line", "text": "x".repeat(EncounterPhases.MAX_LINE + 1)}]), "a line needs 1 to"],
		"two waves": [_fx(refs, [{"type": "wave", "units": [a]}, {"type": "wave", "units": [a]}]), "one wave per phase"],
		"empty wave": [_fx(refs, [{"type": "wave", "units": []}]), "a wave needs at least one unit"],
		"wave unit not enemy": [_fx(refs, [{"type": "wave", "units": [refs["party"]]}]), "wave unit '%s' is not an enemy" % str(refs["party"])],
		"unknown effect": [_fx(refs, [{"type": "boom"}]), "unknown effect 'boom'"],
		"unknown effect key": [_fx(refs, [{"type": "line", "text": "a", "sound": "x"}]), "unknown line key 'sound'"],
		"not a buff": [_fx(refs, [{"type": "status", "unit": a, "status": _debuff()}]), "'%s' is not a buff" % _debuff()],
		"status unit unknown": [_fx(refs, [{"type": "status", "unit": refs["party"], "status": plain}]), "effect unit '%s' is not in this encounter" % str(refs["party"])],
		"rounds low": [_fx(refs, [{"type": "status", "unit": a, "status": plain, "rounds": 0}]), "rounds must be 1..5"],
		"rounds high": [_fx(refs, [{"type": "status", "unit": a, "status": plain, "rounds": 6}]), "rounds must be 1..5"],
		"magnitude zero on a range": [_fx(refs, [{"type": "status", "unit": a, "status": ranged}]), "%s magnitude must be %d..%d" % [ranged, lo, hi]],
		"magnitude over the range": [_fx(refs, [{"type": "status", "unit": a, "status": ranged, "magnitude": hi + 1}]), "%s magnitude must be" % ranged],
		"magnitude without a range": [_fx(refs, [{"type": "status", "unit": a, "status": plain, "magnitude": 3}]), "%s takes no magnitude" % plain],
		"ai without settings": [_fx(refs, [{"type": "ai", "unit": a}]), "an ai effect needs at least one setting"],
		"ai bad role": [_fx(refs, [{"type": "ai", "unit": a, "role": "TANK"}]), "role must be one of"],
		"ai bad target": [_fx(refs, [{"type": "ai", "unit": a, "target_pref": "FARTHEST"}]), "target_pref must be one of"],
		"ai reckless not bool": [_fx(refs, [{"type": "ai", "unit": a, "reckless": "yes"}]), "reckless must be true or false"],
		"ai ratio out of range": [_fx(refs, [{"type": "ai", "unit": a, "flee_ratio": 1.5}]), "flee_ratio must be 0.0..1.0"],
		"ai unit unknown": [_fx(refs, [{"type": "ai", "unit": refs["party"], "reckless": true}]), "effect unit '%s' is not in this encounter" % str(refs["party"])],
	}

static func _has(errors: Array[String], needle: String) -> bool:
	return TestHelpers.any_contains(errors, needle)

static func _phase_errors(errors: Array[String]) -> Array[String]:
	var out: Array[String] = []
	for e: String in errors:
		if e.contains("hase") or e.contains("12 units") or e.contains("at most %d units" % EncounterPhases.MAX_UNITS):
			out.append(e)
	return out

# Runs the bad matrix and the good / cap / known-unit cases through `validate: Callable(entry) -> Array[String]`.
static func _matrix(refs: Dictionary, validate: Callable) -> Dictionary:
	var checks: Dictionary = {}
	var good: Array[String] = validate.call(_entry(refs, _good_phases(refs)))
	checks["a valid two-phase entry is clean (%s)" % "; ".join(good)] = good.is_empty()
	var none: Array[String] = validate.call(_entry(refs, []))
	checks["no phases is clean"] = _phase_errors(none).is_empty()
	var bad: Dictionary = _bad_cases(refs)
	for label: Variant in bad:
		var pair: Array = bad[label]
		var errors: Array[String] = validate.call(_entry(refs, pair[0]))
		checks["%s -> '%s' (%s)" % [str(label), str(pair[1]), "; ".join(_phase_errors(errors))]] = _has(errors, str(pair[1]))
	# The 12-unit cap counts enemies, wave_units and phase waves together.
	var many: Array = []
	for i: int in range(EncounterPhases.MAX_UNITS):
		many.append(refs["a"])
	var at_cap: Array[String] = validate.call(_entry(refs, [], many))
	checks["12 enemies without phases fit"] = not _has(at_cap, "at most %d units" % EncounterPhases.MAX_UNITS)
	var over: Array[String] = validate.call(_entry(refs, _fx(refs, [{"type": "wave", "units": [refs["b"]]}]), many))
	checks["a phase wave past 12 units is refused"] = _has(over, "Encounter enemies, waves and phase waves hold at most %d units." % EncounterPhases.MAX_UNITS)
	# A status / ai effect may target a unit that only arrives with a phase wave (of another phase).
	var wave_only: Array = [
		_phase(refs, {"id": "w", "effects": [{"type": "wave", "units": [refs["b"]]}]}),
		_phase(refs, {"id": "s", "effects": [{"type": "ai", "unit": refs["b"], "role": "SNIPER"}]}),
	]
	var wave_ok: Array[String] = validate.call(_entry(refs, wave_only))
	checks["effects may target phase-wave units (%s)" % "; ".join(_phase_errors(wave_ok))] = _phase_errors(wave_ok).is_empty()
	return checks

# ------------------------------------------------------------------------------
# Tests
# ------------------------------------------------------------------------------

static func _test_schema_and_parse() -> bool:
	var checks: Dictionary = {}
	var cf: Dictionary = ContentSchema.field("encounters", "phases")
	var wf: Dictionary = WorldSchema.field("encounters", "phases")
	checks["catalog field phases is json, default []"] = str(cf.get("type", "")) == "json" and cf.get("default", null) is Array and DictUtil.array_of(cf.get("default", null)).is_empty()
	checks["world field phases is json, default []"] = str(wf.get("type", "")) == "json" and wf.get("default", null) is Array and DictUtil.array_of(wf.get("default", null)).is_empty()
	checks["normalised entries without phases get []"] = DictUtil.array_of(ContentSchema.normalize("encounters", {"id": "x"}).get("phases", null)).is_empty() and ContentSchema.normalize("encounters", {"id": "x"})["phases"] is Array
	var parsed: Array[Dictionary] = EncounterPhases.parse([{"id": "p", "name": "P", "trigger": {"kind": "round", "round": 3.0},
		"effects": [{"type": "status", "unit": "u", "status": "haste"}, {"type": "wave", "units": ["w1", "w2"]}, {"type": "ai", "unit": "u", "flee_ratio": 0}]}])
	var phase: Dictionary = parsed[0] if not parsed.is_empty() else {}
	var fx: Array = DictUtil.array_of(phase.get("effects", []))
	checks["parse: pause_autoplay defaults to true"] = DictUtil.to_bool(phase.get("pause_autoplay", false)) == true
	checks["parse: trigger round as int"] = DictUtil.dict_of(phase.get("trigger", {})).get("round", null) is int and DictUtil.to_int(DictUtil.dict_of(phase["trigger"])["round"]) == 3
	var status_fx: Dictionary = DictUtil.dict_of(fx[0]) if fx.size() > 0 else {}
	checks["parse: status rounds 2, magnitude 0"] = DictUtil.to_int(status_fx.get("rounds", -1)) == EncounterPhases.DEFAULT_STATUS_ROUNDS and DictUtil.to_int(status_fx.get("magnitude", -1)) == 0
	var wave_fx: Dictionary = DictUtil.dict_of(fx[1]) if fx.size() > 1 else {}
	checks["parse: wave label defaults to ''"] = wave_fx.get("label", null) is String and str(wave_fx["label"]).is_empty()
	var ai_fx: Dictionary = DictUtil.dict_of(fx[2]) if fx.size() > 2 else {}
	checks["parse: ai keeps only given settings, ratios as float"] = ai_fx.get("flee_ratio", null) is float and not ai_fx.has("role")
	checks["reserved_refs lists every phase-wave unit"] = EncounterPhases.reserved_refs({"phases": [phase, phase]}) == (["w1", "w2", "w1", "w2"] as Array[String])
	return TestHelpers.report(TAG, "Schema field (json, default []) in both schemas; parse fills defaults; reserved refs", checks)

static func _test_catalog_validation() -> bool:
	var refs: Dictionary = _catalog_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "Catalog validation", false, "no party / enemy units in the catalog")
	var checks: Dictionary = _matrix(refs, func(e: Dictionary) -> Array[String]: return ContentCatalog.validate_entry("encounters", e))
	return TestHelpers.report(TAG, "Catalog encounters: every phase message fires on a bad entry; a two-phase entry is clean; 12-unit cap", checks)

static func _test_world_validation() -> bool:
	var refs: Dictionary = _world_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "World validation", false, "no world with party and enemy characters")
	var world: String = str(refs["world"])
	var checks: Dictionary = _matrix(refs, func(e: Dictionary) -> Array[String]: return WorldCatalog.validate_entry(world, "encounters", e))
	return TestHelpers.report(TAG, "World encounters (%s): every phase message fires on a bad entry; a two-phase entry is clean; 12-unit cap" % world, checks)

# Shipped encounters carry no phase problems (the new rules do not break existing data).
static func _test_shipped_clean() -> bool:
	var checks: Dictionary = {}
	var any_ref: Callable = func(_r: String) -> bool: return true
	for e: Dictionary in ContentCatalog.entries("encounters"):
		var errs: Array[String] = EncounterPhases.problems(ContentSchema.normalize("encounters", e), any_ref)
		checks["catalog %s (%s)" % [str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
	for w: String in WorldCatalog.world_ids():
		for e: Dictionary in WorldCatalog.own_entries(w, "encounters"):
			var errs: Array[String] = EncounterPhases.problems(WorldSchema.normalize("encounters", e), any_ref)
			checks["%s:%s (%s)" % [w, str(e.get("id", "")), "; ".join(errs)]] = errs.is_empty()
	return TestHelpers.report(TAG, "Shipped encounters pass the phase rules", checks)

# Upsert round trip in sandbox forks; the battle build ignores phases (step 1: no runtime yet).
static func _test_accepted_and_ignored() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _catalog_refs()
	var path: String = TestHelpers.sandbox("boss_phase")
	if refs.is_empty() or not TestHelpers.fork(path):
		TestHelpers.cleanup(path)
		return TestHelpers.report_ok(TAG, "Accepted and ignored", false, "no fixture refs or fork failed")
	var entry: Dictionary = _entry(refs, _good_phases(refs))
	var errors: Array[String] = ContentCatalog.upsert("encounters", entry)
	checks["catalog upsert accepts phases (%s)" % "; ".join(errors)] = errors.is_empty()
	var stored: Dictionary = ContentCatalog.get_entry("encounters", "phase_probe")
	checks["catalog keeps the phases"] = DictUtil.array_of(stored.get("phases", [])).size() == 2
	var plain: Dictionary = entry.duplicate(true)
	plain["phases"] = []
	var with_phases := CatalogEncounter.from_entry(stored)
	with_phases.build()
	var without := CatalogEncounter.from_entry(plain)
	without.build()
	checks["build ignores phases (same enemies, no extra spawns)"] = with_phases.enemies.size() == without.enemies.size() and with_phases.allies.size() == without.allies.size()
	TestHelpers.cleanup(path)
	var wrefs: Dictionary = _world_refs()
	var wpath: String = TestHelpers.sandbox("boss_phase_world")
	if not wrefs.is_empty() and TestHelpers.fork_worlds(wpath):
		var world: String = str(wrefs["world"])
		var werrors: Array[String] = WorldCatalog.upsert(world, "encounters", _entry(wrefs, _good_phases(wrefs)))
		checks["world upsert accepts phases (%s)" % "; ".join(werrors)] = werrors.is_empty()
		checks["world keeps the phases"] = DictUtil.array_of(WorldCatalog.get_own(world, "encounters", "phase_probe").get("phases", [])).size() == 2
	else:
		checks["world fork with fixture refs"] = false
	TestHelpers.cleanup_worlds(wpath)
	return TestHelpers.report(TAG, "A valid phases field is stored by both catalogs and ignored by the battle build", checks)
