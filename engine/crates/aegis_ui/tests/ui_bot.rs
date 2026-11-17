//! UIBOT (REPLAY-CI1): the seeded UI bot (`aegis_ui::shell::bot`) on a test screen: it presses only enabled, clickable,
//! not skipped widgets, a seed gives the same presses, a quit ends the run, and a panic in a press is reported with its
//! step, press and place in the source, not raised.

use aegis_core::scratch::Scratch;
use aegis_ui::art::Art;
use aegis_ui::audio::Audio;
use aegis_ui::canvas::Rect;
use aegis_ui::playtest::Playtests;
use aegis_ui::screen::{Context, Cx, Screen, Settings};
use aegis_ui::shell::bot::{self, Bot};
use aegis_ui::shell::Options;
use aegis_ui::ui::{Gfx, Ui, TEXT};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Default)]
struct App {
    pressed: Vec<String>,
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
}

type AppCx = Cx<App>;

#[derive(Clone, Copy, PartialEq)]
enum Act {
    Go,
    Off,
    Danger,
    Boom,
    Quit,
}

/// Buttons: `go` (pressable), `off` (disabled), `danger` (pressable, skipped by the tests), `label` (no action) and, as
/// the screen is built, `boom` (panics) and `quit` (ends the program).
struct Panel {
    boom: bool,
    quit: bool,
    w: Vec<aegis_ui::ui::Widget<Act>>,
}

impl Panel {
    fn new(boom: bool, quit: bool) -> Self {
        Self { boom, quit, w: Vec::new() }
    }
    fn act(&mut self, cx: &mut AppCx, a: Act) {
        let name = match a {
            Act::Go => "go",
            Act::Off => "off",
            Act::Danger => "danger",
            Act::Boom => panic!("boom pressed"),
            Act::Quit => {
                cx.quit = true;
                "quit"
            }
        };
        cx.app.pressed.push(name.into());
    }
}

impl Screen<App> for Panel {
    fn id(&self) -> &'static str {
        "panel"
    }
    fn tick(&mut self, _cx: &mut AppCx) {}
    fn draw(&mut self, _cx: &mut AppCx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<Act> = Ui::new(g, mouse, live);
        ui.button("go", Rect::new(10, 10, 60, 14), "Go", Act::Go, true);
        ui.button("off", Rect::new(10, 30, 60, 14), "Off", Act::Off, false);
        ui.button("danger", Rect::new(10, 50, 60, 14), "Danger", Act::Danger, true);
        ui.row("label", Rect::new(10, 70, 60, 14), "Label", TEXT, None, false, Vec::new());
        if self.boom {
            ui.button("boom", Rect::new(10, 90, 60, 14), "Boom", Act::Boom, true);
        }
        if self.quit {
            ui.button("quit", Rect::new(10, 110, 60, 14), "Quit", Act::Quit, true);
        }
        self.w = ui.widgets;
    }
    aegis_ui::widget_plumbing!(AppCx);
}

fn context() -> AppCx {
    static N: AtomicU32 = AtomicU32::new(0);
    let settings = Scratch::new(&format!("ui_bot_settings_{}", N.fetch_add(1, Ordering::Relaxed))).until_thread_end().join("settings.json");
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
        app: App::default(),
    }
}

fn options() -> Options {
    Options {
        title: "test".into(),
        font: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf"),
        port: 0,
        icon: String::new(),
        canvas: Value::Null,
        window_sizes: Value::Null,
        headless: None,
    }
}

fn run(panel: Panel, bot: &Bot) -> bot::Run {
    bot::run(context(), vec![Box::new(panel) as Box<dyn Screen<App>>], &options(), bot)
}

fn bot(seed: u64, steps: usize) -> Bot {
    let mut b = Bot::new(seed, steps);
    b.size = (640, 360);
    b
}

#[test]
fn it_presses_only_enabled_clickable_widgets_off_the_skip_list() {
    let mut b = bot(7, 60);
    b.skip = vec!["dan".into()];
    let r = run(Panel::new(false, false), &b);
    assert!(r.panic.is_none(), "{}", r.summary());
    assert_eq!(r.steps, 60);
    let clicks: Vec<&String> = r.presses.iter().filter(|p| p.starts_with("click ")).collect();
    assert!(!clicks.is_empty(), "{:?}", r.presses);
    assert!(clicks.iter().all(|p| *p == "click go"), "only go is pressable: {:?}", r.presses);
    assert!(r.presses.iter().all(|p| p.starts_with("click ") || p.starts_with("key ")));
    assert_eq!(r.screens, ["panel"]);
}

#[test]
fn a_seed_gives_the_same_presses_and_another_seed_others() {
    let a = run(Panel::new(false, false), &bot(3, 40));
    let b = run(Panel::new(false, false), &bot(3, 40));
    let c = run(Panel::new(false, false), &bot(4, 40));
    assert_eq!(a.presses, b.presses);
    assert_ne!(a.presses, c.presses);
}

#[test]
fn a_quit_ends_the_run_cleanly() {
    let mut b = bot(1, 500);
    b.keys.clear();
    let r = run(Panel::new(false, true), &b);
    assert!(r.quit && r.panic.is_none(), "{}", r.summary());
    assert!(r.steps < 500);
    assert_eq!(r.presses.last().map(String::as_str), Some("click quit"));
}

#[test]
fn a_panic_is_reported_with_its_step_press_and_place() {
    let r = run(Panel::new(true, false), &bot(1, 500));
    let p = r.panic.clone().expect("boom is pressed within 500 steps");
    assert_eq!(p.action, "click boom");
    assert_eq!(p.screen, "panel");
    assert_eq!(p.message, "boom pressed");
    assert!(p.location.contains("ui_bot.rs:"), "{}", p.location);
    assert_eq!(r.steps, p.step + 1);
    assert_eq!(r.presses.last().map(String::as_str), Some("click boom"));
    assert!(r.summary().contains("PANIC at step"), "{}", r.summary());
}
