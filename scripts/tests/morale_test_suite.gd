class_name MoraleTestSuite
extends RefCounted

# ==============================================================================
# MORALE1-L: enemy morale (EncounterMorale).
# Stage 1 (schema): the five fields and their defaults in both schemas, old entries normalising to
# morale false, every validation message of spec 1.1 in both catalogs (and none while morale is off),
# valid entries clean in both catalogs, the leader rule (rule 2), the default group (rule 3) and the
# World Forge preview line (rule 16). Fixture refs are picked by rule (side / role / tier); world
# writes go to a sandbox fork only. Nothing depends on shipped numbers.
# Stage 2 (behaviour, section 2): the state table and flee bonus, the crit hook, losses, breaks and
# surrenders, flee drafts per state, cascades, leaders, fearless units, waves, intents, reset and
# morale-off identity. Sessions use ProbeEncounter: in-memory fixture units cloned from a class-based
# catalog unit (no catalog writes), enemies idle unless a test lets them draft, kills set by hand
# (hp 0) and crits recorded through CatalogEncounter.record_crit, so no result depends on AI rolls.
# ==============================================================================

const TAG: String = "Morale"
const MAX_STEPS: int = 600
const EXIT: Vector2i = Vector2i(11, 4)

# A catalog encounter over in-memory fixture units; enemies plan only from round `ai_from`.
class ProbeEncounter extends CatalogEncounter:
	var fixtures: Dictionary = {}
	var ai_from: int = 99
	func _unit_entry(ref: String) -> Dictionary:
		var raw: Variant = fixtures.get(ref, null)
		if raw is Dictionary:
			var e: Dictionary = raw
			return e.duplicate(true)
		return {}
	func schedule_round(resolver: CombatResolver, round_num: int) -> void:
		if round_num >= ai_from:
			super(resolver, round_num)

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
	if _fixtures().is_empty():
		return TestHelpers.report_ok(TAG, "Behaviour fixtures", false, "no class-based party and enemy units in the catalog")
	ok = _test_rule_tables() and ok
	ok = _test_crit_hook() and ok
	ok = _test_values() and ok
	ok = _test_surrender_drops_actions() and ok
	ok = _test_round_end_announcement() and ok
	ok = _test_flee_bonus() and ok
	ok = _test_cascade_and_small_packs() and ok
	ok = _test_leader() and ok
	ok = _test_fearless() and ok
	ok = _test_wave() and ok
	ok = _test_intents_and_lost() and ok
	ok = _test_reset() and ok
	ok = _test_break_xp() and ok
	ok = _test_off_identical() and ok
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

# ------------------------------------------------------------------------------
# Behaviour fixtures (stage 2)
# ------------------------------------------------------------------------------

# In-memory units cloned from the first class-based (derived) party and enemy catalog units; {} when none.
static func _fixtures() -> Dictionary:
	var hero: Dictionary = {}
	var foe: Dictionary = {}
	for u: Dictionary in ContentCatalog.entries("units"):
		var n: Dictionary = ContentSchema.normalize("units", u)
		if str(n.get("class_id", "")).is_empty() or str(n.get("stat_mode", "derived")) != "derived":
			continue
		if str(n["side"]) == "party" and hero.is_empty():
			hero = n
		elif str(n["side"]) == "enemy" and foe.is_empty():
			foe = n
	if hero.is_empty() or foe.is_empty():
		return {}
	var out: Dictionary = {"probe_hero": _clone(hero, "probe_hero", "Probe Hero", {"side": "party"})}
	var specs: Dictionary = {
		"probe_minion": ["Probe Minion", "minion", 0.25, false],
		"probe_runner": ["Probe Runner", "minion", 0.5, false],
		"probe_captain": ["Probe Captain", "minion", 0.25, false],
		"probe_target": ["Probe Target", "minion", 0.25, false],
		"probe_boss": ["Probe Boss", "boss", 0.25, false],
		"probe_elite": ["Probe Elite", "elite", 0.25, false],
		"probe_berserker": ["Probe Berserker", "minion", 0.0, true],
		"probe_stoic": ["Probe Stoic", "minion", 0.0, false],
	}
	for ref: Variant in specs:
		var spec: Array = specs[ref]
		out[str(ref)] = _clone(foe, str(ref), str(spec[0]), {"side": "enemy", "tier": spec[1], "flee_ratio": spec[2], "reckless": spec[3]})
	return out

static func _clone(base: Dictionary, id: String, unit_name: String, changes: Dictionary) -> Dictionary:
	var e: Dictionary = base.duplicate(true)
	e["id"] = id
	e["name"] = unit_name
	e.merge(changes, true)
	return e

# A started session: one sturdy hero against `enemies` (fixture refs), morale on at break 50, exit (11, 4).
static func _session(enemies: Array, changes: Dictionary = {}, ai_from: int = 99) -> CombatSession:
	var entry: Dictionary = {
		"id": "morale_rt", "name": "Morale Probe", "zone": "stone_arena", "max_rounds": 6, "arena": "CLOSED",
		"exits": [[EXIT.x, EXIT.y]], "party": ["probe_hero"], "enemies": enemies, "morale": true, "morale_break": 50,
	}
	entry.merge(changes, true)
	var enc := ProbeEncounter.new()
	enc.fixtures = _fixtures()
	enc.ai_from = ai_from
	enc.encounter_id = "morale_rt"
	enc.apply_definition(entry)
	var s := CombatSession.new(enc)
	s.start()
	for a: Combatant in s.allies:
		a.max_hp = 99999
		a.hp = 99999
	return s

static func _enc(s: CombatSession) -> ProbeEncounter:
	return s.encounter as ProbeEncounter

static func _u(s: CombatSession, ref: String, n: int = 0) -> Combatant:
	return _enc(s).unit(ref, n)

static func _group(s: CombatSession, gi: int) -> Dictionary:
	var groups: Array[Dictionary] = _enc(s).morale_runtime.groups
	return groups[gi] if gi >= 0 and gi < groups.size() else {}

static func _value(s: CombatSession, gi: int = 0) -> int:
	return DictUtil.to_int(_group(s, gi).get("value", -1))

static func _state(s: CombatSession, gi: int = 0) -> String:
	return _enc(s).morale_runtime.group_state(gi)

# Steps until the current block has resolved (the next block is up, or the round concluded).
static func _run_block(s: CombatSession) -> void:
	if s.phase == CombatSession.Phase.PLANNING and not s.confirm_plan(false):
		return
	var b: int = s.current_block
	var r: int = s.current_round
	for i: int in range(MAX_STEPS):
		s.step_next()
		if s.phase == CombatSession.Phase.REPLAY or s.current_block != b or s.current_round != r:
			return

static func _finish_round(s: CombatSession) -> void:
	if s.phase == CombatSession.Phase.PLANNING and not s.confirm_plan(false):
		return
	TestHelpers.resolve_until_replay(s, MAX_STEPS)

static func _kill(units: Array[Combatant]) -> void:
	for c: Combatant in units:
		if c != null:
			c.hp = 0

# Sets `c` to the largest hp whose fraction is at most `pct` (true when that fraction is above `floor_pct`).
static func _hp_at(c: Combatant, pct: float, floor_pct: float) -> bool:
	c.hp = maxi(1, floori(float(c.max_hp) * pct))
	var ratio: float = float(c.hp) / float(c.max_hp)
	return ratio <= pct and ratio > floor_pct

# Sets `c` to the smallest hp whose fraction is above `pct`.
static func _hp_above(c: Combatant, pct: float) -> bool:
	c.hp = floori(float(c.max_hp) * pct) + 1
	return float(c.hp) / float(c.max_hp) > pct and c.hp <= c.max_hp

static func _kind(s: CombatSession, c: Combatant) -> String:
	return str(s.intent_of(c).get("kind", ""))

static func _morale_entries(s: CombatSession) -> Array[ActionLogEntry]:
	var out: Array[ActionLogEntry] = []
	for e: ActionLogEntry in s.log_entries:
		if e.skill_subtype == EncounterMorale.SUBTYPE:
			out.append(e)
	return out

# ------------------------------------------------------------------------------
# Behaviour tests (stage 2)
# ------------------------------------------------------------------------------

static func _test_rule_tables() -> bool:
	var checks: Dictionary = {}
	var b50: Dictionary = {100: "steady", 84: "steady", 83: "shaken", 67: "shaken", 66: "wavering", 51: "wavering", 50: "broken", 0: "broken"}
	for v: Variant in b50:
		checks["B 50: %d is %s" % [DictUtil.to_int(v), str(b50[v])]] = EncounterMorale.state_for(DictUtil.to_int(v), 50) == str(b50[v])
	var b0: Dictionary = {67: "steady", 66: "shaken", 34: "shaken", 33: "wavering", 1: "wavering", 0: "broken"}
	for v: Variant in b0:
		checks["B 0: %d is %s" % [DictUtil.to_int(v), str(b0[v])]] = EncounterMorale.state_for(DictUtil.to_int(v), 0) == str(b0[v])
	checks["steady adds nothing"] = is_equal_approx(EncounterMorale.flee_for(0.25, "steady"), 0.25)
	checks["shaken adds FLEE_SHAKEN"] = is_equal_approx(EncounterMorale.flee_for(0.25, "shaken"), 0.25 + EncounterMorale.FLEE_SHAKEN)
	checks["wavering adds FLEE_WAVERING"] = is_equal_approx(EncounterMorale.flee_for(0.25, "wavering"), 0.25 + EncounterMorale.FLEE_WAVERING)
	checks["capped at FLEE_CAP"] = is_equal_approx(EncounterMorale.flee_for(0.5, "wavering"), EncounterMorale.FLEE_CAP)
	checks["never-flee stays 0"] = is_equal_approx(EncounterMorale.flee_for(0.0, "wavering"), 0.0)
	checks["a ratio above the cap is not lowered"] = is_equal_approx(EncounterMorale.flee_for(0.7, "shaken"), 0.7)
	return TestHelpers.report(TAG, "State table (rule 4) and flee bonus (rule 8)", checks)

# Rule 6 hook: crits are read from the resolved command (attack, area attack, provoked ripostes).
static func _test_crit_hook() -> bool:
	var checks: Dictionary = {}
	var a := Combatant.new("Hook A", 10, 1)
	var b := Combatant.new("Hook B", 10, 1)
	var attack := AttackCommand.new(a, b)
	attack.last_crit = true
	attack.last_target = b
	var one: Array[Dictionary] = EncounterMorale.crit_events(attack)
	checks["attack crit on a living target"] = one.size() == 1 and one[0]["unit"] == b and not DictUtil.to_bool(one[0]["killed"])
	b.hp = 0
	checks["attack crit that defeats"] = DictUtil.to_bool(EncounterMorale.crit_events(attack)[0]["killed"])
	attack.last_crit = false
	checks["no crit, no event"] = EncounterMorale.crit_events(attack).is_empty()
	var cells: Array[Vector2i] = []
	var area := AreaAttackCommand.new(a, null, cells, 1.0)
	area.crit_hits = [{"unit": b, "killed": true}]
	area.reactions = [{"attacker": a, "crit": true, "defeated": false}, {"attacker": a, "crit": false, "defeated": false}]
	var events: Array[Dictionary] = EncounterMorale.crit_events(area)
	checks["area crits then riposte crits, in order (%d)" % events.size()] = events.size() == 2 and events[0]["unit"] == b and events[1]["unit"] == a
	var s: CombatSession = _session(["probe_minion", "probe_minion", "probe_minion"])
	var m: Combatant = _u(s, "probe_minion")
	var hit := AttackCommand.new(s.allies[0], m)
	hit.last_crit = true
	hit.last_target = m
	_enc(s).on_action_resolved(CombatResolver.ScheduledAction.new(s.allies[0], hit, 1, 1))
	checks["the encounter records the crit as pending"] = _enc(s).morale_runtime.pending.size() == 1
	_run_block(s)
	checks["applied at the block end: %d" % _value(s)] = _value(s) == EncounterMorale.START - EncounterMorale.CRIT_HIT and _enc(s).morale_runtime.pending.is_empty()
	s.dispose()
	return TestHelpers.report(TAG, "Crit hook: CombatSession -> on_action_resolved -> EncounterMorale.crit_events", checks)

static func _test_values() -> bool:
	var checks: Dictionary = {}
	var pack: Array = ["probe_minion", "probe_minion", "probe_minion"]
	var per: int = (100 + 2) / 3
	# one plain kill
	var s: CombatSession = _session(pack)
	_kill([_u(s, "probe_minion", 0)])
	_run_block(s)
	checks["plain kill: %d (%s)" % [_value(s), _state(s)]] = _value(s) == 100 - per and _state(s) == "wavering" and s.outcome.is_empty()
	s.dispose()
	# crit kill
	s = _session(pack)
	_enc(s).record_crit(_u(s, "probe_minion", 0), true)
	_kill([_u(s, "probe_minion", 0)])
	_run_block(s)
	checks["crit kill: %d" % _value(s)] = _value(s) == 100 - per - EncounterMorale.CRIT_KILL_EXTRA
	s.dispose()
	# one and two crit hits
	for hits: int in [1, 2]:
		s = _session(pack)
		for i: int in range(hits):
			_enc(s).record_crit(_u(s, "probe_minion", 0), false)
		_run_block(s)
		var want: String = "steady" if hits == 1 else "shaken"
		checks["%d crit hit(s): %d (%s)" % [hits, _value(s), _state(s)]] = _value(s) == 100 - hits * EncounterMorale.CRIT_HIT and _state(s) == want
		s.dispose()
	# two kills break the group; the third surrenders at its tile; won
	s = _session(pack)
	var last: Combatant = _u(s, "probe_minion", 2)
	var tile: Vector2i = s.grid.get_position(last)
	var seen: Array = []
	s.grid.combatant_withdrawn.connect(func(c: Combatant, at: Vector2i) -> void: seen.append([c, at]))
	_kill([_u(s, "probe_minion", 0), _u(s, "probe_minion", 1)])
	_run_block(s)
	var g: Dictionary = _group(s, 0)
	checks["two kills break the group (cause %s)" % str(g.get("cause", ""))] = DictUtil.to_bool(g.get("broken", false)) and str(g.get("cause", "")) == "losses" and str(g.get("at", [])) == str([1, 1])
	checks["the third surrenders at its tile"] = last.is_withdrawn and not s.grid.has_combatant(last) and s.grid.withdrawn_combatants.has(last) and seen.size() == 1 and seen[0][1] == tile
	checks["won at the break block (%s)" % s.outcome] = s.outcome == "won" and s.combat_over and s.current_block == 1
	checks["victory line (%s)" % s.status_text] = s.status_text.contains(EncounterMorale.LINE_ONE_GROUP)
	s.dispose()
	return TestHelpers.report(TAG, "Losses and break (rules 6, 7, 9, 10, 12): kill, crit kill, crit hits, two kills", checks)

# Rule 9: a surrendering unit's later actions this round are dropped; rule 12: the line comes at the next block start.
static func _test_surrender_drops_actions() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(["probe_boss", "probe_minion", "probe_minion", "probe_minion"], {}, 1)
	var last: Combatant = _u(s, "probe_minion", 2)
	var later: int = 0
	for a: CombatResolver.ScheduledAction in s.resolver.scheduled_actions:
		if a.combatant == last and a.start_block >= 2:
			later += 1
	checks["fixture: the last minion has actions after block 1 (%d)" % later] = later > 0
	_kill([_u(s, "probe_minion", 0), _u(s, "probe_minion", 1)])
	_run_block(s)
	checks["broken at block 1 with the boss standing (%d)" % _value(s)] = _value(s) == 50 and last.is_withdrawn and _u(s, "probe_boss").is_active() and s.outcome.is_empty()
	_finish_round(s)
	var ran: int = 0   # its later actions are cancelled like a defeated unit's (the log still notes each as dropped)
	for a: CombatResolver.ScheduledAction in s.resolver.scheduled_actions:
		if a.combatant == last and a.end_block >= 2 and not a.is_cancelled:
			ran += 1
	checks["no later action of the surrendered unit ran (%d)" % ran] = ran == 0 and not s.grid.has_combatant(last)
	var lines: Array[ActionLogEntry] = _morale_entries(s)
	checks["one EVENT line at block 2 (%d)" % lines.size()] = lines.size() == 1 and lines[0].block_num == 2 and lines[0].round_num == 1 and lines[0].category == "EVENT" and lines[0].detail_text == EncounterMorale.LINE_ONE_GROUP
	s.dispose()
	return TestHelpers.report(TAG, "Surrender drops later actions; mid-round line at the next block start", checks)

# Rule 12: a break at the last block is announced in the next round intro; a reset of that round re-logs it once.
static func _test_round_end_announcement() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(["probe_boss", "probe_minion", "probe_minion", "probe_minion"])
	for b: int in range(s.total_blocks - 1):
		_run_block(s)
	_kill([_u(s, "probe_minion", 0), _u(s, "probe_minion", 1)])
	_run_block(s)
	checks["broken at the last block, round over, battle on"] = s.phase == CombatSession.Phase.REPLAY and s.outcome.is_empty() and _u(s, "probe_minion", 2).is_withdrawn
	checks["no line yet"] = _morale_entries(s).is_empty()
	s.advance_round()
	var lines: Array[ActionLogEntry] = _morale_entries(s)
	checks["the line opens round 2 (%d)" % lines.size()] = lines.size() == 1 and lines[0].round_num == 2 and lines[0].block_num == 0
	s.reset_round()
	checks["a reset re-logs it once (%d)" % _morale_entries(s).size()] = _morale_entries(s).size() == 1
	s.dispose()
	return TestHelpers.report(TAG, "Break after the last block: announced in the next round intro", checks)

# Rule 8 through real drafts: Shaken and Wavering minions run sooner, the cap holds, no exit = refused as today.
static func _test_flee_bonus() -> bool:
	var checks: Dictionary = {}
	var pack: Array = ["probe_minion", "probe_minion", "probe_minion"]
	for shaken: bool in [true, false]:
		var s: CombatSession = _session(pack, {}, 2)
		var m: Combatant = _u(s, "probe_minion", 0)
		if shaken:
			_enc(s).record_crit(m, false)
			_enc(s).record_crit(m, false)
		checks["fixture hp 35%% (%s)" % str(shaken)] = _hp_at(m, 0.35, 0.25)
		_finish_round(s)
		s.advance_round()
		var want: bool = shaken
		checks["%s minion at 35%%: flee %s (%s, %s)" % ["shaken" if shaken else "steady", str(want), _state(s), _kind(s, m)]] = (_kind(s, m) == "flee") == want
		s.dispose()
	var w: CombatSession = _session(pack, {}, 2)
	var low: Combatant = _u(w, "probe_minion", 0)
	var mid: Combatant = _u(w, "probe_minion", 1)
	_kill([_u(w, "probe_minion", 2)])
	checks["fixture hp 50% / above 55%"] = _hp_at(low, 0.5, 0.4) and _hp_above(mid, 0.25 + EncounterMorale.FLEE_WAVERING)
	_finish_round(w)
	w.advance_round()
	checks["wavering: 50%% flees (%s), above 55%% does not (%s)" % [_kind(w, low), _kind(w, mid)]] = _state(w) == "wavering" and _kind(w, low) == "flee" and _kind(w, mid) != "flee"
	w.dispose()
	var r: CombatSession = _session(["probe_runner", "probe_runner", "probe_minion"], {}, 2)
	var r58: Combatant = _u(r, "probe_runner", 0)
	var r62: Combatant = _u(r, "probe_runner", 1)
	_kill([_u(r, "probe_minion", 0)])
	checks["fixture hp 58% / above 60%"] = _hp_at(r58, 0.58, 0.5) and _hp_above(r62, EncounterMorale.FLEE_CAP)
	_finish_round(r)
	r.advance_round()
	checks["flee 0.50 wavering: 58%% flees (%s), above the cap does not (%s)" % [_kind(r, r58), _kind(r, r62)]] = _kind(r, r58) == "flee" and _kind(r, r62) != "flee"
	r.dispose()
	var n: CombatSession = _session(pack, {"exits": []}, 2)
	var stuck: Combatant = _u(n, "probe_minion", 0)
	_kill([_u(n, "probe_minion", 2)])
	_hp_at(stuck, 0.5, 0.4)
	_finish_round(n)
	n.advance_round()
	checks["no exit: the flee step is refused (%s)" % _kind(n, stuck)] = _state(n) == "wavering" and _kind(n, stuck) != "flee" and stuck.is_active()
	n.dispose()
	return TestHelpers.report(TAG, "Shaken and Wavering run sooner (rule 8)", checks)

# Open question 1 (A): a runner costs its group the down loss; packs of 2; B 0.
static func _test_cascade_and_small_packs() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(["probe_minion", "probe_minion", "probe_minion"])
	var runner: Combatant = _u(s, "probe_minion", 0)
	var last: Combatant = _u(s, "probe_minion", 1)
	_kill([_u(s, "probe_minion", 2)])
	_run_block(s)
	var cell: Vector2i = EXIT + Vector2i(-1, 0)
	var blocker: Combatant = s.grid.get_combatant_at(cell)
	if blocker != null and blocker != runner:
		s.grid.combatant_positions[blocker] = s.grid.get_position(runner)
	s.grid.combatant_positions[runner] = cell
	s.grid.move_combatant(runner, Vector2i(1, 0))
	checks["the runner walked off the exit"] = runner.is_withdrawn
	_run_block(s)
	var per: int = (100 + 2) / 3
	checks["66 -> %d: broken, the last surrenders, won (%s)" % [_value(s), s.outcome]] = _value(s) == maxi(0, 100 - 2 * per) and last.is_withdrawn and s.outcome == "won"
	s.dispose()
	var two: CombatSession = _session(["probe_minion", "probe_minion"])
	_kill([_u(two, "probe_minion", 0)])
	_run_block(two)
	checks["pack of 2: the first kill wins (%s)" % two.outcome] = two.outcome == "won" and _u(two, "probe_minion", 1).is_withdrawn
	two.dispose()
	var b0: CombatSession = _session(["probe_minion", "probe_minion", "probe_minion"], {"morale_break": 0})
	_kill([_u(b0, "probe_minion", 0), _u(b0, "probe_minion", 1)])
	_run_block(b0)
	checks["B 0: two of three down does not break (%d)" % _value(b0)] = _value(b0) > 0 and not DictUtil.to_bool(_group(b0, 0).get("broken", true)) and b0.outcome.is_empty()
	b0.dispose()
	return TestHelpers.report(TAG, "Flee cascade, pack of 2, break 0", checks)

static func _test_leader() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(["probe_captain", "probe_minion", "probe_minion"], {"morale_leader": "probe_captain"})
	var captain: Combatant = _u(s, "probe_captain")
	_kill([captain])
	_run_block(s)
	var line: String = EncounterMorale.LINE_LEADER % captain.name
	checks["the leader's fall breaks its group (cause %s)" % str(_group(s, 0).get("cause", ""))] = str(_group(s, 0).get("cause", "")) == "leader" and s.outcome == "won"
	checks["leader line in the victory status (%s)" % s.status_text] = s.status_text.contains(line)
	s.dispose()
	var two: CombatSession = _session(["probe_captain", "probe_minion", "probe_minion", "probe_minion"], {"morale_leader": "probe_captain", "morale_groups": [[0, 1], [2, 3]]})
	_kill([_u(two, "probe_captain")])
	_run_block(two)
	checks["own group broken, the other at %d (%s) fights on" % [_value(two, 1), _state(two, 1)]] = DictUtil.to_bool(_group(two, 0).get("broken", false)) and _value(two, 1) == 100 - EncounterMorale.LEADER_DOWN and _state(two, 1) == "wavering" and two.outcome.is_empty() and _u(two, "probe_minion", 1).is_active()
	_run_block(two)
	var lines: Array[ActionLogEntry] = _morale_entries(two)
	checks["leader line logged at block 2 by the leader"] = lines.size() == 1 and lines[0].detail_text == EncounterMorale.LINE_LEADER % _u(two, "probe_captain").name and lines[0].actor_name == _u(two, "probe_captain").name
	two.dispose()
	var auto: CombatSession = _session(["probe_minion", "probe_boss"])
	checks["auto leader: the boss (%s)" % _enc(auto).morale_runtime.leader] = _enc(auto).morale_runtime.leader == "probe_boss#0"
	auto.dispose()
	var none: CombatSession = _session(["probe_minion", "probe_minion"])
	checks["no boss or elite: no leader"] = _enc(none).morale_runtime.leader.is_empty()
	none.dispose()
	return TestHelpers.report(TAG, "Leader (rules 2, 6): explicit, two groups, auto boss", checks)

static func _test_fearless() -> bool:
	var checks: Dictionary = {}
	var roster: Array = ["probe_boss", "probe_elite", "probe_berserker", "probe_stoic", "probe_minion", "probe_minion"]
	var s: CombatSession = _session(roster)
	var enc: ProbeEncounter = _enc(s)
	var fearless: Array[Combatant] = [_u(s, "probe_boss"), _u(s, "probe_elite"), _u(s, "probe_berserker"), _u(s, "probe_stoic")]
	for c: Combatant in fearless:
		checks["%s is fearless" % c.name] = enc.morale_state_of(c) == EncounterMorale.FEARLESS
	checks["a minion is steady"] = enc.morale_state_of(_u(s, "probe_minion")) == "steady"
	_kill([_u(s, "probe_minion", 0)])
	_run_block(s)
	enc._apply_phase_profiles(s.current_round)
	var minion_flee: float = enc.profile_of(_u(s, "probe_minion", 1)).flee_ratio
	checks["shaken group: the minion drafts with %.2f" % minion_flee] = _state(s) == "shaken" and is_equal_approx(minion_flee, 0.25 + EncounterMorale.FLEE_SHAKEN)
	for c: Combatant in fearless:
		var authored: AiProfile = enc._authored_profiles[c]
		checks["%s drafts with its authored flee %.2f" % [c.name, authored.flee_ratio]] = is_equal_approx(enc.profile_of(c).flee_ratio, authored.flee_ratio)
	_kill([_u(s, "probe_minion", 1), _u(s, "probe_stoic"), _u(s, "probe_berserker")])
	_run_block(s)
	var per: int = (100 + 5) / 6
	checks["fearless deaths count: %d" % _value(s)] = _value(s) == 100 - 4 * per and DictUtil.to_bool(_group(s, 0).get("broken", false))
	checks["boss and elite never surrender; the battle goes on"] = _u(s, "probe_boss").is_active() and _u(s, "probe_elite").is_active() and s.outcome.is_empty()
	s.dispose()
	var slay: CombatSession = _session(["probe_target", "probe_minion", "probe_minion", "probe_minion"], {"objective": "slay", "objective_unit": "probe_target"})
	var target: Combatant = _u(slay, "probe_target")
	checks["the slay target is fearless"] = _enc(slay).morale_state_of(target) == EncounterMorale.FEARLESS
	_kill([_u(slay, "probe_minion", 0), _u(slay, "probe_minion", 1)])
	_run_block(slay)
	checks["slay: the minion surrenders, the target fights on (%s)" % slay.outcome] = _u(slay, "probe_minion", 2).is_withdrawn and target.is_active() and slay.outcome.is_empty()
	_kill([target])
	_run_block(slay)
	checks["slay: won when the target falls (%s)" % slay.outcome] = slay.outcome == "won"
	slay.dispose()
	var open: CombatSession = _session(["probe_minion", "probe_minion", "probe_minion"], {"arena": "OPEN", "exits": []})
	_kill([_u(open, "probe_minion", 0), _u(open, "probe_minion", 1)])
	_run_block(open)
	checks["OPEN arena: the break surrenders the same way (%s)" % open.outcome] = _u(open, "probe_minion", 2).is_withdrawn and open.outcome == "won"
	open.dispose()
	return TestHelpers.report(TAG, "Fearless (rule 5): boss, elite, reckless, flee 0, slay target; OPEN arena", checks)

# Rule 3: a wave after a break forms a fresh group and must be beaten.
static func _test_wave() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(["probe_minion", "probe_minion", "probe_minion"], {"wave_units": ["probe_minion"], "wave_round": 1, "wave_block": 3})
	_kill([_u(s, "probe_minion", 0), _u(s, "probe_minion", 1)])
	_run_block(s)
	checks["broken before the wave; the battle waits for it (%s)" % s.outcome] = _u(s, "probe_minion", 2).is_withdrawn and s.outcome.is_empty()
	_run_block(s)
	_run_block(s)
	_run_block(s)
	var arrival: Combatant = _u(s, "probe_minion", 3)
	checks["the wave is a fresh group (%d groups, %d, %s)" % [_enc(s).morale_runtime.groups.size(), _value(s, 1), _enc(s).morale_state_of(arrival)]] = _enc(s).morale_runtime.groups.size() == 2 and _value(s, 1) == 100 and _enc(s).morale_state_of(arrival) == "steady" and arrival.is_active() and s.grid.has_combatant(arrival)
	checks["still not won"] = s.outcome.is_empty()
	_kill([arrival])
	_run_block(s)
	checks["beaten: won (%s)" % s.outcome] = s.outcome == "won"
	s.dispose()
	return TestHelpers.report(TAG, "A wave after a break is a fresh group at 100", checks)

static func _test_intents_and_lost() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(["probe_minion", "probe_minion", "probe_boss"], {}, 1)
	var records: Array[Dictionary] = s.enemy_intents()
	var ok_fields: bool = not records.is_empty()
	var extra: Array[String] = []
	for r: Dictionary in records:
		var c: Combatant = r["unit"]
		var want: String = EncounterMorale.FEARLESS if c == _u(s, "probe_boss") else "steady"
		ok_fields = ok_fields and str(r.get("morale_state", "x")) == want and DictUtil.to_int(r.get("morale_group", -2)) == 0
		for key: Variant in r:
			var k: String = str(key)
			if (k.contains("morale") and k != "morale_state" and k != "morale_group") or k.contains("flee"):
				extra.append(k)
	checks["intents carry morale_state and morale_group"] = ok_fields
	checks["no value or flee ratio in a record (%s)" % ", ".join(extra)] = extra.is_empty()
	s.dispose()
	var off: CombatSession = _session(["probe_minion", "probe_minion"], {"morale": false}, 1)
	var off_ok: bool = not off.enemy_intents().is_empty()
	for r: Dictionary in off.enemy_intents():
		off_ok = off_ok and str(r.get("morale_state", "x")) == "" and DictUtil.to_int(r.get("morale_group", -2)) == -1
	checks["morale off: '' and -1"] = off_ok
	checks["morale off: no morale key in the objective state"] = not off.encounter.objective_state().has("morale")
	off.dispose()
	var lost: CombatSession = _session(["probe_minion", "probe_minion", "probe_minion"])
	_kill([_u(lost, "probe_minion", 0), _u(lost, "probe_minion", 1)])
	_kill(lost.allies)
	_run_block(lost)
	checks["every hero down in the break block: lost (%s)" % lost.outcome] = lost.outcome == "lost"
	lost.dispose()
	return TestHelpers.report(TAG, "Intent fields (rule 13); every hero down in the break block is lost", checks)

# Rule 15 and rule 11: reset rewinds a break and the same plan breaks again; a break win is `won` like a rout.
static func _test_reset() -> bool:
	var checks: Dictionary = {}
	var s: CombatSession = _session(["probe_minion", "probe_minion", "probe_minion"])
	_finish_round(s)
	s.advance_round()
	var last: Combatant = _u(s, "probe_minion", 2)
	_kill([_u(s, "probe_minion", 0), _u(s, "probe_minion", 1)])
	_run_block(s)
	var first_at: String = str(_group(s, 0).get("at", []))
	checks["broken in round 2 (%s)" % first_at] = first_at == str([2, 1]) and s.outcome == "won"
	s.reset_round()
	checks["reset rewinds the break"] = _value(s) == 100 and not DictUtil.to_bool(_group(s, 0).get("broken", true)) and last.is_active() and s.grid.has_combatant(last) and _enc(s).morale_runtime.surrendered.is_empty()
	_kill([_u(s, "probe_minion", 0), _u(s, "probe_minion", 1)])
	_run_block(s)
	checks["the same plan breaks the same group at the same block (%s)" % str(_group(s, 0).get("at", []))] = str(_group(s, 0).get("at", [])) == first_at and last.is_withdrawn and s.outcome == "won"
	s.dispose()
	return TestHelpers.report(TAG, "Reset and replay (rule 15)", checks)

# Rule 11: a break win gives the same XP award (BattleBridge.apply) as a full rout of the same roster. World sandbox
# fork: a test-only enemy character on a catalog enemy unit that can flee (picked by rule), one encounter with morale
# on and one with it off, each played from its own new game.
static func _test_break_xp() -> bool:
	var checks: Dictionary = {}
	var refs: Dictionary = _world_refs()
	var crefs: Dictionary = _catalog_refs()
	var grunt_unit: String = ""
	for u: Dictionary in ContentCatalog.entries("units"):
		var n: Dictionary = ContentSchema.normalize("units", u)
		if str(n["side"]) == "enemy" and DictUtil.to_float(n["flee_ratio"]) > 0.0 and not DictUtil.to_bool(n["reckless"]) and not EncounterMorale.LEADER_TIERS.has(str(n["tier"])):
			grunt_unit = str(n["id"])
			break
	if refs.is_empty() or crefs.is_empty() or grunt_unit.is_empty():
		return TestHelpers.report_ok(TAG, "Break XP", false, "no world refs or no catalog enemy unit that can flee")
	var world: String = str(refs["world"])
	var path: String = TestHelpers.sandbox("morale_xp")
	if not TestHelpers.fork_worlds(path):
		TestHelpers.cleanup_worlds(path)
		return TestHelpers.report_ok(TAG, "Break XP", false, "world fork failed")
	var grunt: Array[String] = WorldCatalog.upsert(world, "characters", {"id": "morale_xp_grunt", "name": "XP Grunt", "role": "enemy", "base_unit": grunt_unit})
	checks["fixture grunt stored (%s)" % "; ".join(grunt)] = grunt.is_empty()
	var awards: Array[Dictionary] = []
	for morale: bool in [true, false]:
		var id: String = "morale_xp_break" if morale else "morale_xp_rout"
		var stored: Array[String] = WorldCatalog.upsert(world, "encounters", {"id": id, "name": "XP Probe", "zone": "stone_arena", "party": [refs["party"]],
			"enemies": ["morale_xp_grunt", "morale_xp_grunt", "morale_xp_grunt"], "morale": morale})
		checks["%s stored (%s)" % [id, "; ".join(stored)]] = stored.is_empty()
		var made: Dictionary = NewGame.create(world, [str(crefs["party"])], 1)
		var gs: GameState = GameState.of(made.get("state", null))
		var built: Dictionary = BattleBridge.encounter_for(gs, id) if gs != null else {}
		var raw_enc: Variant = built.get("encounter", null)
		if gs == null or not (raw_enc is GameEncounter):
			checks["%s: game and encounter built (%s)" % [id, str(built.get("errors", made.get("errors", [])))]] = false
			continue
		var s := CombatSession.new(raw_enc as GameEncounter)
		s.start()
		var foes: Array[Combatant] = s.enemies.duplicate()
		if morale:
			foes.resize(2)   # two of three down: the third surrenders
		_kill(foes)
		_run_block(s)
		var surrendered: bool = s.enemies.size() == 3 and s.enemies[2].is_withdrawn
		checks["%s: won at block 1 (%s)%s" % [id, BattleBridge.outcome(s), " by a break" if morale else ""]] = BattleBridge.outcome(s) == "won" and s.current_block == 1 and (surrendered or not morale)
		var res: Dictionary = BattleBridge.apply(gs, id, BattleBridge.outcome(s))
		var xp: Dictionary = {}
		var raw_xp: Dictionary = DictUtil.dict_of(res.get("xp", {}))
		for member: Variant in raw_xp:
			xp[str(member)] = DictUtil.to_int(DictUtil.dict_of(raw_xp[member]).get("xp", -1))
		awards.append(xp)
		s.dispose()
	checks["the break award equals the rout award (%s)" % str(awards)] = awards.size() == 2 and not awards[0].is_empty() and str(awards[0]) == str(awards[1])
	TestHelpers.cleanup_worlds(path)
	return TestHelpers.report(TAG, "A break win gives the same XP award as a full rout (rule 11)", checks)

# Rule 1: an entry with morale false (and stray morale_* fields) plays exactly like the same entry without them.
static func _test_off_identical() -> bool:
	var checks: Dictionary = {}
	var base: Dictionary = {}
	for e: Dictionary in ContentCatalog.entries("encounters"):
		var n: Dictionary = ContentSchema.normalize("encounters", e)
		if not DictUtil.array_of(n["party"]).is_empty() and not EncounterMorale.enabled(n):
			base = e
			break
	if base.is_empty():
		return TestHelpers.report_ok(TAG, "Morale off is identical", false, "no catalog encounter with a party")
	var stray: Dictionary = base.duplicate(true)
	stray.merge({"morale": false, "morale_break": 10, "morale_leader": "nobody", "morale_groups": [[0]], "morale_text": "x"}, true)
	var logs: Array[String] = []
	for entry: Dictionary in [base, stray]:
		var s := CombatSession.new(CatalogEncounter.from_entry(entry))
		s.start()
		var text: Array[String] = []
		for r: int in range(2):
			s.plan_apply_preset_all("balanced")
			if not s.confirm_plan(false):
				break
			TestHelpers.resolve_until_replay(s, MAX_STEPS)
			if s.combat_over or not s.can_advance_round():
				break
			s.advance_round()
		for e: ActionLogEntry in s.log_entries:
			text.append("%s|%s|%s|%s|%d" % [e.ref_tag, e.actor_name, e.skill_name, e.outcome_tag, e.damage])
		logs.append("\n".join(text))
		s.dispose()
	checks["%s: the same log with stray morale fields while off" % str(base.get("id", ""))] = logs.size() == 2 and not logs[0].is_empty() and logs[0] == logs[1]
	return TestHelpers.report(TAG, "Morale off plays identically (rule 1)", checks)
