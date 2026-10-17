class_name AiSkillsTacticsTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - AI Skills: taunt, ally buffs, debuff attacks, riposte and 50 seeded 3v3 rounds
# (slice AI1; split from AiSkillsTestSuite by CLN-12). Fixtures and helpers: AiSkillsKit. Every
# expectation is derived from the rules, not from shipped skill numbers.
# ==============================================================================

const TAG: String = "AI1"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING AI SKILLS: TACTICS (AI1)               ===")
	print("=======================================================")
	var ok: bool = AiSkillsKit.setup(false)
	if ok:
		ok = _test_taunt() and ok
		ok = _test_buff() and ok
		ok = _test_debuff() and ok
		ok = _test_riposte() and ok
		ok = _test_simulation() and ok
	AiSkillsKit.teardown()
	return ok

static func _test_taunt() -> bool:
	var checks: Dictionary = {}
	var ally_cells: Array[Vector2i] = [Vector2i(5, 5)]
	var foe_cells: Array[Vector2i] = [Vector2i(5, 4)]
	var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_taunt"], foe_cells, ally_cells)
	var steps: Array[PlanStep] = AiSkillsKit.plan_steps(s)
	checks["an ally next to a foe: one taunt step"] = AiSkillsKit.count_of(steps, "ai1_taunt") == 1
	checks["the strike still follows"] = AiSkillsKit.count_of(steps, AiSkillsKit.STRIKE) >= 1 and AiSkillsKit.valid(s, steps)
	var active: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_taunt"], foe_cells, ally_cells)
	var active_actor: Combatant = active["actor"]
	active_actor.statuses[StatusEffects.TAUNT] = true
	checks["already taunting (active): no taunt step"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(active), "ai1_taunt") == 0
	var pending: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_taunt"], foe_cells, ally_cells)
	var pending_actor: Combatant = pending["actor"]
	pending_actor.pending_statuses[StatusEffects.TAUNT] = true
	checks["already taunting (pending): no taunt step"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(pending), "ai1_taunt") == 0
	var distant: Array[Vector2i] = [Vector2i(0, 0)]
	var safe: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_taunt"], foe_cells, distant)
	checks["no ally near a foe: no taunt step"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(safe), "ai1_taunt") == 0
	return TestHelpers.report(TAG, "Rule 5: taunt", checks)

static func _test_buff() -> bool:
	var checks: Dictionary = {}
	var foe_cells: Array[Vector2i] = [Vector2i(8, 4)]
	var ally_cells: Array[Vector2i] = [Vector2i(5, 7), Vector2i(6, 6), Vector2i(6, 7)]
	var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(6, 4), ["ai1_haste"], foe_cells, ally_cells, 3)
	var hasted: Combatant = AiSkillsKit.unit_of(s, "allies", 1)   # the farthest from the foe, but already hasted
	var near_one: Combatant = AiSkillsKit.unit_of(s, "allies", 2)
	var far_one: Combatant = AiSkillsKit.unit_of(s, "allies", 3)
	hasted.statuses[StatusEffects.HASTE] = true
	var steps: Array[PlanStep] = AiSkillsKit.plan_steps(s)
	var cast: int = AiSkillsKit.index_of(steps, "ai1_haste")
	checks["two allies, one hasted already: the cast targets the other, farthest one"] = cast >= 0 and steps[cast].target == far_one and steps[cast].target != near_one
	checks["a strike follows and the plan is valid"] = AiSkillsKit.count_of(steps, AiSkillsKit.STRIKE) >= 1 and AiSkillsKit.valid(s, steps)
	var all_done: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(6, 4), ["ai1_haste"], foe_cells, ally_cells, 3)
	for i: int in range(1, 4):
		AiSkillsKit.unit_of(all_done, "allies", i).statuses[StatusEffects.HASTE] = true
	checks["every ally already hasted: no step"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(all_done), "ai1_haste") == 0
	var pending: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(6, 4), ["ai1_haste"], foe_cells, ally_cells, 3)
	for i: int in range(1, 4):
		AiSkillsKit.unit_of(pending, "allies", i).pending_statuses[StatusEffects.HASTE] = true
	checks["every ally already awaiting haste: no step"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(pending), "ai1_haste") == 0
	var close: Array[Vector2i] = [Vector2i(7, 4), Vector2i(7, 5)]
	var engaged: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(6, 4), ["ai1_haste"], foe_cells, close, 3)
	checks["allies already next to the foe: no step"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(engaged), "ai1_haste") == 0
	return TestHelpers.report(TAG, "Rule 6: ally buffs", checks)

static func _test_debuff() -> bool:
	var checks: Dictionary = {}
	var foe_cells: Array[Vector2i] = [Vector2i(5, 4), Vector2i(4, 5), Vector2i(8, 8)]
	var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], foe_cells)
	var strong: Combatant = AiSkillsKit.unit_of(s, "foes", 0)
	var weak: Combatant = AiSkillsKit.unit_of(s, "foes", 1)
	var unreachable: Combatant = AiSkillsKit.unit_of(s, "foes", 2)
	strong.power_override = 30
	weak.power_override = 10
	unreachable.power_override = 50
	var steps: Array[PlanStep] = AiSkillsKit.plan_steps(s)
	var cast: int = AiSkillsKit.index_of(steps, "ai1_intim")
	checks["the strongest foe in reach is the debuff target"] = cast >= 0 and steps[cast].target == strong
	var last_strike: int = -1
	for i: int in range(steps.size()):
		if steps[i].skill.id == AiSkillsKit.STRIKE:
			last_strike = i
	checks["the main strike still follows the debuff"] = cast >= 0 and last_strike > cast
	checks["the plan is valid"] = AiSkillsKit.valid(s, steps)
	var done: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], foe_cells)
	AiSkillsKit.unit_of(done, "foes", 0).power_override = 30
	AiSkillsKit.unit_of(done, "foes", 1).power_override = 10
	AiSkillsKit.unit_of(done, "foes", 2).power_override = 50
	AiSkillsKit.unit_of(done, "foes", 0).statuses[StatusEffects.WEAKEN] = true
	var done_steps: Array[PlanStep] = AiSkillsKit.plan_steps(done)
	var done_cast: int = AiSkillsKit.index_of(done_steps, "ai1_intim")
	checks["an already weakened foe is skipped"] = done_cast >= 0 and done_steps[done_cast].target == AiSkillsKit.unit_of(done, "foes", 1)
	var queued: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], foe_cells)
	AiSkillsKit.unit_of(queued, "foes", 0).power_override = 30
	AiSkillsKit.unit_of(queued, "foes", 1).power_override = 10
	AiSkillsKit.unit_of(queued, "foes", 0).pending_statuses[StatusEffects.WEAKEN] = true
	var queued_steps: Array[PlanStep] = AiSkillsKit.plan_steps(queued)
	var queued_cast: int = AiSkillsKit.index_of(queued_steps, "ai1_intim")
	checks["a foe awaiting weaken is skipped too"] = queued_cast >= 0 and queued_steps[queued_cast].target == AiSkillsKit.unit_of(queued, "foes", 1)
	var both: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], [Vector2i(5, 4)])
	AiSkillsKit.unit_of(both, "foes", 0).statuses[StatusEffects.WEAKEN] = true
	checks["every foe in reach weakened: no cast"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(both), "ai1_intim") == 0
	return TestHelpers.report(TAG, "Rule 7: debuff attacks", checks)

static func _first_defend(steps: Array[PlanStep]) -> String:
	for s: PlanStep in steps:
		if s.skill != null and s.skill.kind == SkillDef.Kind.DEFEND:
			return s.skill.id
	return ""

static func _test_riposte() -> bool:
	var checks: Dictionary = {}
	var kit: Array[String] = ["ai1_guard", "ai1_rip"]
	var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(5, 4)])
	var low: Combatant = s["actor"]
	low.hp = 100
	checks["low HP facing a melee foe: riposte rather than the plain guard"] = _first_defend(AiSkillsKit.plan_steps(s)) == "ai1_rip"
	var ranged: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(6, 4)], [], 1, 3)
	var low_ranged: Combatant = ranged["actor"]
	low_ranged.hp = 100
	checks["low HP facing a ranged foe: the plain guard"] = _first_defend(AiSkillsKit.plan_steps(ranged)) == "ai1_guard"
	var holding: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(5, 4)])
	var holding_actor: Combatant = holding["actor"]
	holding_actor.hp = 100
	holding_actor.statuses[StatusEffects.RIPOSTE] = true
	checks["already riposting: the plain guard"] = _first_defend(AiSkillsKit.plan_steps(holding)) == "ai1_guard"
	# Healthy: strike first, then close the round in a riposte stance (once; a later strike would cancel it).
	var healthy: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(5, 4)])
	var steps: Array[PlanStep] = AiSkillsKit.plan_steps(healthy)
	checks["healthy, melee foe adjacent: a riposte closes the round"] = steps.size() >= 2 and steps[steps.size() - 1].skill.id == "ai1_rip" and AiSkillsKit.count_of(steps, "ai1_rip") == 1
	checks["exactly one strike precedes it and the plan is valid"] = AiSkillsKit.count_of(steps, AiSkillsKit.STRIKE) == 1 and AiSkillsKit.valid(healthy, steps)
	var reckless: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(5, 4)])
	var reckless_profile: AiProfile = reckless["profile"]
	reckless_profile.reckless = true
	checks["a reckless unit never ripostes"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(reckless), "ai1_rip") == 0
	var ranged_foe: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(5, 4)], [], 1, 3)
	checks["a ranged foe next to it: no riposte"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(ranged_foe), "ai1_rip") == 0
	var already: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(5, 4)])
	var already_actor: Combatant = already["actor"]
	already_actor.statuses[StatusEffects.RIPOSTE] = true
	checks["already riposting: no second cast"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(already), "ai1_rip") == 0
	var out_of_reach: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(0, 0), ["ai1_rip"], [Vector2i(8, 8)])
	checks["no strike this round and no foe adjacent: no riposte"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(out_of_reach), "ai1_rip") == 0
	var charging: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(7, 4)])
	var charge_steps: Array[PlanStep] = AiSkillsKit.plan_steps(charging)
	checks["a unit that walks up and strikes ends next to the melee foe: riposte closes the round"] = AiSkillsKit.count_of(charge_steps, "ai1_rip") == 1 and charge_steps[charge_steps.size() - 1].skill.id == "ai1_rip"
	var all_out: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(5, 4)])
	var all_out_profile: AiProfile = all_out["profile"]
	all_out_profile.defend_below = 0.0
	checks["defend_below 0 (never guards, the all-out preset): no riposte"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(all_out), "ai1_rip") == 0
	return TestHelpers.report(TAG, "Rule 8: riposte", checks)

# ------------------------------------------------------------------------------
# 50 seeded rounds of a 3v3 with mixed kits
# ------------------------------------------------------------------------------

static func _roster(grid: CombatGrid, rng: RandomNumberGenerator, side: int) -> Array[Combatant]:
	var out: Array[Combatant] = []
	var kits: Array = [
		["ai1_sweep", "ai1_taunt", "ai1_rip", "ai1_guard", "ai1_intim"],
		["ai1_back", "ai1_burst", "ai1_guard"],
		["ai1_heal", "ai1_clean_a", "ai1_clean_b", "ai1_haste", "ai1_back", "ai1_guard"],
	]
	for i: int in range(3):
		var cell := Vector2i.ZERO
		for attempt: int in range(60):
			cell = Vector2i(rng.randi_range(0, 3) + side * 5, rng.randi_range(0, 8))
			if not grid.is_occupied(cell):
				break
		var kit: Array[String] = []
		for id: Variant in DictUtil.array_of(kits[i]):
			kit.append(str(id))
		out.append(AiSkillsKit.unit("S%dU%d" % [side, i], grid, cell, kit, 1 if i == 0 else 3, 240))
	return out

static func _play(seed_value: int) -> String:
	var rng := RandomNumberGenerator.new()
	rng.seed = seed_value
	var log_sig: Array[String] = []
	var invalid: int = 0
	var oversize: int = 0
	var grid: CombatGrid = null
	var sides: Array[Array] = []
	var ctx: PlanContext = null
	for round_number: int in range(50):
		var alive_a: int = 0
		var alive_b: int = 0
		if grid != null:
			for c: Combatant in sides[0]:
				alive_a += 1 if c.is_active() else 0
			for c: Combatant in sides[1]:
				alive_b += 1 if c.is_active() else 0
		if grid == null or alive_a == 0 or alive_b == 0:
			grid = CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
			var team_a: Array[Combatant] = _roster(grid, rng, 0)
			var team_b: Array[Combatant] = _roster(grid, rng, 1)
			sides = [team_a, team_b]
			ctx = TestHelpers.hand_ctx(grid, team_a, team_b, 3)
			var roles: Array[AiProfile.Role] = [AiProfile.Role.AGGRESSOR, AiProfile.Role.SNIPER, AiProfile.Role.SUPPORT]
			for team: Array in sides:
				for i: int in range(3):
					var member: Combatant = team[i]
					ctx.profiles[member] = AiSkillsKit.ai_profile(roles[i])
		for team: Array in sides:
			for member_raw: Variant in team:
				var member: Combatant = member_raw
				if not member.is_active() or not grid.has_combatant(member):
					continue
				member.ap = member.max_ap
				var profile: AiProfile = ctx.profile_of(member)
				var first: Array[PlanStep] = TacticalAI.build_steps(member, ctx, profile)
				var again: Array[PlanStep] = TacticalAI.build_steps(member, ctx, profile)
				if AiSkillsKit.sig(first) != AiSkillsKit.sig(again):
					invalid += 1   # same state must give the same plan
				var check := CombatPlan.new(ctx)
				check.set_steps(member, first)
				var st: CombatPlan.SimState = check.compile(member)
				if st.error_index >= 0 or st.runs_dry():
					invalid += 1
				if first.size() > TacticalAI.MAX_STEPS:
					oversize += 1
				log_sig.append(AiSkillsKit.sig(first))
				TestHelpers.run_steps(member, ctx, first)
		for team: Array in sides:
			for member_raw: Variant in team:
				var member: Combatant = member_raw
				if member.is_active():
					StatusEffects.roll_over(member, ctx.total_blocks)
	return "%d|%d|%s" % [invalid, oversize, str(log_sig.hash())]

static func _test_simulation() -> bool:
	var checks: Dictionary = {}
	var first: String = _play(4242)
	var second: String = _play(4242)
	var parts: PackedStringArray = first.split("|")
	checks["50 seeded rounds of a 3v3: every planned step validates (none invalid, none dry)"] = parts[0] == "0"
	checks["MAX_STEPS is respected"] = parts[1] == "0"
	checks["the same seed gives the same plans end to end"] = first == second
	return TestHelpers.report(TAG, "Seeded 3v3 simulation: valid, bounded and deterministic", checks)
