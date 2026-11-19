//! `hello_cli`: the engine's command line for the sample game (`aegis_cli::Game`): no game commands of its own, no
//! content kinds, and the walk's command set as the host (`hello_cli serve`, `hello_cli replay`). The shell's commands
//! (`scenario`, `playtest`, `link`, `content manifest`) work as for any game; a scenario starts the game program
//! `hello` beside this one. A program of its own, like Emberwake's `aegis`, so the game program links no CLI code
//! (the file and clock lints exempt the CLI crate for that reason).

use hello_game::walk;
use aegis_cli::Args;
use aegis_content::store::Hooks;
use aegis_core::json::Dict;
use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub struct Hello;

/// No content kinds: the game has no data catalog, only its manifest.
pub struct NoContent;

impl Hooks for NoContent {
    fn kinds(&self, _world: Option<&str>) -> &'static [&'static str] {
        &[]
    }
    fn file_of(&self, data_dir: &Path, _world: Option<&str>, kind: &str) -> PathBuf {
        data_dir.join(format!("{kind}.json"))
    }
    fn validate(&self, _world: Option<&str>, kind: &str, _entry: &Dict) -> Result<Dict, Vec<String>> {
        Err(vec![format!("this game has no content kinds ('{kind}')")])
    }
    fn file_text(&self, _data_dir: &Path, _world: Option<&str>, kind: &str, _entries: &[Dict]) -> Result<String, String> {
        Err(format!("this game has no content kinds ('{kind}')"))
    }
    fn check(&self, _data_dir: &Path) -> Vec<String> {
        Vec::new()
    }
}

/// The game folder: above `data=` when given, else found from the working folder.
fn game_dir(args: &Args) -> Result<PathBuf> {
    match args.keys.get("data") {
        Some(d) => Ok(Path::new(d).parent().map(Path::to_path_buf).unwrap_or_default()),
        None => Ok(hello_game::game_dir(&std::env::current_dir()?)),
    }
}

impl aegis_cli::Game for Hello {
    type Content = NoContent;
    const USAGE: &'static str = "hello   (the game program beside this one; launch keys port=, saves=, headless=WxH)";

    fn run(&self, _args: &Args) -> Option<Result<()>> {
        None
    }
    fn data_dir(&self, args: &Args) -> Result<PathBuf> {
        match args.keys.get("data") {
            Some(d) => Ok(PathBuf::from(d)),
            None => Ok(game_dir(args)?.join("data")),
        }
    }
    fn manifest_path(&self, args: &Args) -> Result<PathBuf> {
        Ok(game_dir(args)?.join(aegis_content::manifest::FILE))
    }
    fn content(&self) -> Self::Content {
        NoContent
    }
    fn host(&self, _args: &Args) -> Result<Box<dyn FnMut(&Value) -> Value>> {
        let mut host = aegis_host::Host::new(walk::Commands::default());
        Ok(Box::new(move |req| host.handle(req)))
    }
    fn driver(&self, _args: &Args) -> Result<Box<dyn aegis_host::Driver>> {
        Ok(Box::new(aegis_host::Host::new(walk::Commands::default())))
    }
}

fn main() -> Result<()> {
    aegis_cli::run(&Hello)
}
