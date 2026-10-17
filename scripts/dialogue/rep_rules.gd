class_name RepRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Reputation Rules (S11, spec REP1-L)
# Reputation per faction is an int flag `rep.<faction qid with ":" as ".">` in MIN..MAX in the flat flag registry
# (no own GameState key, no save bump). Constants, the key / value rules, the world-level key lint (warnings) and the
# pure band reads (standing, band, buy / sell factors) that ShopRules and the panels use. No state, no RNG.
# No ripple through faction `relations` (S11 Q7 A).
# ==============================================================================

const PREFIX: String = "rep."
const MIN: int = -100
const MAX: int = 100

# Band -> {lo, hi, buy, sell} (inclusive range; buy / sell factors in percent; a hostile shop is closed, factors 0).
# Starting values, tuned after the balance pass (REP1-L 2.3).
const BAND_ORDER: Array[String] = ["hostile", "unfriendly", "neutral", "friendly", "honoured"]
const BANDS: Dictionary = {
	"hostile": {"lo": MIN, "hi": -50, "buy": 0, "sell": 0},
	"unfriendly": {"lo": -49, "hi": -10, "buy": 115, "sell": 85},
	"neutral": {"lo": -9, "hi": 9, "buy": 100, "sell": 100},
	"friendly": {"lo": 10, "hi": 49, "buy": 95, "sell": 105},
	"honoured": {"lo": 50, "hi": MAX, "buy": 90, "sell": 110},
}
const HOSTILE: String = "hostile"
const NEUTRAL: String = "neutral"

# ------------------------------------------------------------------------------
# Keys and values
# ------------------------------------------------------------------------------

# The flag key of a faction qid (`ashlands:ember_wardens` -> `rep.ashlands.ember_wardens`).
static func key(faction_qid: String) -> String:
	return PREFIX + faction_qid.replace(":", ".")

static func is_rep_key(flag_key: String) -> bool:
	return flag_key.begins_with(PREFIX)

# The faction qid a rep key names (`rep.a.b` -> `a:b`), "" when the key is not of that shape.
static func faction_qid_of(flag_key: String) -> String:
	if not is_rep_key(flag_key):
		return ""
	var rest: String = flag_key.trim_prefix(PREFIX)
	var parts: PackedStringArray = rest.split(".")
	if parts.size() != 2 or not JsonUtil.id_ok(parts[0]) or not JsonUtil.id_ok(parts[1]):
		return ""
	return "%s:%s" % [parts[0], parts[1]]

# "" when `value` may be stored under (or compared with) `flag_key`; for rep keys it must be an int in MIN..MAX.
static func value_error(flag_key: String, value: Variant) -> String:
	if not is_rep_key(flag_key):
		return ""
	var v: Variant = JsonUtil.normalize(value)
	if not (v is int) or DictUtil.to_int(v) < MIN or DictUtil.to_int(v) > MAX:
		return "Reputation '%s' must be a whole number from %d to %d." % [flag_key, MIN, MAX]
	return ""

# True when the rep key names a faction resolvable from `world_id` (by its resolved qid).
static func key_resolves(world_id: String, flag_key: String) -> bool:
	var fq: String = faction_qid_of(flag_key)
	if fq.is_empty() or not WorldCatalog.ref_errors(world_id, "Reputation", "factions", fq).is_empty():
		return false
	var hit: Dictionary = WorldCatalog.resolve(world_id, "factions", fq)
	return not hit.is_empty()

# Every rep flag key named in a `key` / `flag` field of `raw` (any depth), in walk order, without duplicates.
static func keys_in(raw: Variant, out: Array[String]) -> void:
	if raw is Dictionary:
		var d: Dictionary = raw
		for k: Variant in d:
			var v: Variant = d[k]
			if (str(k) == "key" or str(k) == "flag") and v is String and is_rep_key(str(v)) and not out.has(str(v)):
				out.append(str(v))
			else:
				keys_in(v, out)
	elif raw is Array:
		for v: Variant in DictUtil.array_of(raw):
			keys_in(v, out)

# Soft lint over the world's OWN entries (prefixed with the entry qid): a rep key that names no faction of the world
# (a renamed or removed faction reports here instead of blocking).
static func key_warnings(world_id: String) -> Array[String]:
	var out: Array[String] = []
	for kind: String in WorldSchema.KINDS:
		for e: Dictionary in WorldCatalog.own_entries(world_id, kind):
			var found: Array[String] = []
			keys_in(e, found)
			for k: String in found:
				if not key_resolves(world_id, k):
					out.append("%s: Reputation key '%s' names no faction of this world." % [str(e.get("_qid", "")), k])
	return out

# ------------------------------------------------------------------------------
# Bands (display and prices)
# ------------------------------------------------------------------------------

static func band(value: int) -> String:
	for b: String in BAND_ORDER:
		var row: Dictionary = BANDS[b]
		if value <= DictUtil.to_int(row["hi"]):
			return b
	return BAND_ORDER[BAND_ORDER.size() - 1]

static func buy_pct(band_name: String) -> int:
	return DictUtil.to_int(DictUtil.dict_of(BANDS.get(band_name, BANDS[NEUTRAL])).get("buy", 100))

static func sell_pct(band_name: String) -> int:
	return DictUtil.to_int(DictUtil.dict_of(BANDS.get(band_name, BANDS[NEUTRAL])).get("sell", 100))

# The party's standing with a faction (ref resolved from gs.world_id): the int flag, 0 when missing, non-int or the
# faction is unknown.
static func standing(gs: GameState, faction_ref: String) -> int:
	if faction_ref.is_empty():
		return 0
	var hit: Dictionary = WorldCatalog.resolve(gs.world_id, "factions", faction_ref)
	if hit.is_empty():
		return 0
	var v: Variant = gs.get_flag(key(str(hit.get("_qid", ""))), 0)
	return DictUtil.to_int(v) if v is int else 0
