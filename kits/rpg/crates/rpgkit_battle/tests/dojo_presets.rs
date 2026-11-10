//! PK1-KITFIX (c) (FB-184 Option A): the Dojo's presets as data, on a scratch catalog built here (no shipped data). With
//! no `dojo_presets` row the Dojo lists exactly the built-in `PRESETS`; a row with a built-in id replaces that preset
//! in its place and a new id follows them in file order; applying a preset with passives rebuilds the hero holding them
//! (a meter passive gives its meter), with the preset's gear and pouch; applying one without passives drops them; a
//! reset drops them too.

use rpgkit_battle::dojo::{presets_of, DojoSession, PRESETS};
use rpgkit_data::{Catalog, PassiveDef};
use rpgkit_rules::gear::Slot;
use serde_json::{json, Value};
use std::sync::Arc;

fn catalog(rows: Value) -> Catalog {
    let unit = |id: &str| json!({"id": id, "name": id, "side": "party", "stat_mode": "fixed", "hp": 100, "power": 10, "armor": 0, "speed": 10, "attack_range": 1, "skill_cap": 5});
    let mut cat = Catalog::from_value(&json!({
        "skills": [{"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"}],
        "units": [unit("dojo_warrior"), unit("dojo_ranger"), unit("training_dummy")],
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

#[test]
fn without_rows_the_dojo_lists_the_built_in_presets() {
    let listed = presets_of(&catalog(json!([])));
    assert_eq!(listed, PRESETS.iter().map(|p| p.to_def()).collect::<Vec<_>>());
    for (def, p) in listed.iter().zip(PRESETS.iter()) {
        assert_eq!((def.id.as_str(), def.name.as_str(), def.description.as_str()), (p.id, p.name, p.desc));
        assert_eq!(def.gear.len(), p.gear.len());
        assert!(def.passives.is_empty(), "a built-in preset grants no passive");
    }
}

#[test]
fn a_row_replaces_a_built_in_in_place_and_new_rows_follow() {
    let replaced = PRESETS[2].id;
    let listed = presets_of(&catalog(json!([duel(), {"id": replaced, "name": "Replaced"}])));
    assert_eq!(listed.len(), PRESETS.len() + 1);
    assert_eq!((listed[2].id.as_str(), listed[2].name.as_str()), (replaced, "Replaced"), "in its place");
    assert!(listed[2].gear.is_empty(), "the row's own gear (none), not the built-in's");
    assert_eq!(listed.last().map(|p| p.id.as_str()), Some("t_duel"), "a new id after the built-in presets");
    let ids: Vec<&str> = listed.iter().map(|p| p.id.as_str()).collect();
    for p in PRESETS.iter().filter(|p| p.id != replaced) {
        assert_eq!(ids.iter().filter(|&&i| i == p.id).count(), 1, "{} listed once", p.id);
    }
}

#[test]
fn a_preset_with_passives_rebuilds_the_hero_holding_them() {
    let mut s = DojoSession::new(Arc::new(catalog(json!([duel()]))));
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
    // a built-in preset (no passives) drops them
    s.apply_preset(PRESETS[0].id);
    assert!(!s.hero().passives.contains(&"p_meter".to_string()), "another preset drops the passive");
    assert!(s.preset_passives[s.active].is_empty());
    s.apply_preset("t_duel");
    s.reset();
    assert!(!s.hero().passives.contains(&"p_meter".to_string()), "a reset drops it");
}
