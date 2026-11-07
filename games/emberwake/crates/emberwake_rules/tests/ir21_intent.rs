//! INTENT2-K1a (A4.2, IR21; READY row 16, FB-57) on the shipped skills: every row derives a 1-or-2 intent list
//! (`rpgkit_rules::intent::derive`), the same from the raw row and from the loaded skill. The dual-intent rows are the
//! SUPPORT rows that may target their user (`Support, Stand`) and the IR21-shaped attacks the rule makes dual
//! (`Field, Assault`). The shipped data still has its 7 legacy SWEEP rows (the SWEEP -> ARC/RING split is data work
//! for W2-4), which derive as unshaped attacks: 22 = 8 + 14 dual rows raw. FB-101 Option A applies the named split
//! in memory (4 ARC, 3 RING) and checks 26 = 8 + 18 and the per-intent totals. Read-only: the rows are game data, so
//! the test lives in the game's façade crate.

use emberwake_data::Catalog;
use emberwake_rules::intent::{self, IntentInput, ASSAULT, FIELD, MANEUVER, STAND, SUPPORT};
use emberwake_rules::ruleset::NEUTRAL;
use serde_json::{json, Map, Value};
use std::path::PathBuf;

fn catalog_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog")
}

/// The shipped skill rows, raw.
fn rows() -> Vec<Value> {
    let path = catalog_dir().join("skills.json");
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing {}", path.display()))).unwrap();
    v.get("entries").and_then(Value::as_array).cloned().expect("skills.json entries")
}

fn id(r: &Value) -> String {
    r.get("id").and_then(Value::as_str).unwrap_or_default().to_string()
}

fn area(r: &Value) -> &str {
    r.get("area").and_then(Value::as_str).unwrap_or_default()
}

fn derive(r: &Value) -> Vec<&'static str> {
    intent::derive(&IntentInput::of_row(r.as_object().expect("a row")))
}

/// The ids whose derived list is exactly `want`.
fn ids_with(rows: &[Value], want: &[&str]) -> Vec<String> {
    rows.iter().filter(|r| derive(r) == want).map(id).collect()
}

/// The option (c) hook: the rows with the given ids re-shaped (in memory only).
fn reshaped(rows: &[Value], ids: &[String], shape: &str) -> Vec<Value> {
    rows.iter()
        .map(|r| {
            let mut r = r.clone();
            if ids.contains(&id(&r)) {
                r.as_object_mut().unwrap().insert("area".into(), json!(shape));
            }
            r
        })
        .collect()
}

#[test]
fn every_shipped_skill_derives_one_or_two_intents_the_same_from_row_and_def() {
    let rows = rows();
    let mut root = Map::new();
    root.insert("skills".into(), Value::Array(rows.clone()));
    let cat = Catalog::from_value(&Value::Object(root));
    for r in &rows {
        let from_row = derive(r);
        assert!((1..=2).contains(&from_row.len()), "{}: {from_row:?}", id(r));
        let def = cat.skills.iter().find(|s| s.id == id(r)).unwrap_or_else(|| panic!("{} not loaded", id(r)));
        assert_eq!(intent::derive(&IntentInput::of_def(def)), from_row, "{}: row vs loaded skill", id(r));
    }
}

#[test]
fn shipped_dual_intent_breakdown_by_rule() {
    let rows = rows();
    let support_stand = ids_with(&rows, &[SUPPORT, STAND]);
    let field_assault = ids_with(&rows, &[FIELD, ASSAULT]);
    let sweeps: Vec<String> = rows.iter().filter(|r| area(r) == "SWEEP").map(id).collect();
    // The rule's own reading of each dual row (not the derive's code path).
    for r in &rows {
        let o = r.as_object().unwrap();
        let kind = o.get("kind").and_then(Value::as_str).unwrap_or_default();
        let mode = o.get("target_mode").and_then(Value::as_str).unwrap_or_default();
        let ss = kind == "SUPPORT" && mode.is_empty();
        assert_eq!(support_stand.contains(&id(r)), ss, "{}: Support,Stand = SUPPORT that may target its user", id(r));
    }
    assert_eq!(support_stand.len(), 8, "READY row 16: 8 Support,Stand");
    assert!(sweeps.iter().all(|s| derive(rows.iter().find(|r| id(r) == *s).unwrap()) == [ASSAULT]), "legacy SWEEP derives as an unshaped attack");
    assert!(field_assault.iter().all(|f| ["LINE", "PLUS", "BURST", "ARC", "CONE"].contains(&area(rows.iter().find(|r| id(r) == *f).unwrap()))));
    assert_eq!(field_assault.len() + sweeps.len(), 21, "14 Field,Assault + 7 SWEEP awaiting the ARC/RING split (FB-101, W2-4)");
}

#[test]
fn sweep_split_hook_moves_rows_by_shape() {
    let rows = rows();
    let sweeps: Vec<String> = rows.iter().filter(|r| area(r) == "SWEEP").map(id).collect();
    let fa = ids_with(&rows, &[FIELD, ASSAULT]).len();
    let stand = ids_with(&rows, &[STAND]).len();
    let as_arc = reshaped(&rows, &sweeps, "ARC");
    assert_eq!(ids_with(&as_arc, &[FIELD, ASSAULT]).len(), fa + sweeps.len(), "every SWEEP as ARC turns dual");
    let as_ring = reshaped(&rows, &sweeps, "RING");
    assert_eq!(ids_with(&as_ring, &[FIELD, ASSAULT]).len(), fa, "RING is never dual");
    assert_eq!(ids_with(&as_ring, &[STAND]).len(), stand + sweeps.len(), "every SWEEP as RING turns Stand");
}

/// FB-101 Option A: the SWEEP rows that become ARC and RING (`INTENT2_SKILLS.md`), applied in memory until W2-4.
const SWEEP_TO_ARC: [&str; 4] = ["foundry_shatter", "crushing_judgment", "gilded_billhook_sunder", "pneumatic_hammer_strike"];
const SWEEP_TO_RING: [&str; 3] = ["sweep_kick", "cyclone_kick", "tendril_flail"];

fn owned(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// The shipped rows with the FB-101 split applied (in memory only).
fn split_rows(rows: &[Value]) -> Vec<Value> {
    reshaped(&reshaped(rows, &owned(&SWEEP_TO_ARC), "ARC"), &owned(&SWEEP_TO_RING), "RING")
}

/// How many rows list `intent` (a dual row counts for both of its intents).
fn listing(rows: &[Value], intent: &str) -> usize {
    rows.iter().filter(|r| derive(r).contains(&intent)).count()
}

/// How many rows have `intent` first (the primary, IR15).
fn primary(rows: &[Value], intent: &str) -> usize {
    rows.iter().filter(|r| derive(r).first() == Some(&intent)).count()
}

#[test]
fn raw_shipped_data_has_22_dual_intent_rows() {
    let rows = rows();
    assert_eq!(rows.len(), 129, "IR21 covers all 129 skills");
    assert_eq!(ids_with(&rows, &[SUPPORT, STAND]).len(), 8);
    assert_eq!(ids_with(&rows, &[FIELD, ASSAULT]).len(), 14, "before the SWEEP split");
    assert_eq!(rows.iter().filter(|r| derive(r).len() == 2).count(), 22, "22 = 8 + 14");
}

#[test]
fn fb101_split_in_memory_gives_26_dual_and_the_intent_totals() {
    let raw = rows();
    let sweeps: Vec<String> = raw.iter().filter(|r| area(r) == "SWEEP").map(id).collect();
    let mut named = owned(&SWEEP_TO_ARC);
    named.extend(owned(&SWEEP_TO_RING));
    let (mut a, mut b) = (sweeps.clone(), named.clone());
    a.sort();
    b.sort();
    assert_eq!(a, b, "the FB-101 ids are exactly the shipped SWEEP rows");
    let rows = split_rows(&raw);
    assert!(rows.iter().all(|r| area(r) != "SWEEP"));
    for r in &rows {
        let want: &[&str] = if SWEEP_TO_ARC.contains(&id(r).as_str()) {
            &[FIELD, ASSAULT]
        } else if SWEEP_TO_RING.contains(&id(r).as_str()) {
            &[STAND]
        } else {
            continue;
        };
        assert_eq!(derive(r), want, "{} after the split", id(r));
    }
    let ss = ids_with(&rows, &[SUPPORT, STAND]).len();
    let fa = ids_with(&rows, &[FIELD, ASSAULT]).len();
    let dual = rows.iter().filter(|r| derive(r).len() == 2).count();
    assert_eq!((dual, ss, fa), (26, 8, 18), "26 = 8 Support,Stand + 18 Field,Assault");
    let totals: Vec<usize> = [ASSAULT, FIELD, STAND, SUPPORT, MANEUVER, NEUTRAL].iter().map(|i| primary(&rows, i)).collect();
    assert_eq!(totals, [59, 23, 16, 12, 14, 5], "FB-101 totals by primary intent: Assault, Field, Stand, Support, Maneuver, Neutral");
    assert_eq!(totals.iter().sum::<usize>(), rows.len(), "every skill has exactly one primary");
    let listed: Vec<usize> = [ASSAULT, FIELD, STAND, SUPPORT, MANEUVER, NEUTRAL].iter().map(|i| listing(&rows, i)).collect();
    assert_eq!(listed, [77, 23, 24, 12, 14, 5], "a dual row lists under both intents: 18 Field,Assault add to Assault, 8 Support,Stand to Stand");
    assert_eq!(listed.iter().sum::<usize>(), rows.len() + dual, "each dual row counts twice");
}

#[test]
fn fb101_confirmations_self_support_and_shaped_attacks_by_rule() {
    let rows = split_rows(&rows());
    let taunt = rows.iter().find(|r| id(r) == "taunt_shout").expect("taunt_shout shipped");
    assert_eq!(taunt.get("target_mode").and_then(Value::as_str), Some("SELF"));
    assert_eq!(derive(taunt), [STAND], "a SELF-only SUPPORT derives Stand only");
    for r in rows.iter().filter(|r| r.get("kind").and_then(Value::as_str) == Some("ATTACK")) {
        let o = r.as_object().unwrap();
        if o.get("dash_tiles").and_then(Value::as_i64).unwrap_or(0) > 0 || o.get("position").and_then(Value::as_str).is_some_and(|p| !p.is_empty()) {
            continue;
        }
        let hazard = o.get("hazard").and_then(Value::as_object).is_some_and(|h| !h.is_empty());
        let reach = o.get("reach_max").and_then(Value::as_i64).unwrap_or(0);
        let want: &[&str] = match area(r) {
            "ARC" | "LINE" | "CONE" if !hazard => &[FIELD, ASSAULT],
            "BURST" | "PLUS" if !hazard && reach > 1 => &[FIELD, ASSAULT],
            "BURST" | "PLUS" | "ARC" | "LINE" | "CONE" => &[FIELD],
            "RING" => &[STAND],
            _ => continue,
        };
        assert_eq!(derive(r), want, "{}: {} reach {reach} hazard {hazard}", id(r), area(r));
    }
    // No shipped BURST/PLUS has reach <= 1 without a hazard: shrink each dual one in memory.
    let wide: Vec<&Value> = rows.iter().filter(|r| ["BURST", "PLUS"].contains(&area(r)) && derive(r) == [FIELD, ASSAULT]).collect();
    assert!(!wide.is_empty());
    for r in wide {
        let mut near = r.clone();
        near.as_object_mut().unwrap().insert("reach_max".into(), json!(1));
        assert_eq!(derive(&near), [FIELD], "{} at reach 1 without a hazard derives Field only", id(r));
    }
}
