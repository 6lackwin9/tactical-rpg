//! emberwake (formerly aegis-gui): the game's screens on the Aegis engine's window shell (`aegis_ui`). A stack of
//! screens (main menu, new game, slots, settings, region map with dialogue and shops, world map, game menu, character,
//! journal, battle results, battle); the agent link (TCP, `aegis link`) also reaches the battle command API while a
//! battle is open. The desktop program is `src/main.rs` (`run_native`); the web build is this library as a `cdylib`
//! (`web`, WEBREL1-G1), which reads the same launch keys from the page's URL query.
//!   emberwake [screen=menu|arena|runner] [encounter=goblin_skirmish] [ruleset=<id>] [seed=N] [port=47800] [data=<export dir>] [saves=<dir>]
//!             [headless=WxH [fps=N] [dpr=X] [idle_exit=S]]
//! `ruleset=`: the ruleset `screen=arena` plays by (default the encounter's own). `headless=`: no window and no sound
//! device; the frames stay in memory and the agent link draws them out (`aegis_ui::shell::headless`).
//! `screen=arena` and `screen=runner` need the dev tools (cargo feature `dev_tools`, default on; `DEV_TOOLS`).
//! Keys: arrows / W A S D move, E interact, Esc back or game menu, Enter confirm, F12 screenshot; battle keys as listed
//! in battle.rs. Right click goes back.

// A build without the dev tools (the web release build) leaves the items that only the Scene Runner and its screens
// reach (arena starts, samples, gallery reels, view pickers) unused; the linker drops them. The default build (dev tools
// on: the desktop, the gate) keeps the full dead-code check.
#![cfg_attr(not(feature = "dev_tools"), allow(dead_code, unused_imports))]

mod battle;
mod board;
#[cfg(feature = "dev_tools")]
mod dojo;
#[cfg(feature = "dev_tools")]
mod forge;
mod fx;
#[cfg(feature = "dev_tools")]
mod gallery;
mod ground;
mod hud;
mod hub;
mod intent_text;
mod mapview;
mod menus;
mod music;
mod party;
mod portraits;
mod props;
mod recipe;
mod region;
#[cfg(feature = "dev_tools")]
mod runner;
mod runover;
mod screens;
mod shop;
mod stage;
mod story;
mod talk;
mod timeline_view;
mod tuning;
mod viewkeys;
#[cfg(target_arch = "wasm32")]
mod web;
mod worldmap;

use aegis_content::manifest::{self, Manifest};
use aegis_ui::art;
use aegis_ui::audio::Audio;
use aegis_ui::screen::Settings;
use aegis_ui::shell::headless::Headless;
use aegis_ui::shell::Options;
use aegis_ui::tuning::TuningFile;
use anyhow::{Context, Result};
use emberwake_world::GAME_ID;
use screens::{default_settings, Cx, GameCx, Screen};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

/// The dev tools are in this build (WEBREL1-G1 4: cargo feature `dev_tools`, default on; the web test build has them,
/// the web release build does not): the main menu's Tools (the Scene Runner: Combat Arena, Content and World Forge
/// viewers, Gear Dojo, UI Gallery), the `screen=arena` and `screen=runner` launches and the debug build's dev keys.
pub const DEV_TOOLS: bool = cfg!(feature = "dev_tools");

/// The debug build's dev keys (F5, F6, F7, F9, F10, F11; VIEWKEYS1, EXPLOREVIEW1, TURNFEEL1): a debug build with the
/// dev tools. A release build takes the keys and does nothing, as before.
pub(crate) const DEV_KEYS: bool = cfg!(all(debug_assertions, feature = "dev_tools"));

fn arg<'a>(args: &'a [(String, String)], k: &str) -> Option<&'a str> {
    args.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
}

/// STAGE1-C, EXPLOREVIEW1: the battle and region views are Scene Runner debug settings; the game itself always shows
/// the Stage and the square map (until the view review picks one).
fn force_shipped_views(settings: &mut serde_json::Value, screen: Option<&str>) {
    if screen != Some("runner") {
        settings["battle_view"] = serde_json::json!("stage");
        settings["region_view"] = serde_json::json!("square");
    }
}

/// `ruleset=`: only `screen=arena` plays by one a launch names; anywhere else it would be ignored, so it is refused.
fn arena_ruleset<'a>(args: &'a [(String, String)]) -> Result<Option<&'a str>> {
    match (arg(args, "ruleset"), arg(args, "screen")) {
        (Some(_), screen) if screen != Some("arena") => anyhow::bail!("ruleset= comes with screen=arena"),
        (ruleset, _) => Ok(ruleset),
    }
}

/// Where the game's folders are looked for (`launch`): `start`, the game folder, the repository or a folder inside
/// either, and the engine folder (its `assets/fonts`), when found.
struct Roots {
    start: PathBuf,
    engine: Option<PathBuf>,
}

/// The desktop's roots: the engine folder found upward from the working folder, else from the program's own folder;
/// `start` is the engine folder, else the working folder.
#[cfg(not(target_arch = "wasm32"))]
fn native_roots() -> Result<Roots> {
    let cwd = std::env::current_dir()?;
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf));
    let engine = [Some(cwd.clone()), exe_dir].into_iter().flatten().find_map(|start| {
        let mut d = start;
        loop {
            if aegis_core::vfs::is_dir(d.join("crates/aegis_ui")) {
                return Some(d);
            }
            if aegis_core::vfs::is_dir(d.join("engine/crates/aegis_ui")) {
                return Some(d.join("engine"));
            }
            if !d.pop() {
                return None;
            }
        }
    });
    Ok(Roots { start: engine.clone().unwrap_or(cwd), engine })
}

/// The game's context, its first screens and the shell's options for the launch keys `args` (`ruleset` already
/// checked by `arena_ruleset`), with the folders found from `roots`. The same on the desktop and the web; only the
/// roots and where the keys come from differ.
fn launch(args: &[(String, String)], roots: Roots, ruleset: Option<&str>, headless: Option<Headless>) -> Result<(Cx, Vec<Box<dyn Screen<GameCx>>>, Options)> {
    let Roots { start, engine: engine_dir } = roots;
    let game_dir = manifest::find_game_dir(&start, GAME_ID);
    let data_dir = arg(args, "data").map(PathBuf::from).or_else(|| emberwake_world::find_data_dir(&start)).context("data folder not found: pass data=<dir> (a data folder or a Godot export folder)")?;
    let (data, mut notices) = emberwake_world::Data::open(&data_dir).with_context(|| format!("loading the data in {}", data_dir.display()))?;
    let data = Arc::new(data);
    let manifest = match game_dir.as_ref().map(|g| Manifest::read(&g.join(manifest::FILE))) {
        Some(Ok(m)) => m,
        found => {
            notices.push(match found {
                Some(Err(e)) => e,
                _ => format!("Game folder games/{GAME_ID} (game.json) not found."),
            });
            Manifest { id: GAME_ID.into(), title: GAME_ID.into(), ..Manifest::default() }
        }
    };
    let project = game_dir.clone().or_else(|| art::find_project(&start));
    let home = game_dir.unwrap_or_else(|| PathBuf::from("."));
    let engine = engine_dir.unwrap_or_else(|| PathBuf::from("."));
    let save_dir = match arg(args, "saves") {
        Some(dir) => PathBuf::from(dir),
        None => {
            if let Err(e) = aegis_save::adopt_shared(GAME_ID) {
                notices.push(format!("Copying the saves into {}: {e}", aegis_save::user_dir(GAME_ID).display()));
            }
            aegis_save::user_dir(GAME_ID).join("saves")
        }
    };
    let title = format!("Aegis - {}", manifest.title);
    let (icon, canvas, window_sizes) = (manifest.icon.clone(), manifest.canvas.clone(), manifest.window_sizes.clone());
    let game = GameCx {
        data,
        manifest,
        game: None,
        save_dir,
        tuning: TuningFile::new(Some(home.join("tuning.json"))),
        log: Vec::new(),
        encounter: arg(args, "encounter").unwrap_or("goblin_skirmish").to_string(),
        seed: arg(args, "seed").and_then(|s| s.parse().ok()),
        debug: false,
        data_dir,
    };
    let mut settings = Settings::load(aegis_save::user_dir(GAME_ID).join("settings.json"), default_settings());
    settings.keep_window_size(&screens::WINDOW_SIZES);
    force_shipped_views(&mut settings.v, arg(args, "screen"));
    // Playtest recordings are never on in the web build (WEBREL1 6.1, `vfs::dev::ON`), so its data is never hashed.
    let record = aegis_core::vfs::dev::ON && settings.v["record_playtests"].as_bool().unwrap_or(cfg!(debug_assertions));
    // The data fingerprint a recording carries; hashed only while recording is on.
    let content = if record { aegis_content::fingerprint::of(&game.data_dir) } else { serde_json::Value::Null };
    let layers = project.as_ref().map(|p| aegis_core::assets::Layers::for_game(p)).unwrap_or_default();
    #[allow(unused_mut)]
    let mut cx = Cx {
        playtest: aegis_ui::playtest::Playtests::new(aegis_save::user_dir(GAME_ID).join("playtests"), record, content),
        settings,
        art: art::Art::new(project),
        // ASSETORG2: sounds and music over the same layers as the art (game, kit, engine)
        audio: if headless.is_some() { Audio::offline_layers(&layers) } else { Audio::of_layers(&layers) },
        nav: Vec::new(),
        notice: notices,
        shot_dir: home.join(".shots"),
        quit: false,
        answers: Vec::new(),
        held: Default::default(),
        app: game,
    };
    #[allow(unused_mut)]
    let mut stack: Vec<Box<dyn Screen<GameCx>>> = vec![Box::new(menus::MainMenu::default())];
    match arg(args, "screen") {
        #[cfg(feature = "dev_tools")]
        Some("arena") => match ruleset {
            Some(ruleset) => {
                emberwake_rules::ruleset::Ruleset::of(&cx.app.data.cat, ruleset).map_err(anyhow::Error::msg)?;
                let encounter = cx.app.encounter.clone();
                stack.push(Box::new(battle::BattleScreen::arena_with(&cx, None, &encounter, Some(ruleset.to_string()), "", false)));
            }
            None => stack.push(Box::new(battle::BattleScreen::sandbox(&cx))),
        },
        #[cfg(feature = "dev_tools")]
        Some("runner") => stack.push(Box::new(runner::SceneRunner::new())),
        #[cfg(not(feature = "dev_tools"))]
        Some(screen @ ("arena" | "runner")) => {
            let _ = ruleset;
            cx.notice.push(format!("screen={screen} needs the dev tools (this build has none): the main menu opens."));
        }
        _ => {}
    }
    let port = arg(args, "port").and_then(|p| p.parse().ok()).unwrap_or(aegis_ui::bridge::DEFAULT_PORT);
    let options = Options { title, font: engine.join("assets/fonts/OpenSans-SemiBold.ttf"), port, icon, canvas, window_sizes, headless };
    Ok((cx, stack, options))
}

/// The desktop program (`src/main.rs`): the launch keys from the command line, the folders found from the working
/// folder or the program's, a window (or none, `headless=`).
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() -> Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<(String, String)> = raw.iter().filter_map(|a| a.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect();
    let headless = Headless::from_args(&args)?;
    let ruleset = arena_ruleset(&args)?;
    let roots = native_roots()?;
    let (cx, stack, options) = launch(&args, roots, ruleset, headless)?;
    aegis_ui::shell::run(cx, stack, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_launch_ruleset_comes_with_the_arena_only() {
        let args = |list: &[(&str, &str)]| list.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<Vec<_>>();
        assert_eq!(arena_ruleset(&args(&[("screen", "arena"), ("ruleset", "r")])).unwrap(), Some("r"));
        assert_eq!(arena_ruleset(&args(&[("screen", "arena")])).unwrap(), None);
        assert_eq!(arena_ruleset(&args(&[])).unwrap(), None);
        assert!(arena_ruleset(&args(&[("ruleset", "r")])).is_err());
        assert!(arena_ruleset(&args(&[("screen", "runner"), ("ruleset", "r")])).is_err());
    }

    /// EXPLOREVIEW1 check 9: outside the Scene Runner the debug views never show; in it they stay as saved.
    #[test]
    fn the_game_forces_the_shipped_views_outside_the_scene_runner() {
        for screen in [None, Some("menu"), Some("arena")] {
            let mut v = json!({"battle_view": "flat", "region_view": "tilted"});
            force_shipped_views(&mut v, screen);
            assert_eq!((v["battle_view"].as_str(), v["region_view"].as_str()), (Some("stage"), Some("square")), "{screen:?}");
        }
        let mut v = json!({"battle_view": "flat", "region_view": "tilted"});
        force_shipped_views(&mut v, Some("runner"));
        assert_eq!((v["battle_view"].as_str(), v["region_view"].as_str()), (Some("flat"), Some("tilted")));
    }
}
