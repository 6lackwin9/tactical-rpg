//! INPUT2 on a scratch catalog built here (no shipped data): the encounter's `start_ap`. With it above 0 every unit
//! begins round 1 with at most that much AP and keeps its max (`unit_ap`), so a later round's `ap_regen` banks AP up to
//! the max (`status::regen_ap`); 0 (the default) starts every unit full, as before. Round 1's start snapshot shows the
//! starting AP.

use indexmap::IndexMap;
use rpgkit_battle::session::{EncounterDef, Phase, Session};
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::sync::Arc;

const UNIT_AP: i32 = 12;
const START_AP: i32 = 10;
const AP_REGEN: i32 = 4;

fn catalog() -> Catalog {
    let skills = vec![
        json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}),
        json!({"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}),
        json!({"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}),
    ];
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5});
    Catalog::from_value(&json!({"skills": skills, "units": [unit("t_hero", "party"), unit("t_mate", "party"), unit("t_foe", "enemy")]}))
}

fn session(extra: Value) -> Session {
    let mut row = json!({"id": "t_fight", "name": "Fight", "party": ["t_hero", "t_mate"], "enemies": ["t_foe"], "total_blocks": 6, "unit_ap": UNIT_AP, "ap_regen": AP_REGEN, "max_rounds": 4});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    Session::start(Arc::new(catalog()), EncounterDef::from_obj(row.as_object().unwrap()), &IndexMap::new())
}

/// No `start_ap` (or 0): every unit starts full at the encounter's `unit_ap`.
#[test]
fn without_start_ap_every_unit_starts_full() {
    for extra in [json!({}), json!({"start_ap": 0})] {
        let s = session(extra);
        assert_eq!(s.def.start_ap, 0);
        assert!(s.w.units.iter().all(|u| u.max_ap == UNIT_AP && u.ap == UNIT_AP), "{:?}", s.w.units.iter().map(|u| (u.ap, u.max_ap)).collect::<Vec<_>>());
    }
}

/// `start_ap` below the max: round 1 plans at `start_ap` with the max kept; the start snapshot shows it; the next round
/// regains `ap_regen`, capped at the max.
#[test]
fn start_ap_starts_below_the_max_and_regen_banks_up_to_it() {
    let mut s = session(json!({"start_ap": START_AP}));
    assert_eq!(s.phase, Phase::Planning);
    for u in &s.w.units {
        assert_eq!((u.ap, u.max_ap), (START_AP, UNIT_AP), "{}", u.name);
    }
    let snap = &s.round_record(1).expect("round 1's record").snapshots[0];
    assert!(snap.iter().all(|u| u.ap == START_AP), "the start snapshot shows the starting AP");
    // the party plans nothing: its AP is unspent, so round 2 starts at min(max, start + regen)
    let party = s.w.allies.clone();
    s.resolve_round();
    if s.can_advance_round() {
        s.advance_round();
        for c in party {
            let u = &s.w.units[c];
            assert_eq!(u.ap, UNIT_AP.min(START_AP + AP_REGEN), "{} regains ap_regen up to its max", u.name);
        }
    }
}

/// `start_ap` at or above the max changes nothing (a unit never starts above its max).
#[test]
fn start_ap_above_the_max_starts_full() {
    let s = session(json!({"start_ap": UNIT_AP + 5}));
    assert!(s.w.units.iter().all(|u| u.ap == UNIT_AP));
}
