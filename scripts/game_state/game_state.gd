class_name GameState
extends RefCounted

# ==============================================================================
# Tactical RPG - Game State (S13): the session-wide snapshot that is saved and loaded.
# world, location, party, flags (bool / int / String, key [a-z0-9_.]) and an opaque quests blob
# (owned by S8: saved verbatim, must be JSON-safe; whole floats come back as ints).
# ==============================================================================

const KEYS: Array[String] = ["world_id", "location_id", "party", "flags", "quests"]

var world_id: String = ""
var location_id: String = ""
var party: PartyState = PartyState.new()
var flags: Dictionary = {}
var quests: Dictionary = {}

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
		"flags": flags.duplicate(true), "quests": quests.duplicate(true),
	}

static func from_dict(d: Dictionary) -> GameState:
	var g := GameState.new()
	g.world_id = str(d.get("world_id", ""))
	g.location_id = str(d.get("location_id", ""))
	g.party = PartyState.from_dict(DictUtil.dict_of(d.get("party", {})))
	var raw_flags: Dictionary = DictUtil.dict_of(JsonUtil.normalize(d.get("flags", {})))
	for key: Variant in raw_flags:
		g.flags[str(key)] = raw_flags[key]
	g.quests = DictUtil.dict_of(JsonUtil.normalize(d.get("quests", {})))
	return g

func equals(other: GameState) -> bool:
	return other != null and JsonUtil.deep_equal(to_dict(), other.to_dict())

func validate() -> Array[String]:
	var errors: Array[String] = []
	if world_id.is_empty():
		errors.append("world_id can't be empty.")
	errors.append_array(party.validate())
	for key: Variant in flags:
		if not (key is String) or not flag_key_ok(str(key)):
			errors.append("Bad flag key '%s'." % str(key))
		elif not flag_value_ok(flags[key]):
			errors.append("Flag '%s' must be a bool, int or String." % str(key))
	JsonUtil.check_safe(quests, "quests", errors)
	return errors
