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
#   resist       {"status": "stagger", "value": 25}   status resistance % (100 = immune); status is any of StatusEffects.resistable() (stagger, stun, root, bleed)
#   ap_regen     {"value": 1}   extra AP regained each round
#   skill        {"skill": "quick_stab"}   a skill the unit may use beyond its class path
#   skill_potency {"family": "strike", "value": 0.25}   +25% magnitude (attack power_mult / heal) for that
#                skill family: adds to Combatant.potency_bonus, read via Combatant.potency_for. Gear affixes
#                and passives use this to scale the single-tier basics later (value -0.9..2.0, not 0).
#                Exactly one of "family" (a skill id) or "group" ("spell" or "heal"): a group bonus applies to every
#                skill of the group (Combatant.potency_for_skill; groups are derived, see SkillDef.in_group)
#   status_potency {"value": 5}   +5 points of hit chance for every status the unit inflicts with an attack skill (1..25)
#   spell_reach   {"value": 1}    +1 maximum reach for spell skills (1..2; total gear bonus is clamped to GEAR_REACH_CAP)
# Equipment carries the same grammar in its `effects` list, but only ITEM_TYPES are allowed there; item
# effects apply while equipped and are rebuilt on every equip change (apply_effects with `gear` = true).
# ==============================================================================

const TYPES: Array[String] = ["stat", "proficiency", "resist", "ap_regen", "skill", "skill_potency", "status_potency", "spell_reach"]
const ITEM_TYPES: Array[String] = ["skill_potency", "status_potency", "spell_reach", "resist"]
const GROUPS: Array[String] = ["spell", "heal"]
const POTENCY_MIN: float = -0.9
const POTENCY_MAX: float = 2.0
const STATUS_POTENCY_MIN: int = 1
const STATUS_POTENCY_MAX: int = 25
const SPELL_REACH_MIN: int = 1
const SPELL_REACH_MAX: int = 2
const GEAR_REACH_CAP: int = 1   # total spell reach from gear, clamped at runtime

# Problems with an effects list (empty = valid). `skill_exists` resolves skill ids.
# `allow_empty`: items may carry no effects; a passive needs at least one. `items_only`: reject types
# that only make sense on a passive (stat, proficiency, ap_regen, skill).
static func validate_effects(effects: Variant, skill_exists: Callable, allow_empty: bool = false, items_only: bool = false) -> Array[String]:
	var errors: Array[String] = []
	if not (effects is Array):
		errors.append("Effects must be a list.")
		return errors
	var list: Array = effects
	if list.is_empty() and not allow_empty:
		errors.append("A passive needs at least one effect.")
	for i: int in range(list.size()):
		var e: Dictionary = DictUtil.dict_of(list[i])
		var type: String = str(e.get("type", ""))
		var at: String = "Effect %d" % (i + 1)
		if items_only and TYPES.has(type) and not ITEM_TYPES.has(type):
			errors.append("%s: '%s' is not allowed on items (use stat_bonuses / passives)." % [at, type])
			continue
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
				if not StatusEffects.resistable().has(str(e.get("status", ""))):
					errors.append("%s: unknown status '%s'." % [at, str(e.get("status", ""))])
				if DictUtil.to_int(e.get("value", 0)) < 1 or DictUtil.to_int(e.get("value", 0)) > 100:
					errors.append("%s: resistance is 1-100." % at)
			"ap_regen":
				if DictUtil.to_int(e.get("value", 0)) < 1 or DictUtil.to_int(e.get("value", 0)) > 4:
					errors.append("%s: extra AP per round is 1-4." % at)
			"skill":
				if not skill_exists.call(str(e.get("skill", ""))):
					errors.append("%s: unknown skill '%s'." % [at, str(e.get("skill", ""))])
			"skill_potency":
				var has_family: bool = e.has("family") and not str(e["family"]).is_empty()
				var has_group: bool = e.has("group") and not str(e["group"]).is_empty()
				if has_family == has_group:
					errors.append("%s: give exactly one of family (a skill id) or group (%s)." % [at, ", ".join(GROUPS)])
				elif has_group and not GROUPS.has(str(e["group"])):
					errors.append("%s: unknown skill group '%s' (use %s)." % [at, str(e["group"]), ", ".join(GROUPS)])
				elif has_family and not skill_exists.call(str(e.get("family", ""))):
					errors.append("%s: unknown skill family '%s'." % [at, str(e.get("family", ""))])
				var pv: float = DictUtil.to_float(e.get("value", 0.0))
				if is_zero_approx(pv) or pv < POTENCY_MIN or pv > POTENCY_MAX:
					errors.append("%s: potency bonus must be non-zero, %.1f to %.1f (0.25 = +25%%)." % [at, POTENCY_MIN, POTENCY_MAX])
			"status_potency":
				var sp: int = DictUtil.to_int(e.get("value", 0))
				if sp < STATUS_POTENCY_MIN or sp > STATUS_POTENCY_MAX:
					errors.append("%s: status potency is %d-%d points." % [at, STATUS_POTENCY_MIN, STATUS_POTENCY_MAX])
			"spell_reach":
				var sr: int = DictUtil.to_int(e.get("value", 0))
				if sr < SPELL_REACH_MIN or sr > SPELL_REACH_MAX:
					errors.append("%s: spell reach is %d-%d tiles." % [at, SPELL_REACH_MIN, SPELL_REACH_MAX])
			_:
				errors.append("%s: unknown effect type '%s' (use %s)." % [at, type, ", ".join(ITEM_TYPES if items_only else TYPES)])
	return errors

# Puts a passive's effects on `c` (once per passive id).
static func apply(c: Combatant, passive: Dictionary) -> void:
	var id: String = str(passive.get("id", ""))
	if id.is_empty() or c.passives.has(id):
		return
	c.passives.append(id)
	apply_effects(c, DictUtil.array_of(passive.get("effects", [])), "passive:" + id, false)

# Puts an effects list on `c`. `gear` = true writes to the gear-sourced fields (cleared and rebuilt from the
# equipment on every equip change, see Combatant.rebuild_gear_effects) instead of the permanent passive fields.
static func apply_effects(c: Combatant, effects: Array, source: String, gear: bool) -> void:
	var id: String = source.trim_prefix("passive:")
	for raw: Variant in effects:
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
				var target: Dictionary = c.gear_status_resist if gear else c.status_resist
				target[status] = mini(100, DictUtil.int_of(target, status) + DictUtil.to_int(e.get("value", 0)))
			"ap_regen":
				c.ap_regen_bonus += DictUtil.to_int(e.get("value", 0))
			"skill":
				var skill_id: String = str(e.get("skill", ""))
				c.learn_skill(skill_id)   # also slots its family into a free extra slot of an active loadout
			"skill_potency":
				var value: float = DictUtil.to_float(e.get("value", 0.0))
				var group: String = str(e.get("group", ""))
				if not group.is_empty():
					var groups: Dictionary = c.gear_potency_group if gear else c.potency_group
					groups[group] = DictUtil.float_of(groups, group) + value
				else:
					var family: String = str(e.get("family", ""))
					var families: Dictionary = c.gear_potency_family if gear else c.potency_bonus
					families[family] = DictUtil.float_of(families, family) + value
			"status_potency":
				if gear:
					c.gear_status_potency += DictUtil.to_int(e.get("value", 0))
				else:
					c.passive_status_potency += DictUtil.to_int(e.get("value", 0))
			"spell_reach":
				if gear:
					c.gear_spell_reach += DictUtil.to_int(e.get("value", 0))
				else:
					c.passive_spell_reach += DictUtil.to_int(e.get("value", 0))

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
			"skill_potency":
				var label: String = str(e.get("group", "")) + " group" if not str(e.get("group", "")).is_empty() else str(e.get("family", ""))
				parts.append("%+d%% %s power" % [roundi(DictUtil.to_float(e.get("value", 0.0)) * 100.0), label])
			"status_potency":
				parts.append("%+d status chance" % DictUtil.to_int(e.get("value", 0)))
			"spell_reach":
				parts.append("%+d spell reach" % DictUtil.to_int(e.get("value", 0)))
	return "; ".join(parts) if not parts.is_empty() else "no effects"
