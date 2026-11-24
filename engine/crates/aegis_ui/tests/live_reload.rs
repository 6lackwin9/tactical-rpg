//! ENGINE-64 (LIVE-RELOAD1): live reload in a run with no window, over the real agent link. A picture edited on disk
//! is drawn with its new pixels on a later frame, the game state kept; an edited data file the game takes is swapped in
//! and the screens are told; one it refuses changes nothing and a notice names it; a run without `live` never reloads.

use aegis_core::assets::Layers;
use aegis_core::clock::{Duration, Instant};
use aegis_core::scratch::Scratch;
use aegis_ui::art::Art;
use aegis_ui::audio::Audio;
use aegis_ui::canvas::Rect;
use aegis_ui::playtest::Playtests;
use aegis_ui::screen::{Context, Cx, Screen, Settings};
use aegis_ui::shell::headless::Headless;
use aegis_ui::shell::{run, Options};
use aegis_ui::ui::{Gfx, Ui};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

const PIC: &str = "pics/swatch.png";
const ITEMS: &str = "items.json";

/// The game's state: the item name its data gives, and how many steps were taken (kept across reloads).
struct App {
    data: PathBuf,
    name: String,
    steps: u32,
}

impl Context for App {
    const MORE_COMMANDS: &'static str = "none";
    fn note(&mut self, _line: String) {}
    fn agent_summary(&self) -> Value {
        json!({"name": self.name, "steps": self.steps})
    }
    fn agent_game(&self) -> Value {
        Value::Null
    }
    fn data_dirs(&self) -> Vec<PathBuf> {
        vec![self.data.clone()]
    }
    fn reload_data(cx: &mut Cx<Self>, _changed: &[PathBuf]) -> Result<Vec<String>, String> {
        cx.app.name = read_name(&cx.app.data)?;
        Ok(Vec::new())
    }
}

fn read_name(data: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(data.join(ITEMS)).map_err(|e| format!("{ITEMS}: {e}"))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{ITEMS}: {e}"))?;
    v["name"].as_str().map(String::from).ok_or(format!("{ITEMS}: no name"))
}

/// The picture filling a box, the item name, and a step button; `reloaded` counts the data reloads it was told of.
struct Shelf {
    reloaded: u32,
    centre: [u8; 4],
    w: Vec<aegis_ui::ui::Widget<()>>,
}

impl Shelf {
    fn act(&mut self, cx: &mut Cx<App>, _a: ()) {
        cx.app.steps += 1;
    }
}

impl Screen<App> for Shelf {
    fn id(&self) -> &'static str {
        "shelf"
    }
    fn draw(&mut self, cx: &mut Cx<App>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<()> = Ui::new(g, mouse, live);
        let r = Rect::new(100, 100, 64, 64);
        if let Some(img) = cx.art.image(PIC, r.w, r.h).cloned() {
            ui.p.image(&img, r);
        }
        let name = cx.app.name.clone();
        ui.button("shelf.step", Rect::new(20, 20, 120, 16), &name, (), true);
        self.w = ui.widgets;
        let c = &g.canvas;
        let at = ((132 * c.w / 640) as usize + (132 * c.h / 360) as usize * c.w as usize) * 4;
        self.centre = [c.px[at], c.px[at + 1], c.px[at + 2], c.px[at + 3]];
    }
    fn data_reloaded(&mut self, _cx: &mut Cx<App>, top: bool) -> Option<String> {
        self.reloaded += 1;
        top.then(|| "the shelf keeps its place".to_string())
    }
    fn state(&self, _cx: &Cx<App>) -> Value {
        json!({"reloaded": self.reloaded, "centre": self.centre})
    }
    aegis_ui::widget_plumbing!(Cx<App>);
}

/// A 4 x 4 PNG of one colour, written whole (a temp file renamed over it, as editors save).
fn write_png(path: &Path, rgb: [u8; 3]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut bytes = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut bytes, 4, 4);
        enc.set_color(png::ColorType::Rgba);
        let mut w = enc.write_header().unwrap();
        let px: Vec<u8> = (0..16).flat_map(|_| [rgb[0], rgb[1], rgb[2], 255]).collect();
        w.write_image_data(&px).unwrap();
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes).unwrap();
    std::fs::rename(&tmp, path).unwrap();
}

fn fresh_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0)).unwrap().local_addr().unwrap().port()
}

fn start(root: &Path, live: bool) -> (BufReader<TcpStream>, std::thread::JoinHandle<anyhow::Result<()>>) {
    let port = fresh_port();
    let options = Options {
        title: "Live Test".into(),
        font: Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf"),
        port,
        icon: String::new(),
        canvas: Value::Null,
        window_sizes: Value::Null,
        headless: Some(Headless { size: (640, 360), fps: 30, dpr: 1.0, idle: Duration::from_secs(60), stepped: true }),
        live,
    };
    let root = root.to_path_buf();
    let thread = std::thread::spawn(move || {
        let data = root.join("data");
        let cx = Cx {
            settings: Settings::load(root.join("settings.json"), json!({})),
            art: Art::layered(Layers::of(vec![root.join("assets")])),
            audio: Audio::offline(None),
            nav: Vec::new(),
            notice: Vec::new(),
            shot_dir: root.join("shots"),
            quit: false,
            answers: Vec::new(),
            playtest: Playtests::off(),
            held: Default::default(),
            app: App { name: read_name(&data).unwrap(), data, steps: 0 },
        };
        run(cx, vec![Box::new(Shelf { reloaded: 0, centre: [0; 4], w: Vec::new() }) as Box<dyn Screen<App>>], options)
    });
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

/// Draws frames until `done` holds for the tree (at most 5 s of wall time, the watcher looking every 500 ms).
fn until(link: &mut BufReader<TcpStream>, what: &str, done: impl Fn(&Value) -> bool) -> Value {
    let by = Instant::wall() + Duration::from_secs(5);
    loop {
        ask(link, json!({"cmd": "frames"}));
        let tree = ask(link, json!({"cmd": "ui_tree"}));
        if done(&tree) {
            return tree;
        }
        assert!(Instant::wall() < by, "{what}: {tree}");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn setup(name: &str) -> Scratch {
    let t = Scratch::new(name);
    write_png(&t.path().join("assets").join(PIC), [200, 30, 30]);
    std::fs::create_dir_all(t.path().join("data")).unwrap();
    std::fs::write(t.path().join("data").join(ITEMS), r#"{"name": "Rusted Billhook"}"#).unwrap();
    t
}

#[test]
fn an_edited_picture_and_data_reload_in_place_and_a_broken_edit_changes_nothing() {
    let t = setup("live_reload");
    let (mut link, thread) = start(t.path(), true);
    let tree = ask(&mut link, json!({"cmd": "ui_tree"}));
    assert_eq!(tree["state"]["centre"], json!([200, 30, 30, 255]), "{tree}");
    assert_eq!(ask(&mut link, json!({"cmd": "click", "id": "shelf.step"}))["ok"], json!(true));
    // (1) a picture saved over: the next frames draw its new pixels, the game state kept, one line says so
    std::thread::sleep(std::time::Duration::from_millis(50));
    write_png(&t.path().join("assets").join(PIC), [30, 30, 200]);
    let tree = until(&mut link, "the new picture", |t| t["state"]["centre"] == json!([30, 30, 200, 255]));
    assert_eq!(tree["game"]["steps"], json!(1), "the game state stays");
    assert_eq!(tree["live"], json!("Reloaded 1 art file"), "{tree}");
    // (2) a good data edit: swapped in, every screen told, the item renamed on the screen
    std::fs::write(t.path().join("data").join(ITEMS), r#"{"name": "Honed Billhook"}"#).unwrap();
    let tree = until(&mut link, "the new data", |t| t["game"]["name"] == json!("Honed Billhook"));
    assert_eq!(tree["state"]["reloaded"], json!(1));
    assert_eq!(tree["live"], json!("Data reloaded; the shelf keeps its place"), "{tree}");
    assert!(tree["widgets"].to_string().contains("Honed Billhook"), "the screen shows the new name");
    // (3) a broken data edit: nothing swapped, a notice names the file
    std::fs::write(t.path().join("data").join(ITEMS), r#"{"name": "#).unwrap();
    let tree = until(&mut link, "the refusal", |t| t["screen"] == json!("notice"));
    let text = tree.to_string();
    assert!(text.contains("Data not reloaded: items.json"), "{text}");
    assert_eq!(tree["game"]["name"], json!("Honed Billhook"), "the old data stays");
    assert_eq!(ask(&mut link, json!({"cmd": "quit"}))["ok"], json!(true));
    thread.join().unwrap().unwrap();
}

#[test]
fn nothing_reloads_without_live_reload() {
    let t = setup("live_reload_off");
    let (mut link, thread) = start(t.path(), false);
    ask(&mut link, json!({"cmd": "frames"}));
    write_png(&t.path().join("assets").join(PIC), [30, 200, 30]);
    std::fs::write(t.path().join("data").join(ITEMS), r#"{"name": "Honed Billhook"}"#).unwrap();
    std::thread::sleep(aegis_ui::live::POLL * 3);
    ask(&mut link, json!({"cmd": "frames"}));
    let tree = ask(&mut link, json!({"cmd": "ui_tree"}));
    assert_eq!((tree["state"]["centre"].clone(), tree["game"]["name"].clone(), tree.get("live").is_none()), (json!([200, 30, 30, 255]), json!("Rusted Billhook"), true), "a run without live reload keeps what it read: {tree}");
    assert_eq!(ask(&mut link, json!({"cmd": "quit"}))["ok"], json!(true));
    thread.join().unwrap().unwrap();
}

/// The changes a watcher reports, sorted by what they touch, and their line.
#[test]
fn changed_files_are_sorted_by_what_they_touch() {
    use aegis_ui::live::Changes;
    let (assets, data) = (PathBuf::from("/g/assets"), PathBuf::from("/g/data"));
    let paths: Vec<PathBuf> = ["/g/assets/units/a.png", "/g/assets/ui/theme.json", "/g/assets/ui/layouts/party.json", "/g/assets/audio/hit.ogg", "/g/data/catalog/items.json", "/elsewhere/x.png", "/g/assets/units/a.png"].iter().map(PathBuf::from).collect();
    let c = Changes::of(&paths, &[assets], &[data]);
    assert_eq!(c.art, ["units/a.png"]);
    assert!(c.theme);
    assert_eq!((c.layouts.clone(), c.sounds.clone(), c.data.len()), (vec!["ui/layouts/party.json".to_string()], vec!["audio/hit.ogg".to_string()], 1));
    assert_eq!(c.line().as_deref(), Some("Reloaded 1 art file, the theme, 1 layout, 1 sound"));
    assert_eq!(Changes::default().line(), None);
    // a scan sees a file change, appear and go
    let t = Scratch::new("live_scan");
    std::fs::write(t.path().join("a.txt"), "1").unwrap();
    let before = aegis_ui::live::scan(&[t.path().to_path_buf()]);
    std::fs::write(t.path().join("a.txt"), "12").unwrap();
    std::fs::write(t.path().join("b.txt"), "1").unwrap();
    let after = aegis_ui::live::scan(&[t.path().to_path_buf()]);
    let mut diff = aegis_ui::live::changed(&before, &after);
    diff.sort();
    assert_eq!(diff, [t.path().join("a.txt"), t.path().join("b.txt")]);
    std::fs::remove_file(t.path().join("b.txt")).unwrap();
    assert_eq!(aegis_ui::live::changed(&after, &aegis_ui::live::scan(&[t.path().to_path_buf()])), [t.path().join("b.txt")]);
}

/// While a save is being written the watcher's finds wait (`Watch::take(true)`), and come on the next call after it.
#[test]
fn finds_wait_while_a_save_is_written() {
    let t = Scratch::new("live_wait");
    let dir = t.path().join("assets");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.png"), "1").unwrap();
    let mut w = aegis_ui::live::Watch::start(vec![dir.clone()], Vec::new()).expect("a watcher");
    assert!(aegis_ui::live::on());
    std::thread::sleep(std::time::Duration::from_millis(100));
    std::fs::write(dir.join("a.png"), "22").unwrap();
    std::thread::sleep(aegis_ui::live::POLL * 3);
    assert!(w.take(true).is_empty(), "held while a save is written");
    assert_eq!(w.take(false).art, ["a.png"], "then handed over");
    assert!(w.take(false).is_empty(), "once");
}
