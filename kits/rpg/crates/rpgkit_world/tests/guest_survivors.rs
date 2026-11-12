//! ACT1-HOOKS1 (FB-241 c): a world encounter's `survivors` `{"flag", "unit"}` sets the flag, at every settle (won, lost
//! or fled), to how many of its guests lived (of `unit`: a guest's ref, character id or base unit; "" = every guest).
//! A world fight's tally carries its guests (`Tally::of_world_fight`: alive and still on the party's side); a fight
//! settled without a battle counts every guest alive (`Tally::default_for`); a tally built by hand (no guests known)
//! leaves the flag as it is, and so does an encounter without the field. Every count is read from the fixture's own
//! guests and the battle's units.

use aegis_core::json::{self, Dict};
use rpgkit_world::battle::{self, Tally};
use rpgkit_world::campaign::trigger;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{explore, Data, GameState};
use serde_json::{json, Value};

const SAVED: &str = "w.saved";
const ALL: &str = "w.all_alive";
const KIND: &str = "captive";

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

/// The rescue's guests (a battle takes at most two): a character on the `KIND` base unit and an aide on another.
const GUESTS: [&str; 2] = ["cap_a", "aide"];

/// World `w`: the encounters `rescue` (counts the `KIND` guests into `SAVED`), `rescue_all` (every guest into `ALL`) and
/// `plain` (the same guests, no field), each against one rat.
fn data() -> Data {
    let enc = |id: &str, survivors: Value| {
        let mut e = json!({"id": id, "name": id, "enemies": ["rat"], "guests": GUESTS});
        if !survivors.is_null() {
            e["survivors"] = survivors;
        }
        e
    };
    let worlds = json!({"w": {"manifest": {"id": "w", "start_location": "hub", "start_party": ["hero"]}, "own": {
        "locations": with_ids("w", json!([{"id": "hub", "name": "Hub"}])),
        "encounters": with_ids("w", json!([
            enc("rescue", json!({"flag": SAVED, "unit": KIND})),
            enc("rescue_all", json!({"flag": ALL, "unit": ""})),
            enc("plain", Value::Null)
        ])),
        "characters": with_ids("w", json!([
            {"id": "rat", "name": "Rat", "role": "npc", "base_unit": "rat"},
            {"id": "cap_a", "name": "Cap A", "role": "npc", "base_unit": KIND},
            {"id": "cap_b", "name": "Cap B", "role": "npc", "base_unit": KIND},
            {"id": "aide", "name": "Aide", "role": "npc", "base_unit": "aide"}
        ]))
    }}});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true},
            {"id": "rest", "name": "Rest", "kind": "REST", "level": 1, "blocks": 1, "system": true, "basic": true, "recover_ap": 2}
        ],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 3},
            {"id": "rat", "name": "Rat", "side": "enemy", "class_id": "drill", "level": 1},
            {"id": KIND, "name": "Captive", "side": "party", "class_id": "drill", "level": 1},
            {"id": "aide", "name": "Aide", "side": "party", "class_id": "drill", "level": 1}
        ]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    gs
}

fn fight(enc: &str) -> Dict {
    json::dict(&json!({"source": trigger::FIGHT_SOURCE, "encounter": format!("w:{enc}")})).clone()
}

/// The fight `enc` started headlessly, guest `dead` killed on the board, settled as `outcome` with the battle's tally.
fn settle_with_a_loss(d: &Data, enc: &str, dead: &str, outcome: &str) -> (GameState, Tally) {
    let mut gs = state(d);
    let mut f = battle::begin_world(d, &mut gs, &fight(enc)).unwrap_or_else(|e| panic!("{enc} starts: {e}"));
    for c in f.session.units_of_ref(dead) {
        f.session.w.units[c].hp = 0;
    }
    let tally = f.tally();
    let res = battle::finish_world(d, &mut gs, trigger::FIGHT_SOURCE, &fight(enc), outcome, Some(&tally));
    assert!(json::b(&res, "ok", false), "{res:?}");
    (gs, tally)
}

#[test]
fn a_world_fight_tally_knows_which_guests_lived() {
    let d = data();
    let (_, tally) = settle_with_a_loss(&d, "rescue", "cap_a", "won");
    let guests = tally.guests.expect("a world fight knows its guests");
    assert_eq!(guests.iter().map(|g| g.unit_ref.as_str()).collect::<Vec<_>>(), GUESTS, "every guest, encounter order");
    assert_eq!(guests.iter().filter(|g| !g.alive).map(|g| g.unit_ref.as_str()).collect::<Vec<_>>(), ["cap_a"], "only the one killed");
}

#[test]
fn the_survivors_flag_counts_the_guests_of_its_unit_on_every_outcome() {
    let d = data();
    let of_kind = GUESTS.iter().filter(|g| g.starts_with("cap")).count() as i64;
    for outcome in battle::OUTCOMES {
        let (gs, _) = settle_with_a_loss(&d, "rescue", "cap_a", outcome);
        assert_eq!(gs.flag(SAVED), Some(&json::int(of_kind - 1)), "{outcome}: the captives that lived");
        let (gs, _) = settle_with_a_loss(&d, "rescue", "aide", outcome);
        assert_eq!(gs.flag(SAVED), Some(&json::int(of_kind)), "{outcome}: another guest's death is not counted");
        let (gs, _) = settle_with_a_loss(&d, "rescue_all", "aide", outcome);
        assert_eq!(gs.flag(ALL), Some(&json::int(GUESTS.len() as i64 - 1)), "{outcome}: unit \"\" counts every guest");
        let (gs, _) = settle_with_a_loss(&d, "plain", "aide", outcome);
        assert!(gs.flag(SAVED).is_none() && gs.flag(ALL).is_none(), "{outcome}: no field, no flag");
    }
}

#[test]
fn a_settle_without_a_battle_counts_every_guest_and_a_hand_tally_leaves_the_flag() {
    let d = data();
    let of_kind = GUESTS.iter().filter(|g| g.starts_with("cap")).count() as i64;
    let mut gs = state(&d);
    let res = battle::finish_world(&d, &mut gs, trigger::FIGHT_SOURCE, &fight("rescue"), "won", None);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(gs.flag(SAVED), Some(&json::int(of_kind)), "the default tally: every guest lived");
    let hand = Tally { world: "w".into(), ..Tally::default() };
    assert!(hand.guests.is_none());
    gs.set_flag(SAVED, json::int(7));
    battle::finish_world(&d, &mut gs, trigger::FIGHT_SOURCE, &fight("rescue"), "won", Some(&hand));
    assert_eq!(gs.flag(SAVED), Some(&json::int(7)), "no guests known: the flag is left as it is");
}
