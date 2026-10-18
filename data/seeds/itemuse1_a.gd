extends SceneTree
# ITEMUSE1-A seed (idempotent): use_reach on three shipped consumables (spec docs/specs/ITEMUSE1.md section 4).
# Run: tools/fc.sh run data/seeds/itemuse1_a.gd

const REACH: Dictionary = {
	"con_heal_potion": 1,
	"con_focus_elixir": 0,
	"con_fire_bomb": 3,
}

func _init() -> void:
	ContentCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	for id: String in REACH:
		var entry: Dictionary = ContentCatalog.get_entry("items", id).duplicate(true)
		entry["use_reach"] = REACH[id]
		var errs: Array = ContentCatalog.upsert("items", entry)
		if not errs.is_empty():
			print("ERROR %s: %s" % [id, str(errs)])
	ContentCatalog.end_system_edit()
	print("validate_all: ", ContentCatalog.validate_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()
