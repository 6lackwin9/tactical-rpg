//! S1.2-S2 and S1.2-S3 (docs/sessions/S1_STATUS.md sections 1-4): the status rules read the rows' behaviour primitives
//! where one carries the meaning, the four status numbers are ruleset numbers, and `try_inflict` honours `resistable`.
//! Expectations come from S1 (stagger locks blocks 1 and 4, stun 1 and 2, root none; the tick formulas; one
//! resistance formula for resistable statuses) and from the ruleset defaults, never from shipped data.

use aegis_core::rng::GodotRng;
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{self, Numbers, Ruleset, FIELD_RULES};
use rpgkit_rules::stats::roundi;
use rpgkit_rules::status::{self, Primitive};
use rpgkit_rules::{build_unit, Unit};
use serde_json::json;

const HP: i32 = 40;

fn unit() -> Unit {
    let cat = Catalog::from_value(&json!({"units": [{"id": "dummy", "name": "Dummy", "stat_mode": "fixed", "hp": HP}]}));
    build_unit(&cat, &cat.units["dummy"], 10)
}

fn row(v: serde_json::Value) -> Ruleset {
    Ruleset::from_row(v.as_object().unwrap())
}

/// S1.2-S2: the block locks come from the `block_lock` rows' `lock_blocks` (S1 section 1); every other row locks none.
#[test]
fn the_block_locks_are_the_block_lock_rows_blocks() {
    assert_eq!(status::blocked_blocks("stagger"), [1, 4]);
    assert_eq!(status::blocked_blocks("stun"), [1, 2]);
    assert!(status::blocked_blocks("root").is_empty(), "root locks Move, not blocks");
    assert!(status::blocked_blocks("no_such_status").is_empty());
    for r in status::ROWS.iter().filter(|r| !r.has_primitive(Primitive::BlockLock)) {
        assert!(r.lock_blocks.is_empty() && status::blocked_blocks(r.id).is_empty(), "{}: no block_lock, no blocks", r.id);
    }
}

/// S1.2-S2: the round-end ticks are the `tick` rows in the old order (bleed, poison, burn; then regen), and a flat tick
/// reads as HP while a flat modifier reads as armor.
#[test]
fn the_tick_rows_keep_the_round_end_order_and_units() {
    assert_eq!(status::tick_rows(false).map(|r| r.id).collect::<Vec<_>>(), ["bleed", "poison", "burn"]);
    assert_eq!(status::tick_rows(true).map(|r| r.id).collect::<Vec<_>>(), ["regen"]);
    assert_eq!(status::format_magnitude("poison", 3), "3 HP");
    assert_eq!(status::format_magnitude("fortify", 4), "4 armor");
    assert_eq!(status::format_magnitude("expose", 4), "4 armor");
    assert_eq!(status::format_magnitude("burn", 5), "5% max HP");
    assert_eq!(status::format_magnitude("weaken", 20), "20%");
}

/// S1.2-S2/S3: each tick's amount by its formula; bleed reads the ruleset's `bleed_pct`.
#[test]
fn tick_amounts_follow_their_formulas_and_bleed_reads_bleed_pct() {
    let mut u = unit();
    for (st, mag) in [("bleed", 0), ("poison", 3), ("burn", 5), ("regen", 4)] {
        status::grant_now(&mut u, st, 2, mag, "");
    }
    let tick = |id: &str, n: &Numbers| status::tick_amount(&u, status::row_of(id).unwrap(), n);
    let d = Numbers::default();
    assert_eq!(tick("bleed", &d), roundi(HP as f64 * status::BLEED_PCT).max(1));
    assert_eq!(tick("poison", &d), 3, "flat: the magnitude");
    assert_eq!(tick("burn", &d), roundi(HP as f64 * 5.0 / 100.0).max(1));
    assert_eq!(tick("regen", &d), roundi(HP as f64 * 4.0 / 100.0).max(1));
    let wide = Numbers { bleed_pct: 0.25, ..Numbers::default() };
    assert_eq!(tick("bleed", &wide), roundi(HP as f64 * 0.25).max(1));
    assert_eq!(status::bleed_damage(&u, &wide), tick("bleed", &wide));
}

/// S1.2-S3: the four numbers default to the old constants, read from a row, and show as row fields.
#[test]
fn the_status_numbers_are_ruleset_numbers_with_the_old_defaults() {
    let d = Numbers::default();
    assert_eq!((d.bleed_pct, d.counter_mult, d.debt_stagger, d.debt_stun), (status::BLEED_PCT, status::COUNTER_MULT, status::DEBT_STAGGER, status::DEBT_STUN));
    assert_eq!((status::BLEED_PCT, status::COUNTER_MULT), (0.06, 0.5), "S1 section 3 defaults");
    assert_eq!(Ruleset::classic().numbers.bleed_pct, status::BLEED_PCT);
    let r = row(json!({"id": "t", "rules": [], "bleed_pct": 0.1, "counter_mult": 1.0, "debt_stagger": 2, "debt_stun": 4}));
    assert_eq!((r.numbers.bleed_pct, r.numbers.counter_mult, r.numbers.debt_stagger, r.numbers.debt_stun), (0.1, 1.0, 2, 4));
    assert_eq!(r.numbers.field("bleed_pct").as_deref(), Some("0.1"));
    assert_eq!(r.numbers.field("debt_stun").as_deref(), Some("4"));
    for key in ["bleed_pct", "counter_mult", "debt_stagger", "debt_stun"] {
        assert!(ruleset::field_rule_of(key).is_some_and(|f| f.needs.is_empty()), "{key}: a field rule, read whatever the rules");
        assert!(ruleset::PROMOTED.iter().any(|(_, k)| *k == key), "{key}: promoted");
    }
    assert_eq!(ruleset::field_rule_of("bleed_pct").unwrap().range, Some((0.01, 0.25)));
    assert_eq!(ruleset::field_rule_of("counter_mult").unwrap().range, Some((0.10, 1.50)));
    assert!(r.notes().errors.is_empty(), "in range: {:?}", r.notes());
}

/// S1.2-S3: a row-only number outside its range is an error; at either bound it is not (bleed_pct 0.01..0.25,
/// counter_mult 0.10..1.50 from S1 section 3; debt_stagger and debt_stun 1..10 from S2 section 3).
#[test]
fn a_status_number_outside_its_range_is_an_error() {
    for f in FIELD_RULES.iter() {
        let Some((lo, hi)) = f.range else { continue };
        let span = hi - lo;
        for (v, bad) in [(lo, false), (hi, false), (lo - span, true), (hi + span, true)] {
            let n = row(json!({"id": "t", "rules": [], f.key: v})).notes();
            assert_eq!(n.errors.iter().any(|e| e.starts_with(&format!("{} ", f.key)) && e.contains("is outside")), bad, "{} {v}: {n:?}", f.key);
        }
    }
}

/// S1.2-S3: the ranges are the specs' (S2 section 3 for the debt numbers).
#[test]
fn the_status_number_ranges_are_the_specs() {
    let range = |k: &str| ruleset::field_rule_of(k).and_then(|f| f.range);
    assert_eq!([range("bleed_pct"), range("counter_mult")], [Some((0.01, 0.25)), Some((0.10, 1.50))]);
    assert_eq!([range("debt_stagger"), range("debt_stun")], [Some((1.0, 10.0)); 2]);
}

/// S1.2-S3: a stagger threshold above the stun one is an error (a debt stuns first, so it would never stagger); equal
/// or below is not.
#[test]
fn a_stagger_threshold_above_the_stun_one_is_an_error() {
    let order = |stagger: i32, stun: i32| row(json!({"id": "t", "rules": [], "debt_stagger": stagger, "debt_stun": stun})).notes().errors;
    assert_eq!(order(6, 5), ["debt_stagger 6 is above debt_stun 5: a debt stuns first, so it never staggers."]);
    assert!(order(5, 5).is_empty() && order(2, 9).is_empty(), "{:?}", order(5, 5));
    assert!(Ruleset::classic().notes().errors.is_empty(), "the defaults 3 and 5");
}

/// S1.2-S3: the AP debt thresholds are the ruleset's; roll-over applies them.
#[test]
fn the_ap_debt_statuses_read_the_ruleset_thresholds() {
    let d = Numbers::default();
    assert_eq!(status::debt_status(d.debt_stagger - 1, &d), "");
    assert_eq!(status::debt_status(d.debt_stagger, &d), "stagger");
    assert_eq!(status::debt_status(d.debt_stun, &d), "stun");
    let low = Numbers { debt_stagger: 1, debt_stun: 2, ..Numbers::default() };
    assert_eq!(status::debt_status(1, &low), "stagger");
    assert_eq!(status::debt_status(2, &low), "stun");
    let mut u = unit();
    u.ap = -1;
    assert!(!status::roll_over(&mut u.clone(), 0, &d).contains(&"stagger".to_string()), "a debt of 1 is below the default");
    assert!(status::roll_over(&mut u, 0, &low).contains(&"stagger".to_string()), "the lowered threshold staggers");
}

/// S1.2-S3: a row that is not `resistable` lands without a roll (no draw, chance 100); a resistable one still rolls.
#[test]
fn try_inflict_honours_resistable() {
    let nonres = status::ROWS.iter().find(|r| !r.resistable).expect("a non-resistable row").id;
    let res = status::ROWS.iter().find(|r| r.resistable).expect("a resistable row").id;
    let mut u = unit();
    assert_eq!(status::apply_chance(&u, nonres, 0), 100);
    assert_eq!(status::apply_chance(&u, res, 0), 0, "potency 0 never lands a resistable status");
    let mut rng = GodotRng::new(7);
    let mut twin = rng.clone();
    assert!(status::try_inflict(&mut u, nonres, 0, Some(&mut rng), 1, 0, "t"));
    assert!(u.st.pending.contains(nonres), "queued for next round");
    assert_eq!(rng.randi_range(1, 100), twin.randi_range(1, 100), "no draw for a non-resistable status");
    assert!(!status::try_inflict(&mut u, res, 0, Some(&mut rng), 1, 0, "t"));
    assert!(status::resistable("no_such_status"), "an id with no row rolls as before");
}

/// F2.2-S1 (FB-58b): the debt statuses are ruleset values with the old ids as defaults (`stagger`, `stun`), read from a
/// row (empty reads as the default), shown as row fields, read whatever the rules; an id no status row has is an error.
#[test]
fn the_debt_status_ids_are_ruleset_values() {
    let d = Numbers::default();
    assert_eq!((d.debt_stagger_status.as_str(), d.debt_stun_status.as_str()), (status::DEBT_STAGGER_STATUS, status::DEBT_STUN_STATUS));
    assert_eq!((status::DEBT_STAGGER_STATUS, status::DEBT_STUN_STATUS), ("stagger", "stun"), "F section 2 defaults");
    let r = row(json!({"id": "t", "rules": [], "debt_stagger_status": "root", "debt_stun_status": "stagger"}));
    assert_eq!((r.numbers.debt_stagger_status.as_str(), r.numbers.debt_stun_status.as_str()), ("root", "stagger"));
    assert_eq!(r.numbers.field("debt_stagger_status").as_deref(), Some("root"));
    assert!(r.notes().errors.is_empty(), "known ids: {:?}", r.notes());
    let blank = row(json!({"id": "t", "rules": [], "debt_stagger_status": "", "debt_stun_status": ""}));
    assert_eq!((blank.numbers.debt_stagger_status, blank.numbers.debt_stun_status), (d.debt_stagger_status.clone(), d.debt_stun_status.clone()));
    for key in ["debt_stagger_status", "debt_stun_status"] {
        assert!(ruleset::field_rule_of(key).is_some_and(|f| f.needs.is_empty() && f.range.is_none()), "{key}: a field rule, read whatever the rules");
        let bad = row(json!({"id": "t", "rules": [], key: "no_such_status"})).notes().errors;
        assert_eq!(bad, [format!("{key} 'no_such_status' is not a status.")]);
    }
    assert!(Ruleset::classic().notes().errors.is_empty(), "the defaults name status rows");
}

/// F2.2-S1 (FB-58b): roll-over puts the ruleset's debt status straight into `st.active` for the coming round with a
/// 1-round `rounds_left`, so the next roll-over expires it as before; a longer instance already there keeps its count.
#[test]
fn the_debt_status_is_active_with_a_consistent_round_count() {
    let d = Numbers::default();
    for (debt, id) in [(d.debt_stagger, "stagger"), (d.debt_stun, "stun")] {
        let mut u = unit();
        u.ap = -debt;
        assert!(status::roll_over(&mut u, 0, &d).contains(&id.to_string()), "debt {debt}");
        assert!(u.st.pending.is_empty(), "not queued: active now");
        assert_eq!(u.st.rounds_left.get(id), Some(&1), "debt {debt}: one round");
        u.ap = 0;
        assert!(!status::roll_over(&mut u, 0, &d).contains(&id.to_string()), "debt {debt}: gone the round after");
        assert!(!u.st.rounds_left.contains_key(id));
    }
    let custom = Numbers { debt_stun_status: "root".into(), ..Numbers::default() };
    let mut u = unit();
    u.ap = -custom.debt_stun;
    let active = status::roll_over(&mut u, 0, &custom);
    assert!(active.contains(&"root".to_string()) && !active.contains(&"stun".to_string()), "{active:?}");
    assert_eq!(u.st.rounds_left.get("root"), Some(&1));
    let mut u = unit();
    status::queue(&mut u, "stagger", 2, 0, "t");
    u.ap = -d.debt_stagger;
    status::roll_over(&mut u, 0, &d);
    assert_eq!(u.st.rounds_left.get("stagger"), Some(&2), "the queued 2-round instance keeps its count");
}
