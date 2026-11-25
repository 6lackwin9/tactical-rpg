//! LAYER-DOWN-1: the game's world façade reads its older saves: a saved deck that holds the PK1-S2a key `marrow` (the
//! kit's `channeled` since LAYER-DOWN-1) loads through `emberwake_world::save` (`DECK_KEY_ALIASES`), and saves back under
//! the kit's key; the kit's own `decode` (no aliases) refuses the old key. Shipped data read only (no content numbers).

use emberwake_world::{save, Data};
use serde_json::{json, Value};
use std::path::Path;

fn text(state: &Value) -> String {
    json!({"format": save::FORMAT, "version": save::VERSION, "saved_at": 0, "game_state": state}).to_string()
}

#[test]
fn an_older_save_with_the_marrow_deck_key_loads_as_channeled() {
    let d = Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0;
    let gs = emberwake_world::new_game::create_for_world(&d, "ashlands", 7).expect("a new game");
    let mut v = gs.to_value();
    assert!(!v["party"]["members"].as_array().expect("members").is_empty(), "a new game has a member");
    v["party"]["members"][0]["deck"] = json!({"marrow": []});
    let loaded = save::decode(&d, &text(&v)).expect("the façade reads the old key");
    assert!(loaded.state.party.members[0].deck.channeled.is_empty());
    assert!(loaded.state.to_value()["party"]["members"][0]["deck"].get("marrow").is_none(), "it saves under the kit's key");
    let refused = rpgkit_world::save::decode(&d, &text(&v)).err().expect("the kit alone refuses the old key");
    assert!(refused.iter().any(|e| e.contains("unknown key 'marrow'")), "{refused:?}");
    assert_eq!(save::DECK_KEY_ALIASES, &[("marrow", "channeled")]);
}

#[test]
fn milestone1_v13_act1_end_fixture_loads_and_round_trips() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let d = Data::open(&Data::find_data_dir(manifest_dir).expect("data folder")).unwrap().0;
    let fixture = manifest_dir.join("tests/fixtures/save_v13_act1_end.json");
    let raw = std::fs::read_to_string(&fixture).expect("v13 Act 1 end fixture");
    let loaded = save::decode(&d, &raw).expect("v13 Act 1 end fixture must decode cleanly");
    assert_eq!(save::VERSION, 13);
    assert_eq!(loaded.state.world_id, "sunk_ward");
    assert_eq!(loaded.state.location_id, "sunk_ward:sluice_undercroft");
    assert_eq!(loaded.state.campaign.as_ref().map(|c| c.act), Some(2));
    assert_eq!(loaded.state.flag("sunk_ward.m5_done"), Some(&json!(true)));
    assert_eq!(loaded.state.flag("pk2.slab_open"), Some(&json!(true)));
    assert_eq!(loaded.state.flag("pk2.hearth_open"), Some(&json!(true)));
    assert_eq!(emberwake_world::campaign::save_refusal(&loaded.state), "");

    let scratch = aegis_core::scratch::Scratch::new("v13_act1_end");
    let errs = save::write(&d, scratch.path(), &loaded.state, 1, 1_700_000_000);
    assert!(errs.is_empty(), "{errs:?}");
    let again = save::read(&d, scratch.path(), 1).expect("round-trip read");
    assert_eq!(again.state.to_value(), loaded.state.to_value());
}
