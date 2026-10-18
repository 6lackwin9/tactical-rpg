class_name SoundEvents
extends RefCounted

# ==============================================================================
# Tactical RPG - Sound event keys (S14-L3)
# The `sfx` keys the game plays (manifest rows of S14-A2; any may be missing, which
# is silent) and the presenter preset -> key mapping. "" means no sound.
# ==============================================================================

# key -> what fires it
const KEYS: Dictionary = {
	"hit": "presenter preset hit",
	"miss": "preset miss (no asset yet: silent)",
	"crit": "preset hit when the outcome tag has CRIT (replaces hit)",
	"mitigated": "preset mitigated",
	"heal": "preset heal",
	"buff": "preset buff",
	"debuff": "preset status on a debuff",
	"defend": "preset defend",
	"move_step": "preset move",
	"defeat": "preset defeat",
	"ui_click": "AudioDirector.ui(\"click\") (playback controls)",
	"ui_confirm": "reserved for UI1-L views",
	"ui_back": "reserved for UI1-L views",
	"ui_error": "reserved for UI1-L views",
}

const FOR_PRESET: Dictionary = {
	"move": "move_step",
	"attack_melee": "",
	"attack_ranged": "",
	"hit": "hit",
	"mitigated": "mitigated",
	"miss": "miss",
	"heal": "heal",
	"buff": "buff",
	"defend": "defend",
	"status": "debuff",   # a buff status plays nothing (key_for)
	"float_text": "",
	"defeat": "defeat",
}

static func keys() -> Array[String]:
	var out: Array[String] = []
	for k: Variant in KEYS:
		out.append(str(k))
	return out

# The key one presenter preset plays for an entry ("" = none).
static func key_for(preset_id: String, entry: ActionLogEntry) -> String:
	var key: String = str(FOR_PRESET.get(preset_id, ""))
	if entry == null:
		return key
	if preset_id == "hit" and "CRIT" in entry.outcome_tag:
		return "crit"
	if preset_id == "status":
		var status_id: String = FxPresets.status_id_of(entry)
		return "" if status_id.is_empty() or StatusArt.is_buff(status_id) else key
	return key
