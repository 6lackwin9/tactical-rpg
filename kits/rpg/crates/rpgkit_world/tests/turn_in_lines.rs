//! PR7-41 (PT-35, PT-36) on a scratch world (no shipped data): a turn-in says the world's `turned_in` service line (its
//! tokens filled) in the node's place once, and nothing new without one; the picker asks who receives only when a
//! passive or skill reward (or a pickable one) needs a recipient (`quest::needs_recipient`); a recruit arriving on the
//! bench or with unspent points gets a line for each (`dialogue::join_flags`, logged by the screens after its note).

use aegis_core::json::{self, Dict};
use rpgkit_world::dialogue::{self, DialogueRunner};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::{explore, quest, Data, GameState};
use serde_json::{json, Value};

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

/// World `w`: the clerk takes the turn-in of `job_a`; `lines` are the manifest's service lines; loot `l_item` (an
/// item) and `l_pass` (a passive).
fn data(lines: Value) -> Data {
    let w = "w";
    let manifest = json!({"id": w, "start_location": "hub", "start_party": ["hero"], "service_lines": lines});
    let worlds = json!({w: {"manifest": manifest, "own": {
        "locations": with_ids(w, json!([{"id": "hub", "name": "Hub"}])),
        "characters": with_ids(w, json!([{"id": "hero", "name": "Hero", "role": "party"}, {"id": "clerk", "name": "Clerk", "role": "npc"}])),
        "quests": with_ids(w, json!([{"id": "job_a", "name": "Job A", "template": "job", "turn_in": "clerk"}])),
        "loot": with_ids(w, json!([{"id": "l_item", "name": "Item", "reward_kind": "item", "ref": "x"}, {"id": "l_pass", "name": "Pass", "reward_kind": "passive", "ref": "p"}]))
    }}});
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}, {"id": "vet", "name": "Vet", "side": "party", "class_id": "fighter", "level": 1}]
    });
    let game = json!({"quests": [{"id": "job", "objectives": [{"kind": "win_encounter"}], "rewards": {"gold": 1}}]});
    Data::from_values(&catalog, &worlds, &game)
}

fn state(d: &Data) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    let mut gs = GameState::create(d, "w", party);
    explore::enter(d, &mut gs, "hub", explore::SPAWN);
    gs.quests.insert("w:job_a".into(), json!({"state": "ready", "objective": 1}));
    gs
}

/// Turns in `job_a` to the clerk: the runner after the confirm and the confirm's result.
fn turn_in(d: &Data, gs: &mut GameState) -> (DialogueRunner, Dict) {
    let mut r = DialogueRunner::open(d, gs, "clerk");
    let services: Vec<String> = json::aget(&r.view(d, gs), "choices").iter().map(|c| json::s(json::dict(c), "service")).collect();
    let i = services.iter().position(|s| s == "turn_in").expect("a turn-in choice");
    r.choose(d, gs, i as i64);
    let res = r.confirm_turn_in(d, gs, "", &[]);
    assert!(json::b(&res, "ok", false), "{res:?}");
    (r, res)
}

#[test]
fn a_turn_in_says_the_worlds_turned_in_line_once() {
    let d = data(json!({"turned_in": "{npc} nods. \"{quest}: done.\""}));
    let mut gs = state(&d);
    let (mut r, res) = turn_in(&d, &mut gs);
    let want = "Clerk nods. \"Job A: done.\"";
    assert_eq!((json::s(&res, "said"), json::s(&r.view(&d, &gs), "text")), (want.to_string(), want.to_string()), "the line, in the node's place");
    // the conversation moving on shows the node's own text again
    let farewell = json::aget(&r.view(&d, &gs), "choices").iter().position(|c| json::s(json::dict(c), "service") == "farewell");
    if let Some(i) = farewell {
        r.choose(&d, &mut gs, i as i64);
    }
    assert!(r.closed || json::s(&r.view(&d, &gs), "text") != want);
    // no line named: nothing new is said
    let d = data(json!({}));
    let mut gs = state(&d);
    let (r, res) = turn_in(&d, &mut gs);
    assert!(!res.contains_key("said") && r.said.is_empty());
}

#[test]
fn the_recipient_is_asked_only_for_a_passive_or_skill_reward() {
    let d = data(json!({}));
    let gs = state(&d);
    let r = |v: Value| json::dict(&v).clone();
    assert!(!quest::needs_recipient(&d, &gs, &r(json!({"xp": 10, "gold": 5, "items": {"x": 1}})), &[]), "xp, gold and items need nobody");
    assert!(quest::needs_recipient(&d, &gs, &r(json!({"passives": ["p"]})), &[]));
    assert!(quest::needs_recipient(&d, &gs, &r(json!({"skills": ["s"]})), &[]));
    assert!(!quest::needs_recipient(&d, &gs, &Dict::new(), &["w:l_item".into()]), "an item offer needs nobody");
    assert!(quest::needs_recipient(&d, &gs, &Dict::new(), &["w:l_item".into(), "w:l_pass".into()]), "a passive offer does");
}

#[test]
fn a_recruit_on_the_bench_or_with_points_is_flagged() {
    let d = data(json!({}));
    let mut gs = state(&d);
    gs.party.active_ids = vec!["hero".into()];
    gs.party.max_active = 1;
    let mut vet = UnitState::create("vet", "vet");
    vet.total_xp = 0;
    gs.party.members.push(vet);
    let flags = dialogue::join_flags(&d, &gs, "vet");
    assert!(flags.iter().any(|f| f.contains("waits on the bench")), "{flags:?}");
    let points = gs.party.member("vet").unwrap().unspent(&d);
    assert_eq!(flags.iter().any(|f| f.contains("unspent point")), points > 0, "{flags:?} with {points} points");
    // fielded with nothing to spend: no note
    gs.party.active_ids.push("vet".into());
    gs.party.max_active = 2;
    if points == 0 {
        assert!(dialogue::join_flags(&d, &gs, "vet").is_empty());
    }
    // levels bring points: the note names them
    gs.party.member_mut("vet").unwrap().total_xp = 100_000;
    let points = gs.party.member("vet").unwrap().unspent(&d);
    assert!(points > 0);
    assert!(dialogue::join_flags(&d, &gs, "vet").iter().any(|f| f.contains(&format!("{points} unspent point"))));
    // the screens find the member a recruit note names
    assert_eq!(dialogue::join_note_member(&d, &gs, "Vet joins the party.").as_deref(), Some("vet"));
    assert_eq!(dialogue::join_note_member(&d, &gs, "Nobody joins the party."), None);
    assert_eq!(dialogue::join_note_member(&d, &gs, "Vet waits."), None);
}
