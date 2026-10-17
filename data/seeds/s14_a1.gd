extends SceneTree
# S14-A1 asset seed (idempotent): first manifest rows (goblin_brawler alias, four sample file overrides that are PNG
# exports of today's placeholders) and goblin_brawler.art as a registry key. Export the PNGs first:
#   tools/fc.sh export_placeholder unit warrior token   (and unit warrior portrait, tile rubble v0)
#   tools/fc.sh export_placeholder item runed_staff icon 2   (RARE: wpn_staff's rarity, so the file equals its template)
# Run: tools/fc.sh run data/seeds/s14_a1.gd

const ROWS: Array[Dictionary] = [
	{"namespace": "unit", "key": "goblin_brawler", "variant": "token", "path": "", "alias_of": "berserker", "size": [], "origin": "user", "note": "CLN-15 reuse: goblin brawler has no own art yet"},
	{"namespace": "unit", "key": "goblin_brawler", "variant": "portrait", "path": "", "alias_of": "berserker", "size": [], "origin": "user", "note": "CLN-15 reuse"},
	{"namespace": "unit", "key": "warrior", "variant": "token", "path": "res://assets/art/unit/warrior_token.png", "alias_of": "", "size": [104, 104], "origin": "system", "note": "sample override: PNG export of the placeholder glyph layer"},
	{"namespace": "unit", "key": "warrior", "variant": "portrait", "path": "res://assets/art/unit/warrior_portrait.png", "alias_of": "", "size": [240, 240], "origin": "system", "note": "sample override: PNG export of the placeholder"},
	{"namespace": "item", "key": "runed_staff", "variant": "icon", "path": "res://assets/art/item/runed_staff_icon.png", "alias_of": "", "size": [96, 96], "origin": "system", "note": "sample override of a unique icon"},
	{"namespace": "tile", "key": "rubble", "variant": "v0", "path": "res://assets/art/tile/rubble_v0.png", "alias_of": "", "size": [108, 108], "origin": "system", "note": "sample override; serves all four variants"},
]

func _init() -> void:
	AssetCatalog.reload_from_disk()
	AssetCatalog.begin_system_edit()
	for row: Dictionary in ROWS:
		var entry: Dictionary = row.duplicate(true)
		entry["id"] = AssetCatalog.id_for(str(row["namespace"]), str(row["key"]), str(row["variant"]))
		var errs: Array[String] = AssetCatalog.upsert(entry)
		if not errs.is_empty():
			print("ERROR ", entry["id"], ": ", errs)
	AssetCatalog.end_system_edit()
	print("assets validate_all: ", AssetCatalog.validate_all())
	print("assets save: ", AssetCatalog.save())

	ContentCatalog.reload_from_disk()
	ContentCatalog.begin_system_edit()
	var unit: Dictionary = ContentCatalog.get_entry("units", "goblin_brawler").duplicate(true)
	if unit.is_empty():
		print("MISSING units goblin_brawler")
	else:
		unit["art"] = "goblin_brawler"
		var errs: Array[String] = ContentCatalog.upsert("units", unit)
		if not errs.is_empty():
			print("ERROR goblin_brawler: ", errs)
	ContentCatalog.end_system_edit()
	print("validate_all: ", ContentCatalog.validate_all())
	print("warnings_all: ", ContentCatalog.warnings_all())
	print("save_all: ", ContentCatalog.save_all())
	quit()
