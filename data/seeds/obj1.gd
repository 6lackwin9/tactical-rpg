extends SceneTree
# OBJ1-A seed (idempotent): objectives on ashlands courtyard_skirmish, wagon_raid, ashmaw stands; wenn_tallow;
# iron_reaches south_road scripted fight; the pass_ambush intro (Bryn-first). Encounter `party` lists stay Mara/Tev:
# the campaign party comes from start_party, and tidewrack re-skins them (world_test_suite).
# Run: tools/fc.sh run data/seeds/obj1.gd. Spec: docs/specs/OBJ1.md section 4.

var failed: bool = false

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)

func _init() -> void:
	ContentCatalog.reload_from_disk()
	WorldCatalog.reload_from_disk()
	# Text pass: a new game starts with Bryn Hale alone.
	var pa: Dictionary = WorldCatalog.get_own("ashlands", "encounters", "pass_ambush")
	pa["intro"] = "Too quiet. Bryn raises a hand: smoke, upwind, much too close."
	_chk("pass_ambush intro", WorldCatalog.upsert("ashlands", "encounters", pa))
	# Wenn
	_chk("wenn_tallow", WorldCatalog.upsert("ashlands", "characters", {
		"id": "wenn_tallow", "name": "Wenn Tallow", "title": "Orsa's Carter", "role": "npc",
		"base_unit": "ranger", "art": "wenn_tallow", "faction": "", "home": "",
		"background": "Orsa's nephew; drives the steel wagon and swears he can shoot.",
	}))
	# Courtyard
	var c: Dictionary = WorldCatalog.get_own("ashlands", "encounters", "courtyard_skirmish")
	c["objective"] = "hold"
	c["objective_tiles"] = []
	c["objective_rounds"] = 3
	c["deployment"] = "standard"
	c["objective_text"] = "Hold the Courtyard: keep the ring for 3 rounds, or drive them off."
	_chk("courtyard_skirmish", WorldCatalog.upsert("ashlands", "encounters", c))
	# Wagon raid
	var w: Dictionary = WorldCatalog.get_own("ashlands", "encounters", "ash_raiders").duplicate(true)
	w["id"] = "wagon_raid"
	w["name"] = "Raid on the Steel Wagon"
	w["description"] = "Raiders hit Orsa's wagon on the south road; Wenn must live to drive it home."
	w["intro"] = "The wagon lurches. Goblins come out of the ditch on both sides."
	w["deployment"] = "ambush"
	w["guests"] = ["wenn_tallow"]
	w["protect"] = ["wenn_tallow"]
	w["objective"] = "rout"
	_chk("wagon_raid", WorldCatalog.upsert("ashlands", "encounters", w))
	# Stands
	for id: String in ["ashmaw_stand", "ashmaw_stand_barred"]:
		var s: Dictionary = WorldCatalog.get_own("ashlands", "encounters", id)
		s["objective"] = "slay"
		s["objective_unit"] = "grakka_ashmaw"
		s["objective_text"] = "Break Grakka Ashmaw; the Horde scatters when he falls."
		_chk(id, WorldCatalog.upsert("ashlands", "encounters", s))
	# Route
	var r: Dictionary = WorldCatalog.get_own("iron_reaches", "routes", "south_road")
	var n: int = 0
	var sc: Array = r.get("scripted", [])
	for s: Variant in sc:
		if s is Dictionary and str((s as Dictionary).get("id", "")) == "wagon_raid":
			(s as Dictionary)["encounter"] = "ashlands:wagon_raid"
			n += 1
	if n != 1:
		failed = true
		print("ERROR south_road scripted wagon_raid not found")
	r["scripted"] = sc
	_chk("south_road", WorldCatalog.upsert("iron_reaches", "routes", r))
	var unit_errs: Dictionary = ContentCatalog.validate_all()
	var world_errs: Dictionary = WorldCatalog.validate_all()
	failed = failed or not unit_errs.is_empty() or not world_errs.is_empty()
	print("units validate_all: ", unit_errs)
	print("world validate_all: ", world_errs)
	var sv: Variant = WorldCatalog.save_world("ashlands")
	var sv2: Variant = WorldCatalog.save_world("iron_reaches")
	print("world save: ", sv, " ", sv2)
	failed = failed or str(sv) != "0" or str(sv2) != "0"
	print("FAILED" if failed else "OK")
	quit()
