class_name CharacterStats
extends RefCounted

# ==============================================================================
# Tactical RPG - Character Stats & Spec Model
# 6-Stat Framework (STR, DEX, INT | CONS, PERC, KNOW), 1-10 Leveling & Masteries
# ==============================================================================

signal leveled_up(new_level: int)
signal attribute_point_allocated(stat_name: String, new_val: int)

const MAX_LEVEL: int = 10

var class_archetype: ClassArchetype
var archetype: ClassArchetype:
	get:
		return class_archetype
var level: int = 1
var experience: int = 0
var unallocated_attribute_points: int = 0
var unspent_attribute_points: int:
	get:
		return unallocated_attribute_points

# Core Attributes (Base + Allocated)
var str_stat: int = 10
var dex_stat: int = 10
var int_stat: int = 10
var cons_stat: int = 10
var perc_stat: int = 10
var know_stat: int = 10

var mastery: MasterySystem
var modifiers: Array[StatModifier] = []

func _init(p_archetype: ClassArchetype = null, p_level: int = 1) -> void:
	mastery = MasterySystem.new()
	if p_archetype != null:
		class_archetype = p_archetype
		str_stat = class_archetype.get_base_stat("STR")
		dex_stat = class_archetype.get_base_stat("DEX")
		int_stat = class_archetype.get_base_stat("INT")
		cons_stat = class_archetype.get_base_stat("CONS")
		perc_stat = class_archetype.get_base_stat("PERC")
		know_stat = class_archetype.get_base_stat("KNOW")

		for w_type: String in class_archetype.innate_masteries:
			var raw_lvl: Variant = class_archetype.innate_masteries[w_type]
			var lvl: int = raw_lvl if raw_lvl is int else 0
			if lvl > 0:
				mastery.set_weapon_mastery(w_type, lvl)
	else:
		class_archetype = ClassArchetype.create_warrior()

	level = 1
	if p_level > 1:
		for i in range(1, mini(p_level, MAX_LEVEL)):
			level_up()

func level_up() -> bool:
	if level >= MAX_LEVEL:
		return false

	level += 1

	# 1. Guaranteed stat gains from Class Archetype
	if class_archetype != null:
		str_stat += class_archetype.get_growth_for_stat("STR")
		dex_stat += class_archetype.get_growth_for_stat("DEX")
		int_stat += class_archetype.get_growth_for_stat("INT")
		cons_stat += class_archetype.get_growth_for_stat("CONS")
		perc_stat += class_archetype.get_growth_for_stat("PERC")
		know_stat += class_archetype.get_growth_for_stat("KNOW")

	# 2. Grant free attribute point for customization/flavor
	unallocated_attribute_points += 1

	leveled_up.emit(level)
	return true

func add_free_attribute_points(amount: int) -> void:
	unallocated_attribute_points += maxi(0, amount)

func allocate_attribute_point(stat_name: String) -> bool:
	if unallocated_attribute_points <= 0:
		return false

	var clean: String = stat_name.to_upper()
	match clean:
		"STR":
			str_stat += 1
		"DEX":
			dex_stat += 1
		"INT":
			int_stat += 1
		"CONS":
			cons_stat += 1
		"PERC":
			perc_stat += 1
		"KNOW":
			know_stat += 1
		_:
			return false

	unallocated_attribute_points -= 1
	attribute_point_allocated.emit(clean, get_effective_stat(clean))
	return true

# ------------------------------------------------------------------------------
# Base and Effective Stat Calculations
# ------------------------------------------------------------------------------

func get_base_stat(stat_name: String) -> int:
	var clean: String = stat_name.to_upper()
	match clean:
		"STR":
			return str_stat
		"DEX":
			return dex_stat
		"INT":
			return int_stat
		"CONS":
			return cons_stat
		"PERC":
			return perc_stat
		"KNOW":
			return know_stat
	return 10

func get_effective_stat(stat_name: String, context: Dictionary = {}) -> int:
	var clean: String = stat_name.to_upper()
	var base_val: int = 10

	match clean:
		"STR":
			base_val = str_stat
		"DEX":
			base_val = dex_stat
		"INT":
			base_val = int_stat
		"CONS":
			base_val = cons_stat
		"PERC":
			base_val = perc_stat
		"KNOW":
			base_val = know_stat

	# Add contextual mastery bonus (e.g. Sword Mastery -> +STR when using sword)
	if mastery != null:
		base_val += mastery.get_conditional_stat_bonus(clean, context)

	# Apply modifiers stack
	var flat_sum: int = 0
	var pct_sum: float = 0.0

	for mod: StatModifier in modifiers:
		if mod.stat_name.to_upper() == clean and mod.matches_context(context):
			flat_sum += mod.flat_bonus
			pct_sum += mod.percent_bonus

	var result: float = float(base_val + flat_sum) * (1.0 + pct_sum)
	return maxi(1, int(roundf(result)))

# ------------------------------------------------------------------------------
# Derived Combat Attributes
# ------------------------------------------------------------------------------

func get_max_hp(context: Dictionary = {}) -> int:
	var effective_cons: int = get_effective_stat("CONS", context)
	var effective_str: int = get_effective_stat("STR", context)
	return 60 + (effective_cons * 8) + (effective_str * 2)

func get_initiative_speed(armor_burden: int = 0, context: Dictionary = {}) -> int:
	var effective_dex: int = get_effective_stat("DEX", context)
	return maxi(1, 8 + effective_dex - armor_burden)

func get_melee_power(weapon_power: int = 0, context: Dictionary = {}) -> int:
	var effective_str: int = get_effective_stat("STR", context)
	return weapon_power + int(roundf(float(effective_str) * 1.5))

func get_ranged_power(weapon_power: int = 0, context: Dictionary = {}) -> int:
	var effective_dex: int = get_effective_stat("DEX", context)
	return weapon_power + int(roundf(float(effective_dex) * 1.5))

func get_tactical_power(skill_power: int = 0, context: Dictionary = {}) -> int:
	var effective_int: int = get_effective_stat("INT", context)
	return skill_power + int(roundf(float(effective_int) * 1.5))

func get_mitigation_rating(shield_mitigation: int = 0, context: Dictionary = {}) -> int:
	var effective_str: int = get_effective_stat("STR", context)
	return shield_mitigation + int(roundf(float(effective_str) * 0.25))

func get_effective_range(base_range: int = 1, context: Dictionary = {}) -> int:
	var effective_perc: int = get_effective_stat("PERC", context)
	var bonus: int = 1 if effective_perc >= 18 else 0
	return maxi(1, base_range + bonus)

func get_intent_unmask_blocks(context: Dictionary = {}) -> int:
	var effective_perc: int = get_effective_stat("PERC", context)
	if effective_perc >= 22:
		return 3
	elif effective_perc >= 14:
		return 2
	return 1

func get_carry_weight_capacity(context: Dictionary = {}) -> int:
	var effective_str: int = get_effective_stat("STR", context)
	var effective_cons: int = get_effective_stat("CONS", context)
	return 30 + (effective_str * 3) + (effective_cons * 1)

# ------------------------------------------------------------------------------
# Modifier Management
# ------------------------------------------------------------------------------

func add_modifier(mod: StatModifier) -> void:
	modifiers.append(mod)

func remove_modifier(source_id: String) -> void:
	modifiers = modifiers.filter(func(m: StatModifier) -> bool: return m.source_id != source_id)

func remove_modifiers_from_source(source_id: String) -> void:
	remove_modifier(source_id)

func tick_modifiers() -> void:
	var expired: Array[StatModifier] = []
	for mod: StatModifier in modifiers:
		if mod.tick_round():
			expired.append(mod)
	for exp_mod: StatModifier in expired:
		modifiers.erase(exp_mod)
