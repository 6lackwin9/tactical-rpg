---
class: SoundBank
extends: RefCounted
description: "Tactical RPG - Sound bank (S14-L3) The one place sounds are looked up: by `sfx` key through AssetRegistry, never by path. An unresolved key is null (silent) and warns once per key per session."
---

# Methods

### `static stream(key: String) -> AudioStream`
```logic
var s: AudioStream = AssetRegistry.stream("sfx", key)
if s == null and not _warned.has(key):
	_warned[key] = true
	_warn_count += 1
	if not _quiet:
		push_warning("SoundBank: no sound for '%s'." % key)
return s
```

### `static has(key: String) -> bool`
```logic
return AssetRegistry.stream("sfx", key) != null
```

### `static clear_warned() -> void`
```logic
_warned.clear()
_warn_count = 0
```

### `static warning_count() -> int`
```logic
return _warn_count
```

### `static set_quiet(quiet: bool) -> void`
```logic
_quiet = quiet
```

