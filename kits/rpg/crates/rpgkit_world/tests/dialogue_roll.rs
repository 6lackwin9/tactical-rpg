//! T1.2-S2 (FB-54b, C25, T_STORY.md section 1 item 2 and section 4 test 1): the choice-time dialogue `roll` effect. The
//! choice row shows `[Roll: <Stat> >= <DC>] `; choosing it draws one face `1..dialogue_roll_die` from the save's
//! `dialogue` roll stream, adds the stat (the active party's best, or the active member `who`'s), and sets the outcome
//! flag true when `face + stat >= dc`, else false; the follow-up node's choices read the flag. Showing the choices never
//! draws; the stream is saved with the game, so a reload of the same state draws the same face (no save-scum). The die
//! is the world's `dialogue_roll_die` (nearest in the lineage, default 20).
//! T1.2-S3 (FB-54c): a dialogue stat names a U1 standing rating too (`initiative`, `move_rate`, `armor`, or a derived
//! stat row id), read from the member as built out of battle.
//! Every expected face is drawn from a copy of the state with `rolls::range_i`; every stat is printed from the build.

use aegis_core::json::{self, Dict};
use rpgkit_world::dialogue::{self, DialogueRunner};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{progression, rolls, Data, GameState};
use serde_json::{json, Value};

const STAT: &str = "STR";
const FLAG: &str = "lifted";

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

/// A world `w` (its `manifest` extras merged in) with two party characters and the tree `gate`: the start node's one
/// choice rolls `STAT` against `dc` (for `who`, when given) and goes to `after`, whose choices read the outcome flag.
fn data_with(manifest: Value, dc: i64, who: &str) -> Data {
    let mut roll = json!({"effect": "roll", "stat": STAT, "dc": dc, "key": FLAG});
    if !who.is_empty() {
        roll["who"] = who.into();
    }
    let nodes = json!([
        {"id": "start", "text": "A fallen beam blocks the way.", "choices": [{"text": "Lift it.", "to": "after", "effects": [roll]}]},
        {"id": "after", "text": "Well?", "choices": [
            {"text": "Through.", "requires": [{"kind": "flag", "key": FLAG, "value": true}]},
            {"text": "Around.", "requires": [{"kind": "flag", "key": FLAG, "value": false}]}
        ]}
    ]);
    let mut m = json!({"id": "w", "start_location": "hub", "start_party": ["hero"]});
    for (k, v) in manifest.as_object().unwrap() {
        m[k] = v.clone();
    }
    let worlds = json!({"w": {"manifest": m, "own": {
        "characters": with_ids("w", json!([{"id": "hero", "name": "Hero", "role": "party"}, {"id": "mate", "name": "Mate", "role": "party"}])),
        "dialogues": with_ids("w", json!([{"id": "gate", "start": "start", "services": false, "nodes": nodes}]))
    }}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}, {"id": "scout", "name": "Scout", "base_stats": {"STR": 6, "DEX": 16, "INT": 8, "CONS": 9, "PERC": 14, "KNOW": 7}}],
        "units": [
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1},
            {"id": "mate", "name": "Mate", "side": "party", "class_id": "scout", "level": 3}
        ],
        "derived_stats": [{"id": "grit", "name": "Grit", "formula": "CONS * 2 + 1", "min": 0, "max": 999}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn data(dc: i64) -> Data {
    data_with(json!({}), dc, "")
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    party.members.push(UnitState::create("mate", "mate"));
    GameState::create(d, "w", party)
}

/// The face the next roll of `gs` draws with a `die`, from a copy (the state is untouched).
fn next_face(gs: &GameState, die: i64) -> i64 {
    rolls::range_i(&mut gs.clone(), rolls::DIALOGUE, 1, die)
}

fn dialogue_rolls(gs: &GameState) -> i64 {
    gs.rolls.get(rolls::DIALOGUE).map(json::to_int).unwrap_or(0)
}

fn choice_texts(view: &Dict) -> Vec<String> {
    json::aget(view, "choices").iter().map(|c| json::s(json::dict(c), "text")).collect()
}

/// Opens `gate`, checks the tag and that showing draws nothing, picks the roll; the result and the follow-up node's
/// choice texts.
fn pick_roll(d: &Data, gs: &mut GameState, dc: i64) -> (Dict, Vec<String>) {
    let mut talk = DialogueRunner::open_scene(d, gs, "gate", "");
    let before = dialogue_rolls(gs);
    for _ in 0..3 {
        assert_eq!(choice_texts(&talk.view(d, gs)), [format!("[Roll: {STAT} >= {dc}] Lift it.")], "the inline tag");
    }
    assert_eq!(dialogue_rolls(gs), before, "showing the choice draws nothing");
    assert!(gs.flag(FLAG).is_none(), "and sets nothing");
    let res = talk.choose(d, gs, 0);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(dialogue_rolls(gs), before + 1, "one draw at choice time");
    let outcome = choice_texts(&talk.view(d, gs));
    assert_eq!(outcome.len(), 1, "the follow-up node shows the one outcome choice");
    (res, outcome)
}

#[test]
fn a_roll_succeeds_exactly_when_face_plus_stat_reaches_the_dc() {
    let probe = data(1);
    let gs0 = state(&probe);
    let best = dialogue::best_stat(&probe, &gs0, STAT);
    let face = next_face(&gs0, rpgkit_data::DIALOGUE_ROLL_DIE);
    assert!((1..=rpgkit_data::DIALOGUE_ROLL_DIE).contains(&face));
    for (dc, want) in [(face + best, true), (face + best + 1, false)] {
        let d = data(dc);
        let mut gs = state(&d);
        let (res, outcome) = pick_roll(&d, &mut gs, dc);
        assert_eq!(gs.flag(FLAG), Some(&Value::Bool(want)), "dc {dc}: face {face} + {STAT} {best}");
        let note = format!("Roll: {STAT} {face} + {best} = {} vs {dc}: {}.", face + best, if want { "success" } else { "failure" });
        assert!(json::sget(&res, "notes").contains(&note), "{note} in {res:?}");
        assert_eq!(outcome, [if want { "Through." } else { "Around." }], "the follow-up reads the flag");
    }
}

#[test]
fn a_reload_draws_the_same_face_and_the_stream_is_saved() {
    let d = data(1);
    let mut gs = state(&d);
    let saved = gs.to_value();
    let first = next_face(&gs, rpgkit_data::DIALOGUE_ROLL_DIE);
    pick_roll(&d, &mut gs, 1);
    let after = GameState::from_dict(&d, json::dict(&gs.to_value()));
    assert_eq!(dialogue_rolls(&after), 1, "the stream count is in the save");
    assert_ne!(next_face(&after, 1_000_000), next_face(&GameState::from_dict(&d, json::dict(&saved)), 1_000_000), "the next draw moved on");
    // reloading the save from before the choice gives the same face (and so the same outcome)
    let mut reloaded = GameState::from_dict(&d, json::dict(&saved));
    assert_eq!(next_face(&reloaded, rpgkit_data::DIALOGUE_ROLL_DIE), first);
    pick_roll(&d, &mut reloaded, 1);
    assert_eq!(reloaded.flag(FLAG), gs.flag(FLAG));
    assert_eq!(reloaded.rolls, gs.rolls);
}

#[test]
fn who_reads_that_member_and_the_die_follows_the_world() {
    let probe = data(1);
    let gs0 = state(&probe);
    let each: Vec<i64> = gs0.party.active_members().into_iter().map(|m| dialogue::member_stat(&probe, &gs0, m, STAT).unwrap()).collect();
    assert_ne!(each[0], each[1], "the fixture's members differ in {STAT}");
    assert_eq!(dialogue::best_stat(&probe, &gs0, STAT), *each.iter().max().unwrap(), "best is the active members' max");
    assert_eq!(dialogue::roll_stat(&probe, &gs0, STAT, "hero"), each[0]);
    assert_eq!(dialogue::roll_stat(&probe, &gs0, STAT, "mate"), each[1]);
    assert_eq!(dialogue::roll_stat(&probe, &gs0, STAT, "stranger"), 0, "no such member: 0");

    // `who`: the outcome follows that member's stat, not the best
    let face = next_face(&gs0, rpgkit_data::DIALOGUE_ROLL_DIE);
    let (lo, lo_who) = if each[0] <= each[1] { (each[0], "hero") } else { (each[1], "mate") };
    let dc = face + lo;
    let d = data_with(json!({}), dc, lo_who);
    let mut gs = state(&d);
    pick_roll(&d, &mut gs, dc);
    assert_eq!(gs.flag(FLAG), Some(&Value::Bool(true)), "face {face} + {lo_who}'s {lo} reaches {dc}");

    // the die: the world's dialogue_roll_die
    assert_eq!(dialogue::roll_die(&probe, "w"), rpgkit_data::DIALOGUE_ROLL_DIE, "default");
    let die = rpgkit_data::DIALOGUE_ROLL_DIE_MIN;
    let small = data_with(json!({"dialogue_roll_die": die}), 1, "");
    assert_eq!(dialogue::roll_die(&small, "w"), die);
    let mut gs = state(&small);
    let face = next_face(&gs, die);
    assert!((1..=die).contains(&face));
    let (res, _) = pick_roll(&small, &mut gs, 1);
    let best = dialogue::best_stat(&small, &gs, STAT);
    assert!(json::sget(&res, "notes").iter().any(|n| n.starts_with(&format!("Roll: {STAT} {face} + {best} ="))), "{res:?}");
    assert_eq!(dialogue::roll_die(&data_with(json!({"dialogue_roll_die": -1}), 1, ""), "w"), rpgkit_data::DIALOGUE_ROLL_DIE, "-1 inherits the default");
}

#[test]
fn a_stat_check_reads_the_standing_ratings_by_name() {
    let d = data(1);
    let gs = state(&d);
    let built: Vec<_> = gs.party.active_members().into_iter().map(|m| progression::build_played(&d, m, "w").unwrap().unit).collect();
    let best = |f: &dyn Fn(&rpgkit_rules::Unit) -> i32| built.iter().map(|u| f(u) as i64).max().unwrap();
    assert_eq!(dialogue::best_stat(&d, &gs, "initiative"), best(&|u| u.initiative_rating()));
    assert_eq!(dialogue::best_stat(&d, &gs, "move_rate"), best(&|u| u.move_per_block()));
    assert_eq!(dialogue::best_stat(&d, &gs, "armor"), best(&|u| rpgkit_rules::damage::armor_rating(u)));
    let cons: Vec<i64> = gs.party.active_members().into_iter().map(|m| dialogue::member_stat(&d, &gs, m, "CONS").unwrap()).collect();
    assert_eq!(dialogue::best_stat(&d, &gs, "grit"), cons.iter().map(|c| c * 2 + 1).max().unwrap(), "a derived row: CONS * 2 + 1");
    assert!(dialogue::best_stat(&d, &gs, "initiative") > 0);
    assert_eq!(dialogue::best_stat(&d, &gs, "nonsense"), 0, "an unknown name reads 0");
    // a rating condition is a pure read like a stat one
    let before = gs.clone();
    let bar = dialogue::best_stat(&d, &gs, "grit");
    for (min, want) in [(bar, true), (bar + 1, false)] {
        let cond = json::dict(&json!({"kind": "stat", "stat": "grit", "threshold": min})).clone();
        assert_eq!(dialogue::condition_holds(&d, &gs, &cond), want);
    }
    assert_eq!(gs, before);
}
