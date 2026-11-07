//! U2.2-S1 (`docs/sessions/U2_CLASS.md` C1-C2, FB-10, FB-15): the unit budget. Largest-remainder rounding gives whole
//! points that sum to the budget exactly, ties to the larger fraction and then the earlier stat; the stats at a level
//! are the running total of both share vectors at that level's budget; a class row given as shares reads back as a
//! computed base_stats/growth view, and an integer row read as shares gives its own stats at every level.

use indexmap::IndexMap;
use rpgkit_data::{ClassDef, STATS};
use rpgkit_rules::stats::{CharacterStats, MAX_LEVEL};
use rpgkit_rules::unit_budget::{self, ClassShares, Shares, BASE_BUDGET, GROWTH_BUDGET};
use serde_json::{json, Value};

fn shares(pairs: &[(&str, f64)]) -> Shares {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn class(row: Value) -> ClassDef {
    ClassDef::from_obj(row.as_object().unwrap())
}

/// A deterministic spread of share vectors (an LCG; no shipped numbers).
fn sample_shares(n: usize) -> Vec<Shares> {
    let mut seed: u64 = 0x5eed;
    let mut next = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) % 1000) as f64 + 1.0
    };
    (0..n)
        .map(|_| {
            let raw: Vec<f64> = STATS.iter().map(|_| next()).collect();
            let sum: f64 = raw.iter().sum();
            STATS.iter().zip(raw).map(|(s, r)| (s.to_string(), r / sum)).collect()
        })
        .collect()
}

#[test]
fn largest_remainder_sums_to_the_budget_exactly() {
    for (i, sh) in sample_shares(200).iter().enumerate() {
        for budget in [1, 5, 13, 62, 107] {
            let out = unit_budget::allocate(sh, budget);
            assert_eq!(out.values().sum::<i32>(), budget, "sample {i} budget {budget}: {out:?}");
            for s in STATS {
                let quota = sh[s] * budget as f64;
                assert!((out[s] as f64 - quota).abs() < 1.0, "sample {i}: {s} {} is within 1 of its quota {quota}", out[s]);
            }
        }
    }
}

#[test]
fn whole_quotas_come_back_unchanged_despite_float_noise() {
    // n / 62 x 62 is not always exactly n in floats; the quota snap keeps each whole
    for n in 0..=62 {
        let sh = shares(&[("STR", n as f64 / 62.0), ("DEX", (62 - n) as f64 / 62.0)]);
        let out = unit_budget::allocate(&sh, BASE_BUDGET);
        assert_eq!((out["STR"], out["DEX"]), (n, 62 - n), "{n}/62");
    }
}

#[test]
fn ties_go_to_the_larger_fraction_then_the_earlier_stat() {
    // left 1: the larger fraction wins although it sits later and has the smaller quota
    assert_eq!(unit_budget::largest_remainder(&[2.3, 0.7, 1.0], 4), vec![2, 1, 1]);
    // equal fractions: the earlier position
    assert_eq!(unit_budget::largest_remainder(&[1.5, 1.5, 1.0], 4), vec![2, 1, 1]);
    assert_eq!(unit_budget::largest_remainder(&[0.5, 1.5, 0.5, 0.5], 3), vec![1, 2, 0, 0]);
    // on stats: equal thirds of 1 point go to the first stat in STATS order
    let third = shares(&[("KNOW", 1.0 / 3.0), ("DEX", 1.0 / 3.0), ("STR", 1.0 / 3.0)]);
    let out = unit_budget::allocate(&third, 1);
    assert_eq!((out["STR"], out["DEX"], out["KNOW"]), (1, 0, 0), "share key order does not decide a tie: {out:?}");
}

/// Equal fractions at different whole parts tie (a float fraction would differ in its last bits): base 13/10/6/13/10/10
/// with growth STR .3, DEX .3, CONS .2, PERC .2 gives STR 17.5 and DEX 14.5 at level 4, and the point goes to STR.
#[test]
fn equal_fractions_tie_whatever_their_whole_part() {
    assert_eq!(unit_budget::largest_remainder(&[17.5, 14.5], 32), vec![18, 14]);
    assert_eq!(unit_budget::largest_remainder(&[1.3 + 16.0, 0.3 + 2.0, 0.4], 21), vec![18, 2, 1], "0.4 first, then 17.3 ties 2.3: the earlier");
    let base = [13.0, 10.0, 6.0, 13.0, 10.0, 10.0];
    let sh = ClassShares {
        base: STATS.iter().zip(base).map(|(s, v)| (s.to_string(), v / BASE_BUDGET as f64)).collect(),
        growth: shares(&[("STR", 0.3), ("DEX", 0.3), ("CONS", 0.2), ("PERC", 0.2)]),
    };
    let l4 = unit_budget::stats_at(&sh, 4);
    assert_eq!((l4["STR"], l4["DEX"]), (18, 14), "level 4: {l4:?}");
    assert_eq!(l4.values().sum::<i32>(), unit_budget::budget_at(4));
}

/// Mutation check: the tie cases above tell the rule apart from its likely mistakes (ties to the later position, to the
/// larger quota, or to the smaller fraction), so a change to the rule fails this test.
#[test]
fn the_tie_cases_catch_a_changed_tie_rule() {
    type Rule = fn(&[f64], usize, usize) -> std::cmp::Ordering;
    fn mutant(quotas: &[f64], budget: i32, rule: Rule) -> Vec<i32> {
        let mut out: Vec<i32> = quotas.iter().map(|q| q.floor() as i32).collect();
        let left = budget - out.iter().sum::<i32>();
        let mut order: Vec<usize> = (0..quotas.len()).collect();
        order.sort_by(|&a, &b| rule(quotas, a, b));
        for &i in order.iter().take(left as usize) {
            out[i] += 1;
        }
        out
    }
    let later: Rule = |q, a, b| (q[b] - q[b].floor()).partial_cmp(&(q[a] - q[a].floor())).unwrap().then(b.cmp(&a));
    let by_quota: Rule = |q, a, b| q[b].partial_cmp(&q[a]).unwrap().then(a.cmp(&b));
    let smaller: Rule = |q, a, b| (q[a] - q[a].floor()).partial_cmp(&(q[b] - q[b].floor())).unwrap().then(a.cmp(&b));
    let cases: [(&[f64], i32); 4] = [(&[2.3, 0.7, 1.0], 4), (&[1.5, 1.5, 1.0], 4), (&[0.5, 1.5, 0.5, 0.5], 3), (&[17.5, 14.5], 32)];
    for (name, rule) in [("later position", later), ("larger quota", by_quota), ("smaller fraction", smaller)] {
        let caught = cases.iter().any(|(q, b)| mutant(q, *b, rule) != unit_budget::largest_remainder(q, *b));
        assert!(caught, "ties to the {name} would pass the tie cases");
    }
}

#[test]
fn the_budget_grows_by_the_growth_budget_each_level() {
    assert_eq!(unit_budget::budget_at(1), BASE_BUDGET);
    assert_eq!(unit_budget::budget_at(0), BASE_BUDGET, "below level 1 is level 1");
    for level in 2..=MAX_LEVEL {
        assert_eq!(unit_budget::budget_at(level) - unit_budget::budget_at(level - 1), GROWTH_BUDGET);
    }
}

#[test]
fn stats_at_a_level_spend_that_levels_budget_and_never_lose_a_point() {
    let samples = sample_shares(60);
    for pair in samples.chunks(2) {
        let sh = ClassShares { base: pair[0].clone(), growth: pair[1].clone() };
        for level in 1..=MAX_LEVEL {
            let now = unit_budget::stats_at(&sh, level);
            assert_eq!(now.values().sum::<i32>(), unit_budget::budget_at(level), "level {level}");
            if level > 1 {
                let gain = unit_budget::gain_at(&sh, level);
                assert_eq!(gain.values().sum::<i32>(), GROWTH_BUDGET, "level {level} gains the growth budget");
            }
        }
    }
}

#[test]
fn shares_sum_to_one_or_not() {
    assert!(unit_budget::sums_to_one(&shares(&[("STR", 0.5), ("DEX", 0.25), ("INT", 0.25)])));
    assert!(!unit_budget::sums_to_one(&shares(&[("STR", 0.5), ("DEX", 0.4)])));
    assert!(!unit_budget::sums_to_one(&Shares::new()));
}

/// An integer row (62 at level 1, 5 a level) and the same row given as shares give the same class at every level, and
/// the share row's computed base_stats/growth view is the integer row (growth in the row's key order).
#[test]
fn a_share_row_reads_as_its_integer_row() {
    let base = json!({"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10});
    let growth = json!({"CONS": 2, "STR": 2, "DEX": 1});
    let ints = class(json!({"id": "drill", "base_stats": base, "growth": growth}));
    let as_shares = |m: &Value, budget: i32| -> Value {
        m.as_object().unwrap().iter().map(|(k, v)| (k.clone(), json!(v.as_f64().unwrap() / budget as f64))).collect()
    };
    // the share row goes through JSON text, as a data file would
    let text = json!({"id": "drill", "base_shares": as_shares(&base, BASE_BUDGET), "growth_shares": as_shares(&growth, GROWTH_BUDGET)}).to_string();
    let row = class(serde_json::from_str(&text).unwrap());
    assert!(row.shares.is_some() && ints.shares.is_none());
    assert_eq!(row.base_stats, ints.base_stats, "base view");
    assert_eq!(row.growth.iter().collect::<Vec<_>>(), ints.growth.iter().collect::<Vec<_>>(), "growth view, key order kept");
    let none = IndexMap::new();
    for level in 1..=MAX_LEVEL {
        assert_eq!(CharacterStats::new(&row, level, &none).core, CharacterStats::new(&ints, level, &none).core, "level {level}");
        assert_eq!(unit_budget::stats_at(&ints.unit_shares(), level), ints.stats_at(level), "integer row read as shares, level {level}");
    }
}

/// Growth shares that are not whole points per level still hit each level's budget: the running total rounds once, so
/// the gain moves between stats from level to level.
#[test]
fn fractional_growth_follows_the_running_total() {
    let sh = ClassShares {
        base: shares(&[("STR", 0.2), ("DEX", 0.2), ("INT", 0.1), ("CONS", 0.2), ("PERC", 0.15), ("KNOW", 0.15)]),
        growth: shares(&[("STR", 0.3), ("DEX", 0.3), ("CONS", 0.4)]),
    };
    let obj = |m: &Shares| -> Value { m.iter().map(|(k, v)| (k.clone(), json!(v))).collect() };
    let row = class(json!({"id": "odd", "base_shares": obj(&sh.base), "growth_shares": obj(&sh.growth)}));
    let none = IndexMap::new();
    let mut gains = Vec::new();
    for level in 1..=MAX_LEVEL {
        let core = CharacterStats::new(&row, level, &none).core;
        assert_eq!(core, unit_budget::stats_at(&sh, level));
        for s in STATS {
            let exact = sh.base.get(s).copied().unwrap_or(0.0) * BASE_BUDGET as f64 + sh.growth.get(s).copied().unwrap_or(0.0) * (GROWTH_BUDGET * (level - 1)) as f64;
            assert!((core[s] as f64 - exact).abs() < 1.0, "level {level} {s}: {} near {exact}", core[s]);
        }
        if level > 1 {
            gains.push(unit_budget::gain_at(&sh, level));
        }
    }
    assert!(gains.windows(2).any(|w| w[0] != w[1]), "the per-level gain varies: {gains:?}");
}
