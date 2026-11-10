//! STORYMUSIC1 (Art/UI request 3) on a scratch world: `story::step_music`, the `music` a running sequence step asks
//! for: its own, else the last one set by the `effects` steps that ran at once just before it; "" keeps the music
//! playing. `rpgkit_data::music_ask` reads a value (keep, silence, a track, a music key). Expectations follow from
//! the fixture's own steps.

use aegis_core::json;
use rpgkit_data::{music_ask, MusicAsk};
use rpgkit_world::state::{PartyState, UnitState};
use rpgkit_world::story;
use rpgkit_world::{Data, GameState};
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

/// World `tale`: sequence `scored` whose steps set music in each way (a key on a vignette step, none on the next, two
/// effects steps before a fight, the last of them with a key, silence on the last vignette).
fn data() -> Data {
    let card = json!([{"style": "text", "text": "A card."}]);
    let vignettes = json!([{"id": "v_a", "name": "A", "cards": card}, {"id": "v_b", "name": "B", "cards": card}]);
    let flag = |k: &str| json!([{"effect": "set_flag", "key": k}]);
    let sequences = json!([{"id": "scored", "name": "Scored", "steps": [
        {"kind": "vignette", "vignette": "v_a", "music": "theme_open"},
        {"kind": "vignette", "vignette": "v_b"},
        {"kind": "effects", "effects": flag("one"), "music": "theme_tense"},
        {"kind": "effects", "effects": flag("two"), "music": "theme_run"},
        {"kind": "effects", "effects": flag("three")},
        {"kind": "fight", "encounter": "fight_a"},
        {"kind": "vignette", "vignette": "v_a", "music": "none"},
        {"kind": "end"}
    ]}]);
    let worlds = json!({
        "tale": {"manifest": {"id": "tale", "start_location": "hub", "start_party": ["hero"]}, "own": {
            "locations": with_ids("tale", json!([{"id": "hub", "name": "Hub"}])),
            "encounters": with_ids("tale", json!([{"id": "fight_a", "name": "A", "enemies": []}])),
            "vignettes": with_ids("tale", vignettes),
            "sequences": with_ids("tale", sequences)
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

#[test]
fn a_value_reads_as_keep_silence_a_track_or_a_key() {
    assert_eq!(music_ask(""), MusicAsk::Keep);
    assert_eq!(music_ask(rpgkit_data::MUSIC_NONE), MusicAsk::Silence);
    assert_eq!(music_ask("music/town.ogg"), MusicAsk::Track("music/town.ogg"));
    assert_eq!(music_ask("theme_open"), MusicAsk::Key("theme_open"));
    assert_eq!(music_ask("Not A Key"), MusicAsk::Key("Not A Key"), "the check refuses its shape");
    for kind in rpgkit_data::STEP_KINDS {
        assert!(rpgkit_data::step_keys(kind).contains(&"music"), "every step kind takes music: {kind}");
    }
}

#[test]
fn a_step_asks_for_its_own_music_or_the_effects_steps_before_it() {
    let d = data();
    let mut gs = state(&d);
    assert_eq!(story::step_music(&d, &gs), "", "no sequence runs");
    assert!(json::b(&story::start(&d, &mut gs, "scored"), "ok", false));
    assert_eq!(story::step_music(&d, &gs), "theme_open", "the first step's own music");
    story::step_done(&d, &mut gs);
    assert_eq!(story::step_music(&d, &gs), "", "a step without music keeps the music playing");
    story::step_done(&d, &mut gs);
    // the three effects steps ran at once; the fight step has none of its own: the nearest effects step that set one
    // (the fixture's second, over its first)
    let fight = story::fight(&d, &gs).expect("at the fight step");
    assert!(gs.flag_true("three"));
    assert_eq!(story::step_music(&d, &gs), "theme_run");
    story::after_fight(&d, &mut gs, &fight, "won", None);
    assert_eq!(story::step_music(&d, &gs), rpgkit_data::MUSIC_NONE, "the vignette step after the fight asks for silence");
    story::step_done(&d, &mut gs);
    assert_eq!(story::step_music(&d, &gs), "", "the end step sets none (and the effects steps are not just before it)");
}
