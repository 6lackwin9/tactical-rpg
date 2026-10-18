---
class: PlanOption
extends: RefCounted
description: "Tactical RPG - Plan Option (one candidate next input, probed against the plan) Built by CombatPlan.options(): the step, whether it is valid as the next step of the sequence, why not if it isn't, and what it would cost."
---

# Properties
* `step`: PlanStep = null
* `valid`: bool = false
* `reason`: String = ""
* `warning`: String = ""   # valid but risky (e.g. the unit would run out of AP here)
* `ap`: int = 0            # AP the step adds (negative: Rest recovers), incl. auto stance switches
* `blocks`: int = 0        # total blocks the step adds
* `moves`: int = 0         # move entries it adds
* `switch_ap`: int = 0     # of `ap`, the automatic stance-switch penalty
* `damage`: int = 0         # expected hit for attack options
* `switches_target`: bool = false
* `heal`: int = 0           # expected HP restored (support options)
* `grants`: String = ""     # buff status the option grants (support options)
* `cleanses`: Array[String] = []   # debuffs the option would remove right now (support options)
* `jump_to`: Vector2i = Vector2i(-1, -1)   # disengage options: the landing tile the planner chose
* `hazard`: String = ""     # "Hazard: <terrain name> (<status>)" chips of hazard tiles the step's route enters (S12T-L)
* `cover`: float = 0.0      # attack options: the terrain cover in the expected hit (S12T-L)
* `flow`: String = ""       # "flows into Defensive" when the step's skill is a flow move (S12C-L)
* `exploits`: String = ""   # display name of the status the attack's expected hit exploits ("" = none; S12C-L)
* `item_left`: int = 0      # ITEM options (ITEMUSE1): pouch copies of the step's item still unplanned this round
* `label`: String = ""      # ITEM options: "Use <item> (<n> left)" (the action popup shows it)
* `out`: Array[Dictionary] = []
* `fam`: String = o.family()
* `group`: Dictionary = {}
* `list`: Array[PlanOption] = []
* `members`: Array[PlanOption] = group["options"]
* `members`: Array[PlanOption] = g["options"]
* `tiers`: Array[SkillDef] = []

# Methods

### `family() -> String`
```logic
return "" if step == null or step.skill == null else step.skill.family_id()
```

### `tier() -> int`
```logic
return 0 if step == null or step.skill == null else step.skill.tier
```

### `cost_text() -> String`
```logic
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
```

