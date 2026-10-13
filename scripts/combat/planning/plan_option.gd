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
var warning: String = ""   # valid but risky (e.g. the unit would run out of AP here)
var ap: int = 0            # AP the step adds (negative: Rest recovers), incl. auto stance switches
var blocks: int = 0        # total blocks the step adds
var moves: int = 0         # move entries it adds
var switch_ap: int = 0     # of `ap`, the automatic stance-switch penalty
var damage: int = 0         # expected hit for attack options
var switches_target: bool = false

func cost_text() -> String:
	if not valid:
		return reason
	var text: String = ("+%d AP" % -ap) if ap < 0 else ("%d AP" % ap)
	text += " • %d block%s" % [blocks, "" if blocks == 1 else "s"]
	if step.mode == PlanStep.Mode.ENGAGE and moves > 0:
		text += " (incl. %d move%s)" % [moves, "" if moves == 1 else "s"]
	if switch_ap > 0:
		text += " (incl. +%d stance switch)" % switch_ap
	if switches_target:
		text += " (incl. Switch Target)"
	if damage > 0:
		text += " • ~%d dmg" % damage
	if not warning.is_empty():
		text += "\n⚠ " + warning
	return text