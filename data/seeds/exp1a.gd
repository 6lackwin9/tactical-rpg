extends SceneTree
# EXP1-A asset seed (idempotent): map templates, Ashlands exploration data.
# Run: tools/fc.sh run data/seeds/exp1a.gd

var failed: bool = false

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)

func _save(label: String, err: Error) -> void:
	if err != OK:
		failed = true
		print("ERROR ", label, ": ", err)

func _a(id: String, x: int, y: int) -> Dictionary:
	return {"id": id, "x": x, "y": y}

func _flag(key: String) -> Array:
	return [{"key": key, "value": true}]

func _m(id: String, kind: String, anchor: String, extra: Dictionary) -> Dictionary:
	var m: Dictionary = {"id": id, "kind": kind, "anchor": anchor, "requires_flags": [], "unless_flags": [], "sets_flags": []}
	m.merge(extra, true)
	return m

func _enc(id: String, nm: String, desc: String, intro: String, enemies: Array) -> Dictionary:
	return {
		"id": id, "name": nm, "description": desc, "intro": intro, "location": "ruined_courtyard", "zone": "ruined_courtyard",
		"arena": "CLOSED", "exits": [[8, 4]], "max_rounds": 6, "total_blocks": 6, "ap_regen": 4, "unit_ap": 12, "rng_seed": 1337,
		"party": ["mara_voss", "tev_quill"], "enemies": enemies, "wave_units": [], "wave_round": 1, "wave_block": 4,
		"wave_label": "Reinforcements arrive!", "tags": [],
	}

func _init() -> void:
	MapCatalog.reload_from_disk()
	WorldCatalog.reload_from_disk()
	var town_rows: Array = [
		"################", "#..............#", "#.###......###.#", "#.###......###.#", "#..............#",
		"#..............#", "#.###......###.#", "#.###......###.#", "#..............#", "#######..#######"]
	var town_zones: Array = []
	for i: int in range(10):
		town_zones.append("ssssssssssssssss")
	_chk("walled_town", MapCatalog.upsert({
		"id": "walled_town", "name": "Walled town, south gate", "rows": town_rows, "zones": town_zones,
		"zone_legend": {"s": {"id": "town", "kind": "safe"}},
		"anchors": [_a("spawn", 7, 8), _a("gate", 7, 9), _a("road", 1, 8), _a("npc_a", 4, 4), _a("npc_b", 11, 4), _a("npc_c", 7, 1), _a("npc_d", 14, 1), _a("chest_a", 1, 1), _a("lore_a", 8, 5)],
		"paths": {}, "tags": []}))
	var gorge_rows: Array = [
		"####################", "#....#........~~~..#", "#....#..##....~~~..#", "#.........#........#", "#....#....#...##...#",
		".....#..............", "#....#...##........#", "#.........#...~~...#", "#....#....#...~~...#", "#....#..........##.#",
		"#....#.............#", "####################"]
	var gorge_zones: Array = []
	for i: int in range(12):
		gorge_zones.append("sssssddddddddddddddd")
	_chk("gorge_ruins", MapCatalog.upsert({
		"id": "gorge_ruins", "name": "Gorge: camp west, ruins east, two openings", "rows": gorge_rows, "zones": gorge_zones,
		"zone_legend": {"s": {"id": "camp", "kind": "safe"}, "d": {"id": "ruins", "kind": "danger"}},
		"anchors": [_a("spawn", 2, 5), _a("west", 0, 5), _a("west_in", 1, 5), _a("east", 19, 5), _a("east_in", 18, 5), _a("camp_npc", 2, 2), _a("camp_chest", 1, 10), _a("site_a", 12, 5), _a("site_b", 16, 5), _a("altar", 17, 1), _a("ruin_chest", 18, 10), _a("ruin_lore", 7, 9)],
		"paths": {"patrol": [[8, 10], [9, 10], [10, 10], [11, 10], [12, 10], [13, 10], [14, 10]]}, "tags": []}))
	_save("map save", MapCatalog.save())

	_chk("manifest", WorldCatalog.update_manifest("ashlands", {"start_location": "warden_keep"}))

	var chars: Array = [
		["ash_brawler", "Horde Brawler", "Warband Muscle", "goblin_brawler", "One of many who fight for whoever feeds them."],
		["ash_skirmisher", "Horde Skirmisher", "Warband Runner", "goblin_skirmisher", "Fast, light and never alone."],
		["ash_archer", "Horde Archer", "Warband Bow", "goblin_archer", "Shoots from the ridges and runs."],
		["ash_shaman", "Horde Smokecaller", "Warband Shaman", "goblin_shaman", "Mutters at the ash and patches the wounded."],
		["ash_berserker", "Horde Berserker", "Warband Berserker", "goblin_berserker", "Screams first, thinks never."],
		["ash_warchief", "Horde Warchief", "Patrol Leader", "ambush_berserker", "Leads the patrols that circle the ruins."],
	]
	for c: Array in chars:
		_chk("char " + str(c[0]), WorldCatalog.upsert("ashlands", "characters", {
			"id": c[0], "name": c[1], "title": c[2], "role": "enemy", "base_unit": c[3], "art": "", "background": c[4], "personality": "",
			"faction": "ashen_horde", "home": "", "disciplines": [], "passives": [], "named_items": [], "tuning": {}, "tags": []}))

	_chk("ash_scouts", WorldCatalog.upsert("ashlands", "encounters", _enc("ash_scouts", "Horde Scouts", "Two scouts testing the Wardens' patrols.", "Movement in the rubble: scouts.", ["ash_skirmisher", "ash_archer"])))
	_chk("ash_raiders", WorldCatalog.upsert("ashlands", "encounters", _enc("ash_raiders", "Horde Raiders", "A small raiding party looking for easy prey.", "Raiders spill out of the ruins.", ["ash_brawler", "ash_skirmisher", "ash_shaman"])))
	_chk("ash_patrol", WorldCatalog.upsert("ashlands", "encounters", _enc("ash_patrol", "Horde Patrol", "A warchief and his berserker on their rounds.", "The patrol has seen you.", ["ash_warchief", "ash_berserker"])))
	var pa: Dictionary = WorldCatalog.get_own("ashlands", "encounters", "pass_ambush")
	pa["location"] = "cinder_gate"
	_chk("pass_ambush", WorldCatalog.upsert("ashlands", "encounters", pa))

	_chk("cinder_gate", WorldCatalog.upsert("ashlands", "locations", {
		"id": "cinder_gate", "name": "The Cinder Gate",
		"description": "A collapsed toll arch at the top of the gorge, where the pass opens onto the wastes.",
		"kind": "gate", "region": "cinder_pass",
		"exits": [{"to": "ruined_courtyard", "label": "back down the gorge"}],
		"encounters": ["pass_ambush", "ash_scouts", "ash_raiders", "ash_patrol"], "tags": [],
		"map": "gorge_ruins",
		"zone_bindings": {"ruins": {"encounters": [["ash_scouts", 2], ["ash_raiders", 3]], "elites": [["ash_patrol", 1]], "elite_pct": 15, "steps_min": 10, "steps_max": 16}},
		"markers": [
			_m("to_gorge", "exit", "west", {"to": "ruined_courtyard", "to_anchor": "east_in"}),
			_m("ambush", "encounter", "site_b", {"encounter": "pass_ambush", "trigger": "proximity", "radius": 2, "once": false, "unless_flags": _flag("ashlands.pass_cleared")}),
			_m("beacon", "point", "altar", {"sets_flags": _flag("ashlands.gate_beacon_lit")}),
			_m("camp_cache", "chest", "camp_chest", {"items": {"con_heal_potion": 1}}),
		]}))

	var rc: Dictionary = WorldCatalog.get_own("ashlands", "locations", "ruined_courtyard")
	rc.merge({
		"exits": [{"to": "warden_keep", "label": "back to the keep"}, {"to": "cinder_gate", "label": "the gate road"}],
		"encounters": ["courtyard_skirmish", "ash_scouts", "ash_raiders", "ash_patrol"],
		"map": "gorge_ruins",
		"zone_bindings": {"ruins": {"encounters": [["ash_scouts", 3], ["ash_raiders", 2]], "elites": [["ash_patrol", 1]], "elite_pct": 10, "steps_min": 12, "steps_max": 20}},
		"markers": [
			_m("to_keep", "exit", "west", {"to": "warden_keep", "to_anchor": "spawn"}),
			_m("gate_road", "exit", "east", {"to": "cinder_gate", "to_anchor": "west_in", "requires_flags": _flag("ashlands.courtyard_held"), "locked_text": "Warden barricades close the gate road until the Courtyard is held."}),
			_m("warband", "encounter", "site_a", {"encounter": "courtyard_skirmish", "trigger": "proximity", "radius": 1, "once": false, "unless_flags": _flag("ashlands.courtyard_held")}),
			{"id": "patrol", "kind": "roamer", "requires_flags": [], "unless_flags": [], "sets_flags": [], "encounter": "ash_patrol", "path": "patrol", "radius": 1, "once": true},
			_m("ruin_cache", "chest", "ruin_chest", {"items": {"con_focus_elixir": 1}, "loot": "keep_stores"}),
			_m("old_stones", "lore", "ruin_lore", {"lore": "the_long_dry"}),
		]}, true)
	_chk("ruined_courtyard", WorldCatalog.upsert("ashlands", "locations", rc))

	var wk: Dictionary = WorldCatalog.get_own("ashlands", "locations", "warden_keep")
	wk.merge({
		"map": "walled_town", "zone_bindings": {},
		"markers": [
			_m("gate", "exit", "gate", {"to": "ruined_courtyard", "to_anchor": "west_in"}),
			_m("mara", "npc", "npc_a", {"character": "mara_voss"}),
			_m("tev", "npc", "npc_b", {"character": "tev_quill"}),
			_m("hest", "npc", "npc_c", {"character": "hest_the_quartermaster"}),
			_m("stores", "chest", "chest_a", {"items": {"con_heal_potion": 1}}),
			_m("hearth", "lore", "lore_a", {"lore": "one_fire_lit", "sets_flags": _flag("ashlands.hearth_read")}),
		]}, true)
	_chk("warden_keep", WorldCatalog.upsert("ashlands", "locations", wk))

	_save("save_world", WorldCatalog.save_world("ashlands"))
	print("MapCatalog.validate_all: ", MapCatalog.validate_all(), " warnings: ", MapCatalog.warnings_all())
	print("WorldCatalog.validate_all: ", WorldCatalog.validate_all())
	print("QuestCatalog.validate_all: ", QuestCatalog.validate_all())
	print("MapRules ashlands warnings: ", MapRules.warnings("ashlands"))
	print("DONE failed=", failed)
	quit(1 if failed else 0)
