//! CLN-39: the balance sim's golden, the check of its printed tables (BAL2_SIM.md modes). Each command line of `RUNS`
//! (every mode, the keys that change what a mode prints, an argument error with the help text) runs on frozen data and
//! must print exactly the lines of `tests/fixtures/sim_golden.json`.
//!
//! The frozen data: the catalog is the timeline golden's (`emberwake_battle/tests/fixtures/timeline_kit.json`, shared
//! with CLN-38's rules golden); the worlds and the game data (maps, quests, services, XP curve, item families) are
//! `tests/fixtures/sim_data.json`, frozen from the shipped data's loader output when CLN-39 landed (decision "Godot
//! parity retired" Q-A6). The golden was recorded once from the engine (CLN-39, 2026-11-01; it replaced the Godot sim
//! recordings of `games/emberwake/parity/sim.sh`). Re-record only for an intended rule or sim output change, named in
//! the commit and a docs/HISTORY.md row; never edit it by hand (delete `sim_data.json` first to re-freeze the worlds
//! and game data from the shipped data):
//!
//! `RECORD=1 tools/fc.sh test --test sim_golden`
//!
//! `SIM_GOLDEN=<n,n>` (1-based) runs only those lines. The command lines and the frozen worlds are game data, so the
//! test lives in the game's façade crate (the kit names no game).

use emberwake_world::Data;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The command lines: the 20 of the retired Godot recordings (run 11 now names both world encounters with `world:`, so
/// it fights them; the Godot line was an argument error), then the two modes without a Godot original.
const RUNS: [&str; 22] = [
    "duel 3 1",
    "duel 5 1 preset=both score=hp band=40-60",
    "duel 5 1 metrics=on loadout=role distance=3 spend=mixed",
    "tier 5 1",
    "tier 3 1 tier_foe=warrior quick",
    "levels 5 1 score=binary",
    "group 5 1 jitter=off",
    "stats all spend=all",
    "stats 5 loadout=newest",
    "party 5 2 enc=mob,line,wave,elite,boss",
    "party 3 1 comp=warrior,priest enc=world:ash_patrol,world:ash_raiders elevel=+1 pouch=con_heal_potion",
    "party 5 1 comp=warrior,ranger,mage enc=all metrics=off",
    "sweep 3 1 size=2 enc=line metrics=off",
    "sweep 5 1 size=3 lead=warrior enc=mob,boss phase=off",
    "role 5 quick",
    "role 7 1 role=support class=priest",
    "campaign 5 1 comp=warrior,priest,mage heal=rally",
    "campaign 3 1 comp=warrior,ranger,rogue heal=25 rest=3 extra=4:ash_raiders",
    "duel 5 1 bad=1",
    "party 5 1 comp=warrior,knight",
    "feel 5 1 enc=tl_duel_windup,tl_skirmish party=both",
    "deadlines",
];

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixtures() -> PathBuf {
    crate_dir().join("tests/fixtures")
}

fn recording() -> bool {
    std::env::var("RECORD").is_ok_and(|v| v == "1")
}

fn read_json(path: &Path, what: &str) -> Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("missing {what} {}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()))
}

/// The frozen worlds and game data: read from the fixture; when recording without one, frozen first from the shipped
/// data's loader output (rows in file order, so the file is the same on every machine). Once per test binary, so the
/// tests running in parallel never read a half-written file.
fn frozen_worlds_and_game() -> &'static Value {
    static FROZEN: OnceLock<Value> = OnceLock::new();
    FROZEN.get_or_init(freeze_or_read)
}

fn freeze_or_read() -> Value {
    let path = fixtures().join("sim_data.json");
    if recording() && !path.is_file() {
        let dir = Data::find_data_dir(&crate_dir()).expect("data folder");
        let src = emberwake_content::source::load(&dir);
        assert!(src.errors.is_empty(), "the shipped data has load errors: {:?}", src.errors);
        std::fs::create_dir_all(fixtures()).unwrap();
        let frozen = json!({"worlds": src.worlds, "game": src.game});
        std::fs::write(&path, serde_json::to_string(&frozen).unwrap() + "\n").unwrap();
        eprintln!("froze the worlds and game data to {}", path.display());
    }
    read_json(&path, "the frozen worlds and game data")
}

fn frozen_data() -> Data {
    let catalog = read_json(&crate_dir().join("../emberwake_battle/tests/fixtures/timeline_kit.json"), "the frozen catalog");
    let wg = frozen_worlds_and_game();
    Data::from_values(&catalog, &wg["worlds"], &wg["game"])
}

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

fn picked() -> Option<Vec<usize>> {
    std::env::var("SIM_GOLDEN").ok().map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
}

#[test]
fn sim_prints_its_golden() {
    let d = frozen_data();
    let path = fixtures().join("sim_golden.json");
    if recording() {
        let runs: Vec<Value> = RUNS.iter().map(|line| json!({"argv": line, "lines": emberwake_sim::run(&d, &argv(line))})).collect();
        std::fs::write(&path, serde_json::to_string_pretty(&json!({"runs": runs})).unwrap() + "\n").unwrap();
        eprintln!("recorded {} sim runs to {}", RUNS.len(), path.display());
        return;
    }
    let golden = read_json(&path, "the sim golden");
    let runs = golden["runs"].as_array().expect("runs");
    let recorded: Vec<&str> = runs.iter().map(|r| r["argv"].as_str().unwrap_or("")).collect();
    assert_eq!(recorded, RUNS.to_vec(), "the golden's command lines are RUNS (re-record after changing RUNS)");
    let only = picked();
    let mut failures = Vec::new();
    for (i, run) in runs.iter().enumerate() {
        if only.as_ref().is_some_and(|o| !o.contains(&(i + 1))) {
            continue;
        }
        let line = RUNS[i];
        let want: Vec<&str> = run["lines"].as_array().expect("lines").iter().map(|l| l.as_str().unwrap_or("")).collect();
        let got = emberwake_sim::run(&d, &argv(line));
        match want.iter().zip(&got).position(|(w, g)| *w != g.as_str()) {
            Some(k) => failures.push(format!("run {} ({line}) line {}:\n  golden {}\n  now    {}", i + 1, k + 1, want[k], got[k])),
            None if want.len() != got.len() => failures.push(format!("run {} ({line}): golden {} lines, now {}", i + 1, want.len(), got.len())),
            None => {}
        }
    }
    assert!(failures.is_empty(), "{} of the {} sim runs differ from the golden:\n{}", failures.len(), runs.len(), failures.join("\n"));
}

/// The golden covers every mode the sim has (a new mode adds a run).
#[test]
fn the_golden_runs_every_mode() {
    for mode in emberwake_sim::args::MODES {
        assert!(RUNS.iter().any(|l| l.split_whitespace().next() == Some(mode)), "no golden run of mode {mode}");
    }
}

/// The argument-error run prints the help text, so the golden holds the whole help (each key the sim takes).
#[test]
fn the_error_run_prints_the_help() {
    let d = frozen_data();
    let out = emberwake_sim::run(&d, &argv("duel 5 1 bad=1"));
    assert_eq!(out.first().map(String::as_str), Some("SIM argument error: unknown key 'bad'"));
    assert_eq!(out[1..].to_vec(), emberwake_sim::args::usage(&emberwake_sim::game()));
}
