class_name GameState
extends RefCounted

# ==============================================================================
# Tactical RPG - Game State (S13): the session-wide snapshot that is saved and loaded.
# world, location, party, flags (bool / int / String, key [a-z0-9_.]), the structured quests blob
# (S8: qualified quest id -> {state, objective}, checked by QuestRules.validate_blob) and loot_claimed
# (qualified world loot ids, unique, saved sorted). Whole floats come back as ints.
# ==============================================================================

const KEYS: Array[String] = ["world_id", "location_id", "party", "flags", "quests", "loot_claimed"]

var world_id: String = ""
var location_id: String = ""
var party: PartyState = PartyState.new()
var flags: Dictionary = {}
var quests: Dictionary = {}
var loot_claimed: Array[String] = []
var load_problems: Array[String] = []   # raw type problems found by from_dict (not saved; validate() reports them)

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
	return errors
