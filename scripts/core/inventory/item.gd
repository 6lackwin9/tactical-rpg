class_name Item
extends RefCounted

# ==============================================================================
# Tactical RPG - Base Item Model
# ==============================================================================

enum ItemType {
	WEAPON,
	SHIELD,
	ARMOR,
	ACCESSORY,
	CONSUMABLE,
	MATERIAL,
	QUEST
}

enum ItemRarity {
	COMMON,
	UNCOMMON,
	RARE,
	EPIC,
	LEGENDARY
}

var id: String = ""
var name: String = "Item"
var description: String = ""
var item_type: ItemType = ItemType.MATERIAL
var rarity: ItemRarity = ItemRarity.COMMON
var weight: float = 1.0
var value_gold: int = 10
var max_stack: int = 1

func _init(
	p_id: String = "",
	p_name: String = "Item",
	p_type: ItemType = ItemType.MATERIAL,
	p_weight: float = 1.0,
	p_value: int = 10,
	p_rarity: ItemRarity = ItemRarity.COMMON,
	p_desc: String = "",
	p_max_stack: int = 1
) -> void:
	id = p_id
	name = p_name
	item_type = p_type
	weight = p_weight
	value_gold = p_value
	rarity = p_rarity
	description = p_desc
	max_stack = p_max_stack

func get_rarity_color() -> Color:
	match rarity:
		ItemRarity.COMMON:
			return Color(0.85, 0.85, 0.85)
		ItemRarity.UNCOMMON:
			return Color(0.3, 0.9, 0.4)
		ItemRarity.RARE:
			return Color(0.3, 0.6, 1.0)
		ItemRarity.EPIC:
			return Color(0.7, 0.35, 1.0)
		ItemRarity.LEGENDARY:
			return Color(1.0, 0.75, 0.2)
	return Color.WHITE

func is_weapon() -> bool:
	return item_type == ItemType.WEAPON

func is_shield() -> bool:
	return item_type == ItemType.SHIELD

static func type_to_string(p_type: ItemType) -> String:
	match p_type:
		ItemType.WEAPON:
			return "Weapon"
		ItemType.SHIELD:
			return "Shield"
		ItemType.ARMOR:
			return "Armor"
		ItemType.ACCESSORY:
			return "Accessory"
		ItemType.CONSUMABLE:
			return "Consumable"
		ItemType.MATERIAL:
			return "Material"
		ItemType.QUEST:
			return "Quest Item"
		_:
			return "Item"
