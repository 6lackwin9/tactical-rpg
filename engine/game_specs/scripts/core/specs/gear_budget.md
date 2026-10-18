---
class: GearBudget
extends: RefCounted
description: "Tactical RPG - Gear budget rules (M2d, mirrors StatRules for gear) One weighted number per equipment item; a class's default loadout value is the sum over its slots (pouch consumables ignored). The sum is compared with TARGET_BUDGET +- TOLERANCE. Items flagged `special` may push a loadout over the band only when they carry a `special_note` documenting the trade-off; they are always reported separately. Pure functions over catalog entries (Dictionaries); validation is soft (warnings). Item value = base_power*W_POWER + mitigation*W_MITIGATION + (sum of stat bonuses)*W_STAT_POINT + burden*W_BURDEN + (block_cost - 1)*W_EXTRA_BLOCK"
---

# Properties
* `W_POWER`: float = 1.0 (const)
* `W_MITIGATION`: float = 1.5 (const)
* `W_STAT_POINT`: float = 2.0 (const)
* `W_BURDEN`: float = -2.0 (const)
* `W_EXTRA_BLOCK`: float = -3.0 (const)
* `W_SKILL_POTENCY`: float = 20.0 (const)
* `W_STATUS_POTENCY`: float = 0.2 (const)
* `W_SPELL_REACH`: float = 3.0 (const)
* `TARGET_BUDGET`: float = 23.0 (const)
* `TOLERANCE`: float = 0.20 (const)
* `UNARMED_VALUE`: float = float(DamageCalculator.UNARMED_BASE_POWER) (const)
* `W_TIER`: float = 0.5 (const)
* `rows`: Array[Dictionary] = []
* `id`: String = str(class_entry.get(slot, ""))
* `item`: Dictionary = ContentCatalog.get_entry("items", id)
* `warnings`: Array[String] = []
* `rows`: Array[Dictionary] = loadout_breakdown(class_entry)
* `total`: float = loadout_value(class_entry)
* `over`: bool = total > band_high()
* `tops`: Array[String] = []
* `warnings`: Array[String] = []

# Methods

### `static band_low() -> float`
```logic
return TARGET_BUDGET * (1.0 - TOLERANCE)
```

### `static band_high() -> float`
```logic
return TARGET_BUDGET * (1.0 + TOLERANCE)
```

### `static is_gear(entry: Dictionary) -> bool`
```logic
return ContentSchema.EQUIP_TYPES.has(str(entry.get("item_type", "")))
```

### `static is_special(entry: Dictionary) -> bool`
```logic
return DictUtil.to_bool(entry.get("special", false))
```

### `static item_value(entry: Dictionary) -> float`
```logic
var e: Dictionary = ContentSchema.normalize("items", entry)
if not is_gear(e):
	return 0.0
var bonus_total: int = 0
var bonuses: Dictionary = DictUtil.dict_of(e["stat_bonuses"])
for key: Variant in bonuses:
	bonus_total += DictUtil.to_int(bonuses[key])
var v: float = DictUtil.to_int(e["base_power"]) * W_POWER
v += DictUtil.to_int(e["mitigation"]) * W_MITIGATION
v += bonus_total * W_STAT_POINT
v += DictUtil.to_int(e["burden"]) * W_BURDEN
v += maxi(0, DictUtil.to_int(e["block_cost"]) - 1) * W_EXTRA_BLOCK
for raw: Variant in DictUtil.array_of(e["effects"]):
	v += effect_value(DictUtil.dict_of(raw))
return v
```

### `static modifier_value(mod: Dictionary) -> float`
```logic
match str(mod.get("kind", "")):
	ItemModifiers.ADJUST:
		var delta: int = DictUtil.to_int(mod.get("delta", 0))
		match str(mod.get("field", "")):
			"power":
				return delta * W_POWER
			"mitigation":
				return delta * W_MITIGATION
			"burden":
				return delta * W_BURDEN
			"block_cost":
				return delta * W_EXTRA_BLOCK
		return 0.0
	ItemModifiers.STAT_BONUS:
		return DictUtil.to_int(mod.get("value", 0)) * W_STAT_POINT
	ItemModifiers.EFFECT:
		return effect_value(DictUtil.dict_of(mod.get("effect", {})))
return 0.0
```

### `static effect_value(fx: Dictionary) -> float`
```logic
match str(fx.get("type", "")):
	"skill_potency":
		return DictUtil.to_float(fx.get("value", 0.0)) * W_SKILL_POTENCY
	"status_potency":
		return DictUtil.to_int(fx.get("value", 0)) * W_STATUS_POTENCY
	"spell_reach":
		return DictUtil.to_int(fx.get("value", 0)) * W_SPELL_REACH
return 0.0
```

### `static tier_bonus_value(bonus: Dictionary) -> float`
```logic
var v: float = DictUtil.to_int(bonus.get("power", 0)) * W_POWER
v += DictUtil.to_int(bonus.get("mitigation", 0)) * W_MITIGATION
var stats: Dictionary = DictUtil.dict_of(bonus.get("stat_bonuses", {}))
for key: Variant in stats:
	v += DictUtil.to_int(stats[key]) * W_STAT_POINT
for raw: Variant in DictUtil.array_of(bonus.get("effects", [])):
	v += effect_value(DictUtil.dict_of(raw))
return v
```

### `static tier_potential(item: Dictionary) -> float`
```logic
var tiers: Array[Dictionary] = ItemModifiers.tiers_of(item)
if tiers.is_empty():
	for raw: Variant in DictUtil.array_of(item.get("threshold_tiers", [])):
		tiers.append(DictUtil.dict_of(raw))
var total: float = 0.0
for tier: Dictionary in tiers:
	total += tier_bonus_value(Equipment.tier_bonus(tier))
return total * W_TIER
```

### `static base_value(family: Dictionary, hand: String) -> float`
```logic
if not is_gear({"item_type": ItemFamilies.item_type_of(family)}):
	return 0.0
var base: Dictionary = ItemFamilies.base_of(family, hand)
var v: float = DictUtil.to_int(base["power"]) * W_POWER   # as item_value: the resolved base_power counts for every gear row
v += DictUtil.to_int(base["mitigation"]) * W_MITIGATION
v += DictUtil.to_int(base["burden"]) * W_BURDEN
v += maxi(0, DictUtil.to_int(base["block_cost"]) - 1) * W_EXTRA_BLOCK
return v
```

### `static loadout_value(class_entry: Dictionary) -> float`
```logic
var total: float = 0.0
for row: Dictionary in loadout_breakdown(class_entry):
	total += DictUtil.to_float(row["value"])
return total
```

### `static special_value(class_entry: Dictionary) -> float`
```logic
var total: float = 0.0
for row: Dictionary in loadout_breakdown(class_entry):
	if DictUtil.to_bool(row["special"]):
		total += DictUtil.to_float(row["value"])
return total
```

### `static in_band(value: float) -> bool`
```logic
return value >= band_low() - 0.0001 and value <= band_high() + 0.0001
```

### `static has_documented_special(class_entry: Dictionary) -> bool`
```logic
for row: Dictionary in loadout_breakdown(class_entry):
	if DictUtil.to_bool(row["special"]) and not str(row["note"]).strip_edges().is_empty():
		return true
return false
```

