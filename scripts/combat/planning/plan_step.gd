class_name PlanStep
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan Step (one input in a unit's round sequence)
# What the player (or AI) chooses; CombatPlan compiles each step into timeline
# entries at the lowest free blocks.
#   ENGAGE    attack skill + target; the moves needed to get in range are
#             inserted automatically
#   MOVE      manual reposition: move skill + direction
#   WITHDRAW  auto-path to the exit tile and leave the battle
#   SELF      Defend / Focus / Reset Stance
# ==============================================================================

enum Mode { ENGAGE, MOVE, WITHDRAW, SELF }

var mode: Mode = Mode.SELF
var skill: SkillDef = null
var target: Combatant = null
var direction: Vector2i = Vector2i.ZERO

static func engage(p_skill: SkillDef, p_target: Combatant) -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.ENGAGE
	step.skill = p_skill
	step.target = p_target
	return step

static func move(p_skill: SkillDef, p_direction: Vector2i) -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.MOVE
	step.skill = p_skill
	step.direction = p_direction
	return step

static func withdraw() -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.WITHDRAW
	return step

static func self_skill(p_skill: SkillDef) -> PlanStep:
	var step := PlanStep.new()
	step.mode = Mode.SELF
	step.skill = p_skill
	return step

# Short chip text for the sequence preview.
func title() -> String:
	match mode:
		Mode.ENGAGE:
			return "%s → %s" % [skill.label(), target.name if target != null else "?"]
		Mode.MOVE:
			return "%s %s" % [skill.label(), PlanEntry.dir_arrow(direction)]
		Mode.WITHDRAW:
			return "🏃 Withdraw"
	return skill.label()

func duplicate_step() -> PlanStep:
	var copy := PlanStep.new()
	copy.mode = mode
	copy.skill = skill
	copy.target = target
	copy.direction = direction
	return copy
