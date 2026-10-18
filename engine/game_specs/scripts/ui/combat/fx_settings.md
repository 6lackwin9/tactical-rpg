---
class: FxSettings
extends: RefCounted
description: "Tactical RPG - Combat FX settings (S14-L2) Animation speed and instant mode for CombatPresenter. Instant mode applies every preset's end state at once (no tween, no temporary node); headless runs and tests always start in it. load_from reads the UI1-L settings file once it exists."
---

# Properties
* `SPEED_MIN`: float = 0.5   # the first and last of SettingsFile.SPEED_LEVELS (a test checks it) (const)
* `SPEED_MAX`: float = 2.0 (const)

# Methods

### `static set_speed(value: float) -> void`
```logic
speed = SettingsFile.snap("anim/speed", value)   # only the fixed levels (SETTINGS-UI)
```

### `static scaled(duration: float) -> float`
```logic
return duration / clampf(speed, SPEED_MIN, SPEED_MAX)
```

### `static load_from(cfg: ConfigFile) -> void`
```logic
if cfg == null:
	return
var raw_speed: Variant = cfg.get_value("anim", "speed", speed)
if raw_speed is float or raw_speed is int:
	set_speed(DictUtil.to_float(raw_speed, speed))
var raw_instant: Variant = cfg.get_value("anim", "instant", instant)
if raw_instant is bool:
	instant = raw_instant or DisplayServer.get_name() == "headless"   # headless runs stay instant
```

