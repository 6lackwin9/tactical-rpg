extends SceneTree
# ITEMUSE1-A2 seed (idempotent): empty class pouch on warrior/ranger/priest/mage (decision "No free class potion").
# Run: tools/fc.sh run data/seeds/itemuse1_a2.gd

const CLASS_IDS: Array[String] = ["warrior", "ranger", "priest", "mage"]

func _init() -> void:
	ContentCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	for id: String in CLASS_IDS:
		var entry: Dictionary = ContentCatalog.get_entry("classes", id).duplicate(true)
		entry["pouch"] = []
		var errs: Array = ContentCatalog.upsert("classes", entry)
		if not errs.is_empty():
			print("ERROR %s: %s" % [id, str(errs)])
	ContentCatalog.end_system_edit()
	print("validate_all: ", ContentCatalog.validate_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()
