---
class: NewGameModel
extends: RefCounted
description: "Tactical RPG - New game model (UI1-L): the playable worlds with their fixed start party; choose() creates the game (NewGame.create_for_world), makes it current, writes the autosave and enters the game. UIR-2: the world cards are selectable (`selected`, the first playable world by default) and one Start (start()) begins the selected world."
---

# Properties
* `NONE_SELECTED`: String = "Choose a world first." (const)
* `router`: MenuRouter = null
* `last_errors`: Array[String] = []   # errors of the last asked start (the host shows them)
* `selected`: String = ""             # the world card picked (UIR-2)
* `errors`: Array[String] = []
* `none`: Array[String] = [NONE_SELECTED]
* `out`: Array[Dictionary] = []
* `m`: Dictionary = WorldCatalog.manifest(w)
* `party`: Array[Dictionary] = []
* `base_id`: String = unit_id
* `shown`: String = ""
* `hero`: UnitState = NewGame.hero_from_character(w, unit_id)
* `e`: Dictionary = ContentCatalog.get_entry("units", base_id)
* `cls`: Dictionary = ContentCatalog.get_entry("classes", str(e.get("class_id", "")))
* `head`: Dictionary = SaveGame.peek(SaveGame.AUTOSAVE_SLOT)
* `world`: String = str(head["world_id"])
* `at`: int = DictUtil.to_int(head["saved_at"])
* `asked`: Array[String] = []
* `res`: Dictionary = NewGame.create_for_world(world_id, p_seed)
* `errors`: Array[String] = []
* `gs`: GameState = GameState.of(res["state"])
* `none`: Array[String] = []

# Methods

### `_init(p_router: MenuRouter) -> void`
```logic
router = p_router
var playable: Array[String] = NewGame.playable_worlds()
selected = playable[0] if not playable.is_empty() else ""
```

### `static party_line(party: Array) -> String`
```logic
var names: PackedStringArray = PackedStringArray()
var levels: Dictionary = {}
for raw: Variant in party:
	levels[DictUtil.to_int(DictUtil.dict_of(raw).get("level", 1), 1)] = true
var shared: bool = levels.size() == 1
for raw: Variant in party:
	var p: Dictionary = DictUtil.dict_of(raw)
	var cls: String = str(p.get("class", ""))
	var bits: PackedStringArray = PackedStringArray()
	if not cls.is_empty() and cls != str(p.get("name", "")):
		bits.append(cls)
	if not shared:
		bits.append("Lv %d" % DictUtil.to_int(p.get("level", 1), 1))
	names.append(str(p.get("name", "")) + (" (%s)" % ", ".join(bits) if not bits.is_empty() else ""))
if names.is_empty():
	return "Party: none"
return ("Party, all Lv %d: " % DictUtil.to_int(levels.keys()[0], 1) if shared else "Party: ") + ", ".join(names)
```

