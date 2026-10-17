class_name SettingsFile
extends RefCounted

# ==============================================================================
# Tactical RPG - Settings file (S14-L3 audio keys; UI1-L adds its keys and the screen)
# One ConfigFile at user://settings.cfg, never part of a save slot. Keys are
# "section/name". A missing file or key gives the key's default; a value of the
# wrong type gives the default and one warning per key. Known float keys clamp.
# Usage: SettingsFile.get_value("audio/sfx", 0.8); use_path(p) points tests at a sandbox.
# ==============================================================================

const DEFAULT_PATH: String = "user://settings.cfg"

# key -> {"type": Variant.Type, "default": value, "min"/"max": clamp for floats}. UI1-L appends its rows.
const KEYS: Dictionary = {
	"audio/master": {"type": TYPE_FLOAT, "default": 1.0, "min": 0.0, "max": 1.0},
	"audio/music": {"type": TYPE_FLOAT, "default": 0.8, "min": 0.0, "max": 1.0},
	"audio/sfx": {"type": TYPE_FLOAT, "default": 0.8, "min": 0.0, "max": 1.0},
	"audio/ui": {"type": TYPE_FLOAT, "default": 0.8, "min": 0.0, "max": 1.0},
}

static var _path: String = DEFAULT_PATH
static var _cfg: ConfigFile = null
static var _warned: Dictionary = {}

static func use_path(path: String = DEFAULT_PATH) -> void:
	_path = path
	_cfg = null
	_warned.clear()

static func path() -> String:
	return _path

# (Re)reads the file; a missing or unreadable file leaves every key at its default.
static func load() -> void:
	_cfg = ConfigFile.new()
	if FileAccess.file_exists(_path):
		var err: Error = _cfg.load(_path)
		if err != OK:
			push_warning("SettingsFile: could not read %s (error %d); using defaults." % [_path, err])
			_cfg = ConfigFile.new()

static func save() -> Error:
	return _file().save(_path)

static func default_of(key: String) -> Variant:
	return _row(key).get("default")

# The stored value of `key`; `fallback` (else the key's default) when missing or of the wrong type.
static func get_value(key: String, fallback: Variant = null) -> Variant:
	var dflt: Variant = fallback if fallback != null else default_of(key)
	var parts: PackedStringArray = _split(key)
	var raw: Variant = _file().get_value(parts[0], parts[1], null)
	if raw == null:
		return dflt
	var clean: Variant = _clean(key, raw)
	if clean == null:
		if not _warned.has(key):
			_warned[key] = true
			push_warning("SettingsFile: '%s' has the wrong type; using the default." % key)
		return dflt
	return clean

static func set_value(key: String, value: Variant) -> void:
	var parts: PackedStringArray = _split(key)
	var clean: Variant = _clean(key, value)
	_file().set_value(parts[0], parts[1], clean if clean != null else default_of(key))

static func get_float(key: String) -> float:
	return DictUtil.to_float(get_value(key))

static func _file() -> ConfigFile:
	if _cfg == null:
		SettingsFile.load()
	return _cfg

static func _row(key: String) -> Dictionary:
	var raw: Variant = KEYS.get(key, {})
	return raw if raw is Dictionary else {}

static func _split(key: String) -> PackedStringArray:
	var i: int = key.find("/")
	return PackedStringArray([key.substr(0, i), key.substr(i + 1)]) if i > 0 else PackedStringArray(["", key])

# The value in its key's type (floats clamped), or null when it does not fit. Unknown keys pass through.
static func _clean(key: String, value: Variant) -> Variant:
	var row: Dictionary = _row(key)
	if row.is_empty():
		return value
	var kind: int = DictUtil.to_int(row.get("type", TYPE_NIL))
	if kind == TYPE_FLOAT:
		if not (value is float or value is int):
			return null
		return clampf(float(value), DictUtil.to_float(row.get("min", -INF)), DictUtil.to_float(row.get("max", INF)))
	return value if typeof(value) == kind else null
