class_name JsonUtil
extends RefCounted

# ==============================================================================
# Tactical RPG - JSON helpers for the save model (S13)
# JSON-safety check, normalisation after a JSON round trip (whole floats back to ints)
# and deep equality that treats 2 and 2.0 as the same number.
# ==============================================================================

# Appends a message per value that cannot survive a JSON round trip (objects, non-String keys, NaN / inf).
static func check_safe(v: Variant, at: String, errors: Array[String]) -> void:
	match typeof(v):
		TYPE_NIL, TYPE_BOOL, TYPE_INT, TYPE_STRING:
			pass
		TYPE_FLOAT:
			var f: float = v
			if is_nan(f) or is_inf(f):
				errors.append("%s: number is not finite." % at)
		TYPE_ARRAY:
			var arr: Array = v
			for i: int in range(arr.size()):
				check_safe(arr[i], "%s[%d]" % [at, i], errors)
		TYPE_DICTIONARY:
			var d: Dictionary = v
			for key: Variant in d:
				if not (key is String):
					errors.append("%s: key '%s' is not a String." % [at, str(key)])
				else:
					check_safe(d[key], "%s.%s" % [at, str(key)], errors)
		_:
			errors.append("%s: value type %d is not JSON-safe." % [at, typeof(v)])

# Deep copy where whole-number floats become ints (JSON returns every number as a float).
static func normalize(v: Variant) -> Variant:
	if v is float:
		var f: float = v
		if is_finite(f) and f == floorf(f) and absf(f) < 9007199254740992.0:
			return int(f)
		return f
	if v is Array:
		var out_a: Array = []
		var arr: Array = v
		for x: Variant in arr:
			out_a.append(normalize(x))
		return out_a
	if v is Dictionary:
		var out_d: Dictionary = {}
		var d: Dictionary = v
		for key: Variant in d:
			out_d[key] = normalize(d[key])
		return out_d
	return v

static func deep_equal(a: Variant, b: Variant) -> bool:
	var a_num: bool = a is int or a is float
	var b_num: bool = b is int or b is float
	if a_num and b_num:
		return DictUtil.to_float(a) == DictUtil.to_float(b)
	if typeof(a) != typeof(b):
		return false
	if a is Array:
		var aa: Array = a
		var ba: Array = b
		if aa.size() != ba.size():
			return false
		for i: int in range(aa.size()):
			if not deep_equal(aa[i], ba[i]):
				return false
		return true
	if a is Dictionary:
		var ad: Dictionary = a
		var bd: Dictionary = b
		if ad.size() != bd.size():
			return false
		for key: Variant in ad:
			if not bd.has(key) or not deep_equal(ad[key], bd[key]):
				return false
		return true
	return a == b

static func strings(v: Variant) -> Array[String]:
	var out: Array[String] = []
	if v is Array:
		var arr: Array = v
		for x: Variant in arr:
			out.append(str(x))
	return out

static func ints(v: Variant) -> Dictionary:
	var out: Dictionary = {}
	if v is Dictionary:
		var d: Dictionary = v
		for key: Variant in d:
			out[str(key)] = DictUtil.to_int(d[key])
	return out

# Lowercase id charset [a-z0-9_], non-empty.
static func id_ok(id: String) -> bool:
	if id.is_empty():
		return false
	for i: int in range(id.length()):
		var c: String = id[i]
		if not ((c >= "a" and c <= "z") or (c >= "0" and c <= "9") or c == "_"):
			return false
	return true
