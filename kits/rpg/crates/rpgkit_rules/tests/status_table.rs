//! S1.2-S1 (docs/sessions/S1_STATUS.md section 4 item 2): the 17 status rows load from the frozen kit table
//! `statuses.json` behind the old readers (S1.2-S4 adds the 18th, `guard`, last). `OLD` is the Rust const the table replaced, copied row for row, so every
//! reader (`row_of`, `cap_of`, `stacking_of`, `display_name`, `intent_bound`, `magnitude_unit`, the buff lists) must
//! give exactly what the const gave.

use rpgkit_rules::status::{self, Primitive, Stacking, Stacking::*, PRIMITIVES};

/// (id, buff, resistable, default_rounds, stacking, cap, display, grant, magnitude_min, magnitude_max, intent_bound,
/// magnitude_unit): the removed `ROWS: [StatusRow; 17]` const in table order, plus the old `magnitude_unit` match, then
/// S1.2-S4's `guard` (a buff no skill grants by `grant_status`: a multi-round Guard skill queues it, 10-90 percent).
type Old = (&'static str, bool, bool, i32, Stacking, i32, &'static str, &'static str, i32, i32, bool, &'static str);
const OLD: [Old; 18] = [
    ("stagger", false, true, 1, RefreshMaxDuration, 0, "Staggered", "", 0, 0, false, "pct"),
    ("stun", false, true, 1, RefreshMaxDuration, 0, "Stunned", "", 0, 0, false, "pct"),
    ("root", false, true, 1, RefreshMaxDuration, 0, "Rooted", "", 0, 0, false, "pct"),
    ("bleed", false, true, 1, RefreshMaxDuration, 0, "Bleeding", "", 0, 0, false, "pct"),
    ("weaken", false, true, 2, RefreshMaxDuration, 25, "Weakened", "", 1, 25, false, "pct"),
    ("haste", true, false, 1, RefreshMaxDuration, 0, "Hasted", "support", 0, 0, false, "pct"),
    ("taunt", true, false, 1, RefreshMaxDuration, 0, "Taunting", "support", 0, 0, false, "pct"),
    ("riposte", true, false, 2, RefreshMaxDuration, 0, "Riposte", "defend", 0, 0, true, "pct"),
    ("slow", false, true, 1, RefreshMaxDuration, 0, "Slowed", "", 0, 0, false, "pct"),
    ("expose", false, true, 2, RefreshMaxDuration, 20, "Exposed", "", 1, 20, false, "flat"),
    ("fortify", true, false, 2, RefreshMaxDuration, 20, "Fortified", "support", 1, 20, false, "flat"),
    ("regen", true, false, 2, RefreshMaxDuration, 10, "Regenerating", "support", 1, 10, false, "pct_max_hp"),
    ("poison", false, true, 3, StackPotency, 30, "Poisoned", "", 1, 15, false, "flat"),
    ("burn", false, true, 2, RefreshMaxDuration, 10, "Burning", "", 1, 10, false, "pct_max_hp"),
    ("empower", true, false, 2, RefreshMaxDuration, 25, "Empowered", "support", 1, 25, false, "pct"),
    ("steadfast", true, false, 1, RefreshMaxDuration, 0, "Steadfast", "support", 0, 0, false, "pct"),
    ("stealth", true, false, 2, RefreshMaxDuration, 0, "Stealthed", "defend", 0, 0, true, "pct"),
    ("guard", true, false, 1, RefreshMaxDuration, 90, "Guarded", "", 10, 90, false, "pct"),
];

/// 1: the table holds the 17 rows in the const's order then `guard`, and every reader returns the const's value for each
/// id.
#[test]
fn the_table_loads_the_seventeen_rows_with_the_old_values() {
    assert_eq!(status::ROWS.len(), 18, "17 rows and guard (S1.2-S4)");
    let ids: Vec<&str> = status::ROWS.iter().map(|r| r.id).collect();
    assert_eq!(ids, OLD.iter().map(|o| o.0).collect::<Vec<_>>(), "table order is the old const order");
    for (id, buff, resistable, rounds, stacking, cap, display, grant, mmin, mmax, bound, unit) in OLD {
        let r = status::row_of(id).unwrap_or_else(|| panic!("{id}: a row"));
        assert_eq!(
            (r.id, r.buff, r.resistable, r.default_rounds, r.stacking, r.cap, r.display, r.grant, r.magnitude_min, r.magnitude_max, r.intent_bound),
            (id, buff, resistable, rounds, stacking, cap, display, grant, mmin, mmax, bound),
            "{id}: row_of"
        );
        assert_eq!(status::cap_of(id), cap, "{id}: cap_of");
        assert_eq!(status::stacking_of(id), stacking, "{id}: stacking_of");
        assert_eq!(status::display_name(id), display, "{id}: display_name");
        assert_eq!(status::intent_bound(id), bound, "{id}: intent_bound");
        assert_eq!(status::magnitude_unit(id), unit, "{id}: magnitude_unit");
        assert_eq!(status::has_magnitude(id), mmax > 0, "{id}: has_magnitude");
    }
}

/// 2: an id outside the table reads as before: no row, cap 0, refresh-max stacking, its own id as the name, "pct".
#[test]
fn an_unknown_id_reads_the_old_defaults() {
    let id = "no_such_status";
    assert!(status::row_of(id).is_none());
    assert_eq!(status::cap_of(id), 0);
    assert_eq!(status::stacking_of(id), RefreshMaxDuration);
    assert_eq!(status::display_name(id), id);
    assert!(!status::intent_bound(id));
    assert_eq!(status::magnitude_unit(id), "pct");
    assert!(!status::has_magnitude(id));
}

/// 3: every row carries at least one of S1's six behaviour primitives, each at most once, and the rows S1 section 1
/// lists under a primitive carry it (guard, the 18th row, is S1.2-S4).
#[test]
fn every_row_has_a_valid_primitive_tag() {
    assert_eq!(PRIMITIVES.map(Primitive::id), ["modifier", "tick", "block_lock", "targeting", "reactive", "rule_flag"]);
    for r in status::ROWS.iter() {
        assert!(!r.primitives.is_empty(), "{}: at least one primitive", r.id);
        for (i, p) in r.primitives.iter().enumerate() {
            assert!(PRIMITIVES.contains(p), "{}: {p:?} is one of the six", r.id);
            assert_eq!(Primitive::from_id(p.id()), Some(*p));
            assert!(!r.primitives[..i].contains(p), "{}: {p:?} once", r.id);
        }
        assert_eq!(r.kind(), if r.buff { "buff" } else { "debuff" });
        assert_eq!(Stacking::from_id(r.stacking.id()), Some(r.stacking));
    }
    let s1: [(Primitive, &[&str]); 6] = [
        (Primitive::Modifier, &["weaken", "empower", "fortify", "expose", "haste", "slow", "guard"]),
        (Primitive::Tick, &["poison", "burn", "regen", "bleed"]),
        (Primitive::BlockLock, &["stagger", "stun", "root"]),
        (Primitive::Targeting, &["taunt", "stealth"]),
        (Primitive::Reactive, &["riposte"]),
        (Primitive::RuleFlag, &["steadfast"]),
    ];
    for (p, ids) in s1 {
        for id in ids {
            assert!(status::row_of(id).is_some_and(|r| r.has_primitive(p)), "{id}: tagged {}", p.id());
        }
    }
    let listed: usize = s1.iter().map(|(_, ids)| ids.len()).sum();
    assert_eq!(listed, status::ROWS.len(), "S1 section 1 places each of the 18 rows");
}

/// 4: the derived lists keep the const's table order (cleanse lists and the content checks follow it).
#[test]
fn the_buff_lists_follow_the_old_table_order() {
    let want = |f: fn(&Old) -> bool| OLD.iter().filter(|o| f(o)).map(|o| o.0).collect::<Vec<_>>();
    assert_eq!(status::buffs(), want(|o| o.1));
    assert_eq!(status::debuffs(), want(|o| !o.1));
    assert_eq!(status::defend_buffs(), want(|o| o.1 && o.7 == "defend"));
}
