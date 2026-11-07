//! Party carry (PartyCarry, core/inventory/party_carry.gd): one shared capacity for everything the party owns. Burden is
//! per unit (worn gear lowers that unit's initiative); carry is party-wide:
//! capacity = BASE_CAPACITY + every member's STR x 3 + CONS; load = stash + worn gear + battle pouches (item weight).
//! Light under 75% of capacity, Heavy from 75% to 100% (a warning), Overloaded above it: every member loses 1 point per
//! 10 weight over capacity (rounded up), off initiative first and off the move rate only past initiative's floor of 1
//! (U1.2-S3c).

use crate::unit::Unit;
use rpgkit_data::Catalog;

pub const BASE_CAPACITY: i32 = 30;
pub const HEAVY_RATIO: f64 = 0.75;
pub const OVERLOAD_STEP: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Light,
    Heavy,
    Overloaded,
}

impl Tier {
    pub fn label(self) -> &'static str {
        match self {
            Tier::Light => "LIGHT",
            Tier::Heavy => "HEAVY",
            Tier::Overloaded => "OVERLOADED",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub capacity: i32,
    pub load: f64,
    pub stash_weight: f64,
    pub equipped_weight: f64,
    pub pouch_weight: f64,
    pub ratio: f64,
    pub tier: Tier,
    /// Overloaded: the points every member loses, initiative first (then move rate past initiative's floor; U1.2-S3c).
    pub overload_penalty: i32,
    /// (member name, capacity it adds).
    pub members: Vec<(String, i32)>,
}

/// CharacterStats.get_carry_contribution: STR x 3 + CONS, with the unit's weapon context.
pub fn contribution(u: &Unit) -> i32 {
    let ctx = u.weapon_context();
    u.stats.stat("STR", ctx.as_deref()) * 3 + u.stats.stat("CONS", ctx.as_deref())
}

/// PartyCarry.evaluate: the party's carry report for a stash holding `stash_weight`.
pub fn evaluate(party: &[Unit], stash_weight: f64) -> Report {
    let mut capacity = BASE_CAPACITY;
    let (mut equipped_weight, mut pouch_weight) = (0.0, 0.0);
    let mut members = Vec::new();
    for u in party {
        let c = contribution(u);
        capacity += c;
        members.push((u.name.clone(), c));
        equipped_weight += u.inventory.gear().map(|e| e.item.weight).sum::<f64>();
        pouch_weight += u.inventory.pouch.iter().map(|i| i.weight).sum::<f64>();
    }
    let load = stash_weight + equipped_weight + pouch_weight;
    let ratio = load / capacity.max(1) as f64;
    let (tier, overload_penalty) = if ratio > 1.0 {
        (Tier::Overloaded, ((load - capacity as f64) / OVERLOAD_STEP).ceil() as i32)
    } else if ratio >= HEAVY_RATIO {
        (Tier::Heavy, 0)
    } else {
        (Tier::Light, 0)
    };
    Report { capacity, load, stash_weight, equipped_weight, pouch_weight, ratio, tier, overload_penalty, members }
}

/// PartyCarry.apply: evaluates and gives every member the overload penalty (initiative first; U1.2-S3c).
pub fn apply(cat: &Catalog, party: &mut [Unit], stash_weight: f64) -> Report {
    let report = evaluate(party, stash_weight);
    for u in party.iter_mut() {
        u.carry_penalty = report.overload_penalty;
        u.sync_from_stats_and_gear(cat);
    }
    report
}
