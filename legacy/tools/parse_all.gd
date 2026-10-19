extends SceneTree

# Parses every .gd file in one Godot process (replaces ~130 process launches).
# Prints "§ FAIL <path>" per script that fails to compile, and "§ parsed N scripts, F failed".
func _initialize() -> void:
	var files: Array[String] = []
	_collect("res://", files)
	var failed: int = 0
	for path: String in files:
		var script: Resource = ResourceLoader.load(path, "", ResourceLoader.CACHE_MODE_REPLACE)
		if script == null:
			print("§ FAIL ", path)
			failed += 1
	print("§ parsed %d scripts, %d failed" % [files.size(), failed])
	quit(1 if failed > 0 else 0)

func _collect(dir_path: String, out: Array[String]) -> void:
	var dir: DirAccess = DirAccess.open(dir_path)
	if dir == null:
		return
	for sub: String in dir.get_directories():
		if not sub.begins_with("."):
			_collect(dir_path.path_join(sub), out)
	for f: String in dir.get_files():
		if f.ends_with(".gd"):
			out.append(dir_path.path_join(f))
