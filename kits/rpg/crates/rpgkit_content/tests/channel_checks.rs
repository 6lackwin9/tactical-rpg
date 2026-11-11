//! PK1-S2a (`docs/sessions/PK1_CORRUPTION.md` 4.1-4.3, FB-167 Q2): the content checks, schema and writer of the
//! Channel fields, on a scratch data folder built here (no shipped data). Each valid form checks clean and each broken
//! rule is one error: a skill's `channel` (keys, `none` / `only`, `cost`, `power`, `effects`; single intent only), its
//! `requires_meter` (a known meter, a `min` in its range) and `hp_cost_pct` (0-99); a ruleset row's Channel numbers
//! (`channel_cost` three whole numbers, `channel_meter` and `channel_rest` naming rows, the rule needing a meter); a
//! unit deck's `channeled` (actives with a channeled version, none twice, at most `channeled_actives`). The schema and the code
//! name the same fields and kinds; the writer stores the Channel numbers at their defaults as no key.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::{self, skill};
use rpgkit_content::{source, store};
use rpgkit_rules::ruleset;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_channel_checks_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn timeline(extra: Value) -> Value {
    let mut row = json!({"id": "timeline", "name": "Timeline", "rules": ["round_intent", "deck", "unit_meters", "field_intent", "channel"], "basics": ["move", "rest", "strike"],
        "deck_slots": {"active": 3, "passive": 0, "keystone": 0}, "channel_meter": "strain", "channel_rest": "feed"});
    for (k, v) in extra.as_object().unwrap() {
        row[k] = v.clone();
    }
    row
}

/// Fixture files: engine skills, Strike, a single-intent Jab and Thrust, a dual-intent Cleave (ARC: Field and Assault),
/// a Feed, a class owning them, the strain meter (0-100), a timeline row with the rule channel and a unit with a deck.
fn catalog(data: &Path, ruleset_row: Value) {
    let one = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(
        data,
        "skills",
        2,
        json!([
            one("move", "MOVE", json!({"system": true})),
            one("rest", "REST", json!({"system": true})),
            one("strike", "ATTACK", json!({"basic": true})),
            one("jab", "ATTACK", json!({})),
            one("thrust", "ATTACK", json!({})),
            one("cleave", "ATTACK", json!({"area": "ARC"})),
            one("feed", "SUPPORT", json!({"target_mode": "SELF", "heal_base": 3, "channel": {"only": true}, "meters": [{"meter": "strain", "delta": 5}]}))
        ]),
    );
    write_kind(
        data,
        "classes",
        1,
        json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {},
                "proficiency": {}, "skill_path": {"jab": 1, "thrust": 1, "cleave": 1}}]),
    );
    write_kind(data, rpgkit_data::UNIT_METERS_KIND, 1, json!([{"id": "strain", "name": "Strain", "short": "St", "min": 0, "max": 100, "start": 0, "show": "bar", "skills": ["feed"]}]));
    write_kind(data, ruleset::KIND, 1, json!([ruleset_row]));
    write_kind(data, "units", 1, json!([{"id": "drillman", "name": "Drillman", "side": "enemy", "class_id": "drill", "level": 1, "deck": {"active": ["jab", "thrust", "cleave"]}}]));
}

fn errors_of(v: &check::View, kind: &str, id: &str, change: &Value) -> Vec<String> {
    let mut e = v.get(kind, id).clone();
    assert!(!e.is_empty(), "fixture {kind} {id}");
    for (k, x) in change.as_object().unwrap() {
        e.insert(k.clone(), x.clone());
    }
    check::validate_entry(v, kind, &e, id)
}

fn field_of(kind: &str, key: &str) -> Dict {
    fields(source::schema(), "content", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("content.{kind}.{key}")).clone()
}

/// The schema and the code name the same skill fields and kinds (the authored kinds), not per rank; the ruleset row's
/// Channel numbers carry the code's defaults; the retired `channel` intent is no option (FB-167 Q2), the rule is.
#[test]
fn the_schema_and_the_code_name_the_channel_fields() {
    let authored = ["ATTACK", "DEFEND", "FOCUS", "SUPPORT", "DISENGAGE"];
    for key in ["channel", "requires_meter", "hp_cost_pct"] {
        let f = field_of("skills", key);
        assert!(!json::b(&f, "rank", false), "{key}: not per rank");
        let users: Vec<&str> = skill::KINDS.iter().copied().filter(|k| skill::has_field(k, key)).collect();
        assert_eq!(users, authored, "{key}: the kinds that use it");
        let shown: Vec<String> = json::aget(json::dget(&f, "when"), "in").iter().map(json::gstr).collect();
        assert_eq!(shown, authored.map(String::from), "{key}: the schema's `when`");
    }
    let n = ruleset::Numbers::default();
    let default = |key: &str| json::get(&field_of(ruleset::KIND, key), "default").clone();
    assert_eq!(default("channel_meter"), json!(n.channel_meter));
    assert_eq!(default("channel_power"), json!(n.channel_power));
    assert_eq!(default("channel_cost"), json!(n.channel_cost));
    assert_eq!(default("channeled_actives"), json!(n.channeled_actives));
    assert_eq!(default("channel_rest"), json!(n.channel_rest));
    assert_eq!(default("channel_ai_edge"), json!(n.channel_ai_edge));
    let options = |kind: &str, key: &str| json::aget(&field_of(kind, key), "options").iter().map(json::gstr).collect::<Vec<_>>();
    assert!(!options(ruleset::KIND, "intents").contains(&ruleset::CHANNEL.to_string()));
    assert!(!options("skills", "round_intent").contains(&ruleset::CHANNEL.to_string()));
    assert!(options(ruleset::KIND, "rules").contains(&ruleset::CHANNEL.to_string()));
    assert_eq!(store::RULESET_CHANNEL_FIELDS.iter().filter(|k| ruleset::field_rule_of(k).is_some_and(|f| f.needs == [ruleset::CHANNEL])).count(), store::RULESET_CHANNEL_FIELDS.len(), "read only under the rule");
}

#[test]
fn the_valid_forms_check_clean() {
    let (_s, data) = scratch("valid");
    catalog(&data, timeline(json!({})));
    let v = store::view(&data);
    let ok: Vec<(&str, &str, Value)> = vec![
        ("skills", "jab", json!({})),
        ("skills", "jab", json!({"channel": {}})),
        ("skills", "jab", json!({"channel": {"cost": 0, "power": 1.5, "effects": [{"type": "inflict", "status": "bleed", "potency": 100, "rounds": 2}]}})),
        ("skills", "jab", json!({"channel": {"none": true}})),
        ("skills", "cleave", json!({"channel": {"none": true}})),
        ("skills", "jab", json!({"channel": {"only": true, "cost": 10}})),
        ("skills", "feed", json!({"requires_meter": {"meter": "strain", "min": 50}, "hp_cost_pct": 15})),
        ("skills", "feed", json!({"requires_meter": {}})),
        ("units", "drillman", json!({"deck": {"active": ["jab", "thrust"], "channeled": ["jab"]}})),
    ];
    for (kind, id, change) in ok {
        assert_eq!(errors_of(&v, kind, id, &change), Vec::<String>::new(), "{kind} {id} {change}");
    }
    let all = store::check(&data);
    assert!(!all.iter().any(|e| e.starts_with("rulesets:") || e.contains("unit_meters.json")), "the ruleset row's Channel fields; Feed (+5, channeled-only) as a meter skill: {all:?}");
}

#[test]
fn each_broken_rule_is_one_error() {
    let (_s, data) = scratch("broken");
    catalog(&data, timeline(json!({})));
    let v = store::view(&data);
    let cases: Vec<(&str, &str, Value, &str)> = vec![
        ("skills", "jab", json!({"channel": {"free": true}}), "Channel: unknown key 'free' (keys: none, only, cost, effects, power)."),
        ("skills", "jab", json!({"channel": {"none": 1}}), "Channel: none must be true or false."),
        ("skills", "jab", json!({"channel": {"none": true, "only": true}}), "Channel: none and only exclude each other."),
        ("skills", "jab", json!({"channel": {"none": true, "cost": 5}}), "Channel: none (no channeled version) takes no cost, power or effects."),
        ("skills", "jab", json!({"channel": {"cost": 101}}), "Channel: cost must be a whole number 0-100."),
        ("skills", "jab", json!({"channel": {"power": 0.5}}), "Channel: power must be 1-3."),
        ("skills", "jab", json!({"channel": {"effects": []}}), "Channel: effects holds 1-4 effects."),
        ("skills", "cleave", json!({"channel": {"only": true}}), "Channel: only a single-intent skill has a channeled version or is channeled-only (this one derives field and assault)."),
        ("skills", "cleave", json!({"channel": {"cost": 5}}), "only a single-intent skill"),
        ("skills", "feed", json!({"requires_meter": {"meter": "fury", "min": 1}}), "Meter gate: unknown unit meter 'fury'."),
        ("skills", "feed", json!({"requires_meter": {"meter": "strain"}}), "Meter gate: min is required (0-100)."),
        ("skills", "feed", json!({"requires_meter": {"meter": "strain", "min": 101}}), "Meter gate: min must be a whole number 0-100."),
        ("skills", "feed", json!({"requires_meter": {"meter": "strain", "min": 50, "max": 60}}), "Meter gate: unknown key 'max' (keys: meter, min)."),
        ("skills", "feed", json!({"hp_cost_pct": 100}), "HP cost % must be 0-99 (is 100)."),
        ("units", "drillman", json!({"deck": {"active": ["jab"], "channeled": ["thrust"]}}), "Deck channeled: 'thrust' is not an active of the deck."),
        ("units", "drillman", json!({"deck": {"active": ["jab", "cleave"], "channeled": ["cleave"]}}), "Deck channeled: 'cleave' has no channeled version"),
        ("units", "drillman", json!({"deck": {"active": ["jab", "thrust"], "channeled": ["jab", "thrust"]}}), "Deck: 2 channeled entries (the Timeline ruleset's channeled actives allow 1)."),
        ("units", "drillman", json!({"deck": {"active": ["jab"], "channeled": ["jab", "jab"]}}), "Deck channeled lists 'jab' twice."),
    ];
    let mut failures = Vec::new();
    for (kind, id, change, want) in &cases {
        let errors = errors_of(&v, kind, id, change);
        if !(errors.len() == 1 && errors[0].contains(want)) {
            failures.push(format!("{kind} {id} {change}: want {want:?}, got {errors:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// PR5-26 (audit B3): an automatic deck (no `active` list) marks the actives the channel ruleset's deck rule fills (the
/// battle's own list, `rpgkit_rules::unit::auto_actives`, truncated to the active slots): with two active slots a mark
/// on an automatic active checks clean, a mark on an owned family past the slots is one error.
#[test]
fn an_automatic_decks_marks_are_its_automatic_actives() {
    let (_s, data) = scratch("auto");
    catalog(&data, timeline(json!({"deck_slots": {"active": 2, "passive": 0, "keystone": 0}})));
    let v = store::view(&data);
    let rule = rpgkit_rules::DeckRule::of(&ruleset::Ruleset::from_row(v.get(ruleset::KIND, "timeline"))).unwrap().with_class(&v.typed, "drill");
    let auto = rpgkit_rules::unit::auto_actives(&v.typed, &rpgkit_data::UnitDef::from_obj(v.get("units", "drillman")), &rule);
    let owned = ["jab", "thrust", "cleave"];
    assert_eq!(auto.len(), 2, "truncated to the slots: {auto:?}");
    let inside = auto.iter().find(|f| *f != "cleave").expect("a single-intent automatic active");
    let past = owned.iter().find(|f| !auto.iter().any(|a| a == *f)).expect("an owned family past the slots");
    assert_eq!(errors_of(&v, "units", "drillman", &json!({"deck": {"channeled": [inside]}})), Vec::<String>::new(), "{inside}");
    let errors = errors_of(&v, "units", "drillman", &json!({"deck": {"channeled": [past]}}));
    assert!(errors.len() == 1 && errors[0].contains(&format!("Deck channeled: '{past}' is not among the automatic actives")), "{errors:?}");
}

/// A ruleset row's Channel numbers: `channel_cost` three whole numbers 0-100; the rule needs a `channel_meter`; the meter
/// and the Channel Rest name rows (`normal` keeps Rest); the rule needs `round_intent` and `unit_meters` (the register).
#[test]
fn the_ruleset_rows_channel_fields_are_checked() {
    let cases: Vec<(Value, &str)> = vec![
        (json!({"channel_cost": [15, 25]}), "Channel cost by AP must be three whole numbers 0-100 (for 1, 2 and 3+ AP)."),
        (json!({"channel_cost": [15, 25, 101]}), "Channel cost by AP must be three whole numbers 0-100"),
        (json!({"channel_meter": ""}), "Channel meter: channel needs a unit meter (else the rule does nothing)."),
        (json!({"channel_meter": "fury"}), "Channel meter: 'fury' has no row in unit_meters"),
        (json!({"channel_rest": "nap"}), "Channel Rest: unknown skill family 'nap'."),
        (json!({"rules": ["round_intent", "deck", "channel"]}), "channel needs unit_meters in Rules: without it the rule does nothing."),
    ];
    for (i, (change, want)) in cases.iter().enumerate() {
        let (_s, data) = scratch(&format!("row{i}"));
        catalog(&data, timeline(change.clone()));
        let all = store::check(&data);
        assert!(all.iter().any(|e| e.contains(want)), "{change}: want {want:?} in {all:?}");
    }
    let (_s, data) = scratch("normal");
    catalog(&data, timeline(json!({"channel_rest": "normal"})));
    assert!(!store::check(&data).iter().any(|e| e.contains("Channel Rest")));
}

/// The writer stores the Channel numbers only off their defaults (a row written before they existed re-saves byte for
/// byte) and keeps a set one.
#[test]
fn the_writer_stores_the_channel_numbers_only_when_set() {
    let row = |extra: Value| {
        let mut r = json!({"id": "timeline", "name": "Timeline", "rules": ["round_intent"]});
        for (k, v) in extra.as_object().unwrap() {
            r[k] = v.clone();
        }
        r.as_object().cloned().unwrap()
    };
    let stored = store::catalog_storage(ruleset::KIND, &row(json!({})), &[]).unwrap();
    for key in store::RULESET_CHANNEL_FIELDS {
        assert!(!stored.contains_key(key), "{key} at its default is no key");
    }
    let set = store::catalog_storage(ruleset::KIND, &row(json!({"channel_meter": "strain", "channel_cost": [10, 20, 30], "channel_rest": "feed", "channel_ai_edge": 1.5})), &[]).unwrap();
    assert_eq!((json::s(&set, "channel_meter"), json::get(&set, "channel_cost").clone(), json::s(&set, "channel_rest")), ("strain".to_string(), json!([10, 20, 30]), "feed".to_string()));
    assert_eq!(json::get(&set, "channel_ai_edge"), &json!(1.5), "PR5-20: a set AI edge is kept");
    assert!(!set.contains_key("channel_power") && !set.contains_key("channeled_actives"));
}
