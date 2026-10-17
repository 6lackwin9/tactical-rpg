class_name SceneRunner
extends Node

# ==============================================================================
# Tactical RPG - Scene Runner (main menu & developer launcher)
# The project's main scene. Lists every runnable scene (known ones with a
# description, plus any other .tscn found under res://scenes), launches one
# inside the runner, and brings you back with F10 or the ☰ Menu pill.
#
#   F10  back to the menu          F6  restart the running scene
#   F5   reload edited scripts and restart the running scene
#
# Developer tools: hot-reload scripts edited on disk (ScriptReloader), restart
# the whole runner process (for new files / class_names), and run the headless
# test suites in a background process with their PASS / FAIL output shown here.
# Playtests (PLAYTEST1, debug builds): the PlaytestHub child records the run while the
# `record_playtests` pref is on; the PLAYTESTS column lists recordings (Open, Notes, Take over,
# Copy to repo) and scenarios (each launched with its seed).
# ==============================================================================

const SELF_SCENE: String = "res://scenes/scene_runner.tscn"
const PREFS_PATH: String = "user://scene_runner.cfg"
const ARENA_SCENE: String = "res://scenes/combat_arena.tscn"
const RIGHT_WIDTH: int = 470
const PLAYTESTS_STRETCH: float = 1.6   # share of the right column's height (DEVELOPER = 1)

# Known scenes: shown first, with descriptions. Anything else under res://scenes
# that isn't hidden is listed under "Other scenes".
const SCENES: Array[Dictionary] = [
	{"path": "res://scenes/main_menu.tscn", "name": "Main Menu", "icon": "▶",
		"desc": "The player's front door: continue, new game in a playable world, load and save slots (0 = autosave, 1-5 manual), settings and the in-game hub."},
	{"path": "res://scenes/combat_arena.tscn", "name": "Combat Arena", "icon": "⚔",
		"desc": "Plan and fight a battle. Pick a catalog encounter or any world's battle (played with that world's characters).", "encounters": true},
	{"path": "res://scenes/content_forge.tscn", "name": "Content Forge", "icon": "⚒",
		"desc": "Read-only asset viewer: browse skills, items, classes, units and encounters, their engine preview and problems; playtest them in the arena."},
	{"path": "res://scenes/world_forge.tscn", "name": "World Forge", "icon": "🌍",
		"desc": "Read-only world viewer: regions, factions, characters, named items, quests and encounters that inherit lore-free base assets; play their battles."},
	{"path": "res://scenes/gear_dojo.tscn", "name": "Gear & Skill Dojo", "icon": "🛡",
		"desc": "Equip heroes, allocate stats and benchmark damage against a training dummy."},
	{"path": "res://scenes/overworld/region_map_debug.tscn", "name": "Region Map (debug)", "icon": "🗺",
		"desc": "Walk a world's region maps: step-counter fights, markers, chests and exits, with Won / Fled / Lost buttons instead of real battles (a game started from the main menu autosaves to slot 0; the standalone debug game writes nothing)."},
]
const HIDDEN_SCENES: Array[String] = ["res://scenes/scene_runner.tscn", "res://scenes/test_arena.tscn"]

# Headless suites run as a separate Godot process (never inside the running game).
const SUITES: Array[Dictionary] = [
	{"name": "Arena suite", "scene": "res://scenes/test_arena.tscn", "desc": "Combat, planning, AI, content, runner"},
	{"name": "Dojo suite", "scene": "res://scenes/gear_dojo.tscn", "desc": "Gear, stats and damage formula"},
]

var reloader: ScriptReloader = null
var menu_layer: CanvasLayer = null
var pill_layer: CanvasLayer = null
var active_scene: Node = null
var active_path: String = ""
var active_encounter: String = ""   # catalog encounter id for the arena ("" = its default)
var active_opts: Dictionary = {}    # fields set on the scene before it enters the tree (playtests)
var hub: PlaytestHub = null         # null when playtests are off (release builds, test runs)
var browser: PlaytestBrowser = null  # null when playtests are off (release builds)
var record_playtests: bool = true   # pref `record_playtests`

var _encounter_pick: OptionButton = null
var _status: Label = null
var _pending_label: Label = null
var _output: RichTextLabel = null
var _suite_thread: Thread = null
var _leave_armed: bool = false
var _pill_hint: Label = null

func _ready() -> void:
	name = "SceneRunner"
	WebFonts.install()
	reloader = ScriptReloader.new()
	var audio := AudioDirector.new()   # S14-L3: one director per main scene
	add_child(audio)
	SettingsModel.push_to_listeners()   # saved animation speed/instant and volumes apply at boot
	if PlaytestStore.enabled():   # release builds (and test runs) get no hub at all
		hub = PlaytestHub.new()
		add_child(hub)
		hub.take_over_requested.connect(func(path: String, battle: int, round_num: int) -> void:
			launch_playtest({"playtest": path, "battle": battle, "take_over": round_num}))
	_build_menu()
	_build_pill()
	_load_prefs()
	_show_menu()

func _exit_tree() -> void:
	if _suite_thread != null and _suite_thread.is_started():
		_suite_thread.wait_to_finish()
	SkillCatalog.clear_cache()
	ContentCatalog.clear_cache()
	WorldCatalog.clear_cache()

# ------------------------------------------------------------------------------
# Scene list
# ------------------------------------------------------------------------------

# Every runnable scene: the known ones (if present) then any other .tscn under res://scenes.
static func runnable_scenes() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var known: Dictionary = {}
	for s: Dictionary in SCENES:
		known[str(s["path"])] = true
		if ResourceLoader.exists(str(s["path"])):
			out.append(s)
	var dir: DirAccess = DirAccess.open("res://scenes")
	if dir != null:
		for file: String in dir.get_files():
			var path: String = "res://scenes/" + file
			if file.ends_with(".tscn") and not known.has(path) and not HIDDEN_SCENES.has(path):
				out.append({"path": path, "name": file.get_basename().capitalize(), "icon": "▶", "desc": path, "other": true})
	return out

# ------------------------------------------------------------------------------
# Launching
# ------------------------------------------------------------------------------

# Runs `path` inside the runner (replacing any running scene). Returns false if it can't load.
# `opts` = properties set on the scene before it enters the tree (the arena's playtest fields).
func launch(path: String, encounter_id: String = "", opts: Dictionary = {}) -> bool:
	_close_active()
	var packed: PackedScene = ResourceLoader.load(path, "PackedScene", ResourceLoader.CACHE_MODE_REPLACE) as PackedScene
	if packed == null:
		_set_status("Couldn't load %s." % path, true)
		return false
	var scene: Node = packed.instantiate()
	if not encounter_id.is_empty() and EncounterSource.exists(encounter_id) and "encounter" in scene:
		scene.set("encounter", EncounterSource.create(encounter_id))
	for key: Variant in opts:
		if str(key) in scene:
			scene.set(str(key), opts[key])
	active_scene = scene
	active_path = path
	active_encounter = encounter_id
	active_opts = opts.duplicate()
	if hub != null:
		hub.scene_changed(path, scene)   # before the scene enters the tree: its battle is followed from start()
	add_child(scene)
	menu_layer.visible = false
	pill_layer.visible = true
	_leave_armed = false
	_save_prefs()
	return true

func restart_scene() -> bool:
	if active_path.is_empty():
		return false
	return launch(active_path, active_encounter, active_opts)

# The arena with playtest fields: {playtest, battle, take_over} (playback / take-over) or
# {battle_seed, seed_source, scenario} (a scenario, see launch_scenario).
func launch_playtest(opts: Dictionary, encounter_id: String = "") -> bool:
	return launch(ARENA_SCENE, encounter_id, opts)

# A scenario (PLAYTEST1 H): its encounter with its seed, recorded with seed_source "scenario".
func launch_scenario(scenario: Dictionary) -> bool:
	return launch_playtest({"battle_seed": DictUtil.to_int(scenario.get("seed", 0)), "seed_source": "scenario", "scenario": str(scenario.get("id", ""))},
		str(scenario.get("encounter", "")))

# Recompiles scripts edited on disk; restarts the running scene so it uses them.
func reload_and_restart() -> Dictionary:
	var path: String = active_path
	var encounter_id: String = active_encounter
	var opts: Dictionary = active_opts
	if not path.is_empty():
		_close_active()
	var result: Dictionary = reload_scripts()
	if not path.is_empty():
		launch(path, encounter_id, opts)
	return result

func reload_scripts(only: Array[String] = []) -> Dictionary:
	var own: Script = get_script()
	var skip: Array[String] = [own.resource_path]   # never recompile the runner while it runs
	var result: Dictionary = reloader.reload_changed(only, skip)
	if not ContentCatalog.is_dirty():
		ContentCatalog.reload_from_disk()   # pick up catalog JSON edited by seed scripts
	var reloaded: Array = result["reloaded"]
	var failed: Array = result["failed"]
	var added: Array = result["new"]
	var parts: Array[String] = ["Reloaded %d script%s" % [reloaded.size(), "" if reloaded.size() == 1 else "s"]]
	if not failed.is_empty():
		parts.append("%d failed (see output)" % failed.size())
	if not added.is_empty():
		parts.append("%d new file%s need a restart" % [added.size(), "" if added.size() == 1 else "s"])
	_set_status(", ".join(parts) + ".", not failed.is_empty())
	var lines: Array[String] = []
	for p: Variant in reloaded:
		lines.append("[color=#4ade80]reloaded[/color] %s" % str(p))
	for p: Variant in failed:
		lines.append("[color=#f87171]failed[/color] %s" % str(p))
	for p: Variant in added:
		lines.append("[color=#fbbf24]new (restart to load)[/color] %s" % str(p))
	if _output != null and not lines.is_empty():
		_output.text = "\n".join(lines)
	_refresh_pending()
	return result

# Starts a fresh runner process (picks up new files and class_names), then quits this one.
func restart_runner() -> void:
	var args: PackedStringArray = ["--path", ProjectSettings.globalize_path("res://"), SELF_SCENE]
	OS.create_process(OS.get_executable_path(), args)
	get_tree().quit()

# Back to the menu. A scene with unsaved work (has_unsaved_changes()) needs a second press.
func back_to_menu() -> bool:
	if active_scene != null and active_scene.has_method("has_unsaved_changes") and DictUtil.to_bool(active_scene.call("has_unsaved_changes")) and not _leave_armed:
		_leave_armed = true
		_set_status("Unsaved changes in %s: press Menu (F10) again to discard them." % _scene_name(active_path), true)
		_flash_pill("Unsaved changes: press again to leave")
		return false
	_close_active()
	if hub != null:
		hub.scene_changed(SELF_SCENE, null)
	_show_menu()
	return true

func is_running() -> bool:
	return active_scene != null

func _close_active() -> void:
	if active_scene != null:
		remove_child(active_scene)
		active_scene.queue_free()
		active_scene = null

func _show_menu() -> void:
	menu_layer.visible = true
	pill_layer.visible = false
	_refresh_pending()
	if browser != null:
		browser.refresh()

func _unhandled_key_input(event: InputEvent) -> void:
	var key: InputEventKey = event as InputEventKey
	if key == null or not key.pressed or key.echo:
		return
	match key.keycode:
		KEY_F10:
			if is_running():
				back_to_menu()
				get_viewport().set_input_as_handled()
		KEY_F6:
			if is_running():
				restart_scene()
				get_viewport().set_input_as_handled()
		KEY_F5:
			reload_and_restart()
			get_viewport().set_input_as_handled()

# ------------------------------------------------------------------------------
# Test suites (separate headless process, output shown in the menu)
# ------------------------------------------------------------------------------

func run_suite(scene_path: String) -> bool:
	if _suite_thread != null and _suite_thread.is_started():
		_set_status("A suite is already running.", true)
		return false
	_output.text = "[color=#94a3b8]Running %s headless... (the menu stays usable)[/color]" % scene_path
	var args: PackedStringArray = ["--headless", "--path", ProjectSettings.globalize_path("res://"), scene_path]
	_suite_thread = Thread.new()
	_suite_thread.start(_run_process.bind(scene_path, args))
	return true

func _run_process(scene_path: String, args: PackedStringArray) -> void:
	var out: Array = []
	var code: int = OS.execute(OS.get_executable_path(), args, out, true)
	_suite_done.call_deferred(scene_path, code, "\n".join(PackedStringArray(out)))

func _suite_done(scene_path: String, code: int, text: String) -> void:
	_suite_thread.wait_to_finish()
	_suite_thread = null
	_output.text = format_suite_output(text)
	var counts: Vector2i = count_results(text)
	var ok: bool = code == 0 and counts.y == 0
	_set_status("%s: %s (%d passed, %d failed, exit %d)." % [scene_path.get_file(), "PASSED" if ok else "FAILED", counts.x, counts.y, code], not ok)

# (passed, failed) assertion lines in a suite's output.
static func count_results(text: String) -> Vector2i:
	var passed: int = 0
	var failed: int = 0
	for line: String in text.split("\n"):
		if line.contains(": PASS"):
			passed += 1
		elif line.contains(": FAIL") or line.contains("SCRIPT ERROR"):
			failed += 1
	return Vector2i(passed, failed)

static func format_suite_output(text: String) -> String:
	var lines: Array[String] = []
	for raw: String in text.split("\n"):
		var line: String = raw.strip_edges(false, true).replace("[", "[lb]")
		if raw.contains(": PASS"):
			lines.append("[color=#4ade80]%s[/color]" % line)
		elif raw.contains(": FAIL") or raw.contains("SCRIPT ERROR"):
			lines.append("[color=#f87171]%s[/color]" % line)
		elif raw.contains("==="):
			lines.append("[color=#7dd3fc]%s[/color]" % line)
	return "\n".join(lines) if not lines.is_empty() else text

# ------------------------------------------------------------------------------
# Menu UI
# ------------------------------------------------------------------------------

func _build_menu() -> void:
	menu_layer = CanvasLayer.new()
	menu_layer.layer = 10
	add_child(menu_layer)
	var bg := Panel.new()
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	bg.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_APP_BG, UiTheme.COL_APP_BG, 0))
	menu_layer.add_child(bg)
	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.offset_left = 24
	root.offset_top = 18
	root.offset_right = -24
	root.offset_bottom = -14
	root.add_theme_constant_override("separation", 10)
	menu_layer.add_child(root)
	root.add_child(UiTheme.label("⚔ TACTICAL RPG  ·  SCENE RUNNER", 20, UiTheme.COL_TITLE))
	root.add_child(UiTheme.label("Launch any scene; F10 returns here, F6 restarts the scene, F5 reloads edited scripts and restarts it.", UiTheme.FONT_MD, UiTheme.COL_MUTED))

	var body := HBoxContainer.new()
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation", 14)
	root.add_child(body)

	var scenes_col: VBoxContainer = _panel(body, "SCENES", 0)
	for s: Dictionary in runnable_scenes():
		_scene_card(scenes_col, s)

	var right := VBoxContainer.new()   # DEVELOPER on top, PLAYTESTS below (debug builds)
	right.custom_minimum_size.x = RIGHT_WIDTH
	right.add_theme_constant_override("separation", 10)
	body.add_child(right)
	var dev_col: VBoxContainer = _panel(right, "DEVELOPER", 0)
	_expand_panel(dev_col, 1.0)
	var reload_row := HBoxContainer.new()
	dev_col.add_child(reload_row)
	reload_row.add_child(UiTheme.button("⟳ Reload scripts", "Recompile scripts edited on disk (F5 also restarts the running scene)", func() -> void: reload_scripts(), UiTheme.FONT_MD))
	reload_row.add_child(UiTheme.button("⏻ Restart runner", "Start a fresh runner process: needed for new files / class_names", func() -> void: restart_runner(), UiTheme.FONT_MD))
	_pending_label = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	_pending_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	dev_col.add_child(_pending_label)
	dev_col.add_child(HSeparator.new())
	dev_col.add_child(UiTheme.header("HEADLESS TEST SUITES (separate process)"))
	var suite_row := HBoxContainer.new()
	dev_col.add_child(suite_row)
	for suite: Dictionary in SUITES:
		var scene_path: String = str(suite["scene"])
		suite_row.add_child(UiTheme.button("▶ " + str(suite["name"]), str(suite["desc"]), func() -> void: run_suite(scene_path), UiTheme.FONT_MD))
	_output = RichTextLabel.new()
	_output.bbcode_enabled = true
	_output.scroll_following = true
	_output.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_output.add_theme_font_size_override("normal_font_size", UiTheme.FONT_SM)
	_output.text = "[color=#94a3b8]Suite output and reload results appear here.[/color]"
	dev_col.add_child(_output)

	if PlaytestStore.enabled():
		var play_col: VBoxContainer = _panel(right, "PLAYTESTS", 0)
		_expand_panel(play_col, PLAYTESTS_STRETCH)
		browser = PlaytestBrowser.new()
		play_col.add_child(browser)
		browser.record_toggled.connect(set_record_playtests)
		browser.open_requested.connect(func(path: String, battle: int) -> void: launch_playtest({"playtest": path, "battle": battle}))
		browser.take_over_requested.connect(func(path: String, battle: int, round_num: int) -> void:
			launch_playtest({"playtest": path, "battle": battle, "take_over": round_num}))
		browser.scenario_requested.connect(func(sc: Dictionary) -> void: launch_scenario(sc))

	_status = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_MUTED)
	root.add_child(_status)

func _panel(parent: Control, title: String, width: int) -> VBoxContainer:
	var panel := PanelContainer.new()
	panel.add_theme_stylebox_override("panel", UiTheme.box(UiTheme.COL_PANEL_BG, UiTheme.COL_BORDER, 6, 12, 10))
	if width > 0:
		panel.custom_minimum_size.x = width
	else:
		panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(panel)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 8)
	panel.add_child(col)
	col.add_child(UiTheme.header(title))
	return col

# Lets the panel around `col` (from _panel) grow vertically with weight `ratio`.
static func _expand_panel(col: VBoxContainer, ratio: float) -> void:
	var panel: Control = col.get_parent() as Control
	panel.size_flags_vertical = Control.SIZE_EXPAND_FILL
	panel.size_flags_stretch_ratio = ratio

func _scene_card(parent: Control, s: Dictionary) -> void:
	var card: VBoxContainer = UiTheme.card(parent)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	card.add_child(row)
	var info := VBoxContainer.new()
	info.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(info)
	info.add_child(UiTheme.label("%s  %s" % [str(s.get("icon", "▶")), str(s["name"])], UiTheme.FONT_LG, UiTheme.COL_HEADER))
	var desc: Label = UiTheme.label(str(s.get("desc", "")), UiTheme.FONT_SM, UiTheme.COL_MUTED)
	desc.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	info.add_child(desc)
	var path: String = str(s["path"])
	if DictUtil.to_bool(s.get("encounters", false)):
		_encounter_pick = OptionButton.new()
		_encounter_pick.tooltip_text = "Any encounter: lore-free catalog battles, or a world's battle played with that world's characters"
		for opt: Dictionary in EncounterSource.options():
			_encounter_pick.add_item(str(opt["label"]))
			_encounter_pick.set_item_metadata(_encounter_pick.item_count - 1, str(opt["id"]))
		info.add_child(_encounter_pick)
	var go: Button = UiTheme.button("▶ Launch", "Run %s" % path, func() -> void: launch(path, selected_encounter() if DictUtil.to_bool(s.get("encounters", false)) else ""), UiTheme.FONT_MD)
	go.custom_minimum_size = Vector2(96, 34)
	go.size_flags_vertical = Control.SIZE_SHRINK_CENTER
	row.add_child(go)

func selected_encounter() -> String:
	if _encounter_pick == null or _encounter_pick.selected < 0:
		return ""
	return str(_encounter_pick.get_item_metadata(_encounter_pick.selected))

func select_encounter(id: String) -> void:
	if _encounter_pick == null:
		return
	for i: int in range(_encounter_pick.item_count):
		if str(_encounter_pick.get_item_metadata(i)) == id:
			_encounter_pick.select(i)

# The in-scene pill (bottom-right): back to menu / restart / reload.
func _build_pill() -> void:
	pill_layer = CanvasLayer.new()
	pill_layer.layer = 100
	add_child(pill_layer)
	var pill := PanelContainer.new()
	pill.add_theme_stylebox_override("panel", UiTheme.box(Color(0.05, 0.08, 0.12, 0.88), UiTheme.COL_BORDER, 10, 6, 2))
	pill.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	pill.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	pill.grow_vertical = Control.GROW_DIRECTION_BEGIN
	pill.offset_right = -6
	pill.offset_bottom = -2
	pill_layer.add_child(pill)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 2)
	pill.add_child(row)
	row.add_child(UiTheme.button("☰ Menu", "Back to the Scene Runner (F10)", func() -> void: back_to_menu()))
	row.add_child(UiTheme.button("↻", "Restart this scene (F6)", func() -> void: restart_scene()))
	row.add_child(UiTheme.button("⟳", "Reload edited scripts and restart (F5)", func() -> void: reload_and_restart()))
	_pill_hint = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_WARN)
	row.add_child(_pill_hint)

func _flash_pill(text: String) -> void:
	if _pill_hint != null:
		_pill_hint.text = text

func _refresh_pending() -> void:
	if _pending_label == null or reloader == null:
		return
	var found: Dictionary = reloader.pending()
	var changed: Array = found["changed"]
	var added: Array = found["new"]
	if changed.is_empty() and added.is_empty():
		_pending_label.text = "Scripts are up to date with the files on disk."
	else:
		_pending_label.text = "%d edited script%s ready to reload%s." % [changed.size(), "" if changed.size() == 1 else "s", (", %d new file%s (restart)" % [added.size(), "" if added.size() == 1 else "s"]) if not added.is_empty() else ""]
	if _pill_hint != null:
		_pill_hint.text = ""

func _set_status(text: String, bad: bool = false) -> void:
	if _status != null:
		_status.text = text
		_status.add_theme_color_override("font_color", UiTheme.COL_BAD if bad else UiTheme.COL_MUTED)

func status_text() -> String:
	return _status.text if _status != null else ""

func _scene_name(path: String) -> String:
	for s: Dictionary in SCENES:
		if str(s["path"]) == path:
			return str(s["name"])
	return path.get_file()

# ------------------------------------------------------------------------------
# Preferences (last encounter picked, record_playtests)
# ------------------------------------------------------------------------------

func _load_prefs() -> void:
	var cfg := ConfigFile.new()
	if cfg.load(PREFS_PATH) == OK:
		select_encounter(str(cfg.get_value("runner", "encounter", "")))
		record_playtests = DictUtil.to_bool(cfg.get_value("runner", "record_playtests", true), true)
	if hub != null:
		hub.set_record_enabled(record_playtests)
	if browser != null:
		browser.set_record_enabled(record_playtests)

func set_record_playtests(on: bool) -> void:
	record_playtests = on
	if hub != null:
		hub.set_record_enabled(on)
	if browser != null:
		browser.set_record_enabled(on)
	_save_prefs()

func _save_prefs() -> void:
	var cfg := ConfigFile.new()
	cfg.load(PREFS_PATH)   # keep the other keys (a missing file is fine)
	if not active_encounter.is_empty():
		cfg.set_value("runner", "encounter", active_encounter)
	cfg.set_value("runner", "record_playtests", record_playtests)
	cfg.save(PREFS_PATH)
