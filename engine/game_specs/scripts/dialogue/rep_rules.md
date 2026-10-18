---
class: RepRules
extends: RefCounted
description: "Tactical RPG - Reputation Rules (S11, spec REP1-L) Reputation per faction is an int flag `rep.<faction qid with \":\" as \".\">` in MIN..MAX in the flat flag registry (no own GameState key, no save bump). Constants, the key / value rules, the world-level key lint (warnings) and the pure band reads (standing, band, buy / sell factors) that ShopRules and the panels use. No state, no RNG. No ripple through faction `relations` (S11 Q7 A)."
---

# Properties
* `PREFIX`: String = "rep." (const)
* `MIN`: int = -100 (const)
* `MAX`: int = 100 (const)
* `BAND_ORDER`: Array[String] = ["hostile", "unfriendly", "neutral", "friendly", "honoured"] (const)
* `BANDS`: Dictionary = { (const)
* `HOSTILE`: String = "hostile" (const)
* `NEUTRAL`: String = "neutral" (const)
* `out`: Array[String] = []
* `found`: Array[String] = []

# Methods

### `static key(faction_qid: String) -> String`
```logic
return PREFIX + faction_qid.replace(":", ".")
```

### `static is_rep_key(flag_key: String) -> bool`
```logic
return flag_key.begins_with(PREFIX)
```

### `static faction_qid_of(flag_key: String) -> String`
```logic
if not is_rep_key(flag_key):
	return ""
var rest: String = flag_key.trim_prefix(PREFIX)
var parts: PackedStringArray = rest.split(".")
if parts.size() != 2 or not JsonUtil.id_ok(parts[0]) or not JsonUtil.id_ok(parts[1]):
	return ""
return "%s:%s" % [parts[0], parts[1]]
```

### `static value_error(flag_key: String, value: Variant) -> String`
```logic
if not is_rep_key(flag_key):
	return ""
var v: Variant = JsonUtil.normalize(value)
if not (v is int) or DictUtil.to_int(v) < MIN or DictUtil.to_int(v) > MAX:
	return "Reputation '%s' must be a whole number from %d to %d." % [flag_key, MIN, MAX]
return ""
```

### `static key_resolves(world_id: String, flag_key: String) -> bool`
```logic
var fq: String = faction_qid_of(flag_key)
if fq.is_empty() or not WorldCatalog.ref_errors(world_id, "Reputation", "factions", fq).is_empty():
	return false
var hit: Dictionary = WorldCatalog.resolve(world_id, "factions", fq)
return not hit.is_empty()
```

### `static keys_in(raw: Variant, out: Array[String]) -> void`
```logic
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
```

### `static band(value: int) -> String`
```logic
for b: String in BAND_ORDER:
	var row: Dictionary = BANDS[b]
	if value <= DictUtil.to_int(row["hi"]):
		return b
return BAND_ORDER[BAND_ORDER.size() - 1]
```

### `static buy_pct(band_name: String) -> int`
```logic
return DictUtil.to_int(DictUtil.dict_of(BANDS.get(band_name, BANDS[NEUTRAL])).get("buy", 100))
```

### `static sell_pct(band_name: String) -> int`
```logic
return DictUtil.to_int(DictUtil.dict_of(BANDS.get(band_name, BANDS[NEUTRAL])).get("sell", 100))
```

### `static standing_text(faction_name: String, band_name: String) -> String`
```logic
return "%s: %s" % [faction_name, band_name]
```

### `static standing(gs: GameState, faction_ref: String) -> int`
```logic
if faction_ref.is_empty():
	return 0
var hit: Dictionary = WorldCatalog.resolve(gs.world_id, "factions", faction_ref)
if hit.is_empty():
	return 0
var v: Variant = gs.get_flag(key(str(hit.get("_qid", ""))), 0)
return DictUtil.to_int(v) if v is int else 0
```

