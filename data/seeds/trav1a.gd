extends SceneTree
# TRAV1-A asset seed (idempotent): Ashlands routes, Iron Reaches south road, two world_map exit markers.
# Run: tools/fc.sh run data/seeds/trav1a.gd

var failed: bool = false

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)
		quit(1)

func _flag(key: String) -> Array:
	return [{"key": key, "value": true}]

func _route(id: String, nm: String, from: String, to: String, time: int, zone: String, pct: int, enc: Array, extra: Dictionary) -> Dictionary:
	var r: Dictionary = {
		"id": id, "name": nm, "from": from, "to": to, "two_way": true, "time": time, "zone": zone,
		"encounter_pct": pct, "repeat_pct": -1, "encounters": enc, "elites": [], "elite_pct": 0,
		"scripted": [], "requires_flags": [], "unless_flags": [], "tags": []}
	r.merge(extra, true)
	return r

func _marker(id: String, anchor: String) -> Dictionary:
	return {"id": id, "kind": "exit", "anchor": anchor, "to": "world_map", "requires_flags": [], "unless_flags": [], "sets_flags": []}

func _add_marker(loc_id: String, m: Dictionary) -> void:
	var loc: Dictionary = WorldCatalog.get_own("ashlands", "locations", loc_id)
	var markers: Array = loc.get("markers", [])
	var out: Array = []
	for x: Variant in markers:
		if x is Dictionary and str((x as Dictionary).get("id", "")) != str(m["id"]):
			out.append(x)
	out.append(m)
	loc["markers"] = out
	_chk("marker " + loc_id, WorldCatalog.upsert("ashlands", "locations", loc))

func _init() -> void:
	WorldCatalog.reload_from_disk()
	_chk("old_road", WorldCatalog.upsert("ashlands", "routes", _route("old_road", "The old road", "warden_keep", "ruined_courtyard", 2, "ruined_courtyard", 15, [["ash_scouts", 1]], {
		"elite_pct": 5,
		"scripted": [{"id": "first_raid", "encounter": "ash_raiders", "requires_flags": [], "unless_flags": _flag("ashlands.courtyard_held"), "sets_flags": _flag("ashlands.old_road_raided")}]})))
	_chk("high_road", WorldCatalog.upsert("ashlands", "routes", _route("high_road", "The high road", "warden_keep", "cinder_gate", 4, "desert_outpost", 25, [["ash_scouts", 2], ["ash_raiders", 2]], {
		"elites": [["ash_patrol", 1]], "elite_pct": 5, "requires_flags": _flag("ashlands.courtyard_held")})))
	_chk("south_road", WorldCatalog.upsert("iron_reaches", "routes", _route("south_road", "The south road", "slag_gate", "ashlands:warden_keep", 6, "frozen_pass", 20, [["ashlands:ash_scouts", 1]], {})))
	_add_marker("warden_keep", _marker("signpost", "road"))
	_add_marker("cinder_gate", _marker("wastes_road", "east"))
	if WorldCatalog.save_world("ashlands") != OK or WorldCatalog.save_world("iron_reaches") != OK:
		failed = true
		print("ERROR save_world")
	print("WorldCatalog.validate_all: ", WorldCatalog.validate_all())
	print("TravelRules warnings: ", TravelRules.warnings("ashlands"), TravelRules.warnings("iron_reaches"))
	print("DONE failed=", failed)
	quit(1 if failed else 0)
