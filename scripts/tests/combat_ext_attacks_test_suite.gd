class_name CombatExtAttacksTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Extensions: gap-close attacks and bleed (S12-A, milestone M1; split from
# CombatExtensionsTestSuite by CLN-12). Helpers in CombatExtKit; expectations come from the rules.
# ==============================================================================

const TAG: String = "CombatExt"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING COMBAT EXTENSIONS: ATTACKS (S12-A)     ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_gap_close() and ok
	ok = _test_bleed() and ok
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return ok

# ------------------------------------------------------------------------------
# Gap-close
# ------------------------------------------------------------------------------

static func _test_gap_close() -> bool:
	var checks: Dictionary = {}
	var lunge: SkillDef = SkillCatalog.find("lunge")
	var strike: SkillDef = SkillCatalog.find("strike")
	var grid: CombatGrid = CombatExtKit.new_grid()
	var rogue: Combatant = CombatExtKit.unit("Rogue", 100, 3, 1, grid, Vector2i(1, 4), ["lunge"])
	var foe: Combatant = CombatExtKit.unit("Foe", 100, 10, 1, grid, Vector2i(5, 4))
	var allies: Array[Combatant] = [rogue]
	var enemies: Array[Combatant] = [foe]
	var ctx: PlanContext = CombatExtKit.new_ctx(grid, allies, enemies)
	var plan := CombatPlan.new(ctx)
	var walk_reach: int = plan.max_move_tiles(rogue)
	checks["setup: the foe is beyond a normal approach"] = grid.get_distance(Vector2i(1, 4), Vector2i(5, 4)) - 1 > walk_reach and lunge.dash_tiles >= 3
	checks["a normal strike can't get there"] = not plan.add_step(rogue, PlanStep.engage(strike, foe)).is_empty()
	var problem: String = plan.add_step(rogue, PlanStep.engage(lunge, foe))
	var st: CombatPlan.SimState = plan.compile(rogue)
	var entry: PlanEntry = st.entries[st.entries.size() - 1] if not st.entries.is_empty() else null
	var move_entries: int = 0
	for e: PlanEntry in st.entries:
		if e.skill.kind == SkillDef.Kind.MOVE:
			move_entries += 1
	checks["lunge is planned across the gap"] = problem.is_empty() and entry != null and entry.skill == lunge
	checks["AP is the skill level (+ stance switch), no per-tile cost"] = entry != null and entry.skill_ap == lunge.ap_cost(rogue) and entry.skill_ap == lunge.level and move_entries == 0
	checks["the plan ends adjacent to the target"] = grid.get_distance(st.pos, grid.get_position(foe)) == 1 and entry != null and entry.pos_after == st.pos and entry.pos_before == Vector2i(1, 4)
	var option: PlanOption = null
	for o: PlanOption in CombatPlan.new(ctx).target_options(rogue, foe):
		if o.step.skill == lunge:
			option = o
	checks["option is offered and valid beyond melee"] = option != null and option.valid and option.damage > 0
	var played: CombatExtKit.Played = CombatExtKit.play(ctx, {rogue: plan.steps_of(rogue)})
	checks["resolver: dashes then strikes"] = grid.get_distance(grid.get_position(rogue), grid.get_position(foe)) == 1 and foe.hp < foe.max_hp and CombatExtKit.has_message(played, "[Dash]")

	# Refusals.
	var near_grid: CombatGrid = CombatExtKit.new_grid()
	var r2: Combatant = CombatExtKit.unit("R2", 100, 3, 1, near_grid, Vector2i(1, 4), ["lunge"])
	var adj: Combatant = CombatExtKit.unit("Adj", 100, 10, 1, near_grid, Vector2i(2, 4))
	var ctx2: PlanContext = CombatExtKit.new_ctx(near_grid, [r2], [adj])
	checks["refused when the target is already in reach"] = CombatPlan.new(ctx2).add_step(r2, PlanStep.engage(lunge, adj)).contains("already in reach")
	var offered_adjacent: bool = false
	for o: PlanOption in CombatPlan.new(ctx2).target_options(r2, adj):
		offered_adjacent = offered_adjacent or (o.step.skill == lunge and o.valid)
	checks["not offered as valid vs an adjacent target"] = not offered_adjacent

	var far_grid: CombatGrid = CombatExtKit.new_grid()
	var r3: Combatant = CombatExtKit.unit("R3", 100, 3, 1, far_grid, Vector2i(0, 4), ["lunge"])
	var distant: Combatant = CombatExtKit.unit("Distant", 100, 10, 1, far_grid, Vector2i(8, 4))
	checks["refused beyond the dash reach"] = CombatPlan.new(CombatExtKit.new_ctx(far_grid, [r3], [distant])).add_step(r3, PlanStep.engage(lunge, distant)).contains("too far")

	var wall_grid: CombatGrid = CombatExtKit.new_grid()
	var r4: Combatant = CombatExtKit.unit("R4", 100, 3, 1, wall_grid, Vector2i(1, 4), ["lunge"])
	var behind: Combatant = CombatExtKit.unit("Behind", 100, 10, 1, wall_grid, Vector2i(5, 4))
	for y: int in range(CombatGrid.GRID_H):
		wall_grid.set_terrain(Vector2i(3, y), Terrain.Type.WALL)
	var wall_ctx: PlanContext = CombatExtKit.new_ctx(wall_grid, [r4], [behind])
	checks["refused when terrain blocks every route"] = CombatPlan.new(wall_ctx).add_step(r4, PlanStep.engage(lunge, behind)).contains("No route")
	var blocked_cmd := AttackCommand.new(r4, behind, 1.2, 2, 0, wall_grid)
	blocked_cmd.min_range = 1
	blocked_cmd.max_range = 1
	blocked_cmd.dash_tiles = 3
	var blocked_note: String = blocked_cmd.on_start()
	checks["resolver: a blocked dash stays put"] = blocked_note.begins_with("[Dash Blocked]") and wall_grid.get_position(r4) == Vector2i(1, 4)

	var root_grid: CombatGrid = CombatExtKit.new_grid()
	var r5: Combatant = CombatExtKit.unit("R5", 100, 3, 1, root_grid, Vector2i(1, 4), ["lunge"])
	var prey: Combatant = CombatExtKit.unit("Prey", 100, 10, 1, root_grid, Vector2i(5, 4))
	r5.statuses[StatusEffects.ROOT] = true
	checks["refused while rooted"] = not CombatPlan.new(CombatExtKit.new_ctx(root_grid, [r5], [prey])).add_step(r5, PlanStep.engage(lunge, prey)).is_empty()
	var rooted_cmd := AttackCommand.new(r5, prey, 1.2, 2, 0, root_grid)
	rooted_cmd.min_range = 1
	rooted_cmd.max_range = 1
	rooted_cmd.dash_tiles = 3
	checks["resolver: a rooted dash is blocked"] = rooted_cmd.on_start().begins_with("[Dash Blocked]") and root_grid.get_position(r5) == Vector2i(1, 4)

	var unit_grid: CombatGrid = CombatExtKit.new_grid()
	var r6: Combatant = CombatExtKit.unit("R6", 100, 3, 1, unit_grid, Vector2i(1, 4), ["lunge"])
	var quarry: Combatant = CombatExtKit.unit("Quarry", 100, 10, 1, unit_grid, Vector2i(4, 4))
	var guard_a: Combatant = CombatExtKit.unit("GuardA", 100, 10, 1, unit_grid, Vector2i(3, 4))
	var unit_cmd := AttackCommand.new(r6, quarry, 1.2, 2, 0, unit_grid)
	unit_cmd.min_range = 1
	unit_cmd.max_range = 1
	unit_cmd.dash_tiles = 3
	var unit_note: String = unit_cmd.on_start()
	var reached: bool = unit_grid.get_distance(unit_grid.get_position(r6), unit_grid.get_position(quarry)) == 1
	checks["resolver: units in the way are routed around (or it stops short)"] = (reached and unit_note.begins_with("[Dash]")) or (not reached and unit_note.begins_with("[Dash Blocked]"))
	checks["resolver: never ends on another unit's tile"] = unit_grid.get_position(r6) != unit_grid.get_position(guard_a)

	# One per round: the dash is the move, so it must come first and can't repeat.
	var two_grid: CombatGrid = CombatExtKit.new_grid()
	var r7: Combatant = CombatExtKit.unit("R7", 100, 3, 1, two_grid, Vector2i(1, 4), ["lunge"])
	var first_prey: Combatant = CombatExtKit.unit("First", 100, 10, 1, two_grid, Vector2i(5, 4))
	var second_prey: Combatant = CombatExtKit.unit("Second", 100, 10, 1, two_grid, Vector2i(1, 8))
	var two_ctx: PlanContext = CombatExtKit.new_ctx(two_grid, [r7], [first_prey, second_prey])
	var two_plan := CombatPlan.new(two_ctx)
	two_plan.add_step(r7, PlanStep.engage(lunge, first_prey))
	checks["a second lunge in the same round is refused (moving must come first)"] = two_plan.add_step(r7, PlanStep.engage(lunge, second_prey)).contains("Moving must come first")
	var acted_plan := CombatPlan.new(two_ctx)
	acted_plan.add_step(r7, PlanStep.self_skill(SkillCatalog.find("guard")))
	checks["can't lunge after acting"] = not acted_plan.add_step(r7, PlanStep.engage(lunge, first_prey)).is_empty()
	return TestHelpers.report(TAG, "Gap-close lunge (plan, resolve, refusals)", checks)

# ------------------------------------------------------------------------------
# Bleed
# ------------------------------------------------------------------------------

static func _test_bleed() -> bool:
	var checks: Dictionary = {}
	var cut: SkillDef = SkillCatalog.find("bleeding_cut")
	var grid: CombatGrid = CombatExtKit.new_grid()
	var attacker: Combatant = CombatExtKit.unit("Cutter", 100, 10, 1, grid, Vector2i(3, 4), ["bleeding_cut"])
	var victim: Combatant = CombatExtKit.unit("Victim", 100, 10, 1, grid, Vector2i(4, 4))
	var ctx: PlanContext = CombatExtKit.new_ctx(grid, [attacker], [victim])
	checks["bleeding_cut inflicts bleed"] = cut.status_effect == StatusEffects.BLEED and cut.status_rounds >= 1 and StatusEffects.debuffs().has(StatusEffects.BLEED)

	# Inflicting goes through the same resist maths as every status.
	var odds: int = StatusEffects.apply_chance(victim, StatusEffects.BLEED, cut.status_potency)
	victim.status_resist[StatusEffects.BLEED] = 100
	checks["resist key applies to bleed"] = StatusEffects.apply_chance(victim, StatusEffects.BLEED, cut.status_potency) < odds and StatusEffects.apply_chance(victim, StatusEffects.BLEED, 100) == 0
	checks["immune: try_inflict fails"] = not StatusEffects.try_inflict(victim, StatusEffects.BLEED, 100, RandomNumberGenerator.new())
	victim.status_resist.clear()

	var rng := RandomNumberGenerator.new()
	rng.seed = 7
	var strike_cmd := AttackCommand.new(attacker, victim, cut.power_mult, cut.blocks, 0, grid)
	strike_cmd.status_effect = StatusEffects.BLEED
	strike_cmd.status_potency = 300   # lands whatever the roll
	strike_cmd.status_rounds = 2
	strike_cmd.rng = rng
	strike_cmd.execute()
	checks["a hit queues bleed for next round with its duration"] = victim.pending_statuses.has(StatusEffects.BLEED) and DictUtil.int_of(victim.pending_status_rounds, StatusEffects.BLEED) == 2 and not victim.has_status(StatusEffects.BLEED)

	# The tick: end of the round, a flat share of max HP, ignoring guard and armor.
	StatusEffects.roll_over(victim, 4)
	checks["bleed active the next round"] = victim.has_status(StatusEffects.BLEED)
	victim.armor_override = 200
	victim.defense_mitigation_ratio = 0.5
	var hp_before: int = victim.hp
	var resolver := CombatResolver.new()
	resolver.grid = grid
	resolver.register_combatant(victim)
	var logs: Array[String] = []
	resolver.log_emitted.connect(func(msg: String) -> void: logs.append(msg))
	var dealt: Dictionary = resolver.apply_round_end_effects()
	var tick: int = StatusEffects.bleed_damage(victim)
	checks["tick = BLEED_PCT of max HP (min 1), ignoring armor and guard"] = tick == maxi(1, roundi(float(victim.max_hp) * StatusEffects.BLEED_PCT)) and hp_before - victim.hp == tick and DictUtil.int_of(dealt, victim) == tick
	checks["the tick is logged"] = logs.size() == 1 and logs[0].begins_with("[Bleed]")
	victim.defense_mitigation_ratio = 0.0
	StatusEffects.roll_over(victim, 4)
	checks["a 2-round bleed is still on in round two"] = victim.has_status(StatusEffects.BLEED)
	StatusEffects.roll_over(victim, 4)
	checks["and over after"] = not victim.has_status(StatusEffects.BLEED)

	# resolve_round applies it; bleeding can kill.
	var other_grid: CombatGrid = CombatExtKit.new_grid()
	var lone: Combatant = CombatExtKit.unit("Lone", 100, 10, 1, other_grid, Vector2i(1, 1))
	lone.statuses[StatusEffects.BLEED] = true
	lone.hp = 1
	var round_resolver := CombatResolver.new()
	round_resolver.grid = other_grid
	round_resolver.register_combatant(lone)
	var round_log: Array[String] = []
	round_resolver.log_emitted.connect(func(msg: String) -> void: round_log.append(msg))
	round_resolver.resolve_round()
	var bleed_logged: bool = false
	for line: String in round_log:
		bleed_logged = bleed_logged or (line.begins_with("[Bleed]") and line.contains("DEFEATED"))
	checks["resolve_round applies the tick and bleeding can defeat"] = not lone.is_alive() and bleed_logged
	var session_formatted: String = CombatLogBuilder.format_resolver_message("[Bleed] X bleeds for 3 damage")
	checks["feed formats the bleed line"] = session_formatted.contains("bleeds")
	return TestHelpers.report(TAG, "Bleed (inflict, resist, tick, duration)", checks)
