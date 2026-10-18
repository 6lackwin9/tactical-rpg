---
class: SoundEvents
extends: RefCounted
description: "Tactical RPG - Sound event keys (S14-L3) The `sfx` keys the game plays (manifest rows of S14-A2; any may be missing, which is silent) and the presenter preset -> key mapping. \"\" means no sound. key -> what fires it"
---

# Properties
* `KEYS`: Dictionary = { (const)
* `FOR_PRESET`: Dictionary = { (const)
* `out`: Array[String] = []

# Methods

### `static key_for(preset_id: String, entry: ActionLogEntry) -> String`
```logic
var key: String = str(FOR_PRESET.get(preset_id, ""))
if entry == null:
	return key
if preset_id == "hit" and "CRIT" in entry.outcome_tag:
	return "crit"
if preset_id == "status":
	var status_id: String = FxPresets.status_id_of(entry)
	return "" if status_id.is_empty() or StatusArt.is_buff(status_id) else key
return key
```

