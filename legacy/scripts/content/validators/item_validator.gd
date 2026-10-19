class_name ItemValidator
extends RefCounted

# Item and item-family validation rules (CLN-9 split; FORGE1_ITEMS 1, 2, 4, 6). ContentCatalog.validate_entry dispatches here.
# Problems are {"field", "index" (1-based list row, 0 = the field), "message"}; the *_errors functions map them to strings.
# Legacy mode (no item families yet, until ITEM1-A): items are flat rows checked by the pre-family rules.
# Tier limits come from ItemModifiers (TIER_* constants), shared with the Forge tier table (ItemFormModel.tier_limits).

static func problem(field: String, index: int, message: String) -> Dictionary:
	return {"field": field, "index": index, "message": message}

static func messages(problems: Array[Dictionary]) -> Array[String]:
	var out: Array[String] = []
	for p: Dictionary in problems:
		out.append(str(p["message"]))
	return out

static func _skills() -> Callable:
	return func(skill_id: String) -> bool: return ContentCatalog.has("skills", skill_id)

# ------------------------------------------------------------------------------
# Items
# ------------------------------------------------------------------------------

# `e` is a resolved (normalised) item row, `family` its family row (empty when unknown), `inherit` = families exist.
static func item_errors(e: Dictionary, family: Dictionary = {}, inherit: bool = false) -> Array[String]:
	return messages(item_problems(e, family, inherit))

static func item_problems(e: Dictionary, family: Dictionary = {}, inherit: bool = false) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var icon: String = str(e.get("icon_id", ""))
	if not icon.is_empty() and not AssetRegistry.has_key("item", icon):
		out.append(problem("icon_id", 0, "Icon '%s' isn't a known item icon key." % icon))
	if not inherit:
		for message: String in _legacy_errors(e):
			out.append(problem("", 0, message))
		return out
	if str(e.get("family", "")).is_empty():
		out.append(problem("family", 0, "Family is required (one of the item families)."))
		return out
	if family.is_empty():
		return out   # the unknown family is reported by the reference check
	var type: String = ItemFamilies.item_type_of(family)
	var family_id: String = str(family.get("id", ""))
	var hands: Array[String] = ItemFamilies.hands_of(family)
	var hand: String = str(e.get("hand", ""))
	if not hands.is_empty() and not hands.has(hand):
		out.append(problem("hand", 0, "Hand '%s' is not allowed by family '%s' (allowed: %s)." % [hand, family_id, ", ".join(hands)]))
	if type == "WEAPON":
		var scaling: Array[String] = ContentFactory._strings(family.get("scaling_stats", []))
		if not scaling.has(str(e.get("scaling_stat", ""))):
			out.append(problem("scaling_stat", 0, "Scales with '%s' is not in family '%s' (allowed: %s)." % [str(e.get("scaling_stat", "")), family_id, ", ".join(scaling)]))
	if ItemFamilies.SLOT_TYPES.has(type):
		var slots: Array[String] = ContentFactory._strings(family.get("slots", []))
		if not slots.has(str(e.get("slot", ""))):
			out.append(problem("slot", 0, "Slot '%s' is not one of family '%s' slots (%s)." % [str(e.get("slot", "")), family_id, ", ".join(slots)]))
	if ItemFamilies.is_gear_type(type) and str(e.get("weapon_type", "")) != family_id:
		out.append(problem("weapon_type", 0, "Gear type '%s' must equal the family id '%s'." % [str(e.get("weapon_type", "")), family_id]))
	out.append_array(modifier_problems(e, family))
	out.append_array(tier_problems(e, family))
	for p: Dictionary in ItemText.validate(str(e.get("description", "")), e):
		out.append(problem("description", 0, str(p["message"])))
	if type == "CONSUMABLE":
		out.append_array(_consumable_problems(e))
	return out

# The modifier list against the family's policy, plus the resolved numbers against the schema ranges.
static func modifier_problems(e: Dictionary, family: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var type: String = ItemFamilies.item_type_of(family)
	var family_id: String = str(family.get("id", ""))
	var mods: Array[Dictionary] = ItemModifiers.modifiers_of(e)
	var raw_list: Array = DictUtil.array_of(e.get("modifiers", []))
	for i: int in range(raw_list.size()):
		if not (raw_list[i] is Dictionary):
			out.append(problem("modifiers", i + 1, "Modifier %d is not an object." % (i + 1)))
	if mods.is_empty():
		return out
	if not ItemFamilies.MODIFIABLE_TYPES.has(type):
		out.append(problem("modifiers", 0, "%s items take no modifiers (family '%s')." % [type, family_id]))
		return out
	var cap: int = DictUtil.to_int(family.get("max_modifiers", 0))
	if mods.size() > cap:
		out.append(problem("modifiers", 0, "At most %d modifiers (family '%s'); this item has %d." % [cap, family_id, mods.size()]))
	var allowed: Array[String] = ContentFactory._strings(family.get("modifier_kinds", []))
	var adjustable: Array[String] = ContentFactory._strings(family.get("adjustable", []))
	var numbers: Array[String] = ItemFamilies.number_fields_for(type)
	var seen: Dictionary = {}
	for i: int in range(mods.size()):
		var mod: Dictionary = mods[i]
		var n: int = i + 1
		var kind: String = str(mod.get("kind", ""))
		if not ItemModifiers.KINDS.has(kind):
			out.append(problem("modifiers", n, "Modifier %d: unknown kind '%s' (one of %s)." % [n, kind, ", ".join(ItemModifiers.kind_names())]))
			continue
		if not allowed.has(kind):
			out.append(problem("modifiers", n, "Modifier %d: kind '%s' is not allowed by family '%s' (allowed: %s)." % [n, kind, family_id, ", ".join(allowed)]))
		var spec: Dictionary = ItemModifiers.KINDS[kind]
		var params: Array = spec["params"]
		for key: Variant in mod:
			if str(key) != "kind" and not params.has(str(key)):
				out.append(problem("modifiers", n, "Modifier %d (%s): unknown key '%s' (keys: %s)." % [n, kind, str(key), ", ".join(PackedStringArray(params))]))
		var unique_tag: String = kind
		match kind:
			ItemModifiers.ADJUST:
				var field: String = str(mod.get("field", ""))
				unique_tag += ":" + field
				if not numbers.has(field):
					out.append(problem("modifiers", n, "Modifier %d: '%s' is not a number of %s items (%s)." % [n, field, type, ", ".join(numbers)]))
				elif not adjustable.has(field):
					out.append(problem("modifiers", n, "Modifier %d: family '%s' does not allow adjusting '%s' (adjustable: %s)." % [n, family_id, field, ", ".join(adjustable)]))
				out.append_array(_int_problems(mod.get("delta", null), spec, n, "delta", true))
			ItemModifiers.STAT_BONUS, ItemModifiers.REQUIREMENT:
				var stat: String = str(mod.get("stat", ""))
				unique_tag += ":" + stat
				if not ContentSchema.STAT_KEYS.has(stat):
					out.append(problem("modifiers", n, "Modifier %d: unknown stat '%s' (one of %s)." % [n, stat, ", ".join(ContentSchema.STAT_KEYS)]))
				out.append_array(_int_problems(mod.get("value", null), spec, n, "value", false))
			ItemModifiers.EFFECT:
				var fx: Variant = mod.get("effect", null)
				unique_tag += ":" + JSON.stringify(ItemModifiers.sorted_value(fx))
				if not (fx is Dictionary):
					out.append(problem("modifiers", n, "Modifier %d: effect needs an effect object ({\"type\": ...})." % n))
				else:
					for message: String in Passives.validate_effects([fx], _skills(), true, true):
						out.append(problem("modifiers", n, "Modifier %d: %s" % [n, message]))
		if seen.has(unique_tag):
			out.append(problem("modifiers", n, "Modifier %d repeats an earlier %s modifier (one per %s)." % [n, kind, str(spec["unique"])]))
		seen[unique_tag] = true
	var raw: Dictionary = ItemModifiers.raw_numbers(e, family)
	var base: Dictionary = ItemFamilies.base_of(family, str(e.get("hand", "")))
	for field: String in numbers:
		var rng: Array = ItemFamilies.NUMBER_RANGES[field]
		var v: int = DictUtil.to_int(raw[field])
		if v < DictUtil.to_int(rng[0]) or v > DictUtil.to_int(rng[1]):
			out.append(problem("modifiers", 0, "%s resolves to %d (family base %d plus adjustments), outside %d-%d." % [field.capitalize(), v, DictUtil.to_int(base[field]), DictUtil.to_int(rng[0]), DictUtil.to_int(rng[1])]))
	return out

static func _int_problems(value: Variant, spec: Dictionary, n: int, key: String, nonzero: bool) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if not (value is int or value is float) or DictUtil.to_float(value) != floorf(DictUtil.to_float(value)):
		out.append(problem("modifiers", n, "Modifier %d: %s must be a whole number." % [n, key]))
		return out
	var v: int = DictUtil.to_int(value)
	if v < DictUtil.to_int(spec["min"]) or v > DictUtil.to_int(spec["max"]):
		out.append(problem("modifiers", n, "Modifier %d: %s %d is outside %d to %d." % [n, key, v, DictUtil.to_int(spec["min"]), DictUtil.to_int(spec["max"])]))
	if nonzero and v == 0:
		out.append(problem("modifiers", n, "Modifier %d: a %s of 0 changes nothing; remove it." % [n, key]))
	return out

# Tier thresholds (section 4): count, closed criteria and bonus lists, ranges, unique criteria, ascending values.
static func tier_problems(e: Dictionary, family: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var type: String = ItemFamilies.item_type_of(family)
	var tiers: Array[Dictionary] = ItemModifiers.tiers_of(e)
	var raw_list: Array = DictUtil.array_of(e.get("tiers", []))
	for i: int in range(raw_list.size()):
		if not (raw_list[i] is Dictionary):
			out.append(problem("tiers", i + 1, "Tier %d is not an object." % (i + 1)))
	if tiers.is_empty():
		return out
	var cap: int = DictUtil.to_int(family.get("max_tiers", 0))
	if tiers.size() > cap:
		out.append(problem("tiers", 0, "At most %d tiers (family '%s'); this item has %d." % [cap, str(family.get("id", "")), tiers.size()]))
	var seen: Dictionary = {}
	var last: Dictionary = {}   # criterion group -> last value
	for i: int in range(tiers.size()):
		var t: Dictionary = tiers[i]
		var n: int = i + 1
		for key: Variant in t:
			if not ItemModifiers.TIER_KEYS.has(str(key)):
				out.append(problem("tiers", n, "Tier %d: unknown key '%s' (keys: %s)." % [n, str(key), ", ".join(ItemModifiers.TIER_KEYS)]))
		if str(t.get("label", "")).length() > ItemModifiers.TIER_LABEL_MAX:
			out.append(problem("tiers", n, "Tier %d: the label is %d characters (max %d)." % [n, str(t.get("label", "")).length(), ItemModifiers.TIER_LABEL_MAX]))
		if str(t.get("desc", "")).length() > ItemModifiers.TIER_DESC_MAX:
			out.append(problem("tiers", n, "Tier %d: the description is %d characters (max %d)." % [n, str(t.get("desc", "")).length(), ItemModifiers.TIER_DESC_MAX]))
		var when: Dictionary = DictUtil.dict_of(t.get("when", {}))
		var group: String = ""
		var value: int = 0
		match str(when.get("type", "")):
			ItemModifiers.WHEN_STAT:
				var stat: String = str(when.get("stat", ""))
				if not ContentSchema.STAT_KEYS.has(stat):
					out.append(problem("tiers", n, "Tier %d: unknown stat '%s'." % [n, stat]))
				value = DictUtil.to_int(when.get("value", 0))
				out.append_array(_range_problem(value, ItemModifiers.TIER_STAT_MIN, ItemModifiers.TIER_STAT_MAX, n, "stat value"))
				out.append_array(_unknown_keys(when, ["type", "stat", "value"], n, "criterion"))
				group = "stat:" + stat
			ItemModifiers.WHEN_PROFICIENCY:
				var gear: String = str(when.get("gear", str(family.get("id", ""))))
				if not Proficiency.gear_types().has(gear):
					out.append(problem("tiers", n, "Tier %d: '%s' is not a gear type (%s)." % [n, gear, ", ".join(Proficiency.gear_types())]))
				value = DictUtil.to_int(when.get("level", 0))
				out.append_array(_range_problem(value, 1, Proficiency.MAX_LEVEL, n, "proficiency level"))
				out.append_array(_unknown_keys(when, ["type", "gear", "level"], n, "criterion"))
				group = "proficiency:" + gear
			ItemModifiers.WHEN_LEVEL:
				value = DictUtil.to_int(when.get("value", 0))
				out.append_array(_range_problem(value, 1, StatRules.MAX_LEVEL, n, "unit level"))
				out.append_array(_unknown_keys(when, ["type", "value"], n, "criterion"))
				group = "level"
			_:
				out.append(problem("tiers", n, "Tier %d: criterion type '%s' is not one of %s." % [n, str(when.get("type", "")), ", ".join(ItemModifiers.WHEN_TYPES)]))
		var tag: String = JSON.stringify(ItemModifiers.sorted_value(when))
		if seen.has(tag):
			out.append(problem("tiers", n, "Tier %d repeats the criterion of an earlier tier." % n))
		seen[tag] = true
		if not group.is_empty():
			if last.has(group) and value <= DictUtil.to_int(last[group]):
				out.append(problem("tiers", n, "Tier %d: tiers on the same criterion go in ascending order (%d after %d)." % [n, value, DictUtil.to_int(last[group])]))
			last[group] = value
		out.append_array(_bonus_problems(DictUtil.dict_of(t.get("bonus", {})), type, n))
	return out

static func _bonus_problems(bonus: Dictionary, type: String, n: int) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var any: bool = false
	var ranges: Dictionary = ItemModifiers.TIER_BONUS_RANGES
	for key: Variant in bonus:
		var k: String = str(key)
		match k:
			"power":
				if type != "WEAPON":
					out.append(problem("tiers", n, "Tier %d: a power bonus is for weapons only." % n))
				var p: int = DictUtil.to_int(bonus[key])
				var p_rng: Array = ranges["power"]
				out.append_array(_range_problem(p, DictUtil.to_int(p_rng[0]), DictUtil.to_int(p_rng[1]), n, "power bonus"))
				any = any or p > 0
			"mitigation":
				if type != "ARMOR" and type != "SHIELD":
					out.append(problem("tiers", n, "Tier %d: a mitigation bonus is for armor and shields only." % n))
				var m: int = DictUtil.to_int(bonus[key])
				var m_rng: Array = ranges["mitigation"]
				out.append_array(_range_problem(m, DictUtil.to_int(m_rng[0]), DictUtil.to_int(m_rng[1]), n, "mitigation bonus"))
				any = any or m > 0
			"stat_bonuses":
				var stats: Dictionary = DictUtil.dict_of(bonus[key])
				if stats.size() > ItemModifiers.TIER_MAX_STATS:
					out.append(problem("tiers", n, "Tier %d: at most %d stat bonuses." % [n, ItemModifiers.TIER_MAX_STATS]))
				var s_rng: Array = ranges["stat_bonuses"]
				for stat: Variant in stats:
					if not ContentSchema.STAT_KEYS.has(str(stat)):
						out.append(problem("tiers", n, "Tier %d: unknown stat '%s'." % [n, str(stat)]))
					out.append_array(_range_problem(DictUtil.to_int(stats[stat]), DictUtil.to_int(s_rng[0]), DictUtil.to_int(s_rng[1]), n, "%s bonus" % str(stat)))
				any = any or not stats.is_empty()
			"effects":
				var fx: Array = DictUtil.array_of(bonus[key])
				if fx.size() > ItemModifiers.TIER_MAX_EFFECTS:
					out.append(problem("tiers", n, "Tier %d: at most %d effects." % [n, ItemModifiers.TIER_MAX_EFFECTS]))
				for message: String in Passives.validate_effects(fx, _skills(), true, true):
					out.append(problem("tiers", n, "Tier %d: %s" % [n, message]))
				any = any or not fx.is_empty()
			_:
				out.append(problem("tiers", n, "Tier %d: unknown bonus '%s' (one of %s)." % [n, k, ", ".join(ItemModifiers.BONUS_KEYS)]))
	if not any:
		out.append(problem("tiers", n, "Tier %d needs at least one non-zero bonus." % n))
	return out

static func _range_problem(v: int, lo: int, hi: int, n: int, what: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if v < lo or v > hi:
		out.append(problem("tiers", n, "Tier %d: %s %d is outside %d-%d." % [n, what, v, lo, hi]))
	return out

static func _unknown_keys(d: Dictionary, allowed: Array[String], n: int, what: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	for key: Variant in d:
		if not allowed.has(str(key)):
			out.append(problem("tiers", n, "Tier %d: unknown %s key '%s'." % [n, what, str(key)]))
	return out

static func _consumable_problems(e: Dictionary) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	if DictUtil.to_int(e["heal"]) == 0 and DictUtil.to_int(e["damage"]) == 0 and DictUtil.to_float(e["buff_pct"]) == 0.0:
		out.append(problem("consumable_type", 0, "A consumable should heal, damage or buff."))
	return out

# Soft warnings of a resolved item: numbers outside the family bands, the bonus value band, literal numbers in the text.
static func item_warnings(e: Dictionary, family: Dictionary) -> Array[String]:
	var out: Array[String] = []
	out.append_array(ItemText.warnings(str(e.get("description", "")), e))
	if str(e.get("item_type", "")) == "CONSUMABLE":
		var con_type: String = str(e.get("consumable_type", ""))
		var type_index: int = Consumable.ConsumableType.keys().find(con_type)
		if type_index >= 0 and not Consumable.offered_in_battle(type_index as Consumable.ConsumableType):
			out.append("Consumable type %s is not offered in battle (no battle use yet)." % con_type)
	if family.is_empty() or GearBudget.is_special(e):
		return out   # special items may break the budget rules by design (their note is GearBudget.check_item's warning)
	var hand: String = str(e.get("hand", ""))
	for field: String in ItemFamilies.number_fields_for(ItemFamilies.item_type_of(family)):
		var band: Dictionary = ItemFamilies.band_of(family, hand, field)
		if band.is_empty():
			continue
		var v: int = DictUtil.to_int(e.get(str(ItemFamilies.FLAT_KEYS[field]), 0))
		if v < DictUtil.to_int(band.get("min", v)) or v > DictUtil.to_int(band.get("max", v)):
			out.append("%s %d is outside the family band %d-%d (soft until the balance pass)." % [field.capitalize(), v, DictUtil.to_int(band["min"]), DictUtil.to_int(band["max"])])
	var bonus_band: Dictionary = ItemFamilies.band_of(family, hand, ItemFamilies.BONUS_BAND)
	if not bonus_band.is_empty():
		var value: float = bonus_value(e)
		if value < DictUtil.to_float(bonus_band.get("min", value)) - 0.0001 or value > DictUtil.to_float(bonus_band.get("max", value)) + 0.0001:
			out.append("Bonus value %.1f (stat bonuses and effects) is outside the family band %s-%s (soft)." % [value, str(bonus_band["min"]), str(bonus_band["max"])])
	return out

# GearBudget value of the non-power modifiers (stat_bonus + effect), the quantity the `bonus_value` band limits.
static func bonus_value(e: Dictionary) -> float:
	var total: float = 0.0
	for mod: Dictionary in ItemModifiers.modifiers_of(e):
		var kind: String = str(mod.get("kind", ""))
		if kind == ItemModifiers.STAT_BONUS or kind == ItemModifiers.EFFECT:
			total += GearBudget.modifier_value(mod)
	return total

# Pre-family rules of a flat item row (legacy mode).
static func _legacy_errors(e: Dictionary) -> Array[String]:
	var errors: Array[String] = []
	var type: String = str(e["item_type"])
	var slot: String = str(e["slot"])
	if type != "WEAPON" and Equipment.FOCUS_TYPES.has(str(e["weapon_type"])) and ContentSchema.EQUIP_TYPES.has(type):
		errors.append("Orb and tome are weapon types (item type WEAPON).")
	if Equipment.FOCUS_TYPES.has(str(e["off_hand_category"])) and str(e["weapon_type"]) != str(e["off_hand_category"]):
		errors.append("Off-hand category '%s' needs the matching gear type '%s'." % [str(e["off_hand_category"]), str(e["off_hand_category"])])
	if ContentSchema.EQUIP_TYPES.has(type):
		errors.append_array(Passives.validate_effects(e["effects"], _skills(), true, true))
		# Threshold tiers apply to every gear type (armor and shield tiers too; the commit 2 tier engine).
		var tiers: Variant = e["threshold_tiers"]
		if not (tiers is Array):
			errors.append("Threshold tiers must be a list.")
		else:
			var tier_list: Array = tiers
			for t: Variant in tier_list:
				var tier: Dictionary = t if t is Dictionary else {}
				if not tier.has("stat") or not tier.has("threshold"):
					errors.append("Each threshold tier needs \"stat\" and \"threshold\".")
					break
	match type:
		"WEAPON":
			if slot != "MAIN_HAND" and slot != "OFF_HAND":
				errors.append("Weapons go in MAIN_HAND or OFF_HAND.")
			if DictUtil.to_bool(e["two_handed"]) and slot != "MAIN_HAND":
				errors.append("Two-handed weapons use the MAIN_HAND slot.")
			if DictUtil.to_bool(e["two_handed"]) and not str(e["off_hand_category"]).is_empty():
				errors.append("Hand conflict: a two-handed weapon (main_2h) can't also be an off-hand category weapon (off).")
			if Equipment.FOCUS_TYPES.has(str(e["weapon_type"])):
				if slot != "OFF_HAND":
					errors.append("Orb and tome weapons are off-hand only (use the OFF_HAND slot).")
				if str(e["off_hand_category"]) != str(e["weapon_type"]):
					errors.append("An orb or tome needs off-hand category '%s' (matching its gear type)." % str(e["weapon_type"]))
				if DictUtil.to_bool(e["two_handed"]):
					errors.append("Orb and tome weapons are never two-handed.")
		"SHIELD":
			if slot != "OFF_HAND":
				errors.append("Shields go in the OFF_HAND slot.")
		"ARMOR":
			if slot != "HEAD" and slot != "BODY":
				errors.append("Armor goes in HEAD or BODY.")
		"ACCESSORY":
			if slot != "ACCESSORY_1" and slot != "ACCESSORY_2":
				errors.append("Accessories go in ACCESSORY_1 or ACCESSORY_2.")
		"CONSUMABLE":
			errors.append_array(messages(_consumable_problems(e)))
	return errors

# ------------------------------------------------------------------------------
# Families
# ------------------------------------------------------------------------------

static func family_errors(f: Dictionary, current_id: String = "") -> Array[String]:
	return messages(family_problems(f, current_id))

# Problems of a family row (derived fields filled). Also re-validates every item of the family against it: a change
# that breaks an item (type, hands, a base number pushing a resolved value out of range) is refused with the item ids.
static func family_problems(f: Dictionary, current_id: String = "") -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var type: String = ItemFamilies.item_type_of(f)
	var id: String = str(f.get("id", ""))
	if ItemFamilies.is_gear_type(type) and not Proficiency.gear_types().has(id):
		out.append(problem("id", 0, "A %s family is named after its gear token (one of %s)." % [type, ", ".join(Proficiency.gear_types())]))
	var hands: Array[String] = ItemFamilies.hands_of(f)
	if ItemFamilies.HAND_TYPES.has(type):
		var seen: Dictionary = {}
		for h: String in hands:
			if not ItemFamilies.HANDS.has(h):
				out.append(problem("hands", 0, "Unknown hand '%s' (one of %s)." % [h, ", ".join(ItemFamilies.HANDS)]))
			if seen.has(h):
				out.append(problem("hands", 0, "Hands list '%s' twice." % h))
			seen[h] = true
		if hands.is_empty():
			out.append(problem("hands", 0, "%s families need at least one hand." % type))
		elif ItemFamilies.category(f).is_empty():
			out.append(problem("hands", 0, "One hand category per family: main hands (main_1h, main_2h; damage weapons) or exactly off (support items); mixing is an error."))
		if type == "SHIELD" and ItemFamilies.category(f) != "off":
			out.append(problem("hands", 0, "Shields are off-hand only (hands = off)."))
		if ItemFamilies.category(f) == "off" and not ItemFamilies.OFF_CATEGORIES.has(id):
			out.append(problem("hands", 0, "Off-hand families are %s (their id is the off-hand category)." % ", ".join(ItemFamilies.OFF_CATEGORIES)))
	if type == "WEAPON":
		var scaling: Array[String] = ContentFactory._strings(f.get("scaling_stats", []))
		if scaling.is_empty():
			out.append(problem("scaling_stats", 0, "Weapon families need at least one scaling stat."))
		for s: String in scaling:
			if not ItemFamilies.SCALING_STATS.has(s):
				out.append(problem("scaling_stats", 0, "Weapons scale from %s only ('%s')." % [", ".join(ItemFamilies.SCALING_STATS), s]))
	if ItemFamilies.SLOT_TYPES.has(type):
		var allowed_slots: Array[String] = ItemFamilies.ARMOR_SLOTS if type == "ARMOR" else ItemFamilies.ACCESSORY_SLOTS
		var slots: Array[String] = ContentFactory._strings(f.get("slots", []))
		if slots.is_empty():
			out.append(problem("slots", 0, "%s families list their slots (%s)." % [type, ", ".join(allowed_slots)]))
		for s: String in slots:
			if not allowed_slots.has(s):
				out.append(problem("slots", 0, "Slot '%s' is not a %s slot (%s)." % [s, type, ", ".join(allowed_slots)]))
	out.append_array(_base_problems(f, type))
	var numbers: Array[String] = ItemFamilies.number_fields_for(type)
	for a: String in ContentFactory._strings(f.get("adjustable", [])):
		if not numbers.has(a):
			out.append(problem("adjustable", 0, "'%s' is not a number of %s items (%s)." % [a, type, ", ".join(numbers)]))
	var kinds: Array[String] = ContentFactory._strings(f.get("modifier_kinds", []))
	for k: String in kinds:
		if not ItemModifiers.KINDS.has(k):
			out.append(problem("modifier_kinds", 0, "Unknown modifier kind '%s' (one of %s)." % [k, ", ".join(ItemModifiers.kind_names())]))
	if not kinds.is_empty() and not ItemFamilies.MODIFIABLE_TYPES.has(type):
		out.append(problem("modifier_kinds", 0, "%s families take no modifiers." % type))
	for rule: String in ContentFactory._strings(f.get("engine_rules", [])):
		if not ItemFamilies.ENGINE_RULES.has(rule):
			out.append(problem("engine_rules", 0, "Unknown engine rule '%s' (closed list: %s)." % [rule, ", ".join(ItemFamilies.ENGINE_RULES)]))
		elif rule == ItemFamilies.RULE_FOCUS and (type != "WEAPON" or ItemFamilies.category(f) != "off"):
			out.append(problem("engine_rules", 0, "focus needs a WEAPON family with hands = off."))
		elif rule == ItemFamilies.RULE_SHIELD and type != "SHIELD":
			out.append(problem("engine_rules", 0, "shield needs a SHIELD family."))
		elif (rule == ItemFamilies.RULE_ALWAYS_RANGED or rule == ItemFamilies.RULE_SPELL_SOURCE) and type != "WEAPON":
			out.append(problem("engine_rules", 0, "%s needs a WEAPON family." % rule))
	var old_id: String = current_id if not current_id.is_empty() else id
	var users: Array[Dictionary] = ContentCatalog.items_of_family(old_id)
	if old_id != id and not users.is_empty():
		out.append(problem("id", 0, "The id of family '%s' is read-only while items use it (%d items)." % [old_id, users.size()]))
	var failing: Array[String] = []
	for item: Dictionary in users:
		var row: Dictionary = ContentSchema.normalize("items", ItemModifiers.resolve(ItemModifiers.stored(item), f))
		var errors: Array[String] = item_errors(row, f, true)
		if not errors.is_empty():
			failing.append("%s (%s)" % [str(item.get("id", "")), errors[0]])
	if not failing.is_empty():
		out.append(problem("items", 0, "This family change breaks %d item(s): %s." % [failing.size(), "; ".join(failing)]))
	return out

# `base` and `bands`: one row per hand (key "none" without hands), the five numbers inside the schema ranges.
static func _base_problems(f: Dictionary, type: String) -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var keys: Array[String] = ItemFamilies.hands_of(f)
	if keys.is_empty():
		keys = [ItemFamilies.NO_HAND_KEY]
	var base: Dictionary = DictUtil.dict_of(f.get("base", {}))
	if not (f.get("base", {}) is Dictionary):
		out.append(problem("base", 0, "Base numbers are an object keyed by hand."))
	for key: Variant in base:
		if not keys.has(str(key)):
			out.append(problem("base", 0, "Base row '%s' is not one of the family's hands (%s)." % [str(key), ", ".join(keys)]))
	var needs_base: bool = ItemFamilies.MODIFIABLE_TYPES.has(type)
	for key: String in keys:
		if not base.has(key):
			if needs_base:
				out.append(problem("base", 0, "Base numbers for '%s' are missing." % key))
			continue
		var row: Dictionary = DictUtil.dict_of(base[key])
		for field: Variant in row:
			if not ItemFamilies.NUMBER_FIELDS.has(str(field)):
				out.append(problem("base", 0, "Base '%s': unknown number '%s' (%s)." % [key, str(field), ", ".join(ItemFamilies.NUMBER_FIELDS)]))
				continue
			var rng: Array = ItemFamilies.NUMBER_RANGES[str(field)]
			var v: int = DictUtil.to_int(row[field])
			if v < DictUtil.to_int(rng[0]) or v > DictUtil.to_int(rng[1]):
				out.append(problem("base", 0, "Base '%s' %s %d is outside %d-%d." % [key, str(field), v, DictUtil.to_int(rng[0]), DictUtil.to_int(rng[1])]))
	var bands: Dictionary = DictUtil.dict_of(f.get("bands", {}))
	for key: Variant in bands:
		if not keys.has(str(key)):
			out.append(problem("bands", 0, "Band row '%s' is not one of the family's hands (%s)." % [str(key), ", ".join(keys)]))
			continue
		var row: Dictionary = DictUtil.dict_of(bands[key])
		for field: Variant in row:
			var name: String = str(field)
			var band: Dictionary = DictUtil.dict_of(row[field])
			var lo: float = DictUtil.to_float(band.get("min", 0))
			var hi: float = DictUtil.to_float(band.get("max", 0))
			if not band.has("min") or not band.has("max"):
				out.append(problem("bands", 0, "Band '%s' %s needs min and max." % [str(key), name]))
			elif lo > hi:
				out.append(problem("bands", 0, "Band '%s' %s: min %s is above max %s." % [str(key), name, str(band["min"]), str(band["max"])]))
			if name == ItemFamilies.BONUS_BAND:
				if lo < 0.0:
					out.append(problem("bands", 0, "Band '%s' bonus_value starts at 0 or above." % str(key)))
			elif not ItemFamilies.NUMBER_FIELDS.has(name):
				out.append(problem("bands", 0, "Band '%s': unknown number '%s'." % [str(key), name]))
			else:
				var rng: Array = ItemFamilies.NUMBER_RANGES[name]
				if lo < DictUtil.to_float(rng[0]) or hi > DictUtil.to_float(rng[1]):
					out.append(problem("bands", 0, "Band '%s' %s %s-%s is outside the schema range %d-%d." % [str(key), name, str(band.get("min", "")), str(band.get("max", "")), DictUtil.to_int(rng[0]), DictUtil.to_int(rng[1])]))
	return out

# Soft warnings of a family: base numbers outside its own bands, a 2H base below the 1H base, a focus power band above 10.
static func family_warnings(f: Dictionary) -> Array[String]:
	var out: Array[String] = []
	var hands: Array[String] = ItemFamilies.hands_of(f)
	var keys: Array[String] = hands.duplicate()
	if keys.is_empty():
		keys.append(ItemFamilies.NO_HAND_KEY)
	for key: String in keys:
		var base: Dictionary = ItemFamilies.base_of(f, key)
		for field: String in ItemFamilies.number_fields_for(ItemFamilies.item_type_of(f)):
			var band: Dictionary = ItemFamilies.band_of(f, key, field)
			if band.is_empty():
				continue
			var v: int = DictUtil.to_int(base[field])
			if v < DictUtil.to_int(band.get("min", v)) or v > DictUtil.to_int(band.get("max", v)):
				out.append("Base '%s' %s %d is outside its own band %s-%s." % [key, field, v, str(band["min"]), str(band["max"])])
	if hands.has(ItemFamilies.HAND_MAIN_1H) and hands.has(ItemFamilies.HAND_MAIN_2H):
		var one: int = DictUtil.to_int(ItemFamilies.base_of(f, ItemFamilies.HAND_MAIN_1H)["power"])
		var two: int = DictUtil.to_int(ItemFamilies.base_of(f, ItemFamilies.HAND_MAIN_2H)["power"])
		if two < one:
			out.append("The main_2h base power %d is below the main_1h base power %d." % [two, one])
	if ItemFamilies.has_rule(f, ItemFamilies.RULE_FOCUS):
		for key: String in keys:
			var band: Dictionary = ItemFamilies.band_of(f, key, "power")
			if not band.is_empty() and DictUtil.to_int(band.get("max", 0)) > 10:
				out.append("A focus weapon is a support item: its power band max %s should stay at 10 or below." % str(band["max"]))
	return out

# Catalog-wide family rules (validate_all; empty in legacy mode): every gear token has its family.
static func family_set_errors(families: Array[Dictionary]) -> Array[String]:
	var errors: Array[String] = []
	if families.is_empty():
		return errors
	var ids: Array[String] = []
	for f: Dictionary in families:
		ids.append(str(f.get("id", "")))
	for token: String in Proficiency.gear_types():
		if not ids.has(token):
			errors.append("Gear token '%s' has no item family (every Proficiency gear type needs one)." % token)
	return errors
