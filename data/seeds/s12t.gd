extends SceneTree
# S12T-A asset seed (idempotent): the 17 terrain rows (system origin). Move costs = old Terrain.INFO; cover and
# hazard numbers are PLACEHOLDERS for the balance pass. Run: tools/fc.sh run data/seeds/s12t.gd

# id: [move_cost, cover_guard, hazard_status, potency, magnitude, rounds, on]
const ROWS: Dictionary = {
	"floor": [1, 0.0, "", 100, 0, 1, "both"],
	"grass": [1, 0.0, "", 100, 0, 1, "both"],
	"dirt": [1, 0.0, "", 100, 0, 1, "both"],
	"stone": [1, 0.0, "", 100, 0, 1, "both"],
	"sand": [2, 0.0, "", 100, 0, 1, "both"],
	"snow": [2, 0.0, "", 100, 0, 1, "both"],
	"shallow_water": [2, 0.0, "", 100, 0, 1, "both"],
	"deep_water": [1, 0.0, "", 100, 0, 1, "both"],
	"mud": [2, 0.0, "", 100, 0, 1, "both"],
	"lava": [1, 0.0, "", 100, 0, 1, "both"],
	"wall": [1, 0.30, "", 100, 0, 1, "both"],
	"tree": [1, 0.15, "", 100, 0, 1, "both"],
	"bridge": [1, 0.0, "", 100, 0, 1, "both"],
	"rubble": [2, 0.15, "", 100, 0, 1, "both"],
	"burning_ground": [1, 0.0, "burn", 100, 5, 1, "both"],
	"toxic_bog": [2, 0.0, "poison", 80, 4, 2, "both"],
	"ice": [1, 0.0, "slow", 100, 0, 1, "enter"],
}

func _init() -> void:
	ContentCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	for id: String in ROWS:
		var a: Array = ROWS[id]
		var e: Dictionary = {"id": id, "move_cost": a[0], "cover_guard": a[1], "hazard_status": a[2],
			"hazard_potency": a[3], "hazard_magnitude": a[4], "hazard_rounds": a[5], "hazard_on": a[6]}
		var errs: Array[String] = ContentCatalog.upsert("terrain", e)
		if not errs.is_empty():
			print("ERROR ", id, ": ", errs)
	ContentCatalog.end_system_edit()
	print("validate_all: ", ContentCatalog.validate_all())
	print("warnings_all: ", ContentCatalog.warnings_all())
	print("save: ", ContentCatalog.save("terrain"))
	quit()
