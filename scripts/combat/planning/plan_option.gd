class_name PlanOption
extends RefCounted

# ==============================================================================
# Tactical RPG - Plan Option (one candidate next input, probed against the plan)
# Built by CombatPlan.options(): the step, whether it is valid as the next step
# of the sequence, why not if it isn't, and what it would cost.
# ==============================================================================

var step: PlanStep = null
var valid: bool = false
var reason: String = ""
var ap: int = 0        # total AP the step adds (including auto-inserted moves)
var blocks: int = 0    # total blocks the step adds
var moves: int = 0     # auto/manual move entries it adds

func cost_text() -> String:
	if not valid:
		return reason
	var text: String = "%d AP • %d block%s" % [ap, blocks, "" if blocks == 1 else "s"]
	if step.mode == PlanStep.Mode.ENGAGE and moves > 0:
		text += " (incl. %d move%s)" % [moves, "" if moves == 1 else "s"]
	return text
