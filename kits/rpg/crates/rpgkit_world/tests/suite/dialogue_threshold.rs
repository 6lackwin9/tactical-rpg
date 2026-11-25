//! T1.2-S1 (FB-54a, T_STORY.md section 1): a dialogue `stat` condition's `threshold` is a pure alias of `min`: the same
//! `best_stat >= bar` test on the active party (a pure read of the state: holding it changes nothing), the same lock
//! text and the same choice prefix. The bar is the party's own best stat, printed from `best_stat`, never a constant.

use aegis_core::json::{self, Dict};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{dialogue, Data, GameState};
use serde_json::{json, Value};

const STAT: &str = "STR";

fn data() -> Data {
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["hero"]}, "own": {}}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    GameState::create(d, "w", party)
}

fn cond(key: &str, bar: i64) -> Dict {
    json::dict(&json!({"kind": "stat", "stat": STAT, key: bar})).clone()
}

fn choice(c: &Dict) -> Dict {
    json::dict(&json!({"text": "Lift it.", "to": "n2", "requires": [Value::Object(c.clone())]})).clone()
}

#[test]
fn threshold_holds_exactly_where_min_holds() {
    let d = data();
    let gs = state(&d);
    let best = dialogue::best_stat(&d, &gs, STAT);
    assert!(best > 0, "the hero has some {STAT}");
    for bar in [best - 1, best, best + 1] {
        let (by_min, by_threshold) = (cond("min", bar), cond("threshold", bar));
        let want = best >= bar;
        assert_eq!(dialogue::condition_holds(&d, &gs, &by_min), want, "min {bar} against best {best}");
        assert_eq!(dialogue::condition_holds(&d, &gs, &by_threshold), want, "threshold {bar} against best {best}");
        assert_eq!(dialogue::stat_threshold(&by_threshold), dialogue::stat_threshold(&by_min), "the same bar");
        assert_eq!(dialogue::lock_reason(&d, &gs, &choice(&by_threshold)), dialogue::lock_reason(&d, &gs, &choice(&by_min)), "the same lock text at {bar}");
        assert_eq!(dialogue::stat_prefix(&choice(&by_threshold)), dialogue::stat_prefix(&choice(&by_min)), "the same prefix at {bar}");
    }
    assert_eq!(dialogue::lock_reason(&d, &gs, &choice(&cond("threshold", best + 1))), format!("Needs {STAT} {}", best + 1));
}

#[test]
fn a_threshold_condition_is_a_pure_read() {
    let d = data();
    let gs = state(&d);
    let before = gs.clone();
    let c = cond("threshold", dialogue::best_stat(&d, &gs, STAT));
    let first = dialogue::condition_holds(&d, &gs, &c);
    for _ in 0..3 {
        assert_eq!(dialogue::condition_holds(&d, &gs, &c), first, "every redraw reads the same answer");
    }
    assert_eq!(gs, before, "holding the condition never changes the state");
}

#[test]
fn min_is_read_first_and_neither_means_zero() {
    let both = json::dict(&json!({"kind": "stat", "stat": STAT, "min": 3, "threshold": 7})).clone();
    assert_eq!(dialogue::stat_threshold(&both), 3, "min wins (the check refuses both on one condition)");
    let none = json::dict(&json!({"kind": "stat", "stat": STAT})).clone();
    assert_eq!(dialogue::stat_threshold(&none), 0, "today's default bar");
}
