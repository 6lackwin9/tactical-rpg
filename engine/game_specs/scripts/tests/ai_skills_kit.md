---
class: AiSkillsKit
extends: RefCounted
description: "Shared fixtures and scene helpers of the AI1 suites (AiSkillsTestSuite, AiSkillsTacticsTestSuite; split by CLN-12). Every skill is an \"ai1_*\" row of a sandbox catalog fork (the shipped catalog is never touched)."
---

# Properties
* `TAG`: String = "AI1" (const)
* `STRIKE`: String = "strike" (const)
* `out`: Array[String] = []
* `actor`: Combatant = scene["actor"]
* `ctx`: PlanContext = scene["ctx"]
* `profile`: AiProfile = scene["profile"]

# Methods

### `static setup(with_report: bool) -> bool`
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
	if not errors.is_empty():
		print("    [AI1] fixture errors: %s" % str(errors))
	checks["every ai1_* fixture row is accepted by the catalog schema"] = errors.is_empty()
	SkillCatalog.clear_cache()
	checks["the fixtures are visible to SkillCatalog"] = SkillCatalog.find("ai1_back") != null and SkillCatalog.find("ai1_intim") != null and not SkillCatalog.find("ai1_intim").deals_damage() and SkillCatalog.find("ai1_focus") != null and SkillCatalog.find("ai1_heavy") != null
	checks["the helper lists find them"] = has_skill(SkillCatalog.cleanse_only(3), "ai1_clean_a") and has_skill(SkillCatalog.cleanse_only(3), "ai1_clean_b") and not has_skill(SkillCatalog.cleanse_only(3), "ai1_heal") and has_skill(SkillCatalog.debuff_attacks(3), "ai1_intim") and has_skill(SkillCatalog.granting(SkillDef.Kind.SUPPORT, StatusEffects.TAUNT, 3), "ai1_taunt") and has_skill(SkillCatalog.ally_buffs(StatusEffects.TAUNT, 3), "ai1_haste") and not has_skill(SkillCatalog.ally_buffs(StatusEffects.TAUNT, 3), "ai1_taunt") and has_skill(SkillCatalog.granting(SkillDef.Kind.DEFEND, StatusEffects.RIPOSTE, 3), "ai1_rip")
var profile := AiProfile.create(AiProfile.Role.AGGRESSOR)
checks["AiProfile default: area_min_foes 2"] = profile.area_min_foes == 2
profile.area_min_foes = 5
var copy: AiProfile = profile.duplicate_profile()
checks["duplicate_profile copies area_min_foes"] = copy.area_min_foes == 5
profile.area_min_foes = 0
checks["area_min_foes is clamped to 1..6 on use"] = profile.area_threshold() == 1
profile.area_min_foes = 99
checks["(upper bound)"] = profile.area_threshold() == 6
if not with_report and TestHelpers.failed(checks).is_empty():
	return true
return TestHelpers.report(TAG, "Sandbox fixtures and profile fields", checks)
```

### `static has_skill(skills: Array[SkillDef], skill_id: String) -> bool`
```logic
for s: SkillDef in skills:
	if s.id == skill_id:
		return true
return false
```

### `static teardown() -> void`
```logic
TestHelpers.cleanup(SANDBOX)
```

### `static unit(unit_name: String, grid: CombatGrid, pos: Vector2i, kit: Array[String], attack_range: int = 1, hp: int = 400, power: int = 20, with_basics: bool = true) -> Combatant`
```logic
var known: Array[String] = basics() if with_basics else []
known.append_array(kit)
var c: Combatant = TestHelpers.unit(unit_name, grid, pos, hp, 10, attack_range, power)
c.known_skills = known
return c
```

### `static ai_profile(role: AiProfile.Role) -> AiProfile`
```logic
return AiProfile.create(role, 0.1, 3)
```

### `static new_scene(role: AiProfile.Role, pos: Vector2i, kit: Array[String], foe_cells: Array[Vector2i], ally_cells: Array[Vector2i] = [], actor_range: int = 1, foe_range: int = 1) -> Dictionary`
```logic
var grid := CombatGrid.new(CombatGrid.ArenaType.CLOSED, [])
var actor: Combatant = unit("Actor", grid, pos, kit, actor_range)
var allies: Array[Combatant] = [actor]
var foes: Array[Combatant] = []
for i: int in range(ally_cells.size()):
	allies.append(unit("Ally%d" % i, grid, ally_cells[i], [], 1))
for i: int in range(foe_cells.size()):
	foes.append(unit("Foe%d" % i, grid, foe_cells[i], [], foe_range))
var ctx: PlanContext = TestHelpers.hand_ctx(grid, allies, foes, 3)
var profile: AiProfile = ai_profile(role)
ctx.profiles[actor] = profile
return {"grid": grid, "actor": actor, "allies": allies, "foes": foes, "ctx": ctx, "profile": profile}
```

### `static first_attack(scene: Dictionary, steps: Array[PlanStep]) -> PlanEntry`
```logic
var actor: Combatant = scene["actor"]
var ctx: PlanContext = scene["ctx"]
var check := CombatPlan.new(ctx)
check.set_steps(actor, steps)
for entry: PlanEntry in check.compile(actor).entries:
	if entry.skill != null and entry.skill.kind == SkillDef.Kind.ATTACK:
		return entry
return null
```

### `static unit_of(scene: Dictionary, key: String, index: int) -> Combatant`
```logic
var list: Array[Combatant] = scene[key]
return list[index]
```

### `static sig(steps: Array[PlanStep]) -> String`
```logic
var parts: Array[String] = []
for s: PlanStep in steps:
	parts.append("%d:%s:%s:%s:%d:%s" % [s.mode, s.skill.id if s.skill != null else "-", s.target.name if s.target != null else "-", str(s.direction), s.tiles, str(s.cell)])
return "|".join(parts)
```

### `static count_of(steps: Array[PlanStep], skill_id: String) -> int`
```logic
var n: int = 0
for s: PlanStep in steps:
	if s.skill != null and s.skill.id == skill_id:
		n += 1
return n
```

### `static index_of(steps: Array[PlanStep], skill_id: String) -> int`
```logic
for i: int in range(steps.size()):
	if steps[i].skill != null and steps[i].skill.id == skill_id:
		return i
return -1
```

### `static valid(scene: Dictionary, steps: Array[PlanStep]) -> bool`
```logic
var actor: Combatant = scene["actor"]
var ctx: PlanContext = scene["ctx"]
var check := CombatPlan.new(ctx)
check.set_steps(actor, steps)
var st: CombatPlan.SimState = check.compile(actor)
return st.error_index < 0 and not st.runs_dry() and steps.size() <= TacticalAI.MAX_STEPS
```

