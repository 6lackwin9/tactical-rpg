class_name DisplaySettings
extends RefCounted

# ==============================================================================
# Tactical RPG - Display settings (DISPLAY1): maps the saved window mode, window size, vsync and frame-rate cap to the window.
# All window calls go through a Backend: the base class only records them (used headless, so the gate and the sims never
# touch a real window, and by tests); RealBackend calls DisplayServer / Engine. Pure helpers are static.
# Usage: DisplaySettings.apply_saved() at boot; DisplaySettings.use_backend(fake) in tests (use_backend() resets).
# ==============================================================================

const MODE_WINDOWED: String = "windowed"
const MODE_BORDERLESS: String = "borderless"
const MODE_FULLSCREEN: String = "fullscreen"
const BASE_SIZE: Vector2i = Vector2i(1280, 720)

const KEY_MODE: String = "display/window_mode"
const KEY_SIZE: String = "display/resolution"
const KEY_VSYNC: String = "display/vsync"
const KEY_FPS: String = "display/max_fps"

# Setting value -> DisplayServer window mode. Borderless is Godot 4's WINDOW_MODE_FULLSCREEN (a borderless window over the screen).
const MODE_OF: Dictionary = {
	MODE_WINDOWED: DisplayServer.WINDOW_MODE_WINDOWED,
	MODE_BORDERLESS: DisplayServer.WINDOW_MODE_FULLSCREEN,
	MODE_FULLSCREEN: DisplayServer.WINDOW_MODE_EXCLUSIVE_FULLSCREEN,
}

# Records every window call; the stand-in used headless and in tests.
class Backend extends RefCounted:
	var usable: Vector2i = Vector2i(2560, 1440)
	var mode: int = DisplayServer.WINDOW_MODE_WINDOWED
	var size: Vector2i = Vector2i(1280, 720)
	var vsync: bool = true
	var fps: int = 0
	var calls: Array[String] = []
	var centred: int = 0

	func usable_size() -> Vector2i:
		return usable

	func current_mode() -> int:
		return mode

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

# The real window (desktop and web); never created headless.
class RealBackend extends Backend:
	func usable_size() -> Vector2i:
		return DisplayServer.screen_get_usable_rect(DisplayServer.window_get_current_screen()).size

	func current_mode() -> int:
		return DisplayServer.window_get_mode()

	func set_mode(m: int) -> void:
		DisplayServer.window_set_mode(m)

	func set_size(s: Vector2i) -> void:
		DisplayServer.window_set_size(s)

	func centre() -> void:
		var area: Rect2i = DisplayServer.screen_get_usable_rect(DisplayServer.window_get_current_screen())
		DisplayServer.window_set_position(area.position + (area.size - DisplayServer.window_get_size()) / 2)

	func set_vsync(on: bool) -> void:
		DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_ENABLED if on else DisplayServer.VSYNC_DISABLED)

	func set_max_fps(n: int) -> void:
		Engine.max_fps = n

static var _backend: Backend = null
static var web_override: int = -1   # tests: 1 forces the web build, 0 the desktop build, -1 asks the engine
static var _booted: bool = false

static func use_backend(b: Backend = null) -> void:
	_backend = b

static func backend() -> Backend:
	if _backend == null:
		_backend = Backend.new() if DisplayServer.get_name() == "headless" else RealBackend.new()
	return _backend

static func is_web() -> bool:
	return OS.has_feature("web") if web_override < 0 else web_override == 1

# "1600x900" -> (1600, 900); the base size for anything malformed.
static func parse_size(text: String) -> Vector2i:
	var parts: PackedStringArray = text.split("x")
	if parts.size() != 2 or not parts[0].is_valid_int() or not parts[1].is_valid_int():
		return BASE_SIZE
	return Vector2i(parts[0].to_int(), parts[1].to_int())

# The window sizes that fit the usable screen area (ascending); 1280x720 is always offered.
static func sizes() -> Array[String]:
	var usable: Vector2i = backend().usable_size()
	var out: Array[String] = []
	for raw: Variant in DictUtil.array_of(DictUtil.dict_of(SettingsFile.KEYS[KEY_SIZE]).get("choices", [])):
		var s: String = str(raw)
		var v: Vector2i = parse_size(s)
		if s == "1280x720" or (v.x <= usable.x and v.y <= usable.y):
			out.append(s)
	return out

# The saved size when it fits, else the largest size that fits (the file is not rewritten).
static func fit_size(saved: String) -> String:
	var list: Array[String] = sizes()
	return saved if list.has(saved) else list[list.size() - 1]

# Window mode and, in windowed mode only, the window size (centred on its screen afterwards).
static func apply_mode_size(mode_id: String, size_id: String) -> void:
	var b: Backend = backend()
	b.set_mode(DictUtil.to_int(MODE_OF.get(mode_id, DisplayServer.WINDOW_MODE_WINDOWED)))
	if mode_id == MODE_WINDOWED:
		b.set_size(parse_size(fit_size(size_id)))
		b.centre()

# Vsync on or off, and the cap (0 = unlimited; with vsync on a cap below the refresh rate still applies).
static func apply_vsync_cap(vsync: bool, cap: int) -> void:
	backend().set_vsync(vsync)
	backend().set_max_fps(cap)

# Boot: applies the saved values once (never on web: the browser owns the canvas).
static func apply_saved() -> void:
	if _booted or is_web():
		return
	_booted = true
	apply_mode_size(str(SettingsFile.get_value(KEY_MODE)), str(SettingsFile.get_value(KEY_SIZE)))
	apply_vsync_cap(DictUtil.to_bool(SettingsFile.get_value(KEY_VSYNC)), DictUtil.to_int(SettingsFile.get_value(KEY_FPS)))

static func reset_boot() -> void:
	_booted = false
