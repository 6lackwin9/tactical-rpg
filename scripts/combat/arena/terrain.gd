class_name Terrain
extends RefCounted

# ==============================================================================
# Tactical RPG - Terrain Types
# Per-tile terrain for the 12x9 combat grid. Phase 1 rules: walkability (walls,
# trees, deep water and lava block movement and deployment). Cover, movement
# cost and hazards are recorded as data for the upcoming terrain rules
# (System 6 "Environmental Hazards"); they do not affect resolution yet.
# Zone maps use the single-character `glyph` of each type.
# ==============================================================================

enum Type { FLOOR, GRASS, DIRT, STONE, SAND, SNOW, SHALLOW_WATER, DEEP_WATER, MUD, LAVA, WALL, TREE, BRIDGE, RUBBLE }

const INFO: Dictionary = {
	Type.FLOOR: {"name": "Arena Floor", "glyph": ".", "walkable": true, "cover": 0, "move_cost": 1, "hazard": ""},
	Type.GRASS: {"name": "Grass", "glyph": ",", "walkable": true, "cover": 0, "move_cost": 1, "hazard": ""},
	Type.DIRT: {"name": "Dirt Path", "glyph": ":", "walkable": true, "cover": 0, "move_cost": 1, "hazard": ""},
	Type.STONE: {"name": "Cobblestone", "glyph": "o", "walkable": true, "cover": 0, "move_cost": 1, "hazard": ""},
	Type.SAND: {"name": "Sand", "glyph": "s", "walkable": true, "cover": 0, "move_cost": 2, "hazard": ""},
	Type.SNOW: {"name": "Snow", "glyph": "*", "walkable": true, "cover": 0, "move_cost": 2, "hazard": ""},
	Type.SHALLOW_WATER: {"name": "Shallow Water", "glyph": "~", "walkable": true, "cover": 0, "move_cost": 2, "hazard": ""},
	Type.DEEP_WATER: {"name": "Deep Water", "glyph": "W", "walkable": false, "cover": 0, "move_cost": 0, "hazard": "Impassable"},
	Type.MUD: {"name": "Mud", "glyph": "m", "walkable": true, "cover": 0, "move_cost": 2, "hazard": "Slows movement"},
	Type.LAVA: {"name": "Lava", "glyph": "L", "walkable": false, "cover": 0, "move_cost": 0, "hazard": "Burns (impassable)"},
	Type.WALL: {"name": "Rock Wall", "glyph": "#", "walkable": false, "cover": 2, "move_cost": 0, "hazard": "Impassable"},
	Type.TREE: {"name": "Tree", "glyph": "T", "walkable": false, "cover": 1, "move_cost": 0, "hazard": "Impassable"},
	Type.BRIDGE: {"name": "Bridge", "glyph": "=", "walkable": true, "cover": 0, "move_cost": 1, "hazard": ""},
	Type.RUBBLE: {"name": "Rubble", "glyph": "r", "walkable": true, "cover": 1, "move_cost": 2, "hazard": "Half cover"},
}

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

static func describe(t: Type) -> String:
	var d: Dictionary = info(t)
	var parts: Array[String] = [display_name(t)]
	parts.append("walkable" if is_walkable(t) else "blocked")
	var cover: Variant = d.get("cover", 0)
	if cover is int and cover > 0:
		parts.append("cover %d" % cover)
	var hazard: String = str(d.get("hazard", ""))
	if not hazard.is_empty():
		parts.append(hazard)
	return " • ".join(parts)
