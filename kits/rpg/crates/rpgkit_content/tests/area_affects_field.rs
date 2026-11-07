//! A3.2-K2 (INTENT2 IR9, IR6; `docs/sessions/A3_SHAPES.md` section 1) on scratch data folders: the skill field
//! `affects` ("" = the row's default, `all`, `enemies`, `allies`) is one schema field of ATTACK skills in the `area`
//! section, loaded into the typed catalog, written only when set (a re-save is byte-identical), and checked: a value
//! outside the options, or `affects` on a single-target attack, is one error naming the field. The `area` enum offers
//! the six A3.1 shapes (CONE, RING and ARC new) beside the legacy SWEEP; hazards take RING and ARC.

use aegis_content::schema::fields;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_content::check::{self, skill};
use rpgkit_content::{source, store};
use rpgkit_data::Affects;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_area_affects_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

fn field_of(kind: &str, key: &str) -> Dict {
    fields(source::schema(), "content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("content.{kind}.{key}")).clone()
}

fn options(kind: &str, key: &str) -> Vec<String> {
    json::aget(&field_of(kind, key), "options").iter().map(gstr).collect()
}

/// Engine skills, a single-target strike, a BURST blast without `affects` and a RING kick with `enemies`.
fn catalog(data: &Path) {
    let one = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    let skills = json!([
        one("move", "MOVE", json!({"system": true})),
        one("rest", "REST", json!({"system": true})),
        one("strike", "ATTACK", json!({"basic": true})),
        one("blast", "ATTACK", json!({"area": "BURST", "reach_min": 1, "reach_max": 3})),
        one("kick", "ATTACK", json!({"area": "RING", "affects": "enemies"}))
    ]);
    fs::write(data.join("catalog").join("skills.json"), json!({"kind": "skills", "version": 2, "entries": skills}).to_string()).unwrap();
}

#[test]
fn the_schema_names_affects_and_the_six_shapes() {
    let f = field_of("skills", "affects");
    assert_eq!((json::s(&f, "type"), json::s(&f, "default"), json::b(&f, "rank", true)), ("enum".to_string(), String::new(), false));
    let mut want = vec![String::new()];
    want.extend(Affects::IDS.iter().map(|s| s.to_string()));
    assert_eq!(options("skills", "affects"), want, "\"\" (the row's default) and the three ids the loader reads");
    assert_eq!(json::aget(json::dget(&f, "when"), "in").iter().map(gstr).collect::<Vec<_>>(), ["ATTACK"], "an ATTACK field, as `area`");
    assert_eq!(json::aget(&f, "section").iter().map(gstr).collect::<Vec<_>>(), ["area"]);
    let users: Vec<&str> = skill::KINDS.iter().copied().filter(|k| skill::has_field(k, "affects")).collect();
    assert_eq!(users, ["ATTACK"], "the checks give it to the same kinds");
    let area = options("skills", "area");
    for shape in ["NONE", "PLUS", "BURST", "LINE", "CONE", "RING", "ARC"] {
        assert!(area.contains(&shape.to_string()), "area offers {shape}");
    }
    assert!(area.contains(&"SWEEP".to_string()), "SWEEP stays accepted until the shipped rows migrate (W2-4 phase C)");
    for shape in ["RING", "ARC"] {
        assert!(rpgkit_data::HAZARD_SHAPES.contains(&shape), "a hazard can take {shape}");
    }
}

#[test]
fn affects_loads_and_is_written_only_when_set() {
    let (_scratch, data) = scratch("load");
    catalog(&data);
    let src = source::load(&data);
    assert_eq!(src.errors.iter().filter(|e| e.contains("catalog")).count(), 0, "{:?}", src.errors);
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    assert_eq!(cat.skill("blast").unwrap().affects, None, "omitted: the row's default");
    assert_eq!(cat.skill("kick").unwrap().affects, Some(Affects::Enemies));
    assert_eq!(cat.skill("strike").unwrap().affects, None);
    let entry = json!({"id": "wave", "name": "Wave", "kind": "ATTACK", "area": "ARC", "affects": "allies", "ranks": []});
    store::upsert(&data, None, "skills", &dict(entry), None).expect("a valid ARC skill with affects");
    let stored: Value = serde_json::from_str(&fs::read_to_string(store::file_of(&data, None, "skills")).unwrap()).unwrap();
    let row = |id: &str| stored["entries"].as_array().unwrap().iter().find(|e| e["id"] == id).unwrap().clone();
    assert_eq!(row("wave")["affects"], json!("allies"));
    assert!(row("blast").get("affects").is_none(), "an unset affects is not written");
    assert_eq!(rpgkit_data::Catalog::from_value(&source::load(&data).catalog).skill("wave").unwrap().affects, Some(Affects::Allies));
    let text = fs::read_to_string(store::file_of(&data, None, "skills")).unwrap();
    assert_eq!(store::resave_text(&data, None, "skills").unwrap(), text, "a re-save without changes is byte-identical");
    drop_scratch(&data);
}

#[test]
fn each_broken_rule_is_one_error_naming_affects() {
    let (_scratch, data) = scratch("rules");
    catalog(&data);
    let v = store::view(&data);
    let errors = |id: &str, change: Value| {
        let mut e = v.get("skills", id).clone();
        for (k, x) in change.as_object().unwrap() {
            e.insert(k.clone(), x.clone());
        }
        check::validate_entry(&v, "skills", &e, id)
    };
    assert_eq!(errors("kick", json!({})), Vec::<String>::new(), "the fixture is clean");
    for value in Affects::IDS {
        assert_eq!(errors("blast", json!({"affects": value})), Vec::<String>::new(), "{value} on an area");
    }
    for shape in ["CONE", "RING", "ARC"] {
        assert_eq!(errors("blast", json!({"area": shape, "reach_min": 0, "reach_max": 0})), Vec::<String>::new(), "{shape} is a valid area");
    }
    let bogus = errors("blast", json!({"affects": "foes"}));
    assert!(bogus.len() == 1 && bogus[0].starts_with("Affects: 'foes'"), "{bogus:?}");
    assert_eq!(errors("strike", json!({"affects": "enemies"})), ["Affects needs an area (it picks which units the shape hits)."]);
    drop_scratch(&data);
}
