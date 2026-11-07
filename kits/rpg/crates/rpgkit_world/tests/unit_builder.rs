//! U2.2-S2 / U4.2-S2 on scratch worlds (no shipped data): the one unit builder (`rpgkit_rules::build_from`) and its
//! passive sources (FB-28): a character's own passives, then its faction's grants, its disciplines' passives and its
//! named items' grants, each once, applied in that order; a party member's sources re-gathered at each build, its other
//! owned passives after them.

use rpgkit_battle::session::Roster;
use rpgkit_rules::{build_from, UnitBuild};
use rpgkit_world::battle::WorldRoster;
use rpgkit_world::{new_game, progression, Data};
use serde_json::{json, Value};

/// One passive per source, in the gather order; each reveals plans (foresight keeps its first source's id).
const SOURCES: [&str; 4] = ["p_char", "p_faction", "p_disc", "p_item"];
/// A grant the faction gains after a new game.
const LATE: &str = "p_late";
/// A passive a quest gave the member.
const QUEST: &str = "p_quest";

fn with_ids(w: &str, rows: Value) -> Value {
    let rows = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            let mut r = r.as_object().unwrap().clone();
            let id = r["id"].as_str().unwrap().to_string();
            r.insert("_world".into(), w.into());
            r.insert("_qid".into(), format!("{w}:{id}").into());
            Value::Object(r)
        })
        .collect();
    Value::Array(rows)
}

/// World `w`: the founder (party; its own passive and the guild's grants), the foe (enemy; all four sources, its
/// discipline also listing its own passive again) and the guild granting `grants`.
fn data(grants: Value) -> Data {
    let passive = |id: &str| json!({"id": id, "name": id, "effects": [{"type": "foresight", "range": 2}]});
    let passives: Vec<Value> = SOURCES.iter().chain(&[LATE, QUEST]).map(|id| passive(id)).collect();
    let catalog = json!({
        "classes": [{"id": "warrior", "name": "Warrior", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "guard", "name": "Guard", "kind": "DEFEND", "level": 1, "blocks": 2, "basic": true, "mitigation": 0.3},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
        ],
        "items": [{"id": "sword", "name": "Sword", "item_type": "WEAPON", "slot": "MAIN_HAND", "weapon_type": "sword", "base_power": 4}],
        "passives": passives,
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "warrior", "level": 1},
            {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "warrior", "level": 1}
        ]
    });
    let own = json!({
        "characters": with_ids("w", json!([
            {"id": "founder", "name": "Founder", "role": "party", "base_unit": "hero", "faction": "guild", "passives": [SOURCES[0]]},
            {"id": "foe", "name": "Foe", "role": "enemy", "base_unit": "brute", "faction": "guild", "passives": [SOURCES[0]],
                "disciplines": ["drill"], "named_items": ["blade"]}
        ])),
        "factions": with_ids("w", json!([{"id": "guild", "name": "Guild", "grants": grants}])),
        "disciplines": with_ids("w", json!([{"id": "drill", "name": "Drill", "passives": [SOURCES[2], SOURCES[0]]}])),
        "items": with_ids("w", json!([{"id": "blade", "name": "Blade", "base_item": "sword", "grants": [SOURCES[3]]}]))
    });
    Data::from_values(&catalog, &json!({"w": {"manifest": {"id": "w"}, "own": own}}), &json!({}))
}

fn strs(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

#[test]
fn an_npc_applies_its_four_sources_once_each_in_gather_order() {
    let d = data(json!([SOURCES[1]]));
    let roster = WorldRoster::new(&d, "w");
    let e = roster.entry(&d.cat, "foe").expect("the foe's entry");
    let over = roster.overrides(&d.cat, "foe");
    assert_eq!(over.passives, strs(&SOURCES), "character, faction, disciplines, named items; the repeat once");
    assert_eq!(over.named_items.len(), 1, "the named item is worn");
    let u = build_from(&d.cat, UnitBuild { row: &e, default_ap: 0, deck: None, wield: Default::default(), over, growth: None }).unit;
    assert_eq!(u.passives, strs(&SOURCES), "applied in the gather order");
    assert_eq!(u.foresight_source, SOURCES[0], "the first source keeps the foresight id");
    assert!(u.inventory.gear().any(|g| g.item.id.starts_with("sword__")), "the named blade over the row's gear");
}

#[test]
fn a_member_regathers_its_sources_at_each_build_and_keeps_what_it_earned() {
    let before = data(json!([SOURCES[1]]));
    let mut hero = new_game::hero_from_character(&before, "w", "founder").expect("the founder joins");
    assert_eq!(hero.passives, strs(&SOURCES[..2]), "the new game saves the sources as they are");
    assert_eq!(progression::member_passives(&before, &hero), hero.passives, "unchanged data: the same list");
    hero.passives.push(QUEST.into());
    // the guild gains a grant after the save was made: the member gets it, before what it earned
    let after = data(json!([SOURCES[1], LATE]));
    let want = strs(&[SOURCES[0], SOURCES[1], LATE, QUEST]);
    assert_eq!(progression::member_passives(&after, &hero), want);
    let built = progression::build(&after, &hero).expect("the member builds");
    assert_eq!(built.unit.passives, want, "the build applies the re-gathered list");
    assert_eq!(built.unit.foresight_source, SOURCES[0]);
}

#[test]
fn a_catalog_member_keeps_its_saved_passives() {
    let d = data(json!([]));
    let mut m = rpgkit_world::state::UnitState::at_level(&d, "hero", 1, "hero");
    m.passives = strs(&[QUEST, SOURCES[2]]);
    assert_eq!(progression::member_passives(&d, &m), m.passives, "no character: its saved list, in order");
}
