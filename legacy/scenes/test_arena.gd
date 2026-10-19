extends Node2D

# ==============================================================================
# Tactical RPG - Arena Test Harness
# Runs every headless suite (SuiteRegistry.run_all). Headless: exits with the
# result code (used by check_project.ps1 and the Scene Runner's suite buttons).
# Windowed: after the tests, opens the Scene Runner.
# ==============================================================================

func _ready() -> void:
	print("\n=======================================================")
	print("=== STARTING TACTICAL RPG ARENA HARNESS             ===")
	print("=======================================================")

	var systems: PackedStringArray = PackedStringArray()
	for arg: String in OS.get_cmdline_user_args():
		if arg.begins_with("--systems="):
			systems = arg.trim_prefix("--systems=").split(",", false)
	if not systems.is_empty():
		print("=== Systems: %s" % ", ".join(systems))
	var all_tests_passed: bool = SuiteRegistry.run_all(self, systems)
	ActionLogEntry.show_formulas = true   # test harness: log cells explain their damage on hover
	var exit_code: int = 0 if all_tests_passed else 1

	if DisplayServer.get_name() == "headless":
		print("Execution finished headlessly. Exiting with code: %d\n" % exit_code)
		# Let nodes freed by the tests (queue_free) actually release before exiting.
		await get_tree().process_frame
		SvgRaster.clear_cache()
		SkillCatalog.clear_cache()
		ContentCatalog.clear_cache()
		get_tree().quit(exit_code)
	else:
		print("Running in windowed mode. Opening the Scene Runner...\n")
		get_tree().change_scene_to_file.call_deferred("res://scenes/scene_runner.tscn")
