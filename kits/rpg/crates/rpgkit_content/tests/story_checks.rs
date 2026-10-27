//! The story schema slice (STORYUI1-S, docs/specs/STORYUI1.md 4.1-4.7): the world kinds `vignettes`, `sequences` and
//! `boards`; the quest board fields; the manifest's `start_sequence`; the story effects `vignette`, `sequence` and
//! `open_board` and the service line `board`; the asset warnings; the writer for the new fields and kinds. A story
//! world written to the spec's section 4 shape (neutral scratch names, spec section 3) checks clean, and each field set
//! wrong adds exactly one error (TS1; the classic worlds' reports are the game façade's test). Scratch data only.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::Step;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const W: &str = "ward";

fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_story_checks_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds").join(W)).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// Writes a world kind file as a fixture (the files are the test's own input; the writer is tested below).
fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(store::file_of(data, Some(W), kind), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

fn manifest() -> Value {
    json!({"id": W, "name": "Ward", "extends": [], "connects": [], "start_location": "hub", "start_sequence": "seq_a", "version": 1})
}

/// A vignette: one card of each style (4.2), tokens, a condition, timings.
fn reel_a() -> Value {
    json!({
        "id": "reel_a", "name": "Reel A", "music": "music/theme_1.ogg", "frame": "#0a0b10", "skippable": true, "once": false,
        "cards": [
            {"style": "image", "image": "backgrounds/story/card_1.png", "top": "#0a0b10", "bottom": "#2c1546", "ash": true, "text": "First card.", "fade_in_ms": 1200, "hold_ms": 3500, "fade_out_ms": 600},
            {"style": "text", "top": "#1e232d", "text": "{member:picked} stayed on day {flag:campaign.day}.", "when": [{"kind": "flag", "key": "picked", "op": "!=", "value": ""}]},
            {"style": "title", "title": "Test Title", "subtitle": "Test Subtitle", "hold_ms": 3000},
            {"style": "line", "speaker": "keeper_a", "text": "A line.", "cue": "sfx/cue_1.wav"}
        ]
    })
}

fn seq_a() -> Value {
    json!({"id": "seq_a", "name": "Sequence A", "steps": [
        {"kind": "vignette", "vignette": "reel_a"},
        {"kind": "fight", "encounter": "raid", "on_lost": "retry"},
        {"kind": "effects", "effects": [{"effect": "set_flag", "key": "act", "value": 1}, {"effect": "vignette", "vignette": "reel_a"}, {"effect": "start_quest", "quest": "q_test"}]},
        {"kind": "enter", "location": "hub", "anchor": ""}
    ]})
}

fn seq_b() -> Value {
    json!({"id": "seq_b", "name": "Sequence B", "steps": [
        {"kind": "effects", "effects": [{"effect": "set_flag", "key": "flag_b", "value": true}]},
        {"kind": "vignette", "vignette": "reel_a"},
        {"kind": "end", "to": "menu"}
    ]})
}

fn board() -> Value {
    json!({"id": "board_a", "name": "Test Board", "keeper": "keeper_a", "intro": "", "requires_flags": [{"key": "act", "op": ">=", "value": 1}]})
}

fn quest_a() -> Value {
    json!({"id": "q_test", "name": "Test Quest", "giver": "keeper_a", "template": "reach", "board": "board_a", "track": "main", "show_locked": true, "closed_text": "Closed for now."})
}

/// A dialogue whose choices use the story effects (4.7).
fn talk_a() -> Value {
    json!({"id": "talk_a", "name": "Keeper A", "start": "start", "services": false, "nodes": [{"id": "start", "text": "Well?", "choices": [
        {"text": "Show me the board.", "to": "", "effects": [{"effect": "set_flag", "key": "asked", "value": true}, {"effect": "open_board", "board": "board_a"}]},
        {"text": "Tell me a story.", "to": "", "effects": [{"effect": "vignette", "vignette": "reel_a"}]},
        {"text": "Let's go.", "to": "", "effects": [{"effect": "sequence", "sequence": "seq_b"}]}
    ]}]})
}

/// A scratch story world: the manifest, a hub with a map, two characters, an encounter, a quest, the dialogue, the
/// vignette, two sequences and the board.
fn story_world(tag: &str) -> PathBuf {
    let data = scratch(tag);
    fs::write(aegis_content::world::manifest_path(&data, W), manifest().to_string()).unwrap();
    fs::create_dir_all(data.join("maps")).unwrap();
    let rows = ["#####", "#...#", "#...#", "#####"];
    let hub_map = json!({"id": "hub_map", "name": "Hub", "rows": rows, "zones": [], "zone_legend": {}, "anchors": [{"id": "spawn", "x": 1, "y": 1}, {"id": "anchor_b", "x": 2, "y": 2}], "paths": {}, "tags": []});
    fs::write(store::file_of(&data, None, "maps"), json!({"kind": "map_templates", "version": 1, "entries": [hub_map]}).to_string()).unwrap();
    write_kind(&data, "locations", json!([{"id": "hub", "name": "Hub", "map": "hub_map"}, {"id": "street", "name": "Street"}]));
    write_kind(&data, "characters", json!([{"id": "keeper_a", "name": "Keeper A"}, {"id": "keeper_b", "name": "Keeper B"}]));
    write_kind(&data, "encounters", json!([{"id": "raid", "name": "Raid"}]));
    write_kind(&data, "quests", json!([quest_a()]));
    write_kind(&data, "dialogues", json!([talk_a()]));
    write_kind(&data, "vignettes", json!([reel_a()]));
    write_kind(&data, "sequences", json!([seq_a(), seq_b()]));
    write_kind(&data, "boards", json!([board()]));
    data
}

fn errors_of(v: &check::View, kind: &str, entry: &Value) -> Vec<String> {
    check::world::validate_entry(v, W, kind, entry.as_object().unwrap(), &json::s(entry.as_object().unwrap(), "id"))
}

/// The errors `bad` has and `good` has not.
fn added(v: &check::View, kind: &str, good: &Value, bad: &Value) -> Vec<String> {
    let before = errors_of(v, kind, good);
    errors_of(v, kind, bad).into_iter().filter(|e| !before.contains(e)).collect()
}

/// `entry` with `path` (keys and list indexes) set to `value`.
fn with(entry: &Value, path: &[&str], value: Value) -> Value {
    let mut out = entry.clone();
    let mut at = &mut out;
    for (i, p) in path.iter().enumerate() {
        let last = i + 1 == path.len();
        at = match p.parse::<usize>() {
            Ok(n) => &mut at[n],
            Err(_) if last => {
                at[*p] = value;
                return out;
            }
            Err(_) => &mut at[*p],
        };
    }
    *at = value;
    out
}

/// `entry` without `path`'s last key.
fn without(entry: &Value, path: &[&str]) -> Value {
    let mut out = entry.clone();
    let (last, head) = path.split_last().unwrap();
    let mut at = &mut out;
    for p in head {
        at = match p.parse::<usize>() {
            Ok(n) => &mut at[n],
            Err(_) => &mut at[*p],
        };
    }
    at.as_object_mut().unwrap().shift_remove(*last);
    out
}

fn assert_one(v: &check::View, kind: &str, good: &Value, bad: &Value, needle: &str, tag: &str) {
    let found = added(v, kind, good, bad);
    assert_eq!(found.len(), 1, "{tag}: {found:?}");
    assert!(found[0].contains(needle), "{tag}: {found:?} (wanted '{needle}')");
}

fn field_keys(kind: &str) -> Vec<String> {
    fields(source::schema(), "world", kind).iter().map(|f| json::s(json::dict(f), "key")).collect()
}

fn field(kind: &str, key: &str) -> Dict {
    fields(source::schema(), "world", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("world.{kind}.{key}"))
}

#[test]
fn the_schema_carries_the_story_kinds_and_fields() {
    for kind in rpgkit_data::STORY_KINDS {
        let keys = field_keys(kind);
        assert_eq!(keys.first().map(String::as_str), Some("id"), "{kind}");
        assert!(keys.ends_with(&["override".to_string(), "tags".to_string()]), "{kind}: {keys:?}");
        assert!(source::WORLD_KINDS.contains(&kind) && check::world::KINDS.contains(&kind), "{kind} is a loaded and checked world kind");
    }
    // 4.1, 4.3, 4.4 field tables, in order
    assert_eq!(field_keys("vignettes"), ["id", "name", "music", "frame", "skippable", "once", "cards", "override", "tags"]);
    assert_eq!(field_keys("sequences"), ["id", "name", "steps", "override", "tags"]);
    assert_eq!(field_keys("boards"), ["id", "name", "keeper", "intro", "requires_flags", "override", "tags"]);
    let defaults = |kind: &str| -> Vec<Value> { field_keys(kind).iter().map(|k| json::get(&field(kind, k), "default").clone()).collect() };
    assert_eq!(defaults("vignettes")[1..7], [json!("New Vignette"), json!(""), json!(rpgkit_data::FRAME_COLOUR), json!(true), json!(false), json!([])]);
    assert_eq!(json::s(&field("boards", "keeper"), "ref"), "characters");
    // 4.5: the quest fields come after tags; 4.6: the manifest field
    assert!(field_keys("quests").ends_with(&["tags", "board", "track", "show_locked", "closed_text"].map(String::from)));
    assert_eq!(json::sget(&field("quests", "track"), "options"), rpgkit_data::TRACKS.map(String::from));
    assert_eq!(json::get(&field("quests", "track"), "default"), &json!("side"));
    assert_eq!(json::s(&field("quests", "board"), "ref"), "boards");
    let start = field("world", "start_sequence");
    assert_eq!((json::s(&start, "ref"), json::get(&start, "default")), ("sequences".to_string(), &json!("")));
    // the loader fills the defaults
    let data = story_world("schema");
    let src = source::load(&data);
    let own = &src.worlds[W]["own"];
    assert_eq!(own["vignettes"][0]["cards"].as_array().map(Vec::len), Some(4));
    assert_eq!(own["sequences"].as_array().map(Vec::len), Some(2));
    assert_eq!(own["boards"][0]["keeper"], json!("keeper_a"));
    assert_eq!(own["encounters"][0].get("board"), None, "board fields are quest fields only");
    write_kind(&data, "vignettes", json!([{"id": "bare", "name": "Bare"}]));
    let bare = &source::load(&data).worlds[W]["own"]["vignettes"][0];
    assert_eq!((bare["frame"].clone(), bare["skippable"].clone(), bare["once"].clone(), bare["music"].clone()), (json!("#0a0b10"), json!(true), json!(false), json!("")));
    drop_scratch(&data);
}

#[test]
fn the_section_4_story_world_checks_clean() {
    let data = story_world("clean");
    let v = store::view(&data);
    for kind in rpgkit_data::STORY_KINDS.into_iter().chain(["dialogues"]) {
        for e in v.worlds.own_entries(W, kind) {
            let found = check::world::validate_entry(&v, W, kind, e, &json::s(e, "id"));
            assert!(found.is_empty(), "{kind}:{}: {found:?}", json::s(e, "id"));
            assert!(check::world::blocking_errors(&v, W, kind, e, &json::s(e, "id")).is_empty(), "{kind} blocking");
        }
    }
    let q = &v.worlds.own_entries(W, "quests")[0];
    assert_eq!(check::board::quest_board_errors(&v, W, q), Vec::<String>::new());
    assert_eq!(check::story::manifest_errors(&v, W, v.worlds.manifest(W)), Vec::<String>::new());
    drop_scratch(&data);
}

#[test]
fn each_bad_vignette_or_card_field_is_one_error() {
    let data = story_world("cards");
    let v = store::view(&data);
    let g = reel_a();
    let many: Vec<Value> = (0..25).map(|_| json!({"style": "text", "text": "x"})).collect();
    let long = |n: usize| "a".repeat(n);
    let cases: Vec<(&str, Value, &str)> = vec![
        ("music not a track", with(&g, &["music"], json!("music/theme_1.mp3")), "must be empty (the music goes on), none (silence) or a track ending .ogg"),
        ("frame not a colour", with(&g, &["frame"], json!("black")), "Frame colour 'black' must be #rrggbb."),
        ("no cards", with(&g, &["cards"], json!([])), "A vignette holds 1-24 cards (has 0)."),
        ("25 cards", with(&g, &["cards"], json!(many)), "A vignette holds 1-24 cards (has 25)."),
        ("cards an object", with(&g, &["cards"], json!({})), "Cards must be a list."),
        ("a card not an object", with(&g, &["cards", "0"], json!("card")), "Card 1 must be an object"),
        ("an unknown card key", with(&g, &["cards", "0", "zoom"], json!(1)), "Card 1: unknown key 'zoom'."),
        ("an unknown style", with(&g, &["cards", "0", "style"], json!("comic")), "style 'comic' is not one of image, text, title, line."),
        ("an image on a text card", with(&g, &["cards", "1", "image"], json!("backgrounds/x.png")), "Card 2: image is for an image card only."),
        ("a top not a colour", with(&g, &["cards", "0", "top"], json!("#12")), "Card 1: top '#12' must be #rrggbb."),
        ("a bottom not a colour", with(&g, &["cards", "0", "bottom"], json!("red")), "Card 1: bottom 'red' must be #rrggbb."),
        ("ash not a bool", with(&g, &["cards", "0", "ash"], json!("yes")), "ash must be true or false."),
        ("a title card without a title", with(&g, &["cards", "2", "title"], json!("")), "Card 3: a title card needs a title."),
        ("a title over 48", with(&g, &["cards", "2", "title"], json!(long(49))), "title is longer than 48 characters."),
        ("a subtitle over 80", with(&g, &["cards", "2", "subtitle"], json!(long(81))), "subtitle is longer than 80 characters."),
        ("a text over 360", with(&g, &["cards", "0", "text"], json!(long(361))), "text is longer than 360 characters."),
        ("a text card without text", without(&g, &["cards", "1", "text"]), "Card 2: a text card needs text."),
        ("a line card without text", with(&g, &["cards", "3", "text"], json!(" ")), "Card 4: a line card needs text."),
        ("a line card without a speaker", without(&g, &["cards", "3", "speaker"]), "Card 4: a line card needs a speaker."),
        ("a speaker on a title card", with(&g, &["cards", "2", "speaker"], json!("keeper_a")), "Card 3: speaker is for a line card only."),
        ("an unknown speaker", with(&g, &["cards", "3", "speaker"], json!("ghost")), "unknown character 'ghost'"),
        ("an unknown token", with(&g, &["cards", "1", "text"], json!("Hi {npc}.")), "unknown token '{npc}'"),
        ("a token kind unknown", with(&g, &["cards", "1", "text"], json!("Hi {quest:q_test}.")), "unknown token '{quest:q_test}'"),
        ("a token key not a flag key", with(&g, &["cards", "1", "text"], json!("Hi {flag:Bad Key}.")), "unknown token '{flag:Bad Key}'"),
        ("a token on a reserved key", with(&g, &["cards", "1", "text"], json!("Day {flag:campaign.week}.")), "token '{flag:campaign.week}': 'campaign.week' is not a read key"),
        ("fade in over 3000", with(&g, &["cards", "0", "fade_in_ms"], json!(3001)), "fade_in_ms must be a whole number 0..3000."),
        ("fade out negative", with(&g, &["cards", "0", "fade_out_ms"], json!(-1)), "fade_out_ms must be a whole number 0..3000."),
        ("hold over 20000", with(&g, &["cards", "0", "hold_ms"], json!(20001)), "hold_ms must be a whole number 0..20000."),
        ("a hold not a number", with(&g, &["cards", "0", "hold_ms"], json!("long")), "hold_ms must be a whole number 0..20000."),
        ("a cue not text", with(&g, &["cards", "3", "cue"], json!(3)), "Card 4: cue must be text."),
        ("a when kind unknown", with(&g, &["cards", "1", "when", "0", "kind"], json!("weather")), "Card 2 when 1: unknown condition kind 'weather'"),
        ("a when on an unknown quest", with(&g, &["cards", "1", "when"], json!([{"kind": "quest", "quest": "q_ghost", "status": "active"}])), "unknown quest 'q_ghost'"),
    ];
    for (tag, bad, needle) in cases {
        assert_one(&v, "vignettes", &g, &bad, needle, tag);
    }
    // a card with no style is an image card (4.2's default); 360 characters and the 24th card are allowed
    assert!(added(&v, "vignettes", &g, &without(&g, &["cards", "0", "style"])).is_empty());
    assert!(added(&v, "vignettes", &g, &with(&g, &["cards", "0", "text"], json!(long(360)))).is_empty());
    let full: Vec<Value> = (0..24).map(|_| json!({"style": "text", "text": "x"})).collect();
    assert!(added(&v, "vignettes", &g, &with(&g, &["cards"], json!(full))).is_empty());
    assert!(added(&v, "vignettes", &g, &with(&g, &["music"], json!("none"))).is_empty(), "none is silence");
    drop_scratch(&data);
}

#[test]
fn each_bad_sequence_or_step_field_is_one_error() {
    let data = story_world("steps");
    let v = store::view(&data);
    let (p, end) = (seq_a(), seq_b());
    let steps41: Vec<Value> = (0..41).map(|_| json!({"kind": "vignette", "vignette": "reel_a"})).collect();
    let fx = |effect: Value| with(&p, &["steps", "2", "effects"], json!([effect]));
    let cases: Vec<(&str, Value, Value, &str)> = vec![
        ("no steps", p.clone(), with(&p, &["steps"], json!([])), "A sequence holds 1-40 steps (has 0)."),
        ("41 steps", p.clone(), with(&p, &["steps"], json!(steps41)), "A sequence holds 1-40 steps (has 41)."),
        ("steps an object", p.clone(), with(&p, &["steps"], json!({})), "Steps must be a list."),
        ("a step not an object", p.clone(), with(&p, &["steps", "0"], json!("reel_a")), "Step 1 must be an object"),
        ("an unknown step kind", p.clone(), with(&p, &["steps", "0", "kind"], json!("dialogue")), "Step 1: kind 'dialogue' is not one of vignette, fight, effects, enter, end."),
        ("an unknown step key", p.clone(), with(&p, &["steps", "0", "encounter"], json!("raid")), "Step 1 (vignette): unknown key 'encounter'."),
        ("a vignette step without a vignette", p.clone(), without(&p, &["steps", "0", "vignette"]), "Step 1 (vignette): choose a vignette."),
        ("an unknown vignette", p.clone(), with(&p, &["steps", "0", "vignette"], json!("v_ghost")), "unknown vignette 'v_ghost'"),
        ("a fight without an encounter", p.clone(), with(&p, &["steps", "1", "encounter"], json!("")), "Step 2 (fight): choose an encounter."),
        ("an unknown encounter", p.clone(), with(&p, &["steps", "1", "encounter"], json!("ghost")), "unknown encounter 'ghost'"),
        ("on_lost unknown", p.clone(), with(&p, &["steps", "1", "on_lost"], json!("flee")), "on_lost 'flee' is not one of retry, abort, continue."),
        ("on_fled unknown", p.clone(), with(&p, &["steps", "1", "on_fled"], json!("again")), "on_fled 'again' is not one of retry, abort, continue."),
        ("an empty effects step", p.clone(), with(&p, &["steps", "2", "effects"], json!([])), "an effects step needs 1-8 effects."),
        ("nine effects", p.clone(), with(&p, &["steps", "2", "effects"], json!((0..9).map(|i| json!({"effect": "set_flag", "key": format!("f{i}")})).collect::<Vec<_>>())), "holds at most 8 effects (has 9)."),
        ("a fight effect", p.clone(), fx(json!({"effect": "fight", "encounter": "raid"})), "fight can't run in a sequence step"),
        ("a turn_in effect", p.clone(), fx(json!({"effect": "turn_in", "quest": "q_test"})), "turn_in can't run in a sequence step"),
        ("an open_shop effect", p.clone(), fx(json!({"effect": "open_shop", "shop": "s"})), "open_shop can't run in a sequence step"),
        ("an open_board effect", p.clone(), fx(json!({"effect": "open_board", "board": "board_a"})), "open_board can't run in a sequence step"),
        ("a sequence effect", p.clone(), fx(json!({"effect": "sequence", "sequence": "seq_b"})), "sequence can't run in a sequence step"),
        ("a vignette effect naming no vignette", p.clone(), fx(json!({"effect": "vignette", "vignette": "v_ghost"})), "unknown vignette 'v_ghost'"),
        ("an enter without a location", p.clone(), with(&p, &["steps", "3", "location"], json!("")), "Step 4 (enter): choose a location."),
        ("an enter to an unknown location", p.clone(), with(&p, &["steps", "3", "location"], json!("nowhere")), "unknown location 'nowhere'"),
        ("an anchor not on the map", p.clone(), with(&p, &["steps", "3", "anchor"], json!("altar")), "anchor 'altar' is not an anchor of map 'hub_map'."),
        ("an anchor at a location without a map", p.clone(), with(&with(&p, &["steps", "3", "location"], json!("street")), &["steps", "3", "anchor"], json!("anchor_b")), "needs a location with a map"),
        ("an enter before the last step", p.clone(), with(&p, &["steps", "2"], json!({"kind": "enter", "location": "hub"})), "Step 3 (enter): enter and end may only be the last step."),
        ("an end before the last step", end.clone(), with(&end, &["steps", "1"], json!({"kind": "end"})), "Step 2 (end): enter and end may only be the last step."),
        ("an end to somewhere else", end.clone(), with(&end, &["steps", "2", "to"], json!("credits")), "to 'credits' is not one of map, menu."),
    ];
    for (tag, good, bad, needle) in cases {
        assert_one(&v, "sequences", &good, &bad, needle, tag);
    }
    // the effects 4.3 allows, the anchor of the map, a step order ending without enter or end (it ends as end map)
    let allowed = json!([{"effect": "add_flag", "key": "n", "delta": 1}, {"effect": "start_quest", "quest": "q_test"}, {"effect": "vignette", "vignette": "reel_a"}, {"effect": "set_act", "act": 1}]);
    assert!(added(&v, "sequences", &p, &with(&p, &["steps", "2", "effects"], allowed)).is_empty());
    assert!(added(&v, "sequences", &p, &with(&p, &["steps", "3", "anchor"], json!("anchor_b"))).is_empty());
    let mut no_enter = p.clone();
    no_enter["steps"].as_array_mut().unwrap().pop();
    assert!(added(&v, "sequences", &p, &no_enter).is_empty());
    drop_scratch(&data);
}

#[test]
fn each_bad_board_or_board_quest_field_is_one_error() {
    let data = story_world("boards");
    let v = store::view(&data);
    let (b, q) = (board(), quest_a());
    let cases: Vec<(&str, &str, Value, Value, &str)> = vec![
        ("boards", "no keeper", b.clone(), with(&b, &["keeper"], json!("")), "Choose the board keeper."),
        ("boards", "an unknown keeper", b.clone(), with(&b, &["keeper"], json!("ghost")), "unknown character 'ghost'"),
        ("boards", "an intro over 160", b.clone(), with(&b, &["intro"], json!("a".repeat(161))), "Intro is longer than 160 characters."),
        ("boards", "an open flag with a bad op", b.clone(), with(&b, &["requires_flags", "0", "op"], json!("~")), "Open when 1"),
        ("boards", "an open flag on a reserved key", b.clone(), with(&b, &["requires_flags", "0", "key"], json!("meter.heat")), "names no meter"),
        ("quests", "an unknown board", q.clone(), with(&q, &["board"], json!("ghost_board")), "unknown board 'ghost_board'"),
        ("quests", "a board quest without a template", q.clone(), with(&q, &["template"], json!("")), "A board quest needs a template"),
        ("quests", "a giver who is not the keeper", q.clone(), with(&q, &["giver"], json!("keeper_b")), "Given by 'keeper_b' is not the keeper of board 'board_a' ('keeper_a')"),
        ("quests", "a track unknown", q.clone(), with(&q, &["track"], json!("urgent")), "Track: 'urgent' is not one of main, side."),
        ("quests", "a closed text over 80", q.clone(), with(&q, &["closed_text"], json!("a".repeat(81))), "Closed text is longer than 80 characters."),
    ];
    for (kind, tag, good, bad, needle) in cases {
        assert_one(&v, kind, &good, &bad, needle, tag);
    }
    // an empty giver is fine on a board quest (4.5), and a quest without a board keeps every board field inert
    assert!(added(&v, "quests", &q, &with(&q, &["giver"], json!(""))).is_empty());
    assert!(added(&v, "quests", &q, &with(&with(&q, &["board"], json!("")), &["template"], json!(""))).iter().all(|e| !e.contains("board")));
    drop_scratch(&data);
}

#[test]
fn the_story_effects_and_the_board_line_are_checked_in_dialogue_and_campaign_lists() {
    let data = story_world("dialogue");
    let v = store::view(&data);
    let d = talk_a();
    let fx = |choice: &'static str, i: &'static str| ["nodes", "0", "choices", choice, "effects", i];
    let cases: Vec<(&str, Value, &str)> = vec![
        ("an open_board naming no board", with(&d, &fx("0", "1"), json!({"effect": "open_board", "board": "ghost"})), "unknown board 'ghost'"),
        ("an open_board without a board", with(&d, &fx("0", "1"), json!({"effect": "open_board"})), "choose a board."),
        ("a vignette naming no vignette", with(&d, &fx("1", "0"), json!({"effect": "vignette", "vignette": "ghost"})), "unknown vignette 'ghost'"),
        ("a sequence naming no sequence", with(&d, &fx("2", "0"), json!({"effect": "sequence", "sequence": "ghost"})), "unknown sequence 'ghost'"),
        ("an unknown key on a story effect", with(&d, &fx("1", "0"), json!({"effect": "vignette", "vignette": "reel_a", "once": true})), "unknown key 'once'"),
        ("open_board before another effect", with(&d, &["nodes", "0", "choices", "0", "effects"], json!([{"effect": "open_board", "board": "board_a"}, {"effect": "set_flag", "key": "asked"}])), "open_board must be the last effect."),
        ("sequence before another effect", with(&d, &["nodes", "0", "choices", "2", "effects"], json!([{"effect": "sequence", "sequence": "seq_b"}, {"effect": "set_flag", "key": "asked"}])), "sequence must be the last effect."),
        ("a sequence on a node", with(&d, &["nodes", "0", "effects"], json!([{"effect": "sequence", "sequence": "seq_b"}])), "sequence closes the conversation or opens a screen; put it on a choice"),
        ("an open_board on a node", with(&d, &["nodes", "0", "effects"], json!([{"effect": "open_board", "board": "board_a"}])), "open_board closes the conversation or opens a screen; put it on a choice"),
        ("a campaign effect on a node (Godot's message kept)", with(&d, &["nodes", "0", "effects"], json!([{"effect": "rest"}])), "Node effects may only be set_flag, add_flag or give."),
    ];
    for (tag, bad, needle) in cases {
        assert_one(&v, "dialogues", &d, &bad, needle, tag);
    }
    // planner ruling 2026-10-26 (4.7): a node queues a vignette, the conversation goes on
    assert!(added(&v, "dialogues", &d, &with(&d, &["nodes", "0", "effects"], json!([{"effect": "vignette", "vignette": "reel_a"}]))).is_empty(), "a vignette on a node");
    let unknown = added(&v, "dialogues", &d, &with(&d, &fx("1", "0"), json!({"effect": "zz"})));
    assert!(unknown.iter().any(|m| m.contains("recruit, vignette, sequence, open_board, meter_add")), "{unknown:?}");
    // a campaign trigger may queue a vignette and start a sequence, but opens no board (CAMPAIGN1 T2)
    let trigger = json!({"id": "t_end", "name": "End", "on": "battle", "encounter": "raid", "outcome": "won", "effects": [{"effect": "vignette", "vignette": "reel_a"}, {"effect": "sequence", "sequence": "seq_b"}]});
    assert!(errors_of(&v, "triggers", &trigger).is_empty(), "{:?}", errors_of(&v, "triggers", &trigger));
    assert_one(&v, "triggers", &trigger, &with(&trigger, &["effects", "1"], json!({"effect": "open_board", "board": "board_a"})), "open_board opens a screen", "a trigger opening the board");
    // the service line `board` (4.7)
    assert_eq!(check::dialogue::line_errors("board", &json!("Show me the board."), "Service lines"), Vec::<String>::new());
    assert_eq!(check::dialogue::line_errors("boards", &json!("x"), "Service lines").len(), 1);
    drop_scratch(&data);
}

#[test]
fn a_start_sequence_must_name_a_sequence() {
    let data = story_world("manifest");
    let v = store::view(&data);
    let errors = |m: Value| check::story::manifest_errors(&v, W, m.as_object().unwrap());
    assert_eq!(errors(manifest()), Vec::<String>::new());
    assert_eq!(errors(with(&manifest(), &["start_sequence"], json!(""))), Vec::<String>::new(), "\"\" = the map, as before");
    let found = errors(with(&manifest(), &["start_sequence"], json!("ghost")));
    assert_eq!(found, vec!["Start sequence: unknown sequence 'ghost'.".to_string()]);
    assert!(!check::world::manifest_errors(&v, W).iter().any(|e| e.starts_with("Start sequence")), "wired into the manifest check");
    drop_scratch(&data);
}

#[test]
fn missing_images_cues_and_music_are_warnings() {
    let data = story_world("assets");
    let assets = data.parent().unwrap().join("assets");
    for rel in ["backgrounds/story/card_1.png", "audio/music/theme_1.ogg", "audio/sfx/cue_1.wav"] {
        let p = assets.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, b"x").unwrap();
    }
    let src = source::load(&data);
    let v = check::View::new(&src.catalog, &src.worlds, &src.game, aegis_core::assets::Layers::of(vec![assets.clone()]));
    assert_eq!(check::story::warnings(&v, W), Vec::<String>::new(), "every file is there");
    for rel in ["backgrounds/story/card_1.png", "audio/music/theme_1.ogg", "audio/sfx/cue_1.wav"] {
        fs::remove_file(assets.join(rel)).unwrap();
    }
    let found = check::story::warnings(&v, W);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(found[0].contains("music 'music/theme_1.ogg' is in no asset folder's audio/"), "{found:?}");
    assert!(found[1].contains("card 1: image 'backgrounds/story/card_1.png' is in no asset folder"), "{found:?}");
    assert!(found[2].contains("card 4: cue 'sfx/cue_1.wav'"), "{found:?}");
    assert!(check::world::validate_entry(&v, W, "vignettes", json::dict(&reel_a()), "reel_a").is_empty(), "missing files never block");
    drop_scratch(&data);
}

#[test]
fn the_step_reader_fills_its_defaults() {
    let step = |v: Value| Step::from_obj(v.as_object().unwrap());
    assert_eq!(step(json!({"kind": "fight", "encounter": "raid"})), Some(Step::Fight { encounter: "raid".into(), on_lost: "abort".into(), on_fled: "abort".into() }));
    assert_eq!(step(json!({"kind": "end"})), Some(Step::End { to: "map".into() }));
    assert_eq!(step(json!({"kind": "enter", "location": "hub"})), Some(Step::Enter { location: "hub".into(), anchor: String::new() }));
    assert_eq!(step(json!({"kind": "vignette", "vignette": "reel_a"})), Some(Step::Vignette { vignette: "reel_a".into() }));
    assert_eq!(step(json!({"kind": "effects", "effects": [{"effect": "rest"}]})), Some(Step::Effects { effects: vec![json!({"effect": "rest"})] }));
    assert_eq!(step(json!({"kind": "dialogue"})), None);
    assert_eq!(rpgkit_data::board_opened_flag("board_a"), "board.board_a.opened");
    assert!(check::quest::flag_key_ok(&rpgkit_data::board_opened_flag("board_a")), "a flag key");
}

#[test]
fn the_writer_stores_the_new_fields_and_kinds() {
    let data = story_world("writer");
    // a hand-written manifest without start_sequence resaves byte-identical (WORLDW1) and keeps it out on an update
    let plain = json!({"id": "plain", "name": "Plain", "extends": [], "connects": [], "ruleset": "timeline", "version": 1});
    let path = aegis_content::world::manifest_path(&data, "plain");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    store::set_world(&data, "plain", &dict(json!({"name": "Plain"})), false).expect("a plain world");
    let first = fs::read_to_string(&path).unwrap();
    assert!(first.contains("\"start_sequence\": \"\""), "a new world is written whole: {first}");
    fs::write(&path, aegis_content::file::stringify(&plain)).unwrap();
    let hand = fs::read_to_string(&path).unwrap();
    store::set_world(&data, "plain", &Dict::new(), false).expect("an untouched resave");
    assert_eq!(fs::read_to_string(&path).unwrap(), hand, "an untouched resave is byte-identical");
    store::set_world(&data, "plain", &dict(json!({"tagline": "Flat."})), false).expect("an update");
    assert!(!fs::read_to_string(&path).unwrap().contains("start_sequence"), "start_sequence stays out while it holds its default");
    // start_sequence is written, and an unknown one refused
    store::set_world(&data, W, &dict(json!({"start_sequence": "seq_b"})), false).expect("a start sequence");
    assert_eq!(source::load(&data).worlds[W]["manifest"]["start_sequence"], json!("seq_b"));
    let refused = store::set_world(&data, W, &dict(json!({"start_sequence": "ghost"})), false).err().expect("an unknown sequence refused");
    assert!(refused.iter().any(|e| e.contains("Start sequence: unknown sequence 'ghost'.")), "{refused:?}");
    // a vignette through upsert; a re-save without changes is byte-identical; a bad shape is refused and rolled back
    store::upsert(&data, Some(W), "vignettes", &dict(json!({"id": "card_one", "name": "Card", "cards": [{"style": "title", "title": "Title B"}]})), None).expect("a vignette");
    let file = store::file_of(&data, Some(W), "vignettes");
    let stored = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, Some(W), "vignettes").unwrap(), stored);
    let doc: Value = serde_json::from_str(&stored).unwrap();
    let keys: Vec<String> = doc["entries"][0].as_object().unwrap().keys().cloned().collect();
    assert_eq!(keys, ["id", "name", "music", "frame", "skippable", "once", "cards", "tags"], "stored in schema order, an empty override left out");
    let refused = store::upsert(&data, Some(W), "vignettes", &dict(json!({"id": "bad", "name": "Bad", "cards": {"style": "text"}})), None).err().expect("cards as an object refused");
    assert!(refused.iter().any(|e| e.contains("Cards must be a list.")), "{refused:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), stored, "rolled back");
    // a board quest's fields are stored after tags
    store::upsert(&data, Some(W), "quests", &dict(json!({"id": "q_job", "name": "Job", "board": "board_a", "track": "side"})), None).expect("a board quest");
    let q = fs::read_to_string(store::file_of(&data, Some(W), "quests")).unwrap();
    let doc: Value = serde_json::from_str(&q).unwrap();
    let job = doc["entries"].as_array().unwrap().iter().find(|e| e["id"] == "q_job").cloned().unwrap();
    let keys: Vec<String> = job.as_object().unwrap().keys().cloned().collect();
    assert!(keys.ends_with(&["tags", "board", "track", "show_locked", "closed_text"].map(String::from)), "{keys:?}");
    for kind in ["sequences", "boards"] {
        let text = fs::read_to_string(store::file_of(&data, Some(W), kind)).unwrap();
        let again = store::resave_text(&data, Some(W), kind).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&again).unwrap()["entries"].as_array().map(Vec::len), serde_json::from_str::<Value>(&text).unwrap()["entries"].as_array().map(Vec::len), "{kind}");
    }
    drop_scratch(&data);
}
