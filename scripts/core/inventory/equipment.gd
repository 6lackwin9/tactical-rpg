class_name Equipment
extends Item

# ==============================================================================
# Tactical RPG - Equipment & Gear Model
# Stat Requirements, Threshold Tiers, Weapon Proficiencies & Timeline Modifiers
# ==============================================================================

enum EquipSlot {
	MAIN_HAND,
	OFF_HAND,
	HEAD,
	BODY,
	ACCESSORY_1,
	ACCESSORY_2
}

var slot: EquipSlot = EquipSlot.MAIN_HAND
var weapon_type: String = ""  # sword, axe, bow, dagger, staff, shield, cloth, leather, plate
var base_power: int = 0
var base_range: int = 1
var block_cost: int = 1
var burden: int = 0           # Speed/initiative penalty applied to user
var mitigation: int = 0       # Flat defensive mitigation
var is_two_handed: bool = false # Two-handed weapon occupies both hands (Claymore, Warhammer, Bow)
var off_hand_category: String = "" # shield, shortsword, knuckles, dagger, knife, orb, tome
var scaling_stat: String = "STR"   # STR, DEX, or INT scaling
var stat_bonuses: Dictionary = {}        # Stat Name (String) -> Bonus (int)
var unlocked_commands: Array[String] = [] # e.g. ["DefendCommand", "FocusCommand"]

# Stat Requirements (e.g. {"STR": 16})
var required_stats: Dictionary = {}

# Threshold Tiers: extra boosts granted when meeting higher stat milestones
# Array of Dictionaries: {"stat": "STR", "threshold": 22, "bonus_power": 4, "desc": "Crushing Blow"}
var threshold_tiers: Array[Dictionary] = []

func _init(
	p_id: String = "",
	p_name: String = "Equipment",
	p_slot: EquipSlot = EquipSlot.MAIN_HAND,
	p_weapon_type: String = "",
	p_type: ItemType = ItemType.WEAPON,
	p_power: int = 0,
	p_reqs: Dictionary = {},
	p_tiers: Array[Dictionary] = [],
	p_weight: float = 2.0,
	p_value: int = 25,
	p_rarity: ItemRarity = ItemRarity.COMMON
) -> void:
	super._init(p_id, p_name, p_type, p_weight, p_value, p_rarity, "", 1)
	slot = p_slot
	weapon_type = p_weapon_type
	base_power = p_power
	required_stats = p_reqs
	threshold_tiers = p_tiers

static func slot_display_name(p_slot: EquipSlot) -> String:
	match p_slot:
		EquipSlot.MAIN_HAND: return "Main Hand"
		EquipSlot.OFF_HAND: return "Off Hand"
		EquipSlot.HEAD: return "Headgear"
		EquipSlot.BODY: return "Body Armor"
		EquipSlot.ACCESSORY_1: return "Accessory 1"
		EquipSlot.ACCESSORY_2: return "Accessory 2"
	return "Gear Slot"

func is_weapon() -> bool:
	return item_type == ItemType.WEAPON

func is_shield() -> bool:
	return item_type == ItemType.SHIELD or off_hand_category == "shield"

func is_ranged_weapon() -> bool:
	if item_type != ItemType.WEAPON:
		return false
	return weapon_type.to_lower() in ["bow", "crossbow", "sling"] or base_range > 1

func is_melee_weapon() -> bool:
	if item_type != ItemType.WEAPON:
		return false
	return not is_ranged_weapon()

func get_requirements_display() -> String:
	return get_tier_requirements_display()

# Convention: [Req <STAT level0>, <STAT level1>] (Requirement 4)
# level0 = wield without penalty; level1 = threshold tier boost (+Power, +STAT, effect)
func get_tier_requirements_display() -> String:
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

func get_requirement_status_summary(stats: CharacterStats, mastery: MasterySystem) -> Dictionary:
	var meets: bool = meets_requirements(stats, mastery)
	var pen: Dictionary = get_wield_penalty(stats, mastery)
	var is_pen: bool = pen.get("is_penalized", false)
	var raw_def: Variant = pen.get("deficit", 0)
	var def_num: int = raw_def if raw_def is int else 0
	var raw_ratio: Variant = pen.get("power_penalty_ratio", 0.0)
	var pen_ratio: float = raw_ratio if raw_ratio is float else 0.0
	var pen_pct: int = int(roundf(pen_ratio * 100.0))
	var tier_boost: int = get_unlocked_threshold_power_boost(stats, mastery)

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

func get_stat_bonuses_display() -> String:
	if stat_bonuses.is_empty():
		return ""
	var parts: Array[String] = []
	for st_k: String in stat_bonuses:
		var v: Variant = stat_bonuses[st_k]
		var n: int = v if v is int else 0
		var sign_str: String = "+%d" % n if n > 0 else "%d" % n
		parts.append("%s %s" % [sign_str, st_k])
	return ", ".join(parts)

# ------------------------------------------------------------------------------
# Stat Requirements & Class Mastery Threshold Discount
# ------------------------------------------------------------------------------

func meets_requirements(stats: CharacterStats, mastery: MasterySystem) -> bool:
	if required_stats.is_empty() or stats == null:
		return true

	var discount: int = 0
	if mastery != null and not weapon_type.is_empty():
		discount = mastery.get_weapon_requirement_discount(weapon_type)

	for req_stat: String in required_stats:
		var raw_req: Variant = required_stats[req_stat]
		var req_val: int = raw_req if raw_req is int else 0
		var eff_stat: int = stats.get_effective_stat(req_stat, {"weapon": weapon_type}) + discount
		if eff_stat < req_val:
			return false

	return true

# Calculates penalty details if character attempts to wield without meeting requirements
func get_wield_penalty(stats: CharacterStats, mastery: MasterySystem) -> Dictionary:
	var result: Dictionary = {
		"is_penalized": false,
		"power_penalty_ratio": 0.0,
		"extra_block_cost": 0,
		"deficit": 0
	}

	if meets_requirements(stats, mastery) or stats == null:
		return result

	var discount: int = 0
	if mastery != null and not weapon_type.is_empty():
		discount = mastery.get_weapon_requirement_discount(weapon_type)

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

# Returns all unlocked threshold tiers when exceeding base requirements
func get_active_threshold_tiers(stats: CharacterStats, mastery: MasterySystem) -> Array[Dictionary]:
	var active_tiers: Array[Dictionary] = []
	if stats == null or threshold_tiers.is_empty():
		return active_tiers

	var discount: int = 0
	if mastery != null and not weapon_type.is_empty():
		discount = mastery.get_weapon_requirement_discount(weapon_type)

	for tier: Dictionary in threshold_tiers:
		var stat_key: String = tier.get("stat", "STR")
		var threshold_val: int = tier.get("threshold", 0)
		var eff_stat: int = stats.get_effective_stat(stat_key, {"weapon": weapon_type}) + discount
		if eff_stat >= threshold_val:
			active_tiers.append(tier)

	return active_tiers

# Helper: sum of unlocked bonus power across all active threshold tiers
func get_unlocked_threshold_power_boost(stats: CharacterStats, mastery: MasterySystem) -> int:
	var total_boost: int = 0
	var active_tiers: Array[Dictionary] = get_active_threshold_tiers(stats, mastery)
	for tier: Dictionary in active_tiers:
		var raw_bonus: Variant = tier.get("bonus_power", 0)
		if raw_bonus is int:
			total_boost += raw_bonus
	return total_boost

# Effective power including tier boosts, mastery boosts and deficit penalties (Requirements 1 & 2)
func get_effective_equipment_power(stats: CharacterStats, mastery: MasterySystem) -> int:
	var total_power: int = base_power

	# Add threshold tier bonuses
	var active_tiers: Array[Dictionary] = get_active_threshold_tiers(stats, mastery)
	for tier: Dictionary in active_tiers:
		var bonus_raw: Variant = tier.get("bonus_power", 0)
		var bonus: int = bonus_raw if bonus_raw is int else 0
		total_power += bonus

	# Add weapon mastery bonus power (Requirement 2)
	if mastery != null and not weapon_type.is_empty():
		total_power += mastery.get_weapon_power_bonus(weapon_type)

	# Deduct deficit penalties: Wep Power * (1 - Deficit)
	var penalty: Dictionary = get_wield_penalty(stats, mastery)
	if penalty.get("is_penalized", false):
		var ratio_raw: Variant = penalty.get("power_penalty_ratio", 0.0)
		var ratio: float = ratio_raw if ratio_raw is float else 0.0
		total_power = maxi(1, int(roundf(float(total_power) * (1.0 - ratio))))

	return total_power

func get_effective_block_cost(stats: CharacterStats, mastery: MasterySystem) -> int:
	var cost: int = block_cost
	var penalty: Dictionary = get_wield_penalty(stats, mastery)
	if penalty["is_penalized"]:
		cost += penalty["extra_block_cost"]
	return cost
