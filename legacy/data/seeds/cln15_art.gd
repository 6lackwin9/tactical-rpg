extends SceneTree
# CLN-15 asset seed (idempotent): map the goblin placeholder art keys to goblin units.
# Run: tools/fc.sh run data/seeds/cln15_art.gd

const ART: Dictionary = {
	"goblin_archer": "archer",
	"goblin_shaman": "shaman",
	"goblin_skirmisher": "skirmisher",
	"goblin_berserker": "berserker",
	"ambush_berserker": "berserker",
	"fleeing_scout": "scout",
	"goblin_brawler": "berserker",  # no own art; reuses berserker (user 2026-10-17)
}

func _init() -> void:
	ContentCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	for id: String in ART.keys():
		var e: Dictionary = ContentCatalog.get_entry("units", id).duplicate(true)
		if e.is_empty():
			print("MISSING units ", id)
			continue
		e["art"] = ART[id]
		var errs: Array[String] = ContentCatalog.upsert("units", e)
		if not errs.is_empty():
			print("ERROR ", id, ": ", errs)
	ContentCatalog.end_system_edit()
	print("validate_all: ", ContentCatalog.validate_all())
	print("warnings_all: ", ContentCatalog.warnings_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()
