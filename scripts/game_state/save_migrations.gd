class_name SaveMigrations
extends RefCounted

# ==============================================================================
# Tactical RPG - Save migrations (S13): from_version -> from_version + 1 steps applied in order.
# Built-in steps: 1 -> 2 (M4-L) keeps each quests entry that passes QuestRules.validate_blob on its own (after
# JsonUtil.normalize), drops the others, and adds loot_claimed: []. 2 -> 3 (EXP1-L) adds position, explore, rolls
# defaults and a deterministic rng_seed (hash of the v2 state), and qualifies location_id. Tests may register() extra
# or replacement steps; reset() restores the built-in table.
# ==============================================================================

static var _steps: Dictionary = {}   # from_version (int) -> Callable(game_state: Dictionary) -> Dictionary
static var _installed: bool = false

static func _table() -> Dictionary:
	if not _installed:
		_installed = true
		_steps[1] = _v1_to_v2
		_steps[2] = _v2_to_v3
		_steps[3] = _v3_to_v4
		_steps[4] = _v4_to_v5
		_steps[5] = _v5_to_v6
		_steps[6] = _v6_to_v7
		_steps[7] = _v7_to_v8
		_steps[8] = _v8_to_v9
		_steps[9] = _v9_to_v10
	return _steps

static func register(from_version: int, step: Callable) -> void:
	_table()[from_version] = step

static func reset() -> void:
	_steps.clear()
	_installed = false
	_table()

# v1 -> v2: structured quests blob (invalid entries dropped) and an empty loot_claimed list.
static func _v1_to_v2(state: Dictionary) -> Dictionary:
	var out: Dictionary = state.duplicate(true)
	var kept: Dictionary = {}
	var quests: Dictionary = DictUtil.dict_of(JsonUtil.normalize(state.get("quests", {})))
	for key: Variant in quests:
		if QuestRules.blob_entry_errors(key, quests[key]).is_empty():
			kept[key] = quests[key]
	out["quests"] = kept
	out["loot_claimed"] = []
	return out

# v2 -> v3: exploration defaults; rng_seed from the v2 state (same file = same seed); location_id qualified with
# world_id when it resolves (a qualified id is kept when it resolves), else "". 3 -> 4 (DLG1-L) adds party.gold 0 and
# shops {}. 4 -> 5 (TRAV1-L) adds time 0. 5 -> 6 (TRAV2-L) adds party.supplies PartyState.SUPPLIES_START. 6 -> 7
# (AUTO1-L) adds autoplay GameState.default_autoplay() (off, Balanced, no per-hero tactics). 7 -> 8 (ITEMUSE1-L) adds
# pouch [] to every party member. 8 -> 9 (HEAL1-L) adds hp_lost 0 to every party member and explore.last_rest "".
static func _v2_to_v3(state: Dictionary) -> Dictionary:
	var out: Dictionary = state.duplicate(true)
	out["rng_seed"] = String(JSON.stringify(state, "", true)).hash() & 0x7FFFFFFF
	out["position"] = [GameState.NO_POSITION.x, GameState.NO_POSITION.y]
	out["explore"] = GameState.default_explore()
	out["rolls"] = {}
	var world_id: String = str(state.get("world_id", ""))
	var loc: String = str(state.get("location_id", ""))
	var hit: Dictionary = WorldCatalog.resolve(world_id, "locations", loc) if not world_id.is_empty() and not loc.is_empty() else {}
	out["location_id"] = str(hit.get("_qid", ""))
	return out

# v3 -> v4 (DLG1-L): party gold 0 (when the party is an object) and no shop sales yet.
static func _v3_to_v4(state: Dictionary) -> Dictionary:
	var out: Dictionary = state.duplicate(true)
	if out.get("party", null) is Dictionary:
		var party: Dictionary = out["party"]
		party["gold"] = 0
	out["shops"] = {}
	return out

# v4 -> v5 (TRAV1-L): the world clock starts at 0; nothing else changes.
static func _v4_to_v5(state: Dictionary) -> Dictionary:
	var out: Dictionary = state.duplicate(true)
	out["time"] = 0
	return out

# v5 -> v6 (TRAV2-L): party supplies start at PartyState.SUPPLIES_START (when the party is an object); nothing else.
static func _v5_to_v6(state: Dictionary) -> Dictionary:
	var out: Dictionary = state.duplicate(true)
	if out.get("party", null) is Dictionary:
		var party: Dictionary = out["party"]
		party["supplies"] = PartyState.SUPPLIES_START
	return out

# v6 -> v7 (AUTO1-L): the remembered battle autoplay starts off with the default party tactic; nothing else.
static func _v6_to_v7(state: Dictionary) -> Dictionary:
	var out: Dictionary = state.duplicate(true)
	out["autoplay"] = GameState.default_autoplay()
	return out

# v7 -> v8 (ITEMUSE1-L): every party member (when the party and the member are objects) gets an empty battle pouch;
# nothing else changes.
static func _v7_to_v8(state: Dictionary) -> Dictionary:
	var out: Dictionary = state.duplicate(true)
	if out.get("party", null) is Dictionary:
		var party: Dictionary = out["party"]
		for raw: Variant in DictUtil.array_of(party.get("members", [])):
			if raw is Dictionary:
				var member: Dictionary = raw
				member["pouch"] = []
	return out

# v8 -> v9 (HEAL1-L): every party member (when the party and the member are objects) gets hp_lost 0 (full HP) and
# explore (when it is an object) gets last_rest "" (the start location); nothing else changes. 9 -> 10 (PURSE1-L) adds
# purses {}.
static func _v8_to_v9(state: Dictionary) -> Dictionary:
	var out: Dictionary = state.duplicate(true)
	if out.get("party", null) is Dictionary:
		var party: Dictionary = out["party"]
		for raw: Variant in DictUtil.array_of(party.get("members", [])):
			if raw is Dictionary:
				var member: Dictionary = raw
				member["hp_lost"] = 0
	if out.get("explore", null) is Dictionary:
		var explore: Dictionary = out["explore"]
		explore["last_rest"] = ""
	return out

# v9 -> v10 (PURSE1-L): no purse encounter won yet (purses {}); nothing else changes. A story purse won before the
# bump pays once more if re-fought (spec rule 1.9, accepted).
static func _v9_to_v10(state: Dictionary) -> Dictionary:
	var out: Dictionary = state.duplicate(true)
	out["purses"] = {}
	return out

# Brings a save envelope to `target_version`. Returns
# {"ok": bool, "errors": Array[String], "warnings": Array[String], "data": Dictionary (envelope at the target)}.
# A newer version is refused; unknown keys inside game_state are dropped and listed as warnings, and so is every
# quests entry a step dropped ("dropped invalid quest entry '<key>'") and a location_id a step cleared ("cleared
# unresolved location_id '<id>'").
static func migrate(envelope: Dictionary, target_version: int = SaveGame.VERSION) -> Dictionary:
	var errors: Array[String] = []
	var warnings: Array[String] = []
	var data: Dictionary = envelope.duplicate(true)
	if not (data.get("version") is int or data.get("version") is float):
		return {"ok": false, "errors": ["save has no version"] as Array[String], "warnings": warnings, "data": data}
	var version: int = DictUtil.to_int(data["version"])
	if version > target_version:
		return {"ok": false, "errors": ["save is from a newer version"] as Array[String], "warnings": warnings, "data": data}
	if version < 1:
		return {"ok": false, "errors": ["save version %d is invalid" % version] as Array[String], "warnings": warnings, "data": data}
	var state: Dictionary = DictUtil.dict_of(data.get("game_state", {}))
	var steps: Dictionary = _table()
	while version < target_version:
		if not steps.has(version):
			errors.append("no migration from save version %d" % version)
			return {"ok": false, "errors": errors, "warnings": warnings, "data": data}
		var step: Callable = steps[version]
		var raw_quests: Variant = state.get("quests", {})
		if not (raw_quests is Dictionary):
			warnings.append("dropped invalid quests blob (not an object)")
		var before: Dictionary = DictUtil.dict_of(raw_quests)
		var loc_before: String = str(state.get("location_id", ""))
		state = DictUtil.dict_of(step.call(state))
		var after: Dictionary = DictUtil.dict_of(state.get("quests", {}))
		for key: Variant in before:
			if not after.has(key):
				warnings.append("dropped invalid quest entry '%s'" % str(key))
		if not loc_before.is_empty() and str(state.get("location_id", "")).is_empty():
			warnings.append("cleared unresolved location_id '%s'" % loc_before)
		version += 1
	for key: Variant in state.keys():
		if not GameState.KEYS.has(str(key)):
			warnings.append("dropped unknown game_state key '%s'" % str(key))
			state.erase(key)
	data["game_state"] = state
	data["version"] = version
	return {"ok": true, "errors": errors, "warnings": warnings, "data": data}
