---
class: DisplaySettings
extends: RefCounted
description: "Tactical RPG - Display settings (DISPLAY1): maps the saved window mode, window size, vsync and frame-rate cap to the window. All window calls go through a Backend: the base class only records them (used headless, so the gate and the sims never touch a real window, and by tests); RealBackend calls DisplayServer / Engine. Pure helpers are static. Usage: DisplaySettings.apply_saved() at boot; DisplaySettings.use_backend(fake) in tests (use_backend() resets)."
---

# Properties
* `MODE_WINDOWED`: String = "windowed" (const)
* `MODE_BORDERLESS`: String = "borderless" (const)
* `MODE_FULLSCREEN`: String = "fullscreen" (const)
* `BASE_SIZE`: Vector2i = Vector2i(1280, 720) (const)
* `KEY_MODE`: String = "display/window_mode" (const)
* `KEY_SIZE`: String = "display/resolution" (const)
* `KEY_VSYNC`: String = "display/vsync" (const)
* `KEY_FPS`: String = "display/max_fps" (const)
* `MODE_OF`: Dictionary = { (const)
* `usable`: Vector2i = Vector2i(2560, 1440)
* `mode`: int = DisplayServer.WINDOW_MODE_WINDOWED
* `size`: Vector2i = Vector2i(1280, 720)
* `vsync`: bool = true
* `fps`: int = 0
* `calls`: Array[String] = []
* `centred`: int = 0
* `scale`: float = 1.0   # physical pixels per logical pixel (a high-DPI screen); the sizes in the list are logical
* `usable`: Vector2i = backend().usable_size()
* `b_scale`: float = backend().screen_scale()   # list sizes are logical, the screen is physical
* `out`: Array[String] = []
* `s`: String = str(raw)
* `v`: Vector2i = parse_size(s)

# Methods

### `usable_size() -> Vector2i`
```logic
	return usable

func screen_scale() -> float:
	return scale

func current_mode() -> int:
	return mode

func current_size() -> Vector2i:
	return size

func set_mode(m: int) -> void:
	mode = m
	calls.append("mode:%d" % m)

func set_size(s: Vector2i) -> void:
	size = s
	calls.append("size:%dx%d" % [s.x, s.y])

func centre() -> void:
	centred += 1
	calls.append("centre")

func set_vsync(on: bool) -> void:
	vsync = on
	calls.append("vsync:%s" % str(on))

func set_max_fps(n: int) -> void:
	fps = n
	calls.append("fps:%d" % n)
```

### `usable_size() -> Vector2i`
```logic
	return DisplayServer.screen_get_usable_rect(DisplayServer.window_get_current_screen()).size

func screen_scale() -> float:
	return maxf(1.0, DisplayServer.screen_get_scale(DisplayServer.window_get_current_screen()))

func current_mode() -> int:
	return DisplayServer.window_get_mode()

func current_size() -> Vector2i:
	return Vector2i((Vector2(DisplayServer.window_get_size()) / screen_scale()).round())

func set_mode(m: int) -> void:
	DisplayServer.window_set_mode(m)

# window_set_size takes physical pixels, so a 1280x720 window on a 200% screen is 2560x1440. A mode change (maximized / fullscreen
# back to windowed) can swallow a size set in the same frame, so the size is set again one frame later.
func set_size(s: Vector2i) -> void:
	var physical: Vector2i = Vector2i((Vector2(s) * screen_scale()).round())
	DisplayServer.window_set_size(physical)
	var loop: MainLoop = Engine.get_main_loop()
	var tree: SceneTree = loop as SceneTree
	if tree != null:
		tree.process_frame.connect(func() -> void:
			if DisplayServer.window_get_mode() == DisplayServer.WINDOW_MODE_WINDOWED:
				DisplayServer.window_set_size(physical)
				centre(), CONNECT_ONE_SHOT)

func centre() -> void:
	var area: Rect2i = DisplayServer.screen_get_usable_rect(DisplayServer.window_get_current_screen())
	DisplayServer.window_set_position(area.position + (area.size - DisplayServer.window_get_size()) / 2)

func set_vsync(on: bool) -> void:
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_ENABLED if on else DisplayServer.VSYNC_DISABLED)

func set_max_fps(n: int) -> void:
	Engine.max_fps = n
```

### `static use_backend(b: Backend = null) -> void`
```logic
_backend = b
```

### `static backend() -> Backend`
```logic
if _backend == null:
	_backend = Backend.new() if DisplayServer.get_name() == "headless" else RealBackend.new()
return _backend
```

### `static is_web() -> bool`
```logic
return OS.has_feature("web") if web_override < 0 else web_override == 1
```

### `static parse_size(text: String) -> Vector2i`
```logic
var parts: PackedStringArray = text.split("x")
if parts.size() != 2 or not parts[0].is_valid_int() or not parts[1].is_valid_int():
	return BASE_SIZE
return Vector2i(parts[0].to_int(), parts[1].to_int())
```

### `static fit_size(saved: String) -> String`
```logic
var list: Array[String] = sizes()
return saved if list.has(saved) else list[list.size() - 1]
```

### `static apply_mode_size(mode_id: String, size_id: String) -> void`
```logic
var b: Backend = backend()
b.set_mode(DictUtil.to_int(MODE_OF.get(mode_id, DisplayServer.WINDOW_MODE_WINDOWED)))
if mode_id == MODE_WINDOWED:
	b.set_size(parse_size(fit_size(size_id)))
	b.centre()
```

### `static apply_vsync_cap(vsync: bool, cap: int) -> void`
```logic
backend().set_vsync(vsync)
backend().set_max_fps(cap)
```

### `static apply_saved() -> void`
```logic
if _booted or is_web():
	return
_booted = true
apply_mode_size(str(SettingsFile.get_value(KEY_MODE)), str(SettingsFile.get_value(KEY_SIZE)))
apply_vsync_cap(DictUtil.to_bool(SettingsFile.get_value(KEY_VSYNC)), DictUtil.to_int(SettingsFile.get_value(KEY_FPS)))
```

### `static reset_boot() -> void`
```logic
_booted = false
```

### `static now_text() -> String`
```logic
var b: Backend = backend()
var id: String = MODE_WINDOWED
for key: String in MODE_OF:
	if DictUtil.to_int(MODE_OF[key]) == b.current_mode():
		id = key
var names: Dictionary = DictUtil.dict_of(SettingsModel.CHOICE_NAMES.get(KEY_MODE, {}))
var v: Vector2i = b.current_size()
return "%s %dx%d" % [str(names.get(id, id)), v.x, v.y]
```

