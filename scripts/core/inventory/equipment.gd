class_name Equipment
extends Item

# ==============================================================================
# Tactical RPG - Equipment & Gear Model
# Stat Requirements, Threshold Tiers, Weapon Proficiencies & Timeline Modifiers
# ==============================================================================

enum EquipSlot {
	MAIN_HAND,
	OFF_HAND,
	HEAD,
	BODY,
	ACCESSORY_1,
	ACCESSORY_2
}

var slot: EquipSlot = EquipSlot.MAIN_HAND
var weapon_type: String = ""  # sword, axe, bow, dagger, staff, shield, cloth, leather, plate
var base_power: int = 0
var base_range: int = 1
var block_cost: int = 1
var burden: int = 0           # Speed/initiative penalty applied to user
var mitigation: int = 0       # Flat defensive mitigation
var is_two_handed: bool = false # Two-handed weapon occupies both hands (Claymore, Warhammer, Bow)
var off_hand_category: String = "" # shield, shortsword, knuckles, dagger, knife, orb, tome
var scaling_stat: String = "STR"   # STR, DEX, or INT scaling
var stat_bonuses: Dictionary = {}        # Stat Name (String) -> Bonus (int)
var unlocked_commands: Array[String] = [] # e.g. ["DefendCommand", "FocusCommand"]
var effects: Array = []                  # item effects (Passives grammar, ITEM_TYPES only); apply while equipped

# Stat Requirements (e.g. {"STR": 16})
var required_stats: Dictionary = {}

# Threshold Tiers: extra boosts granted when meeting higher stat milestones
# Array of Dictionaries: {"stat": "STR", "threshold": 22, "bonus_power": 4, "desc": "Crushing Blow"}
var threshold_tiers: Array[Dictionary] = []

func _init(
	p_id: String = "",
	p_name: String = "Equipment",
	p_slot: EquipSlot = EquipSlot.MAIN_HAND,
	p_weapon_type: String = "",
	p_type: ItemType = ItemType.WEAPON,
	p_power: int = 0,
	p_reqs: Dictionary = {},
	p_tiers: Array[Dictionary] = [],
	p_weight: float = 2.0,
	p_value: int = 25,
	p_rarity: ItemRarity = ItemRarity.COMMON
) -> void:
	super._init(p_id, p_name, p_type, p_weight, p_value, p_rarity, "", 1)
	slot = p_slot
	weapon_type = p_weapon_type
	base_power = p_power
	required_stats = p_reqs
	threshold_tiers = p_tiers

static func slot_display_name(p_slot: EquipSlot) -> String:
	match p_slot:
		EquipSlot.MAIN_HAND: return "Main Hand"
		EquipSlot.OFF_HAND: return "Off Hand"
		EquipSlot.HEAD: return "Headgear"
		EquipSlot.BODY: return "Body Armor"
		EquipSlot.ACCESSORY_1: return "Accessory 1"
		EquipSlot.ACCESSORY_2: return "Accessory 2"
	return "Gear Slot"

func is_weapon() -> bool:
	return item_type == ItemType.WEAPON

func is_shield() -> bool:
	return item_type == ItemType.SHIELD or off_hand_category == "shield"

# Off-hand weapons (shortsword, knuckles, dagger, tome...) can only be held in the off-hand.
func is_off_hand_weapon() -> bool:
	return is_weapon() and not off_hand_category.is_empty() and off_hand_category != "shield" and not is_two_handed

# Legacy dual-wield switch (FORGE1_ITEMS 8.1; code only, never data, never saved): while true a plain main_1h weapon
# may ALSO go into the off hand (Dual Strike, off-hand attacks, Switch Weapon as before strict hand typing). Default
# false; the Gear Dojo sets it for its session (DojoSession) and tests flip and restore it. (StatRules points here.)
static var allow_plain_dual_wield: bool = false

# Slot compatibility, the single equip gate (stash filters, the dojo equip path, class/unit loadout validation):
# 2H weapons -> main hand only; off-hand weapons and shields -> off-hand only; plain 1H weapons -> main hand only,
# unless allow_plain_dual_wield (then main or off hand); armor/accessories -> own slot. Worn gear is never re-gated.
func can_equip_in_slot(target: EquipSlot) -> bool:
	if is_shield():
		return target == EquipSlot.OFF_HAND
	if is_weapon():
		if is_two_handed:
			return target == EquipSlot.MAIN_HAND
		if is_off_hand_weapon():
			return target == EquipSlot.OFF_HAND
		return target == EquipSlot.MAIN_HAND or (target == EquipSlot.OFF_HAND and allow_plain_dual_wield)
	if slot == EquipSlot.ACCESSORY_1 or slot == EquipSlot.ACCESSORY_2:
		return target == EquipSlot.ACCESSORY_1 or target == EquipSlot.ACCESSORY_2
	return target == slot

# Orb and tome: off-hand-only casting aids. They are never two-handed, and a two-handed main weapon
# leaves the off hand empty, so they cannot be paired with one (see conflicts_with_main).
const FOCUS_TYPES: Array[String] = ["orb", "tome"]

func is_focus() -> bool:
	return is_weapon() and FOCUS_TYPES.has(weapon_type)

# True when this item cannot be held together with `main` (a focus needs a main hand that is not two-handed).
func conflicts_with_main(main: Equipment) -> bool:
	return is_focus() and main != null and main.is_two_handed

const HAND_MAIN_1H: String = "main_1h"
const HAND_MAIN_2H: String = "main_2h"
const HAND_OFF: String = "off"
const HANDS: Array[String] = [HAND_MAIN_1H, HAND_MAIN_2H, HAND_OFF]

# Which hand the item occupies (DERIVED, never stored): shields and off-hand-category weapons
# are `off`, two-handed weapons `main_2h`, every other weapon `main_1h`. Armor and accessories: "".
# With a main_2h weapon the off hand is empty, so the off-hand jab attacks with bare hands (STR+DEX).
func hand() -> String:
	if is_shield():
		return HAND_OFF
	if not is_weapon():
		return ""
	if is_two_handed:
		return HAND_MAIN_2H
	if is_off_hand_weapon():
		return HAND_OFF
	return HAND_MAIN_1H

static func hand_display_name(h: String) -> String:
	match h:
		HAND_MAIN_1H: return "main hand, one-handed"
		HAND_MAIN_2H: return "main hand, two-handed"
		HAND_OFF: return "off hand"
	return "-"

# Gear types that are ranged even at range 1 (the item family engine rule `always_ranged`; `sling` is no gear type yet).
const ALWAYS_RANGED_TYPES: Array[String] = ["bow", "crossbow", "sling"]

func is_ranged_weapon() -> bool:
	if item_type != ItemType.WEAPON:
		return false
	return ALWAYS_RANGED_TYPES.has(weapon_type.to_lower()) or base_range > 1

func is_melee_weapon() -> bool:
	if item_type != ItemType.WEAPON:
		return false
	return not is_ranged_weapon()

func get_requirements_display() -> String:
	return get_tier_requirements_display()

# Convention: [Req <STAT level0>, <STAT level1>] (Requirement 4)
# level0 = wield without penalty; level1 = threshold tier boost (+Power, +STAT, effect)
func get_tier_requirements_display() -> String:
	if required_stats.is_empty() and threshold_tiers.is_empty():
		return "[No Req]"

	var parts: Array[String] = []
	var processed_stats: Array[String] = []

	for st_k: String in required_stats:
		processed_stats.append(st_k.to_upper())
		var lvl0: int = required_stats[st_k] if required_stats[st_k] is int else 0
		var stat_parts: Array[String] = ["%s %d" % [st_k, lvl0]]

		for tier: Dictionary in threshold_tiers:
			var t_stat: String = str(tier.get("stat", ""))
			if t_stat.to_upper() == st_k.to_upper():
				var lvl1: int = tier.get("threshold", 0)
				var bonus_pwr: int = tier.get("bonus_power", 0)
				var desc: String = str(tier.get("desc", ""))
				var bonus_label: String = ""
				if bonus_pwr > 0:
					bonus_label = "+%d Power" % bonus_pwr
				elif not desc.is_empty():
					bonus_label = desc
				if not bonus_label.is_empty():
					stat_parts.append("%d (%s)" % [lvl1, bonus_label])
				else:
					stat_parts.append("%d" % lvl1)

		parts.append(", ".join(stat_parts))

	for tier: Dictionary in threshold_tiers:
		var t_stat: String = str(tier.get("stat", "")).to_upper()
		if t_stat.is_empty():
			# Extended criteria (proficiency, unit level; resolved rows carry stat ""): "sword training 2 (+2 STR)".
			var label: String = ItemText.bonus_text(tier_bonus(tier))
			parts.append("%s (%s)" % [tier_criterion_text(tier), label] if not label.is_empty() else tier_criterion_text(tier))
			continue
		if not processed_stats.has(t_stat):
			processed_stats.append(t_stat)
			var lvl1: int = tier.get("threshold", 0)
			var bonus_pwr: int = tier.get("bonus_power", 0)
			var desc: String = str(tier.get("desc", ""))
			var bonus_label: String = ""
			if bonus_pwr > 0:
				bonus_label = "+%d Power" % bonus_pwr
			elif not desc.is_empty():
				bonus_label = desc
			if not bonus_label.is_empty():
				parts.append("%s %d (%s)" % [t_stat, lvl1, bonus_label])
			else:
				parts.append("%s %d" % [t_stat, lvl1])

	return "[Req %s]" % "; ".join(parts)

func get_requirement_status_summary(stats: CharacterStats, prof: Proficiency) -> Dictionary:
	var meets: bool = meets_requirements(stats, prof)
	var pen: Dictionary = get_wield_penalty(stats, prof)
	var is_pen: bool = pen.get("is_penalized", false)
	var raw_def: Variant = pen.get("deficit", 0)
	var def_num: int = raw_def if raw_def is int else 0
	var raw_ratio: Variant = pen.get("power_penalty_ratio", 0.0)
	var pen_ratio: float = raw_ratio if raw_ratio is float else 0.0
	var pen_pct: int = int(roundf(pen_ratio * 100.0))
	var tier_boost: int = get_unlocked_threshold_power_boost(stats, prof)

	var status_tag := ""
	var tag_color := Color.WHITE
	if is_pen:
		status_tag = "⚠️ -%d Deficit (-%d%% Power)" % [def_num, pen_pct]
		tag_color = Color(1.0, 0.4, 0.4)
	elif tier_boost > 0:
		status_tag = "★ Tier 1 Active (+%d Power)" % tier_boost
		tag_color = Color(1.0, 0.85, 0.3)
	elif meets and not required_stats.is_empty():
		status_tag = ""
		tag_color = Color(0.4, 0.9, 0.5)
	else:
		status_tag = ""
		tag_color = Color(0.8, 0.9, 0.8)

	return {
		"meets": meets,
		"is_penalized": is_pen,
		"deficit": def_num,
		"penalty_percent": pen_pct,
		"tier_boost": tier_boost,
		"status_tag": status_tag,
		"tag_color": tag_color,
		"req_display": get_tier_requirements_display()
	}

func get_stat_bonuses_display() -> String:
	if stat_bonuses.is_empty():
		return ""
	var parts: Array[String] = []
	for st_k: String in stat_bonuses:
		var v: Variant = stat_bonuses[st_k]
		var n: int = v if v is int else 0
		var sign_str: String = "+%d" % n if n > 0 else "%d" % n
		parts.append("%s %s" % [sign_str, st_k])
	return ", ".join(parts)

# ------------------------------------------------------------------------------
# Stat Requirements & Proficiency Threshold Discount
# ------------------------------------------------------------------------------

func meets_requirements(stats: CharacterStats, prof: Proficiency) -> bool:
	if required_stats.is_empty() or stats == null:
		return true

	var discount: int = 0
	if prof != null and not weapon_type.is_empty():
		discount = prof.get_weapon_requirement_discount(weapon_type)

	for req_stat: String in required_stats:
		var raw_req: Variant = required_stats[req_stat]
		var req_val: int = raw_req if raw_req is int else 0
		var eff_stat: int = stats.get_effective_stat(req_stat, {"weapon": weapon_type}) + discount
		if eff_stat < req_val:
			return false

	return true

# Calculates penalty details if character attempts to wield without meeting requirements
func get_wield_penalty(stats: CharacterStats, prof: Proficiency) -> Dictionary:
	var result: Dictionary = {
		"is_penalized": false,
		"power_penalty_ratio": 0.0,
		"extra_block_cost": 0,
		"deficit": 0
	}

	if meets_requirements(stats, prof) or stats == null:
		return result

	var discount: int = 0
	if prof != null and not weapon_type.is_empty():
		discount = prof.get_weapon_requirement_discount(weapon_type)

	var max_deficit: int = 0
	for req_stat: String in required_stats:
		var raw_req: Variant = required_stats[req_stat]
		var req_val: int = raw_req if raw_req is int else 0
		var eff_stat: int = stats.get_effective_stat(req_stat, {"weapon": weapon_type}) + discount
		if eff_stat < req_val:
			max_deficit = maxi(max_deficit, req_val - eff_stat)

	result["is_penalized"] = true
	result["deficit"] = max_deficit
	# Suffer 20% power loss per deficit point (up to 60%) and +1 timeline block cost if deficit >= 4
	result["power_penalty_ratio"] = minf(0.60, float(max_deficit) * 0.15)
	if max_deficit >= 4:
		result["extra_block_cost"] = 1

	return result

# ------------------------------------------------------------------------------
# Tier engine (FORGE1_ITEMS 4): criteria stat / proficiency / unit level; bonuses power, mitigation, stat bonuses, effects
# A resolved tier row keeps the legacy keys {stat, threshold, bonus_power, desc}; extended tiers add `when` and `bonus`
# (ItemModifiers.legacy_tiers). Evaluation order (4.3, single pass, no circularity): static gear stat modifiers ->
# tiers are evaluated on stats WITHOUT tier modifiers (source TIER_SOURCE) -> tier stat modifiers are added ->
# gear effects are rebuilt including tier effects (Combatant.sync_from_stats_and_gear). So one tier's stat bonus never
# opens another tier, on this item or another.
# ------------------------------------------------------------------------------

const TIER_SOURCE: String = "gear_tier"   # StatModifier source of every tier stat bonus

# The criterion of a resolved tier row: its `when`, else the legacy stat threshold.
static func tier_when(tier: Dictionary) -> Dictionary:
	var when: Dictionary = DictUtil.dict_of(tier.get("when", {}))
	if not when.is_empty():
		return when
	return {"type": ItemModifiers.WHEN_STAT, "stat": str(tier.get("stat", "STR")), "value": DictUtil.to_int(tier.get("threshold", 0))}

# The bonus of a resolved tier row: its `bonus`, else the legacy power bonus.
static func tier_bonus(tier: Dictionary) -> Dictionary:
	var bonus: Dictionary = DictUtil.dict_of(tier.get("bonus", {}))
	if not bonus.is_empty():
		return bonus
	return {"power": DictUtil.to_int(tier.get("bonus_power", 0))}

# "STR 22", "sword training 2", "unit level 5".
func tier_criterion_text(tier: Dictionary) -> String:
	var when: Dictionary = tier_when(tier)
	match str(when.get("type", "")):
		ItemModifiers.WHEN_PROFICIENCY:
			return "%s training %d" % [_tier_gear(when), DictUtil.to_int(when.get("level", 0))]
		ItemModifiers.WHEN_LEVEL:
			return "unit level %d" % DictUtil.to_int(when.get("value", 0))
	return "%s %d" % [str(when.get("stat", "STR")).to_upper(), DictUtil.to_int(when.get("value", 0))]

# The gear token a proficiency criterion reads (default: the item's own family / gear type).
func _tier_gear(when: Dictionary) -> String:
	var gear: String = str(when.get("gear", ""))
	return gear if not gear.is_empty() else weapon_type

# True when `stats` / `prof` meet the tier criterion. stat_at_least: effective stat in this item's weapon context
# WITHOUT tier stat bonuses + proficiency level x LEVEL_DISCOUNT; proficiency_at_least: Proficiency.level_of(gear);
# unit_level_at_least: CharacterStats.level.
func tier_met(tier: Dictionary, stats: CharacterStats, prof: Proficiency) -> bool:
	if stats == null:
		return false
	var when: Dictionary = tier_when(tier)
	match str(when.get("type", "")):
		ItemModifiers.WHEN_STAT:
			var discount: int = 0
			if prof != null and not weapon_type.is_empty():
				discount = prof.get_weapon_requirement_discount(weapon_type)
			var eff_stat: int = stats.get_effective_stat(str(when.get("stat", "STR")), {"weapon": weapon_type}, TIER_SOURCE) + discount
			return eff_stat >= DictUtil.to_int(when.get("value", 0))
		ItemModifiers.WHEN_PROFICIENCY:
			return prof != null and prof.level_of(_tier_gear(when)) >= DictUtil.to_int(when.get("level", 0))
		ItemModifiers.WHEN_LEVEL:
			return stats.level >= DictUtil.to_int(when.get("value", 0))
	return false

# Returns all unlocked threshold tiers (every criterion type; see tier_met)
func get_active_threshold_tiers(stats: CharacterStats, prof: Proficiency) -> Array[Dictionary]:
	var active_tiers: Array[Dictionary] = []
	if stats == null or threshold_tiers.is_empty():
		return active_tiers
	for tier: Dictionary in threshold_tiers:
		if tier_met(tier, stats, prof):
			active_tiers.append(tier)
	return active_tiers

# 1-based positions (in threshold_tiers) of the unlocked tiers: the `<n>` of the effect source "gear:<id>:tier<n>".
func get_active_tier_numbers(stats: CharacterStats, prof: Proficiency) -> Array[int]:
	var out: Array[int] = []
	if stats == null:
		return out
	for i: int in range(threshold_tiers.size()):
		if tier_met(threshold_tiers[i], stats, prof):
			out.append(i + 1)
	return out

# Sum of the mitigation bonuses of the unlocked tiers (armor, shields).
func get_tier_mitigation_bonus(stats: CharacterStats, prof: Proficiency) -> int:
	var total: int = 0
	for tier: Dictionary in get_active_threshold_tiers(stats, prof):
		total += DictUtil.to_int(tier_bonus(tier).get("mitigation", 0))
	return total

# Mitigation this piece adds to the armor rating: base + unlocked tier mitigation (stats null = base only).
func get_effective_mitigation(stats: CharacterStats, prof: Proficiency) -> int:
	return mitigation + get_tier_mitigation_bonus(stats, prof)

# Stat -> summed bonus of the unlocked tiers (added as TIER_SOURCE modifiers by Combatant.sync_from_stats_and_gear).
func get_tier_stat_bonuses(stats: CharacterStats, prof: Proficiency) -> Dictionary:
	var out: Dictionary = {}
	for tier: Dictionary in get_active_threshold_tiers(stats, prof):
		var bonuses: Dictionary = DictUtil.dict_of(tier_bonus(tier).get("stat_bonuses", {}))
		for stat: Variant in bonuses:
			var key: String = str(stat).to_upper()
			out[key] = DictUtil.int_of(out, key) + DictUtil.to_int(bonuses[stat])
	return out

# The effects list of tier number `n` (1-based; [] when it has none).
func tier_effects(n: int) -> Array:
	if n < 1 or n > threshold_tiers.size():
		return []
	return DictUtil.array_of(tier_bonus(threshold_tiers[n - 1]).get("effects", []))

# Helper: sum of unlocked bonus power across all active threshold tiers
func get_unlocked_threshold_power_boost(stats: CharacterStats, prof: Proficiency) -> int:
	var total_boost: int = 0
	var active_tiers: Array[Dictionary] = get_active_threshold_tiers(stats, prof)
	for tier: Dictionary in active_tiers:
		var raw_bonus: Variant = tier.get("bonus_power", 0)
		if raw_bonus is int:
			total_boost += raw_bonus
	return total_boost

# Effective power including tier boosts, proficiency boosts and deficit penalties (Requirements 1 & 2)
func get_effective_equipment_power(stats: CharacterStats, prof: Proficiency) -> int:
	var total_power: int = base_power

	# Add threshold tier bonuses
	var active_tiers: Array[Dictionary] = get_active_threshold_tiers(stats, prof)
	for tier: Dictionary in active_tiers:
		var bonus_raw: Variant = tier.get("bonus_power", 0)
		var bonus: int = bonus_raw if bonus_raw is int else 0
		total_power += bonus

	# Add weapon proficiency bonus power
	if prof != null and not weapon_type.is_empty():
		total_power += prof.get_weapon_power_bonus(weapon_type)

	# Deduct deficit penalties: Wep Power * (1 - Deficit)
	var penalty: Dictionary = get_wield_penalty(stats, prof)
	if penalty.get("is_penalized", false):
		var ratio_raw: Variant = penalty.get("power_penalty_ratio", 0.0)
		var ratio: float = ratio_raw if ratio_raw is float else 0.0
		total_power = maxi(1, int(roundf(float(total_power) * (1.0 - ratio))))

	return total_power

func get_effective_block_cost(stats: CharacterStats, prof: Proficiency) -> int:
	var cost: int = block_cost
	var penalty: Dictionary = get_wield_penalty(stats, prof)
	if penalty["is_penalized"]:
		cost += penalty["extra_block_cost"]
	if prof != null and not weapon_type.is_empty():
		cost -= prof.block_cost_reduction(weapon_type)   # master-level training
	return maxi(1, cost)
