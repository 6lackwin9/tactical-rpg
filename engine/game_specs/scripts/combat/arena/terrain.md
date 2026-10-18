---
class: Terrain
extends: RefCounted
description: "Tactical RPG - Terrain Types Per-tile terrain for the 12x9 combat grid. Code owns the types, their ids, names, glyphs and walkability (walls, trees, deep water and lava block movement and deployment). The numbers (move cost, cover, hazard status) live in the catalog kind `terrain` (S12T-L; data/catalog/terrain.json, one row per type); a type without a row uses the code defaults below (the pre-S12T values). Zone maps use the single-character `glyph` of each type. New types are APPENDED (TileArt cache keys and saved maps use the ordinal)."
---

# Properties
* `INFO`: Dictionary = { (const)
* `SLOW_TYPES`: Array[Type] = [Type.SAND, Type.SNOW, Type.SHALLOW_WATER, Type.MUD, Type.RUBBLE] (const)
* `CATALOG_KIND`: String = "terrain" (const)
* `out`: Array[String] = []

# Methods

### `static info(t: Type) -> Dictionary`
```logic
var raw: Variant = INFO.get(t, INFO[Type.FLOOR])
return raw if raw is Dictionary else {}
```

### `static display_name(t: Type) -> String`
```logic
return str(info(t).get("name", "Floor"))
```

### `static is_walkable(t: Type) -> bool`
```logic
var raw: Variant = info(t).get("walkable", true)
return raw if raw is bool else true
```

### `static glyph(t: Type) -> String`
```logic
return str(info(t).get("glyph", "."))
```

### `static from_glyph(ch: String) -> Type`
```logic
for t: Type in INFO:
	if glyph(t) == ch:
		return t
return Type.FLOOR
```

### `static id_of(t: Type) -> String`
```logic
var names: Array = Type.keys()
var i: int = int(t)
return str(names[i]).to_lower() if i >= 0 and i < names.size() else ""
```

### `static has_id(id: String) -> bool`
```logic
return ids().has(id)
```

### `static from_id(id: String) -> Type`
```logic
for t: Type in INFO:
	if id_of(t) == id:
		return t
return Type.FLOOR
```

### `static default_move_cost(t: Type) -> int`
```logic
return 2 if SLOW_TYPES.has(t) else 1
```

### `static default_row(t: Type) -> Dictionary`
```logic
return ContentSchema.normalize(CATALOG_KIND, {"id": id_of(t), "move_cost": default_move_cost(t)})
```

### `static clear_cache() -> void`
```logic
_rows.clear()
```

### `static row(t: Type) -> Dictionary`
```logic
if _rows.has(t):
	var cached: Dictionary = _rows[t]
	return cached
var entry: Dictionary = ContentCatalog.get_entry(CATALOG_KIND, id_of(t))
var out: Dictionary = entry if not entry.is_empty() else default_row(t)
_rows[t] = out
return out
```

### `static move_cost(t: Type) -> int`
```logic
if not is_walkable(t):
	return 1
return maxi(1, DictUtil.to_int(row(t).get("move_cost", default_move_cost(t)), default_move_cost(t)))
```

### `static cover_guard(t: Type) -> float`
```logic
return DictUtil.to_float(row(t).get("cover_guard", 0.0))
```

### `static has_hazard(t: Type) -> bool`
```logic
return is_walkable(t) and not str(row(t).get("hazard_status", "")).is_empty()
```

### `static hazard(t: Type) -> Dictionary`
```logic
var r: Dictionary = row(t)
var status: String = str(r.get("hazard_status", ""))
if status.is_empty() or not is_walkable(t):
	return {}
return {
	"status": status,
	"potency": DictUtil.to_int(r.get("hazard_potency", 100), 100),
	"magnitude": DictUtil.to_int(r.get("hazard_magnitude", 0)),
	"rounds": DictUtil.to_int(r.get("hazard_rounds", 1), 1),
	"on": str(r.get("hazard_on", TerrainFields.ON_BOTH)),
}
```

### `static describe(t: Type) -> String`
```logic
var parts: Array[String] = [display_name(t)]
parts.append("walkable" if is_walkable(t) else "blocked")
if is_walkable(t) and move_cost(t) > 1:
	parts.append("move cost %d" % move_cost(t))
var cover: float = cover_guard(t)
if cover > 0.0:
	parts.append("cover %d%%" % roundi(cover * 100.0))
var h: Dictionary = hazard(t)
if not h.is_empty():
	parts.append("%s (%s)" % [StatusEffects.display_name(str(h["status"])), TerrainFields.trigger_label(str(h["on"]))])
return " • ".join(parts)
```

