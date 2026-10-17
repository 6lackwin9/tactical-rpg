class_name MapRules
extends RefCounted

# ==============================================================================
# Tactical RPG - Map Rules (S9, spec EXP1-L section 1)
# Pure rules over lore-free map templates (MapCatalog) and their world bindings (location fields `map`,
# `zone_bindings`, `markers`): template validation, binding and marker validation, soft warnings, tile/zone/anchor
# lookups, roamer path index and the runtime system flag names. No state; every call reads the catalogs.
# Coordinates: x = column, y = row, (0, 0) top-left.
# ==============================================================================

const TILES: Dictionary = {".": "open", "#": "wall", "~": "water"}
const OPEN_TILE: String = "."
const NO_ZONE: String = "-"
const ZONE_KINDS: Array[String] = ["safe", "danger"]
const SPAWN: String = "spawn"
const WORLD_MAP: String = "world_map"

const MIN_SIDE: int = 8
const MAX_SIDE: int = 64
const MAX_ANCHORS: int = 64
const MAX_PATHS: int = 16
const MAX_PATH: int = 32
const MAX_MARKERS: int = 64
const MAX_RADIUS: int = 3
const MAX_BIND_LIST: int = 8
const WEIGHT_MIN: int = 1
const WEIGHT_MAX: int = 100
const STEPS_LO: int = 1
const STEPS_HI: int = 99
const DEFAULT_ELITE_PCT: int = 10
const DEFAULT_STEPS_MIN: int = 12
const DEFAULT_STEPS_MAX: int = 20
const MAX_CHEST_KEYS: int = 8
const CHEST_COUNT_MAX: int = 99
const CHEST_GOLD_MAX: int = 9999   # DLG1-L (S11)
const CHEST_SUPPLIES_MAX: int = 99   # TRAV2-L (S10)

const TEMPLATE_FIELDS: Array[String] = ["id", "name", "rows", "zones", "zone_legend", "anchors", "paths", "tags"]
const BINDING_KEYS: Array[String] = ["encounters", "elites", "elite_pct", "steps_min", "steps_max"]
const MARKER_COMMON: Array[String] = ["id", "kind", "anchor", "requires_flags", "unless_flags", "sets_flags"]
const FLAG_KEYS: Array[String] = ["requires_flags", "unless_flags", "sets_flags"]
const TRIGGERS: Array[String] = ["touch", "proximity"]
# Kind -> its own keys with defaults (the common keys are MARKER_COMMON; a roamer takes no anchor).
const MARKER_KINDS: Dictionary = {
	"exit": {"to": "", "to_anchor": "spawn", "locked_text": "The way is shut."},
	"encounter": {"encounter": "", "trigger": "touch", "radius": 1, "once": true},
	"roamer": {"encounter": "", "path": "", "radius": 0, "once": true},
	"npc": {"character": ""},
	"chest": {"items": {}, "loot": "", "gold": 0, "supplies": 0},
	"lore": {"lore": ""},
	"point": {},
}
# Always solid when present (an exit is solid only while locked, see EXP1-L 2.1).
const SOLID_KINDS: Array[String] = ["npc", "chest", "lore"]

# ------------------------------------------------------------------------------
# Template shape
# ------------------------------------------------------------------------------

# Defaults filled, JSON numbers normalised; unknown keys are kept for validation.
static func normalize_template(entry: Dictionary) -> Dictionary:
	var e: Dictionary = DictUtil.dict_of(JsonUtil.normalize(entry))
	var out: Dictionary = {
		"id": str(e.get("id", "")), "name": e.get("name", ""), "rows": e.get("rows", []), "zones": e.get("zones", []),
		"zone_legend": e.get("zone_legend", {}), "anchors": e.get("anchors", []), "paths": e.get("paths", {}), "tags": e.get("tags", []),
	}
	for key: Variant in e:
		if not out.has(key) and not str(key).begins_with("_"):
			out[key] = e[key]
	return out

static func rows_of(t: Dictionary) -> Array[String]:
	var out: Array[String] = []
	for r: Variant in DictUtil.array_of(t.get("rows", [])):
		out.append(str(r))
	return out

static func size_of(t: Dictionary) -> Vector2i:
	var rows: Array[String] = rows_of(t)
	return Vector2i(rows[0].length() if not rows.is_empty() else 0, rows.size())

# The tile char at (x, y), "" off the map.
static func tile(t: Dictionary, x: int, y: int) -> String:
	var rows: Array[String] = rows_of(t)
	if y < 0 or y >= rows.size() or x < 0 or x >= rows[y].length():
		return ""
	return rows[y][x]

static func is_open(t: Dictionary, x: int, y: int) -> bool:
	return tile(t, x, y) == OPEN_TILE

# Legend char -> {"id", "kind"} for the well-formed legend entries.
static func legend(t: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	var raw: Dictionary = DictUtil.dict_of(t.get("zone_legend", {}))
	for key: Variant in raw:
		if raw[key] is Dictionary:
			out[str(key)] = raw[key]
	return out

static func zone_ids(t: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var lg: Dictionary = legend(t)
	for key: Variant in lg:
		var zid: String = str(DictUtil.dict_of(lg[key]).get("id", ""))
		if not zid.is_empty() and not out.has(zid):
			out.append(zid)
	return out

# "safe", "danger", or "" when the template has no such zone.
static func zone_kind(t: Dictionary, zone_id: String) -> String:
	var lg: Dictionary = legend(t)
	for key: Variant in lg:
		var z: Dictionary = DictUtil.dict_of(lg[key])
		if str(z.get("id", "")) == zone_id:
			return str(z.get("kind", ""))
	return ""

static func danger_zone_ids(t: Dictionary) -> Array[String]:
	var out: Array[String] = []
	for zid: String in zone_ids(t):
		if zone_kind(t, zid) == "danger":
			out.append(zid)
	return out

# The zone id under (x, y); "" = no zone (SAFE).
static func zone_at(t: Dictionary, x: int, y: int) -> String:
	var zones: Array = DictUtil.array_of(t.get("zones", []))
	if y < 0 or y >= zones.size():
		return ""
	var row: String = str(zones[y])
	if x < 0 or x >= row.length() or row[x] == NO_ZONE:
		return ""
	return str(DictUtil.dict_of(legend(t).get(row[x], {})).get("id", ""))

static func anchor_ids(t: Dictionary) -> Array[String]:
	var out: Array[String] = []
	for a: Variant in DictUtil.array_of(t.get("anchors", [])):
		out.append(str(DictUtil.dict_of(a).get("id", "")))
	return out

static func has_anchor(t: Dictionary, anchor_id: String) -> bool:
	return not anchor_id.is_empty() and anchor_ids(t).has(anchor_id)

# The anchor tile, (-1, -1) when the template has no such anchor.
static func anchor_pos(t: Dictionary, anchor_id: String) -> Vector2i:
	for a: Variant in DictUtil.array_of(t.get("anchors", [])):
		var d: Dictionary = DictUtil.dict_of(a)
		if str(d.get("id", "")) == anchor_id:
			return Vector2i(DictUtil.to_int(d.get("x", -1), -1), DictUtil.to_int(d.get("y", -1), -1))
	return Vector2i(-1, -1)

static func has_path(t: Dictionary, path_id: String) -> bool:
	return not path_id.is_empty() and DictUtil.dict_of(t.get("paths", {})).has(path_id)

static func path_points(t: Dictionary, path_id: String) -> Array[Vector2i]:
	var out: Array[Vector2i] = []
	for p: Variant in DictUtil.array_of(DictUtil.dict_of(t.get("paths", {})).get(path_id, [])):
		var pair: Array = DictUtil.array_of(p)
		if pair.size() == 2:
			out.append(Vector2i(DictUtil.to_int(pair[0]), DictUtil.to_int(pair[1])))
	return out

# Roamer point index for a path of `length` points after `loc_steps` steps (ping-pong, EXP1-L 2.3).
static func roamer_index(length: int, loc_steps: int) -> int:
	if length <= 1:
		return 0
	var period: int = 2 * length - 2
	var t: int = posmod(loc_steps, period)
	return t if t < length else period - t

# ------------------------------------------------------------------------------
# Template validation (MapCatalog.validate_entry adds id uniqueness)
# ------------------------------------------------------------------------------

static func template_errors(entry: Dictionary) -> Array[String]:
	var t: Dictionary = normalize_template(entry)
	var errors: Array[String] = []
	for key: Variant in t:
		if not TEMPLATE_FIELDS.has(str(key)):
			errors.append("Unknown map field '%s'." % str(key))
	var id: String = str(t["id"])
	if id.is_empty():
		errors.append("ID is required.")
	elif not JsonUtil.id_ok(id):
		errors.append("Map id '%s' must be lowercase [a-z0-9_]." % id)
	if not (t["name"] is String) or str(t["name"]).strip_edges().is_empty():
		errors.append("Name is required.")
	var grid_ok: bool = _rows_errors(t, errors)
	_zone_errors(t, grid_ok, errors)
	_anchor_errors(t, grid_ok, errors)
	_path_errors(t, grid_ok, errors)
	if not (t["tags"] is Array) or not _all_strings(DictUtil.array_of(t["tags"])):
		errors.append("Tags must be a list of strings.")
	return errors

static func _all_strings(list: Array) -> bool:
	for x: Variant in list:
		if not (x is String):
			return false
	return true

# True when the grid has a usable size (later tile checks run only then).
static func _rows_errors(t: Dictionary, errors: Array[String]) -> bool:
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

static func _zone_errors(t: Dictionary, grid_ok: bool, errors: Array[String]) -> void:
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

static func _anchor_errors(t: Dictionary, grid_ok: bool, errors: Array[String]) -> void:
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

static func _path_errors(t: Dictionary, grid_ok: bool, errors: Array[String]) -> void:
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

# Soft problems of one template: a legend zone that covers no tile.
static func template_warnings(entry: Dictionary) -> Array[String]:
	var t: Dictionary = normalize_template(entry)
	var out: Array[String] = []
	var used: Dictionary = {}
	for row: Variant in DictUtil.array_of(t["zones"]):
		var s: String = str(row)
		for x: int in range(s.length()):
			used[s[x]] = true
	var lg: Dictionary = legend(t)
	for key: Variant in lg:
		if not used.has(str(key)):
			out.append("Zone '%s' has no tiles." % str(DictUtil.dict_of(lg[key]).get("id", key)))
	return out

# ------------------------------------------------------------------------------
# Location bindings (WorldCatalog._location_errors calls location_errors)
# ------------------------------------------------------------------------------

# A marker with its kind's defaults filled (unknown keys kept).
static func marker_full(m: Dictionary) -> Dictionary:
	var out: Dictionary = {"id": "", "kind": "", "requires_flags": [], "unless_flags": [], "sets_flags": []}
	var kind: String = str(m.get("kind", ""))
	if kind != "roamer":
		out["anchor"] = ""
	out.merge(DictUtil.dict_of(MARKER_KINDS.get(kind, {})).duplicate(true), true)
	out.merge(DictUtil.dict_of(JsonUtil.normalize(m)), true)
	return out

# A binding with the defaults filled (unknown keys kept).
static func binding_full(b: Dictionary) -> Dictionary:
	var out: Dictionary = {"encounters": [], "elites": [], "elite_pct": DEFAULT_ELITE_PCT, "steps_min": DEFAULT_STEPS_MIN, "steps_max": DEFAULT_STEPS_MAX}
	out.merge(DictUtil.dict_of(JsonUtil.normalize(b)), true)
	return out

# Every map-binding problem of a (normalised) location entry of `world_id`.
static func location_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var map_id: String = str(e.get("map", ""))
	var bindings_raw: Variant = e.get("zone_bindings", {})
	var markers_raw: Variant = e.get("markers", [])
	# Types first, with or without a map (a wrong-typed value is never read as empty).
	var types_ok: bool = true
	if not (bindings_raw is Dictionary):
		errors.append("Zone bindings must be an object {zone id: binding}.")
		types_ok = false
	if not (markers_raw is Array):
		errors.append("Markers must be a list.")
		types_ok = false
	if map_id.is_empty():
		if not DictUtil.dict_of(bindings_raw).is_empty() or not DictUtil.array_of(markers_raw).is_empty():
			errors.append("Zone bindings and markers need a map.")
		return errors
	var t: Dictionary = MapCatalog.get_entry(map_id)
	if t.is_empty():
		errors.append("Unknown map template '%s'." % map_id)
		return errors
	if not types_ok:
		return errors
	var bindings: Dictionary = bindings_raw
	for zone: Variant in bindings:
		errors.append_array(binding_errors(world_id, t, map_id, str(zone), bindings[zone]))
	var markers: Array = markers_raw
	errors.append_array(markers_errors(world_id, e, t, markers))
	errors.append_array(world_map_exit_errors(world_id, e))
	return errors

# TRAV1-L: a location with a world_map exit marker must be a node of its world's route graph (TravelRules.nodes).
static func world_map_exit_errors(world_id: String, e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var id: String = str(e.get("id", ""))
	for x: Variant in DictUtil.array_of(e.get("markers", [])):
		var m: Dictionary = DictUtil.dict_of(x)
		if str(m.get("kind", "")) == "exit" and str(m.get("to", "")) == WORLD_MAP:
			if not TravelRules.nodes(world_id).has(WorldCatalog.qid(world_id, id)):
				errors.append("'%s' has a world map exit but no route." % id)
			break
	return errors

static func binding_errors(world_id: String, t: Dictionary, map_id: String, zone: String, raw: Variant) -> Array[String]:
	var errors: Array[String] = []
	var kind: String = zone_kind(t, zone)
	if kind.is_empty():
		errors.append("Zone '%s' is not in map '%s'." % [zone, map_id])
	elif kind == "safe":
		errors.append("Zone '%s' is safe: safe zones roll nothing." % zone)
	if not (raw is Dictionary):
		errors.append("Zone binding '%s' must be an object." % zone)
		return errors
	for key: Variant in DictUtil.dict_of(raw):
		if not BINDING_KEYS.has(str(key)):
			errors.append("Zone binding '%s': unknown key '%s'." % [zone, str(key)])
	var b: Dictionary = binding_full(DictUtil.dict_of(raw))
	for list_key: String in ["encounters", "elites"]:
		errors.append_array(_pairs_errors(world_id, zone, list_key, b[list_key]))
	if not (b["elite_pct"] is int) or DictUtil.to_int(b["elite_pct"]) < 0 or DictUtil.to_int(b["elite_pct"]) > 100:
		errors.append("Zone binding '%s': elite_pct must be 0..100." % zone)
	var steps_ok: bool = true
	for key: String in ["steps_min", "steps_max"]:
		if not (b[key] is int) or DictUtil.to_int(b[key]) < STEPS_LO or DictUtil.to_int(b[key]) > STEPS_HI:
			errors.append("Zone binding '%s': %s must be %d..%d." % [zone, key, STEPS_LO, STEPS_HI])
			steps_ok = false
	if steps_ok and DictUtil.to_int(b["steps_min"]) > DictUtil.to_int(b["steps_max"]):
		errors.append("Zone binding '%s': steps_min is above steps_max." % zone)
	if DictUtil.array_of(b["encounters"]).is_empty() and DictUtil.array_of(b["elites"]).is_empty():
		errors.append("Zone binding '%s' binds no encounter." % zone)
	return errors

static func _pairs_errors(world_id: String, zone: String, key: String, raw: Variant) -> Array[String]:
	return pair_list_errors(world_id, "Zone binding '%s'" % zone, key, raw)

# An [[encounter ref, weight 1..100]...] list (max MAX_BIND_LIST, no duplicate ref): zone bindings (EXP1) and routes
# (TRAV1). `label` prefixes every message ("Zone binding 'wilds'", "Route").
static func pair_list_errors(world_id: String, label: String, key: String, raw: Variant) -> Array[String]:
	var errors: Array[String] = []
	if not (raw is Array):
		errors.append("%s: %s must be a list of [encounter, weight] pairs." % [label, key])
		return errors
	var list: Array = raw
	if list.size() > MAX_BIND_LIST:
		errors.append("%s: at most %d %s." % [label, MAX_BIND_LIST, key])
	var seen: Dictionary = {}
	for x: Variant in list:
		var pair: Array = DictUtil.array_of(x)
		if pair.size() != 2 or not (pair[0] is String):
			errors.append("%s: %s entries are [encounter, weight] pairs." % [label, key])
			continue
		var ref: String = str(pair[0])
		if seen.has(ref):
			errors.append("%s: %s lists '%s' twice." % [label, key, ref])
		seen[ref] = true
		errors.append_array(WorldCatalog.ref_errors(world_id, "%s %s" % [label, key], "encounters", ref))
		if not (pair[1] is int) or DictUtil.to_int(pair[1]) < WEIGHT_MIN or DictUtil.to_int(pair[1]) > WEIGHT_MAX:
			errors.append("%s: %s weight of '%s' must be %d..%d." % [label, key, ref, WEIGHT_MIN, WEIGHT_MAX])
	return errors

# The `to` strings of the location's exits list.
static func exit_targets(e: Dictionary) -> Array[String]:
	var out: Array[String] = []
	for x: Variant in DictUtil.array_of(e.get("exits", [])):
		var to: String = str(DictUtil.dict_of(x).get("to", ""))
		if not to.is_empty():
			out.append(to)
	return out

static func markers_errors(world_id: String, e: Dictionary, t: Dictionary, list: Array) -> Array[String]:
	var errors: Array[String] = []
	if list.size() > MAX_MARKERS:
		errors.append("At most %d markers." % MAX_MARKERS)
	var ids: Dictionary = {}
	var anchors_taken: Dictionary = {}
	for i: int in range(list.size()):
		if not (list[i] is Dictionary):
			errors.append("Marker %d must be an object." % i)
			continue
		var raw: Dictionary = DictUtil.dict_of(JsonUtil.normalize(list[i]))
		var mid: String = str(raw.get("id", ""))
		if not (raw.get("id", null) is String) or not JsonUtil.id_ok(mid):
			errors.append("Marker %d: id '%s' must be lowercase [a-z0-9_]." % [i, mid])
			continue
		if ids.has(mid):
			errors.append("Marker id '%s' is used twice." % mid)
		ids[mid] = true
		var kind: String = str(raw.get("kind", ""))
		if not MARKER_KINDS.has(kind):
			errors.append("Marker '%s': unknown kind '%s' (use %s)." % [mid, kind, ", ".join(PackedStringArray(MARKER_KINDS.keys()))])
			continue
		var own_keys: Dictionary = DictUtil.dict_of(MARKER_KINDS[kind])
		for key: Variant in raw:
			var k: String = str(key)
			if k == "anchor" and kind == "roamer":
				errors.append("Marker '%s': a roamer takes no anchor (it walks its path)." % mid)
			elif not MARKER_COMMON.has(k) and not own_keys.has(k):
				errors.append("Marker '%s' (%s): unknown key '%s'." % [mid, kind, k])
		var m: Dictionary = marker_full(raw)
		if kind != "roamer":
			var anchor: String = str(m["anchor"])
			if not has_anchor(t, anchor):
				errors.append("Marker '%s': unknown anchor '%s'." % [mid, anchor])
			elif anchor == SPAWN:
				errors.append("Marker '%s': the '%s' anchor holds no marker." % [mid, SPAWN])
			elif anchors_taken.has(anchor):
				errors.append("Marker '%s': anchor '%s' already holds marker '%s'." % [mid, anchor, str(anchors_taken[anchor])])
			else:
				anchors_taken[anchor] = mid
		for fk: String in FLAG_KEYS:
			errors.append_array(QuestRules.flag_list_errors(m[fk], "Marker '%s' %s" % [mid, fk], fk))
		errors.append_array(_marker_kind_errors(world_id, e, t, mid, kind, m))
	return errors

static func _marker_kind_errors(world_id: String, e: Dictionary, t: Dictionary, mid: String, kind: String, m: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	match kind:
		"exit":
			var to: String = str(m["to"])
			if to.is_empty():
				errors.append("Marker '%s': an exit needs \"to\"." % mid)
			elif to != WORLD_MAP and not exit_targets(e).has(to):
				errors.append("Exit '%s' is not in this location's exits." % to)
			var to_anchor: String = str(m["to_anchor"])
			var target_map: Dictionary = {}
			if not to.is_empty() and to != WORLD_MAP:
				var target: Dictionary = WorldCatalog.resolve(world_id, "locations", to)
				var target_map_id: String = str(target.get("map", ""))
				if not target_map_id.is_empty():
					target_map = MapCatalog.get_entry(target_map_id)
			if not target_map.is_empty():
				if not has_anchor(target_map, to_anchor):
					errors.append("Marker '%s': to_anchor '%s' is not an anchor of map '%s'." % [mid, to_anchor, str(target_map.get("id", ""))])
			elif to == WORLD_MAP:
				if to_anchor != SPAWN:
					errors.append("Marker '%s': a world_map exit always arrives at '%s' (to_anchor must be '%s')." % [mid, SPAWN, SPAWN])
			elif to_anchor != SPAWN:
				errors.append("Marker '%s': '%s' has no map, so to_anchor must be '%s'." % [mid, to, SPAWN])
			if not (m["locked_text"] is String):
				errors.append("Marker '%s': locked_text must be text." % mid)
		"encounter":
			errors.append_array(_ref_key_errors(world_id, mid, "encounter", "encounters", m))
			if not TRIGGERS.has(str(m["trigger"])):
				errors.append("Marker '%s': trigger must be touch or proximity." % mid)
			errors.append_array(_radius_once_errors(mid, m))
		"roamer":
			errors.append_array(_ref_key_errors(world_id, mid, "encounter", "encounters", m))
			if not has_path(t, str(m["path"])):
				errors.append("Marker '%s': Unknown path '%s'." % [mid, str(m["path"])])
			errors.append_array(_radius_once_errors(mid, m))
		"npc":
			errors.append_array(_ref_key_errors(world_id, mid, "character", "characters", m))
		"lore":
			errors.append_array(_ref_key_errors(world_id, mid, "lore", "lore", m))
		"chest":
			errors.append_array(_chest_errors(world_id, mid, m))
	return errors

static func _ref_key_errors(world_id: String, mid: String, key: String, ref_kind: String, m: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var ref: String = str(m[key])
	if not (m[key] is String) or ref.is_empty():
		errors.append("Marker '%s': choose a %s." % [mid, key])
		return errors
	return WorldCatalog.ref_errors(world_id, "Marker '%s' %s" % [mid, key], ref_kind, ref)

static func _radius_once_errors(mid: String, m: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	if not (m["radius"] is int) or DictUtil.to_int(m["radius"]) < 0 or DictUtil.to_int(m["radius"]) > MAX_RADIUS:
		errors.append("Marker '%s': radius must be 0..%d." % [mid, MAX_RADIUS])
	if not (m["once"] is bool):
		errors.append("Marker '%s': once must be true or false." % mid)
	return errors

static func _chest_errors(world_id: String, mid: String, m: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var items_raw: Variant = m["items"]
	var items: Dictionary = DictUtil.dict_of(items_raw)
	if not (items_raw is Dictionary):
		errors.append("Marker '%s': items must be an object {base item id: count}." % mid)
	elif items.size() > MAX_CHEST_KEYS:
		errors.append("Marker '%s': at most %d item kinds." % [mid, MAX_CHEST_KEYS])
	for key: Variant in items:
		if ContentCatalog.get_entry("items", str(key)).is_empty():
			errors.append("Marker '%s': unknown base item '%s'." % [mid, str(key)])
		if not (items[key] is int) or DictUtil.to_int(items[key]) < 1 or DictUtil.to_int(items[key]) > CHEST_COUNT_MAX:
			errors.append("Marker '%s': count of '%s' must be 1..%d." % [mid, str(key), CHEST_COUNT_MAX])
	var loot: String = str(m["loot"])
	if not (m["loot"] is String):
		errors.append("Marker '%s': loot must be a loot ref." % mid)
	elif not loot.is_empty():
		var ref_errs: Array[String] = WorldCatalog.ref_errors(world_id, "Marker '%s' loot" % mid, "loot", loot)
		errors.append_array(ref_errs)
		if ref_errs.is_empty() and str(WorldCatalog.resolve(world_id, "loot", loot).get("reward_kind", "")) != "item":
			errors.append("Chest loot must be an item entry.")
	var gold: Variant = m["gold"]
	if not (gold is int) or DictUtil.to_int(gold) < 0 or DictUtil.to_int(gold) > CHEST_GOLD_MAX:
		errors.append("Marker '%s': gold must be a whole number 0..%d." % [mid, CHEST_GOLD_MAX])
	var supplies: Variant = m.get("supplies", 0)
	if not (supplies is int) or DictUtil.to_int(supplies) < 0 or DictUtil.to_int(supplies) > CHEST_SUPPLIES_MAX:
		errors.append("Marker '%s': supplies must be a whole number 0..%d." % [mid, CHEST_SUPPLIES_MAX])
	if items.is_empty() and loot.is_empty() and DictUtil.to_int(gold) <= 0 and DictUtil.to_int(supplies) <= 0:
		errors.append("Chest '%s' holds nothing." % mid)
	return errors

# ------------------------------------------------------------------------------
# Soft warnings over a world's own locations
# ------------------------------------------------------------------------------

static func location_warnings(e: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var t: Dictionary = MapCatalog.get_entry(str(e.get("map", "")))
	if t.is_empty():
		return out
	var bindings: Dictionary = DictUtil.dict_of(e.get("zone_bindings", {}))
	for zid: String in danger_zone_ids(t):
		if not bindings.has(zid):
			out.append("Danger zone '%s' has no binding (it never rolls)." % zid)
	var marked: Dictionary = {}
	for x: Variant in DictUtil.array_of(e.get("markers", [])):
		var m: Dictionary = DictUtil.dict_of(x)
		if str(m.get("kind", "")) == "exit":
			marked[str(m.get("to", ""))] = true
	for to: String in exit_targets(e):
		if not marked.has(to):
			out.append("Exit to '%s' has no marker on the map." % to)
	return out

static func warnings(world_id: String) -> Array[String]:
	var out: Array[String] = []
	for e: Dictionary in WorldCatalog.own_entries(world_id, "locations"):
		for w: String in location_warnings(e):
			out.append("%s: %s" % [str(e["_qid"]), w])
	return out

# ------------------------------------------------------------------------------
# Start location and runtime flag names
# ------------------------------------------------------------------------------

# The qualified start location of `world_id`: its own `start_location`, else the nearest ancestor's (lineage order),
# resolved where it is written and then seen from `world_id` (so overrides apply). "" when none or unresolved.
static func start_location(world_id: String) -> String:
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

static func dotted(location_qid: String) -> String:
	return location_qid.replace(":", ".")

static func done_flag(location_qid: String, marker_id: String) -> String:
	return "%s.%s.done" % [dotted(location_qid), marker_id]

static func visited_flag(location_qid: String) -> String:
	return "%s.visited" % dotted(location_qid)
