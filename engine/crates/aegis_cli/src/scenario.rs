//! `aegis scenario` (INTEG1-E1, docs/specs/AGENTROLES1.md 4; authoring: engine/docs/authoring/scenarios.md): JSON files walk a
//! game's real screens over the agent link and assert state, with no window by default (HEADLESS1), or in a real muted
//! window that does not take the focus when the file or `window=1` asks.
//!   aegis scenario check [<file> | all]                                       parse the files; no game starts
//!   aegis scenario run <id | all | file.json> [size=WxH] [out=<dir>] [slow=N] [exe=<game exe>] [data=<data dir>] [window=1] [fps=N] [dpr=X] [set=<key>:<value>,...] [audit=1]
//!   aegis scenario shots <name> [screen=] [scene=] [ruleset=] [seed=] [save=] [do=<a>,<b>,...] [the same keys as run]
//! A scenario is named by where it lies: `<game folder>/scenarios/<id>.json`. The game folder is the one holding the game
//! manifest the registered game names, and the game's program is the file named as the manifest's id beside this one
//! (`exe=` says otherwise). Exit codes: 0 pass, 1 a step or a file failed, 2 could not start. `tools/fc.sh gate` runs
//! every scenario with no window (PR7-4); a golden `shot` compares with its baseline (`crate::golden`), `record=1` writes it.

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
/// The latest a shot's `t` may ask for, in seconds after its screen opened (ENGINE-17).
pub const SHOT_T_MAX_S: f64 = 60.0;
/// Frames a `wait` draws between two looks in a run with no window (ENGINE-17): on the shot clock time moves only by
/// the frames asked for, so a wait asks for them, a fixed number each look, and the run is the same every time.
pub const WAIT_FRAMES: u64 = 1;
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
/// What a `perf` request may take per frame, on top of `REPLY`.
const PERF_FRAME_WAIT: Duration = Duration::from_millis(500);
/// What a frame takes at least, for the check against the run's deadline.
const PERF_FRAME_FLOOR: Duration = Duration::from_millis(5);
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

/// How the game starts (`launch` in the file): `screen` is any screen id the game opens on (ENGINE-5: the game's list,
/// `Game::launch_screens`, checked by `check`, `run` and `shots`; `menu` when the file names none), `scene` the game's
/// id of what the screen opens on, `ruleset` the game's id of the ruleset it plays by; both reach the game as the launch
/// keys `scene=` and `ruleset=`, and the game says which screens take them (`Game::launch_problems`).
#[derive(Debug, Clone, PartialEq)]
pub struct Launch {
    pub screen: String,
    pub scene: Option<String>,
    pub seed: Option<u64>,
    pub ruleset: Option<String>,
    /// PR7-4: a saves folder the game starts with, `<game folder>/scenarios/saves/<save>/` copied into the scratch saves.
    pub save: Option<String>,
}

/// LAYER-ENG-KEYS (PR5-29a): the launch keys' primary names and the other names they load under (in a file and, except
/// `RULES_ARG`, as `shots` keys); a launch giving both names of one key is refused. PK2-FOLLOWUP (PR5-36, FB-223 (a)):
/// `ruleset` is the primary name again (a ruleset is not a genre word) and `rules`, PR5-29a's name for it, only loads
/// from a scenario file.
pub const LAUNCH_ALIASES: [(&str, &str); 2] = [("scene", "encounter"), ("ruleset", "rules")];

/// The command-line key that is never a launch alias: on every binary `rules=<-id,+id>` adjusts a ruleset's rules, so
/// `shots` refuses it as a launch key rather than read it as the ruleset id (FB-223).
pub const RULES_ARG: &str = "rules";

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
    /// `golden`: compared against the committed baseline (`crate::golden`), with this tolerance.
    /// `t` (ENGINE-17): the shot waits until the top screen has been open that long, in ms, on the shot clock (0: at once).
    Shot { name: String, max_w: u64, golden: Option<crate::golden::Tolerance>, t_ms: u64 },
    Clip { name: String, frames: u64, every_ms: u64, max_w: u64 },
    /// ENGINE-2: frame times with no window (`perf` link request); `budget` fails the step when a number is over it on a
    /// steady machine.
    Perf { name: String, frames: u64, warm: u64, budget: Budget },
    /// ENGINE-45 (FEEL-PROBE1): replays `script` (keys, clicks, pad buttons, frames) with no window and reports input
    /// and frame times (`feel` link request); `input_ms` / `frame_ms` fail the step when the worst passes them.
    Feel { name: String, script: Value, gap: u64, input_ms: Option<f64>, frame_ms: Option<f64> },
    Pause { ms: u64 },
    Audit { scales: Vec<f64>, fail: Vec<String> },
    Repeat { times: u64, steps: Vec<Step>, until: Option<Cond> },
}

/// A `perf` step's limits in ms (None: not checked). A step with none records its numbers only.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Budget {
    pub p50: Option<f64>,
    pub p95: Option<f64>,
    pub max: Option<f64>,
}

impl Budget {
    pub fn is_empty(&self) -> bool {
        self.p50.is_none() && self.p95.is_none() && self.max.is_none()
    }

    /// Each number of `stats` (a `perf` reply's) over its limit, as "p95 31.2 ms over 25 ms".
    pub fn misses(&self, stats: &Value) -> Vec<String> {
        [("p50", self.p50), ("p95", self.p95), ("max", self.max)]
            .into_iter()
            .filter_map(|(k, limit)| {
                let (limit, got) = (limit?, stats[k].as_f64()?);
                (got > limit).then(|| format!("{k} {got} ms over {limit} ms"))
            })
            .collect()
    }
}

/// What a `perf` step's numbers mean: no budget, within it, over it, or none because the machine's load changed.
pub fn perf_verdict(budget: &Budget, reply: &Value) -> (&'static str, Vec<String>) {
    if budget.is_empty() {
        return ("recorded", Vec::new());
    }
    if reply["steady"] != json!(true) {
        return ("no verdict", Vec::new());
    }
    let misses = budget.misses(&reply["stats"]);
    (if misses.is_empty() { "within" } else { "over" }, misses)
}

/// A `perf` step's bounds (the game's own are the same: `aegis_ui::shell::headless::PERF_*`) and the most a budget says.
pub const PERF_FRAMES_MAX: u64 = 2000;
pub const PERF_WARM_MAX: u64 = 600;
pub const PERF_BUDGET_MS_MAX: f64 = 10_000.0;

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
            keys(&["name", "max_w", "golden", "t"], problems);
            let name = text_of(o, "name", place, problems);
            // ENGINE-17: `"t": <s>`, seconds after the top screen opened (0 to SHOT_T_MAX_S; 0, the default, is at once).
            let t_ms = match o.get("t") {
                None => 0,
                Some(v) => match v.as_f64().filter(|t| (0.0..=SHOT_T_MAX_S).contains(t)) {
                    Some(t) => (t * 1000.0).round() as u64,
                    None => {
                        problems.push(format!("{place}: 't' is a number of seconds from 0 to {SHOT_T_MAX_S}"));
                        0
                    }
                },
            };
            if !name.is_empty() && !plain_name(&name) {
                problems.push(format!("{place}: 'name' becomes a file: letters, digits, _ and - only, at most 80"));
            }
            // PR7-4: `"golden": true` (exact) or `{"tolerance": 0..255, "pixels": N}`.
            let golden = match o.get("golden") {
                None | Some(Value::Bool(false)) => None,
                Some(Value::Bool(true)) => Some(crate::golden::Tolerance::default()),
                Some(Value::Object(g)) => {
                    unknown_keys(g, &["tolerance", "pixels"], &format!("{place}.golden"), problems);
                    let channel = int_of(g, "tolerance", Some(0), (0, 255), &format!("{place}.golden"), problems) as u8;
                    let pixels = int_of(g, "pixels", Some(0), (0, u64::from(SIZE_MAX.0) * u64::from(SIZE_MAX.1)), &format!("{place}.golden"), problems);
                    Some(crate::golden::Tolerance { channel, pixels })
                }
                Some(_) => {
                    problems.push(format!("{place}: 'golden' is true or {{\"tolerance\", \"pixels\"}}"));
                    None
                }
            };
            Step::Shot { name, max_w: int_of(o, "max_w", Some(1920), (1, u64::from(SIZE_MAX.0)), place, problems), golden, t_ms }
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
            keys(&["name", "frames", "warm", "budget"], problems);
            let name = text_of(o, "name", place, problems);
            if !name.is_empty() && !plain_name(&name) {
                problems.push(format!("{place}: 'name' becomes a file name: letters, digits, _ and - only, at most 80"));
            }
            let frames = int_of(o, "frames", Some(300), (1, PERF_FRAMES_MAX), place, problems);
            let warm = int_of(o, "warm", Some(60), (0, PERF_WARM_MAX), place, problems);
            let mut budget = Budget::default();
            match o.get("budget") {
                None => {}
                Some(Value::Object(b)) => {
                    for (k, v) in b {
                        let slot = match k.as_str() {
                            "p50_ms" => &mut budget.p50,
                            "p95_ms" => &mut budget.p95,
                            "max_ms" => &mut budget.max,
                            other => {
                                problems.push(format!("{place}.budget: unknown key '{other}' (p50_ms, p95_ms, max_ms)"));
                                continue;
                            }
                        };
                        match v.as_f64().filter(|n| *n > 0.0 && *n <= PERF_BUDGET_MS_MAX) {
                            Some(n) => *slot = Some(n),
                            None => problems.push(format!("{place}.budget: '{k}' is a number of ms above 0, at most {PERF_BUDGET_MS_MAX}")),
                        }
                    }
                    if budget.is_empty() && b.is_empty() {
                        problems.push(format!("{place}.budget: name at least one of p50_ms, p95_ms, max_ms"));
                    }
                }
                Some(_) => problems.push(format!("{place}: 'budget' is an object (p50_ms, p95_ms, max_ms)")),
            }
            Step::Perf { name, frames, warm, budget }
        }
        "feel" => {
            keys(&["name", "script", "gap", "budget"], problems);
            let name = text_of(o, "name", place, problems);
            if !name.is_empty() && !plain_name(&name) {
                problems.push(format!("{place}: 'name' becomes a file name: letters, digits, _ and - only, at most 80"));
            }
            let gap = int_of(o, "gap", Some(2), (0, 240), place, problems);
            let script = o.get("script").cloned().unwrap_or(Value::Null);
            match script.as_array() {
                Some(list) if !list.is_empty() && list.len() as u64 <= STEPS_MAX => {
                    for (i, item) in list.iter().enumerate() {
                        let at = format!("{place}.script[{i}]");
                        let Some(m) = item.as_object() else {
                            problems.push(format!("{at}: an object: one of key, click, pad or frames, and times"));
                            continue;
                        };
                        let named: Vec<&str> = ["key", "click", "pad", "frames"].into_iter().filter(|k| m.contains_key(*k)).collect();
                        if named.len() != 1 {
                            problems.push(format!("{at}: exactly one of key, click, pad or frames"));
                        }
                        for (k, v) in m {
                            let fine = match k.as_str() {
                                "key" | "click" | "pad" => v.as_str().is_some_and(|t| !t.is_empty()),
                                "frames" => v.as_u64().is_some_and(|n| (1..=240).contains(&n)),
                                "times" => v.as_u64().is_some_and(|n| (1..=200).contains(&n)),
                                _ => {
                                    problems.push(format!("{at}: unknown key '{k}' (key, click, pad, frames, times)"));
                                    true
                                }
                            };
                            if !fine {
                                problems.push(format!("{at}: '{k}' is {}", if k == "frames" { "a whole number from 1 to 240" } else if k == "times" { "a whole number from 1 to 200" } else { "a name" }));
                            }
                        }
                    }
                }
                _ => problems.push(format!("{place}: 'script' is a list of 1 to {STEPS_MAX} inputs")),
            }
            let (mut input_ms, mut frame_ms) = (None, None);
            match o.get("budget") {
                None => {}
                Some(Value::Object(b)) => {
                    for (k, v) in b {
                        let slot = match k.as_str() {
                            "input_ms" => &mut input_ms,
                            "frame_ms" => &mut frame_ms,
                            other => {
                                problems.push(format!("{place}.budget: unknown key '{other}' (input_ms, frame_ms)"));
                                continue;
                            }
                        };
                        match v.as_f64().filter(|n| *n > 0.0 && *n <= PERF_BUDGET_MS_MAX) {
                            Some(n) => *slot = Some(n),
                            None => problems.push(format!("{place}.budget: '{k}' is a number of ms above 0, at most {PERF_BUDGET_MS_MAX}")),
                        }
                    }
                }
                Some(_) => problems.push(format!("{place}: 'budget' is an object (input_ms, frame_ms)")),
            }
            Step::Feel { name, script, gap, input_ms, frame_ms }
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

/// Whether a step needs a run with no window (`clip`, `perf`), and which it is.
fn no_window_step(steps: &[Step]) -> Option<&'static str> {
    steps.iter().find_map(|s| match s {
        Step::Clip { .. } => Some("clip"),
        Step::Perf { .. } => Some("perf"),
        Step::Feel { .. } => Some("feel"),
        Step::Repeat { steps, .. } => no_window_step(steps),
        _ => None,
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
    let mut launch = Launch { screen: "menu".into(), scene: None, seed: None, ruleset: None, save: None };
    if let Some(l) = o.get("launch") {
        match l.as_object() {
            None => p.push("launch: an object".into()),
            Some(l) => {
                let known: Vec<&str> = ["screen", "seed", "save"].into_iter().chain(LAUNCH_ALIASES.iter().flat_map(|(k, old)| [*k, *old])).collect();
                unknown_keys(l, &known, "launch", p);
                match l.get("screen").map(|s| s.as_str()) {
                    None => {}
                    Some(Some(s)) if plain_name(s) => launch.screen = s.to_string(),
                    Some(_) => p.push("launch: 'screen' is a screen id: letters, digits, _ and - only".into()),
                }
                launch.scene = aliased_text(l, "scene", p);
                if l.contains_key("seed") {
                    launch.seed = Some(int_of(l, "seed", None, (0, u64::MAX), "launch", p));
                }
                launch.ruleset = aliased_text(l, "ruleset", p);
                if l.contains_key("save") {
                    let name = text_of(l, "save", "launch", p);
                    if !name.is_empty() && !plain_name(&name) && !Path::new(&name).is_absolute() {
                        p.push("launch: 'save' names a folder under scenarios/saves/ (letters, digits, _ and - only) or is an absolute folder path".into());
                    }
                    launch.save = Some(name);
                }
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
    if let Some(kind) = no_window_step(&steps).filter(|_| window) {
        p.push(format!("scenario: a '{kind}' is taken in a run with no window, and this one has 'window' true"));
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

/// The launch's problems for a game that opens on `screens` (ENGINE-5; `Game::launch_problems`' default): the screen is
/// one of them.
pub fn screen_problems(launch: &Launch, screens: &[&str]) -> Vec<String> {
    if screens.contains(&launch.screen.as_str()) {
        Vec::new()
    } else {
        vec![format!("launch: 'screen' {} is not one this game opens on ({})", launch.screen, screens.join(", "))]
    }
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
    if let Some(r) = &s.launch.ruleset {
        args.push(format!("ruleset={r}"));
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
    /// PR7-4: where the golden shots' baselines lie (`<game folder>/scenarios/baselines`), and `record=1`: write them.
    pub baselines: Option<PathBuf>,
    pub record: bool,
    /// PR7-4: where `launch.save` folders lie (`<game folder>/scenarios/saves`).
    pub saves: Option<PathBuf>,
    /// ENGINE-33: `set=<key>:<value>,...`: settings laid over every scenario's own (a value is JSON, else a string), so
    /// one sweep runs every screen at, say, the largest text size; the driver's held settings and `mute` stay refused.
    pub set: Map<String, Value>,
    /// ENGINE-33: `audit=1`: after every shot, the text audit of that frame (at the run's text size) beside it as
    /// `<shot>.audit.json` (cut, edge, overlap and spill; `text_audit::issues`), for a sweep to list what a size breaks.
    pub audit: bool,
}

/// ENGINE-33: `set=` as settings: `key:value` pairs split on commas (a value parses as JSON, else it is a string); a
/// held setting, `mute`, an empty key or a pair without `:` is an error.
pub fn parse_set(list: &str) -> std::result::Result<Map<String, Value>, String> {
    let mut out = Map::new();
    for pair in list.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let Some((k, v)) = pair.split_once(':') else { return Err(format!("set: '{pair}' is key:value")) };
        let k = k.trim();
        if k.is_empty() {
            return Err(format!("set: '{pair}' has no key"));
        }
        if HELD_SETTINGS.contains(&k) || k == "mute" {
            return Err(format!("set: '{k}' belongs to the driver"));
        }
        let v = v.trim();
        out.insert(k.to_string(), serde_json::from_str(v).unwrap_or_else(|_| Value::String(v.to_string())));
    }
    Ok(out)
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
    /// This size's baselines folder, whether to record them, and the baselines written.
    golden_dir: Option<PathBuf>,
    record: bool,
    /// Shots to become baselines once every step has passed: (the shot, its baseline).
    recorded: Vec<(PathBuf, PathBuf)>,
    /// ENGINE-2: each `perf` step's numbers and verdict, for result.json.
    perf: Vec<Value>,
    /// ENGINE-33: `Options::audit`.
    audit: bool,
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

    /// A `frames` request to a game with no window, whose reply must say `ok`.
    fn frames(&mut self, step: &str, body: Value) -> std::result::Result<(), Failure> {
        let reply = self.ask(step, &body)?;
        if reply["ok"] != json!(true) {
            return Err(Failure { step: step.to_string(), message: format!("the frames asked for ({body}): {}", reply["error"]), gone: false });
        }
        Ok(())
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
                    if self.headless {
                        self.frames(label, json!({"cmd": "frames", "frames": WAIT_FRAMES}))?;
                    } else {
                        std::thread::sleep(POLL);
                    }
                }
            }
            Step::Pause { ms } if self.headless => {
                // The shot clock: the pause is that much game time, drawn at once.
                self.frames(label, json!({"cmd": "frames", "ms": ms}))?;
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
            Step::Shot { name, max_w, golden, t_ms } => {
                self.settle(label)?;
                if *t_ms > 0 {
                    if !self.headless {
                        return Err(Failure { step: label.to_string(), message: "a shot's t is taken in a run with no window (the shot clock)".to_string(), gone: false });
                    }
                    self.frames(label, json!({"cmd": "frames", "since_open_ms": t_ms}))?;
                }
                let path = self.bundle.join(format!("{name}.png"));
                let reply = self.ask(label, &json!({"cmd": "screenshot", "path": path.display().to_string(), "max_w": max_w}))?;
                if reply["ok"] != json!(true) {
                    return Err(Failure { step: label.to_string(), message: format!("the shot {name}: {}", reply["error"]), gone: false });
                }
                if self.audit {
                    // After the shot, so the picture is the plain one; the audit draws a frame of its own and puts it back.
                    let audit = self.ask(label, &json!({"cmd": "text_audit"}))?;
                    let _ = std::fs::write(self.bundle.join(format!("{name}.audit.json")), serde_json::to_string_pretty(&audit).unwrap_or_default());
                }
                if let Some(tol) = golden {
                    let fail = |message: String| Failure { step: label.to_string(), message, gone: false };
                    let dir = self.golden_dir.clone().ok_or_else(|| fail("a golden shot needs the game's baselines folder".into()))?;
                    let base = dir.join(format!("{name}.png"));
                    if self.record {
                        // Written only when the whole run passes: a baseline never comes from a failed run.
                        self.recorded.push((path.clone(), base));
                    } else {
                        match crate::golden::check(&path, &base, *tol, &self.bundle.join(format!("{name}.diff.png"))) {
                            Ok(None) => {}
                            Ok(Some(why)) => return Err(fail(format!("the golden shot {name}: {why}"))),
                            Err(why) => return Err(fail(format!("the golden shot {name}: {why}"))),
                        }
                    }
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
            Step::Perf { name, frames, warm, budget } => {
                let fail = |message: String| Failure { step: label.to_string(), message, gone: false };
                if !self.headless {
                    return Err(fail("frame times are measured in a run with no window".to_string()));
                }
                // A rough floor: every frame takes at least PERF_FRAME_FLOOR, so a request that cannot end in time is refused.
                if Instant::now() + PERF_FRAME_FLOOR * (frames + warm) as u32 > self.deadline {
                    return Err(fail(format!("perf {name}: {} frames run past the run's timeout_s", frames + warm)));
                }
                self.settle(label)?;
                // Back to back: a dev build can take a few hundred ms a frame on a big canvas.
                self.link.patience((REPLY + PERF_FRAME_WAIT * (frames + warm) as u32) * self.slow as u32);
                let reply = self.ask(label, &json!({"cmd": "perf", "frames": frames, "warm": warm}));
                self.link.patience(REPLY * self.slow as u32);
                let reply = reply?;
                if reply["ok"] != json!(true) {
                    return Err(fail(format!("perf {name}: {}", reply["error"])));
                }
                let (verdict, misses) = perf_verdict(budget, &reply);
                let mut kept = json!({"name": name, "step": label, "frames": frames, "warm": warm, "stats": reply["stats"], "calib_before": reply["calib_before"],
                    "calib_after": reply["calib_after"], "steady": reply["steady"], "canvas": reply["canvas"], "build": reply["build"], "verdict": verdict});
                if !budget.is_empty() {
                    kept["budget"] = json!({"p50_ms": budget.p50, "p95_ms": budget.p95, "max_ms": budget.max});
                }
                let _ = std::fs::write(self.bundle.join(format!("perf_{name}.json")), serde_json::to_string_pretty(&kept).unwrap_or_default());
                self.perf.push(kept);
                if !misses.is_empty() {
                    return Err(fail(format!("perf {name}: {} (p50 {} ms, p95 {} ms, max {} ms over {frames} frames, canvas {}x{})", misses.join(", "), reply["stats"]["p50"], reply["stats"]["p95"], reply["stats"]["max"], reply["canvas"][0], reply["canvas"][1])));
                }
            }
            Step::Feel { name, script, gap, input_ms, frame_ms } => {
                let fail = |message: String| Failure { step: label.to_string(), message, gone: false };
                if !self.headless {
                    return Err(fail("input times are measured in a run with no window".to_string()));
                }
                self.settle(label)?;
                let inputs = script.as_array().map_or(0, |l| l.iter().map(|i| i["times"].as_u64().unwrap_or(1) * (gap + 1).max(i["frames"].as_u64().unwrap_or(0))).sum::<u64>());
                self.link.patience((REPLY + PERF_FRAME_WAIT * inputs as u32) * self.slow as u32);
                let reply = self.ask(label, &json!({"cmd": "feel", "script": script, "gap": gap}));
                self.link.patience(REPLY * self.slow as u32);
                let reply = reply?;
                if reply["ok"] != json!(true) {
                    return Err(fail(format!("feel {name}: {}", reply["error"])));
                }
                let mut kept = reply.clone();
                if let Some(o) = kept.as_object_mut() {
                    o.remove("ok");
                    o.insert("name".into(), json!(name));
                    o.insert("step".into(), json!(label));
                    o.insert("kind".into(), json!("feel"));
                }
                let _ = std::fs::write(self.bundle.join(format!("feel_{name}.json")), serde_json::to_string_pretty(&kept).unwrap_or_default());
                self.perf.push(kept);
                let mut misses = Vec::new();
                if let Some(b) = input_ms.filter(|b| reply["to_present"]["worst"].as_f64().is_some_and(|w| w > *b)) {
                    misses.push(format!("worst input {} ms over {b} ms", reply["to_present"]["worst"]));
                }
                if let Some(b) = frame_ms.filter(|b| reply["frame_time"]["worst"].as_f64().is_some_and(|w| w > *b)) {
                    misses.push(format!("worst frame {} ms over {b} ms", reply["frame_time"]["worst"]));
                }
                if !misses.is_empty() {
                    return Err(fail(format!("feel {name}: {}", misses.join(", "))));
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
        Step::Perf { .. } => "perf",
        Step::Feel { .. } => "feel",
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
fn ready(link: &mut dyn Link, game: &mut dyn Process, want: (u32, u32), sized_ms: u64, slow: u64, headless: bool) -> std::result::Result<(Value, Value), StartFailure> {
    let poll = if headless { POLL_HEADLESS } else { POLL };
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
        if headless {
            // The shot clock (ENGINE-17): no frame comes unless asked for; the same frames in every run.
            let reply = link.ask(&json!({"cmd": "frames", "frames": SETTLE_FRAMES})).map_err(|e| fail(format!("the link: {e}"), false))?;
            if reply["ok"] != json!(true) {
                return Err(fail(format!("the frames asked for: {}", reply["error"]), false));
            }
        } else {
            std::thread::sleep(poll);
        }
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

/// A folder's files and folders, copied under `to`.
pub fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let target = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &target)?;
        } else {
            std::fs::copy(e.path(), target)?;
        }
    }
    Ok(())
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
        for (k, v) in &o.set {
            plan.settings[k] = v.clone();
        }
        if headless {
            plan.args.extend(headless_args(size, o));
            // ENGINE-17: a scenario is always on the shot clock, so two runs of a build draw the same frames (the agent
            // playtest, which shares the arguments above, stays on real time).
            plan.args.push("clock=step".to_string());
        }
        if let Err(e) = write_scratch(&plan) {
            out.message = format!("the scratch folder {}: {e}", scratch.display());
            break;
        }
        if let Some(name) = &s.launch.save {
            // ENGINE-36 (1): an absolute path is a save folder anywhere (a kept playtest save in project files)
            let from = if Path::new(name).is_absolute() { Some(PathBuf::from(name)) } else { o.saves.as_ref().map(|d| d.join(name)) };
            match from.filter(|f| f.is_dir()) {
                None => {
                    out.message = if Path::new(name).is_absolute() { format!("launch.save '{name}': no such folder") } else { format!("launch.save '{name}': no folder scenarios/saves/{name}/") };
                    let _ = std::fs::remove_dir_all(&scratch);
                    break;
                }
                Some(from) => {
                    if let Err(e) = copy_dir(&from, &scratch.join("saves")) {
                        out.message = format!("launch.save '{name}': {e}");
                        let _ = std::fs::remove_dir_all(&scratch);
                        break;
                    }
                }
            }
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
        let readiness = ready(link.as_mut(), game.as_mut(), size, o.sized_ms, slow, headless);
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
        let golden_dir = o.baselines.as_ref().map(|b| b.join(&s.id).join(format!("{}x{}", size.0, size.1)));
        let mut run = Run { link: link.as_mut(), game: game.as_mut(), bundle: &bundle, slow, deadline, done: Vec::new(), frame: 0, headless, golden_dir, record: o.record, recorded: Vec::new(), perf: Vec::new(), audit: o.audit };
        let walked = s.steps.iter().enumerate().try_for_each(|(i, step)| run.step(&(i + 1).to_string(), step));
        let done = std::mem::take(&mut run.done);
        let to_record = std::mem::take(&mut run.recorded);
        let perf = std::mem::take(&mut run.perf);
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
        if o.record && out.code == 0 {
            let mut written = Vec::new();
            for (shot, base) in &to_record {
                match base.parent().map(std::fs::create_dir_all).unwrap_or(Ok(())).and_then(|_| std::fs::copy(shot, base)) {
                    Ok(_) => written.push(base.display().to_string()),
                    Err(e) => {
                        out.code = 1;
                        out.step = Some("record".into());
                        out.message = format!("recording {}: {e}", base.display());
                        break;
                    }
                }
            }
            record["recorded"] = json!(written);
        }
        if !perf.is_empty() {
            record["perf"] = Value::Array(perf);
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

const RUN_USAGE: &str = "aegis scenario run <id | all | file.json> [size=WxH] [out=<dir>] [slow=N] [exe=<game exe>] [data=<data dir>] [window=1] [fps=N] [dpr=X] [record=1] [set=<key>:<value>,...] [audit=1]";
const SHOTS_USAGE: &str = "aegis scenario shots <name> [screen=<id>] [scene=<id>] [ruleset=<id>] [seed=N] [save=<name|/abs/dir>] [do=<a>,<b>,...] [size=WxH] [out=<dir>] [slow=N] [exe=<game exe>] [data=<data dir>] [window=1] [fps=N] [dpr=X]";

/// `aegis scenario ...`; returns the exit code.
pub fn command<G: crate::Game>(game: &G, args: &Args) -> Result<i32> {
    let folder = game_folder(game, args)?;
    let scenarios = folder.join("scenarios");
    let launch = |l: &Launch| game.launch_problems(l);
    match args.pos(1) {
        Some("check") => Ok(check(&scenarios, args.pos(2).unwrap_or("all"), &launch)),
        Some("run") => {
            let Some(what) = args.pos(2) else {
                println!("usage: {RUN_USAGE}");
                return Ok(2);
            };
            run(&folder, &scenarios, what, args, &launch)
        }
        Some("shots") => {
            let Some(name) = args.pos(2) else {
                println!("usage: {SHOTS_USAGE}");
                return Ok(2);
            };
            shots(&folder, name, args, &launch)
        }
        _ => {
            println!("usage: aegis scenario check [<file> | all]\n       {RUN_USAGE}\n       {SHOTS_USAGE}");
            Ok(2)
        }
    }
}

/// What the game says of a launch (`Game::launch_problems`).
type LaunchCheck<'a> = dyn Fn(&Launch) -> Vec<String> + 'a;

/// A loaded scenario's problems as the game sees its launch, each naming the scenario.
fn game_problems(s: &Scenario, launch: &LaunchCheck) -> Vec<String> {
    launch(&s.launch).into_iter().map(|p| format!("{}: {p}", s.id)).collect()
}

fn check(scenarios: &Path, what: &str, launch: &LaunchCheck) -> i32 {
    let files = if what == "all" { files_in(scenarios) } else { vec![PathBuf::from(what)] };
    let mut problems = check_files(&files);
    problems.extend(files.iter().filter_map(|f| load(f).ok()).flat_map(|s| game_problems(&s, launch)));
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
    let record = match args.key("record", "0").as_str() {
        "0" => false,
        "1" => true,
        other => anyhow::bail!("record is 1 or 0, not '{other}'"),
    };
    let audit = match args.key("audit", "0").as_str() {
        "0" => false,
        "1" => true,
        other => anyhow::bail!("audit is 1 or 0, not '{other}'"),
    };
    Ok(Options { game_id: manifest.id.clone(), exe, data: args.keys.get("data").map(PathBuf::from), out, slow, size, sized_ms: SIZED_MS, window, fps, dpr: args.keys.get("dpr").cloned(), baselines: Some(folder.join("scenarios").join("baselines")), record, saves: Some(folder.join("scenarios").join("saves")), set: args.keys.get("set").map(|l| parse_set(l)).transpose().map_err(anyhow::Error::msg)?.unwrap_or_default(), audit })
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

/// The scenario `shots` runs: named `name`, launched as the keys say (`scene=` or its old name `encounter=`, and
/// `ruleset=`, `LAUNCH_ALIASES`); `rules=` is refused (`RULES_ARG`: a rule adjustment, never the ruleset id).
pub fn shots_scenario(name: &str, args: &Args) -> std::result::Result<Scenario, Vec<String>> {
    if let Some(v) = args.keys.get(RULES_ARG) {
        return Err(vec![format!("shots: '{RULES_ARG}={v}' adjusts rules (+id,-id) and is no launch key; name the ruleset with 'ruleset=<id>'")]);
    }
    let steps = do_steps(&args.key("do", "")).map_err(|e| vec![e])?;
    let mut launch = json!({"screen": args.key("screen", "menu")});
    for k in LAUNCH_ALIASES.iter().flat_map(|(k, old)| [*k, *old]).filter(|k| *k != RULES_ARG) {
        if let Some(v) = args.keys.get(k) {
            launch[k] = json!(v);
        }
    }
    if let Some(seed) = args.keys.get("seed") {
        launch["seed"] = json!(seed.parse::<u64>().map_err(|_| vec![format!("seed '{seed}' is not a whole number")])?);
    }
    // ENGINE-36 (1): a named save (scenarios/saves/<name>) or a save folder's absolute path, so a map or hub proof shot
    // from a save needs no scenario file (do=click:<continue>,screen:<id>,shot:<name>)
    if let Some(save) = args.keys.get("save") {
        launch["save"] = json!(save);
    }
    parse(name, &json!({"id": name, "title": "Proof shots", "launch": launch, "steps": steps}))
}

/// `aegis scenario shots`: a named fight or screen, driven and photographed with no scenario file. The bundle is a
/// scenario's; the pictures are listed.
fn shots(folder: &Path, name: &str, args: &Args, launch: &LaunchCheck) -> Result<i32> {
    let options = options_of(folder, args, ".agent/shots")?;
    let s = match shots_scenario(name, args).and_then(|s| match game_problems(&s, launch) {
        p if p.is_empty() => Ok(s),
        p => Err(p),
    }) {
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

fn run(folder: &Path, scenarios: &Path, what: &str, args: &Args, launch: &LaunchCheck) -> Result<i32> {
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
        match load(f).and_then(|s| match game_problems(&s, launch) {
            p if p.is_empty() => Ok(s),
            p => Err(p),
        }) {
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
