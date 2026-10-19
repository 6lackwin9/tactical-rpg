extends SceneTree

# Reference screenshot of the Godot combat screen (goblin_skirmish, round 1 planning) for the engine GUI work.
# Needs a rendering window (not --headless):
#   godot --path <project> --resolution 1280x720 -s <abs path> -- out=<abs png> [encounter=goblin_skirmish] [frames=30]

var out_path: String = ""
var frames_left: int = 30

func _initialize() -> void:
	var encounter_id: String = "goblin_skirmish"
	for arg: String in OS.get_cmdline_user_args():
		var kv: PackedStringArray = arg.split("=", true, 1)
		if kv.size() == 2:
			match kv[0]:
				"out":
					out_path = kv[1]
				"encounter":
					encounter_id = kv[1]
				"frames":
					frames_left = int(kv[1])
	var scene: PackedScene = load("res://scenes/combat_arena.tscn")
	var arena: Node = scene.instantiate()
	arena.set("encounter", CatalogEncounter.new(encounter_id))
	root.add_child(arena)

func _process(_delta: float) -> bool:
	frames_left -= 1
	if frames_left > 0:
		return false
	var img: Image = root.get_texture().get_image()
	var err: Error = img.save_png(out_path)
	print("§ shot: ", out_path, " ", error_string(err), " ", img.get_size())
	return true
