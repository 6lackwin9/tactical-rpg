extends SceneTree
# INTENT1-A seed (idempotent): passive wardens_eye (foresight) + granted to ashlands bryn_hale.
# Run: tools/fc.sh run data/seeds/intent1_a.gd. Spec: docs/specs/INTENT1.md section 5.

var failed: bool = false

func _chk(label: String, errs: Array[String]) -> void:
	if not errs.is_empty():
		failed = true
		print("ERROR ", label, ": ", errs)

func _init() -> void:
	ContentCatalog.reload_from_disk()
	WorldCatalog.reload_from_disk()
	# Shipped content is a system asset (origin_test_suite): upsert inside the system-edit scope.
	ContentCatalog.begin_system_edit()
	_chk("passive", ContentCatalog.upsert("passives", {
		"id": "wardens_eye",
		"name": "Warden's Eye",
		"description": "Reads the foe coming for you: shows the whole round plan of every enemy that targets this hero.",
		"effects": [{"type": "foresight"}],
		"origin": "system",
	}))
	ContentCatalog.end_system_edit()
	var b: Dictionary = WorldCatalog.get_own("ashlands", "characters", "bryn_hale")
	var ps: Array = b["passives"]
	if not ps.has("wardens_eye"):
		ps.append("wardens_eye")
	_chk("bryn", WorldCatalog.upsert("ashlands", "characters", b))
	var unit_errs: Dictionary = ContentCatalog.validate_all()
	var world_errs: Dictionary = WorldCatalog.validate_all()
	failed = failed or not unit_errs.is_empty() or not world_errs.is_empty()
	print("units validate_all: ", unit_errs)
	print("units save: ", ContentCatalog.save_all())
	print("world validate_all: ", world_errs)
	print("world save: ", WorldCatalog.save_world("ashlands"))
	print("FAILED" if failed else "OK")
	quit()
