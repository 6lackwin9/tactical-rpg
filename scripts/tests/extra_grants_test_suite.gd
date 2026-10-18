class_name ExtraGrantsTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Extra Grants Test Suite (BAL5-L, spec BAL5_LEVERS section 2 T3)
# Part A (schema): the extra_grants field (json, default [], rank-modifiable, DEFEND / SUPPORT grant section) and every
# validator message with its counter-case. Part B (grant path): a DEFEND granting taunt plus an extra fortify queues
# both for next round; a riposte DEFEND keeps riposte immediate while the extra is queued; a SUPPORT puts its extra on
# the target; a rank override replaces the whole list. Fixtures are test-only skills in a sandbox fork; magnitudes and
# rounds come from the registry and the validator constants.
# ==============================================================================

const TAG: String = "ExtraGrants"
static var SANDBOX: String = TestHelpers.sandbox("extra_grants_test")

const TAUNT_FORT: String = "t_eg_taunt_fort"
const RIPOSTE_FORT: String = "t_eg_riposte_fort"
const SUPPORT_FORT: String = "t_eg_support_fort"
const RANKED: String = "t_eg_ranked"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING EXTRA GRANTS TEST (BAL5-L)              ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_schema() and ok
	ok = _test_validator() and ok
	ok = _test_token() and ok
	if _fixture_skills():
		ok = _test_defend_timing() and ok
		ok = _test_riposte_timing() and ok
		ok = _test_support() and ok
		ok = _test_rank_override() and ok
	else:
		ok = TestHelpers.report_ok(TAG, "extra grant fixture skills", false, "upsert failed") and ok
	TestHelpers.cleanup(SANDBOX)
	SkillCatalog.clear_cache()
	return ok

static func _mag_min() -> int:
	return StatusRegistry.magnitude_min(StatusEffects.FORTIFY)

static func _mag_max() -> int:
	return StatusRegistry.magnitude_max(StatusEffects.FORTIFY)

static func _fort(rounds: int, magnitude: int) -> Dictionary:
	return {"status": StatusEffects.FORTIFY, "rounds": rounds, "magnitude": magnitude}

static func _defend(id: String, extra: Dictionary = {}) -> Dictionary:
	var base: Dictionary = {"id": id, "name": id.capitalize(), "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": 0.5, "description": "{name}"}
	base.merge(extra, true)
	return base

static func _support(id: String, extra: Dictionary = {}) -> Dictionary:
	var base: Dictionary = {"id": id, "name": id.capitalize(), "kind": "SUPPORT", "level": 1, "blocks": 1, "heal_base": 5, "target_mode": "ALLY_OR_SELF", "description": "{name}"}
	base.merge(extra, true)
	return base

static func _problems(raw: Dictionary) -> Array[String]:
	return TestHelpers.messages(SkillValidator.row_problems(ContentSchema.normalize("skills", raw)))

# ------------------------------------------------------------------------------
# Part A: schema and validation
# ------------------------------------------------------------------------------

static func _test_schema() -> bool:
	var checks: Dictionary = {}
	var f: Dictionary = ContentSchema.field("skills", "extra_grants")
	checks["field exists as json"] = not f.is_empty() and str(f.get("type", "")) == "json"
	checks["default []"] = DictUtil.array_of(ContentSchema.normalize("skills", _defend("t_eg_def"))["extra_grants"]).is_empty()
	var kinds: Array[String] = SkillKinds.kinds_with("extra_grants")
	kinds.sort()
	checks["shown on DEFEND and SUPPORT only"] = kinds == (["DEFEND", "SUPPORT"] as Array[String])
	checks["rank-modifiable"] = SkillRanks.RANK_KEYS.has("extra_grants") and DictUtil.to_bool(f.get("rank", false))
	var built: SkillDef = ContentFactory.skill(_defend("t_eg_build", {"grant_status": StatusEffects.TAUNT, "extra_grants": [_fort(2, _mag_min())]}))
	checks["SkillDef carries typed rows"] = built.extra_grants.size() == 1 and str(built.extra_grants[0]["status"]) == StatusEffects.FORTIFY and DictUtil.to_int(built.extra_grants[0]["rounds"]) == 2 and DictUtil.to_int(built.extra_grants[0]["magnitude"]) == _mag_min()
	checks["skill_to_entry round-trips the list"] = DictUtil.array_of(ContentFactory.skill_to_entry(built)["extra_grants"]).size() == 1
	checks["shipped catalog validates unchanged"] = ContentCatalog.validate_all().is_empty()
	return TestHelpers.report(TAG, "extra_grants schema field", checks)

static func _test_validator() -> bool:
	var checks: Dictionary = {}
	var lo: int = SkillValidator.EXTRA_GRANT_ROUNDS_MIN
	var hi: int = SkillValidator.EXTRA_GRANT_ROUNDS_MAX
	var good: Dictionary = _defend("t_ev_ok", {"grant_status": StatusEffects.TAUNT, "extra_grants": [_fort(hi, _mag_max())]})
	checks["DEFEND taunt + fortify (max rounds / magnitude) is valid"] = _problems(good).is_empty()
	checks["band edges: min rounds / magnitude valid"] = _problems(_defend("t_ev_lo", {"grant_status": StatusEffects.TAUNT, "extra_grants": [_fort(lo, _mag_min())]})).is_empty()
	checks["SUPPORT heal + fortify is valid"] = _problems(_support("t_ev_sup", {"extra_grants": [_fort(lo, _mag_min())]})).is_empty()
	checks["DEFEND with no main grant may hold an extra"] = _problems(_defend("t_ev_nogrant", {"extra_grants": [_fort(lo, _mag_min())]})).is_empty()
	checks["ATTACK refused"] = _problems(TestHelpers.attack("t_ev_atk", {"extra_grants": [_fort(lo, _mag_min())]})) == (["extra_grants needs a DEFEND or SUPPORT skill."] as Array[String])
	checks["FOCUS refused"] = _problems({"id": "t_ev_focus", "name": "T", "kind": "FOCUS", "description": "{name}", "extra_grants": [_fort(lo, _mag_min())]}).has("extra_grants needs a DEFEND or SUPPORT skill.")
	var two: Array[Dictionary] = [_fort(lo, _mag_min()), {"status": StatusEffects.HASTE, "rounds": lo, "magnitude": 0}]
	checks["two entries refused"] = _problems(_defend("t_ev_two", {"grant_status": StatusEffects.TAUNT, "extra_grants": two})) == (["A skill holds at most one extra grant."] as Array[String])
	checks["same as grant_status refused"] = _problems(_defend("t_ev_same", {"grant_status": StatusEffects.TAUNT, "extra_grants": [{"status": StatusEffects.TAUNT, "rounds": lo, "magnitude": 0}]})) == (["Extra grant '%s' must be another buff." % StatusEffects.TAUNT] as Array[String])
	var debuff: String = StatusEffects.debuffs()[0]
	checks["a debuff refused"] = _problems(_defend("t_ev_debuff", {"extra_grants": [{"status": debuff, "rounds": lo, "magnitude": 0}]})) == (["Extra grant '%s' must be another buff." % debuff] as Array[String])
	checks["an unknown status refused"] = _problems(_defend("t_ev_unknown", {"extra_grants": [{"status": "t_no_such", "rounds": lo, "magnitude": 0}]})) == (["Extra grant 't_no_such' must be another buff."] as Array[String])
	checks["a defend-only buff on SUPPORT refused, on DEFEND allowed"] = _problems(_support("t_ev_rip", {"extra_grants": [{"status": StatusEffects.RIPOSTE, "rounds": lo, "magnitude": 0}]})) == (["Extra grant '%s' must be another buff." % StatusEffects.RIPOSTE] as Array[String]) and _problems(_defend("t_ev_rip2", {"grant_status": StatusEffects.TAUNT, "extra_grants": [{"status": StatusEffects.RIPOSTE, "rounds": lo, "magnitude": 0}]})).is_empty()
	var rounds_msg: String = "Extra grant '%s' needs rounds %d-%d." % [StatusEffects.FORTIFY, lo, hi]
	checks["rounds below / above refused"] = _problems(_defend("t_ev_r0", {"extra_grants": [_fort(lo - 1, _mag_min())]})) == ([rounds_msg] as Array[String]) and _problems(_defend("t_ev_r9", {"extra_grants": [_fort(hi + 1, _mag_min())]})) == ([rounds_msg] as Array[String])
	var mag_msg: String = "Extra grant '%s' needs a magnitude %d-%d." % [StatusEffects.FORTIFY, _mag_min(), _mag_max()]
	checks["magnitude below / above refused"] = _problems(_defend("t_ev_m0", {"extra_grants": [_fort(lo, _mag_min() - 1)]})) == ([mag_msg] as Array[String]) and _problems(_defend("t_ev_m9", {"extra_grants": [_fort(lo, _mag_max() + 1)]})) == ([mag_msg] as Array[String])
	checks["magnitude on a status without a range refused"] = _problems(_defend("t_ev_hm", {"extra_grants": [{"status": StatusEffects.HASTE, "rounds": lo, "magnitude": 3}]})) == (["Extra grant '%s' takes no magnitude (leave it 0)." % StatusEffects.HASTE] as Array[String])
	checks["SUPPORT taunt extra needs SELF"] = _problems(_support("t_ev_tau", {"extra_grants": [{"status": StatusEffects.TAUNT, "rounds": lo, "magnitude": 0}]})).has("Taunt is cast on the taunter itself: set Targets to SELF.") and _problems(_support("t_ev_tau2", {"target_mode": "SELF", "extra_grants": [{"status": StatusEffects.TAUNT, "rounds": lo, "magnitude": 0}]})).is_empty()
	return TestHelpers.report(TAG, "extra_grants validator messages", checks)

# {extra}: each rank renders its own (overridden or base) list; status display name, registry magnitude unit, rounds.
static func _extra_text(status: String, magnitude: int, rounds: int) -> String:
	var mag: String = SkillText.format_magnitude(status, magnitude)
	var r: String = SkillText.plural(rounds, "round")
	return "%s %s for %s" % [StatusEffects.display_name(status), mag, r] if not mag.is_empty() else "%s for %s" % [StatusEffects.display_name(status), r]

static func _test_token() -> bool:
	var checks: Dictionary = {}
	var hi: int = SkillValidator.EXTRA_GRANT_ROUNDS_MAX
	var base: Dictionary = _defend("t_et", {"grant_status": StatusEffects.TAUNT, "template": "{name}: {extra}", "description": "{name}: {extra}", "extra_grants": [_fort(1, _mag_min())]})
	var mods: Array[Dictionary] = [{"level": 2, "extra_grants": [_fort(hi, _mag_max())]}, {"level": 3}]
	var entry: Dictionary = SkillRanks.entry(base, mods)
	checks["{extra} valid on DEFEND and SUPPORT only"] = SkillText.token_valid_for("extra", "DEFEND") and SkillText.token_valid_for("extra", "SUPPORT") and not SkillText.token_valid_for("extra", "ATTACK") and not SkillText.token_valid_for("extra", "FOCUS")
	checks["the template validates on every rank"] = SkillText.validate("{name}: {extra}", entry).is_empty()
	var texts: Array[String] = []
	for rank: int in [1, 2, 3]:
		texts.append(SkillText.render("{extra}", "DEFEND", SkillRanks.effective(entry, rank), rank))
	checks["rank 1 renders the base list"] = texts[0] == _extra_text(StatusEffects.FORTIFY, _mag_min(), 1)
	checks["rank 2 renders its override"] = texts[1] == _extra_text(StatusEffects.FORTIFY, _mag_max(), hi)
	checks["rank 3 falls back to the base"] = texts[2] == texts[0]
	checks["a magnitude-less status renders without one"] = SkillText.format_extra([{"status": StatusEffects.HASTE, "rounds": 2, "magnitude": 0}]) == _extra_text(StatusEffects.HASTE, 0, 2) and not SkillText.format_extra([{"status": StatusEffects.HASTE, "rounds": 2, "magnitude": 0}]).contains("  ")
	var none: Dictionary = _defend("t_et_none", {"grant_status": StatusEffects.TAUNT})
	checks["{extra} on a skill without extras resolves to nothing (error)"] = TestHelpers.any_contains(TestHelpers.messages(SkillText.validate("{name}: {extra}", none)), "token {extra} resolves to nothing")
	checks["{extra} on an ATTACK is refused"] = TestHelpers.any_contains(TestHelpers.messages(SkillText.validate("{name}: {extra}", TestHelpers.attack("t_et_atk"))), "Token {extra} does not apply to ATTACK")
	return TestHelpers.report(TAG, "{extra} description token", checks)

# ------------------------------------------------------------------------------
# Part B: grant path
# ------------------------------------------------------------------------------

static func _fixture_skills() -> bool:
	if not TestHelpers.fork(SANDBOX):
		return false
	var rows: Array[Dictionary] = [
		_defend(TAUNT_FORT, {"grant_status": StatusEffects.TAUNT, "status_rounds": 1, "extra_grants": [_fort(SkillValidator.EXTRA_GRANT_ROUNDS_MAX, _mag_max())]}),
		_defend(RIPOSTE_FORT, {"grant_status": StatusEffects.RIPOSTE, "status_rounds": 1, "mitigation": 0.3, "extra_grants": [_fort(SkillValidator.EXTRA_GRANT_ROUNDS_MIN, _mag_min())]}),
		_support(SUPPORT_FORT, {"extra_grants": [_fort(SkillValidator.EXTRA_GRANT_ROUNDS_MAX, _mag_min())]}),
	]
	var ok: bool = true
	for row: Dictionary in rows:
		var errors: Array[String] = ContentCatalog.upsert_skill(row)
		if not errors.is_empty():
			print("[%s] fixture %s: %s" % [TAG, str(row["id"]), str(errors)])
			ok = false
	SkillCatalog.clear_cache()
	return ok and SkillCatalog.find(TAUNT_FORT) != null and SkillCatalog.find(SUPPORT_FORT) != null

static func _grid() -> CombatGrid:
	return CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])

static func _defend_command(actor: Combatant, skill_id: String, grid: CombatGrid) -> DefendCommand:
	var entry: PlanEntry = PlanEntry.create(SkillCatalog.find(skill_id), 1, 0)
	entry.skill_ap = entry.skill.ap_cost(actor)
	var cmd: BattleCommand = entry.build_command(actor, grid)
	return cmd as DefendCommand

static func _test_defend_timing() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var tank: Combatant = TestHelpers.unit("Tank", grid, Vector2i(3, 4))
	var skill: SkillDef = SkillCatalog.find(TAUNT_FORT)
	var cmd: DefendCommand = _defend_command(tank, TAUNT_FORT, grid)
	checks["the planner copies the list to the command"] = cmd != null and cmd.extra_grants == skill.extra_grants
	if cmd == null:
		return TestHelpers.report(TAG, "DEFEND taunt + extra fortify timing", checks)
	var line: String = cmd.on_start()
	var rounds: int = DictUtil.to_int(skill.extra_grants[0]["rounds"])
	var magnitude: int = DictUtil.to_int(skill.extra_grants[0]["magnitude"])
	checks["neither is active this round"] = not tank.statuses.has(StatusEffects.TAUNT) and not tank.statuses.has(StatusEffects.FORTIFY)
	checks["both are queued for next round"] = tank.pending_statuses.has(StatusEffects.TAUNT) and tank.pending_statuses.has(StatusEffects.FORTIFY)
	checks["fortify queued with its rounds and magnitude"] = DictUtil.int_of(tank.pending_status_rounds, StatusEffects.FORTIFY) == rounds and DictUtil.int_of(tank.pending_status_magnitudes, StatusEffects.FORTIFY) == magnitude
	checks["taunt keeps the skill's own rounds"] = DictUtil.int_of(tank.pending_status_rounds, StatusEffects.TAUNT) == skill.status_rounds
	checks["log names both grants"] = line.contains("[Taunt]") and line.contains("[Fortify]")
	StatusEffects.roll_over(tank, 0)
	checks["next round both are active, fortify at its magnitude"] = tank.statuses.has(StatusEffects.TAUNT) and tank.statuses.has(StatusEffects.FORTIFY) and DictUtil.int_of(tank.status_magnitudes, StatusEffects.FORTIFY) == magnitude
	return TestHelpers.report(TAG, "DEFEND taunt + extra fortify timing", checks)

static func _test_riposte_timing() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var tank: Combatant = TestHelpers.unit("Tank", grid, Vector2i(3, 4))
	var cmd: DefendCommand = _defend_command(tank, RIPOSTE_FORT, grid)
	checks["command built"] = cmd != null
	if cmd == null:
		return TestHelpers.report(TAG, "riposte DEFEND + extra fortify timing", checks)
	cmd.on_start()
	checks["riposte is active at once"] = tank.statuses.has(StatusEffects.RIPOSTE)
	checks["the support-buff extra is queued, not active"] = tank.pending_statuses.has(StatusEffects.FORTIFY) and not tank.statuses.has(StatusEffects.FORTIFY)
	# A defend-only extra on a DEFEND is immediate like a riposte grant.
	var other: Combatant = TestHelpers.unit("Other", grid, Vector2i(5, 4))
	var extras: Array[Dictionary] = [{"status": StatusEffects.RIPOSTE, "rounds": 1, "magnitude": 0}]
	StatusEffects.grant_extras(other, extras, true, "Other")
	checks["a riposte extra on a DEFEND is immediate"] = other.statuses.has(StatusEffects.RIPOSTE) and not other.pending_statuses.has(StatusEffects.RIPOSTE)
	return TestHelpers.report(TAG, "riposte DEFEND + extra fortify timing", checks)

static func _test_support() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var caster: Combatant = TestHelpers.unit("Priest", grid, Vector2i(3, 4))
	var ally: Combatant = TestHelpers.unit("Ally", grid, Vector2i(4, 4))
	var skill: SkillDef = SkillCatalog.find(SUPPORT_FORT)
	var result: Dictionary = SupportRules.apply(caster, ally, skill, skill.status_rounds)
	checks["the extra goes to the target, queued"] = ally.pending_statuses.has(StatusEffects.FORTIFY) and not ally.statuses.has(StatusEffects.FORTIFY) and not caster.pending_statuses.has(StatusEffects.FORTIFY)
	checks["with its own rounds and magnitude"] = DictUtil.int_of(ally.pending_status_rounds, StatusEffects.FORTIFY) == DictUtil.to_int(skill.extra_grants[0]["rounds"]) and DictUtil.int_of(ally.pending_status_magnitudes, StatusEffects.FORTIFY) == DictUtil.to_int(skill.extra_grants[0]["magnitude"])
	checks["result lists the extra"] = ContentFactory._strings(result.get("extras", [])) == ([StatusEffects.FORTIFY] as Array[String])
	return TestHelpers.report(TAG, "SUPPORT extra grant on the target", checks)

static func _test_rank_override() -> bool:
	var checks: Dictionary = {}
	var base: Dictionary = _defend(RANKED, {"grant_status": StatusEffects.TAUNT, "extra_grants": [_fort(1, _mag_min())]})
	var override: Array[Dictionary] = [_fort(SkillValidator.EXTRA_GRANT_ROUNDS_MAX, _mag_max())]
	var mods: Array[Dictionary] = [{"level": 2, "extra_grants": override}, {"level": 3}]
	var errors: Array[String] = ContentCatalog.upsert_skill(SkillRanks.entry(base, mods))
	checks["a ranked entry with a rank override is accepted"] = errors.is_empty()
	SkillCatalog.clear_cache()
	var r1: SkillDef = SkillCatalog.find(RANKED)
	var r2: SkillDef = SkillCatalog.find(SkillRanks.rank_id(RANKED, 2))
	var r3: SkillDef = SkillCatalog.find(SkillRanks.rank_id(RANKED, 3))
	checks["ranks built"] = r1 != null and r2 != null and r3 != null
	if r1 == null or r2 == null or r3 == null:
		return TestHelpers.report(TAG, "rank override of extra_grants", checks)
	checks["rank 1 keeps the base list"] = DictUtil.to_int(r1.extra_grants[0]["magnitude"]) == _mag_min()
	checks["rank 2 carries the whole override"] = r2.extra_grants.size() == 1 and DictUtil.to_int(r2.extra_grants[0]["magnitude"]) == _mag_max() and DictUtil.to_int(r2.extra_grants[0]["rounds"]) == SkillValidator.EXTRA_GRANT_ROUNDS_MAX
	checks["rank 3 without a modifier falls back to the base"] = DictUtil.to_int(r3.extra_grants[0]["magnitude"]) == _mag_min()
	var bad_mods: Array[Dictionary] = [{"level": 2, "extra_grants": [_fort(1, _mag_max() + 1)]}, {"level": 3}]
	checks["an invalid rank override is refused"] = not ContentCatalog.upsert_skill(SkillRanks.entry(_defend("t_eg_badrank", {"grant_status": StatusEffects.TAUNT}), bad_mods)).is_empty()
	checks["catalog still validates"] = ContentCatalog.validate_all().is_empty()
	return TestHelpers.report(TAG, "rank override of extra_grants", checks)
