//! Emberwake on the engine's command line shell (`aegis_cli`): its own commands, its data folder, its content hooks
//! and its command host for `aegis serve`.
//!   aegis sim duel [level] [seeds] [preset=balanced] [jitter=on|off] [json]   balance-sim duel matrix
//!   aegis battle <encounter id> [seed=N] [preset=balanced] [log]              autoplay a catalog encounter
//! Data: the project's data/ folder (`data=<dir>`), read by the game's loader; `catalog=<path>` (or $AEGIS_CATALOG)
//! reads a resolved Godot export instead.

use crate::{Battle, Host};
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_battle::sim::{fight, FightSpec};
use aegis_cli::{Args, Game};
use emberwake_data::{Catalog, UnitDef};
use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

const CLASSES: [&str; 6] = ["warrior", "ranger", "priest", "brawler", "rogue", "mage"];

/// The game as the shell sees it.
pub struct Emberwake;

impl Game for Emberwake {
    type Content = emberwake_content::store::Emberwake;
    const USAGE: &'static str = "aegis sim duel [level] [seeds] [preset=balanced] [jitter=on|off] [json]\naegis battle <encounter id> [seed=N] [preset=balanced] [log]";

    fn run(&self, args: &Args) -> Option<Result<()>> {
        match (args.pos(0), args.pos(1)) {
            (Some("sim"), Some("duel")) => Some(sim_duel(args)),
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

/// The sim's sandbox unit (SimRunner.define + _write): a class unit of `level`, hero tier, never flees.
fn sim_unit(id: &str, class: &str, level: i32, party: bool) -> UnitDef {
    let entry = json!({"id": id, "name": id, "side": if party { "party" } else { "enemy" }, "class_id": class, "level": level, "tier": "hero", "flee_ratio": 0.0});
    UnitDef::from_obj(entry.as_object().unwrap())
}

fn sim_duel(args: &Args) -> Result<()> {
    let cat = load_catalog(args)?;
    let level: i32 = args.pos(2).unwrap_or("1").parse()?;
    let seeds: i64 = args.pos(3).unwrap_or("3").parse()?;
    let preset = args.key("preset", "balanced");
    let jitter = args.key("jitter", "on") == "on";
    let mut units = IndexMap::new();
    for c in CLASSES {
        units.insert(format!("simp_{c}"), sim_unit(&format!("simp_{c}"), c, level, true));
        units.insert(format!("sime_{c}"), sim_unit(&format!("sime_{c}"), c, level, false));
    }
    let start = Instant::now();
    let mut fights = 0;
    let mut table: IndexMap<String, IndexMap<String, (f64, f64, usize)>> = IndexMap::new();
    for a in CLASSES {
        for b in CLASSES {
            if a == b {
                continue;
            }
            let (mut bin, mut hp, mut n) = (0.0, 0.0, 0usize);
            for i in 0..seeds {
                let seed = 11 + 12 * i;
                let spec = |l: &str, r: &str| FightSpec { left: vec![format!("simp_{l}")], right: vec![format!("sime_{r}")], seed, preset: preset.clone(), cap: 10, jitter, base: None, pouch: Vec::new() };
                let ab = fight(cat.clone(), &units, &spec(a, b), false);
                let ba = fight(cat.clone(), &units, &spec(b, a), false);
                fights += 2;
                if ab.skipped || ba.skipped {
                    continue;
                }
                bin += ab.bin + 1.0 - ba.bin;
                hp += ab.hp + 1.0 - ba.hp;
                n += 2;
            }
            let d = n.max(1) as f64;
            table.entry(a.to_string()).or_default().insert(b.to_string(), (bin / d, hp / d, n));
        }
    }
    let elapsed = start.elapsed();
    if args.flag("json") {
        println!("{}", json!({"mode": "duel", "level": level, "seeds": seeds, "preset": preset, "jitter": jitter, "fights": fights, "seconds": elapsed.as_secs_f64(), "table": table.iter().map(|(a, row)| (a.clone(), serde_json::Value::Object(row.iter().map(|(b, c)| (b.clone(), json!({"bin": c.0, "hp": c.1, "n": c.2}))).collect()))).collect::<serde_json::Map<_, _>>()}));
        return Ok(());
    }
    println!("DUEL level {level}, hero vs hero, score of the ROW class (bin/hp %), {seeds} seeds, preset {preset}, jitter {}", if jitter { "on" } else { "off" });
    print!("{:<10}", "");
    for c in CLASSES {
        print!("{c:<10}");
    }
    println!("avg");
    for a in CLASSES {
        print!("{a:<10}");
        let (mut sb, mut sh, mut k) = (0.0, 0.0, 0.0);
        for b in CLASSES {
            if a == b {
                print!("{:<10}", "-");
                continue;
            }
            let c = table[a][b];
            sb += c.0;
            sh += c.1;
            k += 1.0;
            print!("{:<10}", format!("{}/{}", (c.0 * 100.0).round(), (c.1 * 100.0).round()));
        }
        println!("{}/{}", (sb / k * 100.0).round(), (sh / k * 100.0).round());
    }
    println!("{fights} fights in {:.2} s", elapsed.as_secs_f64());
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

