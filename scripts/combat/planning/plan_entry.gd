class_name PlanEntry
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan Entry (one compiled command on a unit's 10-block timeline)
# Produced by CombatPlan from PlanSteps; build_command() turns it into the real
# planner-controlled BattleCommand: the resolver charges its AP (skill + any
# stance switch) when it starts, and the sequence stops if the unit can't pay.
# `step_index` points back to the step that produced it (an Engage step yields
# moves plus the attack).
# ==============================================================================

var skill: SkillDef = null
var start_block: int = 1
var direction: Vector2i = Vector2i.ZERO   # display hint for a move (first step)
var path: Array[Vector2i] = []            # MOVE: the unit steps taken
var tiles: int = 0                        # MOVE: tiles covered
var target: Combatant = null              # ATTACK target, or the chase target of an Engage move
var power: int = 20                       # the unit's base strike power (the skill scales it)
var step_index: int = 0
var pos_before: Vector2i = Vector2i(-1, -1)  # where the unit stands before / after this command (preview ghosts)
var pos_after: Vector2i = Vector2i(-1, -1)
var skill_ap: int = 0                     # AP for the skill itself
var switch_ap: int = 0                    # automatic stance-switch penalty
var stance_before: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL
var stance_after: CombatTypes.StanceType = CombatTypes.StanceType.NEUTRAL

static func create(p_skill: SkillDef, p_start: int, p_step_index: int) -> PlanEntry:
	var entry := PlanEntry.new()
	entry.skill = p_skill
	entry.start_block = p_start
	entry.step_index = p_step_index
	return entry

func total_ap() -> int:
	return skill_ap + switch_ap

func blocks() -> int:
	return skill.blocks

func end_block() -> int:
	return start_block + skill.blocks - 1

func build_command(actor: Combatant, grid: CombatGrid) -> BattleCommand:
	var command: BattleCommand
	match skill.kind:
		SkillDef.Kind.ATTACK:
			command = AttackCommand.new(actor, target, maxi(1, roundi(float(power) * skill.power_mult)), skill.blocks, 0, grid)
		SkillDef.Kind.DEFEND:
			command = DefendCommand.new(actor, skill.mitigation, skill.blocks, 1)
		SkillDef.Kind.FOCUS:
			command = FocusCommand.new(actor, skill.boost, skill.boost_rounds, skill.blocks, 0)
		SkillDef.Kind.MOVE:
			if target != null:
				command = ApproachCommand.new(actor, grid, target, tiles, 1)
			else:
				command = WalkCommand.new(actor, grid, path, 1)
		_:
			command = RestCommand.new(actor, skill.recover_ap, skill.blocks)
	command.command_name = skill.display_name
	command.planner_controlled = true
	command.ap_cost = skill_ap
	command.stance_switch_cost = switch_ap
	command.stance_on_start = -1 if skill.kind == SkillDef.Kind.MOVE else skill.stance()
	return command

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
