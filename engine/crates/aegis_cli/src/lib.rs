//! `aegis`: the engine's command line shell for agents and humans, game-agnostic. The game registers itself at start
//! (`run(&game)`: its own commands, its data folder, its content hooks and its command host); the shell adds
//!   aegis serve                                                                the game's command API over stdin / stdout
//!   aegis link '<json>' ... [port=47800]                                       requests to a running GUI (agent link)
//!   aegis replay <recording.json> ...                                          replay a playtest recording (see `replay`)
//!   aegis content check | upsert | remove | apply | manifest                    edit the data folder (see `content`)

use aegis_content::file::stringify;
use aegis_content::manifest::Manifest;
use aegis_content::store::{self, Hooks};
use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// The command line: `key=value` words and positional words.
pub struct Args {
    pub positional: Vec<String>,
    pub keys: IndexMap<String, String>,
}

impl Args {
    pub fn parse(raw: &[String]) -> Self {
        let mut positional = Vec::new();
        let mut keys = IndexMap::new();
        for a in raw {
            match a.split_once('=').filter(|_| !a.starts_with('{') && !a.starts_with('[') && !a.starts_with('@')) {
                Some((k, v)) => {
                    keys.insert(k.to_string(), v.to_string());
                }
                None => positional.push(a.clone()),
            }
        }
        Self { positional, keys }
    }
    pub fn pos(&self, i: usize) -> Option<&str> {
        self.positional.get(i).map(|s| s.as_str())
    }
    pub fn key(&self, k: &str, default: &str) -> String {
        self.keys.get(k).cloned().unwrap_or_else(|| default.to_string())
    }
    pub fn flag(&self, f: &str) -> bool {
        self.positional.iter().any(|p| p == f)
    }
}

/// What the shell asks of the game.
pub trait Game {
    type Content: Hooks;
    /// Usage lines of the game's own commands (printed before the shell's).
    const USAGE: &'static str;
    /// Runs the game's own command `args` names; None when it names none.
    fn run(&self, args: &Args) -> Option<Result<()>>;
    /// The data folder `content` edits.
    fn data_dir(&self, args: &Args) -> Result<PathBuf>;
    /// The game manifest `content manifest` shows and edits (game.json).
    fn manifest_path(&self, args: &Args) -> Result<PathBuf>;
    /// The content writer's hooks.
    fn content(&self) -> Self::Content;
    /// The command host `serve` answers with: one request in, one reply out.
    fn host(&self, args: &Args) -> Result<Box<dyn FnMut(&Value) -> Value>>;
    /// A fresh command host as a replay drives it (`replay`: one per run).
    fn driver(&self, args: &Args) -> Result<Box<dyn aegis_host::Driver>>;
}

const USAGE: &str = "aegis serve   (JSON lines on stdin / stdout; the game's command API)
aegis replay <recording.json> [battle=K] [until=N] [json] [variant=<file>] [write=<out dir>] [seeds=a..b or a,b] [seed_key=seed] [fallback=stop or <json>]
aegis link '<json>' ... [port=47800]   (requests to a running game GUI)
aegis content check | upsert <kind> <json or @file> [world=<id>] [replace=<id>] | remove <kind> <id> [world=<id>] | apply <seed.json> | manifest [<json or @file>]";

/// Runs the command line of the process: the game's commands first, then the shell's.
pub fn run<G: Game>(game: &G) -> Result<()> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = Args::parse(&raw);
    if let Some(done) = game.run(&args) {
        return done;
    }
    match (args.pos(0), args.pos(1)) {
        (Some("serve"), _) => serve(game, &args),
        (Some("link"), _) => link(&raw),
        (Some("replay"), Some(_)) => replay(game, &args),
        (Some("content"), Some(_)) => content(game, &args),
        _ => {
            println!("{}\n{USAGE}", G::USAGE);
            Ok(())
        }
    }
}

/// A JSON argument: inline, or `@<file>`.
pub fn json_arg(raw: &str) -> Result<Value> {
    let text = match raw.strip_prefix('@') {
        Some(path) => std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?,
        None => raw.to_string(),
    };
    serde_json::from_str(&text).context("parsing the JSON argument")
}

/// One content step: {"op": "upsert", "kind", "entry", "world"?, "replace"?} or {"op": "remove", "kind", "id", "world"?}.
fn content_step(hooks: &impl Hooks, dir: &Path, step: &Value) -> std::result::Result<String, Vec<String>> {
    let s = |k: &str| step.get(k).and_then(|v| v.as_str()).map(String::from);
    let kind = s("kind").ok_or_else(|| vec!["step needs a kind".to_string()])?;
    let world = s("world");
    let (verb, outcome) = match s("op").as_deref().unwrap_or("upsert") {
        "remove" => {
            let id = s("id").ok_or_else(|| vec!["remove needs an id".to_string()])?;
            ("removed", store::remove(hooks, dir, world.as_deref(), &kind, &id)?)
        }
        "upsert" => {
            let entry = step.get("entry").and_then(|e| e.as_object()).ok_or_else(|| vec!["upsert needs an entry object".to_string()])?;
            let o = store::upsert(hooks, dir, world.as_deref(), &kind, entry, s("replace").as_deref())?;
            (if o.replaced { "replaced" } else { "added" }, o)
        }
        other => return Err(vec![format!("unknown op '{other}' (upsert or remove)")]),
    };
    let id = s("id").or_else(|| step.get("entry").and_then(|e| e.get("id")).and_then(|v| v.as_str()).map(String::from)).unwrap_or_default();
    Ok(format!("{verb} {kind} '{id}' in {}", outcome.file.display()))
}

/// `aegis content ...`: the data folder edited through the engine's writer (`aegis_content::store` with the game's
/// hooks), never by hand.
///   check                                              the game's warnings, then every problem (exit 1 when any)
///   upsert <kind> <json | @file> [world=<id>] [replace=<id>]   add or replace one entry (stored shape)
///   remove <kind> <id> [world=<id>]                    remove one entry
///   apply <seed.json>                                  run a seed: {"steps": [step...]} (stops at the first failure)
///   manifest [<json | @file>]                          show the game manifest, or merge fields into it (checked)
fn content<G: Game>(game: &G, args: &Args) -> Result<()> {
    let dir = game.data_dir(args)?;
    let hooks = game.content();
    let fail = |errors: Vec<String>| -> Result<()> {
        for e in &errors {
            eprintln!("error: {e}");
        }
        bail!("{} problem(s); nothing was written for the failing step", errors.len())
    };
    match args.pos(1) {
        Some("check") => {
            for w in hooks.warnings(&dir) {
                println!("warning: {w}");
            }
            let problems = hooks.check(&dir);
            for p in &problems {
                println!("{p}");
            }
            if !problems.is_empty() {
                bail!("{} problem(s) in {}", problems.len(), dir.display());
            }
            println!("data ok: {}", dir.display());
            Ok(())
        }
        Some("upsert") => {
            let (Some(kind), Some(raw)) = (args.pos(2), args.pos(3)) else { bail!("usage: aegis content upsert <kind> <json | @file> [world=<id>] [replace=<id>]") };
            let mut step = json!({"op": "upsert", "kind": kind, "entry": json_arg(raw)?});
            for k in ["world", "replace"] {
                if let Some(v) = args.keys.get(k) {
                    step[k] = json!(v);
                }
            }
            match content_step(&hooks, &dir, &step) {
                Ok(line) => Ok(println!("{line}")),
                Err(e) => fail(e),
            }
        }
        Some("remove") => {
            let (Some(kind), Some(id)) = (args.pos(2), args.pos(3)) else { bail!("usage: aegis content remove <kind> <id> [world=<id>]") };
            let mut step = json!({"op": "remove", "kind": kind, "id": id});
            if let Some(w) = args.keys.get("world") {
                step["world"] = json!(w);
            }
            match content_step(&hooks, &dir, &step) {
                Ok(line) => Ok(println!("{line}")),
                Err(e) => fail(e),
            }
        }
        Some("apply") => {
            let Some(path) = args.pos(2) else { bail!("usage: aegis content apply <seed.json>") };
            let seed = json_arg(&format!("@{path}"))?;
            let steps = seed.get("steps").and_then(|s| s.as_array()).cloned().or_else(|| seed.as_array().cloned()).context("a seed is {\"steps\": [...]} or a list of steps")?;
            for (i, step) in steps.iter().enumerate() {
                match content_step(&hooks, &dir, step) {
                    Ok(line) => println!("{}/{}: {line}", i + 1, steps.len()),
                    Err(e) => {
                        eprintln!("step {} of {} failed (earlier steps stay written):", i + 1, steps.len());
                        return fail(e);
                    }
                }
            }
            Ok(())
        }
        Some("manifest") => {
            let path = game.manifest_path(args)?;
            let Some(raw) = args.pos(2) else {
                let m = Manifest::read(&path).map_err(anyhow::Error::msg)?;
                print!("{}", stringify(&Value::Object(m.to_dict())));
                return Ok(());
            };
            let patch = json_arg(raw)?;
            let Some(patch) = patch.as_object() else { bail!("the manifest patch is a JSON object") };
            match Manifest::set(&path, patch) {
                Ok(_) => Ok(println!("wrote {}", path.display())),
                Err(e) => fail(e),
            }
        }
        _ => bail!("usage: aegis content check | upsert <kind> <json | @file> [world=<id>] [replace=<id>] | remove <kind> <id> [world=<id>] | apply <seed.json> | manifest [<json | @file>]"),
    }
}

/// `aegis serve`: the game's command API over stdin / stdout, one JSON request per line, one JSON reply per line.
fn serve(game: &impl Game, args: &Args) -> Result<()> {
    use std::io::{BufRead, Write};
    let mut host = game.host(args)?;
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(req) => host(&req),
            Err(e) => json!({"ok": false, "error": format!("bad JSON: {e}")}),
        };
        writeln!(out, "{reply}")?;
        out.flush()?;
    }
    Ok(())
}

/// `aegis link '<json>' ...`: sends requests to a running game GUI (agent link) and prints each reply line.
fn link(raw: &[String]) -> Result<()> {
    use std::io::{BufRead, BufReader, Write};
    let port = raw.iter().find_map(|a| a.strip_prefix("port=")).unwrap_or("47800");
    let stream = std::net::TcpStream::connect(format!("127.0.0.1:{port}")).with_context(|| format!("no GUI listening on port {port}"))?;
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

/// `aegis replay <recording.json>`: replays a playtest recording's sessions on fresh command hosts (PLAYTEST1 on the
/// engine). Lines start with `§`. Exit 0 when every checkpoint matches (stale content only warns), 1 on a divergence or
/// a broken request, 2 when the recording does not load.
///   battle=K            only session K (default every one)
///   until=N             stop after N requests (what a take-over replays before the player goes on)
///   json                one JSON report instead of the lines
///   variant=<file>      {"at", "remove"?, "requests": [...], "then"?: "stop" | "recorded"}: run the session with
///                       those requests in place of the recorded ones, print every `DIFF <at> <path> <recorded> ->
///                       <variant>`; write=<dir> saves the variant run as a recording (origin agent)
///   seeds=a..b | a,b    re-run the recorded requests under each seed (at most 200; seed_key=seed names the starting
///                       request's key); a failed request ends the run as broken@<index>, or with fallback=<json> sends
///                       that request instead; prints each result and the counts
fn replay<G: Game>(game: &G, args: &Args) -> Result<()> {
    use aegis_host::playtest::{self, Recording, Variant, Verdict};
    let path = PathBuf::from(args.pos(1).context("usage: aegis replay <recording.json>")?);
    let rec = match Recording::load(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("§ load error: {e}");
            std::process::exit(2);
        }
    };
    let changes = game.data_dir(args).map(|d| aegis_content::fingerprint::changes(&rec.content, &aegis_content::fingerprint::of(&d))).unwrap_or(Value::Null);
    let changed: Vec<String> = ["added", "removed", "changed"].iter().flat_map(|k| changes[*k].as_array().cloned().unwrap_or_default().into_iter().map(move |p| format!("{k} {}", p.as_str().unwrap_or("")))).collect();
    let only: Option<usize> = args.keys.get("battle").map(|b| b.parse()).transpose().context("battle=K")?;
    let picked: Vec<(usize, &playtest::Session)> = rec.battles.iter().enumerate().filter(|(i, _)| only.is_none_or(|k| k == *i)).collect();
    if picked.is_empty() {
        eprintln!("§ load error: the recording has no battle {}", only.map(|k| k.to_string()).unwrap_or_default());
        std::process::exit(2);
    }
    let json_out = args.flag("json");
    if !json_out {
        let content = if rec.content["total"].as_str().unwrap_or("").is_empty() {
            "unknown (no fingerprint)".to_string()
        } else if changes.is_null() {
            "unknown (no data folder to compare)".to_string()
        } else if changed.is_empty() {
            "current".to_string()
        } else {
            let more = if changed.len() > 8 { format!(" and {} more", changed.len() - 8) } else { String::new() };
            format!("stale: {}{more}", changed[..changed.len().min(8)].join(", "))
        };
        println!("§ recording {} ({} battles, {} notes), content {content}", rec.id, rec.battles.len(), rec.notes.len());
    }
    let mut failed = false;
    if args.keys.contains_key("until") && (args.keys.contains_key("seeds") || args.keys.contains_key("variant")) {
        bail!("until= applies to a plain replay, not to seeds= or variant=");
    }
    if args.keys.contains_key("variant") && picked.len() > 1 {
        bail!("a variant patches one battle: name it with battle=K");
    }
    if let Some(seeds) = args.keys.get("seeds") {
        let seeds = playtest::parse_seeds(seeds, 200).map_err(anyhow::Error::msg)?;
        let key = args.key("seed_key", "seed");
        let fallback = match args.keys.get("fallback").map(String::as_str) {
            None | Some("stop") => None,
            Some(raw) => Some(json_arg(raw)?),
        };
        let mut out = Vec::new();
        for (i, s) in &picked {
            let mut counts: IndexMap<String, usize> = IndexMap::new();
            for &seed in &seeds {
                let result = playtest::sweep_one(game.driver(args)?.as_mut(), s, &key, seed, fallback.as_ref());
                if !json_out {
                    println!("§ battle {i} seed {seed}: {result}");
                }
                *counts.entry(result.clone()).or_default() += 1;
                out.push(json!({"battle": i, "seed": seed, "result": result}));
            }
            if !json_out {
                println!("§ battle {i} summary: {}", counts.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(", "));
            }
        }
        if json_out {
            println!("{}", json!({"recording": rec.id, "stale": changes, "sweep": out}));
        }
        return Ok(());
    }
    if let Some(file) = args.keys.get("variant") {
        let variant = Variant::from_json(&json_arg(&format!("@{file}"))?).map_err(anyhow::Error::msg)?;
        let mut runs = Vec::new();
        for (i, s) in &picked {
            let run = playtest::run_variant(game.driver(args)?.as_mut(), s, &variant);
            if !json_out {
                for d in &run.diffs {
                    println!("§ battle {i} {d}");
                }
                if let Some((at, e)) = &run.broken {
                    println!("§ battle {i} broken at request {at}: {e}");
                }
                println!("§ battle {i} end: recorded {} -> variant {}", s.result(), run.session.result());
            }
            failed |= run.broken.is_some();
            runs.push((*i, run));
        }
        if let Some(dir) = args.keys.get("write") {
            let dir = PathBuf::from(dir);
            let mut out = Recording::new(&playtest::new_id(&dir, playtest::now_secs()), "agent", rec.content.clone());
            out.parent = json!({"id": rec.id, "battle": runs.first().map(|(i, _)| *i).unwrap_or(0), "at": variant.at});
            out.battles = runs.iter().map(|(_, r)| r.session.clone()).collect();
            let p = out.save(&dir).map_err(anyhow::Error::msg)?;
            println!("§ wrote {}", p.display());
        }
        if json_out {
            let rows: Vec<Value> = runs.iter().map(|(i, r)| json!({"battle": i, "diffs": r.diffs, "broken": r.broken.as_ref().map(|(a, e)| json!({"at": a, "error": e})), "end": r.session.end})).collect();
            println!("{}", json!({"recording": rec.id, "stale": changes, "variant": rows}));
        }
        if failed {
            std::process::exit(1);
        }
        return Ok(());
    }
    let until: Option<usize> = args.keys.get("until").map(|n| n.parse()).transpose().context("until=N")?;
    let mut rows = Vec::new();
    for (i, s) in &picked {
        let report = playtest::replay(game.driver(args)?.as_mut(), s, until);
        let checks: Vec<Value> = report
            .checks
            .iter()
            .map(|c| match &c.verdict {
                Verdict::Match => json!({"at": c.at, "verdict": "match"}),
                Verdict::Unverified => json!({"at": c.at, "verdict": "unverified"}),
                Verdict::Diverge { path, recorded, replayed } => json!({"at": c.at, "verdict": "diverge", "path": path, "recorded": recorded, "replayed": replayed}),
            })
            .collect();
        if !json_out {
            for c in &checks {
                match c["verdict"].as_str() {
                    Some("diverge") => println!("§ battle {i} at {}: diverge {} {} -> {}", c["at"], c["path"].as_str().unwrap_or(""), c["recorded"], c["replayed"]),
                    other => println!("§ battle {i} at {}: {}", c["at"], other.unwrap_or("")),
                }
            }
            if let Some((at, e)) = &report.broken {
                println!("§ battle {i} broken at request {at}: {e}");
            }
            println!("§ battle {i}: {} ({} requests, recorded end {})", if report.matched() { "match" } else { "differs" }, report.sent, s.result());
        }
        failed |= !report.matched();
        rows.push(json!({"battle": i, "match": report.matched(), "sent": report.sent, "checks": checks, "broken": report.broken.as_ref().map(|(a, e)| json!({"at": a, "error": e})), "end": s.end}));
    }
    if json_out {
        println!("{}", json!({"recording": rec.id, "stale": changes, "battles": rows}));
    }
    if failed {
        std::process::exit(1);
    }
    Ok(())
}
