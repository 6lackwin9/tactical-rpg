//! DamageCalculator: the one damage formula (scripts/core/combat/damage_calculator.gd).
//!
//!   Attack Power = (Weapon Power + Unit Power) x tier power scale   (authored power replaces it, unscaled)
//!   Hit = Attack Power x Focus x Skill x Stance x Crit x (1 - min(cap, Guard + Cover)) x (1 - Armor), minimum 1

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
}

pub fn attack_power(u: &Unit, ranged: bool, offhand: bool) -> AttackPower {
    let mut res = AttackPower::default();
    if u.power_override > 0 {
        res.authored = true;
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
            res.unit_power = sd as f64;
            if !offhand && trained_fists(u) {
                res.trained_fists = true;
                res.weapon_power = (UNARMED_BASE_POWER + u.stats.proficiency.power_bonus(FIST)) as f64;
            }
        }
        Some(w) => {
            let fits = (ranged && w.is_ranged_weapon()) || (!ranged && w.is_melee_weapon());
            res.weapon_fits = fits;
            let scaling = u.stats.proficiency.scaling_of(w.weapon_type());
            let stat_name = if fits {
                w.item.scaling_stat.as_str()
            } else if ranged {
                "DEX"
            } else {
                "STR"
            };
            if fits {
                let total = w.item.base_power + w.tier_power_boost(&u.stats) + u.stats.proficiency.power_bonus(w.weapon_type());
                let pen = w.wield_penalty(&u.stats);
                res.weapon_power = total as f64 * (1.0 - if pen.penalized { pen.ratio } else { 0.0 });
            }
            res.unit_power = u.stats.stat(stat_name, ctx) as f64 * scaling;
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
}

impl Default for HitInput {
    fn default() -> Self {
        Self { ranged: false, offhand: false, skill_mult: 1.0, force_crit: false, crit_roll: None, attacker_stance: None, include_guard: true, cover: 0.0 }
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
}

pub fn compute(attacker: &Unit, target: &Unit, h: HitInput) -> Hit {
    let power = power_of(attacker, h.ranged, h.offhand);
    let buff = attacker.attack_boost.max(1.0);
    let stance = h.attacker_stance.unwrap_or(attacker.stance);
    let stance_factor = stance.matchup(target.stance);
    let chance = crit_chance(attacker);
    let is_crit = h.force_crit || h.crit_roll.is_some_and(|r| r * 100.0 < chance);
    let crit_mult = if is_crit { CRIT_MULT } else { 1.0 };
    let guard = if h.include_guard { target.guard } else { 0.0 };
    let guard_term = GUARD_CAP.min(guard + h.cover.max(0.0));
    let rating = armor_rating(target);
    let gross = power as f64 * buff * h.skill_mult * stance_factor * crit_mult * (1.0 - guard_term) * (1.0 - armor_reduction(rating));
    Hit { attack_power: power, is_crit, crit_rate: chance, stance_factor, armor_rating: rating, final_dmg: roundi(gross).max(1) }
}
