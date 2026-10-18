---
class: SettingsFile
extends: RefCounted
description: "Tactical RPG - Settings file (S14-L3 audio keys; UI1-L adds its keys and the screen) One ConfigFile at user://settings.cfg, never part of a save slot. Keys are \"section/name\". A missing file or key gives the key's default; a value of the wrong type gives the default and one warning per key. Known float keys clamp. Usage: SettingsFile.get_value(\"audio/sfx\", 0.8); use_path(p) points tests at a sandbox."
---

# Properties
* `DEFAULT_PATH`: String = "user://settings.cfg" (const)
* `SPEED_LEVELS`: Array[float] = [0.5, 1.0, 1.5, 2.0] (const)
* `TEXT_LEVELS`: Array[float] = [0.85, 1.0, 1.15, 1.3] (const)
* `KEYS`: Dictionary = { (const)

# Methods

### `static use_path(path: String = DEFAULT_PATH) -> void`
```logic
_path = path
_cfg = null
_warned.clear()
```

### `static path() -> String`
```logic
return _path
```

### `static load_file() -> void`
```logic
_cfg = ConfigFile.new()
if FileAccess.file_exists(_path):
	var err: Error = _cfg.load(_path)
	if err != OK:
		push_warning("SettingsFile: could not read %s (error %d); using defaults." % [_path, err])
		_cfg = ConfigFile.new()
```

### `static save() -> Error`
```logic
return _file().save(_path)
```

### `static default_of(key: String) -> Variant`
```logic
return _row(key).get("default")
```

### `static get_value(key: String, fallback: Variant = null) -> Variant`
```logic
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
```

### `static set_value(key: String, value: Variant) -> void`
```logic
var parts: PackedStringArray = _split(key)
var clean: Variant = _clean(key, value)
_file().set_value(parts[0], parts[1], clean if clean != null else default_of(key))
```

### `static checked(key: String, value: Variant) -> Variant`
```logic
var clean: Variant = _clean(key, value)
return clean if clean != null else default_of(key)
```

### `static snap(key: String, value: float) -> float`
```logic
var levels: Array = DictUtil.array_of(_row(key).get("levels", []))
if levels.is_empty():
	return value
var best: float = DictUtil.to_float(levels[0])
for raw: Variant in levels:
	var lv: float = DictUtil.to_float(raw)
	if absf(lv - value) < absf(best - value):
		best = lv
return best
```

### `static get_float(key: String) -> float`
```logic
return DictUtil.to_float(get_value(key))
```

### `static get_bool(key: String) -> bool`
```logic
return DictUtil.to_bool(get_value(key))
```

### `static _file() -> ConfigFile`
```logic
if _cfg == null:
	SettingsFile.load_file()
return _cfg
```

### `static _row(key: String) -> Dictionary`
```logic
var raw: Variant = KEYS.get(key, {})
return raw if raw is Dictionary else {}
```

### `static _split(key: String) -> PackedStringArray`
```logic
var i: int = key.find("/")
return PackedStringArray([key.substr(0, i), key.substr(i + 1)]) if i > 0 else PackedStringArray(["", key])
```

### `static _clean(key: String, value: Variant) -> Variant`
```logic
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
```

