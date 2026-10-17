class_name AudioTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Audio Test Suite (S14, spec S14-L3)
# Bus layout, SoundBank silent fallback and warn-once, the headless AudioDirector
# (no voices), SettingsFile audio keys (defaults, clamp, round trip, untouched by save
# slot deletes), a sandbox manifest row resolving to a generated wav, and the
# preset -> sound key table. Fixtures: sandbox settings file, save dir and manifest.
# ==============================================================================

const TAG: String = "Audio"
static var SANDBOX: String = TestHelpers.sandbox("audio")

static func _report(label: String, checks: Dictionary) -> bool:
	return TestHelpers.report(TAG, label, checks)

static func run(tree_parent: Node = null) -> bool:
	print("\n=======================================================")
	print("=== STARTING AUDIO TEST (S14-L3)                     ===")
	print("=======================================================")
	TestHelpers.remove_sandbox(SANDBOX)
	DirAccess.make_dir_recursive_absolute(SANDBOX + "/assets")
	var ok: bool = true
	ok = _test_buses() and ok
	ok = _test_bank() and ok
	ok = _test_director(tree_parent) and ok
	ok = _test_settings() and ok
	ok = _test_manifest_stream() and ok
	ok = _test_event_table() and ok
	SettingsFile.use_path()
	SaveGame.reset_base_dir()
	AssetCatalog.use_path(AssetCatalog.DEFAULT_PATH)
	AssetRegistry.clear_warnings()
	SoundBank.clear_warned()
	SoundBank.set_quiet(false)
	TestHelpers.remove_sandbox(SANDBOX)
	print("  [Audio] Audio suite: %s" % ("PASS" if ok else "FAIL"))
	return ok

# Acceptance 1
static func _test_buses() -> bool:
	var checks: Dictionary = {"Master": AudioServer.get_bus_index("Master") == 0}
	for bus: String in ["Music", "SFX", "UI"]:
		var idx: int = AudioServer.get_bus_index(bus)
		checks[bus] = idx > 0 and AudioServer.get_bus_send(idx) == &"Master"
	return _report("Bus layout: Master, Music, SFX, UI send to Master", checks)

# Acceptance 2
static func _test_bank() -> bool:
	SoundBank.clear_warned()
	SoundBank.set_quiet(true)
	var first: AudioStream = SoundBank.stream("no_such_key")
	var second: AudioStream = SoundBank.stream("no_such_key")
	var checks: Dictionary = {
		"unknown key is silent": first == null and second == null and not SoundBank.has("no_such_key"),
		"warns once": SoundBank.warning_count() == 1,
	}
	SoundBank.clear_warned()
	checks["clear_warned resets"] = SoundBank.warning_count() == 0
	SoundBank.stream("no_such_key")
	checks["warns again after clear"] = SoundBank.warning_count() == 1
	SoundBank.clear_warned()
	return _report("SoundBank: missing key is null and warns once", checks)

# Acceptance 3
static func _test_director(tree_parent: Node) -> bool:
	var baseline: AudioDirector = AudioDirector.active   # the host scene may own a director already
	var director := AudioDirector.new()
	if tree_parent != null:
		tree_parent.add_child(director)
	director.play_sfx("hit")
	director.ui("click")
	AudioDirector.ui_event("click")
	var presenter := CombatPresenter.new()
	director.add_child(presenter)
	director.connect_presenter(presenter)
	presenter.fx_played.emit("hit", ActionLogEntry.new())
	var checks: Dictionary = {
		"disabled headless": not AudioDirector.enabled(),
		"no voices": director.voice_count() == 0,
		"presenter connected": presenter.fx_played.get_connections().size() == 1,
		"nothing looked up": SoundBank.warning_count() == 0,
		"active while in tree": tree_parent == null or AudioDirector.active == director,
	}
	if director.is_inside_tree():
		tree_parent.remove_child(director)
	checks["active cleared on exit"] = tree_parent == null or AudioDirector.active == baseline
	director.free()
	if tree_parent != null:
		var a := AudioDirector.new()
		var b := AudioDirector.new()
		tree_parent.add_child(a)
		tree_parent.add_child(b)
		checks["newest is active"] = AudioDirector.active == b
		tree_parent.remove_child(b)
		checks["older director active again after the newer exits"] = AudioDirector.active == a
		tree_parent.remove_child(a)
		checks["none of ours left gives the baseline"] = AudioDirector.active == baseline
		a.free()
		b.free()
	return _report("AudioDirector headless: no voices, play is a no-op", checks)

# Acceptance 4
static func _test_settings() -> bool:
	var cfg_path: String = SANDBOX + "/settings.cfg"
	SettingsFile.use_path(cfg_path)
	var checks: Dictionary = {}
	checks["defaults when missing"] = (
		is_equal_approx(SettingsFile.get_float("audio/master"), 1.0) and is_equal_approx(SettingsFile.get_float("audio/music"), 0.8)
		and is_equal_approx(SettingsFile.get_float("audio/sfx"), 0.8) and is_equal_approx(SettingsFile.get_float("audio/ui"), 0.8)
	)
	SettingsFile.set_value("audio/sfx", 1.7)
	checks["clamps to 1.0"] = is_equal_approx(SettingsFile.get_float("audio/sfx"), 1.0)
	SettingsFile.set_value("audio/music", -2)
	checks["clamps to 0.0"] = is_equal_approx(SettingsFile.get_float("audio/music"), 0.0)
	SettingsFile.set_value("audio/ui", 0.35)
	checks["saved"] = SettingsFile.save() == OK and FileAccess.file_exists(cfg_path)
	SettingsFile.use_path(cfg_path)   # drops the in-memory file
	SettingsFile.load()
	checks["round trip"] = is_equal_approx(SettingsFile.get_float("audio/ui"), 0.35) and is_equal_approx(SettingsFile.get_float("audio/sfx"), 1.0)
	var bad := ConfigFile.new()
	bad.set_value("audio", "master", "loud")
	bad.save(cfg_path)
	SettingsFile.use_path(cfg_path)
	checks["wrong type gives the default"] = is_equal_approx(SettingsFile.get_float("audio/master"), 1.0)
	SettingsFile.set_value("audio/ui", 0.35)
	SettingsFile.save()
	var before: String = FileAccess.get_file_as_string(cfg_path)
	SaveGame.set_base_dir(SANDBOX + "/saves")
	var slot_file: String = SaveGame.path_for(1)
	DirAccess.make_dir_recursive_absolute(slot_file.get_base_dir())
	var f := FileAccess.open(slot_file, FileAccess.WRITE)
	if f != null:
		f.store_string("{}")
		f.close()
	SaveGame.delete(1)
	checks["save slot delete leaves settings"] = FileAccess.get_file_as_string(cfg_path) == before and not FileAccess.file_exists(slot_file)
	SaveGame.reset_base_dir()
	SettingsFile.use_path()
	return _report("SettingsFile audio keys: defaults, clamp, round trip", checks)

# Acceptance 5
static func _test_manifest_stream() -> bool:
	AssetCatalog.use_path(SANDBOX + "/manifest.json", SANDBOX + "/assets")
	var row_path: String = "%s/hit.wav" % AssetRegistry.folder_of("sfx")
	var write_err: String = PlaceholderSfx.write(AssetCatalog.file_of(row_path), "noise", 0.0, 0.0, 80, "decay", 0.0)
	var errs: Array[String] = AssetCatalog.upsert({"namespace": "sfx", "key": "hit", "variant": "stream", "path": row_path, "alias_of": ""})
	var s: AudioStream = SoundBank.stream("hit")
	var wav: AudioStreamWAV = s as AudioStreamWAV
	var checks: Dictionary = {
		"tool wrote the file": write_err.is_empty(),
		"row stored": errs.is_empty(),
		"stream is a wav": wav != null,
		"80 ms long": wav != null and absf(wav.get_length() - 0.08) < 0.005,
		"bad wave refused": not PlaceholderSfx.write(AssetCatalog.file_of(row_path), "saw", 1.0, 1.0, 10, "decay", 0.0).is_empty(),
	}
	AssetCatalog.use_path(AssetCatalog.DEFAULT_PATH)
	return _report("Manifest sfx row resolves to the generated wav", checks)

# Acceptance 6
static func _test_event_table() -> bool:
	var checks: Dictionary = {}
	for id: Variant in FxPresets.PRESETS:
		checks["preset %s mapped" % id] = SoundEvents.FOR_PRESET.has(id)
	for id: Variant in SoundEvents.FOR_PRESET:
		var key: String = str(SoundEvents.FOR_PRESET[id])
		if not key.is_empty():
			checks["%s -> %s is an event key" % [id, key]] = SoundEvents.KEYS.has(key)
	var crit := ActionLogEntry.new(1, 1, "ATTACK", "A", "B", "", 9, "CRIT")
	checks["crit replaces hit"] = SoundEvents.key_for("hit", crit) == "crit"
	var stun := ActionLogEntry.new(1, 1, "ATTACK", "A", "B", "", 0, StatusEffects.display_name("stun").to_upper())
	var haste := ActionLogEntry.new(1, 1, "BUFF", "A", "B", "", 0, StatusEffects.display_name("haste").to_upper())
	checks["debuff status plays debuff"] = SoundEvents.key_for("status", stun) == "debuff"
	checks["buff status is silent"] = SoundEvents.key_for("status", haste).is_empty()
	return _report("Preset -> sound key table covers every preset", checks)
