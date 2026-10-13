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
# Catalog archetypes (res://data/catalog/classes.json, edited in the Content Forge)
# ------------------------------------------------------------------------------

# Class `class_id` from the content catalog (a plain 10-stat class if it is missing).
static func create(class_id: String) -> ClassArchetype:
	if ContentCatalog.has("classes", class_id):
		return ContentFactory.class_archetype(ContentCatalog.get_entry("classes", class_id))
	push_warning("ClassArchetype: unknown class '%s'" % class_id)
	return ClassArchetype.new(class_id, class_id.capitalize())

static func create_warrior() -> ClassArchetype:
	return create("warrior")

static func create_ranger() -> ClassArchetype:
	return create("ranger")

static func create_shaman() -> ClassArchetype:
	return create("shaman")

static func create_brawler() -> ClassArchetype:
	return create("brawler")

static func create_scout() -> ClassArchetype:
	return create("scout")

static func create_berserker() -> ClassArchetype:
	return create("berserker")