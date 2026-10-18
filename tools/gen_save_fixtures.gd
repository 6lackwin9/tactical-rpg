extends SceneTree

# Regenerates the committed sample saves scripts/tests/fixtures/saves/save_v1.json .. save_v<SaveGame.VERSION>.json
# (SAVE-COMPAT). Builds a played Ashlands state, writes it as the current version through the real envelope, then
# derives each older version by dropping what the later migration steps added (see SaveMigrations).
#   tools/fc.sh run tools/gen_save_fixtures.gd
# Run it only when a save-format change needs the fixtures updated; review the diff of the fixtures.

const OUT_DIR: String = "res://scripts/tests/fixtures/saves"
const SAVED_AT: int = 1700000000

func _init() -> void:
	ContentCatalog.use_root(ContentCatalog.DEFAULT_ROOT)
	WorldCatalog.use_root(WorldCatalog.DEFAULT_ROOT)
	var res: Dictionary = NewGame.create_for_world("ashlands", 12345)
	var g: GameState = GameState.of(res["state"])
	if g == null:
		push_error("gen_save_fixtures: can't create a new Ashlands game: %s" % str(res.get("errors", [])))
		quit(1)
		return
	# A pinned roster (the pre-RECRUIT1 four heroes at the world's start XP), so the fixtures do not follow the shipped
	# start_party (RECRUIT1-A cut it to one hero) and older fixtures stay byte-identical when a version is added.
	var start_xp: int = g.party.members[0].total_xp if not g.party.members.is_empty() else 0
	var roster: Array[UnitState] = []
	for base: String in ["warrior", "ranger", "rogue", "mage"]:
		var m: UnitState = UnitState.create(base, base)
		m.total_xp = start_xp
		roster.append(m)
	g.party.members = roster
	g.party.active_ids = [] as Array[String]
	g.location_id = "ashlands:warden_keep"
	g.party.gold = 75
	g.party.add_to_stash("con_heal_potion", 2)
	g.party.members[0].carried = ["con_heal_potion"] as Array[String]
	g.party.members[0].equipment = {"body": "arm_leather"}
	g.party.members[0].total_xp = XpCurve.xp_for_level(3) + 5
	g.set_flag("intro.done", true)
	g.quests = {
		"ashlands:hold_the_courtyard": {"state": "active", "objective": 1},
		"ashlands:done_quest": {"state": "completed", "objective": 2},
	}
	g.loot_claimed = ["ashlands:wardens_drill"] as Array[String]
	g.shops = {"ashlands:hest_stores": {"con_focus_elixir": 1}}
	g.explore = {"steps": 9, "next_at": 20, "loc_steps": 4, "last_safe": "ashlands:warden_keep"}
	g.position = Vector2i(2, 3)
	g.time = 240
	g.party.supplies = 7
	g.party.members[0].pouch = ["con_heal_potion", "con_heal_potion", "con_fire_bomb"] as Array[String]
	g.autoplay = {"on": true, "party": "aggressive", "heroes": {g.party.members[0].id: "balanced", g.party.members[1].id: PlanPresets.CUSTOM}}
	var errors: Array[String] = g.validate()
	if not errors.is_empty():
		push_error("gen_save_fixtures: played state invalid: %s" % str(errors))
		quit(1)
		return
	var v8: Dictionary = g.to_dict()
	var v7: Dictionary = v8.duplicate(true)
	for raw: Variant in DictUtil.array_of(DictUtil.dict_of(v7["party"]).get("members", [])):
		DictUtil.dict_of(raw).erase("pouch")
	var v6: Dictionary = v7.duplicate(true)
	v6.erase("autoplay")
	var v5: Dictionary = v6.duplicate(true)
	DictUtil.dict_of(v5["party"]).erase("supplies")
	var v4: Dictionary = v5.duplicate(true)
	v4.erase("time")
	var v3: Dictionary = v4.duplicate(true)
	v3.erase("shops")
	DictUtil.dict_of(v3["party"]).erase("gold")
	var v2: Dictionary = v3.duplicate(true)
	for key: String in ["position", "explore", "rng_seed", "rolls"]:
		v2.erase(key)
	v2["location_id"] = "warden_keep"   # v2 saves had an unqualified location id
	var v1: Dictionary = v2.duplicate(true)
	v1.erase("loot_claimed")
	var by_version: Dictionary = {1: v1, 2: v2, 3: v3, 4: v4, 5: v5, 6: v6, 7: v7, 8: v8}
	for version: int in range(1, SaveGame.VERSION + 1):
		if not by_version.has(version):
			push_error("gen_save_fixtures: no derivation for save version %d; add one to by_version first (nothing written)." % version)
			quit(1)
			return
	DirAccess.make_dir_recursive_absolute(OUT_DIR)
	for version: int in range(1, SaveGame.VERSION + 1):
		var envelope: Dictionary = {"format": SaveGame.FORMAT, "version": version, "saved_at": SAVED_AT, "game_state": by_version[version]}
		var f: FileAccess = FileAccess.open("%s/save_v%d.json" % [OUT_DIR, version], FileAccess.WRITE)
		if f == null:
			push_error("gen_save_fixtures: can't open save_v%d.json for writing." % version)
			quit(1)
			return
		f.store_string(JSON.stringify(envelope, "\t"))
		f.close()
		print("§ wrote save_v%d.json" % version)
	quit(0)
