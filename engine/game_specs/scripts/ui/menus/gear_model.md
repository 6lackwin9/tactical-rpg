---
class: GearModel
extends: RefCounted
description: "Tactical RPG - Gear model (UI2-L rule 6): a draft of one member's equipment (slot -> item id; unlisted = the class default) and of the party stash. equip() moves a stash item onto the member and the replaced explicit item back to the stash; unequip() returns the explicit item and falls back to the class default (the \"none\" item is not offered). Each equip is checked by UnitState.validate (gear messages verbatim); confirm() writes both drafts back. ITEMUSE1 rule 1: the battle pouch drafts the same way: pouch_add() moves one consumable from the stash into the next pouch slot, pouch_remove() returns a slot's item to the stash; confirm() writes the pouch with the gear."
---

# Properties
* `member_id`: String = ""
* `work`: UnitState = null
* `stash`: Dictionary = {}   # item id -> count (draft)
* `out`: Array[Dictionary] = []
* `c`: Combatant = PartyBuilder.build(work, false) if work != null else null
* `explicit`: bool = work != null and work.equipment.has(slot)
* `item_id`: String = str(work.equipment[slot]) if explicit else ""
* `idx`: int = Equipment.EquipSlot.keys().find(slot.to_upper())
* `eq`: Equipment = c.inventory.get_equipped(idx as Equipment.EquipSlot) if idx >= 0 else null
* `out`: Array[Dictionary] = []
* `ids`: Array = stash.keys()
* `id`: String = str(raw)
* `entry`: Dictionary = ContentCatalog.get_entry("items", id)
* `fits`: Array[String] = []
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `trial`: UnitState = work.duplicate_state()
* `errors`: Array[String] = []
* `out`: Array[Dictionary] = []
* `out`: Array[Dictionary] = []
* `errors`: Array[String] = []
* `problem`: String = _pouch_problem(item_id)
* `errors`: Array[String] = []
* `errors`: Array[String] = _gear_errors(work)
* `m`: UnitState = _member()

# Methods

### `_init(p_member: String) -> void`
```logic
member_id = p_member
cancel()
```

### `_member() -> UnitState`
```logic
return PartyPreview.member(member_id)
```

### `cancel() -> void`
```logic
var m: UnitState = _member()
work = m.duplicate_state() if m != null else null
stash = CurrentGame.state().party.stash.duplicate() if CurrentGame.has_game() else {}
```

### `has_draft() -> bool`
```logic
var m: UnitState = _member()
return m != null and (not work.equals(m) or not JsonUtil.deep_equal(stash, CurrentGame.state().party.stash))
```

### `static item_name(item_id: String) -> String`
```logic
return str(ContentCatalog.get_entry("items", item_id).get("name", item_id)) if not item_id.is_empty() else ""
```

### `_pouch_problem(item_id: String) -> String`
```logic
var entry: Dictionary = ContentCatalog.get_entry("items", item_id)
if str(entry.get("item_type", "")) != "CONSUMABLE":
	return "'%s' is not a consumable." % item_name(item_id)
var built: Item = ContentFactory.item(entry)
if not (built is Consumable) or not ItemCommand.offered(built as Consumable):
	return "%s can't be used in battle." % item_name(item_id)
return ""
```

### `_to_stash(item_id: String) -> void`
```logic
stash[item_id] = DictUtil.to_int(stash.get(item_id, 0)) + 1
```

### `_from_stash(item_id: String) -> void`
```logic
var left: int = DictUtil.to_int(stash.get(item_id, 0)) - 1
if left <= 0:
	stash.erase(item_id)
else:
	stash[item_id] = left
```

