//! PK2-S1 (`docs/sessions/PK2_GRAFTS.md` section 2): the catalog kind `unit_tracks` and the member grants of the
//! passive grammar, on a scratch data folder built here (no shipped data). A missing file is an empty table; a row's
//! checks (id, name, origin, the cause id (any id: the game names it; needed when a stage departs), the advancing mark, 1-9 stages each named, with a slotless passive row or none and
//! a departure only on the last stage) and across rows (unique ids); the loader keeps the rows that pass and they reach
//! `Catalog::unit_tracks`; the writer takes a row, re-saves it byte for byte and refuses a bad one. A passive's
//! `forbid_hands` {hands: 2} and `pierce_melee` {} check clean and their mistakes are one error each; an item may not
//! carry them; the unknown-type message keeps its list.

use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::{Catalog, DEPART_CAUSES, TRACK_STAGES_MAX, UNIT_TRACKS_KIND};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_unit_tracks_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

/// The test row's departure cause (any id: the kit names none).
const CAUSE: &str = "t_cause";

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

/// Test passives: two slotless stage passives (one with each member grant) and a slotted keystone.
fn passives(data: &Path) {
    write_kind(
        data,
        "passives",
        json!([
            {"id": "pas_stage_one", "name": "Stage One", "effects": [{"type": "resist", "status": "bleed", "value": 5}], "slot": ""},
            {"id": "pas_stage_two", "name": "Stage Two", "effects": [{"type": "forbid_hands", "hands": 2}, {"type": "pierce_melee"}], "slot": ""},
            {"id": "pas_slotted", "name": "Slotted", "effects": [{"type": "resist", "status": "bleed", "value": 5}], "slot": "keystone"}
        ]),
    );
}

/// A valid three-stage row: two stage passives and a last-stage departure.
fn track(id: &str) -> Value {
    json!({"id": id, "name": "Test Track", "mark": "test_mark", "cause": CAUSE, "stages": [
        {"name": "One", "passive": "pas_stage_one"},
        {"name": "Two", "passive": "pas_stage_two"},
        {"name": "Three", "passive": "", "departure": "lost"}
    ]})
}

fn with(mut row: Value, key: &str, v: Value) -> Value {
    row[key] = v;
    row
}

fn stages(v: Value) -> Value {
    with(track("tt"), "stages", v)
}

#[test]
fn a_missing_file_is_an_empty_table() {
    let (_scratch, data) = scratch("missing");
    let src = source::load(&data);
    assert!(json::arr(&src.catalog[UNIT_TRACKS_KIND]).is_empty());
    assert!(src.errors.iter().all(|e| !e.contains(&format!("{UNIT_TRACKS_KIND}.json"))), "{:?}", src.errors);
    assert!(Catalog::from_value(&src.catalog).unit_tracks.is_empty());
}

#[test]
fn each_mistake_has_its_text() {
    let (_scratch, data) = scratch("texts");
    passives(&data);
    let v = store::view(&data);
    let check = |e: Value| check::validate_entry(&v, UNIT_TRACKS_KIND, &dict(e), "");
    assert_eq!(check(track("tt")), Vec::<String>::new());
    assert_eq!(check(with(track("tt"), "mark", json!(""))), Vec::<String>::new(), "no mark: nothing advances it in battle");
    // PR5-26 (audit B9): a track needs a name
    assert_eq!(check(with(track("tt"), "name", json!(""))), ["A unit track needs a name."]);
    assert_eq!(check(stages(json!([]))), [format!("A unit track needs 1-{TRACK_STAGES_MAX} stages (has 0).")]);
    let many: Vec<Value> = (0..=TRACK_STAGES_MAX).map(|i| json!({"name": format!("S{i}")})).collect();
    assert_eq!(check(stages(Value::Array(many))), [format!("A unit track needs 1-{TRACK_STAGES_MAX} stages (has {}).", TRACK_STAGES_MAX + 1)]);
    assert_eq!(check(stages(json!([{"name": "One", "departure": "lost"}, {"name": "Two"}]))), ["Stage 1: only the last stage may carry a departure."]);
    assert_eq!(check(stages(json!([{"name": "One", "departure": "dead"}]))), ["Stage 1: departure 'dead' is not one of lost."]);
    assert_eq!(check(stages(json!([{"name": "One", "passive": "pas_nope"}]))), ["Stage 1: unknown passive 'pas_nope'."]);
    assert_eq!(check(stages(json!([{"name": "One", "passive": "pas_slotted"}]))), ["Stage 1: passive 'pas_slotted' takes a deck slot; a stage passive is slotless."]);
    assert_eq!(check(stages(json!([{"name": ""}]))), ["Stage 1: a name of 1-32 characters."]);
    assert_eq!(check(stages(json!([{"name": "One", "rank": 2}]))), ["Stage 1: unknown key 'rank' (use name, passive, departure)."]);
    assert_eq!(check(stages(json!(["One"]))), ["Stage 1 must be an object {name, passive, departure}."]);
    assert_eq!(check(stages(json!({"name": "One"}))), ["Stages must be a list."]);
    assert_eq!(check(with(track("tt"), "cause", json!("stayed"))), Vec::<String>::new(), "any id, a kit cause too");
    assert_eq!(check(with(track("tt"), "cause", json!("Bad Cause"))), ["Departure cause 'Bad Cause' must be an id of up to 24 characters (a-z, 0-9 and _)."]);
    assert_eq!(check(with(track("tt"), "cause", json!(""))), ["A unit track with a departure stage needs a departure cause."]);
    assert_eq!(check(with(stages(json!([{"name": "One"}])), "cause", json!(""))), Vec::<String>::new(), "no departure: no cause needed");
    assert_eq!(check(with(track("tt"), "mark", json!("Bad Mark"))), ["Advancing mark 'Bad Mark' must be an id of up to 24 characters (a-z, 0-9 and _)."]);
    let bad_id = check(track("T"));
    assert!(bad_id.iter().any(|e| e.starts_with("ID 'T'")), "{bad_id:?}");
}

#[test]
fn the_loader_keeps_the_rows_that_pass() {
    let (_scratch, data) = scratch("loader");
    passives(&data);
    write_kind(&data, UNIT_TRACKS_KIND, json!([track("tt"), track("tt"), with(track("broken"), "stages", json!([]))]));
    let src = source::load(&data);
    let kept: Vec<String> = json::arr(&src.catalog[UNIT_TRACKS_KIND]).iter().map(|e| json::s(json::dict(e), "id")).collect();
    assert_eq!(kept, ["tt"], "the refused rows are skipped");
    let found: Vec<&String> = src.errors.iter().filter(|e| e.contains("unit track '")).collect();
    assert!(found.iter().any(|e| e.ends_with("unit track 'tt': ID 'tt' is already used by another unit track.")), "{found:?}");
    assert!(found.iter().any(|e| e.contains("unit track 'broken': A unit track needs")), "{found:?}");
    let cat = Catalog::from_value(&src.catalog);
    let t = &cat.unit_tracks["tt"];
    assert_eq!((t.mark.as_str(), t.cause.as_str(), t.top()), ("test_mark", CAUSE, 3));
    let causes: Vec<String> = DEPART_CAUSES.iter().map(|c| c.to_string()).chain([CAUSE.to_string()]).collect();
    assert_eq!(cat.depart_causes(), causes, "a row's cause is a known departure cause after the kit's");
    assert!(cat.depart_cause_ok(CAUSE) && cat.depart_cause_ok(DEPART_CAUSES[0]) && !cat.depart_cause_ok("t_other"));
    assert!(!DEPART_CAUSES.contains(&CAUSE), "the kit's list names no track cause");
    assert_eq!(t.passives_to(2), ["pas_stage_one", "pas_stage_two"], "a stage holds every passive up to it");
    assert_eq!(t.passives_to(9), ["pas_stage_one", "pas_stage_two"], "past the top: the top's");
    assert_eq!((t.departure_at(3), t.departure_at(2), t.departure_at(0)), ("lost", "", ""));
    let v = store::view(&data);
    assert_eq!(check::validate_entry(&v, UNIT_TRACKS_KIND, &dict(track("tt")), "tt"), Vec::<String>::new(), "an edit of the row itself");
    assert_eq!(check::validate_entry(&v, UNIT_TRACKS_KIND, &dict(track("tt")), ""), ["ID 'tt' is already used by another unit track."]);
}

#[test]
fn the_writer_takes_a_row_and_refuses_a_bad_one() {
    let (_scratch, data) = scratch("writer");
    passives(&data);
    let file = data.join("catalog").join(format!("{UNIT_TRACKS_KIND}.json"));
    let refused = store::upsert(&data, None, UNIT_TRACKS_KIND, &dict(stages(json!([{"name": "One", "passive": "pas_nope"}]))), None).err().expect("a bad row is refused");
    assert!(refused.iter().any(|e| e.ends_with("Stage 1: unknown passive 'pas_nope'.")), "{refused:?}");
    assert!(!file.exists(), "a refused first row leaves no file");
    store::upsert(&data, None, UNIT_TRACKS_KIND, &dict(track("tt")), None).expect("a valid row");
    let text = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, None, UNIT_TRACKS_KIND).unwrap(), text, "a re-save without changes is byte-identical");
    assert!(store::check(&data).iter().all(|p| !p.contains("unit track '")), "{:?}", store::check(&data));
    assert_eq!(Catalog::from_value(&source::load(&data).catalog).unit_tracks["tt"].stages.len(), 3);
    store::remove(&data, None, UNIT_TRACKS_KIND, "tt").unwrap();
    assert!(Catalog::from_value(&source::load(&data).catalog).unit_tracks.is_empty());
}

fn passive_errors(v: &check::View, effects: Value) -> Vec<String> {
    check::validate_entry(v, "passives", &dict(json!({"id": "pas_test", "name": "Test", "effects": effects})), "")
}

#[test]
fn the_member_grants_check_on_passives_only() {
    let (_scratch, data) = scratch("grants");
    passives(&data);
    let v = store::view(&data);
    assert_eq!(passive_errors(&v, json!([{"type": "forbid_hands", "hands": 2}, {"type": "pierce_melee"}])), Vec::<String>::new());
    assert_eq!(passive_errors(&v, json!([{"type": "forbid_hands", "hands": 1}])), ["Effect 1: forbid_hands needs hands 2 (two-handed items)."]);
    assert_eq!(passive_errors(&v, json!([{"type": "forbid_hands"}])), ["Effect 1: forbid_hands needs hands 2 (two-handed items)."]);
    assert_eq!(passive_errors(&v, json!([{"type": "forbid_hands", "hands": 2, "value": 1}])), ["Effect 1: forbid_hands takes only hands (not 'value')."]);
    assert_eq!(passive_errors(&v, json!([{"type": "pierce_melee", "value": 1}])), ["Effect 1: pierce_melee takes no keys (not 'value')."]);
    let unknown = passive_errors(&v, json!([{"type": "zz"}]));
    assert_eq!(unknown.len(), 1);
    assert!(unknown[0].starts_with("Effect 1: unknown effect type 'zz' (use stat, ") && !unknown[0].contains("forbid_hands"), "the message keeps its list: {unknown:?}");
    // an item carries neither
    let skill_exists = |_: &str| true;
    let ctx = check::hooks::Ctx::of(&v);
    for t in ["forbid_hands", "pierce_melee"] {
        let found = check::simple::effect_errors(&json!([{"type": t, "hands": 2}]), &skill_exists, &[], false, true, &ctx);
        assert_eq!(found, [format!("Effect 1: '{t}' is not allowed on items (use stat_bonuses / passives).")], "{t}");
    }
}
