---
class: ItemCatalog
extends: RefCounted
description: "Tactical RPG - Item & Equipment Catalog Items are authored in the content catalog (res://data/catalog/items.json, edited in the Content Forge); create(id) builds a fresh instance. The create_* helpers below are shorthands for the items the code, dojo and tests use by name. A new instance of catalog item `id` (null if unknown)."
---

# Methods

### `static create(id: String) -> Item`
```logic
if not ContentCatalog.has("items", id):
	push_error("ItemCatalog: unknown item '%s'" % id)
	return null
return ContentFactory.item(ContentCatalog.get_entry("items", id))
```

### `static create_equipment(id: String) -> Equipment`
```logic
var it: Item = create(id)
return it as Equipment
```

### `static create_consumable(id: String) -> Consumable`
```logic
var it: Item = create(id)
return it as Consumable
```

### `static create_iron_claymore() -> Equipment`
```logic
return create_equipment("wpn_claymore")
```

### `static create_titan_warhammer() -> Equipment`
```logic
return create_equipment("wpn_warhammer")
```

### `static create_oak_shortbow() -> Equipment`
```logic
return create_equipment("wpn_shortbow")
```

### `static create_steel_broadsword() -> Equipment`
```logic
return create_equipment("wpn_broadsword")
```

### `static create_iron_shortsword() -> Equipment`
```logic
return create_equipment("wpn_shortsword")
```

### `static create_shadow_dagger() -> Equipment`
```logic
return create_equipment("wpn_dagger")
```

### `static create_mystic_tome() -> Equipment`
```logic
return create_equipment("wpn_tome")
```

### `static create_wand() -> Equipment`
```logic
return create_equipment("wpn_wand")
```

### `static create_runed_staff() -> Equipment`
```logic
return create_equipment("wpn_staff")
```

### `static create_iron_kite_shield() -> Equipment`
```logic
return create_equipment("shd_kite")
```

### `static create_buckler() -> Equipment`
```logic
return create_equipment("shd_buckler")
```

### `static create_steel_plate_cuirass() -> Equipment`
```logic
return create_equipment("arm_plate")
```

### `static create_leather_brigandine() -> Equipment`
```logic
return create_equipment("arm_leather")
```

### `static create_ring_of_the_bear() -> Equipment`
```logic
return create_equipment("acc_bear_ring")
```

### `static create_swiftness_amulet() -> Equipment`
```logic
return create_equipment("acc_swift_amulet")
```

### `static create_healing_draught() -> Consumable`
```logic
return create_consumable("con_heal_potion")
```

### `static create_focus_elixir() -> Consumable`
```logic
return create_consumable("con_focus_elixir")
```

### `static create_throwing_fire_bomb() -> Consumable`
```logic
return create_consumable("con_fire_bomb")
```

### `static populate_party_starter_stash(inv: Inventory) -> void`
```logic
if inv == null:
	return
for entry: Dictionary in ContentCatalog.entries("items"):
	if DictUtil.to_bool(entry.get("starter_stash", false)):
		inv.add_item(ContentFactory.item(entry))
```

### `static setup_default_hero_loadout(hero: Combatant, archetype_name: String) -> void`
```logic
if hero == null or hero.inventory == null:
	return

if archetype_name == "Warrior":
	hero.inventory.equip(create_steel_broadsword())
	hero.inventory.equip(create_buckler(), Equipment.EquipSlot.OFF_HAND)
	hero.inventory.equip(create_leather_brigandine(), Equipment.EquipSlot.BODY)
	hero.inventory.equip(create_ring_of_the_bear(), Equipment.EquipSlot.ACCESSORY_1)
	hero.inventory.add_to_battle_pouch(create_healing_draught())
	hero.inventory.add_to_battle_pouch(create_throwing_fire_bomb())
elif archetype_name == "Ranger":
	hero.inventory.equip(create_oak_shortbow())
	hero.inventory.equip(create_leather_brigandine(), Equipment.EquipSlot.BODY)
	hero.inventory.equip(create_swiftness_amulet(), Equipment.EquipSlot.ACCESSORY_2)
	hero.inventory.add_to_battle_pouch(create_focus_elixir())
	hero.inventory.add_to_battle_pouch(create_healing_draught())

hero.sync_from_stats_and_gear()
```

