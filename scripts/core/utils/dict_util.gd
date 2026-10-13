class_name DictUtil
extends RefCounted

# Safe typed reads from Dictionaries whose values are Variants (any key type).

static func int_of(d: Dictionary, key: Variant, fallback: int = 0) -> int:
	var raw: Variant = d.get(key, fallback)
	return raw if raw is int else fallback

static func float_of(d: Dictionary, key: Variant, fallback: float = 0.0) -> float:
	var raw: Variant = d.get(key, fallback)
	return raw if raw is float else fallback