class_name Passives
extends RefCounted

# ==============================================================================
# Tactical RPG - Passives (inherited, lore-granted traits; replaces mastery)
# A passive is a lore-free template in the content catalog ("passives"): a list of
# effects. Nothing in a class or base unit grants one; SOURCES do: a world
# character's background, a faction, a quest reward, a lore discovery, a named
# item. Passives.apply() puts the effects on a Combatant.
#
# Effect types (each effect is a Dictionary with "type"):
#   stat         {"stat": "KNOW", "value": 3, "context": {"biome": "ruins"}}
#                flat stat bonus, optionally only in a context (empty = always)
#   proficiency  {"gear": "sword", "levels": 1}   gear-training levels (Proficiency)
#   resist       {"status": "stagger", "value": 25}   status resistance % (100 = immune)
#   ap_regen     {"value": 1}   extra AP regained each round
#   skill        {"skill": "quick_stab"}   a skill the unit may use beyond its class path
# ==============================================================================

const TYPES: Array[String] = ["stat", "proficiency", "resist", "ap_regen", "skill"]

# Problems with an effects list (empty = valid). `skill_exists` resolves skill ids.
static func validate_effects(effects: Variant, skill_exists: Callable) -> Array[String]:
	var errors: Array[String] = []
	if not (effects is Array):
		errors.append("Effects must be a list.")
		return errors
	var list: Array = effects
	if list.is_empty():
		errors.append("A passive needs at least one effect.")
	for i: int in range(list.size()):
		var e: Dictionary = DictUtil.dict_of(list[i])
		var type: String = str(e.get("type", ""))
		var at: String = "Effect %d" % (i + 1)
		match type:
			"stat":
				if not StatRules.STATS.has(str(e.get("stat", ""))):
					errors.append("%s: unknown stat '%s'." % [at, str(e.get("stat", ""))])
				if DictUtil.to_int(e.get("value", 0)) == 0:
					errors.append("%s: the stat bonus needs a non-zero value." % at)
				var ctx: Variant = e.get("context", {})
				if not (ctx is Dictionary) or (ctx as Dictionary).size() > 1:
					errors.append("%s: context is one {key: value} pair (or empty)." % at)
			"proficiency":
				if not Proficiency.gear_types().has(str(e.get("gear", ""))):
					errors.append("%s: unknown gear type '%s'." % [at, str(e.get("gear", ""))])
				if DictUtil.to_int(e.get("levels", 0)) < 1 or DictUtil.to_int(e.get("levels", 0)) > Proficiency.MAX_LEVEL:
					errors.append("%s: levels must be 1-%d." % [at, Proficiency.MAX_LEVEL])
			"resist":
				if not [StatusEffects.STAGGER, StatusEffects.STUN, StatusEffects.ROOT].has(str(e.get("status", ""))):
					errors.append("%s: unknown status '%s'." % [at, str(e.get("status", ""))])
				if DictUtil.to_int(e.get("value", 0)) < 1 or DictUtil.to_int(e.get("value", 0)) > 100:
					errors.append("%s: resistance is 1-100." % at)
			"ap_regen":
				if DictUtil.to_int(e.get("value", 0)) < 1 or DictUtil.to_int(e.get("value", 0)) > 4:
					errors.append("%s: extra AP per round is 1-4." % at)
			"skill":
				if not skill_exists.call(str(e.get("skill", ""))):
					errors.append("%s: unknown skill '%s'." % [at, str(e.get("skill", ""))])
			_:
				errors.append("%s: unknown effect type '%s' (use %s)." % [at, type, ", ".join(TYPES)])
	return errors

# Puts a passive's effects on `c` (once per passive id).
static func apply(c: Combatant, passive: Dictionary) -> void:
	var id: String = str(passive.get("id", ""))
	if id.is_empty() or c.passives.has(id):
		return
	c.passives.append(id)
	for raw: Variant in DictUtil.array_of(passive.get("effects", [])):
		var e: Dictionary = DictUtil.dict_of(raw)
		match str(e.get("type", "")):
			"stat":
				var ctx: Dictionary = DictUtil.dict_of(e.get("context", {}))
				var tag: String = "always"
				for key: Variant in ctx:
					tag = "%s:%s" % [str(key), str(ctx[key])]
				if c.stats != null:
					c.stats.add_modifier(StatModifier.new(str(e["stat"]), DictUtil.to_int(e.get("value", 0)), 0.0, "passive:" + id, tag, -1))
			"proficiency":
				if c.stats != null and c.stats.proficiency != null:
					c.stats.proficiency.add_bonus(str(e.get("gear", "")), DictUtil.to_int(e.get("levels", 0)))
			"resist":
				var status: String = str(e.get("status", ""))
				c.status_resist[status] = mini(100, DictUtil.int_of(c.status_resist, status) + DictUtil.to_int(e.get("value", 0)))
			"ap_regen":
				c.ap_regen_bonus += DictUtil.to_int(e.get("value", 0))
			"skill":
				var skill_id: String = str(e.get("skill", ""))
				if not c.granted_skills.has(skill_id):
					c.granted_skills.append(skill_id)

# One-line summary of what a passive does.
static func describe(passive: Dictionary) -> String:
	var parts: Array[String] = []
	for raw: Variant in DictUtil.array_of(passive.get("effects", [])):
		var e: Dictionary = DictUtil.dict_of(raw)
		match str(e.get("type", "")):
			"stat":
				var ctx: Dictionary = DictUtil.dict_of(e.get("context", {}))
				var where: String = ""
				for key: Variant in ctx:
					where = " when %s = %s" % [str(key), str(ctx[key])]
				parts.append("%s %+d%s" % [str(e.get("stat", "")), DictUtil.to_int(e.get("value", 0)), where])
			"proficiency":
				parts.append("%s training +%d" % [str(e.get("gear", "")), DictUtil.to_int(e.get("levels", 0))])
			"resist":
				parts.append("%d%% resist %s" % [DictUtil.to_int(e.get("value", 0)), str(e.get("status", ""))])
			"ap_regen":
				parts.append("+%d AP per round" % DictUtil.to_int(e.get("value", 0)))
			"skill":
				parts.append("learns %s" % str(e.get("skill", "")))
	return "; ".join(parts) if not parts.is_empty() else "no effects"
