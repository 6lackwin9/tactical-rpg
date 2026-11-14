//! The roster schema slice (ROSTER1-L slice 1, docs/specs/ROSTER1.md 11.1-11.3 and RK1): the world manifest block
//! `roster` (bunks, flags, crypts, rule rows, keepsakes), a party character's `keepsake`, a world encounter's
//! `party_min`, the `recruit` effect's `level`, the `roster.` read keys in conditions, and the typed reading of the
//! block. A roster world written to ROSTER1_TASKS 5.1's shape checks clean, and each field set wrong adds exactly one
//! error. Scratch data folders only.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::{roster_read_key, RosterManifest, RosterRead, RosterRule, KEEPSAKE_NAME_DEFAULT, SETTLE_CLASSIC, WOUNDED_PCT_DEFAULT};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const W: &str = "ward";

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_roster_schema_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds").join(W)).unwrap();
    (t, data)
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// A catalog file (items are read from version 2 on; with no family file each row is kept as stored).
fn write_catalog(data: &Path, kind: &str, entries: Value) {
    let version = if kind == "items" { 2 } else { 1 };
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(store::file_of(data, Some(W), kind), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

/// ROSTER1_TASKS 5.1's roster block (RQ1 answered `full`), with a fixture keepsake template.
fn roster() -> Value {
    json!({"bunks": 6, "bunks_flag": "hub_bunks_bonus", "bunks_max": 8, "crypts": 0, "open_flag": "roster_open",
        "rules": [
            {"acts": [0, 1], "hp_after": "full", "downed_won": "wounded", "downed_left": "wounded", "downed_lost": "wounded",
                "standing_lost": "wounded", "wounded_max_hp_pct": 20, "keepsakes_on": ["captured"]},
            {"acts": [2, 3], "hp_after": "full", "bleed_out_rounds": 3, "downed_won": "wounded", "downed_left": "captured",
                "downed_lost": "captured", "standing_lost": "wounded", "wounded_max_hp_pct": 20, "keepsakes_on": ["captured", "dead"]}],
        "keepsakes": {"on": ["captured", "dead"], "by_class": {"drill": "strap"}, "name": "{name}'s {item}"}})
}

fn manifest(roster: Value, party_max: i64) -> Value {
    json!({"id": W, "name": "Ward", "extends": [], "connects": [], "start_location": "hub", "party_max": party_max, "roster": roster, "version": 1})
}

fn kestrel() -> Value {
    json!({"id": "kestrel", "name": "Kestrel", "role": "party", "base_unit": "hero", "keepsake": "sigil"})
}

fn gauntlet() -> Value {
    json!({"id": "gauntlet", "name": "The Gauntlet", "party": ["kestrel"], "enemies": ["warden"], "party_min": 4})
}

fn smith() -> Value {
    json!({"id": "smith", "name": "Smith", "start": "start", "services": false, "nodes": [{"id": "start", "text": "Well?", "choices": [
        {"text": "Join us.", "to": "", "effects": [{"effect": "recruit", "character": "kestrel", "level": "average"}]},
        {"text": "Who is left?", "to": "", "requires": [{"key": "roster.kestrel", "value": "captured"}, {"key": "roster.bunks_free", "op": ">=", "value": 1}]},
        {"text": "Leave.", "to": ""}
    ]}]})
}

/// A scratch roster world: a class, a party unit, four keepsake candidates (one good, three broken), the manifest, a
/// location, the founder and an npc, the Gauntlet and the smith's dialogue.
fn roster_world(tag: &str, party_max: i64) -> PathBuf {
    let (scratch_dir, data) = scratch(tag);
    scratch_dir.until_thread_end();
    write_catalog(&data, "classes", json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2}}]));
    write_catalog(&data, "units", json!([{"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1}, {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1}]));
    let acc = |id: &str, t: &str, value: i64, special: bool| json!({"id": id, "name": id, "item_type": t, "slot": "ACCESSORY_1", "value": value, "special": special});
    write_catalog(&data, "items", json!([acc("sigil", "ACCESSORY", 0, true), acc("strap", "ACCESSORY", 0, true), acc("trinket", "ACCESSORY", 2, true), acc("charm", "ACCESSORY", 0, false), acc("blade", "WEAPON", 0, true)]));
    fs::write(aegis_content::world::manifest_path(&data, W), manifest(roster(), party_max).to_string()).unwrap();
    write_kind(&data, "locations", json!([{"id": "hub", "name": "Hub", "rest": true}]));
    write_kind(&data, "characters", json!([kestrel(), {"id": "innkeeper", "name": "Innkeeper", "role": "npc", "base_unit": "hero"}, {"id": "warden", "name": "Warden", "role": "enemy", "base_unit": "brute"}]));
    write_kind(&data, "encounters", json!([gauntlet()]));
    write_kind(&data, "dialogues", json!([smith()]));
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

#[test]
fn the_schema_carries_the_roster_fields() {
    let schema = source::schema();
    let field = |kind: &str, key: &str| fields(schema, "world", kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("world.{kind}.{key}"));
    let r = field("world", "roster");
    assert_eq!((json::s(&r, "type"), json::get(&r, "default")), ("json".to_string(), &json!({})), "{{}} is today's rule");
    let k = field("characters", "keepsake");
    assert_eq!((json::s(&k, "type"), json::s(&k, "ref"), json::get(&k, "default")), ("ref".to_string(), "base:items".to_string(), &json!("")));
    let p = field("encounters", "party_min");
    assert_eq!((json::s(&p, "type"), json::get(&p, "default")), ("int".to_string(), &json!(0)), "0 = any party");
    let src = source::load(&roster_world("schema", 4));
    assert_eq!(src.worlds[W]["manifest"]["roster"]["bunks"], json!(6));
    assert_eq!(src.worlds[W]["own"]["characters"][1]["keepsake"], json!(""), "the default fills in");
    assert_eq!(src.worlds[W]["own"]["encounters"][0]["party_min"], json!(4));
}

#[test]
fn the_section_5_1_roster_world_checks_clean() {
    let data = roster_world("clean", 4);
    let v = store::view(&data);
    assert_eq!(check::world::roster_errors(&v, W, v.worlds.manifest(W)), Vec::<String>::new());
    let wired = check::world::manifest_errors(&v, W);
    assert!(wired.is_empty(), "{wired:?}");
    for (kind, e) in [("characters", kestrel()), ("dialogues", smith())] {
        let found = errors_of(&v, kind, &e);
        assert!(found.is_empty(), "{kind}: {found:?}");
    }
    let found: Vec<String> = errors_of(&v, "encounters", &gauntlet()).into_iter().filter(|e| e.contains("Party min")).collect();
    assert!(found.is_empty(), "{found:?}");
    drop_scratch(&data);
}

#[test]
fn each_bad_roster_field_is_one_error() {
    let data = roster_world("manifest", 4);
    let v = store::view(&data);
    let m = manifest(roster(), 4);
    let errors = |m: &Value| check::world::roster_errors(&v, W, m.as_object().unwrap());
    let rule = |i: &str, key: &str, x: Value| with(&m, &["roster", "rules", i, key], x);
    let five: Vec<Value> = (0..5).map(|a| json!({"acts": [a]})).collect();
    let cases: Vec<(&str, Value, &str)> = vec![
        ("roster a list", with(&m, &["roster"], json!([6])), "Roster must be an object."),
        ("an unknown roster key", with(&m, &["roster", "beds"], json!(4)), "Roster: unknown key 'beds'."),
        ("bunks 13", with(&m, &["roster", "bunks"], json!(13)), "Bunks must be 0 (party max) or 1..12."),
        ("bunks -1", with(&m, &["roster", "bunks"], json!(-1)), "Bunks must be 0 (party max) or 1..12."),
        ("bunks_max below bunks", with(&m, &["roster", "bunks_max"], json!(5)), "bunks_max must be a whole number 6..12."),
        ("crypts 13", with(&m, &["roster", "crypts"], json!(13)), "crypts must be a whole number 0..12."),
        ("a reserved bunks flag", with(&m, &["roster", "bunks_flag"], json!("roster.bunks")), "is reserved"),
        ("an open flag not a flag key", with(&m, &["roster", "open_flag"], json!("Roster Open")), "open_flag 'Roster Open' must be a flag key"),
        ("five rule rows", with(&m, &["roster", "rules"], json!(five)), "rules hold at most 4 rows (has 5)."),
        ("two rows with no acts", with(&m, &["roster", "rules"], json!([{"acts": []}, {"acts": []}])), "only the last row may have no acts"),
        ("a row with no acts first", with(&m, &["roster", "rules", "0", "acts"], json!([])), "only the last row may have no acts"),
        ("act 10", rule("1", "acts", json!([10])), "acts must be a list of acts 0..9."),
        ("an unknown rule key", rule("0", "death", json!(true)), "Roster rule 1: unknown key 'death'."),
        ("bleed out 7", rule("1", "bleed_out_rounds", json!(7)), "bleed_out_rounds must be a whole number 0..6."),
        ("hp_after half", rule("0", "hp_after", json!("half")), "hp_after 'half' is not one of carry, full."),
        ("captured on a won field", rule("0", "downed_won", json!("captured")), "downed_won 'captured' is not one of classic, wounded."),
        ("standing dead", rule("0", "standing_lost", json!("dead")), "standing_lost 'dead' is not one of classic, wounded."),
        ("downed left exiled", rule("1", "downed_left", json!("exiled")), "downed_left 'exiled' is not one of classic, wounded, captured, dead."),
        ("downed lost exiled", rule("1", "downed_lost", json!("exiled")), "downed_lost 'exiled'"),
        ("wounded 0 percent", rule("0", "wounded_max_hp_pct", json!(0)), "wounded_max_hp_pct must be a whole number 1..90."),
        ("wounded 91 percent", rule("0", "wounded_max_hp_pct", json!(91)), "wounded_max_hp_pct must be a whole number 1..90."),
        ("a row keepsake on retired", rule("0", "keepsakes_on", json!(["retired"])), "keepsakes_on must be a list of captured, dead, lost."),
        ("keepsakes on dismissed", with(&m, &["roster", "keepsakes", "on"], json!(["dismissed"])), "Roster keepsakes: on must be a list of captured, dead, lost."),
        ("an unknown keepsakes key", with(&m, &["roster", "keepsakes", "tier"], json!(1)), "Roster keepsakes: unknown key 'tier'."),
        ("by_class an unknown class", with(&m, &["roster", "keepsakes", "by_class"], json!({"seer": "strap"})), "unknown class 'seer'"),
        ("by_class a weapon", with(&m, &["roster", "keepsakes", "by_class", "drill"], json!("blade")), "'blade' must be an accessory."),
        ("by_class an unknown item", with(&m, &["roster", "keepsakes", "by_class", "drill"], json!("ghost")), "unknown item 'ghost'"),
        ("a name token {class}", with(&m, &["roster", "keepsakes", "name"], json!("{name}'s {class}")), "name takes only the tokens {name} and {item}."),
        ("a name over 40", with(&m, &["roster", "keepsakes", "name"], json!("The treasured and much-loved {item} of {name}")), "name is longer than 40"),
    ];
    assert_eq!(errors(&m), Vec::<String>::new(), "the section 5.1 block");
    assert_eq!(errors(&manifest(json!({}), 4)), Vec::<String>::new(), "{{}} = today's rules");
    assert_eq!(errors(&manifest(json!({"rules": [{"acts": [0]}, {"acts": []}]}), 4)), Vec::<String>::new(), "a last row with no acts");
    for (tag, bad, needle) in cases {
        one(errors(&bad), needle, tag);
    }
    // bunks 0 is the party max: bunks_max may not go below it
    one(errors(&manifest(json!({"bunks": 0, "bunks_max": 3}), 4)), "bunks_max must be a whole number 4..12.", "bunks_max below the party max");
    drop_scratch(&data);
}

#[test]
fn a_keepsake_is_a_special_unsellable_accessory_on_a_party_character() {
    let data = roster_world("keepsake", 4);
    let v = store::view(&data);
    let k = kestrel();
    let cases: Vec<(&str, Value, &str)> = vec![
        ("not an accessory", with(&k, &["keepsake"], json!("blade")), "Keepsake: 'blade' must be an accessory."),
        ("value 2", with(&k, &["keepsake"], json!("trinket")), "Keepsake: 'trinket' must have value 0 (a keepsake is never sold)."),
        ("not special", with(&k, &["keepsake"], json!("charm")), "Keepsake: 'charm' must be a special item."),
        ("on an npc", with(&k, &["role"], json!("npc")), "Keepsake: only a party character carries one."),
        ("an unknown item", with(&k, &["keepsake"], json!("ghost")), "ghost"),
        ("an id that is a roster word", with(&k, &["id"], json!("living")), "Id 'living' is a roster read word"),
    ];
    for (tag, bad, needle) in cases {
        let found = added(&v, "characters", &k, &bad);
        // an npc may also trip the role rules of its party base unit; only that case keeps just the keepsake lines
        let found: Vec<String> = if tag == "on an npc" { found.into_iter().filter(|e| e.contains("Keepsake")).collect() } else { found };
        one(found, needle, tag);
    }
    assert!(errors_of(&v, "characters", &with(&k, &["keepsake"], json!(""))).is_empty(), "no keepsake");
    drop_scratch(&data);
}

#[test]
fn party_min_is_at_most_the_party_max() {
    let data = roster_world("party_min", 4);
    let v = store::view(&data);
    let c = gauntlet();
    // the schema range (0..4) clamps on normalize; the check adds the world's party max
    assert!(added(&v, "encounters", &c, &with(&c, &["party_min"], json!(4))).is_empty(), "4 of 4");
    assert!(added(&v, "encounters", &c, &with(&c, &["party_min"], json!(0))).is_empty(), "0 = any");
    drop_scratch(&data);
    let data = roster_world("party_min3", 3);
    let v = store::view(&data);
    one(added(&v, "encounters", &with(&c, &["party_min"], json!(3)), &c), "Party min must be a whole number 0..3 (the world's party max).", "4 of 3");
    drop_scratch(&data);
}

#[test]
fn the_recruit_level_and_the_roster_read_keys_are_checked() {
    let data = roster_world("dialogue", 4);
    let v = store::view(&data);
    let d = smith();
    let level = ["nodes", "0", "choices", "0", "effects", "0", "level"];
    let key = ["nodes", "0", "choices", "1", "requires", "0", "key"];
    for ok in [json!("highest"), json!("average"), json!(1), json!(20)] {
        assert!(added(&v, "dialogues", &d, &with(&d, &level, ok.clone())).is_empty(), "level {ok}");
    }
    for bad in [json!(0), json!(21), json!("lowest"), json!(2.5)] {
        one(added(&v, "dialogues", &d, &with(&d, &level, bad.clone())), "must be highest, average or a whole number 1..20.", &format!("level {bad}"));
    }
    for ok in ["roster.living", "roster.bunks", "roster.bunks_free", "roster.pool", "roster.open", "roster.departed.captured", "roster.departed.retired", "roster.hero", "roster.recruit_3", "roster.innkeeper"] {
        assert!(added(&v, "dialogues", &d, &with(&d, &key, json!(ok))).is_empty(), "{ok}");
    }
    one(added(&v, "dialogues", &d, &with(&d, &key, json!("roster.ghost"))), "'roster.ghost' names no member of this world", "an unknown member");
    for bad in ["roster.departed", "roster.departed.exiled", "roster.kestrel.hp", "roster."] {
        one(added(&v, "dialogues", &d, &with(&d, &key, json!(bad))), "is not a roster read key", bad);
    }
    drop_scratch(&data);
}

#[test]
fn the_roster_block_reads_with_its_defaults() {
    let m = RosterManifest::from_obj(&Dict::new());
    assert_eq!((m.bunks, m.bunks_flag.as_str(), m.bunks_max, m.crypts, m.open_flag.as_str()), (0, "", 12, 0, ""));
    assert!(m.rules.is_empty() && m.keepsakes_on.is_empty() && m.keepsakes_by_class.is_empty());
    assert_eq!(m.keepsake_name, KEEPSAKE_NAME_DEFAULT);
    assert_eq!(m.rule(0), None, "no row = classic (L6)");
    assert_eq!(m.bunk_cap(4, 0), 4, "bunks 0 = the party max (BK1)");
    let row = RosterRule::from_obj(&Dict::new());
    assert_eq!((row.hp_after.as_str(), row.downed_won.as_str(), row.downed_left.as_str(), row.downed_lost.as_str(), row.standing_lost.as_str()), ("carry", SETTLE_CLASSIC, SETTLE_CLASSIC, SETTLE_CLASSIC, SETTLE_CLASSIC));
    assert_eq!((row.bleed_out_rounds, row.wounded_max_hp_pct, row.keepsakes_on.clone()), (0, WOUNDED_PCT_DEFAULT, None));
    let m = RosterManifest::from_obj(&dict(roster()));
    assert_eq!(m.rule(1).map(|r| r.acts.clone()), Some(vec![0, 1]));
    assert_eq!(m.rule(3).map(|r| r.downed_left.as_str()), Some("captured"));
    assert_eq!(m.rule(5), None, "no row covers act 5");
    assert_eq!(m.keepsake_lives(0), ["captured".to_string()], "the row's keepsakes_on overrides");
    assert_eq!(m.keepsake_lives(5), ["captured".to_string(), "dead".to_string()], "no row: the manifest's list");
    assert_eq!(m.keepsakes_by_class.get("drill").map(String::as_str), Some("strap"));
    assert_eq!((m.bunk_cap(4, 0), m.bunk_cap(4, 2), m.bunk_cap(4, 5), m.bunk_cap(4, -3)), (6, 8, 8, 6), "flag adds, capped at bunks_max, clamped at 0");
    let open = RosterManifest::from_obj(&dict(json!({"rules": [{"acts": [2]}, {"acts": [], "downed_lost": "wounded"}]})));
    assert_eq!(open.rule(2).map(|r| r.downed_lost.as_str()), Some(SETTLE_CLASSIC), "the first covering row wins");
    assert_eq!(open.rule(7).map(|r| r.downed_lost.as_str()), Some("wounded"), "the open row covers the rest");
}

#[test]
fn roster_read_keys_parse() {
    assert_eq!(roster_read_key("roster.living"), Some(RosterRead::Living));
    assert_eq!(roster_read_key("roster.bunks_free"), Some(RosterRead::BunksFree));
    assert_eq!(roster_read_key("roster.departed.dead"), Some(RosterRead::Departed("dead")));
    assert_eq!(roster_read_key("roster.kestrel"), Some(RosterRead::Member("kestrel")));
    for bad in ["roster.departed", "roster.departed.gone", "roster.", "roster.Kestrel", "roster.a.b", "meter.alert", "living"] {
        assert_eq!(roster_read_key(bad), None, "{bad}");
    }
}

#[test]
fn the_writer_stores_the_roster_fields() {
    let data = roster_world("writer", 4);
    // an update that does not set roster keeps it out of a manifest that had none (WORLDW1)
    fs::create_dir_all(data.join("worlds").join("plain")).unwrap();
    fs::write(aegis_content::world::manifest_path(&data, "plain"), json!({"id": "plain", "name": "Plain", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    store::set_world(&data, "plain", &dict(json!({"tagline": "Flat."})), false).expect("an update");
    let text = fs::read_to_string(aegis_content::world::manifest_path(&data, "plain")).unwrap();
    assert!(!text.contains("\"roster\""), "roster stays out while it holds its default: {text}");
    store::set_world(&data, "plain", &dict(json!({"roster": {"bunks": 6}})), false).expect("a roster block");
    assert_eq!(source::load(&data).worlds["plain"]["manifest"]["roster"], json!({"bunks": 6}));
    let refused = store::set_world(&data, "plain", &dict(json!({"roster": {"bunks": 20}})), false).err().expect("bunks 20 refused");
    assert!(refused.iter().any(|e| e.contains("Bunks must be 0 (party max) or 1..12.")), "{refused:?}");
    // a character keepsake through upsert (a rule problem is the content check's, as every entry rule: only shape
    // problems block a world entry's write)
    let file = store::file_of(&data, Some(W), "characters");
    store::upsert(&data, Some(W), "characters", &dict(kestrel()), None).expect("a founder with a keepsake");
    let stored = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, Some(W), "characters").unwrap(), stored, "a re-save without changes is byte-identical");
    assert!(source::load(&data).worlds[W]["own"]["characters"].as_array().unwrap().iter().any(|c| c["keepsake"] == "sigil"));
    let keepsake_problems = |data: &Path| store::check(data).into_iter().filter(|p| p.contains("Keepsake")).collect::<Vec<_>>();
    assert_eq!(keepsake_problems(&data), Vec::<String>::new());
    store::upsert(&data, Some(W), "characters", &dict(with(&kestrel(), &["keepsake"], json!("trinket"))), None).expect("stored");
    let found = keepsake_problems(&data);
    assert!(found.len() == 1 && found[0].contains("ward/characters:kestrel") && found[0].contains("must have value 0"), "{found:?}");
    drop_scratch(&data);
}

#[test]
fn a_party_unit_id_is_never_a_roster_word() {
    // RK1 (the slice 1 review): a start-party member's id is its catalog unit's, so a party unit named as a roster
    // word is one error; an enemy unit may use the word
    let data = roster_world("unit_word", 4);
    let unit = |id: &str, side: &str| json!({"id": id, "name": id, "side": side, "class_id": "drill", "level": 1});
    write_catalog(&data, "units", json!([unit("hero", "party"), unit("brute", "enemy"), unit("living", "party"), unit("pool", "enemy")]));
    let found: Vec<String> = store::check(&data).into_iter().filter(|p| p.contains("roster read word")).collect();
    one(found, "Id 'living' is a roster read word", "a party unit");
    drop_scratch(&data);
}
