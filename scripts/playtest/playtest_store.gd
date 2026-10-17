class_name PlaytestStore
extends RefCounted

# ==============================================================================
# Tactical RPG - Playtest Store (PLAYTEST1: ids, paths, listing, atomic JSON save)
# Recordings live in `user://playtests/<id>.json` with their note images in
# `user://playtests/<id>/`. Shared copies live in the repo under `res://playtests/`
# (inbox: user recordings, repro: agent repros, scenarios: one-seed scenarios).
# Everything is inert outside debug builds (`enabled()`; tests force it with `force_enabled`).
# ==============================================================================

const USER_ROOT: String = "user://playtests"
const REPO_ROOT: String = "res://playtests"
const REPO_DIRS: Array[String] = ["res://playtests/inbox", "res://playtests/repro"]
const SCENARIO_DIR: String = "res://playtests/scenarios"

static var root: String = USER_ROOT   # where new recordings are written (tests point it at a sandbox)
static var force_enabled: int = -1    # test seam: -1 = OS.is_debug_build(), 0 = off, 1 = on

static func enabled() -> bool:
	if force_enabled >= 0:
		return force_enabled == 1
	return OS.is_debug_build()

static func use_root(p_root: String) -> void:
	root = p_root

static func path_for(id: String, dir: String = "") -> String:
	return (root if dir.is_empty() else dir).path_join("%s.json" % id)

static func image_dir(id: String, dir: String = "") -> String:
	return (root if dir.is_empty() else dir).path_join(id)

# `YYYYMMDD-HHMMSS` in local time; `-2`, `-3`, ... when a recording of that id exists in `dir`.
static func new_id(dir: String = "", stamp: Dictionary = {}) -> String:
	var t: Dictionary = stamp if not stamp.is_empty() else Time.get_datetime_dict_from_system()
	var base: String = "%04d%02d%02d-%02d%02d%02d" % [DictUtil.int_of(t, "year"), DictUtil.int_of(t, "month"), DictUtil.int_of(t, "day"),
		DictUtil.int_of(t, "hour"), DictUtil.int_of(t, "minute"), DictUtil.int_of(t, "second")]
	var id: String = base
	var n: int = 2
	while FileAccess.file_exists(path_for(id, dir)) or DirAccess.dir_exists_absolute(image_dir(id, dir)):
		id = "%s-%d" % [base, n]
		n += 1
	return id

# Writes `data` as indented JSON (sorted keys) to `<path>.tmp`, then renames it over `path`.
static func save_json(path: String, data: Dictionary) -> Error:
	var dir: String = path.get_base_dir()
	if not DirAccess.dir_exists_absolute(dir):
		var made: Error = DirAccess.make_dir_recursive_absolute(dir)
		if made != OK:
			return made
	var tmp: String = path + ".tmp"
	var file := FileAccess.open(tmp, FileAccess.WRITE)
	if file == null:
		return FileAccess.get_open_error()
	file.store_string(JSON.stringify(data, "  ", true))
	file.close()
	var moved: Error = DirAccess.rename_absolute(tmp, path)
	if moved != OK and FileAccess.file_exists(path):
		DirAccess.remove_absolute(path)   # platforms whose rename refuses to replace an existing file
		moved = DirAccess.rename_absolute(tmp, path)
	return moved

# {ok, data, error}: the parsed JSON object with whole floats turned back into ints.
static func load_json(path: String) -> Dictionary:
	if not FileAccess.file_exists(path):
		return {"ok": false, "data": {}, "error": "File not found: %s" % path}
	var text: String = FileAccess.get_file_as_string(path)
	var json := JSON.new()
	if json.parse(text) != OK:
		return {"ok": false, "data": {}, "error": "JSON parse error in %s line %d: %s" % [path, json.get_error_line(), json.get_error_message()]}
	if not (json.data is Dictionary):
		return {"ok": false, "data": {}, "error": "%s is not a JSON object." % path}
	return {"ok": true, "data": DictUtil.dict_of(JsonUtil.normalize(json.data)), "error": ""}

# A tool argument as a path: res:// / user:// / absolute paths as given, a bare id searched in the user
# folder and the repo folders, anything else relative to the project.
static func resolve(arg: String) -> String:
	if arg.begins_with("res://") or arg.begins_with("user://") or arg.begins_with("/") or (arg.length() > 2 and arg[1] == ":"):
		return arg
	if not arg.ends_with(".json") and not arg.contains("/"):
		var candidates: Array[String] = [path_for(arg, USER_ROOT), "res://playtests/inbox".path_join(arg).path_join("%s.json" % arg), path_for(arg, "res://playtests/repro")]
		for c: String in candidates:
			if FileAccess.file_exists(c):
				return c
	return "res://" + arg

# Recordings in `dirs` (a flat `<id>.json` or an inbox folder `<id>/<id>.json`), newest first:
# [{id, path, dir, repo: bool, mtime}].
static func list(dirs: Array[String] = []) -> Array[Dictionary]:
	var search: Array[String] = dirs.duplicate()
	if search.is_empty():
		search.append(root)
		search.append_array(REPO_DIRS)
	var out: Array[Dictionary] = []
	for dir: String in search:
		if not DirAccess.dir_exists_absolute(dir):
			continue
		var repo: bool = dir.begins_with("res://")
		for f: String in DirAccess.get_files_at(dir):
			if f.ends_with(".json"):
				out.append(_listing(dir, dir.path_join(f), repo))
		for sub: String in DirAccess.get_directories_at(dir):
			var inner: String = dir.path_join(sub).path_join("%s.json" % sub)
			if FileAccess.file_exists(inner):
				out.append(_listing(dir, inner, repo))
	out.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		var ta: int = DictUtil.int_of(a, "mtime")
		var tb: int = DictUtil.int_of(b, "mtime")
		return ta > tb if ta != tb else str(a["id"]) > str(b["id"]))
	return out

static func _listing(dir: String, path: String, repo: bool) -> Dictionary:
	return {"id": path.get_file().get_basename(), "path": path, "dir": dir, "repo": repo, "mtime": FileAccess.get_modified_time(path)}

# The current git commit, read best effort from the checkout (empty when unknown).
static func source_rev() -> String:
	var git: String = ProjectSettings.globalize_path("res://.git")
	var git_dir: String = git
	if FileAccess.file_exists(git):   # a worktree: ".git" is a file "gitdir: <path>"
		var pointer: String = _read(git)
		if not pointer.begins_with("gitdir:"):
			return ""
		git_dir = pointer.substr(7).strip_edges()
	var head: String = _read(git_dir.path_join("HEAD"))
	if not head.begins_with("ref:"):
		return head.left(12)
	var ref: String = head.substr(4).strip_edges()
	var common: String = git_dir
	var common_file: String = _read(git_dir.path_join("commondir"))
	if not common_file.is_empty():
		common = git_dir.path_join(common_file).simplify_path()
	for base: String in [git_dir, common]:
		var rev: String = _read(base.path_join(ref))
		if not rev.is_empty():
			return rev.left(12)
	for line: String in _read(common.path_join("packed-refs")).split("\n"):
		if line.ends_with(" " + ref):
			return line.left(12)
	return ""

# File text, stripped; empty when the file does not exist (no engine error printed).
static func _read(path: String) -> String:
	return FileAccess.get_file_as_string(path).strip_edges() if FileAccess.file_exists(path) else ""
