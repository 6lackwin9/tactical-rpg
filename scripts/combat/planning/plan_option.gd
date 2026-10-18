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
var heal: int = 0           # expected HP restored (support options)
var grants: String = ""     # buff status the option grants (support options)
var cleanses: Array[String] = []   # debuffs the option would remove right now (support options)
var jump_to: Vector2i = Vector2i(-1, -1)   # disengage options: the landing tile the planner chose
var hazard: String = ""     # "Hazard: <terrain name> (<status>)" chips of hazard tiles the step's route enters (S12T-L)
var cover: float = 0.0      # attack options: the terrain cover in the expected hit (S12T-L)
var flow: String = ""       # "flows into Defensive" when the step's skill is a flow move (S12C-L)
var exploits: String = ""   # display name of the status the attack's expected hit exploits ("" = none; S12C-L)
var item_left: int = 0      # ITEM options (ITEMUSE1): pouch copies of the step's item still unplanned this round

# The skill family of the option's step ("" for steps without a skill, e.g. withdraw) and its tier.
func family() -> String:
	return "" if step == null or step.skill == null else step.skill.family_id()

func tier() -> int:
	return 0 if step == null or step.skill == null else step.skill.tier

# Groups options into [{"family": id, "name": text, "label": "Mend I / II / III", "options": Array[PlanOption]}]
# in first-seen order; options of one family and target stay together, tiers lowest first. Skill-less steps are skipped.
# (A group holds one option per affordable tier per target, so UIs list tiers of the clicked target side by side.)
static func group_by_family(options: Array[PlanOption]) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for o: PlanOption in options:
		var fam: String = o.family()
		if fam.is_empty():
			continue
		var group: Dictionary = {}
		for g: Dictionary in out:
			if str(g["family"]) == fam:
				group = g
		if group.is_empty():
			var list: Array[PlanOption] = []
			group = {"family": fam, "name": SkillCatalog.family_name(fam), "label": "", "options": list}
			out.append(group)
		var members: Array[PlanOption] = group["options"]
		members.append(o)
	for g: Dictionary in out:
		var members: Array[PlanOption] = g["options"]
		members.sort_custom(func(a: PlanOption, b: PlanOption) -> bool: return a.tier() < b.tier())
		var tiers: Array[SkillDef] = []
		for o: PlanOption in members:
			if not tiers.has(o.step.skill):
				tiers.append(o.step.skill)
		g["label"] = SkillCatalog.group_label({"name": g["name"], "tiers": tiers})
	return out

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
		if cover > 0.0:
			text += " (cover -%d%%)" % roundi(cover * 100.0)
		if not exploits.is_empty():
			text += " (exploits %s)" % exploits
	if heal > 0:
		text += " • ~+%d HP" % heal
	if not cleanses.is_empty():
		var names: Array[String] = []
		for status: String in cleanses:
			names.append(StatusEffects.display_name(status))
		text += " • cleanses %s" % ", ".join(names)
	if not grants.is_empty():
		text += " • %s" % StatusEffects.label_with_magnitude(grants, StatusEffects.magnitude_or_min(grants, step.skill.status_magnitude if step != null and step.skill != null else 0))
	if jump_to != Vector2i(-1, -1):
		text += " • jumps to %s" % str(jump_to)
	if not flow.is_empty():
		text += " • " + flow
	if not warning.is_empty():
		text += "\n⚠ " + warning
	if not hazard.is_empty():
		text += "\n⚠ " + hazard
	return text