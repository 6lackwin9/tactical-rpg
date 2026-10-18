---
class: AudioDirector
extends: Node
description: "Tactical RPG - Audio director (S14-L3) One per main scene (combat arena, scene runner). Owns a small pool of AudioStreamPlayer voices on the SFX and UI buses and plays sounds by key from SoundBank. Headless runs (tests, fc.sh suites, the sim) create no voices, so they never touch the audio server beyond the bus layout. Volumes come from SettingsFile. `active` is the most recent director in the tree, for views that opt in to UI sounds."
---

# Properties
* `SFX_VOICES`: int = 8 (const)
* `UI_VOICES`: int = 2 (const)
* `BUSES`: Dictionary = {"Master": "audio/master", "Music": "audio/music", "SFX": "audio/sfx", "UI": "audio/ui"} (const)
* `record_previews`: bool = false
* `previews`: Array[Dictionary] = []
* `_throttle`: Variant
* `_music_bus`: Array[AudioStreamPlayer] = []   # one voice on the Music bus for the music slider's sample
* `_sfx`: Array[AudioStreamPlayer] = []
* `_ui`: Array[AudioStreamPlayer] = []
* `_started`: Dictionary = {}   # AudioStreamPlayer -> Time.get_ticks_usec() at its last start

# Methods

### `static enabled() -> bool`
```logic
return DisplayServer.get_name() != "headless"
```

### `static ui_event(event: String) -> void`
```logic
if active != null and is_instance_valid(active):
	active.ui(event)
```

### `_init() -> void`
```logic
name = "AudioDirector"
```

### `_enter_tree() -> void`
```logic
_stack.append(self)
```

### `_exit_tree() -> void`
```logic
_stack.erase(self)
```

### `_ready() -> void`
```logic
if not enabled():
	return
apply_saved()   # a director coming up (the arena) puts the saved levels on the buses, whatever ran before
for i: int in SFX_VOICES:
	_sfx.append(_voice("SFX"))
for i: int in UI_VOICES:
	_ui.append(_voice("UI"))
_music_bus.append(_voice("Music"))
```

### `_voice(bus: String) -> AudioStreamPlayer`
```logic
var p := AudioStreamPlayer.new()
p.bus = bus
add_child(p)
return p
```

### `preview(setting: String, linear: float, now: float = -1.0) -> bool`
```logic
set_bus_linear(setting, linear)
if mute_now:
	return false   # muted: the sample stays silent
var t: float = now if now >= 0.0 else float(Time.get_ticks_msec()) / 1000.0
if not _throttle.allow(t):
	return false
var sample: String = VolumePreview.sample_of(setting)
if sample.is_empty():
	return false
if record_previews:
	previews.append({"setting": setting, "sample": sample, "bus": VolumePreview.bus_of(setting), "level": linear})
match VolumePreview.bus_of(setting):
	"SFX":
		_play(_sfx, sample)
	"Music":
		_play(_music_bus, sample)
	_:
		_play(_ui, sample)
return true
```

### `reset_throttle() -> void`
```logic
_throttle.reset()
```

### `set_bus_linear(setting: String, linear: float) -> void`
```logic
AudioDirector.set_bus_level(VolumePreview.volume_bus_of(setting), linear)
```

### `voice_count() -> int`
```logic
return _sfx.size() + _ui.size()
```

### `play_sfx(key: String) -> void`
```logic
_play(_sfx, key)
```

### `ui(event: String) -> void`
```logic
_play(_ui, "ui_" + event)
```

### `connect_presenter(p: CombatPresenter) -> void`
```logic
if p != null and not p.fx_played.is_connected(_on_fx_played):
	p.fx_played.connect(_on_fx_played)
```

### `_on_fx_played(preset_id: String, entry: ActionLogEntry) -> void`
```logic
var key: String = SoundEvents.key_for(preset_id, entry)
if not key.is_empty():
	play_sfx(key)
```

### `apply_volumes() -> void`
```logic
AudioDirector.apply_saved()
```

### `static apply_saved() -> void`
```logic
mute_now = SettingsFile.get_bool("audio/mute")
for bus: String in BUSES:
	set_bus_level(bus, SettingsFile.get_float(str(BUSES[bus])))
```

### `static set_bus_level(bus: String, linear: float) -> void`
```logic
var idx: int = AudioServer.get_bus_index(bus)
if idx < 0:
	return
if bus == "Master":
	_master_linear = linear
AudioServer.set_bus_mute(idx, linear <= 0.0 or (bus == "Master" and mute_now))
AudioServer.set_bus_volume_db(idx, linear_to_db(maxf(linear, 0.0001)))
```

### `static set_mute(on: bool) -> void`
```logic
mute_now = on
set_bus_level("Master", _master_linear)
```

### `_play(voices: Array[AudioStreamPlayer], key: String) -> void`
```logic
if voices.is_empty() or key.is_empty():
	return
var stream: AudioStream = SoundBank.stream(key)
if stream == null:
	return
var pick: AudioStreamPlayer = null
var oldest: int = -1
for v: AudioStreamPlayer in voices:
	if not v.playing:
		pick = v
		break
	var at: int = DictUtil.to_int(_started.get(v, 0))
	if oldest < 0 or at < oldest:
		oldest = at
		pick = v
pick.stream = stream
pick.play()
_started[pick] = Time.get_ticks_usec()
```

