class_name BossPhaseTestSuite
extends RefCounted

# ==============================================================================
# BOSS1-L: encounter `phases` (EncounterPhases). Schema: the field and its defaults in both
# schemas, every validation message of spec sections 1.2 to 1.4 in both catalogs, parsing
# defaults. Runtime (spec rules 1 to 9): reserved phase waves, hp_below firing mid-round and at
# the next round start, round phases, drops, reset and replay, autoplay pause, XP / music /
# preview readers. Fixture refs are picked by rule (side / role / tier) from the catalogs and
# battles are HP-boosted so nobody falls by accident; writes go to sandbox forks only.
# Nothing depends on shipped numbers.
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
	ok = _test_pace1_phase_schema() and ok
	ok = _test_pace1_pace() and ok
	ok = _test_pace1_start_status() and ok
	ok = _test_reserved_music() and ok
	ok = _test_mid_round() and ok
	ok = _test_last_block() and ok
	ok = _test_round_phase_wave() and ok
	ok = _test_drop_and_win() and ok
	ok = _test_reset_and_redo() and ok
	ok = _test_autoplay_pause() and ok
	ok = _test_recorded_replay() and ok
	ok = _test_pace1_enrage_mid_round() and ok
	ok = _test_pace1_or_round() and ok
	ok = _test_pace1_boss_warning() and ok
	ok = _test_pace1_start_buffs() and ok
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
		# PACE1 3.2 / 3.3 / rule 5
		"or_round on a round trigger": [[_phase(refs, {"trigger": {"kind": "round", "round": 2, "or_round": 3}})], "Phase 'p': or_round needs an hp_below trigger."],
		"or_round 1": [[_phase(refs, {"trigger": {"kind": "hp_below", "unit": a, "pct": 50, "or_round": 1}})], "Phase 'p': or_round must be 2..%d." % MAX_ROUNDS],
		"or_round past max_rounds": [[_phase(refs, {"trigger": {"kind": "hp_below", "unit": a, "pct": 50, "or_round": MAX_ROUNDS + 1}})], "Phase 'p': or_round must be 2..%d." % MAX_ROUNDS],
		"enrage unit not in enemies": [_fx(refs, [{"type": "enrage", "unit": refs["party"]}]), "Phase 'p': enrage unit '%s' is not in enemies." % str(refs["party"])],
		"enrage power 0": [_fx(refs, [{"type": "enrage", "unit": a, "power": 0}]), "Phase 'p': enrage power must be 1..25."],
		"enrage power 26": [_fx(refs, [{"type": "enrage", "unit": a, "power": EncounterPhases.ENRAGE_POWER_MAX + 1}]), "Phase 'p': enrage power must be 1..25."],
		"enrage unknown key": [_fx(refs, [{"type": "enrage", "unit": a, "rounds": 3}]), "unknown enrage key 'rounds'"],
		"two enrages": [[_phase(refs, {"id": "e1", "effects": [{"type": "enrage", "unit": a}]}), _phase(refs, {"id": "e2", "effects": [{"type": "enrage", "unit": a}]})], "An encounter holds at most one enrage."],
		"enrage phase without pause": [[_phase(refs, {"pause_autoplay": false, "effects": [{"type": "enrage", "unit": a}]})], "Phase 'p': an enrage phase must pause autoplay."],
		"phase status empower": [_fx(refs, [{"type": "status", "unit": a, "status": StatusEffects.EMPOWER, "magnitude": 10}]), "Phase 'p': empower comes only from an enrage effect."],
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

# Upsert round trip in sandbox forks; the build spawns phase waves as reserved units; XP and preview read them.
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
	var reserved_ok: bool = with_phases.enemies.size() == without.enemies.size() + 1 and with_phases.allies.size() == without.allies.size()
	for c: Combatant in with_phases.enemies:
		if without.enemies.size() > 0 and with_phases.enemies.find(c) >= without.enemies.size():
			reserved_ok = reserved_ok and with_phases.is_reserved(c) and not with_phases.grid.has_combatant(c)
	checks["build spawns the phase wave reserved and off the field"] = reserved_ok
	checks["live_enemies skips reserved units"] = with_phases.live_enemies().size() == without.enemies.size()
	TestHelpers.cleanup(path)
	var wrefs: Dictionary = _world_refs()
	var wpath: String = TestHelpers.sandbox("boss_phase_world")
	if not wrefs.is_empty() and TestHelpers.fork_worlds(wpath):
		var world: String = str(wrefs["world"])
		var werrors: Array[String] = WorldCatalog.upsert(world, "encounters", _entry(wrefs, _good_phases(wrefs)))
		checks["world upsert accepts phases (%s)" % "; ".join(werrors)] = werrors.is_empty()
		checks["world keeps the phases"] = DictUtil.array_of(WorldCatalog.get_own(world, "encounters", "phase_probe").get("phases", [])).size() == 2
		# Rule 8: XP counts enemies + the phase wave (one unit in _good_phases); rule 9: one preview line per phase.
		checks["enemy_levels adds phase-wave units"] = BattleBridge.enemy_levels(world, "phase_probe").size() == 2
		var preview: String = WorldPreview.describe(world, "encounters", WorldCatalog.get_own(world, "encounters", "phase_probe"))
		checks["preview: one line per phase"] = preview.contains("Phase Crown: ") and preview.contains("Phase Late Call: round %d -> status" % MAX_ROUNDS) and preview.contains("-> line, status, wave, ai")
	else:
		checks["world fork with fixture refs"] = false
	TestHelpers.cleanup_worlds(wpath)
	return TestHelpers.report(TAG, "A valid phases field is stored by both catalogs; the build reserves phase waves; XP and preview read phases", checks)

# ------------------------------------------------------------------------------
# PACE1-L schema slice (spec docs/specs/PACE1.md 3.1 to 3.5; the bad cases above cover 3.2 / 3.3)
# ------------------------------------------------------------------------------

# The PACE1 Crown of Nails shape over refs a (boss) and b (helper): hp_below a 50 or round MAX_ROUNDS; line, enrage, ai.
static func _pace1_phases(refs: Dictionary) -> Array:
	return [{
		"id": "crown_of_nails", "name": "Crown of Nails", "pause_autoplay": true,
		"trigger": {"kind": "hp_below", "unit": refs["a"], "pct": 50, "or_round": MAX_ROUNDS},
		"effects": [
			{"type": "line", "text": "He tears off the crown, enraged."},
			{"type": "enrage", "unit": refs["a"], "power": EncounterPhases.DEFAULT_ENRAGE_POWER},
			{"type": "ai", "unit": refs["a"], "reckless": true, "target_pref": "WEAKEST"},
			{"type": "ai", "unit": refs["b"], "heal_below": 0.9},
		],
	}]

static func _test_pace1_phase_schema() -> bool:
	var checks: Dictionary = {}
	var crefs: Dictionary = _catalog_refs()
	var wrefs: Dictionary = _world_refs()
	if crefs.is_empty() or wrefs.is_empty():
		return TestHelpers.report_ok(TAG, "PACE1 phase schema", false, "no fixture refs")
	var c_errors: Array[String] = ContentCatalog.validate_entry("encounters", _entry(crefs, _pace1_phases(crefs), [crefs["a"], crefs["b"]]))
	checks["catalog: the PACE1 Crown of Nails is clean (%s)" % "; ".join(c_errors)] = c_errors.is_empty()
	var world: String = str(wrefs["world"])
	var w_errors: Array[String] = WorldCatalog.validate_entry(world, "encounters", _entry(wrefs, _pace1_phases(wrefs), [wrefs["a"], wrefs["b"]]))
	checks["world: the PACE1 Crown of Nails is clean (%s)" % "; ".join(w_errors)] = w_errors.is_empty()
	var parsed: Array[Dictionary] = EncounterPhases.parse([
		{"id": "p", "name": "P", "trigger": {"kind": "hp_below", "unit": "u", "pct": 50}, "effects": [{"type": "enrage", "unit": "u"}]},
		{"id": "q", "name": "Q", "trigger": {"kind": "hp_below", "unit": "u", "pct": 50, "or_round": 6.0}, "effects": [{"type": "enrage", "unit": "u", "power": 15.0}]},
	])
	var t0: Dictionary = DictUtil.dict_of(parsed[0].get("trigger", {}))
	var t1: Dictionary = DictUtil.dict_of(parsed[1].get("trigger", {}))
	checks["parse: or_round defaults to 0, kept as int"] = DictUtil.to_int(t0.get(EncounterPhases.OR_ROUND, -1)) == 0 and t1.get(EncounterPhases.OR_ROUND, null) is int and DictUtil.to_int(t1[EncounterPhases.OR_ROUND]) == 6
	var e0: Array[Dictionary] = EncounterPhases.effects_of(parsed[0], EncounterPhases.EFFECT_ENRAGE)
	var e1: Array[Dictionary] = EncounterPhases.effects_of(parsed[1], EncounterPhases.EFFECT_ENRAGE)
	checks["parse: enrage power defaults to 20, kept as int"] = e0.size() == 1 and DictUtil.to_int(e0[0].get("power", -1)) == EncounterPhases.DEFAULT_ENRAGE_POWER and e1.size() == 1 and e1[0].get("power", null) is int and DictUtil.to_int(e1[0]["power"]) == 15
	var lines: Array[String] = EncounterPhases.summary_lines(_pace1_phases(crefs), func(ref: String) -> String: return ref)
	checks["summary: 'or round N' and the enrage effect (%s)" % "; ".join(lines)] = lines.size() == 1 and lines[0].contains("at 50%% HP or round %d" % MAX_ROUNDS) and lines[0].contains("-> line, enrage, ai, ai")
	checks["the enrage power cap equals the empower row's cap"] = EncounterPhases.ENRAGE_POWER_MAX == StatusRegistry.cap_of(StatusEffects.EMPOWER) and EncounterPhases.ENRAGE_POWER_MAX == StatusRegistry.magnitude_max(StatusEffects.EMPOWER)
	return TestHelpers.report(TAG, "PACE1: or_round and enrage validate and parse in both catalogs; the preview line shows them", checks)

static func _test_pace1_pace() -> bool:
	var checks: Dictionary = {}
	for schema: String in ["catalog", "world"]:
		var f: Dictionary = ContentSchema.field("encounters", "pace") if schema == "catalog" else WorldSchema.field("encounters", "pace")
		var opts: Array[String] = []
		for o: Variant in DictUtil.array_of(f.get("options", [])):
			opts.append(str(o))
		checks["%s field pace: enum, default auto, options auto/mob/elite/boss" % schema] = str(f.get("type", "")) == "enum" and str(f.get("default", "")) == EncounterPhases.PACE_AUTO and opts == EncounterPhases.PACES
	var tiers: Dictionary = {"m": "standard", "n": "minion", "e": "elite", "b": "boss"}
	var tier_of: Callable = func(ref: String) -> String: return str(tiers.get(ref, ""))
	checks["auto: no elite or boss -> mob"] = EncounterPhases.pace_of({"enemies": ["m", "n"]}, tier_of) == EncounterPhases.PACE_MOB
	checks["auto: an elite -> elite"] = EncounterPhases.pace_of({"enemies": ["m", "e"]}, tier_of) == EncounterPhases.PACE_ELITE
	checks["auto: a boss beats an elite -> boss"] = EncounterPhases.pace_of({"enemies": ["e", "b", "m"]}, tier_of) == EncounterPhases.PACE_BOSS
	checks["auto: a boss in wave_units counts"] = EncounterPhases.pace_of({"enemies": ["m"], "wave_units": ["b"]}, tier_of) == EncounterPhases.PACE_BOSS
	var wave_phase: Array = [{"id": "w", "name": "W", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": ["e"]}]}]
	checks["auto: an elite in a phase wave counts"] = EncounterPhases.pace_of({"enemies": ["m"], "phases": wave_phase}, tier_of) == EncounterPhases.PACE_ELITE
	checks["a set pace wins over the tiers"] = EncounterPhases.pace_of({"enemies": ["b"], "pace": "elite"}, tier_of) == EncounterPhases.PACE_ELITE and EncounterPhases.pace_of({"enemies": ["m"], "pace": "boss"}, tier_of) == EncounterPhases.PACE_BOSS
	checks["pace auto spelled out = unset"] = EncounterPhases.pace_of({"enemies": ["e"], "pace": "auto"}, tier_of) == EncounterPhases.pace_of({"enemies": ["e"]}, tier_of)
	var crefs: Dictionary = _catalog_refs()
	if not crefs.is_empty():
		var ok_entry: Dictionary = _entry(crefs, [])
		ok_entry["pace"] = EncounterPhases.PACE_ELITE
		var bad_entry: Dictionary = _entry(crefs, [])
		bad_entry["pace"] = "epic"
		var ok_errors: Array[String] = ContentCatalog.validate_entry("encounters", ok_entry)
		checks["catalog accepts pace elite (%s)" % "; ".join(ok_errors)] = ok_errors.is_empty()
		checks["catalog refuses an unknown pace"] = ContentCatalog.validate_entry("encounters", bad_entry).size() > ok_errors.size()
	var wrefs: Dictionary = _world_refs()
	if not wrefs.is_empty():
		var w_bad: Dictionary = _entry(wrefs, [])
		w_bad["pace"] = "epic"
		var w_ok: Dictionary = _entry(wrefs, [])
		w_ok["pace"] = EncounterPhases.PACE_BOSS
		var w_ok_errors: Array[String] = WorldCatalog.validate_entry(str(wrefs["world"]), "encounters", w_ok)
		checks["world accepts pace boss (%s)" % "; ".join(w_ok_errors)] = w_ok_errors.is_empty()
		checks["world refuses an unknown pace"] = WorldCatalog.validate_entry(str(wrefs["world"]), "encounters", w_bad).size() > w_ok_errors.size()
	return TestHelpers.report(TAG, "PACE1: pace field in both schemas; pace_of resolves mob / elite / boss (waves included), a set pace wins", checks)

# A valid start buff (empower with a flag) and a valid plain one (a buff without magnitude, no flags).
static func _start_ok() -> Dictionary:
	return {"status": StatusEffects.EMPOWER, "rounds": 2, "magnitude": 10, "requires_flags": [{"key": "pace1.test_saved", "value": true}], "text": "A friend rides ahead. The Wardens strike first."}

static func _start_bad_cases() -> Dictionary:
	var plain: String = _plain_buff()
	var ok: Dictionary = _start_ok()
	var one: Callable = func(changes: Dictionary) -> Array:
		var sb: Dictionary = ok.duplicate(true)
		sb.merge(changes, true)
		return [sb]
	var emp: String = StatusEffects.EMPOWER
	var lo: int = StatusRegistry.magnitude_min(emp)
	var hi: int = StatusRegistry.magnitude_max(emp)
	return {
		"not a list": [{"status": emp}, "Start buffs must be a list of objects."],
		"three entries": [[ok, ok, ok], "An encounter holds at most 2 start buffs."],
		"not an object": [["empower"], "Start buff: each entry must be an object."],
		"unknown key": [one.call({"unit": "x"}), "Start buff: unknown key 'unit'."],
		"taunt": [one.call({"status": StatusEffects.TAUNT, "magnitude": 0}), "Start buff 'taunt' must be a buff other than taunt or riposte."],
		"riposte": [one.call({"status": StatusEffects.RIPOSTE, "magnitude": 0}), "Start buff 'riposte' must be a buff other than taunt or riposte."],
		"a debuff": [one.call({"status": _debuff(), "magnitude": 0}), "Start buff '%s' must be a buff other than taunt or riposte." % _debuff()],
		"rounds 0": [one.call({"rounds": 0}), "Start buff '%s': rounds must be 1..5." % emp],
		"rounds 6": [one.call({"rounds": 6}), "Start buff '%s': rounds must be 1..5." % emp],
		"magnitude 0 on a range": [one.call({"magnitude": 0}), "Start buff '%s': magnitude must be %d..%d." % [emp, lo, hi]],
		"magnitude over the range": [one.call({"magnitude": hi + 1}), "Start buff '%s': magnitude must be %d..%d." % [emp, lo, hi]],
		"magnitude without a range": [one.call({"status": plain, "magnitude": 3}), "Start buff '%s': magnitude must be 0..0." % plain],
		"bad flag key": [one.call({"requires_flags": [{"key": "Bad Key", "value": true}]}), "Start buff flags 1: flag key 'Bad Key'"],
		"flags not a list": [one.call({"requires_flags": "x"}), "Start buff flags must be a list"],
		"no text": [one.call({"text": "  "}), "Start buff '%s' needs a text." % emp],
		"long text": [one.call({"text": "x".repeat(EncounterPhases.START_TEXT_MAX + 1)}), "Start buff '%s' needs a text." % emp],
	}

static func _test_pace1_start_status() -> bool:
	var checks: Dictionary = {}
	var wf: Dictionary = WorldSchema.field("encounters", "start_status")
	checks["world field start_status is json, default []"] = str(wf.get("type", "")) == "json" and wf.get("default", null) is Array and DictUtil.array_of(wf["default"]).is_empty()
	checks["the catalog encounter schema has no start_status field"] = ContentSchema.field("encounters", "start_status").is_empty()
	var crefs: Dictionary = _catalog_refs()
	var wrefs: Dictionary = _world_refs()
	if crefs.is_empty() or wrefs.is_empty():
		return TestHelpers.report_ok(TAG, "PACE1 start_status", false, "no fixture refs")
	var c_entry: Dictionary = _entry(crefs, [])
	c_entry["start_status"] = [_start_ok()]
	checks["catalog: start_status is refused (world encounters only)"] = _has(ContentCatalog.validate_entry("encounters", c_entry), "Start buffs (start_status) are for world encounters only.")
	var world: String = str(wrefs["world"])
	var plain_ok: Dictionary = {"status": _plain_buff(), "text": "Fresh legs."}
	var good: Dictionary = _entry(wrefs, [])
	good["start_status"] = [_start_ok(), plain_ok]
	var good_errors: Array[String] = WorldCatalog.validate_entry(world, "encounters", good)
	checks["world: two valid start buffs are clean (%s)" % "; ".join(good_errors)] = good_errors.is_empty()
	var none: Dictionary = _entry(wrefs, [])
	checks["world: no start_status is clean"] = WorldCatalog.validate_entry(world, "encounters", none).is_empty()
	var bad: Dictionary = _start_bad_cases()
	for label: Variant in bad:
		var pair: Array = bad[label]
		var e: Dictionary = _entry(wrefs, [])
		e["start_status"] = pair[0]
		var errors: Array[String] = WorldCatalog.validate_entry(world, "encounters", e)
		checks["start %s -> '%s' (%s)" % [str(label), str(pair[1]), "; ".join(errors)]] = _has(errors, str(pair[1]))
	var parsed: Array[Dictionary] = EncounterPhases.parse_start_status([{"status": StatusEffects.EMPOWER, "magnitude": 10.0, "text": "t"}, "skip"])
	checks["parse: rounds 2, magnitude as int, requires_flags [] ; non-objects skipped"] = parsed.size() == 1 and DictUtil.to_int(parsed[0]["rounds"]) == EncounterPhases.DEFAULT_STATUS_ROUNDS and parsed[0]["magnitude"] is int and DictUtil.array_of(parsed[0]["requires_flags"]).is_empty()
	var lines: Array[String] = EncounterPhases.start_status_lines([_start_ok(), plain_ok])
	checks["preview lines (%s)" % "; ".join(lines)] = lines.size() == 2 and lines[0] == "Start: %s 10, 2 rounds if pace1.test_saved = true" % StatusEffects.display_name(StatusEffects.EMPOWER) and lines[1] == "Start: %s, %d rounds (always)" % [StatusEffects.display_name(_plain_buff()), EncounterPhases.DEFAULT_STATUS_ROUNDS]
	# Upsert round trip in a sandbox world fork; the World Forge preview shows the start buff and the pace.
	var wpath: String = TestHelpers.sandbox("pace1_start_world")
	if TestHelpers.fork_worlds(wpath):
		var stored_entry: Dictionary = _entry(wrefs, [])
		stored_entry["id"] = "pace1_probe"
		stored_entry["start_status"] = [_start_ok()]
		var up_errors: Array[String] = WorldCatalog.upsert(world, "encounters", stored_entry)
		checks["world upsert accepts start_status (%s)" % "; ".join(up_errors)] = up_errors.is_empty()
		var stored: Dictionary = WorldCatalog.get_own(world, "encounters", "pace1_probe")
		checks["world keeps the start buff"] = DictUtil.array_of(stored.get("start_status", [])).size() == 1
		var preview: String = WorldPreview.describe(world, "encounters", stored)
		checks["preview shows the start buff and the pace"] = preview.contains("Start: %s 10, 2 rounds if pace1.test_saved = true" % StatusEffects.display_name(StatusEffects.EMPOWER)) and preview.contains("Pace")
	else:
		checks["world fork"] = false
	TestHelpers.cleanup_worlds(wpath)
	return TestHelpers.report(TAG, "PACE1: start_status on world encounters only; every 3.4 message fires; parse and preview", checks)

# ------------------------------------------------------------------------------
# Runtime fixtures
# ------------------------------------------------------------------------------

const HP_BOOST: int = 20
const MAX_STEPS: int = 400
const MAX_TICKS: int = 4000
const RUNTIME_ROUNDS: int = 6

# Refs with two DIFFERENT enemy units (a = trigger / boss, b = the other one); {} when the catalog has fewer.
static func _rt_refs() -> Dictionary:
	var refs: Dictionary = _catalog_refs()
	if refs.is_empty() or str(refs["a"]) == str(refs["b"]):
		return {}
	return refs

static func _rt_entry(refs: Dictionary, phases: Array, enemies: Array, id: String = "phase_rt") -> Dictionary:
	return {
		"id": id, "name": "Phase Runtime", "max_rounds": RUNTIME_ROUNDS, "total_blocks": 6,
		"party": [refs["party"]], "enemies": enemies, "phases": phases,
	}

# A started session over `entry` with every unit's HP boosted (nobody falls by accident).
static func _session(entry: Dictionary) -> CombatSession:
	var s := CombatSession.new(CatalogEncounter.from_entry(entry))
	s.start()
	for c: Combatant in s.allies + s.enemies:
		c.max_hp *= HP_BOOST
		c.hp = c.max_hp
	return s

static func _enc(s: CombatSession) -> CatalogEncounter:
	return s.encounter as CatalogEncounter

static func _confirm(s: CombatSession) -> bool:
	s.plan_apply_preset_all("balanced")
	return s.confirm_plan(false)

# Steps until the current block has resolved (the session moved to the next block, or the round concluded).
static func _run_block(s: CombatSession) -> void:
	var b: int = s.current_block
	var r: int = s.current_round
	for i: int in range(MAX_STEPS):
		s.step_next()
		if s.phase == CombatSession.Phase.REPLAY or s.current_block != b or s.current_round != r:
			return

static func _play_round(s: CombatSession) -> bool:
	if not _confirm(s):
		return false
	TestHelpers.resolve_until_replay(s, MAX_STEPS)
	return s.phase == CombatSession.Phase.REPLAY

# Kinds (IntentRules.kind_of) of `c`'s scheduled actions this round.
static func _kinds(s: CombatSession, c: Combatant) -> Array[String]:
	var out: Array[String] = []
	for a: CombatResolver.ScheduledAction in s.resolver.scheduled_actions:
		if a.combatant == c:
			out.append(IntentRules.kind_of(a.command))
	return out

static func _withdraws(kinds: Array[String]) -> bool:
	return kinds.has("retreat") or kinds.has("flee")

static func _phase_entries(s: CombatSession, round_num: int = -1) -> Array[ActionLogEntry]:
	var out: Array[ActionLogEntry] = []
	for e: ActionLogEntry in s.log_entries:
		if e.skill_subtype == EncounterPhases.SUBTYPE and (round_num < 0 or e.round_num == round_num):
			out.append(e)
	return out

static func _fired_at(s: CombatSession, id: String) -> Array:
	return DictUtil.array_of(_enc(s).phase_runtime.fired.get(id, []))

# Fires a phase signal counter: every phase_fired emission appends its info.
static func _count_signals(s: CombatSession) -> Array:
	var got: Array = []
	s.phase_fired.connect(func(_e: ActionLogEntry, info: Dictionary) -> void: got.append(info))
	return got

# The ai effect that makes `b` withdraw at once (flee at 100% HP, never reckless).
static func _flee_fx(ref: String) -> Dictionary:
	return {"type": "ai", "unit": ref, "flee_ratio": 1.0, "reckless": false}

# ------------------------------------------------------------------------------
# Runtime tests
# ------------------------------------------------------------------------------

# Rule 8: a boss-tier unit only in a phase wave sets the boss music context.
static func _test_reserved_music() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _rt_refs()
	var boss: String = ""
	var plain: String = ""
	for u: Dictionary in ContentCatalog.entries("units"):
		if str(u.get("side", "")) != "enemy":
			continue
		if str(u.get("tier", "")) == "boss" and boss.is_empty():
			boss = str(u["id"])
		elif str(u.get("tier", "")) != "boss" and plain.is_empty():
			plain = str(u["id"])
	if refs.is_empty() or boss.is_empty() or plain.is_empty():
		return TestHelpers.report_ok(TAG, "Music reads phase waves", false, "no boss-tier and plain enemy units")
	var phases: Array = [{"id": "w", "name": "W", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": [boss]}]}]
	var with_wave := CatalogEncounter.from_entry(_rt_entry(refs, phases, [plain]))
	var without := CatalogEncounter.from_entry(_rt_entry(refs, [], [plain]))
	checks["phase-wave boss -> boss context"] = MusicDirector.combat_context(with_wave) == MusicDirector.CTX_BOSS
	checks["no phases -> combat context"] = MusicDirector.combat_context(without) == MusicDirector.CTX_COMBAT
	return TestHelpers.report(TAG, "Music: a boss-tier unit in a phase wave picks the boss context", checks)

# hp_below met at the end of block 1: the phase fires before block 2 (EVENT, buff active in that block); its AI
# change shows in the next round's scheduled actions, not in this round's.
static func _test_mid_round() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _rt_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "Mid-round phase", false, "fewer than two enemy units")
	var ranged: String = _ranged_buff()
	var a_ref: String = str(refs["a"])
	var b_ref: String = str(refs["b"])
	var phases: Array = [{"id": "rage", "name": "Rage", "trigger": {"kind": "hp_below", "unit": a_ref, "pct": 50},
		"effects": [{"type": "line", "text": "Rage!"}, {"type": "status", "unit": a_ref, "status": ranged, "rounds": 2, "magnitude": StatusRegistry.magnitude_min(ranged)}, _flee_fx(b_ref)]}]
	var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref, b_ref]))
	var got: Array = _count_signals(s)
	var a: Combatant = _enc(s).unit(a_ref, 0)
	var b: Combatant = _enc(s).unit(b_ref, 0)
	var base_flee: float = _enc(s).profile_of(b).flee_ratio
	a.hp = a.max_hp / 2
	checks["plan confirmed"] = _confirm(s)
	var round1_kinds: Array[String] = _kinds(s, b)
	_run_block(s)
	checks["due after block 1, not fired yet"] = _enc(s).phase_runtime.due.has("rage") and _fired_at(s, "rage").is_empty()
	s.step_next()
	var last: ActionLogEntry = s.log_entries.back() if not s.log_entries.is_empty() else null
	checks["block 2 starts with the phase EVENT"] = last != null and last.skill_subtype == EncounterPhases.SUBTYPE and last.skill_name == "Rage" and last.block_num == 2 and last.round_num == 1 and last.detail_text == "Rage!" and last.actor_name == a.name
	checks["fired at [1, 2]"] = _fired_at(s, "rage") == [1, 2]
	checks["buff active in block 2"] = a.statuses.has(ranged) and str(a.status_sources.get(ranged, "")) == "phase:rage"
	checks["one phase_fired signal with the line"] = got.size() == 1 and str(DictUtil.dict_of(got[0]).get("line", "")) == "Rage!"
	checks["AI unchanged this round (profile and plan)"] = is_equal_approx(_enc(s).profile_of(b).flee_ratio, base_flee) and _kinds(s, b) == round1_kinds and not _withdraws(round1_kinds)
	TestHelpers.resolve_until_replay(s, MAX_STEPS)
	s.advance_round()
	checks["AI change in round 2's draft"] = is_equal_approx(_enc(s).profile_of(b).flee_ratio, 1.0) and _withdraws(_kinds(s, b))
	s.dispose()
	return TestHelpers.report(TAG, "hp_below crossed in block 1 fires before block 2; buff live at once; AI from the next round", checks)

# Crossing at the end of the last block fires in the next round's prepare_round, before the enemy draft.
static func _test_last_block() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _rt_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "Last-block phase", false, "fewer than two enemy units")
	var a_ref: String = str(refs["a"])
	var b_ref: String = str(refs["b"])
	var phases: Array = [{"id": "late", "name": "Late", "trigger": {"kind": "hp_below", "unit": a_ref, "pct": 50},
		"effects": [{"type": "line", "text": "Late!"}, _flee_fx(b_ref)]}]
	var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref, b_ref]))
	var got: Array = _count_signals(s)
	var a: Combatant = _enc(s).unit(a_ref, 0)
	var b: Combatant = _enc(s).unit(b_ref, 0)
	checks["plan confirmed"] = _confirm(s)
	for i: int in range(s.total_blocks - 1):
		_run_block(s)
	checks["still in round 1 before the last block"] = s.current_round == 1 and s.current_block == s.total_blocks and _fired_at(s, "late").is_empty()
	a.hp = (a.max_hp * 2) / 5
	_run_block(s)
	checks["round 1 concluded with the phase due, not fired"] = s.phase == CombatSession.Phase.REPLAY and _enc(s).phase_runtime.due.has("late") and _phase_entries(s, 1).is_empty()
	s.advance_round()
	var r2: Array[ActionLogEntry] = _phase_entries(s, 2)
	checks["fired at the round 2 start [2, 0]"] = _fired_at(s, "late") == [2, 0] and r2.size() == 1 and r2[0].block_num == 0
	checks["signal once"] = got.size() == 1
	checks["round 2 draft already uses the change"] = is_equal_approx(_enc(s).profile_of(b).flee_ratio, 1.0) and _withdraws(_kinds(s, b))
	s.dispose()
	return TestHelpers.report(TAG, "A crossing at the end of the last block fires at the next round start, before the enemy draft", checks)

# A round 3 phase fires in round 3's intro; its reserved wave units are placed and drafted that round.
static func _test_round_phase_wave() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _rt_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "Round phase", false, "fewer than two enemy units")
	var a_ref: String = str(refs["a"])
	var b_ref: String = str(refs["b"])
	var phases: Array = [{"id": "horn", "name": "Horn", "trigger": {"kind": "round", "round": 3}, "effects": [{"type": "line", "text": "Horns!"}, {"type": "wave", "units": [b_ref]}]}]
	var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref]))
	var b: Combatant = _enc(s).unit(b_ref, 0)
	checks["wave unit spawned, reserved, off the field"] = b != null and s.enemies.has(b) and _enc(s).is_reserved(b) and not s.grid.has_combatant(b) and not _enc(s).live_enemies().has(b)
	var played: bool = _play_round(s)
	s.advance_round()
	played = _play_round(s) and played
	checks["rounds 1-2 played, still reserved, no tells"] = played and _enc(s).is_reserved(b) and not s.grid.has_combatant(b)
	s.advance_round()
	var r3: Array[ActionLogEntry] = _phase_entries(s, 3)
	checks["phase in round 3's intro"] = r3.size() == 1 and r3[0].block_num == 0 and r3[0].actor_name == _enc(s).encounter_name and _fired_at(s, "horn") == [3, 0]
	checks["wave placed and no longer reserved"] = b != null and s.grid.has_combatant(b) and not _enc(s).is_reserved(b)
	checks["wave units have scheduled actions in round 3"] = b != null and not _kinds(s, b).is_empty()
	s.dispose()
	return TestHelpers.report(TAG, "A round 3 phase fires in round 3's intro and its wave is drafted that round", checks)

# The trigger unit falls from 60% within one block: the phase drops; the battle is won with its wave never placed.
static func _test_drop_and_win() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _rt_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "Drop", false, "fewer than two enemy units")
	var a_ref: String = str(refs["a"])
	var b_ref: String = str(refs["b"])
	var phases: Array = [{"id": "rage", "name": "Rage", "trigger": {"kind": "hp_below", "unit": a_ref, "pct": 50}, "effects": [{"type": "wave", "units": [b_ref]}]}]
	var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref]))
	var a: Combatant = _enc(s).unit(a_ref, 0)
	var b: Combatant = _enc(s).unit(b_ref, 0)
	a.hp = (a.max_hp * 3) / 5
	checks["plan confirmed"] = _confirm(s)
	s.step_next()   # block 1 starts and its first action resolves
	a.hp = 0        # ...and the trigger unit falls inside the block
	TestHelpers.resolve_until_replay(s, MAX_STEPS)
	checks["phase dropped, never fired"] = _enc(s).phase_runtime.is_dropped("rage") and _fired_at(s, "rage").is_empty() and _phase_entries(s).is_empty()
	checks["battle over and won"] = s.combat_over and BattleBridge.outcome(s) == "won"
	checks["wave never placed, still reserved"] = b != null and not s.grid.has_combatant(b) and _enc(s).is_reserved(b) and b.is_alive()
	s.dispose()
	return TestHelpers.report(TAG, "A trigger unit downed before its phase drops it; the battle is won without the reserved wave", checks)

# reset_round rewinds a phase fired in that round (not fired, buff gone, profiles back) and the same plan fires it
# again at the same block; a phase fired at the round start shows once after the reset and is not applied again.
static func _test_reset_and_redo() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _rt_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "Reset", false, "fewer than two enemy units")
	var ranged: String = _ranged_buff()
	var a_ref: String = str(refs["a"])
	var b_ref: String = str(refs["b"])
	var phases: Array = [
		{"id": "dawn", "name": "Dawn", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "line", "text": "Dawn."}, _flee_fx(b_ref)]},
		{"id": "rage", "name": "Rage", "trigger": {"kind": "hp_below", "unit": a_ref, "pct": 50},
			"effects": [{"type": "status", "unit": a_ref, "status": ranged, "rounds": 2, "magnitude": StatusRegistry.magnitude_min(ranged)}, {"type": "ai", "unit": a_ref, "flee_ratio": 1.0, "reckless": false}]},
	]
	var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref, b_ref]))
	var got: Array = _count_signals(s)
	var a: Combatant = _enc(s).unit(a_ref, 0)
	var b: Combatant = _enc(s).unit(b_ref, 0)
	var a_base_flee: float = _enc(s).profile_of(a).flee_ratio
	checks["round 1 played"] = _play_round(s)
	s.advance_round()
	checks["dawn fired at the round 2 start"] = _fired_at(s, "dawn") == [2, 0] and got.size() == 1 and is_equal_approx(_enc(s).profile_of(b).flee_ratio, 1.0)
	a.hp = a.max_hp / 2
	checks["round 2 plan confirmed"] = _confirm(s)
	_run_block(s)
	s.step_next()
	var first: Array = _fired_at(s, "rage")
	checks["rage fired mid-round 2"] = first.size() == 2 and DictUtil.to_int(first[0]) == 2 and a.statuses.has(ranged) and got.size() == 2
	s.reset_round()
	checks["after reset: rage rewound (not fired, not due)"] = _fired_at(s, "rage").is_empty() and not _enc(s).phase_runtime.due.has("rage")
	checks["after reset: its buff is gone"] = not a.statuses.has(ranged)
	checks["after reset: profiles back to the round-start ones"] = is_equal_approx(_enc(s).profile_of(a).flee_ratio, a_base_flee) and is_equal_approx(_enc(s).profile_of(b).flee_ratio, 1.0)
	checks["after reset: dawn shown once, still fired, no new signal"] = _phase_entries(s, 2).size() == 1 and _fired_at(s, "dawn") == [2, 0] and got.size() == 2
	a.hp = a.max_hp / 2
	checks["redo plan confirmed"] = _confirm(s)
	_run_block(s)
	s.step_next()
	checks["the same plan fires rage again at the same block"] = _fired_at(s, "rage") == first and a.statuses.has(ranged)
	s.dispose()
	return TestHelpers.report(TAG, "reset_round rewinds a mid-round phase and the redo fires it again; a round-start phase shows once", checks)

# Rule 7: pause_autoplay true stops autoplay at the next planning phase ("<actor>: <phase>"), the saved switch stays
# on; pause_autoplay false keeps it running.
static func _test_autoplay_pause() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _rt_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "Autoplay", false, "fewer than two enemy units")
	for pause: bool in [true, false]:
		var phases: Array = [{"id": "dawn", "name": "Dawn", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "line", "text": "Dawn."}], "pause_autoplay": pause}]
		var s: CombatSession = _session(_rt_entry(refs, phases, [refs["a"]]))
		var auto := CombatAutoplay.new(s)
		auto.set_autoplay(true)
		var target_round: int = 2 if pause else 3
		var reached: bool = false
		for i: int in range(MAX_TICKS):
			if s.combat_over or (s.current_round >= target_round and s.is_planning() and (not pause or not auto.is_on())):
				reached = true
				break
			auto.tick()
			s.auto_play_tick()
		if pause:
			checks["pause: stopped in round 2 planning"] = reached and s.current_round == 2 and s.is_planning() and not auto.is_on()
			checks["pause: reason '<encounter>: Dawn' (%s)" % auto.pause_reason()] = auto.pause_reason() == "%s: Dawn" % _enc(s).encounter_name
			checks["pause: saved switch stays on"] = auto.remembered_on
			for i: int in range(20):
				auto.tick()
			checks["pause: the round is drafted, not confirmed"] = s.is_planning() and not s.plan_confirmed and s.current_round == 2
		else:
			checks["no pause: still running in round 3"] = reached and s.current_round >= 3 and auto.is_on()
		auto.detach()
		s.dispose()
	return TestHelpers.report(TAG, "Autoplay pauses after a pause_autoplay phase and keeps running without it", checks)

# A recorded battle with phases replays with identical RoundOutcome records (PlaytestRecorder / PlaytestReplayer).
static func _test_recorded_replay() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _rt_refs()
	var path: String = TestHelpers.sandbox("boss_phase_rec")
	if refs.is_empty() or not TestHelpers.fork(path):
		TestHelpers.cleanup(path)
		return TestHelpers.report_ok(TAG, "Recorded replay", false, "no fixture refs or fork failed")
	var ranged: String = _ranged_buff()
	var a_ref: String = str(refs["a"])
	var b_ref: String = str(refs["b"])
	var phases: Array = [
		{"id": "hurt", "name": "Hurt", "trigger": {"kind": "hp_below", "unit": a_ref, "pct": 99},
			"effects": [{"type": "line", "text": "Hurt."}, {"type": "status", "unit": a_ref, "status": ranged, "rounds": 2, "magnitude": StatusRegistry.magnitude_min(ranged)}, {"type": "ai", "unit": a_ref, "reckless": true}, {"type": "enrage", "unit": a_ref, "power": 20}]},   # PACE1: the replay covers the enrage
		{"id": "horn", "name": "Horn", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": [b_ref]}]},
	]
	var errors: Array[String] = ContentCatalog.upsert("encounters", _rt_entry(refs, phases, [a_ref, b_ref], "phase_rec"))
	checks["fixture stored (%s)" % "; ".join(errors)] = errors.is_empty()
	var saved_enabled: int = PlaytestStore.force_enabled
	PlaytestStore.force_enabled = 1
	PlaytestStore.use_root(path.path_join("rec"))
	DirAccess.make_dir_recursive_absolute(path.path_join("rec"))
	var rec: PlaytestRecorder = PlaytestRecorder.open("user", "", "", path)
	var s := CombatSession.new(EncounterSource.create("phase_rec"))
	rec.attach(s)
	s.start()
	for r: int in range(3):
		if not _play_round(s) or s.combat_over or not s.can_advance_round():
			break
		s.advance_round()
	checks["the recording fired phases"] = not _phase_entries(s).is_empty()
	checks["the recording enraged the boss (PACE1)"] = StatusEffects.has_any(s.enemies[0], StatusEffects.EMPOWER)
	rec.flush()
	rec.detach()
	s.dispose()
	var data: Dictionary = PlaytestRecord.normalize(DictUtil.dict_of(PlaytestStore.load_json(rec.path).get("data", {})))
	var rp: PlaytestReplayer = PlaytestReplayer.for_battle(PlaytestRecord.battle_at(data, 0))
	checks["replay starts"] = rp.run()
	checks["every round matches (%s)" % str(rp.first_problem().get("detail", ""))] = not rp.rounds.is_empty() and rp.all_match()
	if rp.session != null:
		rp.session.dispose()
	PlaytestStore.force_enabled = saved_enabled
	PlaytestStore.use_root(PlaytestStore.USER_ROOT)
	TestHelpers.cleanup(path)
	return TestHelpers.report(TAG, "A recorded battle with phases replays with identical round outcomes", checks)

# ------------------------------------------------------------------------------
# PACE1-L runtime (spec docs/specs/PACE1.md section 6)
# ------------------------------------------------------------------------------

const ENRAGE_POWER: int = 20

static func _enrage_phase(a_ref: String, extra_trigger: Dictionary = {}) -> Array:
	var trigger: Dictionary = {"kind": "hp_below", "unit": a_ref, "pct": 50}
	trigger.merge(extra_trigger, true)
	return [{"id": "crown", "name": "Crown", "trigger": trigger,
		"effects": [{"type": "line", "text": "Enraged!"}, {"type": "enrage", "unit": a_ref, "power": ENRAGE_POWER}]}]

# skill_mult of `c`'s scheduled attacks this round, in schedule order.
static func _attack_mults(s: CombatSession, c: Combatant) -> Array[float]:
	var out: Array[float] = []
	for a: CombatResolver.ScheduledAction in s.resolver.scheduled_actions:
		if a.combatant == c and a.command is AttackCommand:
			var cmd: AttackCommand = a.command
			out.append(cmd.skill_mult)
	return out

# A comparable read of the planning intents: [name, kind, enraged, revealed, plan size] per record.
static func _intent_summary(s: CombatSession) -> Array:
	var out: Array = []
	for r: Dictionary in s.enemy_intents():
		var u: Combatant = r["unit"]
		out.append([u.name, str(r["kind"]), DictUtil.to_bool(r.get("enraged", false)), DictUtil.to_bool(r["revealed"]), DictUtil.array_of(r["plan"]).size()])
	return out

static func _intent_for(s: CombatSession, c: Combatant) -> Dictionary:
	return s.intent_of(c)

static func _any_hero_has(s: CombatSession, status: String) -> bool:
	for c: Combatant in s.allies:
		if c.statuses.has(status) or c.pending_statuses.has(status):
			return true
	return false

# Enrage fired mid-round: this round's remaining actions keep their power; from the next planning phase the boss
# hits at x1.20, its intent is enraged and revealed without foresight, and no hero gains a status. reset_round keeps it.
static func _test_pace1_enrage_mid_round() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _rt_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "PACE1 enrage", false, "fewer than two enemy units")
	var a_ref: String = str(refs["a"])
	var b_ref: String = str(refs["b"])
	var s: CombatSession = _session(_rt_entry(refs, _enrage_phase(a_ref), [a_ref, b_ref]))
	var got: Array = _count_signals(s)
	var a: Combatant = _enc(s).unit(a_ref, 0)
	var b: Combatant = _enc(s).unit(b_ref, 0)
	var strike: SkillDef = SkillCatalog.find("strike")
	var base_power: float = strike.power_for(a)
	a.hp = a.max_hp / 2
	var r1_a: Dictionary = _intent_for(s, a)
	checks["round 1: not enraged yet"] = not DictUtil.to_bool(r1_a.get("enraged", true)) and not _enc(s).is_enraged(a)
	checks["plan confirmed"] = _confirm(s)
	var mults_before: Array[float] = _attack_mults(s, a)
	_run_block(s)
	s.step_next()
	checks["fired mid-round at [1, 2]"] = _fired_at(s, "crown") == [1, 2]
	checks["empower queued (pending, not active), power %d, rounds max_rounds + 1, source phase:crown" % ENRAGE_POWER] = a.pending_statuses.has(StatusEffects.EMPOWER) and not a.statuses.has(StatusEffects.EMPOWER) and DictUtil.int_of(a.pending_status_magnitudes, StatusEffects.EMPOWER) == ENRAGE_POWER and DictUtil.int_of(a.pending_status_rounds, StatusEffects.EMPOWER) == RUNTIME_ROUNDS + 1 and str(a.pending_status_sources.get(StatusEffects.EMPOWER, "")) == "phase:crown"
	checks["the rest of round 1 keeps its power (factor 1.0, same attack multipliers)"] = is_equal_approx(a.empower_factor(), 1.0) and _attack_mults(s, a) == mults_before and is_equal_approx(strike.power_for(a), base_power)
	var info: Dictionary = DictUtil.dict_of(got[0]) if got.size() == 1 else {}
	checks["one phase_fired signal carrying enrage %d" % ENRAGE_POWER] = got.size() == 1 and DictUtil.to_int(info.get("enrage", 0)) == ENRAGE_POWER
	var events: Array[ActionLogEntry] = _phase_entries(s, 1)
	checks["the phase EVENT says Enraged"] = events.size() == 1 and events[0].status_text.contains("Enraged")
	checks["no hero gains a status from the phase"] = not _any_hero_has(s, StatusEffects.EMPOWER)
	TestHelpers.resolve_until_replay(s, MAX_STEPS)
	s.advance_round()
	checks["round 2: empower active at %d" % ENRAGE_POWER] = a.statuses.has(StatusEffects.EMPOWER) and DictUtil.int_of(a.status_magnitudes, StatusEffects.EMPOWER) == ENRAGE_POWER
	checks["round 2: the boss's power_for is x1.20"] = is_equal_approx(strike.power_for(a), base_power * (1.0 + float(ENRAGE_POWER) / 100.0))
	var r2_a: Dictionary = _intent_for(s, a)
	var actions: int = IntentRules.actions_of(a, s.resolver.scheduled_actions).size()
	checks["round 2: the boss intent is enraged, revealed, full plan (%d actions)" % actions] = DictUtil.to_bool(r2_a.get("enraged", false)) and DictUtil.to_bool(r2_a.get("revealed", false)) and DictUtil.array_of(r2_a.get("plan", [])).size() == actions
	var r2_b: Dictionary = _intent_for(s, b)
	checks["round 2: the other enemy is not enraged"] = not r2_b.is_empty() and not DictUtil.to_bool(r2_b.get("enraged", true)) and DictUtil.to_bool(r2_b["revealed"]) == not DictUtil.array_of(r2_b["readers"]).is_empty()
	checks["round 2: still no hero status from the phase"] = not _any_hero_has(s, StatusEffects.EMPOWER)
	var before: Array = _intent_summary(s)
	var rounds_before: int = DictUtil.int_of(a.status_rounds_left, StatusEffects.EMPOWER)
	s.reset_round()
	checks["reset_round: identical intents (%s)" % str(before)] = _intent_summary(s) == before
	checks["reset_round: empower kept with the same rounds"] = a.statuses.has(StatusEffects.EMPOWER) and DictUtil.int_of(a.status_rounds_left, StatusEffects.EMPOWER) == rounds_before and _enc(s).is_enraged(a)
	s.dispose()
	return TestHelpers.report(TAG, "PACE1 enrage mid-round: queued power, x1.20 from the next planning phase, enraged intents, reset keeps it", checks)

# or_round: the boss stays at 80% -> the phase fires in the or_round intro (empower active at once); below 50% in
# round 3 -> it fires there and never again.
static func _test_pace1_or_round() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _rt_refs()
	if refs.is_empty():
		return TestHelpers.report_ok(TAG, "PACE1 or_round", false, "fewer than two enemy units")
	var a_ref: String = str(refs["a"])
	var phases: Array = _enrage_phase(a_ref, {"or_round": RUNTIME_ROUNDS})
	var s: CombatSession = _session(_rt_entry(refs, phases, [a_ref]))
	var a: Combatant = _enc(s).unit(a_ref, 0)
	var early: bool = false
	for r: int in range(1, RUNTIME_ROUNDS):
		a.hp = (a.max_hp * 4) / 5
		if not _play_round(s):
			break
		early = early or not _fired_at(s, "crown").is_empty()
		s.advance_round()
	checks["not fired in rounds 1..%d at 80%%" % (RUNTIME_ROUNDS - 1)] = not early and s.current_round == RUNTIME_ROUNDS
	var intro: Array[ActionLogEntry] = _phase_entries(s, RUNTIME_ROUNDS)
	checks["fired in round %d's intro [%d, 0]" % [RUNTIME_ROUNDS, RUNTIME_ROUNDS]] = _fired_at(s, "crown") == [RUNTIME_ROUNDS, 0] and intro.size() == 1 and intro[0].block_num == 0
	checks["fired at a round start: empower active for this planning phase, intent enraged"] = a.statuses.has(StatusEffects.EMPOWER) and DictUtil.to_bool(_intent_for(s, a).get("enraged", false))
	s.dispose()
	var s2: CombatSession = _session(_rt_entry(refs, phases, [a_ref]))
	var a2: Combatant = _enc(s2).unit(a_ref, 0)
	for r: int in range(1, RUNTIME_ROUNDS):
		if s2.current_round == 3:
			a2.hp = (a2.max_hp * 2) / 5   # below 50%: due after block 1
		elif _fired_at(s2, "crown").is_empty():
			a2.hp = (a2.max_hp * 4) / 5
		if not _play_round(s2):
			break
		s2.advance_round()
	var at: Array = _fired_at(s2, "crown")
	checks["below 50%% in round 3: fired in round 3 (%s)" % str(at)] = at.size() == 2 and DictUtil.to_int(at[0]) == 3
	checks["...and never again (one phase entry by round %d, nothing due)" % RUNTIME_ROUNDS] = _phase_entries(s2).size() == 1 and _enc(s2).phase_runtime.due.is_empty() and s2.current_round == RUNTIME_ROUNDS
	s2.dispose()
	return TestHelpers.report(TAG, "PACE1 or_round: due at its round start unless hp_below came first; fires once", checks)

# Spec 3.5 warning: a boss fight without an enrage phase warns (catalog warnings_for, world encounter_warnings).
static func _test_pace1_boss_warning() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _catalog_refs()
	var boss: String = ""
	var plain: String = ""
	for u: Dictionary in ContentCatalog.entries("units"):
		if str(u.get("side", "")) != "enemy":
			continue
		var tier: String = str(ContentSchema.normalize("units", u).get("tier", ""))
		if tier == EncounterPhases.PACE_BOSS and boss.is_empty():
			boss = str(u["id"])
		elif tier != EncounterPhases.PACE_BOSS and tier != EncounterPhases.PACE_ELITE and plain.is_empty():
			plain = str(u["id"])
	if refs.is_empty() or boss.is_empty() or plain.is_empty():
		return TestHelpers.report_ok(TAG, "PACE1 boss warning", false, "no boss-tier and plain enemy units")
	var needle: String = "Boss fight 'pace1_warn' has no enrage phase (PACE1)."
	var bare: Dictionary = _rt_entry(refs, [], [boss], "pace1_warn")
	checks["boss fight without enrage warns"] = ContentCatalog.warnings_for("encounters", bare).has(needle)
	checks["the Forge preview shows Pace boss and the warning"] = ForgePreview.describe("encounters", bare).contains("Pace:[/color] boss (auto)") and ForgePreview.describe("encounters", bare).contains(needle)
	checks["boss fight with an enrage phase is clean"] = ContentCatalog.warnings_for("encounters", _rt_entry(refs, _enrage_phase(boss), [boss], "pace1_warn")).is_empty()
	var as_mob: Dictionary = bare.duplicate(true)
	as_mob["pace"] = EncounterPhases.PACE_MOB
	checks["a boss fight set to pace mob does not warn"] = ContentCatalog.warnings_for("encounters", as_mob).is_empty()
	checks["a plain fight does not warn"] = ContentCatalog.warnings_for("encounters", _rt_entry(refs, [], [plain], "pace1_warn")).is_empty()
	var phase_boss: Array = [{"id": "w", "name": "W", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": [boss]}]}]
	checks["a boss in a phase wave makes it a boss fight"] = ContentCatalog.warnings_for("encounters", _rt_entry(refs, phase_boss, [plain], "pace1_warn")).has(needle)
	# World: a character whose base unit is boss tier, in a sandbox fork.
	var wrefs: Dictionary = _world_refs()
	var wpath: String = TestHelpers.sandbox("pace1_warn_world")
	if not wrefs.is_empty() and TestHelpers.fork_worlds(wpath):
		var world: String = str(wrefs["world"])
		var made: Array[String] = WorldCatalog.upsert(world, "characters", {"id": "pace1_warlord", "name": "Pace Warlord", "role": "enemy", "base_unit": boss})
		var stored: Array[String] = WorldCatalog.upsert(world, "encounters", _entry(wrefs, [], ["pace1_warlord"]).merged({"id": "pace1_warn"}, true))
		checks["world fixtures stored (%s | %s)" % ["; ".join(made), "; ".join(stored)]] = made.is_empty() and stored.is_empty()
		checks["world encounter_warnings lists it with its qid"] = WorldCatalog.encounter_warnings(world).has("%s:pace1_warn: %s" % [world, needle])
		checks["world warnings_all carries it"] = TestHelpers.any_contains(JsonUtil.strings(WorldCatalog.warnings_all().get(world, [])), needle)
	else:
		checks["world fork"] = false
	TestHelpers.cleanup_worlds(wpath)
	return TestHelpers.report(TAG, "PACE1 boss warning: boss fights (waves included) without an enrage phase warn in both catalogs", checks)

const START_FLAG: String = "pace1.test_saved"
const START_TEXT: String = "A friend rides ahead. The Wardens strike first."
const START_MAG: int = 10
const START_ROUNDS: int = 2

static func _start_entries(s: CombatSession) -> Array[ActionLogEntry]:
	var out: Array[ActionLogEntry] = []
	for e: ActionLogEntry in s.log_entries:
		if e.skill_subtype == WorldEncounter.START_SUBTYPE:
			out.append(e)
	return out

# Every ally (heroes and guests) of `s` has an active empower at START_MAG from the start source.
static func _all_allies_empowered(s: CombatSession, source: String) -> bool:
	if s.allies.is_empty():
		return false
	for c: Combatant in s.allies:
		if not c.statuses.has(StatusEffects.EMPOWER) or DictUtil.int_of(c.status_magnitudes, StatusEffects.EMPOWER) != START_MAG or str(c.status_sources.get(StatusEffects.EMPOWER, "")) != source:
			return false
	return true

static func _no_one_empowered(s: CombatSession) -> bool:
	for c: Combatant in s.allies + s.enemies:
		if StatusEffects.has_any(c, StatusEffects.EMPOWER):
			return false
	return true

# A started session over a GameEncounter of `gs`, HP boosted.
static func _game_session(gs: GameState, qid: String) -> CombatSession:
	var built: Dictionary = BattleBridge.encounter_for(gs, qid)
	var raw: Variant = built.get("encounter", null)
	if not (raw is GameEncounter):
		return null
	var enc: GameEncounter = raw
	var s := CombatSession.new(enc)
	s.start()
	for c: Combatant in s.allies + s.enemies:
		c.max_hp *= HP_BOOST
		c.hp = c.max_hp
	return s

# Rule 7: a start buff with its flag set empowers every hero and guest in round 1 planning (one EVENT with the text)
# and ends after its rounds; with the flag unset, or without a game state, nobody is. Reset and a re-run match.
static func _test_pace1_start_buffs() -> bool:
	var checks: Dictionary = {}
	var crefs: Dictionary = _catalog_refs()
	var wrefs: Dictionary = _world_refs()
	if crefs.is_empty() or wrefs.is_empty():
		return TestHelpers.report_ok(TAG, "PACE1 start buffs", false, "no fixture refs")
	var world: String = str(wrefs["world"])
	var path: String = TestHelpers.sandbox("pace1_start_game")
	if not TestHelpers.fork_worlds(path):
		TestHelpers.cleanup_worlds(path)
		return TestHelpers.report_ok(TAG, "PACE1 start buffs", false, "world fork failed")
	var helper: Array[String] = WorldCatalog.upsert(world, "characters", {"id": "pace1_helper", "name": "Pace Helper", "role": "npc", "base_unit": crefs["party"]})
	var entry: Dictionary = _entry(wrefs, [])
	entry["id"] = "pace1_start"
	entry["max_rounds"] = MAX_ROUNDS
	entry["guests"] = ["pace1_helper"]
	entry["start_status"] = [{"status": StatusEffects.EMPOWER, "rounds": START_ROUNDS, "magnitude": START_MAG, "requires_flags": [{"key": START_FLAG, "value": true}], "text": START_TEXT}]
	var stored: Array[String] = WorldCatalog.upsert(world, "encounters", entry)
	checks["fixtures stored (%s | %s)" % ["; ".join(helper), "; ".join(stored)]] = helper.is_empty() and stored.is_empty()
	var made: Dictionary = NewGame.create(world, [str(crefs["party"])], 1)
	var gs: GameState = GameState.of(made.get("state", null))
	var source: String = WorldEncounter.START_SOURCE + "pace1_start"
	if gs == null:
		checks["game created (%s)" % str(made.get("errors", []))] = false
		TestHelpers.cleanup_worlds(path)
		return TestHelpers.report(TAG, "PACE1 start buffs", checks)
	# Flag unset: nobody.
	var unset: CombatSession = _game_session(gs, "pace1_start")
	checks["flag unset: nobody is empowered, no start entry"] = unset != null and _no_one_empowered(unset) and _start_entries(unset).is_empty()
	if unset != null:
		unset.dispose()
	# No game state (a WorldEncounter, as the Forge and the sim default): nobody.
	var bare := CombatSession.new(WorldEncounter.new(world, "pace1_start"))
	bare.start()
	checks["no game state: nobody is empowered"] = _no_one_empowered(bare) and _start_entries(bare).is_empty()
	bare.dispose()
	# Flag set: every hero and the guest, in round 1 planning.
	gs.set_flag(START_FLAG, true)
	var s: CombatSession = _game_session(gs, "pace1_start")
	if s == null:
		checks["game encounter built"] = false
		TestHelpers.cleanup_worlds(path)
		return TestHelpers.report(TAG, "PACE1 start buffs", checks)
	var enc: GameEncounter = s.encounter as GameEncounter
	checks["the battle has a guest"] = enc.guests.size() == 1 and s.allies.has(enc.guests[0])
	checks["flag set: every hero and guest is Empowered %d in round 1 planning" % START_MAG] = s.is_planning() and s.current_round == 1 and _all_allies_empowered(s, source)
	checks["no enemy is empowered"] = s.enemies.all(func(c: Combatant) -> bool: return not StatusEffects.has_any(c, StatusEffects.EMPOWER))
	var starts: Array[ActionLogEntry] = _start_entries(s)
	checks["one EVENT entry carries the text"] = starts.size() == 1 and starts[0].detail_text == START_TEXT and starts[0].round_num == 1 and starts[0].block_num == 0
	checks["its rounds count round 1 (%d left)" % START_ROUNDS] = DictUtil.int_of(s.allies[0].status_rounds_left, StatusEffects.EMPOWER) == START_ROUNDS
	var r1_intents: Array = _intent_summary(s)
	s.reset_round()   # round 1: a full restart (a fresh build, so HP is boosted again)
	for c: Combatant in s.allies + s.enemies:
		c.max_hp *= HP_BOOST
		c.hp = c.max_hp
	checks["round 1 reset: granted again once, same intents"] = _all_allies_empowered(s, source) and _start_entries(s).size() == 1 and _intent_summary(s) == r1_intents
	var played: bool = _play_round(s)
	s.advance_round()
	checks["round 2: still empowered"] = played and _all_allies_empowered(s, source)
	var r2_intents: Array = _intent_summary(s)
	s.reset_round()
	checks["round 2 reset: still empowered, same intents"] = _all_allies_empowered(s, source) and _intent_summary(s) == r2_intents
	played = _play_round(s)
	s.advance_round()
	checks["round 3: the start buff has ended"] = played and s.allies.all(func(c: Combatant) -> bool: return not c.statuses.has(StatusEffects.EMPOWER))
	s.dispose()
	# A re-run with the same game state and plans gives the same round records (start buff included).
	var run_a: Array = _two_round_record(gs)
	var run_b: Array = _two_round_record(gs)
	checks["a re-run gives identical log and status records"] = not run_a.is_empty() and run_a == run_b
	TestHelpers.cleanup_worlds(path)
	return TestHelpers.report(TAG, "PACE1 start buffs: flag-gated, every hero and guest, one EVENT, ends after its rounds; reset and re-run match", checks)

# [actor, skill, outcome, value] per log entry plus each ally's empower rounds, over two played rounds.
static func _two_round_record(gs: GameState) -> Array:
	var s: CombatSession = _game_session(gs, "pace1_start")
	if s == null:
		return []
	var out: Array = []
	for r: int in range(2):
		if not _play_round(s):
			break
		for c: Combatant in s.allies:
			out.append([c.name, DictUtil.int_of(c.status_rounds_left, StatusEffects.EMPOWER)])
		if s.can_advance_round():
			s.advance_round()
	for e: ActionLogEntry in s.log_entries:
		out.append([e.round_num, e.block_num, e.actor_name, e.skill_name, e.outcome_tag, e.value_num])
	s.dispose()
	return out
