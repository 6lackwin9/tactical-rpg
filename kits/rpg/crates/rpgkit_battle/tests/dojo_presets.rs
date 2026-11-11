//! PK1-KITFIX (c) (FB-184 Option A), LAYER-DOWN-2: the Dojo's presets and cast as data, on a scratch catalog built here
//! (no shipped data). The Dojo lists the `dojo_presets` rows in file order and none without rows (the kit has no
//! built-in preset); its heroes are the units marked `dojo` hero in catalog order and its dummy the first marked dummy;
//! applying a preset with passives rebuilds the hero holding them (a meter passive gives its meter), with the preset's
//! gear and pouch; applying one without passives drops them; a reset drops them too.

use rpgkit_battle::dojo::{cast_of, presets_of, DojoSession};
use rpgkit_data::{Catalog, PassiveDef};
use rpgkit_rules::gear::Slot;
use serde_json::{json, Value};
use std::sync::Arc;

fn catalog(rows: Value) -> Catalog {
    let cast = |mut u: Value, part: &str| {
        if !part.is_empty() {
            u["dojo"] = json!(part);
        }
        u
    };
    let unit = |id: &str| json!({"id": id, "name": id, "side": "party", "stat_mode": "fixed", "hp": 100, "power": 10, "armor": 0, "speed": 10, "attack_range": 1, "skill_cap": 5});
    let mut cat = Catalog::from_value(&json!({
        "skills": [{"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}],
        "units": [cast(unit("t_bystander"), ""), cast(unit("t_hero_a"), "hero"), cast(unit("t_dummy"), "dummy"), cast(unit("t_hero_b"), "hero"), cast(unit("t_dummy_2"), "dummy")],
        "items": [
            {"id": "t_blade", "name": "Test Blade", "item_type": "WEAPON", "slot": "MAIN_HAND", "power": 5},
            {"id": "t_salt", "name": "Salt", "item_type": "CONSUMABLE", "consumable_type": "HEALTH_POTION", "heal": 5}
        ],
        "unit_meters": [{"id": "strain", "name": "Strain", "short": "St", "min": 0, "max": 100, "show": "bar", "bands": [{"id": "calm", "from": 0}]}],
        "dojo_presets": rows,
    }));
    cat.passives.insert("p_meter".into(), PassiveDef { id: "p_meter".into(), name: "Meter Test".into(), effects: vec![json!({"type": "meter", "meter": "strain"})], slot: String::new() });
    cat
}

fn duel() -> Value {
    json!({"id": "t_duel", "name": "Test Duel", "description": "A blade, a pouch and a meter", "gear": {"main_hand": "t_blade"}, "pouch": ["t_salt"], "passives": ["p_meter"]})
}

fn plain() -> Value {
    json!({"id": "t_plain", "name": "Test Plain", "description": "A blade only", "gear": {"main_hand": "t_blade"}})
}

#[test]
fn the_dojo_lists_the_rows_in_file_order_and_none_without_rows() {
    assert!(presets_of(&catalog(json!([]))).is_empty(), "no built-in preset");
    let listed = presets_of(&catalog(json!([plain(), duel()])));
    assert_eq!(listed.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["t_plain", "t_duel"]);
    assert_eq!((listed[1].name.as_str(), listed[1].description.as_str()), ("Test Duel", "A blade, a pouch and a meter"));
    assert_eq!(listed[1].gear, [("main_hand".to_string(), "t_blade".to_string())]);
    assert!(listed[0].passives.is_empty());
}

#[test]
fn the_dojo_cast_is_the_marked_units() {
    let cat = catalog(json!([]));
    assert_eq!(cast_of(&cat), (vec!["t_hero_a".to_string(), "t_hero_b".to_string()], Some("t_dummy".to_string())), "heroes in catalog order, the first dummy");
    let s = DojoSession::new(Arc::new(cat));
    assert_eq!(s.hero_ids, ["t_hero_a", "t_hero_b"]);
    assert_eq!(s.dummy_id.as_deref(), Some("t_dummy"));
    assert_eq!(s.dummy.name, "t_dummy");
    // no marked unit: no hero and a blank dummy
    let bare = Catalog::from_value(&json!({"units": [{"id": "t_other", "name": "t_other"}]}));
    assert_eq!(cast_of(&bare), (Vec::new(), None));
}

#[test]
fn a_preset_with_passives_rebuilds_the_hero_holding_them() {
    let mut s = DojoSession::new(Arc::new(catalog(json!([plain(), duel()]))));
    assert_eq!(s.presets().last().map(|p| p.id.clone()), Some("t_duel".to_string()), "the Dojo lists the row");
    assert!(!s.hero().passives.contains(&"p_meter".to_string()));
    s.apply_preset("t_duel");
    let h = s.hero();
    assert!(h.passives.contains(&"p_meter".to_string()), "the hero holds the preset's passive");
    assert!(h.passive_meters.iter().any(|m| m.meter == "strain"), "the meter passive gives the meter");
    assert_eq!(h.inventory.get(Slot::MainHand).map(|e| e.item.id.clone()), Some("t_blade".to_string()), "the preset's gear");
    assert_eq!(h.inventory.pouch.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(), ["t_salt"], "the preset's pouch");
    assert_eq!(s.preset_passives[s.active], vec!["p_meter".to_string()]);
    assert!(s.log.last().is_some_and(|e| e.breakdown.contains("Passives: Meter Test")), "the log names the passive");
    // the other hero is untouched
    assert!(s.heroes.iter().enumerate().filter(|(i, _)| *i != s.active).all(|(_, u)| !u.passives.contains(&"p_meter".to_string())));
    // a preset without passives drops them
    s.apply_preset("t_plain");
    assert!(!s.hero().passives.contains(&"p_meter".to_string()), "another preset drops the passive");
    assert!(s.preset_passives[s.active].is_empty());
    s.apply_preset("t_duel");
    s.reset();
    assert!(!s.hero().passives.contains(&"p_meter".to_string()), "a reset drops it");
}
