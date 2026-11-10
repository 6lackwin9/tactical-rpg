//! Ruleset rows and the world field (RULESET1-R, docs/specs/RULESET1_TASKS.md 1.1-1.2): the schema lists what the code
//! owns (the rule register, the intents) and offers the ruleset rows for the ids (CLN-44: open ids, classic a row too);
//! a valid row loads and reads as its numbers, a row without them reads the schema defaults; every rule of 1.2 broken
//! alone is one load error naming its field; the writer takes the kind; a world naming a ruleset without its row, or a
//! bad flag, is a content error. Scratch data folders only.

use aegis_content::schema::{fields, normalize};
use aegis_core::json::{self, gstr, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_rules::ruleset::{self, Numbers, Ruleset, INTENTS, REGISTER};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

/// A scratch data folder `<tmp>/<tag>/data` with empty catalog and worlds folders.
fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_rulesets_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
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
        "rules": ["round_intent", "enemy_moves", "deck", "poise", "ap_cost", "pierce", "reach", "flank", "plan_badges", "field_intent",
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

/// A field's type and `options_from`.
fn field(section: &str, kind: &str, key: &str) -> (String, String) {
    let f = fields(source::schema(), section, kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("{section}.{kind}.{key}"));
    (json::s(f, "type"), json::s(f, "options_from"))
}

fn strs(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn the_schema_lists_what_the_code_owns() {
    // CLN-44: no closed id list: the fields offer the ruleset rows (a world's also "" = inherited)
    assert_eq!(field("content", "derived_stats", "rulesets"), ("strings".into(), "rulesets".into()), "a derived row's rulesets");
    assert_eq!(field("content", "portrait_cues", "rulesets"), ("strings".into(), "rulesets".into()), "a portrait cue's rulesets");
    assert_eq!(field("world", "world", "ruleset"), ("enum".into(), "rulesets_inherit".into()), "a world's ruleset: empty (inherited) or a row id");
    assert_eq!(field("content", ruleset::KIND, "id"), ("id".into(), String::new()), "row ids are open");
    for (section, kind, key) in [("content", "derived_stats", "rulesets"), ("content", "portrait_cues", "rulesets"), ("world", "world", "ruleset"), ("content", ruleset::KIND, "id")] {
        assert_eq!(options(section, kind, key), Vec::<String>::new(), "{kind}.{key} lists no ids");
    }
    assert_eq!(options("content", ruleset::KIND, "rules"), REGISTER.iter().map(|r| r.id.to_string()).collect::<Vec<_>>(), "the rule register");
    assert_eq!(options("content", ruleset::KIND, "intents"), strs(&INTENTS));
}

/// ATTACKLOCK1: the schema lists the code's `attack_lock` values with `none` as the default; a normalized row reads them
/// and a row that does not name the field stays at none, as the built-in classic values do. Which attacks each value
/// locks: `locks` (melee bands under melee, every attack under all, none under none).
#[test]
fn the_attack_lock_option_reads_from_the_row() {
    use rpgkit_rules::ruleset::AttackLock;
    assert_eq!(options("content", ruleset::KIND, "attack_lock"), strs(&AttackLock::IDS));
    let row = |extra: Value| {
        let mut r = dict(good_row());
        for (k, v) in extra.as_object().unwrap() {
            r.insert(k.clone(), v.clone());
        }
        normalize(fields(source::schema(), "content", ruleset::KIND), &r, false)
    };
    assert_eq!(json::s(&row(json!({})), "attack_lock"), "none", "the schema's default");
    assert_eq!(Ruleset::from_row(&row(json!({}))).attack_lock(), AttackLock::None);
    assert_eq!(Ruleset::classic().attack_lock(), AttackLock::None);
    for id in AttackLock::IDS {
        let lock = AttackLock::parse(id).unwrap();
        assert_eq!(lock.id(), id);
        assert_eq!(Ruleset::from_row(&row(json!({"attack_lock": id}))).attack_lock(), lock);
    }
    assert_eq!([AttackLock::None.locks(true), AttackLock::None.locks(false)], [false, false]);
    assert_eq!([AttackLock::Melee.locks(true), AttackLock::Melee.locks(false)], [true, false]);
    assert_eq!([AttackLock::All.locks(true), AttackLock::All.locks(false)], [true, true]);
}

/// INPUT1 1.7 (MR9, MR10): the schema lists the code's `melee_diagonal` and `flank_protected_cost` values; a row reads
/// them (classic: none and full); CLN-45: `maneuver` reads as set without `round_intent` too (no warning); the cost
/// pays as MR9 says (4 AP: full 4, half 2, fixed 1, none 0); the soft problem is a warning.
#[test]
fn diagonal_melee_and_the_protected_flank_cost_read_and_warn() {
    use rpgkit_rules::ruleset::{FlankCost, MeleeDiagonal};
    assert_eq!(options("content", ruleset::KIND, "melee_diagonal"), strs(&MeleeDiagonal::IDS));
    assert_eq!(options("content", ruleset::KIND, "flank_protected_cost"), strs(&FlankCost::IDS));
    let row = |extra: Value| {
        let mut r = dict(good_row());
        for (k, v) in extra.as_object().unwrap() {
            r.insert(k.clone(), v.clone());
        }
        normalize(fields(source::schema(), "content", ruleset::KIND), &r, false)
    };
    let r = Ruleset::from_row(&row(json!({})));
    assert_eq!((r.numbers.melee_diagonal, r.numbers.flank_protected), (MeleeDiagonal::Maneuver, FlankCost::Full), "the defaults");
    assert_eq!(r.melee_diagonal(), MeleeDiagonal::Maneuver);
    let c = Ruleset::classic();
    assert_eq!((c.melee_diagonal(), c.numbers.flank_protected), (MeleeDiagonal::None, FlankCost::Full), "the built-in classic values");
    let r = Ruleset::from_row(&row(json!({"melee_diagonal": "all", "flank_protected_cost": "fixed", "flank_protected_ap": 2})));
    assert_eq!((r.melee_diagonal(), r.numbers.flank_protected), (MeleeDiagonal::All, FlankCost::Fixed(2)));
    let no_intent = Ruleset::from_row(&row(json!({"rules": ["poise"]})));
    assert_eq!(no_intent.melee_diagonal(), MeleeDiagonal::Maneuver, "CLN-45: maneuver without round_intent reads as set");
    let pays: Vec<i32> = [FlankCost::Full, FlankCost::Half, FlankCost::Fixed(FlankCost::FIXED_AP), FlankCost::None].iter().map(|f| f.pays(4)).collect();
    assert_eq!(pays, [4, 2, 1, 0]);
    assert_eq!(check::ruleset::warnings(&row(json!({}))), Vec::<String>::new());
    assert_eq!(check::ruleset::warnings(&row(json!({"rules": ["poise"]}))), Vec::<String>::new(), "CLN-45: no coupling, no warning");
    assert_eq!(check::ruleset::warnings(&row(json!({"flank_protected_ap": 3}))), ["flank_protected_ap is read only with fixed"]);
    assert_eq!(check::ruleset::warnings(&row(json!({"flank_protected_cost": "fixed", "flank_protected_ap": 3}))), Vec::<String>::new());
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
    let (_scratch, data) = scratch("valid");
    base_catalog(&data);
    write_kind(&data, ruleset::KIND, 1, json!([good_row()]));
    let src = source::load(&data);
    assert_eq!(ruleset_errors(&src), Vec::<String>::new());
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    let r = Ruleset::of(&cat, "timeline").expect("the row");
    // (every rule but PK1-S2a's channel, which needs a unit meter row: tests/channel_checks.rs checks its row)
    assert_eq!(r.rules, REGISTER.iter().map(|x| x.id).filter(|id| *id != ruleset::CHANNEL).collect::<Vec<_>>(), "every rule, in register order");
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
    let (_scratch, data) = scratch("missing");
    let src = source::load(&data);
    assert_eq!(ruleset_errors(&src), Vec::<String>::new());
    assert_eq!(src.catalog[ruleset::KIND], json!([]));
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    assert!(Ruleset::of(&cat, "timeline").is_err());
    assert_eq!(Ruleset::of_or_classic(&cat, "timeline"), Ruleset::classic());
    assert!(Ruleset::of(&cat, "chess").unwrap_err().contains("ruleset 'chess' has no row in rulesets"));
    // CLN-44: classic without its row: the built-in values (with a load note), not a refusal
    assert_eq!(Ruleset::of(&cat, "classic").unwrap(), Ruleset::classic());
    drop_scratch(&data);
}

#[test]
fn each_broken_rule_is_one_error_naming_its_field() {
    let cases: [(&str, Value, &str); 11] = [
        ("id", json!("Classic Rules"), "ID 'Classic Rules' must be lowercase snake_case"),
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
    let (_scratch, data) = scratch("rules");
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
    // A4.2-K1b: the legacy single intent (support) without field_intent; under it mend's derived rows are Support and
    // Stand (`guard`, listed), so it passes (below)
    let derived = mend.clone();
    mend["rules"] = json!(mend["rules"].as_array().unwrap().iter().filter(|r| *r != ruleset::FIELD_INTENT).cloned().collect::<Vec<_>>());
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
    write_kind(&data, ruleset::KIND, 1, json!([derived]));
    assert_eq!(ruleset_errors(&source::load(&data)), Vec::<String>::new(), "field_intent: a basic fits by any derived row");
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
    let (_scratch, data) = scratch("ranges");
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
    // RULESET1_TASKS 1.2's ten numbers, INPUT1 1.7's `flank_protected_ap`, A1.2-S1's `item_ap`, P2.2-S1's `move_after_blocks`
    // PK1-S2a's `channel_power` and `marrow_actives`, and PR5-20's `channel_ai_edge`
    assert_eq!(want.len(), 16, "every number of 1.2 (and INPUT1 1.7, A1.2-S1, P2.2-S1, PK1-S2a, PR5-20) has its range in the schema");
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
    let (_scratch, data) = scratch("writer");
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
    let unknown = up(json!({"id": "Free Form", "name": "Freeform"})).err().expect("the write is refused");
    assert!(unknown.iter().any(|e| e.contains("'Free Form' must be lowercase snake_case")), "{unknown:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), text, "a refused write puts the file back");
    // CLN-44: the ids are open: a third ruleset is data
    up(json!({"id": "freeform", "name": "Freeform"})).unwrap();
    let cat = rpgkit_data::Catalog::from_value(&source::load(&data).catalog);
    assert_eq!(Ruleset::ids(&cat), strs(&["timeline", "freeform"]));
    assert_eq!(Ruleset::of(&cat, "freeform").map(|r| (r.id, r.rules.len())), Ok(("freeform".to_string(), 0)));
    drop_scratch(&data);
}

fn world_problems(data: &Path, world: &str) -> Vec<String> {
    let v = store::view(data);
    check::world::manifest_errors(&v, world)
}

#[test]
fn a_world_names_a_known_ruleset_with_its_row_and_a_flag_key() {
    let (_scratch, data) = scratch("worlds");
    base_catalog(&data);
    write_world(&data, "vale", json!({"id": "vale", "name": "Vale", "ruleset": "timeline"}));
    write_world(&data, "glen", json!({"id": "glen", "name": "Glen", "extends": ["vale"]}));
    write_world(&data, "moor", json!({"id": "moor", "name": "Moor", "ruleset": "chess", "plan_reveal_flag": "Moor Spies"}));
    write_world(&data, "fen", json!({"id": "fen", "name": "Fen", "ruleset": "classic", "plan_reveal_flag": "fen.scouted"}));
    let need = "Ruleset 'timeline' needs its row in rulesets (data/catalog/rulesets.json).";
    assert_eq!(world_problems(&data, "vale"), [need], "a timeline world without the row");
    assert_eq!(world_problems(&data, "glen"), Vec::<String>::new(), "the world that inherits is fixed with its parent");
    // CLN-44: classic is a row too (the content check against the rows; battles fall back with a load note)
    assert_eq!(world_problems(&data, "fen"), ["Ruleset 'classic' needs its row in rulesets (data/catalog/rulesets.json)."], "classic without its row; a dotted flag key");
    assert_eq!(
        world_problems(&data, "moor"),
        ["Ruleset 'chess' needs its row in rulesets (data/catalog/rulesets.json).", "Plan reveal flag 'Moor Spies' must be a flag key (a-z, 0-9, _ and .)."]
    );
    write_kind(&data, ruleset::KIND, 1, json!([good_row(), {"id": "classic", "name": "Classic", "melee_diagonal": "none"}]));
    assert_eq!(world_problems(&data, "vale"), Vec::<String>::new(), "with the row");
    assert_eq!(world_problems(&data, "fen"), Vec::<String>::new(), "with the classic row");
    drop_scratch(&data);
}

/// CLN-44: classic is a row like any other: without it the content check warns (its battles play the built-in classic
/// values); a row with classic's values reads as them; a derived stat or portrait cue naming a ruleset without its row is
/// a content error, and with the row it is clean.
#[test]
fn classic_is_a_row_and_the_rulesets_lists_name_rows() {
    let (_scratch, data) = scratch("classic_row");
    base_catalog(&data);
    write_kind(&data, ruleset::KIND, 1, json!([good_row()]));
    write_kind(&data, "derived_stats", 1, json!([{"id": "max_poise", "name": "Max Poise", "formula": "CONS * 2", "rulesets": ["timeline", "hardcore"]}]));
    let v = store::view(&data);
    let missing = check::warnings_all(&v).get("rulesets:missing").cloned().unwrap_or_default();
    assert_eq!(missing, [ruleset::fallback_note(ruleset::CLASSIC)], "no classic row: a warning");
    let problems = check::validate_all(&v).get("derived_stats:max_poise").cloned().unwrap_or_default();
    assert_eq!(problems, ["Rulesets: 'hardcore' has no row in rulesets (data/catalog/rulesets.json)."]);
    let classic = json!({"id": "classic", "name": "Classic", "rules": [], "melee_diagonal": "none"});
    let hardcore = json!({"id": "hardcore", "name": "Hardcore", "melee_diagonal": "none"});
    write_kind(&data, ruleset::KIND, 1, json!([good_row(), classic, hardcore]));
    let src = source::load(&data);
    assert_eq!(ruleset_errors(&src), Vec::<String>::new());
    let v = store::view(&data);
    assert!(!check::warnings_all(&v).contains_key("rulesets:missing"), "with the classic row");
    assert!(!check::validate_all(&v).contains_key("derived_stats:max_poise"), "every listed ruleset has its row");
    let cat = rpgkit_data::Catalog::from_value(&src.catalog);
    assert_eq!(Ruleset::of(&cat, "").unwrap(), Ruleset::classic(), "a classic row with the built-in values reads as them");
    assert_eq!(Ruleset::of_or_classic(&cat, "chess"), Ruleset::classic(), "a missing row: the built-in classic values");
    drop_scratch(&data);
}

/// CLN-45 (decision "Godot parity retired and cleanup pass 3 answers" Q-B2): the switches the round intent rule used to
/// bundle are row fields. The schema lists "" (by the rules) and the code's values; empty, a row reads the round intent
/// rule's choices when it lists `round_intent` (Rest pays at round end, every skill chases, root stops a chase, no
/// Switch Target step, no stances) and the legacy ones otherwise (the built-in classic values); a set value reads as set
/// whatever the rules; a value not in the list is one load error naming its field; the writer leaves an empty switch out
/// (a row written before the fields re-saves byte for byte) and stores a set one.
#[test]
fn the_switches_read_by_the_rules_or_as_set() {
    use rpgkit_rules::ruleset::{by_rules_values, Chase, RestAp, BY_RULES_FIELDS};
    for key in BY_RULES_FIELDS {
        let mut want = vec![String::new()];
        want.extend(strs(by_rules_values(key)));
        assert_eq!(options("content", ruleset::KIND, key), want, "{key}");
    }
    let read = |v: Value| Ruleset::from_row(&normalize(fields(source::schema(), "content", ruleset::KIND), &dict(v), false));
    let switches = |r: &Ruleset| (r.rest_ap(), r.chase(), r.root_stops_chase(), r.switch_target_step(), r.stances());
    let intent_choices = (RestAp::RoundEnd, Chase::EverySkill, true, false, false);
    let legacy = (RestAp::OnUse, Chase::Classic, false, true, true);
    assert_eq!(switches(&read(good_row())), intent_choices, "empty, with round_intent");
    let plain = read(json!({"id": "classic", "name": "Classic", "rules": ["poise", "flank"]}));
    assert_eq!(switches(&plain), legacy, "empty, without round_intent");
    assert_eq!(switches(&Ruleset::classic()), legacy, "the built-in classic values");
    assert_eq!(switches(&Ruleset::from_row(&dict(json!({"rules": ["round_intent"]})))), intent_choices, "a raw row without the keys");
    let set = |mut row: Value, values: [&str; 5]| {
        for (k, v) in BY_RULES_FIELDS.iter().zip(values) {
            row[*k] = json!(v);
        }
        row
    };
    let flipped = read(set(good_row(), ["on_use", "classic", "off", "on", "on"]));
    assert_eq!(switches(&flipped), legacy, "set, with round_intent");
    assert!(flipped.has(ruleset::ROUND_INTENT));
    let flipped = read(set(json!({"id": "classic", "name": "Classic"}), ["round_end", "every_skill", "on", "off", "off"]));
    assert_eq!(switches(&flipped), intent_choices, "set, without round_intent");
    assert!(!flipped.has(ruleset::ROUND_INTENT));

    let (_scratch, data) = scratch("switches");
    base_catalog(&data);
    for key in BY_RULES_FIELDS {
        let mut row = good_row();
        row[key] = json!("maybe");
        write_kind(&data, ruleset::KIND, 1, json!([row]));
        let errors = ruleset_errors(&source::load(&data));
        assert_eq!(errors.len(), 1, "{key}: {errors:?}");
        assert!(errors[0].ends_with(&format!("{key}: 'maybe' is not one of {} (or empty: by the rules).", by_rules_values(key).join(", "))), "{errors:?}");
    }
    let _ = fs::remove_file(data.join("catalog").join("rulesets.json"));
    let file = data.join("catalog").join("rulesets.json");
    let stored = || -> Value { serde_json::from_str::<Value>(&fs::read_to_string(&file).unwrap()).unwrap()["entries"][0].clone() };
    store::upsert(&data, None, ruleset::KIND, &dict(good_row()), None).unwrap();
    assert!(BY_RULES_FIELDS.iter().all(|k| stored().get(*k).is_none()), "empty switches are left out: {}", stored());
    store::upsert(&data, None, ruleset::KIND, &dict(set(good_row(), ["round_end", "every_skill", "on", "off", "off"])), None).unwrap();
    let row = stored();
    assert_eq!(BY_RULES_FIELDS.map(|k| row[k].clone()), [json!("round_end"), json!("every_skill"), json!("on"), json!("off"), json!("off")], "set switches are stored");
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, ruleset::KIND).unwrap(), text, "a re-save without changes is byte-identical");
    drop_scratch(&data);
}

/// BASERULES1-M: the register's metadata in the row checks. A rule that would be silently inert (flank without
/// round_intent) is one load error and the row is dropped; with what it requires it loads; a conflict, a field set
/// without its rule and a split bundle are warnings only, beside the row's own (`check::ruleset::warnings`).
#[test]
fn the_register_metadata_refuses_an_inert_rule_and_warns_on_the_rest() {
    let (_scratch, data) = scratch("metadata");
    base_catalog(&data);
    let lone = json!({"id": "loose", "name": "Loose", "rules": ["flank"]});
    write_kind(&data, ruleset::KIND, 1, json!([lone]));
    let src = source::load(&data);
    let errors = ruleset_errors(&src);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(errors[0].ends_with("flank needs round_intent in Rules: without it the rule does nothing."), "{errors:?}");
    assert!(src.catalog[ruleset::KIND].as_array().is_none_or(|rows| rows.is_empty()), "the row is dropped");
    write_kind(&data, ruleset::KIND, 1, json!([{"id": "loose", "name": "Loose", "rules": ["round_intent", "flank"]}]));
    assert_eq!(ruleset_errors(&source::load(&data)), Vec::<String>::new(), "with what it requires");
    drop_scratch(&data);

    let norm = |v: Value| normalize(fields(source::schema(), "content", ruleset::KIND), &dict(v), false);
    assert_eq!(check::ruleset::warnings(&norm(good_row())), Vec::<String>::new(), "a complete row");
    let mut conflict = good_row();
    conflict["stances"] = json!("on");
    assert_eq!(check::ruleset::warnings(&norm(conflict)), ["round_intent conflicts with stances=on: it runs, but the two cut against each other."]);
    let plain = norm(json!({"id": "plain", "name": "Plain", "rules": [], "stagger_windup_mult": 3.0}));
    assert_eq!(check::ruleset::warnings(&plain), ["stagger_windup_mult is set but read only with poise."]);
    let split = norm(json!({"id": "split", "name": "Split", "rules": ["round_intent"], "chase": "classic", "flank_protected_ap": 3}));
    assert_eq!(check::ruleset::warnings(&split), ["flank_protected_ap is read only with fixed", "Bundle intent_core ships together: it needs chase=every_skill."]);
}

/// S2.2-S2: the row-only `poise_mode` loads as set (`pips`), absent reads `bar`, and another id is one error naming its
/// values.
#[test]
fn the_row_only_poise_mode_loads_and_refuses_an_unknown_id() {
    let (_scratch, data) = scratch("poise_mode");
    base_catalog(&data);
    for (mode, want) in [(Some("pips"), Some(ruleset::PoiseMode::Pips)), (None, Some(ruleset::PoiseMode::Bar)), (Some("pip"), None)] {
        let mut row = good_row();
        if let Some(m) = mode {
            row["poise_mode"] = json!(m);
        }
        write_kind(&data, ruleset::KIND, 1, json!([row]));
        let src = source::load(&data);
        let errors = ruleset_errors(&src);
        match want {
            Some(m) => {
                assert_eq!(errors, Vec::<String>::new(), "{mode:?}");
                let loaded = src.catalog[ruleset::KIND][0].as_object().expect("the row loads");
                assert_eq!(Ruleset::from_row(loaded).numbers.poise_mode, m, "{mode:?}");
            }
            None => {
                assert_eq!(errors.len(), 1, "{errors:?}");
                assert!(errors[0].ends_with("poise_mode: 'pip' is not one of bar, pips."), "{errors:?}");
            }
        }
    }
    drop_scratch(&data);
}
