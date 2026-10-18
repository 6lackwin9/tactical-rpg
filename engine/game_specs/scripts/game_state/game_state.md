---
class: GameState
extends: RefCounted
description: "Tactical RPG - Game State (S13): the session-wide snapshot that is saved and loaded. world, location, party, flags (bool / int / String, key [a-z0-9_.]), the structured quests blob (S8: qualified quest id -> {state, objective}, checked by QuestRules.validate_blob) and loot_claimed (qualified world loot ids, unique, saved sorted). Whole floats come back as ints. S9 (EXP1-L, save v3): location_id is a QUALIFIED location id (world:id) or \"\"; position is the token tile on the location's map ([-1, -1] = no map); explore holds the step counter; rng_seed + rolls drive GameRolls. S11 (DLG1-L, save v4): party.gold (PartyState) and shops = {shop qid: {item id: sold count >= 1}} (limited stock sold). S10 (TRAV1-L, save v5): time = the world clock (int >= 0), advanced by travel legs. AUTO1-L (save v7): autoplay = {on: bool, party: tactic id, heroes: {party member id: tactic id or \"custom\"}}, the remembered battle-autoplay choice (PlanPresets.AUTO_IDS; off in a new game). CombatAutoplay reads and writes it. HEAL1-L (save v9): explore.last_rest = qualified id of the last rest place entered (\"\" = the start location; an id that no longer resolves is read as \"\" by the runtime, never a load error); party members store hp_lost (UnitState)."
---

# Properties
* `KEYS`: Array[String] = ["world_id", "location_id", "party", "flags", "quests", "loot_claimed", "position", "explore", "rng_seed", "rolls", "shops", "time", "autoplay"] (const)
* `AUTOPLAY_KEYS`: Array[String] = ["on", "party", "heroes"] (const)
* `EXPLORE_KEYS`: Array[String] = ["steps", "next_at", "loc_steps", "last_safe", "last_rest"] (const)
* `HOME_WORLD_KEY`: String = "home_world"   # optional extra explore key: the world a `connects` crossing left (absent = none) (const)
* `NO_POSITION`: Vector2i = Vector2i(-1, -1) (const)
* `SEED_MAX`: int = 2147483647 (const)
* `world_id`: String = ""
* `location_id`: String = ""
* `party`: PartyState = PartyState.new()
* `flags`: Dictionary = {}
* `quests`: Dictionary = {}
* `loot_claimed`: Array[String] = []
* `position`: Vector2i = NO_POSITION
* `explore`: Dictionary = default_explore()
* `rng_seed`: int = 0
* `rolls`: Dictionary = {}   # stream id -> rolls drawn (int >= 0)
* `shops`: Dictionary = {}   # shop qid -> {item id: sold count >= 1}
* `time`: int = 0   # world clock (TRAV1-L)
* `autoplay`: Dictionary = default_autoplay()   # AUTO1-L remembered autoplay choice
* `load_problems`: Array[String] = []   # raw type problems found by from_dict (not saved; validate() reports them)
* `out`: Array[String] = list.duplicate()
* `errors`: Array[String] = load_problems.duplicate()
* `seen`: Dictionary = {}
* `errors`: Array[String] = []
* `ok`: bool = a.size() == AUTOPLAY_KEYS.size()
* `heroes`: Dictionary = a["heroes"]
* `errors`: Array[String] = []
* `sold`: Dictionary = blob[key]
* `errors`: Array[String] = []
* `has_home`: bool = e.has(HOME_WORLD_KEY)
* `ok`: bool = e.size() == EXPLORE_KEYS.size() + (1 if has_home else 0)

# Methods

### `static default_explore() -> Dictionary`
```logic
return {"steps": 0, "next_at": 16, "loc_steps": 0, "last_safe": "", "last_rest": ""}
```

### `static default_autoplay() -> Dictionary`
```logic
return {"on": false, "party": PlanPresets.DEFAULT_TACTIC, "heroes": {}}
```

### `static create(p_world_id: String, p_party: PartyState) -> GameState`
```logic
var g := GameState.new()
g.world_id = p_world_id
g.party = p_party
g.party.apply_cap(WorldCatalog.party_max(p_world_id))   # RECRUIT1-L 1.3
return g
```

### `static of(v: Variant) -> GameState`
```logic
if v is GameState:
	return v
return null
```

### `static flag_key_ok(key: String) -> bool`
```logic
if key.is_empty():
	return false
for i: int in range(key.length()):
	var c: String = key[i]
	if not ((c >= "a" and c <= "z") or (c >= "0" and c <= "9") or c == "_" or c == "."):
		return false
return true
```

### `static flag_value_ok(v: Variant) -> bool`
```logic
return v is bool or v is int or v is String
```

### `set_flag(key: String, value: Variant) -> String`
```logic
if not flag_key_ok(key):
	return "Flag key '%s' must be non-empty [a-z0-9_.]." % key
if not flag_value_ok(value):
	return "Flag '%s' must be a bool, int or String." % key
flags[key] = value
return ""
```

### `get_flag(key: String, fallback: Variant = false) -> Variant`
```logic
return flags.get(key, fallback)
```

### `has_flag(key: String) -> bool`
```logic
return flags.has(key)
```

### `clear_flag(key: String) -> void`
```logic
flags.erase(key)
```

### `to_dict() -> Dictionary`
```logic
return {
	"world_id": world_id, "location_id": location_id, "party": party.to_dict(),
	"flags": flags.duplicate(true), "quests": quests.duplicate(true), "loot_claimed": _sorted(loot_claimed),
	"position": [position.x, position.y], "explore": explore.duplicate(true), "rng_seed": rng_seed, "rolls": rolls.duplicate(true),
	"shops": shops.duplicate(true), "time": time, "autoplay": autoplay.duplicate(true),
}
```

### `static from_dict(d: Dictionary) -> GameState`
```logic
var g := GameState.new()
g.world_id = str(d.get("world_id", ""))
g.location_id = str(d.get("location_id", ""))
g.party = PartyState.from_dict(DictUtil.dict_of(d.get("party", {})))
g.party.apply_cap(WorldCatalog.party_max(g.world_id))   # RECRUIT1-L 1.3: a longer active_ids keeps its first ids
var raw_flags: Dictionary = DictUtil.dict_of(JsonUtil.normalize(d.get("flags", {})))
for key: Variant in raw_flags:
	g.flags[str(key)] = raw_flags[key]
# Damaged blobs are never coerced silently: the raw type problems are kept and validate() reports them, so
# SaveGame.read refuses the save (claimed loot can't become claimable again without anyone noticing).
var raw_quests: Variant = d.get("quests", {})
if not (raw_quests is Dictionary):
	g.load_problems.append("quests must be an object (the save has %s)." % type_string(typeof(raw_quests)))
g.quests = DictUtil.dict_of(JsonUtil.normalize(raw_quests))
var raw_loot: Variant = d.get("loot_claimed", [])
if not (raw_loot is Array):
	g.load_problems.append("loot_claimed must be a list (the save has %s)." % type_string(typeof(raw_loot)))
else:
	for x: Variant in DictUtil.array_of(raw_loot):
		if not (x is String):
			g.load_problems.append("loot_claimed: '%s' is not a String." % str(x))
g.loot_claimed = JsonUtil.strings(raw_loot)
var raw_pos: Variant = JsonUtil.normalize(d.get("position", [NO_POSITION.x, NO_POSITION.y]))
var pos: Array = DictUtil.array_of(raw_pos)
if pos.size() != 2 or not (pos[0] is int) or not (pos[1] is int):
	g.load_problems.append("position must be [x, y].")
else:
	g.position = Vector2i(DictUtil.to_int(pos[0]), DictUtil.to_int(pos[1]))
var raw_explore: Variant = JsonUtil.normalize(d.get("explore", default_explore()))
if not (raw_explore is Dictionary):
	g.load_problems.append("explore must be {steps, next_at, loc_steps, last_safe, last_rest}.")
else:
	g.explore = raw_explore
var raw_seed: Variant = JsonUtil.normalize(d.get("rng_seed", 0))
if not (raw_seed is int):
	g.load_problems.append("rng_seed must be a whole number.")
else:
	g.rng_seed = DictUtil.to_int(raw_seed)
var raw_rolls: Variant = JsonUtil.normalize(d.get("rolls", {}))
if not (raw_rolls is Dictionary):
	g.load_problems.append("rolls must be an object {stream: count}.")
else:
	g.rolls = raw_rolls
var raw_shops: Variant = JsonUtil.normalize(d.get("shops", {}))
if not (raw_shops is Dictionary):
	g.load_problems.append("shops must be an object {shop qid: {item id: sold count}}.")
else:
	g.shops = raw_shops
var raw_time: Variant = JsonUtil.normalize(d.get("time", 0))
if not (raw_time is int):
	g.load_problems.append("time must be a whole number >= 0.")
else:
	g.time = DictUtil.to_int(raw_time)
var raw_auto: Variant = JsonUtil.normalize(d.get("autoplay", default_autoplay()))
if not (raw_auto is Dictionary):
	g.load_problems.append("autoplay must be {on, party, heroes}.")
else:
	g.autoplay = raw_auto
return g
```

### `equals(other: GameState) -> bool`
```logic
return other != null and JsonUtil.deep_equal(to_dict(), other.to_dict())
```

