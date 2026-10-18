class_name PurseRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Encounter purse (PURSE1, spec docs/specs/PURSE1.md)
# An encounter may carry an authored `purse`: a gold range rolled on a win and, optionally, one world loot
# entry. Field on catalog and world encounters (json, default {} = no purse):
#   gold_min  int     0..GOLD_CAP                                       (default 0)
#   gold_max  int     gold_min..GOLD_CAP; above GOLD_SOFT_CAP warns     (default gold_min)
#   loot      String  a loot id of the encounter's world, or `world:id`; reward_kind item (default "")
#   once      bool    true = pays only the first win of the save        (default true)
#
# Static part (commit 1, schema slice): `read` (defaults filled), `problems` (both catalogs) and `warnings`.
# Runtime part (commit 2): roll / pay.
# ==============================================================================

const GOLD_CAP: int = 500
const GOLD_SOFT_CAP: int = 100
const KEYS: Array[String] = ["gold_min", "gold_max", "loot", "once"]

# The purse of an encounter with every default filled: {gold_min, gold_max, loot, once}; {} = no purse
# (missing, empty or not a dictionary). Values are read as validated (bad values are left to `problems`).
static func read(entry: Dictionary) -> Dictionary:
	var raw: Variant = entry.get("purse", {})
	if not (raw is Dictionary):
		return {}
	var p: Dictionary = raw
	if p.is_empty():
		return {}
	var lo: int = DictUtil.to_int(p.get("gold_min", 0))
	var once_raw: Variant = p.get("once", true)
	var once: bool = true
	if once_raw is bool:
		once = once_raw
	return {
		"gold_min": lo,
		"gold_max": DictUtil.to_int(p.get("gold_max", lo), lo),
		"loot": str(p.get("loot", "")),
		"once": once,
	}

# Blocking problems of an encounter's purse. `loot_check(ref) -> String` returns "" when the loot ref is fine,
# "missing" when it does not resolve, or "kind" when it resolves to a non-item reward.
static func problems(entry: Dictionary, loot_check: Callable) -> Array[String]:
	var errors: Array[String] = []
	if not entry.has("purse"):
		return errors
	var raw: Variant = entry["purse"]
	if not (raw is Dictionary):
		errors.append("Purse must be an object {gold_min, gold_max, loot, once}.")
		return errors
	var p: Dictionary = raw
	if p.is_empty():
		return errors
	for key: Variant in p:
		if not KEYS.has(str(key)):
			errors.append("Purse key '%s' is not allowed." % str(key))
	var lo_raw: Variant = JsonUtil.normalize(p.get("gold_min", 0))
	var lo_ok: bool = lo_raw is int and DictUtil.to_int(lo_raw) >= 0 and DictUtil.to_int(lo_raw) <= GOLD_CAP
	if not lo_ok:
		errors.append("Purse gold_min must be a whole number 0..%d." % GOLD_CAP)
	var lo: int = DictUtil.to_int(lo_raw) if lo_ok else 0
	if p.has("gold_max"):
		var hi_raw: Variant = JsonUtil.normalize(p["gold_max"])
		if not (hi_raw is int) or DictUtil.to_int(hi_raw) < lo or DictUtil.to_int(hi_raw) > GOLD_CAP:
			errors.append("Purse gold_max must be gold_min..%d." % GOLD_CAP)
	if p.has("loot"):
		var loot_raw: Variant = p["loot"]
		if not (loot_raw is String):
			errors.append("Purse loot '%s' does not resolve." % str(loot_raw))
		else:
			var ref: String = loot_raw
			if not ref.is_empty():
				match str(loot_check.call(ref)):
					"missing":
						errors.append("Purse loot '%s' does not resolve." % ref)
					"kind":
						errors.append("Purse loot '%s' must be an item reward." % ref)
	if p.has("once") and not (p["once"] is bool):
		errors.append("Purse once must be true or false.")
	return errors

# Soft problems of an encounter's purse (never blocking): gold above the soft cap, an empty purse.
static func warnings(entry: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var p: Dictionary = read(entry)
	if p.is_empty():
		return out
	var hi: int = DictUtil.to_int(p["gold_max"])
	if hi > GOLD_SOFT_CAP:
		out.append("Purse gold above %d: check the budget." % GOLD_SOFT_CAP)
	if hi <= 0 and str(p["loot"]).is_empty():
		out.append("Empty purse.")
	return out

# The shared loot check of `problems` for a world encounter: `ref` resolved from `world_id` (a qualified
# `world:id` resolves in that world). A catalog encounter has no world, so only qualified refs resolve there.
static func world_loot_check(world_id: String, ref: String) -> String:
	var hit: Dictionary = WorldCatalog.resolve(world_id, "loot", ref)
	if hit.is_empty():
		return "missing"
	if str(hit.get("reward_kind", "item")) != "item":
		return "kind"
	return ""
