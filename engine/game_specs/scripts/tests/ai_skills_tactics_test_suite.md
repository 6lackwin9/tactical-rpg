---
class: AiSkillsTacticsTestSuite
extends: RefCounted
description: "Tactical RPG - AI Skills: taunt, ally buffs, debuff attacks, riposte and 50 seeded 3v3 rounds (slice AI1; split from AiSkillsTestSuite by CLN-12). Fixtures and helpers: AiSkillsKit. Every expectation is derived from the rules, not from shipped skill numbers. BAL3-AI: heal reservation and taunt pull (docs/specs/BAL3.md section 4). BAL5-AI: early taunt (T2), healer self-care (S1) and friendly-fire tolerance (A2) (docs/specs/BAL5_LEVERS.md sections 2-4)."
---

# Properties
* `TAG`: String = "AI1" (const)
* `actor`: Combatant = s["actor"]
* `ctx`: PlanContext = s["ctx"]
* `profile`: AiProfile = s["profile"]
* `ctx`: PlanContext = s["ctx"]
* `actor`: Combatant = s["actor"]
* `foes`: Array[Combatant] = s["foes"]
* `allies`: Array[Combatant] = s["allies"]
* `out`: Array[Vector2i] = []
* `hits`: Variant
* `cells`: Array[Vector2i] = step.skill.area_cells(ctx.grid.get_position(actor), step.cell)
* `out`: Array[Combatant] = []
* `kits`: Array = [
* `cell`: Variant
* `kit`: Array[String] = []

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING AI SKILLS: TACTICS (AI1)               ===")
print("=======================================================")
var ok: bool = AiSkillsKit.setup(false)
if ok:
	ok = _test_taunt() and ok
	ok = _test_buff() and ok
	ok = _test_debuff() and ok
	ok = _test_riposte() and ok
	ok = _test_heal_reservation() and ok
	ok = _test_taunt_pull() and ok
	ok = _test_early_taunt() and ok
	ok = _test_self_care() and ok
	ok = _test_friendly_fire() and ok
	ok = _test_simulation() and ok
AiSkillsKit.teardown()
return ok
```

### `static _test_taunt() -> bool`
```logic
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
```

### `static _test_buff() -> bool`
```logic
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
```

### `static _test_debuff() -> bool`
```logic
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
```

### `static _first_defend(steps: Array[PlanStep]) -> String`
```logic
for s: PlanStep in steps:
	if s.skill != null and s.skill.kind == SkillDef.Kind.DEFEND:
		return s.skill.id
return ""
```

### `static _test_riposte() -> bool`
```logic
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
```

### `static _reserve_scene(foe_power: int) -> Dictionary`
```logic
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(2, 4), ["ai1_heal"], [Vector2i(5, 4)], [Vector2i(4, 4)], 3)
AiSkillsKit.unit_of(s, "foes", 0).power_override = foe_power
return s
```

### `static _projected(s: Dictionary) -> float`
```logic
var ctx: PlanContext = s["ctx"]
var ally: Combatant = AiSkillsKit.unit_of(s, "allies", 1)
var incoming: int = TacticalAI.incoming_damage(CombatPlan.new(ctx), ally, ctx.total_blocks, ctx.grid.get_position(ally))
return float(ally.hp - incoming) / float(maxi(1, ally.max_hp))
```

### `static _test_heal_reservation() -> bool`
```logic
var checks: Dictionary = {}
var calm: Dictionary = _reserve_scene(5)
var calm_profile: AiProfile = calm["profile"]
var calm_ally: Combatant = AiSkillsKit.unit_of(calm, "allies", 1)
checks["calm board: the ally is at full HP and projected above heal_below"] = calm_ally.hp == calm_ally.max_hp and _projected(calm) > calm_profile.heal_below
var calm_steps: Array[PlanStep] = AiSkillsKit.plan_steps(calm)
checks["calm: no heal step"] = AiSkillsKit.count_of(calm_steps, "ai1_heal") == 0
var power: int = 5
var hot: Dictionary = _reserve_scene(power)
var hot_profile: AiProfile = hot["profile"]
while _projected(hot) > hot_profile.heal_below and power < 1000:
	power += 5
	hot = _reserve_scene(power)
	hot_profile = hot["profile"]
var hot_ally: Combatant = AiSkillsKit.unit_of(hot, "allies", 1)
checks["pressure board: full HP now, projected at or below heal_below by the round's end"] = hot_ally.hp == hot_ally.max_hp and _projected(hot) <= hot_profile.heal_below
var hot_steps: Array[PlanStep] = AiSkillsKit.plan_steps(hot)
var last: PlanStep = hot_steps[hot_steps.size() - 1] if not hot_steps.is_empty() else null
checks["pressure: one heal, kept last and aimed at the projected ally"] = AiSkillsKit.count_of(hot_steps, "ai1_heal") == 1 and last != null and last.skill.id == "ai1_heal" and last.target == hot_ally
checks["the reserved plan is valid and still fights"] = AiSkillsKit.valid(hot, hot_steps) and AiSkillsKit.first_attack(hot, hot_steps) != null
# The projected ally stands beyond the heal's reach from where the draft leaves the healer: no walk to the front.
var hot_actor: Combatant = hot["actor"]
var heal_reach: int = SkillCatalog.find("ai1_heal").support_reach(hot_actor).y
var beyond: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(2, 4), ["ai1_heal"], [Vector2i(5, 4)], [Vector2i(3 + heal_reach, 4)], 3)
AiSkillsKit.unit_of(beyond, "foes", 0).power_override = power
var beyond_steps: Array[PlanStep] = AiSkillsKit.plan_steps(beyond)
var walked: bool = false
for step: PlanStep in beyond_steps:
	walked = walked or step.mode == PlanStep.Mode.MOVE or step.mode == PlanStep.Mode.GOTO
checks["out of heal reach from the drafted spot: no reserved heal and no approach walk"] = _projected(beyond) <= hot_profile.heal_below and AiSkillsKit.count_of(beyond_steps, "ai1_heal") == 0 and not walked
var sniper: Dictionary = _reserve_scene(power)
var sniper_actor: Combatant = sniper["actor"]
var sniper_ctx: PlanContext = sniper["ctx"]
var sniper_profile: AiProfile = AiSkillsKit.ai_profile(AiProfile.Role.SNIPER)
checks["only the SUPPORT role reserves"] = AiSkillsKit.count_of(TacticalAI.build_steps(sniper_actor, sniper_ctx, sniper_profile), "ai1_heal") == 0
return TestHelpers.report(TAG, "BAL3-AI: heal reservation", checks)
```

### `static _pull_scene(tank_dist: int, taunting: bool) -> Dictionary`
```logic
var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
var actor: Combatant = AiSkillsKit.unit("Puller", grid, Vector2i(CombatGrid.GRID_W - 1 - tank_dist, 4), [])
var near: Combatant = AiSkillsKit.unit("Near", grid, Vector2i(CombatGrid.GRID_W - 2 - tank_dist, 4), [])
var tank: Combatant = AiSkillsKit.unit("Tank", grid, Vector2i(CombatGrid.GRID_W - 1, 4), [])
if taunting:
	tank.statuses[StatusEffects.TAUNT] = true
var ctx: PlanContext = TestHelpers.hand_ctx(grid, [actor], [near, tank], 3)
var profile: AiProfile = AiSkillsKit.ai_profile(AiProfile.Role.AGGRESSOR)
ctx.profiles[actor] = profile
var plan := CombatPlan.new(ctx)
var route: Array[Vector2i] = plan.approach_path(actor, grid.get_position(actor), tank)
actor.ap = route.size() + SkillCatalog.find(AiSkillsKit.STRIKE).ap_cost(actor) - 1
return {"grid": grid, "actor": actor, "near": near, "tank": tank, "ctx": ctx, "profile": profile, "route": route, "fits": not route.is_empty() and plan.fits_move(actor, grid.get_position(actor), route)}
```

### `static _strikes_on(s: Dictionary, unit: Combatant) -> bool`
```logic
var actor: Combatant = s["actor"]
var ctx: PlanContext = s["ctx"]
var profile: AiProfile = s["profile"]
for step: PlanStep in TacticalAI.build_steps(actor, ctx, profile):
	if step.mode == PlanStep.Mode.ENGAGE and step.target == unit:
		return true
return false
```

### `static _test_taunt_pull() -> bool`
```logic
var checks: Dictionary = {}
var probe: Dictionary = _pull_scene(2, false)
var probe_ctx: PlanContext = probe["ctx"]
var probe_actor: Combatant = probe["actor"]
var move: int = CombatPlan.new(probe_ctx).max_move_tiles(probe_actor)
var close: int = mini(move, CombatGrid.GRID_W - 3)   # route = close - 1 tiles: inside the one move
var pull: Dictionary = _pull_scene(close, true)
checks["fixture: the route to the taunter fits the move"] = DictUtil.to_bool(pull["fits"])
var near: Combatant = pull["near"]
var tank: Combatant = pull["tank"]
checks["route within the move: the taunter is the only target (no strike on Near)"] = not _strikes_on(pull, near)
var pull_actor: Combatant = pull["actor"]
var pull_ctx: PlanContext = pull["ctx"]
var pull_profile: AiProfile = pull["profile"]
var walk_plan := CombatPlan.new(pull_ctx)
walk_plan.set_steps(pull_actor, TacticalAI.build_steps(pull_actor, pull_ctx, pull_profile))
var end_pos: Vector2i = walk_plan.compile(pull_actor).pos
checks["(it walks toward the taunter instead)"] = pull_ctx.grid.get_distance(end_pos, pull_ctx.grid.get_position(tank)) < pull_ctx.grid.get_distance(pull_ctx.grid.get_position(pull_actor), pull_ctx.grid.get_position(tank))
var calm: Dictionary = _pull_scene(close, false)
var calm_near: Combatant = calm["near"]
checks["no taunt: the reachable Near is struck"] = _strikes_on(calm, calm_near)
var far_dist: int = move + 2   # route = move + 1 tiles: beyond the one move
var far: Dictionary = _pull_scene(far_dist, true)
checks["fixture: the far route exceeds the move"] = not DictUtil.to_bool(far["fits"]) and far_dist <= CombatGrid.GRID_W - 2
var far_near: Combatant = far["near"]
checks["route beyond the move: no pull, Near is struck"] = _strikes_on(far, far_near)
# Guard suppression: a low-HP unit one tile beyond its reach of its only foe guards (and so loses the move);
# when that foe taunts, the pull skips that guard and the unit walks up and strikes it first (a guard once in
# reach no longer costs the move and stays allowed).
var calm_guard: Dictionary = _guard_scene(false)
var calm_guard_steps: Array[PlanStep] = _guard_steps(calm_guard)
var calm_tank: Combatant = calm_guard["tank"]
checks["fixture: low HP, the foe one tile beyond reach"] = DictUtil.to_bool(calm_guard["low"]) and DictUtil.to_bool(calm_guard["beyond_reach"])
checks["no taunt: the low-HP unit guards and does not strike"] = _has_kind(calm_guard_steps, SkillDef.Kind.DEFEND) and not _engages(calm_guard_steps, calm_tank)
var pull_guard: Dictionary = _guard_scene(true)
var pull_guard_steps: Array[PlanStep] = _guard_steps(pull_guard)
var pull_tank: Combatant = pull_guard["tank"]
var opener: PlanStep = pull_guard_steps[0] if not pull_guard_steps.is_empty() else null
checks["taunting foe within the move: no guard before the move, the unit opens by striking the taunter"] = opener != null and opener.mode == PlanStep.Mode.ENGAGE and opener.target == pull_tank
return TestHelpers.report(TAG, "BAL3-AI lever D: taunt pull", checks)
```

### `static _guard_scene(taunting: bool) -> Dictionary`
```logic
var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
var actor: Combatant = AiSkillsKit.unit("Guarder", grid, Vector2i(4, 4), [])
var reach: int = SkillCatalog.find(AiSkillsKit.STRIKE).reach_for(actor).y
var tank: Combatant = AiSkillsKit.unit("Tank", grid, Vector2i(4 + reach + 1, 4), [])
if taunting:
	tank.statuses[StatusEffects.TAUNT] = true
var ctx: PlanContext = TestHelpers.hand_ctx(grid, [actor], [tank], 3)
var profile: AiProfile = AiSkillsKit.ai_profile(AiProfile.Role.AGGRESSOR)
ctx.profiles[actor] = profile
actor.hp = floori(profile.defend_below * float(actor.max_hp))
var ratio: float = float(actor.hp) / float(actor.max_hp)
return {"actor": actor, "tank": tank, "ctx": ctx, "profile": profile, "low": ratio <= profile.defend_below and ratio > profile.flee_ratio,
	"beyond_reach": grid.get_distance(grid.get_position(actor), grid.get_position(tank)) == reach + 1}
```

### `static _has_kind(steps: Array[PlanStep], kind: SkillDef.Kind) -> bool`
```logic
for step: PlanStep in steps:
	if step.skill != null and step.skill.kind == kind:
		return true
return false
```

### `static _engages(steps: Array[PlanStep], unit: Combatant) -> bool`
```logic
for step: PlanStep in steps:
	if step.mode == PlanStep.Mode.ENGAGE and step.target == unit:
		return true
return false
```

### `static _early_taunt_scene(mage_hp: int) -> Dictionary`
```logic
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(7, 4), ["ai1_taunt"], [Vector2i(8, 4)], [Vector2i(4, 4)])
var ctx: PlanContext = s["ctx"]
var diver: Combatant = AiSkillsKit.unit_of(s, "foes", 0)
ctx.profiles[diver] = AiSkillsKit.ai_profile(AiProfile.Role.SKIRMISHER)
AiSkillsKit.unit_of(s, "allies", 1).hp = mage_hp
return s
```

### `static _test_early_taunt() -> bool`
```logic
var checks: Dictionary = {}
var s: Dictionary = _early_taunt_scene(100)
var ctx: PlanContext = s["ctx"]
var diver: Combatant = AiSkillsKit.unit_of(s, "foes", 0)
var mage: Combatant = AiSkillsKit.unit_of(s, "allies", 1)
var actor: Combatant = s["actor"]
var dist: int = ctx.grid.get_distance(ctx.grid.get_position(diver), ctx.grid.get_position(mage))
var threat: int = CombatPlan.new(ctx).max_move_tiles(diver) + SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(diver).y
checks["fixture: the mage is beyond the 3-tile rule but within the diver's move + reach"] = dist > 3 and dist <= threat
checks["fixture: the diver would pick the mage this round"] = TacticalAI.incoming_damage(CombatPlan.new(ctx), mage, ctx.total_blocks, ctx.grid.get_position(mage)) > 0 and TacticalAI.incoming_damage(CombatPlan.new(ctx), actor, ctx.total_blocks, ctx.grid.get_position(actor)) == 0
var steps: Array[PlanStep] = AiSkillsKit.plan_steps(s)
checks["a foe would pick an ally this round: the warrior drafts one taunt"] = AiSkillsKit.count_of(steps, "ai1_taunt") == 1 and AiSkillsKit.valid(s, steps)
var calm: Dictionary = _early_taunt_scene(AiSkillsKit.unit_of(s, "allies", 1).max_hp)
var calm_ctx: PlanContext = calm["ctx"]
var calm_mage: Combatant = AiSkillsKit.unit_of(calm, "allies", 1)
checks["fixture: a healthy mage is not the diver's pick (the warrior is)"] = TacticalAI.incoming_damage(CombatPlan.new(calm_ctx), calm_mage, calm_ctx.total_blocks, calm_ctx.grid.get_position(calm_mage)) == 0
checks["no foe would pick an ally and none within 3 tiles: no taunt"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(calm), "ai1_taunt") == 0
# Walk-in: the warrior stands out of strike reach of the diver (no taunt in place before a strike is possible).
var walk: Dictionary = _walk_in_scene(4)
var walk_ctx: PlanContext = walk["ctx"]
var walk_actor: Combatant = walk["actor"]
var walk_diver: Combatant = AiSkillsKit.unit_of(walk, "foes", 0)
var walk_mage: Combatant = AiSkillsKit.unit_of(walk, "allies", 1)
var start_gap: int = walk_ctx.grid.get_distance(walk_ctx.grid.get_position(walk_actor), walk_ctx.grid.get_position(walk_diver))
checks["fixture (walk-in): the warrior is beyond its strike reach of the diver, which would pick the mage"] = start_gap > SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(walk_actor).y and TacticalAI.incoming_damage(CombatPlan.new(walk_ctx), walk_mage, walk_ctx.total_blocks, walk_ctx.grid.get_position(walk_mage)) > 0
var walk_steps: Array[PlanStep] = AiSkillsKit.plan_steps(walk)
var walk_plan := CombatPlan.new(walk_ctx)
walk_plan.set_steps(walk_actor, walk_steps)
var end_gap: int = walk_ctx.grid.get_distance(walk_plan.compile(walk_actor).pos, walk_ctx.grid.get_position(walk_diver))
var opens_walk: bool = walk_steps.size() >= 2 and (walk_steps[0].mode == PlanStep.Mode.GOTO or walk_steps[0].mode == PlanStep.Mode.MOVE)
checks["walk-in: it walks toward the diver first, then taunts (plan valid)"] = opens_walk and walk_steps[1].skill.id == "ai1_taunt" and end_gap < start_gap and AiSkillsKit.valid(walk, walk_steps)
var walk_threat: int = CombatPlan.new(walk_ctx).max_move_tiles(walk_diver) + SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(walk_diver).y
var safe: Dictionary = _walk_in_scene(walk_threat + 1)
var safe_ctx: PlanContext = safe["ctx"]
var safe_mage: Combatant = AiSkillsKit.unit_of(safe, "allies", 1)
checks["fixture (counter): the mage is beyond the diver's move + reach and more than 3 tiles away"] = walk_threat + 1 > 3 and 8 - walk_threat - 1 >= 0 and TacticalAI.incoming_damage(CombatPlan.new(safe_ctx), safe_mage, safe_ctx.total_blocks, safe_ctx.grid.get_position(safe_mage)) == 0
var safe_steps: Array[PlanStep] = AiSkillsKit.plan_steps(safe)
var safe_actor: Combatant = safe["actor"]
var safe_plan := CombatPlan.new(safe_ctx)
safe_plan.set_steps(safe_actor, safe_steps)
var safe_diver_at: Vector2i = safe_ctx.grid.get_position(AiSkillsKit.unit_of(safe, "foes", 0))
var safe_walks: bool = safe_ctx.grid.get_distance(safe_plan.compile(safe_actor).pos, safe_diver_at) < safe_ctx.grid.get_distance(safe_ctx.grid.get_position(safe_actor), safe_diver_at)
checks["no foe would pick the mage: the warrior still advances but does not taunt"] = safe_walks and AiSkillsKit.count_of(safe_steps, "ai1_taunt") == 0
return TestHelpers.report(TAG, "BAL5-AI lever T2: early taunt", checks)
```

### `static _walk_in_scene(mage_gap: int) -> Dictionary`
```logic
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(6, 1), ["ai1_taunt"], [Vector2i(8, 4)], [Vector2i(8 - mage_gap, 4)])
var ctx: PlanContext = s["ctx"]
ctx.profiles[AiSkillsKit.unit_of(s, "foes", 0)] = AiSkillsKit.ai_profile(AiProfile.Role.SKIRMISHER)
AiSkillsKit.unit_of(s, "allies", 1).hp = 100
return s
```

### `static _care_scene(kit: Array[String], foe_cells: Array[Vector2i], share: float, actor_hp_ratio: float) -> Dictionary`
```logic
var power: int = 1
var s: Dictionary = {}
while power < 2000:
	s = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, foe_cells, [], 3)
	var actor: Combatant = s["actor"]
	actor.hp = floori(actor_hp_ratio * float(actor.max_hp))
	var foes: Array[Combatant] = s["foes"]
	for foe: Combatant in foes:
		foe.power_override = 1
	AiSkillsKit.unit_of(s, "foes", 0).power_override = power
	if share <= 0.0 or _incoming_share(s) >= share:
		break
	power += 5
return s
```

### `static _incoming_share(s: Dictionary) -> float`
```logic
var ctx: PlanContext = s["ctx"]
var actor: Combatant = s["actor"]
return float(TacticalAI.incoming_damage(CombatPlan.new(ctx), actor, ctx.total_blocks, ctx.grid.get_position(actor))) / float(maxi(1, actor.hp))
```

### `static _test_self_care() -> bool`
```logic
var checks: Dictionary = {}
var kit: Array[String] = ["ai1_heal", "ai1_heal_big"]
var adjacent: Array[Vector2i] = [Vector2i(5, 4)]
var hot: Dictionary = _care_scene(kit, adjacent, 0.6, 0.9)
var hot_actor: Combatant = hot["actor"]
var hot_profile: AiProfile = hot["profile"]
checks["fixture: the healer is above heal_below and projected to lose >= 60% of its HP"] = float(hot_actor.hp) / float(hot_actor.max_hp) > hot_profile.heal_below and _incoming_share(hot) >= 0.6 and hot_actor.hp < hot_actor.max_hp
var hot_steps: Array[PlanStep] = AiSkillsKit.plan_steps(hot)
var opener: PlanStep = hot_steps[0] if not hot_steps.is_empty() else null
checks["under pressure: it heals itself first, with the lowest heal tier"] = opener != null and opener.mode == PlanStep.Mode.SELF and opener.skill.id == "ai1_heal"
checks["one self-heal only and the plan is valid"] = AiSkillsKit.count_of(hot_steps, "ai1_heal") + AiSkillsKit.count_of(hot_steps, "ai1_heal_big") == 1 and AiSkillsKit.valid(hot, hot_steps)
var calm: Dictionary = _care_scene(kit, adjacent, 0.0, 0.9)
checks["fixture: calm board, incoming below half its HP"] = _incoming_share(calm) < TacticalAI.SELF_CARE_RATIO
checks["calm: no self-heal (it is above heal_below)"] = AiSkillsKit.count_of(AiSkillsKit.plan_steps(calm), "ai1_heal") + AiSkillsKit.count_of(AiSkillsKit.plan_steps(calm), "ai1_heal_big") == 0
var sniper: Dictionary = _care_scene(kit, adjacent, 0.6, 0.9)
var sniper_actor: Combatant = sniper["actor"]
var sniper_ctx: PlanContext = sniper["ctx"]
checks["only the SUPPORT role self-cares"] = AiSkillsKit.count_of(TacticalAI.build_steps(sniper_actor, sniper_ctx, AiSkillsKit.ai_profile(AiProfile.Role.SNIPER)), "ai1_heal") == 0
# No self-heal: a strong melee threat adjacent east, a weak target 3 tiles west-north (at the basic ranged reach, so
# a calm healer neither kites nor walks); under pressure it steps one tile away from the threat first.
var threat_cells: Array[Vector2i] = [Vector2i(5, 4), Vector2i(2, 3)]
var no_kit: Array[String] = []
var pressed: Dictionary = _care_scene(no_kit, threat_cells, 0.6, 1.0)
AiSkillsKit.unit_of(pressed, "foes", 1).hp = 50
var pressed_ctx: PlanContext = pressed["ctx"]
var pressed_actor: Combatant = pressed["actor"]
var threat: Combatant = AiSkillsKit.unit_of(pressed, "foes", 0)
checks["fixture: no heal known and pressed by >= 60% of its HP"] = _incoming_share(pressed) >= 0.6
var pressed_steps: Array[PlanStep] = AiSkillsKit.plan_steps(pressed)
var first: PlanStep = pressed_steps[0] if not pressed_steps.is_empty() else null
var moved_away: bool = false
if first != null and first.mode == PlanStep.Mode.MOVE and first.tiles == 1:
	var after: Vector2i = pressed_ctx.grid.get_position(pressed_actor) + first.direction
	moved_away = pressed_ctx.grid.get_distance(after, pressed_ctx.grid.get_position(threat)) > pressed_ctx.grid.get_distance(pressed_ctx.grid.get_position(pressed_actor), pressed_ctx.grid.get_position(threat))
checks["pressed with no self-heal: one step away from the threat first, plan valid"] = moved_away and AiSkillsKit.valid(pressed, pressed_steps)
var easy: Dictionary = _care_scene(no_kit, threat_cells, 0.0, 1.0)
AiSkillsKit.unit_of(easy, "foes", 1).hp = 50
var easy_moves: bool = false
for step: PlanStep in AiSkillsKit.plan_steps(easy):
	easy_moves = easy_moves or step.mode == PlanStep.Mode.MOVE or step.mode == PlanStep.Mode.GOTO
checks["calm, same board: no step back"] = _incoming_share(easy) < TacticalAI.SELF_CARE_RATIO and not easy_moves
return TestHelpers.report(TAG, "BAL5-AI lever S1: healer self-care", checks)
```

### `static _ff_scene(three: bool) -> Dictionary`
```logic
var foe_cells: Array[Vector2i] = [Vector2i(4, 3), Vector2i(6, 3)]
if three:
	foe_cells.append(Vector2i(5, 5))
return AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(1, 4), ["ai1_burst"], foe_cells, [Vector2i(5, 4)])
```

### `static _ally_burned(hits: Array[Vector2i]) -> bool`
```logic
for h: Vector2i in hits:
	if h.y > 0:
		return true
return false
```

### `static _test_friendly_fire() -> bool`
```logic
var checks: Dictionary = {}
var probe: Dictionary = _ff_scene(true)
var probe_actor: Combatant = probe["actor"]
var probe_ally: Combatant = AiSkillsKit.unit_of(probe, "allies", 1)
var dmg: int = DamageCalculator.expected(probe_actor, probe_ally, SkillCatalog.find("ai1_burst"))
checks["fixture: the burst deals damage to the ally"] = dmg > 0
var ok_cast: Dictionary = _ff_scene(true)
AiSkillsKit.unit_of(ok_cast, "allies", 1).hp = dmg * 4 + 1   # the hit costs it just under 25% of its current HP
var ok_steps: Array[PlanStep] = AiSkillsKit.plan_steps(ok_cast)
var ok_hits: Array[Vector2i] = _burst_hits(ok_cast, ok_steps)
checks["3 foes and one ally losing < 25% of its HP: the burst is cast on all four"] = not ok_hits.is_empty() and ok_hits[0] == Vector2i(3, 1) and AiSkillsKit.valid(ok_cast, ok_steps)
var costly: Dictionary = _ff_scene(true)
AiSkillsKit.unit_of(costly, "allies", 1).hp = dmg * 4   # exactly 25%: not tolerated
checks["3 foes but the ally would lose >= 25% of its HP: no burst on the ally"] = not _ally_burned(_burst_hits(costly, AiSkillsKit.plan_steps(costly)))
var two: Dictionary = _ff_scene(false)
checks["2 foes and the ally (healthy): no burst on the ally"] = not _ally_burned(_burst_hits(two, AiSkillsKit.plan_steps(two)))
return TestHelpers.report(TAG, "BAL5-AI lever A2: friendly-fire tolerance", checks)
```

### `static _play(seed_value: int) -> String`
```logic
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
```

### `static _test_simulation() -> bool`
```logic
var checks: Dictionary = {}
var first: String = _play(4242)
var second: String = _play(4242)
var parts: PackedStringArray = first.split("|")
checks["50 seeded rounds of a 3v3: every planned step validates (none invalid, none dry)"] = parts[0] == "0"
checks["MAX_STEPS is respected"] = parts[1] == "0"
checks["the same seed gives the same plans end to end"] = first == second
return TestHelpers.report(TAG, "Seeded 3v3 simulation: valid, bounded and deterministic", checks)
```

