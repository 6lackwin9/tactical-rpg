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

static var active: AudioDirector = null

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
	active = self

func _exit_tree() -> void:
	if active == self:
		active = null

func _ready() -> void:
	if not enabled():
		return
	for i: int in SFX_VOICES:
		_sfx.append(_voice("SFX"))
	for i: int in UI_VOICES:
		_ui.append(_voice("UI"))

func _voice(bus: String) -> AudioStreamPlayer:
	var p := AudioStreamPlayer.new()
	p.bus = bus
	add_child(p)
	return p

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
