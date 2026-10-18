---
class: TerrainFields
extends: RefCounted
description: "The terrain field table of ContentSchema (S12T-L; ContentSchema.fields(\"terrain\") / field() are the API). One row per Terrain.Type: numbers only (types, walkability and glyphs stay in Terrain). Ranges are checked by TerrainValidator (no min / max here, so an out-of-range value reaches the validator instead of being clamped)."
---

# Properties
* `ON_ENTER`: String = "enter" (const)
* `ON_ROUND_END`: String = "round_end" (const)
* `ON_BOTH`: String = "both" (const)
* `TRIGGERS`: Array[String] = [ON_ENTER, ON_ROUND_END, ON_BOTH] (const)

# Methods

### `static _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary`
```logic
return ContentSchema._f(key, label, type, default, extra)
```

### `static trigger_label(on: String) -> String`
```logic
match on:
	ON_ENTER:
		return "on enter"
	ON_ROUND_END:
		return "at round end"
return "on enter and at round end"
```

### `static prepare(row: Dictionary) -> Dictionary`
```logic
var out: Dictionary = row
var id: String = str(out.get("id", ""))
if Terrain.has_id(id) and not Terrain.is_walkable(Terrain.from_id(id)):
	out["move_cost"] = 1
return out
```

