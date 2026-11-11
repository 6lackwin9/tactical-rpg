//! PR5-33 (LAYER-UP-DATA): the world data's battle catalog carries the weapon family looks (`Catalog::family_looks`),
//! which the resolved catalog leaves out with the family rows: `Data::from_values` passes the game's item family rows.

use rpgkit_world::Data;
use serde_json::json;

#[test]
fn the_battle_catalog_carries_the_family_looks() {
    let game = json!({"item_families": [
        {"id": "pike", "item_type": "WEAPON", "fx": "stab", "fx_far": "far_look", "area_clips": ["sweep"]},
        {"id": "club", "item_type": "WEAPON"},
    ]});
    let d = Data::from_values(&json!({}), &json!({}), &game);
    let pike = d.cat.family_looks.get("pike").expect("the pike row's look");
    assert_eq!((pike.fx.as_str(), pike.fx_far.as_str(), pike.area_clips.clone()), ("stab", "far_look", vec!["sweep".to_string()]));
    assert_eq!(d.cat.family_looks.get("club").map(|f| f.fx.as_str()), Some(""), "a row without looks: the defaults");
    assert!(Data::from_values(&json!({}), &json!({}), &json!({})).cat.family_looks.is_empty(), "no family rows: no looks");
}
