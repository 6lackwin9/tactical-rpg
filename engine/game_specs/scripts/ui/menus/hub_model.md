---
class: HubModel
extends: RefCounted
description: "Tactical RPG - Game menu model (UI1-L hub; UIR-2, UI-REVIEW1 Q1 B): during play Esc / the Menu button open it as a popup over the paused map. It shows where the party is and who is in it, and opens Party (roster, level up, skills, gear), Journal, Save game, Load game and Settings; Return to map closes it; quit_to_menu asks when there is unsaved progress (rule 7). An entry that can't open now says why (`reason`): saving and party changes wait while a battle, conversation or shop is open (CurrentGame.busy), the journal waits for a first quest."
---

# Properties
* `ENTRIES`: Array[String] = [MenuRouter.PARTY, MenuRouter.JOURNAL, MenuRouter.SAVE, MenuRouter.LOAD, MenuRouter.SETTINGS] (const)
* `BUSY_PARTY`: String = "Party changes wait until the battle, conversation or shop is over." (const)
* `NO_QUESTS`: String = "No quests yet." (const)
* `NO_SAVES`: String = "There are no saves to load yet." (const)
* `BUSY_LEAVE`: String = "Finish the fight, conversation or shop first."   # quitting and loading wait too (the pre-UIR-2 leave rule) (const)
* `router`: MenuRouter = null
* `out`: Array[Dictionary] = []
* `gs`: GameState = CurrentGame.state()
* `active`: Array[UnitState] = gs.party.active_members()
* `reserve`: Array[Dictionary] = []
* `hp`: Dictionary = HealRules.current_hp(m)
* `row`: Dictionary = {"id": m.id, "name": PartyPreview.name_of(m), "class": PartyPreview.class_name_of(m), "level": m.level(), "active": active.has(m),
* `errors`: Array[String] = []
* `why`: String = reason(id)
* `errors`: Array[String] = []
* `why`: String = quit_reason()
* `refused`: Array[String] = [why]
* `none`: Array[String] = []

# Methods

### `_init(p_router: MenuRouter) -> void`
```logic
router = p_router
```

### `world_name() -> String`
```logic
var gs: GameState = CurrentGame.state()
return str(WorldCatalog.manifest(gs.world_id).get("name", gs.world_id)) if gs != null else ""
```

### `location_name() -> String`
```logic
var gs: GameState = CurrentGame.state()
return SaveSlotsModel.location_name(gs.world_id, gs.location_id) if gs != null else ""
```

### `unspent_points() -> int`
```logic
var total: int = 0
var gs: GameState = CurrentGame.state()
if gs != null:
	for m: UnitState in gs.party.members:
		total += maxi(m.unspent(), 0)
return total
```

### `label(id: String) -> String`
```logic
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
```

### `describe(id: String) -> String`
```logic
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
```

### `reason(id: String) -> String`
```logic
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
```

### `can_open(id: String) -> bool`
```logic
return reason(id).is_empty()
```

### `can_return() -> bool`
```logic
return router.stack.size() >= 2 and router.stack[-1] == MenuRouter.HUB and router.stack[-2] == MenuRouter.PLAY
```

### `static has_quests() -> bool`
```logic
var gs: GameState = CurrentGame.state()
if gs == null:
	return false
for g: Array in JournalModel.GROUPS:
	if not QuestRunner.quests_with(gs, str(g[0])).is_empty():
		return true
return false
```

### `quit_reason() -> String`
```logic
return BUSY_LEAVE if CurrentGame.has_game() and CurrentGame.busy() else ""
```

### `_quit_now() -> void`
```logic
CurrentGame.end()
router.clear_to(MenuRouter.MAIN)
```

