//! `aegis playtest` (PR7-22, AGENT-PLAYTEST1; authoring: engine/docs/authoring/agent_playtest.md): a driver an agent
//! session plays a game's real program with, one command at a time, over the agent link the scenarios use
//! (`scenario.rs`). No game logic: it starts the program with no window (HEADLESS1) or in a real window, sends link
//! requests, and writes a log line per command and a numbered screenshot into a run folder.
//!   aegis playtest start [out=<dir>] [save=<dir | file>] [size=WxH] [seed=N] [screen=<id>] [shots=each | off]
//!                        [idle_exit=S] [window=1] [exe=<game exe>] [data=<data dir>] [fps=N]
//!   aegis playtest do <command ...> [run=<dir>]       one command on the running game (the words after `do`)
//!   aegis playtest run <script | -> [the keys of start] start, every line of the file (`-`: stdin), stop
//!   aegis playtest stop [run=<dir>]
//!   aegis playtest status [run=<dir>]
//! The run folder is `<out>/<YYYY-MM-DD>` (then `-2`, `-3`... when taken), default `out` `playtests/agent`; `<out>/current`
//! names the newest, which `do`, `stop` and `status` use unless `run=` says otherwise. Exit codes: 0 ok, 1 the command
//! failed, 2 could not start (or no running game).

use crate::scenario::{self, copy_dir, cond_holds, parse_cond, Cond, Link, TcpLink};
use crate::Args;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const OUT_DEFAULT: &str = "playtests/agent";
/// A game driven by hand waits this long for its next command before it leaves (`idle_exit=`).
pub const IDLE_DEFAULT_S: u64 = 3600;
/// A script runs its lines back to back, so a stalled driver need not leave a game behind for long.
pub const IDLE_SCRIPT_S: u64 = 300;
pub const WAIT_MS_DEFAULT: u64 = 10_000;
pub const WAIT_MS_MAX: u64 = 600_000;
pub const REPEAT_MAX: u64 = 500;
/// Widgets `screen` prints, at most.
pub const SCREEN_WIDGETS_MAX: usize = 80;
/// More clickable widgets than this under one id family (`tile.*`) print as one line.
pub const FAMILY_MAX: usize = 12;
const REPLY: Duration = Duration::from_secs(30);
const START: Duration = Duration::from_secs(120);
const LEAVE: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(30);
const SHOT_MAX_W: u64 = 1280;

// ---- the command language ----

/// One command line, parsed. Every action is a link request the scenarios also send.
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    /// The top screen, its stack and its widgets (`screen`), or the whole `ui_tree` reply into a file (`tree`).
    Screen,
    Tree,
    /// A request whose reply must not say `ok` false, then the frames after it.
    Act { body: Value, label: String },
    Wait { cond: Cond, text: String, ms: u64 },
    Assert { cond: Cond, text: String },
    /// A value of the `ui_tree` reply (`game:` first: of the `game` reply), printed.
    Get { path: String, game: bool },
    Shot { name: String },
    Pause { ms: u64 },
    /// The saves folder kept as `<run>/saves/<name>`, or put back from there with the game started again on it.
    Save { name: String },
    Load { name: String },
    /// A line of the agent's own in the log (a judgement, a finding): nothing is sent.
    Note { text: String },
    /// `repeat <n> until <cond> : <command>`: the command until the condition holds, at most n times.
    Repeat { times: u64, cond: Cond, text: String, inner: Box<Cmd> },
    /// `key <k> <n>`: the action n times.
    Times { times: u64, inner: Box<Cmd> },
    Quit,
}

impl Cmd {
    /// Whether the screen may change under it, so `shots=each` takes a picture after it.
    pub fn changes(&self) -> bool {
        matches!(self, Cmd::Act { .. } | Cmd::Wait { .. } | Cmd::Repeat { .. } | Cmd::Times { .. } | Cmd::Load { .. } | Cmd::Pause { .. })
    }
}

/// A value as written: JSON when it reads as JSON (`3`, `true`, `null`, `"text"`), else the text itself.
pub fn value_of(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|_| json!(text))
}

/// The short form of a condition, or a JSON one (`{"path": ..., "eq": ...}`, the scenario files' own):
/// `screen=<id>`, `widget=<id>`, `<path>=<v>`, `<path>!=<v>`, `<path>><n>`, `<path><<n>`, `<path>~<v>` (has),
/// `<path>?` (absent); `game:<path>` reads the `game` reply.
pub fn cond_of(text: &str) -> std::result::Result<Cond, String> {
    let text = text.trim();
    let v = if text.starts_with('{') {
        serde_json::from_str::<Value>(text).map_err(|e| format!("condition '{text}': {e}"))?
    } else if let Some(id) = text.strip_prefix("widget=") {
        json!({"widget": id})
    } else {
        let (source, rest) = match text.strip_prefix("game:") {
            Some(r) => ("game", r),
            None => ("ui_tree", text),
        };
        // The first operator in the text splits it (`!=` before `=` at one place); the value may hold any of them.
        let mut found: Option<(&str, &str, &str)> = None;
        for (op, key) in [("!=", "ne"), ("=", "eq"), (">", "gt"), ("<", "lt"), ("~", "has")] {
            if let Some(at) = rest.find(op) {
                if found.is_none_or(|(path, _, _)| at < path.len()) {
                    found = Some((&rest[..at], key, &rest[at + op.len()..]));
                }
            }
        }
        let mut o = match found {
            Some((path, key, value)) if !path.is_empty() => json!({"path": path, key: value_of(value)}),
            None if rest.ends_with('?') && rest.len() > 1 => json!({"path": &rest[..rest.len() - 1], "absent": true}),
            _ => return Err(format!("condition '{text}': write screen=<id>, widget=<id>, <path>=<v>, !=, >, <, ~ (has) or <path>? (absent)")),
        };
        if source == "game" {
            o["source"] = json!("game");
        }
        o
    };
    let mut problems = Vec::new();
    let c = parse_cond(&v, "condition", 0, &mut problems);
    if problems.is_empty() {
        Ok(c)
    } else {
        Err(problems.join("; "))
    }
}

fn ms_of(word: &str) -> Option<u64> {
    word.strip_prefix("ms=").or_else(|| word.strip_prefix("timeout=")).and_then(|n| n.parse().ok())
}

/// One line of the language. None for a blank line or a `#` comment.
pub fn parse_line(line: &str) -> Option<std::result::Result<Cmd, String>> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    Some(parse_cmd(line))
}

fn parse_cmd(line: &str) -> std::result::Result<Cmd, String> {
    let (head, rest) = line.split_once(char::is_whitespace).map(|(h, r)| (h, r.trim())).unwrap_or((line, ""));
    let words: Vec<&str> = rest.split_whitespace().collect();
    let one = |what: &str| -> std::result::Result<String, String> {
        match words.as_slice() {
            [w] => Ok(w.to_string()),
            _ => Err(format!("{head} takes one {what}")),
        }
    };
    let act = |body: Value| Ok(Cmd::Act { body, label: line.to_string() });
    match head {
        "screen" => Ok(Cmd::Screen),
        "tree" => Ok(Cmd::Tree),
        "click" => act(json!({"cmd": "click", "id": one("widget id")?})),
        "hover" => act(json!({"cmd": "hover", "id": one("widget id")?})),
        "key" => match words.as_slice() {
            [k] => act(json!({"cmd": "key", "key": k})),
            [k, n] => {
                let n: u64 = n.parse().ok().filter(|n| (1..=REPEAT_MAX).contains(n)).ok_or(format!("key {k} <times>: 1 to {REPEAT_MAX}"))?;
                Ok(Cmd::Times { times: n, inner: Box::new(Cmd::Act { body: json!({"cmd": "key", "key": k}), label: format!("key {k}") }) })
            }
            _ => Err("key <key> [times]".into()),
        },
        "type" if !rest.is_empty() => act(json!({"cmd": "type", "text": rest})),
        "back" => act(json!({"cmd": "back"})),
        "drag" => match words.as_slice() {
            [id, to] => {
                let to = match to.split_once(',').map(|(x, y)| (x.parse::<i64>(), y.parse::<i64>())) {
                    Some((Ok(x), Ok(y))) => json!([x, y]),
                    _ => json!(to),
                };
                act(json!({"cmd": "drag", "id": id, "to": to}))
            }
            _ => Err("drag <id> <to id | x,y>".into()),
        },
        "link" => {
            let body: Value = serde_json::from_str(rest).map_err(|e| format!("link <json>: {e}"))?;
            match body["cmd"].as_str() {
                Some("quit") => Err("link quit: write quit".into()),
                Some(_) => act(body),
                None => Err("link <json> needs a cmd".into()),
            }
        }
        "wait" | "assert" => {
            let (cond_words, ms): (Vec<&str>, Option<u64>) = match words.last().and_then(|w| ms_of(w)) {
                Some(ms) => (words[..words.len() - 1].to_vec(), Some(ms)),
                None => (words.clone(), None),
            };
            let text = cond_words.join(" ");
            if text.is_empty() {
                return Err(format!("{head} <condition>"));
            }
            let cond = cond_of(&text)?;
            if head == "assert" {
                return Ok(Cmd::Assert { cond, text });
            }
            let ms = ms.unwrap_or(WAIT_MS_DEFAULT);
            if ms == 0 || ms > WAIT_MS_MAX {
                return Err(format!("wait ms=: 1 to {WAIT_MS_MAX}"));
            }
            Ok(Cmd::Wait { cond, text, ms })
        }
        "get" => {
            let p = one("path")?;
            Ok(match p.strip_prefix("game:") {
                Some(path) => Cmd::Get { path: path.to_string(), game: true },
                None => Cmd::Get { path: p, game: false },
            })
        }
        "shot" => match words.as_slice() {
            [] => Ok(Cmd::Shot { name: "shot".into() }),
            [n] if plain(n) => Ok(Cmd::Shot { name: n.to_string() }),
            _ => Err("shot [name]: letters, digits, _ and -".into()),
        },
        "pause" => one("ms")?.parse().ok().filter(|n| (1..=WAIT_MS_MAX).contains(n)).map(|ms| Cmd::Pause { ms }).ok_or(format!("pause <ms>: 1 to {WAIT_MS_MAX}")),
        "frames" => {
            let n: u64 = one("count")?.parse().map_err(|_| "frames <count>".to_string())?;
            act(json!({"cmd": "frames", "frames": n}))
        }
        "save" | "load" => {
            let name = one("name")?;
            if !plain(&name) {
                return Err(format!("{head} <name>: letters, digits, _ and -"));
            }
            Ok(if head == "save" { Cmd::Save { name } } else { Cmd::Load { name } })
        }
        "note" if !rest.is_empty() => Ok(Cmd::Note { text: rest.to_string() }),
        "repeat" => {
            let (spec, cmd) = rest.split_once(" : ").ok_or("repeat <n> until <condition> : <command> (the colon between spaces)")?;
            let spec: Vec<&str> = spec.split_whitespace().collect();
            let (times, cond_text) = match spec.as_slice() {
                [n, "until", c @ ..] if !c.is_empty() => (n.parse::<u64>().ok().filter(|n| (1..=REPEAT_MAX).contains(n)).ok_or(format!("repeat <n>: 1 to {REPEAT_MAX}"))?, c.join(" ")),
                _ => return Err("repeat <n> until <condition> : <command>".into()),
            };
            let inner = parse_cmd(cmd.trim())?;
            if matches!(inner, Cmd::Repeat { .. } | Cmd::Times { .. } | Cmd::Save { .. } | Cmd::Load { .. } | Cmd::Quit | Cmd::Note { .. }) {
                return Err("repeat takes one command other than repeat, save, load, note or quit".into());
            }
            Ok(Cmd::Repeat { times, cond: cond_of(&cond_text)?, text: cond_text, inner: Box::new(inner) })
        }
        "quit" | "stop" => Ok(Cmd::Quit),
        _ => Err(format!("unknown command '{head}': screen, tree, click, hover, key, type, back, drag, link, wait, assert, get, shot, pause, frames, save, load, note, repeat, quit")),
    }
}

fn plain(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

// ---- the session ----

/// A running game this driver started: what `do` reads from `<run>/session.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub run: PathBuf,
    pub exe: PathBuf,
    pub args: Vec<String>,
    pub port: u16,
    pub pid: u32,
    pub step: u64,
    pub shots: bool,
    pub headless: bool,
}

impl Session {
    pub fn to_json(&self) -> Value {
        json!({"run": self.run, "exe": self.exe, "args": self.args, "port": self.port, "pid": self.pid, "step": self.step, "shots": if self.shots { "each" } else { "off" }, "headless": self.headless})
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        Some(Self {
            run: PathBuf::from(v["run"].as_str()?),
            exe: PathBuf::from(v["exe"].as_str()?),
            args: v["args"].as_array()?.iter().filter_map(|a| a.as_str().map(String::from)).collect(),
            port: v["port"].as_u64()? as u16,
            pid: v["pid"].as_u64()? as u32,
            step: v["step"].as_u64().unwrap_or(0),
            shots: v["shots"] != json!("off"),
            headless: v["headless"].as_bool().unwrap_or(true),
        })
    }

    fn file(run: &Path) -> PathBuf {
        run.join("session.json")
    }

    pub fn load(run: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(Self::file(run)).with_context(|| format!("no session in {} (start one: aegis playtest start)", run.display()))?;
        Self::from_json(&serde_json::from_str(&text)?).with_context(|| format!("{} is not a session file", Self::file(run).display()))
    }

    pub fn save(&self) -> Result<()> {
        std::fs::write(Self::file(&self.run), serde_json::to_string_pretty(&self.to_json())?)?;
        Ok(())
    }

    fn profile(&self) -> PathBuf {
        self.run.join("profile")
    }

    fn saves(&self) -> PathBuf {
        self.profile().join("saves")
    }
}

/// Days since 1970-01-01 as a civil date (Howard Hinnant's `civil_from_days`).
pub fn date_of(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { yoe + era * 400 + 1 } else { yoe + era * 400 }, m, d)
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn clock() -> String {
    let s = now_secs() % 86_400;
    format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

/// The run folder for today under `out`: `<date>`, or `<date>-<n>` with the first free n.
pub fn run_dir(out: &Path, secs: u64) -> PathBuf {
    let (y, m, d) = date_of((secs / 86_400) as i64);
    let date = format!("{y:04}-{m:02}-{d:02}");
    let mut n = 1;
    loop {
        let dir = if n == 1 { out.join(&date) } else { out.join(format!("{date}-{n}")) };
        if !dir.exists() {
            return dir;
        }
        n += 1;
    }
}

/// A file name part from a command line: its first words, letters and digits only.
pub fn slug(text: &str) -> String {
    let mut s: String = text.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect();
    while s.contains("__") {
        s = s.replace("__", "_");
    }
    s.trim_matches('_').chars().take(40).collect::<String>().trim_end_matches('_').to_string()
}

/// One log line as written to `log.md` (`log.jsonl` keeps the same as JSON).
pub fn log_line(step: u64, time: &str, line: &str, ok: bool, message: &str, screen: &str, shot: Option<&str>) -> String {
    let cell = |t: &str| t.replace('|', "\\|").replace(['\n', '\r'], " ").replace('`', "'");
    let shot = shot.map(|s| format!("[{s}]({s})")).unwrap_or_default();
    format!("| {step:03} | {time} | `{}` | {} | {} | {} | {shot} |", cell(line), if ok { "ok" } else { "FAIL" }, cell(message), cell(screen))
}

// ---- running a command ----

/// What one command did: whether it held, what to tell the agent, and what the log keeps.
#[derive(Debug, Clone, PartialEq)]
pub struct Done {
    pub ok: bool,
    pub message: String,
    /// What `do` prints (the screen summary, a value); empty for a plain ok.
    pub print: String,
    pub screen: String,
}

fn fail(message: impl Into<String>) -> Done {
    Done { ok: false, message: message.into(), print: String::new(), screen: String::new() }
}

/// Sends one command over a link; every file it writes goes under `run` with the number `step`. Save and load are the
/// caller's (they stop and start the game).
pub struct Driver<'a> {
    pub link: &'a mut dyn Link,
    pub run: &'a Path,
    pub step: u64,
    pub headless: bool,
}

impl Driver<'_> {
    fn ask(&mut self, body: &Value) -> std::result::Result<Value, String> {
        self.link.ask(body).map_err(|e| format!("the link: {e} (is the game still running? see game.log)"))
    }

    fn tree(&mut self) -> std::result::Result<Value, String> {
        let t = self.ask(&json!({"cmd": "ui_tree"}))?;
        if t["frame"].as_u64().is_none() {
            return Err(format!("ui_tree has no frame count: {}", cut(&t.to_string(), 200)));
        }
        Ok(t)
    }

    /// Two more frames drawn, as a scenario's steps wait: with no window asked for, in a window polled for.
    pub fn settle(&mut self) -> std::result::Result<Value, String> {
        if self.headless {
            let r = self.ask(&json!({"cmd": "frames", "frames": scenario::SETTLE_FRAMES}))?;
            if r["ok"] != json!(true) {
                return Err(format!("the frames asked for: {}", r["error"]));
            }
            return self.tree();
        }
        let mut t = self.tree()?;
        let want = t["frame"].as_u64().unwrap_or(0) + scenario::SETTLE_FRAMES;
        let by = Instant::now() + Duration::from_millis(scenario::SETTLE_MS);
        while t["frame"].as_u64().unwrap_or(0) < want {
            if Instant::now() >= by {
                return Err("the window drew no frame".into());
            }
            std::thread::sleep(POLL);
            t = self.tree()?;
        }
        Ok(t)
    }

    fn holds(&mut self, c: &Cond) -> std::result::Result<std::result::Result<(), String>, String> {
        let tree = self.tree()?;
        let game = if needs_game(c) { self.ask(&json!({"cmd": "game"}))? } else { Value::Null };
        Ok(cond_holds(c, &tree, &game))
    }

    /// A screenshot `<step>_<name>.png` once the frames are drawn; its file name.
    pub fn shot(&mut self, name: &str) -> std::result::Result<String, String> {
        self.settle()?;
        let file = format!("{:03}_{name}.png", self.step);
        let r = self.ask(&json!({"cmd": "screenshot", "path": self.run.join(&file).display().to_string(), "max_w": SHOT_MAX_W}))?;
        if r["ok"] != json!(true) {
            return Err(format!("the shot: {}", r["error"]));
        }
        Ok(file)
    }

    pub fn exec(&mut self, cmd: &Cmd) -> Done {
        match self.exec_inner(cmd) {
            Ok(d) => d,
            Err(e) => fail(e),
        }
    }

    fn exec_inner(&mut self, cmd: &Cmd) -> std::result::Result<Done, String> {
        let ok = |message: String, print: String| Done { ok: true, message, print, screen: String::new() };
        Ok(match cmd {
            Cmd::Screen => {
                let t = self.tree()?;
                ok(String::new(), screen_summary(&t))
            }
            Cmd::Tree => {
                let t = self.tree()?;
                let file = format!("{:03}_tree.json", self.step);
                std::fs::write(self.run.join(&file), serde_json::to_string_pretty(&t).unwrap_or_default()).map_err(|e| e.to_string())?;
                ok(file.clone(), format!("wrote {}", self.run.join(file).display()))
            }
            Cmd::Act { body, .. } => {
                let r = self.ask(body)?;
                if r["ok"] == json!(false) {
                    return Ok(fail(format!("refused: {}", r["error"].as_str().unwrap_or("no reason given"))));
                }
                self.settle()?;
                let extra = r.as_object().map(|o| o.iter().filter(|(k, _)| *k != "ok").map(|(k, v)| format!("{k}={}", cut(&v.to_string(), 120))).collect::<Vec<_>>().join(" ")).unwrap_or_default();
                ok(extra, String::new())
            }
            Cmd::Wait { cond, text, ms } => {
                let start = Instant::now();
                let by = start + Duration::from_millis(*ms);
                loop {
                    match self.holds(cond)? {
                        Ok(()) => break ok(format!("after {} ms", start.elapsed().as_millis()), String::new()),
                        Err(why) if Instant::now() >= by => break fail(format!("{text}: timed out after {ms} ms: {why}")),
                        Err(_) => std::thread::sleep(POLL),
                    }
                }
            }
            Cmd::Assert { cond, text } => match self.holds(cond)? {
                Ok(()) => ok(String::new(), String::new()),
                Err(why) => fail(format!("{text}: {why}")),
            },
            Cmd::Get { path, game } => {
                let v = if *game { self.ask(&json!({"cmd": "game"}))? } else { self.tree()? };
                let found = scenario::at_path(&v, path).cloned().unwrap_or(Value::Null);
                let text = serde_json::to_string_pretty(&found).unwrap_or_default();
                ok(cut(&found.to_string(), 160), text)
            }
            Cmd::Shot { name } => {
                let file = self.shot(name)?;
                ok(file, String::new())
            }
            Cmd::Pause { ms } => {
                // A look now and then: a game with no window leaves when nothing asks for a while.
                let by = Instant::now() + Duration::from_millis(*ms);
                while Instant::now() < by {
                    std::thread::sleep(by.saturating_duration_since(Instant::now()).min(Duration::from_secs(1)));
                    self.tree()?;
                }
                ok(String::new(), String::new())
            }
            Cmd::Note { .. } => ok(String::new(), String::new()),
            Cmd::Repeat { times, cond, text, inner } => {
                for turn in 0..*times {
                    if self.holds(cond)?.is_ok() {
                        return Ok(ok(format!("{turn} turns"), String::new()));
                    }
                    let d = self.exec(inner);
                    if !d.ok {
                        return Ok(fail(format!("turn {}: {}", turn + 1, d.message)));
                    }
                }
                match self.holds(cond)? {
                    Ok(()) => ok(format!("{times} turns"), String::new()),
                    Err(why) => fail(format!("{times} turns and {text}: {why}")),
                }
            }
            Cmd::Times { times, inner } => {
                for turn in 0..*times {
                    let d = self.exec(inner);
                    if !d.ok {
                        return Ok(fail(format!("turn {}: {}", turn + 1, d.message)));
                    }
                }
                ok(format!("{times} times"), String::new())
            }
            Cmd::Save { .. } | Cmd::Load { .. } | Cmd::Quit => return Err("save, load and quit are the session's".into()),
        })
    }
}

fn needs_game(c: &Cond) -> bool {
    match c {
        Cond::Path { source, .. } => *source == scenario::Source::Game,
        Cond::Widget(_) => false,
        Cond::All(list) => list.iter().any(needs_game),
    }
}

fn cut(text: &str, max: usize) -> String {
    let mut s: String = text.chars().take(max).collect();
    if text.chars().count() > max {
        s.push('…');
    }
    s
}

/// What `screen` prints: the top screen, the stack, the state in short, and the widgets (clickable first).
pub fn screen_summary(t: &Value) -> String {
    let mut out = format!("screen: {}\nstack: {}\n", t["screen"].as_str().unwrap_or("?"), t["stack"].as_array().map(|s| s.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" > ")).unwrap_or_default());
    if !t["state"].is_null() {
        out.push_str(&format!("state: {}\n", cut(&t["state"].to_string(), 600)));
    }
    if let Some(e) = t["edit"].as_object() {
        out.push_str(&format!("editing: {}\n", e.get("id").and_then(Value::as_str).unwrap_or("?")));
    }
    let widgets: Vec<&Value> = t["widgets"].as_array().map(|w| w.iter().collect()).unwrap_or_default();
    let (click, info): (Vec<&Value>, Vec<&Value>) = widgets.into_iter().partition(|w| w["clickable"] == json!(true));
    let row = |w: &Value| {
        let mut r = format!("  {}", w["id"].as_str().unwrap_or("?"));
        let text = w["text"].as_str().unwrap_or("");
        if !text.is_empty() {
            r.push_str(&format!("  \"{}\"", cut(text, 60)));
        }
        if w["enabled"] == json!(false) {
            r.push_str("  (disabled)");
        }
        if w["active"] == json!(true) {
            r.push_str("  (active)");
        }
        r
    };
    out.push_str(&format!("clickable ({}):\n", click.len()));
    // A big family of rows of one kind (a board's `tile.x.y`, a list's `board.row.<id>`) is one line: its first and last
    // id. Only ids of three parts or more fold, so a family's own buttons (`board.close`) stay listed.
    let family = |w: &Value| {
        let id = w["id"].as_str().unwrap_or("");
        let parts: Vec<&str> = id.splitn(3, '.').collect();
        if parts.len() == 3 { parts[0].to_string() } else { id.to_string() }
    };
    let mut counts: std::collections::BTreeMap<String, Vec<&str>> = std::collections::BTreeMap::new();
    for w in &click {
        counts.entry(family(w)).or_default().push(w["id"].as_str().unwrap_or("?"));
    }
    let mut folded = std::collections::BTreeSet::new();
    let mut listed = 0;
    for w in &click {
        let f = family(w);
        let ids = &counts[&f];
        if ids.len() > FAMILY_MAX {
            if folded.insert(f.clone()) {
                out.push_str(&format!("  {f}.* ({} widgets, {} .. {}; tree lists them)\n", ids.len(), ids[0], ids[ids.len() - 1]));
            }
            continue;
        }
        if listed < SCREEN_WIDGETS_MAX {
            out.push_str(&row(w));
            out.push('\n');
        }
        listed += 1;
    }
    let room = SCREEN_WIDGETS_MAX.saturating_sub(listed);
    let shown: Vec<&&Value> = info.iter().filter(|w| !w["text"].as_str().unwrap_or("").is_empty()).take(room).collect();
    if !shown.is_empty() {
        out.push_str(&format!("text ({}):\n", shown.len()));
        for w in shown {
            out.push_str(&row(w));
            out.push('\n');
        }
    }
    if listed > SCREEN_WIDGETS_MAX {
        out.push_str(&format!("  ... {} more clickable (tree: the whole reply)\n", listed - SCREEN_WIDGETS_MAX));
    }
    out
}

// ---- the process ----

fn spawn(s: &Session) -> Result<u32> {
    let log = std::fs::OpenOptions::new().create(true).append(true).open(s.run.join("game.log"))?;
    let mut c = std::process::Command::new(&s.exe);
    c.args(&s.args).env("APPDATA", s.profile()).stdin(std::process::Stdio::null()).stdout(log.try_clone()?).stderr(log);
    // Its own process group: the game outlives this command (each `do` is a new process) and a shell's hang-up.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        c.process_group(0);
    }
    let child = c.spawn().with_context(|| format!("starting {}", s.exe.display()))?;
    Ok(child.id())
}

fn connect(s: &Session) -> std::io::Result<TcpLink> {
    TcpLink::connect(s.port, REPLY)
}

/// Waits until the started game answers on its port as itself and has drawn its first frames.
fn wait_ready(s: &Session) -> Result<()> {
    let by = Instant::now() + START;
    loop {
        if let Ok(mut link) = connect(s) {
            if let Ok(t) = link.ask(&json!({"cmd": "ui_tree"})) {
                if t["pid"] != json!(s.pid) {
                    bail!("port {} answers for process {}, not the game started ({})", s.port, t["pid"], s.pid);
                }
                if t["frame"].as_u64().unwrap_or(0) >= scenario::SETTLE_FRAMES {
                    return Ok(());
                }
            }
        }
        if !alive(s.pid) {
            bail!("the game ended while starting:\n{}", tail(&s.run.join("game.log"), 30));
        }
        if Instant::now() >= by {
            bail!("nothing answered on port {} in {} s", s.port, START.as_secs());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn alive(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        // A zombie (ended, not yet reaped by its parent) counts as ended.
        std::fs::read_to_string(format!("/proc/{pid}/stat")).map(|s| !s.rsplit(')').next().unwrap_or("").trim_start().starts_with('Z')).unwrap_or(false)
    }
    // Elsewhere there is no cheap look at another process: the link answering is the sign of life.
    #[cfg(not(target_os = "linux"))]
    {
        pid > 0
    }
}

fn kill(pid: u32) {
    #[cfg(unix)]
    let _ = std::process::Command::new("kill").args(["-9", &pid.to_string()]).status();
    #[cfg(windows)]
    let _ = std::process::Command::new("taskkill").args(["/F", "/PID", &pid.to_string()]).status();
    // No process to kill on another target (the web build compiles this crate too).
    #[cfg(not(any(unix, windows)))]
    let _ = pid;
}

/// Asks the game to leave over the link, then kills it if it has not.
fn stop_game(s: &Session) -> &'static str {
    if let Ok(mut link) = connect(s) {
        link.patience(LEAVE);
        let _ = link.ask(&json!({"cmd": "quit"}));
    }
    let by = Instant::now() + LEAVE;
    while Instant::now() < by {
        if connect(s).is_err() && !alive(s.pid) {
            return "left";
        }
        // Elsewhere the closed port is the only sign: a moment more for the process to let go of its files.
        if connect(s).is_err() && cfg!(not(target_os = "linux")) {
            std::thread::sleep(Duration::from_millis(500));
            return "left";
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    kill(s.pid);
    "killed"
}

fn tail(path: &Path, lines: usize) -> String {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let all: Vec<&str> = text.lines().collect();
    all[all.len().saturating_sub(lines)..].join("\n")
}

// ---- the command line ----

const USAGE: &str = "aegis playtest start [out=<dir>] [save=<dir | file>] [size=WxH] [seed=N] [screen=<id>] [shots=each | off] [idle_exit=S] [window=1] [exe=] [data=] [fps=N]
aegis playtest do <command ...> [run=<dir>]
aegis playtest run <script | -> [the keys of start]
aegis playtest stop [run=<dir>] | status [run=<dir>]
commands: screen | tree | click <id> | hover <id> | key <key> [n] | type <text> | back | drag <id> <to> | link <json> |
  wait <cond> [ms=N] | assert <cond> | get <path | game:path> | shot [name] | pause <ms> | frames <n> | save <name> |
  load <name> | note <text> | repeat <n> until <cond> : <command> | quit
conditions: screen=<id> | widget=<id> | <path>=<v> | <path>!=<v> | <path>><n> | <path><<n> | <path>~<v> | <path>? | game:<path>... | {json}";

/// `aegis playtest ...`; `raw` is the whole command line (the words after `do` are the command, `=` and all).
pub fn command<G: crate::Game>(game: &G, args: &Args, raw: &[String]) -> Result<i32> {
    match args.pos(1) {
        Some("start") => {
            let s = start(game, args, IDLE_DEFAULT_S)?;
            println!("§ started {} (pid {}, port {}); run folder {}", s.exe.display(), s.pid, s.port, s.run.display());
            Ok(0)
        }
        Some("do") => {
            let run = current_run(args)?;
            let line = raw.iter().skip_while(|a| a.as_str() != "do").skip(1).filter(|a| !a.starts_with("run=") && !a.starts_with("out=")).cloned().collect::<Vec<_>>().join(" ");
            let mut s = Session::load(&run)?;
            Ok(do_line(&mut s, &line))
        }
        Some("run") => {
            let Some(script) = args.pos(2) else {
                println!("usage: {USAGE}");
                return Ok(2);
            };
            let text = if script == "-" {
                let mut t = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut t)?;
                t
            } else {
                std::fs::read_to_string(script).with_context(|| format!("reading {script}"))?
            };
            // Every line is parsed before the game starts: a typo fails at once.
            let bad: Vec<String> = text.lines().enumerate().filter_map(|(i, l)| match parse_line(l) {
                Some(Err(e)) => Some(format!("line {}: {e}", i + 1)),
                _ => None,
            }).collect();
            if !bad.is_empty() {
                for b in &bad {
                    println!("§ {b}");
                }
                return Ok(2);
            }
            let mut s = start(game, args, IDLE_SCRIPT_S)?;
            println!("§ started (pid {}, port {}); run folder {}", s.pid, s.port, s.run.display());
            let mut code = 0;
            for line in text.lines().filter(|l| parse_line(l).is_some()) {
                code = do_line(&mut s, line);
                if code != 0 || matches!(parse_line(line), Some(Ok(Cmd::Quit))) {
                    break;
                }
            }
            if alive_session(&s) {
                println!("§ stop: {}", stop_game(&s));
            }
            check_exit(&s, &mut code);
            println!("§ {} ({} commands); log {}", if code == 0 { "pass" } else { "FAIL" }, s.step, s.run.join("log.md").display());
            Ok(code)
        }
        Some("stop") => {
            let s = Session::load(&current_run(args)?)?;
            println!("§ stop: {}", stop_game(&s));
            Ok(0)
        }
        Some("status") => {
            let s = Session::load(&current_run(args)?)?;
            println!("§ run {} pid {} port {} step {} {}", s.run.display(), s.pid, s.port, s.step, if alive_session(&s) { "running" } else { "not running" });
            Ok(0)
        }
        _ => {
            println!("usage: {USAGE}");
            Ok(2)
        }
    }
}

fn alive_session(s: &Session) -> bool {
    connect(s).is_ok()
}

/// A panic anywhere in the game's output fails the run, as in a scenario.
fn check_exit(s: &Session, code: &mut i32) {
    let log = std::fs::read_to_string(s.run.join("game.log")).unwrap_or_default();
    if let Some(line) = log.lines().find(|l| l.contains("panicked at")) {
        println!("§ the game panicked: {line}");
        *code = 1;
    }
}

fn current_run(args: &Args) -> Result<PathBuf> {
    if let Some(r) = args.keys.get("run") {
        return Ok(PathBuf::from(r));
    }
    let out = PathBuf::from(args.key("out", OUT_DEFAULT));
    let text = std::fs::read_to_string(out.join("current")).with_context(|| format!("no current run in {} (start one, or pass run=<dir>)", out.display()))?;
    Ok(PathBuf::from(text.trim()))
}

fn start<G: crate::Game>(game: &G, args: &Args, idle_default: u64) -> Result<Session> {
    let mut keys = args.keys.clone();
    keys.shift_remove("data");
    let manifest_path = game.manifest_path(&Args { positional: args.positional.clone(), keys })?;
    let manifest = aegis_content::manifest::Manifest::read(&manifest_path).map_err(anyhow::Error::msg)?;
    let exe = match args.keys.get("exe") {
        Some(p) => PathBuf::from(p),
        None => std::env::current_exe()?.with_file_name(format!("{}{}", manifest.id, std::env::consts::EXE_SUFFIX)),
    };
    if !exe.is_file() {
        bail!("no game program at {} (build it, or pass exe=)", exe.display());
    }
    let size = scenario::parse_size(&args.key("size", "1280x720")).map_err(anyhow::Error::msg)?;
    let shots = match args.key("shots", "each").as_str() {
        "each" => true,
        "off" => false,
        o => bail!("shots is each or off, not '{o}'"),
    };
    let headless = !scenario::window_asked(args)?;
    let idle: u64 = args.key("idle_exit", &idle_default.to_string()).parse().context("idle_exit=S")?;
    let out = PathBuf::from(args.key("out", OUT_DEFAULT));
    let run = run_dir(&out, now_secs());
    std::fs::create_dir_all(&run)?;
    // Not `canonicalize`: on Windows it gives `\\?\` paths, which the game and the log would carry.
    let run = std::path::absolute(&run)?;
    let port = scenario::free_port()?;
    let user = scenario::user_dir_under(&run.join("profile"), &manifest.id);
    std::fs::create_dir_all(&user)?;
    std::fs::create_dir_all(run.join("profile").join("saves"))?;
    let settings = json!({"mute": true, "fullscreen": false, "borderless": false, "window_size": format!("{}x{}", size.0, size.1), "vsync": false, "record_playtests": true, "start_inactive": true});
    std::fs::write(user.join("settings.json"), serde_json::to_string_pretty(&settings)?)?;
    let mut s = Session { run: run.clone(), exe, args: Vec::new(), port, pid: 0, step: 0, shots, headless };
    if let Some(from) = args.keys.get("save") {
        let from = PathBuf::from(from);
        if from.is_dir() {
            copy_dir(&from, &s.saves())?;
        } else {
            let name = from.file_name().context("save= names a file or a folder")?;
            std::fs::copy(&from, s.saves().join(name)).with_context(|| format!("save={}", from.display()))?;
        }
    }
    let mut a = vec![format!("port={port}"), format!("saves={}", s.saves().display()), format!("screen={}", args.key("screen", "menu"))];
    for k in ["seed", "data", "scene"] {
        if let Some(v) = args.keys.get(k) {
            a.push(format!("{k}={v}"));
        }
    }
    if headless {
        let o = scenario::Options { game_id: manifest.id.clone(), exe: s.exe.clone(), data: None, out: out.clone(), slow: 1, size: None, sized_ms: 0, window: false, fps: args.keys.get("fps").map(|f| f.parse()).transpose().context("fps=N")?, dpr: None, baselines: None, record: false, saves: None };
        a.extend(scenario::headless_args(size, &o));
        a.push(format!("idle_exit={idle}"));
    }
    s.args = a;
    std::fs::write(run.join("log.md"), format!("# Agent playtest {}\n\nGame `{}`, {}x{}, {}. Args: `{}`\n\n| # | Time (UTC) | Command | Result | Note | Screen | Shot |\n| --- | --- | --- | --- | --- | --- | --- |\n", run.file_name().unwrap_or_default().to_string_lossy(), manifest.id, size.0, size.1, if headless { "no window" } else { "window" }, s.args.join(" ")))?;
    s.pid = spawn(&s)?;
    s.save()?;
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("current"), run.display().to_string())?;
    if let Err(e) = wait_ready(&s) {
        kill(s.pid);
        return Err(e);
    }
    Ok(s)
}

/// One command line on the session: run, logged, a shot after it under `shots=each`; the exit code.
fn do_line(s: &mut Session, line: &str) -> i32 {
    let cmd = match parse_line(line) {
        None => return 0,
        Some(Err(e)) => {
            println!("§ {e}");
            return 2;
        }
        Some(Ok(c)) => c,
    };
    s.step += 1;
    let _ = s.save();
    let done = match &cmd {
        Cmd::Quit => Done { ok: true, message: stop_game(s).into(), print: String::new(), screen: String::new() },
        Cmd::Save { name } => match copy_dir(&s.saves(), &s.run.join("saves").join(name)) {
            Ok(()) => Done { ok: true, message: format!("saves/{name}"), print: String::new(), screen: String::new() },
            Err(e) => fail(format!("saving: {e}")),
        },
        Cmd::Load { name } => load(s, name),
        _ => match connect(s) {
            Err(e) => fail(format!("no game on port {} ({e}); its log ends:\n{}", s.port, tail(&s.run.join("game.log"), 15))),
            Ok(mut link) => Driver { link: &mut link, run: &s.run, step: s.step, headless: s.headless }.exec(&cmd),
        },
    };
    finish(s, line, &cmd, done)
}

fn load(s: &mut Session, name: &str) -> Done {
    let from = s.run.join("saves").join(name);
    if !from.is_dir() {
        return fail(format!("no saves/{name} (save {name} first)"));
    }
    stop_game(s);
    let _ = std::fs::remove_dir_all(s.saves());
    if let Err(e) = copy_dir(&from, &s.saves()) {
        return fail(format!("loading: {e}"));
    }
    match scenario::free_port().map_err(anyhow::Error::from).and_then(|port| {
        let old = format!("port={}", s.port);
        s.args.iter_mut().filter(|a| **a == old).for_each(|a| *a = format!("port={port}"));
        s.port = port;
        s.pid = spawn(s)?;
        let _ = s.save();
        wait_ready(s)
    }) {
        Ok(()) => Done { ok: true, message: format!("restarted on saves/{name} (pid {})", s.pid), print: String::new(), screen: String::new() },
        Err(e) => fail(format!("restarting: {e:#}")),
    }
}

/// The shot, the log lines and what the agent sees.
fn finish(s: &mut Session, line: &str, cmd: &Cmd, mut done: Done) -> i32 {
    let mut shot = None;
    let mut failure_tree = None;
    if !matches!(cmd, Cmd::Quit) {
        if let Ok(mut link) = connect(s) {
            let mut d = Driver { link: &mut link, run: &s.run, step: s.step, headless: s.headless };
            if let Ok(t) = d.tree() {
                done.screen = t["screen"].as_str().unwrap_or("").to_string();
                if !done.ok {
                    let file = format!("{:03}_fail_tree.json", s.step);
                    let _ = std::fs::write(s.run.join(&file), serde_json::to_string_pretty(&t).unwrap_or_default());
                    failure_tree = Some(file);
                }
            }
            let name = if !done.ok { Some("fail".to_string()) } else if s.shots && cmd.changes() { Some(slug(line)) } else { None };
            if matches!(cmd, Cmd::Shot { .. }) && done.ok {
                shot = Some(std::mem::take(&mut done.message));
            } else if let Some(n) = name {
                match d.shot(&n) {
                    Ok(f) => shot = Some(f),
                    Err(e) => done.message = format!("{}{}no shot: {e}", done.message, if done.message.is_empty() { "" } else { "; " }),
                }
            }
        }
    }
    let time = clock();
    let row = log_line(s.step, &time, line, done.ok, &done.message, &done.screen, shot.as_deref());
    if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(s.run.join("log.md")) {
        let _ = writeln!(f, "{row}");
    }
    let entry = json!({"step": s.step, "time": time, "command": line, "ok": done.ok, "message": done.message, "screen": done.screen, "shot": shot, "fail_tree": failure_tree});
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(s.run.join("log.jsonl")) {
        let _ = writeln!(f, "{entry}");
    }
    let _ = s.save();
    println!("§ {:03} {} {}{}{}", s.step, if done.ok { "ok" } else { "FAIL" }, line, if done.message.is_empty() { String::new() } else { format!(": {}", done.message) }, if done.screen.is_empty() { String::new() } else { format!(" [screen {}]", done.screen) });
    if let Some(f) = &shot {
        println!("§ shot {}", s.run.join(f).display());
    }
    if !done.print.is_empty() {
        println!("{}", done.print.trim_end());
    }
    i32::from(!done.ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn the_short_conditions_read_as_the_scenario_files_conditions() {
        let c = |t: &str| cond_of(t).unwrap();
        let p = |v: Value| parse_cond(&v, "t", 0, &mut Vec::new());
        assert_eq!(c("screen=battle"), p(json!({"path": "screen", "eq": "battle"})));
        assert_eq!(c("screen!=vignette"), p(json!({"path": "screen", "ne": "vignette"})));
        assert_eq!(c("widget=menu.new_game"), p(json!({"widget": "menu.new_game"})));
        assert_eq!(c("state.round>2"), p(json!({"path": "state.round", "gt": 2})));
        assert_eq!(c("state.hp<10"), p(json!({"path": "state.hp", "lt": 10})));
        assert_eq!(c("stack~battle"), p(json!({"path": "stack", "has": "battle"})));
        assert_eq!(c("state.beat?"), p(json!({"path": "state.beat", "absent": true})));
        assert_eq!(c("game:state.day=3"), p(json!({"path": "state.day", "eq": 3, "source": "game"})));
        assert_eq!(c("state.outcome=\"\""), p(json!({"path": "state.outcome", "eq": ""})));
        assert_eq!(c("state.log~a=b"), p(json!({"path": "state.log", "has": "a=b"})));
        assert_eq!(c("state.note=x!=y"), p(json!({"path": "state.note", "eq": "x!=y"})));
        assert_eq!(c(r#"{"all": [{"widget": "a"}, {"path": "screen", "eq": "b"}]}"#), p(json!({"all": [{"widget": "a"}, {"path": "screen", "eq": "b"}]})));
        for bad in ["", "?", "=x", "{nope", r#"{"path": "a"}"#] {
            assert!(cond_of(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn every_command_parses_to_its_link_request_and_bad_lines_say_why() {
        let p = |l: &str| parse_line(l).unwrap().unwrap();
        assert_eq!(parse_line("  # a comment"), None);
        assert_eq!(parse_line(""), None);
        assert_eq!(p("click menu.new_game"), Cmd::Act { body: json!({"cmd": "click", "id": "menu.new_game"}), label: "click menu.new_game".into() });
        assert_eq!(p("type Ash and iron"), Cmd::Act { body: json!({"cmd": "type", "text": "Ash and iron"}), label: "type Ash and iron".into() });
        assert_eq!(p("drag stash.1 slot.weapon"), Cmd::Act { body: json!({"cmd": "drag", "id": "stash.1", "to": "slot.weapon"}), label: "drag stash.1 slot.weapon".into() });
        assert_eq!(p("drag a 10,20"), Cmd::Act { body: json!({"cmd": "drag", "id": "a", "to": [10, 20]}), label: "drag a 10,20".into() });
        assert_eq!(p(r#"link {"cmd": "autoplay"}"#), Cmd::Act { body: json!({"cmd": "autoplay"}), label: r#"link {"cmd": "autoplay"}"#.into() });
        assert!(matches!(p("wait screen=battle ms=20000"), Cmd::Wait { ms: 20000, .. }));
        assert!(matches!(p("wait screen=battle"), Cmd::Wait { ms: WAIT_MS_DEFAULT, .. }));
        assert_eq!(p("get game:state.day"), Cmd::Get { path: "state.day".into(), game: true });
        assert_eq!(p("shot"), Cmd::Shot { name: "shot".into() });
        assert_eq!(p("save act1"), Cmd::Save { name: "act1".into() });
        assert_eq!(p("note the forecast said 6, the hit did 4"), Cmd::Note { text: "the forecast said 6, the hit did 4".into() });
        assert!(matches!(p("repeat 40 until screen!=vignette : key space"), Cmd::Repeat { times: 40, .. }));
        assert!(matches!(p("key space 3"), Cmd::Times { times: 3, .. }));
        assert_eq!(p("quit"), Cmd::Quit);
        for (bad, says) in [
            ("dance", "unknown command"),
            ("click", "one widget id"),
            ("wait", "wait <condition>"),
            ("wait screen=x ms=0", "1 to"),
            (r#"link {"cmd": "quit"}"#, "write quit"),
            ("link {}", "needs a cmd"),
            ("shot a/b", "letters"),
            ("save ../x", "letters"),
            ("repeat 3 until screen=a : save x", "other than repeat"),
            ("repeat x until screen=a : back", "1 to"),
            ("key space 0", "1 to"),
        ] {
            let e = parse_line(bad).unwrap().unwrap_err();
            assert!(e.contains(says), "{bad}: {e}");
        }
    }

    #[test]
    fn the_run_folder_is_the_date_then_a_free_number() {
        assert_eq!(date_of(0), (1970, 1, 1));
        assert_eq!(date_of(20_775), (2026, 11, 18));
        assert_eq!(date_of(11_016), (2000, 2, 29));
        let out = std::env::temp_dir().join(format!("aegis_agentplay_dirs_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&out);
        let secs = 20_775 * 86_400 + 5;
        assert_eq!(run_dir(&out, secs), out.join("2026-11-18"));
        std::fs::create_dir_all(out.join("2026-11-18")).unwrap();
        assert_eq!(run_dir(&out, secs), out.join("2026-11-18-2"));
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn a_log_line_is_a_table_row_with_the_shot_linked() {
        assert_eq!(slug("click menu.new_game"), "click_menu_new_game");
        assert_eq!(slug("repeat 40 until screen!=vignette : key space"), "repeat_40_until_screen_vignette_key_spac");
        assert_eq!(log_line(7, "19:40:01", "wait a|b", true, "after 30 ms", "battle", Some("007_wait.png")), "| 007 | 19:40:01 | `wait a\\|b` | ok | after 30 ms | battle | [007_wait.png](007_wait.png) |");
        assert_eq!(log_line(8, "19:40:02", "back", false, "refused", "", None), "| 008 | 19:40:02 | `back` | FAIL | refused |  |  |");
        assert_eq!(log_line(9, "t", "type a`b", true, "x\r\ny", "s|t", None), "| 009 | t | `type a'b` | ok | x  y | s\\|t |  |");
    }

    #[test]
    fn a_session_file_reads_back_as_written() {
        let s = Session { run: "/r".into(), exe: "/e".into(), args: vec!["port=1".into()], port: 1, pid: 2, step: 3, shots: false, headless: true };
        assert_eq!(Session::from_json(&s.to_json()), Some(s));
    }

    /// A game on a link: every reply in turn, and every request kept.
    struct Fake {
        replies: VecDeque<Value>,
        asked: Vec<Value>,
        frame: u64,
        screen: String,
    }

    impl Link for Fake {
        fn ask(&mut self, body: &Value) -> std::result::Result<Value, String> {
            self.asked.push(body.clone());
            match body["cmd"].as_str() {
                Some("ui_tree") => {
                    self.frame += 1;
                    Ok(json!({"ok": true, "frame": self.frame, "screen": self.screen, "stack": [self.screen], "widgets": [{"id": "a.go", "text": "Go", "clickable": true}, {"id": "a.title", "text": "Title", "clickable": false}]}))
                }
                Some("frames") => Ok(json!({"ok": true})),
                Some("screenshot") => Ok(json!({"ok": true})),
                _ => {
                    let r = self.replies.pop_front().unwrap_or(json!({"ok": true}));
                    if let Some(s) = r["then_screen"].as_str() {
                        self.screen = s.to_string();
                    }
                    Ok(r)
                }
            }
        }
    }

    fn fake(screen: &str, replies: Vec<Value>) -> Fake {
        Fake { replies: replies.into(), asked: Vec::new(), frame: 0, screen: screen.into() }
    }

    #[test]
    fn an_action_settles_its_frames_and_a_refusal_fails_with_the_reason() {
        let run = std::env::temp_dir();
        let mut f = fake("menu", vec![json!({"ok": true, "tip": "hi"}), json!({"ok": false, "error": "no widget 'x'"})]);
        let mut d = Driver { link: &mut f, run: &run, step: 1, headless: true };
        let done = d.exec(&parse_cmd("click a.go").unwrap());
        assert!(done.ok, "{done:?}");
        assert_eq!(done.message, "tip=\"hi\"");
        let done = d.exec(&parse_cmd("click x").unwrap());
        assert!(!done.ok);
        assert!(done.message.contains("no widget 'x'"), "{}", done.message);
        let cmds: Vec<&str> = f.asked.iter().filter_map(|b| b["cmd"].as_str()).collect();
        assert_eq!(cmds, ["click", "frames", "ui_tree", "click"]);
    }

    #[test]
    fn a_wait_polls_until_the_condition_holds_or_times_out() {
        let run = std::env::temp_dir();
        let mut f = fake("menu", vec![]);
        let mut d = Driver { link: &mut f, run: &run, step: 1, headless: true };
        assert!(d.exec(&parse_cmd("wait screen=menu").unwrap()).ok);
        let done = d.exec(&parse_cmd("wait screen=battle ms=100").unwrap());
        assert!(!done.ok);
        assert!(done.message.contains("timed out after 100 ms"), "{}", done.message);
        assert!(d.exec(&parse_cmd("assert widget=a.go").unwrap()).ok);
        assert!(!d.exec(&parse_cmd("assert widget=nope").unwrap()).ok);
    }

    #[test]
    fn a_repeat_stops_once_its_condition_holds_and_key_n_presses_n_times() {
        let run = std::env::temp_dir();
        let mut f = fake("vignette", vec![json!({"ok": true}), json!({"ok": true, "then_screen": "battle"})]);
        let mut d = Driver { link: &mut f, run: &run, step: 1, headless: true };
        let done = d.exec(&parse_cmd("repeat 40 until screen!=vignette : key space").unwrap());
        assert!(done.ok, "{done:?}");
        assert_eq!(f.asked.iter().filter(|b| b["cmd"] == "key").count(), 2);
        let mut f = fake("menu", vec![]);
        let mut d = Driver { link: &mut f, run: &run, step: 1, headless: true };
        assert!(d.exec(&parse_cmd("key down 3").unwrap()).ok);
        assert_eq!(f.asked.iter().filter(|b| b["cmd"] == "key").count(), 3);
        let mut f = fake("vignette", vec![]);
        let mut d = Driver { link: &mut f, run: &run, step: 1, headless: true };
        let done = d.exec(&parse_cmd("repeat 2 until screen=battle : key space").unwrap());
        assert!(!done.ok && done.message.contains("2 turns"), "{done:?}");
    }

    #[test]
    fn screen_lists_the_clickable_widgets_first_and_shots_are_numbered() {
        let t = json!({"screen": "menu", "stack": ["title", "menu"], "state": {"slot": 1}, "widgets": [{"id": "t", "text": "Title", "clickable": false}, {"id": "go", "text": "New game", "clickable": true, "enabled": false}]});
        let s = screen_summary(&t);
        assert!(s.starts_with("screen: menu\nstack: title > menu\nstate: {\"slot\":1}\nclickable (1):\n  go  \"New game\"  (disabled)\ntext (1):\n  t  \"Title\"\n"), "{s}");
        let run = std::env::temp_dir();
        let mut f = fake("menu", vec![]);
        let mut d = Driver { link: &mut f, run: &run, step: 12, headless: true };
        assert_eq!(d.shot("hub").unwrap(), "012_hub.png");
        assert_eq!(f.asked.last().unwrap()["path"], json!(run.join("012_hub.png").display().to_string()));
    }
}
