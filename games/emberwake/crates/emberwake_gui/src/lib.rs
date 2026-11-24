//! emberwake (formerly aegis-gui): the game's screens on the Aegis engine's window shell (`aegis_ui`). A stack of
//! screens (main menu, new game, slots, settings, region map with dialogue and shops, world map, game menu, character,
//! journal, battle results, battle); the agent link (TCP, `aegis link`) also reaches the battle command API while a
//! battle is open. The desktop program is `src/main.rs` (`run_native`); the web build is this library as a `cdylib`
//! (`web`, WEBREL1-G1), which reads the same launch keys from the page's URL query.
//!   emberwake [screen=menu|arena|runner] [scene=goblin_skirmish] [ruleset=<id>] [seed=N] [port=47800] [data=<export dir>] [saves=<dir>]
//!             [headless=WxH [fps=N] [dpr=X] [idle_exit=S]]
//! `scene=` (old name `encounter=`): the encounter `screen=arena` plays (both names at once are refused, PR5-29a);
//! `ruleset=`: the ruleset it plays by (default the encounter's own); `rules=` is refused (it adjusts rules on every
//! command line, never names the ruleset; PR5-36, FB-223 (a)). `headless=`: no window
//! and no sound device; the frames stay in memory and the agent link draws them out (`aegis_ui::shell::headless`).
//! `screen=arena` and `screen=runner` need the dev tools (cargo feature `dev_tools`, default on; `DEV_TOOLS`).
//! Keys: arrows / W A S D move, E interact, Esc back or game menu, Enter confirm, F12 screenshot; battle keys as listed
//! in battle.rs. Right click goes back.

// A build without the dev tools (the web release build) leaves the items that only the Scene Runner and its screens
// reach (arena starts, samples, gallery reels, view pickers) unused; the linker drops them. The default build (dev tools
// on: the desktop, the gate) keeps the full dead-code check.
#![cfg_attr(not(feature = "dev_tools"), allow(dead_code, unused_imports))]

mod battle;
mod battle_view;
mod board;
mod board_view;
mod bunks;
mod character_view;
mod controls_view;
#[cfg(feature = "dev_tools")]
mod dojo;
mod dojo_view;
mod epilogue;
mod epilogue_view;
mod facility;
mod facility_view;
mod floats;
#[cfg(feature = "dev_tools")]
mod forge;
mod forge_view;
mod fx;
#[cfg(feature = "dev_tools")]
mod gallery;
mod ground;
mod hud;
mod hub;
mod intent_text;
mod loadout_view;
mod mapview;
mod menus;
mod music;
mod party;
mod portraits;
mod props;
mod recipe;
mod recipe_view;
mod region;
#[cfg(feature = "dev_tools")]
mod runner;
mod runover;
mod screens;
mod shop;
mod save_view;
mod shop_view;
mod siege;
mod siege_view;
mod stage;
mod story;
mod talk;
mod talk_view;
mod title_fx;
mod timeline_view;
mod tokens;
mod transitions;
mod tuning;
mod viewkeys;
#[cfg(target_arch = "wasm32")]
mod web;
mod worldmap;
mod worldmap_view;

use aegis_content::manifest;
use aegis_ui::art;
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

/// PR7-37 (PT-01, PT-14, PT-33): a dev build (`DEV_KEYS`: a debug build with the dev tools) shows the developer
/// surfaces a player never sees: the main menu's Tools, the test worlds on New game, the world map's Edit layout and
/// the Settings rows for testing (the "(testing)" overlays, the Debug row). Any other build hides them.
pub(crate) const DEV_BUILD: bool = DEV_KEYS;

/// ENGINE-57 (FEEL-RELEASE1): a launch asked for the developer surfaces (`dev_asked`).
static DEV_ASKED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The developer surfaces are shown (`DEV_BUILD`): a debug build with the dev tools, or (ENGINE-57) a build with the dev
/// tools launched with `dev=1`, as every scenario run is (the scenario driver passes it), so one script takes one path
/// in a dev and a release build. A player's release build never shows them; the debug build's dev keys stay
/// `DEV_KEYS`.
pub(crate) fn dev_build() -> bool {
    DEV_BUILD || DEV_ASKED.load(std::sync::atomic::Ordering::Relaxed)
}

/// ENGINE-57: whether the launch asks for the developer surfaces: `dev=1` (`asked`, read by `Start`, ENGINE-66 CLN-77),
/// in a build with the dev tools (`tools`, `DEV_TOOLS`); a build without them never shows them.
fn dev_asked(asked: bool, tools: bool) -> bool {
    tools && asked
}

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

/// LAYER-ENG-KEYS (PR5-29a): the engine's scenario driver launches with `scene=` (the encounter) and `ruleset=` (the
/// ruleset id); the old key `encounter=` still works (URLs, hand launches). Both names at once is refused.
fn launch_key<'a>(args: &'a [(String, String)], key: &str, old: &str) -> Result<Option<&'a str>> {
    match (arg(args, key), arg(args, old)) {
        (Some(_), Some(_)) => anyhow::bail!("{key}= or its old name {old}=, not both"),
        (new, legacy) => Ok(new.or(legacy)),
    }
}

/// `ruleset=`: only `screen=arena` plays by one a launch names; anywhere else it would be ignored, so it is refused.
/// PK2-FOLLOWUP (PR5-36, FB-223 (a)): `rules=` adjusts rules (`+id,-id`) on every command line and never names the
/// ruleset; the launch has no rule adjustment, so it is refused rather than read as the id (PR5-29a's launch name).
fn arena_ruleset<'a>(args: &'a [(String, String)]) -> Result<Option<&'a str>> {
    if let Some(v) = arg(args, "rules") {
        anyhow::bail!("rules={v} adjusts rules and is no launch key; name the ruleset with ruleset=<id>");
    }
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
    // ENGINE-11: the manifest, layers, saves and shots folders, settings, sound and the shell's options come from the
    // engine's game start; the data, the tuning, the launch keys and the first screens are the game's.
    let engine = engine_dir.unwrap_or_else(|| PathBuf::from("."));
    let mut launch = aegis_ui::start::Start::new(GAME_ID, game_dir.clone(), &engine, args, headless);
    DEV_ASKED.store(dev_asked(launch.dev, DEV_TOOLS), std::sync::atomic::Ordering::Relaxed);
    notices.append(&mut launch.notice);
    if arg(args, "saves").is_none() {
        if let Err(e) = aegis_save::adopt_shared(GAME_ID) {
            notices.push(format!("Copying the saves into {}: {e}", aegis_save::user_dir(GAME_ID).display()));
        }
    }
    let project = game_dir.clone().or_else(|| art::find_project(&start));
    if game_dir.is_none() {
        launch.layers = project.as_ref().map(|p| aegis_core::assets::Layers::for_game(p)).unwrap_or_default();
    }
    let home = game_dir.unwrap_or_else(|| PathBuf::from("."));
    let game = GameCx {
        data,
        manifest: launch.manifest.clone(),
        game: None,
        save_dir: launch.save_dir.clone(),
        tuning: TuningFile::new(Some(home.join("tuning.json"))),
        log: Vec::new(),
        encounter: launch_key(args, "scene", "encounter")?.unwrap_or("goblin_skirmish").to_string(),
        seed: arg(args, "seed").and_then(|s| s.parse().ok()),
        debug: false,
        data_dir,
    };
    let mut settings = launch.settings(default_settings());
    settings.keep_window_size(&screens::WINDOW_SIZES);
    force_shipped_views(&mut settings.v, arg(args, "screen"));
    // CONTROLS-MIGRATE1 (PR7-71): an action a file from before the fixed keys put on one goes back to its own key
    let moved = aegis_ui::controls::release_fixed(&mut settings.v, &controls_view::FIXED_KEYS);
    if !moved.is_empty() {
        let names: Vec<String> = moved.iter().map(|k| controls_view::ACTIONS.iter().find(|a| a.key == *k).map_or_else(|| aegis_ui::controls::key_label(*k), |a| a.label.to_string())).collect();
        notices.push(format!("Controls: {} went back to {} own key (that key is fixed).", names.join(", "), if names.len() == 1 { "its" } else { "their" }));
        if let Err(e) = settings.save() {
            notices.push(format!("Saving the settings: {e}"));
        }
    }
    // Playtest recordings are never on in the web build (WEBREL1 6.1, `vfs::dev::ON`), so its data is never hashed.
    let record = aegis_core::vfs::dev::ON && settings.v["record_playtests"].as_bool().unwrap_or(cfg!(debug_assertions));
    // The data fingerprint a recording carries; hashed only while recording is on.
    let content = if record { aegis_content::fingerprint::of(&game.data_dir) } else { serde_json::Value::Null };
    let playtest = aegis_ui::playtest::Playtests::new(aegis_save::user_dir(GAME_ID).join("playtests"), record, content);
    // ASSETORG2: sounds and music over the same layers as the art (game, kit, engine)
    let audio = launch.audio();
    launch.notice = notices;
    #[allow(unused_mut)]
    let (mut cx, mut options) = launch.cx(settings, art::Art::new(project), audio, playtest, game);
    // ENGINE-64 (LIVE-RELOAD1): `dev=1` reloads edited files only in a build with the dev tools, never a player's
    options.live &= DEV_TOOLS;
    #[allow(unused_mut)]
    let mut stack: Vec<Box<dyn Screen<GameCx>>> = vec![Box::new(menus::MainMenu::default())];
    match arg(args, "screen") {
        #[cfg(feature = "dev_tools")]
        Some("arena") => match ruleset {
            Some(ruleset) => {
                emberwake_rules::ruleset::Ruleset::of(&cx.app.data.cat, ruleset).map_err(anyhow::Error::msg)?;
                // PR7-54: `scene=<world>:<encounter>` opens a world's encounter (the playback form), so a scenario reaches
                // a boss fight of a world
                let (world, encounter) = match cx.app.encounter.split_once(':') {
                    Some((w, e)) => (Some(w.to_string()), e.to_string()),
                    None => (None, cx.app.encounter.clone()),
                };
                stack.push(Box::new(battle::BattleScreen::arena_with(&cx, world, &encounter, Some(ruleset.to_string()), "", false)));
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
    Ok((cx, stack, options))
}

/// The desktop program (`src/main.rs`): the launch keys from the command line, the folders found from the working
/// folder or the program's, a window (or none, `headless=`).
///
/// Offline full-map capture (for `aegis align-map`): `render_map=<id> out=<file.png> [fx=<path|none>] [map_override=<json>] [view=square|tilted]`
/// renders the full static map at 32 px/tile with no window or HUD and exits.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() -> Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<(String, String)> = raw.iter().filter_map(|a| a.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect();
    if let Some(map_id) = arg(&args, "render_map") {
        return render_map_native(&args, map_id);
    }
    let headless = Headless::from_args(&args)?;
    let ruleset = arena_ruleset(&args)?;
    let roots = native_roots()?;
    let (cx, stack, options) = launch(&args, roots, ruleset, headless)?;
    aegis_ui::shell::run(cx, stack, options)
}

#[cfg(not(target_arch = "wasm32"))]
fn render_map_native(args: &[(String, String)], map_id: &str) -> Result<()> {
    let out = arg(args, "out").context("render_map requires out=<path.png>")?;
    let roots = native_roots()?;
    let (mut cx, _stack, options) = launch(args, roots, None, None)?;
    let view = match arg(args, "view").unwrap_or("square") {
        "tilted" => mapview::View::Tilted,
        _ => mapview::View::Square,
    };
    let fx_mode = match arg(args, "fx") {
        None => region::StaticFx::Layered,
        Some("none") | Some("off") => region::StaticFx::Disabled,
        Some(p) => {
            let fx = aegis_ui::map_fx::MapFx::load_file(Path::new(p)).with_context(|| format!("reading MapFx '{p}'"))?;
            region::StaticFx::Override(fx)
        }
    };
    let map_override = match arg(args, "map_override") {
        None => None,
        Some(p) => {
            let text = std::fs::read_to_string(p).with_context(|| format!("reading map_override '{p}'"))?;
            let val: serde_json::Value = serde_json::from_str(&text).with_context(|| format!("parsing map_override '{p}'"))?;
            Some(val.as_object().cloned().context("map_override must be a JSON object")?)
        }
    };
    let canvas = region::render_static_map(&mut cx, &options.font, map_id, view, fx_mode, map_override.as_ref())?;
    let out_path = Path::new(out);
    if let Some(parent) = out_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    aegis_ui::shell::save_png(&canvas, out_path, canvas.w)?;
    println!("rendered {map_id} ({}x{}) -> {}", canvas.w, canvas.h, out_path.display());
    Ok(())
}

/// UIBOT (REPLAY-CI1): where the engine's UI bot (`aegis_ui::shell::bot`) starts, by name, with the launch keys of each
/// (`screen=`, `ruleset=`); the bot walks on from there by pressing widgets.
#[cfg(feature = "dev_tools")]
pub const BOT_STARTS: &[(&str, &[(&str, &str)])] =
    &[("menu", &[]), ("arena", &[("screen", "arena")]), ("arena_timeline", &[("screen", "arena"), ("ruleset", "timeline")]), ("runner", &[("screen", "runner")])];
#[cfg(not(feature = "dev_tools"))]
pub const BOT_STARTS: &[(&str, &[(&str, &str)])] = &[("menu", &[])];

/// UIBOT: widgets the bot never presses: they write into the repo (Copy to repo), open an OS window (Open folder) or
/// reach the OS clipboard, and the main menu's Quit (it ends the program, so the run would stop at its first steps; a key
/// that quits still ends a run cleanly).
pub const BOT_SKIP: &[&str] = &["recordings.copy", "recordings.open_folder", "recordings.note_view.copy", "menu.quit"];

/// UIBOT: one bot run from the start `start` (a `BOT_STARTS` name) on the data folder `data` (a scratch copy: the dev
/// tools write into it), saves under `scratch`. The user folder is wherever `APPDATA` points (the caller points it at a
/// scratch folder first). `Err` when the game cannot start; a panic is in the run's report.
#[cfg(not(target_arch = "wasm32"))]
pub fn bot_run(start: &str, data: &Path, scratch: &Path, bot: &aegis_ui::shell::bot::Bot) -> Result<aegis_ui::shell::bot::Run> {
    let keys = BOT_STARTS.iter().find(|(name, _)| *name == start).map(|(_, k)| *k).with_context(|| format!("no bot start '{start}'"))?;
    let mut args: Vec<(String, String)> = keys.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    args.push(("data".into(), data.display().to_string()));
    args.push(("saves".into(), scratch.join("saves").display().to_string()));
    let headless = Headless { size: bot.size, fps: aegis_ui::shell::headless::DEFAULT_FPS, dpr: 1.0, idle: aegis_core::clock::Duration::from_secs(aegis_ui::shell::headless::IDLE_EXIT_S), stepped: false };
    let ruleset = arena_ruleset(&args)?;
    let (cx, stack, options) = launch(&args, native_roots()?, ruleset, Some(headless))?;
    Ok(aegis_ui::shell::bot::run(cx, stack, &options, bot))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// ENGINE-57 (FEEL-RELEASE1): `dev=1` shows the developer surfaces in a build with the dev tools (a release
    /// scenario run, which then takes the dev build's path: New game opens the world picker); a shipped player build,
    /// without the dev tools, ignores it.
    #[test]
    fn dev_1_shows_the_developer_surfaces_only_with_the_dev_tools() {
        let args = |list: &[(&str, &str)]| list.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<Vec<_>>();
        let dev = |list: &[(&str, &str)]| aegis_ui::start::Start::new("emberwake", None, std::path::Path::new("."), &args(list), None).dev;
        assert!(dev_asked(dev(&[("dev", "1")]), true), "a release build with the dev tools");
        assert!(!dev_asked(dev(&[("dev", "1")]), false), "a shipped player build ignores it");
        assert!(!dev_asked(dev(&[("dev", "0")]), true) && !dev_asked(dev(&[]), true));
    }

    #[test]
    fn a_launch_ruleset_comes_with_the_arena_only() {
        let args = |list: &[(&str, &str)]| list.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<Vec<_>>();
        assert_eq!(arena_ruleset(&args(&[("screen", "arena"), ("ruleset", "r")])).unwrap(), Some("r"));
        assert_eq!(arena_ruleset(&args(&[("screen", "arena")])).unwrap(), None);
        assert_eq!(arena_ruleset(&args(&[])).unwrap(), None);
        assert!(arena_ruleset(&args(&[("ruleset", "r")])).is_err());
        assert!(arena_ruleset(&args(&[("screen", "runner"), ("ruleset", "r")])).is_err());
        // PK2-FOLLOWUP (FB-223 (a)): the engine's driver passes `ruleset=`; `rules=` never names the ruleset.
        for list in [&[("screen", "arena"), ("rules", "r")][..], &[("screen", "arena"), ("ruleset", "r"), ("rules", "+x")], &[("screen", "runner"), ("rules", "r")]] {
            let e = arena_ruleset(&args(list)).unwrap_err().to_string();
            assert!(e.contains("ruleset=<id>"), "{list:?}: {e}");
        }
    }

    #[test]
    fn a_launch_scene_takes_the_old_encounter_key_too() {
        let args = |list: &[(&str, &str)]| list.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<Vec<_>>();
        assert_eq!(launch_key(&args(&[("scene", "e1")]), "scene", "encounter").unwrap(), Some("e1"));
        assert_eq!(launch_key(&args(&[("encounter", "e1")]), "scene", "encounter").unwrap(), Some("e1"));
        assert_eq!(launch_key(&args(&[]), "scene", "encounter").unwrap(), None);
        assert!(launch_key(&args(&[("scene", "e1"), ("encounter", "e2")]), "scene", "encounter").is_err());
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
