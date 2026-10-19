extends SceneTree
# S14-A2 asset seed (idempotent): writes the 13 placeholder blips to res://assets/audio/sfx/<key>.wav through
# PlaceholderSfx.write (the generator behind tools/gen_placeholder_sfx.gd, same args) and upserts one manifest row
# per SoundEvents key. Real sounds later replace a file (or its path) under the same key, no code change.
# Run: tools/fc.sh run data/seeds/s14_a2.gd

# key: [wave, hz_from, hz_to, ms, envelope, gain_db]
const SFX: Dictionary = {
	"hit": ["noise", 0.0, 0.0, 80, "decay", 0.0],
	"crit": ["noise_sine", 220.0, 220.0, 140, "decay", 0.0],
	"mitigated": ["sine", 180.0, 180.0, 90, "decay", 0.0],
	"heal": ["sine", 440.0, 660.0, 220, "fade", 0.0],
	"buff": ["sine", 523.0, 523.0, 160, "fade", 0.0],
	"debuff": ["sine", 330.0, 220.0, 200, "fade", 0.0],
	"defend": ["sine", 150.0, 150.0, 120, "decay", 0.0],
	"move_step": ["noise", 0.0, 0.0, 40, "decay", -12.0],
	"defeat": ["sine", 260.0, 110.0, 400, "slow", 0.0],
	"ui_click": ["sine", 880.0, 880.0, 30, "decay", -12.0],
	"ui_confirm": ["sine", 988.0, 988.0, 60, "decay", 0.0],
	"ui_back": ["sine", 660.0, 660.0, 50, "decay", 0.0],
	"ui_error": ["square", 140.0, 140.0, 120, "decay", 0.0],
}

func _init() -> void:
	AssetCatalog.reload_from_disk()
	for key: String in SFX:
		var a: Array = SFX[key]
		var path: String = "%s/%s.wav" % [AssetRegistry.folder_of("sfx"), key]
		var err: String = PlaceholderSfx.write(path, str(a[0]), DictUtil.to_float(a[1]), DictUtil.to_float(a[2]), DictUtil.to_int(a[3]), str(a[4]), DictUtil.to_float(a[5]))
		if not err.is_empty():
			print("ERROR ", key, ": ", err)
			continue
		var entry: Dictionary = {
			"id": AssetCatalog.id_for("sfx", key, "stream"), "namespace": "sfx", "key": key, "variant": "stream",
			"path": path, "alias_of": "", "size": [], "origin": "user", "note": "placeholder blip (generated)",
		}
		var errs: Array[String] = AssetCatalog.upsert(entry)
		if not errs.is_empty():
			print("ERROR ", entry["id"], ": ", errs)
	print("assets validate_all: ", AssetCatalog.validate_all())
	print("assets save: ", AssetCatalog.save())
	# Checks the files as written (the imported copies refresh at the next `godot --headless --import`).
	for key: String in SoundEvents.keys():
		var s: AudioStreamWAV = AudioStreamWAV.load_from_file(AssetCatalog.file_of("%s/%s.wav" % [AssetRegistry.folder_of("sfx"), key]))
		var want: float = DictUtil.to_float(DictUtil.array_of(SFX.get(key, [])).get(3)) / 1000.0
		print("check %s: %s" % [key, "ok" if s != null and absf(s.get_length() - want) <= 0.005 else "BAD"])
	quit()
