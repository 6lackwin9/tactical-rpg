class_name VolumePreview
extends RefCounted

# ==============================================================================
# Tactical RPG - Volume preview (SOUND-PREVIEW)
# Which sample the settings screen plays when a volume slider changes, on which bus, and the
# throttle that keeps a drag from spamming (one sample per INTERVAL seconds).
# ==============================================================================

const INTERVAL: float = 0.15

# SettingsFile key -> {sample: sfx key , bus: the bus it plays on}
const SAMPLES: Dictionary = {
	"audio/master": {"sample": "ui_confirm", "bus": "UI"},
	"audio/music": {"sample": "ui_confirm", "bus": "Music"},
	"audio/sfx": {"sample": "hit", "bus": "SFX"},
	"audio/ui": {"sample": "ui_click", "bus": "UI"},
}

var _last: float = -1000.0

static func sample_of(key: String) -> String:
	return str(DictUtil.dict_of(SAMPLES.get(key, {})).get("sample", ""))

# The bus whose volume the setting controls (Master for audio/master), which is not always the bus its sample plays on.
static func volume_bus_of(key: String) -> String:
	for bus: String in AudioDirector.BUSES:
		if str(AudioDirector.BUSES[bus]) == key:
			return bus
	return ""

# The bus the setting's sample plays on.
static func bus_of(key: String) -> String:
	return str(DictUtil.dict_of(SAMPLES.get(key, {})).get("bus", ""))

# True (and the clock restarts) when at least INTERVAL seconds passed since the last allowed sample; `now` in seconds.
func allow(now: float) -> bool:
	if now - _last < INTERVAL:
		return false
	_last = now
	return true

func reset() -> void:
	_last = -1000.0
