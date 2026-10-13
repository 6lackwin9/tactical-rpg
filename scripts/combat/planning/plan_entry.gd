class_name PlanEntry
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan Entry (one compiled command on a unit's 10-block timeline)
# Produced by CombatPlan from PlanSteps; build_command() turns it into the real
# BattleCommand the resolver executes. `step_index` points back to the step that
# produced it (an Engage step yields moves plus the attack).
# ==============================================================================

var skill: SkillDef = null
var start_block: int = 1
var direction: Vector2i = Vector2i.ZERO
var target: Combatant = null
var power: int = 20        # the unit's base strike power (the skill scales it)
var step_index: int = 0
var ap: int = 1            # AP charged for this entry (after unit effects)

static func create(p_skill: SkillDef, p_start: int, p_direction: Vector2i, p_target: Combatant, p_power: int, p_step_index: int, p_ap: int) -> PlanEntry:
	var entry := PlanEntry.new()
	entry.skill = p_skill
	entry.start_block = p_start
	entry.direction = p_direction
	entry.target = p_target
	entry.power = p_power
	entry.step_index = p_step_index
	entry.ap = p_ap
	return entry

func blocks() -> int:
	return skill.blocks

func end_block() -> int:
	return start_block + skill.blocks - 1

func build_command(actor: Combatant, grid: CombatGrid) -> BattleCommand:
	return skill.build_command(actor, grid, direction, target, power)

static func dir_arrow(dir: Vector2i) -> String:
	if dir == Vector2i(1, 0):
		return "→"
	if dir == Vector2i(-1, 0):
		return "←"
	if dir == Vector2i(0, -1):
		return "↑"
	if dir == Vector2i(0, 1):
		return "↓"
	return "·"
