//! PR4-2 (AUDIT-08) on a scratch world (no shipped data): `save::decode` drops a member's saved loadout and gear
//! entries the current data disallows instead of refusing the save (`UnitState::sanitize_loadout`): a loadout family or
//! worn item whose family left the data is dropped, gear the member can no longer wear (`can_wear`, an off-hand held
//! with a two-handed main) goes to the stash, a `loadout_ranks` entry the rank rules refuse is dropped; each drop is one
//! load warning, the re-save loads clean and a valid member loads (and re-saves) as saved.

use rpgkit_rules::stats::MAX_CLASS_SLOTS;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{progression, save, Data, GameState};
use serde_json::{json, Value};

fn data() -> Data {
    let atk = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY"});
    // the path: f_one, f_two and enough more (every one at level 1) to fill the class slots and one past them
    let path: Vec<String> = ["f_one".to_string(), "f_two".to_string()].into_iter().chain((3..=MAX_CLASS_SLOTS + 1).map(|i| format!("f_{i}"))).collect();
    let mut skills = vec![json!({"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}), atk("f_far")];
    skills.extend(path.iter().map(|f| atk(f)));
    let skill_path: serde_json::Map<String, Value> = path.iter().map(|f| (f.clone(), json!(1))).collect();
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter", "skill_path": skill_path}],
        "skills": skills,
        "items": [
            {"id": "sword", "name": "Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword"},
            {"id": "maul", "name": "Maul", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "hammer", "two_handed": true},
            {"id": "buckler", "name": "Buckler", "item_type": "ARMOR", "slot": "OFF_HAND", "weapon_type": "shield"},
            {"id": "apron", "name": "Apron", "item_type": "ARMOR", "slot": "BODY", "weapon_type": "hide"}
        ],
        "units": [{"id": "plain", "name": "Plain", "side": "party", "class_id": "fighter", "level": 1}]
    });
    let worlds = json!({"w": {"manifest": {"id": "w", "start_party": ["plain"]}, "own": {}}});
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn member() -> UnitState {
    UnitState::create("hero", "plain")
}

fn envelope(m: UnitState, d: &Data) -> String {
    let mut party = PartyState::default();
    party.members.push(m);
    let gs = GameState::create(d, "w", party);
    json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": gs.to_value()}).to_string()
}

fn load(d: &Data, text: &str) -> save::Loaded {
    save::decode(d, text).unwrap_or_else(|e| panic!("the save loads: {e:?}"))
}

/// The load warnings name every dropped entry once, the member is valid after the drop and its re-save loads clean.
fn assert_dropped(d: &Data, loaded: &save::Loaded, want: &[String]) {
    for w in want {
        assert!(loaded.warnings.contains(w), "warned: {w}\n{:#?}", loaded.warnings);
    }
    assert_eq!(loaded.warnings.len(), want.len(), "one warning per dropped entry: {:#?}", loaded.warnings);
    let hero = &loaded.state.party.members[0];
    assert!(hero.validate(d).is_empty(), "{:?}", hero.validate(d));
    let again = save::decode(d, &json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 1, "game_state": loaded.state.to_value()}).to_string()).expect("the re-save loads");
    assert!(again.warnings.is_empty(), "{:?}", again.warnings);
}

fn dropped(problem: &str, what: &str) -> String {
    format!("Unit 'hero': {problem} Dropped from the saved {what} (the data no longer allows it).")
}

fn stashed(problem: &str) -> String {
    format!("Unit 'hero': {problem} Moved from the saved gear to the stash (the data no longer allows it).")
}

#[test]
fn a_stale_family_is_dropped_from_the_loadout_and_the_gear() {
    let d = data();
    let mut m = member();
    m.loadout_class = vec!["f_one".into(), "f_gone".into(), "f_far".into(), "f_one".into()];
    m.equipment.insert("main_hand".into(), "gone_blade".into());
    m.equipment.insert("body".into(), "apron".into());
    let loaded = load(&d, &envelope(m, &d));
    let hero = &loaded.state.party.members[0];
    assert_eq!(hero.loadout_class, vec!["f_one".to_string()]);
    assert_eq!(hero.equipment.iter().map(|(s, i)| (s.as_str(), i.as_str())).collect::<Vec<_>>(), vec![("body", "apron")]);
    assert!(loaded.state.party.stash.is_empty(), "an unknown item is not stashed: {:?}", loaded.state.party.stash);
    assert_dropped(&d, &loaded, &[
        dropped("loadout family 'f_gone' is unknown.", "loadout"),
        dropped("loadout family 'f_far' is not owned.", "loadout"),
        dropped("loadout lists family 'f_one' twice.", "loadout"),
        dropped("unknown item 'gone_blade' in main_hand.", "gear"),
    ]);
}

#[test]
fn gear_the_member_can_no_longer_wear_goes_to_the_stash() {
    let d = data();
    let mut m = member();
    m.equipment.insert("main_hand".into(), "maul".into());
    m.equipment.insert("off_hand".into(), "buckler".into());
    m.equipment.insert("body".into(), "sword".into());
    let worn_in_body = m.can_wear(&d, "sword", "body").expect_err("a sword is no body armour");
    let loaded = load(&d, &envelope(m, &d));
    let hero = &loaded.state.party.members[0];
    assert_eq!(hero.equipment.iter().map(|(s, i)| (s.as_str(), i.as_str())).collect::<Vec<_>>(), vec![("main_hand", "maul")]);
    let stash = &loaded.state.party.stash;
    assert_eq!((stash.get("sword"), stash.get("buckler"), stash.len()), (Some(&1), Some(&1), 2), "{stash:?}");
    assert_dropped(&d, &loaded, &[stashed(&worn_in_body), stashed("Buckler can't be held with the two-handed Maul.")]);
}

#[test]
fn an_orphan_rank_is_dropped_and_a_valid_one_kept() {
    let d = data();
    let mut m = member();
    m.loadout_class = vec!["f_one".into()];
    for (fam, rank) in [("f_one", 1), ("strike", 1), ("f_gone", 1), ("f_far", 1), ("f_two", 9)] {
        m.loadout_ranks.insert(fam.into(), rank);
    }
    let loaded = load(&d, &envelope(m, &d));
    let hero = &loaded.state.party.members[0];
    assert_eq!(hero.loadout_ranks.iter().map(|(f, r)| (f.as_str(), *r)).collect::<Vec<_>>(), vec![("f_one", 1)]);
    assert_eq!(hero.loadout_class, vec!["f_one".to_string()], "the loadout itself was valid");
    assert_dropped(&d, &loaded, &[
        dropped("rank set for family 'strike', which is a basic family.", "loadout ranks"),
        dropped("rank set for family 'f_gone' that is not owned.", "loadout ranks"),
        dropped("rank set for family 'f_far' that is not owned.", "loadout ranks"),
        dropped("family 'f_two' has no rank 9.", "loadout ranks"),
    ]);
}

#[test]
fn a_valid_member_loads_and_resaves_as_saved() {
    let d = data();
    let mut m = member();
    m.loadout_class = vec!["f_two".into(), "f_one".into()];
    m.loadout_ranks.insert("f_two".into(), 1);
    m.equipment.insert("main_hand".into(), "sword".into());
    m.equipment.insert("off_hand".into(), "buckler".into());
    m.equipment.insert("body".into(), "apron".into());
    let text = envelope(m.clone(), &d);
    let loaded = load(&d, &text);
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.party.members[0], m);
    let saved: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(loaded.state.to_value(), saved["game_state"], "the re-save is byte-for-byte the save");
}

#[test]
fn a_loadout_past_the_class_slots_is_cut_to_them() {
    let d = data();
    let mut m = member();
    m.loadout_class = ["f_one", "f_two"].iter().map(|f| f.to_string()).chain((3..=MAX_CLASS_SLOTS + 1).map(|i| format!("f_{i}"))).collect();
    let over = m.loadout_class[MAX_CLASS_SLOTS].clone();
    let want: Vec<String> = m.loadout_class[..MAX_CLASS_SLOTS].to_vec();
    let loaded = load(&d, &envelope(m, &d));
    assert_eq!(loaded.state.party.members[0].loadout_class, want);
    assert_dropped(&d, &loaded, &[dropped(&format!("loadout family '{over}' is past the {MAX_CLASS_SLOTS} slots."), "loadout")]);
}

/// The class default loadout (no saved loadout) with the ranks `progression::pin_ranks` writes for its owned, unslotted
/// families: valid, so it loads without a warning and re-saves byte for byte.
#[test]
fn pinned_ranks_of_an_empty_loadout_are_kept() {
    let d = data();
    let mut m = member();
    progression::pin_ranks(&d, &mut m, "w");
    assert!(m.loadout_class.is_empty() && m.loadout_extra.is_empty(), "the class default loadout");
    assert!(!m.loadout_ranks.is_empty(), "pin_ranks wrote ranks for the default families");
    let text = envelope(m.clone(), &d);
    let loaded = load(&d, &text);
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.party.members[0], m);
    let saved: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(loaded.state.to_value(), saved["game_state"], "the re-save is byte-for-byte the save");
}
