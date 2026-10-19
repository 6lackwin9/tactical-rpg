class_name Terrain
extends RefCounted

# ==============================================================================
# Tactical RPG - Terrain Types
# Per-tile terrain for the 12x9 combat grid. Code owns the types, their ids, names,
# glyphs and walkability (walls, trees, deep water and lava block movement and
# deployment). The numbers (move cost, cover, hazard status) live in the catalog
# kind `terrain` (S12T-L; data/catalog/terrain.json, one row per type); a type
# without a row uses the code defaults below (the pre-S12T values).
# Zone maps use the single-character `glyph` of each type.
# New types are APPENDED (TileArt cache keys and saved maps use the ordinal).
# ==============================================================================

enum Type { FLOOR, GRASS, DIRT, STONE, SAND, SNOW, SHALLOW_WATER, DEEP_WATER, MUD, LAVA, WALL, TREE, BRIDGE, RUBBLE, BURNING_GROUND, TOXIC_BOG, ICE }

const INFO: Dictionary = {
	Type.FLOOR: {"name": "Arena Floor", "glyph": ".", "walkable": true},
	Type.GRASS: {"name": "Grass", "glyph": ",", "walkable": true},
	Type.DIRT: {"name": "Dirt Path", "glyph": ":", "walkable": true},
	Type.STONE: {"name": "Cobblestone", "glyph": "o", "walkable": true},
	Type.SAND: {"name": "Sand", "glyph": "s", "walkable": true},
	Type.SNOW: {"name": "Snow", "glyph": "*", "walkable": true},
	Type.SHALLOW_WATER: {"name": "Shallow Water", "glyph": "~", "walkable": true},
	Type.DEEP_WATER: {"name": "Deep Water", "glyph": "W", "walkable": false},
	Type.MUD: {"name": "Mud", "glyph": "m", "walkable": true},
	Type.LAVA: {"name": "Lava", "glyph": "L", "walkable": false},
	Type.WALL: {"name": "Rock Wall", "glyph": "#", "walkable": false},
	Type.TREE: {"name": "Tree", "glyph": "T", "walkable": false},
	Type.BRIDGE: {"name": "Bridge", "glyph": "=", "walkable": true},
	Type.RUBBLE: {"name": "Rubble", "glyph": "r", "walkable": true},
	Type.BURNING_GROUND: {"name": "Burning Ground", "glyph": "f", "walkable": true},
	Type.TOXIC_BOG: {"name": "Toxic Bog", "glyph": "p", "walkable": true},
	Type.ICE: {"name": "Ice", "glyph": "i", "walkable": true},
}

# Code defaults when a type has no catalog row (the old INFO move costs; no cover, no hazard).
const SLOW_TYPES: Array[Type] = [Type.SAND, Type.SNOW, Type.SHALLOW_WATER, Type.MUD, Type.RUBBLE]
const CATALOG_KIND: String = "terrain"

static func info(t: Type) -> Dictionary:
	var raw: Variant = INFO.get(t, INFO[Type.FLOOR])
	return raw if raw is Dictionary else {}

static func display_name(t: Type) -> String:
	return str(info(t).get("name", "Floor"))

static func is_walkable(t: Type) -> bool:
	var raw: Variant = info(t).get("walkable", true)
	return raw if raw is bool else true

static func glyph(t: Type) -> String:
	return str(info(t).get("glyph", "."))

static func from_glyph(ch: String) -> Type:
	for t: Type in INFO:
		if glyph(t) == ch:
			return t
	return Type.FLOOR

# ------------------------------------------------------------------------------
# Ids (the catalog key): the enum name in lower snake case
# ------------------------------------------------------------------------------

static func id_of(t: Type) -> String:
	var names: Array = Type.keys()
	var i: int = int(t)
	return str(names[i]).to_lower() if i >= 0 and i < names.size() else ""

static func has_id(id: String) -> bool:
	return ids().has(id)

# The type of `id` (FLOOR when unknown; check has_id first).
static func from_id(id: String) -> Type:
	for t: Type in INFO:
		if id_of(t) == id:
			return t
	return Type.FLOOR

static func ids() -> Array[String]:
	var out: Array[String] = []
	for t: Type in INFO:
		out.append(id_of(t))
	return out

# ------------------------------------------------------------------------------
# Numbers (catalog row, else code defaults)
# ------------------------------------------------------------------------------

static func default_move_cost(t: Type) -> int:
	return 2 if SLOW_TYPES.has(t) else 1

# The code-default row of `t` (normalised like a catalog row).
static func default_row(t: Type) -> Dictionary:
	return ContentSchema.normalize(CATALOG_KIND, {"id": id_of(t), "move_cost": default_move_cost(t)})

# Type -> row cache (path searches read move costs per cell). ContentCatalog clears it on every terrain change,
# reload and root switch (clear_cache).
static var _rows: Dictionary = {}

static func clear_cache() -> void:
	_rows.clear()

# The terrain row of `t`: its catalog entry, else the code defaults (do not modify the returned row).
static func row(t: Type) -> Dictionary:
	if _rows.has(t):
		var cached: Dictionary = _rows[t]
		return cached
	var entry: Dictionary = ContentCatalog.get_entry(CATALOG_KIND, id_of(t))
	var out: Dictionary = entry if not entry.is_empty() else default_row(t)
	_rows[t] = out
	return out

# Cost of entering a tile of `t` (1 for blocked types: they are never entered).
static func move_cost(t: Type) -> int:
	if not is_walkable(t):
		return 1
	return maxi(1, DictUtil.to_int(row(t).get("move_cost", default_move_cost(t)), default_move_cost(t)))

# Guard share a ranged hit loses on (or behind) this tile.
static func cover_guard(t: Type) -> float:
	return DictUtil.to_float(row(t).get("cover_guard", 0.0))

# True when `t` carries a hazard (cheap: no dictionary built).
static func has_hazard(t: Type) -> bool:
	return is_walkable(t) and not str(row(t).get("hazard_status", "")).is_empty()

# The hazard of `t`: {status, potency, magnitude, rounds, on}, or {} when the tile carries none.
static func hazard(t: Type) -> Dictionary:
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

static func describe(t: Type) -> String:
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
