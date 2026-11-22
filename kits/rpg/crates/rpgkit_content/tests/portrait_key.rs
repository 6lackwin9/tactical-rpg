//! PAINTED-MAPS1 (PR7-73): a unit's own portrait key (`art::portrait_key`, `art::portrait_chain_keyed`,
//! `art::face_chain_keyed`): tried after a unique tier's file and before the body's common portrait; a world character
//! wearing the unit's body shows it, one with its own art does not.

use aegis_core::json::{self, Dict};
use rpgkit_content::art;
use serde_json::json;

fn unit(portrait: &str) -> Dict {
    json::obj([("id", json!("grunt")), ("class_id", json!("knight")), ("art", json!("knight/goblin")), ("portrait", json!(portrait))])
}

#[test]
fn the_key_comes_after_the_unique_file_and_before_the_body() {
    assert_eq!(art::keyed_portrait(""), None);
    assert_eq!(art::keyed_portrait("grunt_cut").as_deref(), Some("portraits/common/grunt_cut.png"));
    assert_eq!(art::portrait_chain_keyed("common", "grunt", "knight/goblin", "grunt_cut"), ["portraits/common/grunt_cut.png", "portraits/common/knight_goblin.png", "portraits/common/knight_base.png"]);
    assert_eq!(art::portrait_chain_keyed("elite", "grunt", "knight/goblin", "grunt_cut"), ["portraits/elites/grunt.png", "portraits/common/grunt_cut.png", "portraits/common/knight_goblin.png", "portraits/common/knight_base.png"]);
    assert_eq!(art::portrait_chain_keyed("common", "grunt", "knight/goblin", ""), art::portrait_chain("common", "grunt", "knight/goblin"), "no key: today's chain");
    assert_eq!(art::portrait_chain_keyed("common", "grunt", "knight/goblin", "knight_goblin"), ["portraits/common/knight_goblin.png", "portraits/common/knight_base.png"], "a key at the body's file is listed once");
    let faces = art::face_chain_keyed("npc", "grunt", "knight/goblin", "grunt_cut", "hurt");
    assert_eq!(faces[..3], ["portraits/npcs/faces/grunt/hurt.png", "portraits/npcs/faces/grunt/neutral.png", "portraits/npcs/grunt.png"]);
    assert_eq!(faces[3], "portraits/common/grunt_cut.png", "the frames, the unique file, then the key");
    assert_eq!(art::face_chain("common", "grunt", "knight/goblin", ""), art::face_chain_keyed("common", "grunt", "knight/goblin", "", ""));
}

#[test]
fn a_character_wearing_the_units_body_shows_its_key() {
    let keyed = unit("grunt_cut");
    assert_eq!(art::portrait_key(&Dict::new(), &keyed), "grunt_cut", "the unit itself");
    assert_eq!(art::portrait_key(&json::obj([("id", json!("c")), ("art", json!(""))]), &keyed), "grunt_cut", "a character on its body");
    assert_eq!(art::portrait_key(&json::obj([("id", json!("c")), ("art", json!("knight"))]), &keyed), "", "a character with its own art");
    assert_eq!(art::portrait_key(&Dict::new(), &unit("")), "", "no key");
}
