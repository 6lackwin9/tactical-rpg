---
class: Inventory
extends: RefCounted
description: "Tactical RPG - Inventory & Equipment Loadout Model Manages party bag, equipped slots, and 3-slot combat battle pouches"
---

# Properties
* `items`: Array[Item] = []
* `equipped_gear`: Dictionary = {}  # Equipment.EquipSlot (int) -> Equipment
* `battle_pouch`: Array[Consumable] = []
* `max_pouch_slots`: int = 3
* `active_weapon_slot`: Equipment.EquipSlot = Equipment.EquipSlot.MAIN_HAND
* `result`: Array[Equipment] = []
* `raw_eq`: Variant = equipped_gear[slot_key]
* `eq`: Equipment = raw_eq

# Signals
* `item_added(item: Item)`
* `item_removed(item: Item)`
* `equipment_changed(slot: Equipment.EquipSlot, item: Equipment)`

# Methods

### `_init() -> void`
```logic
equipped_gear.clear()
battle_pouch.clear()
```

### `equip(equipment: Equipment, slot_override: int = -1) -> Equipment`
```logic
if equipment == null:
	return null

var target_slot: Equipment.EquipSlot = equipment.slot
if slot_override >= 0 and slot_override in Equipment.EquipSlot.values():
	target_slot = slot_override as Equipment.EquipSlot

var previous: Equipment = null
if equipped_gear.has(target_slot):
	var prev_raw: Variant = equipped_gear[target_slot]
	if prev_raw is Equipment:
		previous = prev_raw
		items.append(previous)

equipped_gear[target_slot] = equipment
if items.has(equipment):
	items.erase(equipment)

_normalize_active_weapon()
equipment_changed.emit(target_slot, equipment)
return previous
```

### `unequip(slot: Equipment.EquipSlot) -> Equipment`
```logic
if not equipped_gear.has(slot):
	return null

var raw_eq: Variant = equipped_gear[slot]
if raw_eq is Equipment:
	var eq: Equipment = raw_eq
	equipped_gear.erase(slot)
	items.append(eq)
	_normalize_active_weapon()
	equipment_changed.emit(slot, null)
	return eq
return null
```

### `get_equipped(slot: Equipment.EquipSlot) -> Equipment`
```logic
if equipped_gear.has(slot):
	var raw_eq: Variant = equipped_gear[slot]
	if raw_eq is Equipment:
		return raw_eq
return null
```

### `get_main_weapon() -> Equipment`
```logic
return get_equipped(Equipment.EquipSlot.MAIN_HAND)
```

### `get_offhand_weapon() -> Equipment`
```logic
var off_eq: Equipment = get_equipped(Equipment.EquipSlot.OFF_HAND)
if off_eq != null and off_eq.item_type == Item.ItemType.WEAPON:
	return off_eq
return null
```

### `needs_weapon_switch() -> bool`
```logic
var main_w: Equipment = get_main_weapon()
return main_w != null and main_w.is_two_handed and get_offhand_weapon() != null
```

### `get_active_weapon() -> Equipment`
```logic
if needs_weapon_switch() and active_weapon_slot == Equipment.EquipSlot.OFF_HAND:
	return get_offhand_weapon()
return get_main_weapon()
```

### `is_offhand_active() -> bool`
```logic
return needs_weapon_switch() and active_weapon_slot == Equipment.EquipSlot.OFF_HAND
```

### `toggle_active_weapon() -> bool`
```logic
if not needs_weapon_switch():
	active_weapon_slot = Equipment.EquipSlot.MAIN_HAND
	return false
active_weapon_slot = Equipment.EquipSlot.MAIN_HAND if is_offhand_active() else Equipment.EquipSlot.OFF_HAND
return true
```

### `_normalize_active_weapon() -> void`
```logic
if not needs_weapon_switch():
	active_weapon_slot = Equipment.EquipSlot.MAIN_HAND
```

### `get_shield() -> Equipment`
```logic
var off_eq: Equipment = get_equipped(Equipment.EquipSlot.OFF_HAND)
if off_eq != null and (off_eq.item_type == Item.ItemType.SHIELD or off_eq.off_hand_category == "shield"):
	return off_eq
return null
```

### `get_total_armor_burden() -> int`
```logic
var total: int = 0
for eq: Equipment in _iter_equipped_gear():
	total += eq.burden
return total
```

### `get_total_shield_mitigation(stats: CharacterStats = null) -> int`
```logic
var off: Equipment = get_equipped(Equipment.EquipSlot.OFF_HAND)
return _mitigation_of(off, stats) if off != null else 0
```

### `get_total_armor_mitigation(stats: CharacterStats = null) -> int`
```logic
var body: Equipment = get_equipped(Equipment.EquipSlot.BODY)
return _mitigation_of(body, stats) if body != null else 0
```

### `get_total_flat_mitigation(stats: CharacterStats = null) -> int`
```logic
var total: int = 0
for eq: Equipment in _iter_equipped_gear():
	total += _mitigation_of(eq, stats)
return total
```

### `static _mitigation_of(eq: Equipment, stats: CharacterStats) -> int`
```logic
if stats == null:
	return eq.mitigation
return eq.get_effective_mitigation(stats, stats.proficiency)
```

### `get_total_gear_stat_bonus(stat_name: String) -> int`
```logic
var total: int = 0
var clean: String = stat_name.to_upper()
for eq: Equipment in _iter_equipped_gear():
	if eq.stat_bonuses.has(clean):
		var b: Variant = eq.stat_bonuses[clean]
		if b is int:
			total += b
return total
```

### `add_item(item: Item) -> bool`
```logic
if item == null:
	return false
items.append(item)
item_added.emit(item)
return true
```

### `remove_item(item: Item) -> bool`
```logic
if items.has(item):
	items.erase(item)
	item_removed.emit(item)
	return true
return false
```

### `assign_pouch_item(index: int, consumable: Consumable) -> bool`
```logic
if index < 0 or index >= max_pouch_slots or consumable == null:
	return false

while battle_pouch.size() <= index:
	battle_pouch.append(null)

battle_pouch[index] = consumable
if items.has(consumable):
	items.erase(consumable)
return true
```

### `use_pouch_item(index: int, target: Combatant) -> Dictionary`
```logic
if index < 0 or index >= battle_pouch.size():
	return {}

var item: Consumable = battle_pouch[index]
if item == null:
	return {}

var outcome: Dictionary = item.apply_to_combatant(target)
battle_pouch.remove_at(index)
return outcome
```

### `add_to_battle_pouch(consumable: Consumable) -> bool`
```logic
if consumable == null or battle_pouch.size() >= max_pouch_slots:
	return false
battle_pouch.append(consumable)
if items.has(consumable):
	items.erase(consumable)
return true
```

### `remove_from_battle_pouch(consumable: Consumable) -> bool`
```logic
if battle_pouch.has(consumable):
	battle_pouch.erase(consumable)
	return true
return false
```

