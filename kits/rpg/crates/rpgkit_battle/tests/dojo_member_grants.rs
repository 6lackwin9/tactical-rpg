//! PR5-26 (audit B5, B6; PK2-S1 member grants in the Gear Dojo) on a scratch catalog built here (no shipped data): a
//! hero whose grant `forbid_hands` refuses two-handed items gets none offered for its main hand (`eligible`) and
//! cannot equip one (`equip`; `Unit::forbids`), a one-handed weapon still equips; a hero with `pierce_melee` pierces
//! armour with a melee hit on the dummy under the rule pierce (the hit's breakdown says so), never without the rule.

use rpgkit_battle::dojo::DojoSession;
use rpgkit_data::Catalog;
use rpgkit_rules::ruleset::{Ruleset, PIERCE, TIMELINE};
use serde_json::json;
use std::sync::Arc;

/// LAYER-DOWN-2: the Dojo's cast is the units the catalog marks (`dojo`); the fixture's ids.
const HEROES: [&str; 2] = ["dojo_warrior", "dojo_ranger"];

/// Unit row `u` with its Dojo part (`dojo`: hero or dummy).
fn in_dojo(mut u: serde_json::Value, part: &str) -> serde_json::Value {
    u["dojo"] = serde_json::json!(part);
    u
}

fn catalog() -> Catalog {
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5, "armor": 20});
    Catalog::from_value(&json!({
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
        ],
        "items": [
            {"id": "t_great", "name": "Great Club", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "club", "two_handed": true, "base_power": 12, "starter_stash": true},
            {"id": "t_club", "name": "Club", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "club", "base_power": 6, "starter_stash": true}
        ],
        "units": [in_dojo(unit(HEROES[0], "party"), "hero"), in_dojo(unit(HEROES[1], "party"), "hero"), in_dojo(unit("training_dummy", "enemy"), "dummy")],
    }))
}

fn dojo(rules: &[&str]) -> DojoSession {
    let mut s = DojoSession::new(Arc::new(catalog()));
    s.set_ruleset(Ruleset::from_row(json!({"id": TIMELINE, "name": "Timeline", "rules": rules}).as_object().unwrap()));
    for id in ["t_great", "t_club"] {
        if !s.stash.iter().any(|x| x == id) {
            s.stash.push(id.to_string());
        }
    }
    s
}

#[test]
fn a_hero_forbidden_two_handers_is_offered_and_equips_none() {
    let mut s = dojo(&[]);
    assert!(s.eligible("main_hand").contains(&"t_great".to_string()), "offered without the grant");
    let a = s.active;
    s.heroes[a].forbid_hands = 2;
    let offered = s.eligible("main_hand");
    assert!(!offered.contains(&"t_great".to_string()) && offered.contains(&"t_club".to_string()), "{offered:?}");
    assert!(!s.equip("t_great", "main_hand"), "refused");
    assert!(s.hero().inventory.main_weapon().is_none_or(|e| e.item.id != "t_great"));
    assert!(s.stash.iter().any(|x| x == "t_great"), "still in the stash");
    assert!(s.equip("t_club", "main_hand"), "a one-handed weapon equips");
}

#[test]
fn a_hero_with_pierce_melee_pierces_the_dummys_armour_under_the_rule() {
    for (rules, grant, pierced) in [(vec![PIERCE], true, true), (vec![PIERCE], false, false), (vec![], true, false)] {
        let mut s = dojo(&rules);
        let a = s.active;
        s.heroes[a].pierce_melee = grant;
        s.use_skill(Some("strike"));
        let hit = s.log.iter().rev().find(|e| e.category == "ATTACK" && !e.by_dummy).expect("the hero's hit");
        assert_eq!(hit.breakdown.contains("armour pierced"), pierced, "rules {rules:?}, grant {grant}: {}", hit.breakdown);
    }
}
