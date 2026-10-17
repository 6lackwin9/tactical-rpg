class_name SettingsModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Settings model (UI1-L): a draft of the SettingsFile keys; apply() writes the file and pushes the
# values to FxSettings (S14-L2 animation) and the active AudioDirector (S14-L3 volumes); revert() drops the draft.
# ==============================================================================

const LABELS: Dictionary = {
	"audio/master": "Master volume", "audio/music": "Music volume", "audio/sfx": "Effects volume", "audio/ui": "Interface volume",
	"anim/speed": "Animation speed", "anim/instant": "Skip animations", "ui/text_scale": "Text size",
	"display/window_mode": "Window mode", "display/resolution": "Window size", "display/vsync": "Vertical sync", "display/max_fps": "Frame rate cap",
}

# DISPLAY1: shown names of the listed choices (a value without a name shows as itself).
const CHOICE_NAMES: Dictionary = {
	"display/window_mode": {"windowed": "Windowed", "borderless": "Borderless fullscreen", "fullscreen": "Fullscreen"},
	"display/max_fps": {0: "Unlimited"},
}
const DISPLAY_KEYS: Array[String] = ["display/window_mode", "display/resolution", "display/vsync", "display/max_fps"]
const CONFIRM_KEYS: Array[String] = ["display/window_mode", "display/resolution"]   # applied live, written only after Keep
const CONFIRM_SECONDS: float = 10.0
const SIZE_HINT: String = "Used in windowed mode"

# SETTINGS-UI: the names of the fixed levels (same order as SettingsFile.SPEED_LEVELS / TEXT_LEVELS).
const LEVEL_NAMES: Dictionary = {
	"anim/speed": ["Slow", "Normal", "Fast", "Very fast"],
	"ui/text_scale": ["Small", "Normal", "Large", "Extra large"],
}

var draft: Dictionary = {}
var confirm_left: float = 0.0   # seconds left to keep a changed window mode / size (0 = no question open)
var _previous: Dictionary = {}   # the saved mode and size to go back to

func _init() -> void:
	revert()

func keys() -> Array[String]:
	var out: Array[String] = []
	for key: String in SettingsFile.KEYS:
		out.append(key)
	return out

# The level values of a stepped key (empty for sliders and checkboxes).
static func levels(key: String) -> Array[float]:
	var out: Array[float] = []
	out.assign(DictUtil.array_of(DictUtil.dict_of(SettingsFile.KEYS.get(key, {})).get("levels", [])))
	return out

# "Normal (1.00x)" for the level at `index`.
static func level_text(key: String, index: int) -> String:
	var lv: Array[float] = levels(key)
	var names: Array = DictUtil.array_of(LEVEL_NAMES.get(key, []))
	return "%s (%.2fx)" % [str(names[index]), lv[index]]

# The allowed values of a listed key (empty for the others); the window size list only has sizes that fit the screen.
static func choices(key: String) -> Array:
	if key == DisplaySettings.KEY_SIZE:
		return DisplaySettings.sizes()
	return DictUtil.array_of(DictUtil.dict_of(SettingsFile.KEYS.get(key, {})).get("choices", []))

static func choice_text(key: String, v: Variant) -> String:
	var names: Dictionary = DictUtil.dict_of(CHOICE_NAMES.get(key, {}))
	return str(names.get(v, v))

func value(key: String) -> Variant:
	return draft.get(key, null)

func has_draft() -> bool:
	for key: String in SettingsFile.KEYS:
		if draft.get(key, null) != SettingsFile.get_value(key):
			return true
	return false

# Changes the draft (clamped like the file); unknown keys are refused.
func set_value(key: String, v: Variant) -> Array[String]:
	var errors: Array[String] = []
	if not SettingsFile.KEYS.has(key):
		errors.append("Unknown setting '%s'." % key)
		return errors
	draft[key] = SettingsFile.checked(key, v)
	return errors

func apply() -> Array[String]:
	var errors: Array[String] = []
	if confirming():
		return errors
	var web: bool = DisplaySettings.is_web()   # the browser build writes no display key
	var changed: bool = false
	for key: String in draft:
		if web and DISPLAY_KEYS.has(key):
			continue
		if CONFIRM_KEYS.has(key):
			changed = changed or draft[key] != SettingsFile.get_value(key)
			continue
		SettingsFile.set_value(key, draft[key])
	if SettingsFile.save() != OK:
		errors.append("Can't write the settings file '%s'." % SettingsFile.path())
	push_to_listeners()
	if web:
		return errors
	DisplaySettings.apply_vsync_cap(DictUtil.to_bool(draft[DisplaySettings.KEY_VSYNC]), DictUtil.to_int(draft[DisplaySettings.KEY_FPS]))
	if changed:
		_previous = {}
		for key: String in CONFIRM_KEYS:
			_previous[key] = SettingsFile.get_value(key)
		DisplaySettings.apply_mode_size(str(draft[DisplaySettings.KEY_MODE]), str(draft[DisplaySettings.KEY_SIZE]))
		confirm_left = CONFIRM_SECONDS
	return errors

# DISPLAY1 rule 6: a changed window mode / size is live but unsaved until Keep; Revert, Esc and the timeout go back.
func confirming() -> bool:
	return not _previous.is_empty()

func keep_display() -> Array[String]:
	var errors: Array[String] = []
	if not confirming():
		return errors
	for key: String in CONFIRM_KEYS:
		SettingsFile.set_value(key, draft[key])
	_previous = {}
	confirm_left = 0.0
	if SettingsFile.save() != OK:
		errors.append("Can't write the settings file '%s'." % SettingsFile.path())
	return errors

func revert_display() -> void:
	if not confirming():
		return
	for key: String in CONFIRM_KEYS:
		draft[key] = _previous[key]
	DisplaySettings.apply_mode_size(str(_previous[DisplaySettings.KEY_MODE]), str(_previous[DisplaySettings.KEY_SIZE]))
	_previous = {}
	confirm_left = 0.0

# Counts the question down; true when the time ran out and the old mode / size came back.
func tick(delta: float) -> bool:
	if not confirming():
		return false
	confirm_left -= delta
	if confirm_left > 0.0:
		return false
	revert_display()
	return true

# SettingsFile defaults for every key into the draft (Apply saves them; a display change still goes through Keep / Revert).
func restore_defaults() -> void:
	for key: String in SettingsFile.KEYS:
		draft[key] = SettingsFile.default_of(key)

func revert() -> void:
	draft = {}
	for key: String in SettingsFile.KEYS:
		draft[key] = SettingsFile.get_value(key)

# Applies the saved values: animation speed and instant mode, then the bus volumes of the running AudioDirector.
static func push_to_listeners() -> void:
	var cfg := ConfigFile.new()
	if cfg.load(SettingsFile.path()) == OK:
		FxSettings.load_from(cfg)
	if AudioDirector.active != null:
		AudioDirector.active.apply_volumes()
