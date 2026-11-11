//! `aegis scenario` (INTEG1-E1, docs/specs/AGENTROLES1.md 4; authoring: engine/docs/authoring/scenarios.md): JSON files walk a
//! game's real screens over the agent link and assert state, with no window by default (HEADLESS1), or in a real muted
//! window that does not take the focus when the file or `window=1` asks.
//!   aegis scenario check [<file> | all]                                       parse the files; no game starts
//!   aegis scenario run <id | all | file.json> [size=WxH] [out=<dir>] [slow=N] [exe=<game exe>] [data=<data dir>] [window=1] [fps=N] [dpr=X]
//!   aegis scenario shots <name> [screen=] [scene=] [rules=] [seed=] [do=<a>,<b>,...] [the same keys as run]
//! A scenario is named by where it lies: `<game folder>/scenarios/<id>.json`. The game folder is the one holding the game
//! manifest the registered game names, and the game's program is the file named as the manifest's id beside this one
//! (`exe=` says otherwise). Exit codes: 0 pass, 1 a step or a file failed, 2 could not start. Nothing here runs in the gate
//! but `check` and the tests on a fake link (`tests/scenario.rs`).

use crate::Args;
use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The largest scenario file read.
pub const FILE_MAX: u64 = 1 << 20;
/// Steps a scenario may run, every `repeat` and `times` counted out.
pub const STEPS_MAX: u64 = 500;
pub const REPEAT_DEPTH_MAX: usize = 4;
pub const SIZE_MIN: (u32, u32) = (1280, 720);
pub const SIZE_MAX: (u32, u32) = (7680, 4320);
/// The longest a single `wait` may ask for, and the longest run.
pub const WAIT_MS_MAX: u64 = 600_000;
pub const TIMEOUT_S_MAX: u64 = 3600;
pub const SLOW_MAX: u32 = 20;
pub const WAIT_MS_DEFAULT: u64 = 10_000;
/// Frames a driver waits for after an action: the link replies before the frame is drawn, so the second one is whole.
pub const SETTLE_FRAMES: u64 = 2;
/// How long the frames may take to come (times `slow`) before the window counts as stalled.
pub const SETTLE_MS: u64 = 10_000;
/// How long a new window may take to answer on the link, and to reach the asked size (a capped size never does).
pub const START_MS: u64 = 120_000;
pub const SIZED_MS: u64 = 1_500;
/// Ports tried: the OS picks a free one, and another program may take it before the game binds it.
pub const PORT_TRIES: u32 = 3;
/// The shell's smallest window: less than this is a minimized one, which draws on a canvas of nothing.
const WINDOW_MIN: (i64, i64) = (640, 360);
const POLL: Duration = Duration::from_millis(5);
/// Between two looks at a game with no window: a look costs it a reply, and frames come at its low rate anyway.
const POLL_HEADLESS: Duration = Duration::from_millis(30);
/// A clip's bounds, as the game takes them (`aegis_ui::shell::headless`): a file that passes the check also runs.
const CLIP_FRAMES_MAX: u64 = 120;
const CLIP_EVERY_MS: (u64, u64) = (10, 2000);
const CLIP_MS_MAX: u64 = 15_000;
/// A long `pause` still asks the game now and then: with no window a game leaves when nothing has asked for a while.
pub const PAUSE_LOOK: Duration = Duration::from_secs(1);
/// How long one link reply may take (times `slow`), and on the way out after a failure.
const REPLY: Duration = Duration::from_secs(30);
const REPLY_LEAVING: Duration = Duration::from_secs(5);
/// How long a game may take to leave after `quit` before it is killed.
pub const LEAVE: Duration = Duration::from_secs(5);
/// How long a dead link waits for its process to count as ended.
const GONE_WAIT: Duration = Duration::from_secs(2);
/// The end of a game's stderr a bundle keeps.
const STDERR_MAX: u64 = 1 << 20;

const AUDIT_KINDS: [&str; 4] = ["cut", "edge", "overlap", "spill"];

/// Settings a scenario may not change: they keep the run off the user's desktop and its sizes in `sizes`.
const HELD_SETTINGS: [&str; 5] = ["start_inactive", "window_size", "fullscreen", "borderless", "window_mode"];

// ---- the file ----

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Active,
    Waiting(String),
}

/// How the game starts (`launch` in the file): `scene` is the game's id of what the screen opens on, `rules` the
/// game's id of the rules it plays by (only with `screen` arena); both reach the game as the launch keys `scene=` and
/// `rules=`.
#[derive(Debug, Clone, PartialEq)]
pub struct Launch {
    pub screen: String,
    pub scene: Option<String>,
    pub seed: Option<u64>,
    pub rules: Option<String>,
}

/// LAYER-ENG-KEYS (PR5-29a): the launch keys' genre-free names and the old names they had, which still load (in a file
/// and as `shots` keys); a launch giving both names of one key is refused.
pub const LAUNCH_ALIASES: [(&str, &str); 2] = [("scene", "encounter"), ("rules", "ruleset")];

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Eq(Value),
    Ne(Value),
    Gt(f64),
    Lt(f64),
    Has(Value),
    Absent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Source {
    Tree,
    Game,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cond {
    Path { source: Source, path: String, op: Op },
    Widget(String),
    All(Vec<Cond>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Click { id: String },
    Hover { id: String },
    Key { key: String, times: u64 },
    Type { text: String },
    Drag { id: String, to: Value },
    Back,
    Link { body: Value },
    Wait { until: Cond, timeout_ms: u64 },
    Assert { until: Cond },
    Shot { name: String, max_w: u64 },
    Clip { name: String, frames: u64, every_ms: u64, max_w: u64 },
    Pause { ms: u64 },
    Audit { scales: Vec<f64>, fail: Vec<String> },
    Repeat { times: u64, steps: Vec<Step>, until: Option<Cond> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub launch: Launch,
    pub sizes: Vec<(u32, u32)>,
    pub settings: Map<String, Value>,
    pub timeout: Duration,
    /// It needs a real window (window behaviour, real input, final performance numbers); else it runs with none.
    pub window: bool,
    pub steps: Vec<Step>,
}

/// "WxH" within the bounds, or why not.
pub fn parse_size(text: &str) -> std::result::Result<(u32, u32), String> {
    let parsed = text.split_once('x').and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)));
    let Some((w, h)) = parsed else { return Err(format!("size '{text}' is not WxH")) };
    if w < SIZE_MIN.0 || h < SIZE_MIN.1 {
        return Err(format!("size '{text}' is under {}x{}", SIZE_MIN.0, SIZE_MIN.1));
    }
    if w > SIZE_MAX.0 || h > SIZE_MAX.1 {
        return Err(format!("size '{text}' is over {}x{}", SIZE_MAX.0, SIZE_MAX.1));
    }
    Ok((w, h))
}

/// A name that becomes a file: letters, digits, `_` and `-` only.
fn plain_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 80 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn unknown_keys(v: &Map<String, Value>, known: &[&str], place: &str, problems: &mut Vec<String>) {
    for k in v.keys().filter(|k| !known.contains(&k.as_str())) {
        problems.push(format!("{place}: unknown key '{k}'"));
    }
}

/// A launch text under `key` or its old name (`LAUNCH_ALIASES`); both at once is a problem.
fn aliased_text(l: &Map<String, Value>, key: &str, problems: &mut Vec<String>) -> Option<String> {
    let old = LAUNCH_ALIASES.iter().find(|(k, _)| *k == key).map_or(key, |(_, o)| *o);
    match (l.contains_key(key), l.contains_key(old)) {
        (true, true) => {
            problems.push(format!("launch: '{key}' and its old name '{old}' are both given"));
            None
        }
        (true, false) => Some(text_of(l, key, "launch", problems)),
        (false, true) => Some(text_of(l, old, "launch", problems)),
        (false, false) => None,
    }
}

fn text_of(v: &Map<String, Value>, key: &str, place: &str, problems: &mut Vec<String>) -> String {
    match v.get(key).and_then(Value::as_str) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            problems.push(format!("{place}: needs '{key}', a text"));
            String::new()
        }
    }
}

/// `key`, a whole number in `range`, or `default` when absent and allowed.
fn int_of(v: &Map<String, Value>, key: &str, default: Option<u64>, range: (u64, u64), place: &str, problems: &mut Vec<String>) -> u64 {
    match (v.get(key), default) {
        (None, Some(d)) => d,
        (None, None) => {
            problems.push(format!("{place}: needs '{key}', a whole number"));
            range.0
        }
        (Some(x), _) => match x.as_u64().filter(|n| *n >= range.0 && *n <= range.1) {
            Some(n) => n,
            None => {
                problems.push(format!("{place}: '{key}' must be a whole number from {} to {}", range.0, range.1));
                range.0
            }
        },
    }
}

/// A condition: {path, source?, one op}, {widget} or {all}.
pub fn parse_cond(v: &Value, place: &str, depth: usize, problems: &mut Vec<String>) -> Cond {
    let bad = Cond::All(Vec::new());
    let Some(o) = v.as_object() else {
        problems.push(format!("{place}: a condition is an object"));
        return bad;
    };
    if depth > REPEAT_DEPTH_MAX {
        problems.push(format!("{place}: conditions nest at most {REPEAT_DEPTH_MAX} deep"));
        return bad;
    }
    if let Some(all) = o.get("all") {
        unknown_keys(o, &["all"], place, problems);
        return match all.as_array().filter(|a| !a.is_empty()) {
            Some(list) => Cond::All(list.iter().enumerate().map(|(i, c)| parse_cond(c, &format!("{place}.all[{i}]"), depth + 1, problems)).collect()),
            None => {
                problems.push(format!("{place}: 'all' is a list of conditions, not empty"));
                bad
            }
        };
    }
    if o.contains_key("widget") {
        unknown_keys(o, &["widget"], place, problems);
        return Cond::Widget(text_of(o, "widget", place, problems));
    }
    if !o.contains_key("path") {
        problems.push(format!("{place}: a condition has 'path', 'widget' or 'all'"));
        return bad;
    }
    unknown_keys(o, &["path", "source", "eq", "ne", "gt", "lt", "has", "absent"], place, problems);
    let path = text_of(o, "path", place, problems);
    let source = match o.get("source").map(|s| s.as_str()) {
        None | Some(Some("ui_tree")) => Source::Tree,
        Some(Some("game")) => Source::Game,
        Some(_) => {
            problems.push(format!("{place}: 'source' is 'game' or 'ui_tree'"));
            Source::Tree
        }
    };
    let ops: Vec<&str> = ["eq", "ne", "gt", "lt", "has", "absent"].into_iter().filter(|k| o.contains_key(*k)).collect();
    if ops.len() != 1 {
        problems.push(format!("{place}: a path condition has one of eq, ne, gt, lt, has, absent"));
        return bad;
    }
    let number = |k: &str, problems: &mut Vec<String>| {
        o[k].as_f64().unwrap_or_else(|| {
            problems.push(format!("{place}: '{k}' needs a number"));
            0.0
        })
    };
    let op = match ops[0] {
        "eq" => Op::Eq(o["eq"].clone()),
        "ne" => Op::Ne(o["ne"].clone()),
        "gt" => Op::Gt(number("gt", problems)),
        "lt" => Op::Lt(number("lt", problems)),
        "has" => Op::Has(o["has"].clone()),
        _ => {
            if o["absent"] != json!(true) {
                problems.push(format!("{place}: 'absent' is written as true"));
            }
            Op::Absent
        }
    };
    Cond::Path { source, path, op }
}

fn parse_step(v: &Value, place: &str, depth: usize, problems: &mut Vec<String>) -> Step {
    let bad = Step::Back;
    let Some(o) = v.as_object() else {
        problems.push(format!("{place}: a step is an object"));
        return bad;
    };
    let Some(kind) = o.get("step").and_then(Value::as_str) else {
        problems.push(format!("{place}: a step names itself in 'step'"));
        return bad;
    };
    let keys = |known: &[&str], problems: &mut Vec<String>| {
        let mut all = vec!["step"];
        all.extend_from_slice(known);
        unknown_keys(o, &all, place, problems);
    };
    let cond = |key: &str, problems: &mut Vec<String>| match o.get(key) {
        Some(c) => parse_cond(c, &format!("{place}.{key}"), 0, problems),
        None => {
            problems.push(format!("{place}: needs '{key}', a condition"));
            Cond::All(Vec::new())
        }
    };
    match kind {
        "click" => {
            keys(&["id"], problems);
            Step::Click { id: text_of(o, "id", place, problems) }
        }
        "hover" => {
            keys(&["id"], problems);
            Step::Hover { id: text_of(o, "id", place, problems) }
        }
        "key" => {
            keys(&["key", "times"], problems);
            Step::Key { key: text_of(o, "key", place, problems), times: int_of(o, "times", Some(1), (1, STEPS_MAX), place, problems) }
        }
        "type" => {
            keys(&["text"], problems);
            match o.get("text").and_then(Value::as_str) {
                Some(t) => Step::Type { text: t.to_string() },
                None => {
                    problems.push(format!("{place}: needs 'text', a text"));
                    bad
                }
            }
        }
        "drag" => {
            keys(&["id", "to"], problems);
            let to = o.get("to").cloned().unwrap_or(Value::Null);
            let point = to.as_array().is_some_and(|a| a.len() == 2 && a.iter().all(Value::is_i64));
            if !point && !to.as_str().is_some_and(|s| !s.is_empty()) {
                problems.push(format!("{place}: 'to' is a widget id or [x, y]"));
            }
            Step::Drag { id: text_of(o, "id", place, problems), to }
        }
        "back" => {
            keys(&[], problems);
            Step::Back
        }
        "link" => {
            keys(&["body"], problems);
            let body = o.get("body").cloned().unwrap_or(Value::Null);
            match body["cmd"].as_str() {
                Some("quit") => problems.push(format!("{place}: the driver alone stops the game ('quit')")),
                Some(c) if !c.is_empty() => {}
                _ => problems.push(format!("{place}: 'body' is a request, an object with 'cmd'")),
            }
            Step::Link { body }
        }
        "wait" => {
            keys(&["until", "timeout_ms"], problems);
            Step::Wait { until: cond("until", problems), timeout_ms: int_of(o, "timeout_ms", Some(WAIT_MS_DEFAULT), (1, WAIT_MS_MAX), place, problems) }
        }
        "assert" => {
            keys(&["until"], problems);
            Step::Assert { until: cond("until", problems) }
        }
        "shot" => {
            keys(&["name", "max_w"], problems);
            let name = text_of(o, "name", place, problems);
            if !name.is_empty() && !plain_name(&name) {
                problems.push(format!("{place}: 'name' becomes a file: letters, digits, _ and - only, at most 80"));
            }
            Step::Shot { name, max_w: int_of(o, "max_w", Some(1920), (1, u64::from(SIZE_MAX.0)), place, problems) }
        }
        "clip" => {
            keys(&["name", "frames", "every_ms", "max_w"], problems);
            let name = text_of(o, "name", place, problems);
            if !name.is_empty() && !plain_name(&name) {
                problems.push(format!("{place}: 'name' becomes a folder: letters, digits, _ and - only, at most 80"));
            }
            let frames = int_of(o, "frames", Some(30), (1, CLIP_FRAMES_MAX), place, problems);
            let every_ms = int_of(o, "every_ms", Some(66), CLIP_EVERY_MS, place, problems);
            if frames.saturating_mul(every_ms) > CLIP_MS_MAX {
                problems.push(format!("{place}: {frames} frames every {every_ms} ms is over {CLIP_MS_MAX} ms"));
            }
            Step::Clip { name, frames, every_ms, max_w: int_of(o, "max_w", Some(1280), (1, u64::from(SIZE_MAX.0)), place, problems) }
        }
        "pause" => {
            keys(&["ms"], problems);
            Step::Pause { ms: int_of(o, "ms", None, (1, WAIT_MS_MAX), place, problems) }
        }
        "audit" => {
            keys(&["scales", "fail"], problems);
            let scales: Vec<f64> = o.get("scales").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_f64).collect()).unwrap_or_default();
            let whole = o.get("scales").and_then(Value::as_array).is_some_and(|a| a.len() == scales.len());
            if scales.is_empty() || !whole || scales.iter().any(|s| !(0.5..=2.0).contains(s)) {
                problems.push(format!("{place}: 'scales' is a list of text sizes from 0.5 to 2.0, not empty"));
            }
            let fail: Vec<String> = match o.get("fail") {
                None => AUDIT_KINDS.iter().map(|k| k.to_string()).collect(),
                Some(list) => {
                    let names: Vec<String> = list.as_array().map(|a| a.iter().filter_map(|k| k.as_str().map(String::from)).collect()).unwrap_or_default();
                    if !list.is_array() || list.as_array().is_some_and(|a| a.len() != names.len()) || names.iter().any(|n| !AUDIT_KINDS.contains(&n.as_str())) {
                        problems.push(format!("{place}: 'fail' is a list of {}", AUDIT_KINDS.join(", ")));
                    }
                    names
                }
            };
            Step::Audit { scales, fail }
        }
        "repeat" => {
            keys(&["times", "steps", "until"], problems);
            let times = int_of(o, "times", None, (1, STEPS_MAX), place, problems);
            if depth >= REPEAT_DEPTH_MAX {
                problems.push(format!("{place}: 'repeat' nests at most {REPEAT_DEPTH_MAX} deep"));
                return bad;
            }
            let steps = match o.get("steps").and_then(Value::as_array).filter(|a| !a.is_empty()) {
                Some(list) => list.iter().enumerate().map(|(i, s)| parse_step(s, &format!("{place}.{}", i + 1), depth + 1, problems)).collect(),
                None => {
                    problems.push(format!("{place}: 'steps' is a list of steps, not empty"));
                    Vec::new()
                }
            };
            let until = o.get("until").map(|c| parse_cond(c, &format!("{place}.until"), 0, problems));
            Step::Repeat { times, steps, until }
        }
        "perf" => {
            problems.push(format!("{place}: step 'perf' is not built yet (INTEG1-E2)"));
            bad
        }
        other => {
            problems.push(format!("{place}: unknown step '{other}'"));
            bad
        }
    }
}

/// Actions the steps come to with every `repeat` and `times` counted out.
pub fn steps_counted(steps: &[Step]) -> u64 {
    steps.iter().fold(0u64, |n, s| {
        n.saturating_add(match s {
            Step::Key { times, .. } => *times,
            Step::Repeat { times, steps, .. } => times.saturating_mul(steps_counted(steps)),
            _ => 1,
        })
    })
}

fn has_clip(steps: &[Step]) -> bool {
    steps.iter().any(|s| match s {
        Step::Clip { .. } => true,
        Step::Repeat { steps, .. } => has_clip(steps),
        _ => false,
    })
}

/// A scenario from its JSON; `stem` is its file's name without the extension. Every problem, each with where it is.
pub fn parse(stem: &str, v: &Value) -> std::result::Result<Scenario, Vec<String>> {
    let mut problems = Vec::new();
    let p = &mut problems;
    let Some(o) = v.as_object() else { return Err(vec!["a scenario is an object".to_string()]) };
    unknown_keys(o, &["id", "title", "status", "why", "launch", "sizes", "settings", "timeout_s", "window", "steps"], "scenario", p);
    let id = text_of(o, "id", "scenario", p);
    if !id.is_empty() && id != stem {
        p.push(format!("scenario: id '{id}' is not the file's name '{stem}'"));
    }
    if !id.is_empty() && !plain_name(&id) {
        p.push(format!("scenario: id '{id}' becomes a folder: letters, digits, _ and - only, at most 80"));
    }
    let title = match o.get("title") {
        None => String::new(),
        Some(t) => t.as_str().map(String::from).unwrap_or_else(|| {
            p.push("scenario: 'title' is a text".into());
            String::new()
        }),
    };
    let why = o.get("why").and_then(Value::as_str).unwrap_or("").to_string();
    if o.contains_key("why") && why.is_empty() {
        p.push("scenario: 'why' is a text".into());
    }
    let status = match o.get("status").map(|s| s.as_str()) {
        None | Some(Some("active")) => Status::Active,
        Some(Some("waiting")) => {
            if why.is_empty() {
                p.push("scenario: status 'waiting' needs 'why'".into());
            }
            Status::Waiting(why)
        }
        Some(_) => {
            p.push("scenario: 'status' is 'active' or 'waiting'".into());
            Status::Active
        }
    };
    let mut launch = Launch { screen: "menu".into(), scene: None, seed: None, rules: None };
    if let Some(l) = o.get("launch") {
        match l.as_object() {
            None => p.push("launch: an object".into()),
            Some(l) => {
                let known: Vec<&str> = ["screen", "seed"].into_iter().chain(LAUNCH_ALIASES.iter().flat_map(|(k, old)| [*k, *old])).collect();
                unknown_keys(l, &known, "launch", p);
                match l.get("screen").map(|s| s.as_str()) {
                    None => {}
                    Some(Some(s @ ("menu" | "arena" | "runner"))) => launch.screen = s.to_string(),
                    Some(_) => p.push("launch: 'screen' is menu, arena or runner".into()),
                }
                launch.scene = aliased_text(l, "scene", p);
                if l.contains_key("seed") {
                    launch.seed = Some(int_of(l, "seed", None, (0, u64::MAX), "launch", p));
                }
                launch.rules = aliased_text(l, "rules", p);
            }
        }
    }
    let mut sizes = Vec::new();
    match o.get("sizes") {
        None => sizes.push(SIZE_MIN),
        Some(list) => match list.as_array().filter(|a| !a.is_empty()) {
            None => p.push("scenario: 'sizes' is a list of WxH, not empty".into()),
            Some(list) => {
                for s in list {
                    match s.as_str().map(parse_size) {
                        Some(Ok(size)) => sizes.push(size),
                        Some(Err(e)) => p.push(format!("scenario: {e}")),
                        None => p.push("scenario: 'sizes' is a list of WxH, not empty".into()),
                    }
                }
            }
        },
    }
    let settings = match o.get("settings") {
        None => Map::new(),
        Some(s) => match s.as_object() {
            Some(s) => s.clone(),
            None => {
                p.push("settings: an object".into());
                Map::new()
            }
        },
    };
    if settings.get("mute").is_some_and(|m| *m != json!(true)) {
        p.push("settings: a run is muted: 'mute' false is refused".into());
    }
    for k in HELD_SETTINGS.iter().filter(|k| settings.contains_key(**k)) {
        p.push(format!("settings: '{k}' belongs to the driver (a run stays windowed, at `sizes`, without the focus)"));
    }
    let timeout_s = int_of(o, "timeout_s", Some(300), (1, TIMEOUT_S_MAX), "scenario", p);
    let window = match o.get("window") {
        None => false,
        Some(w) => w.as_bool().unwrap_or_else(|| {
            p.push("scenario: 'window' is true or false".into());
            false
        }),
    };
    let steps: Vec<Step> = match o.get("steps").and_then(Value::as_array).filter(|a| !a.is_empty()) {
        Some(list) => list.iter().enumerate().map(|(i, s)| parse_step(s, &format!("step {}", i + 1), 0, p)).collect(),
        None => {
            p.push("scenario: 'steps' is a list of steps, not empty".into());
            Vec::new()
        }
    };
    if window && has_clip(&steps) {
        p.push("scenario: a 'clip' is taken in a run with no window, and this one has 'window' true".into());
    }
    if launch.rules.is_some() && launch.screen != "arena" {
        p.push("launch: 'rules' comes with 'screen' arena".into());
    }
    let counted = steps_counted(&steps);
    if counted > STEPS_MAX {
        p.push(format!("scenario: {counted} steps after repeat, at most {STEPS_MAX}"));
    }
    if problems.is_empty() {
        Ok(Scenario { id, title, status, launch, sizes, settings, timeout: Duration::from_secs(timeout_s), window, steps })
    } else {
        Err(problems)
    }
}

/// A scenario file, or its problems, each naming the file.
pub fn load(path: &Path) -> std::result::Result<Scenario, Vec<String>> {
    let named = |e: String| format!("{}: {e}", path.display());
    let size = std::fs::metadata(path).map_err(|e| vec![named(e.to_string())])?.len();
    if size > FILE_MAX {
        return Err(vec![named(format!("{size} bytes, at most {FILE_MAX}"))]);
    }
    let text = std::fs::read_to_string(path).map_err(|e| vec![named(e.to_string())])?;
    let v: Value = serde_json::from_str(&text).map_err(|e| vec![named(format!("bad JSON: {e}"))])?;
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    parse(&stem, &v).map_err(|problems| problems.into_iter().map(named).collect())
}

/// The `*.json` files of a scenarios folder by name (none when it does not exist).
pub fn files_in(dir: &Path) -> Vec<PathBuf> {
    files_with(dir, "json")
}

/// Every problem of the files, each naming its file; empty when all parse. What `check` and a game's parse test call.
pub fn check_files(files: &[PathBuf]) -> Vec<String> {
    files.iter().flat_map(|f| load(f).err().unwrap_or_default()).collect()
}

// ---- conditions ----

/// The value at a dotted path: object keys, and list indexes as numbers. None when it is not there or is null.
pub fn at_path<'a>(v: &'a Value, path: &str) -> Option<&'a Value> {
    let mut at = v;
    for part in path.split('.') {
        at = match at {
            Value::Object(o) => o.get(part)?,
            Value::Array(a) => a.get(part.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    (!at.is_null()).then_some(at)
}

fn same(a: &Value, b: &Value) -> bool {
    match (a.as_f64(), b.as_f64()) {
        (Some(x), Some(y)) => x == y,
        _ => a == b,
    }
}

/// `absent` is the only op a missing path passes, so a mistyped path never reads as true.
pub fn op_holds(value: Option<&Value>, op: &Op) -> bool {
    match (value, op) {
        (None, Op::Absent) => true,
        (None, _) | (Some(_), Op::Absent) => false,
        (Some(v), Op::Eq(want)) => same(v, want),
        (Some(v), Op::Ne(want)) => !same(v, want),
        (Some(v), Op::Gt(n)) => v.as_f64().is_some_and(|x| x > *n),
        (Some(v), Op::Lt(n)) => v.as_f64().is_some_and(|x| x < *n),
        (Some(Value::Array(a)), Op::Has(want)) => a.iter().any(|x| same(x, want)),
        (Some(Value::Object(o)), Op::Has(want)) => want.as_str().is_some_and(|k| o.contains_key(k)),
        (Some(Value::String(s)), Op::Has(want)) => want.as_str().is_some_and(|w| s.contains(w)),
        (Some(_), Op::Has(_)) => false,
    }
}

/// Whether a condition holds on the two replies, and when not, what was there.
pub fn cond_holds(c: &Cond, tree: &Value, game: &Value) -> std::result::Result<(), String> {
    match c {
        Cond::Path { source, path, op } => {
            let value = at_path(if *source == Source::Game { game } else { tree }, path);
            if op_holds(value, op) {
                Ok(())
            } else {
                Err(format!("{path} is {}, wanted {op:?}", value.map(Value::to_string).unwrap_or_else(|| "absent".into())))
            }
        }
        Cond::Widget(id) => {
            if tree["widgets"].as_array().is_some_and(|w| w.iter().any(|x| x["id"] == json!(id))) {
                Ok(())
            } else {
                Err(format!("no widget '{id}' on {}", tree["screen"]))
            }
        }
        Cond::All(list) => list.iter().try_for_each(|c| cond_holds(c, tree, game)),
    }
}

fn reads_game(c: &Cond) -> bool {
    match c {
        Cond::Path { source, .. } => *source == Source::Game,
        Cond::Widget(_) => false,
        Cond::All(list) => list.iter().any(reads_game),
    }
}

// ---- a run ----

/// The agent link of a running game: one request, one reply.
pub trait Link {
    fn ask(&mut self, body: &Value) -> std::result::Result<Value, String>;
    /// How long a reply may take from now on.
    fn patience(&mut self, _reply: Duration) {}
}

/// The process a run started.
pub trait Process {
    fn pid(&self) -> u32;
    /// How it ended, when it has.
    fn exited(&mut self) -> Option<String>;
    /// Ends it, if it has not ended: waits `leave` for a clean exit, then kills it. Called once per start.
    fn stop(&mut self, leave: Duration) -> GameExit;
    /// What it wrote to stderr so far.
    fn stderr(&self) -> String;
}

/// How a started game ended: its exit code (None when the OS gives none), and whether the driver had to kill it.
#[derive(Debug, Clone, PartialEq)]
pub struct GameExit {
    pub code: Option<i32>,
    pub killed: bool,
}

/// What the default panic hook writes to stderr, whichever thread panicked.
const PANIC_TEXT: &str = "panicked at";

/// What is wrong with how a game ended, even if every step passed.
fn exit_problem(exit: &GameExit, stderr: &str) -> Option<String> {
    if let Some(line) = stderr.lines().find(|l| l.contains(PANIC_TEXT)) {
        return Some(format!("the game panicked: {}", cut(line, 300)));
    }
    if exit.killed {
        return Some("the game did not leave when asked and was killed".to_string());
    }
    (exit.code != Some(0)).then(|| format!("the game exited with code {}", exit.code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".into())))
}

/// Starts the game for a plan. A failure is a reason; `retry` when another port may help.
pub struct StartFailure {
    pub reason: String,
    pub retry: bool,
}

pub trait Launcher {
    fn start(&mut self, plan: &Plan) -> std::result::Result<(Box<dyn Link>, Box<dyn Process>), StartFailure>;
}

/// One start of the game: what the program is given, all of it under the scratch folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub exe: PathBuf,
    pub args: Vec<String>,
    pub port: u16,
    /// What the child's APPDATA becomes; the settings, saves and recordings go under it.
    pub appdata: PathBuf,
    pub settings_path: PathBuf,
    pub settings: Value,
    pub recordings: PathBuf,
}

/// The scratch profile, with the scenario's settings over it: muted, windowed at `size`, no vsync, recording on, the
/// window not taking the focus.
pub fn scratch_settings(s: &Scenario, size: (u32, u32)) -> Value {
    let mut v = json!({"mute": true, "fullscreen": false, "borderless": false, "window_size": format!("{}x{}", size.0, size.1), "vsync": false, "record_playtests": true, "start_inactive": true});
    for (k, x) in &s.settings {
        v[k] = x.clone();
    }
    v
}

/// Where a game's user folder lies under a folder standing for APPDATA (`aegis_save::user_dir`; a test holds them equal).
pub fn user_dir_under(appdata: &Path, game_id: &str) -> PathBuf {
    appdata.join("Aegis").join(game_id)
}

/// A free port from the OS, for this start.
pub fn free_port() -> std::io::Result<u16> {
    Ok(std::net::TcpListener::bind(("127.0.0.1", 0))?.local_addr()?.port())
}

pub fn plan(s: &Scenario, size: (u32, u32), game_id: &str, exe: &Path, data: Option<&Path>, port: u16, scratch: &Path) -> Plan {
    let user = user_dir_under(scratch, game_id);
    let mut args = vec![format!("port={port}"), format!("saves={}", scratch.join("saves").display()), format!("screen={}", s.launch.screen)];
    if let Some(e) = &s.launch.scene {
        args.push(format!("scene={e}"));
    }
    if let Some(n) = s.launch.seed {
        args.push(format!("seed={n}"));
    }
    if let Some(r) = &s.launch.rules {
        args.push(format!("rules={r}"));
    }
    if let Some(d) = data {
        args.push(format!("data={}", d.display()));
    }
    Plan { exe: exe.to_path_buf(), args, port, appdata: scratch.to_path_buf(), settings_path: user.join("settings.json"), settings: scratch_settings(s, size), recordings: user.join("playtests") }
}

/// Puts the plan's scratch folder in place: the settings file, the saves folder.
pub fn write_scratch(p: &Plan) -> std::io::Result<()> {
    if let Some(dir) = p.settings_path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::create_dir_all(p.appdata.join("saves"))?;
    std::fs::write(&p.settings_path, serde_json::to_string_pretty(&p.settings).unwrap_or_default())
}

/// What a game with no window is told: the size, and the rate and device scale when asked.
pub fn headless_args(size: (u32, u32), o: &Options) -> Vec<String> {
    let mut args = vec![format!("headless={}x{}", size.0, size.1)];
    args.extend(o.fps.map(|n| format!("fps={n}")));
    args.extend(o.dpr.as_ref().map(|d| format!("dpr={d}")));
    args
}

/// What a run needs beyond the file.
#[derive(Debug, Clone)]
pub struct Options {
    pub game_id: String,
    pub exe: PathBuf,
    pub data: Option<PathBuf>,
    pub out: PathBuf,
    pub slow: u32,
    /// `size=`: this size in place of the file's.
    pub size: Option<(u32, u32)>,
    /// How long a window may take to reach the asked size before the run goes on at the size it has (`SIZED_MS`).
    pub sized_ms: u64,
    /// `window=1`: every scenario in a real window, as if its file asked.
    pub window: bool,
    /// `fps=` and `dpr=`: for a game with no window (its own bounds refuse a bad one).
    pub fps: Option<u32>,
    pub dpr: Option<String>,
}

/// One size's result.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub id: String,
    pub size: (u32, u32),
    /// 0 pass, 1 a step failed, 2 could not start.
    pub code: i32,
    pub step: Option<String>,
    pub message: String,
    pub bundle: PathBuf,
}

/// A step failed: where, and why. `gone`: the game has ended, so there is nothing more to ask it.
struct Failure {
    step: String,
    message: String,
    gone: bool,
}

struct Run<'a> {
    link: &'a mut dyn Link,
    game: &'a mut dyn Process,
    bundle: &'a Path,
    slow: u64,
    deadline: Instant,
    done: Vec<Value>,
    frame: u64,
    /// No window: frames are asked for, and looks are further apart.
    headless: bool,
}

impl Run<'_> {
    fn ask(&mut self, step: &str, body: &Value) -> std::result::Result<Value, Failure> {
        self.link.ask(body).map_err(|e| {
            // The link closes a moment before the process counts as ended.
            let by = Instant::now() + GONE_WAIT;
            loop {
                if let Some(how) = self.game.exited() {
                    return Failure { step: step.to_string(), message: format!("the game ended mid-run: {how}"), gone: true };
                }
                if Instant::now() >= by {
                    return Failure { step: step.to_string(), message: format!("the link: {e}"), gone: false };
                }
                std::thread::sleep(POLL);
            }
        })
    }

    fn tree(&mut self, step: &str) -> std::result::Result<Value, Failure> {
        let tree = self.ask(step, &json!({"cmd": "ui_tree"}))?;
        let Some(frame) = tree["frame"].as_u64() else {
            return Err(Failure { step: step.to_string(), message: format!("ui_tree has no frame count: {}", cut(&tree.to_string(), 200)), gone: false });
        };
        self.frame = frame;
        if tree["px"][0].as_i64().is_some_and(|w| w < WINDOW_MIN.0) || tree["px"][1].as_i64().is_some_and(|h| h < WINDOW_MIN.1) {
            return Err(Failure { step: step.to_string(), message: format!("the window was minimized under the run (it is {} px)", tree["px"]), gone: false });
        }
        Ok(tree)
    }

    fn over_time(&self, step: &str, until: Instant, what: &str) -> Option<Failure> {
        let now = Instant::now();
        if now >= self.deadline {
            return Some(Failure { step: step.to_string(), message: format!("the run's timeout_s ran out: {what}"), gone: false });
        }
        (now >= until).then(|| Failure { step: step.to_string(), message: format!("timed out: {what}"), gone: false })
    }

    /// Waits for two more frames, never a sleep of a guessed length: what the last action changed is on the canvas after.
    fn settle(&mut self, step: &str) -> std::result::Result<Value, Failure> {
        if self.headless {
            // Its frames come at a low rate, and on request at once: drawn before the reply, so nothing to wait for.
            let reply = self.ask(step, &json!({"cmd": "frames", "frames": SETTLE_FRAMES}))?;
            if reply["ok"] != json!(true) {
                return Err(Failure { step: step.to_string(), message: format!("the frames asked for: {}", reply["error"]), gone: false });
            }
            return self.tree(step);
        }
        let mut tree = self.tree(step)?;
        let want = self.frame + SETTLE_FRAMES;
        let until = Instant::now() + Duration::from_millis(SETTLE_MS * self.slow);
        while self.frame < want {
            if let Some(fail) = self.over_time(step, until, &format!("the window drew no frame past {} (wanted {want})", self.frame)) {
                return Err(fail);
            }
            std::thread::sleep(POLL);
            tree = self.tree(step)?;
        }
        Ok(tree)
    }

    /// A request whose reply must not say `ok` false, and the frames after it.
    fn act(&mut self, step: &str, body: Value) -> std::result::Result<(), Failure> {
        let reply = self.ask(step, &body)?;
        if reply["ok"] == json!(false) {
            return Err(Failure { step: step.to_string(), message: format!("{body} refused: {}", reply["error"]), gone: false });
        }
        self.settle(step)?;
        Ok(())
    }

    fn holds(&mut self, step: &str, c: &Cond) -> std::result::Result<std::result::Result<(), String>, Failure> {
        let tree = self.tree(step)?;
        let game = if reads_game(c) { self.ask(step, &json!({"cmd": "game"}))? } else { Value::Null };
        Ok(cond_holds(c, &tree, &game))
    }

    fn step(&mut self, label: &str, s: &Step) -> std::result::Result<(), Failure> {
        let started = Instant::now();
        if let Some(fail) = self.over_time(label, self.deadline, "before the step") {
            return Err(fail);
        }
        match s {
            Step::Click { id } => self.act(label, json!({"cmd": "click", "id": id}))?,
            Step::Hover { id } => self.act(label, json!({"cmd": "hover", "id": id}))?,
            Step::Key { key, times } => {
                for _ in 0..*times {
                    self.act(label, json!({"cmd": "key", "key": key}))?;
                }
            }
            Step::Type { text } => self.act(label, json!({"cmd": "type", "text": text}))?,
            Step::Drag { id, to } => self.act(label, json!({"cmd": "drag", "id": id, "to": to}))?,
            Step::Back => self.act(label, json!({"cmd": "back"}))?,
            Step::Link { body } => self.act(label, body.clone())?,
            Step::Wait { until, timeout_ms } => {
                let by = Instant::now() + Duration::from_millis(timeout_ms * self.slow);
                loop {
                    let why = match self.holds(label, until)? {
                        Ok(()) => break,
                        Err(why) => why,
                    };
                    if let Some(fail) = self.over_time(label, by, &why) {
                        return Err(fail);
                    }
                    std::thread::sleep(if self.headless { POLL_HEADLESS } else { POLL });
                }
            }
            Step::Pause { ms } => {
                let by = Instant::now() + Duration::from_millis(*ms);
                if by > self.deadline {
                    return Err(Failure { step: label.to_string(), message: format!("a pause of {ms} ms runs past the run's timeout_s"), gone: false });
                }
                while Instant::now() + PAUSE_LOOK < by {
                    std::thread::sleep(PAUSE_LOOK);
                    self.tree(label)?;
                }
                std::thread::sleep(by.saturating_duration_since(Instant::now()));
            }
            Step::Assert { until } => {
                if let Err(why) = self.holds(label, until)? {
                    return Err(Failure { step: label.to_string(), message: format!("assert: {why}"), gone: false });
                }
            }
            Step::Shot { name, max_w } => {
                self.settle(label)?;
                let path = self.bundle.join(format!("{name}.png"));
                let reply = self.ask(label, &json!({"cmd": "screenshot", "path": path.display().to_string(), "max_w": max_w}))?;
                if reply["ok"] != json!(true) {
                    return Err(Failure { step: label.to_string(), message: format!("the shot {name}: {}", reply["error"]), gone: false });
                }
            }
            Step::Clip { name, frames, every_ms, max_w } => {
                if !self.headless {
                    return Err(Failure { step: label.to_string(), message: "a clip is taken in a run with no window".to_string(), gone: false });
                }
                if Instant::now() + Duration::from_millis(frames * every_ms) > self.deadline {
                    return Err(Failure { step: label.to_string(), message: format!("a clip of {} ms runs past the run's timeout_s", frames * every_ms), gone: false });
                }
                self.settle(label)?;
                let dir = self.bundle.join(name);
                // It replies when the last frame is written, and frames are written on the game's own time.
                self.link.patience(REPLY * self.slow as u32 + Duration::from_millis(frames * every_ms));
                let reply = self.ask(label, &json!({"cmd": "clip", "dir": dir.display().to_string(), "frames": frames, "every_ms": every_ms, "max_w": max_w}))?;
                self.link.patience(REPLY * self.slow as u32);
                if reply["ok"] != json!(true) {
                    return Err(Failure { step: label.to_string(), message: format!("the clip {name}: {}", reply["error"]), gone: false });
                }
            }
            Step::Audit { scales, fail } => {
                self.settle(label)?;
                for scale in scales {
                    let reply = self.ask(label, &json!({"cmd": "text_audit", "scale": scale}))?;
                    let screen = reply["screen"].as_str().unwrap_or("screen").to_string();
                    let keep = self.bundle.join(format!("audit_{}_{screen}_{scale}.json", label.replace(['.', '#'], "_")));
                    let _ = std::fs::write(keep, serde_json::to_string_pretty(&reply).unwrap_or_default());
                    let found: Vec<String> = fail.iter().filter(|k| reply["counts"][k.as_str()].as_u64().unwrap_or(0) > 0).map(|k| format!("{k} {}", reply["counts"][k.as_str()])).collect();
                    if !found.is_empty() {
                        return Err(Failure { step: label.to_string(), message: format!("the text audit at {scale} on {screen}: {}", found.join(", ")), gone: false });
                    }
                }
            }
            Step::Repeat { times, steps, until } => {
                let mut met = false;
                for turn in 1..=*times {
                    if let Some(c) = until {
                        if self.holds(label, c)?.is_ok() {
                            met = true;
                            break;
                        }
                    }
                    for (i, inner) in steps.iter().enumerate() {
                        self.step(&format!("{label}.{}#{turn}", i + 1), inner)?;
                    }
                }
                if let (Some(c), false) = (until, met) {
                    if let Err(why) = self.holds(label, c)? {
                        return Err(Failure { step: label.to_string(), message: format!("repeat: {times} turns and {why}"), gone: false });
                    }
                }
            }
        }
        self.done.push(json!({"step": label, "kind": kind_of(s), "ms": started.elapsed().as_millis() as u64}));
        Ok(())
    }
}

fn kind_of(s: &Step) -> &'static str {
    match s {
        Step::Click { .. } => "click",
        Step::Hover { .. } => "hover",
        Step::Key { .. } => "key",
        Step::Type { .. } => "type",
        Step::Drag { .. } => "drag",
        Step::Back => "back",
        Step::Link { .. } => "link",
        Step::Wait { .. } => "wait",
        Step::Assert { .. } => "assert",
        Step::Shot { .. } => "shot",
        Step::Clip { .. } => "clip",
        Step::Pause { .. } => "pause",
        Step::Audit { .. } => "audit",
        Step::Repeat { .. } => "repeat",
    }
}

fn cut(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

/// Asks until the window is ready: the link names the process that was started, frames come, and it has reached the size
/// (or will not: a capped size stays smaller). `Err(retry)` says whether another port may help.
fn ready(link: &mut dyn Link, game: &mut dyn Process, want: (u32, u32), sized_ms: u64, slow: u64, poll: Duration) -> std::result::Result<(Value, Value), StartFailure> {
    let fail = |reason: String, retry: bool| StartFailure { reason, retry };
    let first = link.ask(&json!({"cmd": "ui_tree"})).map_err(|e| fail(format!("the link: {e}"), false))?;
    if first["pid"] != json!(game.pid()) {
        return Err(fail(format!("the port answers for process {}, not the game started ({})", first["pid"], game.pid()), true));
    }
    let start = first["frame"].as_u64().ok_or_else(|| fail(format!("ui_tree has no frame count: {}", cut(&first.to_string(), 200)), false))?;
    let wanted = json!([want.0, want.1]);
    let sized_by = Instant::now() + Duration::from_millis(sized_ms * slow);
    let stalled_by = Instant::now() + Duration::from_millis(SETTLE_MS * slow);
    loop {
        let tree = link.ask(&json!({"cmd": "ui_tree"})).map_err(|e| fail(format!("the link: {e}"), false))?;
        let frames = tree["frame"].as_u64().unwrap_or(0);
        if frames >= start + SETTLE_FRAMES && (tree["px"] == wanted || Instant::now() >= sized_by) {
            return Ok((tree["px"].clone(), tree["canvas"].clone()));
        }
        if Instant::now() >= stalled_by {
            return Err(fail(format!("the window drew no frame past {frames}"), false));
        }
        if let Some(how) = game.exited() {
            return Err(fail(format!("the game ended while starting: {how}"), false));
        }
        std::thread::sleep(poll);
    }
}

/// The recording of the run, as a trail report in the bundle (`aegis replay ... trail-report` reads the same): its flag
/// counts, or null when the run recorded nothing.
fn keep_trail(recordings: &Path, bundle: &Path) -> Value {
    let found = aegis_host::playtest::list(recordings).into_iter().next();
    let Some((path, Ok(rec))) = found else { return Value::Null };
    let report = aegis_host::trail::build(&rec.trail, rec.trail_dropped, &rec.notes);
    let _ = std::fs::copy(path, bundle.join("recording.json"));
    let _ = std::fs::write(bundle.join("trail.json"), serde_json::to_string_pretty(&report).unwrap_or_default());
    report["counts"].clone()
}

/// A failed step's picture and tree while the game can still answer. None of this may fail the failing further.
fn keep_failure(link: &mut dyn Link, bundle: &Path) {
    link.patience(REPLY_LEAVING);
    if let Ok(tree) = link.ask(&json!({"cmd": "ui_tree"})) {
        let _ = std::fs::write(bundle.join("fail_tree.json"), serde_json::to_string_pretty(&tree).unwrap_or_default());
    }
    let path = bundle.join("fail.png");
    let _ = link.ask(&json!({"cmd": "screenshot", "path": path.display().to_string(), "max_w": 1920}));
}

/// Empties a bundle of an earlier run. A folder without a `result.json` is not one, and is left alone.
fn clear_bundle(bundle: &Path) -> std::io::Result<()> {
    if bundle.join("result.json").is_file() {
        std::fs::remove_dir_all(bundle)?;
    }
    std::fs::create_dir_all(bundle)
}

/// One scenario at one size: the game started, the steps walked, the game stopped, the bundle written. Each start gets its
/// own port and scratch folder, removed after.
pub fn run_size(s: &Scenario, size: (u32, u32), o: &Options, launcher: &mut dyn Launcher) -> Outcome {
    let slow = u64::from(o.slow.clamp(1, SLOW_MAX));
    let leave = LEAVE * slow as u32;
    let bundle = o.out.join(&s.id).join(format!("{}x{}", size.0, size.1));
    let mut out = Outcome { id: s.id.clone(), size, code: 2, step: None, message: String::new(), bundle: bundle.clone() };
    if let Err(e) = clear_bundle(&bundle) {
        out.message = format!("the bundle folder {}: {e}", bundle.display());
        return out;
    }
    let started = Instant::now();
    let headless = !(s.window || o.window);
    let poll = if headless { POLL_HEADLESS } else { POLL };
    let mut record = json!({"id": s.id, "title": s.title, "size": format!("{}x{}", size.0, size.1), "slow": slow, "window": !headless});
    for attempt in 1..=PORT_TRIES {
        let port = match free_port() {
            Ok(p) => p,
            Err(e) => {
                out.message = format!("no free port: {e}");
                break;
            }
        };
        let scratch = std::env::temp_dir().join("aegis_scenario").join(format!("{}_{port}", std::process::id()));
        let _ = std::fs::remove_dir_all(&scratch);
        let mut plan = plan(s, size, &o.game_id, &o.exe, o.data.as_deref(), port, &scratch);
        if headless {
            plan.args.extend(headless_args(size, o));
        }
        if let Err(e) = write_scratch(&plan) {
            out.message = format!("the scratch folder {}: {e}", scratch.display());
            break;
        }
        let (mut link, mut game) = match launcher.start(&plan) {
            Ok(both) => both,
            Err(f) => {
                out.message = f.reason;
                let _ = std::fs::remove_dir_all(&scratch);
                if f.retry && attempt < PORT_TRIES {
                    continue;
                }
                break;
            }
        };
        record["port"] = json!(port);
        let readiness = ready(link.as_mut(), game.as_mut(), size, o.sized_ms, slow, poll);
        let (px, canvas) = match readiness {
            Ok(both) => both,
            Err(f) => {
                // `retry` is a port some other program answers on: that program is not this run's to stop.
                if !f.retry {
                    link.patience(REPLY_LEAVING);
                    let _ = link.ask(&json!({"cmd": "quit"}));
                }
                game.stop(leave);
                let _ = std::fs::write(bundle.join("stderr.txt"), game.stderr());
                let _ = std::fs::remove_dir_all(&scratch);
                out.message = f.reason;
                if f.retry && attempt < PORT_TRIES {
                    continue;
                }
                break;
            }
        };
        record["px"] = px.clone();
        record["canvas"] = canvas;
        record["capped"] = json!(px != json!([size.0, size.1]));
        let deadline = Instant::now() + s.timeout * slow as u32;
        let mut run = Run { link: link.as_mut(), game: game.as_mut(), bundle: &bundle, slow, deadline, done: Vec::new(), frame: 0, headless };
        let walked = s.steps.iter().enumerate().try_for_each(|(i, step)| run.step(&(i + 1).to_string(), step));
        let done = std::mem::take(&mut run.done);
        let frames = run.frame;
        match walked {
            Ok(()) => {
                out.code = 0;
                out.message = "pass".into();
            }
            Err(f) => {
                out.code = 1;
                out.step = Some(f.step);
                out.message = f.message;
                if !f.gone {
                    keep_failure(link.as_mut(), &bundle);
                }
            }
        }
        link.patience(REPLY_LEAVING);
        let _ = link.ask(&json!({"cmd": "quit"}));
        let exit = game.stop(leave);
        let stderr = game.stderr();
        let _ = std::fs::write(bundle.join("stderr.txt"), &stderr);
        record["game_exit"] = json!({"code": exit.code, "killed": exit.killed, "panicked": stderr.contains(PANIC_TEXT)});
        // Every step passed and the game then ended badly: the run did not pass. A failed step keeps its own message.
        if let (0, Some(problem)) = (out.code, exit_problem(&exit, &stderr)) {
            out.code = 1;
            out.step = Some("stop".into());
            out.message = problem;
        }
        record["steps"] = Value::Array(done);
        record["frames"] = json!(frames);
        record["trail"] = keep_trail(&plan.recordings, &bundle);
        let _ = std::fs::remove_dir_all(&scratch);
        break;
    }
    record["exit"] = json!(out.code);
    record["result"] = json!(match out.code {
        0 => "pass",
        1 => "fail",
        _ => "could not start",
    });
    record["step"] = json!(out.step);
    record["message"] = json!(out.message);
    record["seconds"] = json!(started.elapsed().as_secs_f64());
    if let Err(e) = std::fs::write(bundle.join("result.json"), serde_json::to_string_pretty(&record).unwrap_or_default()) {
        out.code = out.code.max(1);
        out.message = format!("{}; result.json: {e}", out.message);
    }
    out
}

/// A scenario at each of its sizes (`size=` in their place). A waiting one never runs: no outcome.
pub fn run_scenario(s: &Scenario, o: &Options, launcher: &mut dyn Launcher) -> Vec<Outcome> {
    if matches!(s.status, Status::Waiting(_)) {
        return Vec::new();
    }
    let sizes = o.size.map(|one| vec![one]).unwrap_or_else(|| s.sizes.clone());
    sizes.into_iter().map(|size| run_size(s, size, o, launcher)).collect()
}

// ---- the real program ----

/// The agent link over TCP.
pub struct TcpLink {
    reader: std::io::BufReader<std::net::TcpStream>,
    writer: std::net::TcpStream,
}

impl TcpLink {
    pub fn connect(port: u16, reply: Duration) -> std::io::Result<Self> {
        let stream = std::net::TcpStream::connect(("127.0.0.1", port))?;
        stream.set_read_timeout(Some(reply))?;
        stream.set_nodelay(true)?;
        let writer = stream.try_clone()?;
        Ok(Self { reader: std::io::BufReader::new(stream), writer })
    }
}

impl Link for TcpLink {
    fn ask(&mut self, body: &Value) -> std::result::Result<Value, String> {
        use std::io::{BufRead, Write};
        self.writer.write_all(format!("{body}
").as_bytes()).map_err(|e| e.to_string())?;
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => Err("the link closed".to_string()),
            Ok(_) => serde_json::from_str(&line).map_err(|e| format!("a reply that is no JSON: {e}")),
            Err(e) => Err(format!("no reply: {e}")),
        }
    }

    fn patience(&mut self, reply: Duration) {
        let _ = self.reader.get_ref().set_read_timeout(Some(reply));
    }
}

/// The game process this run started. Dropped without a stop it is still ended: nothing outlives the run.
pub struct GameProcess {
    child: std::process::Child,
    stderr_path: PathBuf,
    ended: Option<String>,
    code: Option<i32>,
}

impl Process for GameProcess {
    fn pid(&self) -> u32 {
        self.child.id()
    }

    fn exited(&mut self) -> Option<String> {
        if self.ended.is_none() {
            if let Ok(Some(status)) = self.child.try_wait() {
                self.ended = Some(status.to_string());
                self.code = status.code();
            }
        }
        self.ended.as_ref().map(|how| format!("{how}; stderr: {}", cut(&self.stderr(), 2000)))
    }

    fn stop(&mut self, leave: Duration) -> GameExit {
        let by = Instant::now() + leave;
        while Instant::now() < by {
            if self.exited().is_some() {
                return GameExit { code: self.code, killed: false };
            }
            std::thread::sleep(POLL * 4);
        }
        let _ = self.child.kill();
        if let Ok(status) = self.child.wait() {
            self.ended = Some(format!("{status} (killed by the driver)"));
            self.code = status.code();
        }
        GameExit { code: self.code, killed: true }
    }

    fn stderr(&self) -> String {
        use std::io::{Read, Seek, SeekFrom};
        let Ok(mut f) = std::fs::File::open(&self.stderr_path) else { return String::new() };
        let size = f.metadata().map(|m| m.len()).unwrap_or(0);
        let _ = f.seek(SeekFrom::Start(size.saturating_sub(STDERR_MAX)));
        let mut bytes = Vec::new();
        let _ = f.read_to_end(&mut bytes);
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

impl Drop for GameProcess {
    fn drop(&mut self) {
        if self.ended.is_none() && matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

/// Starts the real program, with or without a window as the plan's arguments say: its APPDATA the scratch folder, its
/// output in a file there.
pub struct GameLauncher {
    pub slow: u64,
}

impl Launcher for GameLauncher {
    fn start(&mut self, plan: &Plan) -> std::result::Result<(Box<dyn Link>, Box<dyn Process>), StartFailure> {
        let fail = |reason: String| StartFailure { reason, retry: false };
        if !plan.exe.is_file() {
            return Err(fail(format!("no game program at {} (build it, or pass exe=)", plan.exe.display())));
        }
        let stderr_path = plan.appdata.join("stderr.txt");
        let log = std::fs::File::create(&stderr_path).map_err(|e| fail(format!("{}: {e}", stderr_path.display())))?;
        let copy = log.try_clone().map_err(|e| fail(e.to_string()))?;
        let child = std::process::Command::new(&plan.exe)
            .args(&plan.args)
            .env("APPDATA", &plan.appdata)
            .stdin(std::process::Stdio::null())
            .stdout(copy)
            .stderr(log)
            .spawn()
            .map_err(|e| fail(format!("starting {}: {e}", plan.exe.display())))?;
        let mut game = GameProcess { child, stderr_path, ended: None, code: None };
        let by = Instant::now() + Duration::from_millis(START_MS * self.slow);
        let link = loop {
            match TcpLink::connect(plan.port, REPLY * self.slow as u32) {
                Ok(link) => break link,
                Err(_) => {
                    if let Some(how) = game.exited() {
                        return Err(fail(format!("the game ended while starting: {how}")));
                    }
                    if Instant::now() >= by {
                        return Err(fail(format!("nothing answered on port {} in {} s", plan.port, START_MS * self.slow / 1000)));
                    }
                    std::thread::sleep(POLL * 4);
                }
            }
        };
        Ok((Box::new(link), Box::new(game)))
    }
}

// ---- the command line ----

fn game_folder<G: crate::Game>(game: &G, args: &Args) -> Result<PathBuf> {
    // `data=` is the game's data folder here, which may be a scratch copy away from the game's folder.
    let mut keys = args.keys.clone();
    keys.shift_remove("data");
    let manifest = game.manifest_path(&Args { positional: args.positional.clone(), keys })?;
    manifest.parent().map(Path::to_path_buf).with_context(|| format!("{} has no folder", manifest.display()))
}

const RUN_USAGE: &str = "aegis scenario run <id | all | file.json> [size=WxH] [out=<dir>] [slow=N] [exe=<game exe>] [data=<data dir>] [window=1] [fps=N] [dpr=X]";
const SHOTS_USAGE: &str = "aegis scenario shots <name> [screen=menu|arena|runner] [scene=<id>] [rules=<id>] [seed=N] [do=<a>,<b>,...] [size=WxH] [out=<dir>] [slow=N] [exe=<game exe>] [data=<data dir>] [window=1] [fps=N] [dpr=X]";

/// `aegis scenario ...`; returns the exit code.
pub fn command<G: crate::Game>(game: &G, args: &Args) -> Result<i32> {
    let folder = game_folder(game, args)?;
    let scenarios = folder.join("scenarios");
    match args.pos(1) {
        Some("check") => Ok(check(&scenarios, args.pos(2).unwrap_or("all"))),
        Some("run") => {
            let Some(what) = args.pos(2) else {
                println!("usage: {RUN_USAGE}");
                return Ok(2);
            };
            run(&folder, &scenarios, what, args)
        }
        Some("shots") => {
            let Some(name) = args.pos(2) else {
                println!("usage: {SHOTS_USAGE}");
                return Ok(2);
            };
            shots(&folder, name, args)
        }
        _ => {
            println!("usage: aegis scenario check [<file> | all]\n       {RUN_USAGE}\n       {SHOTS_USAGE}");
            Ok(2)
        }
    }
}

fn check(scenarios: &Path, what: &str) -> i32 {
    let files = if what == "all" { files_in(scenarios) } else { vec![PathBuf::from(what)] };
    let problems = check_files(&files);
    for p in &problems {
        println!("§ {p}");
    }
    println!("§ scenario check: {} file(s), {} problem(s)", files.len(), problems.len());
    i32::from(!problems.is_empty())
}

/// Whether `window=` asks for a real window in every scenario: by default none is opened.
pub fn window_asked(args: &Args) -> Result<bool> {
    match args.key("window", "0").as_str() {
        "0" => Ok(false),
        "1" => Ok(true),
        other => anyhow::bail!("window is 1 or 0, not '{other}'"),
    }
}

/// The keys `run` and `shots` share; `out` is the bundle folder's default.
fn options_of(folder: &Path, args: &Args, out: &str) -> Result<Options> {
    let manifest = aegis_content::manifest::Manifest::read(&folder.join(aegis_content::manifest::FILE)).map_err(anyhow::Error::msg)?;
    let exe = match args.keys.get("exe") {
        Some(path) => PathBuf::from(path),
        None => std::env::current_exe()?.with_file_name(format!("{}{}", manifest.id, std::env::consts::EXE_SUFFIX)),
    };
    let slow: u32 = args.key("slow", "1").parse().ok().filter(|n| (1..=SLOW_MAX).contains(n)).with_context(|| format!("slow is a whole number from 1 to {SLOW_MAX}"))?;
    let size = args.keys.get("size").map(|s| parse_size(s)).transpose().map_err(anyhow::Error::msg)?;
    let out = PathBuf::from(args.key("out", out));
    let window = window_asked(args)?;
    let fps = args.keys.get("fps").map(|n| n.parse::<u32>().with_context(|| format!("fps '{n}' is not a whole number"))).transpose()?;
    Ok(Options { game_id: manifest.id.clone(), exe, data: args.keys.get("data").map(PathBuf::from), out, slow, size, sized_ms: SIZED_MS, window, fps, dpr: args.keys.get("dpr").cloned() })
}

/// The steps of `do=`, comma separated: `key:<k>`, `click:<id>`, `hover:<id>`, `screen:<id>` (wait for it), `widget:<id>`
/// (wait for it), `pause:<ms>`, `shot:<name>`, `clip:<name>[:<frames>[:<every_ms>]]`. Without a shot or clip anywhere, one
/// shot named `shot` ends the list.
pub fn do_steps(list: &str) -> std::result::Result<Vec<Value>, String> {
    let mut steps = Vec::new();
    for item in list.split(',').filter(|i| !i.is_empty()) {
        let (verb, rest) = item.split_once(':').ok_or_else(|| format!("do: '{item}' is not verb:value"))?;
        let number = |t: &str| t.parse::<u64>().map_err(|_| format!("do: '{item}': '{t}' is not a whole number"));
        steps.push(match verb {
            "key" => json!({"step": "key", "key": rest}),
            "click" | "hover" => json!({"step": verb, "id": rest}),
            "screen" => json!({"step": "wait", "until": {"path": "screen", "eq": rest}}),
            "widget" => json!({"step": "wait", "until": {"widget": rest}}),
            "pause" => json!({"step": "pause", "ms": number(rest)?}),
            "shot" => json!({"step": "shot", "name": rest}),
            "clip" => {
                let mut parts = rest.split(':');
                let mut clip = json!({"step": "clip", "name": parts.next().unwrap_or("")});
                if let Some(frames) = parts.next() {
                    clip["frames"] = json!(number(frames)?);
                }
                if let Some(every) = parts.next() {
                    clip["every_ms"] = json!(number(every)?);
                }
                clip
            }
            _ => return Err(format!("do: '{verb}' is not key, click, hover, screen, widget, pause, shot or clip")),
        });
    }
    if !steps.iter().any(|s| s["step"] == json!("shot") || s["step"] == json!("clip")) {
        steps.push(json!({"step": "shot", "name": "shot"}));
    }
    Ok(steps)
}

/// The scenario `shots` runs: named `name`, launched as the keys say (`scene=`, `rules=`, or their old names
/// `encounter=`, `ruleset=`, `LAUNCH_ALIASES`).
pub fn shots_scenario(name: &str, args: &Args) -> std::result::Result<Scenario, Vec<String>> {
    let steps = do_steps(&args.key("do", "")).map_err(|e| vec![e])?;
    let mut launch = json!({"screen": args.key("screen", "menu")});
    for k in LAUNCH_ALIASES.iter().flat_map(|(k, old)| [*k, *old]) {
        if let Some(v) = args.keys.get(k) {
            launch[k] = json!(v);
        }
    }
    if let Some(seed) = args.keys.get("seed") {
        launch["seed"] = json!(seed.parse::<u64>().map_err(|_| vec![format!("seed '{seed}' is not a whole number")])?);
    }
    parse(name, &json!({"id": name, "title": "Proof shots", "launch": launch, "steps": steps}))
}

/// `aegis scenario shots`: a named fight or screen, driven and photographed with no scenario file. The bundle is a
/// scenario's; the pictures are listed.
fn shots(folder: &Path, name: &str, args: &Args) -> Result<i32> {
    let options = options_of(folder, args, ".agent/shots")?;
    let s = match shots_scenario(name, args) {
        Ok(s) => s,
        Err(problems) => {
            problems.iter().for_each(|p| println!("§ {p}"));
            return Ok(2);
        }
    };
    let mut launcher = GameLauncher { slow: u64::from(options.slow) };
    let mut worst = 0;
    for r in run_scenario(&s, &options, &mut launcher) {
        print_outcome(&r);
        worst = worst.max(r.code);
        for f in files_with(&r.bundle, "png") {
            println!("§ shot {}", f.display());
        }
        for clip in std::fs::read_dir(&r.bundle).map(|d| d.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect::<Vec<_>>()).unwrap_or_default() {
            println!("§ clip {}: {} frames", clip.display(), files_with(&clip, "png").len());
        }
    }
    Ok(worst)
}

/// The files of a folder with an extension, by name.
fn files_with(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.is_file() && p.extension().is_some_and(|x| x == ext)).collect();
    files.sort();
    files
}

fn print_outcome(r: &Outcome) {
    let size = format!("{}x{}", r.size.0, r.size.1);
    match r.code {
        0 => println!("§ {} {size} PASS; bundle {}", r.id, r.bundle.display()),
        1 => println!("§ {} {size} FAIL at step {}: {}; bundle {}", r.id, r.step.as_deref().unwrap_or("?"), r.message, r.bundle.display()),
        _ => println!("§ {} {size} COULD NOT START: {}; bundle {}", r.id, r.message, r.bundle.display()),
    }
}

fn run(folder: &Path, scenarios: &Path, what: &str, args: &Args) -> Result<i32> {
    let options = options_of(folder, args, ".agent/scenario")?;
    let slow = options.slow;
    let files = match what {
        "all" => files_in(scenarios),
        path if path.ends_with(".json") => vec![PathBuf::from(path)],
        id => vec![scenarios.join(format!("{id}.json"))],
    };
    let mut worst = 0;
    let mut loaded = Vec::new();
    for f in &files {
        match load(f) {
            Ok(s) => loaded.push(s),
            Err(problems) => {
                problems.iter().for_each(|p| println!("§ {p}"));
                worst = 2;
            }
        }
    }
    if worst != 0 {
        println!("§ scenario run: nothing ran: {} file(s) have problems", files.len());
        return Ok(worst);
    }
    let mut launcher = GameLauncher { slow: u64::from(slow) };
    let (mut passed, mut failed) = (0, 0);
    for s in &loaded {
        if let Status::Waiting(why) = &s.status {
            println!("§ {} SKIP: waiting: {why}", s.id);
            continue;
        }
        for r in run_scenario(s, &options, &mut launcher) {
            print_outcome(&r);
            if r.code == 0 {
                passed += 1;
            } else {
                failed += 1;
            }
            worst = worst.max(r.code);
        }
    }
    println!("§ scenario run: {passed} passed, {failed} failed");
    Ok(worst)
}
