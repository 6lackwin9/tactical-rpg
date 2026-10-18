extends SceneTree
# SPRITE1-A seed (idempotent): marks the unit token and portrait rows that now hold pixel-art sprites (six party
# classes and the named party characters that use them). The files come from tools/sprites/export_units.py.
# Run: python3 tools/sprites/export_units.py && godot --headless --import && tools/fc.sh run data/seeds/sprite1_a.gd

const KEYS: Array[String] = [
	"warrior", "ranger", "brawler", "priest", "rogue", "mage",
	"mara_voss", "bryn_hale", "tev_quill", "sella_marsh", "dagny_coal", "alda_wren", "kit_vane", "oren_dusk",
]
const NOTE: String = "pixel-art sprite, idle frame (SPRITE1-A)"

var failed: bool = false

func _init() -> void:
	AssetCatalog.reload_from_disk()
	AssetCatalog.begin_system_edit()
	for key: String in KEYS:
		for variant: String in ["token", "portrait"]:
			var id: String = AssetCatalog.id_for("unit", key, variant)
			var row: Dictionary = AssetCatalog.get_entry(id).duplicate(true)
			if row.is_empty():
				failed = true
				print("ERROR missing row ", id)
				continue
			row["note"] = NOTE
			var errs: Array[String] = AssetCatalog.upsert(row)
			if not errs.is_empty():
				failed = true
				print("ERROR ", id, ": ", errs)
	AssetCatalog.end_system_edit()
	if not failed and AssetCatalog.save() != OK:
		failed = true
		print("ERROR save")
	print("sprite1_a seed ", "FAILED" if failed else "ok")
	quit(1 if failed else 0)
