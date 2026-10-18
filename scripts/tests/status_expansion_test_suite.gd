class_name StatusExpansionTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Status Expansion Test Suite (S12S-L)
# Commit 1 (schema): registry magnitude ranges and the six new rows, has_magnitude from the registry,
# the status_magnitude field / sections / validator messages / {magnitude} token, the status source
# dicts (queue, try_inflict, grant_now, roll_over, remove) and round undo restoring statuses,
# magnitudes and sources. Expectations come from the registry rows and the spec rules; skill rows
# that are written go to a sandbox catalog fork.
# ==============================================================================

static var SANDBOX: String = TestHelpers.sandbox("status_expansion_test")
const TAG: String = "StatusExpansion"
const SURE: int = 500   # potency far above any resistance: the chance clamps to 100
const NEW_IDS: Array[String] = ["slow", "expose", "fortify", "regen", "poison", "burn"]

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING STATUS EXPANSION TEST (S12S-L)          ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_registry_rows() and ok
	ok = _test_row_errors() and ok
	ok = _test_skill_schema() and ok
	ok = _test_validation() and ok
	ok = _test_magnitude_token() and ok
	ok = _test_sources() and ok
	ok = _test_cleanse() and ok
	ok = _test_undo() and ok
	return ok

# ------------------------------------------------------------------------------

static func _unit(unit_name: String) -> Combatant:
	var c := Combatant.new(unit_name, 200, 10, 1)
	c.armor_override = 0
	return c

static func _errors(entry: Dictionary) -> String:
	return " | ".join(ContentCatalog.validate_entry("skills", entry, str(entry.get("id", ""))))

static func _range_message(status: String) -> String:
	return "%s needs a magnitude %d-%d" % [status, StatusRegistry.magnitude_min(status), StatusRegistry.magnitude_max(status)]

# The six new rows and the spec table (acceptance 1).
static func _test_registry_rows() -> bool:
	var checks: Dictionary = {}
	checks["row_errors() is empty"] = StatusRegistry.row_errors().is_empty()
	var script: GDScript = StatusEffects
	var consts: Dictionary = script.get_script_constant_map()
	var all_known: bool = true
	for id: String in NEW_IDS:
		all_known = all_known and StatusRegistry.has(id) and consts.values().has(id)
	checks["every new status has a StatusEffects constant and a row"] = all_known
	checks["SLOW_TILES mirrors HASTE_TILES (1)"] = StatusEffects.SLOW_TILES == 1 and StatusEffects.SLOW_TILES == StatusEffects.HASTE_TILES
	# id, kind, resistable, default_rounds, stacking, cap, min, max, unit, display, grant (the spec's placeholder table).
	var table: Array = [
		["weaken", "debuff", true, 2, StatusRegistry.REFRESH_MAX_DURATION, 25, 1, 25, "pct", "Weakened", ""],
		["slow", "debuff", true, 1, StatusRegistry.REFRESH_MAX_DURATION, 0, 0, 0, "pct", "Slowed", ""],
		["expose", "debuff", true, 2, StatusRegistry.REFRESH_MAX_DURATION, 20, 1, 20, "flat", "Exposed", ""],
		["fortify", "buff", false, 2, StatusRegistry.REFRESH_MAX_DURATION, 20, 1, 20, "flat", "Fortified", "support"],
		["regen", "buff", false, 2, StatusRegistry.REFRESH_MAX_DURATION, 10, 1, 10, "pct_max_hp", "Regenerating", "support"],
		["poison", "debuff", true, 3, StatusRegistry.STACK_POTENCY, 30, 1, 15, "flat", "Poisoned", ""],
		["burn", "debuff", true, 2, StatusRegistry.REFRESH_MAX_DURATION, 10, 1, 10, "pct_max_hp", "Burning", ""],
	]
	var rows_ok: bool = true
	var bad: Array[String] = []
	for raw: Variant in table:
		var t: Array = raw
		var r: Dictionary = StatusRegistry.row(str(t[0]))
		var same: bool = str(r.get("kind", "")) == str(t[1]) and DictUtil.to_bool(r.get("resistable", false)) == bool(t[2]) \
			and DictUtil.to_int(r.get("default_rounds", 0)) == int(t[3]) and str(r.get("stacking", "")) == str(t[4]) \
			and DictUtil.to_int(r.get("cap", -1)) == int(t[5]) and DictUtil.to_int(r.get("magnitude_min", -1)) == int(t[6]) \
			and DictUtil.to_int(r.get("magnitude_max", -1)) == int(t[7]) and str(r.get("magnitude_unit", "")) == str(t[8]) \
			and str(r.get("display", "")) == str(t[9]) and str(r.get("grant", "?")) == str(t[10])
		if not same:
			bad.append(str(t[0]))
		rows_ok = rows_ok and same
	checks["weaken and the six new rows match the spec table (%s)" % ", ".join(bad)] = rows_ok
	var others_zero: bool = true
	for id: String in ["stagger", "stun", "root", "bleed", "haste", "taunt", "riposte"]:
		others_zero = others_zero and StatusRegistry.magnitude_min(id) == 0 and StatusRegistry.magnitude_max(id) == 0
	checks["the other seven original rows carry no magnitude (0/0)"] = others_zero
	var derived: bool = true
	for id: String in StatusRegistry.ids():
		derived = derived and StatusEffects.has_magnitude(id) == (StatusRegistry.magnitude_max(id) > 0) and StatusRegistry.UNITS.has(StatusRegistry.magnitude_unit(id))
	checks["has_magnitude is magnitude_max > 0 for every row"] = derived and not StatusEffects.has_magnitude("mystery")
	checks["magnitude_ids lists the ranged statuses in table order"] = StatusRegistry.magnitude_ids() == ["weaken", "expose", "fortify", "regen", "poison", "burn", "empower"]
	checks["fortify and regen join support_buffs; slow, expose, poison, burn join debuffs and resistable"] = StatusEffects.support_buffs().has(StatusEffects.FORTIFY) and StatusEffects.support_buffs().has(StatusEffects.REGEN) \
		and [StatusEffects.SLOW, StatusEffects.EXPOSE, StatusEffects.POISON, StatusEffects.BURN].all(func(id: String) -> bool: return StatusEffects.debuffs().has(id) and StatusEffects.resistable().has(id))
	return TestHelpers.report(TAG, "Registry rows and magnitude ranges (acceptance 1)", checks)

# row_errors' magnitude rules on broken copies of a valid row.
static func _test_row_errors() -> bool:
	var checks: Dictionary = {}
	var good: Dictionary = StatusRegistry.row(StatusEffects.EXPOSE).duplicate()
	var rows: Array[Dictionary] = [good]
	checks["a valid row has no errors"] = StatusRegistry.errors_of(rows).is_empty()
	var cases: Dictionary = {
		"magnitude_min above 100": {"magnitude_min": 101, "magnitude_max": 120},
		"negative magnitude_min": {"magnitude_min": -1},
		"magnitude_max below magnitude_min": {"magnitude_min": 5, "magnitude_max": 4},
		"unknown magnitude_unit": {"magnitude_unit": "tiles"},
	}
	for label: Variant in cases:
		var broken: Dictionary = good.duplicate()
		broken.merge(DictUtil.dict_of(cases[label]), true)
		var one: Array[Dictionary] = [broken]
		checks["%s is an error" % str(label)] = not StatusRegistry.errors_of(one).is_empty()
	var none: Dictionary = good.duplicate()
	none.merge({"magnitude_min": 0, "magnitude_max": 0}, true)
	var none_rows: Array[Dictionary] = [none]
	checks["0/0 (no magnitude) is valid"] = StatusRegistry.errors_of(none_rows).is_empty()
	return TestHelpers.report(TAG, "row_errors checks magnitude fields", checks)

static func _test_skill_schema() -> bool:
	var checks: Dictionary = {}
	var f: Dictionary = ContentSchema.field("skills", "status_magnitude")
	var when: Dictionary = DictUtil.dict_of(f.get("when", {}))
	checks["label 'Status magnitude', range 0..100, default 0"] = str(f.get("label", "")) == "Status magnitude" and DictUtil.to_int(f.get("min", -1)) == 0 and DictUtil.to_int(f.get("max", -1)) == 100 and DictUtil.to_int(f.get("default", -1)) == 0
	checks["no weaken-only condition (shown by kind)"] = str(when.get("key", "")) == "kind"
	checks["ATTACK, SUPPORT and DEFEND have the field; FOCUS does not"] = SkillKinds.kinds_with("status_magnitude") == ["ATTACK", "DEFEND", "SUPPORT"]
	checks["it sits in the inflict and grant sections"] = SkillKinds.sections_with("ATTACK", "status_magnitude") == ["inflict"] and SkillKinds.sections_with("SUPPORT", "status_magnitude") == ["grant"] and SkillKinds.sections_with("DEFEND", "status_magnitude") == ["grant"]
	checks["the magnitude status is status_effect, else grant_status"] = SkillText.magnitude_status({"status_effect": "poison", "grant_status": "regen"}) == "poison" and SkillText.magnitude_status({"status_effect": "", "grant_status": "regen"}) == "regen" and SkillText.magnitude_status({}) == ""
	return TestHelpers.report(TAG, "status_magnitude field and sections", checks)

# Acceptance 2: the range messages, the "only applies" message, a valid SUPPORT regen grant, validate_all.
static func _test_validation() -> bool:
	var checks: Dictionary = {}
	var w: String = StatusEffects.WEAKEN
	var weaken: Dictionary = {"id": "t_sx_w", "name": "W", "kind": "ATTACK", "power_mult": 0.0, "status_effect": w, "status_potency": 70}
	var low: Dictionary = weaken.duplicate()
	low["status_magnitude"] = StatusRegistry.magnitude_min(w) - 1
	var high: Dictionary = weaken.duplicate()
	high["status_magnitude"] = StatusRegistry.magnitude_max(w) + 1
	checks["weaken below / above its range names the range"] = _errors(low).contains(_range_message(w)) and _errors(high).contains(_range_message(w)) and _range_message(w) == "weaken needs a magnitude 1-25"
	var e: String = StatusEffects.EXPOSE
	var expose: Dictionary = {"id": "t_sx_e", "name": "E", "kind": "ATTACK", "power_mult": 1.0, "status_effect": e, "status_potency": 60, "status_magnitude": StatusRegistry.magnitude_max(e) + 1}
	checks["expose above its range: 'expose needs a magnitude 1-20'"] = _errors(expose).contains(_range_message(e)) and _range_message(e) == "expose needs a magnitude 1-20"
	expose["status_magnitude"] = StatusRegistry.magnitude_max(e)
	checks["expose at its max is valid"] = _errors(expose).is_empty()
	var stagger: Dictionary = {"id": "t_sx_s", "name": "S", "kind": "ATTACK", "status_effect": "stagger", "status_potency": 50, "status_magnitude": 5}
	var only: String = "magnitude only applies to %s" % ", ".join(StatusRegistry.magnitude_ids())
	checks["stagger with a magnitude: 'magnitude only applies to ...' listing the ranged statuses"] = _errors(stagger).contains(only) and only.begins_with("magnitude only applies to")
	var plain: Dictionary = {"id": "t_sx_p", "name": "P", "kind": "ATTACK", "status_magnitude": 5}
	checks["a magnitude without a status is rejected the same way"] = _errors(plain).contains(only)
	var r: String = StatusEffects.REGEN
	var regen: Dictionary = {"id": "t_sx_r", "name": "R", "kind": "SUPPORT", "grant_status": r, "status_magnitude": 4, "status_rounds": 2}
	checks["a SUPPORT regen grant with magnitude 4 is valid"] = _errors(regen).is_empty()
	var regen_zero: Dictionary = regen.duplicate()
	regen_zero["status_magnitude"] = 0
	checks["a regen grant without magnitude names the regen range"] = _errors(regen_zero).contains(_range_message(r))
	var haste: Dictionary = {"id": "t_sx_h", "name": "H", "kind": "SUPPORT", "grant_status": StatusEffects.HASTE, "status_magnitude": 3}
	checks["a magnitude on a haste grant is rejected"] = _errors(haste).contains(only)
	var fortify: Dictionary = {"id": "t_sx_f", "name": "F", "kind": "DEFEND", "grant_status": StatusEffects.FORTIFY, "status_magnitude": 8, "mitigation": 0.3}
	checks["a DEFEND fortify grant with a magnitude in range is valid"] = _errors(fortify).is_empty()
	var riposte: Dictionary = {"id": "t_sx_rp", "name": "Rp", "kind": "DEFEND", "grant_status": StatusEffects.RIPOSTE, "status_magnitude": 0}
	checks["a riposte grant without magnitude is valid"] = _errors(riposte).is_empty()
	var poison_rank: Dictionary = {"id": "t_sx_pr", "name": "Pr", "kind": "ATTACK", "status_effect": StatusEffects.POISON, "status_potency": 60, "status_magnitude": StatusRegistry.magnitude_max(StatusEffects.POISON) + 1, "tier": 2}
	checks["a rank 2 row's magnitude problem carries rank 2"] = _has_rank_problem(poison_rank, 2)
	checks["validate_all on the shipped catalog stays {}"] = ContentCatalog.validate_all().is_empty()
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		regen["id"] = "t_sx_regen"
		checks["the regen grant is accepted and reads back"] = ContentCatalog.upsert("skills", regen).is_empty() and ContentFactory.skill(ContentCatalog.get_entry("skills", "t_sx_regen")).status_magnitude == 4
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "Validator magnitude messages (acceptance 2)", checks)

# True when the row has a status_magnitude problem for `rank`.
static func _has_rank_problem(entry: Dictionary, rank: int) -> bool:
	for p: Dictionary in SkillValidator.row_problems(ContentSchema.normalize("skills", entry)):
		if str(p["field"]) == "status_magnitude" and DictUtil.to_int(p["rank"]) == rank:
			return true
	return false

static func _test_magnitude_token() -> bool:
	var checks: Dictionary = {}
	checks["{magnitude} is valid for ATTACK, SUPPORT, DEFEND only"] = SkillText.token_valid_for("magnitude", "ATTACK") and SkillText.token_valid_for("magnitude", "SUPPORT") and SkillText.token_valid_for("magnitude", "DEFEND") and not SkillText.token_valid_for("magnitude", "FOCUS")
	var weaken: Dictionary = {"kind": "ATTACK", "status_effect": "weaken", "status_magnitude": 15}
	var expose: Dictionary = {"kind": "ATTACK", "status_effect": "expose", "status_magnitude": 8}
	var poison: Dictionary = {"kind": "ATTACK", "status_effect": "poison", "status_magnitude": 4}
	var regen: Dictionary = {"kind": "SUPPORT", "grant_status": "regen", "status_magnitude": 5}
	var fortify: Dictionary = {"kind": "DEFEND", "grant_status": "fortify", "status_magnitude": 6}
	checks["weaken renders a percent (15%)"] = SkillText.render("{magnitude}", "ATTACK", weaken, 1) == "15%"
	checks["expose / fortify render armor points"] = SkillText.render("{magnitude}", "ATTACK", expose, 1) == "8 armor" and SkillText.render("{magnitude}", "DEFEND", fortify, 1) == "6 armor"
	checks["poison renders HP (4 HP)"] = SkillText.render("{magnitude}", "ATTACK", poison, 1) == "4 HP"
	checks["regen renders % max HP"] = SkillText.render("{magnitude}", "SUPPORT", regen, 1) == "5% max HP"
	var stagger: Dictionary = {"kind": "ATTACK", "status_effect": "stagger", "status_magnitude": 0}
	checks["a status without a range renders nothing"] = SkillText.render("{magnitude}", "ATTACK", stagger, 1).is_empty()
	var template_entry: Dictionary = {"id": "t_sx_tok", "name": "Tok", "kind": "SUPPORT", "grant_status": "regen", "status_magnitude": 5}
	checks["{magnitude} on a SUPPORT regen template validates"] = SkillText.validate("{name}: {magnitude}", template_entry).is_empty()
	return TestHelpers.report(TAG, "{magnitude} token units", checks)

# Source dicts: queue / try_inflict / grant_now record them, roll_over moves them (newer wins), remove clears them.
static func _test_sources() -> bool:
	var checks: Dictionary = {}
	var c: Combatant = _unit("Target")
	var p: String = StatusEffects.POISON
	StatusEffects.queue(c, p, 2, 10, "Alpha")
	checks["queue records the pending source"] = str(c.pending_status_sources.get(p, "")) == "Alpha"
	StatusEffects.queue(c, StatusEffects.ROOT, 1)
	checks["queue without a source records none"] = not c.pending_status_sources.has(StatusEffects.ROOT)
	var rng := RandomNumberGenerator.new()
	rng.seed = 7
	checks["try_inflict passes the source"] = StatusEffects.try_inflict(c, p, SURE, rng, 1, 25, "Beta") and str(c.pending_status_sources.get(p, "")) == "Beta"
	var stacked: Dictionary = StatusRegistry.stack(2, 10, 1, 25, p)
	checks["poison queue stacks by the registry rule (cap)"] = DictUtil.int_of(c.pending_status_magnitudes, p) == DictUtil.to_int(stacked["potency"]) and DictUtil.int_of(c.pending_status_rounds, p) == DictUtil.to_int(stacked["rounds"])
	StatusEffects.roll_over(c, -1)
	checks["roll_over moves the pending source to active"] = str(c.status_sources.get(p, "")) == "Beta" and c.pending_status_sources.is_empty()
	checks["roll_over keeps the stacked magnitude"] = DictUtil.int_of(c.status_magnitudes, p) == DictUtil.to_int(stacked["potency"])
	StatusEffects.queue(c, p, 1, 1, "Gamma")
	StatusEffects.roll_over(c, -1)
	checks["a newer pending source wins over the carried one"] = str(c.status_sources.get(p, "")) == "Gamma"
	var d: Combatant = _unit("Carry")
	StatusEffects.queue(d, StatusEffects.BURN, 3, 5, "Delta")
	StatusEffects.roll_over(d, -1)
	StatusEffects.roll_over(d, -1)
	checks["a carried multi-round status keeps its source and magnitude"] = d.has_status(StatusEffects.BURN) and str(d.status_sources.get(StatusEffects.BURN, "")) == "Delta" and DictUtil.int_of(d.status_magnitudes, StatusEffects.BURN) == 5
	var e: Combatant = _unit("Self")
	StatusEffects.grant_now(e, StatusEffects.RIPOSTE, 2, 0, "Self")
	checks["grant_now records the source; no magnitude for a status without a range"] = str(e.status_sources.get(StatusEffects.RIPOSTE, "")) == "Self" and not e.status_magnitudes.has(StatusEffects.RIPOSTE)
	StatusEffects.grant_now(e, StatusEffects.FORTIFY, 2, 8, "Self")
	checks["grant_now stores a ranged magnitude"] = DictUtil.int_of(e.status_magnitudes, StatusEffects.FORTIFY) == 8
	StatusEffects.queue(e, StatusEffects.FORTIFY, 2, 4, "Ally")
	checks["StatusEffects.remove clears both source dicts"] = StatusEffects.remove(e, StatusEffects.FORTIFY) and not e.status_sources.has(StatusEffects.FORTIFY) and not e.pending_status_sources.has(StatusEffects.FORTIFY)
	StatusEffects.grant_now(e, StatusEffects.REGEN, 2, 5, "Ally")
	e.remove_status(StatusEffects.REGEN)
	checks["Combatant.remove_status clears the source"] = not e.status_sources.has(StatusEffects.REGEN)
	return TestHelpers.report(TAG, "Status sources follow the instance", checks)

# Acceptance 9: cleanse options and 'all' cover the new debuffs, never fortify or regen.
static func _test_cleanse() -> bool:
	var checks: Dictionary = {}
	var options: Array[String] = ContentSchema.options_of(ContentSchema.field("skills", "cleanse_statuses"))
	var debuffs: Array[String] = [StatusEffects.SLOW, StatusEffects.EXPOSE, StatusEffects.POISON, StatusEffects.BURN]
	checks["cleanse options include the new debuffs, not fortify or regen"] = debuffs.all(func(id: String) -> bool: return options.has(id)) and not options.has(StatusEffects.FORTIFY) and not options.has(StatusEffects.REGEN)
	checks["a named cleanse of poison validates; of regen does not"] = _errors({"id": "t_sx_c1", "name": "C", "kind": "SUPPORT", "cleanse_statuses": ["poison"]}).is_empty() and not _errors({"id": "t_sx_c2", "name": "C", "kind": "SUPPORT", "cleanse_statuses": ["regen"]}).is_empty()
	var skill: SkillDef = SkillDef.create("t_sx_all", "All", SkillDef.Kind.SUPPORT, 1, 1, "")
	skill.cleanse_statuses = [SkillDef.CLEANSE_ALL]
	var caster: Combatant = _unit("Healer")
	var active: Combatant = _unit("Active")
	var pending: Combatant = _unit("Pending")
	for id: String in debuffs:
		StatusEffects.queue(active, id, 2, StatusRegistry.magnitude_min(id), "Foe")
		StatusEffects.queue(pending, id, 2, StatusRegistry.magnitude_min(id), "Foe")
	StatusEffects.queue(active, StatusEffects.FORTIFY, 2, 5, "Ally")
	StatusEffects.queue(active, StatusEffects.REGEN, 2, 5, "Ally")
	StatusEffects.roll_over(active, -1)
	SupportRules.apply(caster, active, skill, 1)
	SupportRules.apply(caster, pending, skill, 1)
	checks["'all' removes active slow, expose, poison and burn"] = debuffs.all(func(id: String) -> bool: return not StatusEffects.has_any(active, id))
	checks["'all' removes pending slow, expose, poison and burn"] = debuffs.all(func(id: String) -> bool: return not StatusEffects.has_any(pending, id)) and pending.pending_status_sources.is_empty() and pending.pending_status_magnitudes.is_empty()
	checks["'all' never removes fortify or regen"] = active.has_status(StatusEffects.FORTIFY) and active.has_status(StatusEffects.REGEN) and str(active.status_sources.get(StatusEffects.REGEN, "")) == "Ally"
	return TestHelpers.report(TAG, "Cleanse covers the new debuffs (acceptance 9)", checks)

# The status state of a unit, for equality checks.
static func _state(c: Combatant) -> Dictionary:
	return {"statuses": c.statuses.duplicate(), "rounds": c.status_rounds_left.duplicate(), "magnitudes": c.status_magnitudes.duplicate(), "sources": c.status_sources.duplicate(),
		"pending": c.pending_statuses.duplicate(), "pending_rounds": c.pending_status_rounds.duplicate(), "pending_magnitudes": c.pending_status_magnitudes.duplicate(), "pending_sources": c.pending_status_sources.duplicate()}

# Acceptance 8: reset_round restores statuses, magnitudes and sources exactly.
static func _test_undo() -> bool:
	var checks: Dictionary = {}
	var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	session.start()
	session.plan_apply_preset_all("balanced")
	session.confirm_plan()
	TestHelpers.resolve_until_replay(session)
	checks["round 2 reachable"] = session.can_advance_round()
	var unit: Combatant = session.allies[0] if not session.allies.is_empty() else null
	if session.can_advance_round() and unit != null:
		# Queued at the end of round 1, so round 2 starts with active instances that carry magnitudes and sources.
		StatusEffects.queue(unit, StatusEffects.POISON, 3, 10, "Venom")
		StatusEffects.queue(unit, StatusEffects.FORTIFY, 2, 6, "Guardian")
		session.advance_round()
		var start: Dictionary = _state(unit)
		var snap: BlockSnapshot = session.snapshot_at(session.current_round, 0)
		checks["the round start has the poison and its source"] = str(unit.status_sources.get(StatusEffects.POISON, "")) == "Venom" and DictUtil.int_of(unit.status_magnitudes, StatusEffects.POISON) == 10
		checks["the round-start snapshot stores both source dicts and pending magnitudes"] = snap != null and DictUtil.dict_of(snap.status_sources.get(unit, {})) == unit.status_sources and snap.pending_status_sources.has(unit) and snap.pending_status_magnitudes.has(unit)
		# Changes made during the undone round.
		StatusEffects.queue(unit, StatusEffects.POISON, 3, 15, "Other")
		StatusEffects.grant_now(unit, StatusEffects.REGEN, 2, 5, "Healer")
		unit.status_sources[StatusEffects.POISON] = "Changed"
		unit.status_magnitudes[StatusEffects.FORTIFY] = 20
		StatusEffects.remove(unit, StatusEffects.FORTIFY)
		session.reset_round()
		var after: Dictionary = _state(unit)
		checks["reset_round restores statuses, magnitudes and sources exactly"] = after == start
		checks["nothing queued in the undone round survives"] = unit.pending_status_sources.is_empty() and unit.pending_status_magnitudes.is_empty()
	session.dispose()
	return TestHelpers.report(TAG, "Round undo restores sources and magnitudes (acceptance 8)", checks)
