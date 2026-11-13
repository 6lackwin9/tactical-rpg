//! Emberwake on the engine's command line shell (`aegis_cli`): its own commands, its data folder, its content hooks
//! and its command host for `aegis serve`.
//!   aegis sim <mode> [level] [seeds] [key=value ...]   the balance sim (duel, tier, levels, group, stats, party, role,
//!                                                      sweep, campaign; `aegis sim help` lists the keys)
//!   aegis battle <encounter id> [seed=N] [preset=balanced] [log]              autoplay a catalog encounter
//!                [ruleset=classic|timeline] [rules=-id,+id]               RULESET1: its rules (default classic; a
//!                                                                         timeline test fight without ruleset= notes it)
//!   aegis battle world:<world>/<id> [same keys]                           WORLD-SIM1: a world encounter with its
//!                                                                         authored party and levels, by its world's
//!                                                                         ruleset unless ruleset= picks one
//!   aegis rules [--contract]                                       BASERULES1-M: the ruleset tweak report (rules,
//!                                                                  fields, bundles, tested sets, locked, this game's
//!                                                                  rows); --contract: the stable surface only
//! Data: the project's data/ folder (`data=<dir>`), read by the game's loader. A sim key given twice keeps its last
//! value.

use crate::{Battle, Host};
use emberwake_battle::session::{EncounterDef, Session};
use aegis_cli::{Args, Game};
use emberwake_data::Catalog;
use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use aegis_core::clock::Instant;

/// The game as the shell sees it.
pub struct Emberwake;

impl Game for Emberwake {
    type Content = emberwake_content::store::RpgContent;
    const USAGE: &'static str = "aegis sim <mode> [level] [seeds] [key=value ...]   (aegis sim help: modes and keys)\naegis battle <encounter id | world:<world>/<id>> [seed=N] [preset=balanced] [ruleset=classic|timeline] [rules=-id,+id] [log]
aegis rules [--contract]   (the ruleset tweak report; --contract: tunable and locked, one line each)";

    fn run(&self, args: &Args) -> Option<Result<()>> {
        match (args.pos(0), args.pos(1)) {
            (Some("sim"), _) => Some(sim(args)),
            (Some("battle"), Some(_)) => Some(battle(args)),
            (Some("rules"), _) => Some(rules(args)),
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
        emberwake_content::store::RpgContent
    }

    fn host(&self, args: &Args) -> Result<Box<dyn FnMut(&Value) -> Value>> {
        let mut host = battle_host(args)?;
        Ok(Box::new(move |req| host.handle(req)))
    }

    fn driver(&self, args: &Args) -> Result<Box<dyn aegis_host::Driver>> {
        Ok(Box::new(battle_host(args)?))
    }
}

/// The battle host on the data folder with its worlds (world encounters "<world>:<id>" too).
fn battle_host(args: &Args) -> Result<Host> {
    Ok(Host::new(Battle::with_data(world_data(args)?)))
}

/// The data folder with its worlds, loaded once per process (a seed sweep asks for a fresh host per seed).
fn world_data(args: &Args) -> Result<Arc<emberwake_world::Data>> {
    static DATA: std::sync::OnceLock<Arc<emberwake_world::Data>> = std::sync::OnceLock::new();
    if let Some(d) = DATA.get() {
        return Ok(d.clone());
    }
    let (d, problems) = emberwake_world::Data::open(&data_dir(args)?)?;
    for p in problems {
        eprintln!("data: {p}");
    }
    Ok(DATA.get_or_init(|| Arc::new(d)).clone())
}

/// The catalog: `data=<dir>` or the project's data/ folder above the working directory, read by the engine's own
/// loader (problems go to stderr).
fn load_catalog(args: &Args) -> Result<Arc<Catalog>> {
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
        None => match emberwake_world::find_data_dir(&std::env::current_dir()?) {
            Some(d) => Ok(d),
            None => bail!("data folder not found: run from the project or pass data=<dir>"),
        },
    }
}

/// `aegis rules [--contract]` (BASERULES1-M): the kit's tweak report on the game's ruleset rows, or with `--contract`
/// the stable surface alone (no data read).
fn rules(args: &Args) -> Result<()> {
    let fields = aegis_content::schema::fields(emberwake_content::source::schema(), "content", emberwake_rules::ruleset::KIND);
    let lines = if args.flag("--contract") {
        rpgkit_host::rules::contract(fields)
    } else {
        let cat = load_catalog(args)?;
        let rows: Vec<emberwake_rules::ruleset::Ruleset> = cat.rulesets.values().map(emberwake_rules::ruleset::Ruleset::from_row).collect();
        rpgkit_host::rules::report(fields, &rows)
    };
    for line in lines {
        println!("{line}");
    }
    Ok(())
}

/// `aegis sim <mode> [level] [seeds] [key=value ...]`: the balance sim (emberwake_sim) on the game's data folder;
/// prints what the sim prints, the time on stderr.
fn sim(args: &Args) -> Result<()> {
    let (d, problems) = emberwake_world::Data::open(&data_dir(args)?)?;
    for p in problems {
        eprintln!("data: {p}");
    }
    let mut argv: Vec<String> = args.positional[1..].to_vec();
    argv.extend(args.keys.iter().filter(|(k, _)| k.as_str() != "data").map(|(k, v)| format!("{k}={v}")));
    let start = Instant::now();
    for line in emberwake_sim::run(&d, &argv) {
        println!("{line}");
    }
    eprintln!("sim: {:.2} s", start.elapsed().as_secs_f64());
    Ok(())
}

/// WORLD-SIM1 (PR5-42): the battle of world encounter `reference` (`world:<world>/<id>`, or `world:<id>` in the sim's
/// default world) with its authored side, as `aegis serve`'s `new` "<world>:<id>" and `sim feel world:` build it
/// (`rpgkit_world::battle::authored_fight`: the encounter's `party` and `guests` as the world's characters at their
/// authored levels, its phases and fields, the world's ruleset). `seed`: its rng seed (None: the encounter's);
/// `ruleset`: a ruleset row over the world's ("" keeps it); `rules`: single extensions switched (`Ruleset::adjust`).
pub fn world_session(d: &emberwake_world::Data, reference: &str, seed: Option<i64>, ruleset: &str, rules: &str) -> Result<Session> {
    let game = emberwake_sim::game();
    let Some((world, local)) = emberwake_sim::args::parse_ref(reference, &game.default_world) else {
        bail!("'{reference}' is not a world encounter reference (world:<world>/<id> or world:<id>)")
    };
    let (mut def, roster) = emberwake_world::battle::authored_fight(d, &world, &local, 0).with_context(|| format!("unknown encounter '{local}' in world '{world}'"))?;
    if let Some(seed) = seed {
        def.rng_seed = seed;
    }
    if !ruleset.is_empty() {
        def.ruleset = emberwake_rules::ruleset::Ruleset::of(&d.cat, ruleset).map_err(anyhow::Error::msg)?;
    }
    def.ruleset.adjust(rules).map_err(anyhow::Error::msg)?;
    Ok(Session::start(d.cat.clone(), def, &roster))
}

fn battle(args: &Args) -> Result<()> {
    let id = args.pos(1).context("usage: aegis battle <encounter id | world:<world>/<id>>")?;
    let seed = args.keys.get("seed").map(|s| s.parse::<i64>()).transpose()?;
    if id.starts_with(emberwake_sim::args::REF_PREFIX) {
        let d = world_data(args)?;
        let s = world_session(&d, id, seed, &args.key("ruleset", ""), &args.key("rules", ""))?;
        rules_notes(args, &s.def.ruleset);
        return autoplay(args, s);
    }
    let cat = load_catalog(args)?;
    let entry = cat.encounters.get(id).with_context(|| format!("unknown encounter {id}"))?.clone();
    let mut def = EncounterDef::from_obj(&entry);
    if let Some(seed) = seed {
        def.rng_seed = seed;
    }
    // RULESET1 3.1.3: a catalog encounter plays classic unless ruleset= picks one; rules= switches single extensions
    def.ruleset = emberwake_rules::ruleset::Ruleset::of(&cat, &args.key("ruleset", "")).map_err(anyhow::Error::msg)?;
    def.ruleset.adjust(&args.key("rules", "")).map_err(anyhow::Error::msg)?;
    rules_notes(args, &def.ruleset);
    // RULESET1-G2: a timeline test fight stays classic by default (the catalog encounter default), with a note on stderr
    if let Some(note) = classic_test_fight_note(id, &entry, args.keys.contains_key("ruleset")) {
        eprintln!("{note}");
    }
    autoplay(args, Session::start(cat.clone(), def, &IndexMap::new()))
}

/// BASERULES1-M: an adjusted set (`rules=`) prints the register metadata's notes (it never refuses: the tools try things).
fn rules_notes(args: &Args, ruleset: &emberwake_rules::ruleset::Ruleset) {
    if args.keys.contains_key("rules") {
        let n = ruleset.notes();
        for note in n.errors.iter().chain(&n.warnings) {
            eprintln!("rules: {note}");
        }
    }
}

/// `aegis battle`'s autoplay of a started battle `s` (a catalog or a world encounter) to its end, round by round.
fn autoplay(args: &Args, mut s: Session) -> Result<()> {
    let preset = args.key("preset", "balanced");
    let show_log = args.flag("log");
    println!("{} (seed {}): {} rounds max", s.def.name, s.def.rng_seed, s.def.max_rounds);
    // ENCOUNTER2-U (decision "RULESET1-A3 loose ends"): the encounter's phase and kill lines print with the log
    let mut said = 0;
    loop {
        // RULESET1-G2: the auto driver (the battle screen's autoplay, the host's `autoplay`, `sim feel`); ENCOUNTER2 H1:
        // a hold escape's holder by the encounter's `pick_default`
        if let Some(h) = s.auto_draft(&preset) {
            println!("{} holds {}", s.w.units[h].name, s.w.hold.as_ref().map(|g| g.label.as_str()).unwrap_or(""));
        }
        if !s.confirm_plan() {
            println!("round {}: the party plan was refused", s.current_round);
            break;
        }
        let from = s.resolver.log.len();
        s.resolve_round();
        println!("-- round {} --", s.current_round);
        if show_log {
            for line in s.log_lines(from, said) {
                println!("  {line}");
            }
            said = s.encounter_lines.len();
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
    // ENCOUNTER2 S2: the round limit gives the encounter's timeout_outcome (fled unless it names lost)
    let outcome = s.final_outcome();
    println!("outcome: {}{}", if outcome.is_empty() { "undecided" } else { &outcome }, if s.outcome.is_empty() { " (round limit)" } else { "" });
    // S3: a stay-behind battle names each hero's fate
    if s.stay.is_some() {
        for (c, fate) in s.party_fates() {
            println!("  {} {}", s.w.units[c].name, fate.name());
        }
    }
    Ok(())
}


/// The id prefix of the game's timeline test units (`tl_`, RULESET1_TASKS 2.7: test data CAST1's bestiary replaces).
pub const TIMELINE_TEST_PREFIX: &str = "tl_";

/// RULESET1-G2: the note `battle` prints (stderr) for a timeline test fight played without `ruleset=`: every party and
/// enemy ref is a `tl_` unit, so the fight was built for the timeline rules, yet a catalog encounter plays classic by
/// default (RULESET1 3.1.3). None when `ruleset=` was given or a ref is not a test
/// unit.
pub fn classic_test_fight_note(id: &str, entry: &serde_json::Map<String, Value>, ruleset_given: bool) -> Option<String> {
    let refs: Vec<&str> = ["party", "enemies"].iter().filter_map(|k| entry.get(*k).and_then(Value::as_array)).flatten().filter_map(Value::as_str).collect();
    if ruleset_given || refs.is_empty() || !refs.iter().all(|r| r.starts_with(TIMELINE_TEST_PREFIX)) {
        return None;
    }
    Some(format!("note: {id} is a timeline test fight ({TIMELINE_TEST_PREFIX} units) played by the classic rules; add ruleset=timeline to play it as `sim feel` does"))
}
