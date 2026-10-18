---
class: AudioSettingsTestSuite
extends: RefCounted
description: "Tactical RPG - Audio settings tests (VOLUME-FIX, MUTE) Rules: saved volumes reach the (global) buses through the Apply path even with no AudioDirector in the tree (they only did when one existed, so a lowered Master did nothing at startup or in a menu run without one); every named bus exists and every player in the tree routes to one; `audio/mute` mutes the Master bus without touching the levels, the popup's Mute box follows the same draft / revert-on-close path as the sliders, it sits first in the Sound tab, and slider samples stay silent while muted. Settings go to a sandbox file."
---

# Properties
* `TAG`: String = "AUDIO-SETTINGS" (const)
* `VIEW`: Vector2 = Vector2(1280, 720) (const)

# Methods

### `static run(tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING AUDIO SETTINGS TEST (VOLUME-FIX, MUTE)  ===")
print("=======================================================")
TestHelpers.remove_sandbox(SANDBOX)
DirAccess.make_dir_recursive_absolute(SANDBOX)
SettingsFile.use_path(SANDBOX + "/settings.cfg")
AudioDirector.apply_saved()
var ok: bool = _test_apply_without_director()
ok = _test_buses_and_players(tree_parent) and ok
ok = _test_mute_apply() and ok
ok = _test_samples_silent_when_muted() and ok
if tree_parent != null:
	ok = _test_mute_row(tree_parent) and ok
	ok = _test_mute_draft(tree_parent) and ok
SettingsFile.use_path()
AudioDirector.apply_saved()
TestHelpers.remove_sandbox(SANDBOX)
return ok
```

### `static _master() -> int`
```logic
return AudioServer.get_bus_index("Master")
```

### `static _apply(key: String, value: Variant) -> void`
```logic
var m := SettingsModel.new()   # the same path the Apply button takes
var _e: Array[String] = m.set_value(key, value)
var _a: Array[String] = m.apply()
```

### `static _test_apply_without_director() -> bool`
```logic
var checks: Dictionary = {}
_apply("audio/master", 0.1)
checks["Master bus is at the saved level"] = absf(AudioServer.get_bus_volume_db(_master()) - linear_to_db(0.1)) < 0.01
checks["Master is lower than full"] = AudioServer.get_bus_volume_db(_master()) < -1.0
for key: String in ["audio/music", "audio/sfx", "audio/ui"]:
	_apply(key, 0.3)
for bus: String in ["Music", "SFX", "UI"]:
	checks[bus + " bus is at the saved level"] = absf(AudioServer.get_bus_volume_db(AudioServer.get_bus_index(bus)) - linear_to_db(0.3)) < 0.01
_apply("audio/master", 1.0)
checks["raising it again works"] = absf(AudioServer.get_bus_volume_db(_master())) < 0.01
return TestHelpers.report(TAG, "Apply puts the saved volumes on the buses with no director running", checks)
```

### `static _test_buses_and_players(tree_parent: Node) -> bool`
```logic
var checks: Dictionary = {}
for bus: String in AudioDirector.BUSES:
	checks[bus + " bus exists"] = AudioServer.get_bus_index(bus) >= 0
checks["music plays on the Music bus, which exists"] = MusicDirector.BUS == "Music" and AudioServer.get_bus_index(MusicDirector.BUS) >= 0
var tree: SceneTree = Engine.get_main_loop() as SceneTree
var roots: Array[Node] = [tree.root]
if tree_parent != null:
	roots.append(tree_parent)
var seen: int = 0
for r: Node in roots:
	for n: Node in r.find_children("*", "AudioStreamPlayer", true, false):
		seen += 1
		var p: AudioStreamPlayer = n as AudioStreamPlayer
		checks["player %s routes to an existing bus (%s)" % [p.get_path(), p.bus]] = AudioServer.get_bus_index(p.bus) >= 0 and p.bus != "Master"
checks["players were inspected (%d)" % seen] = true
return TestHelpers.report(TAG, "Every bus exists and every player routes to a named bus", checks)
```

### `static _test_mute_apply() -> bool`
```logic
var checks: Dictionary = {}
checks["default is not muted"] = not SettingsFile.get_bool("audio/mute") and SettingsFile.default_of("audio/mute") == false
_apply("audio/master", 0.6)
var before_db: float = AudioServer.get_bus_volume_db(_master())
_apply("audio/mute", true)
checks["muted: Master bus is muted"] = AudioServer.is_bus_mute(_master())
checks["muted: the level is kept"] = is_equal_approx(AudioServer.get_bus_volume_db(_master()), before_db)
checks["muted: saved in the file"] = SettingsFile.get_bool("audio/mute")
checks["muted: other buses are not muted"] = not AudioServer.is_bus_mute(AudioServer.get_bus_index("SFX"))
_apply("audio/mute", false)
checks["unmuted: Master plays again"] = not AudioServer.is_bus_mute(_master())
_apply("audio/master", 1.0)
return TestHelpers.report(TAG, "Mute mutes the Master bus and keeps the level", checks)
```

### `static _test_samples_silent_when_muted() -> bool`
```logic
var checks: Dictionary = {}
var d := AudioDirector.new()
d.record_previews = true
AudioDirector.set_mute(true)
checks["muted: no sample"] = not d.preview("audio/sfx", 0.5, 10.0) and d.previews.is_empty()
checks["muted: the bus level still follows the slider"] = absf(AudioServer.get_bus_volume_db(AudioServer.get_bus_index("SFX")) - linear_to_db(0.5)) < 0.01
AudioDirector.set_mute(false)
checks["unmuted: a sample is recorded"] = d.preview("audio/sfx", 0.5, 20.0) and d.previews.size() == 1
AudioDirector.apply_saved()
d.free()
return TestHelpers.report(TAG, "Slider samples stay silent while muted", checks)
```

### `static _host(tree_parent: Node) -> Array`
```logic
var packed: PackedScene = load("res://scenes/main_menu.tscn") as PackedScene
var holder := SubViewport.new()
holder.size = Vector2i(VIEW)
tree_parent.add_child(holder)
var host: MainMenuView = packed.instantiate() as MainMenuView
holder.add_child(host)
host.router.push(MenuRouter.SETTINGS)
SettingsUiTestSuite.layout(host)
return [holder, host]
```

### `static _free(holder: Node) -> void`
```logic
holder.get_parent().remove_child(holder)
holder.free()
```

### `static _test_mute_row(tree_parent: Node) -> bool`
```logic
var checks: Dictionary = {}
var pair: Array = _host(tree_parent)
var host: MainMenuView = pair[1] as MainMenuView
var grid: Control = host.find_child("SettingsGrid_Sound", true, false) as Control
var mute: CheckBox = host.find_child("Check_audio_mute", true, false) as CheckBox
var master: HSlider = host.find_child("Slider_audio_master", true, false) as HSlider
checks["the Mute box is in the Sound tab"] = grid != null and mute != null and mute.text == "Mute" and grid.is_ancestor_of(mute)
checks["it is above the Master slider"] = mute != null and master != null and mute.get_global_rect().position.y < master.get_global_rect().position.y
_free(pair[0] as Node)
return TestHelpers.report(TAG, "The Sound tab has a Mute box on top, next to Master", checks)
```

### `static _test_mute_draft(tree_parent: Node) -> bool`
```logic
var checks: Dictionary = {}
var pair: Array = _host(tree_parent)
var host: MainMenuView = pair[1] as MainMenuView
var mute: CheckBox = host.find_child("Check_audio_mute", true, false) as CheckBox
checks["the Mute box exists"] = mute != null
if mute != null:
	mute.button_pressed = true
	checks["ticking mutes Master at once (draft)"] = AudioServer.is_bus_mute(_master())
	checks["but nothing is saved yet"] = not SettingsFile.get_bool("audio/mute")
	host.cancel_pressed()   # asks about the unapplied change
	host.answer(true)       # discard
	checks["closing without Apply restores the sound"] = not AudioServer.is_bus_mute(_master()) and not AudioDirector.mute_now
	checks["the popup is closed"] = host.router.top() != MenuRouter.SETTINGS
AudioDirector.apply_saved()
_free(pair[0] as Node)
return TestHelpers.report(TAG, "The Mute box applies as a draft and reverts on close", checks)
```

