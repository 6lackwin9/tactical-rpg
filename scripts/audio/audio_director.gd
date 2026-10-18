class_name AudioDirector
extends Node

# ==============================================================================
# Tactical RPG - Audio director (S14-L3)
# One per main scene (combat arena, scene runner). Owns a small pool of
# AudioStreamPlayer voices on the SFX and UI buses and plays sounds by key from
# SoundBank. Headless runs (tests, fc.sh suites, the sim) create no voices, so they
# never touch the audio server beyond the bus layout. Volumes come from SettingsFile.
# `active` is the most recent director in the tree, for views that opt in to UI sounds.
# ==============================================================================

const SFX_VOICES: int = 8
const UI_VOICES: int = 2
const BUSES: Dictionary = {"Master": "audio/master", "Music": "audio/music", "SFX": "audio/sfx", "UI": "audio/ui"}

static var _stack: Array[AudioDirector] = []

# The most recently entered director still in the tree, or null.
static var active: AudioDirector:
	get:
		for i: int in range(_stack.size() - 1, -1, -1):
			if is_instance_valid(_stack[i]):
				return _stack[i]
		return null

var record_previews: bool = false
var previews: Array[Dictionary] = []

var _throttle := VolumePreview.new()
var _music_bus: Array[AudioStreamPlayer] = []   # one voice on the Music bus for the music slider's sample
var _sfx: Array[AudioStreamPlayer] = []
var _ui: Array[AudioStreamPlayer] = []
var _started: Dictionary = {}   # AudioStreamPlayer -> Time.get_ticks_usec() at its last start

static func enabled() -> bool:
	return DisplayServer.get_name() != "headless"

# Plays the UI sound for an event ("click" -> ui_click) on the active director, if any.
static func ui_event(event: String) -> void:
	if active != null and is_instance_valid(active):
		active.ui(event)

func _init() -> void:
	name = "AudioDirector"

func _enter_tree() -> void:
	_stack.append(self)

func _exit_tree() -> void:
	_stack.erase(self)

func _ready() -> void:
	if not enabled():
		return
	for i: int in SFX_VOICES:
		_sfx.append(_voice("SFX"))
	for i: int in UI_VOICES:
		_ui.append(_voice("UI"))
	_music_bus.append(_voice("Music"))

func _voice(bus: String) -> AudioStreamPlayer:
	var p := AudioStreamPlayer.new()
	p.bus = bus
	add_child(p)
	return p

# ------------------------------------------------------------------------------
# Volume previews (SOUND-PREVIEW)
# `previews` is the recording seam (headless tests): one entry per sample asked for, when `record_previews` is on.
# ------------------------------------------------------------------------------

# A volume slider moved: the bus follows the draft level at once and, at most every VolumePreview.INTERVAL seconds, the key's
# sample plays on its bus (the music slider plays a short sample on the Music bus). `now` is in seconds (the clock is injectable for tests).
func preview(setting: String, linear: float, now: float = -1.0) -> bool:
	set_bus_linear(setting, linear)
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

func reset_throttle() -> void:
	_throttle.reset()

# One bus (by its SettingsFile key) to a linear level, without touching the saved settings.
func set_bus_linear(setting: String, linear: float) -> void:
	var idx: int = AudioServer.get_bus_index(VolumePreview.volume_bus_of(setting))
	if idx < 0:
		return
	AudioServer.set_bus_mute(idx, linear <= 0.0)
	AudioServer.set_bus_volume_db(idx, linear_to_db(maxf(linear, 0.0001)))

func voice_count() -> int:
	return _sfx.size() + _ui.size()

func play_sfx(key: String) -> void:
	_play(_sfx, key)

func ui(event: String) -> void:
	_play(_ui, "ui_" + event)

func connect_presenter(p: CombatPresenter) -> void:
	if p != null and not p.fx_played.is_connected(_on_fx_played):
		p.fx_played.connect(_on_fx_played)

func _on_fx_played(preset_id: String, entry: ActionLogEntry) -> void:
	var key: String = SoundEvents.key_for(preset_id, entry)
	if not key.is_empty():
		play_sfx(key)

# Sets the four bus volumes from SettingsFile (linear 0..1; 0 mutes the bus).
func apply_volumes() -> void:
	for bus: String in BUSES:
		var idx: int = AudioServer.get_bus_index(bus)
		if idx < 0:
			continue
		var linear: float = SettingsFile.get_float(str(BUSES[bus]))
		AudioServer.set_bus_mute(idx, linear <= 0.0)
		AudioServer.set_bus_volume_db(idx, linear_to_db(maxf(linear, 0.0001)))

# The first idle voice plays the stream; when all are busy the one that started earliest restarts with it.
func _play(voices: Array[AudioStreamPlayer], key: String) -> void:
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
