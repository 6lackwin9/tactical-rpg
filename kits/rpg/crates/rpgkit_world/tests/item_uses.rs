//! PR7-41 (PT-30, PT-34) on a scratch world (no shipped data): `recipe::uses_of` names each recipe whose cost takes an
//! item, with its character, once each in catalog order (offered now or not); `explore::locked_words` never doubles
//! "Locked".

use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{explore, recipe, Data, GameState};
use serde_json::json;

fn data() -> Data {
    let w = "w";
    let row = |id: &str, name: &str, items: serde_json::Value| json!({"id": id, "name": name, "npc": "smith", "requires": [{"kind": "flag", "key": "open", "value": true, "op": "=="}], "cost": {"gold": 1, "items": items}, "target": "stash", "effects": []});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}],
        "items": [{"id": "valve", "name": "Valve", "item_type": "MATERIAL"}, {"id": "spur", "name": "Spur", "item_type": "MATERIAL"}],
        "recipes": [row("r_one", "Graft One", json!({"valve": 1, "spur": 1})), row("r_two", "Graft Two", json!({"valve": 1})), row("r_three", "Graft Three", json!({"spur": 2}))]
    });
    let worlds = json!({w: {"manifest": {"id": w, "start_location": "hub", "start_party": ["hero"]}, "own": {
        "locations": [{"id": "hub", "name": "Hub", "_world": w, "_qid": "w:hub"}],
        "characters": [{"id": "hero", "name": "Hero", "role": "party", "_world": w, "_qid": "w:hero"}, {"id": "smith", "name": "Smith Ada", "role": "npc", "_world": w, "_qid": "w:smith"}]
    }}});
    Data::from_values(&catalog, &worlds, &json!({}))
}

#[test]
fn an_items_uses_name_each_recipe_that_takes_it() {
    let d = data();
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let gs = GameState::create(&d, "w", party);
    assert_eq!(recipe::uses_of(&d, &gs, "valve"), ["Graft One (Smith Ada)", "Graft Two (Smith Ada)"], "not offered yet, still named");
    assert_eq!(recipe::uses_of(&d, &gs, "spur"), ["Graft One (Smith Ada)", "Graft Three (Smith Ada)"]);
    assert!(recipe::uses_of(&d, &gs, "nothing").is_empty());
}

#[test]
fn a_locked_markers_words_never_double_locked() {
    assert_eq!(explore::locked_words("Locked"), "Locked.");
    assert_eq!(explore::locked_words(""), "Locked.");
    assert_eq!(explore::locked_words("locked."), "Locked.");
    assert_eq!(explore::locked_words("Locked until the gate opens."), "Locked until the gate opens.");
    assert_eq!(explore::locked_words("The gate is barred."), "Locked: The gate is barred.");
}
