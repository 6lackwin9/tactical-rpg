extends Node2D

# ==============================================================================
# Tactical RPG - Arena Test Harness
# Runs the headless integration suite (ArenaTestSuite). Headless: exits with the
# result code (used by check_project.ps1). Windowed: after the tests, launches
# the real combat UI foundation scene (combat_arena.tscn) for manual testing.
# ==============================================================================

const COMBAT_ARENA_SCENE: PackedScene = preload("res://scenes/combat_arena.tscn")

func _ready() -> void:
	print("\n=======================================================")
	print("=== STARTING TACTICAL RPG ARENA HARNESS             ===")
	print("=======================================================")

	var all_tests_passed: bool = ArenaTestSuite.run_all_tests(self)
	ActionLogEntry.show_formulas = true   # test harness: log cells explain their damage on hover
	var exit_code: int = 0 if all_tests_passed else 1

	if DisplayServer.get_name() == "headless":
		print("Execution finished headlessly. Exiting with code: %d\n" % exit_code)
		# Let nodes freed by the tests (queue_free) actually release before exiting.
		await get_tree().process_frame
		SvgRaster.clear_cache()
		SkillCatalog.clear_cache()
		get_tree().quit(exit_code)
	else:
		print("Running in windowed mode. Launching combat arena UI...\n")
		add_child(COMBAT_ARENA_SCENE.instantiate())
