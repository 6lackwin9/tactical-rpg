---
class: Equipment
extends: Item
description: "Tactical RPG - Equipment & Gear Model Stat Requirements, Threshold Tiers, Weapon Proficiencies & Timeline Modifiers"
---

# Properties
* `slot`: EquipSlot = EquipSlot.MAIN_HAND
* `weapon_type`: String = ""  # sword, axe, bow, dagger, staff, shield, cloth, leather, plate
* `base_power`: int = 0
* `base_range`: int = 1
* `block_cost`: int = 1
* `burden`: int = 0           # Speed/initiative penalty applied to user
* `mitigation`: int = 0       # Flat defensive mitigation
* `is_two_handed`: bool = false # Two-handed weapon occupies both hands (Claymore, Warhammer, Bow)
* `off_hand_category`: String = "" # shield, shortsword, knuckles, dagger, knife, orb, tome
* `scaling_stat`: String = "STR"   # STR, DEX, or INT scaling
* `stat_bonuses`: Dictionary = {}        # Stat Name (String) -> Bonus (int)
* `unlocked_commands`: Array[String] = [] # e.g. ["DefendCommand", "FocusCommand"]
* `effects`: Array = []                  # item effects (Passives grammar, ITEM_TYPES only); apply while equipped
* `required_stats`: Dictionary = {}
* `threshold_tiers`: Array[Dictionary] = []
* `FOCUS_TYPES`: Array[String] = ["orb", "tome"] (const)
* `HAND_MAIN_1H`: String = "main_1h" (const)
* `HAND_MAIN_2H`: String = "main_2h" (const)
* `HAND_OFF`: String = "off" (const)
* `HANDS`: Array[String] = [HAND_MAIN_1H, HAND_MAIN_2H, HAND_OFF] (const)
* `ALWAYS_RANGED_TYPES`: Array[String] = ["bow", "crossbow", "sling"] (const)
* `TIER_SOURCE`: String = "gear_tier"   # StatModifier source of every tier stat bonus (const)
* `active_tiers`: Array[Dictionary] = []
* `out`: Array[int] = []

# Methods

### `static slot_display_name(p_slot: EquipSlot) -> String`
```logic
match p_slot:
	EquipSlot.MAIN_HAND: return "Main Hand"
	EquipSlot.OFF_HAND: return "Off Hand"
	EquipSlot.HEAD: return "Headgear"
	EquipSlot.BODY: return "Body Armor"
	EquipSlot.ACCESSORY_1: return "Accessory 1"
	EquipSlot.ACCESSORY_2: return "Accessory 2"
return "Gear Slot"
```

### `is_weapon() -> bool`
```logic
return item_type == ItemType.WEAPON
```

### `is_shield() -> bool`
```logic
return item_type == ItemType.SHIELD or off_hand_category == "shield"
```

### `is_off_hand_weapon() -> bool`
```logic
return is_weapon() and not off_hand_category.is_empty() and off_hand_category != "shield" and not is_two_handed
```

### `can_equip_in_slot(target: EquipSlot) -> bool`
```logic
if is_shield():
	return target == EquipSlot.OFF_HAND
if is_weapon():
	if is_two_handed:
		return target == EquipSlot.MAIN_HAND
	if is_off_hand_weapon():
		return target == EquipSlot.OFF_HAND
	return target == EquipSlot.MAIN_HAND or (target == EquipSlot.OFF_HAND and allow_plain_dual_wield)
if slot == EquipSlot.ACCESSORY_1 or slot == EquipSlot.ACCESSORY_2:
	return target == EquipSlot.ACCESSORY_1 or target == EquipSlot.ACCESSORY_2
return target == slot
```

### `is_focus() -> bool`
```logic
return is_weapon() and FOCUS_TYPES.has(weapon_type)
```

### `conflicts_with_main(main: Equipment) -> bool`
```logic
return is_focus() and main != null and main.is_two_handed
```

### `hand() -> String`
```logic
if is_shield():
	return HAND_OFF
if not is_weapon():
	return ""
if is_two_handed:
	return HAND_MAIN_2H
if is_off_hand_weapon():
	return HAND_OFF
return HAND_MAIN_1H
```

### `static hand_display_name(h: String) -> String`
```logic
match h:
	HAND_MAIN_1H: return "main hand, one-handed"
	HAND_MAIN_2H: return "main hand, two-handed"
	HAND_OFF: return "off hand"
return "-"
```

### `is_ranged_weapon() -> bool`
```logic
if item_type != ItemType.WEAPON:
	return false
return ALWAYS_RANGED_TYPES.has(weapon_type.to_lower()) or base_range > 1
```

### `is_melee_weapon() -> bool`
```logic
if item_type != ItemType.WEAPON:
	return false
return not is_ranged_weapon()
```

### `get_requirements_display() -> String`
```logic
return get_tier_requirements_display()
```

### `get_tier_requirements_display() -> String`
```logic
if required_stats.is_empty() and threshold_tiers.is_empty():
	return "[No Req]"

var parts: Array[String] = []
var processed_stats: Array[String] = []

for st_k: String in required_stats:
	processed_stats.append(st_k.to_upper())
	var lvl0: int = required_stats[st_k] if required_stats[st_k] is int else 0
	var stat_parts: Array[String] = ["%s %d" % [st_k, lvl0]]

	for tier: Dictionary in threshold_tiers:
		var t_stat: String = str(tier.get("stat", ""))
		if t_stat.to_upper() == st_k.to_upper():
			var lvl1: int = tier.get("threshold", 0)
			var bonus_pwr: int = tier.get("bonus_power", 0)
			var desc: String = str(tier.get("desc", ""))
			var bonus_label: String = ""
			if bonus_pwr > 0:
				bonus_label = "+%d Power" % bonus_pwr
			elif not desc.is_empty():
				bonus_label = desc
			if not bonus_label.is_empty():
				stat_parts.append("%d (%s)" % [lvl1, bonus_label])
			else:
				stat_parts.append("%d" % lvl1)

	parts.append(", ".join(stat_parts))

for tier: Dictionary in threshold_tiers:
	var t_stat: String = str(tier.get("stat", "")).to_upper()
	if t_stat.is_empty():
		# Extended criteria (proficiency, unit level; resolved rows carry stat ""): "sword training 2 (+2 STR)".
		var label: String = ItemText.bonus_text(tier_bonus(tier))
		parts.append("%s (%s)" % [tier_criterion_text(tier), label] if not label.is_empty() else tier_criterion_text(tier))
		continue
	if not processed_stats.has(t_stat):
		processed_stats.append(t_stat)
		var lvl1: int = tier.get("threshold", 0)
		var bonus_pwr: int = tier.get("bonus_power", 0)
		var desc: String = str(tier.get("desc", ""))
		var bonus_label: String = ""
		if bonus_pwr > 0:
			bonus_label = "+%d Power" % bonus_pwr
		elif not desc.is_empty():
			bonus_label = desc
		if not bonus_label.is_empty():
			parts.append("%s %d (%s)" % [t_stat, lvl1, bonus_label])
		else:
			parts.append("%s %d" % [t_stat, lvl1])

return "[Req %s]" % "; ".join(parts)
```

### `get_requirement_status_summary(stats: CharacterStats, prof: Proficiency) -> Dictionary`
```logic
var meets: bool = meets_requirements(stats, prof)
var pen: Dictionary = get_wield_penalty(stats, prof)
var is_pen: bool = pen.get("is_penalized", false)
var raw_def: Variant = pen.get("deficit", 0)
var def_num: int = raw_def if raw_def is int else 0
var raw_ratio: Variant = pen.get("power_penalty_ratio", 0.0)
var pen_ratio: float = raw_ratio if raw_ratio is float else 0.0
var pen_pct: int = int(roundf(pen_ratio * 100.0))
var tier_boost: int = get_unlocked_threshold_power_boost(stats, prof)

var status_tag := ""
var tag_color := Color.WHITE
if is_pen:
	status_tag = "⚠️ -%d Deficit (-%d%% Power)" % [def_num, pen_pct]
	tag_color = Color(1.0, 0.4, 0.4)
elif tier_boost > 0:
	status_tag = "★ Tier 1 Active (+%d Power)" % tier_boost
	tag_color = Color(1.0, 0.85, 0.3)
elif meets and not required_stats.is_empty():
	status_tag = ""
	tag_color = Color(0.4, 0.9, 0.5)
else:
	status_tag = ""
	tag_color = Color(0.8, 0.9, 0.8)

return {
	"meets": meets,
	"is_penalized": is_pen,
	"deficit": def_num,
	"penalty_percent": pen_pct,
	"tier_boost": tier_boost,
	"status_tag": status_tag,
	"tag_color": tag_color,
	"req_display": get_tier_requirements_display()
}
```

### `get_stat_bonuses_display() -> String`
```logic
if stat_bonuses.is_empty():
	return ""
var parts: Array[String] = []
for st_k: String in stat_bonuses:
	var v: Variant = stat_bonuses[st_k]
	var n: int = v if v is int else 0
	var sign_str: String = "+%d" % n if n > 0 else "%d" % n
	parts.append("%s %s" % [sign_str, st_k])
return ", ".join(parts)
```

### `meets_requirements(stats: CharacterStats, prof: Proficiency) -> bool`
```logic
if required_stats.is_empty() or stats == null:
	return true

var discount: int = 0
if prof != null and not weapon_type.is_empty():
	discount = prof.get_weapon_requirement_discount(weapon_type)

for req_stat: String in required_stats:
	var raw_req: Variant = required_stats[req_stat]
	var req_val: int = raw_req if raw_req is int else 0
	var eff_stat: int = stats.get_effective_stat(req_stat, {"weapon": weapon_type}) + discount
	if eff_stat < req_val:
		return false

return true
```

### `get_wield_penalty(stats: CharacterStats, prof: Proficiency) -> Dictionary`
```logic
var result: Dictionary = {
	"is_penalized": false,
	"power_penalty_ratio": 0.0,
	"extra_block_cost": 0,
	"deficit": 0
}

if meets_requirements(stats, prof) or stats == null:
	return result

var discount: int = 0
if prof != null and not weapon_type.is_empty():
	discount = prof.get_weapon_requirement_discount(weapon_type)

var max_deficit: int = 0
for req_stat: String in required_stats:
	var raw_req: Variant = required_stats[req_stat]
	var req_val: int = raw_req if raw_req is int else 0
	var eff_stat: int = stats.get_effective_stat(req_stat, {"weapon": weapon_type}) + discount
	if eff_stat < req_val:
		max_deficit = maxi(max_deficit, req_val - eff_stat)

result["is_penalized"] = true
result["deficit"] = max_deficit
# Suffer 20% power loss per deficit point (up to 60%) and +1 timeline block cost if deficit >= 4
result["power_penalty_ratio"] = minf(0.60, float(max_deficit) * 0.15)
if max_deficit >= 4:
	result["extra_block_cost"] = 1

return result
```

### `static tier_when(tier: Dictionary) -> Dictionary`
```logic
var when: Dictionary = DictUtil.dict_of(tier.get("when", {}))
if not when.is_empty():
	return when
return {"type": ItemModifiers.WHEN_STAT, "stat": str(tier.get("stat", "STR")), "value": DictUtil.to_int(tier.get("threshold", 0))}
```

### `static tier_bonus(tier: Dictionary) -> Dictionary`
```logic
var bonus: Dictionary = DictUtil.dict_of(tier.get("bonus", {}))
if not bonus.is_empty():
	return bonus
return {"power": DictUtil.to_int(tier.get("bonus_power", 0))}
```

### `tier_criterion_text(tier: Dictionary) -> String`
```logic
var when: Dictionary = tier_when(tier)
match str(when.get("type", "")):
	ItemModifiers.WHEN_PROFICIENCY:
		return "%s training %d" % [_tier_gear(when), DictUtil.to_int(when.get("level", 0))]
	ItemModifiers.WHEN_LEVEL:
		return "unit level %d" % DictUtil.to_int(when.get("value", 0))
return "%s %d" % [str(when.get("stat", "STR")).to_upper(), DictUtil.to_int(when.get("value", 0))]
```

### `_tier_gear(when: Dictionary) -> String`
```logic
var gear: String = str(when.get("gear", ""))
return gear if not gear.is_empty() else weapon_type
```

### `tier_met(tier: Dictionary, stats: CharacterStats, prof: Proficiency) -> bool`
```logic
if stats == null:
	return false
var when: Dictionary = tier_when(tier)
match str(when.get("type", "")):
	ItemModifiers.WHEN_STAT:
		var discount: int = 0
		if prof != null and not weapon_type.is_empty():
			discount = prof.get_weapon_requirement_discount(weapon_type)
		var eff_stat: int = stats.get_effective_stat(str(when.get("stat", "STR")), {"weapon": weapon_type}, TIER_SOURCE) + discount
		return eff_stat >= DictUtil.to_int(when.get("value", 0))
	ItemModifiers.WHEN_PROFICIENCY:
		return prof != null and prof.level_of(_tier_gear(when)) >= DictUtil.to_int(when.get("level", 0))
	ItemModifiers.WHEN_LEVEL:
		return stats.level >= DictUtil.to_int(when.get("value", 0))
return false
```

### `get_tier_mitigation_bonus(stats: CharacterStats, prof: Proficiency) -> int`
```logic
var total: int = 0
for tier: Dictionary in get_active_threshold_tiers(stats, prof):
	total += DictUtil.to_int(tier_bonus(tier).get("mitigation", 0))
return total
```

### `get_effective_mitigation(stats: CharacterStats, prof: Proficiency) -> int`
```logic
return mitigation + get_tier_mitigation_bonus(stats, prof)
```

### `get_tier_stat_bonuses(stats: CharacterStats, prof: Proficiency) -> Dictionary`
```logic
var out: Dictionary = {}
for tier: Dictionary in get_active_threshold_tiers(stats, prof):
	var bonuses: Dictionary = DictUtil.dict_of(tier_bonus(tier).get("stat_bonuses", {}))
	for stat: Variant in bonuses:
		var key: String = str(stat).to_upper()
		out[key] = DictUtil.int_of(out, key) + DictUtil.to_int(bonuses[stat])
return out
```

### `tier_effects(n: int) -> Array`
```logic
if n < 1 or n > threshold_tiers.size():
	return []
return DictUtil.array_of(tier_bonus(threshold_tiers[n - 1]).get("effects", []))
```

### `get_unlocked_threshold_power_boost(stats: CharacterStats, prof: Proficiency) -> int`
```logic
var total_boost: int = 0
var active_tiers: Array[Dictionary] = get_active_threshold_tiers(stats, prof)
for tier: Dictionary in active_tiers:
	var raw_bonus: Variant = tier.get("bonus_power", 0)
	if raw_bonus is int:
		total_boost += raw_bonus
return total_boost
```

### `get_effective_equipment_power(stats: CharacterStats, prof: Proficiency) -> int`
```logic
var total_power: int = base_power

# Add threshold tier bonuses
var active_tiers: Array[Dictionary] = get_active_threshold_tiers(stats, prof)
for tier: Dictionary in active_tiers:
	var bonus_raw: Variant = tier.get("bonus_power", 0)
	var bonus: int = bonus_raw if bonus_raw is int else 0
	total_power += bonus

# Add weapon proficiency bonus power
if prof != null and not weapon_type.is_empty():
	total_power += prof.get_weapon_power_bonus(weapon_type)

# Deduct deficit penalties: Wep Power * (1 - Deficit)
var penalty: Dictionary = get_wield_penalty(stats, prof)
if penalty.get("is_penalized", false):
	var ratio_raw: Variant = penalty.get("power_penalty_ratio", 0.0)
	var ratio: float = ratio_raw if ratio_raw is float else 0.0
	total_power = maxi(1, int(roundf(float(total_power) * (1.0 - ratio))))

return total_power
```

### `get_effective_block_cost(stats: CharacterStats, prof: Proficiency) -> int`
```logic
var cost: int = block_cost
var penalty: Dictionary = get_wield_penalty(stats, prof)
if penalty["is_penalized"]:
	cost += penalty["extra_block_cost"]
if prof != null and not weapon_type.is_empty():
	cost -= prof.block_cost_reduction(weapon_type)   # master-level training
return maxi(1, cost)
```

