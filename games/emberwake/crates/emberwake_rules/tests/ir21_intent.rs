//! INTENT2-K1a (A4.2, IR21; READY row 16, FB-57) on the shipped skills: every row derives a 1-or-2 intent list
//! (`rpgkit_rules::intent::derive`), the same from the raw row and from the loaded skill. The dual-intent rows are the
//! SUPPORT rows that may target their user (`Support, Stand`) and the IR21-shaped attacks the rule makes dual
//! (`Field, Assault`). W2-4 (FB-114) migrated the 7 legacy SWEEP rows per FB-101 Option A (4 ARC, 3 RING) and retired
//! SWEEP, so the shipped data has 26 = 8 + 18 dual rows and the FB-101 per-intent totals directly; R2-W2b CLASS-BASICS
//! (FB-133) added three class basics, one row per intent shape (a Maneuver, a self-targetable SUPPORT and a PLUS
//! attack at range), so 28 = 9 + 19; PR5-7 (PK1-S3) added five single-intent Channel test skills (2 Stand, 3 Assault); PR5-19 (PK2-S6) added nine (the
//! self-targetable SUPPORT `purge_marrow`, so 29 = 10 + 19, and eight single-target attacks in Assault). Read-only: the rows are game data, so the test lives in the game's façade crate.

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
    // The rule's own reading of each dual row (not the derive's code path).
    for r in &rows {
        let o = r.as_object().unwrap();
        let kind = o.get("kind").and_then(Value::as_str).unwrap_or_default();
        let mode = o.get("target_mode").and_then(Value::as_str).unwrap_or_default();
        let ss = kind == "SUPPORT" && mode.is_empty();
        assert_eq!(support_stand.contains(&id(r)), ss, "{}: Support,Stand = SUPPORT that may target its user", id(r));
    }
    assert_eq!(support_stand.len(), 10, "READY row 16: 8 Support,Stand + tend_wounds (CLASS-BASICS) + purge_marrow (PR5-19 PK2-S6)");
    assert!(field_assault.iter().all(|f| ["LINE", "PLUS", "BURST", "ARC", "CONE"].contains(&area(rows.iter().find(|r| id(r) == *f).unwrap()))));
    assert_eq!(field_assault.len(), 19, "14 Field,Assault + the 4 ARC rows of the SWEEP split (FB-101, W2-4) + cinder_spark (CLASS-BASICS)");
}

/// FB-101 Option A: the former SWEEP rows, now ARC and RING (`INTENT2_SKILLS.md`, migrated in W2-4).
const SPLIT_ARC: [&str; 4] = ["foundry_shatter", "crushing_judgment", "gilded_billhook_sunder", "pneumatic_hammer_strike"];
const SPLIT_RING: [&str; 3] = ["sweep_kick", "cyclone_kick", "tendril_flail"];

/// How many rows list `intent` (a dual row counts for both of its intents).
fn listing(rows: &[Value], intent: &str) -> usize {
    rows.iter().filter(|r| derive(r).contains(&intent)).count()
}

/// How many rows have `intent` first (the primary, IR15).
fn primary(rows: &[Value], intent: &str) -> usize {
    rows.iter().filter(|r| derive(r).first() == Some(&intent)).count()
}

#[test]
fn the_fb101_split_ships_with_its_shapes_and_no_sweep_remains() {
    let rows = rows();
    assert!(rows.iter().all(|r| area(r) != "SWEEP"), "W2-4 retired SWEEP");
    for (ids, shape, want) in [(&SPLIT_ARC[..], "ARC", &[FIELD, ASSAULT][..]), (&SPLIT_RING[..], "RING", &[STAND][..])] {
        for i in ids {
            let r = rows.iter().find(|r| id(r) == *i).unwrap_or_else(|| panic!("{i} shipped"));
            assert_eq!(area(r), shape, "{i}: FB-101 shape");
            assert_eq!(derive(r), want, "{i}: IR21 by its shape");
        }
    }
}

#[test]
fn shipped_data_gives_28_dual_and_the_intent_totals() {
    let rows = rows();
    assert_eq!(rows.len(), 147, "IR21 covers all 147 skills (129 + brace, W2-4 slice 4 / FB-119, + 3 class basics, CLASS-BASICS, + 5 PK1-S3 Channel test skills, + 9 PR5-19 PK2-S6 graft and technique skills)");
    let brace = rows.iter().find(|r| id(r) == "brace").expect("brace shipped");
    assert_eq!(derive(brace), [STAND], "brace (DEFEND, guard) is single-intent Stand: the dual counts stay");
    // CLASS-BASICS (FB-133): each new class basic derives by its shape, one per intent list
    for (i, want) in [("disengage_step", &[MANEUVER][..]), ("tend_wounds", &[SUPPORT, STAND][..]), ("cinder_spark", &[FIELD, ASSAULT][..])] {
        let r = rows.iter().find(|r| id(r) == i).unwrap_or_else(|| panic!("{i} shipped"));
        assert_eq!(derive(r), want, "{i}: IR21 by its row");
    }
    // PR5-7 (PK1-S3): the five Channel test skills derive single-intent by their shape: the two SELF-only basics
    // (feed SUPPORT, vent FOCUS) Stand, the three single-target attacks Assault; the dual counts stay
    for (i, want) in [("feed", STAND), ("vent", STAND), ("marrow_lash", ASSAULT), ("marrow_bolt", ASSAULT), ("strain_bite", ASSAULT)] {
        let r = rows.iter().find(|r| id(r) == i).unwrap_or_else(|| panic!("{i} shipped"));
        assert_eq!(derive(r), [want], "{i}: IR21 by its row");
    }
    // PR5-19 (PK2-S6): the self-targetable SUPPORT purge_marrow is Support,Stand; the two graft attacks and the six
    // technique test attacks (single target, no shape) are Assault
    assert_eq!(derive(rows.iter().find(|r| id(r) == "purge_marrow").expect("purge_marrow shipped")), [SUPPORT, STAND], "purge_marrow: IR21 by its row");
    for i in ["marrow_rend", "marrow_litany", "tq_polearm_hook", "tq_crossbow_volley", "tq_fist_breaker", "tq_dagger_flurry", "tq_censer_swing", "tq_sprayer_jet"] {
        let r = rows.iter().find(|r| id(r) == i).unwrap_or_else(|| panic!("{i} shipped"));
        assert_eq!(derive(r), [ASSAULT], "{i}: IR21 by its row");
    }
    let ss = ids_with(&rows, &[SUPPORT, STAND]).len();
    let fa = ids_with(&rows, &[FIELD, ASSAULT]).len();
    let dual = rows.iter().filter(|r| derive(r).len() == 2).count();
    assert_eq!((dual, ss, fa), (29, 10, 19), "29 = 10 Support,Stand + 19 Field,Assault");
    let totals: Vec<usize> = [ASSAULT, FIELD, STAND, SUPPORT, MANEUVER, NEUTRAL].iter().map(|i| primary(&rows, i)).collect();
    assert_eq!(totals, [70, 24, 19, 14, 15, 5], "FB-101 totals by primary intent (+ brace in Stand, + the 3 class basics in Field, Support, Maneuver, + the 5 PK1-S3 skills: 3 in Assault, 2 in Stand, + the 9 PK2-S6 skills: 8 in Assault, 1 in Support): Assault, Field, Stand, Support, Maneuver, Neutral");
    assert_eq!(totals.iter().sum::<usize>(), rows.len(), "every skill has exactly one primary");
    let listed: Vec<usize> = [ASSAULT, FIELD, STAND, SUPPORT, MANEUVER, NEUTRAL].iter().map(|i| listing(&rows, i)).collect();
    assert_eq!(listed, [89, 24, 29, 14, 15, 5], "a dual row lists under both intents: 19 Field,Assault add to Assault, 10 Support,Stand to Stand");
    assert_eq!(listed.iter().sum::<usize>(), rows.len() + dual, "each dual row counts twice");
}

#[test]
fn fb101_confirmations_self_support_and_shaped_attacks_by_rule() {
    let rows = rows();
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
