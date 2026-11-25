//! `aegis`: the engine's command line shell for agents and humans, game-agnostic. The game registers itself at start
//! (`run(&game)`: its own commands, its data folder, its content hooks and its command host); the shell adds
//!   aegis serve                                                                the game's command API over stdin / stdout
//!   aegis link '<json>' ... [port=47800]                                       requests to a running GUI (agent link)
//!   aegis replay <recording.json> ...                                          replay a playtest recording (see `replay`)
//!   aegis record <requests.json | recording.json> [out=<dir>] [id=<id>]        record requests as a playtest recording (see `record`)
//!   aegis content check | upsert | remove | world | apply | manifest | <sidecar op>  edit the data folder (see `content`)
//!   aegis scenario check | run                                                 walk real screens from JSON files (see `scenario`)
//!   aegis playtest start | do | run | stop | status                          an agent plays the real game (see `agentplay`)
//!   aegis bundle <manifest.json> [out=<dir>]                                 pack a web build's bundle parts (see `bundle`)

pub mod agentplay;
pub mod align;
pub mod batch_sheet;
pub mod bundle;
pub mod golden;
pub mod new_game;
pub mod scenario;
pub mod sprite;

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
    /// ENGINE-5: the screens a scenario's `launch.screen` (and `scenario shots screen=`) may name: the ids the game's
    /// program takes as `screen=`. Default: the main menu only.
    fn launch_screens(&self) -> &'static [&'static str] {
        &["menu"]
    }
    /// The problems of a scenario's launch for this game (`scenario check`, `run`, `shots`); by default its screen is
    /// one of `launch_screens`. A game adds its own rules (which screens take `scene` or `ruleset`).
    fn launch_problems(&self, launch: &scenario::Launch) -> Vec<String> {
        scenario::screen_problems(launch, self.launch_screens())
    }
}

const USAGE: &str = "aegis serve   (JSON lines on stdin / stdout; the game's command API)
aegis replay <recording.json> [session=K] [until=N] [json] [variant=<file>] [write=<out dir>] [seeds=a..b or a,b] [seed_key=seed] [fallback=stop or <json>] [trail-report]
aegis record <requests.json | recording.json> [out=<dir>] [id=<id>]   (play requests on fresh command hosts, save them as a recording)
aegis link '<json>' ... [port=47800]   (requests to a running game GUI)
aegis content check | get <kind> <id> [world=<id>] | get @<list.json> | upsert <kind> <json or @file> [world=<id>] [replace=<id>] [fields=a,b] | remove <kind> <id> [world=<id>] | world <id> [<json or @file>] [full] | apply <seed.json> [check=1 | --check] | manifest [<json or @file>] | manifest --kit <id> [<json or @file>] | <sidecar op> <name> [<json or @file>] | refs <kind> <id> | rename <kind> <old> <new> [world=<id>] [--apply]
aegis scenario check [<file> | all] | run <id | all | file.json> [size=WxH] [out=<dir>] [slow=N] [exe=<game exe>] [data=<data dir>] [set=<key>:<value>,...] [audit=1]
aegis playtest start [save=] [out=] [size=] [shots=each|off] [keep_saves=<dir>] [window=1] | do <command ...> | run <script | -> | stop | status   (an agent plays the real game, logged and screenshot per command)
aegis bundle <manifest.json> [out=<dir>]   (pack a web build's bundle parts and bundle.json)
aegis align-map refspec | check-ref | enqueue | score | patch | run   (reference diorama validation, Drive queue bridge, and closed-loop map alignment)
aegis sprite chroma | key | slice | check | batch-sheet | import-set   (headless sprite chroma keying, slicing, drift QA, batch review, and character-set import)
aegis new game <id> [title=<title>] [root=<workspace>]   (a new game folder games/<id>/ on the engine alone: menu, play screen, host, test, scenario)";

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
        (Some("record"), Some(_)) => record(game, &args),
        (Some("content"), Some(_)) => content(game, &args),
        (Some("align-map"), _) => align::command(game, &args),
        (Some("sprite"), _) => sprite::command(&args),
        (Some("bundle"), Some(manifest)) => bundle::command(manifest, &args.key("out", ".")),
        (Some("new"), Some("game")) => new_game::command(&args),
        (Some("playtest"), _) => {
            let code = agentplay::command(game, &args, &raw).unwrap_or_else(|e| {
                println!("§ could not start: {e:#}");
                2
            });
            std::process::exit(code)
        }
        (Some("scenario"), _) => {
            let code = scenario::command(game, &args).unwrap_or_else(|e| {
                println!("§ could not start: {e:#}");
                2
            });
            std::process::exit(code)
        }
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

enum ParsedStep<'a> {
    Sidecar {
        op: String,
        name: String,
        edits: Value,
    },
    World {
        id: String,
        fields: &'a serde_json::Map<String, Value>,
        full: bool,
    },
    Remove {
        kind: String,
        id: String,
        world: Option<String>,
        if_exists: bool,
    },
    Upsert {
        kind: String,
        entry: &'a serde_json::Map<String, Value>,
        world: Option<String>,
        replace: Option<String>,
        fields: Option<Vec<String>>,
    },
}

fn check_unknown_step_keys(op: &str, obj: &serde_json::Map<String, Value>, allowed: &[&str]) -> std::result::Result<(), Vec<String>> {
    let mut unknown: Vec<&str> = obj.keys().map(String::as_str).filter(|k| !allowed.contains(k)).collect();
    if !unknown.is_empty() {
        unknown.sort_unstable();
        return Err(vec![format!("{op} step has unknown key(s): {} (allowed: {})", unknown.join(", "), allowed.join(", "))]);
    }
    Ok(())
}

fn parse_content_step<'a>(hooks: &(impl Hooks + ?Sized), step: &'a Value) -> std::result::Result<ParsedStep<'a>, Vec<String>> {
    let Some(obj) = step.as_object() else {
        return Err(vec!["step must be a JSON object".to_string()]);
    };
    let op = match obj.get("op") {
        None => "upsert",
        Some(Value::String(s)) => s.as_str(),
        Some(_) => return Err(vec!["step 'op' must be a string".to_string()]),
    };
    let opt_str = |key: &str| -> std::result::Result<Option<String>, Vec<String>> {
        match obj.get(key) {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(vec![format!("{op} '{key}' must be a string")]),
        }
    };
    let opt_bool = |key: &str| -> std::result::Result<bool, Vec<String>> {
        match obj.get(key) {
            None => Ok(false),
            Some(Value::Bool(b)) => Ok(*b),
            Some(_) => Err(vec![format!("{op} '{key}' must be a boolean")]),
        }
    };

    // PR5-48: a sidecar op of the game's writer, {"op": <op>, "name", "edits"} (no edits: written again as it is)
    if hooks.sidecar_ops().contains(&op) {
        check_unknown_step_keys(op, obj, &["op", "name", "edits"])?;
        let name = obj.get("name").and_then(Value::as_str).filter(|s| !s.is_empty()).ok_or_else(|| vec![format!("{op} needs a name")])?.to_string();
        let edits = obj.get("edits").cloned().unwrap_or_else(|| json!([]));
        return Ok(ParsedStep::Sidecar { op: op.to_string(), name, edits });
    }
    if op == "world" {
        check_unknown_step_keys("world", obj, &["op", "id", "fields", "full"])?;
        let id = obj.get("id").and_then(Value::as_str).filter(|s| !s.is_empty()).ok_or_else(|| vec!["world needs an id".to_string()])?.to_string();
        let fields = obj.get("fields").and_then(Value::as_object).ok_or_else(|| vec!["world needs a fields object".to_string()])?;
        let full = opt_bool("full")?;
        return Ok(ParsedStep::World { id, fields, full });
    }
    if op == "remove" {
        check_unknown_step_keys("remove", obj, &["op", "kind", "id", "world", "if_exists"])?;
        let kind = obj.get("kind").and_then(Value::as_str).filter(|s| !s.is_empty()).ok_or_else(|| vec!["step needs a kind".to_string()])?.to_string();
        let id = obj.get("id").and_then(Value::as_str).filter(|s| !s.is_empty()).ok_or_else(|| vec!["remove needs an id".to_string()])?.to_string();
        let world = opt_str("world")?;
        let if_exists = opt_bool("if_exists")?;
        return Ok(ParsedStep::Remove { kind, id, world, if_exists });
    }
    if op == "upsert" {
        check_unknown_step_keys("upsert", obj, &["op", "kind", "entry", "world", "replace", "fields"])?;
        let kind = obj.get("kind").and_then(Value::as_str).filter(|s| !s.is_empty()).ok_or_else(|| vec!["step needs a kind".to_string()])?.to_string();
        let entry = obj.get("entry").and_then(Value::as_object).ok_or_else(|| vec!["upsert needs an entry object".to_string()])?;
        let world = opt_str("world")?;
        let replace = opt_str("replace")?;
        let fields = match obj.get("fields") {
            None => None,
            Some(v) => {
                if replace.is_some() {
                    return Err(vec!["upsert cannot combine 'fields' with 'replace'".to_string()]);
                }
                let arr = v.as_array().ok_or_else(|| vec!["'fields' is a list of the entry's top-level field names".to_string()])?;
                if arr.is_empty() || !arr.iter().all(Value::is_string) {
                    return Err(vec!["'fields' is a list of the entry's top-level field names".to_string()]);
                }
                Some(arr.iter().filter_map(Value::as_str).map(String::from).collect())
            }
        };
        return Ok(ParsedStep::Upsert { kind, entry, world, replace, fields });
    }
    let extra: String = hooks.sidecar_ops().iter().map(|o| format!(", {o}")).collect();
    Err(vec![format!("unknown op '{op}' (upsert, remove, world{extra})")])
}

/// One content step: {"op": "upsert", "kind", "entry", "world"?, "replace"?, "fields"? (ENGINE-20: only these top-level
/// fields of the stored entry change, each to the entry's value or, left out of it, removed; the entry must exist)}, {"op": "remove", "kind", "id",
/// "world"?, "if_exists"? (true: a missing entry is a no-op, PR5-51b)}, {"op": "world", "id", "fields", "full"?} (a world manifest, WORLDW1) or {"op": <a sidecar op of the
/// game>, "name", "edits"?} (PR5-48, `Hooks::sidecar`).
pub fn content_step(hooks: &impl Hooks, dir: &Path, step: &Value) -> std::result::Result<String, Vec<String>> {
    match parse_content_step(hooks, step)? {
        ParsedStep::Sidecar { op, name, edits } => hooks.sidecar(dir, &op, &name, Some(&edits)),
        ParsedStep::World { id, fields, full } => {
            let o = store::set_world(hooks, dir, &id, fields, full)?;
            Ok(format!("{} world '{id}' in {}", if o.replaced { "updated" } else { "created" }, o.file.display()))
        }
        ParsedStep::Remove { kind, id, world, if_exists } => {
            // PR5-51b: `"if_exists": true` makes a re-run seed idempotent (a missing entry is a no-op, reported absent)
            if if_exists && !store::contains(hooks, dir, world.as_deref(), &kind, &id) {
                return Ok(format!("absent {kind} '{id}' in {} (nothing removed)", hooks.file_of(dir, world.as_deref(), &kind).display()));
            }
            let o = store::remove(hooks, dir, world.as_deref(), &kind, &id)?;
            Ok(format!("removed {kind} '{id}' in {}", o.file.display()))
        }
        ParsedStep::Upsert { kind, entry, world, replace, fields } => {
            let o = match fields {
                Some(names) => store::upsert_fields(hooks, dir, world.as_deref(), &kind, entry, &names)?,
                None => store::upsert(hooks, dir, world.as_deref(), &kind, entry, replace.as_deref())?,
            };
            let id = entry.get("id").and_then(Value::as_str).unwrap_or_default();
            Ok(format!("{} {kind} '{id}' in {}", if o.replaced { "replaced" } else { "added" }, o.file.display()))
        }
    }
}

/// ENGINE-20: non-mutating dry-run check of one content step (`aegis content apply <seed.json> check=1 | --check`).
/// Validates step shape, target kind/world/entry, and `store::merge_fields` / `Hooks::validate` without writing to disk.
pub fn check_content_step(hooks: &impl Hooks, dir: &Path, step: &Value) -> std::result::Result<String, Vec<String>> {
    match parse_content_step(hooks, step)? {
        ParsedStep::Sidecar { op, name, .. } => Ok(format!("ok {op} '{name}' (check)")),
        ParsedStep::World { id, fields, full } => {
            if !aegis_content::schema::is_id(&id) {
                return Err(vec![format!("World '{id}' must be a world id (lowercase snake_case).")]);
            }
            hooks.world_manifest_text(dir, &id, fields, full)?;
            Ok(format!("ok world '{id}' (check)"))
        }
        ParsedStep::Remove { kind, id, world, if_exists } => {
            let known = hooks.kinds(world.as_deref());
            if !known.contains(&kind.as_str()) {
                return Err(vec![format!("Unknown kind '{kind}' (one of {}).", known.join(", "))]);
            }
            if let Some(w) = world.as_deref() {
                if !aegis_core::vfs::is_file(aegis_content::world::manifest_path(dir, w)) {
                    return Err(vec![format!("Unknown world '{w}'.")]);
                }
            }
            if store::contains(hooks, dir, world.as_deref(), &kind, &id) {
                Ok(format!("ok remove {kind} '{id}' (check)"))
            } else if if_exists {
                Ok(format!("ok absent {kind} '{id}' (check)"))
            } else {
                Err(vec![format!("No {kind} entry '{id}'.")])
            }
        }
        ParsedStep::Upsert { kind, entry, world, replace, fields } => {
            if let Some(names) = fields {
                store::merge_fields(hooks, dir, world.as_deref(), &kind, entry, &names)?;
                let id = entry.get("id").and_then(Value::as_str).unwrap_or_default();
                return Ok(format!("ok upsert {kind} '{id}' (fields: {}, check)", names.join(", ")));
            }
            let known = hooks.kinds(world.as_deref());
            if !known.contains(&kind.as_str()) {
                return Err(vec![format!("Unknown kind '{kind}' (one of {}).", known.join(", "))]);
            }
            if let Some(w) = world.as_deref() {
                if !aegis_core::vfs::is_file(aegis_content::world::manifest_path(dir, w)) {
                    return Err(vec![format!("Unknown world '{w}'.")]);
                }
            }
            let norm = aegis_core::json::normalize_dict(entry);
            let full = hooks.validate(world.as_deref(), &kind, &norm)?;
            let id = aegis_content::schema::sget(&full, "id", "");
            let target = replace.as_deref().unwrap_or(&id);
            if let Some(rep) = replace.as_deref() {
                if !store::contains(hooks, dir, world.as_deref(), &kind, rep) {
                    return Err(vec![format!("No {kind} entry '{rep}' to replace.")]);
                }
            }
            if target != id && store::contains(hooks, dir, world.as_deref(), &kind, &id) {
                return Err(vec![format!("A {kind} entry '{id}' already exists.")]);
            }
            Ok(format!("ok upsert {kind} '{id}' (check)"))
        }
    }
}

/// ENGINE-38: the asset folder beside a game's data folder (`<game>/assets`), where art keyed by ids lives.
fn asset_roots(data_dir: &Path) -> Vec<PathBuf> {
    data_dir.parent().map(|g| g.join("assets")).filter(|a| a.is_dir()).into_iter().collect()
}

/// `aegis content ...`: the data folder edited through the engine's writer (`aegis_content::store` with the game's
/// hooks), never by hand.
///   check                                              the game's warnings, then every problem (exit 1 when any)
///   upsert <kind> <json | @file> [world=<id>] [replace=<id>] [fields=a,b]  add or replace one entry (or named fields)
///   remove <kind> <id> [world=<id>] [if_exists]        remove one entry (if_exists: a missing one is a no-op)
///   world <id> [<json | @file>] [full]                 show a world manifest, or create / update it (data/worlds/<id>/
///                                                      world.json; fields merged, checked, kept when the data loads)
///   apply <seed.json> [check=1 | --check]              run a seed: {"steps": [step...]} (or validate without writing)
///   manifest [<json | @file>]                          show the game manifest, or merge fields into it (checked)
///   manifest --kit <id> [<json | @file>]               show or write a kit manifest, kits/<id>/kit.json (ASSETORG2)
///   <sidecar op> <name> [<json | @file>]               PR5-48: show a sidecar of the game's writer, or apply edits to
///                                                      it (checked; nothing written when refused; `Hooks::sidecar`)
///   refs <kind> <id>                                   ENGINE-38: every string naming the id in the data and seeds,
///                                                      and the asset files named by it (`aegis_content::rename`)
///   rename <kind> <old> <new> [world=<id>] [--apply]   ENGINE-38: a dry run of the rename (the files and moves), or
///                                                      with --apply made and kept only when it adds no problem
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
            let (Some(kind), Some(raw)) = (args.pos(2), args.pos(3)) else { bail!("usage: aegis content upsert <kind> <json | @file> [world=<id>] [replace=<id>] [fields=a,b]") };
            let mut step = json!({"op": "upsert", "kind": kind, "entry": json_arg(raw)?});
            for k in ["world", "replace"] {
                if let Some(v) = args.keys.get(k) {
                    step[k] = json!(v);
                }
            }
            if let Some(raw_fields) = args.keys.get("fields") {
                let list: Vec<Value> = if raw_fields.is_empty() {
                    Vec::new()
                } else {
                    raw_fields.split(',').map(|s| json!(s.trim())).collect()
                };
                step["fields"] = Value::Array(list);
            }
            match content_step(&hooks, &dir, &step) {
                Ok(line) => Ok(println!("{line}")),
                Err(e) => fail(e),
            }
        }
        Some("remove") => {
            let (Some(kind), Some(id)) = (args.pos(2), args.pos(3)) else { bail!("usage: aegis content remove <kind> <id> [world=<id>] [if_exists]") };
            let mut step = json!({"op": "remove", "kind": kind, "id": id, "if_exists": args.flag("if_exists")});
            if let Some(w) = args.keys.get("world") {
                step["world"] = json!(w);
            }
            match content_step(&hooks, &dir, &step) {
                Ok(line) => Ok(println!("{line}")),
                Err(e) => fail(e),
            }
        }
        Some("refs") => {
            let (Some(_kind), Some(id)) = (args.pos(2), args.pos(3)) else { bail!("usage: aegis content refs <kind> <id>") };
            let found = aegis_content::rename::refs(&dir, &asset_roots(&dir), id);
            for r in &found {
                if r.line > 0 {
                    println!("{}:{}: {}", r.path.display(), r.line, r.text);
                } else {
                    println!("{}: asset", r.path.display());
                }
            }
            println!("{} reference(s) to '{id}'", found.len());
            Ok(())
        }
        Some("rename") => {
            let (Some(kind), Some(old), Some(new)) = (args.pos(2), args.pos(3), args.pos(4)) else { bail!("usage: aegis content rename <kind> <old> <new> [world=<id>] [--apply]") };
            let world = args.keys.get("world").map(String::as_str);
            if !aegis_content::rename::plain_id(new) {
                bail!("'{new}' is not an id (lowercase letters, digits, _, - and .)");
            }
            // ENGINE-38: a flag has no entry; every other kind renames a stored entry to an id not taken
            if kind != "flag" {
                if !aegis_content::store::contains(&hooks, &dir, world, kind, old) {
                    bail!("no {kind} entry '{old}' in {}", hooks.file_of(&dir, world, kind).display());
                }
                if aegis_content::store::contains(&hooks, &dir, world, kind, new) {
                    bail!("a {kind} entry '{new}' already exists");
                }
            }
            let roots = asset_roots(&dir);
            let plan = aegis_content::rename::plan(&dir, &roots, old, new);
            for (f, n) in &plan.files {
                println!("{}: {n} string(s)", f.display());
            }
            for (from, to) in &plan.moves {
                println!("move {} -> {}", from.display(), to.display());
            }
            if !args.flag("--apply") {
                println!("dry run: {} file(s), {} move(s); nothing written (add --apply)", plan.files.len(), plan.moves.len());
                return Ok(());
            }
            let before = hooks.check(&dir);
            match aegis_content::rename::apply(&plan, old, new, &before, || hooks.check(&dir)) {
                Ok(()) => Ok(println!("renamed '{old}' to '{new}': {} file(s), {} move(s)", plan.files.len(), plan.moves.len())),
                Err(e) => fail(e),
            }
        }
        Some("get") if args.pos(2).is_some_and(|a| a.starts_with('@')) => {
            // ENGINE-20: many entries in one run, `get @<list.json>` of [{"kind", "id", "world"?}]: one JSON array line,
            // null for an entry not stored (the seed order check, `tools/fc.sh seeds`)
            let list = json_arg(args.pos(2).unwrap_or_default())?;
            let out: Vec<Value> = list
                .as_array()
                .context("get @<file>: a list of {\"kind\", \"id\", \"world\"?}")?
                .iter()
                .map(|k| {
                    let f = |n: &str| k.get(n).and_then(Value::as_str);
                    match (f("kind"), f("id")) {
                        (Some(kind), Some(id)) => aegis_content::store::get(&hooks, &dir, f("world"), kind, id).map_or(Value::Null, Value::Object),
                        _ => Value::Null,
                    }
                })
                .collect();
            Ok(println!("{}", Value::Array(out)))
        }
        Some("get") => {
            // ENGINE-15: the stored entry as JSON (one line), for a seed built from it (`tools/fc.sh seed-from`)
            let (Some(kind), Some(id)) = (args.pos(2), args.pos(3)) else { bail!("usage: aegis content get <kind> <id> [world=<id>]") };
            let world = args.keys.get("world").map(String::as_str);
            match aegis_content::store::get(&hooks, &dir, world, kind, id) {
                Some(e) => Ok(println!("{}", serde_json::Value::Object(e))),
                None => bail!("no {kind} entry '{id}' in {}", hooks.file_of(&dir, world, kind).display()),
            }
        }
        Some("world") => {
            let Some(id) = args.pos(2) else { bail!("usage: aegis content world <id> [<json | @file>] [full]") };
            let Some(raw) = args.pos(3).filter(|r| *r != "full") else {
                let path = aegis_content::world::manifest_path(&dir, id);
                let text = std::fs::read_to_string(&path).with_context(|| format!("{}", path.display()))?;
                print!("{text}");
                return Ok(());
            };
            let step = json!({"op": "world", "id": id, "fields": json_arg(raw)?, "full": args.flag("full")});
            match content_step(&hooks, &dir, &step) {
                Ok(line) => Ok(println!("{line}")),
                Err(e) => fail(e),
            }
        }
        Some("apply") => {
            let check_only = args.key("check", "0") == "1" || args.flag("--check");
            let Some(path) = args.positional.iter().skip(2).find(|p| p.as_str() != "--check") else {
                bail!("usage: aegis content apply <seed.json> [check=1 | --check]")
            };
            let seed = json_arg(&format!("@{path}"))?;
            let steps = seed.get("steps").and_then(|s| s.as_array()).cloned().or_else(|| seed.as_array().cloned()).context("a seed is {\"steps\": [...]} or a list of steps")?;
            for (i, step) in steps.iter().enumerate() {
                let res = if check_only { check_content_step(&hooks, &dir, step) } else { content_step(&hooks, &dir, step) };
                match res {
                    Ok(line) => println!("{}/{}: {line}", i + 1, steps.len()),
                    Err(e) => {
                        if check_only {
                            eprintln!("step {} of {} failed (check mode: nothing written):", i + 1, steps.len());
                        } else {
                            eprintln!("step {} of {} failed (earlier steps stay written):", i + 1, steps.len());
                        }
                        return fail(e);
                    }
                }
            }
            if check_only {
                println!("check ok: {} step(s); nothing written", steps.len());
            }
            Ok(())
        }
        Some("manifest") if args.pos(2) == Some("--kit") => {
            // ASSETORG2: a kit's manifest, kits/<id>/kit.json in the game's workspace
            let game_json = game.manifest_path(args)?;
            let Some(id) = args.pos(3) else { bail!("usage: aegis content manifest --kit <id> [<json | @file>]") };
            if !aegis_content::schema::is_id(id) {
                bail!("kit '{id}' must be a kit id (lowercase snake_case)");
            }
            let Some(ws) = game_json.parent().and_then(aegis_content::manifest::workspace_of) else { bail!("no engine/assets above {}", game_json.display()) };
            let path = ws.join("kits").join(id).join(aegis_content::manifest::KIT_FILE);
            let Some(raw) = args.pos(4) else {
                let text = std::fs::read_to_string(&path).with_context(|| format!("{}", path.display()))?;
                print!("{text}");
                return Ok(());
            };
            let patch = json_arg(raw)?;
            let Some(patch) = patch.as_object() else { bail!("the kit manifest patch is a JSON object") };
            match aegis_content::manifest::set_kit(&path, patch) {
                Ok(_) => Ok(println!("wrote {}", path.display())),
                Err(e) => fail(e),
            }
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
            match aegis_content::store::set_manifest(&hooks, &path, patch) {
                Ok(_) => Ok(println!("wrote {}", path.display())),
                Err(e) => fail(e),
            }
        }
        Some(op) if hooks.sidecar_ops().contains(&op) => {
            let Some(name) = args.pos(2) else { bail!("usage: aegis content {op} <name> [<json | @file>]") };
            let edits = args.pos(3).map(json_arg).transpose()?;
            match sidecar_command(&hooks, &dir, op, name, edits.as_ref()) {
                Ok(out) => Ok(print!("{out}")),
                Err(e) => fail(e),
            }
        }
        _ => bail!("usage: aegis content check | upsert <kind> <json | @file> [world=<id>] [replace=<id>] [fields=a,b] | remove <kind> <id> [world=<id>] | world <id> [<json | @file>] [full] | apply <seed.json> [check=1 | --check] | manifest [<json | @file>] | manifest --kit <id> [<json | @file>] | <sidecar op> <name> [<json | @file>]"),
    }
}

/// PR5-48: `aegis content <sidecar op> <name> [<json | @file>]` as printed: the sidecar's text (`edits` None), else the
/// writer's line and a newline; the problems when refused (nothing written).
pub fn sidecar_command(hooks: &impl Hooks, dir: &Path, op: &str, name: &str, edits: Option<&Value>) -> std::result::Result<String, Vec<String>> {
    let out = hooks.sidecar(dir, op, name, edits)?;
    Ok(if edits.is_some() { format!("{out}\n") } else { out })
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
///   session=K           only session K (default every one; `battle=K` is its old name, LAYER-ENG-KEYS)
///   until=N             stop after N requests (what a take-over replays before the player goes on)
///   json                one JSON report instead of the lines
///   variant=<file>      {"at", "remove"?, "requests": [...], "then"?: "stop" | "recorded"}: run the session with
///                       those requests in place of the recorded ones, print every `DIFF <at> <path> <recorded> ->
///                       <variant>`; write=<dir> saves the variant run as a recording (origin agent)
///   trail-report        the UI trail's flags (dead clicks, repeats, long hovers, refused confirms, errors, panels)
///   seeds=a..b | a,b    re-run the recorded requests under each seed (at most 200; seed_key=seed names the starting
///                       request's key); a failed request ends the run as broken@<index>, or with fallback=<json> sends
///                       that request instead; prints each result and the counts
/// `aegis replay`'s `session=K`, or its old name `battle=K` (LAYER-ENG-KEYS); both at once is refused.
pub fn replay_session_arg(args: &Args) -> Result<Option<usize>> {
    let raw = match (args.keys.get("session"), args.keys.get("battle")) {
        (Some(_), Some(_)) => bail!("session=K or its old name battle=K, not both"),
        (Some(k), None) | (None, Some(k)) => Some(k),
        (None, None) => None,
    };
    raw.map(|k| k.parse()).transpose().context("session=K")
}

/// The sessions' requests `aegis record` plays: a list of requests (one session), a list of such lists, or a playtest
/// recording (its sessions' requests, recorded again).
pub fn record_sessions(input: &Value) -> Result<Vec<Vec<Value>>> {
    use aegis_host::playtest::Recording;
    if input["format"].is_string() {
        let rec = Recording::from_json(input).map_err(anyhow::Error::msg)?;
        return Ok(rec.sessions.into_iter().map(|s| s.requests).collect());
    }
    let list = input.as_array().context("record takes a list of requests, a list of such lists or a playtest recording")?;
    let sessions: Vec<Vec<Value>> = if list.iter().all(Value::is_array) { list.iter().map(|s| s.as_array().cloned().unwrap_or_default()).collect() } else { vec![list.clone()] };
    if sessions.is_empty() || sessions.iter().any(|s| s.is_empty() || s.iter().any(|r| !r["cmd"].is_string())) {
        bail!("every session is a non-empty list of requests, each an object with a cmd");
    }
    Ok(sessions)
}

/// `aegis record <requests.json | recording.json> [out=<dir>] [id=<id>]` (REPLAY-CI1): plays each session's requests on a
/// fresh command host and saves them, with the checkpoints the host took and the data folder's fingerprint, as a playtest
/// recording (origin agent) `<out>/<id>.json` (out default `.`, id default the time stamp). A refused request stops with
/// exit 1 and writes nothing. A recording given as input is recorded again on the data as it is now (a repro whose
/// checkpoints an intended rule or data change moved).
fn record<G: Game>(game: &G, args: &Args) -> Result<()> {
    use aegis_host::playtest::{self, Recording, Session};
    let path = args.pos(1).context("usage: aegis record <requests.json | recording.json>")?;
    let sessions = record_sessions(&json_arg(&format!("@{path}"))?)?;
    let dir = PathBuf::from(args.key("out", "."));
    let content = aegis_content::fingerprint::of(&game.data_dir(args)?);
    let id = args.keys.get("id").cloned().unwrap_or_else(|| playtest::new_id(&dir, playtest::now_secs()));
    let mut rec = Recording::new(&id, "agent", content);
    for (i, requests) in sessions.iter().enumerate() {
        let mut host = game.driver(args)?;
        for (k, req) in requests.iter().enumerate() {
            let reply = host.send(req);
            if reply["ok"] != json!(true) {
                println!("§ session {i} refused request {k}: {}", reply["error"].as_str().unwrap_or("refused"));
                std::process::exit(1);
            }
        }
        let s = Session::of(host.as_ref());
        println!("§ session {i}: {} requests, {} checkpoints, end {}", s.requests.len(), s.checkpoints.len(), s.result());
        rec.sessions.push(s);
    }
    let p = rec.save(&dir).map_err(anyhow::Error::msg)?;
    println!("§ wrote {}", p.display());
    Ok(())
}

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
    let only: Option<usize> = replay_session_arg(args)?;
    let picked: Vec<(usize, &playtest::Session)> = rec.sessions.iter().enumerate().filter(|(i, _)| only.is_none_or(|k| k == *i)).collect();
    if picked.is_empty() {
        eprintln!("§ load error: the recording has no session {}", only.map(|k| k.to_string()).unwrap_or_default());
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
        println!("§ recording {} ({} sessions, {} notes), content {content}", rec.id, rec.sessions.len(), rec.notes.len());
    }
    if args.flag("trail-report") {
        let report = aegis_host::trail::build(&rec.trail, rec.trail_dropped, &rec.notes);
        if json_out {
            println!("{report}");
        } else {
            for line in aegis_host::trail::lines(&report) {
                println!("§ {line}");
            }
        }
        return Ok(());
    }
    let mut failed = false;
    if args.keys.contains_key("until") && (args.keys.contains_key("seeds") || args.keys.contains_key("variant")) {
        bail!("until= applies to a plain replay, not to seeds= or variant=");
    }
    if args.keys.contains_key("variant") && picked.len() > 1 {
        bail!("a variant patches one session: name it with session=K");
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
                    println!("§ session {i} seed {seed}: {result}");
                }
                *counts.entry(result.clone()).or_default() += 1;
                out.push(json!({"session": i, "seed": seed, "result": result}));
            }
            if !json_out {
                println!("§ session {i} summary: {}", counts.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(", "));
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
                    println!("§ session {i} {d}");
                }
                if let Some((at, e)) = &run.broken {
                    println!("§ session {i} broken at request {at}: {e}");
                }
                println!("§ session {i} end: recorded {} -> variant {}", s.result(), run.session.result());
            }
            failed |= run.broken.is_some();
            runs.push((*i, run));
        }
        if let Some(dir) = args.keys.get("write") {
            let dir = PathBuf::from(dir);
            let mut out = Recording::new(&playtest::new_id(&dir, playtest::now_secs()), "agent", rec.content.clone());
            out.parent = json!({"id": rec.id, "session": runs.first().map(|(i, _)| *i).unwrap_or(0), "at": variant.at});
            out.sessions = runs.iter().map(|(_, r)| r.session.clone()).collect();
            let p = out.save(&dir).map_err(anyhow::Error::msg)?;
            println!("§ wrote {}", p.display());
        }
        if json_out {
            let rows: Vec<Value> = runs.iter().map(|(i, r)| json!({"session": i, "diffs": r.diffs, "broken": r.broken.as_ref().map(|(a, e)| json!({"at": a, "error": e})), "end": r.session.end})).collect();
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
                    Some("diverge") => println!("§ session {i} at {}: diverge {} {} -> {}", c["at"], c["path"].as_str().unwrap_or(""), c["recorded"], c["replayed"]),
                    other => println!("§ session {i} at {}: {}", c["at"], other.unwrap_or("")),
                }
            }
            if let Some((at, e)) = &report.broken {
                println!("§ session {i} broken at request {at}: {e}");
            }
            println!("§ session {i}: {} ({} requests, recorded end {})", if report.matched() { "match" } else { "differs" }, report.sent, s.result());
        }
        failed |= !report.matched();
        rows.push(json!({"session": i, "match": report.matched(), "sent": report.sent, "checks": checks, "broken": report.broken.as_ref().map(|(a, e)| json!({"at": a, "error": e})), "end": s.end}));
    }
    if json_out {
        println!("{}", json!({"recording": rec.id, "stale": changes, "sessions": rows}));
    }
    if failed {
        std::process::exit(1);
    }
    Ok(())
}
