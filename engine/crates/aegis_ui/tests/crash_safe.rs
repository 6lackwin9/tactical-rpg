//! ENGINE-62 (CRASH-SAFE1): a panic in a run with no window, over the real agent link. The shell has the game write its
//! rescue save (`Context::rescue`) and leaves the report and `crashed.json` beside the logs; the next start shows the
//! one line over its first screen. One test: the panic hook and the crash notes are per process.

use aegis_core::scratch::Scratch;
use aegis_ui::art::Art;
use aegis_ui::audio::Audio;
use aegis_ui::canvas::Rect;
use aegis_ui::playtest::Playtests;
use aegis_ui::screen::{Context, Cx, Screen, Settings};
use aegis_ui::shell::headless::Headless;
use aegis_ui::shell::{run, Options};
use aegis_ui::ui::{Gfx, Ui};
use aegis_core::clock::{Duration, Instant};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

const FORMAT: aegis_save::Format = aegis_save::Format { tag: "crash_test", version: 1, first_slot: 0, last_slot: 1, keys: &["steps"] };

struct App {
    saves: PathBuf,
    steps: i64,
}

impl Context for App {
    const MORE_COMMANDS: &'static str = "none";
    fn note(&mut self, _line: String) {}
    fn agent_summary(&self) -> Value {
        Value::Null
    }
    fn agent_game(&self) -> Value {
        Value::Null
    }
    fn rescue(&self) -> Option<Vec<String>> {
        Some(aegis_save::write_keyed(&FORMAT, &self.saves, aegis_save::RESCUE_KEY, 0, || Ok(json!({"steps": self.steps}))))
    }
}

struct Title {
    w: Vec<aegis_ui::ui::Widget<()>>,
}

impl Title {
    fn act(&mut self, _cx: &mut Cx<App>, _a: ()) {}
}

impl Screen<App> for Title {
    fn id(&self) -> &'static str {
        "title"
    }
    fn draw(&mut self, _cx: &mut Cx<App>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<()> = Ui::new(g, mouse, live);
        ui.button("title.start", Rect::new(40, 40, 80, 16), "Start", (), true);
        self.w = ui.widgets;
    }
    aegis_ui::widget_plumbing!(Cx<App>);
}

fn context(settings: &Path, saves: &Path) -> Cx<App> {
    Cx {
        settings: Settings::load(settings.to_path_buf(), json!({})),
        art: Art::new(None),
        audio: Audio::offline(None),
        nav: Vec::new(),
        notice: Vec::new(),
        shot_dir: std::env::temp_dir(),
        quit: false,
        answers: Vec::new(),
        playtest: Playtests::off(),
        held: Default::default(),
        app: App { saves: saves.to_path_buf(), steps: 7 },
    }
}

fn fresh_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port()
}

/// Starts a run with no window on a thread; its link and the thread.
fn start(settings: &Path, saves: &Path) -> (BufReader<TcpStream>, std::thread::JoinHandle<anyhow::Result<()>>) {
    let port = fresh_port();
    let options = Options {
        title: "Crash Test".into(),
        font: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf"),
        port,
        icon: String::new(),
        canvas: Value::Null,
        window_sizes: Value::Null,
        headless: Some(Headless { size: (640, 360), fps: 30, dpr: 1.0, idle: Duration::from_secs(60), stepped: true }),
    };
    let (settings, saves) = (settings.to_path_buf(), saves.to_path_buf());
    let thread = std::thread::spawn(move || run(context(&settings, &saves), vec![Box::new(Title { w: Vec::new() }) as Box<dyn Screen<App>>], options));
    let by = Instant::wall() + Duration::from_secs(30);
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
fn a_panic_leaves_a_report_and_a_rescue_save_and_the_next_start_says_where_the_report_is() {
    let t = Scratch::new("crash_safe");
    let (settings, saves, logs) = (t.path().join("settings.json"), t.path().join("saves"), t.path().join("logs"));
    // A run that panics: the thread ends with the panic, after the shell's rescue and report.
    let (mut link, thread) = start(&settings, &saves);
    assert_eq!(ask(&mut link, json!({"cmd": "ui_tree"}))["screen"], json!("title"));
    writeln!(link.get_mut(), "{}", json!({"cmd": "panic", "message": "the board has no tile 9,9"})).unwrap();
    assert!(thread.join().is_err(), "the panic goes on after the rescue");
    let reports: Vec<PathBuf> = std::fs::read_dir(&logs).unwrap().flatten().map(|e| e.path()).filter(|p| p.file_name().unwrap().to_string_lossy().starts_with("crash-")).collect();
    assert_eq!(reports.len(), 1, "{reports:?}");
    let report = std::fs::read_to_string(&reports[0]).unwrap();
    for want in ["Crash Test crashed.", "What: the board has no tile 9,9", "Build: dev", "Screen: title", "Backtrace:"] {
        assert!(report.contains(want), "{want} in {report}");
    }
    assert!(logs.join(aegis_ui::crashlog::CRASHED_FILE).is_file());
    // The rescue save: a keyed save beside the slots, never a slot.
    assert!(aegis_save::exists_keyed(&saves, aegis_save::RESCUE_KEY));
    assert!(!aegis_save::exists(&saves, 0) && !aegis_save::exists(&saves, 1));
    let rescued = aegis_save::open(&FORMAT, &aegis_save::read_keyed(&saves, aegis_save::RESCUE_KEY).unwrap()).unwrap();
    assert_eq!(rescued["game_state"]["steps"], json!(7));
    // The next start: the line over the first screen, once; OK closes it.
    let (mut link, thread) = start(&settings, &saves);
    let tree = ask(&mut link, json!({"cmd": "ui_tree"}));
    assert_eq!(tree["screen"], json!("crash_note"), "{tree}");
    let text = tree.to_string();
    assert!(text.contains("The game closed unexpectedly. A report is at") && text.contains(&reports[0].file_name().unwrap().to_string_lossy().to_string()), "{text}");
    assert!(text.contains("crash.open") && text.contains("Open folder"), "{text}");
    assert!(!logs.join(aegis_ui::crashlog::CRASHED_FILE).exists(), "the line shows once");
    assert_eq!(ask(&mut link, json!({"cmd": "click", "id": "crash.ok"}))["ok"], json!(true));
    assert_eq!(ask(&mut link, json!({"cmd": "ui_tree"}))["screen"], json!("title"));
    assert_eq!(ask(&mut link, json!({"cmd": "quit"}))["ok"], json!(true));
    thread.join().unwrap().unwrap();
}
