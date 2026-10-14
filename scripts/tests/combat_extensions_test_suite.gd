class_name CombatExtensionsTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat Extensions Test Suite (S12-A + S4-A, milestone M1)
# Ally-target commands (heal / buff), gap-close attacks, the haste / bleed / taunt
# statuses, and the AI fixes (taunt targeting, kiting, no idle turns, support heals,
# reckless profile). Units are hand-built (fixed strike power, no armor) and learn
# the example skills (mend, rally, taunt_shout, lunge, bleeding_cut) explicitly:
# those are `learned_only`, so no existing unit or number changes. Expectations are
# derived from the rules (SupportRules, StatusEffects, SkillDef), not magic numbers.
# ==============================================================================

const SANDBOX: String = "user://combat_ext_test"

static func _report(label: String, checks: Dictionary) -> bool:
	var failed: Array[String] = []
	for key: Variant in checks:
		if not DictUtil.to_bool(checks[key]):
			failed.append(str(key))
	var ok: bool = failed.is_empty()
	print("    [CombatExt] %s: %s%s" % [label, "PASS" if ok else "FAIL", "" if ok else " (%s)" % ", ".join(failed)])
	return ok

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING COMBAT EXTENSIONS TEST (S12-A / S4-A)  ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	var ok: bool = true
	ok = _test_heal_and_clamp() and ok
	ok = _test_ally_validation() and ok
	ok = _test_buffs_and_haste() and ok
	ok = _test_gap_close() and ok
	ok = _test_bleed() and ok
	ok = _test_taunt_ai() and ok
	ok = _test_kiting_and_idle() and ok
	ok = _test_support_ai() and ok
	ok = _test_reckless() and ok
	ok = _test_reset_round_clears_queued_statuses() and ok
	ok = _test_catalog_and_schema() and ok
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return ok

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

static func _grid(exits: Array[Vector2i] = []) -> CombatGrid:
	return CombatGrid.new(CombatGrid.ArenaType.CLOSED, exits)

static func _unit(unit_name: String, hp: int, speed: int, attack_range: int, grid: CombatGrid, pos: Vector2i, learn: Array[String] = []) -> Combatant:
	var c := Combatant.new(unit_name, hp, speed, attack_range)
	c.power_override = 15   # fixed strike power, no armor: expected numbers stay simple
	c.armor_override = 0
	for skill_id: String in learn:
		c.learn_skill(skill_id)
	grid.place_combatant(c, pos)
	return c

static func _ctx(grid: CombatGrid, allies: Array[Combatant], enemies: Array[Combatant], profiles: Dictionary = {}) -> PlanContext:
	var all_profiles: Dictionary = profiles.duplicate()
	for c: Combatant in allies + enemies:
		if not all_profiles.has(c):
			all_profiles[c] = AiProfile.create(AiProfile.Role.SNIPER if c.attack_range > 1 else AiProfile.Role.AGGRESSOR, 0.0, 3)
	return PlanContext.create(grid, allies, enemies, all_profiles)

static func _profile(role: AiProfile.Role, flee: float = 0.0) -> AiProfile:
	return AiProfile.create(role, flee, 3)

# Commits the given steps and plays one full round on a bare resolver.
class Played:
	extends RefCounted
	var resolver: CombatResolver = null
	var messages: Array[String] = []
	var actions: Array[CombatResolver.ScheduledAction] = []

static func _play(ctx: PlanContext, plans: Dictionary) -> Played:
	var played := Played.new()
	played.resolver = CombatResolver.new()
	played.resolver.grid = ctx.grid
	played.resolver.log_emitted.connect(func(msg: String) -> void: played.messages.append(msg))
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
	cp.commit(played.resolver)
	played.actions = played.resolver.scheduled_actions.duplicate()
	played.resolver.resolve_round()
	return played

static func _has_message(played: Played, prefix: String) -> bool:
	for m: String in played.messages:
		if m.begins_with(prefix):
			return true
	return false

static func _any_skill_kind(steps: Array[PlanStep], kind: SkillDef.Kind) -> bool:
	for s: PlanStep in steps:
		if s.skill != null and s.skill.kind == kind:
			return true
	return false

static func _any_mode(steps: Array[PlanStep], mode: PlanStep.Mode) -> bool:
	for s: PlanStep in steps:
		if s.mode == mode:
			return true
	return false

static func _base_move(c: Combatant) -> int:
	return clampi(floori(float(c.speed + 2) / 5.0), 1, 4)

# ------------------------------------------------------------------------------
# Heal / ally-target commands
# ------------------------------------------------------------------------------

static func _test_heal_and_clamp() -> bool:
	var checks: Dictionary = {}
	var mend: SkillDef = SkillCatalog.find("mend")
	var grid: CombatGrid = _grid()
	var healer: Combatant = _unit("Healer", 100, 10, 1, grid, Vector2i(2, 4), ["mend"])
	var friend: Combatant = _unit("Friend", 100, 10, 1, grid, Vector2i(4, 4))
	var foe: Combatant = _unit("Foe", 100, 10, 1, grid, Vector2i(8, 8))
	var allies: Array[Combatant] = [healer, friend]
	var enemies: Array[Combatant] = [foe]
	var ctx: PlanContext = _ctx(grid, allies, enemies)
	checks["mend is a SUPPORT ally heal at AP = level"] = mend != null and mend.kind == SkillDef.Kind.SUPPORT and mend.heals() and mend.targets_ally() and mend.ap_cost() == mend.level

	# The heal comes from SupportRules (stat x multiplier), not DamageCalculator.
	var know: int = healer.stats.get_effective_stat("KNOW", healer.get_weapon_context())
	var expected_amount: int = mend.heal_base + roundi(float(know) * mend.heal_mult)
	var amount: int = SupportRules.heal_amount(healer, mend)
	checks["heal = base + round(KNOW x mult)"] = amount == expected_amount and amount > 0
	var sage: Combatant = Combatant.new("Sage", 100, 10, 1)
	sage.stats.add_modifier(StatModifier.new("KNOW", 6, 0.0, "test", "always", -1))
	checks["a higher KNOW heals more"] = SupportRules.heal_amount(sage, mend) > amount and SupportRules.heal_amount(sage, mend) == mend.heal_base + roundi(float(know + 6) * mend.heal_mult)

	friend.hp = 40
	var missing: int = friend.max_hp - friend.hp
	var plan := CombatPlan.new(ctx)
	var problem: String = plan.add_step(healer, PlanStep.ally(mend, friend))
	var st: CombatPlan.SimState = plan.compile(healer)
	var entry: PlanEntry = st.entries[st.entries.size() - 1] if not st.entries.is_empty() else null
	checks["plan accepts a heal in reach"] = problem.is_empty() and entry != null and entry.skill == mend and entry.target == friend
	checks["forecast: expected heal is clamped to missing HP"] = entry != null and entry.est_heal == mini(amount, missing)
	checks["support keeps the stance (no switch cost)"] = entry != null and entry.switch_ap == 0 and entry.stance_after == entry.stance_before and entry.skill_ap == mend.level
	var played: Played = _play(ctx, {healer: plan.steps_of(healer)})
	checks["heal applied (hp + min(heal, missing))"] = friend.hp == 40 + mini(amount, missing)
	checks["support logs a line"] = _has_message(played, "[Support]")
	var heal_action: CombatResolver.ScheduledAction = null
	for a: CombatResolver.ScheduledAction in played.actions:
		if a.command is SupportCommand:
			heal_action = a
	var log_entry: ActionLogEntry = CombatLogBuilder.from_action(heal_action, 1, 1, allies, grid) if heal_action != null else null
	checks["replay entry renders the heal"] = log_entry != null and log_entry.category == "BUFF" and log_entry.outcome_tag == "+%d HP" % (friend.hp - 40) and log_entry.target_name == "Friend"

	# Clamp: a heal never exceeds max HP.
	friend.hp = friend.max_hp - 2
	var clamp_plan := CombatPlan.new(ctx)
	clamp_plan.add_step(healer, PlanStep.ally(mend, friend))
	var clamp_st: CombatPlan.SimState = clamp_plan.compile(healer)
	checks["forecast clamps at 2"] = not clamp_st.entries.is_empty() and clamp_st.entries[clamp_st.entries.size() - 1].est_heal == 2
	_play(ctx, {healer: clamp_plan.steps_of(healer)})
	checks["heal clamps to max HP"] = friend.hp == friend.max_hp

	# Full HP: valid but flagged; no revive of the dead.
	var full_plan := CombatPlan.new(ctx)
	var options: Array[PlanOption] = full_plan.target_options(healer, friend)
	var mend_option: PlanOption = null
	for o: PlanOption in options:
		if o.step.skill == mend:
			mend_option = o
	checks["full-HP target: valid option with a warning, no heal"] = mend_option != null and mend_option.valid and mend_option.heal == 0 and mend_option.warning.contains("full HP")
	friend.hp = 40
	var fresh: Array[PlanOption] = CombatPlan.new(ctx).target_options(healer, friend)
	var shown: PlanOption = null
	for o: PlanOption in fresh:
		if o.step.skill == mend:
			shown = o
	checks["preview text shows the heal"] = shown != null and shown.heal == mini(amount, missing) and shown.cost_text().contains("+%d HP" % shown.heal)
	friend.hp = 0
	checks["Combatant.heal never revives"] = (func() -> bool:
		friend.heal(50)
		return friend.hp == 0).call()
	return _report("Heal amount, forecast, clamp, log", checks)

static func _test_ally_validation() -> bool:
	var checks: Dictionary = {}
	var mend: SkillDef = SkillCatalog.find("mend")
	var rally: SkillDef = SkillCatalog.find("rally")
	var shout: SkillDef = SkillCatalog.find("taunt_shout")
	var grid: CombatGrid = _grid()
	var healer: Combatant = _unit("Healer", 100, 10, 1, grid, Vector2i(2, 4), ["mend", "rally", "taunt_shout"])
	var friend: Combatant = _unit("Friend", 100, 10, 1, grid, Vector2i(3, 6))
	var far_friend: Combatant = _unit("FarFriend", 100, 10, 1, grid, Vector2i(8, 4))
	var foe: Combatant = _unit("Foe", 100, 10, 1, grid, Vector2i(6, 0))
	var novice: Combatant = _unit("Novice", 100, 10, 1, grid, Vector2i(0, 8))
	friend.hp = 50
	far_friend.hp = 50
	var allies: Array[Combatant] = [healer, friend, far_friend, novice]
	var enemies: Array[Combatant] = [foe]
	var ctx: PlanContext = _ctx(grid, allies, enemies)
	var plan := CombatPlan.new(ctx)
	checks["enemy is not a legal heal target"] = not plan.add_step(healer, PlanStep.ally(mend, foe)).is_empty()
	checks["a SELF skill can't be aimed at an ally"] = not plan.add_step(healer, PlanStep.ally(shout, friend)).is_empty()
	checks["an ALLY-only skill can't be cast on yourself"] = not plan.add_step(healer, PlanStep.self_skill(rally)).is_empty() and not plan.add_step(healer, PlanStep.ally(rally, healer)).is_empty()
	checks["ALLY_OR_SELF can heal yourself"] = plan.add_step(healer, PlanStep.self_skill(mend)).is_empty()
	plan.clear(healer)
	checks["a SELF skill is fine on yourself"] = plan.add_step(healer, PlanStep.self_skill(shout)).is_empty()
	plan.clear(healer)
	checks["an unlearned unit can't use it"] = not CombatPlan.new(ctx).add_step(novice, PlanStep.ally(mend, friend)).is_empty() and not novice.knows_skill("mend")

	# Out of reach: the healer walks into the skill's reach first (unless rooted / too far).
	var band: Vector2i = mend.support_reach(healer)
	var walk := CombatPlan.new(ctx)
	var walk_problem: String = walk.add_step(healer, PlanStep.ally(mend, far_friend))
	var walk_st: CombatPlan.SimState = walk.compile(healer)
	var moved: bool = false
	for e: PlanEntry in walk_st.entries:
		moved = moved or e.skill.kind == SkillDef.Kind.MOVE
	checks["out of reach: walks into reach, then heals"] = walk_problem.is_empty() and moved and grid.get_distance(walk_st.pos, grid.get_position(far_friend)) <= band.y
	healer.statuses[StatusEffects.ROOT] = true
	checks["rooted healer can't walk into reach"] = not CombatPlan.new(ctx).add_step(healer, PlanStep.ally(mend, far_friend)).is_empty()
	checks["rooted healer still heals in reach"] = CombatPlan.new(ctx).add_step(healer, PlanStep.ally(mend, friend)).is_empty()
	healer.statuses.clear()

	var option_list: Array[PlanOption] = CombatPlan.new(ctx).target_options(healer, friend)
	var offered: Array[String] = []
	for o: PlanOption in option_list:
		if o.valid:
			offered.append(o.step.skill.id)
	checks["clicking an ally offers mend and rally, not the self-only shout"] = offered.has("mend") and offered.has("rally") and not offered.has("taunt_shout")
	var foe_options: Array[PlanOption] = CombatPlan.new(ctx).target_options(healer, foe)
	var foe_support: bool = false
	for o: PlanOption in foe_options:
		foe_support = foe_support or o.step.skill.kind == SkillDef.Kind.SUPPORT
	checks["clicking an enemy offers no support skills"] = not foe_support
	var all_options: Array[PlanOption] = CombatPlan.new(ctx).options(healer)
	var ally_option: bool = false
	for o: PlanOption in all_options:
		ally_option = ally_option or (o.step.mode == PlanStep.Mode.ALLY and o.step.target == friend)
	checks["options() includes ally-target steps"] = ally_option
	friend.hp = 0
	var dead_problem: String = CombatPlan.new(ctx).add_step(healer, PlanStep.ally(mend, friend))
	checks["a defeated ally can't be targeted"] = not dead_problem.is_empty()

	# Resolution: if the ally is gone or out of reach when the heal lands, nothing happens.
	friend.hp = 50
	var cmd := SupportCommand.new(healer, far_friend, mend, grid, mend.blocks)
	cmd.execute()
	checks["the command refuses an ally out of reach"] = cmd.failed and far_friend.hp == 50 and cmd.execution_log.begins_with("[Support Failed]")
	return _report("Ally-target validation and options", checks)

# ------------------------------------------------------------------------------
# Buffs: haste and taunt durations
# ------------------------------------------------------------------------------

static func _test_buffs_and_haste() -> bool:
	var checks: Dictionary = {}
	var rally: SkillDef = SkillCatalog.find("rally")
	var shout: SkillDef = SkillCatalog.find("taunt_shout")
	var grid: CombatGrid = _grid()
	var caller: Combatant = _unit("Caller", 100, 10, 1, grid, Vector2i(2, 4), ["rally", "taunt_shout"])
	var runner: Combatant = _unit("Runner", 100, 3, 1, grid, Vector2i(3, 4))
	var foe: Combatant = _unit("Foe", 100, 10, 1, grid, Vector2i(8, 8))
	var allies: Array[Combatant] = [caller, runner]
	var enemies: Array[Combatant] = [foe]
	var ctx: PlanContext = _ctx(grid, allies, enemies)
	var plan := CombatPlan.new(ctx)
	checks["rally plans on an ally"] = plan.add_step(caller, PlanStep.ally(rally, runner)).is_empty()
	plan.add_step(caller, PlanStep.self_skill(shout))
	var before_tiles: int = runner.move_per_block()
	_play(ctx, {caller: plan.steps_of(caller)})
	checks["buffs queue for next round (not active now)"] = runner.pending_statuses.has(StatusEffects.HASTE) and not runner.has_status(StatusEffects.HASTE) and caller.pending_statuses.has(StatusEffects.TAUNT)
	checks["status rounds recorded from the skill"] = DictUtil.int_of(caller.pending_status_rounds, StatusEffects.TAUNT) == shout.status_rounds and DictUtil.int_of(runner.pending_status_rounds, StatusEffects.HASTE) == rally.status_rounds

	var slow_cap: int = CombatPlan.new(ctx).max_move_tiles(runner)
	var goto_far: Vector2i = Vector2i(3 + slow_cap + 1, 4)
	checks["before haste: a slow unit can't move that far"] = not CombatPlan.new(ctx).add_step(runner, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), goto_far)).is_empty()
	StatusEffects.roll_over(runner, 4)
	StatusEffects.roll_over(caller, 4)
	checks["haste active next round: +HASTE_TILES per move block"] = runner.has_status(StatusEffects.HASTE) and runner.move_per_block() == _base_move(runner) + StatusEffects.HASTE_TILES and before_tiles == _base_move(runner)
	checks["haste widens the planner's move"] = CombatPlan.new(ctx).add_step(runner, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), goto_far)).is_empty()
	checks["taunt active next round"] = caller.has_status(StatusEffects.TAUNT)
	StatusEffects.roll_over(runner, 4)
	StatusEffects.roll_over(caller, 4)
	checks["haste lasts its one round"] = not runner.has_status(StatusEffects.HASTE) and runner.move_per_block() == _base_move(runner)
	checks["a 2-round taunt is still up the second round"] = caller.has_status(StatusEffects.TAUNT)
	StatusEffects.roll_over(caller, 4)
	checks["and gone after"] = not caller.has_status(StatusEffects.TAUNT) and caller.status_rounds_left.is_empty()
	# Existing one-round statuses keep their behaviour.
	StatusEffects.try_inflict(runner, StatusEffects.STAGGER, 200, null)
	StatusEffects.roll_over(runner, 4)
	var staggered: bool = runner.has_status(StatusEffects.STAGGER)
	StatusEffects.roll_over(runner, 4)
	checks["stagger still lasts exactly one round"] = staggered and not runner.has_status(StatusEffects.STAGGER)
	return _report("Haste and taunt statuses", checks)

# ------------------------------------------------------------------------------
# Gap-close
# ------------------------------------------------------------------------------

static func _test_gap_close() -> bool:
	var checks: Dictionary = {}
	var lunge: SkillDef = SkillCatalog.find("lunge")
	var strike: SkillDef = SkillCatalog.find("strike")
	var grid: CombatGrid = _grid()
	var rogue: Combatant = _unit("Rogue", 100, 3, 1, grid, Vector2i(1, 4), ["lunge"])
	var foe: Combatant = _unit("Foe", 100, 10, 1, grid, Vector2i(5, 4))
	var allies: Array[Combatant] = [rogue]
	var enemies: Array[Combatant] = [foe]
	var ctx: PlanContext = _ctx(grid, allies, enemies)
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
	var played: Played = _play(ctx, {rogue: plan.steps_of(rogue)})
	checks["resolver: dashes then strikes"] = grid.get_distance(grid.get_position(rogue), grid.get_position(foe)) == 1 and foe.hp < foe.max_hp and _has_message(played, "[Dash]")

	# Refusals.
	var near_grid: CombatGrid = _grid()
	var r2: Combatant = _unit("R2", 100, 3, 1, near_grid, Vector2i(1, 4), ["lunge"])
	var adj: Combatant = _unit("Adj", 100, 10, 1, near_grid, Vector2i(2, 4))
	var ctx2: PlanContext = _ctx(near_grid, [r2], [adj])
	checks["refused when the target is already in reach"] = CombatPlan.new(ctx2).add_step(r2, PlanStep.engage(lunge, adj)).contains("already in reach")
	var offered_adjacent: bool = false
	for o: PlanOption in CombatPlan.new(ctx2).target_options(r2, adj):
		offered_adjacent = offered_adjacent or (o.step.skill == lunge and o.valid)
	checks["not offered as valid vs an adjacent target"] = not offered_adjacent

	var far_grid: CombatGrid = _grid()
	var r3: Combatant = _unit("R3", 100, 3, 1, far_grid, Vector2i(0, 4), ["lunge"])
	var distant: Combatant = _unit("Distant", 100, 10, 1, far_grid, Vector2i(8, 4))
	checks["refused beyond the dash reach"] = CombatPlan.new(_ctx(far_grid, [r3], [distant])).add_step(r3, PlanStep.engage(lunge, distant)).contains("too far")

	var wall_grid: CombatGrid = _grid()
	var r4: Combatant = _unit("R4", 100, 3, 1, wall_grid, Vector2i(1, 4), ["lunge"])
	var behind: Combatant = _unit("Behind", 100, 10, 1, wall_grid, Vector2i(5, 4))
	for y: int in range(CombatGrid.GRID_SIZE):
		wall_grid.set_terrain(Vector2i(3, y), Terrain.Type.WALL)
	var wall_ctx: PlanContext = _ctx(wall_grid, [r4], [behind])
	checks["refused when terrain blocks every route"] = CombatPlan.new(wall_ctx).add_step(r4, PlanStep.engage(lunge, behind)).contains("No route")
	var blocked_cmd := AttackCommand.new(r4, behind, 1.2, 2, 0, wall_grid)
	blocked_cmd.min_range = 1
	blocked_cmd.max_range = 1
	blocked_cmd.dash_tiles = 3
	var blocked_note: String = blocked_cmd.on_start()
	checks["resolver: a blocked dash stays put"] = blocked_note.begins_with("[Dash Blocked]") and wall_grid.get_position(r4) == Vector2i(1, 4)

	var root_grid: CombatGrid = _grid()
	var r5: Combatant = _unit("R5", 100, 3, 1, root_grid, Vector2i(1, 4), ["lunge"])
	var prey: Combatant = _unit("Prey", 100, 10, 1, root_grid, Vector2i(5, 4))
	r5.statuses[StatusEffects.ROOT] = true
	checks["refused while rooted"] = not CombatPlan.new(_ctx(root_grid, [r5], [prey])).add_step(r5, PlanStep.engage(lunge, prey)).is_empty()
	var rooted_cmd := AttackCommand.new(r5, prey, 1.2, 2, 0, root_grid)
	rooted_cmd.min_range = 1
	rooted_cmd.max_range = 1
	rooted_cmd.dash_tiles = 3
	checks["resolver: a rooted dash is blocked"] = rooted_cmd.on_start().begins_with("[Dash Blocked]") and root_grid.get_position(r5) == Vector2i(1, 4)

	var unit_grid: CombatGrid = _grid()
	var r6: Combatant = _unit("R6", 100, 3, 1, unit_grid, Vector2i(1, 4), ["lunge"])
	var quarry: Combatant = _unit("Quarry", 100, 10, 1, unit_grid, Vector2i(4, 4))
	var guard_a: Combatant = _unit("GuardA", 100, 10, 1, unit_grid, Vector2i(3, 4))
	var unit_cmd := AttackCommand.new(r6, quarry, 1.2, 2, 0, unit_grid)
	unit_cmd.min_range = 1
	unit_cmd.max_range = 1
	unit_cmd.dash_tiles = 3
	var unit_note: String = unit_cmd.on_start()
	var reached: bool = unit_grid.get_distance(unit_grid.get_position(r6), unit_grid.get_position(quarry)) == 1
	checks["resolver: units in the way are routed around (or it stops short)"] = (reached and unit_note.begins_with("[Dash]")) or (not reached and unit_note.begins_with("[Dash Blocked]"))
	checks["resolver: never ends on another unit's tile"] = unit_grid.get_position(r6) != unit_grid.get_position(guard_a)

	# One per round: the dash is the move, so it must come first and can't repeat.
	var two_grid: CombatGrid = _grid()
	var r7: Combatant = _unit("R7", 100, 3, 1, two_grid, Vector2i(1, 4), ["lunge"])
	var first_prey: Combatant = _unit("First", 100, 10, 1, two_grid, Vector2i(5, 4))
	var second_prey: Combatant = _unit("Second", 100, 10, 1, two_grid, Vector2i(1, 8))
	var two_ctx: PlanContext = _ctx(two_grid, [r7], [first_prey, second_prey])
	var two_plan := CombatPlan.new(two_ctx)
	two_plan.add_step(r7, PlanStep.engage(lunge, first_prey))
	checks["a second lunge in the same round is refused (moving must come first)"] = two_plan.add_step(r7, PlanStep.engage(lunge, second_prey)).contains("Moving must come first")
	var acted_plan := CombatPlan.new(two_ctx)
	acted_plan.add_step(r7, PlanStep.self_skill(SkillCatalog.find("guard")))
	checks["can't lunge after acting"] = not acted_plan.add_step(r7, PlanStep.engage(lunge, first_prey)).is_empty()
	return _report("Gap-close lunge (plan, resolve, refusals)", checks)

# ------------------------------------------------------------------------------
# Bleed
# ------------------------------------------------------------------------------

static func _test_bleed() -> bool:
	var checks: Dictionary = {}
	var cut: SkillDef = SkillCatalog.find("bleeding_cut")
	var grid: CombatGrid = _grid()
	var attacker: Combatant = _unit("Cutter", 100, 10, 1, grid, Vector2i(3, 4), ["bleeding_cut"])
	var victim: Combatant = _unit("Victim", 100, 10, 1, grid, Vector2i(4, 4))
	var ctx: PlanContext = _ctx(grid, [attacker], [victim])
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
	var other_grid: CombatGrid = _grid()
	var lone: Combatant = _unit("Lone", 100, 10, 1, other_grid, Vector2i(1, 1))
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
	return _report("Bleed (inflict, resist, tick, duration)", checks)

# ------------------------------------------------------------------------------
# AI: taunt
# ------------------------------------------------------------------------------

static func _first_engage_target(steps: Array[PlanStep]) -> Combatant:
	for s: PlanStep in steps:
		if s.mode == PlanStep.Mode.ENGAGE:
			return s.target
	return null

static func _test_taunt_ai() -> bool:
	var checks: Dictionary = {}
	var grid: CombatGrid = _grid()
	var near: Combatant = _unit("Near", 100, 10, 1, grid, Vector2i(4, 4))
	var tank: Combatant = _unit("Tank", 100, 10, 1, grid, Vector2i(6, 8))
	var brute: Combatant = _unit("Brute", 100, 10, 1, grid, Vector2i(6, 4))
	var ctx: PlanContext = _ctx(grid, [near, tank], [brute])
	var profile: AiProfile = _profile(AiProfile.Role.AGGRESSOR)
	checks["baseline: the AI picks the nearest foe"] = _first_engage_target(TacticalAI.build_steps(brute, ctx, profile)) == near
	tank.statuses[StatusEffects.TAUNT] = true
	checks["taunted: the AI is forced onto the reachable taunter"] = _first_engage_target(TacticalAI.build_steps(brute, ctx, profile)) == tank
	var weakest: AiProfile = _profile(AiProfile.Role.SKIRMISHER)
	weakest.target_pref = AiProfile.TargetPref.WEAKEST
	near.hp = 10
	checks["even a weakest-first profile is forced"] = _first_engage_target(TacticalAI.build_steps(brute, ctx, weakest)) == tank
	checks["a reckless profile is forced too"] = (func() -> bool:
		var wild: AiProfile = _profile(AiProfile.Role.AGGRESSOR)
		wild.reckless = true
		return _first_engage_target(TacticalAI.build_steps(brute, ctx, wild)) == tank).call()
	near.hp = near.max_hp

	# Not reachable: walled in, the taunt does not apply.
	var walled: CombatGrid = _grid()
	var n2: Combatant = _unit("Near2", 100, 10, 1, walled, Vector2i(4, 4))
	var t2: Combatant = _unit("Tank2", 100, 10, 1, walled, Vector2i(8, 8))
	var b2: Combatant = _unit("Brute2", 100, 10, 1, walled, Vector2i(6, 4))
	walled.set_terrain(Vector2i(7, 8), Terrain.Type.WALL)
	walled.set_terrain(Vector2i(8, 7), Terrain.Type.WALL)
	t2.statuses[StatusEffects.TAUNT] = true
	var ctx2: PlanContext = _ctx(walled, [n2, t2], [b2])
	checks["an unreachable taunter does not force the AI"] = _first_engage_target(TacticalAI.build_steps(b2, ctx2, _profile(AiProfile.Role.AGGRESSOR))) == n2

	# The player's planner is never forced.
	var pgrid: CombatGrid = _grid()
	var hero: Combatant = _unit("Hero", 100, 10, 1, pgrid, Vector2i(2, 4))
	var taunter: Combatant = _unit("Taunter", 100, 10, 1, pgrid, Vector2i(6, 6))
	var side: Combatant = _unit("Side", 100, 10, 1, pgrid, Vector2i(3, 4))
	taunter.statuses[StatusEffects.TAUNT] = true
	var pctx: PlanContext = _ctx(pgrid, [hero], [taunter, side])
	checks["the player's plan may ignore a taunter"] = CombatPlan.new(pctx).add_step(hero, PlanStep.engage(SkillCatalog.find("strike"), side)).is_empty()
	return _report("Taunt steers the enemy AI only", checks)

# ------------------------------------------------------------------------------
# AI: kiting, no idle turns, buff-before-move
# ------------------------------------------------------------------------------

static func _test_kiting_and_idle() -> bool:
	var checks: Dictionary = {}
	var strike: SkillDef = SkillCatalog.find("strike")
	var focus: SkillDef = SkillCatalog.find("focus")

	# Ranged unit with a foe in its face: step back to maximum range, then shoot.
	var grid: CombatGrid = _grid()
	var archer: Combatant = _unit("Archer", 100, 10, 3, grid, Vector2i(4, 4))
	var brute: Combatant = _unit("Brute", 100, 10, 1, grid, Vector2i(5, 4))
	var ctx: PlanContext = _ctx(grid, [archer], [brute])
	var band: Vector2i = strike.reach_for(archer)
	var steps: Array[PlanStep] = TacticalAI.build_steps(archer, ctx, _profile(AiProfile.Role.SNIPER))
	var plan := CombatPlan.new(ctx)
	plan.set_steps(archer, steps)
	var st: CombatPlan.SimState = plan.compile(archer)
	var struck: bool = false
	for e: PlanEntry in st.entries:
		struck = struck or (e.skill.kind == SkillDef.Kind.ATTACK and e.target == brute)
	checks["kites: a plain move first, then a strike"] = not steps.is_empty() and (steps[0].mode == PlanStep.Mode.GOTO or steps[0].mode == PlanStep.Mode.MOVE) and struck
	checks["the plan ends at maximum range"] = grid.get_distance(st.pos, grid.get_position(brute)) == band.y
	var played: Played = _play(ctx, {archer: steps})
	checks["resolved: shot lands from maximum range"] = brute.hp < brute.max_hp and grid.get_distance(grid.get_position(archer), grid.get_position(brute)) == band.y and played.messages.size() > 0

	# Cornered: can't kite, but still attacks (off-hand jab) instead of idling.
	var corner: CombatGrid = _grid()
	var trapped: Combatant = _unit("Trapped", 100, 10, 3, corner, Vector2i(0, 0))
	var pest: Combatant = _unit("Pest", 100, 10, 1, corner, Vector2i(1, 0))
	var cctx: PlanContext = _ctx(corner, [trapped], [pest])
	var corner_steps: Array[PlanStep] = TacticalAI.build_steps(trapped, cctx, _profile(AiProfile.Role.SNIPER))
	checks["cornered ranged unit still acts"] = _any_mode(corner_steps, PlanStep.Mode.ENGAGE)

	# A reachable target that is too far this round: advance instead of idling.
	var field: CombatGrid = _grid()
	var walker: Combatant = _unit("Walker", 100, 3, 1, field, Vector2i(0, 4))
	var distant: Combatant = _unit("Distant", 100, 10, 1, field, Vector2i(8, 4))
	var fctx: PlanContext = _ctx(field, [walker], [distant])
	var fplan := CombatPlan.new(fctx)
	var walk_steps: Array[PlanStep] = TacticalAI.build_steps(walker, fctx, _profile(AiProfile.Role.AGGRESSOR))
	fplan.set_steps(walker, walk_steps)
	var wst: CombatPlan.SimState = fplan.compile(walker)
	var start_dist: int = field.get_distance(Vector2i(0, 4), Vector2i(8, 4))
	checks["no idle turn: out-of-range target is approached"] = not walk_steps.is_empty() and field.get_distance(wst.pos, Vector2i(8, 4)) == start_dist - fplan.max_move_tiles(walker)
	_play(fctx, {walker: walk_steps})
	checks["resolved: the unit really closed in"] = field.get_distance(field.get_position(walker), Vector2i(8, 4)) < start_dist

	# Buff first only when it does not waste the turn.
	var cgrid: CombatGrid = _grid()
	var caster: Combatant = _unit("Caster", 100, 10, 3, cgrid, Vector2i(2, 4), ["focus"])
	var target: Combatant = _unit("Target", 100, 10, 1, cgrid, Vector2i(5, 4))
	var cctx2: PlanContext = _ctx(cgrid, [caster], [target])
	var in_band: Array[PlanStep] = TacticalAI.build_steps(caster, cctx2, _profile(AiProfile.Role.SNIPER))
	checks["in reach: casts the buff, then attacks"] = in_band.size() >= 2 and in_band[0].skill != null and in_band[0].skill.kind == SkillDef.Kind.FOCUS and _any_mode(in_band, PlanStep.Mode.ENGAGE)
	var cost_both: int = focus.ap_cost(caster) + focus.switch_cost(CombatTypes.StanceType.NEUTRAL) + strike.ap_cost(caster) + strike.switch_cost(CombatTypes.StanceType.TACTICAL)
	caster.ap = cost_both - 1
	var thrifty: Array[PlanStep] = TacticalAI.build_steps(caster, cctx2, _profile(AiProfile.Role.SNIPER))
	checks["not enough AP for both: skips the buff, still attacks"] = not _any_skill_kind(thrifty, SkillDef.Kind.FOCUS) and _any_mode(thrifty, PlanStep.Mode.ENGAGE)
	caster.ap = caster.max_ap
	var far_target: Combatant = _unit("FarTarget", 100, 10, 1, cgrid, Vector2i(8, 4))
	target.hp = 0
	cgrid.remove_combatant(target)
	var cctx3: PlanContext = _ctx(cgrid, [caster], [far_target])
	var must_walk: Array[PlanStep] = TacticalAI.build_steps(caster, cctx3, _profile(AiProfile.Role.SNIPER))
	var walk_plan := CombatPlan.new(cctx3)
	walk_plan.set_steps(caster, must_walk)
	var walk_state: CombatPlan.SimState = walk_plan.compile(caster)
	var attacked: bool = false
	for e: PlanEntry in walk_state.entries:
		attacked = attacked or e.skill.kind == SkillDef.Kind.ATTACK
	checks["target out of reach: no buff before moving, the turn is not wasted"] = not _any_skill_kind(must_walk, SkillDef.Kind.FOCUS) and attacked
	return _report("Kiting, no idle turns, buff only when free", checks)

# ------------------------------------------------------------------------------
# AI: support
# ------------------------------------------------------------------------------

static func _test_support_ai() -> bool:
	var checks: Dictionary = {}
	var mend: SkillDef = SkillCatalog.find("mend")
	var grid: CombatGrid = _grid()
	var medic: Combatant = _unit("Medic", 100, 10, 1, grid, Vector2i(2, 4), ["mend"])
	var hurt: Combatant = _unit("Hurt", 100, 10, 1, grid, Vector2i(3, 4))
	var scratched: Combatant = _unit("Scratched", 100, 10, 1, grid, Vector2i(3, 5))
	var fine: Combatant = _unit("Fine", 100, 10, 1, grid, Vector2i(3, 3))
	var foe: Combatant = _unit("Foe", 100, 10, 1, grid, Vector2i(8, 4))
	hurt.hp = 30
	scratched.hp = 60
	var allies: Array[Combatant] = [medic, hurt, scratched, fine]
	var ctx: PlanContext = _ctx(grid, allies, [foe])
	var support: AiProfile = _profile(AiProfile.Role.SUPPORT)
	var steps: Array[PlanStep] = TacticalAI.build_steps(medic, ctx, support)
	var patients: Array[Combatant] = []
	for s: PlanStep in steps:
		if s.mode == PlanStep.Mode.ALLY:
			patients.append(s.target)
	checks["heals the most wounded ally first, then the next below the threshold"] = patients.size() >= 2 and patients[0] == hurt and patients[1] == scratched
	checks["never heals the healthy or the same ally twice"] = not patients.has(fine) and patients.size() == 2
	var hp_before: int = hurt.hp
	_play(ctx, {medic: steps})
	checks["the heals land"] = hurt.hp == hp_before + mini(SupportRules.heal_amount(medic, mend), hurt.max_hp - hp_before) and scratched.hp > 60

	support.heal_below = 0.5
	scratched.hp = 60
	hurt.hp = 30
	var strict: Array[PlanStep] = TacticalAI.build_steps(medic, ctx, support)
	var strict_patients: int = 0
	for s: PlanStep in strict:
		if s.mode == PlanStep.Mode.ALLY:
			strict_patients += 1
	checks["threshold respected (60% is above 50%)"] = strict_patients == 1

	# Nobody hurt: attacks instead.
	hurt.hp = hurt.max_hp
	scratched.hp = scratched.max_hp
	var near_foe: Combatant = _unit("NearFoe", 100, 10, 1, grid, Vector2i(1, 4))
	var ctx2: PlanContext = _ctx(grid, allies, [near_foe])
	var fight: Array[PlanStep] = TacticalAI.build_steps(medic, ctx2, _profile(AiProfile.Role.SUPPORT))
	checks["no wounded ally: attacks"] = _any_mode(fight, PlanStep.Mode.ENGAGE) and not _any_mode(fight, PlanStep.Mode.ALLY)

	# Heals itself when alone and hurt (ALLY_OR_SELF).
	var solo_grid: CombatGrid = _grid()
	var solo: Combatant = _unit("Solo", 100, 10, 1, solo_grid, Vector2i(2, 4), ["mend"])
	var distant_foe: Combatant = _unit("Distant", 100, 10, 1, solo_grid, Vector2i(8, 8))
	solo.hp = 20
	var self_steps: Array[PlanStep] = TacticalAI.build_steps(solo, _ctx(solo_grid, [solo], [distant_foe]), _profile(AiProfile.Role.SUPPORT))
	checks["heals itself when it is the most wounded"] = not self_steps.is_empty() and self_steps[0].mode == PlanStep.Mode.SELF and self_steps[0].skill == mend

	# A support unit without a heal skill just fights.
	var plain_grid: CombatGrid = _grid()
	var plain: Combatant = _unit("Plain", 100, 10, 1, plain_grid, Vector2i(2, 4))
	var friend: Combatant = _unit("Friend", 100, 10, 1, plain_grid, Vector2i(2, 5))
	var enemy: Combatant = _unit("Enemy", 100, 10, 1, plain_grid, Vector2i(3, 4))
	friend.hp = 10
	var plain_steps: Array[PlanStep] = TacticalAI.build_steps(plain, _ctx(plain_grid, [plain, friend], [enemy]), _profile(AiProfile.Role.SUPPORT))
	checks["no heal skill: attacks"] = _any_mode(plain_steps, PlanStep.Mode.ENGAGE) and not _any_mode(plain_steps, PlanStep.Mode.ALLY)
	return _report("Support AI heals the most wounded ally", checks)

# ------------------------------------------------------------------------------
# AI: reckless profile
# ------------------------------------------------------------------------------

static func _test_reckless() -> bool:
	var checks: Dictionary = {}
	var exits: Array[Vector2i] = [Vector2i(8, 4)]
	var grid: CombatGrid = _grid(exits)
	var berserker: Combatant = _unit("Berserker", 100, 10, 1, grid, Vector2i(4, 4))
	var near: Combatant = _unit("Near", 100, 10, 1, grid, Vector2i(6, 4))
	var weak: Combatant = _unit("Weak", 100, 10, 1, grid, Vector2i(1, 6))
	weak.hp = 10
	berserker.hp = 10
	var ctx: PlanContext = _ctx(grid, [berserker], [near, weak])

	var careful: AiProfile = _profile(AiProfile.Role.AGGRESSOR, 0.5)
	careful.defend_below = 0.9
	var careful_steps: Array[PlanStep] = TacticalAI.build_steps(berserker, ctx, careful)
	checks["baseline: a hurt, non-reckless unit flees"] = _any_mode(careful_steps, PlanStep.Mode.WITHDRAW)

	var wild: AiProfile = careful.duplicate_profile()
	wild.reckless = true
	checks["duplicate keeps the flag"] = wild.reckless
	var wild_steps: Array[PlanStep] = TacticalAI.build_steps(berserker, ctx, wild)
	checks["reckless never flees or guards"] = not _any_mode(wild_steps, PlanStep.Mode.WITHDRAW) and not _any_skill_kind(wild_steps, SkillDef.Kind.DEFEND)
	checks["reckless still fights"] = _any_mode(wild_steps, PlanStep.Mode.ENGAGE)

	var picky: AiProfile = _profile(AiProfile.Role.SKIRMISHER)
	picky.target_pref = AiProfile.TargetPref.WEAKEST
	berserker.hp = berserker.max_hp
	var plain_target: Combatant = _first_engage_target(TacticalAI.build_steps(berserker, ctx, picky))
	picky.reckless = true
	var reckless_target: Combatant = _first_engage_target(TacticalAI.build_steps(berserker, ctx, picky))
	checks["reckless charges the nearest even when told to hunt the weakest"] = plain_target == weak and reckless_target == near and grid.get_distance(Vector2i(4, 4), Vector2i(6, 4)) < grid.get_distance(Vector2i(4, 4), Vector2i(1, 6))

	var entry_profile: AiProfile = ContentFactory.ai_profile({"class_id": "berserker", "level": 3, "reckless": true, "flee_ratio": 0.0}, berserker)
	var other_profile: AiProfile = ContentFactory.ai_profile({"class_id": "berserker", "level": 3}, berserker)
	checks["the unit's AI field sets the profile"] = entry_profile.reckless and not other_profile.reckless
	return _report("Reckless profile", checks)

# ------------------------------------------------------------------------------
# Catalog + schema
# ------------------------------------------------------------------------------

static func _test_catalog_and_schema() -> bool:
	var checks: Dictionary = {}
	var ids: Array[String] = ["mend", "rally", "taunt_shout", "lunge", "bleeding_cut"]
	var present: bool = true
	var round_trip: bool = true
	for id: String in ids:
		var entry: Dictionary = ContentCatalog.get_entry("skills", id)
		present = present and not entry.is_empty() and ContentCatalog.validate_entry("skills", entry, id).is_empty()
		var rebuilt: Dictionary = ContentFactory.skill_to_entry(ContentFactory.skill(entry))
		round_trip = round_trip and rebuilt == ContentSchema.normalize("skills", entry)
	checks["the five example skills validate"] = present
	checks["fields survive entry -> SkillDef -> entry"] = round_trip
	checks["whole catalog validates"] = ContentCatalog.validate_all().is_empty()
	var in_a_path: bool = false
	for cls: Dictionary in ContentCatalog.entries("classes"):
		for step: Variant in DictUtil.array_of(cls.get("skill_path", [])):
			in_a_path = in_a_path or ids.has(str(DictUtil.dict_of(step).get("skill", "")))
	checks["not in any class skill path yet (M2)"] = not in_a_path
	checks["learned_only: hand-built units don't know them until they learn them"] = (func() -> bool:
		var c := Combatant.new("U", 10, 10, 1)
		var before: bool = c.knows_skill("mend") or c.knows_skill("lunge")
		c.learn_skill("mend")
		return not before and c.knows_skill("mend") and not c.knows_skill("lunge") and c.knows_skill("strike")).call()

	# Class-derived units get them through extra_skills.
	var with_extra: Combatant = ContentFactory.unit({"id": "t_healer", "name": "T", "class_id": "shaman", "level": 3, "side": "party", "extra_skills": ["mend"]})
	var without: Combatant = ContentFactory.unit({"id": "t_plain", "name": "P", "class_id": "shaman", "level": 3, "side": "party"})
	checks["extra_skills grants a learned-only skill; others lack it"] = with_extra.can_use(SkillCatalog.find("mend")) and not without.can_use(SkillCatalog.find("mend"))

	# Status resist keys and passive validation include bleed (and refuse buffs).
	var keys_raw: Variant = ContentSchema.field("units", "status_resist").get("keys", [])
	var keys: Array = keys_raw if keys_raw is Array else []
	checks["status_resist keys include bleed"] = keys.has(StatusEffects.BLEED) and keys.has(StatusEffects.STAGGER)
	var exists: Callable = func(_id: String) -> bool: return true
	checks["passive resist accepts bleed, refuses haste"] = Passives.validate_effects([{"type": "resist", "status": "bleed", "value": 25}], exists).is_empty() and not Passives.validate_effects([{"type": "resist", "status": "haste", "value": 25}], exists).is_empty()
	checks["status option lists: debuffs for attacks, buffs for support"] = ContentSchema.options_of({"options_from": "status"}).has("bleed") and not ContentSchema.options_of({"options_from": "status"}).has("haste") and ContentSchema.options_of({"options_from": "status_buff"}).has("haste") and ContentSchema.options_of({"options_from": "status_buff"}).has("taunt")

	# Validation refuses bad data (sandbox copy; nothing is saved).
	for kind: String in ContentSchema.KINDS:
		DirAccess.remove_absolute(ContentCatalog.path_for(kind, SANDBOX))
	var forked: bool = ContentCatalog.fork_to(SANDBOX) == OK
	checks["sandbox fork"] = forked
	if forked:
		checks["heal per stat needs a stat"] = not ContentCatalog.upsert("skills", {"id": "bad_heal", "name": "x", "kind": "SUPPORT", "heal_mult": 1.0}).is_empty()
		checks["a support skill needs an effect"] = not ContentCatalog.upsert("skills", {"id": "bad_empty", "name": "x", "kind": "SUPPORT"}).is_empty()
		checks["grants must be a buff"] = not ContentCatalog.upsert("skills", {"id": "bad_grant", "name": "x", "kind": "SUPPORT", "grant_status": "stun"}).is_empty()
		checks["taunt must target SELF"] = not ContentCatalog.upsert("skills", {"id": "bad_taunt", "name": "x", "kind": "SUPPORT", "grant_status": "taunt", "target_mode": "ALLY"}).is_empty()
		checks["attack can't grant or heal"] = not ContentCatalog.upsert("skills", {"id": "bad_atk", "name": "x", "kind": "ATTACK", "heal_base": 5}).is_empty()
		checks["attack can only inflict debuffs"] = not ContentCatalog.upsert("skills", {"id": "bad_buffatk", "name": "x", "kind": "ATTACK", "status_effect": "haste", "status_potency": 50}).is_empty()
		checks["area attacks can't be gap-closes"] = not ContentCatalog.upsert("skills", {"id": "bad_dash", "name": "x", "kind": "ATTACK", "area": "PLUS", "dash_tiles": 2}).is_empty()
		checks["only attacks gap-close"] = not ContentCatalog.upsert("skills", {"id": "bad_dash2", "name": "x", "kind": "SUPPORT", "grant_status": "haste", "dash_tiles": 2}).is_empty()
		checks["a valid heal is accepted"] = ContentCatalog.upsert("skills", {"id": "good_heal", "name": "Good", "kind": "SUPPORT", "heal_stat": "INT", "heal_mult": 2.0, "heal_base": 3}).is_empty()
		var good: SkillDef = SkillCatalog.find("good_heal")
		checks["and builds a SkillDef with its heal"] = good != null and good.heals() and good.heal_stat == "INT"
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	return _report("Catalog and schema for the new fields", checks)

# Undoing a round (round 2+; round 1 restarts the battle) discards statuses and durations queued during it.
static func _test_reset_round_clears_queued_statuses() -> bool:
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
		session.plan_apply_preset_all("balanced")
		session.confirm_plan()
		var victim: Combatant = session.enemies[0]
		victim.pending_statuses[StatusEffects.BLEED] = true
		victim.pending_status_rounds[StatusEffects.BLEED] = 5
		session.reset_round()
		checks["queued statuses cleared"] = victim.pending_statuses.is_empty()
		checks["queued durations cleared"] = victim.pending_status_rounds.is_empty()
	session.dispose()
	return _report("Round reset discards queued statuses and their durations", checks)
