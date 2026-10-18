---
class: AiSkillsTestSuite
extends: RefCounted
description: "Tactical RPG - AI Skills Test Suite (slice AI1) TacticalAI uses disengage, area attacks, cleanse-only, taunt, ally buffs, debuff attacks and riposte (the only AI path since CLN-16 removed the smart_skills switch). Every skill used here is an \"ai1_*\" row of a sandbox catalog fork (the shipped catalog is never touched) and every expectation is derived from the rules, not from shipped skill numbers. Taunt, buffs, debuffs, riposte and the seeded simulation: AiSkillsTacticsTestSuite; fixtures and helpers: AiSkillsKit (CLN-12). BAL3-AI levers B (Focus under pressure), C (faster tier when likely to die), the incoming-damage estimate and the minion skill ceiling are tested here; the heal reservation and taunt pull in AiSkillsTacticsTestSuite."
---

# Properties
* `TAG`: String = "AI1" (const)

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING AI SKILLS TEST (AI1)                   ===")
print("=======================================================")
var ok: bool = AiSkillsKit.setup(true)
if ok:
	ok = _test_zero_power_never_the_strike() and ok
	ok = _test_disengage() and ok
	ok = _test_area() and ok
	ok = _test_cleanse() and ok
	ok = _test_incoming() and ok
	ok = _test_focus_pressure() and ok
	ok = _test_fast_tier() and ok
	ok = _test_minion_ceiling() and ok
AiSkillsKit.teardown()
return ok
```

### `static _test_zero_power_never_the_strike() -> bool`
```logic
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
```

### `static _test_disengage() -> bool`
```logic
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
```

### `static _test_area() -> bool`
```logic
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
```

### `static _test_cleanse() -> bool`
```logic
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
return TestHelpers.report(TAG, "Rule 4: cleanse-only skills", checks)
```

### `static _test_incoming() -> bool`
```logic
var checks: Dictionary = {}
var strike: SkillDef = SkillCatalog.find(AiSkillsKit.STRIKE)
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), [], [Vector2i(5, 4)])
var actor: Combatant = s["actor"]
var foe: Combatant = AiSkillsKit.unit_of(s, "foes", 0)
var ctx: PlanContext = s["ctx"]
var plan := CombatPlan.new(ctx)
var blocks: int = ctx.total_blocks
var hits: int = mini(floori(float(blocks) / float(strike.blocks)), floori(float(foe.ap) / float(maxi(1, strike.ap_cost(foe)))))
var incoming: int = TacticalAI.incoming_damage(plan, actor, blocks, ctx.grid.get_position(actor))
checks["an adjacent foe: at least its strikes that fit the round"] = incoming >= hits * DamageCalculator.expected(foe, actor, strike) and incoming > 0
checks["block 0: nothing lands yet"] = TacticalAI.incoming_damage(plan, actor, 0, ctx.grid.get_position(actor)) == 0
var far: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(0, 4), [], [Vector2i(CombatGrid.GRID_W - 1, 4)])
var far_actor: Combatant = far["actor"]
var far_ctx: PlanContext = far["ctx"]
checks["a foe beyond one move plus its reach: 0"] = TacticalAI.incoming_damage(CombatPlan.new(far_ctx), far_actor, far_ctx.total_blocks, far_ctx.grid.get_position(far_actor)) == 0
var shielded: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(2, 4), [], [Vector2i(5, 4)], [Vector2i(4, 4)])
var shielded_actor: Combatant = shielded["actor"]
var shielded_ctx: PlanContext = shielded["ctx"]
var front: Combatant = AiSkillsKit.unit_of(shielded, "allies", 1)
var shielded_plan := CombatPlan.new(shielded_ctx)
checks["a nearest-first foe next to another friend: it picks that friend, not the unit"] = TacticalAI.incoming_damage(shielded_plan, shielded_actor, shielded_ctx.total_blocks, shielded_ctx.grid.get_position(shielded_actor)) == 0 and TacticalAI.incoming_damage(shielded_plan, front, shielded_ctx.total_blocks, shielded_ctx.grid.get_position(front)) > 0
return TestHelpers.report(TAG, "BAL3-AI: incoming damage estimate", checks)
```

### `static _focus_scene(foe_power: int) -> Dictionary`
```logic
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SNIPER, Vector2i(2, 4), ["ai1_focus"], [Vector2i(5, 4)], [], 3, 3)
AiSkillsKit.unit_of(s, "foes", 0).power_override = foe_power
return s
```

### `static _pre_cast_incoming(s: Dictionary) -> int`
```logic
var actor: Combatant = s["actor"]
var ctx: PlanContext = s["ctx"]
var cast_end: int = SkillCatalog.find("ai1_focus").blocks + SkillCatalog.find(AiSkillsKit.STRIKE).blocks
return TacticalAI.incoming_damage(CombatPlan.new(ctx), actor, cast_end, ctx.grid.get_position(actor))
```

### `static _test_focus_pressure() -> bool`
```logic
var checks: Dictionary = {}
var calm: Dictionary = _focus_scene(5)
var calm_actor: Combatant = calm["actor"]
checks["calm board: pre-cast incoming below the ratio"] = float(_pre_cast_incoming(calm)) < TacticalAI.FOCUS_PRESSURE_RATIO * float(calm_actor.hp)
var calm_steps: Array[PlanStep] = AiSkillsKit.plan_steps(calm)
checks["calm: Focus goes before the cast"] = AiSkillsKit.index_of(calm_steps, "ai1_focus") == 0 and AiSkillsKit.first_attack(calm, calm_steps) != null
# Pressure: the pre-cast hits reach the ratio, but the whole round stays below the HP (lever C is not involved).
var power: int = 5
var hot: Dictionary = _focus_scene(power)
var hot_actor: Combatant = hot["actor"]
while float(_pre_cast_incoming(hot)) < TacticalAI.FOCUS_PRESSURE_RATIO * float(hot_actor.hp) and power < 1000:
	power += 5
	hot = _focus_scene(power)
	hot_actor = hot["actor"]
var hot_ctx: PlanContext = hot["ctx"]
checks["pressure board: pre-cast incoming reaches the ratio"] = float(_pre_cast_incoming(hot)) >= TacticalAI.FOCUS_PRESSURE_RATIO * float(hot_actor.hp)
checks["(and the whole round stays below the HP)"] = TacticalAI.incoming_damage(CombatPlan.new(hot_ctx), hot_actor, hot_ctx.total_blocks, hot_ctx.grid.get_position(hot_actor)) < hot_actor.hp
var hot_steps: Array[PlanStep] = AiSkillsKit.plan_steps(hot)
checks["pressure: no Focus, the cast goes first"] = AiSkillsKit.count_of(hot_steps, "ai1_focus") == 0 and not hot_steps.is_empty() and hot_steps[0].mode == PlanStep.Mode.ENGAGE
checks["both plans are valid"] = AiSkillsKit.valid(calm, calm_steps) and AiSkillsKit.valid(hot, hot_steps)
return TestHelpers.report(TAG, "BAL3-AI lever B: skip Focus under pressure", checks)
```

### `static _fast_scene(foe_power: int) -> Dictionary`
```logic
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_heavy"], [Vector2i(5, 4)])
AiSkillsKit.unit_of(s, "foes", 0).power_override = foe_power
return s
```

### `static _round_incoming(s: Dictionary) -> int`
```logic
var actor: Combatant = s["actor"]
var ctx: PlanContext = s["ctx"]
return TacticalAI.incoming_damage(CombatPlan.new(ctx), actor, ctx.total_blocks, ctx.grid.get_position(actor))
```

### `static _test_fast_tier() -> bool`
```logic
var checks: Dictionary = {}
var calm: Dictionary = _fast_scene(5)
var calm_actor: Combatant = calm["actor"]
var calm_foe: Combatant = AiSkillsKit.unit_of(calm, "foes", 0)
var heavy: SkillDef = SkillCatalog.find("ai1_heavy")
var strike: SkillDef = SkillCatalog.find(AiSkillsKit.STRIKE)
checks["fixture: the slow tier hits harder and resolves after block 2"] = DamageCalculator.expected(calm_actor, calm_foe, heavy) > DamageCalculator.expected(calm_actor, calm_foe, strike) and heavy.blocks > TacticalAI.FAST_BLOCK and strike.blocks <= TacticalAI.FAST_BLOCK
checks["calm board: incoming below the HP"] = _round_incoming(calm) < calm_actor.hp
var calm_first: PlanEntry = AiSkillsKit.first_attack(calm, AiSkillsKit.plan_steps(calm))
checks["calm: the harder, slower tier first"] = calm_first != null and calm_first.skill.id == "ai1_heavy"
var power: int = 5
var hot: Dictionary = _fast_scene(power)
var hot_actor: Combatant = hot["actor"]
while _round_incoming(hot) < hot_actor.hp and power < 2000:
	power += 5
	hot = _fast_scene(power)
	hot_actor = hot["actor"]
checks["pressure board: incoming reaches the HP"] = _round_incoming(hot) >= hot_actor.hp
var hot_steps: Array[PlanStep] = AiSkillsKit.plan_steps(hot)
var hot_first: PlanEntry = AiSkillsKit.first_attack(hot, hot_steps)
checks["pressure: the first attack resolves by block 2 (the plain strike)"] = hot_first != null and hot_first.end_block() <= TacticalAI.FAST_BLOCK and hot_first.skill.id == AiSkillsKit.STRIKE
checks["the pressure plan is valid"] = AiSkillsKit.valid(hot, hot_steps)
# Nothing resolves by block 2 (the foe must be walked to first): the normal order applies.
var walk: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_heavy"], [Vector2i(7, 4)])
var walk_actor: Combatant = walk["actor"]
walk_actor.hp = 1
var walk_profile: AiProfile = walk["profile"]
walk_profile.flee_ratio = 0.0
walk_profile.defend_below = 0.0
var walk_first: PlanEntry = AiSkillsKit.first_attack(walk, AiSkillsKit.plan_steps(walk))
checks["likely to die but a move first: the normal (highest) tier"] = _round_incoming(walk) >= walk_actor.hp and walk_first != null and walk_first.skill.id == "ai1_heavy"
return TestHelpers.report(TAG, "BAL3-AI lever C: faster tier when likely to die", checks)
```

### `static _test_minion_ceiling() -> bool`
```logic
var checks: Dictionary = {}
var tier_cap: int = StatRules.tier_skill_cap("minion")
var expected_cap: int = tier_cap if tier_cap > 0 else AiProfile.NO_CEILING
checks["the minion tier default is the named constant"] = tier_cap == StatRules.MINION_SKILL_CAP
checks["a minion without skill_cap: the tier ceiling"] = ContentFactory.ai_profile({"class_id": "brawler", "level": 5, "tier": "minion"}, null).skill_cap == expected_cap
checks["an explicit skill_cap wins over the tier default"] = ContentFactory.ai_profile({"class_id": "brawler", "level": 5, "tier": "minion", "skill_cap": 3}, null).skill_cap == 3
var other_tiers: bool = true
for tier: String in StatRules.tier_names():
	if tier != "minion" and StatRules.tier_skill_cap(tier) == 0:
		other_tiers = other_tiers and ContentFactory.ai_profile({"class_id": "brawler", "level": 5, "tier": tier}, null).skill_cap == AiProfile.NO_CEILING
checks["tiers without a ceiling keep NO_CEILING"] = other_tiers
return TestHelpers.report(TAG, "BAL3 section 5: minion skill ceiling", checks)
```

