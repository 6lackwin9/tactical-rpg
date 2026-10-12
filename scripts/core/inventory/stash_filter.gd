class_name StashFilter
extends RefCounted

# ==============================================================================
# Tactical RPG - Stash Filter Model
# Standard gear-slot filter (one slot at a time) combined with an optional custom
# filter across other dimensions. All active dimensions must match (AND); within
# one dimension, any selected value matches (OR).
#
#   slot_filter : ALL | MAIN_HAND | OFF_HAND | BODY | ACCESSORY | CONSUMABLE
#   stats       : item scales with, grants, or requires one of these stats
#   rarities    : Item.ItemRarity values
#   weapon_types: Equipment.weapon_type (sword, bow, plate, shield...)
#   damage_types: Item.damage_type (slash, pierce, blunt, magic, fire)
#   hand        : ANY | 1H | 2H (weapons only)
#   usable_only : only items the given hero's class can equip
#   search      : case-insensitive name substring
#   sort_mode   : DEFAULT | NAME | RARITY | POWER | WEIGHT
# ==============================================================================

const SLOT_FILTERS: Array[String] = ["ALL", "MAIN_HAND", "OFF_HAND", "BODY", "ACCESSORY", "CONSUMABLE"]
const STAT_KEYS: Array[String] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"]
const HAND_MODES: Array[String] = ["ANY", "1H", "2H"]
const SORT_MODES: Array[String] = ["DEFAULT", "NAME", "RARITY", "POWER", "WEIGHT"]

var slot_filter: String = "ALL"
var stats: Array[String] = []
var rarities: Array[int] = []
var weapon_types: Array[String] = []
var damage_types: Array[String] = []
var hand: String = "ANY"
var usable_only: bool = false
var search: String = ""
var sort_mode: String = "DEFAULT"

static func slot_filter_label(slot_key: String) -> String:
	match slot_key:
		"MAIN_HAND": return "Main Hand"
		"OFF_HAND": return "Off Hand"
		"BODY": return "Body"
		"ACCESSORY": return "Accessory"
		"CONSUMABLE": return "Consumable"
	return "All"

# Number of active custom dimensions (the slot filter is not counted).
func custom_count() -> int:
	var count: int = 0
	if not stats.is_empty(): count += 1
	if not rarities.is_empty(): count += 1
	if not weapon_types.is_empty(): count += 1
	if not damage_types.is_empty(): count += 1
	if hand != "ANY": count += 1
	if usable_only: count += 1
	if not search.strip_edges().is_empty(): count += 1
	return count

func reset_custom() -> void:
	stats.clear()
	rarities.clear()
	weapon_types.clear()
	damage_types.clear()
	hand = "ANY"
	usable_only = false
	search = ""
	sort_mode = "DEFAULT"

# Toggles membership of `value` in a multi-select dimension.
func toggle_stat(value: String) -> void:
	_toggle_string(stats, value)

func toggle_weapon_type(value: String) -> void:
	_toggle_string(weapon_types, value)

func toggle_damage_type(value: String) -> void:
	_toggle_string(damage_types, value)

func toggle_rarity(value: int) -> void:
	if rarities.has(value):
		rarities.erase(value)
	else:
		rarities.append(value)

func _toggle_string(list: Array[String], value: String) -> void:
	if list.has(value):
		list.erase(value)
	else:
		list.append(value)

# Standard gear-slot filter: which items can be used in / belong to the chosen slot.
func matches_slot(item: Item) -> bool:
	if slot_filter == "ALL":
		return true
	if slot_filter == "CONSUMABLE":
		return item.item_type == Item.ItemType.CONSUMABLE
	if not (item is Equipment):
		return false
	var eq: Equipment = item
	match slot_filter:
		"MAIN_HAND":
			return eq.is_weapon() and eq.can_equip_in_slot(Equipment.EquipSlot.MAIN_HAND)
		"OFF_HAND":
			return eq.can_equip_in_slot(Equipment.EquipSlot.OFF_HAND) and (eq.is_weapon() or eq.is_shield())
		"BODY":
			return eq.item_type == Item.ItemType.ARMOR and eq.slot == Equipment.EquipSlot.BODY
		"ACCESSORY":
			return eq.item_type == Item.ItemType.ACCESSORY
	return true

func matches_custom(item: Item, hero: Combatant) -> bool:
	if not search.strip_edges().is_empty() and not item.name.to_lower().contains(search.strip_edges().to_lower()):
		return false
	if not rarities.is_empty() and not rarities.has(int(item.rarity)):
		return false
	if not damage_types.is_empty() and not damage_types.has(item.damage_type):
		return false

	var eq: Equipment = item as Equipment
	if not stats.is_empty():
		if eq == null or not _touches_stat(eq):
			return false
	if not weapon_types.is_empty():
		if eq == null or not weapon_types.has(eq.weapon_type):
			return false
	if hand != "ANY":
		if eq == null or not eq.is_weapon():
			return false
		if (hand == "2H") != eq.is_two_handed:
			return false
	if usable_only and eq != null and hero != null and hero.stats != null and hero.stats.archetype != null:
		if not hero.stats.archetype.can_equip_gear(eq):
			return false
	return true

# True if the item scales with, grants a bonus to, or requires one of the selected stats.
func _touches_stat(eq: Equipment) -> bool:
	for key: String in stats:
		if eq.is_weapon() and eq.scaling_stat == key:
			return true
		if eq.stat_bonuses.has(key) or eq.required_stats.has(key):
			return true
	return false

func matches(item: Item, hero: Combatant) -> bool:
	return matches_slot(item) and matches_custom(item, hero)

# Returns the filtered, sorted view of `source` without modifying it.
func apply(source: Array[Item], hero: Combatant) -> Array[Item]:
	var result: Array[Item] = []
	for item: Item in source:
		if matches(item, hero):
			result.append(item)
	match sort_mode:
		"NAME":
			result.sort_custom(func(a: Item, b: Item) -> bool: return a.name < b.name)
		"RARITY":
			result.sort_custom(func(a: Item, b: Item) -> bool: return int(a.rarity) > int(b.rarity))
		"POWER":
			result.sort_custom(func(a: Item, b: Item) -> bool: return _power_of(a) > _power_of(b))
		"WEIGHT":
			result.sort_custom(func(a: Item, b: Item) -> bool: return a.weight > b.weight)
	return result

static func _power_of(item: Item) -> int:
	var eq: Equipment = item as Equipment
	if eq != null:
		return eq.base_power + eq.mitigation
	return 0
