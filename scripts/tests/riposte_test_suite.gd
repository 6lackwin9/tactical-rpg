class_name RiposteTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Riposte Test Suite (milestone M2a)
# The `riposte` DEFEND skill and RiposteRules: counter damage = COUNTER_MULT of attack
# power through DamageCalculator.compute, adjacent melee only, once per attacker per round,
# never when the unit dies, removed on stance change, kept through moves, lasts the
# skill's rounds, guard below Guard. Units are hand-built (fixed strike power, no armor);
# expectations come from the rules, not magic numbers.
# ==============================================================================

const SANDBOX: String = "user://riposte_test"

static func _report(label: String, checks: Dictionary) -> bool:
	var failed: Array[String] = []
	for key: Variant in checks:
		if not DictUtil.to_bool(checks[key]):
			failed.append(str(key))
	var ok: bool = failed.is_empty()
	print("    [Riposte] %s: %s%s" % [label, "PASS" if ok else "FAIL", "" if ok else " (%s)" % ", ".join(failed)])
	return ok

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING RIPOSTE TEST (M2a)                      ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_counter_rules() and ok
	ok = _test_conditions() and ok
	ok = _test_round_flow() and ok
	ok = _test_stance_and_duration() and ok
	ok = _test_undo_round() and ok
	ok = _test_catalog() and ok
	return ok

static func _unit(unit_name: String, hp: int, attack_range: int, grid: CombatGrid, pos: Vector2i, learn: Array[String] = []) -> Combatant:
	var c := Combatant.new(unit_name, hp, 10, attack_range)
	c.power_override = 20
	c.armor_override = 0
	for skill_id: String in learn:
		c.learn_skill(skill_id)
	grid.place_combatant(c, pos)
	return c

static func _grid() -> CombatGrid:
	return CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])

static func _rng(seed_value: int) -> RandomNumberGenerator:
	var r := RandomNumberGenerator.new()
	r.seed = seed_value
	return r

# ------------------------------------------------------------------------------

static func _test_counter_rules() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var tank: Combatant = _unit("Tank", 300, 1, grid, Vector2i(3, 4))
	var foe: Combatant = _unit("Foe", 300, 1, grid, Vector2i(4, 4))
	StatusEffects.grant_now(tank, StatusEffects.RIPOSTE, 2)

	# Same seed -> same crit roll, so the counter must equal compute() at COUNTER_MULT exactly,
	# and be about half of the full-power hit.
	var expected: Dictionary = DamageCalculator.compute(tank, foe, false, false, RiposteRules.COUNTER_MULT, false, _rng(5))
	var full: Dictionary = DamageCalculator.compute(tank, foe, false, false, 1.0, false, _rng(5))
	var hp_before: int = foe.hp
	var result: Dictionary = RiposteRules.counter(foe, tank, grid, false, _rng(5))
	checks["a counter happened"] = not result.is_empty()
	checks["damage = compute() at COUNTER_MULT"] = DictUtil.to_int(result["damage"]) == DictUtil.to_int(expected["final_dmg"]) and hp_before - foe.hp == DictUtil.to_int(result["damage"])
	checks["COUNTER_MULT is 50% of attack power"] = is_equal_approx(RiposteRules.COUNTER_MULT, 0.5) and absi(DictUtil.to_int(full["final_dmg"]) - 2 * DictUtil.to_int(expected["final_dmg"])) <= 1
	checks["the attacker is remembered for this round"] = tank.riposted.has(foe)
	checks["log line is tagged [Riposte]"] = str(result["log"]).begins_with("[Riposte]")
	var second: Dictionary = RiposteRules.counter(foe, tank, grid, false, _rng(5))
	checks["not twice against the same attacker in a round"] = second.is_empty() and foe.hp == hp_before - DictUtil.to_int(result["damage"])
	StatusEffects.roll_over(tank, 4)
	checks["a new round resets the once-per-attacker memory"] = tank.riposted.is_empty()
	var other: Combatant = _unit("Other", 300, 1, grid, Vector2i(3, 5))
	StatusEffects.grant_now(tank, StatusEffects.RIPOSTE, 2)
	checks["a different attacker is countered separately"] = not RiposteRules.counter(foe, tank, grid, false, _rng(1)).is_empty() and not RiposteRules.counter(other, tank, grid, false, _rng(1)).is_empty()
	return _report("Counter damage and once-per-attacker-per-round", checks)

static func _test_conditions() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var tank: Combatant = _unit("Tank", 300, 1, grid, Vector2i(3, 4))
	var adjacent: Combatant = _unit("Adj", 300, 1, grid, Vector2i(4, 4))
	var far: Combatant = _unit("Far", 300, 2, grid, Vector2i(5, 4))
	StatusEffects.grant_now(tank, StatusEffects.RIPOSTE, 2)
	checks["adjacent melee: counters"] = RiposteRules.can_counter(adjacent, tank, grid, false)
	checks["a ranged attack at point-blank is not countered"] = not RiposteRules.can_counter(adjacent, tank, grid, true)
	checks["not adjacent: no counter"] = not RiposteRules.can_counter(far, tank, grid, false)
	var plain: Combatant = _unit("Plain", 300, 1, grid, Vector2i(3, 5))
	checks["without the status: no counter"] = not RiposteRules.can_counter(adjacent, plain, grid, false)
	checks["no grid: no counter"] = not RiposteRules.can_counter(adjacent, tank, null, false)

	# Through the real commands: a ranged shot from distance 2 does not provoke a counter.
	var archer_cmd := AttackCommand.new(far, tank, 1.0, 1, 0, grid)
	far.attack_range = 3
	grid.move_combatant(far, Vector2i(1, 0))   # (5,4) -> (6,4): distance 3
	archer_cmd.rng = _rng(3)
	archer_cmd.execute()
	checks["a ranged AttackCommand does no riposte"] = archer_cmd.reactions.is_empty() and far.hp == far.max_hp and tank.hp < tank.max_hp

	# Melee AttackCommand does, once the target survives; the log carries the reaction.
	tank.hp = tank.max_hp
	var melee_cmd := AttackCommand.new(adjacent, tank, 1.0, 1, 0, grid)
	melee_cmd.rng = _rng(3)
	melee_cmd.execute()
	checks["a melee AttackCommand is countered"] = melee_cmd.reactions.size() == 1 and adjacent.hp < adjacent.max_hp
	var off_cmd := AttackCommand.new(adjacent, tank, 1.0, 1, 0, grid)
	off_cmd.rng = _rng(3)
	off_cmd.execute()
	checks["a second hit in the same round is not countered again"] = off_cmd.reactions.is_empty()

	# Area attacks provoke counters from adjacent riposting victims too.
	var area_grid: CombatGrid = _grid()
	var a_tank: Combatant = _unit("ATank", 300, 1, area_grid, Vector2i(3, 4))
	var a_foe: Combatant = _unit("AFoe", 300, 1, area_grid, Vector2i(4, 4))
	StatusEffects.grant_now(a_tank, StatusEffects.RIPOSTE, 2)
	var cells: Array[Vector2i] = [Vector2i(3, 4)]
	var area_cmd := AreaAttackCommand.new(a_foe, area_grid, cells, 1.0, 1)
	area_cmd.rng = _rng(2)
	area_cmd.execute()
	checks["an area hit is countered by an adjacent riposter"] = area_cmd.reactions.size() == 1 and a_foe.hp < a_foe.max_hp

	# A unit that dies from the hit does not counter.
	var dead_grid: CombatGrid = _grid()
	var frail: Combatant = _unit("Frail", 5, 1, dead_grid, Vector2i(3, 4))
	var killer: Combatant = _unit("Killer", 300, 1, dead_grid, Vector2i(4, 4))
	StatusEffects.grant_now(frail, StatusEffects.RIPOSTE, 2)
	var kill_cmd := AttackCommand.new(killer, frail, 1.0, 1, 0, dead_grid)
	kill_cmd.rng = _rng(4)
	kill_cmd.execute()
	checks["a defeated riposter does not counter"] = not frail.is_alive() and kill_cmd.reactions.is_empty() and killer.hp == killer.max_hp
	return _report("Only adjacent melee, only while the unit survives", checks)

static func _riposte_steps(tank: Combatant, extra: Array[PlanStep]) -> Array[PlanStep]:
	var steps: Array[PlanStep] = [PlanStep.self_skill(SkillCatalog.find("riposte"))]
	steps.append_array(extra)
	return steps

static func _ctx(grid: CombatGrid, allies: Array[Combatant], enemies: Array[Combatant]) -> PlanContext:
	var profiles: Dictionary = {}
	for c: Combatant in allies + enemies:
		profiles[c] = AiProfile.create(AiProfile.Role.AGGRESSOR, 0.0, 3)
	return PlanContext.create(grid, allies, enemies, profiles)

# Plays one planned round; returns the resolver messages.
static func _play(ctx: PlanContext, plans: Dictionary) -> Array[String]:
	var messages: Array[String] = []
	var resolver := CombatResolver.new()
	resolver.grid = ctx.grid
	resolver.log_emitted.connect(func(msg: String) -> void: messages.append(msg))
	var cp := CombatPlan.new(ctx)
	for key: Variant in plans:
		if key is Combatant:
			var actor: Combatant = key
			var steps: Array[PlanStep] = []
			var raw: Variant = plans[key]
			if raw is Array:
				var raw_list: Array = raw
				steps.assign(raw_list)
			cp.set_steps(actor, steps)
	cp.commit(resolver)
	resolver.resolve_round()
	return messages

static func _count(messages: Array[String], prefix: String) -> int:
	var n: int = 0
	for m: String in messages:
		if m.begins_with(prefix):
			n += 1
	return n

static func _test_round_flow() -> bool:
	var checks: Dictionary = {}
	var strike: SkillDef = SkillCatalog.find("strike")
	var grid: CombatGrid = _grid()
	var tank: Combatant = _unit("Tank", 400, 1, grid, Vector2i(3, 4), ["riposte"])
	var foe: Combatant = _unit("Foe", 400, 1, grid, Vector2i(4, 4), ["strike"])
	var ctx: PlanContext = _ctx(grid, [tank], [foe])
	var foe_steps: Array[PlanStep] = [PlanStep.engage(strike, tank), PlanStep.engage(strike, tank)]
	var tank_steps: Array[PlanStep] = _riposte_steps(tank, [])
	var messages: Array[String] = _play(ctx, {tank: tank_steps, foe: foe_steps})
	checks["the riposte skill resolves and is logged"] = _count(messages, "[Riposte] Tank readies") == 1
	checks["active from the block the skill starts: the foe's strikes are answered"] = _count(messages, "[Riposte] Tank counters") >= 1
	checks["two strikes, one counter per round"] = _count(messages, "[Riposte] Tank counters") == 1 and _count(messages, "[Attack] Foe struck") == 2
	checks["the foe lost HP to the counter"] = foe.hp < foe.max_hp
	checks["the tank holds Defensive stance and keeps riposte after the round"] = tank.current_stance == CombatTypes.StanceType.DEFENSIVE and tank.has_status(StatusEffects.RIPOSTE)
	return _report("Planner round: skill start -> counter -> log", checks)

static func _test_stance_and_duration() -> bool:
	var checks: Dictionary = {}
	var riposte: SkillDef = SkillCatalog.find("riposte")
	var strike: SkillDef = SkillCatalog.find("strike")
	var move: SkillDef = SkillCatalog.lowest(SkillDef.Kind.MOVE)

	# Moving keeps stance, so it keeps riposte.
	var grid: CombatGrid = _grid()
	var tank: Combatant = _unit("Tank", 400, 1, grid, Vector2i(3, 4), ["riposte"])
	var foe: Combatant = _unit("Foe", 400, 1, grid, Vector2i(8, 8), ["strike"])
	var ctx: PlanContext = _ctx(grid, [tank], [foe])
	_play(ctx, {tank: _riposte_steps(tank, [])})
	StatusEffects.roll_over(tank, 4)   # round 2: the riposte (2 rounds) is still on
	var move_steps: Array[PlanStep] = [PlanStep.move(move, Vector2i(0, -1), 1)]   # the planner wants moves first, so move next round
	_play(ctx, {tank: move_steps})
	checks["moving keeps the stance and the riposte"] = tank.current_stance == CombatTypes.StanceType.DEFENSIVE and tank.has_status(StatusEffects.RIPOSTE) and grid.get_position(tank) == Vector2i(3, 3)

	# An attack skill forces an Offensive stance switch, which drops it.
	var grid2: CombatGrid = _grid()
	var tank2: Combatant = _unit("Tank", 400, 1, grid2, Vector2i(3, 4), ["riposte", "strike"])
	var target2: Combatant = _unit("Dummy", 400, 1, grid2, Vector2i(4, 4))
	var ctx2: PlanContext = _ctx(grid2, [tank2], [target2])
	var attack_steps: Array[PlanStep] = _riposte_steps(tank2, [PlanStep.engage(strike, target2)])
	_play(ctx2, {tank2: attack_steps})
	checks["using an attack skill (stance switch) removes riposte"] = tank2.current_stance == CombatTypes.StanceType.OFFENSIVE and not tank2.has_status(StatusEffects.RIPOSTE)

	# Explicit stance change, direct.
	var grid3: CombatGrid = _grid()
	var tank3: Combatant = _unit("Tank", 400, 1, grid3, Vector2i(3, 4))
	tank3.set_stance(CombatTypes.StanceType.DEFENSIVE)
	StatusEffects.grant_now(tank3, StatusEffects.RIPOSTE, 2)
	tank3.set_stance(CombatTypes.StanceType.DEFENSIVE)
	checks["re-asserting the same stance keeps it"] = tank3.has_status(StatusEffects.RIPOSTE)
	tank3.set_stance(CombatTypes.StanceType.TACTICAL)
	checks["set_stance to a new stance removes it, with its duration"] = not tank3.has_status(StatusEffects.RIPOSTE) and not tank3.status_rounds_left.has(StatusEffects.RIPOSTE)

	# Duration: the skill's status_rounds, counting the round it was cast.
	var rounds: int = riposte.status_rounds
	var c := Combatant.new("Timer", 100, 10, 1)
	StatusEffects.grant_now(c, StatusEffects.RIPOSTE, rounds)
	var lasted: int = 1
	for i: int in range(6):
		StatusEffects.roll_over(c, 4)
		if not c.has_status(StatusEffects.RIPOSTE):
			break
		lasted += 1
	checks["riposte lasts exactly the skill's status_rounds"] = rounds >= 1 and lasted == rounds
	checks["a riposte skill is not resistable (not a debuff)"] = not StatusEffects.resistable().has(StatusEffects.RIPOSTE) and StatusEffects.buffs().has(StatusEffects.RIPOSTE)
	return _report("Stance change removes it, moving keeps it, 2-round duration", checks)

static func _test_catalog() -> bool:
	var checks: Dictionary = {}
	var riposte: SkillDef = SkillCatalog.find("riposte")
	var guard: SkillDef = SkillCatalog.find("guard")
	checks["riposte exists as a DEFEND skill whose AP is its level"] = riposte != null and riposte.kind == SkillDef.Kind.DEFEND and riposte.ap_cost() == riposte.level
	checks["it grants the riposte status for its status_rounds"] = riposte != null and riposte.grant_status == StatusEffects.RIPOSTE and riposte.status_rounds >= 1
	checks["its mitigation is lower than Guard's"] = riposte != null and guard != null and riposte.mitigation < guard.mitigation
	checks["it is learned_only"] = SkillCatalog.is_learned_only("riposte")
	var entry: Dictionary = ContentCatalog.get_entry("skills", "riposte")
	checks["entry validates and round-trips"] = not entry.is_empty() and ContentCatalog.validate_entry("skills", entry, "riposte").is_empty() and ContentFactory.skill_to_entry(ContentFactory.skill(entry)) == ContentSchema.normalize("skills", entry)
	checks["whole catalog validates"] = ContentCatalog.validate_all().is_empty()
	checks["schema offers riposte as a grant"] = ContentSchema.options_of({"options_from": "status_buff"}).has("riposte")
	# Validation refuses bad data (sandbox copy; nothing is saved).
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, SANDBOX))
	var forked: bool = ContentCatalog.fork_to(SANDBOX) == OK
	checks["sandbox fork"] = forked
	if forked:
		checks["a DEFEND skill may only grant riposte"] = not ContentCatalog.upsert("skills", {"id": "bad_def", "name": "x", "kind": "DEFEND", "grant_status": "haste"}).is_empty()
		checks["a SUPPORT skill may not grant riposte"] = not ContentCatalog.upsert("skills", {"id": "bad_sup", "name": "x", "kind": "SUPPORT", "target_mode": "SELF", "grant_status": "riposte"}).is_empty()
		checks["an attack may not grant riposte"] = not ContentCatalog.upsert("skills", {"id": "bad_atk", "name": "x", "kind": "ATTACK", "grant_status": "riposte"}).is_empty()
		checks["a valid riposting defend is accepted"] = ContentCatalog.upsert("skills", {"id": "good_def", "name": "Good", "kind": "DEFEND", "mitigation": 0.1, "grant_status": "riposte", "status_rounds": 3}).is_empty()
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return _report("Catalog skill, validation and schema", checks)

# Undoing a round rewinds a riposte granted during it (round 2+; round 1 restarts the battle).
static func _test_undo_round() -> bool:
	var checks: Dictionary = {}
	var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	session.start()
	session.plan_apply_preset_all("balanced")
	session.confirm_plan()
	for i: int in range(600):
		if session.phase == CombatSession.Phase.REPLAY:
			break
		session.step_next()
	checks["round 2 reachable"] = session.can_advance_round()
	if session.can_advance_round():
		session.advance_round()
		var tank: Combatant = session.allies[0]
		checks["no riposte at round start"] = not tank.has_status(StatusEffects.RIPOSTE)
		StatusEffects.grant_now(tank, StatusEffects.RIPOSTE, 2)
		tank.riposted[session.enemies[0]] = true
		session.reset_round()
		checks["riposte rewound"] = not tank.has_status(StatusEffects.RIPOSTE)
		checks["counter record rewound"] = tank.riposted.is_empty()
	session.dispose()
	return _report("Round undo rewinds riposte and its once-per-round record", checks)
