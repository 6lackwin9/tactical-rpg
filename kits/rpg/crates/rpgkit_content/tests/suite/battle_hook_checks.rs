//! PR6-7 (`docs/sessions/PK6_PK7_BOSSES_ENDINGS.md` sections 1-2), content side, on contexts built here (no shipped
//! data): the schema carries `start_meter` (world regions and encounters), `captive_fallback` (world encounters) and
//! the ground hazard row's `when_meter`, each stored only when set, so rows written before them re-save byte for byte;
//! the arena hazard help names `every_rounds` and `when_meter`. Each broken rule is one error: a `when_meter` gate (a
//! known unit meter, a whole `at_least` 0..100), an arena `every_rounds` (1..max_rounds), a `start_meter` list (at most
//! `START_METER_MAX` objects, known keys, a known meter, a whole value not 0, a `requires_track` on a known unit track,
//! a flag key, conditions), and the ref `$captive_unit` (enemies or a phase wave only, once, with a fallback).

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::check::hook_kinds::{self, Refs};
use rpgkit_content::check::hooks::{arena_hazards_errors, start_meter_errors, Ctx, START_METER_MAX};
use rpgkit_content::check::{self, View};
use rpgkit_content::{source, store};
use rpgkit_data::{CAPTIVE_UNIT, HAZARDS_KIND};
use serde_json::{json, Value};

fn field(section: &str, kind: &str, key: &str) -> Dict {
    fields(source::schema(), section, kind).iter().map(json::dict).find(|f| json::s(f, "key") == key).cloned().unwrap_or_else(|| panic!("{section} {kind} {key}"))
}

#[test]
fn the_new_fields_are_stored_only_when_set() {
    for (kind, key) in [("regions", "start_meter"), ("encounters", "start_meter"), ("encounters", "captive_fallback")] {
        assert!(json::b(&field("world", kind, key), "omit_default", false), "{kind} {key}");
    }
    let plain = json::dict(&json!({"id": "trial", "name": "Trial", "enemies": ["foe"]})).clone();
    let stored = store::world_storage("encounters", &plain);
    assert!(!stored.contains_key("start_meter") && !stored.contains_key("captive_fallback"));
    let region = json::dict(&json!({"id": "r", "name": "R"})).clone();
    assert!(!store::world_storage("regions", &region).contains_key("start_meter"));
    let set = json::dict(&json!({"id": "trial", "name": "Trial", "enemies": ["foe"], "start_meter": [{"meter": "tension", "value": 5}]})).clone();
    assert_eq!(store::world_storage("encounters", &set).get("start_meter"), set.get("start_meter"));
    assert!(json::b(&field("content", HAZARDS_KIND, "when_meter"), "omit_default", false));
    let hazard = |gate: Value| json!({"id": "smoke", "name": "Smoke", "when_meter": gate}).as_object().unwrap().clone();
    assert!(!store::catalog_storage(HAZARDS_KIND, &hazard(json!({})), &[]).unwrap().contains_key("when_meter"));
    let gate = json!({"id": "tension", "at_least": 50});
    assert_eq!(store::catalog_storage(HAZARDS_KIND, &hazard(gate.clone()), &[]).unwrap().get("when_meter"), Some(&gate));
    for section in ["content", "world"] {
        let help = json::s(&field(section, "encounters", "arena_hazards"), "help");
        assert!(help.contains("\"every_rounds\"") && help.contains("\"when_meter\""), "{section}: {help}");
    }
}

// ---- when_meter and every_rounds -------------------------------------------------------------------------------------

fn arena(extra: Value, effect: Value) -> Vec<String> {
    let mut h = json!({"id": "toll", "name": "Toll", "columns": [3], "blocks": [6], "effect": effect});
    for (k, v) in extra.as_object().unwrap() {
        h[k] = v.clone();
    }
    let e = json!({"max_rounds": 6, "total_blocks": 6, "arena_hazards": [h]});
    arena_hazards_errors(e.as_object().unwrap(), &["tension".to_string()])
}

fn one(got: Vec<String>, want: &str) {
    assert_eq!(got.len(), 1, "{got:?} wants {want}");
    assert!(got[0].contains(want), "{got:?} wants {want}");
}

#[test]
fn an_arena_cadence_and_gate_check() {
    let pulse = json!({"damage_pct": 10, "meter": {"meter": "tension", "delta": 10}, "when_meter": {"id": "tension", "at_least": 50}});
    assert_eq!(arena(json!({"every_rounds": 2}), pulse.clone()), Vec::<String>::new());
    assert_eq!(arena(json!({"every_rounds": 6, "rounds": [1]}), json!({"damage": 3})), Vec::<String>::new());
    one(arena(json!({"every_rounds": 0}), json!({"damage": 3})), "every_rounds must be a whole number 1-6");
    one(arena(json!({"every_rounds": 7}), json!({"damage": 3})), "every_rounds must be a whole number 1-6");
    one(arena(json!({}), json!({"damage": 3, "when_meter": {"id": "rage", "at_least": 5}})), "when_meter: unknown unit meter 'rage'.");
    one(arena(json!({}), json!({"damage": 3, "when_meter": {"id": "tension", "at_least": 101}})), "when_meter: at_least must be a whole number 0-100.");
    one(arena(json!({}), json!({"damage": 3, "when_meter": {"id": "tension"}})), "when_meter: at_least is required");
    one(arena(json!({}), json!({"damage": 3, "when_meter": {"at_least": 5}})), "when_meter: id is required");
    one(arena(json!({}), json!({"damage": 3, "when_meter": {"id": "tension", "at_least": 5, "x": 1}})), "when_meter: unknown key 'x'");
    one(arena(json!({}), json!({"damage": 3, "when_meter": 5})), "when_meter must be an object");
    one(arena(json!({}), json!({"when_meter": {"id": "tension", "at_least": 5}})), "set at least one of damage");
    assert_eq!(arena(json!({}), json!({"damage": 3, "when_meter": {}})), Vec::<String>::new(), "an empty gate is none");
}

#[test]
fn a_ground_row_gate_checks() {
    let meters = vec!["tension".to_string()];
    let refs = |m| Refs { skills: &[], derived: &[], families: &[], meters: m, terrain: &[] };
    let row = |gate: Value| json!({"id": "hum", "name": "Hum", "origin": "user", "on": "both", "affects": "all", "tint": "#808080", "glyph": "*", "damage": 2, "when_meter": gate}).as_object().unwrap().clone();
    let errs = |gate: Value, m| hook_kinds::row_errors(HAZARDS_KIND, &row(gate), &Ctx::default(), &refs(m));
    assert_eq!(errs(json!({"id": "tension", "at_least": 50}), Some(&meters[..])), Vec::<String>::new());
    one(errs(json!({"id": "rage", "at_least": 50}), Some(&meters[..])), "Hazard when_meter: unknown unit meter 'rage'.");
    assert_eq!(errs(json!({"id": "rage", "at_least": 50}), None), Vec::<String>::new(), "the meters not loaded yet: the shape only");
}

// ---- start_meter -----------------------------------------------------------------------------------------------------

fn start(list: Value) -> Vec<String> {
    let e = json!({"id": "x", "start_meter": list});
    start_meter_errors(e.as_object().unwrap(), &["tension".to_string()], &["rot".to_string()])
}

#[test]
fn a_start_meter_list_checks() {
    assert_eq!(start(json!([])), Vec::<String>::new());
    let full = json!({"meter": "tension", "value": -8, "requires_track": {"id": "rot", "at_least": 1}, "unless_flag": "w.done", "requires_flags": [{"key": "w.ready", "value": true}]});
    assert_eq!(start(json!([full])), Vec::<String>::new());
    assert_eq!(start(Value::Array(vec![json!({"meter": "tension", "value": 1}); START_METER_MAX])), Vec::<String>::new());
    one(start(Value::Array(vec![json!({"meter": "tension", "value": 1}); START_METER_MAX + 1])), "Battle-start meters");
    one(start(json!({})), "Battle-start meters");
    one(start(json!([5])), "Battle-start meter 1 must be an object");
    one(start(json!([{"meter": "rage", "value": 5}])), "Battle-start meter 1: unknown unit meter 'rage'.");
    one(start(json!([{"meter": "tension", "value": 0}])), "value must be a whole number -100..100, not 0.");
    one(start(json!([{"meter": "tension", "value": 101}])), "value must be a whole number -100..100, not 0.");
    one(start(json!([{"meter": "tension", "value": 5, "side": "party"}])), "unknown key 'side'");
    one(start(json!([{"meter": "tension", "value": 5, "requires_track": {"id": "calm", "at_least": 1}}])), "requires_track: unknown unit track 'calm'.");
    one(start(json!([{"meter": "tension", "value": 5, "requires_track": {"id": "rot", "at_least": 100}}])), "requires_track: at_least must be a whole number -99-99.");
    one(start(json!([{"meter": "tension", "value": 5, "unless_flag": "Not A Flag"}])), "unless_flag must be a flag key");
    one(start(json!([{"meter": "tension", "value": 5, "requires_flags": [{"key": "W A"}]}])), "requires_flags 1: flag key 'W A'");
}

// ---- world rows ------------------------------------------------------------------------------------------------------

fn view() -> View {
    let catalog = json!({
        "units": [{"id": "hero", "name": "Hero", "side": "party"}, {"id": "brute", "name": "Brute", "side": "enemy"}],
        "unit_meters": [{"id": "tension", "name": "Tension", "min": 0, "max": 100}],
        "unit_tracks": [{"id": "rot", "name": "Rot"}]
    });
    let chars = json!([
        {"id": "lead", "name": "Lead", "role": "party", "base_unit": "hero", "_world": "w", "_qid": "w:lead"},
        {"id": "foe", "name": "Foe", "role": "enemy", "base_unit": "brute", "_world": "w", "_qid": "w:foe"}
    ]);
    let worlds = json!({"w": {"manifest": {"id": "w", "name": "W"}, "own": {"characters": chars}}});
    View::new(&catalog, &worlds, &json!({}), Default::default())
}

fn world_errors(kind: &str, e: Value) -> Vec<String> {
    let e = e.as_object().unwrap().clone();
    check::world::validate_entry(&view(), "w", kind, &e, &json::s(&e, "id"))
}

fn enc(extra: Value) -> Value {
    let mut e = json!({"id": "crucible", "name": "Crucible", "party": ["lead"], "enemies": ["foe", CAPTIVE_UNIT]});
    for (k, v) in extra.as_object().unwrap() {
        e[k] = v.clone();
    }
    e
}

#[test]
fn the_captive_ref_needs_a_fallback_and_appears_once() {
    let base = world_errors("encounters", enc(json!({"enemies": ["foe"]})));
    let only_new = |extra: Value| world_errors("encounters", enc(extra)).into_iter().filter(|e| !base.contains(e)).collect::<Vec<_>>();
    assert_eq!(only_new(json!({"captive_fallback": "foe"})), Vec::<String>::new(), "a fallback: the ref resolves");
    assert_eq!(only_new(json!({})), [format!("The captive unit {CAPTIVE_UNIT} needs a Captive fallback character (it plays when no member was captured).")]);
    assert_eq!(only_new(json!({"captive_fallback": "foe", "enemies": ["foe", CAPTIVE_UNIT, CAPTIVE_UNIT]})), [format!("The captive unit {CAPTIVE_UNIT} is listed 2 times (once at most).")]);
    let unknown = only_new(json!({"captive_fallback": "ghost"}));
    assert_eq!(unknown.len(), 1, "{unknown:?}");
    assert!(unknown[0].contains("unknown character 'ghost'"), "{unknown:?}");
    // the party never takes the token
    let party = only_new(json!({"captive_fallback": "foe", "party": ["lead", CAPTIVE_UNIT]}));
    assert!(party.iter().any(|e| e.contains(CAPTIVE_UNIT)), "{party:?}");
}

#[test]
fn world_rows_check_their_start_meters() {
    let base = world_errors("encounters", enc(json!({"enemies": ["foe"]})));
    let added = world_errors("encounters", enc(json!({"enemies": ["foe"], "start_meter": [{"meter": "rage", "value": 5}]})));
    assert_eq!(added.into_iter().filter(|e| !base.contains(e)).collect::<Vec<_>>(), ["Battle-start meter 1: unknown unit meter 'rage'."]);
    assert_eq!(world_errors("regions", json!({"id": "r", "name": "R", "start_meter": [{"meter": "tension", "value": 10, "requires_track": {"id": "rot", "at_least": 1}}]})), Vec::<String>::new());
    assert_eq!(world_errors("regions", json!({"id": "r", "name": "R", "start_meter": [{"meter": "tension", "value": 0}]})), ["Battle-start meter 1: value must be a whole number -100..100, not 0."]);
}
