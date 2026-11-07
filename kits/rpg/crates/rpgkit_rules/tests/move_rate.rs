//! U1.2-S3c (docs/sessions/U1_STATS.md "initiative and move rate from DEX", feasibility 4) on a scratch catalog built
//! here (no shipped data): the old speed splits into initiative and move rate. Initiative keeps the old formula,
//! `max(1, max(1, 8 + DEX - burden) - carry overload)`. The move rate is the old move curve
//! `clamp(floor((v + 2) / 5), 1, 4)` on `8 + DEX - excess`, where `excess = max(0, 1 - (8 + DEX - burden - overload))`:
//! burden and overload lower initiative first and the move rate only past initiative's floor of 1. A fixed-stat row's
//! `speed` is its initiative and its move rate is the curve on it, as before.

use rpgkit_data::Catalog;
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::stats::move_rate_curve;
use rpgkit_rules::{build_unit, Unit};
use serde_json::json;

/// The old move-per-block base, written out (the curve speed used before the split).
fn old_curve(speed: i32) -> i32 {
    ((speed + 2) / 5).clamp(1, 4)
}

fn catalog() -> Catalog {
    let fixed = |id: &str, speed: i32| json!({"id": id, "name": id, "class_id": "drill", "stat_mode": "fixed", "hp": 100, "speed": speed, "attack_range": 1, "side": "enemy"});
    let mut units = vec![json!({"id": "probe", "name": "Probe", "class_id": "drill", "level": 1, "side": "player"})];
    units.extend((1..=24).map(|s| fixed(&format!("fixed_{s}"), s)));
    Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 14}, "growth": {}}],
        "units": units,
        "items": [{"id": "plate", "name": "Plate", "item_type": "ARMOR", "slot": "BODY", "burden": 1}]
    }))
}

fn dex(u: &Unit) -> i32 {
    u.stats.stat("DEX", u.weapon_context().as_deref())
}

/// The probe wearing plate of `burden`, with carry overload `overload`, synced.
fn probe(cat: &Catalog, burden: i32, overload: i32) -> Unit {
    let mut u = build_unit(cat, &cat.units["probe"], 0);
    if burden > 0 {
        let mut item = cat.items["plate"].clone();
        item.burden = burden;
        u.inventory.equip(Equipment { item }, Slot::Body);
    }
    u.carry_penalty = overload;
    u.sync_from_stats_and_gear(cat);
    u
}

#[test]
fn the_curve_is_the_old_move_curve() {
    for v in -5..=40 {
        assert_eq!(move_rate_curve(v), old_curve(v).max(1), "value {v}");
    }
}

#[test]
fn initiative_keeps_the_old_speed_formula() {
    let cat = catalog();
    for (burden, overload) in [(0, 0), (1, 0), (5, 0), (0, 3), (4, 2), (17, 0), (20, 0), (12, 9), (30, 4)] {
        let u = probe(&cat, burden, overload);
        assert_eq!(u.worn_burden(), burden, "the plate's burden is worn as is");
        let old = ((8 + dex(&u) - burden).max(1) - overload).max(1);
        assert_eq!(u.initiative, old, "burden {burden}, overload {overload}");
    }
}

#[test]
fn the_move_rate_ignores_load_until_initiative_hits_its_floor_then_spills() {
    let cat = catalog();
    let free = 8 + dex(&probe(&cat, 0, 0));
    let full = move_rate_curve(free);
    for load in 0..=free + 30 {
        // the same load as burden alone, or split between burden and carry overload
        for (burden, overload) in [(load, 0), (load / 2, load - load / 2)] {
            let u = probe(&cat, burden, overload);
            let raw = free - load;
            let excess = (1 - raw).max(0);
            assert_eq!(u.move_rate, move_rate_curve(free - excess), "load {load} ({burden} + {overload})");
            if raw >= 1 {
                assert_eq!((u.move_rate, u.initiative), (full, raw), "load {load}: initiative takes it all, the move rate none");
            } else {
                assert_eq!(u.initiative, 1, "load {load}: initiative at its floor");
                assert_eq!(u.move_rate, move_rate_curve(free - (1 - raw)), "load {load}: the excess spills");
            }
            assert_eq!(u.move_per_block(), u.move_rate, "no haste or slow");
        }
    }
    assert!(move_rate_curve(free - 30) < full, "a large enough excess lowers the move rate");
}

#[test]
fn a_fixed_stat_row_moves_as_before() {
    let cat = catalog();
    for s in 1..=24 {
        let u = build_unit(&cat, &cat.units[&format!("fixed_{s}")], 0);
        assert_eq!((u.initiative, u.move_rate, u.move_per_block()), (s, old_curve(s).max(1), old_curve(s).max(1)), "speed {s}");
    }
}
