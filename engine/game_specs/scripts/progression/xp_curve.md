---
class: XpCurve
extends: RefCounted
description: "Tactical RPG - XP curve (S7). Pure functions over data/progression/xp_curve.json. level_for(total_xp)     level 1..MAX_LEVEL derived from cumulative XP xp_for_level(level)     cumulative XP threshold of a level (inverse) kill_xp(...)            XP of one defeated enemy for one member The loader never crashes: an invalid or missing file yields DEFAULTS plus an error list."
---

# Properties
* `PATH`: String = "res://data/progression/xp_curve.json" (const)
* `DEFAULTS`: Dictionary = { (const)
* `errors`: Array[String] = []
* `steps`: Array = DictUtil.array_of(doc.get("xp_to_next", []))
* `kill`: Dictionary = DictUtil.dict_of(doc.get("kill_xp", {}))
* `v`: Variant = kill.get(key)
* `mult`: Dictionary = DictUtil.dict_of(kill.get("tier_mult", {}))
* `names`: Array[String] = StatRules.tier_names()
* `errors`: Array[String] = validate(p_data)

# Methods

### `static load_file(path: String) -> Dictionary`
```logic
var errors: Array[String] = []
var doc: Dictionary = {}
if not FileAccess.file_exists(path):
	errors.append("XP curve file '%s' not found; using defaults." % path)
else:
	var parsed: Variant = JSON.parse_string(FileAccess.get_file_as_string(path))
	if parsed is Dictionary:
		doc = parsed
		errors = validate(doc)
	else:
		errors.append("XP curve file '%s' is not valid JSON; using defaults." % path)
if not errors.is_empty():
	doc = DEFAULTS.duplicate(true)
return {"data": doc, "errors": errors}
```

### `static _ensure() -> void`
```logic
if _loaded:
	return
var res: Dictionary = load_file(PATH)
_data = DictUtil.dict_of(res["data"])
_errors.assign(DictUtil.array_of(res["errors"]))
_loaded = true
```

### `static data() -> Dictionary`
```logic
_ensure()
return _data
```

### `static reset() -> void`
```logic
_loaded = false
```

### `static xp_for_level(level: int) -> int`
```logic
var steps: Array = DictUtil.array_of(data()["xp_to_next"])
var total: int = 0
for i: int in range(clampi(level, 1, StatRules.MAX_LEVEL) - 1):
	total += DictUtil.to_int(steps[i])
return total
```

### `static level_for(total_xp: int) -> int`
```logic
var level: int = 1
while level < StatRules.MAX_LEVEL and total_xp >= xp_for_level(level + 1):
	level += 1
return level
```

### `static xp_to_next(total_xp: int) -> int`
```logic
var level: int = level_for(total_xp)
return 0 if level >= StatRules.MAX_LEVEL else xp_for_level(level + 1) - total_xp
```

### `static kill_xp(enemy_level: int, enemy_tier: String, member_level: int) -> int`
```logic
var kill: Dictionary = DictUtil.dict_of(data()["kill_xp"])
var mult: Dictionary = DictUtil.dict_of(kill["tier_mult"])
var tier_mult: float = DictUtil.to_float(mult.get(enemy_tier, 1.0), 1.0)
var delta: float = clampf(1.0 + DictUtil.to_float(kill["level_delta_step"]) * float(enemy_level - member_level),
	DictUtil.to_float(kill["level_delta_min"]), DictUtil.to_float(kill["level_delta_max"]))
var raw: float = (DictUtil.to_float(kill["base"]) + DictUtil.to_float(kill["per_level"]) * float(enemy_level)) * tier_mult * delta
return maxi(1, roundi(raw))
```

### `static encounter_xp(enemies: Array[Dictionary], member_level: int) -> int`
```logic
var total: int = 0
for e: Dictionary in enemies:
	total += kill_xp(DictUtil.to_int(e.get("level", 1), 1), str(e.get("tier", "standard")), member_level)
return total
```

