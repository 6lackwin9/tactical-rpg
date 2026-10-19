extends SceneTree
# MORALE1-A seed (idempotent): morale on the two Ashlands starter encounters.
# Run: tools/fc.sh run data/seeds/morale1.gd. Spec: docs/specs/MORALE1.md section 4.

var errs: Array[String] = []

func _init() -> void:
	WorldCatalog.reload_from_disk()
	_edit("ash_scouts", {"morale": true, "morale_break": 50, "morale_leader": "", "morale_groups": [],
		"morale_text": "The last scout drops its blade in the ash.",
		"description": "Two scouts testing the Wardens' patrols; drop one and the other yields."})
	_edit("ash_raiders", {"morale": true, "morale_break": 50, "morale_leader": "ash_shaman", "morale_groups": [],
		"morale_text": "With the smokecaller down, the raiders throw down their arms.",
		"description": "A small raiding party looking for easy prey; their smokecaller holds them together."})
	var failed: bool = not errs.is_empty()
	for e: String in errs:
		print("ERROR: ", e)
	if not failed and WorldCatalog.save_world("ashlands") != OK:
		failed = true
		print("ERROR save")
	print("morale1 seed ", "FAILED" if failed else "ok")
	quit(1 if failed else 0)

func _edit(id: String, changes: Dictionary) -> void:
	var e: Dictionary = WorldCatalog.get_own("ashlands", "encounters", id).duplicate(true)
	if e.is_empty():
		errs.append("MISSING encounter %s" % id)
		return
	for k: String in changes.keys():
		e[k] = changes[k]
	errs.append_array(WorldCatalog.upsert("ashlands", "encounters", e))
