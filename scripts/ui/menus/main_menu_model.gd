class_name MainMenuModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Main menu model (UI1-L): Continue (the newest save, autosave included; hidden when none), New game,
# Load, Settings, Tools (debug builds only) and Quit (not in the browser, where a tab can't be closed by the game).
# ==============================================================================

var router: MenuRouter = null

func _init(p_router: MenuRouter) -> void:
	router = p_router

func can_continue() -> bool:
	return SaveGame.newest_slot() >= 0

func show_tools() -> bool:
	return OS.is_debug_build()

func show_quit() -> bool:
	return not OS.has_feature("web")

# Loads the newest save (rule 2).
func continue_game() -> Array[String]:
	var slot: int = SaveGame.newest_slot()
	if slot < 0:
		var none: Array[String] = ["There is no save to continue."]
		return none
	return SaveSlotsModel.load_slot(router, slot)

func open(id: String) -> void:
	router.push(id)

# Quit asks when the game in progress has unsaved changes (rule 7).
func quit_requested() -> Array[String]:
	if CurrentGame.is_dirty():
		router.ask("Quit without saving?", router.quit_requested.emit, true)
	else:
		router.quit_requested.emit()
	var none: Array[String] = []
	return none
