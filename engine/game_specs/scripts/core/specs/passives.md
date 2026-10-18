---
class: Passives
extends: RefCounted
description: "Tactical RPG - Passives (inherited, lore-granted traits; replaces mastery) A passive is a lore-free template in the content catalog (\"passives\"): a list of effects. Nothing in a class or base unit grants one; SOURCES do: a world character's background, a faction, a quest reward, a lore discovery, a named item. Passives.apply() puts the effects on a Combatant. Effect types (each effect is a Dictionary with \"type\"): stat         {\"stat\": \"KNOW\", \"value\": 3, \"context\": {\"biome\": \"ruins\"}} flat stat bonus, optionally only in a context (empty = always) proficiency  {\"gear\": \"sword\", \"levels\": 1}   gear-training levels (Proficiency) resist       {\"status\": \"stagger\", \"value\": 25}   status resistance % (100 = immune); status is any of StatusEffects.resistable() (stagger, stun, root, bleed) ap_regen     {\"value\": 1}   extra AP regained each round skill        {\"skill\": \"quick_stab\"}   a skill the unit may use beyond its class path skill_potency {\"family\": \"strike\", \"value\": 0.25}   +25% magnitude (attack power_mult / heal) for that skill family: adds to Combatant.potency_bonus, read via Combatant.potency_for. Gear affixes and passives use this to scale the single-tier basics later (value -0.9..2.0, not 0). Exactly one of \"family\" (a skill id) or \"group\" (\"spell\" or \"heal\"): a group bonus applies to every skill of the group (Combatant.potency_for_skill; groups are derived, see SkillDef.in_group) unlock       {\"family\": \"riposte\" | \"group\": \"heal\", \"levels\": 1-2}   opens a class-path skill family (or every family of the group) that many unit levels earlier (Combatant.family_unlock_level). Never grants ownership; the largest value per family / group wins (never summed) and the total reduction is capped by StatRules.MAX_UNLOCK_REDUCTION. Passives only, not items. status_potency {\"value\": 5}   +5 points of hit chance for every status the unit inflicts with an attack skill (1..25) spell_reach   {\"value\": 1}    +1 maximum reach for spell skills (1..2; total gear bonus is clamped to GEAR_REACH_CAP) foresight     {}              no params: the holder reads the whole round plan of every enemy that targets it (Combatant.has_foresight; read by IntentRules / CombatSession.enemy_intents; INTENT1). Passives only. Equipment carries the same grammar in its `effects` list, but only ITEM_TYPES are allowed there; item effects apply while equipped and are rebuilt on every equip change (apply_effects with `gear` = true)."
---

# Properties
* `TYPES`: Array[String] = ["stat", "proficiency", "resist", "ap_regen", "skill", "skill_potency", "status_potency", "spell_reach", "unlock", "foresight"] (const)
* `UNLOCK_LEVELS_MAX`: int = 2 (const)
* `ITEM_TYPES`: Array[String] = ["skill_potency", "status_potency", "spell_reach", "resist"] (const)
* `GROUPS`: Array[String] = ["spell", "heal"] (const)
* `POTENCY_MIN`: float = -0.9 (const)
* `POTENCY_MAX`: float = 2.0 (const)
* `STATUS_POTENCY_MIN`: int = 1 (const)
* `STATUS_POTENCY_MAX`: int = 25 (const)
* `SPELL_REACH_MIN`: int = 1 (const)
* `SPELL_REACH_MAX`: int = 2 (const)
* `GEAR_REACH_CAP`: int = 1   # total spell reach from gear, clamped at runtime (const)
* `all`: Array[Dictionary] = [
* `out`: Array[Dictionary] = []
* `errors`: Array[String] = []
* `list`: Array = effects
* `e`: Dictionary = DictUtil.dict_of(list[i])
* `type`: String = str(e.get("type", ""))
* `at`: String = "Effect %d" % (i + 1)
* `ctx`: Variant = e.get("context", {})
* `has_family`: bool = e.has("family") and not str(e["family"]).is_empty()
* `has_group`: bool = e.has("group") and not str(e["group"]).is_empty()
* `pv`: float = DictUtil.to_float(e.get("value", 0.0))
* `u_family`: bool = e.has("family") and not str(e["family"]).is_empty()
* `u_group`: bool = e.has("group") and not str(e["group"]).is_empty()
* `ul`: int = DictUtil.to_int(e.get("levels", 0))
* `sp`: int = DictUtil.to_int(e.get("value", 0))
* `sr`: int = DictUtil.to_int(e.get("value", 0))

# Methods

### `static effect_spec(type: String) -> Dictionary`
```logic
for spec: Dictionary in effect_specs():
	if str(spec["type"]) == type:
		return spec
return {}
```

### `static apply(c: Combatant, passive: Dictionary) -> void`
```logic
var id: String = str(passive.get("id", ""))
if id.is_empty() or c.passives.has(id):
	return
c.passives.append(id)
apply_effects(c, DictUtil.array_of(passive.get("effects", [])), "passive:" + id, false)
c.refresh_path_skills()   # a KNOW bonus can open an advanced family earlier
```

### `static apply_effects(c: Combatant, effects: Array, source: String, gear: bool) -> void`
```logic
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
		"unlock":
			if not gear:
				var levels: int = clampi(DictUtil.to_int(e.get("levels", 0)), 0, UNLOCK_LEVELS_MAX)
				var u_group: String = str(e.get("group", ""))
				var bonus: Dictionary = c.unlock_bonus_group if not u_group.is_empty() else c.unlock_bonus_family
				var key: String = u_group if not u_group.is_empty() else str(e.get("family", ""))
				bonus[key] = maxi(DictUtil.int_of(bonus, key), levels)
				c.refresh_path_skills()
		"foresight":
			if not gear and c.foresight_source.is_empty():
				c.foresight_source = id
```

### `static describe(passive: Dictionary) -> String`
```logic
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
		"unlock":
			var u_label: String = str(e.get("group", "")) + " group" if not str(e.get("group", "")).is_empty() else str(e.get("family", ""))
			parts.append("opens %s %d levels earlier" % [u_label, DictUtil.to_int(e.get("levels", 0))])
		"foresight":
			parts.append("reads the round plan of foes targeting it")
return "; ".join(parts) if not parts.is_empty() else "no effects"
```

