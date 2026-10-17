class_name SettingsModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Settings model (UI1-L): a draft of the SettingsFile keys; apply() writes the file and pushes the
# values to FxSettings (S14-L2) and AudioDirector (S14-L3) when those classes exist; revert() drops the draft.
# ==============================================================================

const LABELS: Dictionary = {
	"audio/master": "Master volume", "audio/music": "Music volume", "audio/sfx": "Effects volume", "audio/ui": "Interface volume",
	"anim/speed": "Animation speed", "anim/instant": "Instant animations", "ui/text_scale": "Text size",
}
# Classes that read the settings file through a static load_from(ConfigFile), when they exist.
const LISTENERS: Array[String] = ["FxSettings", "AudioDirector"]

var draft: Dictionary = {}

func _init() -> void:
	revert()

func keys() -> Array[String]:
	var out: Array[String] = []
	for key: String in SettingsFile.KEYS:
		out.append(key)
	return out

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
	for key: String in draft:
		SettingsFile.set_value(key, draft[key])
	if SettingsFile.save() != OK:
		errors.append("Can't write the settings file '%s'." % SettingsFile.path())
	push_to_listeners()
	return errors

func revert() -> void:
	draft = {}
	for key: String in SettingsFile.KEYS:
		draft[key] = SettingsFile.get_value(key)

# Hands the saved file to every listener class present in this build (none is required).
static func push_to_listeners() -> void:
	var cfg := ConfigFile.new()
	if cfg.load(SettingsFile.path()) != OK:
		return
	for g: Dictionary in ProjectSettings.get_global_class_list():
		if LISTENERS.has(str(g["class"])):
			var script: Script = load(str(g["path"])) as Script
			if script != null and script.has_method("load_from"):
				script.call("load_from", cfg)
