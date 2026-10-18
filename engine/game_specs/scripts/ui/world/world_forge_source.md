---
class: WorldForgeSource
extends: ForgeSource
description: "The World Forge's source: WorldSchema fields; references resolve from the world being edited (its own entries, its ancestors', or the base catalog)."
---

# Properties
* `world_id`: String = ""
* `out`: Array[Dictionary] = []
* `ref_kind`: String = str(f.get("ref", ""))
* `id`: String = str(e.get("id", ""))
* `value`: String = WorldCatalog.ref_value(world_id, e)

# Methods

### `_init(p_world_id: String = "") -> void`
```logic
world_id = p_world_id
```

### `normalize(kind: String, entry: Dictionary) -> Dictionary`
```logic
return WorldSchema.normalize(kind, entry)
```

### `ref_label(f: Dictionary, value: String) -> String`
```logic
if value.is_empty():
	return "(none)"
var ref_kind: String = str(f.get("ref", ""))
if ref_kind == "worlds":
	var m: Dictionary = WorldCatalog.manifest(value)
	return "%s  [%s]" % [str(m.get("name", "?")), value] if not m.is_empty() else "%s (missing)" % value
if ref_kind.begins_with("base:"):
	var base: Dictionary = ContentCatalog.get_entry(ref_kind.trim_prefix("base:"), value)
	return "%s  [%s]" % [str(base.get("name", "?")), value] if not base.is_empty() else "%s (missing)" % value
var hit: Dictionary = WorldCatalog.resolve(world_id, ref_kind, value)
if hit.is_empty():
	return "%s (missing)" % value
var origin: String = "" if str(hit["_world"]) == world_id else "  ↑ %s" % str(hit["_world"])
return "%s  [%s]%s" % [str(hit.get("name", "?")), str(hit.get("id", "")), origin]
```

