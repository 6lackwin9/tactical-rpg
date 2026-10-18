extends SceneTree
# MUSIC1-A asset seed (idempotent): one manifest row per generated music loop (files made by tools/gen_music.py).
# Run: tools/fc.sh run data/seeds/music1_a.gd

const KEYS: Array[String] = ["menu_theme", "ashlands", "iron_reaches", "tidewrack", "combat"]

func _init() -> void:
	AssetCatalog.reload_from_disk()
	for key: String in KEYS:
		var path: String = "res://assets/audio/music/%s.ogg" % key
		var entry: Dictionary = {
			"id": AssetCatalog.id_for("music", key, "stream"), "namespace": "music", "key": key, "variant": "stream",
			"path": path, "alias_of": "", "size": [], "origin": "user", "note": "generated loop (MUSIC1)",
		}
		var errs: Array[String] = AssetCatalog.upsert(entry)
		if not errs.is_empty():
			print("ERROR ", entry["id"], ": ", errs)
	print("assets validate_all: ", AssetCatalog.validate_all())
	print("assets save: ", AssetCatalog.save())
	for key: String in KEYS:
		var s: AudioStreamOggVorbis = AudioStreamOggVorbis.load_from_file(AssetCatalog.file_of("res://assets/audio/music/%s.ogg" % key))
		print("check %s: %s" % [key, ("ok len=%.2fs" % s.get_length()) if s != null else "BAD"])
	quit()
