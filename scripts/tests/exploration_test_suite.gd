class_name ExplorationTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Exploration Test Suite (S9, spec EXP1-L)
# Commit 1 (schema): acceptance checks 1-5. Map template validation (MapCatalog / MapRules), zone bindings,
# markers, the world manifest `start_location` and the reference scans through the new location fields.
# Fixtures: MapCatalog on a sandbox file and two hand-built worlds in an EMPTY sandbox world root; the shipped
# catalog is only read (an item and a passive picked by rule), shipped worlds and maps are never written.
# ==============================================================================

const TAG: String = "Explore"
static var SANDBOX: String = TestHelpers.sandbox("explore")
const PARENT: String = "xa_base"
const CHILD: String = "xa_child"

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

# True when `errors` is exactly one message containing `needle` (prints the list otherwise, for debugging).
static func _one(errors: Array[String], needle: String) -> bool:
	var ok: bool = errors.size() == 1 and errors[0].contains(needle)
	if not ok:
		print("      expected one error with '%s', got: %s" % [needle, str(errors)])
	return ok

static func _loc_errs(entry: Dictionary, world: String = PARENT) -> Array[String]:
	return WorldCatalog.validate_entry(world, "locations", entry, str(entry.get("id", "")))

# ------------------------------------------------------------------------------
# Fixtures
# ------------------------------------------------------------------------------

# 10 x 8: walls round the edge, water at (4, 3); zones: camp (safe) top-left, wilds (danger) right, deep (danger, left
# unbound for the warning check) lower right.
static func _field_map() -> Dictionary:
	return {
		"id": "m_field", "name": "Field",
		"rows": ["##########", "#........#", "#........#", "#...~....#", "#........#", "#........#", "#........#", "##########"],
		"zones": ["----------", "-sss-ddddd", "-sss-ddddd", "-----ddddd", "-----eeeee", "-----eeeee", "----------", "----------"],
		"zone_legend": {"s": {"id": "camp", "kind": "safe"}, "d": {"id": "wilds", "kind": "danger"}, "e": {"id": "deep", "kind": "danger"}},
		"anchors": [
			{"id": "spawn", "x": 1, "y": 1}, {"id": "a_npc", "x": 2, "y": 4}, {"id": "a_chest", "x": 3, "y": 4},
			{"id": "a_lore", "x": 4, "y": 4}, {"id": "a_exit", "x": 1, "y": 6}, {"id": "a_enc", "x": 6, "y": 2},
			{"id": "a_point", "x": 7, "y": 6}, {"id": "a_spare", "x": 8, "y": 6},
		],
		"paths": {"loop": [[6, 5], [7, 5], [8, 5]]},
		"tags": ["test"],
	}

static func _small_map() -> Dictionary:
	return {
		"id": "m_small", "name": "Small",
		"rows": ["########", "#......#", "#......#", "#......#", "#......#", "#......#", "#......#", "########"],
		"anchors": [{"id": "spawn", "x": 1, "y": 1}, {"id": "gate", "x": 2, "y": 1}],
	}

# The valid mapped location of PARENT; every check below breaks exactly one thing in a copy of it.
static func _field(fx: Dictionary) -> Dictionary:
	return {
		"id": "field", "name": "Field", "map": "m_field",
		"exits": [{"to": "town", "label": "to town"}, {"to": "keep", "label": "to the keep"}],
		"zone_bindings": {"wilds": {"encounters": [["enc_a", 3], ["enc_b", 1]], "elites": [["elite_a", 1]]}},
		"markers": [
			{"id": "to_town", "kind": "exit", "anchor": "a_exit", "to": "town"},
			{"id": "to_keep", "kind": "exit", "anchor": "a_spare", "to": "keep", "to_anchor": "gate"},
			{"id": "ambush", "kind": "encounter", "anchor": "a_enc", "encounter": "enc_a", "trigger": "proximity", "radius": 2},
			{"id": "patrol", "kind": "roamer", "encounter": "elite_a", "path": "loop"},
			{"id": "guide", "kind": "npc", "anchor": "a_npc", "character": "npc_a"},
			{"id": "box", "kind": "chest", "anchor": "a_chest", "items": {str(fx["item"]): 2}, "loot": "loot_item"},
			{"id": "stone", "kind": "lore", "anchor": "a_lore", "lore": "lore_a", "sets_flags": [{"key": "xa.read"}]},
			{"id": "mark", "kind": "point", "anchor": "a_point", "requires_flags": [{"key": "xa.read"}]},
		],
	}

static func _marker_index(loc: Dictionary, marker_id: String) -> int:
	var list: Array = DictUtil.array_of(loc["markers"])
	for i: int in range(list.size()):
		if str(DictUtil.dict_of(list[i]).get("id", "")) == marker_id:
			return i
	return -1

# A copy of `loc` with `changes` merged into marker `marker_id`.
static func _with_marker(loc: Dictionary, marker_id: String, changes: Dictionary) -> Dictionary:
	var out: Dictionary = loc.duplicate(true)
	var list: Array = out["markers"]
	var m: Dictionary = list[_marker_index(out, marker_id)]
	m.merge(changes, true)
	return out

static func _without_key(loc: Dictionary, marker_id: String, key: String) -> Dictionary:
	var out: Dictionary = loc.duplicate(true)
	var list: Array = out["markers"]
	var m: Dictionary = list[_marker_index(out, marker_id)]
	m.erase(key)
	return out

static func _with_binding(loc: Dictionary, zone: String, binding: Dictionary) -> Dictionary:
	var out: Dictionary = loc.duplicate(true)
	var b: Dictionary = out["zone_bindings"]
	b[zone] = binding
	return out

static func _build_worlds(fx: Dictionary) -> bool:
	var errors: Array[String] = []
	errors.append_array(WorldCatalog.create_world(PARENT, "XA Base"))
	errors.append_array(WorldCatalog.create_world(CHILD, "XA Child", [PARENT]))
	for id: String in ["enc_a", "enc_b", "elite_a"]:
		errors.append_array(WorldCatalog.upsert(PARENT, "encounters", {"id": id, "name": id}))
	errors.append_array(WorldCatalog.upsert(PARENT, "characters", {"id": "npc_a", "name": "Npc A"}))
	errors.append_array(WorldCatalog.upsert(PARENT, "lore", {"id": "lore_a", "name": "Lore A", "text": "Old stones."}))
	errors.append_array(WorldCatalog.upsert(PARENT, "loot", {"id": "loot_item", "name": "Loot Item", "reward_kind": "item", "ref": str(fx["item"]), "count": 1}))
	errors.append_array(WorldCatalog.upsert(PARENT, "loot", {"id": "loot_passive", "name": "Loot Passive", "reward_kind": "passive", "ref": str(fx["passive"])}))
	errors.append_array(WorldCatalog.upsert(PARENT, "locations", {"id": "town", "name": "Town", "exits": [{"to": "field"}]}))
	errors.append_array(WorldCatalog.upsert(PARENT, "locations", {"id": "hut", "name": "Hut"}))
	errors.append_array(WorldCatalog.upsert(PARENT, "locations", {"id": "keep", "name": "Keep", "map": "m_small", "exits": [{"to": "field"}],
		"markers": [{"id": "back", "kind": "exit", "anchor": "gate", "to": "field", "to_anchor": "a_exit"}]}))
	errors.append_array(WorldCatalog.upsert(PARENT, "locations", _field(fx)))
	var checks: Dictionary = {"no errors": errors.is_empty(), "lineage": WorldCatalog.lineage(CHILD) == [PARENT, CHILD]}
	for id: String in ["field", "town", "keep", "hut"]:
		checks["%s validates" % id] = _loc_errs(WorldCatalog.get_own(PARENT, "locations", id)).is_empty()
	if not errors.is_empty():
		print("      fixture errors: %s" % str(errors))
	return _report("Fixture worlds (parent + child) built through upsert", checks)

# ------------------------------------------------------------------------------

static func run() -> bool:
	print("\n=======================================================")
	print("=== STARTING EXPLORATION TEST (EXP1-L schema)        ===")
	print("=======================================================")
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	MapCatalog.use_path(MapCatalog.DEFAULT_PATH)
	var ok: bool = true
	ok = _test_shipped() and ok
	ok = _test_helpers() and ok
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/worlds")
	MapCatalog.use_path(SANDBOX + "/maps.json")
	WorldCatalog.use_root(SANDBOX + "/worlds")
	var sandboxed: bool = MapCatalog.file_path().begins_with("user://") and WorldCatalog.get_root().begins_with("user://") and WorldCatalog.world_ids().is_empty() and MapCatalog.ids().is_empty()
	ok = _report("Sandbox (empty map file and world root under user://)", {"sandboxed": sandboxed}) and ok
	if sandboxed:
		var items: Array[String] = ContentCatalog.ids("items")
		var passives: Array[String] = ContentCatalog.ids("passives")
		var fx: Dictionary = {"item": items[0] if not items.is_empty() else "", "passive": passives[0] if not passives.is_empty() else ""}
		ok = _report("Content fixtures picked by rule", {"an item": not str(fx["item"]).is_empty(), "a passive": not str(fx["passive"]).is_empty()}) and ok
		ok = _test_templates() and ok
		ok = _build_worlds(fx) and ok
		ok = _test_bindings(fx) and ok
		ok = _test_markers(fx) and ok
		ok = _test_start_location() and ok
		ok = _test_ref_scans() and ok
	MapCatalog.use_path(MapCatalog.DEFAULT_PATH)
	TestHelpers.cleanup_worlds(SANDBOX)   # back to the shipped worlds; removes the whole sandbox (map file included)
	return ok

static func _test_shipped() -> bool:
	var checks: Dictionary = {}
	checks["shipped maps load"] = MapCatalog.load_errors().is_empty() and not MapCatalog.save_blocked()
	checks["shipped maps validate"] = MapCatalog.validate_all().is_empty()
	checks["shipped worlds validate with the new fields"] = WorldCatalog.validate_all().is_empty()
	var d: Dictionary = WorldSchema.defaults("locations")
	checks["location defaults = menu location"] = str(d["map"]).is_empty() and DictUtil.dict_of(d["zone_bindings"]).is_empty() and DictUtil.array_of(d["markers"]).is_empty()
	checks["world default start_location empty"] = str(WorldSchema.defaults("world")["start_location"]).is_empty()
	return _report("Shipped data validates (maps file, worlds with the new location and manifest fields)", checks)

static func _test_helpers() -> bool:
	var checks: Dictionary = {}
	var seen: Array[int] = []
	for s: int in range(7):
		seen.append(MapRules.roamer_index(4, s))
	checks["4-point path ping-pongs 0,1,2,3,2,1,0"] = seen == [0, 1, 2, 3, 2, 1, 0]
	checks["1-point path stays"] = MapRules.roamer_index(1, 5) == 0
	checks["done flag"] = MapRules.done_flag("ashlands:ruined_courtyard", "chest_a") == "ashlands.ruined_courtyard.chest_a.done"
	checks["visited flag"] = MapRules.visited_flag("ashlands:ruined_courtyard") == "ashlands.ruined_courtyard.visited"
	checks["flags are valid flag keys"] = GameState.flag_key_ok(MapRules.done_flag("w:l", "m")) and GameState.flag_key_ok(MapRules.visited_flag("w:l"))
	var t: Dictionary = MapRules.normalize_template(_field_map())
	checks["zone lookup"] = MapRules.zone_at(t, 2, 1) == "camp" and MapRules.zone_at(t, 6, 1) == "wilds" and MapRules.zone_at(t, 0, 0) == "" and MapRules.zone_kind(t, "wilds") == "danger"
	checks["tiles"] = MapRules.is_open(t, 1, 1) and not MapRules.is_open(t, 0, 0) and not MapRules.is_open(t, 4, 3) and not MapRules.is_open(t, -1, 2) and MapRules.size_of(t) == Vector2i(10, 8)
	checks["anchor lookup"] = MapRules.anchor_pos(t, "spawn") == Vector2i(1, 1) and MapRules.anchor_pos(t, "nope") == Vector2i(-1, -1)
	return _report("MapRules helpers (roamer index, flag names, lookups)", checks)

# ------------------------------------------------------------------------------
# Check 1: templates
# ------------------------------------------------------------------------------

static func _rows_with(t: Dictionary, y: int, row: String) -> Dictionary:
	var out: Dictionary = t.duplicate(true)
	var rows: Array = out["rows"]
	rows[y] = row
	return out

static func _test_templates() -> bool:
	var checks: Dictionary = {}
	var valid: Dictionary = _field_map()
	checks["valid template: no errors"] = MapCatalog.validate_entry(valid).is_empty()
	var t: Dictionary = valid.duplicate(true)
	DictUtil.array_of(t["rows"]).pop_back()
	checks["7 rows"] = _one(MapCatalog.validate_entry(t), "Rows:")
	var long_row: String = "#" + ".".repeat(MapRules.MAX_SIDE - 1) + "#"
	checks["65-char row"] = long_row.length() == MapRules.MAX_SIDE + 1 and _one(MapCatalog.validate_entry(_rows_with(valid, 1, long_row)), "Rows:")
	checks["unequal rows"] = _one(MapCatalog.validate_entry(_rows_with(valid, 1, "#.........#")), "Rows:")
	checks["unknown tile x"] = _one(MapCatalog.validate_entry(_rows_with(valid, 2, "#.x......#")), "unknown tile 'x'")
	t = valid.duplicate(true)
	DictUtil.array_of(t["anchors"]).append({"id": "a_wall", "x": 0, "y": 0})
	checks["anchor on a wall"] = _one(MapCatalog.validate_entry(t), "Anchor 'a_wall' is not on an open tile.")
	t = valid.duplicate(true)
	var first_anchor: Dictionary = DictUtil.dict_of(DictUtil.array_of(t["anchors"])[0])
	first_anchor["id"] = "start"
	checks["missing spawn"] = _one(MapCatalog.validate_entry(t), "Every map needs a 'spawn' anchor.")
	t = valid.duplicate(true)
	DictUtil.array_of(t["anchors"]).append({"id": "a_npc", "x": 5, "y": 5})
	checks["duplicate anchor"] = _one(MapCatalog.validate_entry(t), "used twice")
	t = valid.duplicate(true)
	var paths: Dictionary = t["paths"]
	paths["loop"] = [[6, 5], [7, 6]]
	checks["diagonal path step"] = _one(MapCatalog.validate_entry(t), "Path 'loop' step 1 is not next to the step before.")
	t = valid.duplicate(true)
	paths = t["paths"]
	paths["wet"] = [[4, 3]]
	checks["path point on water"] = _one(MapCatalog.validate_entry(t), "Path 'wet' step 0 is not on an open tile.")
	t = valid.duplicate(true)
	DictUtil.array_of(t["zones"]).pop_back()
	checks["zone grid of another size"] = _one(MapCatalog.validate_entry(t), "Zones must match the rows (10 x 8).")
	t = valid.duplicate(true)
	var zones: Array = t["zones"]
	zones[1] = "qsss-ddddd"
	checks["unknown zone char"] = _one(MapCatalog.validate_entry(t), "Zone char 'q' at (0, 1) is not in the legend.")
	t = valid.duplicate(true)
	var lg: Dictionary = t["zone_legend"]
	lg["-"] = {"id": "dash", "kind": "safe"}
	checks["legend key '-'"] = _one(MapCatalog.validate_entry(t), "Zone legend keys are single characters")
	t = valid.duplicate(true)
	lg = t["zone_legend"]
	var deep: Dictionary = lg["e"]
	deep["kind"] = "calm"
	checks["zone kind calm"] = _one(MapCatalog.validate_entry(t), "Zone 'deep': kind must be safe or danger.")
	t = valid.duplicate(true)
	t["color"] = "red"
	checks["unknown key"] = _one(MapCatalog.validate_entry(t), "Unknown map field 'color'.")
	checks["upsert valid"] = MapCatalog.upsert(valid).is_empty() and MapCatalog.upsert(_small_map()).is_empty()
	checks["upsert refuses a broken one"] = not MapCatalog.upsert(_rows_with(valid, 2, "#.x......#")).is_empty() and MapCatalog.validate_all().is_empty()
	checks["duplicate id refused"] = _one(MapCatalog.validate_entry(_small_map().merged({"id": "m_field"}, true), "m_small"), "Map id 'm_field' is already used.")
	var before: Array[Dictionary] = MapCatalog.entries()
	checks["save to the sandbox"] = MapCatalog.save() == OK and MapCatalog.file_path().begins_with("user://")
	MapCatalog.reload_from_disk()
	checks["reload equal and clean"] = JsonUtil.deep_equal(before, MapCatalog.entries()) and MapCatalog.validate_all().is_empty()
	return _report("Check 1: map templates (one error per broken rule; valid file round-trips)", checks)

# ------------------------------------------------------------------------------
# Check 2: zone bindings
# ------------------------------------------------------------------------------

static func _test_bindings(fx: Dictionary) -> bool:
	var checks: Dictionary = {}
	var loc: Dictionary = _field(fx)
	var good: Dictionary = {"encounters": [["enc_a", 1]]}
	checks["unknown zone"] = _one(_loc_errs(_with_binding(loc, "nowhere", good)), "Zone 'nowhere' is not in map 'm_field'.")
	checks["safe zone bound"] = _one(_loc_errs(_with_binding(loc, "camp", good)), "Zone 'camp' is safe: safe zones roll nothing.")
	checks["weight 0"] = _one(_loc_errs(_with_binding(loc, "wilds", {"encounters": [["enc_a", 0]]})), "weight")
	checks["weight 101"] = _one(_loc_errs(_with_binding(loc, "wilds", {"encounters": [["enc_a", MapRules.WEIGHT_MAX + 1]]})), "weight")
	checks["unknown encounter"] = _one(_loc_errs(_with_binding(loc, "wilds", {"encounters": [["enc_zz", 1]]})), "unknown encounter 'enc_zz'")
	checks["steps_min 21 > steps_max 20"] = _one(_loc_errs(_with_binding(loc, "wilds", {"encounters": [["enc_a", 1]], "steps_min": 21, "steps_max": 20})), "steps_min is above steps_max.")
	checks["empty binding"] = _one(_loc_errs(_with_binding(loc, "wilds", {})), "Zone binding 'wilds' binds no encounter.")
	checks["unknown key"] = _one(_loc_errs(_with_binding(loc, "wilds", {"encounters": [["enc_a", 1]], "rate": 5})), "Zone binding 'wilds': unknown key 'rate'.")
	checks["duplicate ref"] = _one(_loc_errs(_with_binding(loc, "wilds", {"encounters": [["enc_a", 1], ["enc_a", 2]]})), "twice")
	checks["elites only is fine"] = _loc_errs(_with_binding(loc, "wilds", {"elites": [["elite_a", 1]]})).is_empty()
	var warns: Array[String] = MapRules.location_warnings(WorldSchema.normalize("locations", loc))
	checks["unbound danger zone: warning only"] = _loc_errs(loc).is_empty() and warns.size() == 1 and warns[0].contains("'deep'")
	checks["world warnings carry the qid"] = TestHelpers.any_contains(MapRules.warnings(PARENT), "%s:field: Danger zone 'deep'" % PARENT)
	return _report("Check 2: zone bindings (errors; an unbound DANGER zone is a warning)", checks)

# ------------------------------------------------------------------------------
# Check 3: markers
# ------------------------------------------------------------------------------

static func _test_markers(fx: Dictionary) -> bool:
	var checks: Dictionary = {}
	var loc: Dictionary = _field(fx)
	checks["unknown anchor"] = _one(_loc_errs(_with_marker(loc, "guide", {"anchor": "nope"})), "Marker 'guide': unknown anchor 'nope'.")
	checks["two markers on one anchor"] = _one(_loc_errs(_with_marker(loc, "mark", {"anchor": "a_npc"})), "already holds marker")
	checks["marker on spawn"] = _one(_loc_errs(_with_marker(loc, "mark", {"anchor": "spawn"})), "holds no marker")
	checks["roamer with an anchor"] = _one(_loc_errs(_with_marker(loc, "patrol", {"anchor": "a_point"})), "a roamer takes no anchor")
	checks["unknown path"] = _one(_loc_errs(_with_marker(loc, "patrol", {"path": "nope"})), "Unknown path 'nope'.")
	checks["exit to not in exits"] = _one(_loc_errs(_with_marker(loc, "to_town", {"to": "hut"})), "Exit 'hut' is not in this location's exits.")
	checks["to_anchor unknown on the target map"] = _one(_loc_errs(_with_marker(loc, "to_keep", {"to_anchor": "nope"})), "to_anchor 'nope'")
	checks["to_anchor on a map-less target must be spawn"] = _one(_loc_errs(_with_marker(loc, "to_town", {"to_anchor": "gate"})), "to_anchor must be 'spawn'")
	checks["world_map exit needs no exits entry"] = _loc_errs(_with_marker(loc, "to_town", {"to": MapRules.WORLD_MAP})).is_empty()
	checks["chest with nothing"] = _one(_loc_errs(_with_marker(loc, "box", {"items": {}, "loot": ""})), "Chest 'box' holds nothing.")
	checks["chest with passive loot"] = _one(_loc_errs(_with_marker(loc, "box", {"loot": "loot_passive"})), "Chest loot must be an item entry.")
	checks["radius 4"] = _one(_loc_errs(_with_marker(loc, "ambush", {"radius": MapRules.MAX_RADIUS + 1})), "radius must be 0..3")
	checks["unknown kind"] = _one(_loc_errs(_with_marker(loc, "mark", {"kind": "statue"})), "unknown kind 'statue'")
	checks["missing required ref"] = _one(_loc_errs(_without_key(loc, "guide", "character")), "choose a character")
	for mid: String in ["to_town", "ambush", "patrol", "guide", "box", "stone", "mark"]:
		var kind: String = str(DictUtil.dict_of(DictUtil.array_of(loc["markers"])[_marker_index(loc, mid)])["kind"])
		checks["unknown key on %s" % kind] = _one(_loc_errs(_with_marker(loc, mid, {"bogus": 1})), "Marker '%s' (%s): unknown key 'bogus'." % [mid, kind])
	checks["bad flag condition"] = _one(_loc_errs(_with_marker(loc, "mark", {"sets_flags": [{"key": "Bad Key"}]})), "flag key")
	var menu: Dictionary = {"id": "town", "name": "Town", "exits": [{"to": "field"}], "markers": [{"id": "p", "kind": "point", "anchor": "a_point"}]}
	checks["map '' with markers"] = _one(_loc_errs(menu), "Zone bindings and markers need a map.")
	checks["unknown map template"] = _one(_loc_errs(loc.merged({"map": "m_none"}, true)), "Unknown map template 'm_none'.")
	var no_marker: Dictionary = loc.duplicate(true)
	DictUtil.array_of(no_marker["markers"]).remove_at(_marker_index(no_marker, "to_town"))
	checks["exit without a marker: warning only"] = _loc_errs(no_marker).is_empty() and TestHelpers.any_contains(MapRules.location_warnings(no_marker), "Exit to 'town' has no marker on the map.")
	return _report("Check 3: markers (one error per broken rule; map-less locations hold none)", checks)

# ------------------------------------------------------------------------------
# Check 4: start_location
# ------------------------------------------------------------------------------

static func _test_start_location() -> bool:
	var checks: Dictionary = {}
	checks["empty everywhere = none"] = MapRules.start_location(CHILD).is_empty()
	checks["set unknown"] = WorldCatalog.update_manifest(PARENT, {"start_location": "nowhere"}).is_empty()
	checks["unknown start: manifest error"] = TestHelpers.any_contains(WorldCatalog.manifest_errors(PARENT), "Start location 'nowhere' is unknown.")
	checks["set field"] = WorldCatalog.update_manifest(PARENT, {"start_location": "field"}).is_empty() and WorldCatalog.manifest_errors(PARENT).is_empty()
	checks["parent resolves its own"] = MapRules.start_location(PARENT) == WorldCatalog.qid(PARENT, "field")
	checks["child with '' inherits the parent's"] = str(WorldCatalog.manifest(CHILD)["start_location"]).is_empty() and MapRules.start_location(CHILD) == WorldCatalog.qid(PARENT, "field") and WorldCatalog.manifest_errors(CHILD).is_empty()
	var local: String = WorldCatalog.override_entry(CHILD, "locations", WorldCatalog.qid(PARENT, "field"))
	checks["a child override of the start is followed"] = not local.is_empty() and MapRules.start_location(CHILD) == WorldCatalog.qid(CHILD, local) and MapRules.start_location(PARENT) == WorldCatalog.qid(PARENT, "field")
	checks["child's own value wins"] = WorldCatalog.update_manifest(CHILD, {"start_location": "town"}).is_empty() and MapRules.start_location(CHILD) == WorldCatalog.qid(PARENT, "town")
	checks["reset child"] = WorldCatalog.update_manifest(CHILD, {"start_location": ""}).is_empty()
	return _report("Check 4: manifest start_location (unknown = error; '' inherits the nearest ancestor's)", checks)

# ------------------------------------------------------------------------------
# Check 5: reference scans
# ------------------------------------------------------------------------------

static func _test_ref_scans() -> bool:
	var checks: Dictionary = {}
	var users: Array[Dictionary] = WorldCatalog.references_to(PARENT, "encounters", "enc_a")
	var keys: Array[String] = []
	for u: Dictionary in users:
		if str(u["world"]) == PARENT and str(u["id"]) == "field":
			keys.append(str(u["key"]))
	checks["scan finds the binding and the marker"] = keys.has("zone_bindings") and keys.has("markers")
	var renamed: Array[String] = WorldCatalog.upsert(PARENT, "encounters", {"id": "enc_a2", "name": "enc_a"}, "enc_a")
	var field: Dictionary = WorldCatalog.get_own(PARENT, "locations", "field")
	var pair: Array = DictUtil.array_of(DictUtil.array_of(DictUtil.dict_of(DictUtil.dict_of(field["zone_bindings"])["wilds"])["encounters"])[0])
	var marker: Dictionary = DictUtil.dict_of(DictUtil.array_of(field["markers"])[_marker_index(field, "ambush")])
	checks["rename rewrites the binding"] = renamed.is_empty() and str(pair[0]) == "enc_a2" and DictUtil.to_int(pair[1]) == 3
	checks["rename rewrites the marker"] = str(marker["encounter"]) == "enc_a2"
	checks["renamed location still validates"] = _loc_errs(field).is_empty()
	checks["deleting the lore reports the lore marker"] = WorldCatalog.remove(PARENT, "lore", "lore_a").contains("locations %s:field" % PARENT)
	var lore_keys: Array[String] = []
	for u: Dictionary in WorldCatalog.references_to(PARENT, "lore", "lore_a"):
		lore_keys.append(str(u["key"]))
	checks["lore scan key is markers"] = lore_keys.has("markers")
	checks["character and loot are scanned"] = _scan_has(PARENT, "characters", "npc_a", "markers") and _scan_has(PARENT, "loot", "loot_item", "markers")
	checks["rename a location rewrites the exit marker with the exits"] = WorldCatalog.upsert(PARENT, "locations", {"id": "town2", "name": "Town", "exits": [{"to": "field"}]}, "town").is_empty()
	field = WorldCatalog.get_own(PARENT, "locations", "field")
	var exit_m: Dictionary = DictUtil.dict_of(DictUtil.array_of(field["markers"])[_marker_index(field, "to_town")])
	checks["exit marker to follows"] = str(exit_m["to"]) == "town2" and MapRules.exit_targets(field).has("town2") and _loc_errs(field).is_empty()
	return _report("Check 5: reference scans (renames rewrite bindings and markers; deletes report markers)", checks)

static func _scan_has(world: String, kind: String, local: String, key: String) -> bool:
	for u: Dictionary in WorldCatalog.references_to(world, kind, local):
		if str(u["key"]) == key and str(u["kind"]) == "locations":
			return true
	return false
