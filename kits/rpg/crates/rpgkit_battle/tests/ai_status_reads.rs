//! I1.2-S1 (docs/sessions/I_AI.md section 1, FB-52a): the AI reads statuses through S1 behaviour predicates instead of
//! the 8 status ids it named (taunt, riposte, regen, root, stun, slow, expose, fortify). Strict 1-to-1: over every row of
//! the frozen kit status table, each call site's predicate selects exactly the ids the old literal check did.

use rpgkit_battle::ai;
use rpgkit_rules::status::{self, Rating, Targeting, Tick};

/// (call site read, the ids its old literal check named).
fn sites() -> [(&'static str, fn(&str) -> bool, &'static [&'static str]); 7] {
    [
        ("pulls_targeting", ai::pulls_targeting, &["taunt"]),
        ("counters", ai::counters, &["riposte"]),
        ("heals_over_time", ai::heals_over_time, &["regen"]),
        ("stops_moving", ai::stops_moving, &["stun", "root"]),
        ("lowers_move_rate", ai::lowers_move_rate, &["slow"]),
        ("lowers_armor", ai::lowers_armor, &["expose"]),
        ("raises_armor", ai::raises_armor, &["fortify"]),
    ]
}

/// 1: every call-site predicate matches exactly its old id set over every table row (and nothing outside the table).
#[test]
fn each_ai_read_selects_exactly_the_old_status_ids() {
    let mut named: Vec<&str> = Vec::new();
    for (site, read, old) in sites() {
        for r in status::ROWS.iter() {
            assert_eq!(read(r.id), old.contains(&r.id), "{site}: {} (predicate vs old literal)", r.id);
        }
        assert!(!read("no_such_status") && !read(""), "{site}: an unknown or empty id reads false");
        for id in old {
            assert!(status::row_of(id).is_some(), "{site}: {id} is a table row");
        }
        named.extend_from_slice(old);
    }
    named.sort_unstable();
    assert_eq!(named, ["expose", "fortify", "regen", "riposte", "root", "slow", "stun", "taunt"], "the 8 ids the AI named");
}

/// 2: the kit predicates read the table: `modifies_rating` by the row's `rating`, `has_targeting_pull` by its
/// `targeting`, `has_tick` by kind, `locks_movement` by the lock blocks over the move blocks.
#[test]
fn the_kit_predicates_follow_the_table_columns() {
    for r in status::ROWS.iter() {
        for rating in [Rating::AttackPower, Rating::Armor, Rating::MoveRate, Rating::Guard] {
            assert_eq!(status::modifies_rating(r.id, rating), r.rating == Some(rating), "{}: rating {}", r.id, rating.id());
        }
        assert_eq!(status::has_targeting_pull(r.id), r.targeting == Some(Targeting::Pull), "{}", r.id);
        let tick = r.has_primitive(status::Primitive::Tick);
        assert_eq!(status::has_tick(r.id, Tick::Heal), tick && r.buff, "{}", r.id);
        assert_eq!(status::has_tick(r.id, Tick::Damage), tick && !r.buff, "{}", r.id);
    }
    // a lock over blocks 1 and 2 stops a 2-block move; one leaving block 2 open does not; a lock on Move (no blocks) does
    let lock = |id: &str| status::row_of(id).map(|r| r.lock_blocks.to_vec()).unwrap_or_default();
    for r in status::ROWS.iter().filter(|r| r.has_primitive(status::Primitive::BlockLock)) {
        let want = lock(r.id).is_empty() || (lock(r.id).contains(&1) && lock(r.id).contains(&2));
        assert_eq!(status::locks_movement(r.id, 2), want, "{}", r.id);
    }
}
