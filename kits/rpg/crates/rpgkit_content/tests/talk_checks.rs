//! BUNKS1 BK5 (spec BUNKS1 section 2): the `talks` world kind's field table, loader and
//! checks. A written pair talk and a template talk check clean; each field set wrong adds exactly one error; the
//! writer stores the kind (defaults left out) and re-saves it byte for byte. Scratch data folders only.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::talk::TALKS_KIND;
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

/// A written talk with every field a pair takes.
fn pair_talk() -> Value {
    json!({"id": "pair_a", "name": "Pair A", "pair": ["ana", "ben"], "requires_flags": [{"key": "ward.calm", "value": true}], "min_together": 2, "dialogue": "chat", "reward": "token"})
}

/// A template talk picked by a temperament.
fn template_talk() -> Value {
    json!({"id": "tpl_a", "name": "Tpl A", "template": true, "temperament": "temper_x", "dialogue": "chat"})
}

/// A scratch world: a class, a party unit, two party characters, an item, two slotless passives and a dialogue.
fn world(tag: &str) -> PathBuf {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_talk_checks_{tag}"));
    let data = t.join("data");
    t.until_thread_end();
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds").join(W)).unwrap();
    write_catalog(&data, "classes", json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2}}]));
    write_catalog(&data, "units", json!([{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}]));
    fs::write(data.join("catalog").join("items.json"), json!({"kind": "items", "version": 2, "entries": [{"id": "token", "name": "Token", "item_type": "MATERIAL", "value": 1}]}).to_string()).unwrap();
    write_catalog(&data, "passives", json!([{"id": "temper_x", "name": "Temper X", "slot": ""}, {"id": "orig_x", "name": "Origin X", "slot": ""}]));
    fs::write(aegis_content::world::manifest_path(&data, W), json!({"id": W, "name": "Ward", "extends": [], "connects": [], "start_location": "hub", "version": 1}).to_string()).unwrap();
    write_kind(&data, "locations", json!([{"id": "hub", "name": "Hub", "rest": true}]));
    write_kind(&data, "characters", json!([{"id": "ana", "name": "Ana", "role": "party", "base_unit": "hero"}, {"id": "ben", "name": "Ben", "role": "party", "base_unit": "hero"}]));
    write_kind(&data, "dialogues", json!([{"id": "chat", "name": "Chat", "start": "n0", "services": false, "nodes": [{"id": "n0", "text": "{a} and {b}.", "end": true}]}]));
    write_kind(&data, TALKS_KIND, json!([pair_talk(), template_talk()]));
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn errors_of(v: &check::View, entry: &Value) -> Vec<String> {
    check::world::validate_entry(v, W, TALKS_KIND, entry.as_object().unwrap(), &json::s(entry.as_object().unwrap(), "id"))
}

fn added(v: &check::View, good: &Value, bad: &Value) -> Vec<String> {
    let before = errors_of(v, good);
    errors_of(v, bad).into_iter().filter(|e| !before.contains(e)).collect()
}

fn with(entry: &Value, key: &str, value: Value) -> Value {
    let mut out = entry.clone();
    out[key] = value;
    out
}

fn one(found: Vec<String>, needle: &str, tag: &str) {
    assert_eq!(found.len(), 1, "{tag}: {found:?}");
    assert!(found[0].contains(needle), "{tag}: {found:?} (wanted '{needle}')");
}

#[test]
fn the_schema_carries_the_talks_kind() {
    let schema = source::schema();
    let field = |key: &str| fields(schema, "world", TALKS_KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("world.talks.{key}"));
    assert_eq!((json::s(&field("pair"), "type"), json::s(&field("pair"), "ref")), ("refs".to_string(), "characters".to_string()));
    assert_eq!((json::s(&field("dialogue"), "type"), json::s(&field("dialogue"), "ref")), ("ref".to_string(), "dialogues".to_string()));
    assert_eq!(json::s(&field("reward"), "ref"), "base:items");
    for key in ["temperament", "origin"] {
        assert_eq!(json::s(&field(key), "ref"), "base:passives", "{key}");
    }
    assert_eq!((json::s(&field("template"), "type"), json::s(&field("min_together"), "type")), ("bool".to_string(), "int".to_string()));
    assert!(source::WORLD_KINDS.contains(&TALKS_KIND) && check::world::KINDS.contains(&TALKS_KIND), "a loaded and checked world kind");
    let data = world("schema");
    let src = source::load(&data);
    let rows: Vec<String> = json::arr(&src.worlds[W]["own"][TALKS_KIND]).iter().map(|r| json::s(json::dict(r), "id")).collect();
    assert_eq!(rows, ["pair_a", "tpl_a"], "loaded in order");
    drop_scratch(&data);
}

#[test]
fn a_talk_checks_clean_and_each_bad_field_is_one_error() {
    let data = world("rows");
    let v = store::view(&data);
    for good in [pair_talk(), template_talk(), with(&with(&template_talk(), "temperament", json!("")), "origin", json!("orig_x"))] {
        let found = errors_of(&v, &good);
        assert!(found.is_empty(), "{good}: {found:?}");
    }
    let pair = pair_talk();
    let tpl = template_talk();
    let cases: Vec<(&Value, Value, &str)> = vec![
        (&pair, with(&pair, "pair", json!(["ana"])), "names a pair of two characters"),
        (&pair, with(&pair, "pair", json!(["ana", "ana"])), "lists 'ana' twice"),
        (&pair, with(&pair, "pair", json!(["ana", "ghost"])), "unknown character 'ghost'"),
        (&pair, with(&pair, "temperament", json!("temper_x")), "template talks only"),
        (&pair, with(&pair, "requires_flags", json!([{"key": "Bad Key", "value": true}])), "must be non-empty"),
        (&pair, with(&pair, "min_together", json!(1000)), "min_together must be a whole number 0..999"),
        (&pair, with(&pair, "dialogue", json!("")), "choose the dialogue"),
        (&pair, with(&pair, "dialogue", json!("ghost")), "unknown dialogue 'ghost'"),
        (&pair, with(&pair, "reward", json!("ghost")), "unknown base item 'ghost'"),
        (&tpl, with(&tpl, "pair", json!(["ana", "ben"])), "a template talk names no pair"),
        (&tpl, with(&tpl, "temperament", json!("")), "needs a temperament or an origin"),
        (&tpl, with(&tpl, "temperament", json!("ghost")), "unknown base passive 'ghost'"),
    ];
    for (good, bad, needle) in cases {
        one(added(&v, good, &bad), needle, needle);
    }
    let blocking = check::world::blocking_errors(&v, W, TALKS_KIND, &dict(with(&pair, "requires_flags", json!({}))), "pair_a");
    assert!(blocking.iter().any(|e| e.contains("must be a list")), "{blocking:?}");
    drop_scratch(&data);
}

#[test]
fn the_writer_stores_talks_and_resaves_them_byte_for_byte() {
    let data = world("writer");
    fs::remove_file(store::file_of(&data, Some(W), TALKS_KIND)).unwrap();
    store::upsert(&data, Some(W), TALKS_KIND, &dict(pair_talk()), None).expect("a pair talk");
    store::upsert(&data, Some(W), TALKS_KIND, &dict(template_talk()), None).expect("a template talk");
    let file = store::file_of(&data, Some(W), TALKS_KIND);
    assert!(file.ends_with("talks.json"), "{file:?}");
    let stored = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, Some(W), TALKS_KIND).unwrap(), stored, "a re-save without changes is byte-identical");
    let saved: Value = serde_json::from_str(&stored).unwrap();
    let pair_row = json::dict(&saved["entries"][0]);
    assert!(!pair_row.contains_key("template") && !pair_row.contains_key("temperament") && !pair_row.contains_key("origin"), "defaults left out: {pair_row:?}");
    let problems: Vec<String> = store::check(&data).into_iter().filter(|p| p.starts_with(&format!("{W}/talks")) || p.starts_with(&format!("{W}/dialogues"))).collect();
    assert!(problems.is_empty(), "the template's dialogue may name {{a}} and {{b}}: {problems:?}");
    // without a template talk naming it, the dialogue's {a} and {b} are unknown tokens, as before
    store::remove(&data, Some(W), TALKS_KIND, "tpl_a").expect("removed");
    let problems: Vec<String> = store::check(&data).into_iter().filter(|p| p.starts_with(&format!("{W}/dialogues"))).collect();
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems.iter().all(|p| p.contains("Unknown token")), "{problems:?}");
    drop_scratch(&data);
}
