class_name GameState
extends RefCounted

# ==============================================================================
# Tactical RPG - Game State (S13): the session-wide snapshot that is saved and loaded.
# world, location, party, flags (bool / int / String, key [a-z0-9_.]), the structured quests blob
# (S8: qualified quest id -> {state, objective}, checked by QuestRules.validate_blob) and loot_claimed
# (qualified world loot ids, unique, saved sorted). Whole floats come back as ints.
# S9 (EXP1-L, save v3): location_id is a QUALIFIED location id (world:id) or ""; position is the token tile on the
# location's map ([-1, -1] = no map); explore holds the step counter; rng_seed + rolls drive GameRolls.
# ==============================================================================

const KEYS: Array[String] = ["world_id", "location_id", "party", "flags", "quests", "loot_claimed", "position", "explore", "rng_seed", "rolls"]
const EXPLORE_KEYS: Array[String] = ["steps", "next_at", "loc_steps", "last_safe"]
const NO_POSITION: Vector2i = Vector2i(-1, -1)
const SEED_MAX: int = 2147483647

var world_id: String = ""
var location_id: String = ""
var party: PartyState = PartyState.new()
var flags: Dictionary = {}
var quests: Dictionary = {}
var loot_claimed: Array[String] = []
var position: Vector2i = NO_POSITION
var explore: Dictionary = default_explore()
var rng_seed: int = 0
var rolls: Dictionary = {}   # stream id -> rolls drawn (int >= 0)
var load_problems: Array[String] = []   # raw type problems found by from_dict (not saved; validate() reports them)

static func default_explore() -> Dictionary:
	return {"steps": 0, "next_at": 16, "loc_steps": 0, "last_safe": ""}

static func create(p_world_id: String, p_party: PartyState) -> GameState:
	var g := GameState.new()
	g.world_id = p_world_id
	g.party = p_party
	return g

# ------------------------------------------------------------------------------
# Flags
# ------------------------------------------------------------------------------

static func flag_key_ok(key: String) -> bool:
	if key.is_empty():
		return false
	for i: int in range(key.length()):
		var c: String = key[i]
		if not ((c >= "a" and c <= "z") or (c >= "0" and c <= "9") or c == "_" or c == "."):
			return false
	return true

static func flag_value_ok(v: Variant) -> bool:
	return v is bool or v is int or v is String

# Returns an error text ("" = set).
func set_flag(key: String, value: Variant) -> String:
	if not flag_key_ok(key):
		return "Flag key '%s' must be non-empty [a-z0-9_.]." % key
	if not flag_value_ok(value):
		return "Flag '%s' must be a bool, int or String." % key
	flags[key] = value
	return ""

func get_flag(key: String, fallback: Variant = false) -> Variant:
	return flags.get(key, fallback)

func has_flag(key: String) -> bool:
	return flags.has(key)

func clear_flag(key: String) -> void:
	flags.erase(key)

# ------------------------------------------------------------------------------
# Serialisation
# ------------------------------------------------------------------------------

func to_dict() -> Dictionary:
	return {
		"world_id": world_id, "location_id": location_id, "party": party.to_dict(),
		"flags": flags.duplicate(true), "quests": quests.duplicate(true), "loot_claimed": _sorted(loot_claimed),
		"position": [position.x, position.y], "explore": explore.duplicate(true), "rng_seed": rng_seed, "rolls": rolls.duplicate(true),
	}

static func _sorted(list: Array[String]) -> Array[String]:
	var out: Array[String] = list.duplicate()
	out.sort()
	return out

static func from_dict(d: Dictionary) -> GameState:
	var g := GameState.new()
	g.world_id = str(d.get("world_id", ""))
	g.location_id = str(d.get("location_id", ""))
	g.party = PartyState.from_dict(DictUtil.dict_of(d.get("party", {})))
	var raw_flags: Dictionary = DictUtil.dict_of(JsonUtil.normalize(d.get("flags", {})))
	for key: Variant in raw_flags:
		g.flags[str(key)] = raw_flags[key]
	# Damaged blobs are never coerced silently: the raw type problems are kept and validate() reports them, so
	# SaveGame.read refuses the save (claimed loot can't become claimable again without anyone noticing).
	var raw_quests: Variant = d.get("quests", {})
	if not (raw_quests is Dictionary):
		g.load_problems.append("quests must be an object (the save has %s)." % type_string(typeof(raw_quests)))
	g.quests = DictUtil.dict_of(JsonUtil.normalize(raw_quests))
	var raw_loot: Variant = d.get("loot_claimed", [])
	if not (raw_loot is Array):
		g.load_problems.append("loot_claimed must be a list (the save has %s)." % type_string(typeof(raw_loot)))
	else:
		for x: Variant in DictUtil.array_of(raw_loot):
			if not (x is String):
				g.load_problems.append("loot_claimed: '%s' is not a String." % str(x))
	g.loot_claimed = JsonUtil.strings(raw_loot)
	var raw_pos: Variant = JsonUtil.normalize(d.get("position", [NO_POSITION.x, NO_POSITION.y]))
	var pos: Array = DictUtil.array_of(raw_pos)
	if pos.size() != 2 or not (pos[0] is int) or not (pos[1] is int):
		g.load_problems.append("position must be [x, y].")
	else:
		g.position = Vector2i(DictUtil.to_int(pos[0]), DictUtil.to_int(pos[1]))
	var raw_explore: Variant = JsonUtil.normalize(d.get("explore", default_explore()))
	if not (raw_explore is Dictionary):
		g.load_problems.append("explore must be {steps, next_at, loc_steps, last_safe}.")
	else:
		g.explore = raw_explore
	var raw_seed: Variant = JsonUtil.normalize(d.get("rng_seed", 0))
	if not (raw_seed is int):
		g.load_problems.append("rng_seed must be a whole number.")
	else:
		g.rng_seed = DictUtil.to_int(raw_seed)
	var raw_rolls: Variant = JsonUtil.normalize(d.get("rolls", {}))
	if not (raw_rolls is Dictionary):
		g.load_problems.append("rolls must be an object {stream: count}.")
	else:
		g.rolls = raw_rolls
	return g

func equals(other: GameState) -> bool:
	return other != null and JsonUtil.deep_equal(to_dict(), other.to_dict())

func validate() -> Array[String]:
	var errors: Array[String] = load_problems.duplicate()
	if world_id.is_empty():
		errors.append("world_id can't be empty.")
	errors.append_array(party.validate())
	for key: Variant in flags:
		if not (key is String) or not flag_key_ok(str(key)):
			errors.append("Bad flag key '%s'." % str(key))
		elif not flag_value_ok(flags[key]):
			errors.append("Flag '%s' must be a bool, int or String." % str(key))
	errors.append_array(QuestRules.validate_blob(quests))
	var seen: Dictionary = {}
	for id: String in loot_claimed:
		if not QuestRules.blob_key_ok(id):
			errors.append("loot_claimed: '%s' must be a qualified id world:id." % id)
		elif seen.has(id):
			errors.append("loot_claimed lists '%s' twice." % id)
		seen[id] = true
	if not location_id.is_empty() and not QuestRules.blob_key_ok(location_id):
		errors.append("location_id '%s' must be a qualified id world:id (or empty)." % location_id)
	if position.x < -1 or position.y < -1:
		errors.append("position must be [x, y].")
	errors.append_array(explore_errors(explore))
	if rng_seed < 0 or rng_seed > SEED_MAX:
		errors.append("rng_seed must be 0..%d." % SEED_MAX)
	for key: Variant in rolls:
		if not (key is String) or not JsonUtil.id_ok(str(key)) or not (rolls[key] is int) or DictUtil.to_int(rolls[key]) < 0:
			errors.append("rolls: '%s' must be a stream id with a count >= 0." % str(key))
	return errors

static func explore_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var ok: bool = e.size() == EXPLORE_KEYS.size()
	for key: String in EXPLORE_KEYS:
		ok = ok and e.has(key)
	if ok:
		ok = e["steps"] is int and DictUtil.to_int(e["steps"]) >= 0 and e["loc_steps"] is int and DictUtil.to_int(e["loc_steps"]) >= 0
		ok = ok and e["next_at"] is int and DictUtil.to_int(e["next_at"]) >= 1 and DictUtil.to_int(e["next_at"]) <= 99
		ok = ok and e["last_safe"] is String and (str(e["last_safe"]).is_empty() or QuestRules.blob_key_ok(str(e["last_safe"])))
	if not ok:
		errors.append("explore must be {steps, next_at, loc_steps, last_safe}.")
	return errors
