---
class: ItemValidator
extends: RefCounted
description: "Item and item-family validation rules (CLN-9 split; FORGE1_ITEMS 1, 2, 4, 6). ContentCatalog.validate_entry dispatches here. Problems are {\"field\", \"index\" (1-based list row, 0 = the field), \"message\"}; the *_errors functions map them to strings. Legacy mode (no item families yet, until ITEM1-A): items are flat rows checked by the pre-family rules. Tier limits come from ItemModifiers (TIER_* constants), shared with the Forge tier table (ItemFormModel.tier_limits)."
---

# Properties
* `out`: Array[String] = []
* `out`: Array[Dictionary] = []
* `icon`: String = str(e.get("icon_id", ""))
* `type`: String = ItemFamilies.item_type_of(family)
* `family_id`: String = str(family.get("id", ""))
* `hands`: Array[String] = ItemFamilies.hands_of(family)
* `hand`: String = str(e.get("hand", ""))
* `scaling`: Array[String] = ContentFactory._strings(family.get("scaling_stats", []))
* `slots`: Array[String] = ContentFactory._strings(family.get("slots", []))
* `out`: Array[Dictionary] = []
* `type`: String = ItemFamilies.item_type_of(family)
* `family_id`: String = str(family.get("id", ""))
* `mods`: Array[Dictionary] = ItemModifiers.modifiers_of(e)
* `raw_list`: Array = DictUtil.array_of(e.get("modifiers", []))
* `cap`: int = DictUtil.to_int(family.get("max_modifiers", 0))
* `allowed`: Array[String] = ContentFactory._strings(family.get("modifier_kinds", []))
* `adjustable`: Array[String] = ContentFactory._strings(family.get("adjustable", []))
* `numbers`: Array[String] = ItemFamilies.number_fields_for(type)
* `seen`: Dictionary = {}
* `mod`: Dictionary = mods[i]
* `n`: int = i + 1
* `kind`: String = str(mod.get("kind", ""))
* `spec`: Dictionary = ItemModifiers.KINDS[kind]
* `params`: Array = spec["params"]
* `unique_tag`: String = kind
* `field`: String = str(mod.get("field", ""))
* `stat`: String = str(mod.get("stat", ""))
* `fx`: Variant = mod.get("effect", null)
* `raw`: Dictionary = ItemModifiers.raw_numbers(e, family)
* `base`: Dictionary = ItemFamilies.base_of(family, str(e.get("hand", "")))
* `rng`: Array = ItemFamilies.NUMBER_RANGES[field]
* `v`: int = DictUtil.to_int(raw[field])
* `out`: Array[Dictionary] = []
* `v`: int = DictUtil.to_int(value)
* `out`: Array[Dictionary] = []
* `type`: String = ItemFamilies.item_type_of(family)
* `tiers`: Array[Dictionary] = ItemModifiers.tiers_of(e)
* `raw_list`: Array = DictUtil.array_of(e.get("tiers", []))
* `cap`: int = DictUtil.to_int(family.get("max_tiers", 0))
* `seen`: Dictionary = {}
* `last`: Dictionary = {}   # criterion group -> last value
* `t`: Dictionary = tiers[i]
* `n`: int = i + 1
* `when`: Dictionary = DictUtil.dict_of(t.get("when", {}))
* `group`: String = ""
* `value`: int = 0
* `stat`: String = str(when.get("stat", ""))
* `gear`: String = str(when.get("gear", str(family.get("id", ""))))
* `tag`: String = JSON.stringify(ItemModifiers.sorted_value(when))
* `out`: Array[Dictionary] = []
* `any`: bool = false
* `ranges`: Dictionary = ItemModifiers.TIER_BONUS_RANGES
* `k`: String = str(key)
* `p`: int = DictUtil.to_int(bonus[key])
* `p_rng`: Array = ranges["power"]
* `m`: int = DictUtil.to_int(bonus[key])
* `m_rng`: Array = ranges["mitigation"]
* `stats`: Dictionary = DictUtil.dict_of(bonus[key])
* `s_rng`: Array = ranges["stat_bonuses"]
* `fx`: Array = DictUtil.array_of(bonus[key])
* `out`: Array[Dictionary] = []
* `out`: Array[Dictionary] = []
* `out`: Array[Dictionary] = []
* `out`: Array[String] = []
* `con_type`: String = str(e.get("consumable_type", ""))
* `type_index`: int = Consumable.ConsumableType.keys().find(con_type)
* `hand`: String = str(e.get("hand", ""))
* `band`: Dictionary = ItemFamilies.band_of(family, hand, field)
* `v`: int = DictUtil.to_int(e.get(str(ItemFamilies.FLAT_KEYS[field]), 0))
* `bonus_band`: Dictionary = ItemFamilies.band_of(family, hand, ItemFamilies.BONUS_BAND)
* `value`: float = bonus_value(e)
* `errors`: Array[String] = []
* `type`: String = str(e["item_type"])
* `slot`: String = str(e["slot"])
* `tiers`: Variant = e["threshold_tiers"]
* `tier_list`: Array = tiers
* `tier`: Dictionary = t if t is Dictionary else {}
* `out`: Array[Dictionary] = []
* `type`: String = ItemFamilies.item_type_of(f)
* `id`: String = str(f.get("id", ""))
* `hands`: Array[String] = ItemFamilies.hands_of(f)
* `seen`: Dictionary = {}
* `scaling`: Array[String] = ContentFactory._strings(f.get("scaling_stats", []))
* `allowed_slots`: Array[String] = ItemFamilies.ARMOR_SLOTS if type == "ARMOR" else ItemFamilies.ACCESSORY_SLOTS
* `slots`: Array[String] = ContentFactory._strings(f.get("slots", []))
* `numbers`: Array[String] = ItemFamilies.number_fields_for(type)
* `kinds`: Array[String] = ContentFactory._strings(f.get("modifier_kinds", []))
* `old_id`: String = current_id if not current_id.is_empty() else id
* `users`: Array[Dictionary] = ContentCatalog.items_of_family(old_id)
* `failing`: Array[String] = []
* `row`: Dictionary = ContentSchema.normalize("items", ItemModifiers.resolve(ItemModifiers.stored(item), f))
* `errors`: Array[String] = item_errors(row, f, true)
* `out`: Array[Dictionary] = []
* `keys`: Array[String] = ItemFamilies.hands_of(f)
* `base`: Dictionary = DictUtil.dict_of(f.get("base", {}))
* `needs_base`: bool = ItemFamilies.MODIFIABLE_TYPES.has(type)
* `row`: Dictionary = DictUtil.dict_of(base[key])
* `rng`: Array = ItemFamilies.NUMBER_RANGES[str(field)]
* `v`: int = DictUtil.to_int(row[field])
* `bands`: Dictionary = DictUtil.dict_of(f.get("bands", {}))
* `row`: Dictionary = DictUtil.dict_of(bands[key])
* `name`: String = str(field)
* `band`: Dictionary = DictUtil.dict_of(row[field])
* `lo`: float = DictUtil.to_float(band.get("min", 0))
* `hi`: float = DictUtil.to_float(band.get("max", 0))
* `rng`: Array = ItemFamilies.NUMBER_RANGES[name]
* `out`: Array[String] = []
* `hands`: Array[String] = ItemFamilies.hands_of(f)
* `keys`: Array[String] = hands.duplicate()
* `base`: Dictionary = ItemFamilies.base_of(f, key)
* `band`: Dictionary = ItemFamilies.band_of(f, key, field)
* `v`: int = DictUtil.to_int(base[field])
* `one`: int = DictUtil.to_int(ItemFamilies.base_of(f, ItemFamilies.HAND_MAIN_1H)["power"])
* `two`: int = DictUtil.to_int(ItemFamilies.base_of(f, ItemFamilies.HAND_MAIN_2H)["power"])
* `band`: Dictionary = ItemFamilies.band_of(f, key, "power")
* `errors`: Array[String] = []
* `ids`: Array[String] = []

# Methods

### `static problem(field: String, index: int, message: String) -> Dictionary`
```logic
return {"field": field, "index": index, "message": message}
```

### `static _skills() -> Callable`
```logic
return func(skill_id: String) -> bool: return ContentCatalog.has("skills", skill_id)
```

### `static bonus_value(e: Dictionary) -> float`
```logic
var total: float = 0.0
for mod: Dictionary in ItemModifiers.modifiers_of(e):
	var kind: String = str(mod.get("kind", ""))
	if kind == ItemModifiers.STAT_BONUS or kind == ItemModifiers.EFFECT:
		total += GearBudget.modifier_value(mod)
return total
```

