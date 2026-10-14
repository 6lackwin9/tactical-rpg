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

# Core Attributes (Base + Allocated) backed by dictionary
var _core_stats: Dictionary = {
	"STR": 10,
	"DEX": 10,
	"INT": 10,
	"CONS": 10,
	"PERC": 10,
	"KNOW": 10
}

var str_stat: int:
	get: return _core_stats.get("STR", 10)
	set(v): _core_stats["STR"] = v
var dex_stat: int:
	get: return _core_stats.get("DEX", 10)
	set(v): _core_stats["DEX"] = v
var int_stat: int:
	get: return _core_stats.get("INT", 10)
	set(v): _core_stats["INT"] = v
var cons_stat: int:
	get: return _core_stats.get("CONS", 10)
	set(v): _core_stats["CONS"] = v
var perc_stat: int:
	get: return _core_stats.get("PERC", 10)
	set(v): _core_stats["PERC"] = v
var know_stat: int:
	get: return _core_stats.get("KNOW", 10)
	set(v): _core_stats["KNOW"] = v

var proficiency: Proficiency          # what this unit can wield and how well (class + passives)
var modifiers: Array[StatModifier] = []

# A unit of `p_level` of the class: class base stats + `p_spend` (spent from the level's
# spend pool, see StatRules) + the class growth of every level gained. Points of the
# pool not in `p_spend` stay unallocated for the player to spend later.
func _init(p_archetype: ClassArchetype = null, p_level: int = 1, p_spend: Dictionary = {}) -> void:
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

func level_up() -> bool:
	if level >= MAX_LEVEL:
		return false

	level += 1
	_apply_growth()

	# Free attribute point for customization (StatRules.FREE_PER_LEVEL)
	unallocated_attribute_points += StatRules.FREE_PER_LEVEL

	leveled_up.emit(level)
	return true

# Guaranteed class-weighted stat gains of one level.
func _apply_growth() -> void:
	if class_archetype != null:
		for s: String in _core_stats:
			_core_stats[s] += class_archetype.get_growth_for_stat(s)

func add_free_attribute_points(amount: int) -> void:
	unallocated_attribute_points += maxi(0, amount)

func allocate_attribute_point(stat_name: String) -> bool:
	if unallocated_attribute_points <= 0:
		return false

	var clean: String = stat_name.to_upper()
	if not _core_stats.has(clean):
		return false

	_core_stats[clean] += 1
	unallocated_attribute_points -= 1
	attribute_point_allocated.emit(clean, get_effective_stat(clean))
	return true

# ------------------------------------------------------------------------------
# Base and Effective Stat Calculations
# ------------------------------------------------------------------------------

func get_base_stat(stat_name: String) -> int:
	return _core_stats.get(stat_name.to_upper(), 10)

func get_effective_stat(stat_name: String, context: Dictionary = {}) -> int:
	var clean: String = stat_name.to_upper()
	var base_val: int = get_base_stat(clean)

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

# What this unit adds to the PARTY-WIDE carry capacity (see PartyCarry). Burden, by contrast,
# is per-unit: it only affects the wearer.
func get_carry_contribution(context: Dictionary = {}) -> int:
	var effective_str: int = get_effective_stat("STR", context)
	var effective_cons: int = get_effective_stat("CONS", context)
	return (effective_str * 3) + effective_cons

# Critical Hit Rate (DEX based) & Critical Damage Multiplier (Requirements 9 & 10)
func get_crit_rate(context: Dictionary = {}) -> float:
	var effective_dex: int = get_effective_stat("DEX", context)
	return DamageCalculator.CRIT_BASE + float(maxi(0, effective_dex - DamageCalculator.CRIT_DEX_FLOOR)) * DamageCalculator.CRIT_PER_DEX

func get_crit_damage_multiplier(_context: Dictionary = {}) -> float:
	return DamageCalculator.CRIT_MULT

# Status & Debuff Resistance (INT based) (Requirement 10)
func get_debuff_resistance(context: Dictionary = {}) -> float:
	var effective_int: int = get_effective_stat("INT", context)
	return float(effective_int) * 2.5

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
