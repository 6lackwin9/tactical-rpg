//! PR6-11 (`docs/sessions/PK6_PK7_BOSSES_ENDINGS.md` 4): the `endings` world kind's field table, loader and checks,
//! and the `end` effect's kind `ending` with its `ending` key. An ending written to section 4's shape checks clean and
//! each field set wrong adds exactly one error; an `end` of kind `ending` needs a world whose last ending has no gates
//! and takes no card or text; only that kind takes `ending`; the writer stores the kind and re-saves it byte for byte.
//! Scratch data folders only.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::ending::{ENDINGS_KIND, END_KIND_ENDING};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const W: &str = "ward";

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn write_catalog(data: &Path, kind: &str, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(store::file_of(data, Some(W), kind), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

/// An ending written to section 4's shape (every field set).
fn ending_a() -> Value {
    json!({"id": "ending_a", "name": "Ending A", "title": "The First Ending", "tone": "warn", "art": "fin",
        "requires_flags": [{"key": "ward.spikes", "value": true}], "requires_meter": [{"id": "lean", "op": "<=", "value": 40}],
        "epilogue_paragraphs": ["The city lives.", "The warmth fails."],
        "world_outcomes": [{"flag": "ward.youths", "op": ">=", "value": 1, "text_if_true": "{value} youths were saved.", "text_if_false": "None were saved."}],
        "member_fates_template": {"track": "mark", "member": "{name} ({stage}).", "departed": "{name} fell ({cause}).", "causes": {"story": "{name} was marked."}}})
}

/// The fallback: no gates.
fn fallback() -> Value {
    json!({"id": "ending_b", "name": "Ending B", "epilogue_paragraphs": ["They hold the roof."]})
}

/// A scratch campaign world: a class, a party unit, the staged unit track `mark`, the manifest, a location, the gauge
/// `lean`, the vignette `fin` and the two endings (the fallback last).
fn world(tag: &str) -> PathBuf {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_ending_checks_{tag}"));
    let data = t.join("data");
    t.until_thread_end();
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds").join(W)).unwrap();
    write_catalog(&data, "classes", json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2}}]));
    write_catalog(&data, "units", json!([{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}]));
    write_catalog(&data, "unit_tracks", json!([{"id": "mark", "name": "Mark", "mark": "", "cause": "", "stages": [{"name": "Grey"}, {"name": "Still"}]}]));
    fs::write(aegis_content::world::manifest_path(&data, W), json!({"id": W, "name": "Ward", "extends": [], "connects": [], "start_location": "hub", "campaign": {"start_act": 1}, "version": 1}).to_string()).unwrap();
    write_kind(&data, "locations", json!([{"id": "hub", "name": "Hub", "rest": true}]));
    write_kind(&data, "meters", json!([{"id": "lean", "name": "Lean", "kind": "gauge", "min": 0, "max": 100, "start": 0, "bands": [{"id": "b0", "name": "Any", "from": 0, "tone": "calm"}], "hud": {"short": "Lean", "order": 1}}]));
    write_kind(&data, "vignettes", json!([{"id": "fin", "name": "Fin", "cards": [{"style": "title", "title": "Fin"}]}]));
    write_kind(&data, ENDINGS_KIND, json!([ending_a(), fallback()]));
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn errors_of(v: &check::View, kind: &str, entry: &Value) -> Vec<String> {
    check::world::validate_entry(v, W, kind, entry.as_object().unwrap(), &json::s(entry.as_object().unwrap(), "id"))
}

fn added(v: &check::View, kind: &str, good: &Value, bad: &Value) -> Vec<String> {
    let before = errors_of(v, kind, good);
    errors_of(v, kind, bad).into_iter().filter(|e| !before.contains(e)).collect()
}

fn with(entry: &Value, path: &[&str], value: Value) -> Value {
    let mut out = entry.clone();
    let mut at = &mut out;
    for p in path {
        at = match p.parse::<usize>() {
            Ok(n) => &mut at[n],
            Err(_) => &mut at[*p],
        };
    }
    *at = value;
    out
}

fn one(found: Vec<String>, needle: &str, tag: &str) {
    assert_eq!(found.len(), 1, "{tag}: {found:?}");
    assert!(found[0].contains(needle), "{tag}: {found:?} (wanted '{needle}')");
}

/// The problems of an effect list holding `fx` in world `W`.
fn fx_errors(v: &check::View, fx: Value) -> Vec<String> {
    check::campaign::effects_errors(v, W, &json!([fx]), "Effects")
}

#[test]
fn the_schema_carries_the_endings_kind_and_the_end_kind() {
    let schema = source::schema();
    let field = |key: &str| fields(schema, "world", ENDINGS_KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("world.endings.{key}"));
    assert_eq!((json::s(&field("art"), "type"), json::s(&field("art"), "ref")), ("ref".to_string(), "vignettes".to_string()));
    assert_eq!(json::sget(&field("tone"), "options"), rpgkit_data::METER_TONES.map(String::from).to_vec());
    for key in ["requires_flags", "requires_meter", "epilogue_paragraphs", "world_outcomes"] {
        assert_eq!((json::s(&field(key), "type"), json::get(&field(key), "default")), ("json".to_string(), &json!([])), "{key}");
    }
    assert_eq!(json::get(&field("member_fates_template"), "default"), &json!({}));
    assert!(source::WORLD_KINDS.contains(&ENDINGS_KIND) && check::world::KINDS.contains(&ENDINGS_KIND), "a loaded and checked world kind");
    assert!(rpgkit_data::END_KINDS.contains(&END_KIND_ENDING));
    let data = world("schema");
    let src = source::load(&data);
    let rows: Vec<String> = json::arr(&src.worlds[W]["own"][ENDINGS_KIND]).iter().map(|r| json::s(json::dict(r), "id")).collect();
    assert_eq!(rows, ["ending_a", "ending_b"], "loaded in order");
    drop_scratch(&data);
}

#[test]
fn an_ending_checks_clean_and_each_bad_field_is_one_error() {
    let data = world("rows");
    let v = store::view(&data);
    let good = ending_a();
    let found = errors_of(&v, ENDINGS_KIND, &good);
    assert!(found.is_empty(), "{found:?}");
    assert!(errors_of(&v, ENDINGS_KIND, &fallback()).is_empty(), "{:?}", errors_of(&v, ENDINGS_KIND, &fallback()));
    let long = "x".repeat(601);
    let cases: Vec<(&str, Value, &str)> = vec![
        ("a long title", with(&good, &["title"], json!("t".repeat(49))), "title is longer than 48"),
        ("an unknown tone", with(&good, &["tone"], json!("gold")), "'gold' is not one of"),
        ("an unknown art", with(&good, &["art"], json!("ghost")), "unknown vignette 'ghost'"),
        ("a bad flag gate", with(&good, &["requires_flags", "0", "key"], json!("Bad Key")), "must be non-empty"),
        ("an unknown meter", with(&good, &["requires_meter", "0", "id"], json!("ghost")), "unknown meter 'ghost'"),
        ("no meter", with(&good, &["requires_meter", "0", "id"], json!("")), "choose a meter"),
        ("a bad op", with(&good, &["requires_meter", "0", "op"], json!("~")), "op '~' is not one of"),
        ("a text value", with(&good, &["requires_meter", "0", "value"], json!("40")), "value must be a whole number"),
        ("an unknown meter key", with(&good, &["requires_meter", "0", "band"], json!("x")), "unknown key 'band'"),
        ("five meter gates", with(&good, &["requires_meter"], json!([{"id": "lean", "value": 1}, {"id": "lean", "value": 1}, {"id": "lean", "value": 1}, {"id": "lean", "value": 1}, {"id": "lean", "value": 1}])), "at most 4 meter conditions"),
        ("no paragraphs", with(&good, &["epilogue_paragraphs"], json!([])), "must hold 1..6 texts"),
        ("a blank paragraph", with(&good, &["epilogue_paragraphs", "1"], json!(" ")), "paragraph 2: must not be empty"),
        ("a long paragraph", with(&good, &["epilogue_paragraphs", "0"], json!(long)), "longer than 600"),
        ("an outcome with no text", with(&with(&good, &["world_outcomes", "0", "text_if_true"], json!("")), &["world_outcomes", "0", "text_if_false"], json!("")), "needs text_if_true or text_if_false"),
        ("an outcome bad flag", with(&good, &["world_outcomes", "0", "flag"], json!("Bad Key")), "must be non-empty"),
        ("an unknown outcome key", with(&good, &["world_outcomes", "0", "note"], json!("x")), "unknown key 'note'"),
        ("an unknown track", with(&good, &["member_fates_template", "track"], json!("ghost")), "unknown unit track 'ghost'"),
        ("an unknown fate key", with(&good, &["member_fates_template", "who"], json!("x")), "unknown key 'who'"),
        ("a long fate line", with(&good, &["member_fates_template", "member"], json!("y".repeat(201))), "member is longer than 200"),
        ("causes as a list", with(&good, &["member_fates_template", "causes"], json!(["story"])), "causes must be an object"),
    ];
    for (tag, bad, needle) in cases {
        one(added(&v, ENDINGS_KIND, &good, &bad), needle, tag);
    }
    let blocking = check::world::blocking_errors(&v, W, ENDINGS_KIND, &dict(with(&good, &["epilogue_paragraphs"], json!({}))), "ending_a");
    assert!(blocking.iter().any(|e| e.contains("must be a list")), "{blocking:?}");
    drop_scratch(&data);
}

#[test]
fn the_end_of_kind_ending_names_a_row_and_needs_the_fallback_last() {
    let data = world("end");
    let v = store::view(&data);
    let fine = [json!({"effect": "end", "kind": "ending"}), json!({"effect": "end", "kind": "ending", "ending": "ending_a"}), json!({"effect": "end", "kind": "fail", "text": "Lost."})];
    for fx in fine {
        let found = fx_errors(&v, fx.clone());
        assert!(found.is_empty(), "{fx}: {found:?}");
    }
    one(fx_errors(&v, json!({"effect": "end", "kind": "ending", "ending": "ghost"})), "unknown ending 'ghost'", "an unknown ending");
    one(fx_errors(&v, json!({"effect": "end", "kind": "fail", "text": "Lost.", "ending": "ending_a"})), "ending is only for kind ending", "ending on a fail");
    one(fx_errors(&v, json!({"effect": "end", "kind": "fail"})), "an end needs a card or a text", "a fail still needs a card or text");
    one(fx_errors(&v, json!({"effect": "end", "kind": "won", "text": "Won."})), "is not one of fail, bad_end, ending", "an unknown kind");
    // the gated row last: no fallback
    write_kind(&data, ENDINGS_KIND, json!([fallback(), ending_a()]));
    let v = store::view(&data);
    one(fx_errors(&v, json!({"effect": "end", "kind": "ending"})), "last ending must have no requires", "no fallback last");
    // no endings at all
    fs::remove_file(store::file_of(&data, Some(W), ENDINGS_KIND)).unwrap();
    let v = store::view(&data);
    one(fx_errors(&v, json!({"effect": "end", "kind": "ending"})), "has no endings", "no endings");
    drop_scratch(&data);
}

#[test]
fn the_writer_stores_endings_and_resaves_them_byte_for_byte() {
    let data = world("writer");
    fs::remove_file(store::file_of(&data, Some(W), ENDINGS_KIND)).unwrap();
    store::upsert(&data, Some(W), ENDINGS_KIND, &dict(ending_a()), None).expect("an ending");
    store::upsert(&data, Some(W), ENDINGS_KIND, &dict(fallback()), None).expect("the fallback");
    let file = store::file_of(&data, Some(W), ENDINGS_KIND);
    assert!(file.ends_with("endings.json"), "{file:?}");
    let stored = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, Some(W), ENDINGS_KIND).unwrap(), stored, "a re-save without changes is byte-identical");
    let problems: Vec<String> = store::check(&data).into_iter().filter(|p| p.starts_with(&format!("{W}/endings"))).collect();
    assert!(problems.is_empty(), "{problems:?}");
    let refused = store::upsert(&data, Some(W), ENDINGS_KIND, &dict(with(&ending_a(), &["world_outcomes"], json!({}))), None).err().expect("a shape problem blocks");
    assert!(refused.iter().any(|e| e.contains("must be a list")), "{refused:?}");
    drop_scratch(&data);
}
