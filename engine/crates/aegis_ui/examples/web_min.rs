//! The smallest program on the window shell (WEBREL1-E2): one screen showing the last key, click and right click and
//! whether audio is live. On the web (`wasm32-unknown-unknown`, bound with `wasm-bindgen` 0.2.129) it starts on the
//! page's canvas `aegis` through `shell::start_web` and mirrors what it shows into the page title, which the headless
//! browser check reads (`tools/web/web_min_check.js`); on the desktop it opens a window through `shell::run`.
//! It names no game: the font is the engine's own, mounted into the read seam on the web.

use aegis_ui::art::Art;
use aegis_ui::audio::Audio;
use aegis_ui::canvas::{rgb, Rect};
use aegis_ui::painter::Painter;
use aegis_ui::playtest::Playtests;
use aegis_ui::screen::{Context, Cx, Key, Screen, Settings};
use aegis_ui::shell::{self, Options};
use aegis_ui::ui::Gfx;
use serde_json::{json, Value};
use std::path::PathBuf;

/// The engine's font, as the web bundle would carry it.
#[cfg(target_arch = "wasm32")]
const FONT: &[u8] = include_bytes!("../../../assets/fonts/OpenSans-SemiBold.ttf");

/// What the player did last.
#[derive(Default)]
struct Last {
    key: String,
    click: Option<(i32, i32)>,
    backs: u32,
    presses: u32,
    frames: u64,
}

impl Context for Last {
    const MORE_COMMANDS: &'static str = "none";
    fn note(&mut self, _line: String) {}
    fn agent_summary(&self) -> Value {
        Value::Null
    }
    fn agent_game(&self) -> Value {
        json!({"key": self.key, "presses": self.presses})
    }
}

/// The state line the screen draws (and the web page shows as its title): the last input, audio, the frames drawn and
/// the canvas the shell draws into (px) at the device pixel ratio it was told.
fn status(cx: &Cx<Last>, g: &Gfx) -> String {
    let click = cx.app.click.map_or("-".to_string(), |(x, y)| format!("{x},{y}"));
    let key = if cx.app.key.is_empty() { "-" } else { cx.app.key.as_str() };
    let audio = if cx.audio.live() { "on" } else { "off" };
    format!("key={key} click={click} back={} audio={audio} frames={} canvas={}x{} dpr={}", cx.app.backs, cx.app.frames, g.canvas.w, g.canvas.h, g.kit.device.dpr)
}

#[cfg(target_arch = "wasm32")]
fn show(line: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        doc.set_title(line);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn show(_line: &str) {}

struct Board;

impl Screen<Last> for Board {
    fn id(&self) -> &'static str {
        "web_min"
    }
    fn draw(&mut self, cx: &mut Cx<Last>, g: &mut Gfx, _mouse: (i32, i32), _live: bool) {
        cx.app.frames += 1;
        // The background steps through four colours, one per key or click, so a press shows in the pixels too.
        let bg = [rgb(32, 48, 96), rgb(96, 32, 48), rgb(32, 96, 48), rgb(96, 96, 32)][(cx.app.presses % 4) as usize];
        let line = status(cx, g);
        let mut p = Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
        p.fill(Rect::new(0, 0, 640, 360), bg);
        p.fill(Rect::new(20, 20, 600, 4), rgb(240, 240, 240));
        p.text(20.0, 40.0, "Aegis web start", 24.0, rgb(255, 255, 255));
        p.text(20.0, 80.0, &line, 12.0, rgb(255, 255, 255));
        if let Some((x, y)) = cx.app.click {
            p.fill(Rect::new(x - 3, y - 3, 7, 7), rgb(255, 220, 0));
        }
        show(&line);
    }
    fn widgets(&self) -> Vec<Value> {
        Vec::new()
    }
    fn draw_tip(&self, _p: &mut Painter, _mouse: (i32, i32)) {}
    fn click(&mut self, cx: &mut Cx<Last>, pos: (i32, i32)) -> bool {
        cx.app.click = Some(pos);
        cx.app.presses += 1;
        true
    }
    fn click_id(&mut self, _cx: &mut Cx<Last>, id: &str) -> Result<(), String> {
        Err(format!("no widget {id}"))
    }
    fn widget_rect(&self, _id: &str) -> Option<(Rect, Vec<String>)> {
        None
    }
    fn key(&mut self, cx: &mut Cx<Last>, k: Key) {
        cx.app.key = match k {
            Key::Char(c) => c.to_string(),
            other => format!("{other:?}"),
        };
        cx.app.presses += 1;
    }
    /// Right click (and Esc): counted; the only screen stays.
    fn back(&mut self, cx: &mut Cx<Last>) {
        cx.app.backs += 1;
        cx.app.presses += 1;
    }
}

fn context() -> Cx<Last> {
    Cx {
        settings: Settings::load(PathBuf::from("web_min_settings.json"), json!({})),
        art: Art::new(None),
        audio: Audio::layered(Vec::new()),
        nav: Vec::new(),
        notice: Vec::new(),
        shot_dir: PathBuf::new(),
        quit: false,
        answers: Vec::new(),
        playtest: Playtests::off(),
        held: Default::default(),
        app: Last::default(),
    }
}

fn options(font: PathBuf) -> Options {
    Options { title: "Aegis web start".into(), font, port: 0, icon: String::new(), canvas: Value::Null, window_sizes: Value::Null, headless: None }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let mut files = aegis_core::vfs::Table::new();
    files.insert("fonts/OpenSans-SemiBold.ttf", FONT.to_vec());
    aegis_core::vfs::mount(files);
    if let Err(e) = shell::start_web(context(), vec![Box::new(Board)], options(PathBuf::from("fonts/OpenSans-SemiBold.ttf")), "aegis") {
        web_sys::console::error_1(&format!("web_min: {e:#}").into());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    let font = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf");
    shell::run(context(), vec![Box::new(Board)], options(font))
}
