class_name ComboRulesTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Combo Rules Test Suite (S12C-L)
# Part A (schema): the four family-level combo fields (ends_in_stance, disrupt, exploit, exploit_consume), their kind
# sections, every validator message, the rank key rule, the SkillText tokens {flow} {exploit} {disrupt} and the SkillDef
# build. Fixtures are test-only skills in a sandbox fork; statuses are picked by rule from the registry.
# Part B (rules 1-5): flow planning (switch cost against the end stance) and resolution, a flowing DEFEND keeps its guard
# to its last block, stance break (after the hit and its riposte; running guards end), exploit (active statuses only,
# highest multiplier, consume), planner estimates equal resolved hits, round undo.
# Part C (AI, rule 6): a stance-break skill against a Defensive or riposting foe; exploit only when it applies.
# Units are hand-built (fixed power, no armor); expected numbers come from DamageCalculator.compute.
# ==============================================================================

const TAG: String = "ComboRules"
static var SANDBOX: String = TestHelpers.sandbox("combo_rules_test")

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING COMBO RULES TEST (S12C-L)               ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_schema() and ok
	ok = _test_validator() and ok
	ok = _test_rank_rule() and ok
	ok = _test_skill_def() and ok
	ok = _test_tokens() and ok
	# Part B (rules) on a fresh fork with test-only combo skills.
	if _fixture_skills():
		ok = _test_flow_plan() and ok
		ok = _test_flow_defend() and ok
		ok = _test_break() and ok
		ok = _test_break_skips() and ok
		ok = _test_exploit() and ok
		ok = _test_estimates() and ok
		ok = _test_undo() and ok
		# Part C (AI, rule 6) on the same fixture skills.
		ok = _test_ai_break() and ok
		ok = _test_ai_exploit() and ok
	else:
		ok = TestHelpers.report_ok(TAG, "combo fixture skills", false, "upsert failed") and ok
	TestHelpers.cleanup(SANDBOX)
	return ok

# ------------------------------------------------------------------------------
# Part A: schema
# ------------------------------------------------------------------------------

static func _sorted(list: Array[String]) -> Array[String]:
	var out: Array[String] = list.duplicate()
	out.sort()
	return out

static func _test_schema() -> bool:
	var checks: Dictionary = {}
	checks["ends_in_stance on ATTACK, DEFEND, FOCUS, SUPPORT"] = _sorted(SkillKinds.kinds_with("ends_in_stance")) == _sorted(["ATTACK", "DEFEND", "FOCUS", "SUPPORT"] as Array[String])
	for key: String in ["disrupt", "exploit", "exploit_consume"]:
		checks["%s on ATTACK only" % key] = SkillKinds.kinds_with(key) == (["ATTACK"] as Array[String])
	var defaults_ok: bool = true
	var rank_flag_off: bool = true
	for key: String in SkillRanks.FAMILY_KEYS:
		var f: Dictionary = ContentSchema.field("skills", key)
		defaults_ok = defaults_ok and not f.is_empty()
		rank_flag_off = rank_flag_off and not DictUtil.to_bool(f.get("rank", true)) and not SkillRanks.RANK_KEYS.has(key)
	checks["the four fields exist in the schema"] = defaults_ok and SkillRanks.FAMILY_KEYS.size() == 4
	checks["none of them is rank-modifiable"] = rank_flag_off
	var row: Dictionary = ContentSchema.normalize("skills", TestHelpers.attack("t_combo_defaults"))
	checks["defaults: empty / NONE / [] / false"] = str(row["ends_in_stance"]).is_empty() and str(row["disrupt"]) == "NONE" and DictUtil.array_of(row["exploit"]).is_empty() and not DictUtil.to_bool(row["exploit_consume"])
	checks["Forge sections: ATTACK has combo and flow, DEFEND flow only"] = SkillKinds.sections_of("ATTACK").has("combo") and SkillKinds.sections_of("ATTACK").has("flow") and SkillKinds.sections_of("DEFEND").has("flow") and not SkillKinds.sections_of("DEFEND").has("combo")
	checks["shipped catalog validates unchanged"] = ContentCatalog.validate_all().is_empty()
	return TestHelpers.report(TAG, "combo schema fields", checks)

static func _problems(raw: Dictionary) -> Array[String]:
	var e: Dictionary = ContentSchema.normalize("skills", raw)
	return TestHelpers.messages(SkillValidator.row_problems(e))

static func _defend(id: String, extra: Dictionary = {}) -> Dictionary:
	var base: Dictionary = {"id": id, "name": id.capitalize(), "kind": "DEFEND", "level": 1, "blocks": 1, "mitigation": 0.5, "description": "{name}"}
	base.merge(extra, true)
	return base

static func _test_validator() -> bool:
	var checks: Dictionary = {}
	var debuffs: Array[String] = StatusEffects.debuffs()
	var buffs: Array[String] = StatusEffects.buffs()
	checks["fixture: the registry has 2+ debuffs and a buff"] = debuffs.size() >= 2 and not buffs.is_empty()
	if debuffs.size() < 2 or buffs.is_empty():
		return TestHelpers.report(TAG, "combo validator messages", checks)
	var d1: String = debuffs[0]
	var d2: String = debuffs[1]
	var good: Dictionary = TestHelpers.attack("t_combo_ok", {"ends_in_stance": "DEFENSIVE", "disrupt": "STANCE_BREAK", "exploit": [{"status": d1, "power_mult": 1.3}, {"status": d2, "power_mult": 1.2}], "exploit_consume": true})
	checks["a full combo attack is valid"] = _problems(good).is_empty()
	checks["flow to the skill's own stance (ATTACK -> Offensive)"] = _problems(TestHelpers.attack("t_c1", {"ends_in_stance": "OFFENSIVE"})) == (["Ends in stance equals the skill's stance; leave it empty."] as Array[String])
	checks["flow to the override stance (Defensive attack -> Defensive)"] = _problems(TestHelpers.attack("t_c2", {"stance": "DEFENSIVE", "ends_in_stance": "DEFENSIVE"})).has("Ends in stance equals the skill's stance; leave it empty.")
	checks["a Defensive attack may flow to Offensive"] = _problems(TestHelpers.attack("t_c2b", {"stance": "DEFENSIVE", "ends_in_stance": "OFFENSIVE"})).is_empty()
	checks["DEFEND -> Defensive and FOCUS -> Tactical are refused"] = _problems(_defend("t_c3", {"ends_in_stance": "DEFENSIVE"})).has("Ends in stance equals the skill's stance; leave it empty.") and _problems({"id": "t_c4", "name": "T", "kind": "FOCUS", "description": "{name}", "ends_in_stance": "TACTICAL"}).has("Ends in stance equals the skill's stance; leave it empty.")
	checks["a non-riposte DEFEND may flow to Offensive"] = _problems(_defend("t_c5", {"ends_in_stance": "OFFENSIVE"})).is_empty()
	var riposte: Dictionary = _defend("t_c6", {"grant_status": StatusEffects.RIPOSTE, "ends_in_stance": "OFFENSIVE"})
	checks["a riposte DEFEND cannot flow"] = _problems(riposte).has("A riposte skill cannot flow: a stance change ends riposte.") and not _problems(_defend("t_c6b", {"grant_status": StatusEffects.RIPOSTE})).has("A riposte skill cannot flow: a stance change ends riposte.")
	checks["a SUPPORT skill may flow"] = _problems({"id": "t_c7", "name": "T", "kind": "SUPPORT", "heal_stat": "", "heal_base": 1, "description": "{name}", "ends_in_stance": "DEFENSIVE"}).is_empty()
	var disengage: Array[String] = _problems({"id": "t_c8", "name": "T", "kind": "DISENGAGE", "jump_tiles": 1, "description": "{name}", "ends_in_stance": "OFFENSIVE"})
	checks["a DISENGAGE cannot flow (belongs-to rule)"] = disengage.size() == 1 and disengage[0].begins_with("Ends in stance belongs to")
	var zero: Dictionary = {"power_mult": 0.0, "status_effect": d1, "status_potency": 50}
	var zero_break: Dictionary = zero.duplicate()
	zero_break["disrupt"] = "STANCE_BREAK"
	checks["zero-power break"] = _problems(TestHelpers.attack("t_c9", zero_break)).has("A zero-power attack cannot break a stance.")
	var zero_exploit: Dictionary = zero.duplicate()
	zero_exploit["exploit"] = [{"status": d2, "power_mult": 1.3}]
	checks["zero-power exploit"] = _problems(TestHelpers.attack("t_c10", zero_exploit)).has("A zero-power attack cannot exploit.")
	var three: Array[Dictionary] = []
	for i: int in range(mini(3, debuffs.size())):
		three.append({"status": debuffs[i], "power_mult": 1.2})
	while three.size() < 3:
		three.append({"status": d1, "power_mult": 1.2})
	checks["at most 2 statuses"] = _problems(TestHelpers.attack("t_c11", {"exploit": three})).has("Exploit takes at most 2 statuses.")
	checks["status must be a debuff"] = _problems(TestHelpers.attack("t_c12", {"exploit": [{"status": buffs[0], "power_mult": 1.3}]})) == (["Exploit status '%s' is not a debuff." % buffs[0]] as Array[String])
	checks["duplicate status"] = _problems(TestHelpers.attack("t_c13", {"exploit": [{"status": d1, "power_mult": 1.3}, {"status": d1, "power_mult": 1.2}]})) == (["Exploit lists '%s' twice." % d1] as Array[String])
	var low: Array[String] = _problems(TestHelpers.attack("t_c14", {"exploit": [{"status": d1, "power_mult": SkillValidator.EXPLOIT_MULT_MIN - 0.05}]}))
	var high: Array[String] = _problems(TestHelpers.attack("t_c15", {"exploit": [{"status": d1, "power_mult": SkillValidator.EXPLOIT_MULT_MAX + 0.1}]}))
	var edges: Array[String] = _problems(TestHelpers.attack("t_c16", {"exploit": [{"status": d1, "power_mult": SkillValidator.EXPLOIT_MULT_MIN}, {"status": d2, "power_mult": SkillValidator.EXPLOIT_MULT_MAX}]}))
	checks["power below / above the band"] = low == (["Exploit power must be 1.05-2.0."] as Array[String]) and high == (["Exploit power must be 1.05-2.0."] as Array[String])
	checks["band edges are valid"] = edges.is_empty()
	checks["consume needs an exploit status"] = _problems(TestHelpers.attack("t_c17", {"exploit_consume": true})) == (["Consume needs an exploit status."] as Array[String])
	checks["disrupt on a DEFEND is refused (belongs-to rule)"] = TestHelpers.any_contains(_problems(_defend("t_c18", {"disrupt": "STANCE_BREAK"})), "Disrupt belongs to")
	checks["unknown end stance is an enum error"] = not ContentSchema.field_errors("skills", ContentSchema.normalize("skills", TestHelpers.attack("t_c19", {"ends_in_stance": "NEUTRAL"}))).is_empty()
	return TestHelpers.report(TAG, "combo validator messages", checks)

static func _test_rank_rule() -> bool:
	var checks: Dictionary = {}
	checks["sandbox fork"] = TestHelpers.fork(SANDBOX)
	var d1: String = StatusEffects.debuffs()[0]
	var values: Dictionary = {"ends_in_stance": "DEFENSIVE", "disrupt": "STANCE_BREAK", "exploit": [{"status": d1, "power_mult": 1.3}], "exploit_consume": true}
	for key: String in SkillRanks.FAMILY_KEYS:
		var mods: Array[Dictionary] = [{"level": 3, key: values[key]}]
		var errors: Array[String] = ContentCatalog.upsert_skill(SkillRanks.entry(TestHelpers.attack("t_cr_%s" % key), mods))
		checks["a rank that sets %s is refused" % key] = errors.has(SkillRanks.FAMILY_KEY_MESSAGE % key)
	var base: Dictionary = TestHelpers.attack("t_cr_family", values)
	var ok_mods: Array[Dictionary] = [{"level": 3}, {"level": 4}]
	checks["a family-level combo entry is accepted"] = ContentCatalog.upsert_skill(SkillRanks.entry(base, ok_mods)).is_empty()
	var same: bool = true
	for rank: int in [2, 3]:
		var row: Dictionary = ContentCatalog.get_entry("skills", SkillRanks.rank_id("t_cr_family", rank))
		for key: String in SkillRanks.FAMILY_KEYS:
			same = same and not row.is_empty() and row.get(key, null) == ContentSchema.normalize("skills", base).get(key, null)
	checks["every rank carries the family values"] = same
	checks["catalog still validates"] = ContentCatalog.validate_all().is_empty()
	TestHelpers.cleanup(SANDBOX)
	return TestHelpers.report(TAG, "combo rank key rule", checks)

static func _test_skill_def() -> bool:
	var checks: Dictionary = {}
	var d1: String = StatusEffects.debuffs()[0]
	var plain: SkillDef = ContentFactory.skill(TestHelpers.attack("t_cd_plain"))
	checks["defaults: -1 / NONE / [] / false"] = plain.ends_in_stance == -1 and plain.disrupt == SkillDef.Disrupt.NONE and plain.exploit.is_empty() and not plain.exploit_consume
	var combo: SkillDef = ContentFactory.skill(TestHelpers.attack("t_cd_combo", {"ends_in_stance": "DEFENSIVE", "disrupt": "STANCE_BREAK", "exploit": [{"status": d1, "power_mult": 1.3}], "exploit_consume": true}))
	checks["ends_in_stance builds the stance ordinal"] = combo.ends_in_stance == CombatTypes.StanceType.DEFENSIVE
	checks["disrupt builds STANCE_BREAK"] = combo.disrupt == SkillDef.Disrupt.STANCE_BREAK
	checks["exploit builds typed rows"] = combo.exploit.size() == 1 and str(combo.exploit[0]["status"]) == d1 and is_equal_approx(DictUtil.to_float(combo.exploit[0]["power_mult"]), 1.3) and combo.exploit_consume
	var back: Dictionary = ContentFactory.skill_to_entry(combo)
	checks["skill_to_entry round trip"] = str(back["ends_in_stance"]) == "DEFENSIVE" and str(back["disrupt"]) == "STANCE_BREAK" and DictUtil.array_of(back["exploit"]).size() == 1 and DictUtil.to_bool(back["exploit_consume"])
	checks["stance_during is the skill's stance, stance_after the flow stance"] = combo.stance_during(CombatTypes.StanceType.TACTICAL) == CombatTypes.StanceType.OFFENSIVE and combo.stance_after(CombatTypes.StanceType.TACTICAL) == CombatTypes.StanceType.DEFENSIVE and plain.stance_after(CombatTypes.StanceType.TACTICAL) == CombatTypes.StanceType.OFFENSIVE
	return TestHelpers.report(TAG, "combo SkillDef build", checks)

static func _test_tokens() -> bool:
	var checks: Dictionary = {}
	var debuffs: Array[String] = StatusEffects.debuffs()
	var d1: String = debuffs[0]
	var d2: String = debuffs[1] if debuffs.size() > 1 else debuffs[0]
	var row: Dictionary = ContentSchema.normalize("skills", TestHelpers.attack("t_ct", {"ends_in_stance": "DEFENSIVE", "disrupt": "STANCE_BREAK", "exploit": [{"status": d1, "power_mult": 1.3}, {"status": d2, "power_mult": 1.2}]}))
	var plain: Dictionary = ContentSchema.normalize("skills", TestHelpers.attack("t_ct_plain"))
	checks["{flow} -> ends in <stance name>"] = SkillText.token_value("flow", "", row, 1) == "ends in %s" % CombatTypes.stance_to_string(CombatTypes.StanceType.DEFENSIVE)
	checks["{exploit} -> x<mult> vs <display name>, ..."] = SkillText.token_value("exploit", "", row, 1) == "x1.3 vs %s, x1.2 vs %s" % [StatusEffects.display_name(d1), StatusEffects.display_name(d2)]
	checks["{disrupt} -> breaks stance"] = SkillText.token_value("disrupt", "", row, 1) == "breaks stance"
	checks["empty fields render nothing"] = SkillText.token_value("flow", "", plain, 1).is_empty() and SkillText.token_value("exploit", "", plain, 1).is_empty() and SkillText.token_value("disrupt", "", plain, 1).is_empty()
	checks["token kinds: flow on DEFEND/FOCUS/SUPPORT, exploit and disrupt ATTACK only"] = SkillText.tokens_for("DEFEND").has("flow") and SkillText.tokens_for("FOCUS").has("flow") and SkillText.tokens_for("SUPPORT").has("flow") and not SkillText.tokens_for("DISENGAGE").has("flow") and not SkillText.tokens_for("DEFEND").has("exploit") and not SkillText.tokens_for("DEFEND").has("disrupt")
	checks["a template using the tokens validates and renders"] = SkillText.validate("{name}: {power}, {flow}, {exploit}, {disrupt}", row).is_empty() and SkillText.render("{flow}", "ATTACK", row, 1) == SkillText.token_value("flow", "", row, 1)
	checks["a token on an empty field is refused"] = not SkillText.validate("{name} {exploit}", plain).is_empty()
	return TestHelpers.report(TAG, "combo description tokens", checks)

# ------------------------------------------------------------------------------
# Part B: rules
# ------------------------------------------------------------------------------

const FLOW: String = "t_cb_flow"
const PLAIN: String = "t_cb_plain"
const FLOW_DEF: String = "t_cb_flowdef"
const BREAK: String = "t_cb_break"
const BREAK2: String = "t_cb_break2"
const EXPLOIT: String = "t_cb_exploit"
const CONSUME: String = "t_cb_consume"
const RIPOSTE_DEF: String = "t_cb_riposte"
const EXPLOIT_MULT: float = 1.3
const SECOND_MULT: float = 1.2

static func _fixture_skills() -> bool:
	if not TestHelpers.fork(SANDBOX):
		return false
	var one: Dictionary = {"level": 1, "blocks": 1, "power_mult": 1.0}
	var rows: Array[Dictionary] = [
		TestHelpers.attack(FLOW, _with(one, {"ends_in_stance": "DEFENSIVE"})),
		TestHelpers.attack(PLAIN, one),
		_defend(FLOW_DEF, {"level": 1, "blocks": 2, "mitigation": 0.5, "ends_in_stance": "OFFENSIVE"}),
		TestHelpers.attack(BREAK, _with(one, {"disrupt": "STANCE_BREAK"})),
		TestHelpers.attack(BREAK2, _with(one, {"blocks": 2, "disrupt": "STANCE_BREAK"})),
		TestHelpers.attack(EXPLOIT, _with(one, {"exploit": [{"status": StatusEffects.EXPOSE, "power_mult": EXPLOIT_MULT}, {"status": StatusEffects.SLOW, "power_mult": SECOND_MULT}]})),
		TestHelpers.attack(CONSUME, _with(one, {"exploit": [{"status": StatusEffects.EXPOSE, "power_mult": EXPLOIT_MULT}], "exploit_consume": true})),
		_defend(RIPOSTE_DEF, {"level": 1, "blocks": 1, "mitigation": 0.3, "grant_status": StatusEffects.RIPOSTE, "status_rounds": 1}),
	]
	var ok: bool = true
	for row: Dictionary in rows:
		var errors: Array[String] = ContentCatalog.upsert_skill(row)
		if not errors.is_empty():
			print("[%s] fixture %s: %s" % [TAG, str(row["id"]), str(errors)])
			ok = false
	SkillCatalog.clear_cache()
	return ok and SkillCatalog.find(FLOW) != null and SkillCatalog.find(CONSUME) != null

static func _with(base: Dictionary, extra: Dictionary) -> Dictionary:
	var out: Dictionary = base.duplicate(true)
	out.merge(extra, true)
	return out

static func _grid() -> CombatGrid:
	return CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])

static func _rng(seed_value: int) -> RandomNumberGenerator:
	var r := RandomNumberGenerator.new()
	r.seed = seed_value
	return r

static func _active(c: Combatant, status: String) -> void:
	StatusEffects.queue(c, status, 2, 0, "Setup")
	StatusEffects.roll_over(c, -1)

static func _resolver(grid: CombatGrid, lines: Array[String]) -> CombatResolver:
	var r := CombatResolver.new()
	r.grid = grid
	r.log_emitted.connect(func(msg: String) -> void: lines.append(msg))
	return r

static func _index_of_prefix(lines: Array[String], prefix: String) -> int:
	for i: int in range(lines.size()):
		if lines[i].begins_with(prefix):
			return i
	return -1

static func _count_prefix(lines: Array[String], prefix: String) -> int:
	var n: int = 0
	for line: String in lines:
		if line.begins_with(prefix):
			n += 1
	return n

# A command of `skill_id` built the way the planner builds it (PlanEntry.build_command).
static func _command(actor: Combatant, foe: Combatant, skill_id: String, grid: CombatGrid) -> BattleCommand:
	var entry: PlanEntry = PlanEntry.create(SkillCatalog.find(skill_id), 1, 0)
	entry.target = foe
	entry.skill_ap = entry.skill.ap_cost(actor)
	return entry.build_command(actor, grid)

static func _actions_of(r: CombatResolver, c: Combatant) -> Array[CombatResolver.ScheduledAction]:
	var out: Array[CombatResolver.ScheduledAction] = []
	for a: CombatResolver.ScheduledAction in r.scheduled_actions:
		if a.combatant == c:
			out.append(a)
	return out

# Acceptance 2: flow planning and resolution.
static func _test_flow_plan() -> bool:
	var checks: Dictionary = {}
	var guard: SkillDef = SkillCatalog.find("guard")
	var flow: SkillDef = SkillCatalog.find(FLOW)
	checks["fixture: guard is a Defensive skill"] = guard != null and guard.stance() == CombatTypes.StanceType.DEFENSIVE and not guard.keeps_stance()
	checks["stance_after reads the end stance; stance_during the fighting stance"] = flow.stance_after(CombatTypes.StanceType.NEUTRAL) == CombatTypes.StanceType.DEFENSIVE and flow.stance_during(CombatTypes.StanceType.NEUTRAL) == CombatTypes.StanceType.OFFENSIVE
	var grid: CombatGrid = _grid()
	var hero: Combatant = TestHelpers.unit("Hero", grid, Vector2i(3, 4))
	var foe: Combatant = TestHelpers.unit("Foe", grid, Vector2i(4, 4), 900)
	var ctx: PlanContext = TestHelpers.hand_ctx(grid, [hero], [foe])
	var plan := CombatPlan.new(ctx)
	plan.set_steps(hero, [PlanStep.engage(flow, foe), PlanStep.self_skill(guard)] as Array[PlanStep])
	var entries: Array[PlanEntry] = plan.entries_for(hero)
	var flowed: bool = entries.size() == 2 and entries[0].stance_after == CombatTypes.StanceType.DEFENSIVE and entries[1].switch_ap == 0
	checks["Guard after a flow into Defensive plans with switch cost 0"] = flowed
	checks["the flow entry shows 'flows into Defensive'"] = not entries.is_empty() and entries[0].flow_note() == "flows into %s" % CombatTypes.stance_to_string(CombatTypes.StanceType.DEFENSIVE)
	checks["the flow attack's hit is estimated in its fighting stance"] = not entries.is_empty() and entries[0].est_damage == plan.estimate_damage(hero, flow, CombatTypes.StanceType.OFFENSIVE, foe)
	var plain_plan := CombatPlan.new(ctx)
	plain_plan.set_steps(hero, [PlanStep.engage(SkillCatalog.find(PLAIN), foe), PlanStep.self_skill(guard)] as Array[PlanStep])
	var plain_entries: Array[PlanEntry] = plain_plan.entries_for(hero)
	checks["without the field Guard pays the switch (STANCE_SWITCH_COST)"] = plain_entries.size() == 2 and plain_entries[1].switch_ap == CombatTypes.STANCE_SWITCH_COST and plain_entries[0].flow_note().is_empty()
	var option_flow: String = ""
	for o: PlanOption in plan.target_options(hero, foe):
		if o.step.skill != null and o.step.skill.id == FLOW:
			option_flow = o.flow
	checks["the option preview names the flow"] = option_flow == "flows into %s" % CombatTypes.stance_to_string(CombatTypes.StanceType.DEFENSIVE)
	# Resolution: the unit fights in Offensive and ends Defensive, logged right after the attack.
	var lines: Array[String] = []
	var r: CombatResolver = _resolver(grid, lines)
	var solo := CombatPlan.new(ctx)
	solo.set_steps(hero, [PlanStep.engage(flow, foe)] as Array[PlanStep])
	solo.commit(r)
	var attack: AttackCommand = _actions_of(r, hero)[0].command as AttackCommand
	r.resolve_round()
	var flow_line: int = _index_of_prefix(lines, "[Flow] Hero flows into Defensive stance")
	checks["after resolution the unit is Defensive"] = hero.current_stance == CombatTypes.StanceType.DEFENSIVE
	checks["the log has the flow line right after the attack"] = flow_line > 0 and lines[flow_line - 1].begins_with("[Attack] Hero struck Foe")
	checks["the hit was made in Offensive (stance matchup of the attack)"] = attack != null and attack.last_damage > 0
	# A failed command does not flow.
	var grid2: CombatGrid = _grid()
	var hero2: Combatant = TestHelpers.unit("Hero2", grid2, Vector2i(3, 4))
	var gone: Combatant = TestHelpers.unit("Gone", grid2, Vector2i(4, 4))
	var lines2: Array[String] = []
	var r2: CombatResolver = _resolver(grid2, lines2)
	r2.register_combatant(gone)
	var cmd: BattleCommand = _command(hero2, gone, FLOW, grid2)
	cmd.stance_on_start = -1
	r2.schedule_action(hero2, cmd, 1)
	gone.hp = 0
	r2.resolve_round()
	checks["a failed attack does not flow"] = cmd.failed and hero2.current_stance == CombatTypes.StanceType.NEUTRAL and _count_prefix(lines2, "[Flow]") == 0
	return TestHelpers.report(TAG, "flow planning and resolution (acceptance 2)", checks)

# Acceptance 3: a non-riposte DEFEND flowing to Offensive keeps its guard through its last block.
static func _test_flow_defend() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var tank: Combatant = TestHelpers.unit("Tank", grid, Vector2i(3, 4))
	var foe: Combatant = TestHelpers.unit("Foe", grid, Vector2i(4, 4))
	var ctx: PlanContext = TestHelpers.hand_ctx(grid, [tank], [foe])
	var lines: Array[String] = []
	var r: CombatResolver = _resolver(grid, lines)
	var plan := CombatPlan.new(ctx)
	plan.set_steps(tank, [PlanStep.self_skill(SkillCatalog.find(FLOW_DEF))] as Array[PlanStep])
	plan.commit(r)
	var on_last: BattleCommand = _command(foe, tank, PLAIN, grid)
	var after: BattleCommand = _command(foe, tank, PLAIN, grid)
	r.schedule_action(foe, on_last, 2)
	r.schedule_action(foe, after, 3)
	r.resolve_round()
	checks["a hit on the defend's last block is guarded"] = on_last.execution_log.contains("Guard")
	checks["a hit after it is not"] = after.execution_log.begins_with("[Attack]") and not after.execution_log.contains("Guard")
	checks["the tank ends Offensive with a flow line"] = tank.current_stance == CombatTypes.StanceType.OFFENSIVE and _count_prefix(lines, "[Flow] Tank flows into Offensive stance") == 1
	return TestHelpers.report(TAG, "flowing DEFEND keeps its guard to its last block (acceptance 3)", checks)

# Acceptance 4: break on block 2 of a guard over blocks 2-4.
static func _test_break() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var atk: Combatant = TestHelpers.unit("Atk", grid, Vector2i(3, 4), 900)
	var tank: Combatant = TestHelpers.unit("Tank", grid, Vector2i(4, 4), 900)
	var lines: Array[String] = []
	var r: CombatResolver = _resolver(grid, lines)
	var defend := DefendCommand.new(tank, 0.5, 3, 1)
	defend.grant_status = StatusEffects.RIPOSTE
	defend.status_rounds = 1
	defend.stance_on_start = CombatTypes.StanceType.DEFENSIVE
	r.schedule_action(tank, defend, 2)
	var hits: Array[AttackCommand] = []
	for i: int in range(3):
		var cmd: AttackCommand = _command(atk, tank, BREAK if i == 0 else PLAIN, grid) as AttackCommand
		cmd.stance_on_start = CombatTypes.StanceType.OFFENSIVE
		hits.append(cmd)
		r.schedule_action(atk, cmd, 2 + i)
	r.resolve_round()
	checks["the breaking hit (block 2) is guarded: the break comes after it"] = hits[0].execution_log.contains("-50% Guard")
	checks["the riposte counters the breaking hit once"] = hits[0].reactions.size() == 1 and hits[1].reactions.is_empty() and hits[2].reactions.is_empty()
	var riposte_line: int = _index_of_prefix(lines, "[Riposte] Tank counters Atk")
	var break_line: int = _index_of_prefix(lines, "[Break] Atk breaks Tank's stance (now Tactical)")
	checks["the break is logged after the riposte counter"] = riposte_line >= 0 and break_line > riposte_line
	checks["the tank is Tactical without riposte; its guard is marked broken"] = tank.current_stance == CombatTypes.StanceType.TACTICAL and not tank.has_status(StatusEffects.RIPOSTE) and defend.guard_broken
	var unguarded: bool = true
	for i: int in [1, 2]:
		var expected: int = DictUtil.to_int(DamageCalculator.compute(atk, tank, false, false, hits[i].skill_mult, hits[i].last_crit, null, -1, false)["final_dmg"])
		unguarded = unguarded and not hits[i].execution_log.contains("Guard") and hits[i].last_damage == expected
	checks["blocks 3-4 hits equal compute with guard 0"] = unguarded
	return TestHelpers.report(TAG, "stance break after the hit and its riposte (acceptance 4)", checks)

# Acceptance 5: no break on a Tactical / Neutral target, none on a killed target.
static func _test_break_skips() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var atk: Combatant = TestHelpers.unit("Atk", grid, Vector2i(3, 4))
	var stances: Array[CombatTypes.StanceType] = [CombatTypes.StanceType.TACTICAL, CombatTypes.StanceType.NEUTRAL]
	for stance: CombatTypes.StanceType in stances:
		var foe: Combatant = TestHelpers.unit("Foe", grid, Vector2i(4, 4), 900)
		foe.current_stance = stance
		var cmd: BattleCommand = _command(atk, foe, BREAK, grid)
		cmd.rng = _rng(1)
		cmd.execute()
		checks["%s target: no break, no line" % CombatTypes.stance_to_string(stance)] = foe.current_stance == stance and cmd.broken_targets.is_empty() and cmd.combo_notes.is_empty()
		grid.remove_combatant(foe)
	var frail: Combatant = TestHelpers.unit("Frail", grid, Vector2i(4, 4), 1)
	frail.current_stance = CombatTypes.StanceType.DEFENSIVE
	var kill: BattleCommand = _command(atk, frail, BREAK, grid)
	kill.rng = _rng(1)
	kill.execute()
	checks["a killed target is not broken"] = not frail.is_alive() and kill.broken_targets.is_empty() and kill.combo_notes.is_empty()
	var off: Combatant = TestHelpers.unit("Off", grid, Vector2i(3, 5), 900)
	off.current_stance = CombatTypes.StanceType.OFFENSIVE
	var hit_off: BattleCommand = _command(atk, off, BREAK, grid)
	hit_off.rng = _rng(1)
	hit_off.execute()
	checks["an Offensive target is broken too"] = off.current_stance == CombatTypes.StanceType.TACTICAL and hit_off.broken_targets == ([off] as Array[Combatant])
	return TestHelpers.report(TAG, "break skips Tactical, Neutral and killed targets (acceptance 5)", checks)

# Acceptance 6: exploit against active statuses only, the larger multiplier, consume.
static func _test_exploit() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var atk: Combatant = TestHelpers.unit("Atk", grid, Vector2i(3, 4))
	var exposed: Combatant = TestHelpers.unit("Exposed", grid, Vector2i(4, 4), 900)
	_active(exposed, StatusEffects.EXPOSE)
	var cmd: AttackCommand = _command(atk, exposed, EXPLOIT, grid) as AttackCommand
	var want: int = DictUtil.to_int(DamageCalculator.compute(atk, exposed, false, false, cmd.skill_mult * EXPLOIT_MULT, false, _rng(7))["final_dmg"])
	cmd.rng = _rng(7)
	cmd.execute()
	checks["active expose: compute(skill_mult x 1.3)"] = cmd.last_damage == want
	checks["the log names the exploited status"] = cmd.execution_log.ends_with(" (exploits %s)" % StatusEffects.display_name(StatusEffects.EXPOSE))
	var pending: Combatant = TestHelpers.unit("Pending", grid, Vector2i(3, 5), 900)
	StatusEffects.queue(pending, StatusEffects.EXPOSE, 2, 0, "Setup")
	var cmd_p: AttackCommand = _command(atk, pending, EXPLOIT, grid) as AttackCommand
	var plain_p: int = DictUtil.to_int(DamageCalculator.compute(atk, pending, false, false, cmd_p.skill_mult, false, _rng(7))["final_dmg"])
	cmd_p.rng = _rng(7)
	cmd_p.execute()
	checks["pending expose only: the plain hit"] = cmd_p.last_damage == plain_p and not cmd_p.execution_log.contains("exploits")
	var both: Combatant = TestHelpers.unit("Both", grid, Vector2i(2, 4), 900)
	_active(both, StatusEffects.EXPOSE)
	_active(both, StatusEffects.SLOW)
	checks["two matches: the larger multiplier"] = is_equal_approx(DictUtil.to_float(SkillDef.exploit_match(SkillCatalog.find(EXPLOIT).exploit, both)["mult"]), maxf(EXPLOIT_MULT, SECOND_MULT))
	var reversed: Array[Dictionary] = [{"status": StatusEffects.EXPOSE, "power_mult": SECOND_MULT}, {"status": StatusEffects.SLOW, "power_mult": EXPLOIT_MULT}]
	checks["...in either order"] = str(SkillDef.exploit_match(reversed, both)["status"]) == StatusEffects.SLOW
	var cmd_b: AttackCommand = _command(atk, both, EXPLOIT, grid) as AttackCommand
	var want_b: int = DictUtil.to_int(DamageCalculator.compute(atk, both, false, false, cmd_b.skill_mult * maxf(EXPLOIT_MULT, SECOND_MULT), false, _rng(9))["final_dmg"])
	cmd_b.rng = _rng(9)
	cmd_b.execute()
	checks["the resolved hit uses the larger multiplier"] = cmd_b.last_damage == want_b
	# Consume: the active status goes after the hit; a pending one stays; the second hit is plain.
	var target: Combatant = TestHelpers.unit("Target", grid, Vector2i(3, 3), 900)
	_active(target, StatusEffects.EXPOSE)
	StatusEffects.queue(target, StatusEffects.EXPOSE, 1, 0, "Later")
	var first: AttackCommand = _command(atk, target, CONSUME, grid) as AttackCommand
	first.rng = _rng(3)
	first.execute()
	checks["consume removes the active status"] = not target.has_status(StatusEffects.EXPOSE) and target.pending_statuses.has(StatusEffects.EXPOSE)
	var second: AttackCommand = _command(atk, target, CONSUME, grid) as AttackCommand
	var plain_2: int = DictUtil.to_int(DamageCalculator.compute(atk, target, false, false, second.skill_mult, false, _rng(4))["final_dmg"])
	second.rng = _rng(4)
	second.execute()
	checks["the second hit of the round is plain"] = second.last_damage == plain_2 and not second.execution_log.contains("exploits")
	# Area attacks exploit per unit hit.
	var area_rows: Dictionary = TestHelpers.attack("t_cb_area", {"level": 1, "blocks": 1, "area": "PLUS", "exploit": [{"status": StatusEffects.EXPOSE, "power_mult": EXPLOIT_MULT}]})
	var area_skill: SkillDef = ContentFactory.skill(area_rows)
	var grid_a: CombatGrid = _grid()
	var caster: Combatant = TestHelpers.unit("Caster", grid_a, Vector2i(2, 4))
	var hit_x: Combatant = TestHelpers.unit("X", grid_a, Vector2i(4, 4), 900)
	var hit_y: Combatant = TestHelpers.unit("Y", grid_a, Vector2i(5, 4), 900)
	_active(hit_x, StatusEffects.EXPOSE)
	var area := AreaAttackCommand.new(caster, grid_a, [Vector2i(4, 4), Vector2i(5, 4)] as Array[Vector2i], area_skill.power_for(caster), 1)
	area.exploit = area_skill.exploit
	var r_seed: RandomNumberGenerator = _rng(11)
	var want_x: int = DictUtil.to_int(DamageCalculator.compute(caster, hit_x, false, false, area.skill_mult * EXPLOIT_MULT, false, r_seed)["final_dmg"])
	var want_y: int = DictUtil.to_int(DamageCalculator.compute(caster, hit_y, false, false, area.skill_mult, false, r_seed)["final_dmg"])
	area.rng = _rng(11)
	area.execute()
	checks["an area hit exploits per unit"] = hit_x.max_hp - hit_x.hp == want_x and hit_y.max_hp - hit_y.hp == want_y
	return TestHelpers.report(TAG, "exploit active statuses, larger multiplier, consume (acceptance 6)", checks)

# Acceptance 7: planner estimates equal resolved damage (no crit) for exploit and for the post-break hits.
static func _test_estimates() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var atk: Combatant = TestHelpers.unit("Atk", grid, Vector2i(3, 4), 900)
	var tank: Combatant = TestHelpers.unit("Tank", grid, Vector2i(4, 4), 900)
	tank.current_stance = CombatTypes.StanceType.TACTICAL   # its guard (blocks 2-4) enters Defensive; the break returns it to Tactical
	_active(tank, StatusEffects.EXPOSE)
	var ctx: PlanContext = TestHelpers.hand_ctx(grid, [atk], [tank])
	atk.power_override = 20
	var plan := CombatPlan.new(ctx)
	var exploit_skill: SkillDef = SkillCatalog.find(CONSUME)
	plan.set_steps(atk, [PlanStep.engage(SkillCatalog.find(BREAK2), tank), PlanStep.engage(exploit_skill, tank), PlanStep.engage(exploit_skill, tank)] as Array[PlanStep])
	var entries: Array[PlanEntry] = plan.entries_for(atk)
	checks["fixture: three attack entries"] = entries.size() == 3 and entries[0].end_block() == 2 and entries[1].start_block == 3
	if entries.size() != 3:
		return TestHelpers.report(TAG, "planner estimates match resolution (acceptance 7)", checks)
	checks["the exploit estimate uses x1.3; the consumed second hit is plain"] = entries[1].est_damage == plan.estimate_damage(atk, exploit_skill, CombatTypes.StanceType.OFFENSIVE, tank) \
		and entries[2].est_damage == plan.estimate_damage(atk, SkillCatalog.find(PLAIN), CombatTypes.StanceType.OFFENSIVE, tank) and entries[1].est_damage > entries[2].est_damage
	var chips: Array[String] = []
	for prefix: int in [1, 2]:
		var probe := CombatPlan.new(ctx)
		var probe_steps: Array[PlanStep] = [PlanStep.engage(SkillCatalog.find(BREAK2), tank)]
		if prefix == 2:
			probe_steps.append(PlanStep.engage(exploit_skill, tank))
		probe.set_steps(atk, probe_steps)
		for o: PlanOption in probe.target_options(atk, tank):
			if o.step.skill == exploit_skill:
				chips.append(o.exploits)
	checks["the 'exploits' chip shows for the first hit, not after a consuming one"] = chips == ([StatusEffects.display_name(StatusEffects.EXPOSE), ""] as Array[String])
	var lines: Array[String] = []
	var r: CombatResolver = _resolver(grid, lines)
	plan.commit(r)
	var defend := DefendCommand.new(tank, 0.5, 3, 1)
	defend.stance_on_start = CombatTypes.StanceType.DEFENSIVE
	r.schedule_action(tank, defend, 2)
	var hits: Array[AttackCommand] = []
	for a: CombatResolver.ScheduledAction in _actions_of(r, atk):
		if a.command is AttackCommand:
			hits.append(a.command as AttackCommand)
	r.resolve_round()
	checks["the break landed"] = hits.size() == 3 and _count_prefix(lines, "[Break] Atk breaks Tank") == 1
	var exact: bool = hits.size() == 3
	for i: int in [1, 2]:
		if exact and not hits[i].last_crit:
			exact = hits[i].last_damage == entries[i].est_damage
		elif exact:
			var no_crit: int = DictUtil.to_int(DamageCalculator.compute(atk, tank, false, false, hits[i].skill_mult * (EXPLOIT_MULT if i == 1 else 1.0), false, null, -1, false)["final_dmg"])
			exact = no_crit == entries[i].est_damage
	checks["post-break exploit hits resolve to the estimate (no crit)"] = exact
	return TestHelpers.report(TAG, "planner estimates match resolution (acceptance 7)", checks)

static func _state(c: Combatant) -> Dictionary:
	return {"hp": c.hp, "stance": c.current_stance, "statuses": c.statuses.duplicate(), "rounds": c.status_rounds_left.duplicate(), "magnitudes": c.status_magnitudes.duplicate(), "sources": c.status_sources.duplicate()}

# Acceptance 8: round undo restores stances, consumed statuses, and guard_broken lives on rebuilt commands.
static func _test_undo() -> bool:
	var checks: Dictionary = {}
	var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	session.start()
	session.plan_apply_preset_all("balanced")
	session.confirm_plan()
	TestHelpers.resolve_until_replay(session)
	checks["round 2 reachable"] = session.can_advance_round()
	var hero: Combatant = null
	for a: Combatant in session.allies:
		if a.is_active() and hero == null:
			hero = a
	var foe: Combatant = null
	for e: Combatant in session.enemies:
		if e.is_active() and foe == null:
			foe = e
	if session.can_advance_round() and hero != null and foe != null:
		StatusEffects.queue(foe, StatusEffects.EXPOSE, 2, 0, "Setup")
		foe.current_stance = CombatTypes.StanceType.DEFENSIVE
		foe.max_hp = 9999   # the combo hit must not defeat it (a defeated unit is not broken)
		foe.hp = foe.max_hp
		session.advance_round()
		var start: Dictionary = _state(foe)
		checks["the round starts with an active expose on a Defensive foe"] = foe.has_status(StatusEffects.EXPOSE) and foe.current_stance == CombatTypes.StanceType.DEFENSIVE
		var cmd: BattleCommand = _command(hero, foe, CONSUME, null)
		cmd.disrupt = SkillDef.Disrupt.STANCE_BREAK
		cmd.rng = _rng(5)
		cmd.execute()
		checks["the combo hit broke the stance and consumed the status"] = foe.current_stance == CombatTypes.StanceType.TACTICAL and not foe.has_status(StatusEffects.EXPOSE)
		session.reset_round()
		checks["reset_round restores stance, HP and the consumed status exactly"] = _state(foe) == start
		var any_broken: bool = false
		for a: CombatResolver.ScheduledAction in session.resolver.scheduled_actions:
			if a.command is DefendCommand and (a.command as DefendCommand).guard_broken:
				any_broken = true
		checks["no rebuilt guard starts broken"] = not any_broken
	session.dispose()
	return TestHelpers.report(TAG, "round undo restores combo effects (acceptance 8)", checks)

# ------------------------------------------------------------------------------
# Part C: AI (rule 6)
# ------------------------------------------------------------------------------

# The first damaging single-target attack the AI drafted for `actor` ("" when none).
static func _first_strike(steps: Array[PlanStep]) -> String:
	for step: PlanStep in steps:
		if step.mode == PlanStep.Mode.ENGAGE and step.skill != null and step.skill.kind == SkillDef.Kind.ATTACK and step.skill.deals_damage() and not step.skill.is_area():
			return step.skill.id
	return ""

static func _ai_duel(foe_stance: CombatTypes.StanceType, foe_status: String = "", foe_riposte_plan: bool = false) -> String:
	var grid: CombatGrid = _grid()
	var atk: Combatant = TestHelpers.unit("Atk", grid, Vector2i(3, 4), 900)
	var foe: Combatant = TestHelpers.unit("Foe", grid, Vector2i(4, 4), 900)
	foe.current_stance = foe_stance
	if not foe_status.is_empty():
		if foe_status == StatusEffects.RIPOSTE:
			StatusEffects.grant_now(foe, foe_status, 1)
		else:
			_active(foe, foe_status)
	var ctx: PlanContext = TestHelpers.hand_ctx(grid, [atk], [foe])
	var shared := CombatPlan.new(ctx)
	if foe_riposte_plan:
		shared.set_steps(foe, [PlanStep.self_skill(SkillCatalog.find(RIPOSTE_DEF))] as Array[PlanStep])
	var profile: AiProfile = AiProfile.create(AiProfile.Role.AGGRESSOR, 0.0, 3)
	return _first_strike(TacticalAI.build_steps(atk, ctx, profile, shared))

# True when `skill_id` is one of the fixture stance-break skills.
static func _breaks(skill_id: String) -> bool:
	var skill: SkillDef = SkillCatalog.find(skill_id)
	return skill != null and skill.disrupt == SkillDef.Disrupt.STANCE_BREAK

# Acceptance 9: against a Defensive foe the AI plans the affordable stance-break skill.
static func _test_ai_break() -> bool:
	var checks: Dictionary = {}
	checks["Defensive foe: the AI opens with the break skill"] = _breaks(_ai_duel(CombatTypes.StanceType.DEFENSIVE))
	checks["riposting foe: the break skill"] = _breaks(_ai_duel(CombatTypes.StanceType.NEUTRAL, StatusEffects.RIPOSTE))
	checks["a foe planning a riposte: the break skill"] = _breaks(_ai_duel(CombatTypes.StanceType.NEUTRAL, "", true))
	var plain: String = _ai_duel(CombatTypes.StanceType.TACTICAL)
	checks["Tactical foe: the AI strikes, and not with an exploit skill"] = not plain.is_empty() and plain != EXPLOIT and plain != CONSUME
	return TestHelpers.report(TAG, "AI stance break vs guarded foes (acceptance 9)", checks)

static func _test_ai_exploit() -> bool:
	var checks: Dictionary = {}
	var with_status: String = _ai_duel(CombatTypes.StanceType.TACTICAL, StatusEffects.EXPOSE)
	checks["active expose: the AI picks an exploit skill"] = with_status == EXPLOIT or with_status == CONSUME
	var without: String = _ai_duel(CombatTypes.StanceType.TACTICAL, StatusEffects.SLOW)
	checks["the slow-listed exploit applies too (any listed status)"] = without == EXPLOIT
	var ordered: Array[SkillDef] = TacticalAI._combo_ordered(CombatPlan.new(TestHelpers.hand_ctx(_grid(), [], [])), [SkillCatalog.find(EXPLOIT), SkillCatalog.find(PLAIN)] as Array[SkillDef], TestHelpers.unit("Bare", _grid(), Vector2i(1, 1)))
	checks["no listed status: the exploit skill goes after the plain strike"] = ordered.size() == 2 and ordered[0].id == PLAIN and ordered[1].id == EXPLOIT
	return TestHelpers.report(TAG, "AI exploit only when it applies (rule 6c)", checks)
