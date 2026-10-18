---
class: ClassUnitFields
extends: RefCounted
description: "The classes and units field tables of ContentSchema plus their shared gear slots (moved here by CLN-9b; ContentSchema.fields(\"classes\" / \"units\") / field() are the API)."
---

# Properties
* `CLASS_GROUPS`: Array[String] = ["identity", "stats", "proficiency", "skills", "loadout"] (const)
* `UNIT_AI_GROUP`: String = "ai" (const)
* `UNIT_AI_ADVANCED_GROUP`: String = "ai_advanced" (const)
* `AI_ROLES`: Array[String] = ["AGGRESSOR", "SKIRMISHER", "SNIPER", "SUPPORT"] (const)
* `ID_MIN_LEN`: int = 2 (const)
* `ID_MAX_LEN`: int = 24 (const)
* `NAME_MAX_LEN`: int = 32 (const)
* `DESCRIPTION_MAX_LEN`: int = 240 (const)
* `HEAL_BELOW_MIN`: float = 0.1 (const)
* `HEAL_BELOW_MAX`: float = 1.0 (const)
* `LIST_PATH_CONVERTER`: bool = false (const)
* `out`: Array[Dictionary] = []
* `pouch_opts`: Dictionary = {"ref": "items", "group": "loadout", "ref_filter": {"item_type": ["CONSUMABLE"]}}
* `list`: Array[Dictionary] = [
* `fixed`: Dictionary = {"key": "stat_mode", "in": ["fixed"]}
* `list`: Array[Dictionary] = [

# Methods

### `static _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary`
```logic
return ContentSchema._f(key, label, type, default, extra)
```

