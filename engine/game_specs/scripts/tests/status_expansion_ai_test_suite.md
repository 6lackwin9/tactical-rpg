---
class: StatusExpansionAiTestSuite
extends: RefCounted
description: "Tactical RPG - Status Expansion AI and sim metrics Test Suite (S12S-L commit 3) Rule 8 (TacticalAI AI1 section): stacking debuffs stay valid below the cap, expose and slow target order, fortify and regen targets. Rule 9 (tools/sim/sim_metrics.gd): a real fight in which a party member casts poison and another casts regen keeps HP conservation (no METRIC FAIL), the regen caster's heal_done equals the regen its targets received, and dot_dealt matches dot_taken. Every skill is a \"t_sx_*\" row of a sandbox catalog fork; numbers derive from the registry rows."
---

# Properties
* `SimArgs`: Variant = preload("res://tools/sim/sim_args.gd") (const)
* `SimRunner`: Variant = preload("res://tools/sim/sim_runner.gd") (const)
* `SimMetrics`: Variant = preload("res://tools/sim/sim_metrics.gd") (const)
* `SimScenarios`: Variant = preload("res://tools/sim/sim_scenarios.gd") (const)
* `TAG`: String = "StatusAiS12S" (const)
* `SIM_LEVEL`: int = 5 (const)
* `SIM_ROUNDS`: int = 5 (const)
* `out`: Array[Combatant] = []
* `actor`: Combatant = scene["actor"]
* `profile`: AiProfile = scene["profile"]
* `actor`: Combatant = scene["actor"]
* `profile`: AiProfile = scene["profile"]
* `out`: Array[Combatant] = []

# Methods

### `static run() -> bool`
```logic
print("\n=======================================================")
print("=== STARTING STATUS EXPANSION AI + SIM TEST (S12S-L) ===")
print("=======================================================")
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
var ok: bool = _setup()
if ok:
	ok = _test_stacking_debuff() and ok
	ok = _test_expose_order() and ok
	ok = _test_slow_order() and ok
	ok = _test_fortify_targets() and ok
	ok = _test_regen() and ok
	ok = _test_sim_metrics() and ok
TestHelpers.cleanup(SANDBOX)
SkillCatalog.clear_cache()
ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
return ok
```

### `static _setup() -> bool`
```logic
var checks: Dictionary = {}
var forked: bool = TestHelpers.fork(SANDBOX)
checks["sandbox fork"] = forked
if forked:
	var errors: Array[String] = []
	for row: Dictionary in rows():
		var entry: Dictionary = row.duplicate()
		entry["learned_only"] = true
		errors.append_array(ContentCatalog.upsert("skills", entry))
	checks["every t_sx_* fixture row is accepted %s" % str(errors)] = errors.is_empty()
	SkillCatalog.clear_cache()
	checks["the fixtures are visible to SkillCatalog"] = SkillCatalog.find("t_sx_poison") != null and SkillCatalog.find("t_sx_regen") != null
return TestHelpers.report(TAG, "Sandbox fixtures", checks)
```

### `static _plan(scene: Dictionary) -> CombatPlan`
```logic
var ctx: PlanContext = scene["ctx"]
return CombatPlan.new(ctx)
```

### `static _test_stacking_debuff() -> bool`
```logic
var checks: Dictionary = {}
var p: String = StatusEffects.POISON
var cap: int = StatusRegistry.cap_of(p)
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["t_sx_poison"], [Vector2i(5, 4)])
var foe: Combatant = AiSkillsKit.unit_of(s, "foes", 0)
StatusEffects.queue(foe, p, 3, 10, "X")
StatusEffects.roll_over(foe, -1)
checks["fixture: foe at poison 10, cap 30"] = DictUtil.int_of(foe.status_magnitudes, p) == 10 and cap == 30
checks["poison 10 < cap: the debuff option is offered"] = _debuff_targets(s).has(foe)
var steps: Array[PlanStep] = AiSkillsKit.plan_steps(s)
checks["the drafted plan casts it and stays valid"] = AiSkillsKit.count_of(steps, "t_sx_poison") == 1 and AiSkillsKit.valid(s, steps)
foe.status_magnitudes[p] = cap
checks["poison at the cap: not offered"] = not _debuff_targets(s).has(foe) and AiSkillsKit.count_of(AiSkillsKit.plan_steps(s), "t_sx_poison") == 0
foe.status_magnitudes[p] = cap - 10
StatusEffects.queue(foe, p, 1, 10, "Y")
checks["active 20 + pending 10 reaches the cap: not offered"] = not _debuff_targets(s).has(foe)
var w: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["t_sx_expose"], [Vector2i(5, 4)])
var exposed: Combatant = AiSkillsKit.unit_of(w, "foes", 0)
StatusEffects.queue(exposed, StatusEffects.EXPOSE, 2, 1, "X")
checks["a non-stacking status still skips a foe that has it (even below the cap)"] = not _debuff_targets(w).has(exposed)
return TestHelpers.report(TAG, "Stacking debuffs below the cap (acceptance 10)", checks)
```

### `static _test_expose_order() -> bool`
```logic
var checks: Dictionary = {}
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["t_sx_expose"], [Vector2i(5, 4), Vector2i(4, 5), Vector2i(4, 3)], [Vector2i(3, 3)])
var weak: Combatant = AiSkillsKit.unit_of(s, "foes", 0)    # (5, 4): no ally next to it
var strong: Combatant = AiSkillsKit.unit_of(s, "foes", 1)  # (4, 5): the highest power
var flanked: Combatant = AiSkillsKit.unit_of(s, "foes", 2) # (4, 3)
weak.power_override = 10
strong.power_override = 50
flanked.power_override = 20
# The ally at (3, 3) stands next to `flanked` only.
var ranked: Array[Combatant] = [weak]
var order: Array[Combatant] = _debuff_targets(s, ranked)
checks["the ranked first target comes first"] = not order.is_empty() and order[0] == weak
checks["then the foe adjacent to an ally, then by power"] = order.size() == 3 and order[1] == flanked and order[2] == strong
var no_rank: Array[Combatant] = _debuff_targets(s)
checks["without a ranked target the ally-adjacent foe leads"] = not no_rank.is_empty() and no_rank[0] == flanked
return TestHelpers.report(TAG, "Expose target order (rule 8)", checks)
```

### `static _test_slow_order() -> bool`
```logic
var checks: Dictionary = {}
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["t_sx_slow"], [Vector2i(5, 4), Vector2i(1, 3), Vector2i(2, 1)], [Vector2i(1, 1)])
var near_actor: Combatant = AiSkillsKit.unit_of(s, "foes", 0)
var threat_near: Combatant = AiSkillsKit.unit_of(s, "foes", 1)
var threat_far: Combatant = AiSkillsKit.unit_of(s, "foes", 2)
near_actor.power_override = 90   # the default order would take it first
var fragile: Combatant = AiSkillsKit.unit_of(s, "allies", 1)
fragile.max_hp = 50
fragile.hp = 50
var grid: CombatGrid = s["grid"]
var plan: CombatPlan = _plan(s)
var threat: Callable = func(foe: Combatant) -> bool:
	return grid.get_distance(grid.get_position(foe), grid.get_position(fragile)) <= plan.max_move_tiles(foe) + SkillCatalog.lowest(SkillDef.Kind.ATTACK).reach_for(foe).y
checks["fixture: two foes can reach the fragile ally this round, one cannot"] = bool(threat.call(threat_near)) and bool(threat.call(threat_far)) and not bool(threat.call(near_actor))
var order: Array[Combatant] = _debuff_targets(s)
checks["threats to the fragile ally come first, nearest to the actor first"] = order.size() == 3 and order[0] == threat_near and order[1] == threat_far and order[2] == near_actor
return TestHelpers.report(TAG, "Slow target order (rule 8)", checks)
```

### `static _test_fortify_targets() -> bool`
```logic
var checks: Dictionary = {}
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), ["t_sx_fort"], [Vector2i(1, 2), Vector2i(6, 7), Vector2i(7, 6)], [Vector2i(1, 1), Vector2i(6, 6)])
var actor: Combatant = s["actor"]
var lone: Combatant = AiSkillsKit.unit_of(s, "allies", 1)     # one adjacent foe
var crowded: Combatant = AiSkillsKit.unit_of(s, "allies", 2)  # two adjacent foes
var order: Array[Combatant] = _self_or(_buff_targets(s), actor)
checks["the ally with the most adjacent foes comes first"] = order.size() == 3 and order[0] == crowded and order[1] == lone and order[2] == actor
crowded.statuses[StatusEffects.FORTIFY] = true
var skip: Array[Combatant] = _self_or(_buff_targets(s), actor)
checks["an already fortified unit is skipped"] = not skip.has(crowded) and not skip.is_empty() and skip[0] == lone
crowded.statuses.erase(StatusEffects.FORTIFY)
var grid: CombatGrid = s["grid"]
var extra: Combatant = AiSkillsKit.unit_of(s, "foes", 2)
grid.remove_combatant(extra)
grid.place_combatant(extra, Vector2i(7, 0))   # now each ally has one adjacent foe
lone.armor_override = 30
crowded.armor_override = 5
var tie: Array[Combatant] = _self_or(_buff_targets(s), actor)
checks["a tie goes to the lower armor rating"] = not tie.is_empty() and tie[0] == crowded
lone.statuses[StatusEffects.TAUNT] = true
var taunt: Array[Combatant] = _self_or(_buff_targets(s), actor)
checks["a taunting unit wins the tie before armor"] = not taunt.is_empty() and taunt[0] == lone
return TestHelpers.report(TAG, "Fortify targets (rule 8)", checks)
```

### `static _test_regen() -> bool`
```logic
var checks: Dictionary = {}
var s: Dictionary = AiSkillsKit.new_scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), ["t_sx_regen", "t_sx_heal5"], [Vector2i(5, 4)], [Vector2i(3, 4), Vector2i(4, 6)])
var actor: Combatant = s["actor"]
var profile: AiProfile = s["profile"]
profile.skill_cap = SkillDef.MAX_LEVEL   # the level 5 heal is in reach, it is only too expensive
var hurt: Combatant = AiSkillsKit.unit_of(s, "allies", 1)
var scratched: Combatant = AiSkillsKit.unit_of(s, "allies", 2)
hurt.hp = roundi(float(hurt.max_hp) * profile.heal_below * 0.5)
scratched.hp = scratched.max_hp - 1   # wounded but above the threshold
actor.ap = _regen_budget(s, hurt)
var ctx: PlanContext = s["ctx"]
var probe := CombatPlan.new(ctx)
var heal_problem: String = probe.add_step(actor, PlanStep.ally(SkillCatalog.find("t_sx_heal5"), hurt), false)
checks["fixture: AP %d fits regen + a strike but not the heal" % actor.ap] = actor.ap > 0 and not heal_problem.is_empty()
var steps: Array[PlanStep] = AiSkillsKit.plan_steps(s)
var cast: int = AiSkillsKit.index_of(steps, "t_sx_regen")
checks["no heal is affordable: the wounded ally receives regen %s" % AiSkillsKit.sig(steps)] = AiSkillsKit.count_of(steps, "t_sx_heal5") == 0 and cast >= 0 and steps[cast].target == hurt
checks["the plan is valid"] = AiSkillsKit.valid(s, steps)
var targets: Array[Combatant] = _self_or(_buff_targets(s), actor)
checks["only allies below the heal threshold are regen targets"] = targets == [hurt]
hurt.pending_statuses[StatusEffects.REGEN] = true
checks["an ally awaiting regen is skipped"] = _buff_targets(s).is_empty()
hurt.pending_statuses.erase(StatusEffects.REGEN)
actor.ap = actor.max_ap
var rich: Array[PlanStep] = AiSkillsKit.plan_steps(s)
var heal_at: int = AiSkillsKit.index_of(rich, "t_sx_heal5")
var regen_at: int = AiSkillsKit.index_of(rich, "t_sx_regen")
checks["an affordable heal comes first %s" % AiSkillsKit.sig(rich)] = heal_at >= 0 and rich[heal_at].target == hurt and (regen_at < 0 or heal_at < regen_at)
return TestHelpers.report(TAG, "Regen targets (acceptance 10)", checks)
```

### `static _regen_budget(scene: Dictionary, patient: Combatant) -> int`
```logic
var actor: Combatant = scene["actor"]
var ctx: PlanContext = scene["ctx"]
var foe: Combatant = AiSkillsKit.unit_of(scene, "foes", 0)
for budget: int in range(1, actor.max_ap + 1):
	actor.ap = budget
	var plan := CombatPlan.new(ctx)
	if plan.add_step(actor, PlanStep.ally(SkillCatalog.find("t_sx_regen"), patient), false).is_empty() and plan.add_step(actor, PlanStep.engage(SkillCatalog.find(AiSkillsKit.STRIKE), foe), false).is_empty():
		return budget
return 0
```

### `static _grant(c: Combatant, skill_id: String) -> void`
```logic
if not c.known_skills.has(skill_id):
	c.known_skills.append(skill_id)
var family: String = SkillCatalog.family_of(skill_id)
if not c.active_families.is_empty() and not c.active_extra_families.has(family):
	c.active_extra_families.append(family)
	c._refresh_active()
```

### `static _replace(session: CombatSession, actor: Combatant, options: Array) -> bool`
```logic
if not actor.is_active():
	return false
var drafted: Array[PlanStep] = session.plan.steps_of(actor).duplicate()
for raw: Variant in options:
	var steps: Array[PlanStep] = []
	steps.assign(raw)
	session.plan.set_steps(actor, steps)
	var st: CombatPlan.SimState = session.plan.compile(actor)
	if st.error_index < 0 and not st.runs_dry():
		return true
session.plan.set_steps(actor, drafted)
return false
```

### `static _first_active(units: Array[Combatant]) -> Combatant`
```logic
for c: Combatant in units:
	if c.is_active():
		return c
return null
```

### `static _test_sim_metrics() -> bool`
```logic
var checks: Dictionary = {}
var args := SimArgs.new()
var runner := SimRunner.new(args)
var party: Array[String] = SimScenarios.define_party(runner, ["warrior", "warrior"] as Array[String], SIM_LEVEL)   # no class heals: heal_done is regen only
var foes: Array[String] = SimScenarios.define_enemies(runner, [["warrior", "standard"], ["warrior", "standard"]], SIM_LEVEL, 1, "warrior", "brawler")
var written: bool = true
for id: String in party + foes:
	written = written and runner.build(id) != null
checks["sandbox units written"] = written
if not written:
	return TestHelpers.report(TAG, "Sim metrics with regen and poison (acceptance 11)", checks)
var entry_def: Dictionary = {"id": "t_sx_sim", "name": "Probe", "zone": "stone_arena", "arena": "CLOSED", "total_blocks": 6, "unit_ap": 12, "ap_regen": 4,
	"party": party, "enemies": foes, "max_rounds": SIM_ROUNDS, "rng_seed": 3}
var session := CombatSession.new(CatalogEncounter.from_entry(entry_def))
session.start()
var poisoner: Combatant = session.allies[0]
var healer: Combatant = session.allies[1]
_grant(poisoner, "t_sx_poison")
_grant(healer, "t_sx_regen")
poisoner.hp = roundi(float(poisoner.max_hp) * 0.6)   # wounded from the start: regen has room to heal
var metrics := SimMetrics.new()
metrics.quiet = true
metrics.on_start(session)
var poison_casts: int = 0
var regen_casts: int = 0
for round_num: int in range(SIM_ROUNDS):
	session.plan_apply_preset_all("balanced")
	var foe: Combatant = _first_active(session.enemies)
	if foe != null:
		var poison: PlanStep = PlanStep.engage(SkillCatalog.find("t_sx_poison"), foe)
		if _replace(session, poisoner, [[poison, PlanStep.engage(SkillCatalog.find("strike"), foe)], [poison]]):
			poison_casts += 1
	var patient: Combatant = poisoner if poisoner.is_active() else healer
	var regen: PlanStep = PlanStep.ally(SkillCatalog.find("t_sx_regen"), patient) if patient != healer else PlanStep.self_skill(SkillCatalog.find("t_sx_regen"))
	var healer_steps: Array[PlanStep] = [regen]
	healer_steps.append_array(session.plan.steps_of(healer))
	if _replace(session, healer, [healer_steps, [regen]]):
		regen_casts += 1
	if not session.confirm_plan():
		break
	for i: int in range(SimRunner.MAX_STEPS_PER_ROUND):
		if session.phase == CombatSession.Phase.REPLAY:
			break
		session.step_next()
		metrics.on_step(session)
	if session.combat_over or not session.can_advance_round():
		break
	session.advance_round()
	metrics.on_round(session)
metrics.on_end(session, {})
session.dispose()
var regen_total: int = 0
var dot_dealt: int = 0
var dot_taken: int = 0
for n: Variant in metrics.units:
	var r: Dictionary = metrics.units[n]
	regen_total += DictUtil.int_of(r, "regen_received")
	dot_dealt += DictUtil.int_of(r, "dot_dealt")
	dot_taken += DictUtil.int_of(r, "dot_taken")
var healer_rec: Dictionary = metrics.units[healer.name]
var poisoner_rec: Dictionary = metrics.units[poisoner.name]
checks["fixture: poison and regen were cast (%d / %d)" % [poison_casts, regen_casts]] = poison_casts > 0 and regen_casts > 0
checks["no METRIC FAIL %s" % str(metrics.fail_lines)] = metrics.fail_lines.is_empty()
checks["regen healed something (%d)" % regen_total] = regen_total > 0
checks["heal_done of the regen caster = its targets' regen_received (the caster has no other heal)"] = DictUtil.int_of(healer_rec, "heal_done") == regen_total
checks["the poisoner dealt damage over time (%d)" % DictUtil.int_of(poisoner_rec, "dot_dealt")] = DictUtil.int_of(poisoner_rec, "dot_dealt") > 0
checks["dot_dealt summed over units = dot_taken summed (%d / %d)" % [dot_dealt, dot_taken]] = dot_dealt == dot_taken
checks["the casts count as buffs / debuffs applied"] = DictUtil.int_of(healer_rec, "buffs_applied") > 0 and DictUtil.int_of(poisoner_rec, "debuffs_applied") > 0
return TestHelpers.report(TAG, "Sim metrics with regen and poison (acceptance 11)", checks)
```

