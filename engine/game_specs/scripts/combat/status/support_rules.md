---
class: SupportRules
extends: RefCounted
description: "Tactical RPG - Support Rules (heals and buffs; deliberately NOT DamageCalculator) heal = heal_base + round(heal_stat x heal_mult), where heal_stat is the caster's effective INT or KNOW. Healing never exceeds max HP (Combatant.heal clamps) and never revives. Buff statuses (haste, taunt, fortify, regen) are queued like every status: they become active next round and last `status_rounds` rounds, carrying the skill's status_magnitude and the caster's name (Combatant.pending_status_sources)."
---

# Properties
* `HEAL_STATS`: Array[String] = ["INT", "KNOW"] (const)
* `out`: Array[String] = []

# Methods

### `static heal_amount(caster: Combatant, skill: SkillDef) -> int`
```logic
if skill == null or not skill.heals() or caster == null:
	return 0
var stat_value: int = 0
if not skill.heal_stat.is_empty() and caster.stats != null:
	stat_value = caster.stats.get_effective_stat(skill.heal_stat, caster.get_weapon_context())
var base: int = skill.heal_base + roundi(float(stat_value) * skill.heal_mult)
return maxi(0, roundi(float(base) * caster.potency_for_skill(skill)))   # potency 1.0 = unchanged
```

### `static effective_heal(target: Combatant, amount: int) -> int`
```logic
if target == null or not target.is_alive():
	return 0
return clampi(amount, 0, target.max_hp - target.hp)
```

### `static target_problem(caster: Combatant, skill: SkillDef, target: Combatant, friends: Array[Combatant]) -> String`
```logic
if target == null or not target.is_active():
	return "Choose a friendly unit."
if target == caster:
	if skill.target_mode == SkillDef.TargetMode.ALLY:
		return "%s can only be used on another ally." % skill.display_name
	return ""
if skill.target_mode == SkillDef.TargetMode.SELF:
	return "%s can only be used on %s itself." % [skill.display_name, caster.name]
if not friends.has(target):
	return "%s is not an ally." % target.name
return ""
```

### `static apply(caster: Combatant, target: Combatant, skill: SkillDef, rounds: int) -> Dictionary`
```logic
var healed: int = 0
var granted: String = ""
var cleansed: Array[String] = []
# Cleanse first: a skill that both removes and grants the same kind of status keeps the new one.
for status: String in skill.cleanse_list():
	if StatusEffects.remove(target, status):
		cleansed.append(status)
if skill.heals():
	var before: int = target.hp
	target.heal(heal_amount(caster, skill))
	healed = target.hp - before
if not skill.grant_status.is_empty():
	StatusEffects.queue(target, skill.grant_status, rounds, skill.status_magnitude, caster.name if caster != null else "")
	granted = skill.grant_status
return {"healed": healed, "status": granted, "cleansed": cleansed}
```

