class_name ExplorationTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Exploration Test Suite (S9, spec EXP1-L)
# Schema (checks 1-5): map template validation (MapCatalog / MapRules), zone bindings, markers, the world manifest
# `start_location` and the reference scans through the new location fields.
# Runtime (checks 6-20): ExploreRunner step order, counter, triggers, roamers, once markers, flee / defeat, entry,
# exits, chests, flag markers, BattleBridge, GameRolls determinism and save v3.
# Fixtures: MapCatalog and QuestCatalog on sandbox files, hand-built worlds in an EMPTY sandbox world root, saves in
# a sandbox folder; the shipped catalog is only read (units and items picked by rule), shipped data is never written.
# ==============================================================================

const TAG: String = "Explore"
static var SANDBOX: String = TestHelpers.sandbox("explore")
const PARENT: String = "xa_base"
const CHILD: String = "xa_child"
const RUN: String = "xr_base"   # runtime fixture world
const FAR: String = "xr_far"    # connected (not inherited) by RUN
const KID: String = "xr_kid"    # extends RUN
const SEED: int = 4242
static var _rx: Dictionary = {}   # runtime content picks: party / enemy unit, two items

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

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING EXPLORATION TEST (EXP1-L)               ===")
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
		ok = _test_marker_text(fx) and ok
		ok = _test_start_location() and ok
		ok = _test_ref_scans() and ok
		ok = _test_start_refs_and_used_by() and ok
		QuestCatalog.use_path(SANDBOX + "/quests.json")
		SaveGame.set_base_dir(SANDBOX + "/saves")
		if _build_runtime():
			ok = _test_step_order() and ok
			ok = _test_counter() and ok
			ok = _test_one_fight() and ok
			ok = _test_priority() and ok
			ok = _test_determinism() and ok
			ok = _test_elites() and ok
			ok = _test_roamer() and ok
			ok = _test_once() and ok
			ok = _test_flee_defeat() and ok
			ok = _test_entry() and ok
			ok = _test_exits() and ok
			ok = _test_chest() and ok
			ok = _test_marker_view() and ok
			ok = _test_map_read(tree_parent) and ok   # MAPREAD1-L2: icons, hint, legend, notes on the marker_view fixture
			ok = _test_flag_markers() and ok
			ok = _test_bridge() and ok
			ok = _test_save_v3() and ok
			ok = _test_debug_scene(tree_parent) and ok
			ok = MapViewChecks.run_pure() and ok   # MAPVIEW1: camera and world layout (pure)
			ok = MapReadChecks.run_pure() and ok   # MAPREAD1-L2: icon keys, placement rules (pure)
		else:
			ok = false
		SaveGame.reset_base_dir()
		QuestCatalog.use_path(QuestCatalog.DEFAULT_PATH)
	MapCatalog.use_path(MapCatalog.DEFAULT_PATH)
	TestHelpers.cleanup_worlds(SANDBOX)   # back to the shipped worlds; removes the whole sandbox (map, quest, saves)
	return ok

static func _test_shipped() -> bool:
	var checks: Dictionary = {}
	checks["shipped maps load"] = MapCatalog.load_errors().is_empty() and not MapCatalog.save_blocked()
	checks["shipped maps validate"] = MapCatalog.validate_all().is_empty()
	checks["shipped worlds validate with the new fields"] = WorldCatalog.validate_all().is_empty()
	var d: Dictionary = WorldSchema.defaults("locations")
	checks["location defaults = menu location"] = str(d["map"]).is_empty() and DictUtil.dict_of(d["zone_bindings"]).is_empty() and DictUtil.array_of(d["markers"]).is_empty()
	checks["world default start_location empty"] = str(WorldSchema.defaults("world")["start_location"]).is_empty()
	checks["marker label/note and map_pos defaults"] = MapRules.marker_full({"kind": "point"}).get("label", null) == "" and MapRules.marker_full({"kind": "point"}).get("note", null) == "" and DictUtil.array_of(d["map_pos"]).is_empty()
	checks["shipped marker views all have a label, verb and icon"] = _shipped_views_ok()
	return _report("Shipped data validates (maps file, worlds with the new location and manifest fields)", checks)

# MAPREAD1-L1: every marker of every shipped world has a non-empty label, verb and icon; a point whose sets_flags
# all hold reports spent (a fresh throwaway GameState per point; shipped data has at least one such point).
static func _shipped_views_ok() -> bool:
	var ok: bool = true
	var spent_seen: bool = false
	for w: String in WorldCatalog.world_ids():
		var gs: GameState = GameState.new()
		gs.world_id = w
		for loc: Dictionary in WorldCatalog.merged_entries(w, "locations"):
			for m: Dictionary in ExploreRunner.markers_of(loc):
				var v: Dictionary = ExploreRunner.marker_view(gs, loc, m)
				if str(v["label"]).is_empty() or str(v["verb"]).is_empty() or str(v["icon"]).is_empty():
					print("      empty marker view: %s %s" % [str(loc.get("_qid", "")), str(m["id"])])
					ok = false
				if str(m["kind"]) == "point" and not DictUtil.array_of(m["sets_flags"]).is_empty():
					var fresh: GameState = GameState.new()   # flags set here must not leak into other markers
					fresh.world_id = w
					var before: String = str(ExploreRunner.marker_view(fresh, loc, m)["state"])
					for c: Variant in DictUtil.array_of(m["sets_flags"]):
						var cond: Dictionary = DictUtil.dict_of(c)
						fresh.flags[str(cond["key"])] = cond.get("value", true)
					if before == "present" and str(ExploreRunner.marker_view(fresh, loc, m)["state"]) == "spent":
						spent_seen = true
	return ok and spent_seen

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
	# TRAV1-L: a world_map exit needs the location to be a route node; a temporary route makes `field` one.
	var route_errs: Array[String] = WorldCatalog.upsert(PARENT, "routes", {"id": "r_wm", "name": "Road", "from": "field", "to": "town"})
	checks["world_map exit needs no exits entry"] = route_errs.is_empty() and _loc_errs(_with_marker(loc, "to_town", {"to": MapRules.WORLD_MAP})).is_empty()
	checks["world_map exit arrives at spawn"] = _one(_loc_errs(_with_marker(loc, "to_town", {"to": MapRules.WORLD_MAP, "to_anchor": "gate"})), "a world_map exit always arrives at 'spawn'")
	checks["temporary route removed"] = WorldCatalog.remove(PARENT, "routes", "r_wm").is_empty()
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
	var typed: Dictionary = {"id": "town", "name": "Town", "exits": [{"to": "field"}], "zone_bindings": [], "markers": {}}
	var typed_errs: Array[String] = _loc_errs(typed)
	checks["map '' with wrong-typed fields: type errors"] = typed_errs.size() == 2 and TestHelpers.any_contains(typed_errs, "Zone bindings must be an object") and TestHelpers.any_contains(typed_errs, "Markers must be a list.")
	checks["wrong-typed markers with a map"] = _one(_loc_errs(loc.merged({"markers": {}}, true)), "Markers must be a list.")
	checks["unknown map template"] = _one(_loc_errs(loc.merged({"map": "m_none"}, true)), "Unknown map template 'm_none'.")
	var no_marker: Dictionary = loc.duplicate(true)
	DictUtil.array_of(no_marker["markers"]).remove_at(_marker_index(no_marker, "to_town"))
	checks["exit without a marker: warning only"] = _loc_errs(no_marker).is_empty() and TestHelpers.any_contains(MapRules.location_warnings(no_marker), "Exit to 'town' has no marker on the map.")
	return _report("Check 3: markers (one error per broken rule; map-less locations hold none)", checks)

# MAPREAD1-L1: marker label / note and location map_pos validation.
static func _test_marker_text(fx: Dictionary) -> bool:
	var checks: Dictionary = {}
	var loc: Dictionary = _field(fx)
	var long_label: String = "x".repeat(MapRules.LABEL_MAX + 1)
	checks["label of max length is fine"] = _loc_errs(_with_marker(loc, "mark", {"label": "x".repeat(MapRules.LABEL_MAX), "note": "n".repeat(MapRules.NOTE_MAX)})).is_empty()
	checks["33-character label"] = _one(_loc_errs(_with_marker(loc, "mark", {"label": long_label})), "Marker 'mark': label is at most %d characters." % MapRules.LABEL_MAX)
	checks["note too long"] = _one(_loc_errs(_with_marker(loc, "mark", {"note": "n".repeat(MapRules.NOTE_MAX + 1)})), "Marker 'mark': note is at most %d characters." % MapRules.NOTE_MAX)
	checks["note on a chest"] = _one(_loc_errs(_with_marker(loc, "box", {"note": "Hi"})), "Marker 'box': only points take a note.")
	checks["label on a chest is fine"] = _loc_errs(_with_marker(loc, "box", {"label": "Old box"})).is_empty()
	checks["non-text label"] = _one(_loc_errs(_with_marker(loc, "mark", {"label": 5})), "label must be text")
	checks["defaults filled"] = str(MapRules.marker_full({"kind": "chest"}).get("label", "x")).is_empty() and str(MapRules.marker_full({"kind": "chest"}).get("note", "x")).is_empty()
	var good: Array = [[], [0, 1], [0.25, 0.75], [1.0, 0.0]]
	for p: Variant in good:
		checks["map_pos %s ok" % str(p)] = _loc_errs(loc.merged({"map_pos": p}, true)).is_empty()
	var bad: Array = [[0.5], [0.1, 0.2, 0.3], [1.2, 0.5], [-0.1, 0.5], ["a", "b"], {"x": 1}]
	for p: Variant in bad:
		checks["map_pos %s rejected" % str(p)] = _one(_loc_errs(loc.merged({"map_pos": p}, true)), "Map position must be empty or two numbers [x, y] between 0 and 1.")
	checks["map-less location may carry map_pos"] = _loc_errs({"id": "town", "name": "Town", "exits": [{"to": "field"}], "map_pos": [0.5, 0.5]}).is_empty()
	return _report("MAPREAD1-L1: marker label / note and location map_pos validation", checks)

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

# Review folds: start_location in the ref scans (scan, rename, delete) and MapCatalog used-by on remove / rename.
static func _test_start_refs_and_used_by() -> bool:
	var checks: Dictionary = {}
	checks["inn built"] = WorldCatalog.upsert(PARENT, "locations", {"id": "inn", "name": "Inn"}).is_empty() and WorldCatalog.update_manifest(CHILD, {"start_location": "inn"}).is_empty()
	var found: bool = false
	for u: Dictionary in WorldCatalog.references_to(PARENT, "locations", "inn"):
		found = found or (str(u["kind"]) == "world" and str(u["world"]) == CHILD and str(u["key"]) == "start_location")
	checks["scan finds the child manifest"] = found
	checks["rename inn"] = WorldCatalog.upsert(PARENT, "locations", {"id": "inn2", "name": "Inn"}, "inn").is_empty()
	checks["rename rewrites start_location (qualified from the child)"] = str(WorldCatalog.manifest(CHILD)["start_location"]) == WorldCatalog.qid(PARENT, "inn2") and MapRules.start_location(CHILD) == WorldCatalog.qid(PARENT, "inn2") and WorldCatalog.manifest_errors(CHILD).is_empty()
	checks["rename the parent's own start keeps it local"] = WorldCatalog.upsert(PARENT, "locations", {"id": "field_b", "name": "Field", "map": "m_field", "exits": WorldCatalog.get_own(PARENT, "locations", "field")["exits"]}, "field").is_empty() and str(WorldCatalog.manifest(PARENT)["start_location"]) == "field_b"
	var refused: String = WorldCatalog.remove(PARENT, "locations", "inn2")
	checks["delete names the manifest"] = refused.contains("world %s start_location" % CHILD)
	checks["delete after reset"] = WorldCatalog.update_manifest(CHILD, {"start_location": ""}).is_empty() and WorldCatalog.remove(PARENT, "locations", "inn2").is_empty()
	var users: Array[String] = MapCatalog.used_by("m_field")
	checks["used_by lists the locations"] = users.has(WorldCatalog.qid(PARENT, "field_b"))
	checks["remove refused while used"] = MapCatalog.remove("m_field").contains("still used by") and MapCatalog.has("m_field")
	var renamed: Dictionary = _field_map()
	renamed["id"] = "m_field_x"
	checks["rename refused while used"] = TestHelpers.any_contains(MapCatalog.upsert(renamed, "m_field"), "is used by") and MapCatalog.has("m_field") and not MapCatalog.has("m_field_x")
	var spare: Dictionary = _small_map()
	spare["id"] = "m_spare"
	checks["unused template: add, rename, remove"] = MapCatalog.upsert(spare).is_empty() and MapCatalog.upsert(_small_map().merged({"id": "m_spare2"}, true), "m_spare").is_empty() and not MapCatalog.has("m_spare") and MapCatalog.remove("m_spare2").is_empty()
	return _report("Review folds: start_location ref scans; map templates report the locations using them", checks)

static func _scan_has(world: String, kind: String, local: String, key: String) -> bool:
	for u: Dictionary in WorldCatalog.references_to(world, kind, local):
		if str(u["key"]) == key and str(u["kind"]) == "locations":
			return true
	return false

# ==============================================================================
# Runtime fixtures (checks 6-20)
# ==============================================================================

const R: Vector2i = Vector2i(1, 0)
const L: Vector2i = Vector2i(-1, 0)
const U: Vector2i = Vector2i(0, -1)
const D: Vector2i = Vector2i(0, 1)

static func _q(local: String, world: String = RUN) -> String:
	return WorldCatalog.qid(world, local)

# Checked cast of a result dict's "encounter" (null when it is not a GameEncounter).
static func _encounter_of(v: Variant) -> GameEncounter:
	if v is GameEncounter:
		return v
	return null

# 12 x 10: walls round the edge except an open edge tile (0, 4); x 1..3 SAFE (no zone), x 4..7 `wilds` (DANGER,
# bound by default), x 8..10 `fen` (DANGER, never bound).
static func _run_map(id: String = "m_run", danger_everywhere: bool = false) -> Dictionary:
	var rows: Array = ["############"]
	var zones: Array = ["------------"]
	for y: int in range(1, 9):
		rows.append(("." if y == 4 else "#") + "..........#")
		zones.append("-dddddddddd-" if danger_everywhere else "----dddduuu-")
	rows.append("############")
	zones.append("------------")
	var anchors: Array = []
	var spots: Dictionary = {
		"spawn": [1, 1], "a_edge": [0, 4], "a_npc": [2, 3], "a_chest": [2, 5], "a_lore": [3, 7], "a_point": [1, 7], "a_exit": [1, 8],
		"a_far": [2, 8], "a_wm": [3, 8], "a_lock": [3, 1], "a_touch": [5, 2], "a_px": [5, 3], "a_near": [2, 2], "a_gate": [10, 1],
	}
	for key: Variant in spots:
		var xy: Array = spots[key]
		anchors.append({"id": str(key), "x": xy[0], "y": xy[1]})
	return {
		"id": id, "name": id, "rows": rows, "zones": zones, "anchors": anchors,
		"zone_legend": {"d": {"id": "wilds", "kind": "danger"}, "u": {"id": "fen", "kind": "danger"}},
		"paths": {"p4": [[5, 5], [6, 5], [7, 5], [7, 6]], "near": [[6, 2]], "p1": [[9, 7]]},
	}

static func _default_bindings() -> Dictionary:
	return {"wilds": {"encounters": [["enc_a", 1]]}}

# Stores RUN's `field` (map m_run) with these markers / bindings.
static func _set_field(markers: Array, bindings: Dictionary = {}) -> bool:
	var entry: Dictionary = {
		"id": "field", "name": "Field", "map": "m_run",
		"exits": [{"to": "town"}, {"to": "keep"}, {"to": _q("dock", FAR)}],
		"zone_bindings": bindings if not bindings.is_empty() else _default_bindings(), "markers": markers,
	}
	var errors: Array[String] = WorldCatalog.upsert(RUN, "locations", entry)
	var full: Array[String] = _loc_errs(WorldCatalog.get_own(RUN, "locations", "field"), RUN)
	if not errors.is_empty() or not full.is_empty():
		print("      field fixture errors: %s %s" % [str(errors), str(full)])
	return errors.is_empty() and full.is_empty()

static func _party_unit() -> String:
	for e: Dictionary in ContentCatalog.entries("units"):
		if str(e.get("side", "")) == "party" and ContentFactory.class_of(e) != null and str(e.get("stat_mode", "derived")) == "derived":
			return str(e["id"])
	return ""

static func _enemy_unit() -> String:
	for e: Dictionary in ContentCatalog.entries("units"):
		if str(e.get("side", "")) == "enemy" and ContentFactory.class_of(e) != null:
			return str(e["id"])
	return ""

static func _build_runtime() -> bool:
	var items: Array[String] = ContentCatalog.ids("items")
	_rx = {"party": _party_unit(), "enemy": _enemy_unit(), "item0": items[0] if items.size() > 1 else "", "item1": items[1] if items.size() > 1 else ""}
	var errors: Array[String] = []
	for key: Variant in _rx:
		if str(_rx[key]).is_empty():
			errors.append("no %s picked" % str(key))
	errors.append_array(MapCatalog.upsert(_run_map()))
	errors.append_array(MapCatalog.upsert(_run_map("m_danger", true)))
	for t: Dictionary in [
		{"id": "t_reach", "name": "Reach", "objectives": [{"kind": "reach_location"}]},
		{"id": "t_reach2", "name": "Reach twice", "objectives": [{"kind": "reach_location"}, {"kind": "reach_location"}]},
		{"id": "t_flag", "name": "Flag", "objectives": [{"kind": "flag"}]},
		{"id": "t_win", "name": "Win", "objectives": [{"kind": "win_encounter"}]},
	]:
		errors.append_array(QuestCatalog.upsert(t))
	errors.append_array(WorldCatalog.create_world(FAR, "XR Far"))
	errors.append_array(WorldCatalog.create_world(RUN, "XR Base"))
	errors.append_array(WorldCatalog.create_world(KID, "XR Kid", [RUN]))
	errors.append_array(WorldCatalog.update_manifest(RUN, {"connects": [FAR], "start_location": "town"}))
	errors.append_array(WorldCatalog.upsert(FAR, "locations", {"id": "dock", "name": "Dock"}))
	errors.append_array(WorldCatalog.upsert(RUN, "characters", {"id": "hero", "name": "Hero XR", "role": "party", "base_unit": str(_rx["party"])}))
	errors.append_array(WorldCatalog.upsert(RUN, "characters", {"id": "foe", "name": "Foe XR", "role": "enemy", "base_unit": str(_rx["enemy"])}))
	errors.append_array(WorldCatalog.upsert(RUN, "characters", {"id": "sage", "name": "Sage XR", "role": "npc", "base_unit": str(_rx["party"])}))
	for id: String in ["enc_a", "enc_b", "elite_a"]:
		errors.append_array(WorldCatalog.upsert(RUN, "encounters", {"id": id, "name": id, "party": ["hero"], "enemies": ["foe"], "max_rounds": 2}))
	errors.append_array(WorldCatalog.upsert(RUN, "lore", {"id": "stone", "name": "Old Stone", "text": "Carved runes."}))
	errors.append_array(WorldCatalog.upsert(RUN, "loot", {"id": "lt_box", "name": "Box Loot", "reward_kind": "item", "ref": str(_rx["item1"]), "count": 1}))
	errors.append_array(WorldCatalog.upsert(RUN, "locations", {"id": "town", "name": "Town", "exits": [{"to": "field", "label": "to the field"}]}))
	errors.append_array(WorldCatalog.upsert(RUN, "locations", {"id": "keep", "name": "Keep", "map": "m_danger", "exits": [{"to": "field"}], "zone_bindings": _default_bindings()}))
	# TRAV1-L: the field's world_map exit marker (check 16) needs the field to be a route node (keep, not the map-less
	# town, so the town's menu lists no world map entry).
	errors.append_array(WorldCatalog.upsert(RUN, "routes", {"id": "r_field_keep", "name": "Field road", "from": "field", "to": "keep", "encounter_pct": 0}))
	var quests: Array[Dictionary] = [
		{"id": "q_reach", "name": "Reach", "template": "t_reach", "auto_start": true, "steps": [{"text": "Go", "location": "field"}]},
		{"id": "q_reach2", "name": "Reach twice", "template": "t_reach2", "auto_start": true, "steps": [{"text": "Go", "location": "field"}, {"text": "Again", "location": "field"}]},
		{"id": "q_point", "name": "Point", "template": "t_flag", "auto_start": true, "steps": [{"text": "Mark", "flag": "xr.point"}]},
		{"id": "q_lore", "name": "Lore", "template": "t_flag", "auto_start": true, "steps": [{"text": "Read", "flag": "xr.read"}]},
		{"id": "q_win", "name": "Win", "template": "t_win", "auto_start": true, "steps": [{"text": "Win", "encounter": "enc_a"}]},
	]
	for q: Dictionary in quests:
		errors.append_array(WorldCatalog.upsert(RUN, "quests", q))
	var field_ok: bool = _set_field([])
	if not errors.is_empty():
		print("      runtime fixture errors: %s" % str(errors))
	return _report("Runtime fixtures (maps, quest templates, worlds with connection and child, party picked by rule)", {"no errors": errors.is_empty(), "field": field_ok, "start": MapRules.start_location(KID) == _q("town")})

static func _new_gs(world: String = RUN) -> GameState:
	var res: Dictionary = NewGame.create(world, [str(_rx["party"]), str(_rx["party"])] as Array[String], SEED)
	return GameState.of(res["state"])

# A fresh state standing in RUN's field at (x, y) (entered at spawn first: hooks ran, last_safe = field).
static func _in_field(x: int = 1, y: int = 1, world: String = RUN) -> GameState:
	var gs: GameState = _new_gs(world)
	var _entered: Dictionary = ExploreRunner.enter(gs, "field")
	gs.position = Vector2i(x, y)
	return gs

static func _snap(gs: GameState) -> String:
	return JSON.stringify(gs.to_dict(), "", true)

static func _events(res: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for e: Variant in DictUtil.array_of(res.get("events", [])):
		out.append(DictUtil.dict_of(e))
	return out

static func _find(events: Array[Dictionary], kind: String) -> Dictionary:
	for e: Dictionary in events:
		if str(e.get("kind", "")) == kind:
			return e
	return {}

static func _count(events: Array[Dictionary], kind: String) -> int:
	var n: int = 0
	for e: Dictionary in events:
		if str(e.get("kind", "")) == kind:
			n += 1
	return n

static func _fight_of(gs: GameState, dir: Vector2i) -> Dictionary:
	return _find(_events(ExploreRunner.step(gs, dir)), "fight")

static func _steps(gs: GameState) -> int:
	return DictUtil.to_int(gs.explore["steps"])

static func _rolls(gs: GameState) -> int:
	return DictUtil.to_int(gs.rolls.get(GameRolls.EXPLORE, 0))

# ------------------------------------------------------------------------------
# Check 6: step order (blocked steps change nothing)
# ------------------------------------------------------------------------------

static func _test_step_order() -> bool:
	var checks: Dictionary = {}
	checks["field"] = _set_field([
		{"id": "sage", "kind": "npc", "anchor": "a_npc", "character": "sage"},
		{"id": "gate", "kind": "exit", "anchor": "a_lock", "to": "town", "requires_flags": [{"key": "xr.key"}], "locked_text": "Gate barred."},
		{"id": "patrol", "kind": "roamer", "encounter": "elite_a", "path": "p4"},
	])
	var gs: GameState = _in_field(1, 1)
	gs.explore["loc_steps"] = 2
	var before: String = _snap(gs)
	var wall: Array[Dictionary] = _events(ExploreRunner.step(gs, U))
	checks["into a wall: blocked, state equal"] = _find(wall, "blocked").size() > 0 and wall.size() == 1 and _snap(gs) == before
	gs.position = Vector2i(0, 4)
	before = _snap(gs)
	var off: Array[Dictionary] = _events(ExploreRunner.step(gs, L))
	checks["out of bounds: blocked, state equal"] = _find(off, "blocked").size() > 0 and _snap(gs) == before
	gs.position = Vector2i(2, 2)
	before = _snap(gs)
	var npc: Array[Dictionary] = _events(ExploreRunner.step(gs, D))
	checks["onto an npc: blocked, state equal (counters, rolls, roamer index)"] = str(_find(npc, "blocked").get("marker", "")) == "sage" and _snap(gs) == before
	gs.position = Vector2i(2, 1)
	before = _snap(gs)
	var locked: Dictionary = _find(_events(ExploreRunner.step(gs, R)), "locked")
	checks["locked exit: locked with its text, state equal"] = str(locked.get("text", "")) == "Gate barred." and _snap(gs) == before
	var unlocked: GameState = _in_field(2, 1)
	var _key: String = QuestRunner.set_flag(unlocked, "xr.key", true)
	var through: Dictionary = _find(_events(ExploreRunner.step(unlocked, R)), "exit")
	checks["the same exit opens once its flag holds"] = str(through.get("to", "")) == "town" and unlocked.location_id == _q("town")
	checks["bad direction refused"] = not DictUtil.to_bool(ExploreRunner.step(gs, Vector2i(1, 1))["ok"]) and _snap(gs) == before
	return _report("Check 6: step order (wall, edge, npc and locked exit change nothing)", checks)

# ------------------------------------------------------------------------------
# Check 7: the step counter
# ------------------------------------------------------------------------------

static func _test_counter() -> bool:
	var checks: Dictionary = {}
	checks["field"] = _set_field([])
	var gs: GameState = _in_field(1, 1)
	gs.explore["next_at"] = 3
	var safe: Array[Dictionary] = _events(ExploreRunner.step(gs, R)) + _events(ExploreRunner.step(gs, R))
	checks["SAFE steps change neither steps nor next_at"] = _steps(gs) == 0 and DictUtil.to_int(gs.explore["next_at"]) == 3 and _count(safe, "fight") == 0 and DictUtil.to_int(gs.explore["loc_steps"]) == 2
	var d1: Dictionary = _fight_of(gs, R)   # (4, 1) wilds
	checks["DANGER step counts"] = _steps(gs) == 1 and d1.is_empty()
	var back: Dictionary = _fight_of(gs, L)   # (3, 1) SAFE: no reset
	checks["SAFE step in between does not reset"] = _steps(gs) == 1 and back.is_empty()
	var d2: Dictionary = _fight_of(gs, R)
	checks["second DANGER step: no fight"] = _steps(gs) == 2 and d2.is_empty()
	var rolls_before: int = _rolls(gs)
	var d3: Dictionary = _fight_of(gs, R)   # (5, 1)
	checks["third DANGER step fights from the binding"] = str(d3.get("source", "")) == "counter" and str(d3.get("encounter", "")) == _q("enc_a") and not DictUtil.to_bool(d3.get("elite", true)) and _steps(gs) == 3
	checks["fight carries from and location"] = JsonUtil.deep_equal(d3.get("from", []), [4, 1]) and str(d3.get("location", "")) == _q("field")
	checks["the pick drew exactly one roll"] = _rolls(gs) == rolls_before + 1
	var fen: GameState = _in_field(8, 2)
	fen.explore["next_at"] = 1
	var fen_events: Array[Dictionary] = _events(ExploreRunner.step(fen, R)) + _events(ExploreRunner.step(fen, R)) + _events(ExploreRunner.step(fen, L))
	checks["an unbound DANGER zone never counts"] = _steps(fen) == 0 and _count(fen_events, "fight") == 0 and _rolls(fen) == 0
	return _report("Check 7: counter (SAFE steps neither count nor reset; third DANGER step fights; unbound never counts)", checks)

# ------------------------------------------------------------------------------
# Checks 8 and 9: one fight per step, trigger priority
# ------------------------------------------------------------------------------

static func _test_one_fight() -> bool:
	var checks: Dictionary = {}
	checks["field"] = _set_field([{"id": "ambush", "kind": "encounter", "anchor": "a_touch", "encounter": "enc_b"}])
	var gs: GameState = _in_field(4, 2)
	gs.explore["next_at"] = 1
	var rolls_before: int = _rolls(gs)
	var events: Array[Dictionary] = _events(ExploreRunner.step(gs, R))
	var fight: Dictionary = _find(events, "fight")
	checks["exactly one fight: the marker's"] = _count(events, "fight") == 1 and str(fight.get("source", "")) == "marker" and str(fight.get("marker", "")) == "ambush" and str(fight.get("encounter", "")) == _q("enc_b")
	checks["the ripe counter neither rolled nor reset"] = _steps(gs) == 1 and _rolls(gs) == rolls_before
	var after: Dictionary = ExploreRunner.after_fight(gs, fight, "fled")
	var next_at: int = DictUtil.to_int(gs.explore["next_at"])
	checks["after_fight resets the counter"] = DictUtil.to_bool(after["ok"]) and _steps(gs) == 0 and next_at >= MapRules.DEFAULT_STEPS_MIN and next_at <= MapRules.DEFAULT_STEPS_MAX
	return _report("Check 8: one fight per step (a touch marker beats a ripe counter; steps reset only by after_fight)", checks)

static func _test_priority() -> bool:
	var checks: Dictionary = {}
	var touch: Dictionary = {"id": "touch", "kind": "encounter", "anchor": "a_touch", "encounter": "enc_a"}
	var roamer: Dictionary = {"id": "rover", "kind": "roamer", "encounter": "elite_a", "path": "near", "radius": 1}
	var prox: Dictionary = {"id": "prox", "kind": "encounter", "anchor": "a_px", "encounter": "enc_b", "trigger": "proximity", "radius": 1}
	var cases: Array = [[[touch, roamer, prox], "marker", "touch"], [[roamer, prox], "roamer", "rover"], [[prox], "marker", "prox"]]
	for c: Variant in cases:
		var row: Array = c
		checks["field %s" % str(row[2])] = _set_field(row[0])
		var gs: GameState = _in_field(4, 2)
		var fight: Dictionary = _fight_of(gs, R)
		checks["%s wins" % str(row[2])] = str(fight.get("source", "")) == str(row[1]) and str(fight.get("marker", "")) == str(row[2])
	return _report("Check 9: trigger priority (touch marker, then roamer in radius, then proximity marker)", checks)

# ------------------------------------------------------------------------------
# Checks 10 and 11: determinism, elites
# ------------------------------------------------------------------------------

# Walks `dirs`, settling every fight as `result`; returns the events (JSON text per step) and the fight count.
static func _play(gs: GameState, dirs: Array[Vector2i], result: String = "won") -> Dictionary:
	var log: Array[String] = []
	var fights: int = 0
	for d: Vector2i in dirs:
		var events: Array[Dictionary] = _events(ExploreRunner.step(gs, d))
		log.append(JSON.stringify(events, "", true))
		var fight: Dictionary = _find(events, "fight")
		if not fight.is_empty():
			fights += 1
			log.append(JSON.stringify(_events(ExploreRunner.after_fight(gs, fight, result)), "", true))
	return {"log": log, "fights": fights}

static func _test_determinism() -> bool:
	var checks: Dictionary = {}
	checks["field"] = _set_field([], {"wilds": {"encounters": [["enc_a", 2], ["enc_b", 1]], "steps_min": 2, "steps_max": 3}})
	var base: GameState = _in_field(4, 2)
	base.explore["next_at"] = 2
	var dirs: Array[Vector2i] = []
	for i: int in range(24):
		dirs.append(R if i % 2 == 0 else L)
	var a: GameState = GameState.from_dict(base.to_dict())
	var b: GameState = GameState.from_dict(base.to_dict())
	checks["save the start"] = SaveGame.write(base, 1).is_empty()
	var loaded: GameState = GameState.of(SaveGame.read(1)["state"])
	var rolls_before: int = _rolls(a)
	var run_a: Dictionary = _play(a, dirs)
	var run_b: Dictionary = _play(b, dirs)
	var run_c: Dictionary = _play(loaded, dirs) if loaded != null else {}
	checks["fights happened"] = DictUtil.to_int(run_a["fights"]) >= 2
	checks["two copies: equal events"] = JsonUtil.deep_equal(run_a["log"], run_b["log"]) and a.equals(b)
	checks["save / load / repeat: same fights"] = loaded != null and JsonUtil.deep_equal(run_a["log"], run_c.get("log", [])) and a.equals(loaded)
	# Encounters only: one pick per counter fight plus one next_at roll per reset.
	checks["rolls.explore rises by exactly the rolls drawn"] = _rolls(a) == rolls_before + 2 * DictUtil.to_int(run_a["fights"])
	var other: GameState = GameState.from_dict(base.to_dict())
	other.rng_seed = SEED + 1
	checks["another seed gives another sequence"] = not JsonUtil.deep_equal(_play(other, dirs)["log"], run_a["log"])
	return _report("Check 10: determinism (copies and a saved / loaded copy replay the same fights; rolls counted)", checks)

static func _elite_flags(gs: GameState, n: int) -> Array[bool]:
	var out: Array[bool] = []
	for i: int in range(n):
		gs.explore["next_at"] = 1
		var fight: Dictionary = _fight_of(gs, R if gs.position.x == 4 else L)
		if fight.is_empty():
			return out
		out.append(DictUtil.to_bool(fight["elite"]) and str(fight["encounter"]) == _q("elite_a"))
		var _settled: Dictionary = ExploreRunner.after_fight(gs, fight, "fled")
	return out

static func _test_elites() -> bool:
	var checks: Dictionary = {}
	var cases: Array = [[100, true, "elite_pct 100: every fight elite"], [0, false, "elite_pct 0: none"]]
	for c: Variant in cases:
		var row: Array = c
		checks["field %d" % DictUtil.to_int(row[0])] = _set_field([], {"wilds": {"encounters": [["enc_a", 1]], "elites": [["elite_a", 1]], "elite_pct": row[0]}})
		var flags: Array[bool] = _elite_flags(_in_field(4, 2), 8)
		checks[str(row[2])] = flags.size() == 8 and not flags.has(not DictUtil.to_bool(row[1]))
	checks["field elites only"] = _set_field([], {"wilds": {"elites": [["elite_a", 1]]}})
	var only: Array[bool] = _elite_flags(_in_field(4, 2), 8)
	checks["elites only: always elite"] = only.size() == 8 and not only.has(false)
	return _report("Check 11: elite rule (pct 100 / 0, elites-only binding)", checks)

# ------------------------------------------------------------------------------
# Checks 12 and 13: roamers, once markers
# ------------------------------------------------------------------------------

static func _test_roamer() -> bool:
	var checks: Dictionary = {}
	var rover: Dictionary = {"id": "rover", "kind": "roamer", "encounter": "elite_a", "path": "p4"}
	var still: Dictionary = {"id": "still", "kind": "roamer", "encounter": "elite_a", "path": "p1"}
	checks["field"] = _set_field([rover, still])
	var gs: GameState = _in_field(1, 1)
	var t: Dictionary = MapCatalog.get_entry("m_run")
	var pts: Array[Vector2i] = MapRules.path_points(t, "p4")
	var m_rover: Dictionary = MapRules.marker_full(rover)
	var m_still: Dictionary = MapRules.marker_full(still)
	var seen: Array[int] = [pts.find(ExploreRunner.marker_pos(t, m_rover, DictUtil.to_int(gs.explore["loc_steps"])))]
	var stays: bool = true
	for i: int in range(6):
		var _moved: Dictionary = ExploreRunner.step(gs, R if i % 2 == 0 else L)
		var ls: int = DictUtil.to_int(gs.explore["loc_steps"])
		seen.append(pts.find(ExploreRunner.marker_pos(t, m_rover, ls)))
		stays = stays and ExploreRunner.marker_pos(t, m_still, ls) == Vector2i(9, 7)
	checks["4-point path visits 0,1,2,3,2,1,0 over loc_steps 0..6"] = seen == [0, 1, 2, 3, 2, 1, 0]
	checks["1-point path stays"] = stays
	var _first: Dictionary = ExploreRunner.step(gs, R)
	var _again: Dictionary = ExploreRunner.enter(gs, "field")
	checks["re-entering resets loc_steps and the roamer"] = DictUtil.to_int(gs.explore["loc_steps"]) == 0 and ExploreRunner.marker_pos(t, m_rover, 0) == pts[0]
	return _report("Check 12: roamers (ping-pong patrol, one-point path, reset on entry)", checks)

static func _test_once() -> bool:
	var checks: Dictionary = {}
	var ambush: Dictionary = {"id": "ambush", "kind": "encounter", "anchor": "a_touch", "encounter": "enc_a"}
	checks["field"] = _set_field([ambush])
	var m: Dictionary = MapRules.marker_full(ambush)
	var done_key: String = MapRules.done_flag(_q("field"), "ambush")
	for result: String in ["fled", "lost"]:
		var gs: GameState = _in_field(4, 2)
		var fight: Dictionary = _fight_of(gs, R)
		var _r: Dictionary = ExploreRunner.after_fight(gs, fight, result)
		checks["after %s it stays" % result] = not fight.is_empty() and not gs.has_flag(done_key) and ExploreRunner.presence(gs, _q("field"), m) == "present"
	var won: GameState = _in_field(4, 2)
	var wf: Dictionary = _fight_of(won, R)
	var _w: Dictionary = ExploreRunner.after_fight(won, wf, "won")
	checks["won: done flag set, marker absent"] = won.get_flag(done_key, false) == true and ExploreRunner.presence(won, _q("field"), m) == "absent"
	var _off: Dictionary = ExploreRunner.step(won, L)
	won.explore["next_at"] = 99
	checks["no second fight on the tile"] = _fight_of(won, R).is_empty()
	var repeat: Dictionary = {"id": "camp", "kind": "encounter", "anchor": "a_touch", "encounter": "enc_a", "once": false, "unless_flags": [{"key": "xr.cleared"}]}
	checks["field repeat"] = _set_field([repeat])
	var rm: Dictionary = MapRules.marker_full(repeat)
	var gs2: GameState = _in_field(4, 2)
	var f2: Dictionary = _fight_of(gs2, R)
	var _w2: Dictionary = ExploreRunner.after_fight(gs2, f2, "won")
	checks["once false: no done flag, still present"] = not gs2.has_flag(MapRules.done_flag(_q("field"), "camp")) and ExploreRunner.presence(gs2, _q("field"), rm) == "present"
	var _cleared: String = QuestRunner.set_flag(gs2, "xr.cleared", true)
	checks["unless flag set: absent"] = ExploreRunner.presence(gs2, _q("field"), rm) == "absent"
	return _report("Check 13: once markers (won sets the done flag; fled / lost keep it; unless_flags hide)", checks)

# ------------------------------------------------------------------------------
# Check 14: flee and defeat
# ------------------------------------------------------------------------------

static func _counter_ok(gs: GameState, lo: int, hi: int) -> bool:
	var n: int = DictUtil.to_int(gs.explore["next_at"])
	return _steps(gs) == 0 and n >= lo and n <= hi

static func _test_flee_defeat() -> bool:
	var checks: Dictionary = {}
	checks["field"] = _set_field([], {"wilds": {"encounters": [["enc_a", 1]], "steps_min": 5, "steps_max": 7}})
	var fled: GameState = _in_field(4, 2)
	fled.explore["next_at"] = 1
	var f: Dictionary = _fight_of(fled, R)
	var _r1: Dictionary = ExploreRunner.after_fight(fled, f, "fled")
	checks["fled: back on from"] = fled.position == Vector2i(4, 2) and not f.is_empty()
	checks["fled: next_at from the zone under the party"] = _counter_ok(fled, 5, 7)
	var lost: GameState = _in_field(5, 2)
	var fight: Dictionary = {"kind": "fight", "source": "counter", "encounter": _q("enc_a"), "marker": "", "elite": false, "from": [4, 2], "location": _q("field")}
	var res: Dictionary = ExploreRunner.after_fight(lost, fight, "lost")
	checks["lost with a SAFE spawn: wake at its spawn"] = lost.location_id == _q("field") and lost.position == Vector2i(1, 1) and _count(_events(res), "autosave") == 1
	checks["counter reset (SAFE spawn: default range)"] = _counter_ok(lost, MapRules.DEFAULT_STEPS_MIN, MapRules.DEFAULT_STEPS_MAX)
	var keep: GameState = _in_field(1, 1)
	var _k: Dictionary = ExploreRunner.enter(keep, "keep")
	checks["DANGER spawn keeps last_safe"] = str(keep.explore["last_safe"]) == _q("field") and keep.location_id == _q("keep")
	keep.position = Vector2i(5, 5)
	var _r2: Dictionary = ExploreRunner.after_fight(keep, fight, "lost")
	checks["lost with a DANGER spawn: wake at last_safe"] = keep.location_id == _q("field") and keep.position == Vector2i(1, 1) and _counter_ok(keep, MapRules.DEFAULT_STEPS_MIN, MapRules.DEFAULT_STEPS_MAX)
	var fresh: GameState = _new_gs()
	var _k2: Dictionary = ExploreRunner.enter(fresh, "keep")
	var _r3: Dictionary = ExploreRunner.after_fight(fresh, fight, "lost")
	checks["no last_safe: wake at the start location"] = fresh.location_id == _q("town")
	return _report("Check 14: flee and defeat (fled = from tile; lost = SAFE spawn, last_safe, start; counter reset)", checks)

# ------------------------------------------------------------------------------
# Checks 15 and 16: entry, exits
# ------------------------------------------------------------------------------

static func _test_entry() -> bool:
	var checks: Dictionary = {}
	checks["field"] = _set_field([{"id": "watch", "kind": "encounter", "anchor": "a_near", "encounter": "enc_a", "trigger": "proximity", "radius": 2}])
	var gs: GameState = _new_gs()
	var first: Array[Dictionary] = _events(ExploreRunner.enter(gs, "field"))
	checks["visited flag"] = gs.get_flag(MapRules.visited_flag(_q("field")), false) == true
	checks["auto-start refreshed first, then the reach hook"] = QuestRunner.status(gs, "q_reach") == "ready"
	checks["one hook per entry"] = DictUtil.to_int(DictUtil.dict_of(gs.quests[_q("q_reach2")])["objective"]) == 1
	checks["one autosave, no fight (spawn inside a proximity radius)"] = _count(first, "autosave") == 1 and _count(first, "fight") == 0 and _find(first, "entered").get("location", "") == _q("field")
	checks["entered lists the advanced quests"] = DictUtil.array_of(_find(first, "entered").get("advanced", [])).has(_q("q_reach2"))
	var second: Array[Dictionary] = _events(ExploreRunner.enter(gs, "field"))
	checks["re-entry counts again"] = QuestRunner.status(gs, "q_reach2") == "ready" and _count(second, "autosave") == 1
	checks["begin enters the start location"] = DictUtil.to_bool(ExploreRunner.begin(_new_gs(KID))["ok"])
	var started: GameState = _new_gs(KID)
	var _b: Dictionary = ExploreRunner.begin(started)
	checks["begin: child inherits the parent's start"] = started.location_id == _q("town") and started.world_id == KID and started.position == GameState.NO_POSITION
	checks["menu lists the exits of a map-less location"] = ExploreRunner.menu(started).size() == 1 and str(ExploreRunner.menu(started)[0]["to"]) == "field"
	checks["menu is empty with a map"] = ExploreRunner.menu(gs).is_empty()
	checks["unknown anchor refused, nothing changes"] = _refused_unchanged(gs, func(g: GameState) -> Dictionary: return ExploreRunner.enter(g, "field", "nope"))
	checks["unknown location refused"] = _refused_unchanged(gs, func(g: GameState) -> Dictionary: return ExploreRunner.enter(g, "nowhere"))
	return _report("Check 15: entry (visited, refresh then reach hook per entry, one autosave, never a fight)", checks)

static func _refused_unchanged(gs: GameState, call: Callable) -> bool:
	var before: String = _snap(gs)
	var res: Dictionary = call.call(gs)
	return not DictUtil.to_bool(res["ok"]) and _snap(gs) == before

static func _test_exits() -> bool:
	var checks: Dictionary = {}
	checks["field"] = _set_field([
		{"id": "to_keep", "kind": "exit", "anchor": "a_exit", "to": "keep", "to_anchor": "a_gate", "sets_flags": [{"key": "xr.left"}]},
		{"id": "to_dock", "kind": "exit", "anchor": "a_far", "to": _q("dock", FAR)},
		{"id": "to_map", "kind": "exit", "anchor": "a_wm", "to": MapRules.WORLD_MAP},
	])
	var gs: GameState = _in_field(1, 7)
	var events: Array[Dictionary] = _events(ExploreRunner.step(gs, D))
	var ex: Dictionary = _find(events, "exit")
	checks["exit marker: target at to_anchor"] = gs.location_id == _q("keep") and gs.position == MapRules.anchor_pos(MapCatalog.get_entry("m_danger"), "a_gate") and str(ex.get("to", "")) == "keep"
	checks["exit sets_flags applied, entry events nested"] = gs.get_flag("xr.left", false) == true and _count(_events({"events": ex.get("events", [])}), "autosave") == 1
	var cross: GameState = _in_field(2, 7)
	var _c: Dictionary = ExploreRunner.step(cross, D)
	checks["qualified exit into a connected world sets world_id"] = cross.world_id == FAR and cross.location_id == _q("dock", FAR) and cross.position == GameState.NO_POSITION
	var kid: GameState = _in_field(1, 1, KID)
	checks["an inherited location keeps the child world_id"] = kid.world_id == KID and kid.location_id == _q("field")
	var wm: GameState = _in_field(3, 7)
	var before: String = _snap(wm)
	var wm_exit: Dictionary = _find(_events(ExploreRunner.step(wm, D)), "exit")
	checks["world_map exit: event only, state equal"] = str(wm_exit.get("to", "")) == MapRules.WORLD_MAP and _snap(wm) == before
	var flagged: Dictionary = WorldCatalog.get_own(RUN, "locations", "field")
	var to_map: Dictionary = DictUtil.dict_of(DictUtil.array_of(flagged["markers"])[2])
	to_map["sets_flags"] = [{"key": "xr.mapped"}]
	# TRAV1-L commit 3: a world_map exit's sets_flags apply when the party steps onto it (no warning any more).
	var plain: Dictionary = WorldCatalog.get_own(RUN, "locations", "field")
	checks["world_map exit with sets_flags: no warning"] = not TestHelpers.any_contains(MapRules.location_warnings(flagged), "world map")
	var stored: bool = WorldCatalog.upsert(RUN, "locations", flagged).is_empty()
	var wm_flag: GameState = _in_field(3, 7)
	var flag_events: Array[Dictionary] = _events(ExploreRunner.step(wm_flag, D))
	checks["world_map exit with sets_flags: flags set, token stays"] = stored and wm_flag.get_flag("xr.mapped", false) == true and wm_flag.position == Vector2i(3, 7) and str(_find(flag_events, "exit").get("to", "")) == MapRules.WORLD_MAP and not _find(flag_events, "flags_set").is_empty()
	checks["field restored"] = WorldCatalog.upsert(RUN, "locations", plain).is_empty()
	# An exit whose target can't be entered refuses the step and changes nothing (no move, no sets_flags).
	var gateless: Dictionary = _run_map("m_danger", true)
	var anchors: Array = gateless["anchors"]
	for i: int in range(anchors.size() - 1, -1, -1):
		if str(DictUtil.dict_of(anchors[i])["id"]) == "a_gate":
			anchors.remove_at(i)
	checks["target map loses its anchor"] = MapCatalog.upsert(gateless).is_empty()
	var stuck: GameState = _in_field(1, 7)
	var stuck_before: String = _snap(stuck)
	var refused: Dictionary = ExploreRunner.step(stuck, D)
	checks["failed exit changes nothing"] = not DictUtil.to_bool(refused["ok"]) and _snap(stuck) == stuck_before and not stuck.has_flag("xr.left")
	checks["target map restored"] = MapCatalog.upsert(_run_map("m_danger", true)).is_empty()
	var town: GameState = _new_gs()
	var _t: Dictionary = ExploreRunner.enter(town, "town")
	checks["take_exit from a menu location"] = DictUtil.to_bool(ExploreRunner.take_exit(town, "field")["ok"]) and town.location_id == _q("field") and town.position == Vector2i(1, 1)
	checks["take_exit refuses a target not in exits"] = _refused_unchanged(town, func(g: GameState) -> Dictionary: return ExploreRunner.take_exit(g, "nowhere"))
	return _report("Check 16: exits (to_anchor, connected world crossing, inherited location, world_map hand-over)", checks)

# MAPREAD1-L2: the region view on the marker_view fixture (a marker of every kind; `mark` has a note and sets a flag).
static func _test_map_read(tree_parent: Node) -> bool:
	var packed: PackedScene = load(DEBUG_SCENE) as PackedScene
	if packed == null or tree_parent == null:
		return _report("Map read-out (headless without a tree: skipped)", {"scene loads": packed != null})
	var markers: Array = [
		{"id": "sage", "kind": "npc", "anchor": "a_npc", "character": "sage"},
		{"id": "box", "kind": "chest", "anchor": "a_chest", "items": {str(_rx["item0"]): 1}},
		{"id": "stone", "kind": "lore", "anchor": "a_lore", "lore": "stone"},
		{"id": "to_town", "kind": "exit", "anchor": "a_exit", "to": "town"},
		{"id": "gate", "kind": "exit", "anchor": "a_lock", "to": "town", "requires_flags": [{"key": "xr.key"}], "locked_text": "Gate barred."},
		{"id": "ambush", "kind": "encounter", "anchor": "a_touch", "encounter": "enc_a"},
		{"id": "patrol", "kind": "roamer", "encounter": "elite_a", "path": "p4"},
		{"id": "mark", "kind": "point", "anchor": "a_point", "sets_flags": [{"key": "xr.marked"}], "label": "Signal", "note": "You look."},
		{"id": "plain", "kind": "point", "anchor": "a_far"},
	]
	if not _set_field(markers):
		return _report("Map read-out", {"fixture": false})
	var view: RegionMapView = packed.instantiate() as RegionMapView
	tree_parent.add_child(view)
	view.play_state(_in_field(2, 4))
	var ok: bool = MapReadChecks.run_region(view)
	view.play_state(_in_field(2, 4))
	ok = MapReadChecks.run_notes(view, "mark", "plain", "box") and ok
	ok = _set_field([]) and ok
	tree_parent.remove_child(view)
	view.free()
	return ok

# ------------------------------------------------------------------------------
# Checks 17 and 18: chests, flag markers
# ------------------------------------------------------------------------------

# MAPREAD1-L1: marker_view labels / verbs / states / badges and the TravelRules.view node extras.
static func _test_marker_view() -> bool:
	var checks: Dictionary = {}
	var item0: String = str(_rx["item0"])
	var errors: Array[String] = WorldCatalog.upsert(RUN, "regions", {"id": "reg_x", "name": "Region X"})
	var markers: Array = [
		{"id": "sage", "kind": "npc", "anchor": "a_npc", "character": "sage"},
		{"id": "box", "kind": "chest", "anchor": "a_chest", "items": {item0: 1}},
		{"id": "stone", "kind": "lore", "anchor": "a_lore", "lore": "stone"},
		{"id": "to_town", "kind": "exit", "anchor": "a_exit", "to": "town"},
		{"id": "to_map", "kind": "exit", "anchor": "a_wm", "to": MapRules.WORLD_MAP},
		{"id": "gate", "kind": "exit", "anchor": "a_lock", "to": "town", "requires_flags": [{"key": "xr.key"}], "locked_text": "Gate barred."},
		{"id": "ambush", "kind": "encounter", "anchor": "a_touch", "encounter": "enc_a"},
		{"id": "patrol", "kind": "roamer", "encounter": "elite_a", "path": "p4"},
		{"id": "mark", "kind": "point", "anchor": "a_point", "sets_flags": [{"key": "xr.marked"}], "label": "Signal", "note": "You look."},
		{"id": "plain", "kind": "point", "anchor": "a_far"},
	]
	checks["fixtures"] = errors.is_empty() and _set_field(markers)
	var gs: GameState = _in_field(2, 4)
	var loc: Dictionary = ExploreRunner.location(gs)
	var views: Dictionary = {}
	for m: Dictionary in ExploreRunner.markers_of(loc):
		views[str(m["id"])] = ExploreRunner.marker_view(gs, loc, m)
	checks["every label, verb and icon is non-empty"] = views.values().all(func(v: Variant) -> bool: return not str(DictUtil.dict_of(v)["label"]).is_empty() and not str(DictUtil.dict_of(v)["verb"]).is_empty() and not str(DictUtil.dict_of(v)["icon"]).is_empty())
	checks["npc: character name, Talk, no badge"] = views["sage"]["label"] == "Sage XR" and views["sage"]["verb"] == "Talk" and views["sage"]["badge"] == "" and views["sage"]["state"] == "present"
	checks["chest: Chest / Open"] = views["box"]["label"] == "Chest" and views["box"]["verb"] == "Open"
	checks["lore: lore name / Read"] = views["stone"]["label"] == "Old Stone" and views["stone"]["verb"] == "Read"
	checks["exit: To <location name>, world map"] = views["to_town"]["label"] == "To Town" and views["to_town"]["verb"] == "Walk onto it" and views["to_map"]["label"] == "World map"
	checks["locked exit: state and verb"] = views["gate"]["state"] == "locked" and views["gate"]["verb"] == "Locked: Gate barred."
	checks["encounter and roamer: encounter name"] = views["ambush"]["label"] == "enc_a" and views["ambush"]["verb"] == "Fight" and views["patrol"]["label"] == "elite_a" and views["patrol"]["verb"] == "Patrol: fight on contact"
	checks["point: own label wins, default otherwise"] = views["mark"]["label"] == "Signal" and views["plain"]["label"] == "Something here" and views["plain"]["verb"] == "Look"
	checks["icons"] = views["sage"]["icon"] == "overlay_npc" and views["box"]["icon"] == "overlay_chest" and views["mark"]["icon"] == "overlay_poi" and views["to_town"]["icon"] == "overlay_exit" and views["patrol"]["icon"] == "overlay_roamer"
	# Spent: an opened chest; a point whose sets_flags all hold. A point without sets_flags never spends.
	var _opened: Dictionary = ExploreRunner.interact(gs, D)
	var box: Dictionary = ExploreRunner.marker_view(gs, loc, ExploreRunner.markers_of(loc)[1])
	checks["opened chest: spent, Empty"] = box["state"] == "spent" and box["verb"] == "Empty"
	var mark: Dictionary = DictUtil.dict_of(ExploreRunner.markers_of(loc)[8])
	checks["point not yet used: present"] = ExploreRunner.marker_view(gs, loc, mark)["state"] == "present"
	var _flagged: String = QuestRunner.set_flag(gs, "xr.marked", true)
	var used: Dictionary = ExploreRunner.marker_view(gs, loc, mark)
	checks["used point: spent, Nothing more here"] = used["state"] == "spent" and used["verb"] == "Nothing more here"
	checks["point without sets_flags never spends"] = ExploreRunner.marker_view(gs, loc, DictUtil.dict_of(ExploreRunner.markers_of(loc)[9]))["state"] == "present"
	# Badges: quest offer, then ready turn-in.
	var q_errs: Array[String] = WorldCatalog.upsert(RUN, "quests", {"id": "q_offer", "name": "Offer", "template": "t_flag", "giver": "sage", "steps": [{"text": "Mark", "flag": "xr.offer"}]})
	checks["offer fixture"] = q_errs.is_empty()
	var sage: Dictionary = DictUtil.dict_of(ExploreRunner.markers_of(loc)[0])
	checks["badge quest while an offer is open"] = QuestRunner.status(gs, "q_offer") == "available" and ExploreRunner.marker_view(gs, loc, sage)["badge"] == "quest"
	var started: Array[String] = QuestRunner.start(gs, "q_offer")
	checks["no badge while the quest runs"] = started.is_empty() and ExploreRunner.marker_view(gs, loc, sage)["badge"] == ""
	var _done: String = QuestRunner.set_flag(gs, "xr.offer", true)
	checks["badge ready when it can be turned in"] = QuestRunner.status(gs, "q_offer") == "ready" and ExploreRunner.marker_view(gs, loc, sage)["badge"] == "ready"
	# TravelRules.view node extras on the route node `keep` (kind fortress, region, map_pos), quest flag per active step.
	var keep_errs: Array[String] = WorldCatalog.upsert(RUN, "locations", {"id": "keep", "name": "Keep", "kind": "fortress", "region": "reg_x", "map_pos": [0.25, 0.5], "map": "m_danger", "exits": [{"to": "field"}], "zone_bindings": _default_bindings()})
	checks["keep fixture"] = keep_errs.is_empty()
	var keep_before: Dictionary = _node_of(TravelRules.view(gs), _q("keep"))
	checks["node: kind, region name, map_pos, quest false"] = str(keep_before.get("kind", "")) == "fortress" and keep_before.get("region", "") == "Region X" and keep_before.get("map_pos", []) == [0.25, 0.5] and keep_before.get("quest", true) == false
	var field_node: Dictionary = _node_of(TravelRules.view(gs), _q("field"))
	checks["node defaults: kind wild, region '', map_pos []"] = field_node.get("kind", "") == "wild" and field_node.get("region", "x") == "" and DictUtil.array_of(field_node.get("map_pos", [1])).is_empty()
	var goto_errs: Array[String] = WorldCatalog.upsert(RUN, "quests", {"id": "q_goto", "name": "Goto", "template": "t_reach", "steps": [{"text": "Go", "location": "keep"}]})
	var goto_started: Array[String] = QuestRunner.start(gs, "q_goto")
	checks["quest flag while the step targets the node"] = goto_errs.is_empty() and goto_started.is_empty() and TravelRules.quest_nodes(gs).has(_q("keep")) and _node_of(TravelRules.view(gs), _q("keep")).get("quest", false) == true
	checks["quest_steps_at: the step text at its node, not at another"] = TravelRules.quest_steps_at(gs, _q("keep")).size() == 1 and TravelRules.quest_steps_at(gs, _q("keep")).has("Go") and not TravelRules.quest_steps_at(gs, _q("field")).has("Go")
	var _abandoned: Array[String] = QuestRunner.abandon(gs, "q_goto")
	checks["quest_steps_at is empty once the quest is gone"] = TravelRules.quest_steps_at(gs, _q("keep")).is_empty()
	checks["quest flag clears when the quest is gone"] = _node_of(TravelRules.view(gs), _q("keep")).get("quest", true) == false
	checks["keep restored"] = WorldCatalog.upsert(RUN, "locations", {"id": "keep", "name": "Keep", "map": "m_danger", "exits": [{"to": "field"}], "zone_bindings": _default_bindings()}).is_empty()
	checks["cleanup"] = WorldCatalog.remove(RUN, "quests", "q_offer").is_empty() and WorldCatalog.remove(RUN, "quests", "q_goto").is_empty() and WorldCatalog.remove(RUN, "regions", "reg_x").is_empty()
	checks["field restored"] = _set_field([])
	return _report("MAPREAD1-L1: marker_view and TravelRules.view node extras", checks)

static func _node_of(view: Dictionary, qid: String) -> Dictionary:
	for n: Variant in DictUtil.array_of(view.get("nodes", [])):
		if str(DictUtil.dict_of(n).get("qid", "")) == qid:
			return DictUtil.dict_of(n)
	return {}

static func _test_chest() -> bool:
	var checks: Dictionary = {}
	var item0: String = str(_rx["item0"])
	var item1: String = str(_rx["item1"])
	checks["field"] = _set_field([{"id": "box", "kind": "chest", "anchor": "a_chest", "items": {item0: 2}, "loot": "lt_box"}])
	var gs: GameState = _in_field(2, 4)
	gs.loot_claimed = [_q("zz_other")] as Array[String]
	var stash0: int = DictUtil.to_int(gs.party.stash.get(item0, 0))
	var stash1: int = DictUtil.to_int(gs.party.stash.get(item1, 0))
	var opened: Dictionary = ExploreRunner.interact(gs, D)
	checks["two distinct items picked"] = item0 != item1
	checks["first open grants items and the unclaimed loot"] = str(opened.get("kind", "")) == "chest" and not DictUtil.to_bool(opened.get("empty", true)) and DictUtil.to_int(gs.party.stash.get(item0, 0)) == stash0 + 2 and DictUtil.to_int(gs.party.stash.get(item1, 0)) == stash1 + 1
	checks["loot claimed, list sorted"] = gs.loot_claimed == ([_q("lt_box"), _q("zz_other")] as Array[String])
	checks["done flag set"] = gs.get_flag(MapRules.done_flag(_q("field"), "box"), false) == true
	var snap: String = _snap(gs)
	var again: Dictionary = ExploreRunner.interact(gs, D)
	checks["second open is empty and grants nothing"] = DictUtil.to_bool(again.get("empty", false)) and _snap(gs) == snap
	checks["an opened chest stays solid"] = _find(_events(ExploreRunner.step(gs, D)), "blocked").size() > 0
	var quest_claimed: GameState = _in_field(2, 4)
	quest_claimed.loot_claimed = [_q("lt_box")] as Array[String]
	var q1: int = DictUtil.to_int(quest_claimed.party.stash.get(item1, 0))
	var q0: int = DictUtil.to_int(quest_claimed.party.stash.get(item0, 0))
	var _o: Dictionary = ExploreRunner.interact(quest_claimed, D)
	checks["loot already claimed by a quest: items only"] = DictUtil.to_int(quest_claimed.party.stash.get(item0, 0)) == q0 + 2 and DictUtil.to_int(quest_claimed.party.stash.get(item1, 0)) == q1 and quest_claimed.loot_claimed.size() == 1
	checks["claim_loot refuses a second claim"] = not RewardGrant.claim_loot(quest_claimed, _q("lt_box")).is_empty()
	var nothing: Dictionary = ExploreRunner.interact(quest_claimed, L)
	checks["nothing there: kind ''"] = DictUtil.to_bool(nothing["ok"]) and str(nothing["kind"]).is_empty()
	return _report("Check 17: chests (items + unclaimed loot once, claimed sorted; quest-claimed loot skipped; empty after)", checks)

static func _test_flag_markers() -> bool:
	var checks: Dictionary = {}
	checks["field"] = _set_field([
		{"id": "mark", "kind": "point", "anchor": "a_point", "sets_flags": [{"key": "xr.point"}]},
		{"id": "stone", "kind": "lore", "anchor": "a_lore", "lore": "stone", "sets_flags": [{"key": "xr.read"}]},
		{"id": "sage", "kind": "npc", "anchor": "a_npc", "character": "sage"},
	])
	var gs: GameState = _in_field(1, 6)
	checks["flag quests waiting"] = QuestRunner.status(gs, "q_point") == "active" and QuestRunner.status(gs, "q_lore") == "active"
	var events: Array[Dictionary] = _events(ExploreRunner.step(gs, D))
	checks["point sets its flag (flags_set event)"] = JsonUtil.strings(_find(events, "flags_set").get("flags", [])) == ["xr.point"]
	checks["the point's quest advances in the same call"] = QuestRunner.status(gs, "q_point") == "ready"
	gs.position = Vector2i(3, 6)
	var lore: Dictionary = ExploreRunner.interact(gs, D)
	checks["lore returns its text"] = str(lore.get("kind", "")) == "lore" and str(lore.get("lore", "")) == _q("stone") and str(lore.get("name", "")) == "Old Stone" and str(lore.get("text", "")) == "Carved runes."
	checks["the lore quest advances in the same call"] = QuestRunner.status(gs, "q_lore") == "ready"
	gs.position = Vector2i(2, 2)
	var npc: Dictionary = ExploreRunner.interact(gs, D)
	checks["npc returns the character"] = str(npc.get("kind", "")) == "npc" and str(npc.get("character", "")) == _q("sage")
	gs.position = Vector2i(1, 2)
	var hints: Array[Dictionary] = _events(ExploreRunner.step(gs, D))   # to (1, 3), next to the npc at (2, 3)
	checks["interact hint next to the npc"] = _find(hints, "interact_hint").get("marker", "") == "sage"
	return _report("Check 18: point and lore sets_flags reach quest flag objectives in the same call", checks)

# ------------------------------------------------------------------------------
# Check 19: battle bridge
# ------------------------------------------------------------------------------

static func _test_bridge() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _in_field(1, 1)
	checks["win quest waiting"] = QuestRunner.status(gs, "q_win") == "active"
	var levels: Array[Dictionary] = BattleBridge.enemy_levels(gs.world_id, _q("enc_a"))
	var expected: Dictionary = {}
	for m: UnitState in gs.party.active_members():
		expected[m.id] = m.total_xp + XpCurve.encounter_xp(levels, m.level())
	var snap: String = _snap(gs)
	checks["lost changes nothing"] = DictUtil.to_bool(BattleBridge.apply(gs, _q("enc_a"), "lost")["ok"]) and _snap(gs) == snap
	checks["fled changes nothing"] = DictUtil.to_bool(BattleBridge.apply(gs, _q("enc_a"), "fled")["ok"]) and _snap(gs) == snap
	checks["unknown outcome refused"] = not DictUtil.to_bool(BattleBridge.apply(gs, _q("enc_a"), "draw")["ok"]) and _snap(gs) == snap
	var won: Dictionary = BattleBridge.apply(gs, _q("enc_a"), "won")
	var xp_ok: bool = not levels.is_empty() and not expected.is_empty()
	for m: UnitState in gs.party.active_members():
		xp_ok = xp_ok and m.total_xp == DictUtil.to_int(expected[m.id])
	checks["won: XpCurve.encounter_xp to each active member"] = DictUtil.to_bool(won["ok"]) and xp_ok
	checks["won: the win_encounter quest advances"] = QuestRunner.status(gs, "q_win") == "ready" and DictUtil.array_of(won["advanced"]).has(_q("q_win"))
	var made: Dictionary = BattleBridge.encounter_for(gs, _q("enc_a"))
	var enc: GameEncounter = _encounter_of(made["encounter"])
	var session: CombatSession = null
	if enc != null:
		session = CombatSession.new(enc)
		session.start()
	var names_ok: bool = session != null
	if session != null:
		for c: Combatant in session.allies:
			names_ok = names_ok and c.name != "Hero XR"
	checks["encounter_for: exactly the active members as allies"] = session != null and session.allies.size() == gs.party.active_members().size() and names_ok and not session.enemies.is_empty()
	var zoned: GameEncounter = _encounter_of(BattleBridge.encounter_for(gs, _q("enc_a"), "forest")["encounter"])
	checks["zone override"] = zoned != null and zoned.zone_id == "forest"
	if session != null:
		checks["live battle: no outcome yet"] = BattleBridge.outcome(session) == ""
		session.phase = CombatSession.Phase.REPLAY
		session.current_round = enc.max_rounds
		checks["round limit with both sides standing = fled (X1)"] = BattleBridge.outcome(session) == "fled"
		session.combat_over = true
		for a: Combatant in session.allies:
			a.is_withdrawn = true
		checks["over with the whole party withdrawn (alive) = fled"] = BattleBridge.outcome(session) == "fled"
		for a: Combatant in session.allies:
			a.is_withdrawn = false
		session.combat_over = false
		for e: Combatant in session.enemies:
			e.hp = 0
		session.combat_over = true
		checks["over with a member standing = won"] = BattleBridge.outcome(session) == "won"
		for a: Combatant in session.allies:
			a.hp = 0
		checks["over with every member down = lost"] = BattleBridge.outcome(session) == "lost"
		session.dispose()
	checks["unknown encounter refused"] = not DictUtil.to_bool(BattleBridge.encounter_for(gs, "nope")["ok"])
	return _report("Check 19: battle bridge (XP and quest hook on won only; active party as allies; outcomes)", checks)

# ------------------------------------------------------------------------------
# Check 20: save v3
# ------------------------------------------------------------------------------

static func _write_raw(slot: int, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(SaveGame.base_dir())
	var f: FileAccess = FileAccess.open(SaveGame.path_for(slot), FileAccess.WRITE)
	f.store_string(text)
	f.close()

static func _without_saved_at(text: String) -> String:
	var env: Dictionary = DictUtil.dict_of(JSON.parse_string(text))
	env.erase("saved_at")
	return JSON.stringify(env, "", true)

# A v2 save with flags, quests and loot_claimed: every v2 key survives unchanged (location_id qualified), and
# migrate -> write -> read -> write is byte-stable apart from saved_at.
static func _lossless_v2() -> bool:
	var gs: GameState = _in_field(1, 1)
	var _f1: String = gs.set_flag("xr.count", 7)
	var _f2: String = gs.set_flag("xr.name", "x")
	gs.loot_claimed = [_q("lt_box"), _q("zz_other")] as Array[String]
	var v2: Dictionary = gs.to_dict()
	for key: String in ["position", "explore", "rng_seed", "rolls"]:
		v2.erase(key)
	v2["location_id"] = "field"
	var ok: bool = not DictUtil.dict_of(v2["quests"]).is_empty() and DictUtil.dict_of(v2["flags"]).size() >= 3
	_write_raw(6, JSON.stringify({"format": SaveGame.FORMAT, "version": 2, "saved_at": 0, "game_state": v2}))
	var res: Dictionary = SaveGame.read(6)
	var st: GameState = GameState.of(res["state"])
	if st == null:
		return false
	ok = ok and JsonUtil.strings(res["warnings"]).is_empty()
	var now: Dictionary = st.to_dict()
	for key: Variant in v2:
		if str(key) == "location_id":
			ok = ok and str(now[key]) == _q("field")
		else:
			ok = ok and JsonUtil.deep_equal(now[key], v2[key])
	ok = ok and SaveGame.write(st, 1).is_empty()
	var again: GameState = GameState.of(SaveGame.read(1)["state"])
	ok = ok and again != null and SaveGame.write(again, 2).is_empty()
	return ok and _without_saved_at(FileAccess.get_file_as_string(SaveGame.path_for(1))) == _without_saved_at(FileAccess.get_file_as_string(SaveGame.path_for(2)))

static func _test_save_v3() -> bool:
	var checks: Dictionary = {}
	var gs: GameState = _new_gs()
	var v2: Dictionary = gs.to_dict()
	for key: String in ["position", "explore", "rng_seed", "rolls"]:
		v2.erase(key)
	v2["location_id"] = "field"
	var text: String = JSON.stringify({"format": SaveGame.FORMAT, "version": 2, "saved_at": 0, "game_state": v2})
	_write_raw(3, text)
	var one: Dictionary = SaveGame.read(3)
	var two: Dictionary = SaveGame.read(3)
	var s1: GameState = GameState.of(one["state"])
	var s2: GameState = GameState.of(two["state"])
	checks["v2 migrates"] = DictUtil.to_bool(one["ok"]) and s1 != null and JsonUtil.strings(one["warnings"]).is_empty()
	checks["defaults added"] = s1 != null and s1.position == GameState.NO_POSITION and JsonUtil.deep_equal(s1.explore, GameState.default_explore()) and s1.rolls.is_empty()
	checks["local location_id qualified"] = s1 != null and s1.location_id == _q("field")
	checks["same rng_seed on two runs, in range"] = s1 != null and s2 != null and s1.rng_seed == s2.rng_seed and s1.rng_seed >= 0 and s1.rng_seed <= GameState.SEED_MAX
	v2["location_id"] = "nowhere"
	_write_raw(3, JSON.stringify({"format": SaveGame.FORMAT, "version": 2, "saved_at": 0, "game_state": v2}))
	var lost_read: Dictionary = SaveGame.read(3)
	var lost_loc: GameState = GameState.of(lost_read["state"])
	checks["unresolved location_id becomes '' with a warning"] = lost_loc != null and lost_loc.location_id.is_empty() and JsonUtil.strings(lost_read["warnings"]) == ["cleared unresolved location_id 'nowhere'"]
	checks["lossless v2 -> v3 (flags, quests, loot_claimed)"] = _lossless_v2()
	var bad_pos: GameState = GameState.from_dict(gs.to_dict())
	bad_pos.position = Vector2i(-2, 0)
	var bad_explore: GameState = GameState.from_dict(gs.to_dict())
	bad_explore.explore["extra"] = 1
	var bad_rolls: GameState = GameState.from_dict(gs.to_dict())
	bad_rolls.rolls[GameRolls.EXPLORE] = -1
	var bad_next: GameState = GameState.from_dict(gs.to_dict())
	bad_next.explore["next_at"] = 0
	var raw_pos: Dictionary = gs.to_dict()
	raw_pos["position"] = ["a", 1]
	var _gone: bool = SaveGame.delete(2)   # slots are 0..3 (UI1-L): the refusals below write slot 2
	for pair: Array in [["bad position", bad_pos], ["explore with an extra key", bad_explore], ["negative rolls", bad_rolls], ["next_at 0", bad_next], ["position not ints", GameState.from_dict(raw_pos)]]:
		var g: GameState = GameState.of(pair[1])
		checks["%s fails validate and write" % str(pair[0])] = not g.validate().is_empty() and not SaveGame.write(g, 2).is_empty()
	checks["nothing written for refusals"] = not SaveGame.exists(2)
	var walked: GameState = _in_field(5, 2)
	walked.rolls[GameRolls.EXPLORE] = 3
	var written: bool = SaveGame.write(walked, 1).is_empty()
	var back: GameState = GameState.of(SaveGame.read(1)["state"])
	checks["v3 round trip"] = written and back != null and back.equals(walked) and DictUtil.to_int(DictUtil.dict_of(JSON.parse_string(FileAccess.get_file_as_string(SaveGame.path_for(1))))["version"]) == SaveGame.VERSION and SaveGame.VERSION >= 3   # 4 since DLG1-L
	var blank: GameState = _new_gs()
	var resumed: Dictionary = ExploreRunner.resume(blank)
	checks["resume with no location begins at the start location"] = DictUtil.to_bool(resumed["ok"]) and blank.location_id == _q("town")
	var wall: GameState = _in_field(0, 0)
	var res: Dictionary = ExploreRunner.resume(wall)
	checks["resume fixes a position on a wall to spawn"] = DictUtil.to_bool(res["ok"]) and wall.position == Vector2i(1, 1) and _count(_events(res), "autosave") == 0
	checks["NewGame seed used as given"] = _new_gs().rng_seed == SEED
	var clock: GameState = GameState.of(NewGame.create(RUN, [str(_rx["party"])] as Array[String])["state"])
	checks["NewGame clock seed in range"] = clock != null and clock.rng_seed >= 0 and clock.rng_seed <= GameState.SEED_MAX and clock.validate().is_empty()
	return _report("Check 20: save v3 (v2 migration, qualified location, stable seed, validation, resume)", checks)

# ------------------------------------------------------------------------------
# Commit 3: the debug region-map scene (thin: drives ExploreRunner, autosaves to the autosave slot 0 in the sandbox)
# ------------------------------------------------------------------------------

const DEBUG_SCENE: String = "res://scenes/overworld/region_map_debug.tscn"

static func _test_debug_scene(tree_parent: Node) -> bool:
	var checks: Dictionary = {}
	var packed: PackedScene = load(DEBUG_SCENE) as PackedScene
	checks["scene loads"] = packed != null
	if packed == null or tree_parent == null:
		return _report("Debug scene (headless without a tree: load only)", checks)
	checks["field"] = _set_field([{"id": "ambush", "kind": "encounter", "anchor": "a_touch", "encounter": "enc_a"}])
	var view: RegionMapView = packed.instantiate() as RegionMapView
	tree_parent.add_child(view)
	checks["new game at the start location"] = view.start_game(RUN, SEED).is_empty() and view.gs != null and view.gs.location_id == _q("town")
	if view.gs == null:
		tree_parent.remove_child(view)
		view.free()
		return _report("Debug scene (new game, menu exit, fight panel, debug outcome, autosave slot 0)", checks)
	# REVIEW-F1: the standalone debug game never writes the real autosave; a main-menu game (CurrentGame) does
	checks["debug game leaves slot 0 alone"] = not SaveGame.exists(SaveGame.AUTOSAVE_SLOT)
	CurrentGame.start(view.gs)
	view._autosave()
	checks["main-menu game autosave wrote slot 0 in the sandbox"] = SaveGame.exists(SaveGame.AUTOSAVE_SLOT) and SaveGame.path_for(SaveGame.AUTOSAVE_SLOT).begins_with(SANDBOX)
	var slot0: String = FileAccess.get_file_as_string(SaveGame.path_for(SaveGame.AUTOSAVE_SLOT))
	var model := SaveSlotsModel.new(MenuRouter.new(MenuRouter.LOAD))
	checks["no fight: manual save works"] = model.save(1).is_empty() and CurrentGame.busy() == false
	CurrentGame.end()
	view._autosave()
	checks["debug game after the menu game ended: slot 0 unchanged"] = FileAccess.get_file_as_string(SaveGame.path_for(SaveGame.AUTOSAVE_SLOT)) == slot0
	CurrentGame.start(view.gs)
	view.take_exit("field")
	checks["menu exit enters the field"] = view.gs.location_id == _q("field") and view.gs.position == Vector2i(1, 1)
	view.gs.position = Vector2i(4, 2)
	view.do_step(R)
	checks["a fight opens the outcome panel"] = not view.pending_fight.is_empty() and view.gs.position == Vector2i(5, 2)
	checks["busy while the fight is pending"] = CurrentGame.busy() and model.save(2) == [SaveSlotsModel.BUSY] and not SaveGame.exists(2)
	view.do_step(R)
	checks["steps are ignored while a fight is pending"] = view.gs.position == Vector2i(5, 2)
	view.settle("fled")
	checks["fled button settles through after_fight"] = view.pending_fight.is_empty() and view.gs.position == Vector2i(4, 2) and _steps(view.gs) == 0
	checks["not busy after the fight settles: save works"] = not CurrentGame.busy() and model.save(2).is_empty() and SaveGame.exists(2)
	CurrentGame.end()
	var map_ok: bool = MapViewChecks.run_region(view)   # MAPVIEW1: full-screen canvas, camera, click with offset
	tree_parent.remove_child(view)
	view.free()
	return _report("Debug scene (new game, menu exit, fight panel, debug outcome, autosave slot 0)", checks) and map_ok
