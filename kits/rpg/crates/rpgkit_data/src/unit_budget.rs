//! The unit budget (U2.2-S1, `docs/sessions/U2_CLASS.md` C1-C2, FB-10, FB-15): one stat budget per level for every
//! class (62 at level 1, +5 each level after), and each class's share of it per stat. A class has two share vectors:
//! `base_shares` (its identity at level 1) and `growth_shares` (its identity in growth). The stats at a level are the
//! running total `base_share x 62 + growth_share x 5 x (level - 1)` per stat, rounded by largest remainder so they sum
//! to the level's budget exactly (ties: the larger fraction, then stat order).
//!
//! Not the gear budget (`rpgkit_rules::budget`). The free spend pool stays separate (`rpgkit_rules::stats::spend_pool`).
//! The stat weights are 1 in this slice, so the budget is the plain stat total.

use crate::STATS;
use indexmap::IndexMap;

/// The level 1 budget every class shares (the base stats total).
pub const BASE_BUDGET: i32 = 62;
/// The budget every class gains per level after the first (the growth total).
pub const GROWTH_BUDGET: i32 = 5;
/// How far a share vector's sum may be from 1 and still count as 1.
pub const SHARE_SUM_TOLERANCE: f64 = 1e-6;
/// Quotas are counted in billionths of a point before rounding, so float noise (13/62 x 62, or the last bits of a
/// fraction) never decides a remainder or a tie.
const QUOTA_TICKS: i64 = 1_000_000_000;

/// Stat -> share of a budget (0-1; a full vector sums to 1).
pub type Shares = IndexMap<String, f64>;

/// The two share vectors of a class.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClassShares {
    pub base: Shares,
    pub growth: Shares,
}

/// The unit budget at `level` (level 1 and below = the base budget).
pub fn budget_at(level: i32) -> i32 {
    BASE_BUDGET + (level.max(1) - 1) * GROWTH_BUDGET
}

/// Largest-remainder rounding: whole numbers, one per quota, summing to `budget`. Each gets the floor of its quota;
/// the points left go one each to the largest fractions, ties to the earlier position. A budget below the floors' sum
/// takes nothing back (the check refuses shares that do not sum to 1, so it does not happen on checked data).
pub fn largest_remainder(quotas: &[f64], budget: i32) -> Vec<i32> {
    // ranked on integers (quota in billionths): a float fraction depends on the whole part (17.5 - 17 and 14.5 - 14
    // differ in the last bits), so equal fractions would not tie
    let ticks: Vec<i64> = quotas.iter().map(|q| (q * QUOTA_TICKS as f64).round() as i64).collect();
    let mut out: Vec<i32> = ticks.iter().map(|k| k.div_euclid(QUOTA_TICKS) as i32).collect();
    let left = budget - out.iter().sum::<i32>();
    let mut order: Vec<usize> = (0..ticks.len()).collect();
    let frac = |i: usize| ticks[i].rem_euclid(QUOTA_TICKS);
    order.sort_by(|&a, &b| frac(b).cmp(&frac(a)).then(a.cmp(&b)));
    for &i in order.iter().cycle().take(if order.is_empty() { 0 } else { left.max(0) as usize }) {
        out[i] += 1;
    }
    out
}

/// `budget` split over the six stats by `shares` (stat order; a missing stat has share 0).
pub fn allocate(shares: &Shares, budget: i32) -> IndexMap<String, i32> {
    let quotas: Vec<f64> = STATS.iter().map(|s| share(shares, s) * budget as f64).collect();
    STATS.iter().zip(largest_remainder(&quotas, budget)).map(|(s, v)| (s.to_string(), v)).collect()
}

/// The stats of a class at `level` (clamped to 1+): the running total of both share vectors, rounded once.
pub fn stats_at(shares: &ClassShares, level: i32) -> IndexMap<String, i32> {
    let gained = (level.max(1) - 1) as f64 * GROWTH_BUDGET as f64;
    let quotas: Vec<f64> =
        STATS.iter().map(|s| share(&shares.base, s) * BASE_BUDGET as f64 + share(&shares.growth, s) * gained).collect();
    STATS.iter().zip(largest_remainder(&quotas, budget_at(level))).map(|(s, v)| (s.to_string(), v)).collect()
}

/// The points a level gains (`stats_at(level) - stats_at(level - 1)`), keyed in the order of `shares.growth`, zeros
/// left out: the per-level growth view.
pub fn gain_at(shares: &ClassShares, level: i32) -> IndexMap<String, i32> {
    let now = stats_at(shares, level);
    let before = stats_at(shares, level - 1);
    let mut keys: Vec<&str> = shares.growth.keys().map(String::as_str).filter(|k| STATS.contains(k)).collect();
    keys.extend(STATS.iter().copied().filter(|s| !shares.growth.contains_key(*s)));
    keys.into_iter().map(|s| (s.to_string(), now[s] - before[s])).filter(|(_, v)| *v != 0).collect()
}

/// `points` as shares of `budget` (`points / budget`), keyed in the map's order: today's integer rows read as shares.
pub fn shares_of(points: &IndexMap<String, i32>, budget: i32) -> Shares {
    points.iter().map(|(k, &v)| (k.clone(), v as f64 / budget as f64)).collect()
}

/// Whether a share vector sums to 1 (within `SHARE_SUM_TOLERANCE`).
pub fn sums_to_one(shares: &Shares) -> bool {
    (shares.values().sum::<f64>() - 1.0).abs() <= SHARE_SUM_TOLERANCE
}

fn share(shares: &Shares, stat: &str) -> f64 {
    shares.get(stat).copied().unwrap_or(0.0)
}
