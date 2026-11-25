//! ACT1-HOOKS1 (FB-241 c): travel supplies as a reward: a quest template's `rewards.supplies` (paid on completion, in
//! `granted.supplies`) and a dialogue `give` effect's `supplies`, both through `quest::reward_apply`: added to the
//! party's supplies up to the purse's `supplies_max`, the rest a note (`terms::upkeep_capped`); `granted.supplies` is
//! there only when the rewards name some, so a result without them reads as before. Every number is read from the
//! fixture or the party's purse.

use aegis_core::json::{self, Dict};
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{quest, terms, Data, GameState};
use serde_json::{json, Value};

const QUEST_SUPPLIES: i64 = 4;
const GIVE_SUPPLIES: i64 = 3;

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

/// World `w`: the quest `haul` (template `haul_job`: `QUEST_SUPPLIES` supplies and 5 gold) and the scene `gift` whose
/// one reply gives `GIVE_SUPPLIES` supplies.
fn data() -> Data {
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["hero"]}, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub"}])),
        "characters": with_ids("w", json!([{"id": "hero", "name": "Hero", "role": "party"}])),
        "quests": with_ids("w", json!([{"id": "haul", "name": "Haul", "template": "haul_job"}])),
        "dialogues": with_ids("w", json!([{"id": "gift", "start": "g", "services": false, "nodes": [
            {"id": "g", "text": "Take these.", "choices": [{"text": "Thanks.", "to": "", "effects": [{"effect": "give", "supplies": GIVE_SUPPLIES}]}]}
        ]}]))
    }}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    let game = json!({"quests": [{"id": "haul_job", "objectives": [{"kind": "win_encounter"}], "rewards": {"gold": 5, "supplies": QUEST_SUPPLIES}}]});
    Data::from_values(&catalog, &worlds, &game)
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    GameState::create(d, "w", party)
}

fn granted(res: &Dict) -> &Dict {
    json::dget(res, "granted")
}

#[test]
fn reward_supplies_fill_the_purse_up_to_its_cap_and_note_the_rest() {
    let d = data();
    let mut gs = state(&d);
    let before = gs.party.supplies;
    let res = quest::reward_apply(&d, &mut gs, json::dict(&json!({"supplies": 2})), "", &[]);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!((gs.party.supplies, json::i(granted(&res), "supplies", -1)), (before + 2, 2));
    let plain = quest::reward_apply(&d, &mut gs, json::dict(&json!({"gold": 1})), "", &[]);
    assert!(!granted(&plain).contains_key("supplies"), "no supplies named: no key ({plain:?})");
    let room = gs.party.purse.supplies_max - gs.party.supplies;
    let over = quest::reward_apply(&d, &mut gs, json::dict(&json!({"supplies": room + 5})), "", &[]);
    assert_eq!((gs.party.supplies, json::i(granted(&over), "supplies", -1)), (gs.party.purse.supplies_max, room), "capped at supplies_max");
    assert_eq!(json::sget(&over, "notes"), [terms::upkeep_capped(&d, "w", 5)], "the rest is a note");
    let bad = quest::reward_apply(&d, &mut gs, json::dict(&json!({"supplies": -1})), "", &[]);
    assert!(!json::b(&bad, "ok", true), "a negative grant is refused");
}

#[test]
fn a_completed_quest_pays_its_templates_supplies() {
    let d = data();
    let mut gs = state(&d);
    gs.quests.insert("w:haul".into(), json!({"state": "ready", "objective": 1}));
    assert_eq!(quest::status(&d, &gs, "w:haul"), "ready");
    let before = gs.party.supplies;
    let res = quest::complete(&d, &mut gs, "w:haul", "", &[]);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(json::i(granted(&res), "supplies", -1), QUEST_SUPPLIES);
    assert_eq!(gs.party.supplies, before + QUEST_SUPPLIES);
}

#[test]
fn a_dialogue_give_hands_over_supplies() {
    let d = data();
    let mut gs = state(&d);
    let before = gs.party.supplies;
    let mut r = DialogueRunner::open_scene(&d, &mut gs, "gift", "");
    let res = r.choose(&d, &mut gs, 0);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(gs.party.supplies, before + GIVE_SUPPLIES);
}
