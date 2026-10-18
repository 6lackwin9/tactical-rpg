---
class: ContentSchema
extends: RefCounted
description: "Tactical RPG - Content Schema (what each catalog entry looks like) One field list per catalog kind. The schema drives everything generic: normalising JSON (types, defaults, key order), field validation, and the Content Forge form (labels, widgets, ranges, help, conditional visibility). Field: {\"key\", \"label\", \"type\", \"default\", ...} type: id | string | text | int | float | bool | enum | ref | refs | stats | strings | json | multi_pick | template | table (ordered {key: int}, e.g. the class skill path) | dict_pick ({key: {..}} rows, e.g. class proficiency) | modifier_table / tier_table (item `modifiers` / `tiers` lists) | hand_grid (family `base` / `bands`, per hand); these three store like json (ITEM1 commit 3: typed Forge widgets, ItemFormModel / ItemTables) min / max / step     numeric range (table: the value range) min_len / max_len    string length caps (field_errors) group                Forge section of a class / unit field (identity, stats, proficiency, skills, loadout; units: ai) key_field            locked on system assets (ContentCatalog.key_field_errors) options              enum choices (Array[String]); \"options_from\": \"zones\" | \"art\" | \"status\" ref                  catalog kind a ref/refs field points at; \"ref_filter\": {key: value} when                 {\"key\": k, \"in\": [...]} or {\"key\": k, \"not_in\": [...]}: shown/saved only then keys                 stats: the allowed keys readonly             shown, never edited (e.g. the \"system\" flag) help                 tooltip text Enum values are stored by name (\"ATTACK\", \"MAIN_HAND\") so the JSON stays readable. item_families (FORGE1_ITEMS 1) is the item template kind; ContentCatalog loads it before items (items resolve against it). terrain (S12T-L): the numbers of each Terrain.Type (move cost, cover, hazard); TerrainFields / TerrainValidator."
---

# Properties
* `KINDS`: Array[String] = ["skills", "items", "item_families", "passives", "classes", "units", "encounters", "terrain"] (const)
* `ORIGIN_SYSTEM`: String = "system" (const)
* `ORIGIN_USER`: String = "user" (const)
* `STAT_KEYS`: Array[String] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"]   # = StatRules.STATS (const)
* `EQUIP_TYPES`: Array[String] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY"] (const)
* `LOADOUT_SLOTS`: Array[String] = ["main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2"] (const)
* `NO_ITEM`: String = "none"   # unit gear slot / pouch ["none"]: deliberately empty, no class fallback (BAL2-L 3.1) (const)
* `list`: Array[Dictionary] = []
* `origin`: Dictionary = origin_field()
* `out`: Array[Dictionary] = []
* `out`: Array[String] = []
* `raw`: Variant = f.get("options", [])
* `arr`: Array = raw
* `errors`: Array[String] = []
* `id`: String = str(entry.get("id", ""))
* `value`: String = str(entry.get(str(f["key"]), ""))

# Methods

### `static kind_label(kind: String) -> String`
```logic
match kind:
	"skills":
		return "Skills"
	"items":
		return "Items"
	"item_families":
		return "Item families"
	"passives":
		return "Passives"
	"classes":
		return "Classes"
	"units":
		return "Units"
	"encounters":
		return "Encounters"
	"terrain":
		return "Terrain"
return kind.capitalize()
```

### `static origin_field() -> Dictionary`
```logic
return _f("origin", "Origin", "enum", ORIGIN_SYSTEM, {"options": [ORIGIN_SYSTEM, ORIGIN_USER], "readonly": true, "key_field": true, "help": "system: shipped (key fields locked, delete only through SAFE DELETE). user: your own asset. Set by the Forge, never typed."})
```

### `static field(kind: String, key: String) -> Dictionary`
```logic
if kind == "skills":   # look up in the cache and copy only the one field (fields() deep-copies the whole list)
	if _skill_cache.is_empty():
		_skill_cache = SkillFields.build()
	for f: Dictionary in _skill_cache:
		if str(f.get("key", "")) == key:
			return f.duplicate(true)
	return {}
for f: Dictionary in fields(kind):
	if str(f.get("key", "")) == key:
		return f
return {}
```

### `static _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary`
```logic
var f: Dictionary = {"key": key, "label": label, "type": type, "default": default}
f.merge(extra, true)
return f
```

### `static is_visible(f: Dictionary, entry: Dictionary) -> bool`
```logic
var raw: Variant = f.get("when", null)
if not (raw is Dictionary):
	return true
var cond: Dictionary = raw
var value: String = str(entry.get(str(cond.get("key", "")), ""))
if cond.has("in"):
	var allowed: Array = cond["in"]
	return allowed.has(value)
if cond.has("not_in"):
	var denied: Array = cond["not_in"]
	return not denied.has(value)
return true
```

### `static passes_filter(f: Dictionary, target: Dictionary) -> bool`
```logic
var raw: Variant = f.get("ref_filter", null)
if not (raw is Dictionary):
	return true
var filter: Dictionary = raw
for key: Variant in filter:
	var want: Variant = filter[key]
	var have: Variant = target.get(key, null)
	if want is Array:
		var wants: Array = want
		if not wants.has(have):
			return false
	elif str(want) != str(have):
		return false
return true
```

### `static defaults(kind: String) -> Dictionary`
```logic
var d: Dictionary = {}
for f: Dictionary in fields(kind):
	d[str(f["key"])] = _copy(f["default"])
return d
```

### `static normalize(kind: String, entry: Dictionary) -> Dictionary`
```logic
var src: Dictionary = entry
if kind == "classes":
	src = _legacy_class(entry)
var out: Dictionary = {}
for f: Dictionary in fields(kind):
	var key: String = str(f["key"])
	out[key] = coerce(f, src.get(key, f["default"]))
for key: Variant in src:
	if not out.has(key):
		out[key] = _copy(src[key])
return out
```

### `static _legacy_class(entry: Dictionary) -> Dictionary`
```logic
var raw: Variant = entry.get("skill_path", null)
if not (raw is Array) or not ClassUnitFields.list_path_converter:
	return entry
var steps: Array = raw
var out: Dictionary = entry.duplicate()
out["skill_path"] = ClassArchetype.path_from_list(steps)
return out
```

### `static to_storage(kind: String, entry: Dictionary) -> Dictionary`
```logic
var full: Dictionary = normalize(kind, entry)
var out: Dictionary = {}
for f: Dictionary in fields(kind):
	var key: String = str(f["key"])
	if not is_visible(f, full) or DictUtil.to_bool(f.get("internal", false)):
		continue
	if key == "system" and not DictUtil.to_bool(full[key]):
		continue
	if key == "origin" and str(full[key]) == ORIGIN_SYSTEM:
		continue
	out[key] = full[key]
for key: Variant in full:
	if not out.has(key) and field(kind, str(key)).is_empty():
		out[key] = full[key]
return out
```

### `static coerce(f: Dictionary, value: Variant) -> Variant`
```logic
var fallback: Variant = f.get("default", null)
match str(f.get("type", "string")):
	"int":
		var n: int = DictUtil.to_int(value, DictUtil.to_int(fallback))
		if f.has("min"):
			n = maxi(n, DictUtil.to_int(f["min"]))
		if f.has("max"):
			n = mini(n, DictUtil.to_int(f["max"]))
		return n
	"float":
		var x: float = DictUtil.to_float(value, DictUtil.to_float(fallback))
		if f.has("min"):
			x = maxf(x, DictUtil.to_float(f["min"]))
		if f.has("max"):
			x = minf(x, DictUtil.to_float(f["max"]))
		return snappedf(x, 0.0001)
	"bool":
		return DictUtil.to_bool(value, DictUtil.to_bool(fallback))
	"refs", "strings", "multi_pick":
		var list: Array[String] = []
		if value is Array:
			var arr: Array = value
			for v: Variant in arr:
				if not str(v).is_empty():
					list.append(str(v))
		return list
	"stats":
		var stats: Dictionary = {}
		if value is Dictionary:
			var src: Dictionary = value
			for key: Variant in src:
				var raw: Variant = src[key]
				if raw is int or raw is float:
					var amount: int = DictUtil.to_int(raw)
					if amount != 0:
						stats[str(key)] = amount
		return stats
	"json", "modifier_table", "tier_table", "hand_grid":
		return _copy(value) if (value is Array or value is Dictionary) else _copy(fallback)
	"table":
		# Ordered {String key: level}; insertion order is kept. Values stay unclamped (the validator names a bad level);
		# a non-dictionary value is kept as is so the validator reports it and saving never drops it.
		if value is Dictionary:
			var src_table: Dictionary = value
			var table: Dictionary = {}
			for key: Variant in src_table:
				table[str(key)] = ClassArchetype.whole_level(src_table[key])
			return table
		return _copy(value) if value is Array else _copy(fallback)
	"dict_pick":
		return _copy(value) if value is Dictionary else _copy(fallback)
return str(value) if value != null else str(fallback)
```

### `static _copy(v: Variant) -> Variant`
```logic
if v is Array:
	var arr: Array = v
	return _json_ints(arr.duplicate(true))
if v is Dictionary:
	var dict: Dictionary = v
	return _json_ints(dict.duplicate(true))
return v
```

### `static _json_ints(v: Variant) -> Variant`
```logic
if v is float:
	var x: float = v
	return roundi(x) if x == floorf(x) else x
if v is Array:
	var arr: Array = v
	for i: int in range(arr.size()):
		arr[i] = _json_ints(arr[i])
	return arr
if v is Dictionary:
	var dict: Dictionary = v
	for key: Variant in dict:
		dict[key] = _json_ints(dict[key])
	return dict
return v
```

### `static is_valid_id(id: String) -> bool`
```logic
var re := RegEx.new()
re.compile("^[a-z][a-z0-9_]*$")
return re.search(id) != null
```

