class_name DamageCalculator
extends RefCounted

# ==============================================================================
# Tactical RPG - Damage Calculator (the one damage formula)
# Every hit in the game goes through compute(): arena attacks (single target and
# area), the planner's damage estimates, the forecast, and the Gear Dojo.
#
#   Attack Power = Weapon Power + Unit Power
#     Weapon Power = (Base + Tier bonus + Training bonus) x (1 - Deficit)
#     Unit Power   = Stat x Scaling       (the weapon's scaling stat; Scaling = the
#                                          class's Proficiency scaling for that gear)
#     x tier power scale (Combatant.power_scale: StatRules.TIERS, minion 0.6 ... boss 1.1)
#     Unarmed      = Weapon 0, Unit Power = STR + DEX
#     Trained fists (main-hand attack, no main weapon, fist proficiency >= 2):
#                    Weapon = UNARMED_BASE_POWER + fist training, Unit Power = STR + DEX,
#                    plus the fist crit bonus (BAL2-L Rule 5)
#     A weapon that doesn't suit the attack (a sword for a ranged shot) adds
#     nothing: Unit Power = DEX (ranged) or STR (melee) x Scaling.
#     Authored strike power (Combatant.power_override > 0) replaces all of it.
#
#   Hit = Attack Power x Focus x Skill x Stance Matchup x Crit
#         x (1 - Guard) x (1 - Armor)                        (minimum 1)
#     Stance Matchup  CombatTypes.get_matchup_multiplier: 1.25 / 0.75 / 1.0
#     Crit            seeded roll: 5% + 1% per DEX above 10, x1.5
#     Guard           Defend skills (Guard -30%, Brace -50%, Fortress -70%)
#     Armor           rating / (rating + 50); rating = gear mitigation + STR / 4
#                     (or the unit's authored armor, Combatant.armor_override)
#                     + active fortify - active expose magnitude (floor 0)
# ==============================================================================

const DEFAULT_UNIT_SCALING: float = 1.5
const ARMOR_K: float = 50.0
const CRIT_BASE: float = 5.0         # % chance with DEX <= 10
const CRIT_PER_DEX: float = 1.0      # % per DEX point above 10
const CRIT_DEX_FLOOR: int = 10
const CRIT_MULT: float = 1.5
const UNARMED_BASE_POWER: int = 15     # trained fists: bare-hand weapon power (GearBudget.UNARMED_VALUE follows it)
const TRAINED_FIST_MIN_LEVEL: int = 2  # fist proficiency level that trains the fists (brawler 3; rogue 1 stays plain)

# ------------------------------------------------------------------------------
# Attack power
# ------------------------------------------------------------------------------

# Unit-power scaling for `attacker` wielding `weapon` (null = unarmed): how easy the
# weapon is to wield for the unit's class (Proficiency scaling, 1.0-2.0; 1.5 default).
static func unit_power_scaling(attacker: Combatant, weapon: Equipment) -> float:
	if weapon != null and attacker.stats != null and attacker.stats.proficiency != null:
		return attacker.stats.proficiency.scaling_of(weapon.weapon_type)
	return DEFAULT_UNIT_SCALING

# The weapon an attack swings: off-hand weapon (or bare hands) for off-hand
# attacks, otherwise the main weapon.
static func weapon_for(attacker: Combatant, use_offhand: bool) -> Equipment:
	if attacker.inventory == null:
		return null
	return attacker.inventory.get_offhand_weapon() if use_offhand else attacker.inventory.get_main_weapon()

const FIST: String = "fist"

# True when `attacker` fights with trained fists: no main weapon, no authored power, and fist
# proficiency >= TRAINED_FIST_MIN_LEVEL (main-hand attacks only; the caller checks the hand).
static func trained_fists(attacker: Combatant) -> bool:
	if attacker.power_override > 0 or weapon_for(attacker, false) != null:
		return false
	if attacker.stats == null or attacker.stats.proficiency == null:
		return false
	return attacker.stats.proficiency.level_of(FIST) >= TRAINED_FIST_MIN_LEVEL

static func _stat(attacker: Combatant, stat_name: String, ctx: Dictionary) -> int:
	return attacker.stats.get_effective_stat(stat_name, ctx) if attacker.stats != null else 10

# Attack power and its parts. Keys: authored, weapon, weapon_fits, base_w_power,
# tier_boost, training_pwr, total_w_power, pen_ratio, def_amount, weapon_power,
# stat_name, stat_val, unit_scaling, unit_power, attack_power, trained_fists.
static func attack_power(attacker: Combatant, is_ranged: bool = false, use_offhand: bool = false) -> Dictionary:
	var w: Equipment = weapon_for(attacker, use_offhand)
	var res: Dictionary = {
		"authored": false, "weapon": w, "weapon_fits": false, "base_w_power": 0, "tier_boost": 0, "training_pwr": 0,
		"total_w_power": 0, "pen_ratio": 0.0, "def_amount": 0, "weapon_power": 0.0,
		"stat_name": "", "stat_val": 0, "unit_scaling": 1.0, "unit_power": 0.0, "attack_power": 0, "trained_fists": false,
	}
	if attacker.power_override > 0:
		res["authored"] = true
		res["stat_name"] = "authored"
		res["unit_power"] = float(attacker.power_override)
		res["attack_power"] = attacker.power_override   # authored power is absolute: tiers don't scale it
		return res
	var ctx: Dictionary = {"weapon": w.weapon_type} if w != null else {}
	if w == null:
		# Bare hands: no weapon power, the unit's own STR + DEX.
		var str_val: int = _stat(attacker, "STR", ctx)
		var dex_val: int = _stat(attacker, "DEX", ctx)
		res["stat_name"] = "STR+DEX"
		res["stat_val"] = str_val + dex_val
		res["unit_power"] = float(str_val + dex_val)
		if not use_offhand and trained_fists(attacker):
			# Trained fists: a weapon-equivalent base plus fist training (no wield penalty, no tiers).
			var training: int = attacker.stats.proficiency.get_weapon_power_bonus(FIST)
			res["trained_fists"] = true
			res["base_w_power"] = UNARMED_BASE_POWER
			res["training_pwr"] = training
			res["total_w_power"] = UNARMED_BASE_POWER + training
			res["weapon_power"] = float(UNARMED_BASE_POWER + training)
	else:
		var fits: bool = (is_ranged and w.is_ranged_weapon()) or (not is_ranged and w.is_melee_weapon())
		res["weapon_fits"] = fits
		var scaling: float = unit_power_scaling(attacker, w)
		res["unit_scaling"] = scaling
		var stat_name: String = w.scaling_stat if fits else ("DEX" if is_ranged else "STR")
		if fits:
			var tier: int = w.get_unlocked_threshold_power_boost(attacker.stats, attacker.stats.proficiency) if attacker.stats != null else 0
			var training: int = attacker.stats.proficiency.get_weapon_power_bonus(w.weapon_type) if attacker.stats != null and attacker.stats.proficiency != null else 0
			var total: int = w.base_power + tier + training
			res["base_w_power"] = w.base_power
			res["tier_boost"] = tier
			res["training_pwr"] = training
			res["total_w_power"] = total
			var pen_ratio: float = 0.0
			if attacker.stats != null:
				var pen: Dictionary = w.get_wield_penalty(attacker.stats, attacker.stats.proficiency)
				if DictUtil.to_bool(pen.get("is_penalized", false)):
					pen_ratio = DictUtil.to_float(pen.get("power_penalty_ratio", 0.0))
					res["def_amount"] = DictUtil.to_int(pen.get("deficit", 0))
			res["pen_ratio"] = pen_ratio
			res["weapon_power"] = float(total) * (1.0 - pen_ratio)
		var stat_val: int = _stat(attacker, stat_name, ctx)
		res["stat_name"] = stat_name
		res["stat_val"] = stat_val
		res["unit_power"] = float(stat_val) * scaling
	res["power_scale"] = attacker.power_scale
	res["attack_power"] = maxi(1, roundi((DictUtil.to_float(res["weapon_power"]) + DictUtil.to_float(res["unit_power"])) * attacker.power_scale))
	return res

static func power_of(attacker: Combatant, is_ranged: bool = false, use_offhand: bool = false) -> int:
	return DictUtil.to_int(attack_power(attacker, is_ranged, use_offhand)["attack_power"])

# ------------------------------------------------------------------------------
# Defence & crits
# ------------------------------------------------------------------------------

# Armor rating: the base rating plus an active fortify magnitude minus an active expose magnitude
# (S12S-L, floor 0). Every caller (compute, planner estimate, forecast, sim stats) reads this one value.
static func armor_rating(target: Combatant) -> int:
	var fortify: int = target.active_magnitude(StatusEffects.FORTIFY)
	var expose: int = target.active_magnitude(StatusEffects.EXPOSE)
	return maxi(0, base_armor_rating(target) + fortify - expose)

# Base armor rating: equipped gear mitigation (incl. unlocked tier mitigation) + STR / 4, unless the unit has authored armor.
static func base_armor_rating(target: Combatant) -> int:
	if target.armor_override >= 0:
		return target.armor_override
	var gear: int = target.inventory.get_total_flat_mitigation(target.stats) if target.inventory != null else 0
	if target.stats == null:
		return gear
	return target.stats.get_mitigation_rating(gear, target.get_weapon_context())

# Damage share removed by `rating` (diminishing: 10 -> 17%, 20 -> 29%, 40 -> 44%).
static func armor_reduction(rating: int) -> float:
	return float(maxi(0, rating)) / (float(maxi(0, rating)) + ARMOR_K)

# Crit chance: DEX-based (CharacterStats) plus training in the wielded weapon type.
static func crit_chance(attacker: Combatant) -> float:
	if attacker.stats == null:
		return CRIT_BASE
	var ctx: Dictionary = attacker.get_weapon_context()
	var chance: float = attacker.stats.get_crit_rate(ctx)
	if attacker.stats.proficiency != null and ctx.has("weapon"):
		chance += attacker.stats.proficiency.crit_bonus(str(ctx["weapon"]))
	elif trained_fists(attacker):
		chance += attacker.stats.proficiency.crit_bonus(FIST)
	return chance

# ------------------------------------------------------------------------------
# A hit
# ------------------------------------------------------------------------------

# One hit of `attacker` on `target`. `rng` rolls the crit (null: no random crit;
# battles pass the resolver's seeded RNG so replays match). `attacker_stance`
# (-1 = current) lets the planner estimate with the stance a skill will use.
# The result's final_dmg already includes guard and armor: apply it with
# Combatant.take_damage(final_dmg, true).
static func compute(
	attacker: Combatant,
	target: Combatant,
	is_ranged: bool,
	use_offhand: bool,
	skill_mult: float = 1.0,
	force_crit: bool = false,
	rng: RandomNumberGenerator = null,
	attacker_stance: int = -1,
	include_guard: bool = true
) -> Dictionary:
	var res: Dictionary = attack_power(attacker, is_ranged, use_offhand)
	var power: int = DictUtil.to_int(res["attack_power"])

	var buff_mult: float = maxf(1.0, attacker.attack_boost_multiplier)
	var stance: CombatTypes.StanceType = attacker.current_stance if attacker_stance < 0 else attacker_stance as CombatTypes.StanceType
	var defender_stance: CombatTypes.StanceType = target.current_stance if target != null else CombatTypes.StanceType.NEUTRAL
	var stance_factor: float = CombatTypes.get_matchup_multiplier(stance, defender_stance)

	var chance: float = crit_chance(attacker)
	var is_crit: bool = force_crit or (rng != null and rng.randf() * 100.0 < chance)
	var crit_mult: float = attacker.stats.get_crit_damage_multiplier() if (is_crit and attacker.stats != null) else (CRIT_MULT if is_crit else 1.0)

	var guard: float = target.defense_mitigation_ratio if (target != null and include_guard) else 0.0
	var rating: int = armor_rating(target) if target != null else 0
	var armor: float = armor_reduction(rating)

	var gross: float = float(power) * buff_mult * skill_mult * stance_factor * crit_mult * (1.0 - guard) * (1.0 - armor)
	res.merge({
		"buff_mult": buff_mult,
		"final_attack_power": maxi(1, roundi(float(power) * buff_mult)),
		"skill_mult": skill_mult,
		"stance_factor": stance_factor,
		"stance_mult": stance_factor - 1.0,
		"crit_rate": chance,
		"is_crit": is_crit,
		"crit_dmg_mult": crit_mult,
		"mit_ratio": guard,
		"mit_mult": 1.0 - guard,
		"armor_rating": rating,
		"armor_ratio": armor,
		"final_dmg": maxi(1, roundi(gross)),
	}, true)
	return res

# The planner's expected hit for `skill`: no crit, no guard (unknown ahead), armor included.
static func expected(attacker: Combatant, target: Combatant, skill: SkillDef, attacker_stance: int = -1) -> int:
	var res: Dictionary = compute(attacker, target, attacker.attack_range > 1 and not skill.is_off_hand(), skill.is_off_hand(), skill.power_for(attacker), false, null, attacker_stance, false)
	return DictUtil.to_int(res["final_dmg"])

# One-line explanation of a result (log hover, forecast tooltips).
static func describe(res: Dictionary) -> String:
	var parts: Array[String] = []
	if DictUtil.to_bool(res.get("authored", false)):
		parts.append("Strike power %d (authored)" % DictUtil.to_int(res["attack_power"]))
	else:
		var weapon_txt: String = "Weapon %.0f" % DictUtil.to_float(res["weapon_power"])
		if DictUtil.to_bool(res.get("trained_fists", false)):
			weapon_txt = "Trained fists %.0f (base %d +%d training)" % [DictUtil.to_float(res["weapon_power"]), DictUtil.to_int(res["base_w_power"]), DictUtil.to_int(res["training_pwr"])]
		elif DictUtil.to_int(res["tier_boost"]) > 0 or DictUtil.to_int(res["training_pwr"]) > 0:
			weapon_txt += " (base %d +%d tier +%d training)" % [DictUtil.to_int(res["base_w_power"]), DictUtil.to_int(res["tier_boost"]), DictUtil.to_int(res["training_pwr"])]
		if DictUtil.to_float(res["pen_ratio"]) > 0.0:
			weapon_txt += " -%d%% deficit" % roundi(DictUtil.to_float(res["pen_ratio"]) * 100.0)
		var unit_txt: String = ("Unit %.0f (%s %d)" % [DictUtil.to_float(res["unit_power"]), str(res["stat_name"]), DictUtil.to_int(res["stat_val"])]) if str(res["stat_name"]) == "STR+DEX" else ("Unit %.0f (%s %d x %.1f)" % [DictUtil.to_float(res["unit_power"]), str(res["stat_name"]), DictUtil.to_int(res["stat_val"]), DictUtil.to_float(res["unit_scaling"])])
		var tier_scale: float = DictUtil.to_float(res.get("power_scale", 1.0), 1.0)
		if is_equal_approx(tier_scale, 1.0):
			parts.append("%s + %s = %d" % [weapon_txt, unit_txt, DictUtil.to_int(res["attack_power"])])
		else:
			parts.append("(%s + %s) x tier %.2f = %d" % [weapon_txt, unit_txt, tier_scale, DictUtil.to_int(res["attack_power"])])
	var mults: Array[String] = []
	if DictUtil.to_float(res.get("buff_mult", 1.0)) > 1.0:
		mults.append("Focus x%.2f" % DictUtil.to_float(res["buff_mult"]))
	if not is_equal_approx(DictUtil.to_float(res.get("skill_mult", 1.0)), 1.0):
		mults.append("skill x%.2f" % DictUtil.to_float(res["skill_mult"]))
	mults.append("stance x%.2f" % DictUtil.to_float(res.get("stance_factor", 1.0)))
	if DictUtil.to_bool(res.get("is_crit", false)):
		mults.append("CRIT x%.1f" % DictUtil.to_float(res["crit_dmg_mult"]))
	if DictUtil.to_float(res.get("mit_ratio", 0.0)) > 0.0:
		mults.append("guard -%d%%" % roundi(DictUtil.to_float(res["mit_ratio"]) * 100.0))
	if DictUtil.to_int(res.get("armor_rating", 0)) > 0:
		mults.append("armor %d -%d%%" % [DictUtil.to_int(res["armor_rating"]), roundi(DictUtil.to_float(res["armor_ratio"]) * 100.0)])
	return "%s x %s = %d" % [parts[0], " x ".join(mults), DictUtil.to_int(res.get("final_dmg", 0))]
