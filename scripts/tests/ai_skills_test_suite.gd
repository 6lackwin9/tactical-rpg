class_name AiSkillsTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - AI Skills Test Suite (slice AI1)
# TacticalAI uses disengage, area attacks, cleanse-only, taunt, ally buffs, debuff attacks and
# riposte, and AiProfile.smart_skills = false restores the pre-AI1 behaviour. Every skill used
# here is an "ai1_*" row of a sandbox catalog fork (the shipped catalog is never touched) and
# every expectation is derived from the rules, not from shipped skill numbers.
# ==============================================================================

const TAG: String = "AI1"
const STRIKE: String = "strike"

static var SANDBOX: String = TestHelpers.sandbox("ai1_skills_test")

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING AI SKILLS TEST (AI1)                    ===")
	print("=======================================================")
	var ok: bool = _setup()
	if ok:
		ok = _test_switch_off() and ok
		ok = _test_zero_power_never_the_strike() and ok
		ok = _test_disengage() and ok
		ok = _test_area() and ok
		ok = _test_cleanse() and ok
		ok = _test_taunt() and ok
		ok = _test_buff() and ok
		ok = _test_debuff() and ok
		ok = _test_riposte() and ok
		ok = _test_simulation() and ok
	_teardown()
	return ok

# ------------------------------------------------------------------------------
# Fixtures
# ------------------------------------------------------------------------------

static func _rows() -> Array[Dictionary]:
	return [
		{"id": "ai1_back", "name": "Ai1 Back", "kind": "DISENGAGE", "level": 2, "blocks": 1, "jump_tiles": 3},
		{"id": "ai1_sweep", "name": "Ai1 Sweep", "kind": "ATTACK", "level": 2, "blocks": 2, "power_mult": 0.7, "area": "SWEEP"},
		{"id": "ai1_burst", "name": "Ai1 Burst", "kind": "ATTACK", "level": 2, "blocks": 2, "power_mult": 0.7, "area": "BURST", "reach_min": 2, "reach_max": 4},
		{"id": "ai1_clean_a", "name": "Ai1 Clean A", "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "ALLY_OR_SELF", "cleanse_statuses": ["stun"]},
		{"id": "ai1_clean_b", "name": "Ai1 Clean B", "kind": "SUPPORT", "level": 2, "blocks": 1, "target_mode": "ALLY_OR_SELF", "cleanse_statuses": ["all"]},
		{"id": "ai1_taunt", "name": "Ai1 Taunt", "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "SELF", "grant_status": "taunt", "status_rounds": 2},
		{"id": "ai1_haste", "name": "Ai1 Haste", "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "ALLY", "grant_status": "haste", "status_rounds": 1},
		{"id": "ai1_heal", "name": "Ai1 Heal", "kind": "SUPPORT", "level": 1, "blocks": 1, "target_mode": "ALLY_OR_SELF", "heal_base": 20},
		{"id": "ai1_intim", "name": "Ai1 Intimidate", "kind": "ATTACK", "level": 3, "blocks": 1, "power_mult": 0.0, "stance": "DEFENSIVE", "status_effect": "weaken", "status_potency": 70, "status_magnitude": 15, "status_rounds": 2},
		{"id": "ai1_guard", "name": "Ai1 Guard", "kind": "DEFEND", "level": 2, "blocks": 2, "mitigation": 0.5},
		{"id": "ai1_rip", "name": "Ai1 Riposte", "kind": "DEFEND", "level": 2, "blocks": 2, "mitigation": 0.15, "grant_status": "riposte", "status_rounds": 2},
	]

static func _setup() -> bool:
	var checks: Dictionary = {}
	var forked: bool = TestHelpers.fork(SANDBOX)
	checks["sandbox fork"] = forked
	if forked:
		var errors: Array[String] = []
		for row: Dictionary in _rows():
			var entry: Dictionary = row.duplicate()
			entry["learned_only"] = true
			errors.append_array(ContentCatalog.upsert("skills", entry))
		if not errors.is_empty():
			print("    [AI1] fixture errors: %s" % str(errors))
		checks["every ai1_* fixture row is accepted by the catalog schema"] = errors.is_empty()
		SkillCatalog.clear_cache()
		checks["the fixtures are visible to SkillCatalog"] = SkillCatalog.find("ai1_back") != null and SkillCatalog.find("ai1_intim") != null and not SkillCatalog.find("ai1_intim").deals_damage()
		checks["the helper lists find them"] = _has(SkillCatalog.cleanse_only(3), "ai1_clean_a") and _has(SkillCatalog.cleanse_only(3), "ai1_clean_b") and not _has(SkillCatalog.cleanse_only(3), "ai1_heal") and _has(SkillCatalog.debuff_attacks(3), "ai1_intim") and _has(SkillCatalog.granting(SkillDef.Kind.SUPPORT, StatusEffects.TAUNT, 3), "ai1_taunt") and _has(SkillCatalog.ally_buffs(StatusEffects.TAUNT, 3), "ai1_haste") and not _has(SkillCatalog.ally_buffs(StatusEffects.TAUNT, 3), "ai1_taunt") and _has(SkillCatalog.granting(SkillDef.Kind.DEFEND, StatusEffects.RIPOSTE, 3), "ai1_rip")
	var profile := AiProfile.create(AiProfile.Role.AGGRESSOR)
	checks["AiProfile defaults: smart_skills on, area_min_foes 2"] = profile.smart_skills and profile.area_min_foes == 2
	profile.smart_skills = false
	profile.area_min_foes = 5
	var copy: AiProfile = profile.duplicate_profile()
	checks["duplicate_profile copies both fields"] = not copy.smart_skills and copy.area_min_foes == 5
	profile.area_min_foes = 0
	checks["area_min_foes is clamped to 1..6 on use"] = profile.area_threshold() == 1
	profile.area_min_foes = 99
	checks["(upper bound)"] = profile.area_threshold() == 6
	return TestHelpers.report(TAG, "Sandbox fixtures and profile fields", checks)

static func _has(skills: Array[SkillDef], skill_id: String) -> bool:
	for s: SkillDef in skills:
		if s.id == skill_id:
			return true
	return false

static func _teardown() -> void:
	TestHelpers.cleanup(SANDBOX)

# ------------------------------------------------------------------------------
# Scene helpers
# ------------------------------------------------------------------------------

static func _basics() -> Array[String]:
	var out: Array[String] = []
	for s: SkillDef in SkillCatalog.all():
		if s.basic:
			out.append(s.id)
	return out

static func _unit(unit_name: String, grid: CombatGrid, pos: Vector2i, kit: Array[String], attack_range: int = 1, hp: int = 400, power: int = 20, with_basics: bool = true) -> Combatant:
	var known: Array[String] = _basics() if with_basics else []
	known.append_array(kit)
	var c: Combatant = TestHelpers.unit(unit_name, grid, pos, hp, 10, attack_range, power)
	c.known_skills = known
	return c

static func _profile(role: AiProfile.Role, smart: bool = true) -> AiProfile:
	var p: AiProfile = AiProfile.create(role, 0.1, 3)
	p.smart_skills = smart
	return p

# A hand-built scene: the actor at `pos` (kit + the basics), foes and allies at the given cells.
static func _scene(role: AiProfile.Role, pos: Vector2i, kit: Array[String], foe_cells: Array[Vector2i], ally_cells: Array[Vector2i] = [], actor_range: int = 1, foe_range: int = 1, smart: bool = true) -> Dictionary:
	var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
	var actor: Combatant = _unit("Actor", grid, pos, kit, actor_range)
	var allies: Array[Combatant] = [actor]
	var foes: Array[Combatant] = []
	for i: int in range(ally_cells.size()):
		allies.append(_unit("Ally%d" % i, grid, ally_cells[i], [], 1))
	for i: int in range(foe_cells.size()):
		foes.append(_unit("Foe%d" % i, grid, foe_cells[i], [], foe_range))
	var ctx: PlanContext = TestHelpers.hand_ctx(grid, allies, foes, 3)
	var profile: AiProfile = _profile(role, smart)
	ctx.profiles[actor] = profile
	return {"grid": grid, "actor": actor, "allies": allies, "foes": foes, "ctx": ctx, "profile": profile}

static func _plan(scene: Dictionary) -> Array[PlanStep]:
	var actor: Combatant = scene["actor"]
	var ctx: PlanContext = scene["ctx"]
	var profile: AiProfile = scene["profile"]
	return TacticalAI.build_steps(actor, ctx, profile)

static func _unit_of(scene: Dictionary, key: String, index: int) -> Combatant:
	var list: Array[Combatant] = scene[key]
	return list[index]

static func _sig(steps: Array[PlanStep]) -> String:
	var parts: Array[String] = []
	for s: PlanStep in steps:
		parts.append("%d:%s:%s:%s:%d:%s" % [s.mode, s.skill.id if s.skill != null else "-", s.target.name if s.target != null else "-", str(s.direction), s.tiles, str(s.cell)])
	return "|".join(parts)

static func _count(steps: Array[PlanStep], skill_id: String) -> int:
	var n: int = 0
	for s: PlanStep in steps:
		if s.skill != null and s.skill.id == skill_id:
			n += 1
	return n

static func _index_of(steps: Array[PlanStep], skill_id: String) -> int:
	for i: int in range(steps.size()):
		if steps[i].skill != null and steps[i].skill.id == skill_id:
			return i
	return -1

# The plan passes the planner's own validation (no step was swallowed) and does not overspend AP.
static func _valid(scene: Dictionary, steps: Array[PlanStep]) -> bool:
	var actor: Combatant = scene["actor"]
	var ctx: PlanContext = scene["ctx"]
	var check := CombatPlan.new(ctx)
	check.set_steps(actor, steps)
	var st: CombatPlan.SimState = check.compile(actor)
	return st.error_index < 0 and not st.runs_dry() and steps.size() <= TacticalAI.MAX_STEPS

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
			scene = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(5, 4), Vector2i(4, 5)], [Vector2i(1, 1)], 1, 1, smart)
		1:
			scene = _scene(AiProfile.Role.SKIRMISHER, Vector2i(4, 4), kit, [Vector2i(5, 4)], [Vector2i(3, 4)], 1, 1, smart)
			var hurt: Combatant = scene["actor"]
			hurt.hp = 120
		2:
			scene = _scene(AiProfile.Role.SNIPER, Vector2i(4, 4), kit, [Vector2i(5, 4), Vector2i(7, 7)], [], 3, 1, smart)
		3:
			scene = _scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, [Vector2i(6, 4)], [Vector2i(3, 4), Vector2i(4, 3)], 3, 1, smart)
			var patient: Combatant = _unit_of(scene, "allies", 1)
			patient.hp = 100
			var stunned: Combatant = _unit_of(scene, "allies", 2)
			stunned.statuses[StatusEffects.STUN] = true   # inert without a cleanse skill the old code uses
		4:
			scene = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(7, 4)], [], 1, 1, smart)
			var rash: AiProfile = scene["profile"]
			rash.reckless = true
			var rash_unit: Combatant = scene["actor"]
			rash_unit.hp = 80
		_:
			scene = _scene(AiProfile.Role.AGGRESSOR, Vector2i(2, 2), kit, [Vector2i(3, 2), Vector2i(2, 3), Vector2i(3, 3)], [Vector2i(0, 1)], 1, 1, smart)
	return _sig(_plan(scene))

static func _test_switch_off() -> bool:
	var checks: Dictionary = {}
	for i: int in range(6):
		var bare_old: String = _scenario_sig(i, _inert_kit(), false)
		var bare_new: String = _scenario_sig(i, _inert_kit(), true)
		var rich_old: String = _scenario_sig(i, _rich_kit(), false)
		checks["scenario %d: a unit without the new skills plans the same under both switch values" % i] = bare_old == bare_new and not bare_old.is_empty()
		checks["scenario %d: smart_skills = false ignores the new skills (equals the plain kit)" % i] = rich_old == bare_old and not rich_old.contains("ai1_back") and not rich_old.contains("ai1_sweep") and not rich_old.contains("ai1_burst")
	# The switch really changes something when the kit has a use for it.
	var on: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], [Vector2i(5, 4), Vector2i(4, 5)])
	var off: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], [Vector2i(5, 4), Vector2i(4, 5)], [], 1, 1, false)
	checks["the same kit plans differently with the switch on and off"] = _sig(_plan(on)) != _sig(_plan(off))
	return TestHelpers.report(TAG, "smart_skills = false equals the old behaviour", checks)

static func _test_zero_power_never_the_strike() -> bool:
	var checks: Dictionary = {}
	var both: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], [Vector2i(5, 4)])
	var steps: Array[PlanStep] = _plan(both)
	var damage_steps: int = 0
	var strike_ok: bool = false
	for s: PlanStep in steps:
		if s.mode == PlanStep.Mode.ENGAGE and s.skill.deals_damage():
			damage_steps += 1
			strike_ok = strike_ok or s.skill.id == STRIKE
	checks["with intimidate (highest level) and a normal strike, the damage dealer is the normal strike"] = damage_steps >= 1 and strike_ok
	checks["the zero-power attack is only ever a debuff cast, never counted as damage"] = _count(steps, "ai1_intim") <= 1
	checks["the plan is valid"] = _valid(both, steps)
	var old: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], [Vector2i(5, 4)], [], 1, 1, false)
	var old_steps: Array[PlanStep] = _plan(old)
	checks["pre-AI1 (switch off) still picks it as the main attack (documents the old behaviour)"] = not old_steps.is_empty() and old_steps[0].skill.id == "ai1_intim"
	# A unit that knows only the zero-power attack behaves like an unarmed idler.
	var lone: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(0, 4), [], [Vector2i(8, 4)])
	var lone_actor: Combatant = lone["actor"]
	lone_actor.known_skills = ["ai1_intim"]
	var lone_steps: Array[PlanStep] = _plan(lone)
	var idle: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(0, 4), [], [Vector2i(8, 4)])
	var idle_actor: Combatant = idle["actor"]
	idle_actor.known_skills = ["ai1_guard"]
	var idle_steps: Array[PlanStep] = _plan(idle)
	checks["a unit with only the zero-power attack never plans it"] = _count(lone_steps, "ai1_intim") == 0
	checks["it advances like an unarmed idler (identical plan)"] = not lone_steps.is_empty() and _sig(lone_steps) == _sig(idle_steps)
	return TestHelpers.report(TAG, "Rule 1: zero-power attacks are never the strike", checks)

static func _test_disengage() -> bool:
	var checks: Dictionary = {}
	var s: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)], [], 3)
	var steps: Array[PlanStep] = _plan(s)
	var foe: Combatant = _unit_of(s, "foes", 0)
	var grid: CombatGrid = s["grid"]
	var start_dist: int = grid.get_distance(Vector2i(4, 4), grid.get_position(foe))
	checks["adjacent foe: the first step is the disengage skill"] = not steps.is_empty() and steps[0].skill.id == "ai1_back"
	checks["an attack step follows"] = steps.size() >= 2 and steps[1].mode == PlanStep.Mode.ENGAGE
	checks["the landing is farther from the foe than the start tile"] = not steps.is_empty() and grid.get_distance(steps[0].cell, grid.get_position(foe)) > start_dist
	checks["the plan is valid"] = _valid(s, steps)
	var support: Dictionary = _scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)], [], 3)
	var support_steps: Array[PlanStep] = _plan(support)
	checks["the SUPPORT role disengages too"] = not support_steps.is_empty() and support_steps[0].skill.id == "ai1_back"
	# Rooted: no disengage, the old kite / strike path runs.
	var rooted: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)], [], 3)
	var rooted_actor: Combatant = rooted["actor"]
	rooted_actor.statuses[StatusEffects.ROOT] = true
	checks["rooted: no disengage step"] = _count(_plan(rooted), "ai1_back") == 0
	# No skill: the plan equals the pre-AI1 plan.
	var bare_on: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(4, 4), [], [Vector2i(5, 4)], [], 3)
	var bare_off: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(4, 4), [], [Vector2i(5, 4)], [], 3, 1, false)
	checks["without the skill the plan is the old one"] = _sig(_plan(bare_on)) == _sig(_plan(bare_off))
	# No foe next to the unit: no disengage.
	var far: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_back"], [Vector2i(7, 4)], [], 3)
	checks["no adjacent foe: no disengage"] = _count(_plan(far), "ai1_back") == 0
	# A hurt skirmisher with a foe next to it jumps instead of the retreat step.
	var skirmisher: Dictionary = _scene(AiProfile.Role.SKIRMISHER, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)])
	var skirmisher_actor: Combatant = skirmisher["actor"]
	skirmisher_actor.hp = 140
	var skirmisher_steps: Array[PlanStep] = _plan(skirmisher)
	checks["a hurt skirmisher uses the disengage in place of the retreat move"] = not skirmisher_steps.is_empty() and skirmisher_steps[0].skill.id == "ai1_back" and _valid(skirmisher, skirmisher_steps)
	var healthy: Dictionary = _scene(AiProfile.Role.SKIRMISHER, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)])
	checks["a healthy skirmisher does not"] = _count(_plan(healthy), "ai1_back") == 0
	var reckless: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_back"], [Vector2i(5, 4)], [], 3)
	var reckless_profile: AiProfile = reckless["profile"]
	reckless_profile.reckless = true
	checks["a reckless unit never disengages"] = _count(_plan(reckless), "ai1_back") == 0
	return TestHelpers.report(TAG, "Rule 2: disengage", checks)

static func _test_area() -> bool:
	var checks: Dictionary = {}
	var foes_both: Array[Vector2i] = [Vector2i(5, 4), Vector2i(4, 5)]
	var s: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], foes_both, [Vector2i(1, 1)])
	var steps: Array[PlanStep] = _plan(s)
	var area_steps: Array[PlanStep] = []
	for step: PlanStep in steps:
		if step.mode == PlanStep.Mode.AREA:
			area_steps.append(step)
	checks["two foes in the sweep and no ally: the plan contains the area skill"] = not area_steps.is_empty() and area_steps[0].skill.id == "ai1_sweep"
	checks["it is aimed at the diagonal neighbour that hits both foes"] = not area_steps.is_empty() and area_steps[0].cell == Vector2i(5, 5)
	checks["the plan is valid"] = _valid(s, steps)
	# An ally inside the shape: the single-target strike is used instead.
	var allied: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], foes_both, [Vector2i(5, 5)])
	var allied_steps: Array[PlanStep] = _plan(allied)
	checks["an ally in the shape: no area step"] = _count(allied_steps, "ai1_sweep") == 0
	checks["the single-target strike is used instead"] = _count(allied_steps, STRIKE) >= 1
	# A single foe: strike, unless area_min_foes is 1.
	var single: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], [Vector2i(5, 4)])
	checks["a single foe: the single-target strike"] = _count(_plan(single), "ai1_sweep") == 0
	var eager: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], [Vector2i(5, 4)])
	var eager_profile: AiProfile = eager["profile"]
	eager_profile.area_min_foes = 1
	checks["area_min_foes = 1: the single-foe case picks the area skill"] = _count(_plan(eager), "ai1_sweep") >= 1
	var picky: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], foes_both)
	var picky_profile: AiProfile = picky["profile"]
	picky_profile.area_min_foes = 99
	checks["an out-of-range area_min_foes is clamped (6): two foes are not enough"] = _count(_plan(picky), "ai1_sweep") == 0
	var off: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_sweep"], foes_both, [], 1, 1, false)
	checks["switch off: no area step"] = _count(_plan(off), "ai1_sweep") == 0
	# A tile-aimed burst for a ranged unit.
	var burst: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_burst"], [Vector2i(7, 4), Vector2i(7, 5)], [], 3)
	var burst_steps: Array[PlanStep] = _plan(burst)
	checks["a ranged unit aims a burst at two clustered foes"] = _count(burst_steps, "ai1_burst") >= 1 and _valid(burst, burst_steps)
	var friendly: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(4, 4), ["ai1_burst"], [Vector2i(7, 4), Vector2i(7, 5)], [Vector2i(8, 4)], 3)
	var friendly_steps: Array[PlanStep] = _plan(friendly)
	var hit_ally: bool = false
	var friend_unit: Combatant = _unit_of(friendly, "allies", 1)
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
	var s: Dictionary = _scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1)
	var stunned: Combatant = _unit_of(s, "allies", 1)
	var wounded: Combatant = _unit_of(s, "allies", 2)
	stunned.statuses[StatusEffects.STUN] = true
	wounded.hp = 100
	var steps: Array[PlanStep] = _plan(s)
	var cleanse_at: int = _index_of(steps, "ai1_clean_a")
	var heal_at: int = _index_of(steps, "ai1_heal")
	checks["a stunned ally: a cleanse step targets it"] = cleanse_at >= 0 and steps[cleanse_at].target == stunned
	checks["the lowest tier that covers the stun is chosen"] = cleanse_at >= 0 and _count(steps, "ai1_clean_b") == 0
	checks["the cleanse comes before the heal step"] = cleanse_at >= 0 and heal_at > cleanse_at and steps[heal_at].target == wounded
	checks["the plan is valid"] = _valid(s, steps)
	# More debuffs than the first tier covers: the covering tier.
	var multi: Dictionary = _scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1)
	var multi_target: Combatant = _unit_of(multi, "allies", 1)
	multi_target.statuses[StatusEffects.STUN] = true
	multi_target.statuses[StatusEffects.BLEED] = true
	var multi_steps: Array[PlanStep] = _plan(multi)
	checks["stun and bleed: the tier that covers both"] = _count(multi_steps, "ai1_clean_b") == 1 and _count(multi_steps, "ai1_clean_a") == 0
	# Nothing cleansable: no cleanse step.
	var clean: Dictionary = _scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1)
	var clean_steps: Array[PlanStep] = _plan(clean)
	checks["nothing cleansable: no cleanse step"] = _count(clean_steps, "ai1_clean_a") + _count(clean_steps, "ai1_clean_b") == 0
	# Two stunned allies: still one cleanse per round.
	var twice: Dictionary = _scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1)
	_unit_of(twice, "allies", 1).statuses[StatusEffects.STUN] = true
	_unit_of(twice, "allies", 2).statuses[StatusEffects.STUN] = true
	var twice_steps: Array[PlanStep] = _plan(twice)
	checks["two stunned allies: never two cleanses in a round"] = _count(twice_steps, "ai1_clean_a") + _count(twice_steps, "ai1_clean_b") == 1
	# A debuff that does not lock the ally down: cleansed after the heal step.
	var late: Dictionary = _scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1)
	_unit_of(late, "allies", 1).statuses[StatusEffects.BLEED] = true
	_unit_of(late, "allies", 2).hp = 100
	var late_steps: Array[PlanStep] = _plan(late)
	var late_heal: int = _index_of(late_steps, "ai1_heal")
	checks["a non-locking debuff is cleansed after the heal"] = late_heal >= 0 and _index_of(late_steps, "ai1_clean_b") > late_heal
	# Any role can cleanse (not only the SUPPORT role).
	var fighter: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_clean_a"], far_foe, cells, 1)
	_unit_of(fighter, "allies", 1).statuses[StatusEffects.STUN] = true
	checks["an aggressor that owns a cleanse uses it on a stunned ally"] = _count(_plan(fighter), "ai1_clean_a") == 1
	var off: Dictionary = _scene(AiProfile.Role.SUPPORT, Vector2i(4, 4), kit, far_foe, cells, 1, 1, false)
	_unit_of(off, "allies", 1).statuses[StatusEffects.STUN] = true
	var off_steps: Array[PlanStep] = _plan(off)
	checks["switch off: no cleanse step"] = _count(off_steps, "ai1_clean_a") + _count(off_steps, "ai1_clean_b") == 0
	return TestHelpers.report(TAG, "Rule 4: cleanse-only skills", checks)

static func _test_taunt() -> bool:
	var checks: Dictionary = {}
	var ally_cells: Array[Vector2i] = [Vector2i(5, 5)]
	var foe_cells: Array[Vector2i] = [Vector2i(5, 4)]
	var s: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_taunt"], foe_cells, ally_cells)
	var steps: Array[PlanStep] = _plan(s)
	checks["an ally next to a foe: one taunt step"] = _count(steps, "ai1_taunt") == 1
	checks["the strike still follows"] = _count(steps, STRIKE) >= 1 and _valid(s, steps)
	var active: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_taunt"], foe_cells, ally_cells)
	var active_actor: Combatant = active["actor"]
	active_actor.statuses[StatusEffects.TAUNT] = true
	checks["already taunting (active): no taunt step"] = _count(_plan(active), "ai1_taunt") == 0
	var pending: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_taunt"], foe_cells, ally_cells)
	var pending_actor: Combatant = pending["actor"]
	pending_actor.pending_statuses[StatusEffects.TAUNT] = true
	checks["already taunting (pending): no taunt step"] = _count(_plan(pending), "ai1_taunt") == 0
	var distant: Array[Vector2i] = [Vector2i(0, 0)]
	var safe: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_taunt"], foe_cells, distant)
	checks["no ally near a foe: no taunt step"] = _count(_plan(safe), "ai1_taunt") == 0
	var off: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_taunt"], foe_cells, ally_cells, 1, 1, false)
	checks["switch off: no taunt step"] = _count(_plan(off), "ai1_taunt") == 0
	return TestHelpers.report(TAG, "Rule 5: taunt", checks)

static func _test_buff() -> bool:
	var checks: Dictionary = {}
	var foe_cells: Array[Vector2i] = [Vector2i(8, 4)]
	var ally_cells: Array[Vector2i] = [Vector2i(5, 7), Vector2i(6, 6), Vector2i(6, 7)]
	var s: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(6, 4), ["ai1_haste"], foe_cells, ally_cells, 3)
	var hasted: Combatant = _unit_of(s, "allies", 1)   # the farthest from the foe, but already hasted
	var near_one: Combatant = _unit_of(s, "allies", 2)
	var far_one: Combatant = _unit_of(s, "allies", 3)
	hasted.statuses[StatusEffects.HASTE] = true
	var steps: Array[PlanStep] = _plan(s)
	var cast: int = _index_of(steps, "ai1_haste")
	checks["two allies, one hasted already: the cast targets the other, farthest one"] = cast >= 0 and steps[cast].target == far_one and steps[cast].target != near_one
	checks["a strike follows and the plan is valid"] = _count(steps, STRIKE) >= 1 and _valid(s, steps)
	var all_done: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(6, 4), ["ai1_haste"], foe_cells, ally_cells, 3)
	for i: int in range(1, 4):
		_unit_of(all_done, "allies", i).statuses[StatusEffects.HASTE] = true
	checks["every ally already hasted: no step"] = _count(_plan(all_done), "ai1_haste") == 0
	var pending: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(6, 4), ["ai1_haste"], foe_cells, ally_cells, 3)
	for i: int in range(1, 4):
		_unit_of(pending, "allies", i).pending_statuses[StatusEffects.HASTE] = true
	checks["every ally already awaiting haste: no step"] = _count(_plan(pending), "ai1_haste") == 0
	var close: Array[Vector2i] = [Vector2i(7, 4), Vector2i(7, 5)]
	var engaged: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(6, 4), ["ai1_haste"], foe_cells, close, 3)
	checks["allies already next to the foe: no step"] = _count(_plan(engaged), "ai1_haste") == 0
	var off: Dictionary = _scene(AiProfile.Role.SNIPER, Vector2i(6, 4), ["ai1_haste"], foe_cells, ally_cells, 3, 1, false)
	checks["switch off: no step"] = _count(_plan(off), "ai1_haste") == 0
	return TestHelpers.report(TAG, "Rule 6: ally buffs", checks)

static func _test_debuff() -> bool:
	var checks: Dictionary = {}
	var foe_cells: Array[Vector2i] = [Vector2i(5, 4), Vector2i(4, 5), Vector2i(8, 8)]
	var s: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], foe_cells)
	var strong: Combatant = _unit_of(s, "foes", 0)
	var weak: Combatant = _unit_of(s, "foes", 1)
	var unreachable: Combatant = _unit_of(s, "foes", 2)
	strong.power_override = 30
	weak.power_override = 10
	unreachable.power_override = 50
	var steps: Array[PlanStep] = _plan(s)
	var cast: int = _index_of(steps, "ai1_intim")
	checks["the strongest foe in reach is the debuff target"] = cast >= 0 and steps[cast].target == strong
	var last_strike: int = -1
	for i: int in range(steps.size()):
		if steps[i].skill.id == STRIKE:
			last_strike = i
	checks["the main strike still follows the debuff"] = cast >= 0 and last_strike > cast
	checks["the plan is valid"] = _valid(s, steps)
	var done: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], foe_cells)
	_unit_of(done, "foes", 0).power_override = 30
	_unit_of(done, "foes", 1).power_override = 10
	_unit_of(done, "foes", 2).power_override = 50
	_unit_of(done, "foes", 0).statuses[StatusEffects.WEAKEN] = true
	var done_steps: Array[PlanStep] = _plan(done)
	var done_cast: int = _index_of(done_steps, "ai1_intim")
	checks["an already weakened foe is skipped"] = done_cast >= 0 and done_steps[done_cast].target == _unit_of(done, "foes", 1)
	var queued: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], foe_cells)
	_unit_of(queued, "foes", 0).power_override = 30
	_unit_of(queued, "foes", 1).power_override = 10
	_unit_of(queued, "foes", 0).pending_statuses[StatusEffects.WEAKEN] = true
	var queued_steps: Array[PlanStep] = _plan(queued)
	var queued_cast: int = _index_of(queued_steps, "ai1_intim")
	checks["a foe awaiting weaken is skipped too"] = queued_cast >= 0 and queued_steps[queued_cast].target == _unit_of(queued, "foes", 1)
	var both: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_intim"], [Vector2i(5, 4)])
	_unit_of(both, "foes", 0).statuses[StatusEffects.WEAKEN] = true
	checks["every foe in reach weakened: no cast"] = _count(_plan(both), "ai1_intim") == 0
	return TestHelpers.report(TAG, "Rule 7: debuff attacks", checks)

static func _first_defend(steps: Array[PlanStep]) -> String:
	for s: PlanStep in steps:
		if s.skill != null and s.skill.kind == SkillDef.Kind.DEFEND:
			return s.skill.id
	return ""

static func _test_riposte() -> bool:
	var checks: Dictionary = {}
	var kit: Array[String] = ["ai1_guard", "ai1_rip"]
	var s: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(5, 4)])
	var low: Combatant = s["actor"]
	low.hp = 100
	checks["low HP facing a melee foe: riposte rather than the plain guard"] = _first_defend(_plan(s)) == "ai1_rip"
	var ranged: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(6, 4)], [], 1, 3)
	var low_ranged: Combatant = ranged["actor"]
	low_ranged.hp = 100
	checks["low HP facing a ranged foe: the plain guard"] = _first_defend(_plan(ranged)) == "ai1_guard"
	var holding: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(5, 4)])
	var holding_actor: Combatant = holding["actor"]
	holding_actor.hp = 100
	holding_actor.statuses[StatusEffects.RIPOSTE] = true
	checks["already riposting: the plain guard"] = _first_defend(_plan(holding)) == "ai1_guard"
	var off: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), kit, [Vector2i(5, 4)], [], 1, 1, false)
	var off_actor: Combatant = off["actor"]
	off_actor.hp = 100
	var off_first: String = _first_defend(_plan(off))
	checks["switch off: the old highest-level guard pick (no riposte preference)"] = off_first == "ai1_rip" or off_first == "ai1_guard"
	# Healthy: strike first, then close the round in a riposte stance (once; a later strike would cancel it).
	var healthy: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(5, 4)])
	var steps: Array[PlanStep] = _plan(healthy)
	checks["healthy, melee foe adjacent: a riposte closes the round"] = steps.size() >= 2 and steps[steps.size() - 1].skill.id == "ai1_rip" and _count(steps, "ai1_rip") == 1
	checks["exactly one strike precedes it and the plan is valid"] = _count(steps, STRIKE) == 1 and _valid(healthy, steps)
	var reckless: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(5, 4)])
	var reckless_profile: AiProfile = reckless["profile"]
	reckless_profile.reckless = true
	checks["a reckless unit never ripostes"] = _count(_plan(reckless), "ai1_rip") == 0
	var ranged_foe: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(5, 4)], [], 1, 3)
	checks["a ranged foe next to it: no riposte"] = _count(_plan(ranged_foe), "ai1_rip") == 0
	var already: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(5, 4)])
	var already_actor: Combatant = already["actor"]
	already_actor.statuses[StatusEffects.RIPOSTE] = true
	checks["already riposting: no second cast"] = _count(_plan(already), "ai1_rip") == 0
	var out_of_reach: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(0, 0), ["ai1_rip"], [Vector2i(8, 8)])
	checks["no strike this round and no foe adjacent: no riposte"] = _count(_plan(out_of_reach), "ai1_rip") == 0
	var charging: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(7, 4)])
	var charge_steps: Array[PlanStep] = _plan(charging)
	checks["a unit that walks up and strikes ends next to the melee foe: riposte closes the round"] = _count(charge_steps, "ai1_rip") == 1 and charge_steps[charge_steps.size() - 1].skill.id == "ai1_rip"
	var all_out: Dictionary = _scene(AiProfile.Role.AGGRESSOR, Vector2i(4, 4), ["ai1_rip"], [Vector2i(5, 4)])
	var all_out_profile: AiProfile = all_out["profile"]
	all_out_profile.defend_below = 0.0
	checks["defend_below 0 (never guards, the all-out preset): no riposte"] = _count(_plan(all_out), "ai1_rip") == 0
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
		for id: Variant in (kits[i] as Array):
			kit.append(str(id))
		out.append(_unit("S%dU%d" % [side, i], grid, cell, kit, 1 if i == 0 else 3, 240))
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
					ctx.profiles[member] = _profile(roles[i])
		for team: Array in sides:
			for member_raw: Variant in team:
				var member: Combatant = member_raw
				if not member.is_active() or not grid.has_combatant(member):
					continue
				member.ap = member.max_ap
				var profile: AiProfile = ctx.profile_of(member)
				var first: Array[PlanStep] = TacticalAI.build_steps(member, ctx, profile)
				var again: Array[PlanStep] = TacticalAI.build_steps(member, ctx, profile)
				if _sig(first) != _sig(again):
					invalid += 1   # same state must give the same plan
				var check := CombatPlan.new(ctx)
				check.set_steps(member, first)
				var st: CombatPlan.SimState = check.compile(member)
				if st.error_index >= 0 or st.runs_dry():
					invalid += 1
				if first.size() > TacticalAI.MAX_STEPS:
					oversize += 1
				log_sig.append(_sig(first))
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
