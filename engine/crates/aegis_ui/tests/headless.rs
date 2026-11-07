//! HEADLESS1: the shell with no window (`aegis_ui::shell::headless`), driven over the real agent link on a real port. Nothing
//! here opens a window or a sound device. Frame counts are read as bounds: the upper ones are the rule (the pace holds), the
//! lower ones only say that frames come at all, so a loaded machine does not fail them.

use aegis_core::scratch::Scratch;
use aegis_ui::art::{load_png, Art};
use aegis_ui::audio::Audio;
use aegis_ui::canvas::{rgb, Canvas, Rect};
use aegis_ui::pacing::FrameNeed;
use aegis_ui::playtest::Playtests;
use aegis_ui::screen::{Context, Cx, Key, Screen, Settings};
use aegis_ui::shell::headless::{Headless, CLIP_FRAMES_MAX, DEFAULT_FPS, FPS_MIN, FRAMES_MAX, IDLE_EXIT_S};
use aegis_ui::shell::{blank_reason, run, save_png, Options, BLANK_TOLERANCE, SHOT_SIDE_MIN};
use aegis_ui::ui::{Gfx, Ui, TEXT};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const SIZE: (u32, u32) = (800, 450);

struct App;

impl Context for App {
    const MORE_COMMANDS: &'static str = "none";
    fn note(&mut self, _line: String) {}
    fn agent_summary(&self) -> Value {
        Value::Null
    }
    fn agent_game(&self) -> Value {
        Value::Null
    }
}

type AppCx = Cx<App>;

/// A screen with a known need and a clock on wall time that takes at most 100 ms a frame. `blank` draws nothing.
struct Probe {
    need: FrameNeed,
    blank: bool,
    clicks: u32,
    w: Vec<aegis_ui::ui::Widget<&'static str>>,
}

impl Probe {
    fn new(need: FrameNeed, blank: bool) -> Self {
        Self { need, blank, clicks: 0, w: Vec::new() }
    }
    fn act(&mut self, _cx: &mut AppCx, _a: &'static str) {
        self.clicks += 1;
    }
}

impl Screen<App> for Probe {
    fn id(&self) -> &'static str {
        "probe"
    }
    fn frame_need(&self, _cx: &AppCx) -> FrameNeed {
        self.need
    }
    fn tick(&mut self, _cx: &mut AppCx) {}
    fn draw(&mut self, _cx: &mut AppCx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        if !self.blank {
            let mut ui: Ui<&'static str> = Ui::new(g, mouse, live);
            ui.row("go", Rect::new(10, 10, 60, 14), "Go", TEXT, Some("go"), false, Vec::new());
            self.w = ui.widgets;
        }
    }
    /// A window size picked in the game's settings.
    fn key(&mut self, cx: &mut AppCx, k: Key) {
        if k == Key::Char('s') {
            cx.settings.v["window_size"] = json!("960x540");
        }
    }
    /// A command of the screen's own, under the name a game's battle uses.
    fn command(&mut self, _cx: &AppCx, body: &Value) -> Option<Value> {
        (body["cmd"] == "step").then(|| {
            self.clicks += 100;
            json!({"ok": true, "stepped": true})
        })
    }
    fn state(&self, _cx: &AppCx) -> Value {
        json!({"clicks": self.clicks})
    }
    aegis_ui::widget_plumbing!(AppCx);
}

fn context() -> AppCx {
    static N: AtomicU32 = AtomicU32::new(0);
    // The settings file sits in a scratch folder held until the test thread ends (the context outlives this helper).
    let settings = Scratch::new(&format!("headless_settings_{}", N.fetch_add(1, Ordering::Relaxed))).until_thread_end().join("settings.json");
    Cx {
        settings: Settings::load(settings, json!({})),
        art: Art::new(None),
        audio: Audio::offline(None),
        nav: Vec::new(),
        notice: Vec::new(),
        shot_dir: std::env::temp_dir(),
        quit: false,
        answers: Vec::new(),
        playtest: Playtests::off(),
        held: Default::default(),
        app: App,
    }
}

fn options(port: u16, h: Headless) -> Options {
    Options {
        title: "test".into(),
        font: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf"),
        port,
        icon: String::new(),
        canvas: Value::Null,
        window_sizes: Value::Null,
        headless: Some(h),
    }
}

/// A headless run on a thread of its own, and the link to it.
struct Game {
    link: BufReader<TcpStream>,
    thread: JoinHandle<anyhow::Result<()>>,
}

impl Game {
    fn start(need: FrameNeed, blank: bool, fps: u32) -> Self {
        Self::start_idle(need, blank, fps, Duration::from_secs(IDLE_EXIT_S))
    }

    fn start_idle(need: FrameNeed, blank: bool, fps: u32, idle: Duration) -> Self {
        let port = TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port();
        let h = Headless { size: SIZE, fps, dpr: 1.0, idle };
        let thread = std::thread::spawn(move || run(context(), vec![Box::new(Probe::new(need, blank)) as Box<dyn Screen<App>>], options(port, h)));
        let by = Instant::now() + Duration::from_secs(30);
        let stream = loop {
            match TcpStream::connect(("127.0.0.1", port)) {
                Ok(s) => break s,
                Err(e) => {
                    assert!(Instant::now() < by, "no link: {e}");
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        };
        stream.set_nodelay(true).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
        Self { link: BufReader::new(stream), thread }
    }

    fn ask(&mut self, body: Value) -> Value {
        writeln!(self.link.get_mut(), "{body}").unwrap();
        let mut line = String::new();
        self.link.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }

    fn frames(&mut self) -> u64 {
        self.ask(json!({"cmd": "ui_tree"}))["frame"].as_u64().unwrap()
    }

    /// `quit` over the link: the screens leave and the run returns, with no process exit.
    fn stop(mut self) {
        assert_eq!(self.ask(json!({"cmd": "quit"}))["ok"], json!(true));
        self.thread.join().unwrap().unwrap();
    }
}

fn scratch(name: &str) -> Scratch {
    let dir = Scratch::new(&format!("headless_{name}"));
    dir
}

fn args(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

// ---- the launch ----

#[test]
fn the_launch_arguments_select_it_and_bad_ones_are_refused() {
    assert_eq!(Headless::from_args(&args(&[("port", "1")])).unwrap(), None, "no headless=, a window");
    let h = Headless::from_args(&args(&[("headless", "1280x720")])).unwrap().unwrap();
    assert_eq!(h, Headless { size: (1280, 720), fps: DEFAULT_FPS, dpr: 1.0, idle: Duration::from_secs(IDLE_EXIT_S) });
    let h = Headless::from_args(&args(&[("headless", "1920x1080"), ("fps", "30"), ("dpr", "1.5"), ("idle_exit", "3600")])).unwrap().unwrap();
    assert_eq!(h, Headless { size: (1920, 1080), fps: 30, dpr: 1.5, idle: Duration::from_secs(3600) });
    let below = (FPS_MIN - 1).to_string();
    for bad in [
        vec![("headless", "1280")],
        vec![("headless", "1x1")],
        vec![("headless", "99999x99999")],
        vec![("headless", "1280x720"), ("fps", "0")],
        vec![("headless", "1280x720"), ("fps", below.as_str())],
        vec![("headless", "1280x720"), ("fps", "many")],
        vec![("headless", "1280x720"), ("dpr", "0")],
        vec![("headless", "1280x720"), ("idle_exit", "0")],
        vec![("headless", "1280x720"), ("idle_exit", "99999999")],
        vec![("headless", "1280x720"), ("idle_exit", "soon")],
        vec![("fps", "15")],
        vec![("dpr", "2")],
        vec![("idle_exit", "60")],
    ] {
        assert!(Headless::from_args(&args(&bad)).is_err(), "{bad:?}");
    }
}

#[test]
fn a_taken_port_is_an_error_and_not_a_silent_run() {
    let taken = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = taken.local_addr().unwrap().port();
    let h = Headless { size: SIZE, fps: DEFAULT_FPS, dpr: 1.0, idle: Duration::from_secs(IDLE_EXIT_S) };
    let e = run(context(), vec![Box::new(Probe::new(FrameNeed::Idle, false)) as Box<dyn Screen<App>>], options(port, h)).unwrap_err();
    assert!(e.to_string().contains("busy"), "{e}");
}

// ---- the same link ----

#[test]
fn the_link_sees_and_drives_the_screen_at_the_asked_size() {
    let mut g = Game::start(FrameNeed::Idle, false, DEFAULT_FPS);
    let tree = g.ask(json!({"cmd": "ui_tree"}));
    assert_eq!((tree["screen"].clone(), tree["px"].clone(), tree["pid"].clone()), (json!("probe"), json!([SIZE.0, SIZE.1]), json!(std::process::id())));
    assert_eq!(g.ask(json!({"cmd": "frames"}))["ok"], json!(true));
    assert_eq!(g.ask(json!({"cmd": "click", "id": "go"}))["ok"], json!(true));
    assert_eq!(g.ask(json!({"cmd": "ui_tree"}))["state"]["clicks"], json!(1), "the click reached the screen");
    assert_eq!(g.ask(json!({"cmd": "click", "id": "gone"}))["ok"], json!(false), "the shell's own answers");
    assert_eq!(g.ask(json!({"cmd": "no_such"}))["ok"], json!(false));
    g.stop();
}

/// The name a battle gives its own command stays the battle's: drawing frames goes by another.
#[test]
fn a_screens_own_step_reaches_it_and_does_not_draw_in_its_place() {
    let mut g = Game::start(FrameNeed::Idle, false, DEFAULT_FPS);
    let reply = g.ask(json!({"cmd": "step"}));
    assert_eq!(reply["stepped"], json!(true), "{reply}");
    assert_eq!(g.ask(json!({"cmd": "ui_tree"}))["state"]["clicks"], json!(100));
    let before = g.frames();
    let drawn = g.ask(json!({"cmd": "frames", "frames": 3}));
    assert!(drawn["frame"].as_u64().unwrap() >= before + 3, "{drawn}");
    assert_eq!(g.ask(json!({"cmd": "ui_tree"}))["state"]["clicks"], json!(100), "frames reach no screen command");
    g.stop();
}

#[test]
fn a_window_size_picked_in_the_settings_resizes_the_canvas() {
    let mut g = Game::start(FrameNeed::Idle, false, DEFAULT_FPS);
    g.ask(json!({"cmd": "frames"}));
    g.ask(json!({"cmd": "key", "key": "s"}));
    g.ask(json!({"cmd": "frames", "frames": 2}));
    assert_eq!(g.ask(json!({"cmd": "ui_tree"}))["px"], json!([960, 540]));
    g.stop();
}

// ---- the pace ----

#[test]
fn a_screen_in_full_need_holds_the_low_rate_however_often_the_link_reads() {
    let mut g = Game::start(FrameNeed::Full, false, DEFAULT_FPS);
    let (t0, f0) = (Instant::now(), g.frames());
    let mut last = f0;
    while t0.elapsed() < Duration::from_millis(1500) {
        last = g.frames(); // a window draws a frame per request; this must not
        std::thread::sleep(Duration::from_millis(2));
    }
    let drawn = last - f0;
    let most = (t0.elapsed().as_secs_f64() * f64::from(DEFAULT_FPS)).ceil() as u64 + 1;
    assert!(drawn <= most, "{drawn} frames in {:?}, at most {most}", t0.elapsed());
    assert!(drawn >= 3, "only {drawn} frames in {:?}", t0.elapsed());
    g.stop();
}

#[test]
fn a_resting_screen_draws_next_to_nothing_while_nothing_asks() {
    let mut g = Game::start(FrameNeed::Idle, false, DEFAULT_FPS);
    let f0 = g.frames();
    std::thread::sleep(Duration::from_millis(2500));
    let drawn = g.frames() - f0;
    // The read above keeps frames coming for pacing::SETTLE (a quarter second, 4 at this rate), then the 1 s heartbeat.
    assert!(drawn <= 8, "{drawn} frames in 2.5 s on a screen at rest");
    assert!(drawn >= 1, "the heartbeat stopped");
    g.stop();
}

#[test]
fn frames_asked_for_draw_at_once_and_are_bounded() {
    let mut g = Game::start(FrameNeed::Idle, false, DEFAULT_FPS);
    std::thread::sleep(Duration::from_millis(400)); // at rest
    let before = g.frames();
    let t0 = Instant::now();
    let reply = g.ask(json!({"cmd": "frames", "frames": 120}));
    assert!(reply["frame"].as_u64().unwrap() >= before + 120, "{reply}");
    let budget = Duration::from_secs_f64(120.0 / f64::from(DEFAULT_FPS));
    assert!(t0.elapsed() < budget, "they do not wait for the pace: {:?}", t0.elapsed());
    for bad in [json!(0), json!(FRAMES_MAX + 1), json!("two"), json!(-1)] {
        assert_eq!(g.ask(json!({"cmd": "frames", "frames": bad}))["ok"], json!(false), "{bad}");
    }
    g.stop();
}

// ---- the end of a run with no driver ----

fn waits_for_its_end(g: &Game, by: Duration) -> bool {
    let t0 = Instant::now();
    while t0.elapsed() < by {
        if g.thread.is_finished() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn a_run_nobody_asks_anything_ends_and_says_why() {
    let idle = Duration::from_secs(1);
    let g = Game::start_idle(FrameNeed::Idle, false, DEFAULT_FPS, idle);
    assert!(waits_for_its_end(&g, Duration::from_secs(20)), "it went on with no driver");
    let e = g.thread.join().unwrap().unwrap_err();
    assert!(e.to_string().contains("no link request for 1 s"), "{e}");
    // A screen in full need keeps drawing: drawing is not being asked.
    let g = Game::start_idle(FrameNeed::Full, false, DEFAULT_FPS, idle);
    assert!(waits_for_its_end(&g, Duration::from_secs(20)), "frames kept it alive");
}

#[test]
fn a_driver_that_keeps_asking_is_never_cut_off() {
    let idle = Duration::from_secs(5);
    let mut g = Game::start_idle(FrameNeed::Idle, false, DEFAULT_FPS, idle);
    let t0 = Instant::now();
    while t0.elapsed() < idle + Duration::from_secs(1) {
        g.frames();
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(!g.thread.is_finished(), "cut off with a driver asking");
    g.stop();
}

/// A long request is the driver's too: the limit counts from when it was done.
#[test]
fn a_clip_longer_than_the_limit_is_not_idle() {
    let dir = scratch("long_clip");
    let mut g = Game::start_idle(FrameNeed::Full, false, DEFAULT_FPS, Duration::from_secs(1));
    let reply = g.ask(json!({"cmd": "clip", "dir": dir.join("c").display().to_string(), "frames": 8, "every_ms": 200, "max_w": 200}));
    assert_eq!(reply["ok"], json!(true), "{reply}");
    g.stop();
    std::fs::remove_dir_all(&dir).unwrap();
}

// ---- frames from memory ----

#[test]
fn a_blank_canvas_or_a_minimized_ones_is_refused_and_nothing_is_written() {
    let dir = scratch("blank");
    let path = dir.join("shot.png");
    // One pixel: what a minimized window leaves.
    let tiny = Canvas::new(1, 1);
    assert!(save_png(&tiny, &path, 0).unwrap_err().to_string().contains("1 x 1"));
    let narrow = Canvas::new(SHOT_SIDE_MIN - 1, 200);
    assert!(blank_reason(&narrow).is_some());
    // An older shot at the path does not outlive a refused one.
    std::fs::write(&path, b"older").unwrap();
    assert!(save_png(&tiny, &path, 0).is_err());
    assert!(!path.exists(), "it would pass for this one");
    // Never drawn, and cleared to one colour.
    let mut c = Canvas::new(200, 100);
    assert!(save_png(&c, &path, 0).unwrap_err().to_string().contains("blank"));
    c.clear(rgb(12, 34, 56));
    assert!(save_png(&c, &path, 0).unwrap_err().to_string().contains("blank"));
    assert!(!path.exists(), "a refused shot writes nothing");
    // Within the tolerance is still one colour; past it is a picture.
    let o = (50 * 200 + 50) * 4;
    c.px[o] = c.px[0] + BLANK_TOLERANCE;
    assert!(blank_reason(&c).is_some());
    c.px[o] = c.px[0] + BLANK_TOLERANCE + 1;
    assert_eq!(blank_reason(&c), None);
    save_png(&c, &path, 0).unwrap();
    let back = load_png(&path).unwrap();
    assert_eq!((back.w, back.h), (200, 100));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_screenshot_over_the_link_is_the_screen_and_a_blank_one_fails() {
    let dir = scratch("link_shot");
    let mut g = Game::start(FrameNeed::Idle, false, DEFAULT_FPS);
    g.ask(json!({"cmd": "frames"}));
    let path = dir.join("drawn.png");
    let reply = g.ask(json!({"cmd": "screenshot", "path": path.display().to_string(), "max_w": 0}));
    assert_eq!(reply["ok"], json!(true), "{reply}");
    let shot = load_png(&path).unwrap();
    assert_eq!((shot.w as u32, shot.h as u32), SIZE);
    assert!(shot.px.chunks_exact(4).any(|p| p != &shot.px[..4]), "the shot shows something");
    g.stop();

    let mut g = Game::start(FrameNeed::Idle, true, DEFAULT_FPS);
    g.ask(json!({"cmd": "frames"}));
    let path = dir.join("blank.png");
    let reply = g.ask(json!({"cmd": "screenshot", "path": path.display().to_string()}));
    assert_eq!(reply["ok"], json!(false), "{reply}");
    assert!(reply["error"].as_str().unwrap().contains("blank"), "{reply}");
    assert!(!path.exists());
    g.stop();
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn a_clip_is_numbered_frames_on_time_and_refuses_what_it_cannot_do() {
    let dir = scratch("clip");
    let mut g = Game::start(FrameNeed::Full, false, DEFAULT_FPS);
    let clip = dir.join("one");
    let before = g.frames();
    let reply = g.ask(json!({"cmd": "clip", "dir": clip.display().to_string(), "frames": 4, "every_ms": 200, "max_w": 400}));
    assert_eq!(reply["ok"], json!(true), "{reply}");
    assert_eq!(reply["blank"], json!([]));
    for i in 1..=4 {
        let frame = load_png(&clip.join(format!("frame_{i:04}.png"))).unwrap();
        assert_eq!(frame.w, 400, "max_w holds");
    }
    assert!(!clip.join("frame_0005.png").exists());
    let ms: Vec<u64> = reply["ms"].as_array().unwrap().iter().map(|m| m.as_u64().unwrap()).collect();
    for (i, at) in ms.iter().enumerate() {
        assert!(*at >= i as u64 * 200, "frame {i} came at {at} ms, before its time");
    }
    // 600 ms at 15 a second: frames went on between the saves, so a clock on wall time is not held.
    assert!(reply["frame"].as_u64().unwrap() >= before + 8, "{reply}");

    let again = g.ask(json!({"cmd": "clip", "dir": clip.display().to_string(), "frames": 1}));
    assert_eq!(again["ok"], json!(false), "the frames there would pass for this clip's: {again}");
    let other = dir.join("two").display().to_string();
    for bad in [
        json!({"cmd": "clip"}),
        json!({"cmd": "clip", "dir": other, "frames": 0}),
        json!({"cmd": "clip", "dir": other, "frames": CLIP_FRAMES_MAX + 1}),
        json!({"cmd": "clip", "dir": other, "every_ms": 1}),
        json!({"cmd": "clip", "dir": other, "frames": 100, "every_ms": 1000}),
    ] {
        assert_eq!(g.ask(bad.clone())["ok"], json!(false), "{bad}");
    }
    assert!(!dir.join("two").exists(), "refused before anything is written");
    g.stop();

    let mut g = Game::start(FrameNeed::Idle, true, DEFAULT_FPS);
    let nothing = dir.join("blank");
    let reply = g.ask(json!({"cmd": "clip", "dir": nothing.display().to_string(), "frames": 3, "every_ms": 20}));
    assert_eq!(reply["ok"], json!(false), "{reply}");
    assert!(!nothing.join("frame_0001.png").exists());
    g.stop();
    std::fs::remove_dir_all(&dir).unwrap();
}
