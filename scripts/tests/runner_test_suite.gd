class_name RunnerTestSuite
extends RefCounted

# ==============================================================================
# Tactical RPG - Scene Runner Test Suite
# The runner lists every runnable scene, launches the arena with a chosen
# catalog encounter, restarts it, returns to the menu, guards unsaved Forge
# work, hot-reloads an edited script and parses suite output. (Suites run as a
# separate process in the real runner; that path isn't spawned here.)
# ==============================================================================

const TAG: String = "Runner"

static func run(tree_parent: Node = null) -> bool:
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

	if tree_parent == null:
		return TestHelpers.report_ok(TAG, "Launch / menu / reload (needs a scene tree)", true, "skipped") and ok
	var runner: SceneRunner = (load("res://scenes/scene_runner.tscn") as PackedScene).instantiate() as SceneRunner
	tree_parent.add_child(runner)

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

	runner.queue_free()
	return ok
