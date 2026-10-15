extends SceneTree
# SK2 schema slice (idempotent): rewrites the catalog once so every skill row carries the new default fields
# (advanced = false, unlock_offset = -1). No content changes; the asset step (data/seeds/sk2.gd) sets the real values.
# Run: tools/fc.sh run data/seeds/sk2_schema.gd

func _init() -> void:
	ContentCatalog.reload_from_disk()
	print("validate_all: ", ContentCatalog.validate_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()
