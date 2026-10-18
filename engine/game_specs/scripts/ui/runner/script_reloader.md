---
class: ScriptReloader
extends: RefCounted
description: "Tactical RPG - Script Reloader (hot-reload GDScript edited on disk) Remembers each project script's modification time; reload_changed() re-reads the ones edited since and recompiles them in place (GDScript.reload), so the next scene instance runs the new code without restarting the game. Limits (Godot): a script with a NEW class_name, or a brand-new file referenced by class_name, isn't known to the running game's class registry; those are reported as \"new\" and need a full restart (SceneRunner's Restart button). Running instances keep their state; relaunch the scene to rebuild them."
---

# Properties
* `ROOTS`: Array[String] = ["res://scripts", "res://scenes"] (const)
* `_mtimes`: Dictionary = {}   # res path -> modified time when last loaded / reloaded
* `out`: Array[String] = []

# Methods

### `_init() -> void`
```logic
mark_all_current()
```

### `static _collect(dir_path: String, out: Array[String]) -> void`
```logic
var dir: DirAccess = DirAccess.open(dir_path)
if dir == null:
	return
for sub: String in dir.get_directories():
	if not sub.begins_with("."):
		_collect(dir_path.path_join(sub), out)
for file: String in dir.get_files():
	if file.ends_with(".gd"):
		out.append(dir_path.path_join(file))
```

### `mark_all_current() -> void`
```logic
_mtimes.clear()
for path: String in script_paths():
	_mtimes[path] = FileAccess.get_modified_time(path)
```

### `pending() -> Dictionary`
```logic
var changed: Array[String] = []
var added: Array[String] = []
for path: String in script_paths():
	var mtime: int = FileAccess.get_modified_time(path)
	if not _mtimes.has(path):
		added.append(path)
	elif DictUtil.int_of(_mtimes, path) != mtime:
		changed.append(path)
return {"changed": changed, "new": added}
```

### `reload_changed(only: Array[String] = [], skip: Array[String] = []) -> Dictionary`
```logic
var todo: Array[String] = []
var added: Array[String] = []
if only.is_empty():
	var found: Dictionary = pending()
	var changed: Array = found["changed"]
	var fresh: Array = found["new"]
	todo.assign(changed)
	added.assign(fresh)
else:
	todo = only.duplicate()
var reloaded: Array[String] = []
var failed: Array[String] = []
for path: String in todo:
	if skip.has(path):
		continue
	var err: Error = reload_script(path)
	if err == OK:
		reloaded.append(path)
	else:
		failed.append("%s: %s" % [path, error_string(err)])
	_mtimes[path] = FileAccess.get_modified_time(path)
# Static caches hold instances built by the old code: drop them.
if not reloaded.is_empty():
	SkillCatalog.clear_cache()
	SvgRaster.clear_cache()
return {"reloaded": reloaded, "failed": failed, "new": added}
```

### `static reload_script(path: String) -> Error`
```logic
if not ResourceLoader.exists(path):
	return ERR_FILE_NOT_FOUND
var script: GDScript = load(path) as GDScript
if script == null:
	return ERR_CANT_OPEN
script.source_code = FileAccess.get_file_as_string(path)
return script.reload(true)
```

