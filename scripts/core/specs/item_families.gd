class_name ItemFamilies
extends RefCounted

# ==============================================================================
# Tactical RPG - Item families (FORGE1_ITEMS section 1): the TEMPLATE of a base item
# A family fixes item type, hand category, slots, scaling stats, base numbers per hand, soft bands,
# the modifier policy and the closed engine rules; a base item is family + modifiers (ItemModifiers).
# Pure static helpers over family Dictionaries (rows of the `item_families` catalog kind).
# ==============================================================================

const HAND_MAIN_1H: String = "main_1h"
const HAND_MAIN_2H: String = "main_2h"
const HAND_OFF: String = "off"
const HANDS: Array[String] = [HAND_MAIN_1H, HAND_MAIN_2H, HAND_OFF]
const MAIN_HANDS: Array[String] = [HAND_MAIN_1H, HAND_MAIN_2H]
const NO_HAND_KEY: String = "none"   # `base` / `bands` key of families without hands (armor, accessories, ...)

const ITEM_TYPES: Array[String] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY", "CONSUMABLE", "MATERIAL", "QUEST"]
const GEAR_TYPES: Array[String] = ["WEAPON", "SHIELD", "ARMOR"]   # family id = a Proficiency.gear_types() token
const MODIFIABLE_TYPES: Array[String] = ["WEAPON", "SHIELD", "ARMOR", "ACCESSORY"]
# Off-hand (support) families; their id is the derived `off_hand_category` of their items.
const OFF_CATEGORIES: Array[String] = ["shield", "orb", "tome"]

const ARMOR_SLOTS: Array[String] = ["HEAD", "BODY"]
const ACCESSORY_SLOTS: Array[String] = ["ACCESSORY_1", "ACCESSORY_2"]
const SCALING_STATS: Array[String] = ["STR", "DEX", "INT"]   # weapons never scale from KNOW

# The five base numbers (`base[hand]`) and their schema ranges (ItemFields: base_power, base_range, block_cost, burden, mitigation).
const NUMBER_FIELDS: Array[String] = ["power", "range", "block_cost", "burden", "mitigation"]
const NUMBER_RANGES: Dictionary = {"power": [0, 200], "range": [1, 6], "block_cost": [1, 6], "burden": [0, 10], "mitigation": [0, 100]}
const FLAT_KEYS: Dictionary = {"power": "base_power", "range": "base_range", "block_cost": "block_cost", "burden": "burden", "mitigation": "mitigation"}
const BONUS_BAND: String = "bonus_value"   # band of the GearBudget value of the stat_bonus + effect modifiers

# Closed engine rules (code-owned, section 1.3). The code constants they mirror stay the engine's source in commit 1:
# focus = Equipment.FOCUS_TYPES, always_ranged = Equipment.ALWAYS_RANGED_TYPES, shield = item type SHIELD,
# spell_source = SkillDef.SPELL_SOURCE_TYPES (item_families_test_suite asserts the family lists equal them).
const RULE_FOCUS: String = "focus"
const RULE_ALWAYS_RANGED: String = "always_ranged"
const RULE_SHIELD: String = "shield"
const RULE_SPELL_SOURCE: String = "spell_source"
const ENGINE_RULES: Array[String] = [RULE_FOCUS, RULE_ALWAYS_RANGED, RULE_SHIELD, RULE_SPELL_SOURCE]
const RULE_HELP: Dictionary = {
	"focus": "Off-hand only, never two-handed; a two-handed main refuses it and equipping one stows it.",
	"always_ranged": "Ranged even at range 1.",
	"shield": "Counts as a shield (blocks two-handed pairing and dual wield).",
	"spell_source": "A main-hand type that opens spell skills.",
}

static func item_type_of(family: Dictionary) -> String:
	return str(family.get("item_type", "WEAPON"))

static func is_gear_type(item_type: String) -> bool:
	return GEAR_TYPES.has(item_type)

const HAND_TYPES: Array[String] = ["WEAPON", "SHIELD"]   # types with hands (slot derived from the hand)
const SLOT_TYPES: Array[String] = ["ARMOR", "ACCESSORY"]   # types that list their slots

# The hands of a family (empty for types without hands, whatever the row holds).
static func hands_of(family: Dictionary) -> Array[String]:
	if not HAND_TYPES.has(item_type_of(family)):
		var none: Array[String] = []
		return none
	return ContentFactory._strings(family.get("hands", []))

# "main" (damage weapons), "off" (support items: shield, orb, tome) or "" (no hands, or a mixed list: an error).
static func category(family: Dictionary) -> String:
	var hands: Array[String] = hands_of(family)
	if hands.is_empty():
		return ""
	if hands.size() == 1 and hands[0] == HAND_OFF:
		return "off"
	for h: String in hands:
		if not MAIN_HANDS.has(h):
			return ""
	return "main"

# The off-hand category every item of the family resolves to (derived, never authored).
static func off_hand_category(family: Dictionary) -> String:
	var id: String = str(family.get("id", ""))
	return id if category(family) == "off" and OFF_CATEGORIES.has(id) else ""

static func default_hand(family: Dictionary) -> String:
	var hands: Array[String] = hands_of(family)
	var wanted: String = str(family.get("default_hand", ""))
	if hands.has(wanted):
		return wanted
	return hands[0] if not hands.is_empty() else ""

# The `base` / `bands` key an item of `hand` reads ("none" for families without hands).
static func hand_key(family: Dictionary, hand: String) -> String:
	if hands_of(family).is_empty():
		return NO_HAND_KEY
	return hand if hands_of(family).has(hand) else default_hand(family)

# The five base numbers of `hand` (missing values default to the schema minimum).
static func base_of(family: Dictionary, hand: String) -> Dictionary:
	var all: Dictionary = DictUtil.dict_of(family.get("base", {}))
	var row: Dictionary = DictUtil.dict_of(all.get(hand_key(family, hand), {}))
	var out: Dictionary = {}
	for field: String in NUMBER_FIELDS:
		var rng: Array = NUMBER_RANGES[field]
		out[field] = DictUtil.to_int(row.get(field, rng[0]), DictUtil.to_int(rng[0]))
	return out

# {min, max} of `field` (or BONUS_BAND) for `hand`; empty when the family sets no band.
static func band_of(family: Dictionary, hand: String, field: String) -> Dictionary:
	var all: Dictionary = DictUtil.dict_of(family.get("bands", {}))
	var row: Dictionary = DictUtil.dict_of(all.get(hand_key(family, hand), {}))
	return DictUtil.dict_of(row.get(field, {}))

# Numbers an item of this type uses (fields hidden by the item schema are not adjustable).
static func number_fields_for(item_type: String) -> Array[String]:
	match item_type:
		"WEAPON":
			return NUMBER_FIELDS.duplicate()
		"SHIELD", "ARMOR", "ACCESSORY":
			var out: Array[String] = ["block_cost", "burden", "mitigation"]
			return out
	var none: Array[String] = []
	return none

# Slots an item of `hand` fits. Weapons and shields derive it from the hand (main hands -> MAIN_HAND, off -> OFF_HAND;
# a main_1h also fits OFF_HAND only with the legacy dual-wield flag); armor and accessories list them in `slots`.
static func slots_for(family: Dictionary, hand: String, allow_dual: bool = false) -> Array[String]:
	var out: Array[String] = []
	if hands_of(family).is_empty():
		return ContentFactory._strings(family.get("slots", []))
	if hand == HAND_OFF:
		out.append("OFF_HAND")
	elif MAIN_HANDS.has(hand):
		out.append("MAIN_HAND")
		if hand == HAND_MAIN_1H and allow_dual:
			out.append("OFF_HAND")
	return out

static func has_rule(family: Dictionary, rule: String) -> bool:
	return ContentFactory._strings(family.get("engine_rules", [])).has(rule)

# The family a FLAT (pre-family) item row belongs to: its gear type, else its item type's catch-all family.
static func family_id_for_row(row: Dictionary) -> String:
	var gear: String = str(row.get("weapon_type", ""))
	if not gear.is_empty():
		return gear
	match str(row.get("item_type", "")):
		"ACCESSORY":
			return "accessory"
		"CONSUMABLE":
			return "consumable"
		"MATERIAL":
			return "material"
		"QUEST":
			return "quest"
	return ""

# Fills the derived fields of a family row (off_hand_category, a default_hand inside `hands`).
static func derive(family: Dictionary) -> Dictionary:
	var out: Dictionary = family.duplicate(true)
	var type: String = item_type_of(out)
	var empty: Array[String] = []
	if not HAND_TYPES.has(type):
		out["hands"] = empty.duplicate()
	if type != "WEAPON":
		out["scaling_stats"] = empty.duplicate()
	if not SLOT_TYPES.has(type):
		out["slots"] = empty.duplicate()
	out["off_hand_category"] = off_hand_category(out)
	out["default_hand"] = default_hand(out)
	return out
