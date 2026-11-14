//! ACT1-HOOKS2 (FB-244 (1)) on a scratch world built here (no shipped data): the dialogue condition `in_active_party`
//! holds for a deployed party character only (a benched member, an unknown or non-member character fails it, and the
//! locked reply says who must be deployed); the effect `install` `{member_from_flag, item, slot}` puts the graft into
//! the body of the member whose character id the flag holds, through the recipe's install path
//! (`UnitState::install_graft`): with no slot, the first empty slot of the catalog that takes it; a named occupied slot
//! is overwritten (the old graft destroyed); a member who already carries the item, a flag naming nobody and a graft
//! that fits no empty slot change nothing and say why. A dialogue choice and a sequence's effects step both run it.

use aegis_core::json::{self, Dict};
use rpgkit_world::dialogue::{self, DialogueRunner};
use rpgkit_world::state::PartyState;
use rpgkit_world::{new_game, story, Data, GameState};
use serde_json::{json, Value};

const CHOSEN: &str = "w.chosen";

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

fn graft(id: &str, role: &str) -> Value {
    json!({"id": id, "name": format!("Graft {id}"), "item_type": "GRAFT", "tier": 1, "body_role": role, "body_effects": []})
}

/// Body slots: a class slot first, then two monster slots; two monster grafts and a class graft. The talker's one
/// choice runs `effects`; sequence `seq_install` is one effects step running `effects`.
fn data(effects: Value) -> Data {
    let stats = json!({"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": stats}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}],
        "items": [graft("g_one", "monster"), graft("g_two", "monster"), graft("g_rank", "class")],
        "body_slots": [{"id": "s_rank", "name": "Rank", "role": "class"}, {"id": "s_a", "name": "A", "role": "monster"}, {"id": "s_b", "name": "B", "role": "monster"}]
    });
    let worlds = json!({"w": {"manifest": {"id": "w"}, "own": {
        "characters": with_ids("w", json!([
            {"id": "lead", "name": "Lead", "role": "party", "base_unit": "hero"},
            {"id": "second", "name": "Second", "role": "party", "base_unit": "hero"},
            {"id": "stray", "name": "Stray", "role": "party", "base_unit": "hero"},
            {"id": "clerk", "name": "Clerk", "role": "npc", "dialogue": "talk"}
        ])),
        "dialogues": with_ids("w", json!([{"id": "talk", "start": "a", "services": false, "nodes": [{"id": "a", "text": "Well?",
            "choices": [{"text": "Do it.", "effects": effects.clone()}]}]}])),
        "sequences": with_ids("w", json!([{"id": "seq_install", "name": "Install", "steps": [{"kind": "effects", "effects": effects}]}]))
    }}});
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// Lead deployed, Second on the bench (both members built from their characters).
fn state(d: &Data) -> GameState {
    let members = ["lead", "second"].iter().map(|id| new_game::hero_from_character(d, "w", id).expect("a party character")).collect();
    GameState::create(d, "w", PartyState { members, active_ids: vec!["lead".into()], ..PartyState::default() })
}

fn install(extra: Value) -> Value {
    let mut fx = json!({"effect": "install", "member_from_flag": CHOSEN, "item": "g_one"});
    for (k, v) in extra.as_object().unwrap() {
        fx[k] = v.clone();
    }
    fx
}

/// The talker's choice once, with `CHOSEN` holding `who`: the result's notes.
fn choose(d: &Data, gs: &mut GameState, who: &str) -> Vec<String> {
    gs.set_flag(CHOSEN, json!(who));
    let mut r = DialogueRunner::open(d, gs, "clerk");
    let res = r.choose(d, gs, 0);
    assert!(json::b(&res, "ok", false), "{res:?}");
    json::sget(&res, "notes")
}

fn body(gs: &GameState, member: &str) -> Vec<(String, String)> {
    gs.party.members.iter().find(|m| m.id == member).expect("a member").body.iter().map(|(s, i)| (s.clone(), i.clone())).collect()
}

fn pair(slot: &str, item: &str) -> (String, String) {
    (slot.to_string(), item.to_string())
}

fn cond(who: &str) -> Dict {
    json::dict(&json!({"kind": "in_active_party", "character": who})).clone()
}

#[test]
fn in_active_party_holds_for_a_deployed_member_only() {
    let d = data(json!([]));
    let mut gs = state(&d);
    assert!(dialogue::condition_holds(&d, &gs, &cond("lead")), "deployed");
    assert!(dialogue::condition_holds(&d, &gs, &cond("w:lead")), "by its qualified ref");
    assert!(!dialogue::condition_holds(&d, &gs, &cond("second")), "a member on the bench");
    assert!(!dialogue::condition_holds(&d, &gs, &cond("stray")), "a party character not in the party");
    assert!(!dialogue::condition_holds(&d, &gs, &cond("ghost")), "an unknown character");
    let gated = json::dict(&json!({"text": "Go.", "requires": [cond("second")]})).clone();
    assert!(!dialogue::choice_open(&d, &gs, &gated));
    assert_eq!(dialogue::lock_reason(&d, &gs, &gated), "Needs Second in the active party");
    gs.party.active_ids = vec!["second".into()];
    assert!(dialogue::choice_open(&d, &gs, &gated), "deployed now");
    assert!(!dialogue::condition_holds(&d, &gs, &cond("lead")), "benched now");
}

#[test]
fn install_fills_the_first_empty_slot_that_takes_the_graft_once() {
    let d = data(json!([install(json!({}))]));
    let mut gs = state(&d);
    let notes = choose(&d, &mut gs, "second");
    assert_eq!(body(&gs, "second"), [pair("s_a", "g_one")], "the first monster slot (the class slot does not take it)");
    assert!(notes.contains(&"Second takes Graft g_one.".to_string()), "{notes:?}");
    assert!(body(&gs, "lead").is_empty(), "only the member the flag names (benched or not)");
    let notes = choose(&d, &mut gs, "second");
    assert_eq!(body(&gs, "second"), [pair("s_a", "g_one")], "a replayed list installs once");
    assert!(notes.contains(&"Second already carries Graft g_one.".to_string()), "{notes:?}");
    // a qualified ref names the member too
    choose(&d, &mut gs, "w:lead");
    assert_eq!(body(&gs, "lead"), [pair("s_a", "g_one")]);
}

#[test]
fn install_changes_nothing_when_it_cannot_and_says_why() {
    for (who, note) in [("stray", "Flag 'w.chosen' names no party member; nothing is installed."), ("ghost", "Flag 'w.chosen' names no party member; nothing is installed.")] {
        let d = data(json!([install(json!({}))]));
        let mut gs = state(&d);
        let notes = choose(&d, &mut gs, who);
        assert!(notes.contains(&note.to_string()), "{who}: {notes:?}");
        assert!(gs.party.members.iter().all(|m| m.body.is_empty()), "{who}");
    }
    // a class graft: its slot opens with the class rite, so no empty slot takes it
    let d = data(json!([install(json!({"item": "g_rank"}))]));
    let mut gs = state(&d);
    let notes = choose(&d, &mut gs, "lead");
    assert!(notes.contains(&"Lead: no empty body slot takes Graft g_rank.".to_string()), "{notes:?}");
    assert!(body(&gs, "lead").is_empty());
    // a named slot of the wrong role: the install path's refusal
    let d = data(json!([install(json!({"slot": "s_rank"}))]));
    let mut gs = state(&d);
    let notes = choose(&d, &mut gs, "lead");
    assert!(notes.contains(&"Lead: body slot s_rank takes class grafts; Graft g_one is a monster graft.".to_string()), "{notes:?}");
    assert!(body(&gs, "lead").is_empty());
}

#[test]
fn a_named_occupied_slot_is_overwritten_and_the_old_graft_destroyed() {
    let d = data(json!([install(json!({})), install(json!({"item": "g_two", "slot": "s_a"}))]));
    let mut gs = state(&d);
    let notes = choose(&d, &mut gs, "lead");
    assert_eq!(body(&gs, "lead"), [pair("s_a", "g_two")]);
    assert!(notes.contains(&"Graft g_one is destroyed.".to_string()), "{notes:?}");
}

#[test]
fn a_sequence_effects_step_installs_too() {
    let d = data(json!([install(json!({"item": "g_two"}))]));
    let mut gs = state(&d);
    gs.set_flag(CHOSEN, json!("lead"));
    let (mut notes, mut events) = (Vec::new(), Vec::new());
    story::start_quiet(&d, &mut gs, "seq_install", &mut notes, &mut events).expect("the sequence starts");
    assert_eq!(body(&gs, "lead"), [pair("s_a", "g_two")]);
    assert!(notes.contains(&"Lead takes Graft g_two.".to_string()), "{notes:?}");
}
