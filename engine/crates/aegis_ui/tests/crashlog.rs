//! ROBUST1 (ENGINE-10): the run's log and the crash report. One test: the log and the panic hook are per process.

use aegis_core::scratch::Scratch;
use aegis_ui::crashlog;

#[test]
fn the_log_keeps_the_last_run_and_a_panic_writes_a_report_with_the_last_lines() {
    let t = Scratch::new("crashlog");
    let dir = t.path().join("logs");
    crashlog::line("before any log: dropped");
    let log = crashlog::install(&dir, "Test Game").unwrap();
    crashlog::line("screen main_menu (new stack)");
    crashlog::line("a line\nwith a break");
    let text = std::fs::read_to_string(&log).unwrap();
    assert!(text.starts_with("# Test Game: log of the run"), "{text}");
    assert!(text.contains("screen main_menu") && text.contains("a line / with a break") && !text.contains("dropped"), "{text}");
    // A second start keeps the first run's log as game.1.log.
    crashlog::install(&dir, "Test Game").unwrap();
    assert!(std::fs::read_to_string(dir.join(crashlog::OLD_LOG_FILE)).unwrap().contains("screen main_menu"));
    crashlog::line("screen battle (pushed)");
    let caught = std::panic::catch_unwind(|| panic!("the board has no tile 9,9"));
    assert!(caught.is_err());
    let crashes: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.path()).filter(|p| p.file_name().unwrap().to_string_lossy().starts_with("crash-")).collect();
    assert_eq!(crashes.len(), 1, "{crashes:?}");
    let report = std::fs::read_to_string(&crashes[0]).unwrap();
    for want in ["Test Game crashed.", "What: the board has no tile 9,9", "Where: ", "crashlog.rs", "screen battle (pushed)", "Backtrace:"] {
        assert!(report.contains(want), "{want} in {report}");
    }
    assert!(std::fs::read_to_string(dir.join(crashlog::LOG_FILE)).unwrap().contains("crash: the board has no tile 9,9"));
    // Old reports beyond the keep count go, oldest first.
    for i in 0..8 {
        std::fs::write(dir.join(format!("crash-100000000{i}.txt")), "old").unwrap();
    }
    crashlog::prune(&dir);
    let left = std::fs::read_dir(&dir).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().starts_with("crash-")).count();
    assert_eq!(left, crashlog::KEEP_CRASHES);
    assert!(!dir.join("crash-1000000000.txt").exists(), "the oldest went first");
}
