extends SceneTree
# ART2-A seed (idempotent): manifest rows for the new generated placeholders (intent chips, foresight/slay/guest chips,
# overlay_objective_v0) plus the `miss` SFX (written through PlaceholderSfx.write).
# Run: python3 tools/gen_art.py chips overlays && godot --headless --import && tools/fc.sh run data/seeds/art2_a.gd

const KINDS: Array[String] = ["attack", "area", "support", "guard", "focus", "rest", "retreat", "move", "flee", "switch", "stunned", "act", "arriving"]
const NOTE: String = "generated placeholder (ART2)"

func _row(ns: String, key: String, variant: String, path: String, size: Array) -> void:
	var entry: Dictionary = {
		"id": AssetCatalog.id_for(ns, key, variant), "namespace": ns, "key": key, "variant": variant,
		"path": path, "alias_of": "", "size": size, "origin": "user", "note": NOTE,
	}
	var errs: Array[String] = AssetCatalog.upsert(entry)
	if not errs.is_empty():
		print("ERROR ", entry["id"], ": ", errs)

func _png(ns: String, key: String, variant: String, rel: String) -> void:
	var path: String = "%s/%s" % [AssetRegistry.folder_of(ns), rel]
	var img: Image = AssetRegistry.load_image(AssetCatalog.file_of(path))
	if img == null or img.is_empty():
		print("ERROR unreadable ", path)
		return
	_row(ns, key, variant, path, [img.get_width(), img.get_height()])

func _init() -> void:
	AssetCatalog.reload_from_disk()
	for k: String in KINDS:
		_png("ui", "intent_" + k, "icon", "intent_%s_icon.png" % k)
	for k: String in ["badge_foresight", "chip_slay", "chip_guest"]:
		_png("ui", k, "icon", k + "_icon.png")
	_png("tile", "overlay_objective", "v0", "overlay_objective_v0.png")
	var path: String = "%s/miss.wav" % AssetRegistry.folder_of("sfx")
	var err: String = PlaceholderSfx.write(path, "noise", 0.0, 0.0, 60, "fade", -6.0)
	if not err.is_empty():
		print("ERROR miss: ", err)
	else:
		_row("sfx", "miss", "stream", path, [])
	print("assets validate_all: ", AssetCatalog.validate_all())
	print("assets save: ", AssetCatalog.save())
	quit()
