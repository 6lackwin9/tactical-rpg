---
class: ItemFamilies
extends: RefCounted
description: "Tactical RPG - Item families (FORGE1_ITEMS section 1): the TEMPLATE of a base item A family fixes item type, hand category, slots, scaling stats, base numbers per hand, soft bands, the modifier policy and the closed engine rules; a base item is family + modifiers (ItemModifiers). Pure static helpers over family Dictionaries (rows of the `item_families` catalog kind)."
---

# Properties
* `HAND_MAIN_1H`: String = "main_1h" (const)
* `HAND_MAIN_2H`: String = "main_2h" (const)
* `HAND_OFF`: String = "off" (const)
* `HANDS`: Array[String] = [HAND_MAIN_1H, HAND_MAIN_2H, HAND_OFF] (const)
* `MAIN_HANDS`: Array[String] = [HAND_MAIN_1H, HAND_MAIN_2H] (const)
* `NO_HAND_KEY`: String = "none"   # `base` / `bands` key of families without hands (armor, accessories, ...) (const)
* `ITEM_TYPES`: Array[String] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY", "CONSUMABLE", "MATERIAL", "QUEST"] (const)
* `GEAR_TYPES`: Array[String] = ["WEAPON", "SHIELD", "ARMOR"]   # family id = a Proficiency.gear_types() token (const)
* `MODIFIABLE_TYPES`: Array[String] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY"] (const)
* `OFF_CATEGORIES`: Array[String] = ["shield", "orb", "tome"] (const)
* `ARMOR_SLOTS`: Array[String] = ["HEAD", "BODY"] (const)
* `ACCESSORY_SLOTS`: Array[String] = ["ACCESSORY_1", "ACCESSORY_2"] (const)
* `SCALING_STATS`: Array[String] = ["STR", "DEX", "INT"]   # weapons never scale from KNOW (const)
* `NUMBER_FIELDS`: Array[String] = ["power", "range", "block_cost", "burden", "mitigation"] (const)
* `NUMBER_RANGES`: Dictionary = {"power": [0, 200], "range": [1, 6], "block_cost": [1, 6], "burden": [0, 10], "mitigation": [0, 100]} (const)
* `FLAT_KEYS`: Dictionary = {"power": "base_power", "range": "base_range", "block_cost": "block_cost", "burden": "burden", "mitigation": "mitigation"} (const)
* `BONUS_BAND`: String = "bonus_value"   # band of the GearBudget value of the stat_bonus + effect modifiers (const)
* `RULE_FOCUS`: String = "focus" (const)
* `RULE_ALWAYS_RANGED`: String = "always_ranged" (const)
* `RULE_SHIELD`: String = "shield" (const)
* `RULE_SPELL_SOURCE`: String = "spell_source" (const)
* `ENGINE_RULES`: Array[String] = [RULE_FOCUS, RULE_ALWAYS_RANGED, RULE_SHIELD, RULE_SPELL_SOURCE] (const)
* `RULE_HELP`: Dictionary = { (const)
* `HAND_TYPES`: Array[String] = ["WEAPON", "SHIELD"]   # types with hands (slot derived from the hand) (const)
* `SLOT_TYPES`: Array[String] = ["ARMOR", "ACCESSORY"]   # types that list their slots (const)
* `none`: Array[String] = []
* `out`: Array[String] = ["block_cost", "burden", "mitigation"]
* `none`: Array[String] = []
* `out`: Array[String] = []

# Methods

### `static item_type_of(family: Dictionary) -> String`
```logic
return str(family.get("item_type", "WEAPON"))
```

### `static is_gear_type(item_type: String) -> bool`
```logic
return GEAR_TYPES.has(item_type)
```

### `static category(family: Dictionary) -> String`
```logic
var hands: Array[String] = hands_of(family)
if hands.is_empty():
	return ""
if hands.size() == 1 and hands[0] == HAND_OFF:
	return "off"
for h: String in hands:
	if not MAIN_HANDS.has(h):
		return ""
return "main"
```

### `static off_hand_category(family: Dictionary) -> String`
```logic
var id: String = str(family.get("id", ""))
return id if category(family) == "off" and OFF_CATEGORIES.has(id) else ""
```

### `static default_hand(family: Dictionary) -> String`
```logic
var hands: Array[String] = hands_of(family)
var wanted: String = str(family.get("default_hand", ""))
if hands.has(wanted):
	return wanted
return hands[0] if not hands.is_empty() else ""
```

### `static hand_key(family: Dictionary, hand: String) -> String`
```logic
if hands_of(family).is_empty():
	return NO_HAND_KEY
return hand if hands_of(family).has(hand) else default_hand(family)
```

### `static base_of(family: Dictionary, hand: String) -> Dictionary`
```logic
var all: Dictionary = DictUtil.dict_of(family.get("base", {}))
var row: Dictionary = DictUtil.dict_of(all.get(hand_key(family, hand), {}))
var out: Dictionary = {}
for field: String in NUMBER_FIELDS:
	var rng: Array = NUMBER_RANGES[field]
	out[field] = DictUtil.to_int(row.get(field, rng[0]), DictUtil.to_int(rng[0]))
return out
```

### `static band_of(family: Dictionary, hand: String, field: String) -> Dictionary`
```logic
var all: Dictionary = DictUtil.dict_of(family.get("bands", {}))
var row: Dictionary = DictUtil.dict_of(all.get(hand_key(family, hand), {}))
return DictUtil.dict_of(row.get(field, {}))
```

### `static has_rule(family: Dictionary, rule: String) -> bool`
```logic
return ContentFactory._strings(family.get("engine_rules", [])).has(rule)
```

### `static family_id_for_row(row: Dictionary) -> String`
```logic
var gear: String = str(row.get("weapon_type", ""))
if not gear.is_empty():
	return gear
match str(row.get("item_type", "")):
	"ACCESSORY":
		return "accessory"
	"CONSUMABLE":
		return "consumable"
	"MATERIAL":
		return "material"
	"QUEST":
		return "quest"
return ""
```

### `static derive(family: Dictionary) -> Dictionary`
```logic
var out: Dictionary = family.duplicate(true)
var type: String = item_type_of(out)
var empty: Array[String] = []
if not HAND_TYPES.has(type):
	out["hands"] = empty.duplicate()
if type != "WEAPON":
	out["scaling_stats"] = empty.duplicate()
if not SLOT_TYPES.has(type):
	out["slots"] = empty.duplicate()
out["off_hand_category"] = off_hand_category(out)
out["default_hand"] = default_hand(out)
return out
```

