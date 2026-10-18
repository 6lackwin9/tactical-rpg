---
class: PartyModel
extends: RefCounted
description: "Tactical RPG - Party model (UI2-L): the roster with the active members (at most max_active, RECRUIT1-L), the selected member, gold, and the doors to level-up, loadout and gear. Toggling active edits PartyState.active_ids directly (rule 2)."
---

# Properties
* `MAX_ACTIVE`: String = "The active party holds at most %d." (const)
* `KEEP_ONE`: String = "Keep at least one active member." (const)
* `router`: MenuRouter = null
* `selected`: String = ""
* `out`: Array[Dictionary] = []
* `active`: Array[UnitState] = _party().active_members()
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `party`: PartyState = _party()
* `out`: Array[Dictionary] = []
* `m`: UnitState = _party().member(selected)
* `errors`: Array[String] = []

# Methods

### `_init(p_router: MenuRouter) -> void`
```logic
router = p_router
var gs: GameState = CurrentGame.state()
if gs != null and not gs.party.members.is_empty():
	selected = gs.party.members[0].id
```

### `_party() -> PartyState`
```logic
return CurrentGame.state().party
```

### `active_count() -> int`
```logic
return _party().active_members().size()
```

### `max_active() -> int`
```logic
return _party().max_active
```

### `badge() -> bool`
```logic
for m: UnitState in _party().members:
	if m.unspent() > 0:
		return true
return false
```

### `gold() -> int`
```logic
return _party().gold
```

### `stats() -> StatsModel`
```logic
return StatsModel.new(selected)
```

