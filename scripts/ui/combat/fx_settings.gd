class_name FxSettings
extends RefCounted

# ==============================================================================
# Tactical RPG - Combat FX settings (S14-L2)
# Animation speed and instant mode for CombatPresenter. Instant mode applies every
# preset's end state at once (no tween, no temporary node); headless runs and tests
# always start in it. load_from reads the UI1-L settings file once it exists.
# ==============================================================================

const SPEED_MIN: float = 0.25
const SPEED_MAX: float = 4.0

static var speed: float = 1.0
static var instant: bool = DisplayServer.get_name() == "headless"

static func set_speed(value: float) -> void:
	speed = clampf(value, SPEED_MIN, SPEED_MAX)

# A preset duration at the current speed.
static func scaled(duration: float) -> float:
	return duration / clampf(speed, SPEED_MIN, SPEED_MAX)

# Reads `anim/speed` and `anim/instant` (user://settings.cfg, owned by UI1-L).
static func load_from(cfg: ConfigFile) -> void:
	if cfg == null:
		return
	var raw_speed: Variant = cfg.get_value("anim", "speed", speed)
	if raw_speed is float or raw_speed is int:
		set_speed(DictUtil.to_float(raw_speed, speed))
	var raw_instant: Variant = cfg.get_value("anim", "instant", instant)
	if raw_instant is bool:
		instant = raw_instant or DisplayServer.get_name() == "headless"   # headless runs stay instant
