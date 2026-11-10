//! Playtest recordings on the command API (PLAYTEST1 on the engine), game-agnostic. A recording keeps every session
//! (a "battle": the host's history from its starting request, with the checkpoints the game took) of one play run,
//! plus the UI trail and the player's notes, and the content fingerprint of the data it was played on. Because every
//! state change goes through the host, replaying a session's requests on a fresh host gives the same states: the
//! replayer compares each checkpoint, reports the first difference (later checkpoints are unverified), and can take
//! over at any request, run a variant (some requests replaced) or re-run the requests under other seeds.
//!
//! File (`<id>.json`, written atomically through `<id>.json.tmp`):
//!   {format: "playtest", version: 2, id, origin: user | agent, created, engine, source_rev, scenario,
//!    parent: null | {id, battle, at}, content: {algo, total, files}, battles: [{index, requests, checkpoints, end}],
//!    trail: [event], trail_dropped, notes: [note]}
//! Unknown keys are kept. `end` is {result, at} (`abandoned` when the session was left before its end).

use crate::Driver;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use aegis_core::clock::{SystemTime, UNIX_EPOCH};

pub const FORMAT: &str = "playtest";
/// Version 1 was the Godot build's shape (plans and RNG states per round, PLAYTEST1-L); the engine replays command logs.
pub const VERSION: i64 = 2;
pub const TRAIL_MAX: usize = 20000;
pub const ABANDONED: &str = "abandoned";

/// One recorded session: the requests from its starting one, the checkpoints after them, how it ended.
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    pub requests: Vec<Value>,
    pub checkpoints: Vec<Value>,
    pub end: Value,
}

impl Session {
    /// The host's session as it stands (`result` "" while it runs: `abandoned` unless the last checkpoint has one).
    pub fn of(host: &dyn Driver) -> Self {
        let result = host.checkpoints().last().and_then(|c| c["state"]["result"].as_str()).filter(|r| !r.is_empty()).unwrap_or(ABANDONED).to_string();
        Self { requests: host.history().to_vec(), checkpoints: host.checkpoints().to_vec(), end: json!({"result": result, "at": host.history().len()}) }
    }

    pub fn result(&self) -> String {
        self.end["result"].as_str().unwrap_or(ABANDONED).to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Recording {
    pub id: String,
    pub origin: String,
    pub created: String,
    pub engine: String,
    pub source_rev: String,
    pub scenario: String,
    pub parent: Value,
    pub content: Value,
    pub battles: Vec<Session>,
    pub trail: Vec<Value>,
    pub trail_dropped: u64,
    pub notes: Vec<Value>,
    /// Keys this version does not know (kept on save).
    pub extra: Map<String, Value>,
}

const KNOWN: [&str; 14] = ["format", "version", "id", "origin", "created", "engine", "source_rev", "scenario", "parent", "content", "battles", "trail", "trail_dropped", "notes"];

impl Recording {
    pub fn new(id: &str, origin: &str, content: Value) -> Self {
        Self {
            id: id.into(),
            origin: origin.into(),
            created: utc_text(now_secs()),
            engine: format!("aegis {}", env!("CARGO_PKG_VERSION")),
            source_rev: String::new(),
            scenario: String::new(),
            parent: Value::Null,
            content,
            battles: Vec::new(),
            trail: Vec::new(),
            trail_dropped: 0,
            notes: Vec::new(),
            extra: Map::new(),
        }
    }

    /// Adds a trail event (`t` and `type` set by the caller); past TRAIL_MAX only the drop counter grows.
    pub fn push_trail(&mut self, event: Value) {
        if self.trail.len() < TRAIL_MAX {
            self.trail.push(event);
        } else {
            self.trail_dropped += 1;
        }
    }

    pub fn to_json(&self) -> Value {
        let battles: Vec<Value> = self.battles.iter().enumerate().map(|(i, b)| json!({"index": i, "requests": b.requests, "checkpoints": b.checkpoints, "end": b.end})).collect();
        let mut out = json!({
            "format": FORMAT, "version": VERSION, "id": self.id, "origin": self.origin, "created": self.created, "engine": self.engine,
            "source_rev": self.source_rev, "scenario": self.scenario, "parent": self.parent, "content": self.content, "battles": battles,
            "trail": self.trail, "trail_dropped": self.trail_dropped, "notes": self.notes,
        });
        if let Some(o) = out.as_object_mut() {
            for (k, v) in &self.extra {
                o.insert(k.clone(), v.clone());
            }
        }
        out
    }

    /// A recording from its file shape; refuses another format, a newer or older version and broken sessions.
    pub fn from_json(v: &Value) -> Result<Self, String> {
        if v["format"] != json!(FORMAT) {
            return Err("not a playtest recording (format must be \"playtest\")".into());
        }
        match v["version"].as_i64() {
            Some(VERSION) => {}
            Some(n) if n > VERSION => return Err(format!("recording version {n} is newer than this engine reads ({VERSION})")),
            Some(1) => return Err("version 1 is a Godot build recording (plans and RNG states); the engine replays its own recordings (version 2)".into()),
            _ => return Err(format!("recording version must be {VERSION}")),
        }
        let s = |k: &str, d: &str| v[k].as_str().unwrap_or(d).to_string();
        let mut battles = Vec::new();
        for (i, b) in v["battles"].as_array().map(|a| a.as_slice()).unwrap_or(&[]).iter().enumerate() {
            let requests = b["requests"].as_array().cloned().ok_or_else(|| format!("battle {i}: requests must be a list"))?;
            if requests.iter().any(|r| !r.is_object() || !r["cmd"].is_string()) {
                return Err(format!("battle {i}: every request is an object with a cmd"));
            }
            let checkpoints = b["checkpoints"].as_array().cloned().unwrap_or_default();
            if checkpoints.iter().any(|c| c["at"].as_u64().is_none_or(|at| at as usize > requests.len())) {
                return Err(format!("battle {i}: a checkpoint's at is past its requests"));
            }
            let end = if b["end"].is_object() { b["end"].clone() } else { json!({"result": ABANDONED, "at": requests.len()}) };
            battles.push(Session { requests, checkpoints, end });
        }
        let extra = v.as_object().map(|o| o.iter().filter(|(k, _)| !KNOWN.contains(&k.as_str())).map(|(k, v)| (k.clone(), v.clone())).collect()).unwrap_or_default();
        Ok(Self {
            id: s("id", ""),
            origin: s("origin", "user"),
            created: s("created", ""),
            engine: s("engine", ""),
            source_rev: s("source_rev", ""),
            scenario: s("scenario", ""),
            parent: v["parent"].clone(),
            content: v["content"].clone(),
            battles,
            trail: v["trail"].as_array().cloned().unwrap_or_default(),
            trail_dropped: v["trail_dropped"].as_u64().unwrap_or(0),
            notes: v["notes"].as_array().cloned().unwrap_or_default(),
            extra,
        })
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        Self::read(path).map_err(|u| u.detail)
    }

    /// `load`, with the failure split into what a screen shows and the full cause.
    pub fn read(path: &Path) -> Result<Self, Unreadable> {
        let text = std::fs::read_to_string(path).map_err(|e| Unreadable { reason: "The file cannot be read.".into(), detail: format!("{}: {e}", path.display()) })?;
        let v: Value = serde_json::from_str(&text).map_err(|e| Unreadable { reason: format!("Not valid JSON: {e}"), detail: format!("{}: {e}", path.display()) })?;
        Self::from_json(&v).map_err(|e| Unreadable { reason: e.clone(), detail: e })
    }

    /// Writes `<dir>/<id>.json` through a temporary file (an interrupted save never leaves half a recording).
    pub fn save(&self, dir: &Path) -> Result<PathBuf, String> {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = dir.join(format!("{}.json", self.id));
        let tmp = dir.join(format!("{}.json.tmp", self.id));
        let text = serde_json::to_string_pretty(&self.to_json()).map_err(|e| e.to_string())?;
        std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(path)
    }
}

// ---- ids and the folder ----------------------------------------------------------------------------------------

pub fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// (year, month, day, hour, minute, second) in UTC of a Unix time.
fn civil(secs: i64) -> (i64, i64, i64, i64, i64, i64) {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    (year, month, day, rem / 3600, rem % 3600 / 60, rem % 60)
}

/// "2026-10-20T14:03:09Z".
pub fn utc_text(secs: i64) -> String {
    let (y, mo, d, h, mi, s) = civil(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// A new recording id in `dir`: "YYYYMMDD-HHMMSS" (UTC), then "-2", "-3" ... while one exists.
pub fn new_id(dir: &Path, secs: i64) -> String {
    let (y, mo, d, h, mi, s) = civil(secs);
    let base = format!("{y:04}{mo:02}{d:02}-{h:02}{mi:02}{s:02}");
    let mut id = base.clone();
    let mut n = 2;
    while dir.join(format!("{id}.json")).exists() {
        id = format!("{base}-{n}");
        n += 1;
    }
    id
}

/// Why a recording did not load.
#[derive(Clone, Debug, PartialEq)]
pub struct Unreadable {
    /// What a screen shows: a function of the file's bytes alone, with no path and no operating system wording, so that
    /// a drawn frame is the same on every platform (the list beside it already names the file).
    pub reason: String,
    /// The full cause for a person or an agent, in the operating system's own words where it has any: `load`'s error.
    pub detail: String,
}

/// The recordings in `dir` (a flat `<id>.json`, or a shared folder `<id>/<id>.json`), newest first (by id), with what
/// failed to load.
pub fn list(dir: &Path) -> Vec<(PathBuf, Result<Recording, Unreadable>)> {
    let entry = |p: PathBuf| -> Option<PathBuf> {
        if p.is_dir() {
            let inner = p.join(format!("{}.json", p.file_name()?.to_string_lossy()));
            inner.is_file().then_some(inner)
        } else {
            p.extension().is_some_and(|x| x == "json").then_some(p)
        }
    };
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir).map(|r| r.flatten().filter_map(|e| entry(e.path())).collect()).unwrap_or_default();
    paths.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    paths.into_iter().map(|p| {
        let r = Recording::read(&p);
        (p, r)
    }).collect()
}

/// Where the note images of the recording at `path` are: `<dir>/<id>/` beside a flat `<dir>/<id>.json`, the folder
/// itself for a shared `<id>/<id>.json`.
pub fn image_dir(path: &Path) -> PathBuf {
    let id = path.file_stem().map(|s| s.to_os_string()).unwrap_or_default();
    let parent = path.parent().unwrap_or(Path::new(""));
    if parent.file_name() == Some(id.as_os_str()) {
        parent.to_path_buf()
    } else {
        parent.join(id)
    }
}

/// Copies the recording at `path` and its note images into `<dest_root>/<id>/` (the repo inbox a run from source can
/// write: the user pushes it and names the id); returns that folder.
pub fn copy_to(path: &Path, dest_root: &Path) -> Result<PathBuf, String> {
    let id = path.file_stem().ok_or_else(|| format!("{}: not a recording file", path.display()))?;
    let dest = dest_root.join(id);
    std::fs::create_dir_all(&dest).map_err(|e| format!("Cannot create {}: {e}", dest.display()))?;
    std::fs::copy(path, dest.join(path.file_name().unwrap_or(id))).map_err(|e| format!("Cannot copy {}: {e}", path.display()))?;
    let images = image_dir(path);
    if images != dest {
        for f in std::fs::read_dir(&images).map(|r| r.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect::<Vec<_>>()).unwrap_or_default() {
            if let Some(name) = f.file_name() {
                std::fs::copy(&f, dest.join(name)).map_err(|e| format!("Cannot copy {}: {e}", f.display()))?;
            }
        }
    }
    Ok(dest)
}

// ---- comparing states --------------------------------------------------------------------------------------------

/// Every difference between two JSON values: (path, left, right), left's key order first, then keys only right has.
pub fn diff(a: &Value, b: &Value) -> Vec<(String, Value, Value)> {
    let mut out = Vec::new();
    diff_at("", a, b, &mut out);
    out
}

fn diff_at(path: &str, a: &Value, b: &Value, out: &mut Vec<(String, Value, Value)>) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for (k, va) in x {
                diff_at(&join(path, k), va, y.get(k).unwrap_or(&Value::Null), out);
            }
            for (k, vb) in y.iter().filter(|(k, _)| !x.contains_key(*k)) {
                diff_at(&join(path, k), &Value::Null, vb, out);
            }
        }
        (Value::Array(x), Value::Array(y)) if x.len() == y.len() => {
            for (i, (va, vb)) in x.iter().zip(y).enumerate() {
                diff_at(&join(path, &i.to_string()), va, vb, out);
            }
        }
        _ if a != b => out.push((if path.is_empty() { ".".into() } else { path.to_string() }, a.clone(), b.clone())),
        _ => {}
    }
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

// ---- replay ------------------------------------------------------------------------------------------------------

/// What became of one recorded checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Match,
    /// The first difference: path in the state, recorded value, replayed value.
    Diverge { path: String, recorded: Value, replayed: Value },
    /// After a divergence, a broken request or the end of the run.
    Unverified,
}

#[derive(Debug, Clone)]
pub struct Check {
    /// The number of requests the checkpoint follows.
    pub at: usize,
    pub verdict: Verdict,
}

#[derive(Debug, Clone, Default)]
pub struct Report {
    pub checks: Vec<Check>,
    /// The first request that failed: (index, error).
    pub broken: Option<(usize, String)>,
    /// Requests sent.
    pub sent: usize,
}

impl Report {
    pub fn matched(&self) -> bool {
        self.broken.is_none() && self.checks.iter().all(|c| c.verdict == Verdict::Match)
    }
}

/// Replays `session` on `host` (fresh), comparing each checkpoint the run takes with the recorded one at the same
/// position; `until` stops after that many requests (a take-over: the host is left there, live). A failed request
/// stops the run.
pub fn replay(host: &mut dyn Driver, session: &Session, until: Option<usize>) -> Report {
    let mut report = Report::default();
    let stop = until.unwrap_or(session.requests.len()).min(session.requests.len());
    let mut diverged = false;
    for (i, req) in session.requests[..stop].iter().enumerate() {
        let reply = host.send(req);
        report.sent = i + 1;
        if reply["ok"] != json!(true) {
            report.broken = Some((i, reply["error"].as_str().unwrap_or("refused").to_string()));
            break;
        }
    }
    for (k, recorded) in session.checkpoints.iter().enumerate() {
        let at = recorded["at"].as_u64().unwrap_or(0) as usize;
        if at > stop {
            break;
        }
        let verdict = match host.checkpoints().get(k) {
            _ if diverged => Verdict::Unverified,
            None => Verdict::Unverified,
            Some(got) if got["at"] != recorded["at"] => {
                diverged = true;
                Verdict::Diverge { path: "at".into(), recorded: recorded["at"].clone(), replayed: got["at"].clone() }
            }
            Some(got) => match diff(&recorded["state"], &got["state"]).into_iter().next() {
                None => Verdict::Match,
                Some((path, a, b)) => {
                    diverged = true;
                    Verdict::Diverge { path, recorded: a, replayed: b }
                }
            },
        };
        report.checks.push(Check { at, verdict });
    }
    report
}

/// A variant: `remove` recorded requests from index `at` replaced by `requests`, then the recorded requests after the
/// replaced ones: to the end (`then_recorded`), or only until the first checkpoint after `at` (`stop`).
#[derive(Debug, Clone)]
pub struct Variant {
    pub at: usize,
    pub remove: usize,
    pub requests: Vec<Value>,
    pub then_recorded: bool,
}

impl Variant {
    /// {"at", "remove"?, "requests": [...], "then"?: "stop" | "recorded"}.
    pub fn from_json(v: &Value) -> Result<Self, String> {
        let at = v["at"].as_u64().ok_or("a variant needs at (the index of the first request it replaces)")? as usize;
        let requests = v["requests"].as_array().cloned().ok_or("a variant needs requests (a list)")?;
        let then_recorded = match v["then"].as_str().unwrap_or("stop") {
            "stop" => false,
            "recorded" => true,
            other => return Err(format!("then must be stop or recorded, not '{other}'")),
        };
        Ok(Self { at, remove: v["remove"].as_u64().unwrap_or(0) as usize, requests, then_recorded })
    }
}

pub struct VariantRun {
    /// The variant's own session (save it as a recording with origin agent).
    pub session: Session,
    /// "DIFF <at> <path> <recorded> -> <variant>" for every difference of every checkpoint both runs took.
    pub diffs: Vec<String>,
    pub broken: Option<(usize, String)>,
}

/// Runs the variant on `host` (fresh) and compares its checkpoints with the recorded ones by position.
pub fn run_variant(host: &mut dyn Driver, session: &Session, variant: &Variant) -> VariantRun {
    let at = variant.at.min(session.requests.len());
    let rest_from = (at + variant.remove).min(session.requests.len());
    let mut plan: Vec<Value> = session.requests[..at].to_vec();
    plan.extend(variant.requests.iter().cloned());
    let patched_to = plan.len();
    plan.extend(session.requests[rest_from..].iter().cloned());
    let mut broken = None;
    for (i, req) in plan.iter().enumerate() {
        // stop: past the patch, the run ends once a checkpoint was taken after the first replaced request.
        if !variant.then_recorded && i >= patched_to && host.checkpoints().iter().any(|c| c["at"].as_u64().unwrap_or(0) as usize > at) {
            break;
        }
        let reply = host.send(req);
        if reply["ok"] != json!(true) {
            broken = Some((i, reply["error"].as_str().unwrap_or("refused").to_string()));
            break;
        }
    }
    let mut diffs = Vec::new();
    for (recorded, got) in session.checkpoints.iter().zip(host.checkpoints()) {
        for (path, a, b) in diff(&recorded["state"], &got["state"]) {
            diffs.push(format!("DIFF {} {path} {a} -> {b}", got["at"]));
        }
    }
    VariantRun { session: Session::of(host), diffs, broken }
}

/// Re-runs the recorded requests with the starting request's `seed_key` set to `seed`, without comparing. A failed
/// request ends the run as "broken@<index>" unless `fallback` is given: then the fallback request is sent instead and
/// the run ends there. Returns the result of the last checkpoint (`abandoned` when it has none, as a recorded session that never ended).
pub fn sweep_one(host: &mut dyn Driver, session: &Session, seed_key: &str, seed: i64, fallback: Option<&Value>) -> String {
    for (i, req) in session.requests.iter().enumerate() {
        let mut req = req.clone();
        if i == 0 {
            req[seed_key] = json!(seed);
        }
        if host.send(&req)["ok"] != json!(true) {
            match fallback {
                Some(f) => {
                    if host.send(f)["ok"] != json!(true) {
                        return format!("broken@{i}");
                    }
                    break;
                }
                None => return format!("broken@{i}"),
            }
        }
    }
    host.checkpoints().last().and_then(|c| c["state"]["result"].as_str()).filter(|r| !r.is_empty()).unwrap_or(ABANDONED).to_string()
}

/// Seeds from "a..b" (inclusive) or "a,b,c"; at most `max`.
pub fn parse_seeds(text: &str, max: usize) -> Result<Vec<i64>, String> {
    let seeds: Vec<i64> = match text.split_once("..") {
        Some((a, b)) => {
            let (a, b) = (a.trim().parse::<i64>().map_err(|e| e.to_string())?, b.trim().parse::<i64>().map_err(|e| e.to_string())?);
            if b < a {
                return Err("seeds a..b needs a <= b".into());
            }
            (a..=b).take(max + 1).collect()
        }
        None => text.split(',').map(|s| s.trim().parse::<i64>().map_err(|e| format!("seed '{s}': {e}"))).collect::<Result<_, _>>()?,
    };
    if seeds.len() > max {
        return Err(format!("at most {max} seeds per call"));
    }
    Ok(seeds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Commands, Host};

    /// A counter game: `add {n}` adds, a checkpoint every time the total passes a multiple of 10, over at 30.
    struct Counter {
        total: i64,
        seed: i64,
        marks: i64,
    }

    impl Commands for Counter {
        fn mutating(&self, cmd: &str) -> bool {
            matches!(cmd, "new" | "add")
        }
        fn restarts(&self, cmd: &str) -> bool {
            cmd == "new"
        }
        fn handle(&mut self, cmd: &str, req: &Value) -> Value {
            match cmd {
                "new" => {
                    self.seed = req["seed"].as_i64().unwrap_or(1);
                    self.total = 0;
                    self.marks = 0;
                    json!({"ok": true})
                }
                "add" if self.total >= 30 => json!({"ok": false, "error": "over"}),
                "add" => {
                    self.total += req["n"].as_i64().unwrap_or(0) * self.seed;
                    json!({"ok": true})
                }
                _ => json!({"ok": false, "error": "unknown"}),
            }
        }
        fn checkpoint(&mut self) -> Option<Value> {
            if self.total / 10 > self.marks {
                self.marks = self.total / 10;
                let result = if self.total >= 30 { "won" } else { "" };
                return Some(json!({"total": self.total, "result": result}));
            }
            None
        }
    }

    fn host() -> Host<Counter> {
        Host::new(Counter { total: 0, seed: 1, marks: 0 })
    }

    fn played() -> Session {
        let mut h = host();
        for r in [json!({"cmd": "new", "seed": 1}), json!({"cmd": "add", "n": 6}), json!({"cmd": "add", "n": 6}), json!({"cmd": "add", "n": 9}), json!({"cmd": "add", "n": 12})] {
            assert_eq!(h.handle(&r)["ok"], json!(true));
        }
        Session::of(&h)
    }

    #[test]
    fn a_recording_replays_and_survives_save_and_load() {
        let s = played();
        assert_eq!(s.checkpoints.len(), 3);
        assert_eq!(s.result(), "won");
        assert!(replay(&mut host(), &s, None).matched());
        let mut rec = Recording::new("20261020-120000", "user", json!({"algo": "sha256-12", "total": "x", "files": {}}));
        rec.battles.push(s.clone());
        rec.extra.insert("future_key".into(), json!(7));
        let dir = aegis_core::scratch::Scratch::vacant("playtest");
        let path = rec.save(&dir).unwrap();
        let back = Recording::load(&path).unwrap();
        assert_eq!(back.battles[0], s);
        assert_eq!(back.extra["future_key"], json!(7));
        assert_eq!(new_id(&dir, 0), "19700101-000000");
        // Copy to repo: the recording and its images land in <inbox>/<id>/, which lists like the flat file.
        std::fs::create_dir_all(image_dir(&path)).unwrap();
        std::fs::write(image_dir(&path).join("note_1.jpg"), b"jpg").unwrap();
        let inbox = dir.join("inbox");
        let dest = copy_to(&path, &inbox).unwrap();
        assert_eq!(dest, inbox.join("20261020-120000"));
        assert!(dest.join("20261020-120000.json").is_file() && dest.join("note_1.jpg").is_file());
        let shared = list(&inbox);
        assert_eq!(shared.len(), 1);
        assert!(shared[0].1.is_ok());
        assert_eq!(image_dir(&shared[0].0), dest, "a shared recording keeps its images beside it");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_changed_checkpoint_diverges_there_and_later_ones_are_unverified() {
        let mut s = played();
        s.checkpoints[1]["state"]["total"] = json!(99);
        let r = replay(&mut host(), &s, None);
        assert_eq!(r.checks[0].verdict, Verdict::Match);
        assert_eq!(r.checks[1].verdict, Verdict::Diverge { path: "total".into(), recorded: json!(99), replayed: json!(21) });
        assert_eq!(r.checks[2].verdict, Verdict::Unverified);
        assert!(!r.matched());
    }

    #[test]
    fn take_over_stops_at_a_request_and_leaves_the_host_live() {
        let s = played();
        let mut h = host();
        let r = replay(&mut h, &s, Some(3));
        assert!(r.matched() && r.sent == 3 && r.checks.len() == 1);
        assert_eq!(h.total, 12);
        assert_eq!(h.handle(&json!({"cmd": "add", "n": 1}))["ok"], json!(true));
    }

    #[test]
    fn variants_diff_against_the_recording() {
        let s = played();
        let same = Variant { at: 2, remove: 1, requests: vec![json!({"cmd": "add", "n": 6})], then_recorded: true };
        let run = run_variant(&mut host(), &s, &same);
        assert!(run.diffs.is_empty() && run.broken.is_none());
        let other = Variant { at: 2, remove: 1, requests: vec![json!({"cmd": "add", "n": 8})], then_recorded: true };
        let run = run_variant(&mut host(), &s, &other);
        assert!(run.diffs.iter().any(|d| d.starts_with("DIFF") && d.contains("total")), "{:?}", run.diffs);
        assert_eq!(run.session.requests[2], json!({"cmd": "add", "n": 8}));
        // stop: a patch with no checkpoint of its own runs on until the next one, then ends there.
        let stop = Variant { at: 2, remove: 1, requests: vec![json!({"cmd": "add", "n": 1})], then_recorded: false };
        let run = run_variant(&mut host(), &s, &stop);
        assert_eq!(run.session.requests.len(), 4, "the prefix, the patch, then up to the checkpoint: {:?}", run.session.requests);
        assert!(run.diffs.iter().any(|d| d.contains("total")), "{:?}", run.diffs);
    }

    #[test]
    fn a_seed_sweep_counts_results_and_falls_back() {
        let s = played();
        assert_eq!(sweep_one(&mut host(), &s, "seed", 1, None), "won");
        assert_eq!(sweep_one(&mut host(), &s, "seed", 5, None), "broken@2");
        assert_eq!(sweep_one(&mut host(), &s, "seed", 5, Some(&json!({"cmd": "add", "n": 0}))), "broken@2", "a refused fallback breaks too");
        assert_eq!(sweep_one(&mut host(), &s, "seed", 5, Some(&json!({"cmd": "new", "seed": 1}))), ABANDONED, "an accepted fallback ends the run there");
        assert_eq!(parse_seeds("3..5", 200).unwrap(), vec![3, 4, 5]);
        assert_eq!(parse_seeds("7, 9", 200).unwrap(), vec![7, 9]);
        assert!(parse_seeds("1..500", 200).is_err());
    }

    #[test]
    fn validation_refuses_other_formats_and_versions() {
        assert!(Recording::from_json(&json!({"format": "x", "version": 1})).is_err());
        assert!(Recording::from_json(&json!({"format": "playtest", "version": 3})).unwrap_err().contains("newer"));
        assert!(Recording::from_json(&json!({"format": "playtest", "version": 1})).unwrap_err().contains("Godot"));
        let bad = json!({"format": "playtest", "version": 2, "battles": [{"requests": [{"cmd": "new"}], "checkpoints": [{"at": 5}]}]});
        assert!(Recording::from_json(&bad).unwrap_err().contains("past its requests"));
        let ok = Recording::from_json(&json!({"format": "playtest", "version": 2})).unwrap();
        assert!(ok.battles.is_empty() && ok.origin == "user");
        assert_eq!(utc_text(1_792_500_000), "2026-10-20T12:40:00Z");
    }
}
