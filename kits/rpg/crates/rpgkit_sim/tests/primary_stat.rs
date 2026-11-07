//! U2.2-S1: the sim's `primary_stat` reads the class's growth shares (its growth identity), not the rounded per-level
//! gain view: an integer row picks as before (first largest in its growth order), a share row by its largest share.

use rpgkit_data::Catalog;
use rpgkit_sim::variants::primary_stat;
use serde_json::json;

#[test]
fn the_primary_stat_is_the_largest_growth_share() {
    let base = json!({"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10});
    let base_shares: serde_json::Value = base.as_object().unwrap().iter().map(|(k, v)| (k.clone(), json!(v.as_f64().unwrap() / 62.0))).collect();
    let cat = Catalog::from_value(&json!({"classes": [
        {"id": "ints", "base_stats": base, "growth": {"CONS": 2, "STR": 2, "PERC": 1}},
        // level 2 gains STR 2, DEX 1, CONS 2 (the .3s round apart), but CONS holds the largest share
        {"id": "shares", "base_shares": base_shares, "growth_shares": {"STR": 0.3, "DEX": 0.3, "CONS": 0.4}},
        {"id": "even", "base_shares": base_shares, "growth_shares": {"DEX": 0.5, "STR": 0.5}}
    ]}));
    assert_eq!(primary_stat(&cat, "ints"), "CONS", "integer row: the first largest in its growth order");
    assert_eq!(primary_stat(&cat, "shares"), "CONS");
    assert_eq!(primary_stat(&cat, "even"), "DEX", "a share tie: the first in the row's growth order");
    assert_eq!(primary_stat(&cat, "ghost"), "");
}
