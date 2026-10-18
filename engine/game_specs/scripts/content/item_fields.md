---
class: ItemFields
extends: RefCounted
description: "The items field table of ContentSchema (moved here by CLN-9b; ContentSchema.fields(\"items\") / field() are the API)."
---

# Properties
* `handed`: Dictionary = {"key": "item_type", "in": ItemFamilies.HAND_TYPES}
* `weapon`: Dictionary = {"key": "item_type", "in": ["WEAPON"]}
* `slotted`: Dictionary = {"key": "item_type", "in": ItemFamilies.SLOT_TYPES}
* `equip`: Dictionary = {"key": "item_type", "in": ContentSchema.EQUIP_TYPES}
* `weapon`: Dictionary = {"key": "item_type", "in": ["WEAPON"]}
* `consumable`: Dictionary = {"key": "item_type", "in": ["CONSUMABLE"]}
* `handed`: Dictionary = {"key": "item_type", "in": ItemFamilies.HAND_TYPES}

# Methods

### `static _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary`
```logic
return ContentSchema._f(key, label, type, default, extra)
```

