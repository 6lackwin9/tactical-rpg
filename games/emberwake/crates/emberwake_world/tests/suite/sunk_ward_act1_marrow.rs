//! PR7-9 (`LORE-DATA1`, ruling `F5`) on the shipped data, read only: Act 1 is Cold Steel only (`CL-02`), so no
//! encounter in the Act 1 region (`sunk_ward_quarter`; Act 2 and 3 get their own regions) grants a marrow-tier skill
//! through `flag_skills`. A marrow-tier skill is any catalog skill that is Channel-only (`channel.only`). The
//! Interlude dose choice still records: `dlg_int_dose` sets `sunk_ward.sabina_dose` and `sunk_ward.sabina_dose_to`
//! in every choice, so Act 2 (`PR6-12`) can grant the skill from that flag.

use serde_json::Value;
use std::collections::BTreeSet;
use std::path::Path;

fn entries(rel: &str) -> Vec<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data").join(rel);
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).expect("read")).expect("json");
    v["entries"].as_array().expect("entries").clone()
}

const ACT1_REGION: &str = "sunk_ward_quarter";

fn marrow_tier_skills() -> BTreeSet<String> {
    entries("catalog/skills.json")
        .into_iter()
        .filter(|s| s["channel"]["only"].as_bool() == Some(true))
        .filter_map(|s| s["id"].as_str().map(str::to_string))
        .collect()
}

#[test]
fn marrow_tier_set_is_not_empty() {
    let skills = marrow_tier_skills();
    assert!(skills.contains("marrow_lash"), "{skills:?}");
    assert!(skills.contains("marrow_bolt"), "{skills:?}");
}

#[test]
fn no_act1_encounter_grants_a_marrow_tier_skill() {
    let marrow = marrow_tier_skills();
    let act1: BTreeSet<String> = entries("worlds/sunk_ward/locations.json")
        .into_iter()
        .filter(|l| l["region"] == ACT1_REGION)
        .filter_map(|l| l["id"].as_str().map(str::to_string))
        .collect();
    let encounters = entries("worlds/sunk_ward/encounters.json");
    let in_act1: Vec<&Value> = encounters
        .iter()
        .filter(|e| e["location"].as_str().is_some_and(|l| act1.contains(l)))
        .collect();
    assert!(!in_act1.is_empty(), "no Act 1 encounters found");
    for e in in_act1 {
        for grant in e["flag_skills"].as_array().into_iter().flatten() {
            let skill = grant["skill"].as_str().unwrap_or_default();
            assert!(!marrow.contains(skill), "Act 1 encounter {} grants marrow-tier skill {skill}", e["id"]);
        }
    }
}

#[test]
fn dose_dialogue_still_sets_the_dose_flags() {
    let dlg = entries("worlds/sunk_ward/dialogues.json")
        .into_iter()
        .find(|d| d["id"] == "dlg_int_dose")
        .expect("dlg_int_dose");
    let mut members = BTreeSet::new();
    let mut dose_choices = 0;
    for node in dlg["nodes"].as_array().expect("nodes") {
        for choice in node["choices"].as_array().into_iter().flatten() {
            let effects = choice["effects"].as_array().cloned().unwrap_or_default();
            let set = |key: &str| {
                effects
                    .iter()
                    .find(|f| f["effect"] == "set_flag" && f["key"] == key)
                    .map(|f| f["value"].clone())
            };
            let Some(to) = set("sunk_ward.sabina_dose_to") else { continue };
            dose_choices += 1;
            assert_eq!(set("sunk_ward.sabina_dose"), Some(Value::Bool(true)), "{choice}");
            members.insert(to.as_str().expect("member id").to_string());
        }
    }
    assert!(dose_choices >= 1, "dlg_int_dose sets no dose flags");
    assert_eq!(members.len(), dose_choices, "one choice per member: {members:?}");
}
