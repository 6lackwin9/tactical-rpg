class_name PlanningAiExtraTestSuite
extends RefCounted

# Second half of the planning/AP/AI checks (reach bands, presets, AI patterns, zone skirmishes, round reset).
# PlanningTestSuite.run() calls compute() and prints the combined assertion block, so the log order is unchanged.

static func _play_with_preset(zone_id: String, preset_id: String) -> Dictionary:
	var enc := CatalogEncounter.new("goblin_skirmish")
	enc.zone_id = zone_id
	var session := CombatSession.new(enc)
	session.start()
	for round_num: int in range(enc.max_rounds):
		session.plan_apply_preset_all(preset_id)
		if not session.confirm_plan():
			break
		TestHelpers.resolve_until_replay(session)
		if session.combat_over or not session.can_advance_round():
			break
		session.advance_round()
	var hp_sig: Array[int] = []
	var party_hurt: bool = false
	var foes_hurt: bool = false
	for c: Combatant in session.allies + session.enemies:
		hp_sig.append(c.hp)
		if c.hp < c.max_hp:
			if session.is_ally(c):
				party_hurt = true
			else:
				foes_hurt = true
	var result: Dictionary = {"hp": hp_sig, "log": session.log_entries.size(), "party_hurt": party_hurt, "foes_hurt": foes_hurt}
	session.dispose()
	return result

static func compute() -> Dictionary:
	var strike: SkillDef = SkillCatalog.find("strike")
	var mover: SkillDef = SkillCatalog.find("move")
	# 5e. Reach bands, move-first, statuses with resist rolls, area attacks, allies moving around each other
	var mgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var archer_u := Combatant.new("ArcherU", 50, 12, 3)
	var brute_u := Combatant.new("BruteU", 80, 10, 1)
	var far_u := Combatant.new("FarU", 80, 10, 1)
	mgrid.place_combatant(archer_u, Vector2i(2, 4))
	mgrid.place_combatant(brute_u, Vector2i(3, 4))
	mgrid.place_combatant(far_u, Vector2i(2, 1))
	var archer_side: Array[Combatant] = [archer_u]
	var brute_side: Array[Combatant] = [brute_u, far_u]
	var mctx: PlanContext = TestHelpers.hand_ctx(mgrid, archer_side, brute_side)
	var mplan := CombatPlan.new(mctx)
	var jab: SkillDef = SkillCatalog.find("offhand_jab")
	# ranged units: no ranged attack at distance 1 (once they can't step back), but the off-hand jab works there; 2-3 is fine
	var stepped_back: String = mplan.add_step(archer_u, PlanStep.engage(strike, brute_u))   # unacted: backs off to reach
	var backed_off: bool = stepped_back.is_empty() and mgrid.get_distance(mplan.compile(archer_u).pos, mgrid.get_position(brute_u)) >= 2
	mplan.clear(archer_u)
	mplan.add_step(archer_u, PlanStep.self_skill(SkillCatalog.find("rest")))
	var too_close: String = mplan.add_step(archer_u, PlanStep.engage(strike, brute_u))
	var jab_ok: String = mplan.add_step(archer_u, PlanStep.engage(jab, brute_u))
	mplan.clear(archer_u)
	var at_range: String = mplan.add_step(archer_u, PlanStep.engage(strike, far_u))
	var reach_ok: bool = backed_off and archer_u.min_attack_range() == 2 and brute_u.min_attack_range() == 1 and not too_close.is_empty() and jab_ok.is_empty() and at_range.is_empty()
	reach_ok = reach_ok and SkillCatalog.single_attacks(archer_u, 3).has(jab) and not SkillCatalog.single_attacks(brute_u, 3).has(jab)
	# moving must come first
	mplan.clear(archer_u)
	mplan.add_step(archer_u, PlanStep.self_skill(SkillCatalog.find("guard")))
	var late_move: String = mplan.add_step(archer_u, PlanStep.go_to(mover, Vector2i(1, 6)))
	var move_first_ok: bool = "must come first" in late_move
	# rooted units can't move
	mplan.clear(archer_u)
	archer_u.statuses[StatusEffects.ROOT] = true
	var rooted_move: String = mplan.add_step(archer_u, PlanStep.go_to(mover, Vector2i(1, 6)))
	move_first_ok = move_first_ok and "rooted" in rooted_move
	archer_u.statuses.clear()
	# status rolls: potency vs resistance, seeded; immunity blocks it
	var rng_a := RandomNumberGenerator.new()
	rng_a.seed = 7
	var rng_b := RandomNumberGenerator.new()
	rng_b.seed = 7
	var victim_a := Combatant.new("VictimA", 50, 10, 1)
	var victim_b := Combatant.new("VictimB", 50, 10, 1)
	var rolls_a: Array[bool] = []
	var rolls_b: Array[bool] = []
	for i: int in range(20):
		victim_a.pending_statuses.clear()
		victim_b.pending_statuses.clear()
		rolls_a.append(StatusEffects.try_inflict(victim_a, StatusEffects.STAGGER, 60, rng_a))
		rolls_b.append(StatusEffects.try_inflict(victim_b, StatusEffects.STAGGER, 60, rng_b))
	var landed: int = rolls_a.count(true)
	var immune := Combatant.new("Immune", 50, 10, 1)
	immune.status_resist[StatusEffects.ROOT] = 100
	var status_ok: bool = rolls_a == rolls_b and landed > 0 and landed < 20 and StatusEffects.apply_chance(immune, StatusEffects.ROOT, 90) == 0
	status_ok = status_ok and not StatusEffects.try_inflict(immune, StatusEffects.ROOT, 90, rng_a)
	victim_a.pending_statuses.clear()
	victim_a.pending_statuses[StatusEffects.ROOT] = true
	StatusEffects.roll_over(victim_a, 4)
	status_ok = status_ok and victim_a.has_status(StatusEffects.ROOT) and victim_a.pending_statuses.is_empty()
	StatusEffects.roll_over(victim_a, 4)
	status_ok = status_ok and not victim_a.has_status(StatusEffects.ROOT)   # lasts one round
	# area attack: Volley on a tile hits everyone in the plus, friends included (friendly fire)
	var agrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var thrower := Combatant.new("Thrower", 60, 12, 3)
	var buddy := Combatant.new("Buddy", 60, 10, 1)
	var foe_x := Combatant.new("FoeX", 60, 10, 1)
	var foe_y := Combatant.new("FoeY", 60, 10, 1)
	agrid.place_combatant(thrower, Vector2i(1, 4))
	agrid.place_combatant(foe_x, Vector2i(4, 4))
	agrid.place_combatant(foe_y, Vector2i(4, 5))
	agrid.place_combatant(buddy, Vector2i(4, 3))
	var throw_side: Array[Combatant] = [thrower, buddy]
	var target_side: Array[Combatant] = [foe_x, foe_y]
	var actx: PlanContext = TestHelpers.hand_ctx(agrid, throw_side, target_side)
	var aplan := CombatPlan.new(actx)
	var volley_opt: PlanOption = null
	for o: PlanOption in aplan.cell_options(thrower, Vector2i(4, 4)):
		if o.step.mode == PlanStep.Mode.AREA and o.step.skill.id == "volley":
			volley_opt = o
	var area_ok: bool = volley_opt != null and volley_opt.valid and volley_opt.damage > 0 and "Friendly fire" in volley_opt.warning
	aplan.add_step(thrower, volley_opt.step if volley_opt != null else PlanStep.withdraw())
	var aresolver := CombatResolver.new()
	aresolver.grid = agrid
	aplan.commit(aresolver)
	aresolver.resolve_round()
	area_ok = area_ok and foe_x.hp < 60 and foe_y.hp < 60 and buddy.hp < 60 and thrower.hp == 60
	var far_aim: Array[PlanOption] = aplan.cell_options(thrower, Vector2i(7, 7))
	var out_of_reach: bool = false
	for o: PlanOption in far_aim:
		if o.step.mode == PlanStep.Mode.AREA and o.step.skill.id == "volley" and not o.valid:
			out_of_reach = true
	area_ok = area_ok and out_of_reach
	# allies: the slower ally can take the faster ally's tile once that one moves off it in block 1
	var csession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	csession.start()
	var cw: Combatant = csession.allies[0] if csession.allies[0].speed >= csession.allies[1].speed else csession.allies[1]
	var cr: Combatant = csession.allies[1] if cw == csession.allies[0] else csession.allies[0]
	var w_start: Vector2i = csession.grid.get_position(cw)
	var blocked_before: String = ""
	for o: PlanOption in csession.plan_cell_options(w_start):
		pass
	csession.select_combatant(cr)
	var before_opts: Array[PlanOption] = csession.plan_cell_options(w_start)
	blocked_before = before_opts[0].reason if not before_opts.is_empty() and not before_opts[0].valid else ""
	csession.plan_add_step(cw, PlanStep.go_to(mover, Vector2i(2, 6)))   # the faster ally steps off its tile
	var cw_dest: Vector2i = csession.planned_position(cw)
	var into_vacated: String = csession.plan_add_step(cr, PlanStep.go_to(mover, w_start))
	var into_dest: String = csession.plan_add_step(cr, PlanStep.go_to(mover, cw_dest))
	var coord_ok: bool = not blocked_before.is_empty() and cw_dest != w_start and into_vacated.is_empty() and not into_dest.is_empty()
	coord_ok = coord_ok and csession.planned_position(cr) == w_start and csession.planned_occupant(w_start) == cr and csession.planned_occupant(cw_dest) == cw
	csession.dispose()
	var mechanics_ok: bool = reach_ok and move_first_ok and status_ok and area_ok and coord_ok
	# ranged single-target reach is 2-3 even for range-2 units; area reach is skill-specific
	var range2 := Combatant.new("Range2", 50, 10, 2)
	var reach2_ok: bool = strike.reach_for(range2) == Vector2i(2, 3) and strike.reach_for(brute_u) == Vector2i(1, 1)
	reach2_ok = reach2_ok and SkillCatalog.find("volley").reach_for(range2) == Vector2i(2, 3) and SkillCatalog.find("firebomb").reach_for(range2) == Vector2i(2, 2) and SkillCatalog.find("offhand_jab").reach_for(range2) == Vector2i(1, 1)
	# undo: removes the focused hero's last step; harmless when there is nothing to undo
	var usession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	usession.start()
	var uhero: Combatant = usession.focused_hero
	usession.plan_add_step(uhero, PlanStep.engage(strike, usession.enemies[1]))
	usession.plan_add_step(uhero, PlanStep.self_skill(SkillCatalog.find("guard")))
	usession.plan_undo_last()
	var undo_ok: bool = usession.plan.steps_of(uhero).size() == 1 and usession.plan.steps_of(uhero)[0].mode == PlanStep.Mode.ENGAGE
	usession.plan_undo_last()
	usession.plan_undo_last()
	undo_ok = undo_ok and usession.plan.steps_of(uhero).is_empty()
	usession.dispose()
	mechanics_ok = mechanics_ok and reach2_ok and undo_ok

	# the AI pack drafts into one shared plan: no two goblins plan to end on the same tile
	var pack := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	pack.start()
	var pack_dest: Dictionary = {}
	var pack_ok: bool = true
	for sa: CombatResolver.ScheduledAction in pack.resolver.scheduled_actions:
		var walk_cmd: WalkCommand = sa.command as WalkCommand
		if walk_cmd != null and walk_cmd.goal != Vector2i(-1, -1):
			pack_dest[sa.combatant] = walk_cmd.goal
	var seen_tiles: Dictionary = {}
	for unit: Variant in pack_dest:
		var tile: Vector2i = pack_dest[unit]
		if seen_tiles.has(tile):
			pack_ok = false
		seen_tiles[tile] = true
	pack.dispose()
	mechanics_ok = mechanics_ok and pack_ok

	# split moves: a 4-tile move at 2 tiles/block is two 1-block halves (first lands after block 1);
	# a half blocked by a unit that stepped in stops short, refunds its tiles and logs CUT SHORT
	var sgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var walker2 := Combatant.new("Walker2", 60, 8, 1)
	var cutter := Combatant.new("Cutter", 60, 30, 1)
	sgrid.place_combatant(walker2, Vector2i(1, 4))
	sgrid.place_combatant(cutter, Vector2i(6, 7))
	var walk_side: Array[Combatant] = [walker2]
	var cut_side: Array[Combatant] = [cutter]
	var sctx: PlanContext = TestHelpers.hand_ctx(sgrid, walk_side, cut_side)
	var splan := CombatPlan.new(sctx)
	var split_added: String = splan.add_step(walker2, PlanStep.go_to(mover, Vector2i(5, 4)))
	var halves: Array[PlanEntry] = splan.entries_for(walker2)
	var split_ok: bool = split_added.is_empty() and walker2.move_per_block() == 2 and halves.size() == 2
	if split_ok:
		split_ok = halves[0].start_block == 1 and halves[1].start_block == 2 and halves[0].blocks() == 1 and halves[0].pos_after == Vector2i(3, 4) and halves[1].pos_after == Vector2i(5, 4) and halves[0].skill_ap == 2 and halves[1].goal == Vector2i(5, 4)
	var sresolver := CombatResolver.new()
	sresolver.grid = sgrid
	splan.commit(sresolver)
	sresolver.execute_block(1)
	split_ok = split_ok and sgrid.get_position(walker2) == Vector2i(3, 4)
	sgrid.combatant_positions[cutter] = Vector2i(4, 4)   # an enemy steps into the path
	var ap_before_half: int = walker2.ap
	sresolver.execute_block(2)
	var cut_entry: ActionLogEntry = CombatLogBuilder.from_action(sresolver.scheduled_actions[1], 1, 2, walk_side, sgrid)
	split_ok = split_ok and sgrid.get_position(walker2) == Vector2i(3, 4) and walker2.ap == ap_before_half - 2 + 2 and cut_entry.outcome_tag == "CUT SHORT" and "Cutter" in cut_entry.status_text
	mechanics_ok = mechanics_ok and split_ok


	# 6. Presets: every preset drafts a valid, affordable sequence; each shows its focus
	var psession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	psession.start()
	PlanningTestSuite._pin_skirmish_layout(psession)   # the original layout: the weakest goblin stands within reach
	var hero: Combatant = psession.allies[0]
	var weakest: Combatant = psession.enemies[4]
	weakest.hp = 5
	var presets_ok: bool = true
	for preset_id: String in PlanPresets.IDS:
		psession.plan_apply_preset(hero, preset_id)
		presets_ok = presets_ok and psession.plan.is_valid(hero) and not psession.plan.compile(hero).runs_dry()
	psession.plan_apply_preset(hero, "aggressive")
	# AI1: the draft may open with a SUPPORT cast (taunt); everything else must be an attack
	var aggressive_ok: bool = not psession.plan.steps_of(hero).is_empty()
	var aggressive_engages: int = 0
	for s: PlanStep in psession.plan.steps_of(hero):
		var support_cast: bool = s.skill != null and s.skill.kind == SkillDef.Kind.SUPPORT
		aggressive_ok = aggressive_ok and (s.mode == PlanStep.Mode.ENGAGE or support_cast)
		aggressive_engages += 1 if s.mode == PlanStep.Mode.ENGAGE else 0
	aggressive_ok = aggressive_ok and aggressive_engages >= 1
	psession.plan_apply_preset(hero, "focus_weakest")
	var focus_steps: Array[PlanStep] = psession.plan.steps_of(hero)
	var first_attack: PlanStep = null
	for s: PlanStep in focus_steps:
		if first_attack == null and s.mode == PlanStep.Mode.ENGAGE and s.skill.deals_damage():
			first_attack = s
	var weakest_ok: bool = first_attack != null and first_attack.target == weakest
	psession.plan_apply_preset(hero, "economy")
	var economy_ok: bool = not psession.plan.steps_of(hero).is_empty()
	for s: PlanStep in psession.plan.steps_of(hero):
		economy_ok = economy_ok and (s.skill == null or s.skill.level == 1)
	psession.plan_apply_preset(hero, "defensive")
	# AI1: SUPPORT casts (taunt) and zero-power debuff casts (intimidate) may precede the guard; the first other step is the guard
	var first_action: PlanStep = null
	for s: PlanStep in psession.plan.steps_of(hero):
		if first_action == null and s.skill != null and s.skill.kind != SkillDef.Kind.SUPPORT and (s.skill.kind != SkillDef.Kind.ATTACK or s.skill.deals_damage()):
			first_action = s
	var defensive_ok: bool = first_action != null and first_action.mode == PlanStep.Mode.SELF and first_action.skill.kind == SkillDef.Kind.DEFEND
	psession.plan_apply_preset(hero, "withdraw")
	var withdraw_steps: Array[PlanStep] = psession.plan.steps_of(hero)
	var withdraw_ok: bool = not withdraw_steps.is_empty() and withdraw_steps[0].mode == PlanStep.Mode.WITHDRAW
	var preset_ok: bool = presets_ok and aggressive_ok and weakest_ok and economy_ok and defensive_ok and withdraw_ok
	psession.dispose()

	# 7. AI patterns on hand-built grids, verified by executing the drafted sequences
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var hunter := Combatant.new("Hunter", 60, 10, 1)
	var quarry := Combatant.new("Quarry", 100, 10, 1)
	grid.place_combatant(hunter, Vector2i(5, 4))
	grid.place_combatant(quarry, Vector2i(1, 4))
	var hunter_side: Array[Combatant] = [hunter]
	var quarry_side: Array[Combatant] = [quarry]
	var hunt_ctx: PlanContext = TestHelpers.hand_ctx(grid, quarry_side, hunter_side)
	var hunt_steps: Array[PlanStep] = TacticalAI.build_steps(hunter, hunt_ctx, hunt_ctx.profile_of(hunter))
	var hunt_plan := CombatPlan.new(hunt_ctx)
	hunt_plan.set_steps(hunter, hunt_steps)
	var closes_in: bool = not hunt_steps.is_empty() and hunt_steps[0].mode == PlanStep.Mode.ENGAGE and hunt_plan.is_valid(hunter) and not hunt_plan.compile(hunter).runs_dry()
	TestHelpers.run_steps(hunter, hunt_ctx, hunt_steps)
	closes_in = closes_in and quarry.hp < 100

	# Engage: the approach walks the planned path; if the target stepped one tile away,
	# the attack closes the gap when it starts (a target that ran farther is out of reach)
	var chase_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var chaser2 := Combatant.new("Chaser2", 60, 10, 1)
	var dodger := Combatant.new("Dodger", 100, 10, 1)
	chase_grid.place_combatant(chaser2, Vector2i(1, 4))
	chase_grid.place_combatant(dodger, Vector2i(5, 4))
	var chaser_side2: Array[Combatant] = [chaser2]
	var dodger_side: Array[Combatant] = [dodger]
	var chase_ctx: PlanContext = TestHelpers.hand_ctx(chase_grid, dodger_side, chaser_side2)
	var chase_plan := CombatPlan.new(chase_ctx)
	var chase_added: String = chase_plan.add_step(chaser2, PlanStep.engage(strike, dodger))
	var chase_resolver := CombatResolver.new()
	chase_resolver.grid = chase_grid
	var chase_log: Array[String] = []
	chase_resolver.log_emitted.connect(func(msg: String) -> void: chase_log.append(msg))
	chase_plan.commit(chase_resolver)
	chase_grid.combatant_positions[dodger] = Vector2i(5, 3)  # after confirming, the target steps 1 tile
	chase_resolver.resolve_round()
	var chased: bool = false
	for msg: String in chase_log:
		if msg.begins_with("[Chase]"):
			chased = true
	var chase_ok: bool = chase_added.is_empty() and dodger.hp < 100 and chased
	# a target that runs 3 tiles away is not chased down
	var runner_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var chaser3 := Combatant.new("Chaser3", 60, 10, 1)
	var sprinter := Combatant.new("Sprinter", 100, 10, 1)
	runner_grid.place_combatant(chaser3, Vector2i(1, 4))
	runner_grid.place_combatant(sprinter, Vector2i(5, 4))
	var sprinter_side: Array[Combatant] = [sprinter]
	var chaser3_side: Array[Combatant] = [chaser3]
	var runner_ctx: PlanContext = TestHelpers.hand_ctx(runner_grid, sprinter_side, chaser3_side)
	var runner_plan := CombatPlan.new(runner_ctx)
	runner_plan.add_step(chaser3, PlanStep.engage(strike, sprinter))
	var runner_resolver := CombatResolver.new()
	runner_resolver.grid = runner_grid
	runner_plan.commit(runner_resolver)
	runner_grid.combatant_positions[sprinter] = Vector2i(7, 7)
	runner_resolver.resolve_round()
	chase_ok = chase_ok and sprinter.hp == 100 and runner_grid.get_position(chaser3) != Vector2i(7, 6)
	# Only damage-focused attacks adjust: after a hand-picked tile move the unit holds its tile
	var hold_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var holder := Combatant.new("Holder", 60, 10, 1)
	var post := Combatant.new("Post", 100, 10, 1)
	hold_grid.place_combatant(holder, Vector2i(1, 4))
	hold_grid.place_combatant(post, Vector2i(4, 4))
	var holder_side: Array[Combatant] = [holder]
	var post_side: Array[Combatant] = [post]
	var hold_plan := CombatPlan.new(TestHelpers.hand_ctx(hold_grid, post_side, holder_side))
	hold_plan.add_step(holder, PlanStep.go_to(SkillCatalog.lowest(SkillDef.Kind.MOVE), Vector2i(3, 4)))
	hold_plan.add_step(holder, PlanStep.engage(strike, post))
	var hold_entries: Array[PlanEntry] = hold_plan.entries_for(holder)
	var hold_attack: PlanEntry = hold_entries[hold_entries.size() - 1]
	var hold_resolver := CombatResolver.new()
	hold_resolver.grid = hold_grid
	hold_plan.commit(hold_resolver)
	hold_grid.combatant_positions[post] = Vector2i(5, 4)   # steps away after the plan is locked
	hold_resolver.resolve_round()
	var held: bool = not hold_attack.chase and post.hp == 100 and hold_grid.get_position(holder) == Vector2i(3, 4)
	# ...while an engage (no tile move) keeps its one-tile adjustment
	var adjacent_plan := CombatPlan.new(TestHelpers.hand_ctx(hold_grid, post_side, holder_side))
	adjacent_plan.add_step(holder, PlanStep.engage(strike, post))
	var adjacent_entries: Array[PlanEntry] = adjacent_plan.entries_for(holder)
	held = held and adjacent_entries[adjacent_entries.size() - 1].chase
	# A ranged attacker whose target stepped point-blank backs off one tile to shoot
	var back_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var bowman := Combatant.new("Bowman", 60, 10, 3)
	var rusher := Combatant.new("Rusher", 100, 10, 1)
	back_grid.place_combatant(bowman, Vector2i(2, 4))
	back_grid.place_combatant(rusher, Vector2i(5, 4))
	var bowman_side: Array[Combatant] = [bowman]
	var rusher_side: Array[Combatant] = [rusher]
	var back_plan := CombatPlan.new(TestHelpers.hand_ctx(back_grid, rusher_side, bowman_side))
	var back_added: String = back_plan.add_step(bowman, PlanStep.engage(strike, rusher))
	var back_resolver := CombatResolver.new()
	back_resolver.grid = back_grid
	var back_log: Array[String] = []
	back_resolver.log_emitted.connect(func(msg: String) -> void: back_log.append(msg))
	back_plan.commit(back_resolver)
	back_grid.combatant_positions[rusher] = Vector2i(3, 4)   # rushes into point-blank range
	back_resolver.resolve_round()
	var backed: bool = false
	for msg: String in back_log:
		if msg.begins_with("[Chase]") and msg.contains("backs off"):
			backed = true
	chase_ok = chase_ok and held and back_added.is_empty() and backed and rusher.hp < 100

	var wall_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	for y: int in range(0, 5):
		wall_grid.set_terrain(Vector2i(4, y), Terrain.Type.WALL)
	var runner := Combatant.new("Runner", 60, 10, 1)
	var prey := Combatant.new("Prey", 60, 10, 1)
	wall_grid.place_combatant(runner, Vector2i(2, 4))
	wall_grid.place_combatant(prey, Vector2i(5, 4))
	var runner_side: Array[Combatant] = [runner]
	var prey_side: Array[Combatant] = [prey]
	var wall_ctx: PlanContext = TestHelpers.hand_ctx(wall_grid, prey_side, runner_side)
	var wall_steps: Array[PlanStep] = TacticalAI.build_steps(runner, wall_ctx, wall_ctx.profile_of(runner))
	TestHelpers.run_steps(runner, wall_ctx, wall_steps)
	var walls_ok: bool = prey.hp < 60

	var snipe_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var sniper := Combatant.new("Sniper", 40, 10, 3)
	var bruiser := Combatant.new("Bruiser", 60, 10, 1)
	snipe_grid.place_combatant(sniper, Vector2i(4, 4))
	snipe_grid.place_combatant(bruiser, Vector2i(3, 4))
	var sniper_side: Array[Combatant] = [sniper]
	var bruiser_side: Array[Combatant] = [bruiser]
	var snipe_ctx: PlanContext = TestHelpers.hand_ctx(snipe_grid, bruiser_side, sniper_side)
	var snipe_steps: Array[PlanStep] = TacticalAI.build_steps(sniper, snipe_ctx, snipe_ctx.profile_of(sniper))
	var sniper_ok: bool = not snipe_steps.is_empty() and snipe_steps[0].mode == PlanStep.Mode.MOVE and snipe_steps[0].direction == Vector2i(1, 0)
	var support_profile: AiProfile = snipe_ctx.profile_of(sniper).duplicate_profile()
	support_profile.role = AiProfile.Role.SUPPORT
	var support_steps: Array[PlanStep] = TacticalAI.build_steps(sniper, snipe_ctx, support_profile)
	var support_ok: bool = not support_steps.is_empty() and support_steps[0].skill.kind == SkillDef.Kind.FOCUS

	var weakling := Combatant.new("Weakling", 100, 10, 1)
	weakling.hp = 10
	var chaser := Combatant.new("Chaser", 100, 10, 1)
	var flee_grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	flee_grid.place_combatant(weakling, Vector2i(5, 4))
	flee_grid.place_combatant(chaser, Vector2i(2, 4))
	var weak_side: Array[Combatant] = [weakling]
	var chaser_side: Array[Combatant] = [chaser]
	var flee_ctx: PlanContext = TestHelpers.hand_ctx(flee_grid, chaser_side, weak_side)
	var flee_steps: Array[PlanStep] = TacticalAI.build_steps(weakling, flee_ctx, flee_ctx.profile_of(weakling))
	TestHelpers.run_steps(weakling, flee_ctx, flee_steps)
	var flee_ok: bool = not flee_steps.is_empty() and flee_steps[0].mode == PlanStep.Mode.WITHDRAW and weakling.is_withdrawn
	var ai_ok: bool = closes_in and walls_ok and sniper_ok and support_ok and flee_ok and chase_ok

	# 8. AI vs AI preset skirmishes: every zone is deterministic and hurts both sides
	var zones_ok: bool = true
	var both_hurt: bool = true
	for zone_id: String in CombatZones.zone_ids():
		var run_a: Dictionary = _play_with_preset(zone_id, "balanced")
		var run_b: Dictionary = _play_with_preset(zone_id, "balanced")
		if run_a.get("hp", []) != run_b.get("hp", []) or run_a.get("log", 0) != run_b.get("log", -1):
			zones_ok = false
			print("    non-deterministic result on zone: %s" % zone_id)
		if not (DictUtil.to_bool(run_a.get("party_hurt", false)) and DictUtil.to_bool(run_a.get("foes_hurt", false))):
			both_hurt = false
			print("    one side untouched on zone: %s %s" % [zone_id, str(run_a.get("hp", []))])

	# 9. Resetting a planned round rewinds units to the start of that round
	var rs := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	rs.start()
	rs.plan_apply_preset_all("balanced")
	rs.confirm_plan()
	TestHelpers.resolve_until_replay(rs)
	var reset_ok: bool = false
	if rs.can_advance_round():
		rs.advance_round()
		var start_hp: Array[int] = []
		var start_ap: Array[int] = []
		var start_pos: Array[Vector2i] = []
		for c: Combatant in rs.allies + rs.enemies:
			start_ap.append(c.ap)
			start_hp.append(c.hp)
			start_pos.append(rs.grid.get_position(c))
		rs.plan_apply_preset_all("balanced")
		rs.confirm_plan()
		TestHelpers.resolve_until_replay(rs)
		rs.reset_round()
		var end_hp: Array[int] = []
		var end_ap: Array[int] = []
		var end_pos: Array[Vector2i] = []
		for c: Combatant in rs.allies + rs.enemies:
			end_ap.append(c.ap)
			end_hp.append(c.hp)
			end_pos.append(rs.grid.get_position(c))
		reset_ok = start_hp == end_hp and start_pos == end_pos and rs.is_planning() and rs.current_round == 2 and start_ap == end_ap
	rs.dispose()
	return {"mechanics_ok": mechanics_ok, "preset_ok": preset_ok, "ai_ok": ai_ok, "zones_ok": zones_ok, "both_hurt": both_hurt, "reset_ok": reset_ok}
