//! Hello: the Aegis engine's sample game (ENGINE-1, HELLO-GAME1). It is built on the engine alone, with no kit, to
//! prove a game needs nothing of the RPG kit: a main menu, a walker sprite moved over a walled field, one save slot,
//! the engine's interface sounds and a scenario the gate runs. Read it as the smallest whole game: `walk` (the rules),
//! `save` (the slot), `screens` (the menu and the field); `src/bin/hello_cli.rs` is the engine's command line for it.

pub mod save;
pub mod screens;
pub mod walk;

use aegis_ui::screen::{Context, Cx, Screen, Settings};
use aegis_ui::shell::headless::Headless;
use aegis_ui::shell::Options;
use anyhow::{Context as _, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const GAME_ID: &str = "hello";

/// The game's half of the context.
pub struct Hello {
    pub walk: walk::Walk,
    pub save_dir: PathBuf,
    /// The last sound asked for (a path under the asset layers' `audio/`), for the agent's `game` reply.
    pub last_sound: String,
    /// Event lines for the shell's screenshot note.
    pub log: Vec<String>,
}

impl Context for Hello {
    const MORE_COMMANDS: &'static str = "none (the field takes keys: arrows or W A S D)";
    fn note(&mut self, line: String) {
        self.log.push(line);
    }
    fn agent_summary(&self) -> Value {
        json!({"walk": self.walk.to_value()})
    }
    fn agent_game(&self) -> Value {
        json!({"walk": self.walk.to_value(), "saved": save::exists(&self.save_dir), "sound": self.last_sound})
    }
}

/// The interface sounds the game plays: the engine's own (`engine/assets/audio/sfx/`), found through the asset layers.
pub mod sound {
    pub const PRESS: &str = "sfx/ui_click.wav";
    pub const STEP: &str = "sfx/ui_click.wav";
    pub const BUMP: &str = "sfx/ui_error.wav";
    pub const SAVED: &str = "sfx/ui_confirm.wav";
    pub const BACK: &str = "sfx/ui_back.wav";
}

/// Plays `rel` on the interface bus and keeps it as the last sound.
pub fn play(cx: &mut Cx<Hello>, rel: &str) {
    cx.audio.play(aegis_ui::audio::Bus::Ui, rel);
    cx.app.last_sound = rel.to_string();
}

/// The game folder (games/hello: game.json, assets/, scenarios/): found upward from `start`, else where this crate
/// was built from.
pub fn game_dir(start: &Path) -> PathBuf {
    aegis_content::manifest::find_game_dir(start, GAME_ID).unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

fn arg<'a>(args: &'a [(String, String)], k: &str) -> Option<&'a str> {
    args.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
}

/// The context, the first screen and the shell's options for launch keys `args` (`port=`, `saves=`, `screen=`; the
/// rest are the shell's) in game folder `game`.
pub fn launch(args: &[(String, String)], game: &Path, headless: Option<Headless>) -> Result<(Cx<Hello>, Vec<Box<dyn Screen<Hello>>>, Options)> {
    let manifest = aegis_content::manifest::Manifest::read(&game.join(aegis_content::manifest::FILE)).map_err(anyhow::Error::msg).with_context(|| format!("reading the manifest in {}", game.display()))?;
    let layers = aegis_core::assets::Layers::for_game(game);
    let mut notice = layers.notices.clone();
    if let Some(screen) = arg(args, "screen").filter(|s| *s != "menu") {
        notice.push(format!("screen={screen}: this game opens on its menu only."));
    }
    let save_dir = arg(args, "saves").map(PathBuf::from).unwrap_or_else(|| aegis_save::user_dir(GAME_ID).join("saves"));
    let engine = layers.roots.last().and_then(|r| r.parent()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("engine"));
    let cx = Cx {
        settings: Settings::load(aegis_save::user_dir(GAME_ID).join("settings.json"), json!({})),
        audio: if headless.is_some() { aegis_ui::audio::Audio::offline_layers(&layers) } else { aegis_ui::audio::Audio::of_layers(&layers) },
        art: aegis_ui::art::Art::layered(layers),
        nav: Vec::new(),
        notice,
        shot_dir: game.join(".shots"),
        quit: false,
        answers: Vec::new(),
        playtest: aegis_ui::playtest::Playtests::off(),
        held: Default::default(),
        app: Hello { walk: walk::Walk::default(), save_dir, last_sound: String::new(), log: Vec::new() },
    };
    let port = arg(args, "port").and_then(|p| p.parse().ok()).unwrap_or(aegis_ui::bridge::DEFAULT_PORT);
    let options = Options {
        title: format!("Aegis - {}", manifest.title),
        font: engine.join("assets/fonts/OpenSans-SemiBold.ttf"),
        port,
        icon: manifest.icon.clone(),
        canvas: manifest.canvas.clone(),
        window_sizes: manifest.window_sizes.clone(),
        headless,
    };
    Ok((cx, vec![Box::new(screens::Menu::default())], options))
}

/// The desktop program: launch keys from the command line, the game folder found from the working folder.
pub fn run_native() -> Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<(String, String)> = raw.iter().filter_map(|a| a.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect();
    let headless = Headless::from_args(&args)?;
    let game = game_dir(&std::env::current_dir()?);
    let (cx, stack, options) = launch(&args, &game, headless)?;
    aegis_ui::shell::run(cx, stack, options)
}
