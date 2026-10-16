class_name SaveMigrations
extends RefCounted

# ==============================================================================
# Tactical RPG - Save migrations (S13): from_version -> from_version + 1 steps applied in order.
# Built-in steps: 1 -> 2 (M4-L) keeps each quests entry that passes QuestRules.validate_blob on its own (after
# JsonUtil.normalize), drops the others, and adds loot_claimed: []. Tests may register() extra or replacement
# steps; reset() restores the built-in table.
# ==============================================================================

static var _steps: Dictionary = {}   # from_version (int) -> Callable(game_state: Dictionary) -> Dictionary
static var _installed: bool = false

static func _table() -> Dictionary:
	if not _installed:
		_installed = true
		_steps[1] = _v1_to_v2
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

# Brings a save envelope to `target_version`. Returns
# {"ok": bool, "errors": Array[String], "warnings": Array[String], "data": Dictionary (envelope at the target)}.
# A newer version is refused; unknown keys inside game_state are dropped and listed as warnings, and so is every
# quests entry a step dropped ("dropped invalid quest entry '<key>'").
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
		state = DictUtil.dict_of(step.call(state))
		var after: Dictionary = DictUtil.dict_of(state.get("quests", {}))
		for key: Variant in before:
			if not after.has(key):
				warnings.append("dropped invalid quest entry '%s'" % str(key))
		version += 1
	for key: Variant in state.keys():
		if not GameState.KEYS.has(str(key)):
			warnings.append("dropped unknown game_state key '%s'" % str(key))
			state.erase(key)
	data["game_state"] = state
	data["version"] = version
	return {"ok": true, "errors": errors, "warnings": warnings, "data": data}
