//! CLN-45: the switches the round intent rule used to bundle are ruleset row fields (decision "Godot parity retired
//! and cleanup pass 3 answers" Q-B2). The shipped rows play as before whether or not they set the fields yet: the
//! `timeline` row (with `round_intent`) reads the round intent rule's choices and the `classic` row the legacy ones
//! (the built-in classic values). The seed `data/seeds/cln45.json` sets the fields to exactly those values: applied to
//! a scratch copy of the data, every row reads as before bar the rules HOOKS1-A appended later to `timeline`, and the
//! file re-saves byte for byte; `data/seeds/hooks1a.json` then `data/seeds/s22s3.json` (S2.2-S3: `poise_mode` pips)
//! over it give the shipped row back. The unit deck orders `data/seeds/w24k1b.json` set by derived intent rows (valid
//! only under `field_intent`) are set aside during the replay and re-applied after `w24s1.json`; `w25lock.json`
//! (ATTACKLOCK1-O: `attack_lock` melee) replays next, then `classbasics.json` (R2-W2b CLASS-BASICS: role basics move
//! to the classes, `flank_cut` leaves the `timeline` basics, four founder deck orders change) replays next, in its
//! order, then the unit steps of `r41s2.json` (R4-1 slice 2: `template` on the caster units) and `r42.json` (R4-2 L2:
//! `brace` leaves `founder_kaelen`'s extra skills), then the `rulesets` step of `pk1s3.json` (PR5-7 PK1-S3: `channel`
//! appended to the `timeline` rules with `channel_meter` strain and `channel_rest` feed), then the `rulesets` step of
//! `pk2s6.json` (PR5-19 PK2-S6: the extension rule `durability` added to the `timeline` rules), then the unit steps
//! of `sunkfix1.json` (PR5-34 SUNKWARD-FIX1: `founder_maren` gains `thurible_smite` in her extra skills, active deck
//! and `assault` order) last.

use aegis_core::json::{self, Dict};
use emberwake_content::{source, store};
use emberwake_rules::ruleset::{self, AttackLock, Chase, PoiseMode, RestAp, Ruleset, AI_TACTICS, ARENA_HAZARDS, BY_RULES_FIELDS, CHANNEL, CHANNEL_REST_NORMAL, CLASSIC, DURABILITY, EXECUTE, FORCED_MOVE, GROUND_HAZARDS, HP_COST, IGNORE_GUARD, ROUND_INTENT, TIMELINE, TRIGGERS, UNIT_METERS};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// HOOKS1_TASKS 2.4: what `rulesets.timeline.rules` appends, in its order.
const HOOKS1_RULES: [&str; 9] = [FORCED_MOVE, GROUND_HAZARDS, ARENA_HAZARDS, TRIGGERS, UNIT_METERS, HP_COST, IGNORE_GUARD, AI_TACTICS, EXECUTE];

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn switches(r: &Ruleset) -> (RestAp, Chase, bool, bool, bool) {
    (r.rest_ap(), r.chase(), r.root_stops_chase(), r.switch_target_step(), r.stances())
}

/// The loaded ruleset rows of the data folder `dir`, by id.
fn rows(dir: &Path) -> Vec<(String, Ruleset)> {
    let v = store::view(dir);
    v.entries(ruleset::KIND).iter().map(|e| (json::s(e, "id"), Ruleset::from_row(e))).collect()
}

/// The ruleset rows the seed `name` upserts, as given.
fn seed_entries(name: &str) -> Vec<Dict> {
    seed_rows(name, ruleset::KIND)
}

/// The rows of kind `kind` the seed `name` upserts, as given.
fn seed_rows(name: &str, kind: &str) -> Vec<Dict> {
    let text = std::fs::read_to_string(data_dir().join("seeds").join(name)).expect("the seed");
    let seed: Value = serde_json::from_str(&text).expect("seed JSON");
    json::arr(&seed["steps"]).iter().map(json::dict).filter(|s| json::s(s, "kind") == kind).map(|s| json::dget(s, "entry").clone()).collect()
}

/// The unit decks the seed `w24k1b.json` (A4.2-K1b) re-ordered by derived intent rows (a `field` list, a ring in
/// `guard`): those orders are valid only under `field_intent`, so while the pre-W2-4 ruleset seeds replay they are
/// set aside (an empty order, valid under either reading) and the seed puts them back after `w24s1.json`.
const DERIVED_ORDERS_SEED: &str = "w24k1b.json";

/// R2-W2b CLASS-BASICS: the seed that moved the role basics to the classes; it re-orders some decks the derived-row
/// seed set, so those units ship with its deck.
const CLASS_BASICS_SEED: &str = "classbasics.json";

/// R4-1 slice 2 (FB-144): the seed that set `template` on the caster units (after the class-basics seed).
const TEMPLATES_SEED: &str = "r41s2.json";
/// R4-2 (FB-138 L2): the seed that drops the duplicate class basic `brace` from `founder_kaelen.extra_skills`.
const R42_SEED: &str = "r42.json";

/// PR5-7 (PK1-S3): the seed that appended `channel` to the `timeline` rules and set its `channel_meter` and
/// `channel_rest` (after the R4-2 seed).
const PK1S3_SEED: &str = "pk1s3.json";

/// PR5-19 (PK2-S6): the seed that added the extension rule `durability` to the `timeline` rules (after the PK1-S3
/// seed). The loader keeps the rules in the register's order, so where the seed lists it does not matter here.
const PK2S6_SEED: &str = "pk2s6.json";

/// PR5-34 (SUNKWARD-FIX1): the seed that gave `founder_maren` the censer attack `thurible_smite` (extra skills, active
/// deck, `assault` order) and added the unit `tithe_captive` (after the PK2-S6 seed; no ruleset row changes).
const SUNKFIX1_SEED: &str = "sunkfix1.json";

/// PR5-41 (ACT1-DATA1): the seed that re-applied the unit `tithe_captive` with `flee_ratio` 0.0 (a captive no longer
/// flees; after the SUNKWARD-FIX1 seed; no ruleset row changes). A later seed's row supersedes an earlier seed's.
const ACT1FIX1_SEED: &str = "act1fix1.json";

/// What the class-basics seed dropped from the `timeline` basics (now the Rogue's class basic).
const CLASS_BASICS_DROPPED: &str = "flank_cut";

/// Every step of the seed `name` as (kind, entry), in its order.
fn seed_steps(name: &str) -> Vec<(String, Dict)> {
    let text = std::fs::read_to_string(data_dir().join("seeds").join(name)).expect("the seed");
    let seed: Value = serde_json::from_str(&text).expect("seed JSON");
    json::arr(&seed["steps"]).iter().map(json::dict).map(|s| (json::s(s, "kind"), json::dget(s, "entry").clone())).collect()
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_owned();
        if p.is_dir() {
            copy_dir(&p, &to.join(name));
        } else {
            std::fs::copy(&p, to.join(name)).unwrap();
        }
    }
}

#[test]
fn the_shipped_rows_play_as_before() {
    let shipped = rows(&data_dir());
    assert!(!shipped.is_empty(), "the shipped rulesets");
    let intent_choices = (RestAp::RoundEnd, Chase::EverySkill, true, false, false);
    for (id, r) in &shipped {
        let want = if r.has(ROUND_INTENT) { intent_choices } else { switches(&Ruleset::classic()) };
        assert_eq!(switches(r), want, "ruleset '{id}'");
    }
    let classic = shipped.iter().find(|(id, _)| id == CLASSIC).map(|(_, r)| r).expect("the classic row");
    assert_eq!(classic, &Ruleset::classic(), "classic battles unchanged");
}

#[test]
fn the_seed_sets_the_values_the_rows_read_now() {
    let before = rows(&data_dir());
    let entries = seed_entries("cln45.json");
    assert_eq!(entries.len(), before.len(), "one upsert per shipped row");
    for e in &entries {
        assert!(BY_RULES_FIELDS.iter().all(|k| !json::s(e, k).is_empty()), "the seed sets every switch of '{}'", json::s(e, "id"));
    }
    let tmp = aegis_core::scratch::Scratch::vacant("emberwake_cln45_seed");
    for sub in ["catalog", "worlds", "maps", "quests", "dialogue", "progression"] {
        copy_dir(&data_dir().join(sub), &tmp.join(sub));
    }
    // the units the derived-row seed re-ordered, as they stood before it as far as the ruleset rows care: no order
    let shipped_units = store::view(&data_dir());
    let reordered = seed_rows(DERIVED_ORDERS_SEED, "units");
    assert!(!reordered.is_empty(), "the derived-row seed re-orders unit decks");
    let class_basics = seed_steps(CLASS_BASICS_SEED);
    let last_unit = |id: &str| class_basics.iter().rev().find(|(k, e)| k == "units" && json::s(e, "id") == id).map(|(_, e)| e);
    for u in &reordered {
        let id = json::s(u, "id");
        let mut e = shipped_units.get("units", &id).clone();
        let last = last_unit(&id).unwrap_or(u);
        assert_eq!(e.get("deck"), last.get("deck"), "unit '{id}' ships with the deck the last seed leaves");
        if let Some(Value::Object(deck)) = e.get_mut("deck") {
            deck.shift_remove("order");
        }
        store::upsert(&tmp, None, "units", &e, None).unwrap_or_else(|err| panic!("unit '{id}' without its order: {err:?}"));
    }
    for e in &entries {
        store::upsert(&tmp, None, ruleset::KIND, e, None).unwrap_or_else(|err| panic!("seed row '{}': {err:?}", json::s(e, "id")));
    }
    // PR5-19 (PK2-S6): the seed pk2s6 added the durability rule to the timeline rules later (no number changes)
    let pk2s6 = seed_entries(PK2S6_SEED);
    assert!(!pk2s6.is_empty(), "the PK2-S6 seed upserts a ruleset row");
    let mut pre_pk2s6 = before.clone();
    for (id, r) in pre_pk2s6.iter_mut().filter(|(id, _)| id == TIMELINE) {
        assert!(r.rules.contains(&DURABILITY), "ruleset '{id}' lists the durability rule PK2-S6 adds");
        r.rules.retain(|x| *x != DURABILITY);
    }
    // PR5-7 (PK1-S3): the seed pk1s3 appended channel to the timeline rules and set its channel meter and Rest later
    let pk1s3 = seed_entries(PK1S3_SEED);
    assert!(!pk1s3.is_empty(), "the PK1-S3 seed upserts a ruleset row");
    let mut pre_pk1s3 = pre_pk2s6.clone();
    for (id, r) in pre_pk1s3.iter_mut().filter(|(id, _)| id == TIMELINE) {
        assert_eq!(r.rules.last().copied(), Some(CHANNEL), "ruleset '{id}' ends with the channel rule PK1-S3 appends");
        r.rules.pop();
        r.numbers.channel_meter = String::new();
        r.numbers.channel_rest = CHANNEL_REST_NORMAL.to_string();
    }
    // R2-W2b CLASS-BASICS: the seed classbasics dropped flank_cut from the timeline basics later
    let mut pre_class_basics = pre_pk1s3.clone();
    for (_, r) in pre_class_basics.iter_mut().filter(|(id, _)| id == TIMELINE) {
        assert!(!r.numbers.basics.iter().any(|b| b == CLASS_BASICS_DROPPED), "the shipped timeline basics leave {CLASS_BASICS_DROPPED} to the classes");
        r.numbers.basics.push(CLASS_BASICS_DROPPED.to_string());
    }
    let mut want = pre_class_basics.clone();
    for (id, r) in want.iter_mut().filter(|(id, _)| id == TIMELINE) {
        let (kept, added) = r.rules.split_at(r.rules.len() - HOOKS1_RULES.len());
        assert_eq!(added, HOOKS1_RULES, "ruleset '{id}' ends with what HOOKS1 appends");
        // W2-4 slice 1 (FB-106): the seed w24s1 added field_intent later
        r.rules = kept.iter().filter(|x| **x != ruleset::FIELD_INTENT).cloned().collect();
        // S2.2-S3: the seed s22s3 set poise_mode pips later
        r.numbers.poise_mode = PoiseMode::Bar;
        // R1-BAL H6 (FB-98): the seed r1bal set stagger_windup_mult 2.0 -> 1.0 later
        r.numbers.stagger_windup_mult = 2.0;
        // A1.2-S1 (FB-55): the seed a12s1 set item_ap 0 later
        r.numbers.item_ap = 1;
        // A1.2-K3 (W2-4 slice 7a): the seed w24k3r set the three item switches on later
        r.numbers.item_mitigation = false;
        r.numbers.thrown_miss_spends = false;
        r.numbers.pouch_stacks = false;
        // ATTACKLOCK1-O (W2-5 slice 2, FB-58a): the seed w25lock set attack_lock none -> melee later
        r.numbers.attack_lock = AttackLock::None;
    }
    assert_eq!(rows(&tmp), want, "every row reads as before, bar the rules HOOKS1 appended later");
    let file = store::file_of(&tmp, None, ruleset::KIND);
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(BY_RULES_FIELDS.iter().all(|k| text.contains(&format!("\"{k}\""))), "the switches are written");
    assert_eq!(store::resave_text(&tmp, None, ruleset::KIND).unwrap(), text, "the seeded file re-saves byte for byte");
    assert!(source::load(&tmp).errors.iter().all(|e| !e.contains("rulesets.json")), "the seeded rows load clean");
    for e in seed_entries("hooks1a.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("HOOKS1-A row '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_entries("s22s3.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("S2.2-S3 row '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_entries("r1bal.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("R1-BAL row '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_entries("a12s1.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("A1.2-S1 row '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_entries("w24s1.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("W2-4 slice 1 row '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_entries("w24k3r.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("A1.2-K3 row '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_entries("w25lock.json") {
        store::upsert(&tmp, None, ruleset::KIND, &e, None).unwrap_or_else(|err| panic!("ATTACKLOCK1-O row '{}': {err:?}", json::s(&e, "id")));
    }
    assert_eq!(rows(&tmp), pre_class_basics, "the HOOKS1-A, S2.2-S3, R1-BAL, A1.2-S1, W2-4 slice 1, A1.2-K3 and ATTACKLOCK1-O rows over it give the row before CLASS-BASICS");
    // A4.2-K1b: under field_intent the derived-row seed's orders are valid again and give the shipped units back
    for u in &reordered {
        store::upsert(&tmp, None, "units", u, None).unwrap_or_else(|err| panic!("W2-4 slice 6a unit '{}': {err:?}", json::s(u, "id")));
    }
    // R2-W2b CLASS-BASICS: the class-basics seed, in its order, gives the shipped rows and units back
    for (kind, e) in &class_basics {
        store::upsert(&tmp, None, kind, e, None).unwrap_or_else(|err| panic!("CLASS-BASICS {kind} '{}': {err:?}", json::s(e, "id")));
    }
    assert_eq!(rows(&tmp), pre_pk1s3, "the CLASS-BASICS seed over it gives the rows before PK1-S3");
    // R4-1 slice 2: the template seed's units (their stat template; no ruleset row changes)
    for e in seed_rows(TEMPLATES_SEED, "units") {
        store::upsert(&tmp, None, "units", &e, None).unwrap_or_else(|err| panic!("R4-1 unit '{}': {err:?}", json::s(&e, "id")));
    }
    for e in seed_rows(R42_SEED, "units") {
        store::upsert(&tmp, None, "units", &e, None).unwrap_or_else(|err| panic!("R4-2 unit '{}': {err:?}", json::s(&e, "id")));
    }
    // PR5-7 (PK1-S3): the seed's rulesets step gives the shipped rows back
    for e in &pk1s3 {
        store::upsert(&tmp, None, ruleset::KIND, e, None).unwrap_or_else(|err| panic!("PK1-S3 row '{}': {err:?}", json::s(e, "id")));
    }
    assert_eq!(rows(&tmp), pre_pk2s6, "the PK1-S3 rulesets step over it gives the rows before PK2-S6");
    // PR5-19 (PK2-S6): the seed's rulesets step gives the shipped rows back
    for e in &pk2s6 {
        store::upsert(&tmp, None, ruleset::KIND, e, None).unwrap_or_else(|err| panic!("PK2-S6 row '{}': {err:?}", json::s(e, "id")));
    }
    assert_eq!(rows(&tmp), before, "the PK2-S6 rulesets step over it gives the shipped rows");
    // PR5-34 (SUNKWARD-FIX1): the seed's units steps (founder_maren's new skill and deck, the new unit tithe_captive)
    let sunkfix1 = seed_rows(SUNKFIX1_SEED, "units");
    assert!(sunkfix1.iter().any(|e| json::s(e, "id") == "founder_maren"), "the SUNKWARD-FIX1 seed re-decks founder_maren");
    assert!(seed_entries(SUNKFIX1_SEED).is_empty(), "the SUNKWARD-FIX1 seed changes no ruleset row");
    for e in &sunkfix1 {
        store::upsert(&tmp, None, "units", e, None).unwrap_or_else(|err| panic!("SUNKWARD-FIX1 unit '{}': {err:?}", json::s(e, "id")));
    }
    assert_eq!(rows(&tmp), before, "the SUNKWARD-FIX1 units steps leave the shipped ruleset rows");
    // PR5-41 (ACT1-DATA1): the seed's units steps (tithe_captive no longer flees) over SUNKWARD-FIX1's
    let act1fix1 = seed_rows(ACT1FIX1_SEED, "units");
    assert!(seed_entries(ACT1FIX1_SEED).is_empty(), "the ACT1-DATA1 seed changes no ruleset row");
    for e in &act1fix1 {
        store::upsert(&tmp, None, "units", e, None).unwrap_or_else(|err| panic!("ACT1-DATA1 unit '{}': {err:?}", json::s(e, "id")));
    }
    assert_eq!(rows(&tmp), before, "the ACT1-DATA1 units steps leave the shipped ruleset rows");
    let seeded = store::view(&tmp);
    let units = reordered
        .iter()
        .map(|u| json::s(u, "id"))
        .chain(class_basics.iter().filter(|(k, _)| k == "units").map(|(_, e)| json::s(e, "id")))
        .chain(sunkfix1.iter().map(|e| json::s(e, "id")))
        .chain(act1fix1.iter().map(|e| json::s(e, "id")));
    for id in units {
        assert_eq!(seeded.get("units", &id), shipped_units.get("units", &id), "unit '{id}' as shipped");
    }
    let _ = std::fs::remove_dir_all(&tmp);
}
