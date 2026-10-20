//! Emberwake on the engine's command line shell (`aegis_cli`): its own commands, its data folder, its content hooks
//! and its command host for `aegis serve`.
//!   aegis sim <mode> [level] [seeds] [key=value ...]   the balance sim (duel, tier, levels, group, stats, party, role,
//!                                                      sweep, campaign; `aegis sim help` lists the keys)
//!   aegis battle <encounter id> [seed=N] [preset=balanced] [log]              autoplay a catalog encounter
//! Data: the project's data/ folder (`data=<dir>`), read by the game's loader; `catalog=<path>` (or $AEGIS_CATALOG)
//! reads a resolved Godot export instead (battle, serve); `sim` reads a Godot export folder with `parity=<dir>`. A sim
//! key given twice keeps its last value.

use crate::{Battle, Host};
use emberwake_battle::session::{EncounterDef, Phase, Session};
use aegis_cli::{Args, Game};
use emberwake_data::Catalog;
use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// The game as the shell sees it.
pub struct Emberwake;

impl Game for Emberwake {
    type Content = emberwake_content::store::Emberwake;
    const USAGE: &'static str = "aegis sim <mode> [level] [seeds] [key=value ...]   (aegis sim help: modes and keys)\naegis battle <encounter id> [seed=N] [preset=balanced] [log]";

    fn run(&self, args: &Args) -> Option<Result<()>> {
        match (args.pos(0), args.pos(1)) {
            (Some("sim"), _) => Some(sim(args)),
            (Some("battle"), Some(_)) => Some(battle(args)),
            _ => None,
        }
    }

    fn data_dir(&self, args: &Args) -> Result<PathBuf> {
        data_dir(args)
    }

    /// games/emberwake/game.json, beside the data folder.
    fn manifest_path(&self, args: &Args) -> Result<PathBuf> {
        let dir = data_dir(args)?;
        dir.parent().map(|g| g.join(aegis_content::manifest::FILE)).with_context(|| format!("{} has no game folder above it", dir.display()))
    }

    fn content(&self) -> Self::Content {
        emberwake_content::store::Emberwake
    }

    fn host(&self, args: &Args) -> Result<Box<dyn FnMut(&Value) -> Value>> {
        let mut host = Host::new(Battle::new(load_catalog(args)?));
        Ok(Box::new(move |req| host.handle(req)))
    }
}

/// The catalog: `catalog=<file>` (or $AEGIS_CATALOG) reads a resolved Godot export; otherwise `data=<dir>` or the
/// project's data/ folder above the working directory, read by the engine's own loader (problems go to stderr).
fn load_catalog(args: &Args) -> Result<Arc<Catalog>> {
    if let Some(p) = args.keys.get("catalog").cloned().or_else(|| std::env::var("AEGIS_CATALOG").ok()) {
        let path = PathBuf::from(p);
        return Ok(Arc::new(Catalog::load_resolved(&path).with_context(|| format!("loading {}", path.display()))?));
    }
    let dir = data_dir(args)?;
    let src = emberwake_content::source::load(&dir);
    for e in &src.errors {
        eprintln!("data: {e}");
    }
    Ok(Arc::new(Catalog::from_value(&src.catalog)))
}

/// `data=<dir>`, else the game's data folder (games/emberwake/data) found from the working directory.
fn data_dir(args: &Args) -> Result<PathBuf> {
    match args.keys.get("data") {
        Some(d) => Ok(PathBuf::from(d)),
        None => match emberwake_world::Data::find_data_dir(&std::env::current_dir()?) {
            Some(d) => Ok(d),
            None => bail!("data folder not found: run from the project or pass data=<dir> (or catalog=<export file>)"),
        },
    }
}

/// `aegis sim <mode> [level] [seeds] [key=value ...]`: the balance sim (emberwake_sim) on the game's data folder, or
/// on a Godot export folder with `parity=<dir>` (the parity checks); prints what the sim prints, the time on stderr.
fn sim(args: &Args) -> Result<()> {
    let d = match args.keys.get("parity") {
        Some(dir) => emberwake_world::Data::load(&PathBuf::from(dir)).with_context(|| format!("loading the exports in {dir}"))?,
        None => {
            let (d, problems) = emberwake_world::Data::open(&data_dir(args)?)?;
            for p in problems {
                eprintln!("data: {p}");
            }
            d
        }
    };
    let mut argv: Vec<String> = args.positional[1..].to_vec();
    argv.extend(args.keys.iter().filter(|(k, _)| !matches!(k.as_str(), "data" | "parity" | "catalog")).map(|(k, v)| format!("{k}={v}")));
    let start = Instant::now();
    for line in emberwake_sim::run(&d, &argv) {
        println!("{line}");
    }
    eprintln!("sim: {:.2} s", start.elapsed().as_secs_f64());
    Ok(())
}

fn battle(args: &Args) -> Result<()> {
    let cat = load_catalog(args)?;
    let id = args.pos(1).context("usage: aegis battle <encounter id>")?;
    let entry = cat.encounters.get(id).with_context(|| format!("unknown encounter {id}"))?.clone();
    let mut def = EncounterDef::from_obj(&entry);
    if let Some(seed) = args.keys.get("seed") {
        def.rng_seed = seed.parse()?;
    }
    let preset = args.key("preset", "balanced");
    let mut s = Session::start(cat.clone(), def, &IndexMap::new());
    let show_log = args.flag("log");
    println!("{} (seed {}): {} rounds max", s.def.name, s.def.rng_seed, s.def.max_rounds);
    loop {
        s.plan_apply_preset_all(&preset);
        if !s.confirm_plan() {
            println!("round {}: the party plan was refused", s.current_round);
            break;
        }
        let from = s.resolver.log.len();
        while s.phase != Phase::Replay {
            s.step_next();
        }
        println!("-- round {} --", s.current_round);
        if show_log {
            for line in &s.resolver.log[from..] {
                println!("  {line}");
            }
        }
        for &u in s.w.allies.iter().chain(s.w.enemies.iter()) {
            let x = &s.w.units[u];
            let side = if s.w.allies.contains(&u) { "ally" } else { "enemy" };
            println!("  {side:<5} {:<20} HP {:>4}/{:<4} AP {:>3}  at {}{}", x.name, x.hp, x.max_hp, x.ap, s.w.grid.position(u), if x.is_withdrawn { " (withdrew)" } else { "" });
        }
        if s.combat_over || !s.can_advance_round() {
            break;
        }
        s.advance_round();
    }
    println!("outcome: {}", if s.outcome.is_empty() { "undecided (round limit)" } else { &s.outcome });
    Ok(())
}

