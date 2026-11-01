//! MAPNAV1-U on a scratch world (no shipped data): the member who leads on the maps. The lead defaults to the first
//! active member, cycles over the active members the caller keeps standing (benched and downed ones are skipped),
//! is saved as the optional `explore.leader` key and survives a save and a load; a save without the key loads as
//! before (CLN-47: a new optional key, no version bump), a damaged key is refused by `explore_errors`.

use rpgkit_world::state::{explore_errors, PartyState, UnitState, LEADER_KEY};
use rpgkit_world::{save, Data, GameState};
use serde_json::{json, Value};
use std::path::PathBuf;

fn data() -> Data {
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "skill_path": {}}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}]
    });
    let worlds = json!({
        "w": {"manifest": {"id": "w", "start_location": "camp", "start_party": ["hero"]}, "own": {
            "locations": [{"id": "camp", "name": "Camp", "_world": "w", "_qid": "w:camp"}]
        }}
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// Members `a`, `b`, `c`, `d`; all active unless `bench` names one (then the other three are).
fn state(d: &Data, bench: Option<&str>) -> GameState {
    let mut party = PartyState::default();
    for id in ["a", "b", "c", "d"] {
        party.members.push(UnitState::create(id, "hero"));
    }
    if let Some(b) = bench {
        party.active_ids = ["a", "b", "c", "d"].iter().filter(|i| **i != b).map(|i| i.to_string()).collect();
    }
    GameState::create(d, "w", party)
}

fn all(_: &UnitState) -> bool {
    true
}

fn ids(gs: &GameState) -> Vec<String> {
    gs.party.active_members().iter().map(|m| m.id.clone()).collect()
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(tag: &str) -> Scratch {
    let dir = std::env::temp_dir().join(format!("rpgkit_explore_leader_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    Scratch(dir)
}

#[test]
fn the_lead_starts_with_the_first_active_member_and_cycles_over_the_active_ones() {
    let d = data();
    let mut gs = state(&d, None);
    let active = ids(&gs);
    assert!(active.len() > 1, "the world's party cap keeps more than one active: {active:?}");
    assert_eq!(gs.explore_leader(all).map(|m| m.id.clone()), Some(active[0].clone()), "no saved lead: the first active member");
    assert!(!gs.explore.contains_key(LEADER_KEY), "nothing saved until the lead changes");
    let mut seen = Vec::new();
    for _ in 0..active.len() {
        seen.push(gs.cycle_explore_leader(1, all).expect("someone leads"));
    }
    let mut want = active[1..].to_vec();
    want.push(active[0].clone());
    assert_eq!(seen, want, "the lead passes along the roster order and wraps");
    assert_eq!(gs.cycle_explore_leader(-1, all), active.last().cloned(), "back one wraps to the last");
}

#[test]
fn benched_and_downed_members_never_lead() {
    let d = data();
    let mut gs = state(&d, Some("b"));
    let active = ids(&gs);
    assert!(!active.contains(&"b".to_string()));
    assert!(!gs.set_explore_leader("b", all), "a benched member cannot lead");
    let standing = |m: &UnitState| m.id != "c";
    assert!(!gs.set_explore_leader("c", standing), "a downed member cannot lead");
    let mut seen = Vec::new();
    for _ in 0..active.len() * 2 {
        seen.push(gs.cycle_explore_leader(1, standing).expect("someone leads"));
    }
    assert!(seen.iter().all(|id| id != "b" && id != "c"), "{seen:?}");
    // a leader who goes down hands the lead to the first standing member (the saved id stays for when they stand)
    assert!(gs.set_explore_leader(&active[0], all));
    let first_up = active.iter().find(|i| standing(gs.party.member(i).unwrap())).cloned();
    let down_first = |m: &UnitState| m.id != active[0];
    assert_eq!(gs.explore_leader(down_first).map(|m| m.id.clone()), active.iter().find(|i| **i != active[0]).cloned());
    assert_eq!(gs.explore_leader(standing).map(|m| m.id.clone()), first_up);
    // no one standing: the first active member still draws (the map is never without its party)
    assert_eq!(gs.explore_leader(|_: &UnitState| false).map(|m| m.id.clone()), Some(active[0].clone()));
    assert_eq!(gs.cycle_explore_leader(1, |_: &UnitState| false), None);
}

#[test]
fn the_lead_survives_a_save_and_a_load() {
    let d = data();
    let mut gs = state(&d, None);
    let to = gs.cycle_explore_leader(1, all).expect("someone leads");
    assert_eq!(gs.explore[LEADER_KEY], json!(to));
    assert!(explore_errors(&gs.explore).is_empty(), "{:?}", explore_errors(&gs.explore));
    let dir = scratch("trip");
    assert!(save::write(&d, &dir.0, &gs, 1, 5).is_empty());
    let back = save::read(&d, &dir.0, 1).expect("reads back");
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    assert_eq!(back.state, gs);
    assert_eq!(back.state.explore_leader(all).map(|m| m.id.clone()), Some(to));
}

#[test]
fn a_save_without_the_lead_loads_and_a_damaged_one_is_refused() {
    let d = data();
    let gs = state(&d, None);
    let mut raw = gs.to_value();
    assert!(raw["explore"].get(LEADER_KEY).is_none(), "an unchanged lead writes no key (older saves read the same)");
    let back = GameState::from_dict(&d, raw.as_object().unwrap());
    assert_eq!(back.explore_leader(all).map(|m| m.id.clone()), ids(&gs).first().cloned());
    for bad in [json!(""), json!(3), json!("Not An Id")] {
        raw["explore"][LEADER_KEY] = bad.clone();
        let e = raw["explore"].as_object().unwrap().clone();
        assert!(!explore_errors(&e).is_empty(), "explore.leader {bad} is refused");
    }
    // a saved lead who left the party: the first active member leads
    raw["explore"][LEADER_KEY] = Value::String("gone".into());
    let back = GameState::from_dict(&d, raw.as_object().unwrap());
    assert!(explore_errors(&back.explore).is_empty());
    assert_eq!(back.explore_leader(all).map(|m| m.id.clone()), ids(&gs).first().cloned());
}
