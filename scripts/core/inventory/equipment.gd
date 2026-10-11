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

# Effective power including tier boosts and deficit penalties
func get_effective_equipment_power(stats: CharacterStats, mastery: MasterySystem) -> int:
	var total_power: int = base_power

	# Add threshold tier bonuses
	var active_tiers: Array[Dictionary] = get_active_threshold_tiers(stats, mastery)
	for tier: Dictionary in active_tiers:
		var bonus: int = tier.get("bonus_power", 0)
		total_power += bonus

	# Deduct deficit penalties
	var penalty: Dictionary = get_wield_penalty(stats, mastery)
	if penalty["is_penalized"]:
		var ratio: float = penalty["power_penalty_ratio"]
		total_power = maxi(1, int(roundf(float(total_power) * (1.0 - ratio))))

	return total_power

func get_effective_block_cost(stats: CharacterStats, mastery: MasterySystem) -> int:
	var cost: int = block_cost
	var penalty: Dictionary = get_wield_penalty(stats, mastery)
	if penalty["is_penalized"]:
		cost += penalty["extra_block_cost"]
	return cost
