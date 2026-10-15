extends SceneTree
# M2C2B phase D seed (idempotent): phase-2 families (X1/X2: cyclone_kick, inferno, glacial_burst, shield_wall), the
# final class paths, and rogue/mage party units (level 5 heroes, like warrior/ranger) so all six classes have one.
# Run: tools/fc.sh run data/seeds/m2c2b_d.gd
const Lib = preload("res://data/seeds/m2c2b_lib.gd")

func _init() -> void:
	ContentCatalog.reload_from_disk()
	var errs: Array[String] = []
	errs.append_array(Lib.attack_family({"id": "cyclone_kick", "name": "Cyclone Kick", "ap": [3, 4, 5], "blocks": 3, "power": [0.9, 1.1, 1.3], "area": "SWEEP", "status": "stagger", "pot": [50, 55, 60], "text": "Bare hands or feet."}))
	errs.append_array(Lib.attack_family({"id": "inferno", "name": "Inferno", "ap": [4, 5, 5], "blocks": 3, "power": [0.8, 1.0, 1.2], "area": "BURST", "rmin": 2, "rmax": 3, "status": "bleed", "pot": [50, 60, 70], "rounds": 2, "wt": ["staff", "wand"], "minp": 2}))
	errs.append_array(Lib.attack_family({"id": "glacial_burst", "name": "Glacial Burst", "ap": [3, 4, 5], "blocks": 3, "power": [0.6, 0.75, 0.9], "area": "BURST", "rmin": 2, "rmax": 3, "status": "root", "pot": [60, 70, 80], "wt": ["staff", "wand"], "minp": 2}))
	errs.append_array(Lib.defend_family({"id": "shield_wall", "name": "Shield Wall", "ap": [3, 4, 5], "blocks": 3, "mit": [0.35, 0.45, 0.55], "rounds": [1, 2, 3], "grant": "taunt", "text": "Foes that can reach you must target you, starting next round."}))
	errs.append_array(Lib.apply_paths())
	for cid: String in ["rogue", "mage"]:
		var u: Dictionary = ContentCatalog.get_entry("units", "warrior").duplicate(true)
		u["id"] = cid
		u["name"] = cid.capitalize()
		u["class_id"] = cid
		u["description"] = "Skirmish party %s: a level 5 %s." % [cid, cid.capitalize()]
		errs.append_array(ContentCatalog.upsert("units", u))
	Lib.finish(errs, self)
