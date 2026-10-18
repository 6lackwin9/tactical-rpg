---
class: Item
extends: RefCounted
description: "Tactical RPG - Base Item Model"
---

# Properties
* `id`: String = ""
* `name`: String = "Item"
* `description`: String = ""
* `item_type`: ItemType = ItemType.MATERIAL
* `rarity`: ItemRarity = ItemRarity.COMMON
* `weight`: float = 1.0
* `value_gold`: int = 10
* `max_stack`: int = 1
* `damage_type`: String = ""  # slash, pierce, blunt, magic, fire (empty = non-damaging)
* `icon_id`: String = ""      # optional item art key (ItemArt.icon_key); empty = icon by gear type
* `special`: bool = false       # may exceed the class gear budget (GearBudget)
* `special_note`: String = "" # documented trade-off of a special item

# Methods

### `is_weapon() -> bool`
```logic
return item_type == ItemType.WEAPON
```

### `is_shield() -> bool`
```logic
return item_type == ItemType.SHIELD
```

### `hand() -> String`
```logic
return ""
```

### `static type_to_string(p_type: ItemType) -> String`
```logic
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
```

