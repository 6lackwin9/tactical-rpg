//! PR7-41 (PT-26, PT-34) on a scratch catalog (no shipped data): a heal line names the full heal when the missing HP
//! capped it ("7 of 52"; a heal lost whole to the cap reads 0, as before); a walk stopped with no route left reads "no
//! way through"; the round summary never says "no breaks" under a logged break (`Session::round_summary`).

use indexmap::IndexMap;
use rpgkit_battle::command::{blocked_words, heal_words};
use rpgkit_battle::session::{EncounterDef, Session};
use rpgkit_data::Catalog;
use serde_json::json;
use std::sync::Arc;

#[test]
fn a_capped_heal_names_its_full_amount() {
    assert_eq!(heal_words(7, 52), "7 of 52");
    assert_eq!(heal_words(52, 52), "52", "uncapped");
    assert_eq!(heal_words(0, 52), "0", "lost whole to the cap: as before");
    assert_eq!(heal_words(10, 0), "10");
}

#[test]
fn a_walk_with_no_route_reads_no_way_through() {
    assert_eq!(blocked_words("no route"), "no way through");
    assert_eq!(blocked_words("Tev Quill"), "blocked by Tev Quill");
}

#[test]
fn the_round_summary_counts_a_logged_break() {
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "stat_mode": "fixed", "hp": 400, "speed": 12, "attack_range": 1, "skill_cap": 5});
    let cat = Arc::new(Catalog::from_value(&json!({
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
        ],
        "units": [unit("k_ash", "party"), unit("k_brute", "enemy")]
    })));
    let row = json!({"id": "t", "name": "T", "party": ["k_ash"], "enemies": ["k_brute"], "total_blocks": 6, "unit_ap": 10, "max_rounds": 3});
    let mut s = Session::start(cat, EncounterDef::from_obj(row.as_object().unwrap()), &IndexMap::new());
    let counted = s.round_summary().breaks;
    assert_eq!(counted, 0, "nothing happened yet");
    s.resolver.log.push("[Break] k_brute is broken".into());
    assert_eq!(s.round_summary().breaks, 1, "a break logged this round counts");
}
