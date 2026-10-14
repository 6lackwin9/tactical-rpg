class_name SupportRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Support Rules (heals and buffs; deliberately NOT DamageCalculator)
# heal = heal_base + round(heal_stat x heal_mult), where heal_stat is the caster's
# effective INT or KNOW. Healing never exceeds max HP (Combatant.heal clamps) and
# never revives. Buff statuses (haste, taunt) are queued like every status: they
# become active next round and last `status_rounds` rounds.
# ==============================================================================

const HEAL_STATS: Array[String] = ["INT", "KNOW"]

# Points a SUPPORT skill restores when `caster` uses it (0 for non-healing skills).
static func heal_amount(caster: Combatant, skill: SkillDef) -> int:
	if skill == null or not skill.heals() or caster == null:
		return 0
	var stat_value: int = 0
	if not skill.heal_stat.is_empty() and caster.stats != null:
		stat_value = caster.stats.get_effective_stat(skill.heal_stat, caster.get_weapon_context())
	return maxi(0, skill.heal_base + roundi(float(stat_value) * skill.heal_mult))

# HP `target` would actually regain from `amount` (clamped to its missing HP).
static func effective_heal(target: Combatant, amount: int) -> int:
	if target == null or not target.is_alive():
		return 0
	return clampi(amount, 0, target.max_hp - target.hp)

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
	if skill.heals():
		var before: int = target.hp
		target.heal(heal_amount(caster, skill))
		healed = target.hp - before
	if not skill.grant_status.is_empty():
		StatusEffects.queue(target, skill.grant_status, rounds)
		granted = skill.grant_status
	return {"healed": healed, "status": granted}
