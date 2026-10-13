class_name DictUtil
extends RefCounted

# Safe typed reads from Dictionaries whose values are Variants (any key type).

static func int_of(d: Dictionary, key: Variant, fallback: int = 0) -> int:
	var raw: Variant = d.get(key, fallback)
	return raw if raw is int else fallback

static func float_of(d: Dictionary, key: Variant, fallback: float = 0.0) -> float:
	var raw: Variant = d.get(key, fallback)
	return raw if raw is float else fallback

static func bool_of(d: Dictionary, key: Variant, fallback: bool = false) -> bool:
	return to_bool(d.get(key, fallback), fallback)

# Lenient conversions (JSON numbers arrive as floats, form input as strings).
static func to_int(v: Variant, fallback: int = 0) -> int:
	if v is int:
		return v
	if v is float:
		var x: float = v
		return roundi(x)
	if v is bool:
		var b: bool = v
		return 1 if b else 0
	if v is String:
		var s: String = v
		return roundi(s.to_float()) if s.is_valid_float() else fallback
	return fallback

static func to_float(v: Variant, fallback: float = 0.0) -> float:
	if v is float:
		return v
	if v is int:
		var n: int = v
		return float(n)
	if v is String:
		var s: String = v
		return s.to_float() if s.is_valid_float() else fallback
	return fallback

static func to_bool(v: Variant, fallback: bool = false) -> bool:
	if v is bool:
		return v
	if v is int or v is float:
		return to_float(v) != 0.0
	if v is String:
		var s: String = v
		return s.to_lower() in ["true", "1", "yes"]
	return fallback