class_name HubModel
extends RefCounted

# ==============================================================================
# Tactical RPG - In-game hub model (UI1-L): where the party is, who is in it, and the doors to the party screen (UI2),
# journal (UI3), save and settings; quit_to_menu asks when there is unsaved progress (rule 7).
# ==============================================================================

const ENTRIES: Array[String] = [MenuRouter.PARTY, MenuRouter.JOURNAL, MenuRouter.SAVE, MenuRouter.SETTINGS]

var router: MenuRouter = null

func _init(p_router: MenuRouter) -> void:
	router = p_router

func world_name() -> String:
	var gs: GameState = CurrentGame.state()
	return str(WorldCatalog.manifest(gs.world_id).get("name", gs.world_id)) if gs != null else ""

func location_name() -> String:
	var gs: GameState = CurrentGame.state()
	return SaveSlotsModel.location_name(gs.world_id, gs.location_id) if gs != null else ""

# [{id, name, level, active}] per party member.
func party() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var gs: GameState = CurrentGame.state()
	if gs == null:
		return out
	var active: Array[UnitState] = gs.party.active_members()
	for m: UnitState in gs.party.members:
		var unit: Dictionary = ContentCatalog.get_entry("units", m.base_unit)
		out.append({"id": m.id, "name": str(unit.get("name", m.base_unit)), "level": XpCurve.level_for(m.total_xp), "active": active.has(m)})
	return out

# True when `id` has a screen in this build (the party screen and journal land with UI2-L and UI3-L).
func can_open(id: String) -> bool:
	return router.make_model(id) != null

func open(id: String) -> Array[String]:
	var errors: Array[String] = []
	if not ENTRIES.has(id):
		errors.append("Unknown hub entry '%s'." % id)
	elif not can_open(id):
		errors.append("The %s screen is not built yet." % id)
	else:
		router.push(id)
	return errors

func quit_to_menu() -> Array[String]:
	if CurrentGame.is_dirty():
		router.ask("Quit without saving?", _quit_now, true)
	else:
		_quit_now()
	var none: Array[String] = []
	return none

func _quit_now() -> void:
	CurrentGame.end()
	router.clear_to(MenuRouter.MAIN)
