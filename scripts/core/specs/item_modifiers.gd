class_name ItemModifiers
extends RefCounted

# ==============================================================================
# Tactical RPG - Item modifiers (FORGE1_ITEMS section 2): base item = family + an ordered modifier list
# A STORED item (items.json version 2) names its family, hand, scaling stat, modifiers and tiers; the RESOLVED
# flat row (what ContentCatalog.entries / get_entry return, and every engine consumer reads) carries the old flat
# keys computed from the family. resolve() and collapse() are inverses on every engine key (ENGINE_KEYS).
# Pure static functions; no catalog access.
# ==============================================================================

const ADJUST: String = "adjust"
const STAT_BONUS: String = "stat_bonus"
const REQUIREMENT: String = "requirement"
const EFFECT: String = "effect"
# The closed modifier kinds (single source for the validator, the Forge widgets, GearBudget and ItemText).
# params: the keys a modifier of the kind carries besides "kind"; unique: the param that may appear once per item.
const KINDS: Dictionary = {
	"adjust": {"label": "Adjust a number", "params": ["field", "delta"], "unique": "field", "min": -100, "max": 100},
	"stat_bonus": {"label": "Stat bonus", "params": ["stat", "value"], "unique": "stat", "min": 1, "max": 8},
	"requirement": {"label": "Stat requirement", "params": ["stat", "value"], "unique": "stat", "min": 1, "max": 30},
	"effect": {"label": "Item effect", "params": ["effect"], "unique": "effect"},
}

static func kind_names() -> Array[String]:
	var out: Array[String] = []
	for key: Variant in KINDS:
		out.append(str(key))
	return out

# Flat keys a resolved row derives from the family and the modifiers (never stored in a version 2 file). `slot` is
# derived too for families with hands (weapons, shields).
const DERIVED_KEYS: Array[String] = ["item_type", "weapon_type", "two_handed", "off_hand_category", "base_power", "base_range",
	"block_cost", "burden", "mitigation", "required_stats", "stat_bonuses", "effects", "threshold_tiers"]
# Keys whose presence marks a FLAT entry (legacy rows, get_entry copies); a stored entry has none of them.
const FLAT_MARKERS: Array[String] = ["base_power", "base_range", "block_cost", "burden", "mitigation", "required_stats",
	"stat_bonuses", "effects", "threshold_tiers"]
# Every key the engine reads from a resolved row (the round-trip acceptance, matrix section 1 rule 5).
const ENGINE_KEYS: Array[String] = ["item_type", "slot", "weapon_type", "base_power", "base_range", "two_handed", "off_hand_category",
	"scaling_stat", "block_cost", "burden", "mitigation", "required_stats", "stat_bonuses", "effects", "threshold_tiers"]

# ------------------------------------------------------------------------------
# Resolve: stored entry + family -> flat row
# ------------------------------------------------------------------------------

static func modifiers_of(entry: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for raw: Variant in DictUtil.array_of(entry.get("modifiers", [])):
		out.append(DictUtil.dict_of(raw))
	return out

static func tiers_of(entry: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for raw: Variant in DictUtil.array_of(entry.get("tiers", [])):
		out.append(DictUtil.dict_of(raw))
	return out

# The hand of a stored entry inside `family` ("" for families without hands; the family default when unset).
static func hand_in(entry: Dictionary, family: Dictionary) -> String:
	if ItemFamilies.hands_of(family).is_empty():
		return ""
	var hand: String = str(entry.get("hand", ""))
	return hand if not hand.is_empty() else ItemFamilies.default_hand(family)

# The resolved numbers BEFORE any clamping: family base of the item's hand + the adjust deltas (validation reads these).
static func raw_numbers(entry: Dictionary, family: Dictionary) -> Dictionary:
	var numbers: Dictionary = ItemFamilies.base_of(family, hand_in(entry, family))
	for mod: Dictionary in modifiers_of(entry):
		if str(mod.get("kind", "")) != ADJUST:
			continue
		var field: String = str(mod.get("field", ""))
		if numbers.has(field):
			numbers[field] = DictUtil.to_int(numbers[field]) + DictUtil.to_int(mod.get("delta", 0))
	return numbers

# The flat row of a stored entry (not normalised: ContentCatalog normalises it). Unknown keys survive.
static func resolve(entry: Dictionary, family: Dictionary) -> Dictionary:
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

# ------------------------------------------------------------------------------
# Collapse: flat row + family -> stored entry
# ------------------------------------------------------------------------------

# The hand a FLAT row occupies (as Equipment.hand(): two-handed -> main_2h, shield or off-hand category -> off, else main_1h).
static func flat_hand(row: Dictionary) -> String:
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

# The stored entry of a flat row against `family`: numbers become `adjust` deltas from the family base of the item's
# hand, requirements / bonuses / effects one modifier each, legacy threshold tiers become `tiers`.
# Strict hand typing (8.1): an `off` row in a main-category family (the old off-hand shortsword, knuckles, dagger)
# collapses to main_1h; its legacy off-hand category is dropped (the category is derived from the family).
static func collapse(row: Dictionary, family: Dictionary) -> Dictionary:
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

# The identity of a modifier for ordering and uniqueness: kind + its unique param ("adjust:power", "requirement:STR").
static func tag_of(mod: Dictionary) -> String:
	var kind: String = str(mod.get("kind", ""))
	var spec: Dictionary = DictUtil.dict_of(KINDS.get(kind, {}))
	var key: String = str(spec.get("unique", ""))
	return "%s:%s" % [kind, JSON.stringify(sorted_value(mod.get(key, null)))]

# `fresh` modifiers in the order of `previous` (matched by tag); new tags keep their order at the end. Collapsing a
# changed flat row therefore never re-sorts an item's list.
static func stable_order(fresh: Array[Dictionary], previous: Array[Dictionary]) -> Array:
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

# The stored (version 2) form of a resolved row: derived keys left out.
static func stored(row: Dictionary) -> Dictionary:
	var out: Dictionary = {}
	var has_hand: bool = not str(row.get("hand", "")).is_empty()
	for key: Variant in row:
		var k: String = str(key)
		if DERIVED_KEYS.has(k) or (k == "slot" and has_hand) or (k == "hand" and not has_hand):
			continue
		out[key] = ContentSchema._copy(row[key])
	return out

# True when `entry` is a stored (family + modifiers) entry rather than a flat row.
static func is_stored_shape(entry: Dictionary) -> bool:
	if str(entry.get("family", "")).is_empty():
		return false
	for key: String in FLAT_MARKERS:
		if entry.has(key):
			return false
	return true

# True when two (normalised) rows agree on every engine key.
static func engine_equal(a: Dictionary, b: Dictionary) -> bool:
	return engine_diff(a, b).is_empty()

static func engine_diff(a: Dictionary, b: Dictionary) -> Array[String]:
	var out: Array[String] = []
	for key: String in ENGINE_KEYS:
		if not same_value(a.get(key, null), b.get(key, null)):
			out.append(key)
	return out

# Value equality for JSON-shaped data: key order and int/float (1 vs 1.0) never matter.
static func same_value(a: Variant, b: Variant) -> bool:
	if (a is int or a is float) and (b is int or b is float):
		return is_equal_approx(DictUtil.to_float(a), DictUtil.to_float(b))
	return JSON.stringify(sorted_value(a)) == JSON.stringify(sorted_value(b))

# A canonical copy for comparisons and tags: dictionaries with sorted keys, whole floats as ints.
static func sorted_value(v: Variant) -> Variant:
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

# ------------------------------------------------------------------------------
# Tiers (section 4): stored `tiers` <-> resolved legacy `threshold_tiers`
# ------------------------------------------------------------------------------

const WHEN_STAT: String = "stat_at_least"
const WHEN_PROFICIENCY: String = "proficiency_at_least"
const WHEN_LEVEL: String = "unit_level_at_least"
const WHEN_TYPES: Array[String] = [WHEN_STAT, WHEN_PROFICIENCY, WHEN_LEVEL]
const BONUS_KEYS: Array[String] = ["power", "mitigation", "stat_bonuses", "effects"]
const TIER_KEYS: Array[String] = ["label", "when", "bonus", "desc"]

# The tier label of a legacy description: the text before ":" ("Sunder Guard: +4 Power" -> "Sunder Guard").
static func label_of_desc(desc: String) -> String:
	return desc.get_slice(":", 0).strip_edges() if desc.contains(":") else desc.strip_edges()

# The resolved `threshold_tiers` (legacy shape the engine reads: stat, threshold, bonus_power, desc) of stored tiers.
# A tier with bonuses beyond power also carries "when" and "bonus" (read by the tier engine, ITEM1-L commit 2).
# Criteria other than stat_at_least are not resolved yet (commit 2 evaluates them); they stay in `tiers`.
static func legacy_tiers(tiers: Array[Dictionary], item_type: String = "WEAPON") -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for tier: Dictionary in tiers:
		var when: Dictionary = DictUtil.dict_of(tier.get("when", {}))
		if str(when.get("type", "")) != WHEN_STAT:
			continue
		var bonus: Dictionary = DictUtil.dict_of(tier.get("bonus", {}))
		var desc: String = str(tier.get("desc", ""))
		var row: Dictionary = {
			"stat": str(when.get("stat", "STR")),
			"threshold": DictUtil.to_int(when.get("value", 0)),
			"bonus_power": DictUtil.to_int(bonus.get("power", 0)),
			"desc": desc if not desc.is_empty() else ItemText.tier_desc(tier, item_type),
		}
		for key: Variant in bonus:
			if str(key) != "power":
				row["when"] = when.duplicate(true)
				row["bonus"] = bonus.duplicate(true)
				break
		out.append(row)
	return out

# Stored tiers of a legacy threshold tier list ({stat, threshold, bonus_power, desc}; extended rows keep when/bonus).
static func tiers_from_legacy(legacy: Array) -> Array:
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

# The stored tiers of a flat row: its `tiers` when they still resolve to its threshold_tiers, else converted from them.
static func tiers_for_row(row: Dictionary, item_type: String) -> Array:
	var own: Array[Dictionary] = tiers_of(row)
	var legacy: Array = DictUtil.array_of(row.get("threshold_tiers", []))
	if not own.is_empty() and same_value(legacy_tiers(own, item_type), legacy):
		return DictUtil.array_of(ContentSchema._copy(DictUtil.array_of(row.get("tiers", []))))
	return tiers_from_legacy(legacy)
