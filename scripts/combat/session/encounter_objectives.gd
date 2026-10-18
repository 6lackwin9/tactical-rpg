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
#   guests            refs    party-side extras, at most MAX_GUESTS, no repeats (default [])
#   protect           refs    each also in guests                           (default [])
#
# Static part: the shared validator `problems` (both catalogs) and the generated objective text.
# Runtime part (one instance per battle, built by CatalogEncounter.build()): the parsed objective,
# `held` (hold rounds counted, kept in every BlockSnapshot) and `verdict` (spec section 2, rules 3 to 7).
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

const WON: String = "won"
const LOST: String = "lost"
const FLED: String = "fled"

var objective: String = ROUT
var tiles: Array[Vector2i] = []
var rounds: int = DEFAULT_ROUNDS
var unit_ref: String = ""           # slay target roster ref
var count: int = COUNT_MIN          # escape: heroes that must leave
var held: int = 0                   # hold: rounds counted so far (rule 5; snapshotted)
var note: String = ""               # set with a verdict that needs a reason ("%s got away."); "" otherwise

# The runtime objective of a normalised encounter entry (`held` starts at 0).
static func create(entry: Dictionary) -> EncounterObjectives:
	var o := EncounterObjectives.new()
	var raw: String = str(entry.get("objective", ROUT))
	o.objective = raw if OBJECTIVES.has(raw) else ROUT
	o.tiles = tiles_of(entry)
	o.rounds = maxi(1, DictUtil.to_int(entry.get("objective_rounds", DEFAULT_ROUNDS), DEFAULT_ROUNDS))
	o.unit_ref = str(entry.get("objective_unit", ""))
	o.count = clampi(DictUtil.to_int(entry.get("objective_count", COUNT_MIN), COUNT_MIN), COUNT_MIN, COUNT_MAX)
	return o

func state() -> Dictionary:
	return {"held": held}

func restore(saved: Dictionary) -> void:
	held = maxi(0, DictUtil.to_int(saved.get("held", 0)))
	note = ""

# Rule 3: "", won, lost or fled after a block (first match wins). `heroes` are the party units, `guests` the
# party-side extras, `protect` the guests that must live, `foes` every enemy that is not reserved; `target` is the
# slay unit (null when none). Hold counting (rule 5) happens here, at a round end only.
func verdict(heroes: Array[Combatant], guests: Array[Combatant], protect: Array[Combatant], foes: Array[Combatant],
		grid: CombatGrid, end_of_round: bool, target: Combatant) -> String:
	note = ""
	# a. every hero down, or a protected guest down: lost
	var heroes_down: bool = true
	var hero_active: bool = false
	var withdrew: int = 0
	for h: Combatant in heroes:
		heroes_down = heroes_down and not h.is_alive()
		hero_active = hero_active or h.is_active()
		if h.is_alive() and h.is_withdrawn:
			withdrew += 1
	for g: Combatant in protect:
		if not g.is_alive():
			return LOST
	if heroes_down:
		return LOST
	# b. every enemy that counts is down or withdrawn: won
	var foe_active: bool = false
	for e: Combatant in foes:
		foe_active = foe_active or e.is_active()
	if not foe_active:
		return WON
	# c. the objective's own rule
	match objective:
		HOLD:
			if end_of_round:
				_count_hold(heroes + guests, foes, grid)
				if held >= rounds:
					return WON
		ESCAPE:
			var protect_out: bool = true
			for g: Combatant in protect:
				protect_out = protect_out and g.is_withdrawn
			if withdrew >= count and not hero_active and protect_out:
				return WON
		SLAY:
			if target != null:
				if not target.is_alive():
					return WON
				if target.is_withdrawn:
					note = "%s got away." % target.name
					return FLED
	# d. no hero on the field and at least one withdrew: fled
	if not hero_active and withdrew > 0:
		return FLED
	return ""

# Rule 5: party side alone on the tiles +1, enemies alone -1 (floor 0), nobody or both: unchanged.
func _count_hold(party_side: Array[Combatant], foes: Array[Combatant], grid: CombatGrid) -> void:
	var ours: bool = _on_tiles(party_side, grid)
	var theirs: bool = _on_tiles(foes, grid)
	if ours and not theirs:
		held += 1
	elif theirs and not ours:
		held = maxi(0, held - 1)

func _on_tiles(units: Array[Combatant], grid: CombatGrid) -> bool:
	for c: Combatant in units:
		if c.is_active() and grid.has_combatant(c) and tiles.has(grid.get_position(c)):
			return true
	return false

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
	var seen: Array[String] = []
	for ref: String in guests:
		if seen.has(ref):
			errors.append("Guest '%s' is listed twice." % ref)
			continue
		seen.append(ref)
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
