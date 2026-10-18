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

# SETTINGS-UI: the fixed levels of the two stepped settings. Any stored or set value snaps to the nearest level
# (a tie goes to the lower one), so old files with free values (0.8, 1.37 ...) load as a valid level; no file version change.
const SPEED_LEVELS: Array[float] = [0.5, 1.0, 1.5, 2.0]
const TEXT_LEVELS: Array[float] = [0.85, 1.0, 1.15, 1.3]

# key -> {"type": Variant.Type, "default": value, "min"/"max": clamp for floats, or "levels": snap list}. UI1-L appends its rows.
const KEYS: Dictionary = {
	"audio/master": {"type": TYPE_FLOAT, "default": 1.0, "min": 0.0, "max": 1.0},
	"audio/mute": {"type": TYPE_BOOL, "default": false},   # CONTROLS-HELP: mutes the Master bus (the levels are kept)
	"audio/music": {"type": TYPE_FLOAT, "default": 0.8, "min": 0.0, "max": 1.0},
	"audio/sfx": {"type": TYPE_FLOAT, "default": 0.8, "min": 0.0, "max": 1.0},
	"audio/ui": {"type": TYPE_FLOAT, "default": 0.8, "min": 0.0, "max": 1.0},
	# UI1-L: animation (read by FxSettings.load_from) and text size (the menu theme's default font size).
	"anim/speed": {"type": TYPE_FLOAT, "default": 1.0, "levels": SPEED_LEVELS},
	"anim/instant": {"type": TYPE_BOOL, "default": false},
	"ui/text_scale": {"type": TYPE_FLOAT, "default": 1.0, "levels": TEXT_LEVELS},
	"ui/intent_tells": {"type": TYPE_BOOL, "default": true},   # INTENT1-U: enemy kind chips and arrows on the grid while planning (hotkey I)
	"ui/warn_missing_actions": {"type": TYPE_BOOL, "default": true},   # COMBAT-UI3: Confirm round asks first when a hero has empty blocks
	# DISPLAY1 (desktop only; the web build writes none): "choices" is the list of allowed values, anything else reads as the default.
	"display/window_mode": {"type": TYPE_STRING, "default": "windowed", "choices": ["windowed", "borderless", "fullscreen"]},
	"display/resolution": {"type": TYPE_STRING, "default": "1280x720", "choices": ["960x540", "1024x576", "1152x648", "1280x720", "1366x768", "1600x900", "1920x1080", "2560x1440"]},
	"display/vsync": {"type": TYPE_BOOL, "default": true},
	"display/max_fps": {"type": TYPE_INT, "default": 0, "choices": [30, 60, 120, 144, 0]},
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
static func load_file() -> void:
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
	if not _file().has_section_key(parts[0], parts[1]):
		return dflt
	var raw: Variant = _file().get_value(parts[0], parts[1])
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

# `value` as `key` would store it: clamped, or the default when it has the wrong type (UI1-L settings draft).
static func checked(key: String, value: Variant) -> Variant:
	var clean: Variant = _clean(key, value)
	return clean if clean != null else default_of(key)

# The level of `key` nearest to `value` (a tie goes to the lower level); `value` unchanged for keys without levels.
static func snap(key: String, value: float) -> float:
	var levels: Array = DictUtil.array_of(_row(key).get("levels", []))
	if levels.is_empty():
		return value
	var best: float = DictUtil.to_float(levels[0])
	for raw: Variant in levels:
		var lv: float = DictUtil.to_float(raw)
		if absf(lv - value) < absf(best - value):
			best = lv
	return best

static func get_float(key: String) -> float:
	return DictUtil.to_float(get_value(key))

static func get_bool(key: String) -> bool:
	return DictUtil.to_bool(get_value(key))

static func _file() -> ConfigFile:
	if _cfg == null:
		SettingsFile.load_file()
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
		if row.has("levels"):
			return snap(key, DictUtil.to_float(value))
		return clampf(DictUtil.to_float(value), DictUtil.to_float(row.get("min", -INF)), DictUtil.to_float(row.get("max", INF)))
	if typeof(value) != kind:
		return null
	if row.has("choices") and not DictUtil.array_of(row["choices"]).has(value):
		return null
	return value
