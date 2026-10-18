---
class: SettingsModel
extends: RefCounted
description: "Tactical RPG - Settings model (UI1-L): a draft of the SettingsFile keys; apply() writes the file and pushes the values to FxSettings (S14-L2 animation) and the active AudioDirector (S14-L3 volumes); revert() drops the draft."
---

# Properties
* `LABELS`: Dictionary = { (const)
* `CHOICE_NAMES`: Dictionary = { (const)
* `DISPLAY_KEYS`: Array[String] = ["display/window_mode", "display/resolution", "display/vsync", "display/max_fps"] (const)
* `CONFIRM_KEYS`: Array[String] = ["display/window_mode", "display/resolution"]   # applied live, written only after Keep (const)
* `CONFIRM_SECONDS`: float = 10.0 (const)
* `SIZE_HINT`: String = "Used in windowed mode" (const)
* `LEVEL_NAMES`: Dictionary = { (const)
* `draft`: Dictionary = {}
* `confirm_left`: float = 0.0   # seconds left to keep a changed window mode / size (0 = no question open)
* `tab`: int = 0   # the settings tab on show (kept across re-renders)
* `_previous`: Dictionary = {}   # the saved mode and size to go back to
* `out`: Array[String] = []
* `out`: Array[float] = []
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `web`: bool = DisplaySettings.is_web()   # the browser build writes no display key
* `changed`: bool = false
* `_started`: bool = start_display_change()
* `errors`: Array[String] = []

# Methods

### `_init() -> void`
```logic
revert()
```

### `static level_text(key: String, index: int) -> String`
```logic
var lv: Array[float] = levels(key)
var names: Array = DictUtil.array_of(LEVEL_NAMES.get(key, []))
return "%s (%.2fx)" % [str(names[index]), lv[index]]
```

### `static choices(key: String) -> Array`
```logic
if key == DisplaySettings.KEY_SIZE:
	return DisplaySettings.sizes()
return DictUtil.array_of(DictUtil.dict_of(SettingsFile.KEYS.get(key, {})).get("choices", []))
```

### `static choice_text(key: String, v: Variant) -> String`
```logic
var names: Dictionary = DictUtil.dict_of(CHOICE_NAMES.get(key, {}))
return str(names.get(v, v))
```

### `value(key: String) -> Variant`
```logic
return draft.get(key, null)
```

### `has_draft() -> bool`
```logic
for key: String in SettingsFile.KEYS:
	if draft.get(key, null) != SettingsFile.get_value(key):
		return true
return false
```

### `start_display_change() -> bool`
```logic
if DisplaySettings.is_web() or confirming():
	return false
var differs: bool = false
for key: String in CONFIRM_KEYS:
	differs = differs or draft[key] != SettingsFile.get_value(key)
if not differs:
	return false
_previous = {}
for key: String in CONFIRM_KEYS:
	_previous[key] = SettingsFile.get_value(key)
DisplaySettings.apply_mode_size(str(draft[DisplaySettings.KEY_MODE]), str(draft[DisplaySettings.KEY_SIZE]))
confirm_left = CONFIRM_SECONDS
return true
```

### `confirming() -> bool`
```logic
return not _previous.is_empty()
```

### `revert_display() -> void`
```logic
if not confirming():
	return
for key: String in CONFIRM_KEYS:
	draft[key] = _previous[key]
DisplaySettings.apply_mode_size(str(_previous[DisplaySettings.KEY_MODE]), str(_previous[DisplaySettings.KEY_SIZE]))
_previous = {}
confirm_left = 0.0
```

### `tick(delta: float) -> bool`
```logic
if not confirming():
	return false
confirm_left -= delta
if confirm_left > 0.0:
	return false
revert_display()
return true
```

### `restore_defaults() -> void`
```logic
for key: String in SettingsFile.KEYS:
	draft[key] = SettingsFile.default_of(key)
```

### `revert() -> void`
```logic
draft = {}
for key: String in SettingsFile.KEYS:
	draft[key] = SettingsFile.get_value(key)
```

### `static push_to_listeners() -> void`
```logic
var cfg := ConfigFile.new()
if cfg.load(SettingsFile.path()) == OK:
	FxSettings.load_from(cfg)
AudioDirector.apply_saved()   # the buses are global: saved levels apply with or without a director in the tree (startup, Apply)
```

