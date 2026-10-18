---
class: CharacterStats
extends: RefCounted
description: "Tactical RPG - Character Stats & Spec Model 6-Stat Framework (STR, DEX, INT | CONS, PERC, KNOW), 1-10 Leveling"
---

# Properties
* `MAX_LEVEL`: int = 10 (const)
* `HP_PER_CONS`: int = 8   # max HP per effective CONS (const)
* `RANGE_PERC_THRESHOLD`: int = 18   # effective PERC that adds +1 range to a ranged weapon (const)
* `class_archetype`: ClassArchetype
* `archetype`: ClassArchetype:
* `level`: int = 1
* `unallocated_attribute_points`: int = 0
* `unspent_attribute_points`: int:
* `_core_stats`: Dictionary = {
* `str_stat`: int:
* `dex_stat`: int:
* `int_stat`: int:
* `cons_stat`: int:
* `perc_stat`: int:
* `know_stat`: int:
* `proficiency`: Proficiency
* `modifiers`: Array[StatModifier] = []

# Signals
* `leveled_up(new_level: int)`
* `attribute_point_allocated(stat_name: String, new_val: int)`

# Methods

### `_init(p_archetype: ClassArchetype = null, p_level: int = 1, p_spend: Dictionary = {}) -> void`
```logic
class_archetype = p_archetype if p_archetype != null else ClassArchetype.create_warrior()
proficiency = class_archetype.make_proficiency()
for s: String in _core_stats:
	_core_stats[s] = class_archetype.get_base_stat(s)
var target: int = clampi(p_level, 1, MAX_LEVEL)
var spent: int = 0
for s: String in StatRules.STATS:
	var pts: int = maxi(0, DictUtil.int_of(p_spend, s))
	_core_stats[s] += pts
	spent += pts
level = 1
while level < target:
	level += 1
	_apply_growth()
unallocated_attribute_points = maxi(0, StatRules.spend_pool(target) - spent)
```

### `level_up() -> bool`
```logic
if level >= MAX_LEVEL:
	return false

level += 1
_apply_growth()

# Free attribute point for customization (StatRules.FREE_PER_LEVEL)
unallocated_attribute_points += StatRules.FREE_PER_LEVEL

leveled_up.emit(level)
return true
```

### `_apply_growth() -> void`
```logic
if class_archetype != null:
	for s: String in _core_stats:
		_core_stats[s] += class_archetype.get_growth_for_stat(s)
```

### `add_free_attribute_points(amount: int) -> void`
```logic
unallocated_attribute_points += maxi(0, amount)
```

### `allocate_attribute_point(stat_name: String) -> bool`
```logic
if unallocated_attribute_points <= 0:
	return false

var clean: String = stat_name.to_upper()
if not _core_stats.has(clean):
	return false

_core_stats[clean] += 1
unallocated_attribute_points -= 1
attribute_point_allocated.emit(clean, get_effective_stat(clean))
return true
```

### `get_base_stat(stat_name: String) -> int`
```logic
return _core_stats.get(stat_name.to_upper(), 10)
```

### `get_effective_stat(stat_name: String, context: Dictionary = {}, exclude_source: String = "") -> int`
```logic
var clean: String = stat_name.to_upper()
var base_val: int = get_base_stat(clean)

# Apply modifiers stack
var flat_sum: int = 0
var pct_sum: float = 0.0

for mod: StatModifier in modifiers:
	if not exclude_source.is_empty() and mod.source_id == exclude_source:
		continue
	if mod.stat_name.to_upper() == clean and mod.matches_context(context):
		flat_sum += mod.flat_bonus
		pct_sum += mod.percent_bonus

var result: float = float(base_val + flat_sum) * (1.0 + pct_sum)
return maxi(1, int(roundf(result)))
```

### `get_max_hp(context: Dictionary = {}) -> int`
```logic
var effective_cons: int = get_effective_stat("CONS", context)
var effective_str: int = get_effective_stat("STR", context)
return 60 + (effective_cons * HP_PER_CONS) + (effective_str * 2)
```

### `get_initiative_speed(armor_burden: int = 0, context: Dictionary = {}) -> int`
```logic
var effective_dex: int = get_effective_stat("DEX", context)
return maxi(1, 8 + effective_dex - armor_burden)
```

### `get_mitigation_rating(shield_mitigation: int = 0, context: Dictionary = {}) -> int`
```logic
var effective_str: int = get_effective_stat("STR", context)
return shield_mitigation + int(roundf(float(effective_str) * 0.25))
```

### `get_effective_range(base_range: int = 1, context: Dictionary = {}) -> int`
```logic
var effective_perc: int = get_effective_stat("PERC", context)
var bonus: int = 1 if effective_perc >= RANGE_PERC_THRESHOLD and base_range > 1 else 0
return maxi(1, base_range + bonus)
```

### `get_carry_contribution(context: Dictionary = {}) -> int`
```logic
var effective_str: int = get_effective_stat("STR", context)
var effective_cons: int = get_effective_stat("CONS", context)
return (effective_str * 3) + effective_cons
```

### `get_crit_rate(context: Dictionary = {}) -> float`
```logic
var effective_dex: int = get_effective_stat("DEX", context)
return DamageCalculator.CRIT_BASE + float(maxi(0, effective_dex - DamageCalculator.CRIT_DEX_FLOOR)) * DamageCalculator.CRIT_PER_DEX
```

### `get_crit_damage_multiplier(_context: Dictionary = {}) -> float`
```logic
return DamageCalculator.CRIT_MULT
```

### `get_debuff_resistance(context: Dictionary = {}) -> float`
```logic
var effective_int: int = get_effective_stat("INT", context)
return float(effective_int) * 2.5
```

### `add_modifier(mod: StatModifier) -> void`
```logic
modifiers.append(mod)
```

### `remove_modifier(source_id: String) -> void`
```logic
modifiers = modifiers.filter(func(m: StatModifier) -> bool: return m.source_id != source_id)
```

