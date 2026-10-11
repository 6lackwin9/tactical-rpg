class_name DamageCalculator
extends RefCounted

# ==============================================================================
# Tactical RPG – Authoritative Damage Calculator
#
# Single source of truth for all combat math. Call compute() from any system
# (Gear Dojo, Arena, AI planner, etc.) to guarantee identical results.
#
# Formula (per spec):
#   Melee/Ranged Power =
#       (Wep Power * (1 - Deficit) + Stat * 1.5) * (1 + ΣBuffs - ΣDebuffs)
#   where Wep Power = Base Power + Σ Tier Bonus Power + Mastery Bonus Power
#
#   Final Damage =
#       Attack Power * Skill Mult * (1 - Mitigation%) * (1 + Stance) * Crit * (1 + Effect)
# ==============================================================================

# Stance bonus constants (centralised so arena & dojo stay in sync)
const OFFENSIVE_STANCE_BONUS: float = 0.20
const DEFENSIVE_STANCE_PENALTY: float = -0.15
const SCALING_FACTOR: float = 1.5

# ------------------------------------------------------------------------------
# Primary compute entry point
#
# Returns a rich breakdown Dictionary so callers can display every step
# without recomputing anything themselves.
# Keys:
#   weapon              Equipment (or null for unarmed)
#   base_w_power        int
#   tier_boost          int
#   mastery_pwr         int
#   total_w_power       int
#   pen_ratio           float  [0.0 – 0.60]
#   def_amount          int    deficit point count
#   penalized_w_power   float  after deficit deduction
#   stat_name           String
#   stat_val            int
#   stat_scaling_contrib float
#   unbuffed_attack_power float
#   buff_mult           float
#   final_attack_power  int
#   skill_mult          float
#   mit_ratio           float
#   mit_mult            float
#   stance_mult         float
#   stance_factor       float
#   crit_rate           float  [%]
#   is_crit             bool
#   crit_dmg_mult       float
#   effect_factor       float
#   final_dmg           int    pre-application result (call Combatant.take_damage separately)
# ------------------------------------------------------------------------------

static func compute(
	attacker: Combatant,
	target: Combatant,
	is_ranged: bool,
	use_offhand: bool,
	skill_mult: float = 1.0,
	force_crit: bool = false,
	allow_random_crit: bool = false
) -> Dictionary:
	# --- Weapon selection ---
	var w: Equipment = null
	if attacker.inventory != null:
		w = attacker.inventory.get_offhand_weapon() if use_offhand else attacker.inventory.get_main_weapon()

	var ctx: Dictionary = {"weapon": w.weapon_type} if w != null else {}

	# --- Step 1: Weapon Power = Base + Tier Boost + Mastery ---
	var base_w_power: int = w.base_power if w != null else 10
	var tier_boost: int = 0
	var mastery_pwr: int = 0
	if w != null and attacker.stats != null:
		tier_boost = w.get_unlocked_threshold_power_boost(attacker.stats, attacker.stats.mastery)
		if attacker.stats.mastery != null:
			mastery_pwr = attacker.stats.mastery.get_weapon_power_bonus(w.weapon_type)
	var total_w_power: int = base_w_power + tier_boost + mastery_pwr

	# --- Deficit penalty: Wep Power * (1 − Deficit) ---
	var pen_ratio: float = 0.0
	var def_amount: int = 0
	if w != null and attacker.stats != null:
		var pen: Dictionary = w.get_wield_penalty(attacker.stats, attacker.stats.mastery)
		if pen.get("is_penalized", false):
			var raw_ratio: Variant = pen.get("power_penalty_ratio", 0.0)
			pen_ratio = raw_ratio if raw_ratio is float else 0.0
			var raw_def: Variant = pen.get("deficit", 0)
			def_amount = raw_def if raw_def is int else 0
	var penalized_w_power: float = float(total_w_power) * (1.0 - pen_ratio)

	# --- Step 2: Stat scaling (STR / DEX / INT) * 1.5 ---
	var stat_name: String = "STR"
	if w != null:
		stat_name = w.scaling_stat
	elif is_ranged:
		stat_name = "DEX"
	var stat_val: int = attacker.stats.get_effective_stat(stat_name, ctx) if attacker.stats != null else 10
	var stat_scaling_contrib: float = float(stat_val) * SCALING_FACTOR

	# --- Unbuffed attack power ---
	var unbuffed_attack_power: float = penalized_w_power + stat_scaling_contrib

	# --- Buff / Debuff multiplier: (1 + ΣBuff − ΣDebuff) ---
	var buff_sum: float = (attacker.attack_boost_multiplier - 1.0) if attacker.attack_boost_multiplier > 1.0 else 0.0
	var debuff_sum: float = 0.0
	var buff_mult: float = 1.0 + buff_sum - debuff_sum
	var final_attack_power: int = maxi(1, int(roundf(unbuffed_attack_power * buff_mult)))

	# --- Step 3: Target mitigation (1 − Mitigation%) ---
	var mit_ratio: float = target.defense_mitigation_ratio if target != null else 0.0
	var mit_mult: float = 1.0 - mit_ratio

	# --- Step 4: Stance multiplier ---
	var stance_mult: float = 0.0
	if attacker.current_stance == CombatTypes.StanceType.OFFENSIVE:
		stance_mult = OFFENSIVE_STANCE_BONUS
	elif attacker.current_stance == CombatTypes.StanceType.DEFENSIVE:
		stance_mult = DEFENSIVE_STANCE_PENALTY
	var stance_factor: float = 1.0 + stance_mult

	# --- Step 5: Critical hit ---
	var crit_rate: float = attacker.stats.get_crit_rate(ctx) if attacker.stats != null else 5.0
	var is_crit: bool = false
	if force_crit:
		is_crit = true
	elif allow_random_crit:
		is_crit = (randf() * 100.0) < crit_rate

	var crit_dmg_mult: float = attacker.stats.get_crit_damage_multiplier(ctx) if (is_crit and attacker.stats != null) else (2.5 if is_crit else 1.0)

	# --- Step 6: Effect bonus / penalty ---
	var effect_factor: float = 1.0  # Reserved for future status-effect integration

	# --- Final damage ---
	var gross: float = float(final_attack_power) * skill_mult * mit_mult * stance_factor * crit_dmg_mult * effect_factor
	var final_dmg: int = maxi(1, int(roundf(gross)))

	return {
		"weapon": w,
		"base_w_power": base_w_power,
		"tier_boost": tier_boost,
		"mastery_pwr": mastery_pwr,
		"total_w_power": total_w_power,
		"pen_ratio": pen_ratio,
		"def_amount": def_amount,
		"penalized_w_power": penalized_w_power,
		"stat_name": stat_name,
		"stat_val": stat_val,
		"stat_scaling_contrib": stat_scaling_contrib,
		"unbuffed_attack_power": unbuffed_attack_power,
		"buff_mult": buff_mult,
		"final_attack_power": final_attack_power,
		"skill_mult": skill_mult,
		"mit_ratio": mit_ratio,
		"mit_mult": mit_mult,
		"stance_mult": stance_mult,
		"stance_factor": stance_factor,
		"crit_rate": crit_rate,
		"is_crit": is_crit,
		"crit_dmg_mult": crit_dmg_mult,
		"effect_factor": effect_factor,
		"final_dmg": final_dmg
	}
