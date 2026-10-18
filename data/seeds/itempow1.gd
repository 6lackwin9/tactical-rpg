extends SceneTree
# ITEMPOW1-A seed (idempotent): battle items worth a pouch slot at level 5, and more Ashlands start gold.
# Healing Draught heals 90, Throwing Fire Bomb does 45, Ashlands start gold 100.
# Run: tools/fc.sh run data/seeds/itempow1.gd. Spec: docs/specs/DESIGN_REVIEW2.md section 5 (option A).

var errs: Array[String] = []

func _init() -> void:
	ContentCatalog.reload_from_disk()
	WorldCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	_edit("items", "con_heal_potion", {"heal": 90,
		"description": "Herbal restorative that instantly patches wounds for +90 HP."})
	_edit("items", "con_fire_bomb", {"damage": 45,
		"description": "Clay vessel of volatile pitch exploding on impact for 45 damage."})
	ContentCatalog.end_system_edit()
	errs.append_array(WorldCatalog.update_manifest("ashlands", {"start_gold": 100}))
	var failed: bool = not errs.is_empty()
	for e: String in errs:
		print("ERROR: ", e)
	if not failed:
		if ContentCatalog.save_all() != OK or WorldCatalog.save_world("ashlands") != OK:
			failed = true
			print("ERROR save")
	print("itempow1 seed ", "FAILED" if failed else "ok")
	quit(1 if failed else 0)

func _edit(kind: String, id: String, changes: Dictionary) -> void:
	var e: Dictionary = ContentCatalog.get_entry(kind, id).duplicate(true)
	if e.is_empty():
		errs.append("MISSING %s %s" % [kind, id])
		return
	for k: String in changes.keys():
		e[k] = changes[k]
	errs.append_array(ContentCatalog.upsert(kind, e))
