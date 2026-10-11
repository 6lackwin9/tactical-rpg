class_name ClassArchetype
extends RefCounted

# ==============================================================================
# Tactical RPG - Class Archetype Specification
# Defines starting stat baselines, level-up guaranteed growths, and masteries
# ==============================================================================

var id: String = "warrior"
var display_name: String = "Warrior"
var archetype_name: String:
	get:
		return display_name
var base_stats: Dictionary = {}         # Stat Name (String) -> Base Value (int)
var growth_per_level: Dictionary = {}   # Stat Name (String) -> Guaranteed Gain Per Level (int)
var innate_masteries: Dictionary = {}   # Weapon/Lore (String) -> Initial Level (int)
var allowed_gear_types: Array[String] = [] # Weapon/Armor types this class can equip (Accessories unrestricted)

func _init(
	p_id: String = "warrior",
	p_name: String = "Warrior",
	p_base_stats: Dictionary = {},
	p_growth: Dictionary = {},
	p_masteries: Dictionary = {},
	p_allowed_gear: Array[String] = []
) -> void:
	id = p_id
	display_name = p_name
	base_stats = p_base_stats
	growth_per_level = p_growth
	innate_masteries = p_masteries
	allowed_gear_types = p_allowed_gear

func get_base_stat(stat_name: String) -> int:
	var clean: String = stat_name.to_upper()
	if base_stats.has(clean):
		var raw: Variant = base_stats[clean]
		if raw is int:
			return raw
	return 10  # Standard human baseline

func get_growth_for_stat(stat_name: String) -> int:
	var clean: String = stat_name.to_upper()
	if growth_per_level.has(clean):
		var raw: Variant = growth_per_level[clean]
		if raw is int:
			return raw
	return 0

# Class-based gear equip check (Requirement 5)
# Accessories can be equipped by any class with no restriction
func can_equip_gear(eq: Equipment) -> bool:
	if eq == null:
		return false
	if eq.item_type == Item.ItemType.ACCESSORY or eq.slot in [Equipment.EquipSlot.ACCESSORY_1, Equipment.EquipSlot.ACCESSORY_2]:
		return true
	if eq.weapon_type.is_empty():
		return true
	if allowed_gear_types.is_empty():
		return true
	return eq.weapon_type.to_lower() in allowed_gear_types

# ------------------------------------------------------------------------------
# Built-in Standard Class Archetypes
# ------------------------------------------------------------------------------

static func create_warrior() -> ClassArchetype:
	return ClassArchetype.new(
		"warrior",
		"Warrior",
		{"STR": 16, "DEX": 10, "INT": 8, "CONS": 14, "PERC": 10, "KNOW": 8},
		{"STR": 2, "CONS": 2, "PERC": 1},
		{"sword": 1, "shield": 1},
		["sword", "axe", "mace", "hammer", "shield", "plate", "leather"]
	)

static func create_ranger() -> ClassArchetype:
	return ClassArchetype.new(
		"ranger",
		"Ranger",
		{"STR": 10, "DEX": 16, "INT": 10, "CONS": 10, "PERC": 14, "KNOW": 10},
		{"DEX": 2, "PERC": 2, "STR": 1},
		{"bow": 1, "dagger": 1, "wilderness": 1},
		["bow", "crossbow", "dagger", "leather", "cloth"]
	)

static func create_shaman() -> ClassArchetype:
	return ClassArchetype.new(
		"shaman",
		"Shaman",
		{"STR": 8, "DEX": 10, "INT": 16, "CONS": 10, "PERC": 12, "KNOW": 14},
		{"INT": 2, "KNOW": 2, "CONS": 1},
		{"staff": 1, "ancient_ruins": 1},
		["staff", "wand", "orb", "cloth", "leather"]
	)

static func create_brawler() -> ClassArchetype:
	return ClassArchetype.new(
		"brawler",
		"Brawler",
		{"STR": 14, "DEX": 12, "INT": 6, "CONS": 14, "PERC": 8, "KNOW": 6},
		{"STR": 2, "CONS": 2, "DEX": 1},
		{"fist": 1, "axe": 1},
		["fist", "axe", "leather"]
	)

static func create_scout() -> ClassArchetype:
	return ClassArchetype.new(
		"scout",
		"Scout",
		{"STR": 8, "DEX": 16, "INT": 10, "CONS": 8, "PERC": 16, "KNOW": 10},
		{"DEX": 2, "PERC": 2, "CONS": 1},
		{"dagger": 1, "bow": 1, "wilderness": 1},
		["dagger", "bow", "leather", "cloth"]
	)

static func create_berserker() -> ClassArchetype:
	return ClassArchetype.new(
		"berserker",
		"Berserker",
		{"STR": 18, "DEX": 12, "INT": 6, "CONS": 14, "PERC": 8, "KNOW": 6},
		{"STR": 3, "CONS": 2},
		{"axe": 2},
		["axe", "mace", "leather", "plate"]
	)
