//! LOOK1-E1B: the window, timeline, recordings, vignette, notes and pips colours read the theme (`theme::th()`). With
//! no theme file each pane draws pixel for pixel as before the migration (frame hashes at 1280 x 720, recorded before
//! the first edit in `fixtures/theme/pane_hashes.json`). Each role a pane draws with changes that pane when a theme file
//! sets it, a pip block keeps its caller's colour under any theme, and a source scan finds no colour literal left in the
//! six files but those kept on purpose (`KEPT`).

use aegis_core::scratch::Scratch;
use aegis_core::assets::Layers;
use aegis_ui::art::Art;
use aegis_ui::audio::Audio;
use aegis_ui::canvas::{Canvas, Rect};
use aegis_ui::notes::{self, NoteBox};
use aegis_ui::painter::{Fonts, View};
use aegis_ui::pips;
use aegis_ui::playtest::Playtests;
use aegis_ui::recordings::{NoteView, Recordings};
use aegis_ui::screen::{Context, Cx, Screen, Settings};
use aegis_ui::theme;
use aegis_ui::timeline;
use aegis_ui::ui::{Gfx, Kit, Ui, BG};
use aegis_ui::vignette::{self, Palette, Phase, Player, Reel, TextSpeed};
use aegis_ui::window::{Desk, Frac, Layout, Win};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn font() -> Vec<u8> {
    std::fs::read(root().join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

const TEXT_SIZES: [f32; 4] = [0.85, 1.0, 1.15, 1.3];
const PANES: [&str; 6] = ["window", "timeline", "recordings", "vignette", "notes", "pips"];

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

/// A context whose recordings folder is `dir` (relative, as the recordings screen prints it), with the content
/// fingerprint "this data".
fn cx(dir: &str) -> Cx<App> {
    Cx {
        settings: Settings::load(std::env::temp_dir().join("aegis_theme_panes_no_settings.json"), json!({})),
        art: Art::new(None),
        audio: Audio::offline(None),
        nav: Vec::new(),
        notice: Vec::new(),
        shot_dir: std::env::temp_dir(),
        quit: false,
        answers: Vec::new(),
        playtest: Playtests::new(PathBuf::from(dir), false, json!("this data")),
        held: Default::default(),
        app: App,
    }
}

fn gfx(scale: f32) -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(1280, 720), view: View::fit(1280, 720), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.fonts.scale = scale;
    g.canvas.clear(BG);
    g
}

/// A named frame.
type Shot = (String, Vec<u8>);

/// One frame of `draw` at the mouse, and the widgets it pushed.
fn shot(scale: f32, mouse: (i32, i32), draw: &impl Fn(&mut Ui<()>)) -> (Vec<u8>, Vec<(String, Rect)>) {
    let mut g = gfx(scale);
    let widgets = {
        let mut ui: Ui<()> = Ui::new(&mut g, mouse, true);
        draw(&mut ui);
        ui.widgets.iter().map(|w| (w.id.clone(), w.rect)).collect()
    };
    (g.canvas.px, widgets)
}

/// `draw` idle, then at Normal text with the mouse over each widget of `hover` (a hover recolours; the idle frames
/// check the layouts at every size).
fn shots(name: &str, scale: f32, hover: &[&str], draw: impl Fn(&mut Ui<()>)) -> Vec<Shot> {
    let (idle, widgets) = shot(scale, (-1, -1), &draw);
    let mut out = vec![(name.to_string(), idle)];
    if scale == 1.0 {
        for id in hover {
            let r = widgets.iter().find(|w| w.0 == *id).unwrap_or_else(|| panic!("widget {id} drawn")).1;
            out.push((format!("{name}#{id}"), shot(scale, (r.x + r.w / 2, r.y + r.h / 2), &draw).0));
        }
    }
    out
}

/// A desk of two windows open (the second in front) and a third closed; then with the first collapsed.
fn window_shots(scale: f32) -> Vec<Shot> {
    let wins = vec![Win::new("a", "Alpha", (60, 40)), Win::new("b", "Beta", (60, 40)), Win::new("c", "Gamma", (60, 40))];
    let layouts = vec![
        Layout { name: "Two".into(), places: vec![("a".into(), Frac::new(0.0, 0.0, 0.5, 0.8)), ("b".into(), Frac::new(0.5, 0.0, 0.5, 0.8))] },
        Layout { name: "One".into(), places: vec![("c".into(), Frac::new(0.25, 0.25, 0.5, 0.5))] },
    ];
    let mut desk = Desk::new(Rect::new(0, 20, 640, 340), wins, layouts);
    desk.apply_layout("Two");
    let draw = |ui: &mut Ui<()>, d: &Desk| {
        d.toolbar(ui, 4, 4);
        d.draw(ui, |_, _, _| {});
    };
    let hover = ["win.b.close", "win.b.collapse", "win.b.grip", "win.a.close", "desk.toggle.c", "desk.toggle.a", "desk.layout.One"];
    let mut out = shots("window", scale, &hover, |ui| draw(ui, &desk));
    desk.collapse("a");
    out.extend(shots("window.collapsed", scale, &[], |ui| draw(ui, &desk)));
    out
}

/// The sample round cut so that every kind of track is on view (the full one scrolls).
fn short_sample() -> timeline::Timeline {
    let mut t = timeline::sample();
    t.sections[0].tracks.truncate(5);
    t.sections[1].tracks.truncate(3);
    t
}

fn timeline_shots(scale: f32) -> Vec<Shot> {
    let r = Rect::new(4, 4, 632, 352);
    let draw_of = |t: &timeline::Timeline, ui: &mut Ui<()>| t.draw(ui, "tl", r, |_| (), |_| (), |_, _| ());
    let (short, full) = (short_sample(), timeline::sample());
    let hover = ["tl.block.1", "tl.track.p2", "tl.seg.p2.shot", "tl.seg.p1.strike"];
    let mut out = shots("timeline", scale, &hover, |ui| draw_of(&short, ui));
    out.extend(shots("timeline.full", scale, &[], |ui| draw_of(&full, ui)));
    out
}

const PLAYTESTS: &str = "tests/fixtures/theme/playtests";

/// The recordings screen on a folder with a stale recording (picked), a current one and an unreadable one; its message
/// in both colours; a folder with only an unreadable one (picked); no folder; the note view.
fn recordings_shots(scale: f32) -> Vec<Shot> {
    fn launch(_: &mut Cx<App>, _: &aegis_host::playtest::Session, _: Option<(Value, usize)>) -> Option<Box<dyn Screen<App>>> {
        None
    }
    let mut out = Vec::new();
    for (name, dir, message) in [
        ("recordings", PLAYTESTS, ""),
        ("recordings.copied", PLAYTESTS, "Copied to the repo."),
        ("recordings.problem", PLAYTESTS, "A problem."),
        ("recordings.unreadable", "tests/fixtures/theme/playtests_bad", ""),
        ("recordings.empty", "tests/fixtures/theme/no_such_folder", ""),
    ] {
        let mut c = cx(dir);
        let mut s = Recordings::new(&c, launch, |_| Value::Null, Vec::new());
        s.message = message.into();
        let mut g = gfx(scale);
        s.draw(&mut c, &mut g, (-1, -1), true);
        out.push((name.to_string(), g.canvas.px));
    }
    let mut view = NoteView::new(&json!({"n": 1, "tags": ["ui"], "scene": "battle", "round": 2, "text": "A note on a battle round.\n\nIts second paragraph."}));
    let mut c = cx(PLAYTESTS);
    for (name, message) in [("recordings.note", ""), ("recordings.note.copied", "Copied to the clipboard.")] {
        view.message = message.into();
        let mut g = gfx(scale);
        Screen::<App>::draw(&mut view, &mut c, &mut g, (-1, -1), true);
        out.push((name.to_string(), g.canvas.px));
    }
    out
}

/// Each sample card alone, early in its fade in and in its hold (the text out, the hint when it waits); the reel, the
/// sample and the palette built under the theme in force.
fn vignette_shots(scale: f32) -> Vec<Shot> {
    let sample = vignette::sample_reel();
    let pal = Palette::default();
    let mut out = Vec::new();
    for (k, card) in sample.cards.iter().enumerate() {
        let mut p = Player::new(Reel { cards: vec![card.clone()], ..sample.clone() }, TextSpeed::Normal);
        let mut art = Art::new(None);
        let mut frame = |name: &str, p: &Player| {
            let mut g = gfx(scale);
            {
                let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
                vignette::draw(&mut ui, &mut art, p, &pal, false);
            }
            out.push((format!("vignette.{k}.{name}"), g.canvas.px));
        };
        p.advance(100);
        frame("fade", &p);
        while p.phase() != Phase::Hold {
            p.advance(50);
        }
        frame("hold", &p);
    }
    out
}

/// The note box with its hint, then with a message.
fn notes_shots(scale: f32) -> Vec<Shot> {
    let mut out = Vec::new();
    for (name, message) in [("notes", ""), ("notes.message", "Write the note first.")] {
        let mut b = NoteBox::new(None, json!({}));
        b.message = message.into();
        let mut c = cx(PLAYTESTS);
        Screen::<App>::text_changed(&mut b, &mut c, notes::TEXT_ID, "A note.");
        let mut g = gfx(scale);
        Screen::<App>::draw(&mut b, &mut c, &mut g, (-1, -1), true);
        out.push((name.to_string(), g.canvas.px));
    }
    out
}

/// Every Gallery sample of the pip gauge, one a row.
fn pips_shots(scale: f32) -> Vec<Shot> {
    shots("pips", scale, &[], |ui| {
        for (k, (_, gauge)) in pips::samples().iter().enumerate() {
            gauge.draw(ui, &format!("pips.{k}"), 20, 16 + k as i32 * 16);
        }
    })
}

fn pane_shots(pane: &str, scale: f32) -> Vec<Shot> {
    match pane {
        "window" => window_shots(scale),
        "timeline" => timeline_shots(scale),
        "recordings" => recordings_shots(scale),
        "vignette" => vignette_shots(scale),
        "notes" => notes_shots(scale),
        "pips" => pips_shots(scale),
        _ => panic!("no pane {pane}"),
    }
}

/// FNV-1a 64.
fn hash(px: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in px {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn pane_hashes() -> Map<String, Value> {
    let mut out = Map::new();
    for pane in PANES {
        for scale in TEXT_SIZES {
            for (name, px) in pane_shots(pane, scale) {
                out.insert(format!("{name}@{scale}"), json!(hash(&px)));
            }
        }
    }
    out
}

/// Pixel identity: with no theme file every recorded frame hash matches, on every platform (so a frame draws no path or
/// operating system wording, CIPANES1). Recording (only for an intended change, then diff the file):
/// `AEGIS_RECORD_PANE_HASHES=1`.
#[test]
fn the_default_theme_draws_the_six_panes_exactly_as_before() {
    let file = root().join("tests/fixtures/theme/pane_hashes.json");
    let now = pane_hashes();
    if std::env::var("AEGIS_RECORD_PANE_HASHES").is_ok() {
        std::fs::write(&file, serde_json::to_string_pretty(&Value::Object(now)).unwrap() + "\n").unwrap();
        return;
    }
    let recorded: Map<String, Value> = serde_json::from_str(&std::fs::read_to_string(&file).expect("pane_hashes.json recorded")).unwrap();
    assert_eq!(recorded.len(), now.len(), "the same frames as recorded");
    let differ: Vec<&String> = recorded.keys().filter(|k| now.get(*k) != recorded.get(*k)).collect();
    assert!(differ.is_empty(), "frames that changed under the default theme: {differ:?}");
}

// ---- a theme reaches each pane -------------------------------------------------------------------------------------

/// The roles each pane draws with itself (the kit widgets it holds have their own suite). A role that stops changing
/// its pane has gone back to a literal.
const PANE_ROLES: [(&str, &[&str]); 6] = [
    ("window", &["card", "panel_edge", "edge", "card_hot", "title_idle", "gold", "text", "dim", "focus", "grip", "selected"]),
    ("timeline", &["ruler", "dim", "playing", "playing_edge", "card_hot", "card", "focus", "text", "edge", "selected", "lane", "lane_hatch", "lane_stripe", "impact_edge", "bg", "foe", "gold"]),
    ("recordings", &["tool_bg", "tool_pane", "gold", "dim", "edge", "text", "foe", "info", "good", "scrim", "input"]),
    ("vignette", &["stage", "mote", "text", "gold", "dim"]),
    ("notes", &["dim", "text"]),
    ("pips", &["socket", "socket_edge", "dim", "foe", "info", "gold"]),
];

/// The theme with `colors` set, loaded from a scratch asset layer as a game's would be.
fn loaded_theme(tag: &str, colors: Map<String, Value>) -> theme::Theme {
    let dir = Scratch::vacant(&format!("theme_panes_{tag}"));
    std::fs::create_dir_all(dir.join("ui")).unwrap();
    std::fs::write(dir.join(theme::FILE), json!({"version": 1, "colors": colors}).to_string()).unwrap();
    let t = theme::load(&Layers::of(vec![dir.to_path_buf()])).unwrap();
    let _ = std::fs::remove_dir_all(dir);
    t
}

/// Every role of each pane, set alone to a loud colour, changes a frame of that pane.
#[test]
fn each_role_a_pane_draws_with_changes_that_pane() {
    for (pane, roles) in PANE_ROLES {
        theme::set_theme(theme::Theme::default());
        let plain = pane_shots(pane, 1.0);
        let mut unmoved = Vec::new();
        for role in roles {
            let mut colors = Map::new();
            colors.insert(role.to_string(), json!("#ff00ff"));
            theme::set_theme(loaded_theme(pane, colors));
            if pane_shots(pane, 1.0) == plain {
                unmoved.push(*role);
            }
        }
        theme::set_theme(theme::Theme::default());
        assert!(unmoved.is_empty(), "{pane}: roles that change nothing: {unmoved:?}");
    }
}

/// A pip block's colour is the caller's (a meaning, such as the skill kind): under a theme that sets every role it stays,
/// its ticks and rim follow it alone (`mix` toward black and white), and only the chrome around it moves.
#[test]
fn a_pip_block_keeps_its_own_colour_under_any_theme() {
    let colour = [20, 200, 90, 255];
    let frame = || {
        let mut g = Gfx { canvas: Canvas::new(640, 360), view: View::fit(640, 360), fonts: Fonts::new(&font()), kit: Kit::default() };
        g.canvas.clear(BG);
        let gauge = pips::PipGauge::planned(4, &[pips::PipBlock::new(2, colour)], 12, 12);
        {
            let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
            gauge.draw(&mut ui, "g", 20, 40);
        }
        g.canvas
    };
    let px = |c: &Canvas, x: i32, y: i32| {
        let i = ((y * c.w + x) * 4) as usize;
        [c.px[i], c.px[i + 1], c.px[i + 2], c.px[i + 3]]
    };
    let plain = frame();
    let colors: Map<String, Value> = theme::ROLES.iter().map(|r| (r.to_string(), json!("#ff00ff"))).collect();
    theme::set_theme(loaded_theme("all", colors));
    let loud = frame();
    theme::set_theme(theme::Theme::default());
    let (first, free) = (pips::pip_rect(20, 40, 0), pips::pip_rect(20, 40, 3));
    let tick_x = first.x + pips::PIP_W + pips::PIP_GAP - 1;
    let cy = first.y + first.h / 2;
    assert_eq!(px(&loud, first.x + 2, cy), colour, "the capsule's fill");
    assert_eq!(px(&loud, tick_x, cy), pips::mix(colour, [0, 0, 0, 255], pips::TICK_DARK), "its tick");
    assert_eq!(px(&loud, first.x + 2, first.y), pips::mix(colour, [255, 255, 255, 255], pips::RIM_LIGHT), "its rim");
    for (x, y) in [(first.x + 2, cy), (tick_x, cy), (first.x + 2, first.y)] {
        assert_eq!(px(&loud, x, y), px(&plain, x, y));
    }
    assert_eq!(px(&loud, free.x + 2, cy), [255, 0, 255, 255], "a free socket is chrome: the theme's");
}

// ---- no colour literal left ----------------------------------------------------------------------------------------

/// The colour literals kept on purpose: "no colour" (`rgba(0, 0, 0, 0)`, an unfilled panel); pure black and white that
/// shift a caller's colour (timeline part shades and hatch, the vignette caption's darkening of the reel's frame, pip
/// ticks and rims); and the sample rounds' data colours (the timeline's and the sample reel's backdrops: a game passes
/// its own).
const KEPT: [(&str, &[&str]); 6] = [
    ("window", &["rgba(0, 0, 0, 0)"]),
    ("timeline", &["rgba(0, 0, 0, 0)", "rgba(0, 0, 0, 90)", "rgba(255, 255, 255, 70)", "rgba(0, 0, 0, 40)", "rgba(0, 0, 0, 120)", "rgb(70, 110, 170)", "rgb(60, 120, 100)", "rgb(44, 56, 78)", "rgb(160, 70, 64)", "rgb(110, 170, 255)", "rgb(60, 80, 110)"]),
    ("recordings", &[]),
    ("vignette", &["rgb(0, 0, 0)", "rgb(44, 21, 70)", "rgb(30, 35, 45)"]),
    ("notes", &[]),
    ("pips", &["[0, 0, 0, 255]", "[255, 255, 255, 255]"]),
];

/// The old colour constants of `ui`.
const CONSTANTS: [&str; 13] = ["BG", "PANEL_HI", "CARD", "MODAL", "BORDER", "FRAME", "TEXT", "DIM", "GOLD", "INFO", "FOE", "GOOD", "SELECTED"];

/// The code of a source file: comment lines and the unit tests left out.
fn code_lines(pane: &str) -> Vec<String> {
    let text = std::fs::read_to_string(root().join(format!("src/{pane}.rs"))).unwrap();
    text.lines().take_while(|l| !l.starts_with("#[cfg(test)]")).filter(|l| !l.trim_start().starts_with("//")).map(String::from).collect()
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// The numeric colour literals of a line: `rgb(1, 2, 3)`, `rgba(1, 2, 3, 4)`, `[1, 2, 3]`, `[1, 2, 3, 4]`.
fn literals(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (open, close) in [("rgb(", ')'), ("rgba(", ')'), ("[", ']')] {
        for (at, _) in line.match_indices(open) {
            if open != "[" && line[..at].chars().next_back().is_some_and(is_word) {
                continue;
            }
            let rest = &line[at + open.len()..];
            let Some(end) = rest.find(close) else { continue };
            let parts: Vec<&str> = rest[..end].split(',').map(str::trim).collect();
            let numeric = parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_hexdigit() || c == 'x'));
            if numeric && (3..=4).contains(&parts.len()) {
                out.push(line[at..at + open.len() + end + 1].to_string());
            }
        }
    }
    out
}

/// Whether `word` stands alone in the line.
fn has_word(line: &str, word: &str) -> bool {
    line.match_indices(word).any(|(at, _)| !line[..at].chars().next_back().is_some_and(is_word) && !line[at + word.len()..].chars().next().is_some_and(is_word))
}

/// The six files hold no colour constant of `ui` and no colour literal but those `KEPT` names, and each `KEPT` literal
/// is still there (a stale list hides nothing).
#[test]
fn no_colour_literal_is_left_in_the_six_files() {
    for (pane, kept) in KEPT {
        let lines = code_lines(pane);
        let found: Vec<String> = lines.iter().flat_map(|l| literals(l)).collect();
        let stray: Vec<&String> = found.iter().filter(|l| !kept.contains(&l.as_str())).collect();
        assert!(stray.is_empty(), "{pane}: colour literals: {stray:?}");
        for k in kept {
            assert!(found.iter().any(|l| l == k), "{pane}: {k} is no longer there: take it off KEPT");
        }
        for c in CONSTANTS {
            let at: Vec<&String> = lines.iter().filter(|l| has_word(l, c)).collect();
            assert!(at.is_empty(), "{pane}: the constant {c}: {at:?}");
        }
    }
}

/// The scan itself finds what it looks for.
#[test]
fn the_scan_sees_literals_and_constants() {
    assert_eq!(literals("let c = rgba(1, 2, 3, 4); f(rgb(5, 6, 7));"), ["rgb(5, 6, 7)", "rgba(1, 2, 3, 4)"]);
    assert_eq!(literals("mix(a, [0, 0, 0, 255], k); for k in [2, 5] {} rgba(c[0], c[1], c[2], 90); my_rgb(1, 2, 3)"), ["[0, 0, 0, 255]"]);
    assert!(has_word("x, GOLD)", "GOLD") && !has_word("CARD_BODY, TEXT_BOX", "CARD") && !has_word("CARD_BODY, TEXT_BOX", "TEXT"));
}
