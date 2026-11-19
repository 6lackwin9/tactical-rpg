//! LOOK1-E1B for the kit's panes (ENGINE-5 moved them from the engine with their hashes): the vignette player and the
//! pip gauge read the theme (`theme::th()`). With no theme file each draws pixel for pixel as before the migration (the
//! frame hashes the engine recorded, `fixtures/theme/pane_hashes.json`, split unchanged from the engine's file). Each
//! role a pane draws with changes it when a theme file sets it, a pip block keeps its caller's colour under any theme,
//! and a source scan finds no colour literal left in the two files but those kept on purpose (`KEPT`).

use aegis_core::assets::Layers;
use aegis_core::scratch::Scratch;
use aegis_ui::art::Art;
use aegis_ui::canvas::{Canvas, Rect};
use aegis_ui::painter::{Fonts, View};
use aegis_ui::theme;
use aegis_ui::ui::{Gfx, Kit, Ui, BG};
use rpgkit_ui::pips;
use rpgkit_ui::vignette::{self, Palette, Phase, Player, Reel, TextSpeed};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn font() -> Vec<u8> {
    std::fs::read(root().join("../../../../engine/assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

const TEXT_SIZES: [f32; 4] = [0.85, 1.0, 1.15, 1.3];
const PANES: [&str; 2] = ["vignette", "pips"];

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
        "vignette" => vignette_shots(scale),
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

#[test]
fn the_default_theme_draws_the_kit_panes_exactly_as_before() {
    let mut now = Map::new();
    for pane in PANES {
        for scale in TEXT_SIZES {
            for (name, px) in pane_shots(pane, scale) {
                now.insert(format!("{name}@{scale}"), json!(hash(&px)));
            }
        }
    }
    let file = root().join("tests/fixtures/theme/pane_hashes.json");
    // Re-recording (only for an intended change, named in the commit; then diff the file): `AEGIS_RECORD_PANE_HASHES=1`.
    // ENGINE-13 re-recorded the line card's 8 frames (`vignette.3.*`: its portrait on the well inside the F2 frame).
    if std::env::var("AEGIS_RECORD_PANE_HASHES").is_ok() {
        std::fs::write(&file, serde_json::to_string_pretty(&Value::Object(now)).unwrap() + "\n").unwrap();
        return;
    }
    let recorded: Map<String, Value> = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(recorded.len(), now.len(), "the same frames as recorded");
    let differ: Vec<&String> = recorded.keys().filter(|k| now.get(*k) != recorded.get(*k)).collect();
    assert!(differ.is_empty(), "frames that changed under the default theme: {differ:?}");
}

const PANE_ROLES: [(&str, &[&str]); 2] = [("vignette", &["stage", "mote", "text", "gold", "dim"]), ("pips", &["socket", "socket_edge", "dim", "foe", "info", "gold"])];

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
fn each_role_a_kit_pane_draws_with_changes_that_pane() {
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

/// The colour literals kept on purpose: pure black and white that shift a caller's colour (the vignette caption's
/// darkening of the reel's frame, pip ticks and rims) and the sample reel's backdrops (a game passes its own).
const KEPT: [(&str, &[&str]); 2] = [("vignette", &["rgb(0, 0, 0)", "rgb(44, 21, 70)", "rgb(30, 35, 45)"]), ("pips", &["[0, 0, 0, 255]", "[255, 255, 255, 255]"])];

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

/// The two files hold no colour constant of `ui` and no colour literal but those `KEPT` names, and each `KEPT` literal
/// is still there.
#[test]
fn no_colour_literal_is_left_in_the_kit_panes() {
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

/// `AEGIS_SHOT_DIR=<folder>`: the line card in its hold at Normal text, as a PNG, for review (no check).
#[test]
fn the_line_card_shot_for_review() {
    let Ok(dir) = std::env::var("AEGIS_SHOT_DIR") else { return };
    let (_, px) = vignette_shots(1.0).into_iter().find(|(n, _)| n == "vignette.3.hold").expect("the line card");
    let path = Path::new(&dir).join("vignette_line_card.png");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), 1280, 720);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&px).unwrap();
}
