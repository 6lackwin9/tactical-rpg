extends SceneTree
# TRAV2-A asset seed (idempotent): Ashlands supply sources and start supplies (docs/specs/TRAV2_A.md sections 2-3).
# Run: tools/fc.sh run res://data/seeds/trav2a.gd

var failed: bool = false

func _own(kind: String, id: String) -> Dictionary:
	var e: Dictionary = WorldCatalog.get_own("ashlands", kind, id)
	if e.is_empty():
		failed = true
		print("ERROR missing ashlands:", kind, ":", id)
	return e

func _chest(loc: String, marker: String, n: int) -> void:
	if failed:
		return
	var e: Dictionary = _own("locations", loc)
	if e.is_empty():
		return
	var found: bool = false
	var ms: Array = e.get("markers", [])
	for m: Variant in ms:
		if m is Dictionary and str((m as Dictionary).get("id", "")) == marker:
			(m as Dictionary)["supplies"] = n
			found = true
	if not found:
		failed = true
		print("ERROR missing marker ", loc, ":", marker)
		return
	e["markers"] = ms
	var errs: Array[String] = WorldCatalog.upsert("ashlands", "locations", e)
	if not errs.is_empty():
		failed = true
		print("ERROR ", loc, ": ", errs)

func _init() -> void:
	var merrs: Variant = WorldCatalog.update_manifest("ashlands", {"start_supplies": 10})
	if merrs is Array and not (merrs as Array).is_empty():
		failed = true
		print("ERROR manifest: ", merrs)
	var shop: Dictionary = _own("shops", "hest_stores")
	if not failed:
		shop["supply_price"] = 2
		var errs: Array[String] = WorldCatalog.upsert("ashlands", "shops", shop)
		if not errs.is_empty():
			failed = true
			print("ERROR shop: ", errs)
	_chest("warden_keep", "stores", 4)
	_chest("ruined_courtyard", "ruin_cache", 6)
	_chest("cinder_gate", "camp_cache", 6)
	if failed:
		print("TRAV2-A FAILED, not saving")
		quit(1)
		return
	print("validate_all: ", WorldCatalog.validate_all())
	print("save_world: ", WorldCatalog.save_world("ashlands"))
	quit(0)
