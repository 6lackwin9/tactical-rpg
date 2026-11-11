//! RULESET1-G2 (RULESET1_BALANCE1 4.2, decision BREPORT1: one source for the report and the sims): the host's
//! `autoplay` (the battle screen's autoplay and the CLI `battle` draft through the same `Session::auto_draft`) and the
//! sim's `feel` mode (jitter off) fight each timeline test encounter the same way: for seeds 11, 23 and 35 they give the
//! same outcome and rounds. Reads the shipped data (no number of it); the CLI note for a timeline test fight played
//! classic is checked on rows built here.

use emberwake_host::cli::{classic_test_fight_note, TIMELINE_TEST_PREFIX};
use emberwake_host::{Battle, Host};
use emberwake_world::Data;
use serde_json::{json, Value};
use std::sync::Arc;

const SEEDS: [i64; 3] = [11, 23, 35];

fn shipped() -> Option<Data> {
    let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")))?;
    Data::open(&dir).ok().map(|(d, _)| d)
}

/// The host's verdict of one autoplayed fight: (outcome, rounds); "" = capped (the round limit, both sides standing).
fn host_fight(d: &Arc<Data>, id: &str, seed: i64) -> (String, i64) {
    let mut host = Host::new(Battle::with_data(d.clone()));
    let r = host.handle(&json!({"cmd": "new", "encounter": id, "seed": seed, "ruleset": "timeline"}));
    assert_eq!(r["ok"], json!(true), "{id}: {r}");
    let r = host.handle(&json!({"cmd": "autoplay"}));
    assert_eq!(r["ok"], json!(true), "{id}: {r}");
    let state = &r["state"];
    (state["outcome"].as_str().unwrap_or("").to_string(), state["round"].as_i64().unwrap_or(0))
}

/// The sim's verdict of the same fight (feel, jitter off, one seed).
fn sim_fight(d: &Data, id: &str, seed: i64) -> (String, i64) {
    let a = emberwake_sim::parse(d, &["feel".into(), format!("enc={id}"), "jitter=off".into(), "seeds=1".into()]).expect("feel arguments");
    let mut r = emberwake_sim::runner::Runner::new(d, &a);
    let c = emberwake_sim::feel::fight_encounter(&mut r, id, &[seed]);
    let outcome = [("won", c.won), ("lost", c.lost), ("fled", c.fled)].iter().find(|(_, n)| *n > 0).map_or("", |(o, _)| o);
    (outcome.to_string(), c.rounds.first().copied().unwrap_or(0) as i64)
}

#[test]
fn host_autoplay_and_sim_feel_fight_alike() {
    let Some(d) = shipped() else {
        eprintln!("no shipped data folder found");
        return;
    };
    let d = Arc::new(d);
    let mut compared = 0;
    for id in emberwake_sim::game().feel_encounters.iter().filter(|e| d.cat.encounters.contains_key(*e)) {
        for seed in SEEDS {
            assert_eq!(host_fight(&d, id, seed), sim_fight(&d, id, seed), "{id} seed {seed}: host autoplay vs sim feel (outcome, rounds)");
            compared += 1;
        }
    }
    assert!(compared > 0, "the game's feel encounters are in the data");
}

#[test]
fn battle_notes_a_timeline_test_fight_played_classic() {
    let row = |party: &[&str], enemies: &[&str]| -> serde_json::Map<String, Value> { serde_json::from_value(json!({"party": party, "enemies": enemies})).unwrap() };
    let t = |s: &str| format!("{TIMELINE_TEST_PREFIX}{s}");
    let test_fight = row(&[&t("hero")], &[&t("foe"), &t("foe")]);
    let note = classic_test_fight_note("x", &test_fight, false).expect("every unit a test unit, no ruleset=: a note");
    assert!(note.contains("ruleset=timeline"), "{note}");
    assert_eq!(classic_test_fight_note("x", &test_fight, true), None, "ruleset= given: no note");
    assert_eq!(classic_test_fight_note("x", &row(&[&t("hero")], &["raider"]), false), None, "a unit that is not a test unit: no note");
    assert_eq!(classic_test_fight_note("x", &row(&[], &[]), false), None, "no units: no note");
}
