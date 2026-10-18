---
class: MainMenuModel
extends: RefCounted
description: "Tactical RPG - Main menu model (UI1-L): Continue (the newest save, autosave included; hidden when none), New game, Load, Settings, Tools (debug builds only) and Quit (not in the browser, where a tab can't be closed by the game)."
---

# Properties
* `router`: MenuRouter = null
* `slot`: int = SaveGame.newest_slot()
* `none`: Array[String] = ["There is no save to continue."]
* `none`: Array[String] = []

# Methods

### `_init(p_router: MenuRouter) -> void`
```logic
router = p_router
```

### `can_continue() -> bool`
```logic
return SaveGame.newest_slot() >= 0
```

### `show_tools() -> bool`
```logic
return OS.is_debug_build()
```

### `static version_text() -> String`
```logic
var version: String = str(ProjectSettings.get_setting("application/config/version", ""))
var engine: Dictionary = Engine.get_version_info()
return "Version %s · Godot %s.%s.%s · %s" % [version if not version.is_empty() else "dev", str(engine.get("major", "")), str(engine.get("minor", "")),
	str(engine.get("patch", "")), "debug" if OS.is_debug_build() else "release"]
```

### `show_quit() -> bool`
```logic
return not OS.has_feature("web")
```

### `open(id: String) -> void`
```logic
router.push(id)
```

