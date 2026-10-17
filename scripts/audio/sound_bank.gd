class_name SoundBank
extends RefCounted

# ==============================================================================
# Tactical RPG - Sound bank (S14-L3)
# The one place sounds are looked up: by `sfx` key through AssetRegistry, never by
# path. An unresolved key is null (silent) and warns once per key per session.
# ==============================================================================

static var _warned: Dictionary = {}
static var _warn_count: int = 0
static var _quiet: bool = false

static func stream(key: String) -> AudioStream:
	var s: AudioStream = AssetRegistry.stream("sfx", key)
	if s == null and not _warned.has(key):
		_warned[key] = true
		_warn_count += 1
		if not _quiet:
			push_warning("SoundBank: no sound for '%s'." % key)
	return s

static func has(key: String) -> bool:
	return AssetRegistry.stream("sfx", key) != null

# Test hooks: warnings since the last clear, and silencing them while a test counts.
static func clear_warned() -> void:
	_warned.clear()
	_warn_count = 0

static func warning_count() -> int:
	return _warn_count

static func set_quiet(quiet: bool) -> void:
	_quiet = quiet
