---
class: VolumePreview
extends: RefCounted
description: "Tactical RPG - Volume preview (SOUND-PREVIEW) Which sample the settings screen plays when a volume slider changes, on which bus, and the throttle that keeps a drag from spamming (one sample per INTERVAL seconds)."
---

# Properties
* `INTERVAL`: float = 0.15 (const)
* `SAMPLES`: Dictionary = { (const)
* `_last`: float = -1000.0

# Methods

### `static sample_of(key: String) -> String`
```logic
return str(DictUtil.dict_of(SAMPLES.get(key, {})).get("sample", ""))
```

### `static volume_bus_of(key: String) -> String`
```logic
for bus: String in AudioDirector.BUSES:
	if str(AudioDirector.BUSES[bus]) == key:
		return bus
return ""
```

### `static bus_of(key: String) -> String`
```logic
return str(DictUtil.dict_of(SAMPLES.get(key, {})).get("bus", ""))
```

### `allow(now: float) -> bool`
```logic
if now - _last < INTERVAL:
	return false
_last = now
return true
```

### `reset() -> void`
```logic
_last = -1000.0
```

