---
class: SupportCommand
extends: BattleCommand
description: "Tactical RPG - Support Command (heal and/or buff a friendly unit, or yourself) Lands at the end of its last block. The target must be alive, on the field and in the skill's reach when it lands (positions may have changed since planning). Heals go through SupportRules (not DamageCalculator); buffs are queued statuses."
---

# Properties
* `target`: Combatant = null
* `skill`: SkillDef = null
* `grid`: CombatGrid = null
* `est_heal`: int = 0           # planner's expected heal (clamped to missing HP), for previews
* `last_heal`: int = 0
* `last_status`: String = ""
* `last_cleansed`: Array[String] = []

# Methods

### `_init(p_source: Combatant, p_target: Combatant, p_skill: SkillDef, p_grid: CombatGrid, p_block_cost: int = 1) -> void`
```logic
super(p_source, 0, p_block_cost)
target = p_target if p_target != null else p_source
skill = p_skill
grid = p_grid
command_name = p_skill.display_name if p_skill != null else "Support"
```

### `in_reach() -> bool`
```logic
if grid == null or target == source:
	return true
if not grid.has_combatant(source) or not grid.has_combatant(target):
	return false
var band: Vector2i = skill.support_reach(source)
var dist: int = grid.get_distance(grid.get_position(source), grid.get_position(target))
return dist >= band.x and dist <= band.y
```

### `can_execute() -> bool`
```logic
return super.can_execute() and source.is_active() and target != null and target.is_active() and in_reach()
```

### `execute() -> void`
```logic
last_heal = 0
last_status = ""
last_cleansed = []
failed = false
if not source.is_active():
	failed = true
	execution_log = "[Action Failed] %s cannot act (unit is inactive)." % source.name
	return
if target == null or not target.is_active():
	failed = true
	execution_log = "[Support Failed] %s's %s has no target left." % [source.name, command_name]
	return
if not in_reach():
	failed = true
	execution_log = "[Support Failed] %s cannot reach %s with %s." % [source.name, target.name, command_name]
	return
var result: Dictionary = SupportRules.apply(source, target, skill, skill.status_rounds)
last_heal = DictUtil.to_int(result["healed"])
last_status = str(result["status"])
last_cleansed = ContentFactory._strings(result["cleansed"])
var parts: Array[String] = []
if skill.heals():
	parts.append("healed %d (HP %d/%d)" % [last_heal, target.hp, target.max_hp])
if skill.cleanses():
	var names: Array[String] = []
	for status: String in last_cleansed:
		names.append(StatusEffects.display_name(status))
	parts.append("cleansed %s" % (", ".join(names) if not names.is_empty() else "nothing"))
if not last_status.is_empty():
	parts.append("%s next round%s" % [StatusEffects.display_name(last_status), "" if skill.status_rounds <= 1 else (" for %d rounds" % skill.status_rounds)])
var who: String = "themselves" if target == source else target.name
execution_log = "[Support] %s used %s on %s: %s" % [source.name, command_name, who, ", ".join(parts) if not parts.is_empty() else "no effect"]
```

