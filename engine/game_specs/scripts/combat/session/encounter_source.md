---
class: EncounterSource
extends: RefCounted
description: "Tactical RPG - Encounter Source (every battle the game can run) One place that lists and creates encounters from both layers: \"goblin_ambush\"       a content-catalog encounter (lore-free base battle) \"ashlands:skirmish\"   a world encounter, played in that world (its characters) Used by the Scene Runner, the arena's --encounter argument and the World Forge."
---

# Properties
* `out`: Array[Dictionary] = []
* `world_name`: String = str(WorldCatalog.manifest(w).get("name", w))
* `inherited`: String = "" if str(e["_world"]) == w else " (from %s)" % str(e["_world"])

# Methods

### `static exists(id: String) -> bool`
```logic
if id.contains(":"):
	var parts: Array[String] = WorldCatalog.split_qid(id)
	return parts.size() == 2 and WorldCatalog.has_entry(parts[0], "encounters", parts[1])
return ContentCatalog.has("encounters", id)
```

### `static create(id: String) -> CombatEncounter`
```logic
if id.contains(":"):
	return WorldEncounter.create(id)
return CatalogEncounter.new(id)
```

