//! The campaign schema slice (CAMPAIGN1-S, docs/specs/CAMPAIGN1.md 11.1-11.4): the world manifest blocks `campaign`,
//! `purse`, `rest`, `defeat`; the world kinds `meters`, `checkpoints`, `triggers`; the campaign effects in every effect
//! list; the read keys in conditions; the writer for the new fields and kinds. A campaign world written to the spec's
//! section 4 shape checks clean, and each field set wrong adds exactly one error (the classic worlds' reports are the
//! game façade's test). Scratch data folders only.

use aegis_content::schema::fields;
use aegis_core::json::{self, Dict};
use rpgkit_content::{check, source, store};
use rpgkit_data::{CampaignManifest, DefeatRule, Purse, RestRule};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const W: &str = "ward";

fn scratch(tag: &str) -> PathBuf {
    let t = std::env::temp_dir().join(format!("rpgkit_campaign_schema_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&t);
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds").join(W)).unwrap();
    data
}

fn drop_scratch(data: &Path) {
    let _ = fs::remove_dir_all(data.parent().unwrap());
}

fn dict(v: Value) -> Dict {
    v.as_object().cloned().unwrap()
}

/// Writes a world kind file as a fixture (the files are the test's own input; the writer is tested below).
fn write_kind(data: &Path, kind: &str, entries: Value) {
    fs::write(store::file_of(data, Some(W), kind), json!({"kind": kind, "version": 1, "entries": entries}).to_string()).unwrap();
}

/// The manifest blocks of the spec's section 4.1 (the asset lane's `sunk_ward` shape, with fixture names).
fn manifest() -> Value {
    json!({
        "id": W, "name": "Ward", "extends": [], "connects": [], "start_location": "hub",
        "campaign": {"start_act": 0, "act_saves": true},
        "purse": {
            "currency": {"name": "Scrap", "icon": "ui/hud/scrap.png", "tip": "Salvaged iron."},
            "upkeep": {"name": "Warded Tallow", "short": "Tallow", "icon": "ui/hud/tallow.png", "tip": "1 a day.", "per_day": 1, "at_zero": {"road_pct": 100, "start_ap": -2}}
        },
        "rest": {"days": 1, "cost": 1, "where": "rest_places", "on_enter": false, "needs_upkeep": false},
        "defeat": {"mode": "retreat", "to": "hub", "days": 1},
        "version": 1
    })
}

fn alert() -> Value {
    json!({
        "id": "alert", "name": "Inquisition Alert", "kind": "gauge", "min": 0, "max": 100, "start": 0, "unit": "%", "visible": false,
        "bands": [
            {"id": "guarded", "name": "Guarded", "from": 0, "tone": "calm"},
            {"id": "suspicious", "name": "Suspicious", "from": 31, "tone": "warn"},
            {"id": "lockdown", "name": "Lockdown", "from": 71, "tone": "danger"},
            {"id": "breach", "name": "Breach", "from": 100, "tone": "fatal"}
        ],
        "rules": [
            {"id": "a_suspicious", "at": "suspicious", "warn": "Gates are watched."},
            {"id": "a_last", "at": 90, "acts": [1], "warn": "One more loud job."},
            {"id": "a_breach_1", "at": "breach", "acts": [1], "effects": [{"effect": "end", "kind": "fail", "card": "extinguished_breach"}]}
        ],
        "inputs": [
            {"on": "battle", "tags": ["loud"], "outcomes": ["won"], "delta": 8},
            {"on": "battle", "tags": ["loud"], "outcomes": ["lost", "fled"], "delta": 4},
            {"on": "day", "delta": 1, "acts": [2]},
            {"on": "leg", "route_tags": ["main"], "delta": 1}
        ],
        "modifiers": [{"bands": ["lockdown"], "road_pct": 150, "road_encounters": [["raid", 10]], "route_tags": ["main"]}],
        "floors": [{"from_act": 2, "min": 30}],
        "hud": {"short": "Alert", "icon": "ui/hud/alert.png", "order": 2}
    })
}

fn doom() -> Value {
    json!({
        "id": "doom_clock", "name": "The Red Tithing", "kind": "countdown", "min": 0, "max": 99, "start": 0, "unit": "days",
        "bands": [{"id": "due", "name": "Due", "from": 0, "tone": "danger"}, {"id": "near", "name": "Near", "from": 1, "tone": "warn"}, {"id": "far", "name": "Far", "from": 8, "tone": "calm"}],
        "rules": [
            {"id": "d_week", "at": 7, "warn": "Seven days."},
            {"id": "d_zero", "at": 0, "acts": [1, 2], "effects": [{"effect": "add_flag", "key": "lesser_tithings", "delta": 1}], "then": [{"vignette": "v_lesser_tithing"}], "restart_days": 14}
        ],
        "hud": {"short": "Tithing", "icon": "ui/hud/doom_clock.png", "order": 1}
    })
}

fn checkpoints() -> Value {
    json!([
        {"id": "cp_cages", "name": "The Cages", "act": 1, "quest": "q_cages", "deadline": 14, "warn_days": [7, 3, 1], "mode": "end_run", "card": "bad_end_cages", "text": "Too late."},
        {"id": "cp_gate", "name": "The Gate", "act": 2, "met_when": [{"key": "ward.gate_open", "value": true}], "deadline": 30, "mode": "go_on", "effects": [{"effect": "meter_add", "meter": "alert", "delta": 10}]}
    ])
}

fn triggers() -> Value {
    json!([
        {"id": "t_act1", "name": "Act 1", "on": "enter", "location": "hub", "when": [{"key": "ward.prologue_done", "value": true}],
            "effects": [{"effect": "set_act", "act": 1}, {"effect": "meter_show", "meter": "alert"}, {"effect": "clock_start", "meter": "doom_clock", "days": 42}], "then": [{"vignette": "v_act1_title"}]},
        {"id": "t_wiped", "name": "Wiped", "on": "any", "when": [{"key": "roster.living", "op": "<", "value": 1}], "effects": [{"effect": "end", "kind": "fail", "card": "extinguished_wiped"}]},
        {"id": "t_raid", "name": "Raid", "on": "battle", "encounter": "raid", "outcome": "won", "once": false,
            "effects": [{"effect": "meter_set", "meter": "alert", "value": 50}, {"effect": "clock_set", "meter": "doom_clock", "days": 10, "label": "The Great Tithing"}],
            "then": [{"battle": "raid"}, {"dialogue": "tess"}]}
    ])
}

fn tess() -> Value {
    json!({"id": "tess", "name": "Tess", "start": "start", "services": false, "nodes": [{"id": "start", "text": "Well?", "choices": [
        {"text": "Pay to cool things down.", "to": "", "requires": [{"kind": "flag", "key": "meter.alert", "op": ">=", "value": 31}],
            "effects": [{"effect": "take", "gold": 40}, {"effect": "meter_add", "meter": "alert", "delta": -10}]},
        {"text": "Rest a day.", "to": "", "requires": [{"key": "meter.alert.band", "value": "lockdown", "op": "!="}, {"key": "campaign.act", "op": ">=", "value": 1}], "effects": [{"effect": "rest"}]},
        {"text": "Leave.", "to": ""}
    ]}]})
}

/// A scratch campaign world: the manifest, two locations (the hub a rest place), a quest, an encounter, the dialogue,
/// the two meters, the checkpoints and the triggers.
fn campaign_world(tag: &str) -> PathBuf {
    let data = scratch(tag);
    fs::write(aegis_content::world::manifest_path(&data, W), manifest().to_string()).unwrap();
    write_kind(&data, "locations", json!([{"id": "hub", "name": "Hub", "rest": true}, {"id": "street", "name": "Street"}]));
    write_kind(&data, "quests", json!([{"id": "q_cages", "name": "The Cages"}]));
    write_kind(&data, "encounters", json!([{"id": "raid", "name": "Raid"}]));
    write_kind(&data, "dialogues", json!([tess()]));
    write_kind(&data, "meters", json!([alert(), doom()]));
    write_kind(&data, "checkpoints", checkpoints());
    write_kind(&data, "triggers", triggers());
    // STORYUI1-S: a `then` vignette names a row of the world kind `vignettes`
    let card = json!([{"style": "text", "text": "The bells toll."}]);
    write_kind(&data, "vignettes", json!([{"id": "v_lesser_tithing", "name": "Lesser Tithing", "cards": card}, {"id": "v_act1_title", "name": "Act 1", "cards": card}]));
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
    for (i, p) in path.iter().enumerate() {
        let last = i + 1 == path.len();
        at = match p.parse::<usize>() {
            Ok(n) => &mut at[n],
            Err(_) if last => {
                at[*p] = value;
                return out;
            }
            Err(_) => &mut at[*p],
        };
    }
    *at = value;
    out
}

/// `entry` without `key`.
fn without(entry: &Value, key: &str) -> Value {
    let mut out = entry.clone();
    out.as_object_mut().unwrap().shift_remove(key);
    out
}

fn assert_one(v: &check::View, kind: &str, good: &Value, bad: &Value, needle: &str, tag: &str) {
    let found = added(v, kind, good, bad);
    assert_eq!(found.len(), 1, "{tag}: {found:?}");
    assert!(found[0].contains(needle), "{tag}: {found:?} (wanted '{needle}')");
}

#[test]
fn the_schema_carries_the_campaign_fields_and_kinds() {
    let schema = source::schema();
    for key in ["campaign", "purse", "rest", "defeat"] {
        let f = fields(schema, "world", "world").iter().map(json::dict).find(|f| json::s(f, "key") == key).unwrap_or_else(|| panic!("world.world.{key}"));
        assert_eq!(json::s(f, "type"), "json", "{key}");
        assert_eq!(json::get(f, "default"), &json!({}), "{key}: {{}} is today's rule");
    }
    for kind in rpgkit_data::CAMPAIGN_KINDS {
        let keys: Vec<String> = fields(schema, "world", kind).iter().map(|f| json::s(json::dict(f), "key")).collect();
        assert_eq!(keys.first().map(String::as_str), Some("id"), "{kind}");
        assert!(keys.ends_with(&["override".to_string(), "tags".to_string()]), "{kind}: {keys:?}");
        assert!(source::WORLD_KINDS.contains(&kind) && check::world::KINDS.contains(&kind), "{kind} is a loaded and checked world kind");
    }
    let src = source::load(&campaign_world("schema"));
    let own = &src.worlds[W]["own"];
    assert_eq!(own["meters"].as_array().map(Vec::len), Some(2));
    assert_eq!(own["checkpoints"].as_array().map(Vec::len), Some(2));
    assert_eq!(own["triggers"].as_array().map(Vec::len), Some(3));
    assert_eq!(own["triggers"][1]["once"], json!(true), "a trigger's once defaults to true");
    assert_eq!(own["checkpoints"][0]["met_when"], json!([]), "defaults fill in");
}

#[test]
fn the_section_4_campaign_world_checks_clean() {
    let data = campaign_world("clean");
    let v = store::view(&data);
    for kind in rpgkit_data::CAMPAIGN_KINDS.into_iter().chain(["dialogues"]) {
        for e in v.worlds.own_entries(W, kind) {
            let found = check::world::validate_entry(&v, W, kind, e, &json::s(e, "id"));
            assert!(found.is_empty(), "{kind}:{}: {found:?}", json::s(e, "id"));
            assert!(check::world::blocking_errors(&v, W, kind, e, &json::s(e, "id")).is_empty(), "{kind} blocking");
        }
    }
    assert_eq!(check::campaign::manifest_errors(&v, W, v.worlds.manifest(W)), Vec::<String>::new());
    drop_scratch(&data);
}

#[test]
fn each_bad_meter_field_is_one_error() {
    let data = campaign_world("meters");
    let v = store::view(&data);
    let (gauge, clock) = (alert(), doom());
    let cases: Vec<(&str, Value, Value, &str)> = vec![
        ("bands not starting at min", gauge.clone(), with(&gauge, &["bands", "0", "from"], json!(5)), "The first band must start at min (0)."),
        ("a rule at naming no band", gauge.clone(), with(&gauge, &["rules", "0", "at"], json!("nope")), "at 'nope' names no band"),
        ("a rule at out of range", gauge.clone(), with(&gauge, &["rules", "1", "at"], json!(140)), "at must be a band id or a whole number"),
        ("a countdown without days", clock.clone(), with(&clock, &["unit"], json!("")), "A countdown needs the unit days."),
        ("a unit scope row", gauge.clone(), with(&gauge, &["scope"], json!("unit")), "CAMPAIGN1-DOC"),
        ("min not below max", gauge.clone(), with(&gauge, &["min"], json!(100)), "Min must be below max."),
        ("start out of range", gauge.clone(), with(&gauge, &["start"], json!(101)), "Start must be in min..max"),
        ("a countdown max over 999", clock.clone(), with(&clock, &["max"], json!(1000)), "max must be 1..999"),
        ("bands falling", gauge.clone(), with(&gauge, &["bands", "2", "from"], json!(20)), "bands must rise"),
        ("an unknown tone", gauge.clone(), with(&gauge, &["bands", "1", "tone"], json!("loud")), "tone 'loud'"),
        ("a band id twice", gauge.clone(), with(&gauge, &["bands", "0", "id"], json!("suspicious")), "is used twice"),
        ("seven bands", clock.clone(), with(&clock, &["bands"], json!((0..7).map(|i| json!({"id": format!("b{i}"), "name": "B", "from": i, "tone": "calm"})).collect::<Vec<_>>())), "1-6 bands"),
        ("restart_days on a gauge", gauge.clone(), with(&gauge, &["rules", "0", "restart_days"], json!(5)), "restart_days is for countdowns only."),
        ("a rule dir unknown", gauge.clone(), with(&gauge, &["rules", "0", "dir"], json!("sideways")), "dir 'sideways'"),
        ("a rule act 10", gauge.clone(), with(&gauge, &["rules", "1", "acts"], json!([10])), "acts must be a list of acts 0..9"),
        ("an input on a countdown", clock.clone(), with(&clock, &["inputs"], json!([{"on": "day", "delta": 1}])), "Inputs move gauges only"),
        ("an input delta 0", gauge.clone(), with(&gauge, &["inputs", "2", "delta"], json!(0)), "delta must be a whole number"),
        ("an input outcome unknown", gauge.clone(), with(&gauge, &["inputs", "0", "outcomes"], json!(["draw"])), "outcomes must be a list of won, lost, fled"),
        ("a leg input with battle tags", gauge.clone(), with(&gauge, &["inputs", "3", "tags"], json!(["loud"])), "unknown key 'tags'"),
        ("a modifier band unknown", gauge.clone(), with(&gauge, &["modifiers", "0", "bands"], json!(["riot"])), "band 'riot'"),
        ("a modifier encounter unknown", gauge.clone(), with(&gauge, &["modifiers", "0", "road_encounters"], json!([["ghost", 10]])), "unknown encounter 'ghost'"),
        ("a floor above max", gauge.clone(), with(&gauge, &["floors", "0", "min"], json!(130)), "min must be a whole number in min..max"),
        ("a HUD short over 16", gauge.clone(), with(&gauge, &["hud", "short"], json!("Inquisition Alert!")), "short is longer than 16"),
        ("a name over 32", gauge.clone(), with(&gauge, &["name"], json!("The Inquisition's Alert Level Gauge")), "Name is longer than 32"),
        ("a rule effect naming an unknown meter", gauge.clone(), with(&gauge, &["rules", "2", "effects", "0"], json!({"effect": "meter_add", "meter": "heat", "delta": 5})), "unknown meter 'heat'"),
        ("a rule then item with two keys", clock.clone(), with(&clock, &["rules", "1", "then", "0"], json!({"vignette": "v_x", "battle": "raid"})), "exactly one of vignette, battle or dialogue"),
    ];
    for (tag, good, bad, needle) in cases {
        assert_one(&v, "meters", &good, &bad, needle, tag);
    }
    drop_scratch(&data);
}

#[test]
fn each_bad_checkpoint_or_trigger_field_is_one_error() {
    let data = campaign_world("rows");
    let v = store::view(&data);
    let (cp, gate) = (checkpoints()[0].clone(), checkpoints()[1].clone());
    let (act1, wiped, raid) = (triggers()[0].clone(), triggers()[1].clone(), triggers()[2].clone());
    let cases: Vec<(&str, &str, Value, Value, &str)> = vec![
        ("checkpoints", "both quest and met_when", cp.clone(), with(&cp, &["met_when"], json!([{"key": "ward.x"}])), "not both"),
        ("checkpoints", "neither quest nor met_when", cp.clone(), with(&cp, &["quest"], json!("")), "Set a quest or Met when."),
        ("checkpoints", "an unknown quest", cp.clone(), with(&cp, &["quest"], json!("q_ghost")), "unknown quest 'q_ghost'"),
        ("checkpoints", "end_run without a card", cp.clone(), with(&cp, &["card"], json!("")), "needs a card"),
        ("checkpoints", "go_on without effects", gate.clone(), with(&gate, &["effects"], json!([])), "needs effects"),
        ("checkpoints", "warn days rising", cp.clone(), with(&cp, &["warn_days"], json!([1, 3, 7])), "descending"),
        ("checkpoints", "act 0", cp.clone(), with(&cp, &["act"], json!(0)), "Act must be 1..9."),
        ("checkpoints", "deadline 1000", cp.clone(), with(&cp, &["deadline"], json!(1000)), "Deadline day must be 1..999."),
        ("triggers", "an encounter with on day", raid.clone(), with(&without(&raid, "outcome"), &["on"], json!("day")), "Encounter is for on battle only."),
        ("triggers", "an outcome with on enter", act1.clone(), with(&act1, &["outcome"], json!("lost")), "Outcome is for on battle only."),
        ("triggers", "a location with on battle", act1.clone(), with(&act1, &["on"], json!("battle")), "Location is for on enter only."),
        ("triggers", "no effects and no then", wiped.clone(), with(&wiped, &["effects"], json!([])), "needs effects or then"),
        ("triggers", "an effect that opens a screen", wiped.clone(), with(&wiped, &["effects", "0"], json!({"effect": "fight", "encounter": "raid"})), "opens a screen"),
        ("triggers", "a then battle unknown", raid.clone(), with(&raid, &["then", "0"], json!({"battle": "ghost"})), "unknown encounter 'ghost'"),
        ("triggers", "a then vignette unknown (STORYUI1-S)", act1.clone(), with(&act1, &["then", "0", "vignette"], json!("v_ghost")), "unknown vignette 'v_ghost'"),
        ("triggers", "a speaker on a vignette", act1.clone(), with(&act1, &["then", "0", "speaker"], json!("tess")), "speaker is for a dialogue only."),
        ("triggers", "a when on an unknown meter", act1.clone(), with(&act1, &["when", "0", "key"], json!("meter.heat")), "names no meter"),
        ("triggers", "clock_start on a gauge", act1.clone(), with(&act1, &["effects", "2", "meter"], json!("alert")), "is not a countdown"),
        ("triggers", "meter_set on a countdown", raid.clone(), with(&raid, &["effects", "0", "meter"], json!("doom_clock")), "is not a gauge"),
        ("triggers", "meter_set out of range", raid.clone(), with(&raid, &["effects", "0", "value"], json!(101)), "value must be a whole number in the meter's min..max."),
        ("triggers", "set_act 10", act1.clone(), with(&act1, &["effects", "0", "act"], json!(10)), "act must be a whole number 0..9."),
        ("triggers", "an end kind unknown", wiped.clone(), with(&wiped, &["effects", "0", "kind"], json!("win")), "kind 'win'"),
        ("triggers", "clock_start for 0 days", act1.clone(), with(&act1, &["effects", "2", "days"], json!(0)), "days must be a whole number 1..999."),
    ];
    for (kind, tag, good, bad, needle) in cases {
        assert_one(&v, kind, &good, &bad, needle, tag);
    }
    drop_scratch(&data);
}

#[test]
fn campaign_effects_and_read_keys_are_checked_in_dialogue() {
    let data = campaign_world("dialogue");
    let v = store::view(&data);
    let d = tess();
    let fx = ["nodes", "0", "choices", "0", "effects"];
    let req = ["nodes", "0", "choices", "1", "requires"];
    let rest = ["nodes", "0", "choices", "1", "effects"];
    let cases: Vec<(&str, Value, &str)> = vec![
        ("a choice meter_add naming an unknown meter", with(&d, &[&fx[..], &["1", "meter"]].concat(), json!("heat")), "unknown meter 'heat'"),
        ("a meter_add of 0", with(&d, &[&fx[..], &["1", "delta"]].concat(), json!(0)), "delta must be a whole number"),
        ("a condition on meter.nope", with(&d, &[&req[..], &["0", "key"]].concat(), json!("meter.nope")), "'meter.nope' names no meter of this world."),
        ("a meter key with an unknown suffix", with(&d, &[&req[..], &["0", "key"]].concat(), json!("meter.alert.lean")), "is not a read key"),
        ("an unknown campaign key", with(&d, &[&req[..], &["1", "key"]].concat(), json!("campaign.week")), "is not a read key"),
        ("set_flag on a reserved key", with(&d, &fx, json!([{"effect": "set_flag", "key": "meter.alert", "value": 5}])), "is reserved"),
        ("add_flag on a reserved key", with(&d, &fx, json!([{"effect": "add_flag", "key": "roster.living", "delta": 1}])), "is reserved"),
        ("an end before another effect", with(&d, &fx, json!([{"effect": "end", "kind": "bad_end", "card": "c"}, {"effect": "take", "gold": 1}])), "end must be the last effect."),
        ("a campaign effect on a node", with(&d, &["nodes", "0", "effects"], json!([{"effect": "rest"}])), "Node effects may only be set_flag, add_flag or give."),
        ("an unknown key on rest", with(&d, &rest, json!([{"effect": "rest", "days": 2}])), "unknown key 'days'"),
    ];
    for (tag, bad, needle) in cases {
        assert_one(&v, "dialogues", &d, &bad, needle, tag);
    }
    // the unknown-effect message lists the campaign effects after the dialogue ones (and STORYUI1's story effects)
    let unknown = added(&v, "dialogues", &d, &with(&d, &[&fx[..], &["1", "effect"]].concat(), json!("zz")));
    assert!(unknown.iter().any(|m| m.contains("recruit, vignette, sequence, open_board, meter_add, meter_set, meter_show, clock_start, clock_set, clock_stop, set_act, rest, end)")), "{unknown:?}");
    drop_scratch(&data);
}

#[test]
fn a_reserved_key_in_a_flag_list_is_refused() {
    let data = campaign_world("lists");
    let v = store::view(&data);
    let q = json!({"id": "q_cages", "name": "The Cages"});
    assert_one(&v, "quests", &q, &with(&q, &["complete_flags"], json!([{"key": "campaign.act", "value": 2}])), "is reserved", "complete_flags write");
    let route = json!({"id": "r1", "name": "R1", "from": "hub", "to": "street"});
    assert_one(&v, "routes", &route, &with(&route, &["requires_flags"], json!([{"key": "meter.heat", "op": ">=", "value": 1}])), "names no meter", "a route gate");
    assert!(added(&v, "routes", &route, &with(&route, &["requires_flags"], json!([{"key": "meter.alert.running", "value": true}]))).is_empty(), "a meter read key");
    drop_scratch(&data);
}

#[test]
fn each_bad_manifest_block_is_one_error() {
    let data = campaign_world("manifest");
    let v = store::view(&data);
    let m = manifest();
    let errors = |m: &Value| check::campaign::manifest_errors(&v, W, m.as_object().unwrap());
    let cases: Vec<(&str, Value, &str)> = vec![
        ("start_act 10", with(&m, &["campaign", "start_act"], json!(10)), "start_act must be a whole number 0..9."),
        ("act_saves not a bool", with(&m, &["campaign", "act_saves"], json!("yes")), "act_saves must be true or false."),
        ("an unknown campaign key", with(&m, &["campaign", "acts"], json!(3)), "unknown key 'acts'"),
        ("campaign a list", with(&m, &["campaign"], json!([0])), "Campaign must be an object."),
        ("a currency name over 24", with(&m, &["purse", "currency", "name"], json!("Salvaged Iron Scrap Pieces")), "name is longer than 24"),
        ("an empty upkeep short", with(&m, &["purse", "upkeep", "short"], json!(" ")), "short must not be empty."),
        ("per_day 6", with(&m, &["purse", "upkeep", "per_day"], json!(6)), "per_day must be a whole number 0..5."),
        ("road_pct 50", with(&m, &["purse", "upkeep", "at_zero", "road_pct"], json!(50)), "road_pct must be a whole number 100..300."),
        ("start_ap +1", with(&m, &["purse", "upkeep", "at_zero", "start_ap"], json!(1)), "start_ap must be a whole number -5..0."),
        ("rest days 4", with(&m, &["rest", "days"], json!(4)), "days must be a whole number 0..3."),
        ("rest cost 11", with(&m, &["rest", "cost"], json!(11)), "cost must be a whole number 0..10."),
        ("rest where camp", with(&m, &["rest", "where"], json!("camp")), "where 'camp' is not one of nodes, rest_places, none."),
        ("a retreat without to", with(&m, &["defeat", "to"], json!("")), "a retreat needs \"to\""),
        ("a retreat to an unknown location", with(&m, &["defeat", "to"], json!("nowhere")), "unknown location 'nowhere'"),
        ("defeat mode flee", with(&m, &["defeat", "mode"], json!("flee")), "mode 'flee'"),
        ("defeat days 4", with(&m, &["defeat", "days"], json!(4)), "days must be a whole number 0..3."),
    ];
    assert_eq!(errors(&m), Vec::<String>::new(), "the section 4.1 blocks");
    for (tag, bad, needle) in cases {
        let found = errors(&bad);
        assert_eq!(found.len(), 1, "{tag}: {found:?}");
        assert!(found[0].contains(needle), "{tag}: {found:?} (wanted '{needle}')");
    }
    let wired = check::world::manifest_errors(&store::view(&data), W);
    assert!(!wired.iter().any(|e| e.starts_with("Campaign") || e.starts_with("Purse") || e.starts_with("Rest") || e.starts_with("Defeat")), "{wired:?}");
    drop_scratch(&data);
}

#[test]
fn the_manifest_blocks_read_with_their_defaults() {
    let empty = Dict::new();
    let c = CampaignManifest::from_obj(&empty);
    assert!(!c.on, "{{}} = no campaign (X1)");
    assert_eq!((c.start_act, c.act_saves), (0, true));
    let p = Purse::from_obj(&empty);
    assert_eq!((p.currency.name.as_str(), p.currency.icon.as_str(), p.upkeep.name.as_str(), p.upkeep.short.as_str(), p.upkeep.icon.as_str()), ("Gold", "ui/hud/gold.png", "Supplies", "Supplies", "ui/hud/supplies.png"));
    assert_eq!((p.per_day, p.road_pct, p.start_ap), (1, 150, 0), "P2-P3 defaults: a day costs 1, x1.5 road fights, no AP change");
    assert_eq!(RestRule::from_obj(&empty), RestRule { days: 0, cost: 3, place: "nodes".into(), on_enter: true, needs_upkeep: true });
    assert_eq!(DefeatRule::from_obj(&empty), DefeatRule { mode: "wake".into(), to: String::new(), days: 0 });
    let m = dict(manifest());
    let block = |k: &str| json::dget(&m, k).clone();
    assert!(CampaignManifest::from_obj(&block("campaign")).on);
    let p = Purse::from_obj(&block("purse"));
    assert_eq!((p.currency.name.as_str(), p.upkeep.name.as_str(), p.upkeep.short.as_str(), p.road_pct, p.start_ap), ("Scrap", "Warded Tallow", "Tallow", 100, -2));
    assert_eq!(Purse::from_obj(&dict(json!({"upkeep": {"name": "Rations"}}))).upkeep.short, "Rations", "short defaults to the name");
    assert_eq!(RestRule::from_obj(&block("rest")), RestRule { days: 1, cost: 1, place: "rest_places".into(), on_enter: false, needs_upkeep: false });
    assert_eq!(DefeatRule::from_obj(&block("defeat")), DefeatRule { mode: "retreat".into(), to: "hub".into(), days: 1 });
}

#[test]
fn the_writer_stores_the_new_fields_and_kinds() {
    let data = campaign_world("writer");
    // a manifest that sets none of the new fields keeps them out of its file on an unrelated write (WORLDW1)
    store::set_world(&data, "plain", &dict(json!({"name": "Plain"})), true).expect("a plain world");
    let full = fs::read_to_string(aegis_content::world::manifest_path(&data, "plain")).unwrap();
    assert!(full.contains("\"campaign\": {}"), "a full write lists every field");
    fs::write(aegis_content::world::manifest_path(&data, "plain"), json!({"id": "plain", "name": "Plain", "extends": [], "connects": [], "version": 1}).to_string()).unwrap();
    store::set_world(&data, "plain", &dict(json!({"tagline": "Flat."})), false).expect("an update");
    let text = fs::read_to_string(aegis_content::world::manifest_path(&data, "plain")).unwrap();
    for key in ["campaign", "purse", "rest", "defeat"] {
        assert!(!text.contains(&format!("\"{key}\"")), "{key} stays out while it holds its default: {text}");
    }
    // a manifest block is written, and a bad one refused
    store::set_world(&data, "plain", &dict(json!({"campaign": {"start_act": 1}})), false).expect("a campaign block");
    assert_eq!(json::dget(&source::load(&data).worlds["plain"]["manifest"].as_object().cloned().unwrap(), "campaign")["start_act"], json!(1));
    let refused = store::set_world(&data, "plain", &dict(json!({"campaign": {"start_act": 12}})), false).err().expect("start_act 12 refused");
    assert!(refused.iter().any(|e| e.contains("start_act must be a whole number 0..9.")), "{refused:?}");
    // a meter row through upsert; a re-save without changes is byte-identical; a bad shape is refused
    let heat = dict(json!({"id": "heat", "name": "Heat", "bands": [{"id": "low", "name": "Low", "from": 0, "tone": "calm"}]}));
    store::upsert(&data, Some(W), "meters", &heat, None).expect("a meter row");
    let file = store::file_of(&data, Some(W), "meters");
    let stored = fs::read_to_string(&file).unwrap();
    assert_eq!(store::resave_text(&data, Some(W), "meters").unwrap(), stored);
    assert!(source::load(&data).worlds[W]["own"]["meters"].as_array().unwrap().iter().any(|m| m["id"] == "heat"));
    let bad = dict(json!({"id": "bad", "name": "Bad", "bands": {"low": 0}}));
    let refused = store::upsert(&data, Some(W), "meters", &bad, None).err().expect("bands as an object refused");
    assert!(refused.iter().any(|e| e.contains("Bands must be a list.")), "{refused:?}");
    assert_eq!(fs::read_to_string(&file).unwrap(), stored, "rolled back");
    for kind in ["checkpoints", "triggers"] {
        let text = fs::read_to_string(store::file_of(&data, Some(W), kind)).unwrap();
        let again = store::resave_text(&data, Some(W), kind).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&again).unwrap()["entries"].as_array().map(Vec::len), serde_json::from_str::<Value>(&text).unwrap()["entries"].as_array().map(Vec::len), "{kind}");
    }
    drop_scratch(&data);
}
