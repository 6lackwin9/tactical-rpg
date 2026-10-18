---
class: ContentFingerprint
extends: RefCounted
description: "Tactical RPG - Content Fingerprint (PLAYTEST1 rule 3) Every *.json under `root` (recursively, sorted path), FileAccess.get_sha256 cut to 12 hex chars; `total` = the same cut of the SHA-256 of the joined \"path:hash\n\" lines. A recording is stale when the totals differ; staleness warns and never blocks a replay."
---

# Properties
* `ALGO`: String = "sha256-12" (const)
* `DEFAULT_ROOT`: String = "res://data" (const)
* `CUT`: int = 12 (const)

# Methods

### `static compute(root: String = DEFAULT_ROOT) -> Dictionary`
```logic
var paths: Array[String] = []
_collect(root.trim_suffix("/"), paths)
paths.sort()
var files: Dictionary = {}
var lines: String = ""
for path: String in paths:
	var h: String = FileAccess.get_sha256(path).left(CUT)
	files[path] = h
	lines += "%s:%s\n" % [path, h]
return {"algo": ALGO, "total": lines.sha256_text().left(CUT), "files": files}
```

### `static _collect(dir: String, out: Array[String]) -> void`
```logic
if not DirAccess.dir_exists_absolute(dir):
	return
for f: String in DirAccess.get_files_at(dir):
	if f.get_extension() == "json":
		out.append(dir.path_join(f))
for sub: String in DirAccess.get_directories_at(dir):
	_collect(dir.path_join(sub), out)
```

### `static diff(recorded: Dictionary, current: Dictionary) -> Dictionary`
```logic
var rec_files: Dictionary = DictUtil.dict_of(recorded.get("files", {}))
var cur_files: Dictionary = DictUtil.dict_of(current.get("files", {}))
var added: Array[String] = []
var removed: Array[String] = []
var changed: Array[String] = []
for path: Variant in cur_files:
	if not rec_files.has(path):
		added.append(str(path))
	elif str(rec_files[path]) != str(cur_files[path]):
		changed.append(str(path))
for path: Variant in rec_files:
	if not cur_files.has(path):
		removed.append(str(path))
added.sort()
removed.sort()
changed.sort()
var stale: bool = str(recorded.get("total", "")) != str(current.get("total", ""))
return {"stale": stale, "added": added, "removed": removed, "changed": changed}
```

