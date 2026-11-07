//! W.2-S1 (FB-50a, W_WORLD.md section 3) on a scratch world: `WorldRoster::new` is the one constructor of a world
//! battle's roster (it resolves the world's characters like the struct it replaced), and `battle::finish_world` settles
//! a world fight by the runner that started it: each runner's settle exactly (`travel`, `dialogue`, `story`, `trigger`,
//! anything else `explore`), on the same state with the same result. A trigger fight settled through it ticks the
//! campaign like the trigger runner. Every expectation follows from the dispatch rule and the fixture's own numbers.

use aegis_core::json::{self, Dict};
use rpgkit_battle::session::Roster;
use rpgkit_world::battle::{self, WorldRoster};
use rpgkit_world::campaign::trigger;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{dialogue, explore, story, travel, Data, GameState};
use serde_json::{json, Value};

const LOUD_WON: i64 = 8;

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

/// World `w`: a campaign from act 1 with an `alert` gauge moved +8 by a won loud battle, the hub, the encounter
/// `enc_loud` (tagged loud, no enemies) and the character Clerk (a party-side unit row).
fn data() -> Data {
    let w = "w";
    let meters = json!([{"id": "alert", "name": "Alert", "max": 100, "bands": [{"id": "calm", "name": "Calm", "from": 0, "tone": "calm"}],
                         "inputs": [{"on": "battle", "tags": ["loud"], "outcomes": ["won"], "delta": LOUD_WON}]}]);
    let worlds = json!({
        w: {"manifest": {"id": w, "start_location": "hub", "start_party": ["hero"], "campaign": {"start_act": 1}}, "own": {
            "locations": with_ids(w, json!([{"id": "hub", "name": "Hub"}])),
            "encounters": with_ids(w, json!([{"id": "enc_loud", "name": "Loud", "tags": ["loud"], "enemies": []}])),
            "characters": with_ids(w, json!([{"id": "clerk", "name": "Clerk", "role": "npc", "base_unit": "hero"}])),
            "meters": with_ids(w, meters)
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
    let mut gs = GameState::create(d, "w", party);
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    if let Some(c) = gs.campaign.as_mut() {
        c.sync(gs.time);
    }
    gs
}

fn alert(gs: &GameState) -> i64 {
    gs.campaign.as_ref().map_or(-1, |c| c.value("alert", gs.time))
}

type Settle = fn(&Data, &mut GameState, &Dict, &str, Option<&battle::Tally>) -> Dict;

#[test]
fn finish_world_settles_each_runner_exactly_like_its_own_after_fight() {
    let d = data();
    let runners: [(&str, Settle); 6] = [
        ("explore", explore::after_fight),
        ("travel", travel::after_fight),
        ("dialogue", dialogue::after_fight),
        (story::FIGHT_SOURCE, story::after_fight),
        (trigger::FIGHT_SOURCE, trigger::after_fight),
        ("some_other_source", explore::after_fight),
    ];
    for (runner, own) in runners {
        for outcome in battle::OUTCOMES {
            let fight = json::dict(&json!({"source": runner, "encounter": "w:enc_loud"})).clone();
            let (mut via_kit, mut direct) = (state(&d), state(&d));
            let got = battle::finish_world(&d, &mut via_kit, runner, &fight, outcome, None);
            let want = own(&d, &mut direct, &fight, outcome, None);
            assert_eq!(got, want, "{runner} {outcome}: the same result");
            assert_eq!(via_kit, direct, "{runner} {outcome}: the same state after");
        }
    }
}

#[test]
fn a_trigger_fight_finished_in_the_kit_ticks_the_campaign() {
    let d = data();
    let mut gs = state(&d);
    let before = alert(&gs);
    let fight = json::dict(&json!({"source": trigger::FIGHT_SOURCE, "encounter": "w:enc_loud"})).clone();
    let res = battle::finish_world(&d, &mut gs, trigger::FIGHT_SOURCE, &fight, "won", None);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(alert(&gs), before + LOUD_WON, "the battle tick of a won loud fight");
}

#[test]
fn world_roster_new_resolves_the_worlds_characters() {
    let d = data();
    let cat = d.cat.clone();
    let from_str = WorldRoster::new(&d, "w");
    let from_string = WorldRoster::new(&d, String::from("w"));
    assert_eq!((from_str.world.as_str(), from_string.world.as_str()), ("w", "w"));
    assert!(std::ptr::eq(from_str.d, &d), "the roster reads the given data");
    assert_eq!(from_str.lore_ref(&cat, "clerk"), "w:clerk", "a character's lore ref is its qualified id");
    assert_eq!(from_str.entry(&cat, "clerk").map(|u| u.id), from_string.entry(&cat, "clerk").map(|u| u.id), "either form resolves alike");
    assert!(from_str.entry(&cat, "clerk").is_some(), "the world's character spawns as its unit");
    assert!(WorldRoster::new(&d, "elsewhere").entry(&cat, "clerk").is_none(), "another world's roster does not see it");
}
