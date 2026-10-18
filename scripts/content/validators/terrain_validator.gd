class_name TerrainValidator
extends RefCounted

# Validation of the catalog kind `terrain` (S12T-L). entry_errors checks one normalised row (exact messages from
# docs/specs/S12T_L.md); missing_ids lists the Terrain types without a row (validate_all: terrain:missing once
# terrain.json exists, warnings_all before that, when the code defaults apply).

const MOVE_COST_MIN: int = 1
const MOVE_COST_MAX: int = 3
const COVER_MAX: float = 0.5
const POTENCY_MIN: int = 1
const POTENCY_MAX: int = 100
const ROUNDS_MIN: int = 1
const ROUNDS_MAX: int = 5

static func entry_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var id: String = str(e.get("id", ""))
	if not Terrain.has_id(id):
		errors.append("Unknown terrain type '%s'." % id)
		return errors
	var walkable: bool = Terrain.is_walkable(Terrain.from_id(id))
	var cost: int = DictUtil.to_int(e.get("move_cost", 1), 1)
	if walkable and (cost < MOVE_COST_MIN or cost > MOVE_COST_MAX):
		errors.append("Move cost must be 1-3.")
	var cover: float = DictUtil.to_float(e.get("cover_guard", 0.0))
	if cover < 0.0 or cover > COVER_MAX:
		errors.append("Cover must be 0.0-0.5.")
	var status: String = str(e.get("hazard_status", ""))
	if not status.is_empty():
		if not StatusEffects.debuffs().has(status):
			errors.append("Hazard status '%s' is not a debuff." % status)
		if not walkable:
			errors.append("Only walkable terrain can carry a hazard.")
	var potency: int = DictUtil.to_int(e.get("hazard_potency", 100), 100)
	if potency < POTENCY_MIN or potency > POTENCY_MAX:
		errors.append("Hazard potency must be 1-100.")
	var magnitude: int = DictUtil.to_int(e.get("hazard_magnitude", 0))
	if StatusEffects.has_magnitude(status):
		var mag_min: int = StatusRegistry.magnitude_min(status)
		var mag_max: int = StatusRegistry.magnitude_max(status)
		if magnitude < mag_min or magnitude > mag_max:
			errors.append("%s needs a magnitude %d-%d" % [status, mag_min, mag_max])
	elif magnitude != 0:
		errors.append("magnitude only applies to %s" % ", ".join(StatusRegistry.magnitude_ids()))
	var rounds: int = DictUtil.to_int(e.get("hazard_rounds", 1), 1)
	if rounds < ROUNDS_MIN or rounds > ROUNDS_MAX:
		errors.append("Hazard rounds must be 1-5.")
	if not TerrainFields.TRIGGERS.has(str(e.get("hazard_on", ""))):
		errors.append("Hazard trigger must be enter, round_end or both.")
	if str(e.get("origin", ContentSchema.ORIGIN_SYSTEM)) != ContentSchema.ORIGIN_SYSTEM:
		errors.append("Terrain rows are system assets (origin system).")
	return errors

# Terrain type ids without a row in `rows` (Terrain order).
static func missing_ids(rows: Array[Dictionary]) -> Array[String]:
	var have: Dictionary = {}
	for r: Dictionary in rows:
		have[str(r.get("id", ""))] = true
	var out: Array[String] = []
	for id: String in Terrain.ids():
		if not have.has(id):
			out.append(id)
	return out
