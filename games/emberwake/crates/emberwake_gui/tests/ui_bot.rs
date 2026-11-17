//! UIBOT (REPLAY-CI1): the engine's seeded UI bot (`aegis_ui::shell::bot`) on every start the game names
//! (`emberwake_gui::BOT_STARTS`), `SEEDS` seeds each: it presses valid widgets for `steps` steps and no step may panic.
//! The game runs headless in this process on a scratch copy of the data folder, with its user folder and saves in a
//! scratch folder, so nothing the bot presses reaches the repo or the player's profile.
//!
//! `AEGIS_BOT_STEPS` sets the steps per run (default `STEPS`); `AEGIS_BOT_SEEDS` the seeds ("4,9" or "1..20"). A start
//! with a known panic stays listed in `KNOWN_PANICS` (start, seed, why): it is still run and reported, not failed, until
//! its bug row lands.

use aegis_ui::shell::bot::Bot;
use std::path::{Path, PathBuf};

const SEEDS: [u64; 3] = [1, 2, 3];
const STEPS: usize = 40;
/// (start, seed, bug note): runs that panic for a known, logged reason.
const KNOWN_PANICS: &[(&str, u64, &str)] = &[];

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dest = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            std::fs::copy(e.path(), &dest).unwrap();
        }
    }
}

fn seeds() -> Vec<u64> {
    match std::env::var("AEGIS_BOT_SEEDS") {
        Err(_) => SEEDS.to_vec(),
        Ok(t) => match t.split_once("..") {
            Some((a, b)) => (a.trim().parse().unwrap()..=b.trim().parse().unwrap()).collect(),
            None => t.split(',').map(|s| s.trim().parse().unwrap()).collect(),
        },
    }
}

#[test]
fn the_ui_bot_presses_every_start_without_a_panic() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let scratch: PathBuf = std::env::temp_dir().join(format!("aegis_ui_bot_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    let data = scratch.join("data");
    copy_dir(&repo.join("games/emberwake/data"), &data);
    // The game's user folder (settings, recordings) is %APPDATA%/Aegis/<game>: here, the scratch folder.
    std::env::set_var("APPDATA", scratch.join("appdata"));
    let steps = std::env::var("AEGIS_BOT_STEPS").ok().and_then(|s| s.parse().ok()).unwrap_or(STEPS);
    let mut failed = Vec::new();
    for (start, _) in emberwake_gui::BOT_STARTS {
        for seed in seeds() {
            let mut bot = Bot::new(seed, steps);
            bot.skip = emberwake_gui::BOT_SKIP.iter().map(|s| s.to_string()).collect();
            let run = emberwake_gui::bot_run(start, &data, &scratch.join(format!("{start}_{seed}")), &bot).unwrap_or_else(|e| panic!("{start}: the game did not start: {e:#}"));
            println!("[bot] {start} {}", run.summary());
            if let Some(p) = &run.panic {
                match KNOWN_PANICS.iter().find(|(s, n, _)| s == start && *n == seed) {
                    Some((_, _, why)) => println!("[bot] {start} seed {seed}: known panic, skipped ({why})"),
                    None => failed.push(format!("{start} seed {seed}: {} at step {} after {} ({}); presses: {}", p.message, p.step, p.action, p.location, run.presses.join(" | "))),
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(failed.is_empty(), "the UI bot hit a panic (repro: AEGIS_BOT_SEEDS=<seed> AEGIS_BOT_STEPS={steps}):\n{}", failed.join("\n"));
}
