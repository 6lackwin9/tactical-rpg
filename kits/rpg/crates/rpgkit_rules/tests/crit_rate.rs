//! Crit from PERC (U1.2-S3 part a, docs/sessions/U1_STATS.md "stats to values"): the crit rate before the weapon
//! proficiency bonus is `CRIT_BASE + max(0, PERC - CRIT_PERC_FLOOR) * CRIT_PER_PERC`; DEX no longer moves it. Every
//! expectation is worked out here from that rule and the locked constants (no shipped data).

use indexmap::IndexMap;
use rpgkit_data::STATS;
use rpgkit_rules::damage::{CRIT_BASE, CRIT_MULT, CRIT_PERC_FLOOR, CRIT_PER_PERC};
use rpgkit_rules::stats::{CharacterStats, Proficiency};

/// A stat block with every core stat at 10, then `perc` and `dex` set.
fn stats(perc: i32, dex: i32) -> CharacterStats {
    let mut core: IndexMap<String, i32> = STATS.iter().map(|s| (s.to_string(), 10)).collect();
    core.insert("PERC".into(), perc);
    core.insert("DEX".into(), dex);
    CharacterStats { level: 1, unallocated: 0, core, proficiency: Proficiency::default(), modifiers: Vec::new() }
}

fn rule(perc: i32) -> f64 {
    CRIT_BASE + (perc - CRIT_PERC_FLOOR).max(0) as f64 * CRIT_PER_PERC
}

#[test]
fn crit_rate_follows_perc_at_and_above_the_floor() {
    // Below and at the floor: the base alone.
    for perc in [1, CRIT_PERC_FLOOR - 3, CRIT_PERC_FLOOR] {
        assert_eq!(stats(perc, 10).crit_rate(None), CRIT_BASE, "PERC {perc} at or below the floor");
    }
    // Above the floor: one weight per point.
    for above in [1, 4, 12] {
        let perc = CRIT_PERC_FLOOR + above;
        let got = stats(perc, 10).crit_rate(None);
        assert_eq!(got, rule(perc), "PERC {perc}");
        assert_eq!(got, CRIT_BASE + above as f64 * CRIT_PER_PERC, "PERC {perc}");
    }
    // Monotonic in PERC.
    assert!(stats(CRIT_PERC_FLOOR + 5, 10).crit_rate(None) > stats(CRIT_PERC_FLOOR + 1, 10).crit_rate(None));
}

#[test]
fn crit_rate_ignores_dex() {
    for perc in [CRIT_PERC_FLOOR - 2, CRIT_PERC_FLOOR, CRIT_PERC_FLOOR + 6] {
        let low = stats(perc, 1).crit_rate(None);
        for dex in [CRIT_PERC_FLOOR, CRIT_PERC_FLOOR + 8, 40] {
            assert_eq!(stats(perc, dex).crit_rate(None), low, "PERC {perc}, DEX {dex}");
        }
        assert_eq!(low, rule(perc));
    }
}

/// The spec's weights (U1_STATS parameter table: 5 % + a PERC term, x1.5): U1.2-S3a moved the stat, not the weights.
#[test]
fn the_crit_weights_are_the_specs() {
    assert_eq!((CRIT_BASE, CRIT_PER_PERC, CRIT_PERC_FLOOR, CRIT_MULT), (5.0, 1.0, 10, 1.5));
}
