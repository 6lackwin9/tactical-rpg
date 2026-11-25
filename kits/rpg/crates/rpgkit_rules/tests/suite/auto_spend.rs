//! U2.2-S3 (FB-46): the default free spend (`stats::auto_spend`) weighs the class growth shares, not level 2's rounded
//! gain: each stat with a share gets the floor of its proportional part, heaviest first (ties in stat order), the
//! points left go one each in that order; with no share at all every stat weighs the same. Shares are compared in
//! billionths, so the writer's 15 digits never change a result. Expectations are worked from the rule on made-up rows.

use indexmap::IndexMap;
use rpgkit_data::{ClassDef, STATS};
use rpgkit_rules::stats::auto_spend;
use rpgkit_rules::unit_budget::Shares;
use serde_json::json;

fn shares(pairs: &[(&str, f64)]) -> Shares {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn spend(pairs: &[(&str, i32)]) -> IndexMap<String, i32> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[test]
fn points_follow_the_shares_heaviest_first() {
    let g = shares(&[("INT", 0.2), ("STR", 0.5), ("DEX", 0.3)]);
    // 10 points: floors 5 / 3 / 2, nothing left
    assert_eq!(auto_spend(&g, 10), spend(&[("STR", 5), ("DEX", 3), ("INT", 2)]));
    // 4 points: floors 2 / 1 / 0, one left to the heaviest (STR)
    assert_eq!(auto_spend(&g, 4), spend(&[("STR", 3), ("DEX", 1)]));
    // ties go in stat order: DEX before CONS
    let tie = shares(&[("CONS", 0.5), ("DEX", 0.5)]);
    assert_eq!(auto_spend(&tie, 3), spend(&[("DEX", 2), ("CONS", 1)]));
    assert!(auto_spend(&g, 0).is_empty() && auto_spend(&g, -2).is_empty());
}

#[test]
fn no_shares_spread_over_every_stat() {
    let out = auto_spend(&Shares::new(), 8);
    assert_eq!(out.values().sum::<i32>(), 8);
    assert_eq!(out.keys().map(String::as_str).collect::<Vec<_>>(), STATS, "every stat, in stat order");
    assert!(out.values().all(|v| *v == 1 || *v == 2));
}

#[test]
fn float_noise_in_a_share_changes_nothing() {
    // 0.1 + 0.2 is not 0.3 in floats, and the writer keeps 15 digits: both still tie with an exact 0.3
    let exact = shares(&[("STR", 0.3), ("DEX", 0.3), ("INT", 0.4)]);
    let noisy = shares(&[("STR", 0.1 + 0.2), ("DEX", 0.299999999999999), ("INT", 0.4)]);
    for points in 0..=40 {
        assert_eq!(auto_spend(&noisy, points), auto_spend(&exact, points), "{points} points");
    }
}

#[test]
fn a_share_row_spends_by_its_shares_not_by_level_two_gain() {
    // growth shares that are not whole points a level (1.5 / 1.5 / 1.5 / 0.5): level 2's rounded gain (the growth
    // view) has no KNOW, but the spend follows the shares, so KNOW gets its part
    let row = json!({"id": "drift", "base_shares": {"STR": 0.25, "DEX": 0.25, "INT": 0.25, "CONS": 0.25},
        "growth_shares": {"STR": 0.3, "DEX": 0.3, "INT": 0.3, "KNOW": 0.1}});
    let c = ClassDef::from_obj(row.as_object().unwrap());
    let view: i32 = c.growth.values().sum();
    assert_eq!(view, 5, "the growth view is one level's whole points");
    assert!(c.growth.get("KNOW").is_none(), "level 2 gains no KNOW in the view: {:?}", c.growth);
    // 10 points by the shares: floors 3 / 3 / 3 / 1, nothing left
    assert_eq!(auto_spend(&c.unit_shares().growth, 10), spend(&[("STR", 3), ("DEX", 3), ("INT", 3), ("KNOW", 1)]));
}
