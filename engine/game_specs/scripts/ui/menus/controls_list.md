---
class: ControlsList
extends: RefCounted
description: "Tactical RPG - Controls list (CONTROLS-HELP): the data behind the Settings \"Controls\" tab. Two sources, one list: (1) InputMap actions (project input actions, read live, so a changed binding shows up), (2) the keys the scenes handle themselves in _input / _unhandled_input (HARD_CODED). Each hard-coded row names the script that handles it and the tokens that must appear in that script's source; ControlsListTestSuite scans the sources, so the help cannot drift from the code. Rebinding is not supported; this is a read-only help."
---

# Properties
* `GROUP_MENUS`: String = "Menus" (const)
* `GROUP_MAPS`: String = "Maps" (const)
* `GROUP_COMBAT`: String = "Combat" (const)
* `GROUP_GENERAL`: String = "General" (const)
* `GROUP_TESTING`: String = "Testing"   # dev-only keys (arena edit mode, Scene Runner) (const)
* `GROUPS`: Array[String] = [GROUP_MENUS, GROUP_MAPS, GROUP_COMBAT, GROUP_GENERAL, GROUP_TESTING] (const)
* `INPUT_ACTIONS`: Array[Array] = [ (const)
* `HARD_CODED`: Array[Dictionary] = [ (const)
* `out`: Array[Dictionary] = []
* `h`: Dictionary = HARD_CODED[i]

# Methods

### `static action_keys(action: String) -> String`
```logic
if not InputMap.has_action(action):
	return ""
var names: Array[String] = []
for ev: InputEvent in InputMap.action_get_events(action):
	var text: String = ""
	if ev is InputEventKey:
		var key: InputEventKey = ev
		var code: Key = key.get_keycode_with_modifiers() if key.keycode != KEY_NONE else key.get_physical_keycode_with_modifiers()
		text = OS.get_keycode_string(code)
	elif ev is InputEventMouseButton:
		text = "Mouse button %d" % (ev as InputEventMouseButton).button_index
	if not text.is_empty() and not names.has(text):
		names.append(text)
return ", ".join(PackedStringArray(names))
```

