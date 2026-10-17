extends SceneTree

# Writes a generated placeholder blip as .wav (S14-L3 rule 5; S14-A2 lists the values).
#   godot --headless --path . -s tools/gen_placeholder_sfx.gd -- <key> <wave> <hz_from> <hz_to> <ms> <envelope> <gain_db>
#   tools/fc.sh gen_sfx <key> <wave> <hz_from> <hz_to> <ms> <envelope> <gain_db>
# wave sine|square|noise|noise_sine; envelope decay|fade|slow. Writes res://assets/audio/sfx/<key>.wav and prints
# "§ wrote <path> <ms>ms". The manifest row is added separately (AssetCatalog.upsert in a seed).

func _init() -> void:
	var args: PackedStringArray = OS.get_cmdline_user_args()
	if args.size() < 7:
		print("§ usage: gen_placeholder_sfx <key> <wave> <hz_from> <hz_to> <ms> <envelope> <gain_db>")
		quit(2)
		return
	var path: String = "%s/%s.wav" % [AssetRegistry.folder_of("sfx"), args[0]]
	var err: String = PlaceholderSfx.write(path, args[1], float(args[2]), float(args[3]), int(args[4]), args[5], float(args[6]))
	if not err.is_empty():
		print("§ %s" % err)
		quit(1)
		return
	print("§ wrote %s %dms" % [path, int(args[4])])
	quit(0)
