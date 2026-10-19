class_name GameState
extends RefCounted

# ==============================================================================
# Tactical RPG - Game State (S13): the session-wide snapshot that is saved and loaded.
# world, location, party, flags (bool / int / String, key [a-z0-9_.]), the structured quests blob
# (S8: qualified quest id -> {state, objective}, checked by QuestRules.validate_blob) and loot_claimed
# (qualified world loot ids, unique, saved sorted). Whole floats come back as ints.
# S9 (EXP1-L, save v3): location_id is a QUALIFIED location id (world:id) or ""; position is the token tile on the
# location's map ([-1, -1] = no map); explore holds the step counter; rng_seed + rolls drive GameRolls.
# S11 (DLG1-L, save v4): party.gold (PartyState) and shops = {shop qid: {item id: sold count >= 1}} (limited stock sold).
# S10 (TRAV1-L, save v5): time = the world clock (int >= 0), advanced by travel legs.
# AUTO1-L (save v7): autoplay = {on: bool, party: tactic id, heroes: {party member id: tactic id or "custom"}}, the
# remembered battle-autoplay choice (PlanPresets.AUTO_IDS; off in a new game). CombatAutoplay reads and writes it.
# HEAL1-L (save v9): explore.last_rest = qualified id of the last rest place entered ("" = the start location; an id
# that no longer resolves is read as "" by the runtime, never a load error); party members store hp_lost (UnitState).
# PURSE1-L (save v10): purses = {qualified encounter id: wins >= 0} for encounters with a purse (PurseRules.pay).
# ==============================================================================

const KEYS: Array[String] = ["world_id", "location_id", "party", "flags", "quests", "loot_claimed", "position", "explore", "rng_seed", "rolls", "shops", "time", "autoplay", "purses"]
const AUTOPLAY_KEYS: Array[String] = ["on", "party", "heroes"]
const EXPLORE_KEYS: Array[String] = ["steps", "next_at", "loc_steps", "last_safe", "last_rest"]
const HOME_WORLD_KEY: String = "home_world"   # optional extra explore key: the world a `connects` crossing left (absent = none)
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
var shops: Dictionary = {}   # shop qid -> {item id: sold count >= 1}
var time: int = 0   # world clock (TRAV1-L)
var autoplay: Dictionary = default_autoplay()   # AUTO1-L remembered autoplay choice
var purses: Dictionary = {}   # PURSE1-L: encounter qid -> wins of that purse encounter (int >= 0)
var load_problems: Array[String] = []   # raw type problems found by from_dict (not saved; validate() reports them)

static func default_explore() -> Dictionary:
	return {"steps": 0, "next_at": 16, "loc_steps": 0, "last_safe": "", "last_rest": ""}

static func default_autoplay() -> Dictionary:
	return {"on": false, "party": PlanPresets.DEFAULT_TACTIC, "heroes": {}}

static func create(p_world_id: String, p_party: PartyState) -> GameState:
	var g := GameState.new()
	g.world_id = p_world_id
	g.party = p_party
	g.party.apply_cap(WorldCatalog.party_max(p_world_id))   # RECRUIT1-L 1.3
	return g

# Checked cast from a Variant (e.g. a result dict's "state"): the GameState, or null for anything else.
static func of(v: Variant) -> GameState:
	if v is GameState:
		return v
	return null

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
		"shops": shops.duplicate(true), "time": time, "autoplay": autoplay.duplicate(true),
		"purses": purses.duplicate(true),
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
	g.party.apply_cap(WorldCatalog.party_max(g.world_id))   # RECRUIT1-L 1.3: a longer active_ids keeps its first ids
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
		g.load_problems.append("explore must be {steps, next_at, loc_steps, last_safe, last_rest}.")
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
	var raw_shops: Variant = JsonUtil.normalize(d.get("shops", {}))
	if not (raw_shops is Dictionary):
		g.load_problems.append("shops must be an object {shop qid: {item id: sold count}}.")
	else:
		g.shops = raw_shops
	var raw_time: Variant = JsonUtil.normalize(d.get("time", 0))
	if not (raw_time is int):
		g.load_problems.append("time must be a whole number >= 0.")
	else:
		g.time = DictUtil.to_int(raw_time)
	var raw_auto: Variant = JsonUtil.normalize(d.get("autoplay", default_autoplay()))
	if not (raw_auto is Dictionary):
		g.load_problems.append("autoplay must be {on, party, heroes}.")
	else:
		g.autoplay = raw_auto
	var raw_purses: Variant = JsonUtil.normalize(d.get("purses", {}))
	if not (raw_purses is Dictionary):
		g.load_problems.append("purses must be an object {encounter qid: wins}.")
	else:
		g.purses = raw_purses
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
	errors.append_array(shops_errors(shops))
	if time < 0:
		errors.append("time must be a whole number >= 0.")
	errors.append_array(autoplay_errors(autoplay))
	errors.append_array(purses_errors(purses))
	return errors

# PURSE1-L: {qualified encounter id: whole number >= 0}.
static func purses_errors(blob: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	for key: Variant in blob:
		if not (key is String) or not QuestRules.blob_key_ok(str(key)) or not (blob[key] is int) or DictUtil.to_int(blob[key]) < 0:
			errors.append("purses: '%s' must be a qualified encounter id world:id with a win count >= 0." % str(key))
	return errors

# {on: bool, party: one of PlanPresets.AUTO_IDS, heroes: {non-empty key: AUTO_IDS id or "custom"}}, nothing else.
static func autoplay_errors(a: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var ok: bool = a.size() == AUTOPLAY_KEYS.size()
	for key: String in AUTOPLAY_KEYS:
		ok = ok and a.has(key)
	if not ok:
		errors.append("autoplay must be {on, party, heroes}.")
		return errors
	if not (a["on"] is bool):
		errors.append("autoplay.on must be true or false.")
	if not (a["party"] is String) or not PlanPresets.AUTO_IDS.has(str(a["party"])):
		errors.append("autoplay.party '%s' must be one of %s." % [str(a["party"]), ", ".join(PlanPresets.AUTO_IDS)])
	if not (a["heroes"] is Dictionary):
		errors.append("autoplay.heroes must be an object {hero id: tactic}.")
		return errors
	var heroes: Dictionary = a["heroes"]
	for key: Variant in heroes:
		if not (key is String) or str(key).is_empty() or not (heroes[key] is String) or not PlanPresets.is_tactic(str(heroes[key])):
			errors.append("autoplay.heroes: '%s' must map a hero id to a tactic (%s)." % [str(key), ", ".join(PlanPresets.tactic_ids())])
	return errors

static func shops_errors(blob: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	for key: Variant in blob:
		if not (key is String) or not QuestRules.blob_key_ok(str(key)):
			errors.append("shops: '%s' must be a qualified shop id world:id." % str(key))
		if not (blob[key] is Dictionary):
			errors.append("shops.%s must be an object {item id: sold count}." % str(key))
			continue
		var sold: Dictionary = blob[key]
		for item: Variant in sold:
			if not (item is String) or not JsonUtil.id_ok(str(item)) or not (sold[item] is int) or DictUtil.to_int(sold[item]) < 1:
				errors.append("shops.%s: '%s' must be an item id with a sold count >= 1." % [str(key), str(item)])
	return errors

static func explore_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var has_home: bool = e.has(HOME_WORLD_KEY)
	var ok: bool = e.size() == EXPLORE_KEYS.size() + (1 if has_home else 0)
	ok = ok and (not has_home or (e[HOME_WORLD_KEY] is String and JsonUtil.id_ok(str(e[HOME_WORLD_KEY]))))
	for key: String in EXPLORE_KEYS:
		ok = ok and e.has(key)
	if ok:
		ok = e["steps"] is int and DictUtil.to_int(e["steps"]) >= 0 and e["loc_steps"] is int and DictUtil.to_int(e["loc_steps"]) >= 0
		ok = ok and e["next_at"] is int and DictUtil.to_int(e["next_at"]) >= 1 and DictUtil.to_int(e["next_at"]) <= 99
		ok = ok and e["last_safe"] is String and (str(e["last_safe"]).is_empty() or QuestRules.blob_key_ok(str(e["last_safe"])))
		ok = ok and e["last_rest"] is String and (str(e["last_rest"]).is_empty() or QuestRules.blob_key_ok(str(e["last_rest"])))
	if not ok:
		errors.append("explore must be {steps, next_at, loc_steps, last_safe, last_rest}.")
	return errors
