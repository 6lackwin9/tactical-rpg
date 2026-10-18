---
class: StashFilter
extends: RefCounted
description: "Tactical RPG - Stash Filter Model Standard gear-slot filter (one slot at a time) combined with an optional custom filter across other dimensions. All active dimensions must match (AND); within one dimension, any selected value matches (OR). slot_filter : ALL | MAIN_HAND | OFF_HAND | BODY | ACCESSORY | CONSUMABLE stats       : item scales with, grants, or requires one of these stats rarities    : Item.ItemRarity values weapon_types: Equipment.weapon_type (sword, bow, plate, shield...) damage_types: Item.damage_type (slash, pierce, blunt, magic, fire) hand        : ANY | 1H | 2H (weapons only) usable_only : only items the given hero's class can equip search      : case-insensitive name substring sort_mode   : DEFAULT | NAME | RARITY | POWER | WEIGHT"
---

# Properties
* `SLOT_FILTERS`: Array[String] = ["ALL", "MAIN_HAND", "OFF_HAND", "BODY", "ACCESSORY", "CONSUMABLE"] (const)
* `STAT_KEYS`: Array[String] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"] (const)
* `HAND_MODES`: Array[String] = ["ANY", "1H", "2H"] (const)
* `SORT_MODES`: Array[String] = ["DEFAULT", "NAME", "RARITY", "POWER", "WEIGHT"] (const)
* `slot_filter`: String = "ALL"
* `stats`: Array[String] = []
* `rarities`: Array[int] = []
* `weapon_types`: Array[String] = []
* `damage_types`: Array[String] = []
* `hand`: String = "ANY"
* `usable_only`: bool = false
* `search`: String = ""
* `sort_mode`: String = "DEFAULT"
* `result`: Array[Item] = []

# Methods

### `static slot_filter_label(slot_key: String) -> String`
```logic
match slot_key:
	"MAIN_HAND": return "Main Hand"
	"OFF_HAND": return "Off Hand"
	"BODY": return "Body"
	"ACCESSORY": return "Accessory"
	"CONSUMABLE": return "Consumable"
return "All"
```

### `custom_count() -> int`
```logic
var count: int = 0
if not stats.is_empty(): count += 1
if not rarities.is_empty(): count += 1
if not weapon_types.is_empty(): count += 1
if not damage_types.is_empty(): count += 1
if hand != "ANY": count += 1
if usable_only: count += 1
if not search.strip_edges().is_empty(): count += 1
return count
```

### `reset_custom() -> void`
```logic
stats.clear()
rarities.clear()
weapon_types.clear()
damage_types.clear()
hand = "ANY"
usable_only = false
search = ""
sort_mode = "DEFAULT"
```

### `toggle_stat(value: String) -> void`
```logic
_toggle_string(stats, value)
```

### `toggle_weapon_type(value: String) -> void`
```logic
_toggle_string(weapon_types, value)
```

### `toggle_damage_type(value: String) -> void`
```logic
_toggle_string(damage_types, value)
```

### `toggle_rarity(value: int) -> void`
```logic
if rarities.has(value):
	rarities.erase(value)
else:
	rarities.append(value)
```

### `_toggle_string(list: Array[String], value: String) -> void`
```logic
if list.has(value):
	list.erase(value)
else:
	list.append(value)
```

### `matches_slot(item: Item) -> bool`
```logic
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
```

### `matches_custom(item: Item, hero: Combatant) -> bool`
```logic
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
```

### `_touches_stat(eq: Equipment) -> bool`
```logic
for key: String in stats:
	if eq.is_weapon() and eq.scaling_stat == key:
		return true
	if eq.stat_bonuses.has(key) or eq.required_stats.has(key):
		return true
return false
```

### `matches(item: Item, hero: Combatant) -> bool`
```logic
return matches_slot(item) and matches_custom(item, hero)
```

### `static _power_of(item: Item) -> int`
```logic
var eq: Equipment = item as Equipment
if eq != null:
	return eq.base_power + eq.mitigation
return 0
```

