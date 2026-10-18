---
class: PartyGearBridge
extends: RefCounted
description: "Tactical RPG - Party gear bridge (UI2-L rule 6): runs CharacterEquipmentModal on a GearModel draft. Builds the member's Combatant (PartyBuilder.build) and a stash Inventory from the draft stash, and maps each equipment_updated back to GearModel.equip / unequip so the validator stays the only rule. A refused change rebuilds the modal from the draft."
---

# Properties
* `model`: GearModel = null
* `hero`: Combatant = null
* `stash_inventory`: Inventory = null
* `last_errors`: Array[String] = []
* `slot`: String = str(row["slot"])
* `now`: String = equipped_id(c, slot)
* `errors`: Array[String] = model.unequip(slot) if now.is_empty() else model.equip(slot, now)

# Methods

### `_init(p_model: GearModel) -> void`
```logic
model = p_model
rebuild()
```

### `rebuild() -> void`
```logic
hero = PartyBuilder.build(model.work, false) if model.work != null else null
stash_inventory = Inventory.new()
for row: Dictionary in model.stash_rows():
	var entry: Dictionary = ContentCatalog.get_entry("items", str(row["item"]))
	for _i: int in range(DictUtil.to_int(row["count"])):
		var item: Item = ContentFactory.item(entry)
		if item != null:
			stash_inventory.add_item(item)
```

### `attach(modal: CharacterEquipmentModal) -> void`
```logic
var allies: Array[Combatant] = [hero]
modal.setup_heroes(allies, stash_inventory)
if not modal.equipment_updated.is_connected(_on_updated):
	modal.equipment_updated.connect(_on_updated.bind(modal))
```

### `static equipped_id(c: Combatant, slot: String) -> String`
```logic
var idx: int = Equipment.EquipSlot.keys().find(slot.to_upper())
var eq: Equipment = c.inventory.get_equipped(idx as Equipment.EquipSlot) if idx >= 0 and c != null else null
return eq.id if eq != null else ""
```

### `_on_updated(c: Combatant, modal: CharacterEquipmentModal) -> void`
```logic
if not sync(c).is_empty():
	rebuild()
	attach(modal)
```

