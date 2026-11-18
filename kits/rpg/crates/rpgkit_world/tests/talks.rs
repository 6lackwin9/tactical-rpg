//! BUNKS1 BK5-BK8 (spec BUNKS1 section 2): bunk talks. The pair counter counts each pair of
//! members who came home from a won or withdrawn battle (sorted `a|b` keys, saved only when not empty); a talk waits
//! while both its members live in bunks, its flags hold, the counter reaches `min_together` and its seen flag is
//! unset; template talks pair each generated recruit their temperament (else origin) names with the first founder and
//! play once per recruit; playing one sets the seen flag, gives the reward and fills `{a}` and `{b}`. Every
//! expectation follows from the fixture's own constants.

use aegis_core::json;
use rpgkit_data::talk::{pair_key, seen_key};
use rpgkit_world::state::UnitState;
use rpgkit_world::{new_game, roster, talks, Data, GameState};
use serde_json::{json, Value};

const MIN: i64 = 2;
const FLAG: &str = "w.ready";
const TEMPER: &str = "calm_mind";
const ORIGIN: &str = "far_land";
const GIFT: &str = "token";

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

fn node(text: &str) -> Value {
    json!({"id": "n0", "text": text, "end": true})
}

/// World `w` with a roster block, three party characters (`ana`, `ben`, `cid`) and `rows` as its talks; dialogues
/// `d_pair` and `d_tpl` (the template's line names `{a}` and `{b}`).
fn data_with(rows: Value) -> Data {
    let ch = |id: &str| json!({"id": id, "name": id.to_uppercase(), "role": "party", "base_unit": "hero"});
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["hero"], "party_max": 4, "roster": {"bunks": 6}}, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub", "rest": true}])),
        "characters": with_ids("w", json!([ch("ana"), ch("ben"), ch("cid")])),
        "dialogues": with_ids("w", json!([{"id": "d_pair", "name": "Pair", "start": "n0", "services": false, "nodes": [node("We made it.")]},
                                          {"id": "d_tpl", "name": "Tpl", "start": "n0", "services": false, "nodes": [node("{a} sits by {b}.")]}])),
        "talks": with_ids("w", rows)
    }}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}],
        "items": [{"id": GIFT, "name": "Token", "item_type": "ACCESSORY", "value": 1}],
        "passives": [{"id": TEMPER, "name": "Calm", "slot": ""}, {"id": ORIGIN, "name": "Far", "slot": ""}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// A game whose party is the characters `ana`, `ben`, `cid` (all active, roster order).
fn game(d: &Data) -> GameState {
    let mut gs = new_game::create_for_world(d, "w", 1).expect("a new game");
    gs.party.members = ["ana", "ben", "cid"].iter().map(|c| new_game::hero_from_character(d, "w", c).expect("a party character")).collect();
    gs.party.active_ids = Vec::new();
    gs
}

/// A generated recruit `id` with temperament `temper` and origin `origin`.
fn recruit(d: &Data, id: &str, temper: &str, origin: &str) -> UnitState {
    let mut m = UnitState::at_level(d, "hero", 1, id);
    m.name = format!("Rec {id}");
    m.template = "w:tpl".into();
    m.temperament = temper.into();
    m.origin = origin.into();
    m
}

fn pair(id: &str, a: &str, b: &str, extra: Value) -> Value {
    let mut row = json!({"id": id, "name": id, "pair": [a, b], "dialogue": "d_pair"});
    for (k, v) in json::dict(&extra) {
        row[k] = v.clone();
    }
    row
}

fn settle(d: &Data, gs: &mut GameState, outcome: &str) {
    let ends = roster::default_ends(d, gs, "w", outcome);
    roster::settle_party(d, gs, &roster::FightEnd::new(outcome, "w"), &ends);
}

#[test]
fn the_pair_counter_counts_won_and_withdrawn_battles_side_by_side() {
    let d = data_with(json!([]));
    let mut gs = game(&d);
    assert!(gs.party.together.is_empty() && !gs.party.to_value().as_object().unwrap().contains_key("together"), "nothing saved before a battle");
    gs.party.active_ids = vec!["ben".into(), "ana".into()];
    settle(&d, &mut gs, "won");
    settle(&d, &mut gs, "fled");
    settle(&d, &mut gs, "lost");
    assert_eq!(talks::together(&gs, "ana", "ben"), 2, "a win and a withdrawal count, a loss does not");
    assert_eq!(talks::together(&gs, "ben", "ana"), 2, "the key is sorted");
    assert_eq!(talks::together(&gs, "ana", "cid"), 0, "a benched member fought with no one");
    assert_eq!(gs.party.together.keys().cloned().collect::<Vec<_>>(), vec![pair_key("ana", "ben")]);
    assert_eq!(pair_key("ben", "ana"), "ana|ben");
    let saved = gs.party.to_value();
    assert_eq!(saved["together"], json!({"ana|ben": 2}));
    let back = rpgkit_world::state::PartyState::from_dict(json::dict(&saved));
    assert_eq!(back.together, gs.party.together, "the counter round-trips");
}

#[test]
fn a_written_talk_waits_on_its_flags_counter_bunks_and_seen_flag() {
    let d = data_with(json!([pair("t1", "ana", "ben", json!({"min_together": MIN, "requires_flags": [{"key": FLAG, "value": true}]}))]));
    let mut gs = game(&d);
    assert!(talks::waiting(&d, &gs).is_empty(), "flag unset, counter 0");
    gs.set_flag(FLAG, Value::Bool(true));
    assert!(talks::waiting(&d, &gs).is_empty(), "the counter is below min_together");
    gs.party.together.insert(pair_key("ana", "ben"), MIN);
    let w = talks::waiting(&d, &gs);
    assert_eq!(w.len(), 1);
    assert_eq!((w[0].a.as_str(), w[0].b.as_str(), w[0].seen.as_str()), ("ana", "ben", "talk.t1"));
    assert_eq!(seen_key("t1", ""), "talk.t1");
    // a member in the crypts has no bunk talk
    gs.party.members[1].housing = "crypt".into();
    assert!(talks::waiting(&d, &gs).is_empty(), "both must live in bunks");
    gs.party.members[1].housing = String::new();
    // the seen flag ends it
    gs.set_flag("talk.t1", Value::Bool(true));
    assert!(talks::waiting(&d, &gs).is_empty(), "a seen talk does not wait");
}

#[test]
fn a_member_shows_its_oldest_waiting_talk() {
    let d = data_with(json!([pair("first", "ana", "ben", json!({})), pair("second", "ana", "cid", json!({})), pair("third", "ben", "cid", json!({}))]));
    let gs = game(&d);
    assert_eq!(talks::waiting(&d, &gs).len(), 3);
    assert_eq!(talks::waiting_for(&d, &gs, "ana").map(|w| w.id), Some("first".into()));
    assert_eq!(talks::waiting_for(&d, &gs, "cid").map(|w| w.id), Some("second".into()), "table order: the oldest first");
}

#[test]
fn template_talks_pick_recruits_by_temperament_or_origin_with_the_first_founder() {
    let d = data_with(json!([{"id": "by_temper", "name": "T", "template": true, "temperament": TEMPER, "dialogue": "d_tpl"},
                             {"id": "by_origin", "name": "O", "template": true, "origin": ORIGIN, "dialogue": "d_tpl", "min_together": MIN}]));
    let mut gs = game(&d);
    gs.party.members.push(recruit(&d, "recruit_1", TEMPER, "other"));
    gs.party.members.push(recruit(&d, "recruit_2", "other", ORIGIN));
    gs.party.members.push(recruit(&d, "recruit_3", "other", "other"));
    let w = talks::waiting(&d, &gs);
    assert_eq!(w.iter().map(|w| (w.id.as_str(), w.a.as_str(), w.b.as_str())).collect::<Vec<_>>(), vec![("by_temper", "ana", "recruit_1")], "the origin talk waits on its counter; recruit_3 fits none");
    gs.party.together.insert(pair_key("ana", "recruit_2"), MIN);
    let w = talks::waiting(&d, &gs);
    assert_eq!(w.iter().map(|w| (w.id.as_str(), w.b.as_str(), w.seen.as_str())).collect::<Vec<_>>(), vec![("by_temper", "recruit_1", "talk.by_temper.recruit_1"), ("by_origin", "recruit_2", "talk.by_origin.recruit_2")]);
    // the founder is the first non-generated bunk member: with ana in the crypts it is ben
    gs.party.members[0].housing = "crypt".into();
    assert_eq!(talks::founder(&gs).map(|m| m.id.clone()), Some("ben".into()));
    assert_eq!(talks::waiting(&d, &gs)[0].a, "ben");
}

#[test]
fn playing_a_talk_sets_its_seen_flag_gives_the_reward_and_fills_the_names() {
    let d = data_with(json!([{"id": "tpl", "name": "T", "template": true, "temperament": TEMPER, "dialogue": "d_tpl", "reward": GIFT}]));
    let mut gs = game(&d);
    gs.party.members.push(recruit(&d, "recruit_1", TEMPER, ""));
    gs.party.members.push(recruit(&d, "recruit_2", TEMPER, ""));
    let w = talks::waiting_for(&d, &gs, "recruit_1").expect("a waiting template talk");
    let (r, reward) = talks::play(&d, &mut gs, &w);
    assert!(!r.closed && r.errors.is_empty(), "{:?}", r.errors);
    assert_eq!(json::s(&r.view(&d, &gs), "text"), "ANA sits by Rec recruit_1.");
    assert_eq!(reward, GIFT);
    assert_eq!(gs.party.stash.get(GIFT).copied(), Some(1));
    assert!(gs.flag_true("talk.tpl.recruit_1"));
    let left: Vec<String> = talks::waiting(&d, &gs).into_iter().map(|w| w.b).collect();
    assert_eq!(left, vec!["recruit_2".to_string()], "once per recruit: the other recruit still has it");
}

#[test]
fn a_talk_with_an_unknown_dialogue_sets_nothing() {
    let d = data_with(json!([{"id": "t", "name": "T", "pair": ["ana", "ben"], "dialogue": "missing"}]));
    let mut gs = game(&d);
    let w = talks::waiting_for(&d, &gs, "ana").expect("waits");
    let (r, reward) = talks::play(&d, &mut gs, &w);
    assert!(r.closed && !r.errors.is_empty());
    assert!(reward.is_empty() && gs.flag("talk.t").is_none(), "refused: still waiting");
}
