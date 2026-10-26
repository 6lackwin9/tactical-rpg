//! Ruleset rows and the world field (RULESET1-R, docs/specs/RULESET1_TASKS.md 1.1-1.2): the schema lists what the code
//! owns (ruleset ids, row ids, the rule register, the intents); a valid row loads and reads as its numbers, a row
//! without them reads the schema defaults; every rule of 1.2 broken alone is one load error naming its field; the
//! writer takes the kind; a world naming a ruleset without its row, or an unknown ruleset or flag, is a content error.
//! Scratch data folders only.

use aegis_content::schema::{fields, normalize};
use aegis_core::json::{self, gstr, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_rules::ruleset::{self, Numbers, Ruleset, INTENTS, REGISTER, ROW_IDS, RULESETS};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// A scratch data folder `<tmp>/<tag>/data` with empty catalog and worlds folders.
fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_rulesets_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

/// Writes a catalog file as a fixture (the files are the test's own input; the writer is tested below).
fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn write_world(data: &Path, id: &str, manifest: Value) {
    let dir = data.join("worlds").join(id);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), manifest.to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// Skill families of each kind the basics rule reads (one rank each), and a derived stat for the poise rule.
fn base_catalog(data: &Path) {
    let skill = |id: &str, kind: &str| json!({"id": id, "name": id, "kind": kind, "ranks": []});
    write_kind(data, "skills", 2, json!([skill("move", "MOVE"), skill("rest", "REST"), skill("strike", "ATTACK"), skill("brace", "DEFEND"), skill("mend", "SUPPORT"), skill("leap", "DISENGAGE")]));
    write_kind(data, "derived_stats", 1, json!([{"id": "max_poise", "name": "Max Poise", "formula": "CONS * 2"}]));
}

/// A row that passes every rule, using each rule the checks tie to another field.
fn good_row() -> Value {
    json!({
        "id": "timeline", "name": "Timeline",
        "rules": ["round_intent", "enemy_moves", "deck", "poise", "ap_cost", "pierce", "reach", "flank", "plan_badges",
                  "forced_move", "ground_hazards", "arena_hazards", "triggers", "unit_meters", "hp_cost", "ignore_guard", "ai_tactics", "execute"],
        "intents": ["assault", "guard", "support", "maneuver"],
        "basics": ["move", "rest", "strike", "brace"],
        "poise_stat": "max_poise",
        "break_chance_scale": 0.5,
        "deck_slots": {"active": 3, "passive": 2, "keystone": 1}
    })
}

fn ruleset_errors(src: &source::Source) -> Vec<String> {
    src.errors.iter().filter(|e| e.contains("rulesets.json")).cloned().collect()
}

fn options(section: &str, kind: &str, key: &str) -> Vec<String> {
    let f = fields(source::schema(), section, kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("{section}.{kind}.{key}"));
    json::aget(f, "options").iter().map(gstr).collect()
}

fn strs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn the_schema_lists_what_the_code_owns() {
    assert_eq!(options("content", "derived_stats", "rulesets"), strs(&RULESETS), "a derived row's rulesets");
    let mut world = vec![String::new()];
    world.extend(strs(&RULESETS));
    assert_eq!(options("world", "world", "ruleset"), world, "a world's ruleset: empty (inherited) or a ruleset id");
    assert_eq!(options("content", ruleset::KIND, "id"), strs(&ROW_IDS), "row ids: classic has no row");
    assert_eq!(options("content", ruleset::KIND, "rules"), REGISTER.iter().map(|r| r.id.to_string()).collect::<Vec<_>>(), "the rule register");
    assert_eq!(options("content", ruleset::KIND, "intents"), strs(&INTENTS));
}

#[test]
fn a_row_without_numbers_reads_the_schema_defaults() {
    let row = normalize(fields(source::schema(), "content", ruleset::KIND), &dict(json!({"id": "timeline"})), false);
    let r = Ruleset::from_row(&row);
    assert_eq!(r.numbers, Numbers::default(), "Numbers::default() is the schema's defaults");
    assert_eq!((r.id.as_str(), r.name.as_str(), r.rules.len()), ("timeline", "Timeline", 0));
}

#[test]
fn a_valid_row_loads_and_reads_as_its_rules_and_numbers() {
    let data = scratch("valid");
    base_catalog(&data);
    write_kind(&data, ruleset::KIND, 1, json!([good_row()]));
    let src = source::load(&data);
    assert_eq!(ruleset_errors(&src), Vec::<String>::new());
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    let r = Ruleset::of(&cat, "timeline").expect("the row");
    assert_eq!(r.rules, REGISTER.iter().map(|x| x.id).collect::<Vec<_>>(), "every rule, in register order");
    assert!(!r.stances(), "round_intent makes the classic stances inert");
    assert_eq!((r.numbers.break_chance_scale, r.numbers.poise_stat.as_str()), (0.5, "max_poise"));
    assert_eq!((r.numbers.deck_slots.active, r.numbers.deck_slots.passive, r.numbers.deck_slots.keystone), (3, 2, 1));
    assert_eq!(r.numbers.basics, strs(&["move", "rest", "strike", "brace"]));
    let d = Numbers::default();
    assert_eq!((r.numbers.stagger_windup_mult, r.numbers.rest_ap_cap), (d.stagger_windup_mult, d.rest_ap_cap), "unset numbers keep their defaults");
    assert_eq!(Ruleset::of(&cat, "").unwrap(), Ruleset::classic());
    assert!(Ruleset::of(&cat, "classic").unwrap().stances());
    drop_scratch(&data);
}

#[test]
fn without_a_file_there_is_no_row_and_timeline_cannot_be_had() {
    let data = scratch("missing");
    let src = source::load(&data);
    assert_eq!(ruleset_errors(&src), Vec::<String>::new());
    assert_eq!(src.catalog[ruleset::KIND], json!([]));
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    assert!(Ruleset::of(&cat, "timeline").is_err());
    assert_eq!(Ruleset::of_or_classic(&cat, "timeline"), Ruleset::classic());
    assert!(Ruleset::of(&cat, "chess").unwrap_err().contains("unknown ruleset 'chess'"));
    drop_scratch(&data);
}

#[test]
fn each_broken_rule_is_one_error_naming_its_field() {
    let cases: [(&str, Value, &str); 11] = [
        ("id", json!("classic"), "ID: 'classic' is not one of timeline"),
        ("name", json!(""), "Name is required."),
        ("name", json!("A name that runs past the limit"), "Name is 31 characters (max 24)."),
        ("rules", json!(["round_intent", "chains"]), "Rules: 'chains' is not one of"),
        ("rules", json!(["round_intent", "deck", "poise", "deck"]), "Rules lists 'deck' twice."),
        ("intents", json!(["assault", "guard", "support", "maneuver", "dance"]), "Intents: 'dance' is not one of"),
        ("basics", json!(["move", "rest", "strike", "brace", "move"]), "Basics lists 'move' twice."),
        ("basics", json!(["move", "rest", "strike", "brace", "nothing"]), "Basics: unknown skill family 'nothing'."),
        ("basics", json!([]), "Basics: deck needs at least one basic."),
        ("poise_stat", json!("grit"), "Poise stat: unknown derived stat 'grit'."),
        ("poise_stat", json!(""), "Poise stat: poise needs a derived stat."),
    ];
    let slots: [(Value, &str); 4] = [
        (json!({"active": 9, "passive": 2, "keystone": 1}), "Deck slots: active must be a whole number 1..8."),
        (json!({"active": 3, "passive": 5}), "Deck slots: passive must be a whole number 0..4."),
        (json!({"passive": 2}), "Deck slots: deck needs an active count (1..8)."),
        (json!({"active": 3, "relic": 1}), "Deck slots: 'relic' is not one of active, passive, keystone."),
    ];
    let extra: [(&str, Value, &str); 2] = [
        // an intent the row does not list, on a basic of that intent (mend: SUPPORT, so support)
        ("intents", json!(["assault", "guard", "maneuver"]), "Basics: 'mend' is of intent support, which Intents does not list."),
        // intents other than the default need the round_intent rule
        ("rules", json!(["deck", "poise"]), "Intents need round_intent in Rules."),
    ];
    let data = scratch("rules");
    base_catalog(&data);
    let mut runs: Vec<(Value, String)> = Vec::new();
    for (key, v, want) in cases {
        let mut row = good_row();
        row[key] = v;
        runs.push((row, want.to_string()));
    }
    for (v, want) in slots {
        let mut row = good_row();
        row["deck_slots"] = v;
        runs.push((row, want.to_string()));
    }
    let mut mend = good_row();
    mend["basics"] = json!(["move", "rest", "strike", "brace", "mend"]);
    mend["intents"] = extra[0].1.clone();
    runs.push((mend, extra[0].2.to_string()));
    let mut empty = good_row();
    empty["intents"] = json!([]);
    empty["basics"] = json!(["move", "rest"]);
    runs.push((empty, "Intents: list at least one.".to_string()));
    let mut no_intent = good_row();
    no_intent["rules"] = extra[1].1.clone();
    no_intent["intents"] = json!(["assault", "guard"]);
    runs.push((no_intent, extra[1].2.to_string()));
    for (row, want) in runs {
        write_kind(&data, ruleset::KIND, 1, json!([row]));
        let src = source::load(&data);
        let errors = ruleset_errors(&src);
        assert_eq!(errors.len(), 1, "{row}: {errors:?}");
        assert!(errors[0].contains(&want), "{row}: {errors:?} lacks {want:?}");
        assert_eq!(src.catalog[ruleset::KIND], json!([]), "a row with a problem does not load");
    }
    write_kind(&data, ruleset::KIND, 1, json!([good_row(), good_row()]));
    let errors = ruleset_errors(&source::load(&data));
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].ends_with("ID 'timeline' is already used by another ruleset."), "{errors:?}");
    drop_scratch(&data);
}

/// RULESET1_TASKS 1.2 ranges: the schema's min and max, which loading a row applies (a number past a bound is read as
/// the bound, as for every kind).
#[test]
fn numbers_past_their_range_load_at_the_schema_bounds() {
    let data = scratch("ranges");
    base_catalog(&data);
    let mut row = good_row();
    let mut want: Vec<(String, f64)> = Vec::new();
    for f in fields(source::schema(), "content", ruleset::KIND).iter().map(json::dict) {
        let (Some(lo), Some(hi)) = (f.get("min").and_then(Value::as_f64), f.get("max").and_then(Value::as_f64)) else { continue };
        let key = json::s(f, "key");
        let below = key.len() % 2 == 0;
        row[&key] = json!(if below { lo - 5.0 } else { hi + 5.0 });
        want.push((key, if below { lo } else { hi }));
    }
    assert_eq!(want.len(), 10, "every number of 1.2 has its range in the schema");
    write_kind(&data, ruleset::KIND, 1, json!([row]));
    let src = source::load(&data);
    assert_eq!(ruleset_errors(&src), Vec::<String>::new());
    let loaded = json::dict(&src.catalog[ruleset::KIND][0]).clone();
    for (key, bound) in want {
        assert_eq!(json::f(&loaded, &key, f64::NAN), bound, "{key}");
    }
    drop_scratch(&data);
}

#[test]
fn the_writer_takes_the_kind_and_refuses_a_row_that_adds_a_problem() {
    let data = scratch("writer");
    base_catalog(&data);
    let file = data.join("catalog").join("rulesets.json");
    let up = |entry: Value| store::upsert(&data, None, ruleset::KIND, &dict(entry), None);
    let mut bad = good_row();
    bad["poise_stat"] = json!("grit");
    let refused = up(bad).err().expect("the write is refused");
    assert!(refused.iter().any(|e| e.ends_with("Poise stat: unknown derived stat 'grit'.")), "{refused:?}");
    assert!(!file.exists(), "a refused first row leaves no file");
    up(good_row()).unwrap();
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, ruleset::KIND).unwrap(), text, "a re-save without changes is byte-identical");
    let unknown = up(json!({"id": "freeform", "name": "Freeform"})).err().expect("the write is refused");
    assert!(unknown.iter().any(|e| e.contains("'freeform' is not one of timeline")), "{unknown:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), text, "a refused write puts the file back");
    drop_scratch(&data);
}

fn world_problems(data: &Path, world: &str) -> Vec<String> {
    let v = store::view(data);
    check::world::manifest_errors(&v, world)
}

#[test]
fn a_world_names_a_known_ruleset_with_its_row_and_a_flag_key() {
    let data = scratch("worlds");
    base_catalog(&data);
    write_world(&data, "vale", json!({"id": "vale", "name": "Vale", "ruleset": "timeline"}));
    write_world(&data, "glen", json!({"id": "glen", "name": "Glen", "extends": ["vale"]}));
    write_world(&data, "moor", json!({"id": "moor", "name": "Moor", "ruleset": "chess", "plan_reveal_flag": "Moor Spies"}));
    write_world(&data, "fen", json!({"id": "fen", "name": "Fen", "ruleset": "classic", "plan_reveal_flag": "fen.scouted"}));
    let need = "Ruleset 'timeline' needs its row in rulesets (data/catalog/rulesets.json).";
    assert_eq!(world_problems(&data, "vale"), [need], "a timeline world without the row");
    assert_eq!(world_problems(&data, "glen"), Vec::<String>::new(), "the world that inherits is fixed with its parent");
    assert_eq!(world_problems(&data, "fen"), Vec::<String>::new(), "classic needs no row; a dotted flag key");
    assert_eq!(
        world_problems(&data, "moor"),
        ["Ruleset: 'chess' is not one of classic, timeline (or empty: inherited).", "Plan reveal flag 'Moor Spies' must be a flag key (a-z, 0-9, _ and .)."]
    );
    write_kind(&data, ruleset::KIND, 1, json!([good_row()]));
    assert_eq!(world_problems(&data, "vale"), Vec::<String>::new(), "with the row");
    drop_scratch(&data);
}
