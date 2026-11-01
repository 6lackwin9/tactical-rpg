//! PORTRAITFX1-X (docs/specs/PORTRAITFX1.md 5.4, 7; checks X2, X3) on a scratch world: a dialogue node's `expression`
//! reaches the talk box through `DialogueRunner::view` (only when the node names one, so a node without one reads as
//! before), and a `line` card's portrait chain is the speaker's `face_chain` for the card's `expression`
//! (`story::card_portrait`). Expectations follow from the rules of 5.4 and the fixture's own data.

use aegis_core::json::{self, Dict};
use rpgkit_content::art;
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{story, Data, GameState};
use serde_json::{json, Value};

fn with_ids(w: &str, rows: Value) -> Value {
    let rows = json::arr(&rows)
        .iter()
        .map(|r| {
            let mut r = json::dict(r).clone();
            let id = json::s(&r, "id");
            r.insert("_world".into(), w.into());
            r.insert("_qid".into(), format!("{w}:{id}").into());
            Value::Object(r)
        })
        .collect();
    Value::Array(rows)
}

/// World `tale`: npc `clerk` (a unique tier: npc) whose talk runs node `a` (`angry`), `b` (no key) and `c` ("", the
/// end); party character `ally` (tier hero); enemy `grunt` (a common tier).
fn data() -> Data {
    let dialogues = json!([{"id": "talk", "start": "a", "services": false, "nodes": [
        {"id": "a", "expression": "angry", "text": "Out!", "next": "b"},
        {"id": "b", "text": "Well?", "next": "c"},
        {"id": "c", "expression": "", "text": "Go.", "end": true}
    ]}]);
    let worlds = json!({
        "tale": {"manifest": {"id": "tale", "start_location": "hub", "start_party": ["hero"]}, "own": {
            "locations": with_ids("tale", json!([{"id": "hub", "name": "Hub"}])),
            "characters": with_ids("tale", json!([
                {"id": "clerk", "name": "Clerk", "role": "npc", "base_unit": "hero", "dialogue": "talk"},
                {"id": "ally", "name": "Ally", "role": "party", "base_unit": "hero"},
                {"id": "grunt", "name": "Grunt", "role": "enemy", "base_unit": "hero"}
            ])),
            "dialogues": with_ids("tale", dialogues)
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    GameState::create(d, "tale", party)
}

/// The rule of 5.4 for character `id` of `d`: `face_chain` over its tier, id and body.
fn chain_of(d: &Data, id: &str, expression: &str) -> Vec<String> {
    let c = d.resolve("tale", "characters", id);
    let unit = d.entry("units", &json::s(c, "base_unit"));
    let class = d.entry("classes", &json::s(unit, "class_id"));
    art::face_chain(&art::portrait_tier(c, unit), id, &art::body_of(c, unit, class), expression)
}

/// X2 (kit part): the view names the node's expression only when the node has one; `talk_portrait` keeps the speaker.
#[test]
fn the_dialogue_view_carries_a_nodes_expression_only_when_set() {
    let d = data();
    let mut gs = state(&d);
    let mut talk = DialogueRunner::open(&d, &mut gs, "clerk");
    let a = talk.view(&d, &gs);
    assert_eq!(json::s(&a, "node"), "a");
    assert_eq!(a.get("expression"), Some(&json!("angry")), "{a:?}");
    talk.advance(&d, &mut gs);
    let b = talk.view(&d, &gs);
    assert_eq!(json::s(&b, "node"), "b");
    assert!(b.get("expression").is_none(), "a node without the key reads as before: {b:?}");
    talk.advance(&d, &mut gs);
    let c = talk.view(&d, &gs);
    assert_eq!(json::s(&c, "node"), "c");
    assert!(c.get("expression").is_none(), "an empty expression is left out: {c:?}");
}

/// X3: a `line` card resolves its portrait chain with its `expression`: the frame, the neutral frame, then the single
/// portrait (5.4); "" and `neutral` start at the neutral frame; a common tier has no frames; no speaker, none.
#[test]
fn a_line_card_resolves_its_face_chain_with_its_expression() {
    let d = data();
    let gs = state(&d);
    let card = |speaker: &str, expression: &str| -> Dict { json::dict(&json!({"style": "line", "speaker": speaker, "expression": expression, "text": "Hi."})).clone() };
    let angry = story::card_portrait(&d, "tale", &card("clerk", "angry"));
    assert_eq!(angry, chain_of(&d, "clerk", "angry"));
    assert_eq!(angry[..3], ["portraits/npcs/faces/clerk/angry.png", "portraits/npcs/faces/clerk/neutral.png", "portraits/npcs/clerk.png"]);
    let plain = story::card_portrait(&d, "tale", &card("clerk", ""));
    assert_eq!(plain[..2], ["portraits/npcs/faces/clerk/neutral.png", "portraits/npcs/clerk.png"]);
    assert_eq!(story::card_portrait(&d, "tale", &card("clerk", "neutral")), plain, "neutral is the plain face");
    assert_eq!(story::card_portrait(&d, "tale", &card("ally", "hurt"))[0], "portraits/heroes/faces/ally/hurt.png");
    // a common tier: only the single portrait chain, whatever the expression
    let c = d.resolve("tale", "characters", "grunt");
    let unit = d.entry("units", "hero");
    assert_eq!(art::portrait_tier(c, unit), "common");
    assert_eq!(story::card_portrait(&d, "tale", &card("grunt", "hurt")), art::portrait_chain("common", "grunt", &art::body_of(c, unit, d.entry("classes", "fighter"))));
    assert!(story::card_portrait(&d, "tale", &card("", "hurt")).is_empty(), "no speaker, no portrait");
    assert!(story::card_portrait(&d, "tale", &card("nobody", "hurt")).is_empty(), "an unknown speaker, no portrait");
    // the card as it plays (`card_now`: the speaker becomes its qid) resolves the same chain
    let now = story::card_now(&d, &gs, "tale", &card("clerk", "angry")).expect("plays");
    assert_eq!(json::s(&now, "speaker"), "tale:clerk");
    assert_eq!(json::s(&now, "expression"), "angry", "the card keeps its expression");
    assert_eq!(story::card_portrait(&d, "tale", &now), angry);
}
