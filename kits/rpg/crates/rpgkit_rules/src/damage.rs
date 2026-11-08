//! DamageCalculator: the one damage formula (scripts/core/combat/damage_calculator.gd).
//!
//!   Attack Power = (Weapon Power + Unit Power) x tier power scale   (authored power replaces it, unscaled)
//!   Hit = Attack Power x Focus x Skill x Stance x Crit x (1 - min(cap, Guard + Cover)) x (1 - Armor), minimum 1
//!   (RULESET1 A2: an armour-piercing hit under the rule pierce skips the Armor term.)
//!
//! Both run the stage pipeline (`pipeline`, U1.2 slice 1): attack power is the base stage (weapon and unit power added,
//! the tier power scale, rounding point one); the hit adds the amplifiers (Focus, skill, stance, crit) and the
//! mitigation layers (guard and cover, armour) in that order, then the finish (rounding point two, minimum 1).

use crate::gear::Equipment;
use crate::pipeline::{self, Modifier, Op, Stage, Trace};
use crate::stats::roundi;
use crate::status::{self, Rating};
use crate::unit::{Stance, Unit};

pub const DEFAULT_UNIT_SCALING: f64 = 1.5;
pub const ARMOR_K: f64 = 50.0;
pub const CRIT_BASE: f64 = 5.0;
pub const CRIT_PER_PERC: f64 = 1.0;
pub const CRIT_PERC_FLOOR: i32 = 10;
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
    /// U3.2-S3 (FB-33): the weapon is worn untrained (proficiency 0): its power takes the wield terms'
    /// `untrained_power_mult` (`untrained_mult`), stacking with the deficit ratio.
    pub untrained: bool,
    pub untrained_mult: f64,
    pub stat_name: String,
    pub stat_val: i32,
    pub unit_scaling: f64,
    /// The tier power scale the base stage multiplies by (unused by an authored power).
    /// U1.2-S4: the attacker's `tier_power` (the base stage's `Mul`).
    pub tier_power: f64,
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
                    (res.pen_ratio, res.def_amount, res.untrained) = (pen.ratio, pen.deficit, pen.untrained);
                    res.untrained_mult = if pen.untrained { pen.untrained_mult } else { 0.0 };
                }
                res.weapon_power = if pen.penalized { total as f64 * pen.factor() } else { total as f64 };
            }
            res.stat_name = stat_name.to_string();
            res.stat_val = u.stats.stat(stat_name, ctx);
            res.unit_power = res.stat_val as f64 * scaling;
        }
    }
    res.tier_power = u.tier_power;
    let (base, n) = res.base_modifiers();
    res.attack_power = pipeline::DAMAGE.run(&base[..n]).base.unwrap_or(1);
    res
}

impl AttackPower {
    /// The base stage's modifiers and how many of them count: the authored power alone (unscaled), else weapon power
    /// and unit power added, then the tier power scale. A fixed array, so a hit allocates nothing.
    pub fn base_modifiers(&self) -> ([Modifier; 3], usize) {
        if self.authored {
            let unused = || Modifier::new("unused", Stage::Base, Op::Add(0.0));
            return ([Modifier::new("authored", Stage::Base, Op::Add(self.attack_power as f64)), unused(), unused()], 1);
        }
        let mods = [
            Modifier::new("weapon", Stage::Base, Op::Add(self.weapon_power)),
            Modifier::new("unit", Stage::Base, Op::Add(self.unit_power)),
            Modifier::new("tier", Stage::Base, Op::Mul(self.tier_power)),
        ];
        (mods, 3)
    }
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

/// Base rating + the active Armor buffs' magnitudes (fortify) - the active Armor debuffs' (expose), floor 0; W2-5
/// (C09): the statuses are the table's `modifier` rows with `rating` armor, not named ids.
pub fn armor_rating(u: &Unit) -> i32 {
    let raised = u.active_magnitude_where(|s| status::raises_rating(s, Rating::Armor));
    let lowered = u.active_magnitude_where(|s| status::lowers_rating(s, Rating::Armor));
    (base_armor_rating(u) + raised - lowered).max(0)
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
    // U3.2-S4 (U3 Level 2): the worn families' Level-2 crit (0 without one, so nothing changes)
    if u.gear_crit != 0.0 {
        chance += u.gear_crit;
    }
    // HOOKS1 8.2: the trigger effect `crit` (rule triggers; 0 otherwise, so nothing changes)
    if u.trigger_crit != 0.0 {
        chance += u.trigger_crit;
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
    /// U1.2 slice 1: every pipeline step of the hit (base terms, tier, rounding point one, the amplifiers, the
    /// mitigation layers, the finish); filled by `compute_traced` only, empty from `compute`.
    pub trace: Trace,
}

/// One hit, keeping no trace (`Hit.trace` stays empty and allocates nothing): the planner, forecasts, the sim and
/// every resolved hit.
pub fn compute(attacker: &Unit, target: &Unit, h: HitInput) -> Hit {
    hit(attacker, target, h, false, Vec::new())
}

/// `compute`, keeping every pipeline step in `Hit.trace` (for a hit that is shown with its steps).
pub fn compute_traced(attacker: &Unit, target: &Unit, h: HitInput) -> Hit {
    hit(attacker, target, h, true, Vec::new())
}

/// U4.2-S4 (FB-30): `compute_traced` for a hit of `skill`: the `skill` step also names the grant sources of the
/// attacker's potency for it (`Unit::potency_parts`; the skill factor folds them in when the action is planned,
/// `Unit::power_for`, so its value is `h.skill_mult` as before).
pub fn compute_traced_skill(attacker: &Unit, target: &Unit, h: HitInput, skill: &rpgkit_data::SkillDef) -> Hit {
    hit(attacker, target, h, true, attacker.potency_parts(skill))
}

/// U4.2-S4 (FB-30): a heal's pipeline modifiers (SupportRules.heal_amount): the heal base (`heal_base` + the heal
/// stat x `heal_mult`, rounded) in the base stage, then the caster's skill potency in the mult stage (`potency`, its
/// grant sources as parts when `traced`). The HEAL pipeline rounds once, floor 0: `roundi(base x potency)`, as before.
pub fn heal_modifiers(caster: &Unit, skill: &rpgkit_data::SkillDef, traced: bool) -> [Modifier; 2] {
    let stat = if skill.heal_stat.is_empty() { 0 } else { caster.stats.stat(&skill.heal_stat, caster.weapon_context().as_deref()) };
    let base = skill.heal_base + roundi(stat as f64 * skill.heal_mult);
    let potency = if traced { caster.potency_modifier(skill) } else { Modifier::new("potency", Stage::Mult, Op::Mul(caster.potency_for_skill(skill))) };
    [Modifier::new("heal", Stage::Base, Op::Add(base as f64)), potency]
}

/// U4.2-S4: the heal `caster` gives with `skill` (no trace).
pub fn heal(caster: &Unit, skill: &rpgkit_data::SkillDef) -> i32 {
    pipeline::HEAL.run(&heal_modifiers(caster, skill, false)).value
}

/// U4.2-S4: `heal`, keeping every step (the `potency` step names its sources).
pub fn heal_traced(caster: &Unit, skill: &rpgkit_data::SkillDef) -> (i32, Trace) {
    let (out, trace) = pipeline::HEAL.run_traced(&heal_modifiers(caster, skill, true));
    (out.value, trace)
}

/// A2.2-S1 (FB-43, FB-56a): the pipeline and modifiers of damage a unit takes (Combatant.take_damage): with the guard
/// applied (`mitigated` false and `guard` > 0) the amount, then the guard as a mitigation ratio, rounded once with
/// floor 1 (`TAKE_GUARDED`: `max(1, roundi(amount x (1 - guard)))`, as before); otherwise the amount as it is (`PASS`).
/// The modifiers are the first `n` of the array.
pub fn take_damage_modifiers(amount: i32, guard: f64, mitigated: bool) -> (pipeline::Pipeline, [Modifier; 2], usize) {
    let amount = Modifier::new("amount", Stage::Base, Op::Add(amount as f64));
    let guard_mod = Modifier::new("guard", Stage::Mitigation, Op::Reduce(guard));
    if !mitigated && guard > 0.0 {
        (pipeline::TAKE_GUARDED, [amount, guard_mod], 2)
    } else {
        (pipeline::PASS, [amount, guard_mod], 1)
    }
}

/// A2.2-S1: the damage a unit with `guard` takes from `amount` (no trace; `Unit::take_damage` applies it).
pub fn take_damage(amount: i32, guard: f64, mitigated: bool) -> i32 {
    let (pipe, mods, n) = take_damage_modifiers(amount, guard, mitigated);
    pipe.run(&mods[..n]).value
}

/// A2.2-S1: `take_damage`, keeping every step.
pub fn take_damage_traced(amount: i32, guard: f64, mitigated: bool) -> (i32, Trace) {
    let (pipe, mods, n) = take_damage_modifiers(amount, guard, mitigated);
    let (out, trace) = pipe.run_traced(&mods[..n]);
    (out.value, trace)
}

/// A2.2-S1 (FB-56a): the HP a heal of `amount` adds to a unit at `hp` of `max_hp` (Combatant.heal): the amount through
/// `PASS` (the value is the amount), then capped at the missing HP (`hp + gain = min(max_hp, hp + amount)`, as before;
/// the pipeline has no ceiling stage, so the cap is the integer step after it). Returns the gain and the trace.
pub fn heal_gain_traced(hp: i32, max_hp: i32, amount: i32) -> (i32, Trace) {
    let (out, trace) = pipeline::PASS.run_traced(&[Modifier::new("heal", Stage::Base, Op::Add(amount as f64))]);
    (out.value.min(max_hp - hp), trace)
}

/// A2.2-S1: `heal_gain_traced` without the trace.
pub fn heal_gain(hp: i32, max_hp: i32, amount: i32) -> i32 {
    pipeline::PASS.run(&[Modifier::new("heal", Stage::Base, Op::Add(amount as f64))]).value.min(max_hp - hp)
}

/// A1.2-K3 (A1_VALUE section 1, FB-55(2)): the situational terms of an item's damage on one unit: the factor in the
/// mult stage (`splash_mult` on a shaped Assault row's tile off its target, else 1) and, when the ruleset mitigates
/// item damage (`Numbers::item_mitigation`), the target's guard plus cover and its armour. `None` mitigation is the
/// exact port of the pre-K3 flat damage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemHit {
    pub splash: f64,
    /// (guard + cover, the armour rating) of the target, or None (no mitigation stage).
    pub mitigation: Option<(f64, i32)>,
}

impl Default for ItemHit {
    fn default() -> Self {
        Self { splash: 1.0, mitigation: None }
    }
}

/// A1.2-K3: the damage pipeline's modifiers of an item hit (the item's flat `damage` in the base stage, the splash in the
/// mult stage, guard and cover capped at `GUARD_CAP`, then armour in the mitigation stage); the first `n` count.
pub fn item_damage_modifiers(base: i32, h: ItemHit) -> ([Modifier; 4], usize) {
    let (guard_cover, rating) = h.mitigation.unwrap_or((0.0, 0));
    let mods = [
        Modifier::new("item", Stage::Base, Op::Add(base as f64)),
        Modifier::new("splash", Stage::Mult, Op::Mul(h.splash)),
        Modifier::new("guard_cover", Stage::Mitigation, Op::Reduce(GUARD_CAP.min(guard_cover.max(0.0)))),
        Modifier::new("armour", Stage::Mitigation, Op::Reduce(armor_reduction(rating))),
    ];
    (mods, if h.mitigation.is_some() { 4 } else { 2 })
}

/// A1.2-K3 (A1_VALUE section 1, acceptance 1): the damage an item's flat `base` deals through `pipeline::DAMAGE`
/// (minimum 1). Without mitigation and splash it is `base` exactly (the exact port of `w.damage(t, damage, true)`).
pub fn item_damage(base: i32, h: ItemHit) -> i32 {
    let (mods, n) = item_damage_modifiers(base, h);
    pipeline::DAMAGE.run(&mods[..n]).value
}

/// A1.2-K3: `item_damage`, keeping every step.
pub fn item_damage_traced(base: i32, h: ItemHit) -> (i32, Trace) {
    let (mods, n) = item_damage_modifiers(base, h);
    let (out, trace) = pipeline::DAMAGE.run_traced(&mods[..n]);
    (out.value, trace)
}

/// A1.2-K3 (A1_VALUE section 1): the heal an item's flat `heal` gives through `pipeline::HEAL` (one rounding, floor 0;
/// the receiver caps it at its missing HP, `heal_gain`): `heal` exactly.
pub fn item_heal(heal: i32) -> i32 {
    pipeline::HEAL.run(&[Modifier::new("item", Stage::Base, Op::Add(heal as f64))]).value
}

/// A2.2-S2 (FB-56b): ids of the factors a caller multiplies into `HitInput.skill_mult` before the hit (trace only).
pub const FOLD_SKILL: &str = "fold:skill";
pub const FOLD_EXPLOIT: &str = "fold:exploit";
pub const FOLD_FLANK: &str = "fold:flank";
pub const FOLD_SPLASH: &str = "fold:splash";
pub const FOLD_COUNTER: &str = "fold:counter";
pub const FOLD_WEAKEN: &str = "fold:weaken";
pub const FOLD_EMPOWER: &str = "fold:empower";

/// A2.2-S2 (FB-56b): `compute_traced` (or `compute_traced_skill` when `skill` is given) with the factors the caller
/// already multiplied into `h.skill_mult` named as trace-only parts of the `skill` step, after the skill's grant
/// sources: `folded` lists them as `(id, factor)` in the caller's order (`FOLD_*`). The step's op stays
/// `Mul(h.skill_mult)`, so the f64 multiplication order and every number are unchanged.
pub fn compute_traced_folded(attacker: &Unit, target: &Unit, h: HitInput, skill: Option<&rpgkit_data::SkillDef>, folded: &[(&'static str, f64)]) -> Hit {
    let mut parts = skill.map(|s| attacker.potency_parts(s)).unwrap_or_default();
    parts.extend(folded.iter().map(|&(id, value)| pipeline::Part { source: std::borrow::Cow::Borrowed(id), value }));
    hit(attacker, target, h, true, parts)
}

fn hit(attacker: &Unit, target: &Unit, h: HitInput, traced: bool, skill_parts: Vec<pipeline::Part>) -> Hit {
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
    // today's order: power, Focus, skill, stance, crit, guard and cover, armour (f64 products are not associative)
    let rest = [
        Modifier::new("focus", Stage::Mult, Op::Mul(buff)),
        Modifier::new("skill", Stage::Mult, Op::Mul(h.skill_mult)),
        Modifier::new("stance", Stage::Mult, Op::Mul(stance_factor)),
        Modifier::new("crit", Stage::Mult, Op::Mul(crit_mult)),
        Modifier::new("guard_cover", Stage::Mitigation, Op::Reduce(guard_term)),
        Modifier::new("armour", Stage::Mitigation, Op::Reduce(armor_ratio)),
    ];
    let (out, trace) = if traced {
        // every base term, so the trace shows them
        let (base, n) = explained.base_modifiers();
        let [r0, r1, r2, r3, r4, r5] = rest;
        let mods: Vec<Modifier> = base.into_iter().take(n).chain([r0, r1.with_parts(skill_parts), r2, r3, r4, r5]).collect();
        pipeline::DAMAGE.run_traced(&mods)
    } else {
        // the base stage already ran in `attack_power`: its rounded result (an integer, so rounding point one keeps it)
        let [r0, r1, r2, r3, r4, r5] = rest;
        (pipeline::DAMAGE.run(&[Modifier::new("attack_power", Stage::Base, Op::Add(power as f64)), r0, r1, r2, r3, r4, r5]), Trace::default())
    };
    debug_assert_eq!(out.base, Some(power), "the hit's base stage is the attack power");
    Hit {
        attack_power: power,
        is_crit,
        crit_rate: chance,
        stance_factor,
        armor_rating: rating,
        final_dmg: out.value,
        power: explained,
        buff_mult: buff,
        skill_mult: h.skill_mult,
        mit_ratio: guard,
        mit_mult: 1.0 - guard_term,
        armor_ratio,
        crit_mult,
        final_attack_power: roundi(power as f64 * buff),
        armor_pierced: h.ignore_armor,
        trace,
    }
}
