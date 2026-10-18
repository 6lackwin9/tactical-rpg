---
class: Proficiency
extends: RefCounted
description: "Tactical RPG - Proficiency (what a unit can wield, and how well) A dictionary keyed by gear type (weapon types and armor types): {\"sword\": {\"level\": 3, \"scaling\": 1.5}, \"leather\": {\"level\": 2, \"scaling\": 1.5}} Defined by the class (editable in the Content Forge) and raised by passives (`bonus_levels`: e.g. a faction's training). Unlisted types can't be equipped, unless a passive grants a level in them. scaling   how easy the gear is to wield for this class: the multiplier on the scaling stat in Unit Power (DamageCalculator; 1.0-2.0, default 1.5). level 1-3 what training adds, per level: +LEVEL_POWER Weapon Power -LEVEL_DISCOUNT stat requirement points (heavier gear earlier) crit chance +1% per level above 1 (weapons) armor burden -1 per level above 1 (armor) level 3 (master): weapon block cost -1 (minimum 1) and gates skills that need training (SkillDef.min_proficiency)."
---

# Properties
* `MAX_LEVEL`: int = 3 (const)
* `DEFAULT_SCALING`: float = 1.5 (const)
* `PASSIVE_ONLY_SCALING`: float = 1.25   # untrained-by-class gear a passive made usable (const)
* `MIN_SCALING`: float = 1.0 (const)
* `MAX_SCALING`: float = 2.0 (const)
* `LEVEL_POWER`: int = 1 (const)
* `LEVEL_DISCOUNT`: int = 2 (const)
* `WEAPON_TYPES`: Array[String] = ["sword", "axe", "mace", "hammer", "fist", "dagger", "bow", "crossbow", "staff", "wand", "orb", "tome", "shield"] (const)
* `ARMOR_TYPES`: Array[String] = ["plate", "leather", "cloth"] (const)
* `out`: Array[String] = []
* `entries`: Dictionary = {}        # gear type -> {"level": int, "scaling": float}
* `bonus_levels`: Dictionary = {}   # gear type -> levels added by passives

# Methods

### `static from_dict(raw: Variant) -> Proficiency`
```logic
var p := Proficiency.new()
if raw is Dictionary:
	var src: Dictionary = raw
	for key: Variant in src:
		var e: Dictionary = DictUtil.dict_of(src[key])
		p.set_entry(str(key), DictUtil.to_int(e.get("level", 1), 1), DictUtil.to_float(e.get("scaling", DEFAULT_SCALING), DEFAULT_SCALING))
return p
```

### `set_entry(gear_type: String, level: int, scaling: float = DEFAULT_SCALING) -> void`
```logic
entries[gear_type.to_lower()] = {"level": clampi(level, 0, MAX_LEVEL), "scaling": clampf(scaling, MIN_SCALING, MAX_SCALING)}
```

### `base_level(gear_type: String) -> int`
```logic
var e: Dictionary = DictUtil.dict_of(entries.get(gear_type.to_lower(), {}))
return DictUtil.to_int(e.get("level", 0))
```

### `add_bonus(gear_type: String, levels: int) -> void`
```logic
var key: String = gear_type.to_lower()
bonus_levels[key] = DictUtil.int_of(bonus_levels, key) + levels
```

### `level_of(gear_type: String) -> int`
```logic
var key: String = gear_type.to_lower()
if key.is_empty():
	return 0
return clampi(base_level(key) + DictUtil.int_of(bonus_levels, key), 0, MAX_LEVEL)
```

### `can_use(gear_type: String) -> bool`
```logic
return gear_type.is_empty() or level_of(gear_type) >= 1
```

### `scaling_of(gear_type: String) -> float`
```logic
var key: String = gear_type.to_lower()
if entries.has(key):
	return DictUtil.to_float(DictUtil.dict_of(entries[key]).get("scaling", DEFAULT_SCALING), DEFAULT_SCALING)
return PASSIVE_ONLY_SCALING if level_of(key) > 0 else DEFAULT_SCALING
```

### `get_weapon_requirement_discount(gear_type: String) -> int`
```logic
return level_of(gear_type) * LEVEL_DISCOUNT
```

### `get_weapon_power_bonus(gear_type: String) -> int`
```logic
return level_of(gear_type) * LEVEL_POWER
```

### `crit_bonus(gear_type: String) -> float`
```logic
return float(maxi(0, level_of(gear_type) - 1))
```

### `burden_reduction(gear_type: String) -> int`
```logic
return maxi(0, level_of(gear_type) - 1)
```

### `block_cost_reduction(gear_type: String) -> int`
```logic
return 1 if level_of(gear_type) >= MAX_LEVEL else 0
```

### `duplicate_set() -> Proficiency`
```logic
var copy := Proficiency.new()
copy.entries = entries.duplicate(true)
copy.bonus_levels = bonus_levels.duplicate()
return copy
```

### `summary() -> String`
```logic
var parts: Array[String] = []
for key: Variant in entries:
	parts.append("%s %d" % [str(key), level_of(str(key))])
return ", ".join(parts)
```

