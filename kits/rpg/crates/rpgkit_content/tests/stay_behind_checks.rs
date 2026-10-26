//! ENCOUNTER2-H content side (docs/specs/ENCOUNTER2.md section 6): the schema carries `unkillable`, `timeout_outcome`
//! and `stay_behind` in both encounter sections with their defaults; the checks give one error per broken rule of the
//! stay-behind block (escape only, known keys, mode, 1-2 hold tiles that are walkable, on the board and no exit, a stand
//! with none, the label, the per-mode keys, the stand numbers and statuses, the lines' one token, `flag` and `then` for
//! world encounters only) and per `unkillable` ref outside `enemies` and the phase waves; a row without the fields
//! checks as before; the writer stores and reloads them. Scratch data folders only.

use aegis_content::schema::{fields, normalize};
use aegis_core::json::{self, Dict};
use rpgkit_battle::session::EncounterDef;
use rpgkit_content::{check, source, store};
use rpgkit_data::Catalog;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_stay_behind_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn write_kind(data: &Path, kind: &str, version: i64, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// A floor board with a wall at (3, 3); the exits sit at (11, 3..5).
fn rows() -> Vec<String> {
    let mut r = vec![".".repeat(12); 9];
    r[3] = "...#........".into();
    r
}

/// The hold escape `winch` (a hero against a brute, an imp arriving as a phase wave).
fn winch() -> Value {
    json!({"id": "winch", "name": "Winch", "zone": "stone_arena", "rows": rows(), "party": ["hero"], "enemies": ["brute"],
        "exits": [[11, 3], [11, 4], [11, 5]], "objective": "escape", "objective_count": 1, "max_rounds": 6,
        "phases": [{"id": "late", "name": "Late", "trigger": {"kind": "round", "round": 2}, "effects": [{"type": "wave", "units": ["imp"]}]}],
        "unkillable": ["brute"], "timeout_outcome": "lost",
        "stay_behind": {"mode": "hold", "tiles": [[10, 1]], "tiles_label": "the winch", "fallback": "any", "pick_default": "highest_hp"}})
}

fn catalog(data: &Path) {
    let skill = |id: &str, kind: &str, extra: Value| {
        let mut e = json!({"id": id, "name": id, "kind": kind, "ranks": []});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        e
    };
    write_kind(data, "skills", 2, json!([skill("move", "MOVE", json!({"system": true})), skill("rest", "REST", json!({"system": true})), skill("strike", "ATTACK", json!({"basic": true}))]));
    write_kind(data, "classes", 1, json!([{"id": "drill", "name": "Drill", "base_stats": {"STR": 13, "DEX": 10, "INT": 6, "CONS": 13, "PERC": 10, "KNOW": 10}, "growth": {"STR": 2}}]));
    write_kind(
        data,
        "units",
        1,
        json!([
            {"id": "hero", "name": "Hero", "side": "party", "class_id": "drill", "level": 1},
            {"id": "brute", "name": "Brute", "side": "enemy", "class_id": "drill", "level": 1},
            {"id": "imp", "name": "Imp", "side": "enemy", "class_id": "drill", "level": 1},
            {"id": "ogre", "name": "Ogre", "side": "enemy", "class_id": "drill", "level": 1}
        ]),
    );
    write_kind(data, "encounters", 1, json!([winch()]));
}

/// The errors of `winch` with `edit` applied (a key set to null is removed).
fn errors_with(v: &check::View, edit: Value) -> Vec<String> {
    let mut e = v.get("encounters", "winch").clone();
    for (k, val) in edit.as_object().unwrap() {
        if val.is_null() {
            e.shift_remove(k);
        } else {
            e.insert(k.clone(), val.clone());
        }
    }
    check::validate_entry(v, "encounters", &e, "winch")
}

/// The errors of `winch` with `stay` merged over its stay-behind block (a key set to null is removed).
fn stay_errors(v: &check::View, stay: Value) -> Vec<String> {
    let mut sb = v.get("encounters", "winch").get("stay_behind").cloned().unwrap();
    for (k, val) in stay.as_object().unwrap() {
        if val.is_null() {
            sb.as_object_mut().unwrap().shift_remove(k);
        } else {
            sb[k] = val.clone();
        }
    }
    errors_with(v, json!({"stay_behind": sb}))
}

#[test]
fn the_schema_carries_the_fields_in_both_encounter_sections() {
    for section in ["content", "world"] {
        let row = |key: &str| fields(source::schema(), section, "encounters").iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("{section}.encounters.{key}"));
        assert_eq!((json::s(&row("unkillable"), "type"), row("unkillable").get("default").cloned()), ("refs".into(), Some(json!([]))), "{section}");
        assert_eq!(row("timeout_outcome").get("default"), Some(&json!("fled")), "{section}");
        assert_eq!(row("timeout_outcome").get("options"), Some(&json!(["fled", "lost"])), "{section}");
        assert_eq!((json::s(&row("stay_behind"), "type"), row("stay_behind").get("default").cloned()), ("json".into(), Some(json!({}))), "{section}");
    }
    let e = normalize(fields(source::schema(), "content", "encounters"), &dict(json!({"id": "x"})), false);
    assert_eq!((e.get("unkillable"), e.get("timeout_outcome"), e.get("stay_behind")), (Some(&json!([])), Some(&json!("fled")), Some(&json!({}))));
    let def = EncounterDef::from_obj(&e);
    assert_eq!(rpgkit_battle::encounter::timeout_outcome(&def.raw), "fled");
    assert!(rpgkit_battle::encounter::StayBehind::read(&def.raw).is_none());
}

#[test]
fn a_valid_hold_and_a_row_without_the_fields_check_clean() {
    let data = scratch("valid");
    catalog(&data);
    let v = store::view(&data);
    assert_eq!(errors_with(&v, json!({})), Vec::<String>::new(), "the winch");
    assert_eq!(errors_with(&v, json!({"unkillable": null, "timeout_outcome": null, "stay_behind": null})), Vec::<String>::new(), "none of the fields");
    assert_eq!(errors_with(&v, json!({"stay_behind": {}})), Vec::<String>::new(), "an empty block");
    assert_eq!(errors_with(&v, json!({"unkillable": ["imp"]})), Vec::<String>::new(), "a phase-wave ref may be unkillable");
    assert_eq!(stay_errors(&v, json!({"tiles": [[10, 1], [10, 7]], "fallback": "none", "pick_default": "first"})), Vec::<String>::new(), "two tiles");
    assert_eq!(
        stay_errors(&v, json!({"mode": "stand", "tiles": [], "fallback": null, "pick_default": null, "exits_open": "after_stand", "stand_hp_pct": 30, "stand_statuses": [{"status": "taunt", "rounds": 5}], "stand_line": "{name} stands alone."})),
        Vec::<String>::new(),
        "a stand"
    );
    drop_scratch(&data);
}

#[test]
fn each_broken_rule_is_one_error() {
    let data = scratch("errors");
    catalog(&data);
    let v = store::view(&data);
    let one = |errors: Vec<String>, want: &str, tag: &str| assert_eq!(errors, vec![want.to_string()], "{tag}");
    one(errors_with(&v, json!({"objective": "rout"})), "Stay-behind needs the escape objective.", "rout");
    one(errors_with(&v, json!({"stay_behind": [1]})), "Stay-behind must be an object ({} = none).", "a list");
    one(errors_with(&v, json!({"unkillable": ["ogre"]})), "Unkillable unit 'ogre' is not in this encounter.", "an enemy of another encounter");
    // a hero is refused by the field's own filter too (enemy-side units only)
    assert!(errors_with(&v, json!({"unkillable": ["hero"]})).contains(&"Unkillable unit 'hero' is not in this encounter.".to_string()), "a hero");
    one(stay_errors(&v, json!({"leave_party": true})), "Stay-behind: unknown key 'leave_party'.", "unknown key");
    one(stay_errors(&v, json!({"mode": "siege"})), "Stay-behind mode 'siege' is not one of hold, stand.", "mode");
    one(stay_errors(&v, json!({"tiles": []})), "A hold needs 1 to 2 tiles.", "no tile");
    one(stay_errors(&v, json!({"tiles": null})), "A hold needs 1 to 2 tiles.", "no tiles key");
    one(stay_errors(&v, json!({"tiles": [[10, 1], [10, 2], [10, 6]]})), "A hold needs 1 to 2 tiles.", "three tiles");
    one(stay_errors(&v, json!({"tiles": [[12, 1]]})), "Hold tile (12, 1) is off the grid, blocked or an exit.", "off the grid");
    one(stay_errors(&v, json!({"tiles": [[3, 3]]})), "Hold tile (3, 3) is off the grid, blocked or an exit.", "a wall on the rows");
    one(stay_errors(&v, json!({"tiles": [[11, 4]]})), "Hold tile (11, 4) is off the grid, blocked or an exit.", "an exit");
    one(stay_errors(&v, json!({"tiles": "x"})), "Hold tiles must be a list of [x, y] pairs.", "not a list");
    one(stay_errors(&v, json!({"tiles_label": ""})), "Stay-behind tiles_label is 1 to 24 characters.", "empty label");
    one(stay_errors(&v, json!({"tiles_label": "x".repeat(25)})), "Stay-behind tiles_label is 1 to 24 characters.", "long label");
    one(stay_errors(&v, json!({"room": "yard"})), "Stay-behind room 'yard' is not a room of this encounter.", "room");
    one(stay_errors(&v, json!({"fallback": "some"})), "Stay-behind fallback 'some' is not one of any, none.", "fallback");
    one(stay_errors(&v, json!({"pick_default": "last"})), "Stay-behind pick_default 'last' is not one of highest_hp, first.", "pick_default");
    one(stay_errors(&v, json!({"stand_hp_pct": 30})), "Stay-behind stand_hp_pct is for mode stand only.", "a stand key on a hold");
    let stand = |extra: Value| {
        let mut sb = json!({"mode": "stand", "tiles": [], "fallback": null, "pick_default": null});
        for (k, val) in extra.as_object().unwrap() {
            sb[k] = val.clone();
        }
        stay_errors(&v, sb)
    };
    one(stand(json!({"tiles": [[10, 1]]})), "A stand holds no tiles.", "stand tiles");
    one(stand(json!({"fallback": "any"})), "Stay-behind fallback is for mode hold only.", "a hold key on a stand");
    one(stand(json!({"exits_open": "never"})), "Stay-behind exits_open 'never' is not one of always, after_stand.", "exits_open");
    one(stand(json!({"stand_hp_pct": 0})), "Stay-behind stand_hp_pct must be 1..100.", "pct 0");
    one(stand(json!({"stand_hp_pct": 101})), "Stay-behind stand_hp_pct must be 1..100.", "pct 101");
    let taunt = json!({"status": "taunt", "rounds": 2});
    one(stand(json!({"stand_statuses": [taunt, taunt, taunt, taunt]})), "Stay-behind stand_statuses: at most 3.", "four statuses");
    one(stand(json!({"stand_statuses": [{"status": "glory", "rounds": 2}]})), "Stand status 'glory' is not a status.", "unknown status");
    one(stand(json!({"stand_statuses": [{"status": "taunt", "rounds": 11}]})), "Stand status 'taunt': rounds must be 1..10.", "rounds 11");
    one(stand(json!({"stand_statuses": "taunt"})), "Stay-behind stand_statuses must be a list of {status, rounds}.", "not a list");
    let line = "Stay-behind result_line holds at most 160 characters, {name} the only token.";
    one(stay_errors(&v, json!({"result_line": "{hero} held."})), line, "another token");
    one(stay_errors(&v, json!({"result_line": "x".repeat(161)})), line, "too long");
    one(stand(json!({"stand_line": 7})), "Stay-behind stand_line holds at most 160 characters, {name} the only token.", "not text");
    one(stay_errors(&v, json!({"flag": "crucible_stayed"})), "Stay-behind flag and then are for world encounters only.", "catalog flag");
    one(stay_errors(&v, json!({"then": "capture"})), "Stay-behind flag and then are for world encounters only.", "catalog then");
    assert_eq!(stay_errors(&v, json!({"then": "none", "result_line": "{name} held the winch."})), Vec::<String>::new(), "then none and one token");
    drop_scratch(&data);
}

#[test]
fn a_world_encounter_checks_its_flag_and_then() {
    let data = scratch("world");
    catalog(&data);
    let dir = data.join("worlds").join("vale");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("world.json"), json!({"id": "vale", "name": "Vale", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    let enc = |id: &str, stay: Value| {
        json!({"id": id, "name": id, "rows": rows(), "exits": [[11, 4]], "objective": "escape",
            "stay_behind": {"mode": "hold", "tiles": [[10, 1]], "flag": stay["flag"].clone(), "then": stay["then"].clone()}})
    };
    let entries = json!([
        enc("good", json!({"flag": "vale.held", "then": "capture"})),
        enc("bad_flag", json!({"flag": "Vale Held", "then": "none"})),
        enc("bad_then", json!({"flag": "", "then": "exile"}))
    ]);
    fs::write(dir.join("encounters.json"), json!({"kind": "encounters", "version": 1, "entries": entries}).to_string()).unwrap();
    let problems: Vec<String> = store::check(&data).into_iter().filter(|p| p.contains("Stay-behind")).collect();
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("bad_flag") && p.contains("Stay-behind flag 'Vale Held' must be a flag key")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("bad_then") && p.contains("Stay-behind then 'exile' is not one of none, capture, kill.")), "{problems:?}");
    drop_scratch(&data);
}

#[test]
fn the_writer_stores_and_reloads_the_fields() {
    let data = scratch("writer");
    catalog(&data);
    let e = dict(winch());
    store::upsert(&data, None, "encounters", &e, None).expect("a valid hold escape");
    let cat = Catalog::from_value(&source::load(&data).catalog);
    let row = &cat.encounters["winch"];
    for key in ["unkillable", "timeout_outcome", "stay_behind"] {
        assert_eq!(row.get(key), e.get(key), "{key}");
    }
    let text = fs::read_to_string(store::file_of(&data, None, "encounters")).unwrap();
    assert_eq!(store::resave_text(&data, None, "encounters").unwrap(), text, "a re-save without changes is byte-identical");
    let mut bad = e.clone();
    bad.insert("objective".into(), json!("rout"));
    let errors = store::upsert(&data, None, "encounters", &bad, None).err().expect("refused");
    assert!(errors.iter().any(|x| x.contains("Stay-behind needs the escape objective.")), "{errors:?}");
    assert_eq!(fs::read_to_string(store::file_of(&data, None, "encounters")).unwrap(), text, "rolled back");
    drop_scratch(&data);
}
