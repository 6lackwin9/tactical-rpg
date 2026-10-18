---
class: UnitState
extends: RefCounted
description: "Tactical RPG - Unit State (S7): the persistent snapshot of one party member. Plain data with to_dict / from_dict / validate. The level is DERIVED from total_xp (never stored); confirmed spend points are permanent. PartyBuilder turns a state into a Combatant."
---

# Properties
* `id`: String = ""
* `base_unit`: String = ""
* `lore_ref`: String = ""
* `total_xp`: int = 0
* `spend`: Dictionary = {}                 # stat -> confirmed points (missing = 0)
* `learned_skills`: Array[String] = []     # extras beyond the class path (Combatant.granted_skills)
* `passives`: Array[String] = []           # permanent passives
* `loadout`: Dictionary = {"class": [], "extra": []}   # family ids; empty = default loadout
* `equipment`: Dictionary = {}             # slot name (ContentSchema.LOADOUT_SLOTS) -> item id; unlisted = class default
* `carried`: Array[String] = []            # item ids in the unit's own carry
* `pouch`: Array[String] = []              # ITEMUSE1: consumable ids in battle-pouch slot order (max POUCH_MAX, repeats allowed)
* `hp_lost`: int = 0
* `POUCH_MAX`: int = 3 (const)
* `arch`: ClassArchetype = archetype()
* `ids`: Array[String] = []
* `errors`: Array[String] = []
* `at`: String = "Unit '%s'" % id
* `entry`: Dictionary = ContentCatalog.get_entry("units", base_unit)
* `arch`: ClassArchetype = ContentFactory.class_of(entry)
* `errors`: Array[String] = []
* `errors`: Array[String] = []
* `cls`: Array[String] = JsonUtil.strings(loadout.get("class", []))
* `ext`: Array[String] = JsonUtil.strings(loadout.get("extra", []))
* `owned`: Array[String] = owned_families()
* `seen`: Array[String] = []
* `errors`: Array[String] = []
* `prof`: Proficiency = arch.make_proficiency()
* `slot`: String = str(key)
* `item_id`: String = str(equipment[key])
* `item`: Dictionary = ContentCatalog.get_entry("items", item_id)

# Methods

### `static create(p_id: String, p_base_unit: String) -> UnitState`
```logic
var s := UnitState.new()
s.id = p_id
s.base_unit = p_base_unit
return s
```

### `static at_level(p_base_unit: String, level: int, p_id: String = "") -> UnitState`
```logic
var s: UnitState = create(p_id if not p_id.is_empty() else p_base_unit, p_base_unit)
s.total_xp = XpCurve.xp_for_level(level)
return s
```

### `level() -> int`
```logic
return XpCurve.level_for(total_xp)
```

### `spent() -> int`
```logic
return StatRules.total_of(spend)
```

### `unspent() -> int`
```logic
return StatRules.spend_pool(level()) - spent()
```

### `class_id() -> String`
```logic
return str(ContentCatalog.get_entry("units", base_unit).get("class_id", ""))
```

### `archetype() -> ClassArchetype`
```logic
return ContentFactory.class_of(ContentCatalog.get_entry("units", base_unit))
```

### `to_dict() -> Dictionary`
```logic
var clean_spend: Dictionary = {}
for stat: String in StatRules.STATS:
	if DictUtil.to_int(spend.get(stat, 0)) > 0:
		clean_spend[stat] = DictUtil.to_int(spend[stat])
return {
	"id": id, "base_unit": base_unit, "lore_ref": lore_ref, "total_xp": total_xp,
	"spend": clean_spend, "learned_skills": learned_skills.duplicate(), "passives": passives.duplicate(),
	"loadout": {"class": JsonUtil.strings(loadout.get("class", [])), "extra": JsonUtil.strings(loadout.get("extra", []))},
	"equipment": equipment.duplicate(), "carried": carried.duplicate(), "pouch": pouch.duplicate(), "hp_lost": hp_lost,
}
```

### `static _hp_lost_of(raw: Variant) -> int`
```logic
if raw == null:
	return 0
if raw is int:
	return DictUtil.to_int(raw)
if raw is float:
	var x: float = raw
	if is_finite(x) and x == floorf(x):
		return int(x)
return -1
```

### `static from_dict(d: Dictionary) -> UnitState`
```logic
var s := UnitState.new()
s.id = str(d.get("id", ""))
s.base_unit = str(d.get("base_unit", ""))
s.lore_ref = str(d.get("lore_ref", ""))
s.total_xp = DictUtil.to_int(d.get("total_xp", 0))
s.spend = JsonUtil.ints(d.get("spend", {}))
s.learned_skills = JsonUtil.strings(d.get("learned_skills", []))
s.passives = JsonUtil.strings(d.get("passives", []))
var lo: Dictionary = DictUtil.dict_of(d.get("loadout", {}))
s.loadout = {"class": JsonUtil.strings(lo.get("class", [])), "extra": JsonUtil.strings(lo.get("extra", []))}
var gear: Dictionary = {}
for key: Variant in DictUtil.dict_of(d.get("equipment", {})):
	gear[str(key)] = str(DictUtil.dict_of(d.get("equipment", {}))[key])
s.equipment = gear
s.carried = JsonUtil.strings(d.get("carried", []))
s.pouch = JsonUtil.strings(d.get("pouch", []))   # missing (v7 and older) = empty
s.hp_lost = _hp_lost_of(d.get("hp_lost", null))
return s
```

### `duplicate_state() -> UnitState`
```logic
return UnitState.from_dict(to_dict())
```

### `equals(other: UnitState) -> bool`
```logic
return other != null and JsonUtil.deep_equal(to_dict(), other.to_dict())
```

### `validate_split() -> Dictionary`
```logic
var loadout_errs: Array[String] = _loadout_errors("Unit '%s'" % id)
var other: Array[String] = []
for e: String in validate():
	if not loadout_errs.has(e):
		other.append(e)
return {"loadout": loadout_errs, "other": other}
```

