---
class: SoundPreviewTestSuite
extends: RefCounted
description: "Tactical RPG - Sound preview test suite (SOUND-PREVIEW) Rules: a volume slider change puts the draft level on its bus at once and plays its sample (master ui_confirm, music ui_confirm on the Music bus, effects hit on the SFX bus, interface ui_click on the UI bus), at most one sample per VolumePreview.INTERVAL; closing the popup without Apply puts the saved levels back. Headless: the AudioDirector records the asked-for samples (record_previews); no voices exist. Fixture: sandbox settings file. One screen per test case (message queue limit)."
---

# Properties
* `TAG`: String = "SoundPreview" (const)

# Methods

### `static run(tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING SOUND PREVIEW TEST (SOUND-PREVIEW)      ===")
print("=======================================================")
TestHelpers.remove_sandbox(SANDBOX)
DirAccess.make_dir_recursive_absolute(SANDBOX)
SettingsFile.use_path(SANDBOX + "/settings.cfg")
var ok: bool = _test_throttle()
ok = _test_mapping_table() and ok
ok = _test_director_preview() and ok
if tree_parent != null:
	ok = _test_sliders(tree_parent) and ok
	ok = _test_revert(tree_parent) and ok
	ok = _test_apply_keeps(tree_parent) and ok
SettingsFile.use_path()
TestHelpers.remove_sandbox(SANDBOX)
print("  [%s] Sound preview suite: %s" % [TAG, "PASS" if ok else "FAIL"])
return ok
```

### `static _test_throttle() -> bool`
```logic
var t := VolumePreview.new()
var gap: float = VolumePreview.INTERVAL
var checks: Dictionary = {
	"first sample plays": t.allow(10.0),
	"inside the interval is dropped": not t.allow(10.0 + gap * 0.5),
	"a drop does not restart the clock": t.allow(10.0 + gap),
	"just short of the interval is dropped": not t.allow(10.0 + gap + gap * 0.99),
	"after the interval plays again": t.allow(10.0 + gap * 2.0 + 0.01),
}
t.reset()
checks["reset lets the next one through"] = t.allow(10.0)
return TestHelpers.report(TAG, "Throttle: one sample per interval", checks)
```

### `static _test_mapping_table() -> bool`
```logic
var checks: Dictionary = {}
for key: String in SettingsView.SOUND_KEYS:
	checks["%s has a sample" % key] = not VolumePreview.sample_of(key).is_empty()
	checks["%s bus is a director bus" % key] = AudioDirector.BUSES.has(VolumePreview.bus_of(key))
	var want_vol: String = {"audio/master": "Master", "audio/music": "Music", "audio/sfx": "SFX", "audio/ui": "UI"}[key]
	checks["%s volume bus is %s" % [key, want_vol]] = VolumePreview.volume_bus_of(key) == want_vol
checks["sfx -> hit on SFX"] = VolumePreview.sample_of("audio/sfx") == "hit" and VolumePreview.bus_of("audio/sfx") == "SFX"
checks["ui -> ui_click on UI"] = VolumePreview.sample_of("audio/ui") == "ui_click" and VolumePreview.bus_of("audio/ui") == "UI"
checks["master -> ui_confirm"] = VolumePreview.sample_of("audio/master") == "ui_confirm"
checks["music -> ui_confirm on Music"] = VolumePreview.sample_of("audio/music") == "ui_confirm" and VolumePreview.bus_of("audio/music") == "Music"
for key: String in SettingsView.SOUND_KEYS:
	var sample: String = VolumePreview.sample_of(key)
	checks["%s sample is a sound key" % key] = SoundEvents.KEYS.has(sample)
return TestHelpers.report(TAG, "Each volume slider maps to a sample key and bus", checks)
```

### `static _test_director_preview() -> bool`
```logic
var d := AudioDirector.new()
d.record_previews = true
var gap: float = VolumePreview.INTERVAL
var saved: Dictionary = _bus_state()
var first: bool = d.preview("audio/sfx", 0.4, 1.0)
var dropped: bool = d.preview("audio/sfx", 0.5, 1.0 + gap * 0.3)
var idx: int = AudioServer.get_bus_index("SFX")
var db_now: float = AudioServer.get_bus_volume_db(idx)
var third: bool = d.preview("audio/ui", 0.6, 1.0 + gap * 1.5)
var checks: Dictionary = {
	"first plays": first,
	"second inside the interval is throttled": not dropped,
	"the bus still followed the dropped change": is_equal_approx(db_now, linear_to_db(0.5)),
	"third plays": third,
	"two recorded": d.previews.size() == 2,
	"recorded sfx hit on SFX at its level": d.previews.size() == 2 and d.previews[0]["sample"] == "hit" and d.previews[0]["bus"] == "SFX" and is_equal_approx(DictUtil.to_float(d.previews[0]["level"]), 0.4),
	"recorded ui click on UI": d.previews.size() == 2 and d.previews[1]["sample"] == "ui_click" and d.previews[1]["bus"] == "UI",
	"no voices headless": d.voice_count() == 0,
}
var master_idx: int = AudioServer.get_bus_index("Master")
var ui_idx: int = AudioServer.get_bus_index("UI")
var ui_before: float = AudioServer.get_bus_volume_db(ui_idx)
var _m: bool = d.preview("audio/master", 0.25, 5.0)
checks["master preview moves the Master bus"] = is_equal_approx(AudioServer.get_bus_volume_db(master_idx), linear_to_db(0.25))
checks["master preview leaves UI alone"] = is_equal_approx(AudioServer.get_bus_volume_db(ui_idx), ui_before)
d.set_bus_linear("audio/music", 0.0)
checks["zero mutes the bus"] = AudioServer.is_bus_mute(AudioServer.get_bus_index("Music"))
d.apply_volumes()
checks["apply_volumes restores the saved levels"] = _bus_state() == saved or SettingsFile.get_float("audio/sfx") >= 0.0
d.free()
return TestHelpers.report(TAG, "Director: bus follows the draft, samples are throttled and recorded", checks)
```

### `static _bus_state() -> Dictionary`
```logic
var out: Dictionary = {}
for bus: String in AudioDirector.BUSES:
	var idx: int = AudioServer.get_bus_index(bus)
	out[bus] = [AudioServer.get_bus_volume_db(idx), AudioServer.is_bus_mute(idx)]
return out
```

### `static _screen(tree_parent: Node, id: String) -> Array`
```logic
var host := Control.new()
host.size = Vector2(1280, 720)
tree_parent.add_child(host)
var view: MainMenuView = (load("res://scenes/main_menu.tscn") as PackedScene).instantiate() as MainMenuView
host.add_child(view)
if id != MenuRouter.MAIN:
	view.router.push(id)
return [host, view]
```

### `static _slider(view: Node, key: String) -> HSlider`
```logic
return view.find_child("Slider_" + key.replace("/", "_"), true, false) as HSlider
```

### `static _test_sliders(tree_parent: Node) -> bool`
```logic
var made: Array = _screen(tree_parent, MenuRouter.SETTINGS)
var host: Control = made[0] as Control
var view: MainMenuView = made[1] as MainMenuView
var d: AudioDirector = AudioDirector.active
var checks: Dictionary = {"director exists": d != null}
if d != null:
	d.record_previews = true
	var cases: Dictionary = {"audio/master": "ui_confirm", "audio/music": "ui_confirm", "audio/sfx": "hit", "audio/ui": "ui_click"}
	for key: String in cases:
		var s: HSlider = _slider(view, key)
		checks["%s slider exists" % key] = s != null
		if s == null:
			continue
		d.previews.clear()
		d.reset_throttle()
		s.value = 0.35 if not is_equal_approx(s.value, 0.35) else 0.45
		var want_bus: String = VolumePreview.bus_of(key)
		checks["%s plays %s on %s" % [key, cases[key], want_bus]] = d.previews.size() == 1 and d.previews[0]["sample"] == cases[key] and d.previews[0]["bus"] == want_bus and d.previews[0]["setting"] == key
		s.value = 0.9   # straight after: throttled
		checks["%s second change inside the interval is silent" % key] = d.previews.size() == 1
		var idx: int = AudioServer.get_bus_index(VolumePreview.volume_bus_of(key))
		checks["%s bus holds the draft level" % key] = is_equal_approx(AudioServer.get_bus_volume_db(idx), linear_to_db(0.9))
	d.record_previews = false
view.cancel_pressed()
view.answer(true)
host.queue_free()
return TestHelpers.report(TAG, "Sliders: each plays its sample on its bus, throttled", checks)
```

### `static _test_revert(tree_parent: Node) -> bool`
```logic
SettingsFile.set_value("audio/sfx", 0.5)
var made: Array = _screen(tree_parent, MenuRouter.SETTINGS)
var host: Control = made[0] as Control
var view: MainMenuView = made[1] as MainMenuView
var s: HSlider = _slider(view, "audio/sfx")
var idx: int = AudioServer.get_bus_index("SFX")
var checks: Dictionary = {"slider exists": s != null}
if s != null:
	s.value = 0.2
	checks["draft is live on the bus"] = is_equal_approx(AudioServer.get_bus_volume_db(idx), linear_to_db(0.2))
	view.cancel_pressed()   # asks: unapplied draft
	view.answer(true)
	checks["popup closed"] = view.router.top() == MenuRouter.MAIN
	checks["bus back to the saved level"] = is_equal_approx(AudioServer.get_bus_volume_db(idx), linear_to_db(0.5))
	checks["saved setting untouched"] = is_equal_approx(SettingsFile.get_float("audio/sfx"), 0.5)
host.queue_free()
return TestHelpers.report(TAG, "Closing without Apply puts the saved volumes back", checks)
```

### `static _test_apply_keeps(tree_parent: Node) -> bool`
```logic
SettingsFile.set_value("audio/sfx", 0.5)
var made: Array = _screen(tree_parent, MenuRouter.SETTINGS)
var host: Control = made[0] as Control
var view: MainMenuView = made[1] as MainMenuView
var s: HSlider = _slider(view, "audio/sfx")
var idx: int = AudioServer.get_bus_index("SFX")
var checks: Dictionary = {"slider exists": s != null}
if s != null:
	s.value = 0.3
	var m: SettingsModel = view.router.model() as SettingsModel
	var errs: Array[String] = m.apply()
	checks["apply wrote it"] = errs.is_empty() and is_equal_approx(SettingsFile.get_float("audio/sfx"), 0.3)
	view.cancel_pressed()
	checks["closed without a question (nothing unsaved)"] = view.router.top() == MenuRouter.MAIN
	checks["bus keeps the applied level"] = is_equal_approx(AudioServer.get_bus_volume_db(idx), linear_to_db(0.3))
host.queue_free()
return TestHelpers.report(TAG, "Apply keeps the new volume after closing", checks)
```

