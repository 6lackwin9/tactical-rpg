class_name EncounterObjectives
extends RefCounted

# ==============================================================================
# Tactical RPG - Encounter Objectives (OBJ1, spec docs/specs/OBJ1.md)
# What wins an encounter (rout, hold, escape, slay), whom the party must keep alive (guests,
# protect) and how both sides start (deployment). Fields on catalog and world encounters:
#   objective         enum    rout | hold | escape | slay                   (default rout)
#   objective_tiles   json    [[x, y], ...]; hold only; [] = the ring RING  (default [])
#   objective_rounds  int     1..max_rounds; hold only                      (default 3)
#   objective_unit    String  slay: an id in `enemies`                      (default "")
#   objective_count   int     escape: 1..4 heroes out by an exit            (default 1)
#   objective_text    String  0..80 chars; "" = the generated text (text_of) (default "")
#   deployment        enum    standard | ambush | centre                    (default standard)
#   guests            refs    party-side extras, at most MAX_GUESTS          (default [])
#   protect           refs    each also in guests                           (default [])
#
# Static part (OBJ1-L stage 1): the shared validator `problems` (both catalogs) and the
# generated objective text. The runtime part (held, verdict) follows in stage 2.
# ==============================================================================

const ROUT: String = "rout"
const HOLD: String = "hold"
const ESCAPE: String = "escape"
const SLAY: String = "slay"
const OBJECTIVES: Array[String] = [ROUT, HOLD, ESCAPE, SLAY]

const STANDARD: String = "standard"
const AMBUSH: String = "ambush"
const CENTRE: String = "centre"
const DEPLOYMENTS: Array[String] = [STANDARD, AMBUSH, CENTRE]

const RING: Array[Vector2i] = [Vector2i(5, 4), Vector2i(6, 4)]
const MAX_GUESTS: int = 2
const MAX_TEXT: int = 80
const COUNT_MIN: int = 1
const COUNT_MAX: int = 4
const DEFAULT_ROUNDS: int = 3

# ------------------------------------------------------------------------------
# Static helpers
# ------------------------------------------------------------------------------

# The objective tiles of `entry` (hold): the authored list, or the ring when it is empty or malformed.
static func tiles_of(entry: Dictionary) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	for raw: Variant in DictUtil.array_of(entry.get("objective_tiles", [])):
		var pair: Array = raw if raw is Array else []
		if pair.size() == 2:
			out.append(Vector2i(DictUtil.to_int(pair[0]), DictUtil.to_int(pair[1])))
	if out.is_empty():
		out.assign(RING)
	return out

# Rule 9: the objective line shown to the player. `objective_text` wins when set; else the generated
# text. `name_of(ref) -> String` turns a roster ref (objective_unit, protect) into a display name.
static func text_of(entry: Dictionary, name_of: Callable) -> String:
	var authored: String = str(entry.get("objective_text", "")).strip_edges()
	if not authored.is_empty():
		return authored
	var text: String = "Defeat every enemy."
	match str(entry.get("objective", ROUT)):
		HOLD:
			text = "Hold the ring for %d rounds, or defeat every enemy." % DictUtil.to_int(entry.get("objective_rounds", DEFAULT_ROUNDS))
		ESCAPE:
			text = "Get %d out by the exit, or defeat every enemy." % DictUtil.to_int(entry.get("objective_count", COUNT_MIN))
		SLAY:
			text = "Defeat %s." % str(name_of.call(str(entry.get("objective_unit", ""))))
	var names: Array[String] = []
	for ref: String in _strings(entry.get("protect", [])):
		names.append(str(name_of.call(ref)))
	if not names.is_empty():
		text += " Keep %s alive." % " and ".join(names)
	return text

# ------------------------------------------------------------------------------
# Validation (both catalogs; `entry` is normalised)
# ------------------------------------------------------------------------------

# Problems with the objective fields of `entry` (spec 1.1). `guest_ok(ref) -> bool` is false for a guest
# ref that resolves but does not fight on the party side (unknown refs are the generic ref check's job).
static func problems(entry: Dictionary, guest_ok: Callable) -> Array[String]:
	var errors: Array[String] = []
	var objective: String = str(entry.get("objective", ROUT))
	var max_rounds: int = DictUtil.to_int(entry.get("max_rounds", 0))
	var arena_open: bool = str(entry.get("arena", "CLOSED")) == "OPEN"
	var exits: Array[Vector2i] = _cells(entry.get("exits", []))
	if objective == HOLD:
		var raw_tiles: Variant = entry.get("objective_tiles", [])
		if not (raw_tiles is Array):
			errors.append("Objective tiles must be a list of [x, y] pairs.")
		else:
			var terrain: Dictionary = CombatZones.terrain_map(str(entry.get("zone", "")))
			for raw: Variant in DictUtil.array_of(raw_tiles):
				var pair: Array = raw if raw is Array else []
				if pair.size() != 2:
					errors.append("Objective tile %s is off the grid, blocked or an exit." % str(raw))
					continue
				var cell := Vector2i(DictUtil.to_int(pair[0]), DictUtil.to_int(pair[1]))
				if not _tile_ok(cell, terrain, arena_open, exits):
					errors.append("Objective tile %s is off the grid, blocked or an exit." % str(cell))
			if DictUtil.array_of(raw_tiles).is_empty():
				for cell: Vector2i in RING:
					if not _tile_ok(cell, terrain, arena_open, exits):
						errors.append("Objective tile %s is off the grid, blocked or an exit." % str(cell))
		var rounds: int = DictUtil.to_int(entry.get("objective_rounds", DEFAULT_ROUNDS))
		if rounds < 1 or rounds > max_rounds:
			errors.append("Hold rounds must be 1..%d." % max_rounds)
	elif objective == SLAY:
		var target: String = str(entry.get("objective_unit", ""))
		if target.is_empty() or not _strings(entry.get("enemies", [])).has(target):
			errors.append("Slay needs a unit from enemies.")
	elif objective == ESCAPE:
		if not arena_open and exits.is_empty():
			errors.append("Escape needs an exit tile.")   # objective_count 1..4 is clamped by the schema
	if str(entry.get("objective_text", "")).length() > MAX_TEXT:
		errors.append("Objective text holds at most %d characters." % MAX_TEXT)
	var guests: Array[String] = _strings(entry.get("guests", []))
	if guests.size() > MAX_GUESTS:
		errors.append("An encounter holds at most %d guests." % MAX_GUESTS)
	for ref: String in guests:
		if not DictUtil.to_bool(guest_ok.call(ref)):
			errors.append("Guest '%s' must fight on the party side." % ref)
	for ref: String in _strings(entry.get("protect", [])):
		if not guests.has(ref):
			errors.append("Protect '%s' is not a guest." % ref)
	return errors

# On the grid, walkable in the zone (missing tiles are floor) and not an exit.
static func _tile_ok(cell: Vector2i, terrain: Dictionary, arena_open: bool, exits: Array[Vector2i]) -> bool:
	if cell.x < 0 or cell.y < 0 or cell.x >= CombatGrid.GRID_W or cell.y >= CombatGrid.GRID_H:
		return false
	var raw: Variant = terrain.get(cell, Terrain.Type.FLOOR)
	var t: Terrain.Type = raw if raw is Terrain.Type else Terrain.Type.FLOOR
	if not Terrain.is_walkable(t):
		return false
	if arena_open:
		return cell.x > 0 and cell.y > 0 and cell.x < CombatGrid.GRID_W - 1 and cell.y < CombatGrid.GRID_H - 1
	return not exits.has(cell)

static func _cells(raw: Variant) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	for v: Variant in DictUtil.array_of(raw):
		var pair: Array = v if v is Array else []
		if pair.size() == 2:
			out.append(Vector2i(DictUtil.to_int(pair[0]), DictUtil.to_int(pair[1])))
	return out

static func _strings(raw: Variant) -> Array[String]:
	var out: Array[String] = []
	for v: Variant in DictUtil.array_of(raw):
		out.append(str(v))
	return out
