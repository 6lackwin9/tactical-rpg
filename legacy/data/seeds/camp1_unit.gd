extends SceneTree
# CAMP1-A1 asset seed (idempotent): catalog unit goblin_warlord (boss of the Ashmaw Camp finale).
# Run: tools/fc.sh run data/seeds/camp1_unit.gd

func _init() -> void:
	ContentCatalog.reload_from_disk()
	var u: Dictionary = ContentCatalog.get_entry("units", "goblin_brawler").duplicate(true)
	u["id"] = "goblin_warlord"
	u["name"] = "Goblin Warlord"
	u["description"] = ""
	u["class_id"] = "warrior"
	u["level"] = 6
	u["tier"] = "boss"
	u["art"] = "berserker"
	u["flee_ratio"] = 0.0
	u["reckless"] = false
	ContentCatalog.begin_system_edit()
	var errs: Array = ContentCatalog.upsert("units", u)
	ContentCatalog.end_system_edit()
	print("upsert errors: ", errs)
	print("validate_all: ", ContentCatalog.validate_all())
	print("save_all: ", ContentCatalog.save_all())
	quit(1 if not errs.is_empty() else 0)
