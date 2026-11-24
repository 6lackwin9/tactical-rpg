//! CLN-47 on a scratch world (no shipped data): the save keeps one version. A save of the current version round trips;
//! a save below `save::OLDEST` (a development-era build) is refused with its message by `decode` and `read`, and its
//! slot header says "older version" so the newest-slot pick skips it. The step table mechanism itself is unit-tested in
//! `save.rs` (decision "Save upgrades at game version updates").

use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{save, Data, GameState};
use serde_json::{json, Value};

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

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    GameState::create(d, "w", party)
}

fn envelope(version: i64, game_state: &Value, saved_at: i64) -> String {
    json!({"format": save::FORMAT, "version": version, "saved_at": saved_at, "game_state": game_state}).to_string()
}

use aegis_core::scratch::Scratch;

fn scratch(tag: &str) -> Scratch {
    let dir = Scratch::new(&format!("rpgkit_save_versions_{tag}"));
    dir
}

#[test]
fn the_current_version_round_trips() {
    let d = data();
    let gs = state(&d);
    let dir = scratch("trip");
    assert!(save::write(&d, dir.path(), &gs, 1, 5).is_empty());
    let back = save::read(&d, dir.path(), 1).expect("reads back");
    assert!(back.warnings.is_empty(), "{:?}", back.warnings);
    assert_eq!(back.state, gs);
    assert_eq!(back.saved_at, 5);
    let head = save::peek(&d, dir.path(), 1);
    assert_eq!((head["ok"].clone(), head["version"].clone(), head["error"].clone()), (json!(true), json!(save::VERSION), json!("")));
}

#[test]
fn a_save_below_the_oldest_version_is_refused() {
    let d = data();
    let v = state(&d).to_value();
    for version in [1, save::OLDEST - 1] {
        let errors = save::decode(&d, &envelope(version, &v, 1)).err().unwrap_or_else(|| panic!("v{version} refused"));
        assert_eq!(errors, [save::older_build_message(version)]);
        assert!(errors[0].contains(&format!("(v{version})")) && errors[0].contains("start a new game"), "{errors:?}");
    }
    assert!(save::decode(&d, &envelope(save::VERSION + 1, &v, 1)).is_err(), "a newer save stays refused");
    assert!(save::decode(&d, &envelope(0, &v, 1)).is_err(), "an invalid version stays refused");
}

#[test]
fn an_older_slot_is_not_readable_and_the_newest_pick_skips_it() {
    let d = data();
    let gs = state(&d);
    let dir = scratch("slots");
    assert!(save::write(&d, dir.path(), &gs, 1, 10).is_empty());
    let old = save::OLDEST - 1;
    std::fs::write(save::path_for(dir.path(), 2), envelope(old, &gs.to_value(), 20)).unwrap();
    assert_eq!(save::read(&d, dir.path(), 2).err(), Some(vec![save::older_build_message(old)]));
    let head = save::peek(&d, dir.path(), 2);
    assert_eq!((head["ok"].clone(), head["error"].clone()), (json!(false), json!("older version")));
    assert_eq!(save::newest_slot(&d, dir.path()), 1, "the older slot is newer by time but not readable");
}

/// ENGINE-62 (CRASH-SAFE1): a slot cut short loads its previous save with a warning and says "damaged" on its header;
/// one that no longer loads (here a version below the oldest) loads its previous save too; the file stays.
#[test]
fn a_damaged_slot_loads_its_previous_save() {
    let d = data();
    let mut gs = state(&d);
    let dir = scratch("damaged");
    assert!(save::write(&d, dir.path(), &gs, 1, 5).is_empty());
    let first = gs.clone();
    gs.party.gold += 40;
    assert!(save::write(&d, dir.path(), &gs, 1, 6).is_empty());
    let path = save::path_for(dir.path(), 1);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, &text[..text.len() / 3]).unwrap();
    let back = save::read(&d, dir.path(), 1).expect("the previous save");
    assert_eq!(back.state, first);
    assert!(back.warnings.iter().any(|w| w.contains("damaged")), "{:?}", back.warnings);
    let head = save::peek(&d, dir.path(), 1);
    assert_eq!((head["ok"].clone(), head["saved_at"].clone()), (json!(true), json!(5)), "the header is the previous save's");
    assert!(head["damaged"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(path.exists(), "never deleted");
    // A save that reads but no longer loads (a refused version) falls back the same way.
    std::fs::write(&path, envelope(1, &gs.to_value(), 7)).unwrap();
    let back = save::read(&d, dir.path(), 1).expect("the previous save");
    assert_eq!(back.state, first);
    assert!(back.warnings.iter().any(|w| w.contains("didn't load")), "{:?}", back.warnings);
}
