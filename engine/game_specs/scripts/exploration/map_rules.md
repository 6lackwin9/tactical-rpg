---
class: MapRules
extends: RefCounted
description: "Tactical RPG - Map Rules (S9, spec EXP1-L section 1) Pure rules over lore-free map templates (MapCatalog) and their world bindings (location fields `map`, `zone_bindings`, `markers`): template validation, binding and marker validation, soft warnings, tile/zone/anchor lookups, roamer path index and the runtime system flag names. No state; every call reads the catalogs. Coordinates: x = column, y = row, (0, 0) top-left."
---

# Properties
* `TILES`: Dictionary = {".": "open", "#": "wall", "~": "water"} (const)
* `OPEN_TILE`: String = "." (const)
* `NO_ZONE`: String = "-" (const)
* `ZONE_KINDS`: Array[String] = ["safe", "danger"] (const)
* `SPAWN`: String = "spawn" (const)
* `WORLD_MAP`: String = "world_map" (const)
* `MIN_SIDE`: int = 8 (const)
* `MAX_SIDE`: int = 64 (const)
* `MAX_ANCHORS`: int = 64 (const)
* `MAX_PATHS`: int = 16 (const)
* `MAX_PATH`: int = 32 (const)
* `MAX_MARKERS`: int = 64 (const)
* `MAX_RADIUS`: int = 3 (const)
* `MAX_BIND_LIST`: int = 8 (const)
* `WEIGHT_MIN`: int = 1 (const)
* `WEIGHT_MAX`: int = 100 (const)
* `STEPS_LO`: int = 1 (const)
* `STEPS_HI`: int = 99 (const)
* `DEFAULT_ELITE_PCT`: int = 10 (const)
* `DEFAULT_STEPS_MIN`: int = 12 (const)
* `DEFAULT_STEPS_MAX`: int = 20 (const)
* `MAX_CHEST_KEYS`: int = 8 (const)
* `CHEST_COUNT_MAX`: int = 99 (const)
* `CHEST_GOLD_MAX`: int = 9999   # DLG1-L (S11) (const)
* `CHEST_SUPPLIES_MAX`: int = 99   # TRAV2-L (S10) (const)
* `TEMPLATE_FIELDS`: Array[String] = ["id", "name", "rows", "zones", "zone_legend", "anchors", "paths", "tags"] (const)
* `BINDING_KEYS`: Array[String] = ["encounters", "elites", "elite_pct", "steps_min", "steps_max"] (const)
* `MARKER_COMMON`: Array[String] = ["id", "kind", "anchor", "requires_flags", "unless_flags", "sets_flags", "label", "note"] (const)
* `LABEL_MAX`: int = 32   # MAPREAD1-L1 (const)
* `NOTE_MAX`: int = 160 (const)
* `FLAG_KEYS`: Array[String] = ["requires_flags", "unless_flags", "sets_flags"] (const)
* `TRIGGERS`: Array[String] = ["touch", "proximity"] (const)
* `MARKER_KINDS`: Dictionary = { (const)
* `SOLID_KINDS`: Array[String] = ["npc", "chest", "lore"] (const)
* `out`: Array[String] = []
* `out`: Array[String] = []
* `lg`: Dictionary = legend(t)
* `zid`: String = str(DictUtil.dict_of(lg[key]).get("id", ""))
* `out`: Array[String] = []
* `out`: Array[String] = []
* `out`: Array[Vector2i] = []
* `pair`: Array = DictUtil.array_of(p)
* `t`: Dictionary = normalize_template(entry)
* `errors`: Array[String] = []
* `id`: String = str(t["id"])
* `grid_ok`: bool = _rows_errors(t, errors)
* `t`: Dictionary = normalize_template(entry)
* `out`: Array[String] = []
* `used`: Dictionary = {}
* `s`: String = str(row)
* `lg`: Dictionary = legend(t)
* `errors`: Array[String] = []
* `map_id`: String = str(e.get("map", ""))
* `bindings_raw`: Variant = e.get("zone_bindings", {})
* `markers_raw`: Variant = e.get("markers", [])
* `types_ok`: bool = true
* `t`: Dictionary = MapCatalog.get_entry(map_id)
* `bindings`: Dictionary = bindings_raw
* `markers`: Array = markers_raw
* `errors`: Array[String] = []
* `list`: Array = raw
* `ok`: bool = list.size() == 2
* `errors`: Array[String] = []
* `id`: String = str(e.get("id", ""))
* `m`: Dictionary = DictUtil.dict_of(x)
* `errors`: Array[String] = []
* `kind`: String = zone_kind(t, zone)
* `b`: Dictionary = binding_full(DictUtil.dict_of(raw))
* `steps_ok`: bool = true
* `errors`: Array[String] = []
* `list`: Array = raw
* `seen`: Dictionary = {}
* `pair`: Array = DictUtil.array_of(x)
* `ref`: String = str(pair[0])
* `out`: Array[String] = []
* `to`: String = str(DictUtil.dict_of(x).get("to", ""))
* `errors`: Array[String] = []
* `ids`: Dictionary = {}
* `anchors_taken`: Dictionary = {}
* `raw`: Dictionary = DictUtil.dict_of(JsonUtil.normalize(list[i]))
* `mid`: String = str(raw.get("id", ""))
* `kind`: String = str(raw.get("kind", ""))
* `own_keys`: Dictionary = DictUtil.dict_of(MARKER_KINDS[kind])
* `k`: String = str(key)
* `m`: Dictionary = marker_full(raw)
* `anchor`: String = str(m["anchor"])
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `to`: String = str(m["to"])
* `to_anchor`: String = str(m["to_anchor"])
* `target_map`: Dictionary = {}
* `target`: Dictionary = WorldCatalog.resolve(world_id, "locations", to)
* `target_map_id`: String = str(target.get("map", ""))
* `errors`: Array[String] = []
* `ref`: String = str(m[key])
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `items_raw`: Variant = m["items"]
* `items`: Dictionary = DictUtil.dict_of(items_raw)
* `loot`: String = str(m["loot"])
* `ref_errs`: Array[String] = WorldCatalog.ref_errors(world_id, "Marker '%s' loot" % mid, "loot", loot)
* `gold`: Variant = m["gold"]
* `supplies`: Variant = m.get("supplies", 0)
* `out`: Array[String] = []
* `t`: Dictionary = MapCatalog.get_entry(str(e.get("map", "")))
* `bindings`: Dictionary = DictUtil.dict_of(e.get("zone_bindings", {}))
* `marked`: Dictionary = {}
* `m`: Dictionary = DictUtil.dict_of(x)
* `out`: Array[String] = []

# Methods

### `static normalize_template(entry: Dictionary) -> Dictionary`
```logic
var e: Dictionary = DictUtil.dict_of(JsonUtil.normalize(entry))
var out: Dictionary = {
	"id": str(e.get("id", "")), "name": e.get("name", ""), "rows": e.get("rows", []), "zones": e.get("zones", []),
	"zone_legend": e.get("zone_legend", {}), "anchors": e.get("anchors", []), "paths": e.get("paths", {}), "tags": e.get("tags", []),
}
for key: Variant in e:
	if not out.has(key) and not str(key).begins_with("_"):
		out[key] = e[key]
return out
```

### `static size_of(t: Dictionary) -> Vector2i`
```logic
var rows: Array[String] = rows_of(t)
return Vector2i(rows[0].length() if not rows.is_empty() else 0, rows.size())
```

### `static tile(t: Dictionary, x: int, y: int) -> String`
```logic
var rows: Array[String] = rows_of(t)
if y < 0 or y >= rows.size() or x < 0 or x >= rows[y].length():
	return ""
return rows[y][x]
```

### `static is_open(t: Dictionary, x: int, y: int) -> bool`
```logic
return tile(t, x, y) == OPEN_TILE
```

### `static legend(t: Dictionary) -> Dictionary`
```logic
var out: Dictionary = {}
var raw: Dictionary = DictUtil.dict_of(t.get("zone_legend", {}))
for key: Variant in raw:
	if raw[key] is Dictionary:
		out[str(key)] = raw[key]
return out
```

### `static zone_kind(t: Dictionary, zone_id: String) -> String`
```logic
var lg: Dictionary = legend(t)
for key: Variant in lg:
	var z: Dictionary = DictUtil.dict_of(lg[key])
	if str(z.get("id", "")) == zone_id:
		return str(z.get("kind", ""))
return ""
```

### `static zone_at(t: Dictionary, x: int, y: int) -> String`
```logic
var zones: Array = DictUtil.array_of(t.get("zones", []))
if y < 0 or y >= zones.size():
	return ""
var row: String = str(zones[y])
if x < 0 or x >= row.length() or row[x] == NO_ZONE:
	return ""
return str(DictUtil.dict_of(legend(t).get(row[x], {})).get("id", ""))
```

### `static has_anchor(t: Dictionary, anchor_id: String) -> bool`
```logic
return not anchor_id.is_empty() and anchor_ids(t).has(anchor_id)
```

### `static anchor_pos(t: Dictionary, anchor_id: String) -> Vector2i`
```logic
for a: Variant in DictUtil.array_of(t.get("anchors", [])):
	var d: Dictionary = DictUtil.dict_of(a)
	if str(d.get("id", "")) == anchor_id:
		return Vector2i(DictUtil.to_int(d.get("x", -1), -1), DictUtil.to_int(d.get("y", -1), -1))
return Vector2i(-1, -1)
```

### `static has_path(t: Dictionary, path_id: String) -> bool`
```logic
return not path_id.is_empty() and DictUtil.dict_of(t.get("paths", {})).has(path_id)
```

### `static roamer_index(length: int, loc_steps: int) -> int`
```logic
if length <= 1:
	return 0
var period: int = 2 * length - 2
var t: int = posmod(loc_steps, period)
return t if t < length else period - t
```

### `static _all_strings(list: Array) -> bool`
```logic
for x: Variant in list:
	if not (x is String):
		return false
return true
```

### `static _rows_errors(t: Dictionary, errors: Array[String]) -> bool`
```logic
var raw: Variant = t["rows"]
var shape_ok: bool = raw is Array and _all_strings(DictUtil.array_of(raw))
var rows: Array[String] = rows_of(t) if shape_ok else []
if shape_ok and (rows.size() < MIN_SIDE or rows.size() > MAX_SIDE):
	shape_ok = false
if shape_ok:
	var w: int = rows[0].length()
	for r: String in rows:
		if r.length() != w or r.length() < MIN_SIDE or r.length() > MAX_SIDE:
			shape_ok = false
			break
if not shape_ok:
	errors.append("Rows: %d..%d rows of %d..%d tiles, all the same length." % [MIN_SIDE, MAX_SIDE, MIN_SIDE, MAX_SIDE])
for y: int in range(rows.size()):
	for x: int in range(rows[y].length()):
		if not TILES.has(rows[y][x]):
			errors.append("Row %d: unknown tile '%s' (use . # ~)." % [y, rows[y][x]])
			break
return shape_ok
```

### `static _zone_errors(t: Dictionary, grid_ok: bool, errors: Array[String]) -> void`
```logic
var lg_raw: Variant = t["zone_legend"]
var known_chars: Dictionary = {}
if not (lg_raw is Dictionary):
	errors.append("Zone legend must be an object {char: {\"id\", \"kind\"}}.")
else:
	var lg: Dictionary = lg_raw
	var seen_ids: Dictionary = {}
	for key: Variant in lg:
		var k: String = str(key)
		if k.length() != 1 or k == NO_ZONE:
			errors.append("Zone legend keys are single characters other than '%s'." % NO_ZONE)
			continue
		known_chars[k] = true
		var z: Dictionary = DictUtil.dict_of(lg[key])
		if not (lg[key] is Dictionary) or z.size() != 2 or not z.has("id") or not z.has("kind"):
			errors.append("Zone legend '%s' must be exactly {\"id\", \"kind\"}." % k)
			continue
		var zid: String = str(z["id"])
		if not (z["id"] is String) or not JsonUtil.id_ok(zid):
			errors.append("Zone legend '%s': id '%s' must be lowercase [a-z0-9_]." % [k, zid])
		elif seen_ids.has(zid):
			errors.append("Zone id '%s' is used twice." % zid)
		seen_ids[zid] = true
		if not ZONE_KINDS.has(str(z["kind"])):
			errors.append("Zone '%s': kind must be safe or danger." % zid)
var zones_raw: Variant = t["zones"]
if not (zones_raw is Array) or not _all_strings(DictUtil.array_of(zones_raw)):
	errors.append("Zones must be a list of strings.")
	return
var zones: Array = zones_raw
if zones.is_empty() or not grid_ok:
	return
var size: Vector2i = size_of(t)
var fits: bool = zones.size() == size.y
for row: Variant in zones:
	fits = fits and str(row).length() == size.x
if not fits:
	errors.append("Zones must match the rows (%d x %d)." % [size.x, size.y])
	return
var reported: Dictionary = {}
for y: int in range(zones.size()):
	var row_s: String = str(zones[y])
	for x: int in range(row_s.length()):
		var c: String = row_s[x]
		if c != NO_ZONE and not known_chars.has(c) and not reported.has(c):
			reported[c] = true
			errors.append("Zone char '%s' at (%d, %d) is not in the legend." % [c, x, y])
```

### `static _anchor_errors(t: Dictionary, grid_ok: bool, errors: Array[String]) -> void`
```logic
var raw: Variant = t["anchors"]
if not (raw is Array):
	errors.append("Anchors must be a list of {\"id\", \"x\", \"y\"}.")
	return
var list: Array = raw
if list.size() > MAX_ANCHORS:
	errors.append("At most %d anchors." % MAX_ANCHORS)
var seen: Dictionary = {}
for i: int in range(list.size()):
	var a: Dictionary = DictUtil.dict_of(list[i])
	if not (list[i] is Dictionary) or a.size() != 3 or not a.has("id") or not a.has("x") or not a.has("y"):
		errors.append("Anchor %d must be exactly {\"id\", \"x\", \"y\"}." % i)
		continue
	var aid: String = str(a["id"])
	if not (a["id"] is String) or not JsonUtil.id_ok(aid):
		errors.append("Anchor %d: id '%s' must be lowercase [a-z0-9_]." % [i, aid])
		continue
	if seen.has(aid):
		errors.append("Anchor '%s' is used twice." % aid)
	seen[aid] = true
	if not (a["x"] is int) or not (a["y"] is int):
		errors.append("Anchor '%s': x and y must be whole numbers." % aid)
	elif grid_ok and not is_open(t, DictUtil.to_int(a["x"]), DictUtil.to_int(a["y"])):
		errors.append("Anchor '%s' is not on an open tile." % aid)
if not seen.has(SPAWN):
	errors.append("Every map needs a '%s' anchor." % SPAWN)
```

### `static _path_errors(t: Dictionary, grid_ok: bool, errors: Array[String]) -> void`
```logic
var raw: Variant = t["paths"]
if not (raw is Dictionary):
	errors.append("Paths must be an object {path id: [[x, y], ...]}.")
	return
var paths: Dictionary = raw
if paths.size() > MAX_PATHS:
	errors.append("At most %d paths." % MAX_PATHS)
for key: Variant in paths:
	var pid: String = str(key)
	if not JsonUtil.id_ok(pid):
		errors.append("Path id '%s' must be lowercase [a-z0-9_]." % pid)
	var pts: Array = DictUtil.array_of(paths[key])
	if not (paths[key] is Array) or pts.is_empty() or pts.size() > MAX_PATH:
		errors.append("Path '%s' needs 1..%d points [x, y]." % [pid, MAX_PATH])
		continue
	var prev: Vector2i = Vector2i(-1, -1)
	for i: int in range(pts.size()):
		var pair: Array = DictUtil.array_of(pts[i])
		if pair.size() != 2 or not (pair[0] is int) or not (pair[1] is int):
			errors.append("Path '%s' step %d must be [x, y]." % [pid, i])
			break
		var p: Vector2i = Vector2i(DictUtil.to_int(pair[0]), DictUtil.to_int(pair[1]))
		if grid_ok and not is_open(t, p.x, p.y):
			errors.append("Path '%s' step %d is not on an open tile." % [pid, i])
		if i > 0 and absi(p.x - prev.x) + absi(p.y - prev.y) != 1:
			errors.append("Path '%s' step %d is not next to the step before." % [pid, i])
		prev = p
```

### `static marker_full(m: Dictionary) -> Dictionary`
```logic
var out: Dictionary = {"id": "", "kind": "", "requires_flags": [], "unless_flags": [], "sets_flags": [], "label": "", "note": ""}
var kind: String = str(m.get("kind", ""))
if kind != "roamer":
	out["anchor"] = ""
out.merge(DictUtil.dict_of(MARKER_KINDS.get(kind, {})).duplicate(true), true)
out.merge(DictUtil.dict_of(JsonUtil.normalize(m)), true)
return out
```

### `static binding_full(b: Dictionary) -> Dictionary`
```logic
var out: Dictionary = {"encounters": [], "elites": [], "elite_pct": DEFAULT_ELITE_PCT, "steps_min": DEFAULT_STEPS_MIN, "steps_max": DEFAULT_STEPS_MAX}
out.merge(DictUtil.dict_of(JsonUtil.normalize(b)), true)
return out
```

### `static start_location(world_id: String) -> String`
```logic
var chain: Array[String] = WorldCatalog.lineage(world_id)
chain.reverse()
for w: String in chain:
	var ref: String = str(WorldCatalog.manifest(w).get("start_location", ""))
	if ref.is_empty():
		continue
	var hit: Dictionary = WorldCatalog.resolve(w, "locations", ref)
	if hit.is_empty():
		return ""
	return str(WorldCatalog.resolve(world_id, "locations", str(hit["_qid"])).get("_qid", hit["_qid"]))
return ""
```

### `static dotted(location_qid: String) -> String`
```logic
return location_qid.replace(":", ".")
```

### `static done_flag(location_qid: String, marker_id: String) -> String`
```logic
return "%s.%s.done" % [dotted(location_qid), marker_id]
```

### `static visited_flag(location_qid: String) -> String`
```logic
return "%s.visited" % dotted(location_qid)
```

