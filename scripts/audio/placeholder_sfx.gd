class_name PlaceholderSfx
extends RefCounted

# ==============================================================================
# Tactical RPG - Placeholder sound blips (S14-L3 rule 5, used by tools/gen_placeholder_sfx.gd)
# Short mono 16-bit 22050 Hz sounds: wave sine | square | noise | noise_sine, a
# linear frequency sweep hz_from -> hz_to, envelope decay (fast) | fade (in/out) |
# slow (slow decay), gain in dB. Noise uses a fixed seed so files are reproducible.
# ==============================================================================

const MIX_RATE: int = 22050
const WAVES: Array[String] = ["sine", "square", "noise", "noise_sine"]
const ENVELOPES: Array[String] = ["decay", "fade", "slow"]

static func make(wave: String, hz_from: float, hz_to: float, ms: int, envelope: String, gain_db: float) -> AudioStreamWAV:
	var count: int = maxi(1, MIX_RATE * ms / 1000)
	var gain: float = db_to_linear(gain_db) * 0.8
	var rng := RandomNumberGenerator.new()
	rng.seed = 1414
	var data := PackedByteArray()
	data.resize(count * 2)
	var phase: float = 0.0
	for i: int in count:
		var t: float = float(i) / float(count)
		phase += TAU * lerpf(hz_from, hz_to, t) / float(MIX_RATE)
		var tone: float = sin(phase)
		var v: float = 0.0
		match wave:
			"square":
				v = 1.0 if tone >= 0.0 else -1.0
			"noise":
				v = rng.randf_range(-1.0, 1.0)
			"noise_sine":
				v = 0.5 * rng.randf_range(-1.0, 1.0) + 0.5 * tone
			_:
				v = tone
		var sample: int = clampi(roundi(v * _envelope(envelope, t) * gain * 32767.0), -32768, 32767)
		data.encode_s16(i * 2, sample)
	var wav := AudioStreamWAV.new()
	wav.format = AudioStreamWAV.FORMAT_16_BITS
	wav.mix_rate = MIX_RATE
	wav.stereo = false
	wav.data = data
	return wav

static func _envelope(kind: String, t: float) -> float:
	match kind:
		"fade":
			return minf(1.0, minf(t / 0.2, (1.0 - t) / 0.2))
		"slow":
			return pow(1.0 - t, 1.5)
		_:
			return pow(1.0 - t, 4.0)

# Writes the blip for `key` to `path` (default res://assets/audio/sfx/<key>.wav). Returns "" or an error text.
static func write(path: String, wave: String, hz_from: float, hz_to: float, ms: int, envelope: String, gain_db: float) -> String:
	if not WAVES.has(wave):
		return "unknown wave '%s' (%s)" % [wave, ", ".join(WAVES)]
	if not ENVELOPES.has(envelope):
		return "unknown envelope '%s' (%s)" % [envelope, ", ".join(ENVELOPES)]
	if ms <= 0:
		return "length must be > 0 ms"
	DirAccess.make_dir_recursive_absolute(ProjectSettings.globalize_path(path.get_base_dir()))
	var err: Error = make(wave, hz_from, hz_to, ms, envelope, gain_db).save_to_wav(path)
	return "" if err == OK else "could not write %s (error %d)" % [path, err]
