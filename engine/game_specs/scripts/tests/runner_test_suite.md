---
class: RunnerTestSuite
extends: RefCounted
description: "Tactical RPG - Scene Runner Test Suite The runner lists every runnable scene, launches the arena with a chosen catalog encounter, restarts it, returns to the menu, guards unsaved Forge work, hot-reloads an edited script and parses suite output. (Suites run as a separate process in the real runner; that path isn't spawned here.)"
---

# Properties
* `TAG`: String = "Runner" (const)

# Methods

### `static run(tree_parent: Node = null) -> bool`
```logic
print("\n=======================================================")
print("=== STARTING SCENE RUNNER TEST                      ===")
print("=======================================================")
var ok: bool = true

# Scene list: every known scene exists; hidden ones (runner, test harness) are not offered.
var listed: Array[String] = []
for s: Dictionary in SceneRunner.runnable_scenes():
	listed.append(str(s["path"]))
var list_ok: bool = listed.has("res://scenes/combat_arena.tscn") and listed.has("res://scenes/content_forge.tscn") and listed.has("res://scenes/gear_dojo.tscn") and not listed.has("res://scenes/scene_runner.tscn") and not listed.has("res://scenes/test_arena.tscn")
for path: String in listed:
	list_ok = list_ok and ResourceLoader.exists(path)
ok = TestHelpers.report_ok(TAG, "Lists every runnable scene (hides the runner and test harness)", list_ok, ", ".join(listed)) and ok

# Suite output parsing (PASS / FAIL counting and colouring).
var sample: String = "  [A] one: PASS\n  [B] two: FAIL\nSCRIPT ERROR: boom\n=== X ==="
var counts: Vector2i = SceneRunner.count_results(sample)
var formatted: String = SceneRunner.format_suite_output(sample)
ok = TestHelpers.report_ok(TAG, "Suite output counts PASS/FAIL and script errors", counts == Vector2i(1, 2) and formatted.contains("#4ade80") and formatted.contains("#f87171"), str(counts)) and ok

# Web glyph fallbacks: the runner's icons resolve in the default font or a bundled fallback.
var base: Font = ThemeDB.fallback_font
var saved: Array[Font] = base.fallbacks.duplicate()
WebFonts.install(true)
var missing: String = ""
var glyphs: String = "⚔⚒🌍🛡🗺▶↻⏻☰⟳✕⚠💀⛶⧉─"
for i: int in glyphs.length():
	var code: int = glyphs.unicode_at(i)
	var found: bool = base.has_char(code)
	for f: Font in base.fallbacks:
		found = found or f.has_char(code)
	if not found:
		missing += glyphs[i]
var added: int = base.fallbacks.size() - saved.size()
base.fallbacks = saved
var web_entry: String = str(ProjectSettings.get_setting("application/run/main_scene.web", ""))
ok = TestHelpers.report_ok(TAG, "Browser build starts in the main menu; desktop keeps the runner", web_entry == "res://scenes/main_menu.tscn" and ResourceLoader.exists(web_entry) and str(ProjectSettings.get_setting("application/run/main_scene")) == SceneRunner.SELF_SCENE, web_entry) and ok
ok = TestHelpers.report_ok(TAG, "Web fallback fonts cover the runner's icon glyphs", missing.is_empty() and added == WebFonts.FALLBACKS.size(), "missing '%s', added %d" % [missing, added]) and ok

var pr_ok: bool = SceneRunner.restart_pulls_blocker("Windows", false, true, true).is_empty()
var pr_web: bool = not SceneRunner.restart_pulls_blocker("Windows", true, true, true).is_empty()
var pr_linux: bool = not SceneRunner.restart_pulls_blocker("Linux", false, true, true).is_empty()
var pr_bat: bool = not SceneRunner.restart_pulls_blocker("Windows", false, false, true).is_empty()
var pr_git: bool = not SceneRunner.restart_pulls_blocker("Windows", false, true, false).is_empty()
ok = TestHelpers.report_ok(TAG, "Restart runner pulls only on Windows desktop with run.bat and .git", pr_ok and pr_web and pr_linux and pr_bat and pr_git) and ok
var sum_diff: String = SceneRunner.pull_summary("abc1234", "def5678")
var sum_same: String = SceneRunner.pull_summary("abc1234", "abc1234")
var sum_empty: String = SceneRunner.pull_summary("", "def5678")
ok = TestHelpers.report_ok(TAG, "Pull summary: changed shows old -> new; equal or unknown old says up to date with a hint",
	sum_diff == "Pulled abc1234 → def5678." and sum_same.begins_with("Already up to date (abc1234)") and sum_same.contains("offline") and sum_empty.begins_with("Restarted; could not read the commit") and SceneRunner.pull_summary("abc1234", "") == sum_empty, sum_same) and ok

var fake := DisplaySettings.Backend.new()
DisplaySettings.use_backend(fake)
var to_full: bool = SceneRunner.toggle_fullscreen() and SceneRunner.is_fullscreen()
var to_window: bool = not SceneRunner.toggle_fullscreen() and fake.mode == DisplayServer.WINDOW_MODE_WINDOWED and fake.size == DisplaySettings.BASE_SIZE
fake.mode = DisplayServer.WINDOW_MODE_EXCLUSIVE_FULLSCREEN
var from_exclusive: bool = not SceneRunner.toggle_fullscreen() and fake.mode == DisplayServer.WINDOW_MODE_WINDOWED
SceneRunner.minimize_window()
var minimized: bool = fake.mode == DisplayServer.WINDOW_MODE_MINIMIZED
DisplaySettings.use_backend()
ok = TestHelpers.report_ok(TAG, "Window controls: F11 toggles fullscreen and windowed (from either fullscreen kind), Minimize minimizes",
	to_full and to_window and from_exclusive and minimized and SceneRunner.window_buttons_shown() == not DisplaySettings.is_web(), str(fake.calls)) and ok

if tree_parent == null:
	return TestHelpers.report_ok(TAG, "Launch / menu / reload (needs a scene tree)", true, "skipped") and ok
var saved_enabled: int = PlaytestStore.force_enabled
PlaytestStore.force_enabled = 0   # no recordings into user://playtests from the tests (PLAYTEST1)
var runner: SceneRunner = (load("res://scenes/scene_runner.tscn") as PackedScene).instantiate() as SceneRunner
tree_parent.add_child(runner)
ok = TestHelpers.report_ok(TAG, "With playtests off the runner has no PlaytestHub and no Playtests panel (zero hooks)", runner.hub == null and runner.browser == null) and ok

ok = TestHelpers.report_ok(TAG, "Note button exists, shows only with a recording hub",
	runner.note_button != null and not runner.note_button.visible and SceneRunner.note_button_shown(true, true)
	and not SceneRunner.note_button_shown(true, false) and not SceneRunner.note_button_shown(false, true)) and ok

ok = TestHelpers.report_ok(TAG, "Pill hidden for the real game (main menu), shown for test scenes",
	not SceneRunner.pill_shown_for("res://scenes/main_menu.tscn") and SceneRunner.pill_shown_for("res://scenes/combat_arena.tscn")) and ok

# Launch the arena with a chosen catalog encounter; restart keeps it; menu frees it.
var launched: bool = runner.launch("res://scenes/combat_arena.tscn", "veterans_drill")
var arena: Node = runner.active_scene
var arena_session: CombatSession = arena.get("session") if arena != null else null
var right_battle: bool = launched and arena_session != null and arena_session.encounter.encounter_name == "Veterans' Drill" and not runner.menu_layer.visible
runner.restart_scene()
var restarted: Node = runner.active_scene
var restart_session: CombatSession = restarted.get("session") if restarted != null else null
var restart_ok: bool = restarted != arena and restart_session != null and restart_session.encounter.encounter_name == "Veterans' Drill"
var back: bool = runner.back_to_menu() and not runner.is_running() and runner.menu_layer.visible
ok = TestHelpers.report_ok(TAG, "Launches the arena with a catalog encounter, restarts it, returns to the menu", right_battle and restart_ok and back,
	"launched %s, restart %s, menu %s" % [right_battle, restart_ok, back]) and ok

# Leaving the Forge with unsaved edits needs a second press.
runner.launch("res://scenes/content_forge.tscn")
ContentCatalog.get_entry("skills", "strike")   # make sure the catalog is loaded
var strike: Dictionary = ContentCatalog.get_entry("skills", "strike")
strike["description"] = "Edited in the runner test."
ContentCatalog.upsert("skills", strike)
var first_press: bool = runner.back_to_menu()
var still_open: bool = runner.is_running()
var second_press: bool = runner.back_to_menu()
var discarded: bool = str(ContentCatalog.get_entry("skills", "strike").get("description", "")) != "Edited in the runner test."
ok = TestHelpers.report_ok(TAG, "Unsaved Forge edits: first Menu press warns, second leaves and discards them", not first_press and still_open and second_press and discarded and not ContentCatalog.is_dirty()) and ok

# Hot reload: an edited script recompiles in place (here: an unchanged reload of DictUtil).
var result: Dictionary = runner.reload_scripts(["res://scripts/core/utils/dict_util.gd"])
var reloaded: Array = result["reloaded"]
var failed: Array = result["failed"]
var reload_ok: bool = reloaded.size() == 1 and failed.is_empty() and DictUtil.to_int("12") == 12
var pending: Dictionary = runner.reloader.pending()
var still_changed: Array = pending["changed"]
var pending_ok: bool = still_changed.is_empty()
ok = TestHelpers.report_ok(TAG, "Hot reload recompiles a script in place; nothing left pending", reload_ok and pending_ok, "reloaded %d, failed %s" % [reloaded.size(), str(failed)]) and ok

tree_parent.remove_child(runner)   # leaves the tree now: its PlaytestHub stops being PlaytestHub.instance
runner.queue_free()
PlaytestStore.force_enabled = saved_enabled
return ok
```

