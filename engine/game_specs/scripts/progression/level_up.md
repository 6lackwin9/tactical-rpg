---
class: LevelUp
extends: RefCounted
description: "Tactical RPG - Level-up rules (S7): award XP, confirm a spend draft, automatic spend. Confirmed points are permanent (no respec). Pure functions over UnitState / PartyState. Adds `xp` (>= 0) to the state. Returns {levels_gained, old_level, new_level, unspent}."
---

# Properties
* `errors`: Array[String] = []
* `sum`: int = 0

# Methods

### `static award(state: UnitState, xp: int) -> Dictionary`
```logic
var old_level: int = state.level()
state.total_xp += maxi(0, xp)
var new_level: int = state.level()
return {"levels_gained": new_level - old_level, "old_level": old_level, "new_level": new_level, "unspent": state.unspent()}
```

### `static award_encounter(party: PartyState, enemies: Array[Dictionary]) -> Dictionary`
```logic
var out: Dictionary = {}
for m: UnitState in party.active_members():
	var xp: int = XpCurve.encounter_xp(enemies, m.level())
	var res: Dictionary = award(m, xp)
	res["xp"] = xp
	out[m.id] = res
return out
```

### `static auto(state: UnitState, class_growth: Dictionary) -> void`
```logic
var left: int = state.unspent()
if left <= 0:
	return
var extra: Dictionary = StatRules.auto_spend(class_growth, left)
for key: Variant in extra:
	state.spend[str(key)] = DictUtil.to_int(state.spend.get(str(key), 0)) + DictUtil.to_int(extra[key])
```

### `static auto_for(state: UnitState) -> void`
```logic
var arch: ClassArchetype = state.archetype()
auto(state, arch.growth_per_level if arch != null else {})
```

