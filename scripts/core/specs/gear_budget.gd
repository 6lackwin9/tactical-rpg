class_name GearBudget
extends RefCounted

# ==============================================================================
# Tactical RPG - Gear budget rules (M2d, mirrors StatRules for gear)
# One weighted number per equipment item; a class's default loadout value is the sum
# over its slots (pouch consumables ignored). The sum is compared with TARGET_BUDGET
# +- TOLERANCE. Items flagged `special` may push a loadout over the band only when they
# carry a `special_note` documenting the trade-off; they are always reported separately.
# Pure functions over catalog entries (Dictionaries); validation is soft (warnings).
# ==============================================================================

# Item value = base_power*W_POWER + mitigation*W_MITIGATION + (sum of stat bonuses)*W_STAT_POINT
#            + burden*W_BURDEN + (block_cost - 1)*W_EXTRA_BLOCK
const W_POWER: float = 1.0
const W_MITIGATION: float = 1.5
const W_STAT_POINT: float = 2.0
const W_BURDEN: float = -2.0
const W_EXTRA_BLOCK: float = -3.0
# Item effects (C1): potency is x20 per 1.0 (+0.10 = 2.0), status potency 0.2 per hit-chance point, spell reach 3.0 per tile.
const W_SKILL_POTENCY: float = 20.0
const W_STATUS_POTENCY: float = 0.2
const W_SPELL_REACH: float = 3.0

# Target loadout value (near the median of the six original class loadouts: 19 19 22 24 27 35) and allowed fraction either side.
const TARGET_BUDGET: float = 23.0
const TOLERANCE: float = 0.20

# Value of an empty main hand (bare hands) = the trained-fists base power (DamageCalculator.UNARMED_BASE_POWER,
# one source of truth; BAL2-L Rule 5): about a typical one-handed weapon (shortsword 14, stiletto 15, knuckles 12).
const UNARMED_VALUE: float = float(DamageCalculator.UNARMED_BASE_POWER)

static func band_low() -> float:
	return TARGET_BUDGET * (1.0 - TOLERANCE)

static func band_high() -> float:
	return TARGET_BUDGET * (1.0 + TOLERANCE)

static func is_gear(entry: Dictionary) -> bool:
	return ContentSchema.EQUIP_TYPES.has(str(entry.get("item_type", "")))

static func is_special(entry: Dictionary) -> bool:
	return DictUtil.to_bool(entry.get("special", false))

# Weighted value of one catalog item entry (0 for non-equipment).
static func item_value(entry: Dictionary) -> float:
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
		var fx: Dictionary = DictUtil.dict_of(raw)
		match str(fx.get("type", "")):
			"skill_potency":
				v += DictUtil.to_float(fx.get("value", 0.0)) * W_SKILL_POTENCY
			"status_potency":
				v += DictUtil.to_int(fx.get("value", 0)) * W_STATUS_POTENCY
			"spell_reach":
				v += DictUtil.to_int(fx.get("value", 0)) * W_SPELL_REACH
	return v

# Per-slot breakdown: [{slot, id, name, value, special, note}] for filled gear slots.
static func loadout_breakdown(class_entry: Dictionary) -> Array[Dictionary]:
	var rows: Array[Dictionary] = []
	for slot: String in ContentSchema.LOADOUT_SLOTS:
		var id: String = str(class_entry.get(slot, ""))
		if id.is_empty() or not ContentCatalog.has("items", id):
			continue
		var item: Dictionary = ContentCatalog.get_entry("items", id)
		if not is_gear(item):
			continue
		rows.append({"slot": slot, "id": id, "name": str(item.get("name", id)), "value": item_value(item),
			"special": is_special(item), "note": str(item.get("special_note", ""))})
	# Bare hands are a weapon of sorts (STR+DEX unit power): an empty main hand counts UNARMED_VALUE, so an
	# unarmed class (brawler-style) isn't read as under budget. A class with no gear at all stays unjudged.
	if not rows.is_empty() and str(class_entry.get("main_hand", "")).is_empty():
		rows.append({"slot": "main_hand", "id": "", "name": "Unarmed", "value": UNARMED_VALUE, "special": false, "note": ""})
	return rows

# Sum over all slots (special items included).
static func loadout_value(class_entry: Dictionary) -> float:
	var total: float = 0.0
	for row: Dictionary in loadout_breakdown(class_entry):
		total += DictUtil.to_float(row["value"])
	return total

# Part of the loadout value that comes from special items (reported separately).
static func special_value(class_entry: Dictionary) -> float:
	var total: float = 0.0
	for row: Dictionary in loadout_breakdown(class_entry):
		if DictUtil.to_bool(row["special"]):
			total += DictUtil.to_float(row["value"])
	return total

static func in_band(value: float) -> bool:
	return value >= band_low() - 0.0001 and value <= band_high() + 0.0001

# True when a special item with a documented trade-off sits in the loadout.
static func has_documented_special(class_entry: Dictionary) -> bool:
	for row: Dictionary in loadout_breakdown(class_entry):
		if DictUtil.to_bool(row["special"]) and not str(row["note"]).strip_edges().is_empty():
			return true
	return false

# Warnings (never errors) for a class entry: out-of-band default loadout. Going over
# the band is tolerated when a documented special item is part of the loadout.
static func check_class(class_entry: Dictionary) -> Array[String]:
	var warnings: Array[String] = []
	var rows: Array[Dictionary] = loadout_breakdown(class_entry)
	if rows.is_empty():
		return warnings
	var total: float = loadout_value(class_entry)
	if in_band(total):
		return warnings
	var over: bool = total > band_high()
	if over and has_documented_special(class_entry):
		return warnings
	rows.sort_custom(func(a: Dictionary, b: Dictionary) -> bool: return absf(DictUtil.to_float(a["value"])) > absf(DictUtil.to_float(b["value"])))
	var tops: Array[String] = []
	for i: int in range(mini(3, rows.size())):
		tops.append("%s %+.1f" % [str(rows[i]["name"]), DictUtil.to_float(rows[i]["value"])])
	warnings.append("Gear budget: default loadout is %.1f, %s the budget %.1f (band %.1f to %.1f). Top items: %s." % [
		total, "above" if over else "below", TARGET_BUDGET, band_low(), band_high(), ", ".join(tops)])
	return warnings

# Warnings for an item entry: special without a documented trade-off.
static func check_item(item_entry: Dictionary) -> Array[String]:
	var warnings: Array[String] = []
	if is_special(item_entry) and str(item_entry.get("special_note", "")).strip_edges().is_empty():
		warnings.append("Special item needs a special note describing its trade-off or playstyle.")
	return warnings
