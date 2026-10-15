class_name XpCurve
extends RefCounted

# ==============================================================================
# Tactical RPG - XP curve (S7). Pure functions over data/progression/xp_curve.json.
#   level_for(total_xp)     level 1..MAX_LEVEL derived from cumulative XP
#   xp_for_level(level)     cumulative XP threshold of a level (inverse)
#   kill_xp(...)            XP of one defeated enemy for one member
# The loader never crashes: an invalid or missing file yields DEFAULTS plus an error list.
# ==============================================================================

const PATH: String = "res://data/progression/xp_curve.json"

const DEFAULTS: Dictionary = {
	"version": 1,
	"xp_to_next": [40, 70, 110, 160, 220, 290, 370, 460, 560],
	"kill_xp": {
		"base": 10, "per_level": 5,
		"tier_mult": {"minion": 0.5, "standard": 1.0, "elite": 2.0, "boss": 5.0, "hero": 1.0},
		"level_delta_step": 0.15, "level_delta_min": 0.25, "level_delta_max": 2.0,
	},
}

static var _data: Dictionary = {}
static var _errors: Array[String] = []
static var _loaded: bool = false

# Problem strings of a curve document (empty = valid).
static func validate(doc: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var steps: Array = DictUtil.array_of(doc.get("xp_to_next", []))
	if steps.size() != StatRules.MAX_LEVEL - 1:
		errors.append("xp_to_next needs exactly %d entries (got %d)." % [StatRules.MAX_LEVEL - 1, steps.size()])
	for s: Variant in steps:
		if not (s is int or s is float) or DictUtil.to_int(s) <= 0:
			errors.append("xp_to_next entries must be positive numbers.")
			break
	var kill: Dictionary = DictUtil.dict_of(doc.get("kill_xp", {}))
	if kill.is_empty():
		errors.append("kill_xp is missing.")
		return errors
	for key: String in ["base", "per_level", "level_delta_step", "level_delta_min", "level_delta_max"]:
		var v: Variant = kill.get(key)
		if not (v is int or v is float):
			errors.append("kill_xp.%s must be a number." % key)
	var mult: Dictionary = DictUtil.dict_of(kill.get("tier_mult", {}))
	var names: Array[String] = StatRules.tier_names()
	for tier: String in names:
		if not mult.has(tier):
			errors.append("kill_xp.tier_mult is missing tier '%s'." % tier)
	for key: Variant in mult:
		if not names.has(str(key)):
			errors.append("kill_xp.tier_mult has unknown tier '%s'." % str(key))
		elif DictUtil.to_float(mult[key]) <= 0.0:
			errors.append("kill_xp.tier_mult.%s must be > 0." % str(key))
	if errors.is_empty():
		if DictUtil.to_float(kill["level_delta_min"]) > 1.0 or DictUtil.to_float(kill["level_delta_max"]) < 1.0:
			errors.append("level_delta_min must be <= 1 <= level_delta_max.")
		if DictUtil.to_float(kill["level_delta_min"]) <= 0.0:
			errors.append("level_delta_min must be > 0.")
	return errors

# {"data": Dictionary, "errors": Array[String]} for the file at `path` (defaults on any problem).
static func load_file(path: String) -> Dictionary:
	var errors: Array[String] = []
	var doc: Dictionary = {}
	if not FileAccess.file_exists(path):
		errors.append("XP curve file '%s' not found; using defaults." % path)
	else:
		var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
		if parsed is Dictionary:
			doc = parsed
			errors = validate(doc)
		else:
			errors.append("XP curve file '%s' is not valid JSON; using defaults." % path)
	if not errors.is_empty():
		doc = DEFAULTS.duplicate(true)
	return {"data": doc, "errors": errors}

static func _ensure() -> void:
	if _loaded:
		return
	var res: Dictionary = load_file(PATH)
	_data = DictUtil.dict_of(res["data"])
	_errors.assign(DictUtil.array_of(res["errors"]))
	_loaded = true

static func data() -> Dictionary:
	_ensure()
	return _data

static func load_errors() -> Array[String]:
	_ensure()
	return _errors

# Tests: use `p_data` (must be valid, else ignored and the errors returned); `reset` goes back to the file.
static func set_override(p_data: Dictionary) -> Array[String]:
	var errors: Array[String] = validate(p_data)
	if errors.is_empty():
		_data = p_data.duplicate(true)
		_errors.clear()
		_loaded = true
	return errors

static func reset() -> void:
	_loaded = false

# Cumulative XP needed to reach `level` (level 1 = 0).
static func xp_for_level(level: int) -> int:
	var steps: Array = DictUtil.array_of(data()["xp_to_next"])
	var total: int = 0
	for i: int in range(clampi(level, 1, StatRules.MAX_LEVEL) - 1):
		total += DictUtil.to_int(steps[i])
	return total

static func level_for(total_xp: int) -> int:
	var level: int = 1
	while level < StatRules.MAX_LEVEL and total_xp >= xp_for_level(level + 1):
		level += 1
	return level

# XP still needed for the next level (0 at max level).
static func xp_to_next(total_xp: int) -> int:
	var level: int = level_for(total_xp)
	return 0 if level >= StatRules.MAX_LEVEL else xp_for_level(level + 1) - total_xp

# round((base + per_level x enemy_level) x tier_mult x clamp(1 + step x (enemy_level - member_level), min, max)), at least 1.
static func kill_xp(enemy_level: int, enemy_tier: String, member_level: int) -> int:
	var kill: Dictionary = DictUtil.dict_of(data()["kill_xp"])
	var mult: Dictionary = DictUtil.dict_of(kill["tier_mult"])
	var tier_mult: float = DictUtil.to_float(mult.get(enemy_tier, 1.0), 1.0)
	var delta: float = clampf(1.0 + DictUtil.to_float(kill["level_delta_step"]) * float(enemy_level - member_level),
		DictUtil.to_float(kill["level_delta_min"]), DictUtil.to_float(kill["level_delta_max"]))
	var raw: float = (DictUtil.to_float(kill["base"]) + DictUtil.to_float(kill["per_level"]) * float(enemy_level)) * tier_mult * delta
	return maxi(1, roundi(raw))

# Encounter XP for one member: the sum over defeated enemies ({"level": int, "tier": String}).
static func encounter_xp(enemies: Array[Dictionary], member_level: int) -> int:
	var total: int = 0
	for e: Dictionary in enemies:
		total += kill_xp(DictUtil.to_int(e.get("level", 1), 1), str(e.get("tier", "standard")), member_level)
	return total
