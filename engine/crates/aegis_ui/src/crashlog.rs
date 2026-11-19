//! ROBUST1 (ENGINE-10): the log file and the crash report of a windowed run (native only; the web has no files and a
//! run with no window has its own bundle). `install` opens `<dir>/game.log` (the last run's kept as `game.1.log`) and
//! sets a panic hook that writes `<dir>/crash-<unix seconds>.txt`: the message, where, the backtrace and the last log
//! lines, then lets the panic go on as before. `line` adds to the log from anywhere; with no log open it does nothing.
//! The shell installs it beside the player's settings (`<user dir>/logs/`) and logs screen changes and notices.

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
    let first = LOG.lock().map(|mut g| g.replace(Log { dir: dir.to_path_buf(), started: Instant::now(), recent: VecDeque::new(), title: title.to_string() }).is_none()).unwrap_or(false);
    if first {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let message = info.payload().downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| info.payload().downcast_ref::<String>().cloned()).unwrap_or_else(|| "(no message)".into());
            let place = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
            let _ = write_crash(&message, &place, &std::backtrace::Backtrace::force_capture().to_string());
            previous(info);
        }));
    }
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

/// The text of a crash report.
pub fn report(title: &str, message: &str, place: &str, backtrace: &str, recent: &[String]) -> String {
    let mut out = format!("{title} crashed.\n\nWhat: {message}\nWhere: {place}\nWhen: unix {}\n\nThe last {} log lines:\n", unix_secs(), recent.len());
    for l in recent {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str("\nBacktrace:\n");
    out.push_str(backtrace);
    out
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
    let mut path = dir.join(format!("crash-{}.txt", unix_secs()));
    let mut n = 1;
    while dev::exists(&path) {
        n += 1;
        path = dir.join(format!("crash-{}-{n}.txt", unix_secs()));
    }
    dev::write(&path, report(&title, message, place, backtrace, &recent)).ok()?;
    let _ = dev::append(dir.join(LOG_FILE), format!("crash: {message} at {place} ({})\n", path.display()));
    prune(&dir);
    Some(path)
}

/// Removes the oldest crash reports past `KEEP_CRASHES` (by name: the unix seconds sort them).
pub fn prune(dir: &Path) {
    let mut crashes: Vec<PathBuf> = dev::list_dir(dir).unwrap_or_default().into_iter().filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("crash-") && n.ends_with(".txt"))).collect();
    crashes.sort();
    while crashes.len() > KEEP_CRASHES {
        let _ = dev::remove_file(crashes.remove(0));
    }
}
