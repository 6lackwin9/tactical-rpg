class_name CoreRulesTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Core Combat Rules Test Suite (current planner rules)
# Replaces the old hand-scripted battles (10-block rounds, scripted commands):
# every scenario here is planned with CombatPlan exactly like the game does and
# resolved by the real CombatResolver / CombatSession.
#   guard mitigation & the stance matchup, stances persisting (incl. Shield
#   Bash staying Defensive), Focus duration, casualty cancellation, withdrawal,
#   and the resolution feed of a full catalog battle (Goblin Ambush).
# ==============================================================================

static func _report(label: String, ok: bool, detail: String = "") -> bool:
	print("    [Rules] %s: %s%s" % [label, "PASS" if ok else "FAIL", (" (%s)" % detail) if not detail.is_empty() else ""])
	return ok

static func _grid() -> CombatGrid:
	return CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])

static func _side(units: Array[Combatant]) -> Array[Combatant]:
	return units

# Plans `steps` per unit (Dictionary unit -> Array[PlanStep]) and returns the committed resolver.
static func _commit(ctx: PlanContext, steps: Dictionary) -> CombatResolver:
	var resolver := CombatResolver.new()
	resolver.grid = ctx.grid
	resolver.total_blocks = ctx.total_blocks
	for c: Combatant in ctx.allies + ctx.enemies:
		resolver.register_combatant(c)
	var plan := CombatPlan.new(ctx)
	for unit: Variant in steps:
		var c: Combatant = unit
		var raw: Array = steps[unit]
		var list: Array[PlanStep] = []
		list.assign(raw)
		plan.set_steps(c, list)
	plan.commit(resolver)
	return resolver

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING CORE COMBAT RULES TEST (PLANNER)       ===")
	print("=======================================================")
	var ok: bool = true
	ok = _test_guard_and_matchup() and ok
	ok = _test_stances_persist() and ok
	ok = _test_focus_duration() and ok
	ok = _test_casualty_cancellation() and ok
	ok = _test_withdrawal() and ok
	ok = _test_feed_from_catalog_battle() and ok
	return ok

# A Brace (-50%) taken in Defensive stance against an Offensive strike: matchup 0.75, then the guard.
static func _test_guard_and_matchup() -> bool:
	var strike: SkillDef = SkillCatalog.find("strike")
	var brace: SkillDef = SkillCatalog.find("brace")
	var grid := _grid()
	var hitter := Combatant.new("Hitter", 100, 12, 1)
	var guard := Combatant.new("Guard", 100, 10, 1)
	var hitter2 := Combatant.new("Hitter2", 100, 12, 1)
	var dummy := Combatant.new("Dummy", 100, 10, 1)
	grid.place_combatant(hitter, Vector2i(3, 4))
	grid.place_combatant(guard, Vector2i(4, 4))
	grid.place_combatant(hitter2, Vector2i(3, 6))
	grid.place_combatant(dummy, Vector2i(4, 6))
	var ctx: PlanContext = ArenaTestSuite._hand_ctx(grid, _side([hitter, hitter2]), _side([guard, dummy]))
	var resolver: CombatResolver = _commit(ctx, {
		hitter: [PlanStep.engage(strike, guard)], guard: [PlanStep.self_skill(brace)],
		hitter2: [PlanStep.engage(strike, dummy)],
	})
	resolver.resolve_round()
	var power: int = ctx.power_of(hitter)   # strike power x1.0
	var struck: int = maxi(1, int(roundf(float(power) * CombatTypes.STANCE_DISADVANTAGE)))
	var expected_guarded: int = maxi(1, int(roundf(float(struck) * (1.0 - brace.mitigation))))
	var guarded_taken: int = 100 - guard.hp
	var open_taken: int = 100 - dummy.hp
	var ok: bool = guarded_taken == expected_guarded and open_taken == power and guarded_taken < open_taken
	return _report("Guard mitigation and stance matchup: Brace -50% in Defensive vs an Offensive strike (x0.75)", ok,
		"guarded %d (expected %d), unguarded %d (power %d)" % [guarded_taken, expected_guarded, open_taken, power])

# Stances set by a skill stay after it resolves and carry into the next round;
# Shield Bash (a Defensive-stance attack) leaves its user Defensive.
static func _test_stances_persist() -> bool:
	var grid := _grid()
	var basher := Combatant.new("Basher", 100, 12, 1)
	var striker := Combatant.new("Striker", 100, 12, 1)
	var target := Combatant.new("Target", 300, 10, 1)
	var target2 := Combatant.new("Target2", 300, 10, 1)
	grid.place_combatant(basher, Vector2i(3, 4))
	grid.place_combatant(target, Vector2i(4, 4))
	grid.place_combatant(striker, Vector2i(3, 6))
	grid.place_combatant(target2, Vector2i(4, 6))
	var ctx: PlanContext = ArenaTestSuite._hand_ctx(grid, _side([basher, striker]), _side([target, target2]))
	var plan_check := CombatPlan.new(ctx)
	plan_check.add_step(basher, PlanStep.engage(SkillCatalog.find("shield_bash"), target))
	var projected: CombatTypes.StanceType = plan_check.compile(basher).stance
	var resolver: CombatResolver = _commit(ctx, {
		basher: [PlanStep.engage(SkillCatalog.find("shield_bash"), target)],
		striker: [PlanStep.engage(SkillCatalog.find("strike"), target2), PlanStep.self_skill(SkillCatalog.find("guard"))],
	})
	resolver.resolve_round()
	var after_round: bool = basher.current_stance == CombatTypes.StanceType.DEFENSIVE and striker.current_stance == CombatTypes.StanceType.DEFENSIVE
	StatusEffects.roll_over(basher, 4)
	var next_plan := CombatPlan.new(ctx)
	var carried: bool = next_plan.compile(basher).stance == CombatTypes.StanceType.DEFENSIVE
	var ok: bool = projected == CombatTypes.StanceType.DEFENSIVE and after_round and carried
	return _report("Stances persist after resolving and into the next round (Shield Bash stays Defensive)", ok,
		"projected %s, basher %s, striker %s" % [CombatTypes.stance_to_string(projected), CombatTypes.stance_to_string(basher.current_stance), CombatTypes.stance_to_string(striker.current_stance)])

# Focus (+50% for 2 rounds) boosts the next round's attacks and expires after its second round.
static func _test_focus_duration() -> bool:
	var focus: SkillDef = SkillCatalog.find("focus")
	var grid := _grid()
	var caster := Combatant.new("Caster", 100, 12, 1)
	var foe := Combatant.new("Foe", 300, 10, 1)
	grid.place_combatant(caster, Vector2i(3, 4))
	grid.place_combatant(foe, Vector2i(5, 4))
	var ctx: PlanContext = ArenaTestSuite._hand_ctx(grid, _side([caster]), _side([foe]))
	var resolver: CombatResolver = _commit(ctx, {caster: [PlanStep.self_skill(focus)]})
	resolver.resolve_round()
	var after_r1: float = caster.attack_boost_multiplier
	resolver.resolve_round()   # round 2: still boosted during it, ticks down at its end
	var after_r2: float = caster.attack_boost_multiplier
	var ok: bool = is_equal_approx(after_r1, focus.boost) and is_equal_approx(after_r2, 1.0)
	return _report("Focus boost lasts its rounds, then expires", ok, "after R1 x%.2f, after R2 x%.2f" % [after_r1, after_r2])

# A unit defeated mid-round loses the rest of its planned round.
static func _test_casualty_cancellation() -> bool:
	var grid := _grid()
	var killer := Combatant.new("Killer", 100, 14, 1)
	var victim := Combatant.new("Victim", 5, 10, 1)
	grid.place_combatant(killer, Vector2i(3, 4))
	grid.place_combatant(victim, Vector2i(4, 4))
	var ctx: PlanContext = ArenaTestSuite._hand_ctx(grid, _side([killer]), _side([victim]))
	var resolver := CombatResolver.new()
	resolver.grid = grid
	resolver.total_blocks = ctx.total_blocks
	resolver.register_combatant(killer)
	resolver.register_combatant(victim)
	var plan := CombatPlan.new(ctx)
	plan.add_step(killer, PlanStep.engage(SkillCatalog.find("strike"), victim))
	plan.add_step(victim, PlanStep.self_skill(SkillCatalog.find("rest")))
	plan.add_step(victim, PlanStep.self_skill(SkillCatalog.find("rest")))
	plan.add_step(victim, PlanStep.engage(SkillCatalog.find("strike"), killer))
	plan.commit(resolver)
	for b: int in range(1, resolver.total_blocks + 1):
		resolver.execute_block(b)   # inspect the schedule before the round rolls over
	var victim_later: int = 0
	var victim_cancelled: int = 0
	for sa: CombatResolver.ScheduledAction in resolver.scheduled_actions:
		if sa.combatant == victim and sa.start_block > 2:
			victim_later += 1
			if sa.is_cancelled:
				victim_cancelled += 1
	var ok: bool = not victim.is_alive() and killer.hp == 100 and victim_later > 0 and victim_cancelled == victim_later
	return _report("A unit defeated mid-round loses the rest of its round", ok,
		"victim hp %d, killer hp %d, later actions cancelled %d/%d" % [victim.hp, killer.hp, victim_cancelled, victim_later])

# Withdraw walks to the exit; the unit leaves the grid and the battle can end on it.
static func _test_withdrawal() -> bool:
	var grid := _grid()
	var runner := Combatant.new("Runner", 50, 10, 1)
	var holder := Combatant.new("Holder", 50, 10, 1)
	grid.place_combatant(runner, Vector2i(6, 4))
	grid.place_combatant(holder, Vector2i(2, 4))
	var ctx: PlanContext = ArenaTestSuite._hand_ctx(grid, _side([holder]), _side([runner]))
	var resolver: CombatResolver = _commit(ctx, {runner: [PlanStep.withdraw()]})
	var foes: Array[Combatant] = [runner]
	var friends: Array[Combatant] = [holder]
	resolver.resolve_round()
	var ok: bool = runner.is_withdrawn and not grid.has_combatant(runner) and grid.withdrawn_combatants.has(runner) and resolver.is_combat_resolved(friends, foes)
	return _report("Withdraw walks to the exit, leaves the grid and can end the battle", ok, "withdrawn %s" % runner.is_withdrawn)

# A full catalog battle through CombatSession (AI presets on both sides): the feed
# carries moves, attacks with damage and target HP, buffs, the wave, withdrawals, defeats.
static func _test_feed_from_catalog_battle() -> bool:
	var enc := CatalogEncounter.new("goblin_ambush")
	var session := CombatSession.new(enc)
	session.start()
	for round_num: int in range(enc.max_rounds):
		session.plan_apply_preset_all("balanced")
		if not session.confirm_plan():
			break
		ArenaTestSuite._resolve_until_replay(session)
		if session.combat_over or not session.can_advance_round():
			break
		session.advance_round()
	var seen: Dictionary = {}
	for e: ActionLogEntry in session.log_entries:
		if e.category == "MOVE" and e.outcome_tag == "MOVED":
			seen["move"] = true
		if e.category == "ATTACK" and e.damage > 0 and e.status_text.begins_with("Target HP"):
			seen["attack"] = true
		if e.category == "BUFF" or e.category == "DEFEND":
			seen["buff/guard"] = true
		if e.category == "EVENT" and e.outcome_tag == "ARRIVED" and e.ref_tag == "R1-B04":
			seen["wave"] = true
		if e.outcome_tag == "WITHDRAWN":
			seen["withdrawn"] = true
		if e.outcome_tag == "DEFEATED":
			seen["defeated"] = true
	var concluded: bool = session.combat_over or session.current_round == enc.max_rounds
	session.dispose()
	var missing: Array[String] = []
	for key: String in ["move", "attack", "buff/guard", "wave", "withdrawn", "defeated"]:
		if not seen.has(key):
			missing.append(key)
	return _report("Goblin Ambush feed: moves, attacks with HP, buffs/guards, wave at R1-B04, withdrawal, defeats", missing.is_empty() and concluded,
		"missing %s, concluded %s" % [str(missing), concluded])
