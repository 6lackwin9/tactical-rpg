extends SceneTree
# MAPREAD1-A seed (idempotent): marker labels/notes, map_pos, 48x30 templates for the four Ashlands locations.
# Run: tools/fc.sh run data/seeds/mapread1_a.gd
# map_pos adjusted to the drawn backdrop glows (keep, ruins, gate, camp), not the spec's first guess.

const MAPS: Dictionary = {"warden_keep": "ashlands_warden_keep", "ruined_courtyard": "ashlands_ruined_courtyard", "cinder_gate": "ashlands_cinder_gate", "ashmaw_camp": "ashlands_ashmaw_camp"}
const POS: Dictionary = {"warden_keep": [0.19, 0.65], "ruined_courtyard": [0.365, 0.55], "cinder_gate": [0.58, 0.33], "ashmaw_camp": [0.78, 0.28]}
const TEXT: Dictionary = {
	"cinder_gate": {"beacon": ["Signal beacon", "You light the old Warden beacon. Smoke climbs over the pass."], "wastes_road": ["World map: the wastes", ""]},
	"warden_keep": {"signpost": ["World map: the roads", ""], "hearth": ["The keep's hearth", ""]},
}

func _init() -> void:
	WorldCatalog.reload_from_disk()
	MapCatalog.reload_from_disk()
	var failed: bool = false
	for id: String in MAPS:
		var e: Dictionary = WorldCatalog.get_own("ashlands", "locations", id)
		e["map"] = MAPS[id]
		e["map_pos"] = POS[id]
		var ms: Array = e.get("markers", [])
		var tx: Dictionary = TEXT.get(id, {})
		for m: Variant in ms:
			var md: Dictionary = m
			if tx.has(str(md["id"])):
				var t: Array = tx[str(md["id"])]
				md["label"] = t[0]
				if str(t[1]) != "":
					md["note"] = t[1]
		var errs: Array[String] = WorldCatalog.upsert("ashlands", "locations", e)
		if not errs.is_empty():
			failed = true
			print("ERROR ", id, ": ", errs)
	print("world validate_all: ", WorldCatalog.validate_all())
	print("world save: ", WorldCatalog.save_world("ashlands"))
	print("FAILED" if failed else "OK")
	quit()
