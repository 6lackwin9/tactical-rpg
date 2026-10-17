class_name SettingsFile
extends RefCounted

# ==============================================================================
# Tactical RPG - Settings file (UI1-L; shared contract with S14-L3): player settings in a ConfigFile at
# user://settings.cfg, never part of a save slot. Keys are "section/name" (KEYS: type, default, clamp). load() reads
# the file (missing = defaults); a value of the wrong type falls back to the default with one warning; numbers are
# clamped. use_path() points it at a sandbox file for tests. S14-L3 adds its keys here when it lands.
# ==============================================================================

const DEFAULT_PATH: String = "user://settings.cfg"
const STATE_KEY: StringName = &"settings_file_state"
# key -> {type: TYPE_FLOAT|TYPE_BOOL, default, min, max}
const KEYS: Dictionary = {
	"audio/master": {"type": TYPE_FLOAT, "default": 1.0, "min": 0.0, "max": 1.0},
	"audio/music": {"type": TYPE_FLOAT, "default": 0.8, "min": 0.0, "max": 1.0},
	"audio/sfx": {"type": TYPE_FLOAT, "default": 0.8, "min": 0.0, "max": 1.0},
	"audio/ui": {"type": TYPE_FLOAT, "default": 0.8, "min": 0.0, "max": 1.0},
	"anim/speed": {"type": TYPE_FLOAT, "default": 1.0, "min": 0.25, "max": 4.0},
	"anim/instant": {"type": TYPE_BOOL, "default": false},
	"ui/text_scale": {"type": TYPE_FLOAT, "default": 1.0, "min": 0.8, "max": 1.5},
}

static func _data() -> Dictionary:
	if not Engine.has_meta(STATE_KEY):
		var warnings: Array[String] = []
		Engine.set_meta(STATE_KEY, {"path": DEFAULT_PATH, "values": defaults(), "warnings": warnings, "loaded": false})
	var d: Dictionary = Engine.get_meta(STATE_KEY)
	return d

static func path() -> String:
	return str(_data()["path"])

# Points the settings at `p` (tests); the values reset to the defaults until load().
static func use_path(p: String) -> void:
	var d: Dictionary = _data()
	d["path"] = p
	d["values"] = defaults()
	d["loaded"] = false
	_warnings().clear()

static func reset_path() -> void:
	use_path(DEFAULT_PATH)

static func defaults() -> Dictionary:
	var out: Dictionary = {}
	for key: String in KEYS:
		out[key] = DictUtil.dict_of(KEYS[key])["default"]
	return out

static func _values() -> Dictionary:
	if not DictUtil.to_bool(_data()["loaded"]):
		var _err: Error = SettingsFile.load()
	var v: Dictionary = _data()["values"]
	return v

static func _warnings() -> Array[String]:
	var w: Array[String] = _data()["warnings"]
	return w

# Warnings from the last load or set (wrong types).
static func warnings() -> Array[String]:
	return _warnings().duplicate()

# Reads the file; missing file or keys = defaults. Returns the file's open error (ERR_FILE_NOT_FOUND is fine).
static func load() -> Error:
	var d: Dictionary = _data()
	d["values"] = defaults()
	d["loaded"] = true
	_warnings().clear()
	var cfg := ConfigFile.new()
	var err: Error = cfg.load(path())
	if err != OK:
		return err
	var values: Dictionary = d["values"]
	for key: String in KEYS:
		var parts: PackedStringArray = key.split("/")
		if cfg.has_section_key(parts[0], parts[1]):
			values[key] = checked(key, cfg.get_value(parts[0], parts[1]))
	return OK

# Writes every key to the file.
static func save() -> Error:
	var cfg := ConfigFile.new()
	var values: Dictionary = _values()
	for key: String in KEYS:
		var parts: PackedStringArray = key.split("/")
		cfg.set_value(parts[0], parts[1], values[key])
	var dir: String = path().get_base_dir()
	if not dir.is_empty() and not DirAccess.dir_exists_absolute(dir):
		DirAccess.make_dir_recursive_absolute(dir)
	return cfg.save(path())

# The value of `key` (clamped, typed); `fallback` for an unknown key.
static func get_value(key: String, fallback: Variant = null) -> Variant:
	var values: Dictionary = _values()
	return values[key] if values.has(key) else fallback

# Sets `key` in memory (clamped; wrong type = default plus a warning). Call save() to write the file.
static func set_value(key: String, value: Variant) -> void:
	if not KEYS.has(key):
		_warnings().append("Unknown setting '%s'." % key)
		return
	_values()[key] = checked(key, value)

static func checked(key: String, value: Variant) -> Variant:
	var spec: Dictionary = DictUtil.dict_of(KEYS[key])
	var t: int = DictUtil.to_int(spec["type"])
	if t == TYPE_BOOL:
		if value is bool:
			return value
	elif value is float or value is int:
		return clampf(DictUtil.to_float(value), DictUtil.to_float(spec["min"]), DictUtil.to_float(spec["max"]))
	_warnings().append("Setting '%s' has the wrong type; using the default." % key)
	return spec["default"]
