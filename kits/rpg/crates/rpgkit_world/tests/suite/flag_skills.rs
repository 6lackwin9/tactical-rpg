//! ACT1-HOOKS2 (FB-244 (1)) on a scratch world built here (no shipped data): a world encounter's `flag_skills`
//! `[{member_from_flag, skill, requires_flags}]` grants, at the battle start, each entry's skill to the fielded party
//! member whose character id the flag holds, while every `requires_flags` condition holds (`battle::flag_skills`,
//! `Unit::battle_skills`). An unset flag, a flag naming a benched member or no member, failing conditions and an encounter
//! without the field grant nothing; the grant lasts the battle only (the member's saved state learns nothing, so the
//! next fight without the flag fields it without the skill).

use aegis_core::json::{self, Dict};
use rpgkit_world::battle::{self, WorldFight};
use rpgkit_world::campaign::trigger;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{explore, Data, GameState};
use serde_json::{json, Value};

const CHOSEN: &str = "w.chosen";
const OTHER: &str = "w.other";
const READY: &str = "w.ready";
const GRANT: &str = "g_boon";
const GATED: &str = "g_gated";

fn with_ids(w: &str, rows: Value) -> Value {
    Value::Array(
        json::arr(&rows)
            .iter()
            .map(|r| {
                let mut r = json::dict(r).clone();
                let id = json::s(&r, "id");
                r.insert("_world".into(), w.into());
                r.insert("_qid".into(), format!("{w}:{id}").into());
                Value::Object(r)
            })
            .collect(),
    )
}

fn data() -> Data {
    let grants = json!([
        {"member_from_flag": CHOSEN, "skill": GRANT},
        {"member_from_flag": OTHER, "skill": GATED, "requires_flags": [{"key": READY, "value": true}]}
    ]);
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["lead"]}, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub"}])),
        "encounters": with_ids("w", json!([
            {"id": "trial", "name": "Trial", "enemies": ["rat"], "flag_skills": grants},
            {"id": "plain", "name": "Plain", "enemies": ["rat"]}
        ])),
        "characters": with_ids("w", json!([
            {"id": "lead", "name": "Lead", "role": "party", "base_unit": "hero"},
            {"id": "second", "name": "Second", "role": "party", "base_unit": "hero"},
            {"id": "bench", "name": "Bench", "role": "party", "base_unit": "hero"},
            {"id": "rat", "name": "Rat", "role": "npc", "base_unit": "rat"}
        ]))
    }}});
    let attack = |id: &str| json!({"id": id, "name": id, "kind": "ATTACK", "level": 1, "blocks": 1, "target_mode": "ENEMY", "learned_only": true});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2},
            attack(GRANT), attack(GATED)
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 3},
            {"id": "rat", "name": "Rat", "side": "enemy", "class_id": "drill", "level": 1}
        ]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// Lead and Second deployed, Bench on the bench; `flags` set.
fn state(d: &Data, flags: &[(&str, Value)]) -> GameState {
    let members = ["lead", "second", "bench"].iter().map(|id| UnitState::create(id, "hero")).collect();
    let mut gs = GameState::create(d, "w", PartyState { members, active_ids: vec!["lead".into(), "second".into()], ..PartyState::default() });
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    for (k, v) in flags {
        gs.set_flag(k, v.clone());
    }
    gs
}

fn fight(enc: &str) -> Dict {
    json::dict(&json!({"source": trigger::FIGHT_SOURCE, "encounter": format!("w:{enc}")})).clone()
}

fn begin(d: &Data, gs: &mut GameState, enc: &str) -> WorldFight {
    battle::begin_world(d, gs, &fight(enc)).unwrap_or_else(|e| panic!("{enc} starts: {e}"))
}

/// Each fielded member's battle grants, in fielded order.
fn granted(f: &WorldFight) -> Vec<(String, Vec<String>)> {
    f.members.iter().enumerate().map(|(i, (m, _))| (m.clone(), f.session.w.units[i].battle_skills.clone())).collect()
}

fn none() -> Vec<(String, Vec<String>)> {
    vec![("lead".into(), Vec::new()), ("second".into(), Vec::new())]
}

#[test]
fn the_member_the_flag_names_fights_holding_the_skill() {
    let d = data();
    let mut gs = state(&d, &[(CHOSEN, json!("second"))]);
    assert_eq!(battle::flag_skills(&d, &gs, d.resolve("w", "encounters", "trial")), [("second".to_string(), GRANT.to_string())]);
    let f = begin(&d, &mut gs, "trial");
    assert_eq!(granted(&f), [("lead".to_string(), Vec::new()), ("second".to_string(), vec![GRANT.to_string()])]);
    let cat = f.session.w.cat.clone();
    assert!(f.session.w.units[1].owns_skill(&cat, GRANT) && !f.session.w.units[0].owns_skill(&cat, GRANT));
    // a qualified character ref names the member too
    let mut gs = state(&d, &[(CHOSEN, json!("w:lead"))]);
    assert_eq!(granted(&begin(&d, &mut gs, "trial"))[0], ("lead".to_string(), vec![GRANT.to_string()]));
}

#[test]
fn requires_flags_gate_an_entry() {
    let d = data();
    let mut gs = state(&d, &[(OTHER, json!("lead"))]);
    assert_eq!(granted(&begin(&d, &mut gs, "trial")), none(), "the condition fails");
    let mut gs = state(&d, &[(OTHER, json!("lead")), (READY, json!(true)), (CHOSEN, json!("lead"))]);
    assert_eq!(granted(&begin(&d, &mut gs, "trial"))[0], ("lead".to_string(), vec![GRANT.to_string(), GATED.to_string()]), "row order");
}

#[test]
fn nothing_is_granted_without_a_fielded_member_to_name() {
    let d = data();
    for flags in [vec![], vec![(CHOSEN, json!("bench"))], vec![(CHOSEN, json!("ghost"))], vec![(CHOSEN, json!(true))]] {
        let mut gs = state(&d, &flags);
        assert_eq!(granted(&begin(&d, &mut gs, "trial")), none(), "{flags:?}");
    }
    let mut gs = state(&d, &[(CHOSEN, json!("lead"))]);
    assert_eq!(granted(&begin(&d, &mut gs, "plain")), none(), "an encounter without the field");
}

#[test]
fn the_grant_lasts_the_battle_only() {
    let d = data();
    let mut gs = state(&d, &[(CHOSEN, json!("lead"))]);
    let before = gs.party.members[0].learned_skills.clone();
    let f = begin(&d, &mut gs, "trial");
    assert_eq!(granted(&f)[0].1, [GRANT]);
    let tally = f.tally();
    let res = battle::finish_world(&d, &mut gs, trigger::FIGHT_SOURCE, &fight("trial"), "won", Some(&tally));
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(gs.party.members[0].learned_skills, before, "nothing learned for good");
    gs.flags.shift_remove(CHOSEN);
    assert_eq!(granted(&begin(&d, &mut gs, "trial")), none(), "the next fight without the flag");
}
