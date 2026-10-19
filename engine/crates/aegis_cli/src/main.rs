//! `aegis`: the engine's command line for agents and humans.
//!   aegis sim duel [level] [seeds] [preset=balanced] [jitter=on|off] [json]   balance-sim duel matrix
//!   aegis battle <encounter id> [seed=N] [preset=balanced] [log]              autoplay a catalog encounter
//! Data: the project's data/ folder (`data=<dir>`), read by the engine's own loader; `catalog=<path>` (or
//! $AEGIS_CATALOG) reads a resolved Godot export instead.

use aegis_battle::session::{EncounterDef, Phase, Session};
use aegis_battle::sim::{fight, FightSpec};
use aegis_data::{Catalog, UnitDef};
use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

const CLASSES: [&str; 6] = ["warrior", "ranger", "priest", "brawler", "rogue", "mage"];

struct Args {
    positional: Vec<String>,
    keys: IndexMap<String, String>,
}

impl Args {
    fn parse(raw: &[String]) -> Self {
        let mut positional = Vec::new();
        let mut keys = IndexMap::new();
        for a in raw {
            match a.split_once('=') {
                Some((k, v)) => {
                    keys.insert(k.to_string(), v.to_string());
                }
                None => positional.push(a.clone()),
            }
        }
        Self { positional, keys }
    }
    fn pos(&self, i: usize) -> Option<&str> {
        self.positional.get(i).map(|s| s.as_str())
    }
    fn key(&self, k: &str, default: &str) -> String {
        self.keys.get(k).cloned().unwrap_or_else(|| default.to_string())
    }
    fn flag(&self, f: &str) -> bool {
        self.positional.iter().any(|p| p == f)
    }
}

/// The catalog: `catalog=<file>` (or $AEGIS_CATALOG) reads a resolved Godot export; otherwise `data=<dir>` or the
/// project's data/ folder above the working directory, read by the engine's own loader (problems go to stderr).
fn load_catalog(args: &Args) -> Result<Arc<Catalog>> {
    if let Some(p) = args.keys.get("catalog").cloned().or_else(|| std::env::var("AEGIS_CATALOG").ok()) {
        let path = PathBuf::from(p);
        return Ok(Arc::new(Catalog::load_resolved(&path).with_context(|| format!("loading {}", path.display()))?));
    }
    let dir = match args.keys.get("data") {
        Some(d) => PathBuf::from(d),
        None => match aegis_world::Data::find_data_dir(&std::env::current_dir()?) {
            Some(d) => d,
            None => bail!("data folder not found: run from the project or pass data=<dir> (or catalog=<export file>)"),
        },
    };
    let src = aegis_world::source::load(&dir);
    for e in &src.errors {
        eprintln!("data: {e}");
    }
    Ok(Arc::new(Catalog::from_value(&src.catalog)))
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

fn main() -> Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = Args::parse(&raw);
    match (args.pos(0), args.pos(1)) {
        (Some("sim"), Some("duel")) => sim_duel(&args),
        (Some("battle"), Some(_)) => battle(&args),
        (Some("serve"), _) => serve(&args),
        (Some("link"), _) => link(&raw),
        _ => {
            println!("aegis sim duel [level] [seeds] [preset=balanced] [jitter=on|off] [json]\naegis battle <encounter id> [seed=N] [preset=balanced] [log]
aegis serve   (JSON lines on stdin / stdout; see crates/aegis_host)
aegis link '<json>' ... [port=47800]   (requests to a running aegis-gui)");
            Ok(())
        }
    }
}

/// `aegis serve`: the command API over stdin / stdout, one JSON request per line, one JSON reply per line.
fn serve(args: &Args) -> Result<()> {
    use std::io::{BufRead, Write};
    let mut host = aegis_host::Host::new(load_catalog(args)?);
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<serde_json::Value>(&line) {
            Ok(req) => host.handle(&req),
            Err(e) => json!({"ok": false, "error": format!("bad JSON: {e}")}),
        };
        writeln!(out, "{reply}")?;
        out.flush()?;
    }
    Ok(())
}

/// `aegis link '<json>' ...`: sends requests to a running aegis-gui (agent link) and prints each reply line.
fn link(raw: &[String]) -> Result<()> {
    use std::io::{BufRead, BufReader, Write};
    let port = raw.iter().find_map(|a| a.strip_prefix("port=")).unwrap_or("47800");
    let stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).with_context(|| format!("no aegis-gui listening on port {port}"))?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    for req in raw.iter().skip(1).filter(|a| !a.starts_with("port=")) {
        writeln!(writer, "{req}")?;
        let mut line = String::new();
        reader.read_line(&mut line)?;
        print!("{line}");
    }
    Ok(())
}
