class_name HubModel
extends RefCounted

# ==============================================================================
# Tactical RPG - Game menu model (UI1-L hub; UIR-2, UI-REVIEW1 Q1 B): during play Esc / the Menu button open it as a
# popup over the paused map. It shows where the party is and who is in it, and opens Party (roster, level up, skills,
# gear), Journal, Save game, Load game and Settings; Return to map closes it; quit_to_menu asks when there is unsaved
# progress (rule 7). An entry that can't open now says why (`reason`): saving and party changes wait while a battle,
# conversation or shop is open (CurrentGame.busy), the journal waits for a first quest.
# ==============================================================================

const ENTRIES: Array[String] = [MenuRouter.PARTY, MenuRouter.JOURNAL, MenuRouter.SAVE, MenuRouter.LOAD, MenuRouter.SETTINGS]
const BUSY_PARTY: String = "Party changes wait until the battle, conversation or shop is over."
const NO_QUESTS: String = "No quests yet."
const NO_SAVES: String = "There are no saves to load yet."
const BUSY_LEAVE: String = "Finish the fight, conversation or shop first."   # quitting and loading wait too (the pre-UIR-2 leave rule)

var router: MenuRouter = null

func _init(p_router: MenuRouter) -> void:
	router = p_router

func world_name() -> String:
	var gs: GameState = CurrentGame.state()
	return str(WorldCatalog.manifest(gs.world_id).get("name", gs.world_id)) if gs != null else ""

func location_name() -> String:
	var gs: GameState = CurrentGame.state()
	return SaveSlotsModel.location_name(gs.world_id, gs.location_id) if gs != null else ""

# [{id, name, class, level, active, hp, max_hp, unspent}] per party member, active ones first (roster order kept).
func party() -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var gs: GameState = CurrentGame.state()
	if gs == null:
		return out
	var active: Array[UnitState] = gs.party.active_members()
	var reserve: Array[Dictionary] = []
	for m: UnitState in gs.party.members:
		var hp: Dictionary = HealRules.current_hp(m)
		var row: Dictionary = {"id": m.id, "name": PartyPreview.name_of(m), "class": PartyPreview.class_name_of(m), "level": m.level(), "active": active.has(m),
			"hp": DictUtil.to_int(hp["hp"]), "max_hp": DictUtil.to_int(hp["max"]), "unspent": m.unspent()}
		if active.has(m):
			out.append(row)
		else:
			reserve.append(row)
	out.append_array(reserve)
	return out

# Unspent level-up points over the whole roster.
func unspent_points() -> int:
	var total: int = 0
	var gs: GameState = CurrentGame.state()
	if gs != null:
		for m: UnitState in gs.party.members:
			total += maxi(m.unspent(), 0)
	return total

# The entry's button label; Party names the points waiting to be spent (UI-REVIEW1 hub finding: no cryptic dot).
func label(id: String) -> String:
	match id:
		MenuRouter.PARTY:
			var pts: int = unspent_points()
			return "Party (%d point%s to spend)" % [pts, "" if pts == 1 else "s"] if pts > 0 else "Party"
		MenuRouter.JOURNAL:
			return "Journal"
		MenuRouter.SAVE:
			return "Save game"
		MenuRouter.LOAD:
			return "Load game"
		MenuRouter.SETTINGS:
			return "Settings"
	return id

# What the entry does, shown on focus in the menu's detail line (Q4 B).
func describe(id: String) -> String:
	match id:
		MenuRouter.PARTY:
			return "Roster, level up, skills and gear."
		MenuRouter.JOURNAL:
			return "Quests ready, active, on hold and completed."
		MenuRouter.SAVE:
			return "Save to one of the manual slots."
		MenuRouter.LOAD:
			return "Load a save; unsaved progress here is lost (you are asked first)."
		MenuRouter.SETTINGS:
			return "Sound, display, gameplay and controls."
	return ""

# Why `id` can't open right now ("" = it can). Settings always opens; Load needs a save on disk.
func reason(id: String) -> String:
	if not ENTRIES.has(id):
		return "Unknown hub entry '%s'." % id
	if id != MenuRouter.SETTINGS and id != MenuRouter.LOAD and not CurrentGame.has_game():
		return SaveSlotsModel.NO_GAME if id == MenuRouter.SAVE else "There is no game in progress."
	match id:
		MenuRouter.PARTY:
			return BUSY_PARTY if CurrentGame.busy() else ""
		MenuRouter.SAVE:
			return SaveSlotsModel.BUSY if CurrentGame.busy() else ""
		MenuRouter.JOURNAL:
			return "" if has_quests() else NO_QUESTS
		MenuRouter.LOAD:
			if CurrentGame.has_game() and CurrentGame.busy():
				return BUSY_LEAVE
			return NO_SAVES if SaveGame.newest_slot() < 0 else ""
	return ""

func can_open(id: String) -> bool:
	return reason(id).is_empty()

func open(id: String) -> Array[String]:
	var errors: Array[String] = []
	var why: String = reason(id)
	if not why.is_empty():
		errors.append(why)
	else:
		router.push(id)
	return errors

# True when the game scene is right below this menu (the popup over the map); Return to map closes the menu.
func can_return() -> bool:
	return router.stack.size() >= 2 and router.stack[-1] == MenuRouter.HUB and router.stack[-2] == MenuRouter.PLAY

func return_to_map() -> Array[String]:
	var errors: Array[String] = []
	if can_return():
		router.pop()
	else:
		errors.append("There is no map to return to.")
	return errors

# True when the journal has something to show (stops at the first status group with a quest; no JournalModel built).
static func has_quests() -> bool:
	var gs: GameState = CurrentGame.state()
	if gs == null:
		return false
	for g: Array in JournalModel.GROUPS:
		if not QuestRunner.quests_with(gs, str(g[0])).is_empty():
			return true
	return false

# Why Quit to main menu can't run now ("" = it can): refused while a battle, conversation or shop is open.
func quit_reason() -> String:
	return BUSY_LEAVE if CurrentGame.has_game() and CurrentGame.busy() else ""

func quit_to_menu() -> Array[String]:
	var why: String = quit_reason()
	if not why.is_empty():
		var refused: Array[String] = [why]
		return refused
	if CurrentGame.is_dirty():
		router.ask("Quit to the main menu without saving? Progress since your last save is lost.", _quit_now, true)
	else:
		_quit_now()
	var none: Array[String] = []
	return none

func _quit_now() -> void:
	CurrentGame.end()
	router.clear_to(MenuRouter.MAIN)
