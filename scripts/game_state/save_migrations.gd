class_name SaveMigrations
extends RefCounted

# ==============================================================================
# Tactical RPG - Save migrations (S13): from_version -> from_version + 1 steps applied in order.
# The table is empty at version 1. Tests register a step with register() and clear it with reset().
# ==============================================================================

static var _steps: Dictionary = {}   # from_version (int) -> Callable(game_state: Dictionary) -> Dictionary

static func register(from_version: int, step: Callable) -> void:
	_steps[from_version] = step

static func reset() -> void:
	_steps.clear()

# Brings a save envelope to `target_version`. Returns
# {"ok": bool, "errors": Array[String], "warnings": Array[String], "data": Dictionary (envelope at the target)}.
# A newer version is refused; unknown keys inside game_state are dropped and listed as warnings.
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
	while version < target_version:
		if not _steps.has(version):
			errors.append("no migration from save version %d" % version)
			return {"ok": false, "errors": errors, "warnings": warnings, "data": data}
		var step: Callable = _steps[version]
		state = DictUtil.dict_of(step.call(state))
		version += 1
	for key: Variant in state.keys():
		if not GameState.KEYS.has(str(key)):
			warnings.append("dropped unknown game_state key '%s'" % str(key))
			state.erase(key)
	data["game_state"] = state
	data["version"] = version
	return {"ok": true, "errors": errors, "warnings": warnings, "data": data}
