//! STORYUI1-W story layer on scratch worlds (STORYUI1.md 4.7, 4.8, 5.2-5.4): the vignette queue (W1, W2, W4), cards
//! resolved at play time (W3, W5), sequences (Q1-Q7) with the story fight runner, the `story` save block (4.8, Q8),
//! the new game's start sequence (5.4), the story effects in conversations and campaign rules, and the campaign's
//! `{vignette}` items joining the queue (CAMPAIGN1 T2). Expectations follow from the rules and the fixture's own data.

use aegis_core::json::{self, Dict};
use rpgkit_data::Step;
use rpgkit_world::dialogue::DialogueRunner;
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::story::{self, Running, RETRY_TEXT, RUNNING_TEXT};
use rpgkit_world::{campaign, new_game, save, Data, GameState};
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

fn text_card(text: &str) -> Value {
    json!({"style": "text", "text": text})
}

/// World `tale` (a story: vignettes, sequences, a start sequence, a talker whose choices start a sequence or queue a
/// vignette; a campaign so its queue can hand items on) and world `classic` (none of it).
fn data() -> Data {
    let vignettes = json!([
        {"id": "v_once", "name": "Once", "once": true, "cards": [text_card("Once.")]},
        {"id": "v_plain", "name": "Plain", "cards": [text_card("Plain.")]},
        {"id": "v_other", "name": "Other", "cards": [text_card("Other.")]},
        {"id": "v_tokens", "name": "Tokens", "cards": [
            text_card("{flag:days} days. {member:stayed} stayed. {member:who} came. [{flag:unset}] {other}"),
            {"style": "text", "text": "Hidden.", "when": [{"key": "gate", "value": true}]},
            {"style": "line", "speaker": "clerk", "text": "Hi."}
        ]},
        {"id": "v_hidden", "name": "Hidden", "cards": [{"style": "text", "text": "Never.", "when": [{"key": "gate", "value": true}]}]}
    ]);
    let sequences = json!([
        {"id": "prologue", "name": "Prologue", "steps": [
            {"kind": "effects", "effects": [{"effect": "set_flag", "key": "story_began"}, {"effect": "vignette", "vignette": "v_other"}]},
            {"kind": "vignette", "vignette": "v_plain"},
            {"kind": "fight", "encounter": "fight_a", "on_lost": "retry"},
            {"kind": "effects", "effects": [{"effect": "recruit", "character": "ally"}]},
            {"kind": "vignette", "vignette": "v_once"},
            {"kind": "enter", "location": "hub", "anchor": ""}
        ]},
        {"id": "abort_seq", "name": "Abort", "steps": [{"kind": "fight", "encounter": "fight_a"}, {"kind": "end"}]},
        {"id": "continue_seq", "name": "Continue", "steps": [{"kind": "fight", "encounter": "fight_a", "on_lost": "continue"}, {"kind": "end", "to": "menu"}]},
        {"id": "side_story", "name": "Side", "steps": [{"kind": "vignette", "vignette": "v_plain"}]},
        {"id": "no_end", "name": "No end", "steps": [{"kind": "vignette", "vignette": "v_plain"}]}
    ]);
    let dialogues = json!([{"id": "talk", "start": "a", "services": false, "nodes": [
        {"id": "a", "text": "Well?", "choices": [
            {"text": "Tell me a story.", "effects": [{"effect": "sequence", "sequence": "side_story"}]},
            {"text": "Show me.", "to": "a", "effects": [{"effect": "vignette", "vignette": "v_plain"}]}
        ]}
    ]}]);
    let worlds = json!({
        "tale": {"manifest": {"id": "tale", "start_location": "hub", "start_party": ["hero"], "start_sequence": "prologue", "campaign": {"start_act": 1}}, "own": {
            "locations": with_ids("tale", json!([{"id": "hub", "name": "Hub"}, {"id": "alley", "name": "Alley"}])),
            "characters": with_ids("tale", json!([
                {"id": "ally", "name": "Ally", "role": "party", "base_unit": "hero"},
                {"id": "clerk", "name": "Clerk", "role": "npc", "dialogue": "talk"}
            ])),
            "dialogues": with_ids("tale", dialogues),
            "encounters": with_ids("tale", json!([{"id": "fight_a", "name": "A", "enemies": []}])),
            "vignettes": with_ids("tale", vignettes),
            "sequences": with_ids("tale", sequences)
        }},
        "classic": {"manifest": {"id": "classic", "start_location": "camp", "start_party": ["hero"]}, "own": {
            "locations": with_ids("classic", json!([{"id": "camp", "name": "Camp"}]))
        }}
    });
    let catalog = json!({
        "classes": [{"id": "fighter", "name": "Fighter"}],
        "units": [{"id": "hero", "name": "Hero", "side": "party", "class_id": "fighter", "level": 1}]
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

/// A state in world `w` with the hero and no sequence running.
fn state(d: &Data, w: &str) -> GameState {
    let mut party = PartyState::default();
    party.members.push(UnitState::create("hero", "hero"));
    GameState::create(d, w, party)
}

fn notes(res: &Dict) -> Vec<String> {
    json::sget(res, "notes")
}

fn step_index(gs: &GameState) -> i64 {
    gs.story.running.as_ref().map(|r| r.step).unwrap_or(-1)
}

// ---- W1-W4 ----

#[test]
fn a_once_vignette_seen_is_not_queued_and_none_is_queued_twice() {
    let d = data();
    let mut gs = state(&d, "tale");
    assert!(story::queue(&d, &mut gs, "v_plain"));
    assert!(!story::queue(&d, &mut gs, "v_plain"), "W1: a queued vignette is not queued twice");
    assert!(story::queue(&d, &mut gs, "v_once"));
    assert!(!story::queue(&d, &mut gs, "nope"), "an unknown ref queues nothing");
    assert_eq!(gs.story.queue, ["tale:v_plain", "tale:v_once"]);
    // W2: the oldest plays first; when done it leaves the queue and joins seen
    let sc = story::next_scene(&d, &mut gs).expect("a scene");
    assert_eq!(json::s(&sc, "vignette"), "tale:v_plain");
    assert!(!json::b(&sc, "seen", true), "W6: not seen yet, so Skip asks first");
    story::vignette_done(&mut gs, "tale:v_plain");
    story::vignette_done(&mut gs, "tale:v_once");
    assert!(gs.story.queue.is_empty());
    assert_eq!(gs.story.seen, ["tale:v_plain", "tale:v_once"]);
    // W1: `once` and seen: not queued again; a vignette without `once` is
    assert!(!story::queue(&d, &mut gs, "v_once"));
    assert!(story::queue(&d, &mut gs, "v_plain"));
    assert!(json::b(&story::next_scene(&d, &mut gs).unwrap(), "seen", false), "W6: seen before, Skip does not ask");
}

#[test]
fn cards_resolve_when_they_play_and_an_all_hidden_vignette_is_marked_seen() {
    let d = data();
    let mut gs = state(&d, "tale");
    gs.set_flag("days", json!(42));
    gs.set_flag("stayed", json!("ally"));
    gs.set_flag("who", json!("hero"));
    let sc = story::scene(&d, &gs, "v_tokens");
    let cards = json::aget(&sc, "cards");
    assert_eq!(cards.len(), 2, "W3: the card whose `when` fails is left out");
    let first = json::dict(&cards[0]);
    // {flag:..} the value as text ("" unset); {member:..} a world character's name, a member's name; other braces stay
    assert_eq!(json::s(first, "text"), "42 days. Ally stayed. Hero came. [] {other}");
    let line = json::dict(&cards[1]);
    assert_eq!((json::s(line, "speaker"), json::s(line, "speaker_name")), ("tale:clerk".to_string(), "Clerk".to_string()), "W5");
    assert!(!line.contains_key("when"));
    // the same vignette later, the state changed: the card shows and the tokens follow
    gs.set_flag("gate", json!(true));
    gs.set_flag("days", json!(41));
    let cards = json::aget(&story::scene(&d, &gs, "v_tokens"), "cards").to_vec();
    assert_eq!(cards.len(), 3);
    assert!(json::s(json::dict(&cards[0]), "text").starts_with("41 days."));
    // W4: every card left out: marked seen, leaves the queue, the next entry plays
    gs.set_flag("gate", json!(false));
    story::queue(&d, &mut gs, "v_hidden");
    story::queue(&d, &mut gs, "v_plain");
    let sc = story::next_scene(&d, &mut gs).expect("the next one");
    assert_eq!(json::s(&sc, "vignette"), "tale:v_plain");
    assert_eq!(gs.story.seen, ["tale:v_hidden"]);
    assert_eq!(gs.story.queue, ["tale:v_plain"]);
}

#[test]
fn campaign_vignette_items_join_the_story_queue_in_order() {
    let d = data();
    let mut gs = state(&d, "tale");
    let items = [json!({"vignette": "v_plain"}), json!({"vignette": "v_other"}), json!({"battle": "fight_a"}), json!({"vignette": "v_once"})];
    gs.campaign.as_mut().expect("a campaign").pending = items.iter().map(|v| json::dict(v).clone()).collect();
    let sc = story::next_scene(&d, &mut gs).expect("a scene");
    assert_eq!(json::s(&sc, "vignette"), "tale:v_plain");
    assert_eq!(gs.story.queue, ["tale:v_plain", "tale:v_other"], "the head's vignette items move, up to the battle");
    let head = campaign::pending_head(&gs).expect("the battle stays");
    assert_eq!(json::s(head, "battle"), "fight_a");
    // a campaign rule's story effects go through the shared effect body
    let mut events = Vec::new();
    let fx = [json::dict(&json!({"effect": "vignette", "vignette": "v_once"})).clone(), json::dict(&json!({"effect": "sequence", "sequence": "side_story"})).clone()];
    campaign::apply_effects(&d, &mut gs, &fx, &mut events);
    assert!(gs.story.queue.contains(&"tale:v_once".to_string()));
    assert_eq!(gs.story.running, Some(Running { sequence: "tale:side_story".into(), step: 0 }));
}

// ---- Q1-Q7 ----

#[test]
fn start_runs_the_leading_effects_and_refuses_a_second_sequence() {
    let d = data();
    let mut gs = state(&d, "tale");
    let res = story::start(&d, &mut gs, "prologue");
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert!(gs.flag_true("story_began"), "Q3: the leading effects step ran");
    assert_eq!(gs.story.queue, ["tale:v_other"], "Q9: a vignette effect queues as usual");
    assert_eq!(step_index(&gs), 1);
    assert_eq!(story::current(&d, &gs), Some(Step::Vignette { vignette: "tale:v_plain".into() }), "Q2: never an effects step");
    let again = story::start(&d, &mut gs, "side_story");
    assert_eq!(json::sget(&again, "errors"), [RUNNING_TEXT]);
    assert_eq!(step_index(&gs), 1, "the running sequence is untouched");
    let mut fresh = state(&d, "tale");
    assert_eq!(json::sget(&story::start(&d, &mut fresh, "ghost"), "errors"), ["Unknown sequence 'ghost'."]);
    assert!(fresh.story.running.is_none());
}

#[test]
fn a_story_fight_moves_on_when_won_and_retries_with_full_hp() {
    let d = data();
    let mut gs = state(&d, "tale");
    story::start(&d, &mut gs, "prologue");
    assert!(story::fight(&d, &gs).is_none(), "no fight at a vignette step");
    let res = story::step_done(&d, &mut gs);
    assert!(json::b(&res, "ok", false));
    assert!(gs.story.seen.contains(&"tale:v_plain".to_string()), "a sequence's vignette joins seen");
    let fight = story::fight(&d, &gs).expect("Q5: the fight step");
    assert_eq!((json::s(&fight, "source"), json::s(&fight, "encounter")), ("story".to_string(), "tale:fight_a".to_string()));
    // lost with retry: everyone back to full HP, the same step, "Try again."
    gs.party.members[0].hp_lost = 5;
    let res = story::after_fight(&d, &mut gs, &fight, "lost", None);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(notes(&res), [RETRY_TEXT]);
    assert_eq!(gs.party.members[0].hp_lost, 0);
    assert_eq!(story::fight(&d, &gs), Some(fight.clone()), "the step is kept");
    // won: the next step; the effects step after it runs at once (recruit), then the next vignette
    let res = story::after_fight(&d, &mut gs, &fight, "won", None);
    assert_eq!(notes(&res), ["Ally joins the party."]);
    assert_eq!(gs.party.members.len(), 2);
    assert_eq!(story::current(&d, &gs), Some(Step::Vignette { vignette: "tale:v_once".into() }));
    // Q6: enter places the party at the location and clears running
    story::step_done(&d, &mut gs);
    assert_eq!(story::current(&d, &gs), Some(Step::Enter { location: "tale:hub".into(), anchor: String::new() }));
    let res = story::finish(&d, &mut gs);
    assert!(json::b(&res, "ok", false), "{res:?}");
    assert_eq!(json::s(&res, "to"), "map");
    assert_eq!(gs.location_id, "tale:hub");
    assert!(gs.story.running.is_none());
}

#[test]
fn lost_story_fights_abort_or_continue_as_the_step_says() {
    let d = data();
    // abort (the default): running cleared, the party wakes (the loss path of a dialogue fight)
    let mut gs = state(&d, "tale");
    story::start(&d, &mut gs, "abort_seq");
    let fight = story::fight(&d, &gs).unwrap();
    let res = story::after_fight(&d, &mut gs, &fight, "lost", None);
    assert!(gs.story.running.is_none());
    assert_eq!(gs.location_id, "tale:hub", "woke at the start location");
    assert!(json::aget(&res, "events").iter().any(|e| json::s(json::dict(e), "kind") == "defeat"));
    // fled with the default abort: running cleared, no wake
    let mut gs = state(&d, "tale");
    story::start(&d, &mut gs, "abort_seq");
    story::after_fight(&d, &mut gs, &fight, "fled", None);
    assert!(gs.story.running.is_none() && gs.location_id.is_empty());
    // continue: the next step; Q7 end menu clears running
    let mut gs = state(&d, "tale");
    story::start(&d, &mut gs, "continue_seq");
    let fight = story::fight(&d, &gs).unwrap();
    story::after_fight(&d, &mut gs, &fight, "lost", None);
    assert_eq!(story::current(&d, &gs), Some(Step::End { to: "menu".into() }));
    let res = story::finish(&d, &mut gs);
    assert_eq!(json::s(&res, "to"), "menu");
    assert!(gs.story.running.is_none());
    // 4.3: a sequence without enter or end ends as end map
    let mut gs = state(&d, "tale");
    story::start(&d, &mut gs, "no_end");
    story::step_done(&d, &mut gs);
    assert_eq!(story::current(&d, &gs), Some(Step::End { to: "map".into() }));
    assert_eq!(json::s(&story::finish(&d, &mut gs), "to"), "map");
    assert!(json::sget(&story::finish(&d, &mut gs), "errors").len() == 1, "nothing left to finish");
}

// ---- 4.7 in conversations ----

#[test]
fn a_sequence_choice_closes_the_conversation_and_a_vignette_choice_goes_on() {
    let d = data();
    let mut gs = state(&d, "tale");
    let mut talk = DialogueRunner::open(&d, &mut gs, "clerk");
    let res = talk.choose(&d, &mut gs, 1);
    assert!(json::b(&res, "ok", false));
    assert!(!talk.closed, "the conversation goes on");
    assert_eq!(gs.story.queue, ["tale:v_plain"]);
    let res = talk.choose(&d, &mut gs, 0);
    assert!(json::b(&res, "ok", false) && talk.closed && json::b(&res, "end", false));
    assert_eq!(json::s(&res, "sequence"), "tale:side_story");
    assert_eq!(gs.story.running, Some(Running { sequence: "tale:side_story".into(), step: 0 }));
    // refused while one runs
    let mut talk = DialogueRunner::open(&d, &mut gs, "clerk");
    let res = talk.choose(&d, &mut gs, 0);
    assert_eq!(json::sget(&res, "errors"), [RUNNING_TEXT]);
    assert!(!talk.closed);
}

// ---- 4.8, Q8 ----

fn envelope(version: i64, game_state: &Value) -> String {
    json!({"format": save::FORMAT, "version": version, "saved_at": 1, "game_state": game_state}).to_string()
}

#[test]
fn the_story_block_saves_only_when_used_and_round_trips() {
    let d = data();
    let gs = state(&d, "tale");
    assert!(!gs.to_value().as_object().unwrap().contains_key("story"), "an empty story writes no key");
    let classic = state(&d, "classic").to_value();
    assert!(!classic.as_object().unwrap().contains_key("story"));
    let mut gs = state(&d, "tale");
    story::start(&d, &mut gs, "prologue");
    let v = gs.to_value();
    assert_eq!(v["story"], json!({"running": {"sequence": "tale:prologue", "step": 1}, "queue": ["tale:v_other"]}), "each key only when not empty");
    let loaded = save::decode(&d, &envelope(save::VERSION, &v)).expect("loads");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.story, gs.story);
    // a save of the version before the story block reads an empty one
    let mut old = state(&d, "tale").to_value();
    old.as_object_mut().unwrap().shift_remove("story");
    let loaded = save::decode(&d, &envelope(save::VERSION - 1, &old)).expect("an older save loads");
    assert!(loaded.state.story.is_empty() && loaded.warnings.is_empty());
}

#[test]
fn a_running_the_data_lacks_is_dropped_and_a_damaged_block_is_refused() {
    let d = data();
    let mut gs = state(&d, "tale");
    story::start(&d, &mut gs, "side_story");
    for (running, warning) in [
        (json!({"sequence": "tale:ghost", "step": 0}), "dropped story.running: unknown sequence 'tale:ghost'"),
        (json!({"sequence": "tale:side_story", "step": 5}), "dropped story.running: step 5 is out of range for 'tale:side_story'"),
    ] {
        let mut v = gs.to_value();
        v["story"]["running"] = running;
        let loaded = save::decode(&d, &envelope(save::VERSION, &v)).expect("loads");
        assert_eq!(loaded.warnings, [warning]);
        assert!(loaded.state.story.running.is_none());
    }
    let damaged: Vec<(Value, &str)> = vec![
        (json!([1]), "story must be an object {running, queue, seen}."),
        (json!({"running": {"sequence": "tale:side_story"}}), "story.running must be {sequence, step}."),
        (json!({"running": {"sequence": "tale:side_story", "step": "1"}}), "story.running must be {sequence, step}."),
        (json!({"queue": "tale:v_plain"}), "story.queue must be a list of text."),
        (json!({"seen": [1]}), "story.seen must be a list of text."),
        (json!({"over": true}), "story: unknown key 'over'."),
        (json!({"queue": ["v_plain"]}), "story.queue: 'v_plain' must be a qualified id world:id."),
        (json!({"seen": ["tale:v_plain", "tale:v_plain"]}), "story.seen lists 'tale:v_plain' twice."),
    ];
    for (block, want) in damaged {
        let mut v = gs.to_value();
        v["story"] = block.clone();
        let errors = save::decode(&d, &envelope(save::VERSION, &v)).err().unwrap_or_else(|| panic!("{block} refused"));
        assert_eq!(errors, [want], "{block}");
    }
}

// ---- 5.4 ----

#[test]
fn a_new_game_runs_the_world_start_sequence_and_classic_worlds_start_on_the_map() {
    let d = data();
    let gs = new_game::create_for_world(&d, "tale", 7).expect("a new game");
    assert_eq!(gs.story.running, Some(Running { sequence: "tale:prologue".into(), step: 1 }), "the leading effects ran");
    assert!(gs.flag_true("story_began"));
    let gs = new_game::create_for_world(&d, "classic", 7).expect("a new game");
    assert!(gs.story.is_empty());
    assert_eq!(story::start_sequence(&d, "classic"), "");
}
