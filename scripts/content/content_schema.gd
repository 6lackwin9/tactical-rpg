class_name ContentSchema
extends RefCounted

# ==============================================================================
# Tactical RPG - Content Schema (what each catalog entry looks like)
# One field list per catalog kind. The schema drives everything generic:
# normalising JSON (types, defaults, key order), field validation, and the
# Content Forge form (labels, widgets, ranges, help, conditional visibility).
#
# Field: {"key", "label", "type", "default", ...}
#   type: id | string | text | int | float | bool | enum | ref | refs | stats | strings | json | multi_pick | template
#         | table (ordered {key: int}, e.g. the class skill path) | dict_pick ({key: {..}} rows, e.g. class proficiency)
#         | modifier_table / tier_table (item `modifiers` / `tiers` lists) | hand_grid (family `base` / `bands`, per hand);
#           these three store like json (ITEM1 commit 3: typed Forge widgets, ItemFormModel / ItemTables)
#   min / max / step     numeric range (table: the value range)
#   min_len / max_len    string length caps (field_errors)
#   group                Forge section of a class / unit field (identity, stats, proficiency, skills, loadout; units: ai)
#   key_field            locked on system assets (ContentCatalog.key_field_errors)
#   options              enum choices (Array[String]); "options_from": "zones" | "art" | "status"
#   ref                  catalog kind a ref/refs field points at; "ref_filter": {key: value}
#   when                 {"key": k, "in": [...]} or {"key": k, "not_in": [...]}: shown/saved only then
#   keys                 stats: the allowed keys
#   readonly             shown, never edited (e.g. the "system" flag)
#   help                 tooltip text
# Enum values are stored by name ("ATTACK", "MAIN_HAND") so the JSON stays readable.
# ==============================================================================

# item_families (FORGE1_ITEMS 1) is the item template kind; ContentCatalog loads it before items (items resolve against it).
const KINDS: Array[String] = ["skills", "items", "item_families", "passives", "classes", "units", "encounters"]
const ORIGIN_SYSTEM: String = "system"
const ORIGIN_USER: String = "user"

const STAT_KEYS: Array[String] = ["STR", "DEX", "INT", "CONS", "PERC", "KNOW"]   # = StatRules.STATS
const EQUIP_TYPES: Array[String] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY"]
const LOADOUT_SLOTS: Array[String] = ["main_hand", "off_hand", "head", "body", "accessory_1", "accessory_2"]
const NO_ITEM: String = "none"   # unit gear slot / pouch ["none"]: deliberately empty, no class fallback (BAL2-L 3.1)

static func kind_label(kind: String) -> String:
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
	return kind.capitalize()

static func fields(kind: String) -> Array[Dictionary]:
	var list: Array[Dictionary] = []
	match kind:
		"skills":
			return _skill_fields()
		"items":
			list = ItemFields.build()
		"item_families":
			list = ItemFields.family_build()
		"passives":
			list = _passive_fields()
		"classes":
			list = ClassUnitFields.class_fields()
		"units":
			list = ClassUnitFields.unit_fields()
		"encounters":
			list = _encounter_fields()
		_:
			return list
	var origin: Dictionary = origin_field()
	if kind == "classes":
		origin["group"] = "identity"
	list.append(origin)
	return list

# The common field of every Forge kind (FORGE1 section 12): who owns the asset. Default system; only user assets store it.
static func origin_field() -> Dictionary:
	return _f("origin", "Origin", "enum", ORIGIN_SYSTEM, {"options": [ORIGIN_SYSTEM, ORIGIN_USER], "readonly": true, "key_field": true, "help": "system: shipped (key fields locked, delete only through SAFE DELETE). user: your own asset. Set by the Forge, never typed."})

static func field(kind: String, key: String) -> Dictionary:
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

# ------------------------------------------------------------------------------
# Field lists: skills, items, classes and units live in SkillFields, ItemFields and ClassUnitFields (CLN-9b);
# the small passive and encounter tables stay here. _f is shared by all of them.
# ------------------------------------------------------------------------------

static func _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary:
	var f: Dictionary = {"key": key, "label": label, "type": type, "default": default}
	f.merge(extra, true)
	return f

static var _skill_cache: Array[Dictionary] = []

# The skill field list (cached: it is generated from SkillKinds, so it is built once). Callers get a deep copy, like
# every other kind's freshly built list, so editing a returned field can never change the schema.
static func _skill_fields() -> Array[Dictionary]:
	if _skill_cache.is_empty():
		_skill_cache = SkillFields.build()
	var out: Array[Dictionary] = []
	out.assign(_skill_cache.duplicate(true))
	return out

static func _passive_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Passive"),
		_f("description", "Description", "text", ""),
		_f("effects", "Effects", "json", [], {"help": "[{\"type\": \"stat\", \"stat\": \"KNOW\", \"value\": 3, \"context\": {\"biome\": \"ruins\"}}, {\"type\": \"proficiency\", \"gear\": \"sword\", \"levels\": 1}, {\"type\": \"resist\", \"status\": \"stagger\", \"value\": 25}, {\"type\": \"ap_regen\", \"value\": 1}, {\"type\": \"skill\", \"skill\": \"quick_stab\"}, {\"type\": \"skill_potency\", \"family\": \"strike\", \"value\": 0.25}, {\"type\": \"unlock\", \"family\": \"riposte\", \"levels\": 2}]"}),
	]

static func _encounter_fields() -> Array[Dictionary]:
	return [
		_f("id", "ID", "id", ""),
		_f("name", "Name", "string", "New Encounter"),
		_f("description", "Description", "text", ""),
		_f("zone", "Zone", "enum", "stone_arena", {"options_from": "zones"}),
		_f("arena", "Arena", "enum", "CLOSED", {"options": ["CLOSED", "OPEN"], "help": "OPEN: every edge is an exit."}),
		_f("exits", "Exit tiles", "json", [[8, 4]], {"help": "[[x, y], ...] on the 9x9 grid (CLOSED arenas)."}),
		_f("max_rounds", "Max rounds", "int", 6, {"min": 1, "max": 30}),
		_f("total_blocks", "Blocks / round", "int", 6, {"min": 3, "max": 10}),
		_f("ap_regen", "AP regen / round", "int", 4, {"min": -1, "max": 20, "help": "-1 = refill to max each round."}),
		_f("unit_ap", "Unit AP", "int", 12, {"min": 0, "max": 40, "help": "Starting / max AP for units without their own pool."}),
		_f("rng_seed", "RNG seed", "int", 1337, {"min": 0, "max": 2147483647}),
		_f("party", "Party", "refs", [], {"ref": "units", "ref_filter": {"side": "party"}}),
		_f("enemies", "Enemies", "refs", [], {"ref": "units", "ref_filter": {"side": "enemy"}, "dupes": true, "help": "The same unit may appear more than once."}),
		_f("wave_units", "Reinforcements", "refs", [], {"ref": "units", "ref_filter": {"side": "enemy"}, "dupes": true, "help": "Enemies that join mid-battle (one wave). They are planned by the AI from the next round."}),
		_f("wave_round", "Wave round", "int", 1, {"min": 1, "max": 30}),
		_f("wave_block", "Wave block", "int", 4, {"min": 1, "max": 10, "help": "The wave enters before this block resolves."}),
		_f("wave_label", "Wave label", "string", "Reinforcements arrive!"),
	]

# ------------------------------------------------------------------------------
# Options & visibility
# ------------------------------------------------------------------------------

static func options_of(f: Dictionary) -> Array[String]:
	var out: Array[String] = []
	match str(f.get("options_from", "")):
		"zones":
			return CombatZones.zone_ids()
		"art":
			return AssetRegistry.keys("unit")
		"art_inherit":
			out.append("")
			out.append_array(AssetRegistry.keys("unit"))
			return out
		"status":
			out.append("")
			out.append_array(StatusEffects.debuffs())
			return out
		"status_buff":
			out.append("")
			out.append_array(StatusEffects.buffs())
			return out
		"weapon_types":
			return SkillDef.weapon_type_tokens()
		"debuffs_or_all":
			out.append_array(StatusEffects.debuffs())
			out.append(SkillDef.CLEANSE_ALL)
			return out
	var raw: Variant = f.get("options", [])
	if raw is Array:
		var arr: Array = raw
		for v: Variant in arr:
			out.append(str(v))
	return out

# True when `f` applies to `entry` (its "when" condition holds).
static func is_visible(f: Dictionary, entry: Dictionary) -> bool:
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

# True when a ref target entry passes the field's "ref_filter".
static func passes_filter(f: Dictionary, target: Dictionary) -> bool:
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

# ------------------------------------------------------------------------------
# Normalising: coerce types, fill defaults, schema key order
# ------------------------------------------------------------------------------

static func defaults(kind: String) -> Dictionary:
	var d: Dictionary = {}
	for f: Dictionary in fields(kind):
		d[str(f["key"])] = _copy(f["default"])
	return d

# A clean copy of `entry`: every schema field present and typed, unknown keys kept
# at the end (so hand-added data survives a round trip through the Forge).
static func normalize(kind: String, entry: Dictionary) -> Dictionary:
	var src: Dictionary = entry
	if kind == "skills":
		src = _legacy_skill(entry)
	elif kind == "classes":
		src = _legacy_class(entry)
	var out: Dictionary = {}
	for f: Dictionary in fields(kind):
		var key: String = str(f["key"])
		out[key] = coerce(f, src.get(key, f["default"]))
	for key: Variant in src:
		if not out.has(key):
			out[key] = _copy(src[key])
	return out

# Pre-FORGE1 skill keys, deprecated (warn on use; CLN-3b drops this normaliser once logs show no hit):
# advanced: true -> grade advanced, offhand: true -> weapon_hand off.
static func _legacy_skill(entry: Dictionary) -> Dictionary:
	if not entry.has("advanced") and not entry.has("offhand"):
		return entry
	push_warning("ContentSchema: skill '%s' uses the deprecated key(s) advanced / offhand; write grade / weapon_hand instead (CLN-3b removes them)" % str(entry.get("id", "")))
	var out: Dictionary = entry.duplicate()
	if out.has("advanced"):
		if DictUtil.to_bool(out["advanced"]) and not out.has("grade"):
			out["grade"] = "advanced"
		out.erase("advanced")
	if out.has("offhand"):
		if DictUtil.to_bool(out["offhand"]) and not out.has("weapon_hand"):
			out["weapon_hand"] = "off"
		out.erase("offhand")
	return out

# Pre-CLASS1 class path, accepted for one release (FORGE1_CLASSES 3.5): a list [{level, skill}] becomes the dict
# {skill: level} in list order, basics dropped (ClassArchetype.path_from_list); saving then writes the dict. CLASS1-M converts
# the shipped data; with ClassUnitFields.list_path_converter off a list stays a list and the validator reports it.
static func _legacy_class(entry: Dictionary) -> Dictionary:
	var raw: Variant = entry.get("skill_path", null)
	if not (raw is Array) or not ClassUnitFields.list_path_converter:
		return entry
	var steps: Array = raw
	var out: Dictionary = entry.duplicate()
	out["skill_path"] = ClassArchetype.path_from_list(steps)
	return out

# The entry as written to disk: fields hidden by "when" (and default "system" / "origin") are left out.
static func to_storage(kind: String, entry: Dictionary) -> Dictionary:
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

static func coerce(f: Dictionary, value: Variant) -> Variant:
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

static func _copy(v: Variant) -> Variant:
	if v is Array:
		var arr: Array = v
		return _json_ints(arr.duplicate(true))
	if v is Dictionary:
		var dict: Dictionary = v
		return _json_ints(dict.duplicate(true))
	return v

# JSON parses every number as float; whole numbers inside free-form json fields go back to int.
static func _json_ints(v: Variant) -> Variant:
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

# Field-level problems (types are already coerced; this checks ids and required values).
static func field_errors(kind: String, entry: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var id: String = str(entry.get("id", ""))
	if id.is_empty():
		errors.append("ID is required.")
	elif not is_valid_id(id):
		errors.append("ID '%s' must be lowercase snake_case (a-z, 0-9, _)." % id)
	if str(entry.get("name", "")).strip_edges().is_empty():
		errors.append("Name is required.")
	for f: Dictionary in fields(kind):
		if str(f.get("type", "")) == "enum" and is_visible(f, entry):
			var value: String = str(entry.get(str(f["key"]), ""))
			if not options_of(f).has(value):
				errors.append("%s: '%s' is not one of %s." % [str(f["label"]), value, ", ".join(options_of(f))])
		if f.has("max_len") and str(entry.get(str(f["key"]), "")).length() > DictUtil.to_int(f["max_len"]):
			errors.append("%s is %d characters (max %d)." % [str(f["label"]), str(entry.get(str(f["key"]), "")).length(), DictUtil.to_int(f["max_len"])])
		if f.has("min_len") and not str(entry.get(str(f["key"]), "")).is_empty() and str(entry.get(str(f["key"]), "")).length() < DictUtil.to_int(f["min_len"]):
			errors.append("%s is %d characters (min %d)." % [str(f["label"]), str(entry.get(str(f["key"]), "")).length(), DictUtil.to_int(f["min_len"])])
	return errors

static func is_valid_id(id: String) -> bool:
	var re := RegEx.new()
	re.compile("^[a-z][a-z0-9_]*$")
	return re.search(id) != null
