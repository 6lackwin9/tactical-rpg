---
class: CombatExtAiTestSuite
extends: RefCounted
description: "Tactical RPG - Combat Extensions: AI fixes (S4-A, milestone M1; split from CombatExtensionsTestSuite by CLN-12): taunt targeting, kiting, no idle turns, support heals, reckless profile. Helpers in CombatExtKit; expectations come from the rules."
---

# Properties
* `TAG`: String = "CombatExt" (const)

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING COMBAT EXTENSIONS: AI (S4-A)           ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
var ok: bool = true
ok = _test_taunt_ai() and ok
ok = _test_kiting_and_idle() and ok
ok = _test_support_ai() and ok
ok = _test_reckless() and ok
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
return ok
```

### `static _first_engage_target(steps: Array[PlanStep]) -> Combatant`
```logic
for s: PlanStep in steps:
	if s.mode == PlanStep.Mode.ENGAGE:
		return s.target
return null
```

### `static _test_taunt_ai() -> bool`
```logic
var checks: Dictionary = {}
var grid: CombatGrid = CombatExtKit.new_grid()
var near: Combatant = CombatExtKit.unit("Near", 100, 10, 1, grid, Vector2i(4, 4))
var tank: Combatant = CombatExtKit.unit("Tank", 100, 10, 1, grid, Vector2i(6, 8))
var brute: Combatant = CombatExtKit.unit("Brute", 100, 10, 1, grid, Vector2i(6, 4))
var ctx: PlanContext = CombatExtKit.new_ctx(grid, [near, tank], [brute])
var profile: AiProfile = CombatExtKit.profile(AiProfile.Role.AGGRESSOR)
checks["baseline: the AI picks the nearest foe"] = _first_engage_target(TacticalAI.build_steps(brute, ctx, profile)) == near
tank.statuses[StatusEffects.TAUNT] = true
checks["taunted: the AI is forced onto the reachable taunter"] = _first_engage_target(TacticalAI.build_steps(brute, ctx, profile)) == tank
var weakest: AiProfile = CombatExtKit.profile(AiProfile.Role.SKIRMISHER)
weakest.target_pref = AiProfile.TargetPref.WEAKEST
near.hp = 10
checks["even a weakest-first profile is forced"] = _first_engage_target(TacticalAI.build_steps(brute, ctx, weakest)) == tank
checks["a reckless profile is forced too"] = (func() -> bool:
	var wild: AiProfile = CombatExtKit.profile(AiProfile.Role.AGGRESSOR)
	wild.reckless = true
	return _first_engage_target(TacticalAI.build_steps(brute, ctx, wild)) == tank).call()
near.hp = near.max_hp

# Not reachable: walled in, the taunt does not apply.
var walled: CombatGrid = CombatExtKit.new_grid()
var n2: Combatant = CombatExtKit.unit("Near2", 100, 10, 1, walled, Vector2i(4, 4))
var t2: Combatant = CombatExtKit.unit("Tank2", 100, 10, 1, walled, Vector2i(8, 8))
var b2: Combatant = CombatExtKit.unit("Brute2", 100, 10, 1, walled, Vector2i(6, 4))
walled.set_terrain(Vector2i(7, 8), Terrain.Type.WALL)
walled.set_terrain(Vector2i(8, 7), Terrain.Type.WALL)
walled.set_terrain(Vector2i(9, 8), Terrain.Type.WALL)
t2.statuses[StatusEffects.TAUNT] = true
var ctx2: PlanContext = CombatExtKit.new_ctx(walled, [n2, t2], [b2])
checks["an unreachable taunter does not force the AI"] = _first_engage_target(TacticalAI.build_steps(b2, ctx2, CombatExtKit.profile(AiProfile.Role.AGGRESSOR))) == n2

# The player's planner is never forced.
var pgrid: CombatGrid = CombatExtKit.new_grid()
var hero: Combatant = CombatExtKit.unit("Hero", 100, 10, 1, pgrid, Vector2i(2, 4))
var taunter: Combatant = CombatExtKit.unit("Taunter", 100, 10, 1, pgrid, Vector2i(6, 6))
var side: Combatant = CombatExtKit.unit("Side", 100, 10, 1, pgrid, Vector2i(3, 4))
taunter.statuses[StatusEffects.TAUNT] = true
var pctx: PlanContext = CombatExtKit.new_ctx(pgrid, [hero], [taunter, side])
checks["the player's plan may ignore a taunter"] = CombatPlan.new(pctx).add_step(hero, PlanStep.engage(SkillCatalog.find("strike"), side)).is_empty()
return TestHelpers.report(TAG, "Taunt steers the enemy AI only", checks)
```

### `static _test_kiting_and_idle() -> bool`
```logic
var checks: Dictionary = {}
var strike: SkillDef = SkillCatalog.find("strike")
var focus: SkillDef = SkillCatalog.find("focus")

# Ranged unit with a foe in its face: step back to maximum range, then shoot.
var grid: CombatGrid = CombatExtKit.new_grid()
var archer: Combatant = CombatExtKit.unit("Archer", 100, 10, 3, grid, Vector2i(4, 4))
var brute: Combatant = CombatExtKit.unit("Brute", 100, 10, 1, grid, Vector2i(5, 4))
var ctx: PlanContext = CombatExtKit.new_ctx(grid, [archer], [brute])
var band: Vector2i = strike.reach_for(archer)
var steps: Array[PlanStep] = TacticalAI.build_steps(archer, ctx, CombatExtKit.profile(AiProfile.Role.SNIPER))
var plan := CombatPlan.new(ctx)
plan.set_steps(archer, steps)
var st: CombatPlan.SimState = plan.compile(archer)
var struck: bool = false
for e: PlanEntry in st.entries:
	struck = struck or (e.skill.kind == SkillDef.Kind.ATTACK and e.target == brute)
checks["kites: a plain move first, then a strike"] = not steps.is_empty() and (steps[0].mode == PlanStep.Mode.GOTO or steps[0].mode == PlanStep.Mode.MOVE) and struck
checks["the plan ends at maximum range"] = grid.get_distance(st.pos, grid.get_position(brute)) == band.y
var played: CombatExtKit.Played = CombatExtKit.play(ctx, {archer: steps})
checks["resolved: shot lands from maximum range"] = brute.hp < brute.max_hp and grid.get_distance(grid.get_position(archer), grid.get_position(brute)) == band.y and played.messages.size() > 0

# Cornered: can't kite, but still attacks (off-hand jab) instead of idling.
var corner: CombatGrid = CombatExtKit.new_grid()
var trapped: Combatant = CombatExtKit.unit("Trapped", 100, 10, 3, corner, Vector2i(0, 0))
var pest: Combatant = CombatExtKit.unit("Pest", 100, 10, 1, corner, Vector2i(1, 0))
var cctx: PlanContext = CombatExtKit.new_ctx(corner, [trapped], [pest])
var corner_steps: Array[PlanStep] = TacticalAI.build_steps(trapped, cctx, CombatExtKit.profile(AiProfile.Role.SNIPER))
checks["cornered ranged unit still acts"] = CombatExtKit.any_mode(corner_steps, PlanStep.Mode.ENGAGE)

# A reachable target that is too far this round: advance instead of idling.
var field: CombatGrid = CombatExtKit.new_grid()
var walker: Combatant = CombatExtKit.unit("Walker", 100, 3, 1, field, Vector2i(0, 4))
var distant: Combatant = CombatExtKit.unit("Distant", 100, 10, 1, field, Vector2i(8, 4))
var fctx: PlanContext = CombatExtKit.new_ctx(field, [walker], [distant])
var fplan := CombatPlan.new(fctx)
var walk_steps: Array[PlanStep] = TacticalAI.build_steps(walker, fctx, CombatExtKit.profile(AiProfile.Role.AGGRESSOR))
fplan.set_steps(walker, walk_steps)
var wst: CombatPlan.SimState = fplan.compile(walker)
var start_dist: int = field.get_distance(Vector2i(0, 4), Vector2i(8, 4))
checks["no idle turn: out-of-range target is approached"] = not walk_steps.is_empty() and field.get_distance(wst.pos, Vector2i(8, 4)) == start_dist - fplan.max_move_tiles(walker)
CombatExtKit.play(fctx, {walker: walk_steps})
checks["resolved: the unit really closed in"] = field.get_distance(field.get_position(walker), Vector2i(8, 4)) < start_dist

# Buff first only when it does not waste the turn.
var cgrid: CombatGrid = CombatExtKit.new_grid()
var caster: Combatant = CombatExtKit.unit("Caster", 100, 10, 3, cgrid, Vector2i(2, 4), ["focus"])
var target: Combatant = CombatExtKit.unit("Target", 100, 10, 1, cgrid, Vector2i(5, 4))
var cctx2: PlanContext = CombatExtKit.new_ctx(cgrid, [caster], [target])
var in_band: Array[PlanStep] = TacticalAI.build_steps(caster, cctx2, CombatExtKit.profile(AiProfile.Role.SNIPER))
checks["in reach: casts the buff, then attacks"] = in_band.size() >= 2 and in_band[0].skill != null and in_band[0].skill.kind == SkillDef.Kind.FOCUS and CombatExtKit.any_mode(in_band, PlanStep.Mode.ENGAGE)
var cost_both: int = focus.ap_cost(caster) + focus.switch_cost(CombatTypes.StanceType.NEUTRAL) + strike.ap_cost(caster) + strike.switch_cost(CombatTypes.StanceType.TACTICAL)
caster.ap = cost_both - 1
var thrifty: Array[PlanStep] = TacticalAI.build_steps(caster, cctx2, CombatExtKit.profile(AiProfile.Role.SNIPER))
checks["not enough AP for both: skips the buff, still attacks"] = not CombatExtKit.any_skill_kind(thrifty, SkillDef.Kind.FOCUS) and CombatExtKit.any_mode(thrifty, PlanStep.Mode.ENGAGE)
caster.ap = caster.max_ap
var far_target: Combatant = CombatExtKit.unit("FarTarget", 100, 10, 1, cgrid, Vector2i(8, 4))
target.hp = 0
cgrid.remove_combatant(target)
var cctx3: PlanContext = CombatExtKit.new_ctx(cgrid, [caster], [far_target])
var must_walk: Array[PlanStep] = TacticalAI.build_steps(caster, cctx3, CombatExtKit.profile(AiProfile.Role.SNIPER))
var walk_plan := CombatPlan.new(cctx3)
walk_plan.set_steps(caster, must_walk)
var walk_state: CombatPlan.SimState = walk_plan.compile(caster)
var attacked: bool = false
for e: PlanEntry in walk_state.entries:
	attacked = attacked or e.skill.kind == SkillDef.Kind.ATTACK
checks["target out of reach: no buff before moving, the turn is not wasted"] = not CombatExtKit.any_skill_kind(must_walk, SkillDef.Kind.FOCUS) and attacked
return TestHelpers.report(TAG, "Kiting, no idle turns, buff only when free", checks)
```

### `static _test_support_ai() -> bool`
```logic
var checks: Dictionary = {}
var mend: SkillDef = SkillCatalog.find("mend")
var grid: CombatGrid = CombatExtKit.new_grid()
var medic: Combatant = CombatExtKit.unit("Medic", 100, 10, 1, grid, Vector2i(2, 4), ["mend"])
var hurt: Combatant = CombatExtKit.unit("Hurt", 100, 10, 1, grid, Vector2i(3, 4))
var scratched: Combatant = CombatExtKit.unit("Scratched", 100, 10, 1, grid, Vector2i(3, 5))
var fine: Combatant = CombatExtKit.unit("Fine", 100, 10, 1, grid, Vector2i(3, 3))
var foe: Combatant = CombatExtKit.unit("Foe", 100, 10, 1, grid, Vector2i(8, 4))
hurt.hp = 30
scratched.hp = 60
var allies: Array[Combatant] = [medic, hurt, scratched, fine]
var ctx: PlanContext = CombatExtKit.new_ctx(grid, allies, [foe])
var support: AiProfile = CombatExtKit.profile(AiProfile.Role.SUPPORT)
var steps: Array[PlanStep] = TacticalAI.build_steps(medic, ctx, support)
var patients: Array[Combatant] = []
for s: PlanStep in steps:
	if s.mode == PlanStep.Mode.ALLY:
		patients.append(s.target)
checks["heals the most wounded ally first, then the next below the threshold"] = patients.size() >= 2 and patients[0] == hurt and patients[1] == scratched
checks["never heals the healthy or the same ally twice"] = not patients.has(fine) and patients.size() == 2
var hp_before: int = hurt.hp
CombatExtKit.play(ctx, {medic: steps})
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
var near_foe: Combatant = CombatExtKit.unit("NearFoe", 100, 10, 1, grid, Vector2i(1, 4))
var ctx2: PlanContext = CombatExtKit.new_ctx(grid, allies, [near_foe])
var fight: Array[PlanStep] = TacticalAI.build_steps(medic, ctx2, CombatExtKit.profile(AiProfile.Role.SUPPORT))
checks["no wounded ally: attacks"] = CombatExtKit.any_mode(fight, PlanStep.Mode.ENGAGE) and not CombatExtKit.any_mode(fight, PlanStep.Mode.ALLY)

# Heals itself when alone and hurt (ALLY_OR_SELF).
var solo_grid: CombatGrid = CombatExtKit.new_grid()
var solo: Combatant = CombatExtKit.unit("Solo", 100, 10, 1, solo_grid, Vector2i(2, 4), ["mend"])
var distant_foe: Combatant = CombatExtKit.unit("Distant", 100, 10, 1, solo_grid, Vector2i(8, 8))
solo.hp = 20
var self_steps: Array[PlanStep] = TacticalAI.build_steps(solo, CombatExtKit.new_ctx(solo_grid, [solo], [distant_foe]), CombatExtKit.profile(AiProfile.Role.SUPPORT))
checks["heals itself when it is the most wounded"] = not self_steps.is_empty() and self_steps[0].mode == PlanStep.Mode.SELF and self_steps[0].skill == mend

# A support unit without a heal skill just fights.
var plain_grid: CombatGrid = CombatExtKit.new_grid()
var plain: Combatant = CombatExtKit.unit("Plain", 100, 10, 1, plain_grid, Vector2i(2, 4))
var friend: Combatant = CombatExtKit.unit("Friend", 100, 10, 1, plain_grid, Vector2i(2, 5))
var enemy: Combatant = CombatExtKit.unit("Enemy", 100, 10, 1, plain_grid, Vector2i(3, 4))
friend.hp = 10
var plain_steps: Array[PlanStep] = TacticalAI.build_steps(plain, CombatExtKit.new_ctx(plain_grid, [plain, friend], [enemy]), CombatExtKit.profile(AiProfile.Role.SUPPORT))
checks["no heal skill: attacks"] = CombatExtKit.any_mode(plain_steps, PlanStep.Mode.ENGAGE) and not CombatExtKit.any_mode(plain_steps, PlanStep.Mode.ALLY)
return TestHelpers.report(TAG, "Support AI heals the most wounded ally", checks)
```

### `static _test_reckless() -> bool`
```logic
var checks: Dictionary = {}
var exits: Array[Vector2i] = [Vector2i(8, 4)]
var grid: CombatGrid = CombatExtKit.new_grid(exits)
var berserker: Combatant = CombatExtKit.unit("Berserker", 100, 10, 1, grid, Vector2i(4, 4))
var near: Combatant = CombatExtKit.unit("Near", 100, 10, 1, grid, Vector2i(6, 4))
var weak: Combatant = CombatExtKit.unit("Weak", 100, 10, 1, grid, Vector2i(1, 6))
weak.hp = 10
berserker.hp = 10
var ctx: PlanContext = CombatExtKit.new_ctx(grid, [berserker], [near, weak])

var careful: AiProfile = CombatExtKit.profile(AiProfile.Role.AGGRESSOR, 0.5)
careful.defend_below = 0.9
var careful_steps: Array[PlanStep] = TacticalAI.build_steps(berserker, ctx, careful)
checks["baseline: a hurt, non-reckless unit flees"] = CombatExtKit.any_mode(careful_steps, PlanStep.Mode.WITHDRAW)

var wild: AiProfile = careful.duplicate_profile()
wild.reckless = true
checks["duplicate keeps the flag"] = wild.reckless
var wild_steps: Array[PlanStep] = TacticalAI.build_steps(berserker, ctx, wild)
checks["reckless never flees or guards"] = not CombatExtKit.any_mode(wild_steps, PlanStep.Mode.WITHDRAW) and not CombatExtKit.any_skill_kind(wild_steps, SkillDef.Kind.DEFEND)
checks["reckless still fights"] = CombatExtKit.any_mode(wild_steps, PlanStep.Mode.ENGAGE)

var picky: AiProfile = CombatExtKit.profile(AiProfile.Role.SKIRMISHER)
picky.target_pref = AiProfile.TargetPref.WEAKEST
berserker.hp = berserker.max_hp
var plain_target: Combatant = _first_engage_target(TacticalAI.build_steps(berserker, ctx, picky))
picky.reckless = true
var reckless_target: Combatant = _first_engage_target(TacticalAI.build_steps(berserker, ctx, picky))
checks["reckless charges the nearest even when told to hunt the weakest"] = plain_target == weak and reckless_target == near and grid.get_distance(Vector2i(4, 4), Vector2i(6, 4)) < grid.get_distance(Vector2i(4, 4), Vector2i(1, 6))

var entry_profile: AiProfile = ContentFactory.ai_profile({"class_id": "brawler", "level": 3, "reckless": true, "flee_ratio": 0.0}, berserker)
var other_profile: AiProfile = ContentFactory.ai_profile({"class_id": "brawler", "level": 3}, berserker)
checks["the unit's AI field sets the profile"] = entry_profile.reckless and not other_profile.reckless
return TestHelpers.report(TAG, "Reckless profile", checks)
```

