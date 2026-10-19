class_name SupportRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Support Rules (heals and buffs; deliberately NOT DamageCalculator)
# heal = heal_base + round(heal_stat x heal_mult), where heal_stat is the caster's
# effective INT or KNOW. Healing never exceeds max HP (Combatant.heal clamps) and
# never revives. Buff statuses (haste, taunt, fortify, regen) are queued like every status: they
# become active next round and last `status_rounds` rounds, carrying the skill's status_magnitude
# and the caster's name (Combatant.pending_status_sources).
# ==============================================================================

const HEAL_STATS: Array[String] = ["INT", "KNOW"]

# Points a SUPPORT skill restores when `caster` uses it (0 for non-healing skills).
static func heal_amount(caster: Combatant, skill: SkillDef) -> int:
	if skill == null or not skill.heals() or caster == null:
		return 0
	var stat_value: int = 0
	if not skill.heal_stat.is_empty() and caster.stats != null:
		stat_value = caster.stats.get_effective_stat(skill.heal_stat, caster.get_weapon_context())
	var base: int = skill.heal_base + roundi(float(stat_value) * skill.heal_mult)
	return maxi(0, roundi(float(base) * caster.potency_for_skill(skill)))   # potency 1.0 = unchanged

# HP `target` would actually regain from `amount` (clamped to its missing HP).
static func effective_heal(target: Combatant, amount: int) -> int:
	if target == null or not target.is_alive():
		return 0
	return clampi(amount, 0, target.max_hp - target.hp)

# Debuffs `skill` would remove from `target` right now (active or pending), in StatusEffects.debuffs() order.
static func cleansable(target: Combatant, skill: SkillDef) -> Array[String]:
	var out: Array[String] = []
	if target == null or skill == null:
		return out
	for status: String in skill.cleanse_list():
		if StatusEffects.has_any(target, status):
			out.append(status)
	return out

# True when `target` is a legal target for `skill` cast by `caster`, ignoring distance.
static func target_problem(caster: Combatant, skill: SkillDef, target: Combatant, friends: Array[Combatant]) -> String:
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

# Applies the skill to `target`; returns what happened for the log.
static func apply(caster: Combatant, target: Combatant, skill: SkillDef, rounds: int) -> Dictionary:
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
	# Extra grants (BAL5-L) go to the same target, queued like the main grant.
	var extras: Array[String] = StatusEffects.grant_extras(target, skill.extra_grants, false, caster.name if caster != null else "")
	return {"healed": healed, "status": granted, "cleansed": cleansed, "extras": extras}
