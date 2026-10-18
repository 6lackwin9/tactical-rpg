---
class: CurrentGame
extends: RefCounted
description: "Tactical RPG - Current game (UI1-L, S13): the GameState being played, shared by the menus, the hub and the scene hosts. Static; the state lives on the Engine singleton like the catalogs (survives scene changes). state        the GameState in play (null = no game) slot         the last slot written or loaded (-1 = none) clean_copy   state.to_dict() at the last save or load (is_dirty compares against it) busy         set by a battle or dialogue scene host while it runs (saving is refused then)"
---

# Properties
* `STATE_KEY`: StringName = &"current_game_state" (const)

# Methods

### `static _data() -> Dictionary`
```logic
if not Engine.has_meta(STATE_KEY):
	Engine.set_meta(STATE_KEY, {"state": null, "slot": -1, "clean_copy": {}, "busy": false})
var d: Dictionary = Engine.get_meta(STATE_KEY)
return d
```

### `static state() -> GameState`
```logic
return GameState.of(_data()["state"])
```

### `static slot() -> int`
```logic
return DictUtil.to_int(_data()["slot"], -1)
```

### `static clean_copy() -> Dictionary`
```logic
return DictUtil.dict_of(_data()["clean_copy"])
```

### `static busy() -> bool`
```logic
return DictUtil.to_bool(_data()["busy"])
```

### `static set_busy(value: bool) -> void`
```logic
_data()["busy"] = value
```

### `static has_game() -> bool`
```logic
return state() != null
```

### `static start(gs: GameState, from_slot: int = -1) -> void`
```logic
var d: Dictionary = _data()
d["state"] = gs
d["slot"] = from_slot
d["clean_copy"] = gs.to_dict() if gs != null else {}
d["busy"] = false
```

### `static end() -> void`
```logic
var d: Dictionary = _data()
d["state"] = null
d["slot"] = -1
d["clean_copy"] = {}
d["busy"] = false
```

### `static is_dirty() -> bool`
```logic
var gs: GameState = state()
return gs != null and not JsonUtil.deep_equal(gs.to_dict(), clean_copy())
```

### `static mark_saved(p_slot: int) -> void`
```logic
var d: Dictionary = _data()
d["slot"] = p_slot
var gs: GameState = state()
d["clean_copy"] = gs.to_dict() if gs != null else {}
```

