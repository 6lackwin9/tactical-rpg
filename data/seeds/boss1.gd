extends SceneTree
# BOSS1-A seed (idempotent): Grakka's phases on ashlands ashmaw_stand and ashmaw_stand_barred.
# Run: godot --headless --path . -s data/seeds/boss1.gd. Spec: docs/specs/BOSS1.md section 4.

var failed: bool = false

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)

func _crown() -> Dictionary:
	return {
		"id": "crown_of_nails",
		"name": "Crown of Nails",
		"trigger": {"kind": "hp_below", "unit": "grakka_ashmaw", "pct": 50},
		"effects": [
			{"type": "line", "text": "Grakka tears the nail-crown from his head and hurls it into the fire. The smokecallers begin to chant."},
			{"type": "status", "unit": "grakka_ashmaw", "status": "fortify", "rounds": 3, "magnitude": 5},
			{"type": "status", "unit": "grakka_ashmaw", "status": "regen", "rounds": 3, "magnitude": 4},
			{"type": "ai", "unit": "grakka_ashmaw", "reckless": true, "target_pref": "WEAKEST"},
			{"type": "ai", "unit": "ash_shaman", "heal_below": 0.9},
		],
		"pause_autoplay": true,
	}

func _gorge() -> Dictionary:
	return {
		"id": "empty_gorge",
		"name": "The Empty Gorge",
		"trigger": {"kind": "round", "round": 4},
		"effects": [{"type": "line", "text": "Grakka bellows at the gorge. Nothing answers."}],
		"pause_autoplay": false,
	}

func _init() -> void:
	ContentCatalog.reload_from_disk()
	WorldCatalog.reload_from_disk()
	var a: Dictionary = WorldCatalog.get_own("ashlands", "encounters", "ashmaw_stand")
	a["description"] = "Grakka and his guard make a stand; the flankers pour out of the gorge at Round 2; at half strength Grakka throws off his crown."
	a["phases"] = [_crown()]
	_chk("ashmaw_stand", WorldCatalog.upsert("ashlands", "encounters", a))
	var b: Dictionary = WorldCatalog.get_own("ashlands", "encounters", "ashmaw_stand_barred")
	b["description"] = "Grakka and his guard stand alone; the barred gate cut off the flankers; at half strength Grakka throws off his crown."
	b["phases"] = [_crown(), _gorge()]
	_chk("ashmaw_stand_barred", WorldCatalog.upsert("ashlands", "encounters", b))
	var unit_errs: Dictionary = ContentCatalog.validate_all()
	var world_errs: Dictionary = WorldCatalog.validate_all()
	failed = failed or not unit_errs.is_empty() or not world_errs.is_empty()
	print("units validate_all: ", unit_errs)
	print("world validate_all: ", world_errs)
	print("world save: ", WorldCatalog.save_world("ashlands"))
	print("FAILED" if failed else "OK")
	quit()
