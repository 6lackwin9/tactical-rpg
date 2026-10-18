---
class: WorldSchema
extends: RefCounted
description: "Tactical RPG - World Schema (the lore layer on top of the content catalog) A world is a folder of lore entries: regions, locations, factions, characters, named items, disciplines, quests, lore entries, encounters, loot (the world loot table quests offer as special rewards), dialogue trees, shops and routes (the world map). Entries never carry gameplay of their own: they REFERENCE base assets from the Content Forge catalog (`base:units`, `base:items`...) and add lore (names, backgrounds, art, factions), plus optional explicit tuning deltas. Field format is the ContentSchema one (type, default, min/max, options, when, help...). `ref` names what a ref/refs field points at: \"regions\" ...      another entry kind of the same world (or an ancestor world) \"base:units\" ...   a content-catalog kind \"worlds\"           another world Entries in other worlds are written \"world:id\"; unqualified refs resolve in the entry's own world first, then its ancestors (nearest wins). Every kind has `override` (the qualified id of an ancestor entry it replaces) and `tags`."
---

# Properties
* `KINDS`: Array[String] = ["regions", "locations", "factions", "characters", "items", "disciplines", "quests", "lore", "encounters", "loot", "dialogues", "shops", "routes"] (const)
* `TUNING_KEYS`: Array[String] = ["hp_pct", "power", "armor", "speed", "ap"] (const)
* `ITEM_TUNING_KEYS`: Array[String] = ["power", "mitigation", "burden"] (const)
* `list`: Array[Dictionary] = []
* `out`: Array[String] = [""]
* `out`: Array[Dictionary] = []
* `ref_kind`: String = str(f.get("ref", ""))

# Methods

### `static kind_label(kind: String) -> String`
```logic
match kind:
	"world":
		return "Overview"
	"npcs":
		return "NPCs"
return kind.capitalize()
```

### `static kind_icon(kind: String) -> String`
```logic
match kind:
	"world":
		return "🌍"
	"regions":
		return "🗺"
	"locations":
		return "📍"
	"factions":
		return "⚑"
	"characters":
		return "🧑"
	"items":
		return "🗡"
	"disciplines":
		return "✦"
	"quests":
		return "📜"
	"lore":
		return "📖"
	"encounters":
		return "⚔"
	"loot":
		return "🎁"
	"dialogues":
		return "💬"
	"shops":
		return "🛒"
	"routes":
		return "🛤"
return "•"
```

### `static _f(key: String, label: String, type: String, default: Variant, extra: Dictionary = {}) -> Dictionary`
```logic
var f: Dictionary = {"key": key, "label": label, "type": type, "default": default}
f.merge(extra, true)
return f
```

### `static field(kind: String, key: String) -> Dictionary`
```logic
for f: Dictionary in fields(kind):
	if str(f.get("key", "")) == key:
		return f
return {}
```

### `static defaults(kind: String) -> Dictionary`
```logic
var d: Dictionary = {}
for f: Dictionary in fields(kind):
	d[str(f["key"])] = ContentSchema._copy(f["default"])
return d
```

### `static normalize(kind: String, entry: Dictionary) -> Dictionary`
```logic
var out: Dictionary = {}
for f: Dictionary in fields(kind):
	var key: String = str(f["key"])
	out[key] = ContentSchema.coerce(f, entry.get(key, f["default"]))
for key: Variant in entry:
	if not out.has(key) and not str(key).begins_with("_"):
		out[key] = ContentSchema._copy(entry[key])
return out
```

### `static to_storage(kind: String, entry: Dictionary) -> Dictionary`
```logic
var full: Dictionary = normalize(kind, entry)
var out: Dictionary = {}
for f: Dictionary in fields(kind):
	var key: String = str(f["key"])
	if key == "override" and str(full[key]).is_empty():
		continue
	out[key] = full[key]
for key: Variant in full:
	if not out.has(key) and field(kind, str(key)).is_empty():
		out[key] = full[key]
return out
```

