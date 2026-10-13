extends Node2D

# ==============================================================================
# Tactical RPG - Gear & Skill Benchmark Dojo (bootstrap)
# Thin scene host: creates the headless DojoSession, then either runs the
# verification suite (headless) or mounts the modular window workspace.
#   Logic    -> scripts/dojo/       (DojoSession, loadouts, breakdowns)
#   UI       -> scripts/ui/dojo/    (workspace, windows, six panel components)
#   Tests    -> scenes/dojo/dojo_assertions.gd
# ==============================================================================

var session: DojoSession = null
var canvas_layer: CanvasLayer = null
var workspace: DojoWorkspace = null
var gear_modal: CharacterEquipmentModal = null

func _ready() -> void:
	print("\n=======================================================")
	print("=== TACTICAL RPG: GEAR & SKILL BENCHMARK DOJO       ===")
	print("=======================================================")

	session = DojoSession.new()

	if DisplayServer.get_name() == "headless":
		print("Running headlessly. Executing Dojo verification assertions...")
		var ok: bool = DojoAssertions.run(session, self)
		var code: int = 0 if ok else 1
		print("Dojo headless test complete. Exiting with code: %d\n" % code)
		SvgRaster.clear_cache()
		SkillCatalog.clear_cache()
		ContentCatalog.clear_cache()
		get_tree().quit(code)
	else:
		print("Initializing interactive Gear & Skill Dojo UI...\n")
		_build_dojo_ui()

func _build_dojo_ui() -> void:
	canvas_layer = CanvasLayer.new()
	add_child(canvas_layer)

	workspace = DojoWorkspace.new()
	canvas_layer.add_child(workspace)

	var gear_panel := DojoGearPanel.new()
	var panels: Array[DojoPanel] = [
		DojoUnitInspectorPanel.new(),
		gear_panel,
		DojoStagePanel.new(),
		DojoConsolePanel.new(),
		DojoDerivedStatsPanel.new(),
		DojoLogPanel.new(),
	]
	workspace.setup(session, panels)

	# Full Party & Stash screen (modal) shares the dojo session, so it stays in sync with every window.
	gear_modal = CharacterEquipmentModal.new()
	gear_modal.attach_session(session)
	canvas_layer.add_child(gear_modal)

	gear_panel.stash_requested.connect(_open_stash)
	workspace.stash_requested.connect(_open_stash)
	workspace.reset_requested.connect(session.reset)

func _open_stash() -> void:
	if gear_modal != null:
		gear_modal.open_modal(session.active_hero)
