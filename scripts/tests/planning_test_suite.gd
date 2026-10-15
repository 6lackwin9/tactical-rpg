class_name PlanningTestSuite
extends RefCounted

# ==============================================================================
# TEST 7: PLANNING SEQUENCES, SKILL AP, STANCES, RUNTIME AP & ENEMY AI
# ==============================================================================

# The roster rework moved the Goblin Shaman (now a melee Priest) and Berserker (now a Brawler); real deployment positions are therefore NOT exercised by
# the planning checks below, which were written against the original layout and pin those two tiles.
static func _pin_skirmish_layout(s: CombatSession) -> void:
	var enc: CatalogEncounter = s.encounter as CatalogEncounter
	if enc == null:
		return
	s.grid.combatant_positions[enc.unit("goblin_shaman")] = Vector2i(6, 4)
	s.grid.combatant_positions[enc.unit("goblin_berserker")] = Vector2i(3, 4)

static func run(_tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING PLANNING, STANCES, AP & ENEMY AI TEST  ===")
	print("=======================================================")

	# 1. Skill AP: level = AP, Move = 1 AP per tile, Rest free; effects shift costs
	var skills_ok: bool = SkillCatalog.all().size() >= 10
	for s: SkillDef in SkillCatalog.all():
		if s.kind == SkillDef.Kind.ATTACK or s.kind == SkillDef.Kind.DEFEND or s.kind == SkillDef.Kind.FOCUS:
			skills_ok = skills_ok and s.ap_cost() == s.level
	var l2_ok: bool = SkillCatalog.find("hamstring").ap_cost() == 2 and SkillCatalog.find("riposte").ap_cost() == SkillCatalog.find("riposte").level and SkillCatalog.find("focus").ap_cost() == 2
	var modded := Combatant.new("Modded", 10, 10, 1)
	modded.ap_cost_modifier = 1
	var mod_ok: bool = SkillCatalog.find("strike").ap_cost(modded) == 2 and SkillCatalog.find("haymaker").ap_cost(modded) == 4 and SkillCatalog.find("move").ap_cost(modded) == 2 and SkillCatalog.find("rest").ap_cost(modded) == 0
	modded.ap_cost_modifier = -5
	mod_ok = mod_ok and SkillCatalog.find("haymaker").ap_cost(modded) == 1
	var fast := Combatant.new("Fast", 10, 15, 1)
	var slow := Combatant.new("Slow", 10, 8, 1)
	var speed_ok: bool = fast.move_per_block() == 3 and slow.move_per_block() == 2 and Combatant.new("Tortoise", 10, 1, 1).move_per_block() == 1
	var ap_model_ok: bool = skills_ok and l2_ok and mod_ok and speed_ok

	# 2. Stances: RPS loop with Neutral outside it; automatic switch costs
	var N: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
	var O: CombatTypes.StanceType = CombatTypes.StanceType.OFFENSIVE
	var D: CombatTypes.StanceType = CombatTypes.StanceType.DEFENSIVE
	var T: CombatTypes.StanceType = CombatTypes.StanceType.TACTICAL
	var rps_ok: bool = CombatTypes.get_matchup_multiplier(O, T) == 1.25 and CombatTypes.get_matchup_multiplier(T, D) == 1.25 and CombatTypes.get_matchup_multiplier(D, O) == 1.25
	rps_ok = rps_ok and CombatTypes.get_matchup_multiplier(T, O) == 0.75 and CombatTypes.get_matchup_multiplier(D, T) == 0.75 and CombatTypes.get_matchup_multiplier(O, D) == 0.75
	rps_ok = rps_ok and CombatTypes.get_matchup_multiplier(N, O) == 1.0 and CombatTypes.get_matchup_multiplier(T, N) == 1.0 and CombatTypes.get_matchup_multiplier(O, O) == 1.0
	var strike: SkillDef = SkillCatalog.find("strike")
	var guard_skill: SkillDef = SkillCatalog.find("guard")
	var mover: SkillDef = SkillCatalog.find("move")
	var rest_skill: SkillDef = SkillCatalog.find("rest")
	var sw: int = CombatTypes.STANCE_SWITCH_COST
	var switch_ok: bool = strike.switch_cost(N) == sw and strike.switch_cost(O) == 0 and guard_skill.switch_cost(O) == sw and guard_skill.switch_cost(D) == 0
	# Rest enters the stance SkillDef.stance() names (Tactical) for free, from any stance; the skill after Rest pays the usual switch.
	var focus_skill: SkillDef = SkillCatalog.find("focus")
	switch_ok = switch_ok and mover.switch_cost(T) == 0 and mover.stance_after(T) == T and rest_skill.switch_cost(D) == 0 and rest_skill.stance_after(D) == rest_skill.stance() and rest_skill.stance() == T
	switch_ok = switch_ok and focus_skill.switch_cost(rest_skill.stance()) == 0 and strike.switch_cost(rest_skill.stance()) == sw and guard_skill.switch_cost(rest_skill.stance()) == sw
	var stance_ok: bool = rps_ok and switch_ok

	# 3. Sequence compile: lowest free blocks, Engage auto-moves, AP incl. stance switch
	var session := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	session.start()
	_pin_skirmish_layout(session)
	var warrior: Combatant = session.allies[0]
	var foe: Combatant = session.enemies[0]
	warrior.learn_skill("hamstring")   # the tank warrior's path does not include it; learn this L2 attack for the planner checks
	var committed: SkillDef = SkillCatalog.find("hamstring")   # a level-2 attack with no weapon gating
	var first: String = session.plan_add_step(warrior, PlanStep.engage(committed, foe))
	var compiled: CombatPlan.SimState = session.plan.compile(warrior)
	var attack_entry: PlanEntry = compiled.entries[compiled.entries.size() - 1] if not compiled.entries.is_empty() else null
	var chain_ok: bool = first.is_empty() and attack_entry != null and attack_entry.skill == committed and attack_entry.target == foe and attack_entry.switch_ap == 1
	var blocks_ok: bool = true
	var expected_block: int = 1
	var ap_sum: int = 0
	var moves_ok: bool = true
	for e: PlanEntry in compiled.entries:
		blocks_ok = blocks_ok and e.start_block == expected_block
		expected_block += e.blocks()
		ap_sum += e.total_ap()
		if e.skill.kind == SkillDef.Kind.MOVE:
			moves_ok = moves_ok and e.blocks() == 1 and e.skill_ap == e.tiles and e.tiles <= warrior.move_per_block()
	blocks_ok = blocks_ok and compiled.ap_left == warrior.ap - ap_sum and compiled.next_block == expected_block
	var in_range_ok: bool = session.grid.get_distance(compiled.pos, session.grid.get_position(foe)) <= warrior.attack_range
	var stance_after_ok: bool = compiled.stance == O
	var seq_ok: bool = chain_ok and blocks_ok and moves_ok and in_range_ok and stance_after_ok

	# Switching stance mid-sequence is allowed and automatic: +1 AP, no block
	session.plan_clear(warrior)
	session.plan_add_step(warrior, PlanStep.engage(strike, foe))
	var before_strike: CombatPlan.SimState = session.plan.compile(warrior)
	var switched: String = session.plan_add_step(warrior, PlanStep.self_skill(SkillCatalog.find("guard")))
	var after_strike: CombatPlan.SimState = session.plan.compile(warrior)
	var switch_entry: PlanEntry = after_strike.entries[after_strike.entries.size() - 1]
	var auto_switch_ok: bool = switched.is_empty() and switch_entry.switch_ap == 1 and switch_entry.stance_before == O and switch_entry.stance_after == D
	auto_switch_ok = auto_switch_ok and before_strike.ap_left - after_strike.ap_left >= 2

	# AP is a runtime resource: over-committing is allowed, the plan projects where it runs dry
	session.plan_clear(warrior)
	warrior.ap = committed.ap_cost()   # the skill plus its +1 stance switch is more than the unit can pay
	var _last_problem: String = session.plan_add_step(warrior, PlanStep.engage(committed, session.enemies[1]))
	_last_problem += session.plan_add_step(warrior, PlanStep.engage(SkillCatalog.find("strike"), session.enemies[1]))
	var dry_state: CombatPlan.SimState = session.plan.compile(warrior)
	var dry_ok: bool = dry_state.runs_dry() and dry_state.dry_step >= 0 and dry_state.ap_left < 0 and dry_state.debt() > 0 and session.plan_is_valid()
	var _option_warns: bool = false
	var rest_option: bool = false
	for o: PlanOption in session.plan_options(warrior):
		if o.step.mode == PlanStep.Mode.ENGAGE and not o.warning.is_empty():
			_option_warns = true
	session.plan_clear(warrior)
	warrior.ap = warrior.max_ap
	for o: PlanOption in session.plan_options(warrior):
		if o.step.mode == PlanStep.Mode.SELF and o.step.skill.kind == SkillDef.Kind.REST and o.valid:
			rest_option = true
	var runtime_plan_ok: bool = dry_ok and rest_option
	var options_ok: bool = rest_option and not session.plan_options(warrior).is_empty()

	# 4. Editing: a sequence broken by an edit is flagged and blocks Confirm; fixing it allows Confirm
	var fallen: Combatant = session.enemies[3]
	var fallen_hp: int = fallen.hp
	fallen.hp = 0  # an enemy that has since been defeated
	var broken: Array[PlanStep] = [PlanStep.engage(strike, foe), PlanStep.engage(strike, fallen)]
	session.plan.set_steps(warrior, broken)
	var flagged: CombatPlan.SimState = session.plan.compile(warrior)
	var edit_ok: bool = flagged.error_index == 1 and not session.plan_is_valid() and not session.confirm_plan()
	fallen.hp = fallen_hp
	session.plan_remove_step(warrior, 1)
	edit_ok = edit_ok and session.plan_is_valid()
	session.plan_add_step(warrior, PlanStep.engage(strike, session.enemies[1]))
	session.plan_move_step(warrior, 1, 0)
	edit_ok = edit_ok and session.plan.steps_of(warrior).size() == 2 and session.plan.steps_of(warrior)[0].target == session.enemies[1] and session.plan_is_valid()
	# 5. Confirm locks the plan, schedules party + AI enemies, and resolution runs
	session.plan_apply_preset(session.allies[1], "balanced")
	var preview: CombatResolver.ScheduledAction = session.find_recorded_action(warrior, 1)
	var preview_ok: bool = preview != null
	var confirmed: bool = session.confirm_plan()
	var locked_ok: bool = confirmed and not session.is_planning() and not session.plan_add_step(warrior, PlanStep.self_skill(guard_skill)).is_empty()
	var party_scheduled: bool = false
	var enemy_scheduled: bool = false
	for a: CombatResolver.ScheduledAction in session.resolver.scheduled_actions:
		if a.combatant == warrior:
			party_scheduled = true
		if not session.is_ally(a.combatant):
			enemy_scheduled = true
	TestHelpers.resolve_until_replay(session)
	var spent_ok: bool = warrior.ap < warrior.max_ap
	var flow_ok2: bool = preview_ok and locked_ok and party_scheduled and enemy_scheduled and session.phase == CombatSession.Phase.REPLAY and spent_ok
	session.dispose()

	# 5b. Runtime AP: overspending runs into AP debt (paid next round: less AP, 3+ Stagger, 5+ Stun); Rest recovers mid-round
	var rgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var spent := Combatant.new("Spent", 60, 10, 1)
	var dummy := Combatant.new("Dummy", 100, 10, 1)
	rgrid.place_combatant(spent, Vector2i(3, 4))
	rgrid.place_combatant(dummy, Vector2i(4, 4))
	var spent_side: Array[Combatant] = [spent]
	var dummy_side: Array[Combatant] = [dummy]
	var rctx: PlanContext = TestHelpers.hand_ctx(rgrid, dummy_side, spent_side)
	spent.ap = 1
	var exhausted_steps: Array[PlanStep] = [PlanStep.engage(strike, dummy), PlanStep.engage(strike, dummy)]
	var exhaust_log: Array[String] = TestHelpers.run_steps(spent, rctx, exhausted_steps)
	var stops_ok: bool = dummy.hp < 100 and spent.ap == -2   # both strikes land: 1 + 1 (stance) + 1 AP
	var said_exhausted: bool = false
	for msg: String in exhaust_log:
		if msg.begins_with("[AP Debt]"):
			said_exhausted = true
	StatusEffects.roll_over(spent, 4)
	stops_ok = stops_ok and spent.ap == 2 and spent.statuses.is_empty()
	spent.ap = -3
	StatusEffects.roll_over(spent, 4)
	var staggered_plan := CombatPlan.new(rctx)
	var stagger_entries: Array[PlanEntry] = staggered_plan.entries_for(spent)
	stops_ok = stops_ok and spent.has_status(StatusEffects.STAGGER) and spent.ap == 1 and stagger_entries.size() == 2 and stagger_entries[0].start_block == 1 and stagger_entries[1].start_block == 4
	staggered_plan.add_step(spent, PlanStep.engage(strike, dummy))
	var after_stagger: Array[PlanEntry] = staggered_plan.entries_for(spent)
	stops_ok = stops_ok and after_stagger[after_stagger.size() - 1].start_block == 2   # skips the lost block 1
	spent.ap = -5
	StatusEffects.roll_over(spent, 4)
	stops_ok = stops_ok and spent.has_status(StatusEffects.STUN) and not spent.has_status(StatusEffects.STAGGER)
	spent.statuses.clear()
	spent.ap = 0
	var rest_steps: Array[PlanStep] = [PlanStep.self_skill(rest_skill), PlanStep.engage(strike, dummy)]
	TestHelpers.run_steps(spent, rctx, rest_steps)
	var rest_ok: bool = dummy.hp < 100 and spent.ap == 0
	# B1: a rested unit is in the stance SkillDef.stance() names (Tactical); Focus after Rest pays no switch, an attack pays STANCE_SWITCH_COST
	rest_ok = rest_ok and rest_skill.stance() == CombatTypes.StanceType.TACTICAL
	var focus_rest: SkillDef = SkillCatalog.find("focus")
	spent.ap = 0
	spent.set_stance(CombatTypes.StanceType.NEUTRAL)
	var rest_focus_steps: Array[PlanStep] = [PlanStep.self_skill(rest_skill), PlanStep.self_skill(focus_rest)]
	TestHelpers.run_steps(spent, rctx, rest_focus_steps)
	rest_ok = rest_ok and spent.current_stance == rest_skill.stance() and focus_rest.stance() == rest_skill.stance()
	rest_ok = rest_ok and spent.ap == mini(spent.max_ap, rest_skill.recover_ap) - focus_rest.ap_cost(spent)
	spent.ap = 0
	spent.set_stance(CombatTypes.StanceType.DEFENSIVE)
	var rest_atk_steps: Array[PlanStep] = [PlanStep.self_skill(rest_skill), PlanStep.engage(strike, dummy)]
	TestHelpers.run_steps(spent, rctx, rest_atk_steps)
	rest_ok = rest_ok and spent.current_stance == strike.stance() and spent.ap == mini(spent.max_ap, rest_skill.recover_ap) - strike.ap_cost(spent) - CombatTypes.STANCE_SWITCH_COST
	var runtime_ok: bool = stops_ok and said_exhausted and rest_ok

	# 5c. On-grid input model: options per clicked target / tile, hover previews, status line
	var gsession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	gsession.start()
	var gh: Combatant = gsession.allies[0]
	var gfoe: Combatant = gsession.enemies[0]
	var foe_opts: Array[PlanOption] = gsession.plan_target_options(gfoe)
	var self_opts: Array[PlanOption] = gsession.plan_target_options(gh)
	var ally_opts: Array[PlanOption] = gsession.plan_target_options(gsession.allies[1])
	var tile_opts: Array[PlanOption] = gsession.plan_cell_options(Vector2i(2, 6))
	var exit_opts: Array[PlanOption] = gsession.plan_cell_options(Vector2i(8, 4))
	var occupied_opts: Array[PlanOption] = gsession.plan_cell_options(gsession.grid.get_position(gfoe))
	var engage_count: int = 0
	var area_count: int = 0
	for o: PlanOption in foe_opts:
		if o.step.mode == PlanStep.Mode.ENGAGE:
			engage_count += 1
		elif o.step.mode == PlanStep.Mode.AREA:
			area_count += 1
	var targets_ok: bool = engage_count == SkillCatalog.single_attacks(gh, 3).size() and area_count == SkillCatalog.area_attacks(3, gh).size() and ally_opts.is_empty() and tile_opts.size() == 1 + area_count and exit_opts.size() == 1
	for o: PlanOption in foe_opts:
		if o.step.mode == PlanStep.Mode.ENGAGE:
			targets_ok = targets_ok and o.step.target == gfoe and o.valid
	var has_rest: bool = false
	for o: PlanOption in self_opts:
		targets_ok = targets_ok and o.step.mode == PlanStep.Mode.SELF
		if o.step.skill.kind == SkillDef.Kind.REST:
			has_rest = true
	targets_ok = targets_ok and has_rest and tile_opts[0].step.mode == PlanStep.Mode.GOTO and tile_opts[0].valid and exit_opts[0].step.mode == PlanStep.Mode.WITHDRAW
	targets_ok = targets_ok and (occupied_opts.is_empty() or not occupied_opts[0].valid)
	var previews: Array[int] = [0]
	gsession.plan_preview_changed.connect(func() -> void: previews[0] += 1)
	gsession.set_plan_preview(tile_opts[0].step)
	var pst: CombatPlan.SimState = gsession.plan_preview_state()
	var grid_preview_ok: bool = previews[0] == 1 and pst != null and pst.pos == Vector2i(2, 6) and gsession.plan.steps_of(gh).is_empty()
	gsession.set_plan_preview(null)
	var added: String = gsession.plan_add_step(gh, tile_opts[0].step)
	var moved_state: CombatPlan.SimState = gsession.plan.compile(gh)
	var trail_ok: bool = added.is_empty() and moved_state.pos == Vector2i(2, 6) and not moved_state.entries.is_empty() and moved_state.entries[0].pos_before == Vector2i(4, 5) and moved_state.entries[moved_state.entries.size() - 1].pos_after == Vector2i(2, 6)
	var info: Dictionary = gsession.plan_info()
	var info_ok: bool = DictUtil.to_bool(info.get("planning", false)) and "Warrior" in str(info.get("text", "")) and DictUtil.to_bool(info.get("can_confirm", false)) and gsession.plan_step_index_at(1) == 0 and gsession.plan_step_index_at(10) == -1
	gsession.set_hovered(gfoe)
	var hover_ok: bool = gsession.inspected_combatant() == gfoe
	gsession.set_hovered(null)
	hover_ok = hover_ok and gsession.inspected_combatant() == gsession.selected_combatant
	var input_ok: bool = targets_ok and grid_preview_ok and trail_ok and info_ok and hover_ok
	gsession.dispose()
	# 5d. Round rules: 6 blocks, AP pool + regen, one move per round, target switching, insurance attacks, live follow
	var rsession := CombatSession.new(CatalogEncounter.new("goblin_skirmish"))
	rsession.start()
	_pin_skirmish_layout(rsession)
	var rhero: Combatant = rsession.allies[0]
	var rules_ok: bool = rsession.total_blocks == 6 and rsession.resolver.total_blocks == 6 and rsession.encounter.plan_context().total_blocks == 6 and rhero.max_ap == 12 and rhero.ap == 12
	# centre tie-break: the Warrior (4,5) closing on the Skirmisher (5,4) steps to the centre (4,4), not (5,5)
	var skirm: Combatant = rsession.enemies[1]
	rsession.plan_add_step(rhero, PlanStep.engage(strike, skirm))
	var approach_entry: PlanEntry = rsession.plan.compile(rhero).entries[0]
	rules_ok = rules_ok and rsession.grid.get_position(skirm) == Vector2i(5, 4) and approach_entry.skill.kind == SkillDef.Kind.MOVE and approach_entry.pos_after == Vector2i(4, 4)
	# the move is spent: a second move or a far second engage is refused, an in-range attack is fine
	var second_move: String = rsession.plan_add_step(rhero, PlanStep.go_to(mover, Vector2i(2, 6)))
	var far_engage: String = rsession.plan_add_step(rhero, PlanStep.engage(strike, rsession.enemies[2]))
	rules_ok = rules_ok and "Moving must come first" in second_move and not far_engage.is_empty()
	# insurance: a target already expected to fall can still be attacked (it may dodge or guard)
	rsession.enemies[1].hp = 5
	var insurance_warned: bool = false
	for o: PlanOption in rsession.plan_target_options(skirm):
		if o.valid and not o.warning.is_empty():
			insurance_warned = true
	var insured: String = rsession.plan_add_step(rhero, PlanStep.engage(strike, skirm))
	rules_ok = rules_ok and insured.is_empty() and insurance_warned
	# a round's move is 2 blocks at most, so farther tiles are refused
	rsession.plan_clear(rhero)
	rhero.speed = 8   # 2 tiles per block: 4 tiles in a round
	var far_tile_fresh: Array[PlanOption] = rsession.plan_cell_options(Vector2i(1, 3))
	rules_ok = rules_ok and not far_tile_fresh.is_empty() and not far_tile_fresh[0].valid and "Too far" in far_tile_fresh[0].reason
	rhero.speed = 15
	# AP carries over: +4 each round up to the pool
	var ap_before: int = rhero.ap
	rsession.plan_add_step(rhero, PlanStep.self_skill(SkillCatalog.find("guard")))
	rsession.confirm_plan()
	var autoplayed: bool = rsession.is_auto_playing
	TestHelpers.resolve_until_replay(rsession)
	var ap_after_round: int = rhero.ap
	var attack_logged: bool = false
	if rsession.can_advance_round():
		rsession.advance_round()
		rules_ok = rules_ok and rhero.ap == mini(rhero.max_ap, ap_after_round + 4) and ap_after_round < ap_before + 1
	rules_ok = rules_ok and autoplayed
	for e: ActionLogEntry in rsession.log_entries:
		if e.category == "ATTACK" and e.damage > 0:
			attack_logged = attack_logged or (e.value_num == e.damage and not e.breakdown.is_empty() and "stance x" in e.breakdown)
	rules_ok = rules_ok and attack_logged
	rsession.dispose()
	# hand grid: target switching costs a block; attacks follow a target that steps away
	var hgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	var duelist := Combatant.new("Duelist", 100, 15, 1)
	var foe_a := Combatant.new("FoeA", 100, 10, 1)
	var foe_b := Combatant.new("FoeB", 100, 10, 1)
	hgrid.place_combatant(duelist, Vector2i(3, 4))
	hgrid.place_combatant(foe_a, Vector2i(4, 4))
	hgrid.place_combatant(foe_b, Vector2i(3, 3))
	var duel_side: Array[Combatant] = [duelist]
	var foes_side: Array[Combatant] = [foe_a, foe_b]
	var hctx: PlanContext = TestHelpers.hand_ctx(hgrid, duel_side, foes_side)
	var hplan := CombatPlan.new(hctx)
	hplan.add_step(duelist, PlanStep.engage(strike, foe_a))   # strike (2) + switch (1) + strike (2) fits a 6-block round
	var no_switch: bool = true
	for e: PlanEntry in hplan.entries_for(duelist):
		no_switch = no_switch and e.skill.kind != SkillDef.Kind.TARGET
	hplan.add_step(duelist, PlanStep.engage(strike, foe_b))
	var switch_seen: PlanEntry = null
	for e: PlanEntry in hplan.entries_for(duelist):
		if e.skill.kind == SkillDef.Kind.TARGET:
			switch_seen = e
	var target_switch_ok: bool = no_switch and switch_seen != null and switch_seen.blocks() == 1 and switch_seen.total_ap() == 0 and switch_seen.target == foe_b
	var follower := Combatant.new("Follower", 100, 10, 1)
	var runner2 := Combatant.new("Runner2", 100, 10, 1)
	var fgrid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [Vector2i(8, 4)])
	fgrid.place_combatant(follower, Vector2i(1, 4))
	fgrid.place_combatant(runner2, Vector2i(2, 4))
	var follow_side: Array[Combatant] = [follower]
	var run_side: Array[Combatant] = [runner2]
	var fctx: PlanContext = TestHelpers.hand_ctx(fgrid, run_side, follow_side)
	var fplan := CombatPlan.new(fctx)
	fplan.add_step(follower, PlanStep.engage(strike, runner2))
	fgrid.combatant_positions[runner2] = Vector2i(3, 4)   # steps away after planning: the attack follows one tile
	var fresolver := CombatResolver.new()
	fresolver.grid = fgrid
	fplan.commit(fresolver)
	fresolver.resolve_round()
	var follow_ok: bool = runner2.hp < 100
	rules_ok = rules_ok and target_switch_ok and follow_ok
	var extra: Dictionary = PlanningAiExtraTestSuite.compute()
	var mechanics_ok: bool = DictUtil.to_bool(extra.get("mechanics_ok", false))
	var preset_ok: bool = DictUtil.to_bool(extra.get("preset_ok", false))
	var ai_ok: bool = DictUtil.to_bool(extra.get("ai_ok", false))
	var zones_ok: bool = DictUtil.to_bool(extra.get("zones_ok", false))
	var both_hurt: bool = DictUtil.to_bool(extra.get("both_hurt", false))
	var reset_ok: bool = DictUtil.to_bool(extra.get("reset_ok", false))

	print("\n=======================================================")
	print("=== PLANNING, STANCES, AP & ENEMY AI ASSERTIONS     ===")
	print("=======================================================")
	print("  [Skills] AP = skill level, Move 1 AP/tile, Rest free, effects shift costs, speed sets tiles/block: %s" % ("PASS" if ap_model_ok else "FAIL"))
	print("  [Stance] Offensive > Tactical > Defensive > Offensive; Neutral neutral; auto switch costs +1 AP: %s" % ("PASS" if stance_ok else "FAIL"))
	print("  [Plan] Sequence compiles to lowest free blocks; Engage auto-inserts 1-block moves; AP incl. switch: %s" % ("PASS" if seq_ok else "FAIL"))
	print("  [Plan] Mid-sequence stance switch is automatic (+1 AP, no block): %s" % ("PASS" if auto_switch_ok else "FAIL"))
	print("  [Plan] Over-committed AP is projected (runs dry) not refused; Rest offered: %s" % ("PASS" if (runtime_plan_ok and options_ok) else "FAIL"))
	print("  [Plan] Reorder/remove edits are flagged when invalid and block Confirm: %s" % ("PASS" if edit_ok else "FAIL"))
	print("  [Plan] Confirm locks the sequence, schedules party + AI enemies, AP spent at runtime: %s" % ("PASS" if flow_ok2 else "FAIL"))
	print("  [Input] Per-target/tile options (foe attacks, self skills, move here, withdraw), previews, status & hover inspect: %s" % ("PASS" if input_ok else "FAIL"))
	print("  [Rules] 6-block rounds, AP pool + regen, one move per round, centre tie-break, target switching, insurance attacks, follow, damage log, auto-play on confirm: %s" % ("PASS" if rules_ok else "FAIL"))
	print("  [Mechanics] Ranged 2+ reach & off-hand jab, move-first, root, seeded status rolls & immunity, area friendly fire, allies moving into vacated tiles, split moves & cut-short refunds, ranged 2-3 reach, undo last step, AI pack avoids its own planned tiles: %s" % ("PASS" if mechanics_ok else "FAIL"))
	print("  [Runtime] Overspending goes into AP debt (next round: less AP, 3+ Stagger, 5+ Stun); Rest recovers AP mid-round: %s" % ("PASS" if runtime_ok else "FAIL"))
	print("  [AI] Presets draft valid, affordable sequences with their focus: %s" % ("PASS" if preset_ok else "FAIL"))
	print("  [AI] Aggressor engages, path-finds walls, Sniper backs away, Support focuses, wounded flee, attack closes a 1-tile gap at its start (not after a tile move; ranged backs off): %s" % ("PASS" if ai_ok else "FAIL"))
	print("  [AI] Preset skirmish is deterministic on every zone: %s" % ("PASS" if zones_ok else "FAIL"))
	print("  [AI] Both sides take damage in a preset-planned skirmish: %s" % ("PASS" if both_hurt else "FAIL"))
	print("  [Plan] Reset round rewinds units and AP to the round start and reopens planning: %s" % ("PASS" if reset_ok else "FAIL"))

	return ap_model_ok and stance_ok and seq_ok and auto_switch_ok and runtime_plan_ok and options_ok and edit_ok and flow_ok2 and input_ok and rules_ok and mechanics_ok and runtime_ok and preset_ok and ai_ok and zones_ok and both_hurt and reset_ok
