//! emberwake (formerly aegis-gui): the game's screens on the Aegis engine's window shell (`aegis_ui`). A stack of
//! screens (main menu, new game, slots, settings, region map with dialogue and shops, world map, game menu, character,
//! journal, battle results, battle); the agent link (TCP, `aegis link`) also reaches the battle command API while a
//! battle is open.
//!   emberwake [screen=menu|arena|runner] [encounter=goblin_skirmish] [seed=N] [port=47800] [data=<export dir>] [saves=<dir>]
//! Keys: arrows / W A S D move, E interact, Esc back or game menu, Enter confirm, F12 screenshot; battle keys as listed
//! in battle.rs. Right click goes back.

mod battle;
mod dojo;
mod forge;
mod hub;
mod menus;
mod party;
mod region;
mod runner;
mod screens;
mod tuning;
mod worldmap;

use aegis_ui::art;
use aegis_ui::screen::Settings;
use aegis_ui::shell::Options;
use aegis_ui::tuning::TuningFile;
use anyhow::{Context, Result};
use screens::{default_settings, Cx, GameCx, Screen};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn arg<'a>(args: &'a [(String, String)], k: &str) -> Option<&'a str> {
    args.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
}

fn app_dir() -> PathBuf {
    std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from(".")).join("Aegis")
}

fn main() -> Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<(String, String)> = raw.iter().filter_map(|a| a.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect();
    let cwd = std::env::current_dir()?;
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf));
    let engine_dir = [Some(cwd.clone()), exe_dir].into_iter().flatten().find_map(|start| {
        let mut d = start;
        loop {
            if d.join("crates/aegis_ui").is_dir() {
                return Some(d);
            }
            if d.join("engine/crates/aegis_ui").is_dir() {
                return Some(d.join("engine"));
            }
            if !d.pop() {
                return None;
            }
        }
    });
    let data_dir = arg(&args, "data").map(PathBuf::from).or_else(|| emberwake_world::Data::find_data_dir(engine_dir.as_deref().unwrap_or(&cwd))).context("data folder not found: pass data=<dir> (a data folder or a Godot export folder)")?;
    let (data, load_errors) = emberwake_world::Data::open(&data_dir).with_context(|| format!("loading the data in {}", data_dir.display()))?;
    let data = Arc::new(data);
    let project = art::find_project(engine_dir.as_deref().unwrap_or(&cwd));
    let engine = engine_dir.unwrap_or_else(|| PathBuf::from("."));
    let save_dir = arg(&args, "saves").map(PathBuf::from).unwrap_or_else(|| app_dir().join("saves"));
    let game = GameCx {
        data,
        game: None,
        save_dir,
        tuning: TuningFile::new(Some(engine.join("data/tuning.json"))),
        log: Vec::new(),
        encounter: arg(&args, "encounter").unwrap_or("goblin_skirmish").to_string(),
        seed: arg(&args, "seed").and_then(|s| s.parse().ok()),
        debug: false,
        data_dir,
    };
    let cx = Cx {
        settings: Settings::load(app_dir().join("settings.json"), default_settings()),
        art: art::Art::new(project),
        nav: Vec::new(),
        notice: load_errors,
        shot_dir: engine.join("parity/out"),
        quit: false,
        answers: Vec::new(),
        app: game,
    };
    let mut stack: Vec<Box<dyn Screen<GameCx>>> = vec![Box::new(menus::MainMenu::default())];
    match arg(&args, "screen") {
        Some("arena") => stack.push(Box::new(battle::BattleScreen::sandbox(&cx))),
        Some("runner") => stack.push(Box::new(runner::SceneRunner::new())),
        _ => {}
    }
    let port = arg(&args, "port").and_then(|p| p.parse().ok()).unwrap_or(aegis_ui::bridge::DEFAULT_PORT);
    let options = Options { title: "Aegis - Tactical RPG".into(), font: engine.join("assets/fonts/OpenSans-SemiBold.ttf"), port };
    aegis_ui::shell::run(cx, stack, options)
}
