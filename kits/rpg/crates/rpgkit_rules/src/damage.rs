//! DamageCalculator: the one damage formula (scripts/core/combat/damage_calculator.gd).
//!
//!   Attack Power = (Weapon Power + Unit Power) x tier power scale   (authored power replaces it, unscaled)
//!   Hit = Attack Power x Focus x Skill x Stance x Crit x (1 - min(cap, Guard + Cover)) x (1 - Armor), minimum 1
//!   (RULESET1 A2: an armour-piercing hit under the rule pierce skips the Armor term.)

use crate::gear::Equipment;
use crate::stats::roundi;
use crate::unit::{Stance, Unit};

pub const DEFAULT_UNIT_SCALING: f64 = 1.5;
pub const ARMOR_K: f64 = 50.0;
pub const CRIT_BASE: f64 = 5.0;
pub const CRIT_PER_DEX: f64 = 1.0;
pub const CRIT_DEX_FLOOR: i32 = 10;
pub const CRIT_MULT: f64 = 1.5;
pub const UNARMED_BASE_POWER: i32 = 15;
pub const TRAINED_FIST_MIN_LEVEL: i32 = 2;
pub const GUARD_CAP: f64 = 0.9;
pub const FIST: &str = "fist";

pub fn weapon_for(u: &Unit, offhand: bool) -> Option<&Equipment> {
    if offhand {
        u.inventory.offhand_weapon()
    } else {
        u.inventory.main_weapon()
    }
}

pub fn trained_fists(u: &Unit) -> bool {
    u.power_override <= 0 && weapon_for(u, false).is_none() && u.stats.proficiency.level_of(FIST) >= TRAINED_FIST_MIN_LEVEL
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AttackPower {
    pub authored: bool,
    pub weapon_fits: bool,
    pub trained_fists: bool,
    pub weapon_power: f64,
    pub unit_power: f64,
    pub attack_power: i32,
    // The explanation (DamageCalculator.attack_power's keys, for breakdowns): the weapon's name ("" unarmed), its base
    // power, tier and training bonuses and their total, the wield-deficit ratio and amount, the stat used, its value
    // and the unit scaling.
    pub weapon: String,
    pub base_w_power: i32,
    pub tier_boost: i32,
    pub training_pwr: i32,
    pub total_w_power: i32,
    pub pen_ratio: f64,
    pub def_amount: i32,
    pub stat_name: String,
    pub stat_val: i32,
    pub unit_scaling: f64,
}

pub fn attack_power(u: &Unit, ranged: bool, offhand: bool) -> AttackPower {
    let mut res = AttackPower::default();
    if u.power_override > 0 {
        res.authored = true;
        res.weapon = weapon_for(u, offhand).map(|w| w.item.name.clone()).unwrap_or_default();
        res.stat_name = "authored".into();
        res.unit_scaling = 1.0;
        res.unit_power = u.power_override as f64;
        res.attack_power = u.power_override;
        return res;
    }
    let w = weapon_for(u, offhand);
    let ctx = w.map(|w| w.weapon_type().to_string());
    let ctx = ctx.as_deref();
    match w {
        None => {
            let sd = u.stats.stat("STR", ctx) + u.stats.stat("DEX", ctx);
            res.stat_name = "STR+DEX".into();
            res.stat_val = sd;
            res.unit_scaling = 1.0;
            res.unit_power = sd as f64;
            if !offhand && trained_fists(u) {
                let training = u.stats.proficiency.power_bonus(FIST);
                res.trained_fists = true;
                res.base_w_power = UNARMED_BASE_POWER;
                res.training_pwr = training;
                res.total_w_power = UNARMED_BASE_POWER + training;
                res.weapon_power = (UNARMED_BASE_POWER + training) as f64;
            }
        }
        Some(w) => {
            let fits = (ranged && w.is_ranged_weapon()) || (!ranged && w.is_melee_weapon());
            res.weapon_fits = fits;
            res.weapon = w.item.name.clone();
            let scaling = u.stats.proficiency.scaling_of(w.weapon_type());
            res.unit_scaling = scaling;
            let stat_name = if fits {
                w.item.scaling_stat.as_str()
            } else if ranged {
                "DEX"
            } else {
                "STR"
            };
            if fits {
                let (tier, training) = (w.tier_power_boost(&u.stats), u.stats.proficiency.power_bonus(w.weapon_type()));
                let total = w.item.base_power + tier + training;
                let pen = w.wield_penalty(&u.stats);
                (res.base_w_power, res.tier_boost, res.training_pwr, res.total_w_power) = (w.item.base_power, tier, training, total);
                if pen.penalized {
                    (res.pen_ratio, res.def_amount) = (pen.ratio, pen.deficit);
                }
                res.weapon_power = total as f64 * (1.0 - if pen.penalized { pen.ratio } else { 0.0 });
            }
            res.stat_name = stat_name.to_string();
            res.stat_val = u.stats.stat(stat_name, ctx);
            res.unit_power = res.stat_val as f64 * scaling;
        }
    }
    res.attack_power = roundi((res.weapon_power + res.unit_power) * u.power_scale).max(1);
    res
}

pub fn power_of(u: &Unit, ranged: bool, offhand: bool) -> i32 {
    attack_power(u, ranged, offhand).attack_power
}

pub fn base_armor_rating(u: &Unit) -> i32 {
    if u.armor_override >= 0 {
        return u.armor_override;
    }
    let gear = u.inventory.total_flat_mitigation(Some(&u.stats));
    u.stats.mitigation_rating(gear, u.weapon_context().as_deref())
}

/// Base rating + active fortify - active expose, floor 0.
pub fn armor_rating(u: &Unit) -> i32 {
    (base_armor_rating(u) + u.active_magnitude("fortify") - u.active_magnitude("expose")).max(0)
}

pub fn armor_reduction(rating: i32) -> f64 {
    let r = rating.max(0) as f64;
    r / (r + ARMOR_K)
}

pub fn crit_chance(u: &Unit) -> f64 {
    let ctx = u.weapon_context();
    let mut chance = u.stats.crit_rate(ctx.as_deref());
    match &ctx {
        Some(w) => chance += u.stats.proficiency.crit_bonus(w),
        None if trained_fists(u) => chance += u.stats.proficiency.crit_bonus(FIST),
        None => {}
    }
    chance
}

#[derive(Debug, Clone, Copy)]
pub struct HitInput {
    pub ranged: bool,
    pub offhand: bool,
    pub skill_mult: f64,
    /// true forces a crit; otherwise `crit_roll` (a uniform [0, 1) draw) rolls it, and None means no crit.
    pub force_crit: bool,
    pub crit_roll: Option<f64>,
    /// Stance to attack with (None = the attacker's current stance).
    pub attacker_stance: Option<Stance>,
    pub include_guard: bool,
    pub cover: f64,
    /// RULESET1 A2 (rule pierce, skill flag `armor_piercing`): the armour term is 1.0 (guard and cover still apply).
    pub ignore_armor: bool,
}

impl Default for HitInput {
    fn default() -> Self {
        Self { ranged: false, offhand: false, skill_mult: 1.0, force_crit: false, crit_roll: None, attacker_stance: None, include_guard: true, cover: 0.0, ignore_armor: false }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub attack_power: i32,
    pub is_crit: bool,
    pub crit_rate: f64,
    pub stance_factor: f64,
    pub armor_rating: i32,
    pub final_dmg: i32,
    // The explanation (DamageCalculator.compute's keys): the attack power's own explanation, the Focus multiplier,
    // the skill multiplier, the guard ratio and the guard-and-cover multiplier, the armor's share, the crit multiplier and the attack power
    // after Focus.
    pub power: AttackPower,
    pub buff_mult: f64,
    pub skill_mult: f64,
    pub mit_ratio: f64,
    pub mit_mult: f64,
    pub armor_ratio: f64,
    pub crit_mult: f64,
    pub final_attack_power: i32,
    /// RULESET1 A2: the hit ignored the armour ("armour pierced"; `armor_rating` still names the target's rating).
    pub armor_pierced: bool,
}

pub fn compute(attacker: &Unit, target: &Unit, h: HitInput) -> Hit {
    let explained = attack_power(attacker, h.ranged, h.offhand);
    let power = explained.attack_power;
    let buff = attacker.attack_boost.max(1.0);
    let stance = h.attacker_stance.unwrap_or(attacker.stance);
    let stance_factor = stance.matchup(target.stance);
    let chance = crit_chance(attacker);
    let is_crit = h.force_crit || h.crit_roll.is_some_and(|r| r * 100.0 < chance);
    let crit_mult = if is_crit { CRIT_MULT } else { 1.0 };
    let guard = if h.include_guard { target.guard } else { 0.0 };
    let guard_term = GUARD_CAP.min(guard + h.cover.max(0.0));
    let rating = armor_rating(target);
    let armor_ratio = if h.ignore_armor { 0.0 } else { armor_reduction(rating) };
    let gross = power as f64 * buff * h.skill_mult * stance_factor * crit_mult * (1.0 - guard_term) * (1.0 - armor_ratio);
    Hit {
        attack_power: power,
        is_crit,
        crit_rate: chance,
        stance_factor,
        armor_rating: rating,
        final_dmg: roundi(gross).max(1),
        power: explained,
        buff_mult: buff,
        skill_mult: h.skill_mult,
        mit_ratio: guard,
        mit_mult: 1.0 - guard_term,
        armor_ratio,
        crit_mult,
        final_attack_power: roundi(power as f64 * buff),
        armor_pierced: h.ignore_armor,
    }
}
