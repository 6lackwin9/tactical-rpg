//! ROBUST1 (ENGINE-10): the log file and the crash report of a windowed run (native only; the web has no files and a
//! run with no window has its own bundle). `install` opens `<dir>/game.log` (the last run's kept as `game.1.log`) and
//! sets a panic hook that writes `<dir>/crash-<unix seconds>.txt`: the message, where, the backtrace and the last log
//! lines, then lets the panic go on as before. `line` adds to the log from anywhere; with no log open it does nothing.
//! The shell installs it beside the player's settings (`<user dir>/logs/`) and logs screen changes and notices.
//! ENGINE-62 (CRASH-SAFE1): the report also names the build, the top screen and the tail of `feel.log`; a panic the
//! shell catches (a window's or a no-window run's) ends in `crashed`, which writes the report when no log is open and
//! leaves `crashed.json` beside it, so the next start shows one line with the report's path (`take_crashed`).

use aegis_core::clock::{Instant, SystemTime, UNIX_EPOCH};
use aegis_core::vfs::dev;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// The log's lines kept in memory for a crash report.
pub const RECENT: usize = 200;
/// Crash reports kept in the folder (the oldest go first).
pub const KEEP_CRASHES: usize = 5;
pub const LOG_FILE: &str = "game.log";
pub const OLD_LOG_FILE: &str = "game.1.log";

struct Log {
    dir: PathBuf,
    started: Instant,
    recent: VecDeque<String>,
    title: String,
}

static LOG: Mutex<Option<Log>> = Mutex::new(None);

/// ENGINE-62: the file a caught crash leaves beside the logs for the next start: {"report": path, "rescue": bool}.
pub const CRASHED_FILE: &str = "crashed.json";
/// ENGINE-62: the lines of `feel.log` a report ends with.
pub const FEEL_TAIL: usize = 40;

/// ENGINE-62: the panic being reported (message, place, backtrace) and the report written for it, if any.
struct Last {
    message: String,
    place: String,
    backtrace: String,
    report: Option<PathBuf>,
}

static LAST: Mutex<Option<Last>> = Mutex::new(None);
/// ENGINE-62: the top screen's id (`set_top`), for the report.
static TOP: Mutex<String> = Mutex::new(String::new());
static ARMED: std::sync::Once = std::sync::Once::new();

/// ENGINE-62: the top screen now (the shell sets it on every screen change).
pub fn set_top(id: &str) {
    if let Ok(mut t) = TOP.lock() {
        id.clone_into(&mut t);
    }
}

/// ENGINE-62: sets, once per process, the panic hook that notes the panic (for `crashed`) and, with a log open, writes
/// its report; then the hook before it runs as before.
pub fn arm() {
    ARMED.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let message = info.payload().downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| info.payload().downcast_ref::<String>().cloned()).unwrap_or_else(|| "(no message)".into());
            let place = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
            let backtrace = std::backtrace::Backtrace::force_capture().to_string();
            let report = write_crash(&message, &place, &backtrace);
            *LAST.lock().unwrap_or_else(|e| e.into_inner()) = Some(Last { message, place, backtrace, report });
            previous(info);
        }));
    });
}

fn unix_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Opens the log in `dir` for the game titled `title`, and sets the panic hook (once per process). Returns the log's
/// path.
pub fn install(dir: &Path, title: &str) -> std::io::Result<PathBuf> {
    dev::create_dir_all(dir)?;
    let log = dir.join(LOG_FILE);
    if dev::exists(&log) {
        let _ = dev::rename(&log, dir.join(OLD_LOG_FILE));
    }
    dev::write(&log, format!("# {title}: log of the run started at unix {}\n", unix_secs()))?;
    if let Ok(mut g) = LOG.lock() {
        *g = Some(Log { dir: dir.to_path_buf(), started: Instant::now(), recent: VecDeque::new(), title: title.to_string() });
    }
    arm();
    Ok(log)
}

/// A line for the log: the seconds since the run started, then `text`.
pub fn line(text: &str) {
    let Ok(mut g) = LOG.lock() else { return };
    let Some(log) = g.as_mut() else { return };
    let l = format!("[{:>9.3}] {}", log.started.elapsed().as_secs_f64(), text.replace('\n', " / "));
    let _ = dev::append(log.dir.join(LOG_FILE), format!("{l}\n"));
    if log.recent.len() == RECENT {
        log.recent.pop_front();
    }
    log.recent.push_back(l);
}

/// The open log's folder (None: no log open): where other run files go beside it (ENGINE-45: `feel.log`).
pub fn dir() -> Option<PathBuf> {
    LOG.lock().ok().and_then(|g| g.as_ref().map(|l| l.dir.clone()))
}

/// Adds `lines` to `name` in the log's folder; with no log open it does nothing.
pub fn append_beside(name: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    if let Some(d) = dir() {
        let _ = dev::append(d.join(name), lines.iter().map(|l| format!("{l}\n")).collect::<String>());
    }
}

/// The text of a crash report. ENGINE-62: with the build (dev or release, its version and commit: `AEGIS_COMMIT` at
/// build time) and the top screen (`set_top`).
pub fn report(title: &str, message: &str, place: &str, backtrace: &str, recent: &[String]) -> String {
    let build = if cfg!(debug_assertions) { "dev" } else { "release" };
    let commit = option_env!("AEGIS_COMMIT").unwrap_or("unknown");
    let top = TOP.lock().map(|t| t.clone()).unwrap_or_default();
    let mut out = format!(
        "{title} crashed.\n\nWhat: {message}\nWhere: {place}\nWhen: unix {}\nBuild: {build}, engine {}, commit {commit}\nScreen: {}\n\nThe last {} log lines:\n",
        unix_secs(),
        env!("CARGO_PKG_VERSION"),
        if top.is_empty() { "(none)" } else { &top },
        recent.len()
    );
    for l in recent {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str("\nBacktrace:\n");
    out.push_str(backtrace);
    out
}

/// ENGINE-62: the last `FEEL_TAIL` lines of `feel.log` in `dir`, under a heading (nothing when there is no such file).
fn feel_tail(dir: &Path) -> String {
    let Ok(text) = dev::read_to_string(dir.join(crate::feel::LOG_FILE)) else { return String::new() };
    let lines: Vec<&str> = text.lines().collect();
    let from = lines.len().saturating_sub(FEEL_TAIL);
    format!("\nThe last {} lines of {}:\n{}\n", lines.len() - from, crate::feel::LOG_FILE, lines[from..].join("\n"))
}

/// Writes a crash report into the log's folder and logs it; the oldest reports past `KEEP_CRASHES` are removed.
/// Returns its path, or None when no log is open.
pub fn write_crash(message: &str, place: &str, backtrace: &str) -> Option<PathBuf> {
    // The lock may be poisoned by the panic being reported: take the data anyway.
    let (dir, title, recent) = {
        let g = LOG.lock().unwrap_or_else(|e| e.into_inner());
        let log = g.as_ref()?;
        (log.dir.clone(), log.title.clone(), log.recent.iter().cloned().collect::<Vec<_>>())
    };
    write_crash_in(&dir, &title, message, place, backtrace, &recent)
}

/// Writes a crash report into `dir` (made when missing); its path.
fn write_crash_in(dir: &Path, title: &str, message: &str, place: &str, backtrace: &str, recent: &[String]) -> Option<PathBuf> {
    dev::create_dir_all(dir).ok()?;
    let mut path = dir.join(format!("crash-{}.txt", unix_secs()));
    let mut n = 1;
    while dev::exists(&path) {
        n += 1;
        path = dir.join(format!("crash-{}-{n}.txt", unix_secs()));
    }
    dev::write(&path, report(title, message, place, backtrace, recent) + &feel_tail(dir)).ok()?;
    if dev::exists(dir.join(LOG_FILE)) {
        let _ = dev::append(dir.join(LOG_FILE), format!("crash: {message} at {place} ({})\n", path.display()));
    }
    prune(dir);
    Some(path)
}

/// ENGINE-62 (CRASH-SAFE1): a panic the shell caught: the report (the hook's, or one written now into the open log's
/// folder, else `fallback`, titled `title`) and `crashed.json` beside it, which says whether a rescue save was written.
/// Returns the report's path (None: nothing could be written).
pub fn crashed(fallback: &Path, title: &str, rescue: bool) -> Option<PathBuf> {
    let last = LAST.lock().unwrap_or_else(|e| e.into_inner()).take();
    let (message, place, backtrace, written) = match last {
        Some(l) => (l.message, l.place, l.backtrace, l.report),
        None => ("(a panic the hook did not see)".to_string(), String::new(), String::new(), None),
    };
    let report = written.or_else(|| write_crash(&message, &place, &backtrace)).or_else(|| write_crash_in(fallback, title, &message, &place, &backtrace, &[]))?;
    let dir = report.parent().map(Path::to_path_buf).unwrap_or_else(|| fallback.to_path_buf());
    let note = serde_json::json!({"report": report.display().to_string(), "rescue": rescue, "message": message});
    dev::write(dir.join(CRASHED_FILE), serde_json::to_string_pretty(&note).unwrap_or_default()).ok()?;
    line(&format!("crash caught: {message}{}", if rescue { " (rescue save written)" } else { "" }));
    Some(report)
}

/// ENGINE-62: what the last run's caught crash left in `dir` (`crashed`): (the report's path, whether a rescue save was
/// written); the file is removed, so the line shows once.
pub fn take_crashed(dir: &Path) -> Option<(PathBuf, bool)> {
    let path = dir.join(CRASHED_FILE);
    let text = dev::read_to_string(&path).ok()?;
    let _ = dev::remove_file(&path);
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some((PathBuf::from(v["report"].as_str()?), v["rescue"].as_bool().unwrap_or(false)))
}

/// Removes the oldest crash reports past `KEEP_CRASHES` (by name: the unix seconds sort them).
pub fn prune(dir: &Path) {
    let mut crashes: Vec<PathBuf> = dev::list_dir(dir).unwrap_or_default().into_iter().filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("crash-") && n.ends_with(".txt"))).collect();
    crashes.sort();
    while crashes.len() > KEEP_CRASHES {
        let _ = dev::remove_file(crashes.remove(0));
    }
}
