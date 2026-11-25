//! ENGINE-62 (CRASH-SAFE1): the sample game gets the engine's crash path with no code of its own: a forced panic in a
//! run with no window leaves a report beside the logs, and the next start shows the line with the report's path over
//! the menu. The game writes no rescue save (it has no `Context::rescue`), so none is offered. One test: the panic hook
//! is per process.

use aegis_core::clock::{Duration, Instant};
use aegis_core::scratch::Scratch;
use aegis_ui::screen::Settings;
use aegis_ui::shell::headless::Headless;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

fn start(settings: &Path, saves: &Path) -> (BufReader<TcpStream>, std::thread::JoinHandle<anyhow::Result<()>>) {
    let port = TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port();
    let args: Vec<(String, String)> = vec![("port".into(), port.to_string()), ("saves".into(), saves.display().to_string())];
    let (settings, game) = (settings.to_path_buf(), hello_game::game_dir(Path::new(env!("CARGO_MANIFEST_DIR"))));
    let thread = std::thread::spawn(move || {
        let h = Headless { size: (640, 360), fps: 30, dpr: 1.0, idle: Duration::from_secs(60), stepped: true };
        let (mut cx, stack, options) = hello_game::launch(&args, &game, Some(h))?;
        // the player's settings (and so the logs beside them) in the test's own folder
        cx.settings = Settings::load(settings, json!({}));
        aegis_ui::shell::run(cx, stack, options)
    });
    let by = Instant::wall() + Duration::from_secs(60);
    let stream = loop {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(s) => break s,
            Err(e) => {
                assert!(Instant::wall() < by, "no link: {e}");
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    };
    stream.set_read_timeout(Some(std::time::Duration::from_secs(60))).unwrap();
    (BufReader::new(stream), thread)
}

fn ask(link: &mut BufReader<TcpStream>, body: Value) -> Value {
    writeln!(link.get_mut(), "{body}").unwrap();
    let mut line = String::new();
    link.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn a_forced_panic_shows_the_crash_line_over_the_menu_at_the_next_start() {
    let t = Scratch::new("hello_crash");
    let (settings, saves, logs) = (t.path().join("settings.json"), t.path().join("saves"), t.path().join("logs"));
    let (mut link, thread) = start(&settings, &saves);
    assert_eq!(ask(&mut link, json!({"cmd": "ui_tree"}))["screen"], json!("main_menu"));
    writeln!(link.get_mut(), "{}", json!({"cmd": "panic", "message": "a forced panic in the sample game"})).unwrap();
    assert!(thread.join().is_err());
    let reports: Vec<PathBuf> = std::fs::read_dir(&logs).unwrap().flatten().map(|e| e.path()).filter(|p| p.file_name().unwrap().to_string_lossy().starts_with("crash-")).collect();
    assert_eq!(reports.len(), 1, "{reports:?}");
    let report = std::fs::read_to_string(&reports[0]).unwrap();
    assert!(report.contains("a forced panic in the sample game") && report.contains("Screen: main_menu"), "{report}");
    assert!(!aegis_save::exists_keyed(&saves, aegis_save::RESCUE_KEY), "no rescue without the game's own");
    let (mut link, thread) = start(&settings, &saves);
    let tree = ask(&mut link, json!({"cmd": "ui_tree"}));
    assert_eq!(tree["screen"], json!("crash_note"), "{tree}");
    assert!(tree.to_string().contains(aegis_ui::screen::CRASH_LINE), "{tree}");
    assert_eq!(ask(&mut link, json!({"cmd": "click", "id": "crash.ok"}))["ok"], json!(true));
    assert_eq!(ask(&mut link, json!({"cmd": "ui_tree"}))["screen"], json!("main_menu"));
    assert_eq!(ask(&mut link, json!({"cmd": "quit"}))["ok"], json!(true));
    thread.join().unwrap().unwrap();
}
