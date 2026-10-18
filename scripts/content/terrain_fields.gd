class_name TerrainFields
extends RefCounted

# The terrain field table of ContentSchema (S12T-L; ContentSchema.fields("terrain") / field() are the API).
# One row per Terrain.Type: numbers only (types, walkability and glyphs stay in Terrain). Ranges are checked by
# TerrainValidator (no min / max here, so an out-of-range value reaches the validator instead of being clamped).

const ON_ENTER: String = "enter"
const ON_ROUND_END: String = "round_end"
const ON_BOTH: String = "both"
const TRIGGERS: Array[String] = [ON_ENTER, ON_ROUND_END, ON_BOTH]

static func _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary:
	return ContentSchema._f(key, label, type, default, extra)

static func build() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", "", {"key_field": true, "help": "A terrain type id (Terrain.ids(): floor, grass, ... burning_ground, toxic_bog, ice)."}),
		_f("move_cost", "Move cost", "int", 1, {"help": "Cost of entering a tile (walkable types 1-3; blocked types ignore it)."}),
		_f("cover_guard", "Cover", "float", 0.0, {"step": 0.05, "help": "Guard share (0.0-0.5) a ranged hit loses on this tile, or behind it when the tile blocks movement."}),
		_f("hazard_status", "Hazard status", "enum", "", {"options_from": "status", "help": "A debuff rolled on units entering or standing on the tile (walkable types only)."}),
		_f("hazard_potency", "Hazard potency", "int", 100, {"help": "Inflict chance 1-100 (resistance applies)."}),
		_f("hazard_magnitude", "Hazard magnitude", "int", 0, {"help": "Inside the status's registry range (0 when it has none)."}),
		_f("hazard_rounds", "Hazard rounds", "int", 1, {"help": "Status duration 1-5 rounds."}),
		_f("hazard_on", "Hazard trigger", "enum", ON_BOTH, {"options": TRIGGERS, "help": "enter: when a move enters the tile; round_end: standing on it at round end; both."}),
	]

# Display text of a trigger (Terrain.describe, Forge help).
static func trigger_label(on: String) -> String:
	match on:
		ON_ENTER:
			return "on enter"
		ON_ROUND_END:
			return "at round end"
	return "on enter and at round end"

# The stored form of a normalised row: blocked types keep move_cost 1 (it is ignored).
static func prepare(row: Dictionary) -> Dictionary:
	var out: Dictionary = row
	var id: String = str(out.get("id", ""))
	if Terrain.has_id(id) and not Terrain.is_walkable(Terrain.from_id(id)):
		out["move_cost"] = 1
	return out
