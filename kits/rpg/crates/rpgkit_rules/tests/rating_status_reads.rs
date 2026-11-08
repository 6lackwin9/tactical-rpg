//! W2-5 slice 1 (C09): the damage and unit rules read the rating-modifier statuses through the status table
//! (`status::raises_rating` / `status::lowers_rating`, i.e. `modifies_rating` + `is_buff`) instead of the 6 ids they
//! named (fortify, expose in `armor_rating`; haste, slow in `move_per_block`; weaken, empower in the attack factors).
//! Strict 1-to-1: over every row of the frozen kit status table, each call site's predicate selects exactly the ids the
//! old literal did, and each rule gives the same value as the old literal formula (written out here) with any one row,
//! or every row, active.

use rpgkit_data::Catalog;
use rpgkit_rules::damage::{armor_rating, base_armor_rating};
use rpgkit_rules::status::{self, Rating};
use rpgkit_rules::{build_unit, Unit};
use serde_json::json;

type Read = fn(&str) -> bool;

/// (call site read, the id its old literal named).
fn sites() -> [(&'static str, Read, &'static str); 6] {
    [
        ("armor_rating +", |s| status::raises_rating(s, Rating::Armor), "fortify"),
        ("armor_rating -", |s| status::lowers_rating(s, Rating::Armor), "expose"),
        ("move_per_block +", |s| status::raises_rating(s, Rating::MoveRate), "haste"),
        ("move_per_block -", |s| status::lowers_rating(s, Rating::MoveRate), "slow"),
        ("weaken_factor", |s| status::lowers_rating(s, Rating::AttackPower), "weaken"),
        ("empower_factor", |s| status::raises_rating(s, Rating::AttackPower), "empower"),
    ]
}

fn unit() -> Unit {
    let cat = Catalog::from_value(&json!({"units": [{"id": "dummy", "name": "Dummy", "stat_mode": "fixed", "hp": 40, "speed": 12}]}));
    let mut u = build_unit(&cat, &cat.units["dummy"], 10);
    u.armor_override = 10;
    u
}

/// `u` with exactly the statuses `ids` active, each at magnitude `mag`.
fn with_active(ids: &[&str], mag: i32) -> Unit {
    let mut u = unit();
    for id in ids {
        u.st.active.insert(id.to_string());
        u.st.magnitudes.insert(id.to_string(), mag);
    }
    u
}

/// The old literal formulas, written out.
fn old_armor(u: &Unit) -> i32 {
    (base_armor_rating(u) + u.active_magnitude("fortify") - u.active_magnitude("expose")).max(0)
}

fn old_move(u: &Unit) -> i32 {
    (u.move_rate + u.has_status("haste") as i32 - u.has_status("slow") as i32).max(1)
}

fn old_capped(u: &Unit, id: &str) -> i32 {
    let cap = status::cap_of(id);
    u.st.magnitudes.get(id).copied().unwrap_or(0).clamp(0, if cap > 0 { cap } else { 100 })
}

fn old_weaken(u: &Unit) -> f64 {
    if !u.has_status("weaken") {
        return 1.0;
    }
    1.0 - old_capped(u, "weaken") as f64 / 100.0
}

fn old_empower(u: &Unit) -> f64 {
    if !u.has_status("empower") {
        return 1.0;
    }
    1.0 + old_capped(u, "empower") as f64 / 100.0
}

fn assert_same(u: &Unit, what: &str) {
    assert_eq!(armor_rating(u), old_armor(u), "{what}: armor_rating");
    assert_eq!(u.move_per_block(), old_move(u), "{what}: move_per_block");
    assert_eq!(u.weaken_factor().to_bits(), old_weaken(u).to_bits(), "{what}: weaken_factor");
    assert_eq!(u.empower_factor().to_bits(), old_empower(u).to_bits(), "{what}: empower_factor");
}

/// 1: every call-site predicate matches exactly its old id over every table row (and nothing outside the table).
#[test]
fn each_rating_read_selects_exactly_the_old_status_id() {
    let mut named: Vec<&str> = Vec::new();
    for (site, read, old) in sites() {
        for r in status::ROWS.iter() {
            assert_eq!(read(r.id), r.id == old, "{site}: {} (predicate vs old literal)", r.id);
        }
        assert!(!read("no_such_status") && !read(""), "{site}: an unknown or empty id reads false");
        assert!(status::row_of(old).is_some(), "{site}: {old} is a table row");
        named.push(old);
    }
    named.sort_unstable();
    assert_eq!(named, ["empower", "expose", "fortify", "haste", "slow", "weaken"], "the 6 ids the rules named");
}

/// 2: the two predicates split `modifies_rating` by the row's `kind` (buff raises, debuff lowers).
#[test]
fn raises_and_lowers_split_modifies_rating_by_kind() {
    for r in status::ROWS.iter() {
        for rating in [Rating::AttackPower, Rating::Armor, Rating::MoveRate, Rating::Guard] {
            let m = status::modifies_rating(r.id, rating);
            assert_eq!(status::raises_rating(r.id, rating), m && r.buff, "{}: {}", r.id, rating.id());
            assert_eq!(status::lowers_rating(r.id, rating), m && !r.buff, "{}: {}", r.id, rating.id());
        }
    }
}

/// 3: with no status, any one table row, or every row active (at a magnitude inside and one above the caps), the four
/// rules give the old literal formula's value bit for bit.
#[test]
fn the_rules_match_the_old_literal_formulas_over_every_row() {
    for mag in [0, 7, 30, 150] {
        assert_same(&with_active(&[], mag), "none");
        for r in status::ROWS.iter() {
            assert_same(&with_active(&[r.id], mag), &format!("{} at {mag}", r.id));
        }
        let all: Vec<&str> = status::ROWS.iter().map(|r| r.id).collect();
        assert_same(&with_active(&all, mag), &format!("every row at {mag}"));
    }
}

/// 4: a magnitude held for a status that is not active does not count (the old `has_status` gate).
#[test]
fn an_inactive_magnitude_does_not_count() {
    let mut u = unit();
    for (_, _, id) in sites() {
        u.st.magnitudes.insert(id.to_string(), 40);
    }
    assert_eq!(armor_rating(&u), base_armor_rating(&u));
    assert_eq!(u.move_per_block(), u.move_rate.max(1));
    assert_eq!(u.weaken_factor(), 1.0);
    assert_eq!(u.empower_factor(), 1.0);
    assert_same(&u, "inactive magnitudes");
}
