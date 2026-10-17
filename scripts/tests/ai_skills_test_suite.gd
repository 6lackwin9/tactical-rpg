class_name AiSkillsTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - AI Skills Test Suite (slice AI1)
# TacticalAI uses disengage, area attacks, cleanse-only, taunt, ally buffs, debuff attacks and
# riposte, and AiProfile.smart_skills = false restores the pre-AI1 behaviour. Every skill used
# here is an "ai1_*" row of a sandbox catalog fork (the shipped catalog is never touched) and
# every expectation is derived from the rules, not from shipped skill numbers. Taunt, buffs, debuffs,
# riposte and the seeded simulation: AiSkillsTacticsTestSuite; fixtures and helpers: AiSkillsKit (CLN-12).
# ==============================================================================

const TAG: String = "AI1"

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING AI SKILLS TEST (AI1)                   ===")
	print("=======================================================")
	var ok: bool = AiSkillsKit.setup(true)
	if ok:
		ok = _test_switch_off() and ok
		ok = _test_zero_power_never_the_strike() and ok
		ok = _test_disengage() and ok
		ok = _test_area() and ok
		ok = _test_cleanse() and ok
	AiSkillsKit.teardown()
	return ok

# ------------------------------------------------------------------------------
# Tests
# ------------------------------------------------------------------------------

static func _inert_kit() -> Array[String]:
	return ["ai1_heal", "ai1_guard"]

static func _rich_kit() -> Array[String]:
	return ["ai1_heal", "ai1_guard", "ai1_back", "ai1_sweep", "ai1_burst", "ai1_clean_a", "ai1_clean_b", "ai1_taunt", "ai1_haste"]

# Six seeded scenarios, one per role pattern; returns the plan signature.
static func _scenario_sig(index: int, kit: Array[String], smart: bool) -> String:
	var scene: Dictionary
	match index:
		0:
			scene = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(5, 4), Vector2i(4, 5)], [Vector2i(1, 1)], 1, 1, smart)
		1:
			scene = AiSkillsKit.new_scene(AiProfile.Role.SKIRMISHER, Vector2i(4, 4), kit, [Vector2i(5, 4)], [Vector2i(3, 4)], 1, 1, smart)
			var hurt: Combatant = scene["actor"]
			hurt.hp = 120
		2:
			scene = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(4, 4), kit, [Vector2i(5, 4), Vector2i(7, 7)], [], 3, 1, smart)
		3:
			scene = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, [Vector2i(6, 4)], [Vector2i(3, 4), Vector2i(4, 3)], 3, 1, smart)
			var patient: Combatant = AiSkillsKit.unit_of(scene, "allies", 1)
			patient.hp = 100
			var stunned: Combatant = AiSkillsKit.unit_of(scene, "allies", 2)
			stunned.statuses[StatusEffects.STUN] = true   # inert without a cleanse skill the old code uses
		4:
			scene = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(7, 4)], [], 1, 1, smart)
			var rash: AiProfile = scene["profile"]
			rash.reckless = true
			var rash_unit: Combatant = scene["actor"]
			rash_unit.hp = 80
		_:
			scene = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(2, 2), kit, [Vector2i(3, 2), Vector2i(2, 3), Vector2i(3, 3)], [Vector2i(0, 1)], 1, 1, smart)
	return AiSkillsKit.sig(AiSkillsKit.plan_steps(scene))

static func _test_switch_off() -> bool:
	var checks: Dictionary = {}
	for i: int in range(6):
		var bare_old: String = _scenario_sig(i, _inert_kit(), false)
		var bare_new: String = _scenario_sig(i, _inert_kit(), true)
		var rich_old: String = _scenario_sig(i, _rich_kit(), false)
		checks["scenario %d: a unit without the new skills plans the same under both switch values" % i] = bare_old == bare_new and not bare_old.is_empty()
		checks["scenario %d: smart_skills = false ignores the new skills (equals the plain kit)" % i] = rich_old == bare_old and not rich_old.contains("ai1_back") and not rich_old.contains("ai1_sweep") and not rich_old.contains("ai1_burst")
	# The switch really changes something when the kit has a use for it.
	var on: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], [Vector2i(5, 4), Vector2i(4, 5)])
	var off: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], [Vector2i(5, 4), Vector2i(4, 5)], [], 1, 1, false)
	checks["the same kit plans differently with the switch on and off"] = AiSkillsKit.sig(AiSkillsKit.plan_steps(on)) != AiSkillsKit.sig(AiSkillsKit.plan_steps(off))
	return TestHelpers.report(TAG, "smart_skills = false equals the old behaviour", checks)

static func _test_zero_power_never_the_strike() -> bool:
	var checks: Dictionary = {}
	var both: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], [Vector2i(5, 4)])
	var steps: Array[PlanStep] = AiSkillsKit.plan_steps(both)
	var damage_steps: int = 0
	var strike_ok: bool = false
	for s: PlanStep in steps:
		if s.mode == PlanStep.Mode.ENGAGE and s.skill.deals_damage():
			damage_steps += 1
			strike_ok = strike_ok or s.skill.id == AiSkillsKit.STRIKE
	checks["with intimidate (highest level) and a normal strike, the damage dealer is the normal strike"] = damage_steps >= 1 and strike_ok
	checks["the zero-power attack is only ever a debuff cast, never counted as damage"] = AiSkillsKit.count_of(steps, "ai1_intim") <= 1
	checks["the plan is valid"] = AiSkillsKit.valid(both, steps)
	var old: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], [Vector2i(5, 4)], [], 1, 1, false)
	var old_steps: Array[PlanStep] = AiSkillsKit.plan_steps(old)
	checks["pre-AI1 (switch off) still picks it as the main attack (documents the old behaviour)"] = not old_steps.is_empty() and old_steps[0].skill.id == "ai1_intim"
	# A unit that knows only the zero-power attack behaves like an unarmed idler.
	var lone: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(0, 4), [], [Vector2i(8, 4)])
	var lone_actor: Combatant = lone["actor"]
	lone_actor.known_skills = ["ai1_intim"]
	var lone_steps: Array[PlanStep] = AiSkillsKit.plan_steps(lone)
	var idle: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(0, 4), [], [Vector2i(8, 4)])
	var idle_actor: Combatant = idle["actor"]
	idle_actor.known_skills = ["ai1_guard"]
	var idle_steps: Array[PlanStep] = AiSkillsKit.plan_steps(idle)
	checks["a unit with only the zero-power attack never plans it"] = AiSkillsKit.count_of(lone_steps, "ai1_intim") == 0
	checks["it advances like an unarmed idler (identical plan)"] = not lone_steps.is_empty() and AiSkillsKit.sig(lone_steps) == AiSkillsKit.sig(idle_steps)
	return TestHelpers.report(TAG, "Rule 1: zero-power attacks are never the strike", checks)

static func _test_disengage() -> bool:
	var checks: Dictionary = {}
	var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)], [], 3)
	var steps: Array[PlanStep] = AiSkillsKit.plan_steps(s)
	var foe: Combatant = AiSkillsKit.unit_of(s, "foes", 0)
	var grid: CombatGrid = s["grid"]
	var start_dist: int = grid.get_distance(Vector2i(4, 4), grid.get_position(foe))
	checks["adjacent foe: the first step is the disengage skill"] = not steps.is_empty() and steps[0].skill.id == "ai1_back"
	checks["an attack step follows"] = steps.size() >= 2 and steps[1].mode == PlanStep.Mode.ENGAGE
	checks["the landing is farther from the foe than the start tile"] = not steps.is_empty() and grid.get_distance(steps[0].cell, grid.get_position(foe)) > start_dist
	checks["the plan is valid"] = AiSkillsKit.valid(s, steps)
	var support: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)], [], 3)
	var support_steps: Array[PlanStep] = AiSkillsKit.plan_steps(support)
	checks["the SUPPORT role disengages too"] = not support_steps.is_empty() and support_steps[0].skill.id == "ai1_back"
	# Rooted: no disengage, the old kite / strike path runs.
	var rooted: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)], [], 3)
	var rooted_actor: Combatant = rooted["actor"]
	rooted_actor.statuses[StatusEffects.ROOT] = true
	checks["rooted: no disengage step"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(rooted), "ai1_back") == 0
	# No skill: the plan equals the pre-AI1 plan.
	var bare_on: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(4, 4), [], [Vector2i(5, 4)], [], 3)
	var bare_off: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(4, 4), [], [Vector2i(5, 4)], [], 3, 1, false)
	checks["without the skill the plan is the old one"] = AiSkillsKit.sig(AiSkillsKit.plan_steps(bare_on)) == AiSkillsKit.sig(AiSkillsKit.plan_steps(bare_off))
	# No foe next to the unit: no disengage.
	var far: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_back"], [Vector2i(7, 4)], [], 3)
	checks["no adjacent foe: no disengage"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(far), "ai1_back") == 0
	# A hurt skirmisher with a foe next to it jumps instead of the retreat step.
	var skirmisher: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SKIRMISHER, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)])
	var skirmisher_actor: Combatant = skirmisher["actor"]
	skirmisher_actor.hp = 140
	var skirmisher_steps: Array[PlanStep] = AiSkillsKit.plan_steps(skirmisher)
	checks["a hurt skirmisher uses the disengage in place of the retreat move"] = not skirmisher_steps.is_empty() and skirmisher_steps[0].skill.id == "ai1_back" and AiSkillsKit.valid(skirmisher, skirmisher_steps)
	var healthy: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SKIRMISHER, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)])
	checks["a healthy skirmisher does not"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(healthy), "ai1_back") == 0
	var reckless: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)], [], 3)
	var reckless_profile: AiProfile = reckless["profile"]
	reckless_profile.reckless = true
	checks["a reckless unit never disengages"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(reckless), "ai1_back") == 0
	return TestHelpers.report(TAG, "Rule 2: disengage", checks)

static func _test_area() -> bool:
	var checks: Dictionary = {}
	var foes_both: Array[Vector2i] = [Vector2i(5, 4), Vector2i(4, 5)]
	var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], foes_both, [Vector2i(1, 1)])
	var steps: Array[PlanStep] = AiSkillsKit.plan_steps(s)
	var area_steps: Array[PlanStep] = []
	for step: PlanStep in steps:
		if step.mode == PlanStep.Mode.AREA:
			area_steps.append(step)
	checks["two foes in the sweep and no ally: the plan contains the area skill"] = not area_steps.is_empty() and area_steps[0].skill.id == "ai1_sweep"
	checks["it is aimed at the diagonal neighbour that hits both foes"] = not area_steps.is_empty() and area_steps[0].cell == Vector2i(5, 5)
	checks["the plan is valid"] = AiSkillsKit.valid(s, steps)
	# An ally inside the shape: the single-target strike is used instead.
	var allied: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], foes_both, [Vector2i(5, 5)])
	var allied_steps: Array[PlanStep] = AiSkillsKit.plan_steps(allied)
	checks["an ally in the shape: no area step"] = AiSkillsKit.count_of(allied_steps, "ai1_sweep") == 0
	checks["the single-target strike is used instead"] = AiSkillsKit.count_of(allied_steps, AiSkillsKit.STRIKE) >= 1
	# A single foe: strike, unless area_min_foes is 1.
	var single: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], [Vector2i(5, 4)])
	checks["a single foe: the single-target strike"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(single), "ai1_sweep") == 0
	var eager: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], [Vector2i(5, 4)])
	var eager_profile: AiProfile = eager["profile"]
	eager_profile.area_min_foes = 1
	checks["area_min_foes = 1: the single-foe case picks the area skill"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(eager), "ai1_sweep") >= 1
	var picky: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], foes_both)
	var picky_profile: AiProfile = picky["profile"]
	picky_profile.area_min_foes = 99
	checks["an out-of-range area_min_foes is clamped (6): two foes are not enough"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(picky), "ai1_sweep") == 0
	var off: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], foes_both, [], 1, 1, false)
	checks["switch off: no area step"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(off), "ai1_sweep") == 0
	# A tile-aimed burst for a ranged unit.
	var burst: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_burst"], [Vector2i(7, 4), Vector2i(7, 5)], [], 3)
	var burst_steps: Array[PlanStep] = AiSkillsKit.plan_steps(burst)
	checks["a ranged unit aims a burst at two clustered foes"] = AiSkillsKit.count_of(burst_steps, "ai1_burst") >= 1 and AiSkillsKit.valid(burst, burst_steps)
	var friendly: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_burst"], [Vector2i(7, 4), Vector2i(7, 5)], [Vector2i(8, 4)], 3)
	var friendly_steps: Array[PlanStep] = AiSkillsKit.plan_steps(friendly)
	var hit_ally: bool = false
	var friend_unit: Combatant = AiSkillsKit.unit_of(friendly, "allies", 1)
	var grid: CombatGrid = friendly["grid"]
	for step: PlanStep in friendly_steps:
		if step.mode == PlanStep.Mode.AREA:
			hit_ally = hit_ally or step.skill.area_cells(Vector2i(4, 4), step.cell).has(grid.get_position(friend_unit))
	checks["the AI never aims an area attack that would hit its own side"] = not hit_ally
	return TestHelpers.report(TAG, "Rule 3: area attacks", checks)

static func _test_cleanse() -> bool:
	var checks: Dictionary = {}
	var kit: Array[String] = ["ai1_heal", "ai1_clean_a", "ai1_clean_b"]
	var far_foe: Array[Vector2i] = [Vector2i(8, 8)]
	var cells: Array[Vector2i] = [Vector2i(3, 4), Vector2i(4, 3)]
	var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1)
	var stunned: Combatant = AiSkillsKit.unit_of(s, "allies", 1)
	var wounded: Combatant = AiSkillsKit.unit_of(s, "allies", 2)
	stunned.statuses[StatusEffects.STUN] = true
	wounded.hp = 100
	var steps: Array[PlanStep] = AiSkillsKit.plan_steps(s)
	var cleanse_at: int = AiSkillsKit.index_of(steps, "ai1_clean_a")
	var heal_at: int = AiSkillsKit.index_of(steps, "ai1_heal")
	checks["a stunned ally: a cleanse step targets it"] = cleanse_at >= 0 and steps[cleanse_at].target == stunned
	checks["the lowest tier that covers the stun is chosen"] = cleanse_at >= 0 and AiSkillsKit.count_of(steps, "ai1_clean_b") == 0
	checks["the cleanse comes before the heal step"] = cleanse_at >= 0 and heal_at > cleanse_at and steps[heal_at].target == wounded
	checks["the plan is valid"] = AiSkillsKit.valid(s, steps)
	# More debuffs than the first tier covers: the covering tier.
	var multi: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1)
	var multi_target: Combatant = AiSkillsKit.unit_of(multi, "allies", 1)
	multi_target.statuses[StatusEffects.STUN] = true
	multi_target.statuses[StatusEffects.BLEED] = true
	var multi_steps: Array[PlanStep] = AiSkillsKit.plan_steps(multi)
	checks["stun and bleed: the tier that covers both"] = AiSkillsKit.count_of(multi_steps, "ai1_clean_b") == 1 and AiSkillsKit.count_of(multi_steps, "ai1_clean_a") == 0
	# Nothing cleansable: no cleanse step.
	var clean: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1)
	var clean_steps: Array[PlanStep] = AiSkillsKit.plan_steps(clean)
	checks["nothing cleansable: no cleanse step"] = AiSkillsKit.count_of(clean_steps, "ai1_clean_a") + AiSkillsKit.count_of(clean_steps, "ai1_clean_b") == 0
	# Two stunned allies: still one cleanse per round.
	var twice: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1)
	AiSkillsKit.unit_of(twice, "allies", 1).statuses[StatusEffects.STUN] = true
	AiSkillsKit.unit_of(twice, "allies", 2).statuses[StatusEffects.STUN] = true
	var twice_steps: Array[PlanStep] = AiSkillsKit.plan_steps(twice)
	checks["two stunned allies: never two cleanses in a round"] = AiSkillsKit.count_of(twice_steps, "ai1_clean_a") + AiSkillsKit.count_of(twice_steps, "ai1_clean_b") == 1
	# A debuff that does not lock the ally down: cleansed after the heal step.
	var late: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1)
	AiSkillsKit.unit_of(late, "allies", 1).statuses[StatusEffects.BLEED] = true
	AiSkillsKit.unit_of(late, "allies", 2).hp = 100
	var late_steps: Array[PlanStep] = AiSkillsKit.plan_steps(late)
	var late_heal: int = AiSkillsKit.index_of(late_steps, "ai1_heal")
	checks["a non-locking debuff is cleansed after the heal"] = late_heal >= 0 and AiSkillsKit.index_of(late_steps, "ai1_clean_b") > late_heal
	# Any role can cleanse (not only the SUPPORT role).
	var fighter: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_clean_a"], far_foe, cells, 1)
	AiSkillsKit.unit_of(fighter, "allies", 1).statuses[StatusEffects.STUN] = true
	checks["an aggressor that owns a cleanse uses it on a stunned ally"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(fighter), "ai1_clean_a") == 1
	var off: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1, 1, false)
	AiSkillsKit.unit_of(off, "allies", 1).statuses[StatusEffects.STUN] = true
	var off_steps: Array[PlanStep] = AiSkillsKit.plan_steps(off)
	checks["switch off: no cleanse step"] = AiSkillsKit.count_of(off_steps, "ai1_clean_a") + AiSkillsKit.count_of(off_steps, "ai1_clean_b") == 0
	return TestHelpers.report(TAG, "Rule 4: cleanse-only skills", checks)
