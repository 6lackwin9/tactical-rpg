//! U3.2-S5 (docs/sessions/U3_GEAR.md "Carry and burden (C01)", FB-36 (d); U1_STATS "carry ... an option, off by default")
//! on a scratch catalog built here (no shipped data): the party-wide carry of `carry.rs` beside per-unit burden.
//!
//! - A member's carry rating is `STR x 3 + CONS` (its effective stats, gear bonuses in); the party's capacity is
//!   `BASE_CAPACITY` plus every member's rating.
//! - The load is the stash, the worn gear and the battle pouches (item weight); light below `HEAVY_RATIO` of the
//!   capacity, heavy up to it, overloaded above it, with one point per `OVERLOAD_STEP` of weight over (rounded up).
//! - `carry::apply` gives every member that overload: with its worn burden it lowers initiative first and the move rate
//!   only past initiative's floor of 1 (the U1.2-S3c speed split; `move_rate.rs` walks every load).
//! - Off unless applied: building a unit never sets an overload (only `carry::apply` does), so a unit that no carry
//!   evaluation reaches fights at its burden alone.

use rpgkit_data::Catalog;
use rpgkit_rules::carry::{self, Tier, BASE_CAPACITY, HEAVY_RATIO, OVERLOAD_STEP};
use rpgkit_rules::gear::{Equipment, Slot};
use rpgkit_rules::stats::move_rate_curve;
use rpgkit_rules::{build_from, build_unit, Unit, UnitBuild};
use serde_json::json;

fn catalog() -> Catalog {
    let class = |id: &str, str_: i32, cons: i32| json!({"id": id, "name": id, "base_stats": {"STR": str_, "DEX": 10, "INT": 8, "CONS": cons, "PERC": 9, "KNOW": 10}, "growth": {}});
    Catalog::from_value(&json!({
        "classes": [class("brute", 16, 14), class("wisp", 7, 8)],
        "units": [
            {"id": "brute", "name": "Brute", "class_id": "brute", "level": 1, "side": "player"},
            {"id": "wisp", "name": "Wisp", "class_id": "wisp", "level": 1, "side": "player"}
        ],
        "items": [
            {"id": "plate", "name": "Plate", "item_type": "ARMOR", "slot": "BODY", "burden": 2, "weight": 12.0},
            {"id": "belt", "name": "Belt", "item_type": "ACCESSORY", "slot": "ACCESSORY_1", "weight": 1.0, "stat_bonuses": {"STR": 2}},
            {"id": "tonic", "name": "Tonic", "item_type": "CONSUMABLE", "weight": 0.5}
        ]
    }))
}

fn bare(cat: &Catalog, id: &str) -> Unit {
    let mut u = build_unit(cat, &cat.units[id], 0);
    for slot in Slot::ALL {
        u.inventory.unequip(slot);
    }
    u.inventory.pouch.clear();
    u.sync_from_stats_and_gear(cat);
    u
}

fn rating(u: &Unit) -> i32 {
    u.stats.stat("STR", None) * 3 + u.stats.stat("CONS", None)
}

#[test]
fn the_capacity_is_the_base_plus_every_members_str_x3_plus_cons() {
    let cat = catalog();
    let (mut brute, wisp) = (bare(&cat, "brute"), bare(&cat, "wisp"));
    assert_eq!(carry::contribution(&brute), rating(&brute));
    let before = carry::evaluate(&[brute.clone(), wisp.clone()], 0.0);
    assert_eq!(before.capacity, BASE_CAPACITY + rating(&brute) + rating(&wisp));
    assert_eq!(before.members, [("Brute".to_string(), rating(&brute)), ("Wisp".to_string(), rating(&wisp))]);
    // a worn STR bonus raises the rating (effective stats)
    brute.inventory.equip(Equipment::new(&cat.items["belt"]), Slot::Accessory1);
    brute.sync_from_stats_and_gear(&cat);
    assert_eq!(carry::contribution(&brute), rating(&bare(&cat, "brute")) + 2 * 3);
}

#[test]
fn the_load_and_the_tiers_follow_the_capacity() {
    let cat = catalog();
    let mut wisp = bare(&cat, "wisp");
    wisp.inventory.equip(Equipment::new(&cat.items["plate"]), Slot::Body);
    wisp.inventory.add_to_pouch(&cat.items["tonic"]);
    wisp.sync_from_stats_and_gear(&cat);
    let worn = cat.items["plate"].weight + cat.items["tonic"].weight;
    let cap = carry::evaluate(std::slice::from_ref(&wisp), 0.0).capacity as f64;
    for stash in [0.0, cap * HEAVY_RATIO - worn - 0.5, cap * HEAVY_RATIO - worn, cap - worn, cap - worn + 0.5, cap - worn + OVERLOAD_STEP, cap - worn + 2.0 * OVERLOAD_STEP + 0.1] {
        let r = carry::evaluate(std::slice::from_ref(&wisp), stash);
        assert_eq!((r.stash_weight, r.equipped_weight, r.pouch_weight), (stash, cat.items["plate"].weight, cat.items["tonic"].weight));
        assert_eq!(r.load, stash + worn);
        let ratio = r.load / cap;
        let (tier, penalty) = if ratio > 1.0 { (Tier::Overloaded, ((r.load - cap) / OVERLOAD_STEP).ceil() as i32) } else if ratio >= HEAVY_RATIO { (Tier::Heavy, 0) } else { (Tier::Light, 0) };
        assert_eq!((r.tier, r.overload_penalty), (tier, penalty), "stash {stash}");
    }
}

#[test]
fn applied_overload_lowers_initiative_first_with_the_worn_burden() {
    let cat = catalog();
    let mut party = vec![bare(&cat, "brute"), bare(&cat, "wisp")];
    party[1].inventory.equip(Equipment::new(&cat.items["plate"]), Slot::Body);
    party[1].sync_from_stats_and_gear(&cat);
    let free: Vec<(i32, i32, i32)> = party.iter().map(|u| (u.initiative, u.move_rate, u.worn_burden())).collect();
    let cap = carry::evaluate(&party, 0.0).capacity as f64;
    let load_worn = cat.items["plate"].weight;
    for over_steps in [0, 1, 3, 40] {
        let stash = cap - load_worn + over_steps as f64 * OVERLOAD_STEP;
        let r = carry::apply(&cat, &mut party, stash);
        assert_eq!(r.overload_penalty, over_steps, "{over_steps} steps over");
        for (u, &(init0, _, burden)) in party.iter().zip(&free) {
            assert_eq!(u.carry_penalty, over_steps, "every member takes it");
            let dex_base = init0 + burden; // 8 + DEX
            assert_eq!(u.initiative, ((dex_base - burden).max(1) - over_steps).max(1), "{}: initiative first", u.name);
            let excess = (1 - (dex_base - burden - over_steps)).max(0);
            assert_eq!(u.move_rate, move_rate_curve(dex_base - excess), "{}: the move rate only past initiative's floor", u.name);
        }
    }
    carry::apply(&cat, &mut party, 0.0);
    for (u, &(init0, mv0, _)) in party.iter().zip(&free) {
        assert_eq!((u.carry_penalty, u.initiative, u.move_rate), (0, init0, mv0), "{}: back with the load", u.name);
    }
}

#[test]
fn building_a_unit_applies_no_carry() {
    let cat = catalog();
    for id in ["brute", "wisp"] {
        let u = build_from(&cat, UnitBuild::row(&cat.units[id], 0, None)).unit;
        assert_eq!(u.carry_penalty, 0, "{id}: only carry::apply sets an overload");
    }
}
