//! `kit_sample_cli`: the engine's command line for the kit sample game (`aegis_cli::Game`), with the RPG kit's content
//! kinds (`rpgkit_content::store::RpgContent`), so `kit_sample_cli content upsert|remove|check|apply` writes the
//! game's `data/` folder through the kit's schema and checks, exactly as a full kit game does. The shell's commands
//! (`scenario`, `playtest`, `link`, `content manifest`) work as for any game; a scenario starts the game program
//! `kit_sample` beside this one.

use aegis_cli::Args;
use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub struct Cli;

/// The game folder: above `data=` when given, else found from the working folder.
fn game_dir(args: &Args) -> Result<PathBuf> {
    match args.keys.get("data") {
        Some(d) => Ok(Path::new(d).parent().map(Path::to_path_buf).unwrap_or_default()),
        None => Ok(kit_sample_game::game_dir(&std::env::current_dir()?)),
    }
}

impl aegis_cli::Game for Cli {
    type Content = rpgkit_content::store::RpgContent;
    const USAGE: &'static str = "kit_sample   (the game program beside this one; launch keys port=, headless=WxH)";

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
        rpgkit_content::store::RpgContent
    }
    fn host(&self, _args: &Args) -> Result<Box<dyn FnMut(&Value) -> Value>> {
        let mut host = aegis_host::Host::new(kit_sample_game::Commands::default());
        Ok(Box::new(move |req| host.handle(req)))
    }
    fn driver(&self, _args: &Args) -> Result<Box<dyn aegis_host::Driver>> {
        Ok(Box::new(aegis_host::Host::new(kit_sample_game::Commands::default())))
    }
}

fn main() -> Result<()> {
    aegis_cli::run(&Cli)
}
