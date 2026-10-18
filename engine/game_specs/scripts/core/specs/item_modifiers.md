---
class: ItemModifiers
extends: RefCounted
description: "Tactical RPG - Item modifiers (FORGE1_ITEMS section 2): base item = family + an ordered modifier list A STORED item (items.json version 2) names its family, hand, scaling stat, modifiers and tiers; the RESOLVED flat row (what ContentCatalog.entries / get_entry return, and every engine consumer reads) carries the old flat keys computed from the family. resolve() and collapse() are inverses on every engine key (ENGINE_KEYS). Pure static functions; no catalog access."
---

# Properties
* `ADJUST`: String = "adjust" (const)
* `STAT_BONUS`: String = "stat_bonus" (const)
* `REQUIREMENT`: String = "requirement" (const)
* `EFFECT`: String = "effect" (const)
* `KINDS`: Dictionary = { (const)
* `out`: Array[String] = []
* `DERIVED_KEYS`: Array[String] = ["item_type", "weapon_type", "two_handed", "off_hand_category", "base_power", "base_range", (const)
* `FLAT_MARKERS`: Array[String] = ["base_power", "base_range", "block_cost", "burden", "mitigation", "required_stats", (const)
* `ENGINE_KEYS`: Array[String] = ["item_type", "slot", "weapon_type", "base_power", "base_range", "two_handed", "off_hand_category", (const)
* `out`: Array[Dictionary] = []
* `out`: Array[Dictionary] = []
* `out`: Array[String] = []
* `WHEN_STAT`: String = "stat_at_least" (const)
* `WHEN_PROFICIENCY`: String = "proficiency_at_least" (const)
* `WHEN_LEVEL`: String = "unit_level_at_least" (const)
* `WHEN_TYPES`: Array[String] = [WHEN_STAT, WHEN_PROFICIENCY, WHEN_LEVEL] (const)
* `BONUS_KEYS`: Array[String] = ["power", "mitigation", "stat_bonuses", "effects"] (const)
* `TIER_KEYS`: Array[String] = ["label", "when", "bonus", "desc"] (const)
* `TIER_LABEL_MAX`: int = 24 (const)
* `TIER_DESC_MAX`: int = 120 (const)
* `TIER_STAT_MIN`: int = 1   # stat_at_least value range (const)
* `TIER_STAT_MAX`: int = 40 (const)
* `TIER_BONUS_RANGES`: Dictionary = {"power": [0, 50], "mitigation": [0, 20], "stat_bonuses": [1, 5]} (const)
* `TIER_MAX_STATS`: int = 3 (const)
* `TIER_MAX_EFFECTS`: int = 2 (const)
* `TIER_BONUS_TYPES`: Dictionary = {"power": ["WEAPON"], "mitigation": ["ARMOR", "SHIELD"], (const)
* `out`: Array[String] = []
* `types`: Array = TIER_BONUS_TYPES[key]
* `out`: Array[Dictionary] = []
* `when`: Dictionary = DictUtil.dict_of(tier.get("when", {}))
* `is_stat`: bool = str(when.get("type", "")) == WHEN_STAT
* `bonus`: Dictionary = DictUtil.dict_of(tier.get("bonus", {}))
* `desc`: String = str(tier.get("desc", ""))
* `row`: Dictionary = {
* `extended`: bool = not is_stat

# Methods

### `static hand_in(entry: Dictionary, family: Dictionary) -> String`
```logic
if ItemFamilies.hands_of(family).is_empty():
	return ""
var hand: String = str(entry.get("hand", ""))
return hand if not hand.is_empty() else ItemFamilies.default_hand(family)
```

### `static raw_numbers(entry: Dictionary, family: Dictionary) -> Dictionary`
```logic
var numbers: Dictionary = ItemFamilies.base_of(family, hand_in(entry, family))
for mod: Dictionary in modifiers_of(entry):
	if str(mod.get("kind", "")) != ADJUST:
		continue
	var field: String = str(mod.get("field", ""))
	if numbers.has(field):
		numbers[field] = DictUtil.to_int(numbers[field]) + DictUtil.to_int(mod.get("delta", 0))
return numbers
```

### `static resolve(entry: Dictionary, family: Dictionary) -> Dictionary`
```logic
var row: Dictionary = entry.duplicate(true)
var type: String = ItemFamilies.item_type_of(family)
var family_id: String = str(family.get("id", ""))
row["family"] = family_id
row["item_type"] = type
row["weapon_type"] = family_id if ItemFamilies.is_gear_type(type) else ""
var hand: String = hand_in(entry, family)
row["hand"] = hand
if not ItemFamilies.hands_of(family).is_empty():
	row["slot"] = "OFF_HAND" if hand == ItemFamilies.HAND_OFF else "MAIN_HAND"
else:
	var slots: Array[String] = ContentFactory._strings(family.get("slots", []))
	if str(entry.get("slot", "")).is_empty() and not slots.is_empty():
		row["slot"] = slots[0]
row["two_handed"] = hand == ItemFamilies.HAND_MAIN_2H
row["off_hand_category"] = ItemFamilies.off_hand_category(family)
if type == "WEAPON":
	var scaling: Array[String] = ContentFactory._strings(family.get("scaling_stats", []))
	if str(entry.get("scaling_stat", "")).is_empty():
		row["scaling_stat"] = scaling[0] if not scaling.is_empty() else "STR"
else:
	row.erase("scaling_stat")
var numbers: Dictionary = raw_numbers(entry, family)
for field: String in ItemFamilies.NUMBER_FIELDS:
	row[str(ItemFamilies.FLAT_KEYS[field])] = DictUtil.to_int(numbers[field])
var reqs: Dictionary = {}
var bonuses: Dictionary = {}
var effects: Array = []
for mod: Dictionary in modifiers_of(entry):
	match str(mod.get("kind", "")):
		REQUIREMENT:
			reqs[str(mod.get("stat", ""))] = DictUtil.to_int(mod.get("value", 0))
		STAT_BONUS:
			bonuses[str(mod.get("stat", ""))] = DictUtil.to_int(mod.get("value", 0))
		EFFECT:
			effects.append(DictUtil.dict_of(mod.get("effect", {})).duplicate(true))
row["required_stats"] = reqs
row["stat_bonuses"] = bonuses
row["effects"] = effects
row["threshold_tiers"] = legacy_tiers(tiers_of(entry), type)
if str(entry.get("damage_type", "")).is_empty():
	row["damage_type"] = str(family.get("damage_type", ""))
return row
```

### `static flat_hand(row: Dictionary) -> String`
```logic
var type: String = str(row.get("item_type", "WEAPON"))
if type == "SHIELD":
	return ItemFamilies.HAND_OFF
if type != "WEAPON":
	return ""
if DictUtil.to_bool(row.get("two_handed", false)):
	return ItemFamilies.HAND_MAIN_2H
if not str(row.get("off_hand_category", "")).is_empty():
	return ItemFamilies.HAND_OFF
return ItemFamilies.HAND_MAIN_1H
```

### `static collapse(row: Dictionary, family: Dictionary) -> Dictionary`
```logic
var out: Dictionary = {}
for key: Variant in row:
	if not DERIVED_KEYS.has(str(key)):
		out[key] = ContentSchema._copy(row[key])
var type: String = ItemFamilies.item_type_of(family)
out["family"] = str(family.get("id", ""))
var hand: String = ""
if not ItemFamilies.hands_of(family).is_empty():
	hand = flat_hand(row)
	if ItemFamilies.category(family) == "main" and hand == ItemFamilies.HAND_OFF:
		hand = ItemFamilies.HAND_MAIN_1H
	if hand.is_empty():
		hand = ItemFamilies.default_hand(family)
	out.erase("slot")
out["hand"] = hand
if type != "WEAPON":
	out.erase("scaling_stat")
var mods: Array = []
var base: Dictionary = ItemFamilies.base_of(family, hand)
for field: String in ItemFamilies.number_fields_for(type):
	var have: int = DictUtil.to_int(row.get(str(ItemFamilies.FLAT_KEYS[field]), base[field]), DictUtil.to_int(base[field]))
	var delta: int = have - DictUtil.to_int(base[field])
	if delta != 0:
		mods.append({"kind": ADJUST, "field": field, "delta": delta})
if ItemFamilies.MODIFIABLE_TYPES.has(type):
	var reqs: Dictionary = DictUtil.dict_of(row.get("required_stats", {}))
	for stat: Variant in reqs:
		mods.append({"kind": REQUIREMENT, "stat": str(stat), "value": DictUtil.to_int(reqs[stat])})
	var bonuses: Dictionary = DictUtil.dict_of(row.get("stat_bonuses", {}))
	for stat: Variant in bonuses:
		mods.append({"kind": STAT_BONUS, "stat": str(stat), "value": DictUtil.to_int(bonuses[stat])})
	for fx: Variant in DictUtil.array_of(row.get("effects", [])):
		mods.append({"kind": EFFECT, "effect": ContentSchema._copy(DictUtil.dict_of(fx))})
out["modifiers"] = mods
out["tiers"] = tiers_for_row(row, type)
if str(row.get("damage_type", "")) == str(family.get("damage_type", "")):
	out["damage_type"] = ""
return out
```

### `static tag_of(mod: Dictionary) -> String`
```logic
var kind: String = str(mod.get("kind", ""))
var spec: Dictionary = DictUtil.dict_of(KINDS.get(kind, {}))
var key: String = str(spec.get("unique", ""))
return "%s:%s" % [kind, JSON.stringify(sorted_value(mod.get(key, null)))]
```

### `static stable_order(fresh: Array[Dictionary], previous: Array[Dictionary]) -> Array`
```logic
var out: Array = []
var used: Dictionary = {}
for old_mod: Dictionary in previous:
	var tag: String = tag_of(old_mod)
	for i: int in range(fresh.size()):
		if not used.has(i) and tag_of(fresh[i]) == tag:
			out.append(fresh[i])
			used[i] = true
			break
for i: int in range(fresh.size()):
	if not used.has(i):
		out.append(fresh[i])
return out
```

### `static stored(row: Dictionary) -> Dictionary`
```logic
var out: Dictionary = {}
var has_hand: bool = not str(row.get("hand", "")).is_empty()
for key: Variant in row:
	var k: String = str(key)
	if DERIVED_KEYS.has(k) or (k == "slot" and has_hand) or (k == "hand" and not has_hand):
		continue
	out[key] = ContentSchema._copy(row[key])
return out
```

### `static is_stored_shape(entry: Dictionary) -> bool`
```logic
if str(entry.get("family", "")).is_empty():
	return false
for key: String in FLAT_MARKERS:
	if entry.has(key):
		return false
return true
```

### `static engine_equal(a: Dictionary, b: Dictionary) -> bool`
```logic
return engine_diff(a, b).is_empty()
```

### `static same_value(a: Variant, b: Variant) -> bool`
```logic
if (a is int or a is float) and (b is int or b is float):
	return is_equal_approx(DictUtil.to_float(a), DictUtil.to_float(b))
return JSON.stringify(sorted_value(a)) == JSON.stringify(sorted_value(b))
```

### `static sorted_value(v: Variant) -> Variant`
```logic
if v is Dictionary:
	var src: Dictionary = v
	var keys: Array = src.keys()
	keys.sort()
	var out: Dictionary = {}
	for k: Variant in keys:
		out[k] = sorted_value(src[k])
	return out
if v is Array:
	var arr: Array = v
	var out_arr: Array = []
	for x: Variant in arr:
		out_arr.append(sorted_value(x))
	return out_arr
if v is float:
	var x: float = v
	if x == floorf(x):
		return roundi(x)
	return x
return v
```

### `static label_of_desc(desc: String) -> String`
```logic
return desc.get_slice(":", 0).strip_edges() if desc.contains(":") else desc.strip_edges()
```

### `static tiers_from_legacy(legacy: Array) -> Array`
```logic
var out: Array = []
for raw: Variant in legacy:
	var t: Dictionary = DictUtil.dict_of(raw)
	var when: Dictionary = DictUtil.dict_of(t.get("when", {}))
	if when.is_empty():
		when = {"type": WHEN_STAT, "stat": str(t.get("stat", "STR")), "value": DictUtil.to_int(t.get("threshold", 0))}
	var bonus: Dictionary = DictUtil.dict_of(t.get("bonus", {}))
	if bonus.is_empty():
		bonus = {"power": DictUtil.to_int(t.get("bonus_power", 0))}
	var desc: String = str(t.get("desc", ""))
	var tier: Dictionary = {"label": label_of_desc(desc), "when": ContentSchema._copy(when), "bonus": ContentSchema._copy(bonus), "desc": ""}
	if ItemText.tier_desc(tier) != desc:
		tier["desc"] = desc
	out.append(tier)
return out
```

### `static tiers_for_row(row: Dictionary, item_type: String) -> Array`
```logic
var own: Array[Dictionary] = tiers_of(row)
var legacy: Array = DictUtil.array_of(row.get("threshold_tiers", []))
if not own.is_empty() and same_value(legacy_tiers(own, item_type), legacy):
	return DictUtil.array_of(ContentSchema._copy(DictUtil.array_of(row.get("tiers", []))))
return tiers_from_legacy(legacy)
```

