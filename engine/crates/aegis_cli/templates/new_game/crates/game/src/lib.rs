//! __TITLE__: made by `aegis new game __ID__`. The game's context (`__TYPE__`), its launch (the context, the first
//! screen and the shell's options) and its command set for the engine's host. Screens are in `screens`.

pub mod screens;

use aegis_ui::screen::{Context, Cx, Screen};
use aegis_ui::shell::headless::Headless;
use aegis_ui::shell::Options;
use anyhow::Result;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub const GAME_ID: &str = "__ID__";

/// The game's half of the context: everything the screens share.
#[derive(Default)]
pub struct __TYPE__ {
    /// Whether a game was started this run.
    pub started: bool,
    /// The last sound asked for (a path under the asset layers' `audio/`), for the agent's `game` reply.
    pub last_sound: String,
    /// Event lines for the shell's screenshot note.
    pub log: Vec<String>,
}

impl Context for __TYPE__ {
    const MORE_COMMANDS: &'static str = "none yet";
    fn note(&mut self, line: String) {
        self.log.push(line);
    }
    fn agent_summary(&self) -> Value {
        json!({"started": self.started})
    }
    fn agent_game(&self) -> Value {
        json!({"started": self.started, "sound": self.last_sound})
    }
}

/// The interface sounds the game plays: the engine's own (`engine/assets/audio/sfx/`), found through the asset layers.
pub mod sound {
    pub const PRESS: &str = "sfx/ui_click.wav";
    pub const BACK: &str = "sfx/ui_back.wav";
}

/// Plays `rel` on the interface bus and keeps it as the last sound.
pub fn play(cx: &mut Cx<__TYPE__>, rel: &str) {
    cx.audio.play(aegis_ui::audio::Bus::Ui, rel);
    cx.app.last_sound = rel.to_string();
}

/// The command set the engine's host answers with (`__ID___cli serve`, a replay): `new` and `state` for now.
#[derive(Debug, Default)]
pub struct Commands {
    pub started: bool,
}

impl aegis_host::Commands for Commands {
    fn mutating(&self, cmd: &str) -> bool {
        cmd == "new"
    }
    fn restarts(&self, cmd: &str) -> bool {
        cmd == "new"
    }
    fn handle(&mut self, cmd: &str, _req: &Value) -> Value {
        match cmd {
            "new" => {
                self.started = true;
                json!({"ok": true, "started": true})
            }
            "state" => json!({"ok": true, "started": self.started}),
            other => json!({"ok": false, "error": format!("unknown cmd '{other}' (new, state, history)")}),
        }
    }
    fn checkpoint(&mut self) -> Option<Value> {
        Some(json!({"started": self.started}))
    }
}

/// The game folder (game.json, assets/, scenarios/): found upward from `start`, else where this crate was built from.
pub fn game_dir(start: &Path) -> PathBuf {
    aegis_content::manifest::find_game_dir(start, GAME_ID).unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
}

/// The context, the first screen and the shell's options for launch keys `args` (`port=`; the rest are the shell's)
/// in game folder `game`.
pub fn launch(args: &[(String, String)], game: &Path, headless: Option<Headless>) -> Result<(Cx<__TYPE__>, Vec<Box<dyn Screen<__TYPE__>>>, Options)> {
    // The engine's game start (`aegis_ui::start`): manifest, asset layers, saves, settings, sound and the shell's options.
    let engine = aegis_core::assets::Layers::for_game(game).roots.last().and_then(|r| r.parent()).map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("engine"));
    let mut start = aegis_ui::start::Start::new(GAME_ID, Some(game.to_path_buf()), &engine, args, headless);
    if let Some(problem) = start.notice.first() {
        anyhow::bail!("reading the manifest in {}: {problem}", game.display());
    }
    start.notice = start.layers.notices.clone();
    let (settings, audio, art) = (start.settings(json!({})), start.audio(), aegis_ui::art::Art::layered(start.layers.clone()));
    let (cx, options) = start.cx(settings, art, audio, aegis_ui::playtest::Playtests::off(), __TYPE__::default());
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
