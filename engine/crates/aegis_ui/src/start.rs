//! ENGINE-11 (GAME-START1): what every game's launch assembles from its game folder, once. `Start::new` reads the
//! manifest (`game.json`; a missing or broken one is a notice and the defaults, so the game still opens), the asset
//! layers over the game folder, the saves folder (`saves=`, else `<user dir>/saves`), the shots folder and the shell's
//! `Options` (title "Aegis - <title>", the engine's UI font, `port=`, the manifest's icon, canvas and window sizes);
//! `settings` loads the player's settings over the game's defaults, `audio` the sounds over the layers (silent with no
//! window), and `cx` puts the context together with the game's own state. A game adds what is its own (its data, its
//! first screens, its launch keys) around it.

use crate::art::Art;
use crate::audio::Audio;
use crate::playtest::Playtests;
use crate::screen::{Cx, Settings};
use crate::shell::headless::Headless;
use crate::shell::Options;
use aegis_content::manifest::{self, Manifest};
use aegis_core::assets::Layers;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The engine's UI font, under the engine folder.
pub const FONT: &str = "assets/fonts/OpenSans-SemiBold.ttf";

/// The value of launch key `k` (`k=value` on the command line, or the web page's query).
pub fn arg<'a>(args: &'a [(String, String)], k: &str) -> Option<&'a str> {
    args.iter().find(|(key, _)| key == k).map(|(_, v)| v.as_str())
}

/// The parts of a launch every game shares (see the module).
pub struct Start {
    /// The game id the user folder is named by.
    pub id: String,
    pub manifest: Manifest,
    /// The game folder, when found.
    pub game_dir: Option<PathBuf>,
    /// The asset layers over the game folder (none without one). Their notices are not in `notice`: a game shows
    /// them or not.
    pub layers: Layers,
    /// What the first screen shows: the manifest's problem, when it has one.
    pub notice: Vec<String>,
    pub save_dir: PathBuf,
    pub shot_dir: PathBuf,
    pub options: Options,
}

impl Start {
    /// The launch of game `id` from its folder `game_dir` (None: not found) with the engine folder `engine` (its UI
    /// font) and the launch keys `args`; `headless` runs it with no window.
    pub fn new(id: &str, game_dir: Option<PathBuf>, engine: &Path, args: &[(String, String)], headless: Option<Headless>) -> Start {
        let mut notice = Vec::new();
        let manifest = match game_dir.as_ref().map(|g| Manifest::read(&g.join(manifest::FILE))) {
            Some(Ok(m)) => m,
            found => {
                notice.push(match found {
                    Some(Err(e)) => e,
                    _ => format!("Game folder games/{id} (game.json) not found."),
                });
                Manifest { id: id.into(), title: id.into(), ..Manifest::default() }
            }
        };
        let layers = game_dir.as_ref().map(|g| Layers::for_game(g)).unwrap_or_default();
        let save_dir = arg(args, "saves").map(PathBuf::from).unwrap_or_else(|| aegis_save::user_dir(id).join("saves"));
        let shot_dir = game_dir.clone().unwrap_or_else(|| PathBuf::from(".")).join(".shots");
        let port = arg(args, "port").and_then(|p| p.parse().ok()).unwrap_or(crate::bridge::DEFAULT_PORT);
        let options = Options {
            title: format!("Aegis - {}", manifest.title),
            font: engine.join(FONT),
            port,
            icon: manifest.icon.clone(),
            canvas: manifest.canvas.clone(),
            window_sizes: manifest.window_sizes.clone(),
            headless,
        };
        Start { id: id.into(), manifest, game_dir, layers, notice, save_dir, shot_dir, options }
    }

    /// The player's settings (`<user dir>/settings.json`) over the game's `defaults`.
    pub fn settings(&self, defaults: Value) -> Settings {
        Settings::load(aegis_save::user_dir(&self.id).join("settings.json"), defaults)
    }

    /// The sounds over the asset layers; silent with no window.
    pub fn audio(&self) -> Audio {
        if self.options.headless.is_some() {
            Audio::offline_layers(&self.layers)
        } else {
            Audio::of_layers(&self.layers)
        }
    }

    /// The context: `settings`, `art`, `audio` (see `audio`), `playtest` and the game's own state `app`, with the
    /// notice and the shots folder of the launch; and the shell's options.
    pub fn cx<G>(self, settings: Settings, art: Art, audio: Audio, playtest: Playtests, app: G) -> (Cx<G>, Options) {
        let cx = Cx { settings, art, audio, nav: Vec::new(), notice: self.notice, shot_dir: self.shot_dir, quit: false, answers: Vec::new(), playtest, held: Default::default(), app };
        (cx, self.options)
    }
}
