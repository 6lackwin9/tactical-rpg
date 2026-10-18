---
class: SceneRunner
extends: Node
description: "Tactical RPG - Scene Runner (main menu & developer launcher) The project's main scene. Lists every runnable scene (known ones with a description, plus any other .tscn found under res://scenes), launches one inside the runner, and brings you back with F10 or the ☰ Menu pill. F10  back to the menu          F6  restart the running scene F5   reload edited scripts and restart the running scene Developer tools: hot-reload scripts edited on disk (ScriptReloader), restart the whole runner process (for new files / class_names), and run the headless test suites in a background process with their PASS / FAIL output shown here. Restart runner on Windows with run.bat + git runs run.bat (fetch + fast-forward, then relaunch) and reports old -> new commit. Playtests (PLAYTEST1, debug builds): the PlaytestHub child records the run while the `record_playtests` pref is on; the PLAYTESTS column lists recordings (Open, Notes, Take over, Copy to repo) and scenarios (each launched with its seed)."
---

# Properties
* `SELF_SCENE`: String = "res://scenes/scene_runner.tscn" (const)
* `PREFS_PATH`: String = "user://scene_runner.cfg" (const)
* `RESTART_TIP`: String = "Pull the latest code for this folder (backs up saves), then start a fresh runner" (const)
* `PULL_PATH`: String = "user://pull_restart.cfg" (const)
* `ARENA_SCENE`: String = "res://scenes/combat_arena.tscn" (const)
* `RIGHT_WIDTH`: int = 470 (const)
* `PLAYTESTS_STRETCH`: float = 1.6   # share of the right column's height (DEVELOPER = 1) (const)
* `SCENES`: Array[Dictionary] = [ (const)
* `HIDDEN_SCENES`: Array[String] = ["res://scenes/scene_runner.tscn", "res://scenes/test_arena.tscn"] (const)
* `SUITES`: Array[Dictionary] = [ (const)
* `reloader`: ScriptReloader = null
* `menu_layer`: CanvasLayer = null
* `pill_layer`: CanvasLayer = null
* `active_scene`: Node = null
* `active_path`: String = ""
* `active_encounter`: String = ""   # catalog encounter id for the arena ("" = its default)
* `active_opts`: Dictionary = {}    # fields set on the scene before it enters the tree (playtests)
* `hub`: PlaytestHub = null         # null when playtests are off (release builds, test runs)
* `browser`: PlaytestBrowser = null  # null when playtests are off (release builds)
* `record_playtests`: bool = true   # pref `record_playtests`
* `_encounter_pick`: OptionButton = null
* `_status`: Label = null
* `_pending_label`: Label = null
* `_output`: RichTextLabel = null
* `_suite_thread`: Thread = null
* `_leave_armed`: bool = false
* `_pill_hint`: Label = null
* `note_button`: Button = null      # "✎ Note" in the pill, shown only while the hub is recording
* `out`: Array[Dictionary] = []
* `known`: Dictionary = {}
* `dir`: DirAccess = DirAccess.open("res://scenes")
* `path`: String = "res://scenes/" + file

# Methods

### `_ready() -> void`
```logic
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
_report_pull()
```

### `_exit_tree() -> void`
```logic
if _suite_thread != null and _suite_thread.is_started():
	_suite_thread.wait_to_finish()
SkillCatalog.clear_cache()
ContentCatalog.clear_cache()
WorldCatalog.clear_cache()
```

### `launch(path: String, encounter_id: String = "", opts: Dictionary = {}) -> bool`
```logic
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
pill_layer.visible = pill_shown_for(path)
_leave_armed = false
_save_prefs()
return true
```

### `restart_scene() -> bool`
```logic
if active_path.is_empty():
	return false
return launch(active_path, active_encounter, active_opts)
```

### `launch_playtest(opts: Dictionary, encounter_id: String = "") -> bool`
```logic
return launch(ARENA_SCENE, encounter_id, opts)
```

### `launch_scenario(scenario: Dictionary) -> bool`
```logic
return launch_playtest({"battle_seed": DictUtil.to_int(scenario.get("seed", 0)), "seed_source": "scenario", "scenario": str(scenario.get("id", ""))},
	str(scenario.get("encounter", "")))
```

### `reload_and_restart() -> Dictionary`
```logic
var path: String = active_path
var encounter_id: String = active_encounter
var opts: Dictionary = active_opts
if not path.is_empty():
	_close_active()
var result: Dictionary = reload_scripts()
if not path.is_empty():
	launch(path, encounter_id, opts)
return result
```

### `reload_scripts(only: Array[String] = []) -> Dictionary`
```logic
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
```

### `restart_runner() -> void`
```logic
if _pull_blocker().is_empty():
	pull_and_restart()
	return
var args: PackedStringArray = ["--path", ProjectSettings.globalize_path("res://"), SELF_SCENE]
OS.create_process(OS.get_executable_path(), args)
get_tree().quit()
```

### `static restart_pulls_blocker(os_name: String, is_web: bool, has_run_bat: bool, has_git: bool) -> String`
```logic
if is_web:
	return "Not available in the browser build"
if os_name != "Windows":
	return "Needs Windows (run.bat does the pull)"
if not has_run_bat:
	return "run.bat not found in the project folder"
if not has_git:
	return "This folder is not a git checkout"
return ""
```

### `static pull_summary(old_sha: String, new_sha: String) -> String`
```logic
if old_sha.is_empty() or new_sha.is_empty():
	return "Restarted; could not read the commit (is git on PATH?)."
if old_sha == new_sha:
	return "Already up to date (%s). Local edits, another branch or being offline skip the pull; see the console." % new_sha
return "Pulled %s → %s." % [old_sha, new_sha]
```

### `static _head_sha(project_dir: String) -> String`
```logic
var out: Array = []
if OS.execute("git", ["-C", project_dir, "rev-parse", "--short", "HEAD"], out) != 0 or out.is_empty():
	return ""
return str(out[0]).strip_edges()
```

### `_pull_blocker() -> String`
```logic
var dir: String = ProjectSettings.globalize_path("res://")
var has_git: bool = DirAccess.dir_exists_absolute(dir.path_join(".git")) or FileAccess.file_exists(dir.path_join(".git"))
return restart_pulls_blocker(OS.get_name(), OS.has_feature("web"), FileAccess.file_exists("res://run.bat"), has_git)
```

### `pull_and_restart() -> void`
```logic
var dir: String = ProjectSettings.globalize_path("res://")
var cfg := ConfigFile.new()
cfg.set_value("pull", "old_sha", _head_sha(dir))
cfg.set_value("pull", "time", Time.get_datetime_string_from_system())
cfg.save(PULL_PATH)
OS.create_process("cmd.exe", ["/c", ProjectSettings.globalize_path("res://run.bat")], true)
get_tree().quit()
```

### `_report_pull() -> void`
```logic
if not FileAccess.file_exists(PULL_PATH):
	return
var cfg := ConfigFile.new()
var old_sha: String = ""
if cfg.load(PULL_PATH) == OK:
	old_sha = str(cfg.get_value("pull", "old_sha", ""))
DirAccess.remove_absolute(ProjectSettings.globalize_path(PULL_PATH))
var text: String = pull_summary(old_sha, _head_sha(ProjectSettings.globalize_path("res://")))
_set_status(text, false)
if _output != null:
	_output.text = text
```

### `back_to_menu() -> bool`
```logic
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
```

### `exit_game() -> bool`
```logic
if active_scene != null and active_scene.has_method("has_unsaved_changes") and DictUtil.to_bool(active_scene.call("has_unsaved_changes")) and not _leave_armed:
	_leave_armed = true
	_set_status("Unsaved changes in %s: press Exit again to discard them." % _scene_name(active_path), true)
	_flash_pill("Unsaved changes: press Exit again to quit")
	return false
get_tree().quit()
return true
```

### `is_running() -> bool`
```logic
return active_scene != null
```

### `_close_active() -> void`
```logic
if active_scene != null:
	remove_child(active_scene)
	active_scene.queue_free()
	active_scene = null
```

### `_show_menu() -> void`
```logic
menu_layer.visible = true
pill_layer.visible = false
_refresh_pending()
if browser != null:
	browser.refresh()
```

### `_unhandled_key_input(event: InputEvent) -> void`
```logic
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
	KEY_F11:
		toggle_fullscreen()
		get_viewport().set_input_as_handled()
```

### `static is_fullscreen() -> bool`
```logic
var m: int = DisplaySettings.backend().current_mode()
return m == DisplayServer.WINDOW_MODE_FULLSCREEN or m == DisplayServer.WINDOW_MODE_EXCLUSIVE_FULLSCREEN
```

### `static toggle_fullscreen() -> bool`
```logic
var b: DisplaySettings.Backend = DisplaySettings.backend()
if is_fullscreen():
	b.set_mode(DisplayServer.WINDOW_MODE_WINDOWED)
	b.set_size(DisplaySettings.BASE_SIZE)
	b.centre()
	return false
b.set_mode(DisplayServer.WINDOW_MODE_FULLSCREEN)
return true
```

### `static minimize_window() -> void`
```logic
DisplaySettings.backend().set_mode(DisplayServer.WINDOW_MODE_MINIMIZED)
```

### `static window_buttons_shown() -> bool`
```logic
return not DisplaySettings.is_web()
```

### `_window_row(row: HBoxContainer) -> void`
```logic
row.add_child(UiTheme.button("Fullscreen", "Fullscreen / windowed (F11)", func() -> void: toggle_fullscreen()))
if window_buttons_shown():
	row.add_child(UiTheme.button("Minimize", "Minimize the window", func() -> void: minimize_window()))
	row.add_child(UiTheme.button("Exit", "Exit the game", func() -> void: exit_game()))
```

### `run_suite(scene_path: String) -> bool`
```logic
if _suite_thread != null and _suite_thread.is_started():
	_set_status("A suite is already running.", true)
	return false
_output.text = "[color=#94a3b8]Running %s headless... (the menu stays usable)[/color]" % scene_path
var args: PackedStringArray = ["--headless", "--path", ProjectSettings.globalize_path("res://"), scene_path]
_suite_thread = Thread.new()
_suite_thread.start(_run_process.bind(scene_path, args))
return true
```

### `_run_process(scene_path: String, args: PackedStringArray) -> void`
```logic
var out: Array = []
var code: int = OS.execute(OS.get_executable_path(), args, out, true)
_suite_done.call_deferred(scene_path, code, "\n".join(PackedStringArray(out)))
```

### `_suite_done(scene_path: String, code: int, text: String) -> void`
```logic
_suite_thread.wait_to_finish()
_suite_thread = null
_output.text = format_suite_output(text)
var counts: Vector2i = count_results(text)
var ok: bool = code == 0 and counts.y == 0
_set_status("%s: %s (%d passed, %d failed, exit %d)." % [scene_path.get_file(), "PASSED" if ok else "FAILED", counts.x, counts.y, code], not ok)
```

### `static count_results(text: String) -> Vector2i`
```logic
var passed: int = 0
var failed: int = 0
for line: String in text.split("\n"):
	if line.contains(": PASS"):
		passed += 1
	elif line.contains(": FAIL") or line.contains("SCRIPT ERROR"):
		failed += 1
return Vector2i(passed, failed)
```

### `static format_suite_output(text: String) -> String`
```logic
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
```

### `_build_menu() -> void`
```logic
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
var title_row := HBoxContainer.new()
root.add_child(title_row)
var title: Label = UiTheme.label("⚔ TACTICAL RPG  ·  SCENE RUNNER", 20, UiTheme.COL_TITLE)
title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
title_row.add_child(title)
_window_row(title_row)
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
reload_row.add_child(UiTheme.button("⏻ Restart runner", RESTART_TIP if _pull_blocker().is_empty() else "Start a fresh runner process: needed for new files / class_names", func() -> void: restart_runner(), UiTheme.FONT_MD))
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
```

### `_panel(parent: Control, title: String, width: int) -> VBoxContainer`
```logic
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
```

### `static _expand_panel(col: VBoxContainer, ratio: float) -> void`
```logic
var panel: Control = col.get_parent() as Control
panel.size_flags_vertical = Control.SIZE_EXPAND_FILL
panel.size_flags_stretch_ratio = ratio
```

### `_scene_card(parent: Control, s: Dictionary) -> void`
```logic
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
```

### `selected_encounter() -> String`
```logic
if _encounter_pick == null or _encounter_pick.selected < 0:
	return ""
return str(_encounter_pick.get_item_metadata(_encounter_pick.selected))
```

### `select_encounter(id: String) -> void`
```logic
if _encounter_pick == null:
	return
for i: int in range(_encounter_pick.item_count):
	if str(_encounter_pick.get_item_metadata(i)) == id:
		_encounter_pick.select(i)
```

### `_build_pill() -> void`
```logic
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
note_button = UiTheme.button("✎ Note", "Add a playtest note with a screenshot (F8)", func() -> void: _open_note_from_pill())
note_button.visible = false
row.add_child(note_button)
_window_row(row)
_pill_hint = UiTheme.label("", UiTheme.FONT_SM, UiTheme.COL_WARN)
row.add_child(_pill_hint)
```

### `static pill_shown_for(path: String) -> bool`
```logic
return path != "res://scenes/main_menu.tscn"
```

### `static note_button_shown(has_hub: bool, recording: bool) -> bool`
```logic
return has_hub and recording
```

### `_process(_delta: float) -> void`
```logic
if note_button != null:
	note_button.visible = note_button_shown(hub != null, hub != null and hub.is_recording())
```

### `_open_note_from_pill() -> void`
```logic
if hub == null or not hub.is_recording():
	return
pill_layer.visible = false
await RenderingServer.frame_post_draw
hub.open_note()
pill_layer.visible = pill_shown_for(active_path)
```

### `_flash_pill(text: String) -> void`
```logic
if _pill_hint != null:
	_pill_hint.text = text
```

### `_refresh_pending() -> void`
```logic
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
```

### `_set_status(text: String, bad: bool = false) -> void`
```logic
if _status != null:
	_status.text = text
	_status.add_theme_color_override("font_color", UiTheme.COL_BAD if bad else UiTheme.COL_MUTED)
```

### `status_text() -> String`
```logic
return _status.text if _status != null else ""
```

### `_scene_name(path: String) -> String`
```logic
for s: Dictionary in SCENES:
	if str(s["path"]) == path:
		return str(s["name"])
return path.get_file()
```

### `_load_prefs() -> void`
```logic
var cfg := ConfigFile.new()
if cfg.load(PREFS_PATH) == OK:
	select_encounter(str(cfg.get_value("runner", "encounter", "")))
	record_playtests = DictUtil.to_bool(cfg.get_value("runner", "record_playtests", true), true)
if hub != null:
	hub.set_record_enabled(record_playtests)
if browser != null:
	browser.set_record_enabled(record_playtests)
```

### `set_record_playtests(on: bool) -> void`
```logic
record_playtests = on
if hub != null:
	hub.set_record_enabled(on)
if browser != null:
	browser.set_record_enabled(on)
_save_prefs()
```

### `_save_prefs() -> void`
```logic
var cfg := ConfigFile.new()
cfg.load(PREFS_PATH)   # keep the other keys (a missing file is fine)
if not active_encounter.is_empty():
	cfg.set_value("runner", "encounter", active_encounter)
cfg.set_value("runner", "record_playtests", record_playtests)
cfg.save(PREFS_PATH)
```

