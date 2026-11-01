//! LOOK1-E1: the runtime theme (`aegis_ui::theme`, spec LOOK1 section 3). With no theme file the Kit widgets pages
//! draw pixel for pixel as before the theme existed (their frame hashes at 1280 x 720 at the four text sizes, idle and
//! with the mouse over one widget of each kind, recorded before the first edit in `fixtures/theme/default_hashes.json`).
//! The layered merge, every validation message, nine-slice frames, the faces' size factors, the contrast ratio, the
//! frames switch, the Theme page's text audit and a themed hover border follow, each on a scratch set of asset layers
//! written by the test (engine, kit, game) under the temp folder.

use aegis_core::assets::Layers;
use aegis_ui::art::Art;
use aegis_ui::audio::Audio;
use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::painter::{Fonts, Painter, Typeface, View};
use aegis_ui::playtest::Playtests;
use aegis_ui::screen::{Context, Cx, Nav, Settings};
use aegis_ui::specimens::{KitAct, Specimens, PAGES};
use aegis_ui::text_audit;
use aegis_ui::theme::{self, contrast, parse_colour, Nine, NineFill, Theme, ROLES};
use aegis_ui::ui::{draw_tip, Gfx, Kit, Ui, Widget, BG, BODY};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn font() -> Vec<u8> {
    std::fs::read(root().join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

/// The Kit page's rect as the gallery gives it (on the 640 x 360 logical canvas).
const KIT: Rect = Rect { x: 112, y: 22, w: 522, h: 332 };
/// The page tabs and the "Last:" line at the page's foot (the pages draw above it): left out of the hashes, as a new
/// page adds a tab there.
const FOOT_H: i32 = 18;
const TEXT_SIZES: [f32; 4] = [0.85, 1.0, 1.15, 1.3];
/// The widget kinds hovered for a hash, the first of each kind on a page (not a page tab).
const HOVER_KINDS: [&str; 11] = ["button", "card", "row", "select", "dropdown", "text", "textarea", "checkbox", "section", "icon_button", "slider_step"];

fn kit_gfx(scale: f32) -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(1280, 720), view: View::fit(1280, 720), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.fonts.scale = scale;
    g.canvas.clear(BG);
    g
}

/// The page drawn as the gallery does with the mouse at `mouse` (its hover card too), and its widgets.
fn draw_page(k: &Specimens, scale: f32, mouse: (i32, i32)) -> (Gfx, Vec<Widget<KitAct>>) {
    let mut g = kit_gfx(scale);
    let w = {
        let mut ui: Ui<KitAct> = Ui::new(&mut g, mouse, true);
        k.draw(&mut ui, KIT, |a| a);
        let w = std::mem::take(&mut ui.widgets);
        draw_tip(&mut ui.p, &w, mouse);
        w
    };
    (g, w)
}

/// FNV-1a 64 over the canvas with the page's foot blanked.
fn frame_hash(g: &Gfx) -> String {
    let foot = g.view.rect(Rect::new(KIT.x, KIT.y + KIT.h - FOOT_H, KIT.w, FOOT_H));
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for y in 0..g.canvas.h {
        for x in 0..g.canvas.w {
            let i = ((y * g.canvas.w + x) * 4) as usize;
            let px = if foot.contains(x, y) { [0u8; 4] } else { [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2], g.canvas.px[i + 3]] };
            for b in px {
                h ^= b as u64;
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
    }
    format!("{h:016x}")
}

/// Every hash: each page (the four the Kit page had before the theme), each text size, idle and hovered.
fn kit_hashes() -> Map<String, Value> {
    let mut out = Map::new();
    for (page, name) in ["widgets", "timeline", "effects", "portraits"].iter().enumerate() {
        let k = Specimens { page, ..Specimens::default() };
        for scale in TEXT_SIZES {
            let (g, w) = draw_page(&k, scale, (-1, -1));
            out.insert(format!("{name}@{scale}"), json!(frame_hash(&g)));
            for kind in HOVER_KINDS {
                let Some(wd) = w.iter().find(|wd| wd.kind == kind && !wd.id.starts_with("kit.page.") && wd.rect.w > 0 && wd.rect.h > 0) else { continue };
                let at = (wd.rect.x + wd.rect.w / 2, wd.rect.y + wd.rect.h / 2);
                let (g, _) = draw_page(&k, scale, at);
                out.insert(format!("{name}@{scale}#{}", wd.id), json!(frame_hash(&g)));
            }
        }
    }
    out
}

/// (1) Pixel identity: with no theme file (the code default) every recorded frame hash matches. Recording (only
/// before the theme's first edit): `AEGIS_RECORD_THEME_HASHES=1`.
#[test]
fn the_default_theme_draws_the_kit_pages_exactly_as_before() {
    let file = root().join("tests/fixtures/theme/default_hashes.json");
    let now = kit_hashes();
    if std::env::var("AEGIS_RECORD_THEME_HASHES").is_ok() {
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, serde_json::to_string_pretty(&Value::Object(now)).unwrap() + "\n").unwrap();
        return;
    }
    let recorded: Map<String, Value> = serde_json::from_str(&std::fs::read_to_string(&file).expect("default_hashes.json recorded")).unwrap();
    assert!(recorded.len() > 40, "the fixture holds every page, size and hover");
    let differ: Vec<&String> = recorded.keys().filter(|k| now.get(*k) != recorded.get(*k)).collect();
    assert!(differ.is_empty(), "frames that changed under the default theme: {differ:?}");
}

// ---- scratch asset layers --------------------------------------------------------------------------------------

/// A fresh scratch folder holding three asset layers, in lookup order: game, kit, engine.
fn layers(name: &str) -> (PathBuf, Layers) {
    let dir = std::env::temp_dir().join(format!("aegis_theme_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let roots: Vec<PathBuf> = ["game", "kit", "engine"].iter().map(|l| dir.join(l)).collect();
    for r in &roots {
        std::fs::create_dir_all(r).unwrap();
    }
    (dir, Layers::of(roots))
}

fn write(root: &Path, rel: &str, bytes: &[u8]) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, bytes).unwrap();
}

fn write_json(root: &Path, rel: &str, v: Value) {
    write(root, rel, v.to_string().as_bytes());
}

fn write_png(root: &Path, rel: &str, img: &Image) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(p).unwrap()), img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&img.px).unwrap();
}

/// A 16 x 16 frame whose every pixel tells where it came from (red = x, green = y).
fn frame_img() -> Image {
    let mut px = Vec::new();
    for y in 0..16u8 {
        for x in 0..16u8 {
            px.extend([x * 16, y * 16, 200, 255]);
        }
    }
    Image { w: 16, h: 16, px }
}

fn img_px(img: &Image, x: i32, y: i32) -> Rgba {
    let i = ((y * img.w + x) * 4) as usize;
    [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]
}

fn canvas_px(c: &Canvas, x: i32, y: i32) -> Rgba {
    let i = ((y * c.w + x) * 4) as usize;
    [c.px[i], c.px[i + 1], c.px[i + 2], c.px[i + 3]]
}

fn errors_of(l: &Layers) -> Vec<String> {
    match theme::load(l) {
        Ok(_) => Vec::new(),
        Err(e) => e,
    }
}

// ---- (2) the merge -----------------------------------------------------------------------------------------------

/// A kit layer sets `text`, the game layer `gold` and `layout.radius`: all three apply, every other role and layout
/// value keeps the default, and both files are named as sources, kit first.
#[test]
fn layers_merge_bottom_up_and_set_only_the_keys_they_name() {
    let (_d, l) = layers("merge");
    write_json(&l.roots[1], theme::FILE, json!({"version": 1, "colors": {"text": "#102030"}}));
    write_json(&l.roots[0], theme::FILE, json!({"version": 1, "colors": {"gold": "#405060"}, "layout": {"radius": 0}}));
    let t = theme::load(&l).unwrap();
    let d = Theme::default();
    assert_eq!((t.text, t.gold, t.layout.radius), ([16, 32, 48, 255], [64, 80, 96, 255], 0.0));
    for role in ROLES.iter().filter(|r| !["text", "gold"].contains(r)) {
        assert_eq!(t.role(role), d.role(role), "{role} keeps the default");
    }
    assert_eq!(theme::Layout { radius: d.layout.radius, ..t.layout }, d.layout, "only radius changed");
    assert_eq!(t.sources, ["ui/theme.json (kit)", "ui/theme.json (game)"]);
    // The game's file wins over the kit's for a key both set.
    write_json(&l.roots[0], theme::FILE, json!({"version": 1, "colors": {"text": "#ABCDEF80"}}));
    assert_eq!(theme::load(&l).unwrap().text, [0xab, 0xcd, 0xef, 0x80], "#rrggbbaa in upper case");
}

#[test]
fn no_theme_file_is_the_default_theme() {
    let (_d, l) = layers("none");
    let t = theme::load(&l).unwrap();
    let d = Theme::default();
    assert!(ROLES.iter().all(|r| t.role(r) == d.role(r)) && t.layout == d.layout && t.frames.is_empty() && t.sources.is_empty() && t.faces == d.faces);
    assert_eq!((d.bg, d.panel, d.card, d.focus), (BG, aegis_ui::ui::MODAL, aegis_ui::ui::CARD, aegis_ui::ui::GOLD), "the defaults are the old constants");
}

// ---- (3) one test per validation error ----------------------------------------------------------------------------

/// The layers after `setup`, which must fail to load with a message holding `needle`.
fn fails_with(name: &str, setup: impl FnOnce(&Layers), needle: &str) {
    let (_d, l) = layers(name);
    setup(&l);
    let e = errors_of(&l);
    assert!(e.iter().any(|m| m.contains(needle)), "expected '{needle}' in {e:?}");
}

fn game_theme(l: &Layers, v: Value) {
    write_json(&l.roots[0], theme::FILE, v);
}

#[test]
fn error_version_2() {
    fails_with("e_version", |l| game_theme(l, json!({"version": 2})), "ui/theme.json (game): version must be 1");
}

#[test]
fn error_unknown_colour_role() {
    fails_with("e_role", |l| game_theme(l, json!({"version": 1, "colors": {"glow": "#ffffff"}})), "ui/theme.json (game): unknown colour role 'glow' (allowed: bg, panel,");
}

#[test]
fn error_colour_with_five_digits() {
    fails_with("e_colour", |l| game_theme(l, json!({"version": 1, "colors": {"text": "#12345"}})), "ui/theme.json (game): colors.text '#12345' is not a colour");
}

#[test]
fn error_game_colour_name() {
    fails_with("e_game", |l| game_theme(l, json!({"version": 1, "game_colors": {"Bad": "#ffffff"}})), "ui/theme.json (game): game colour name 'Bad' must be [a-z][a-z0-9_]{1,31}");
}

#[test]
fn error_missing_font_file() {
    fails_with("e_font", |l| game_theme(l, json!({"version": 1, "fonts": {"display": {"file": "fonts/missing.ttf"}}})), "fonts.display.file 'fonts/missing.ttf' is in no asset layer");
}

#[test]
fn error_unknown_face() {
    fails_with("e_face", |l| game_theme(l, json!({"version": 1, "fonts": {"serif": {"file": "fonts/x.ttf"}}})), "unknown font face 'serif' (allowed: body, display, mono)");
}

#[test]
fn error_layout_grid_9() {
    fails_with("e_grid", |l| game_theme(l, json!({"version": 1, "layout": {"grid": 9}})), "ui/theme.json (game): layout.grid must be a whole number 1-8");
}

#[test]
fn error_png_without_sidecar() {
    fails_with("e_nosidecar", |l| write_png(&l.roots[0], "ui/frames/panel.png", &frame_img()), "ui/frames/panel.png (game): no sidecar panel.json beside it");
}

#[test]
fn error_sidecar_without_png() {
    fails_with("e_nopng", |l| write_json(&l.roots[0], "ui/frames/panel.json", json!({"slice": [4, 4, 4, 4]})), "ui/frames/panel.json (game): no panel.png beside it");
}

#[test]
fn error_slice_wider_than_the_png() {
    let setup = |l: &Layers| {
        write_png(&l.roots[0], "ui/frames/panel.png", &frame_img());
        write_json(&l.roots[0], "ui/frames/panel.json", json!({"slice": [8, 4, 8, 4]}));
    };
    fails_with("e_slice", setup, "ui/frames/panel.json (game): slice left + right (16) must be below the PNG width 16");
}

#[test]
fn error_unknown_frame_role_file() {
    let setup = |l: &Layers| {
        write_png(&l.roots[1], "ui/frames/banner.png", &frame_img());
        write_json(&l.roots[1], "ui/frames/banner.json", json!({"slice": [4, 4, 4, 4]}));
    };
    fails_with("e_framerole", setup, "ui/frames/banner.png (kit): unknown frame role 'banner'");
}

#[test]
fn error_unknown_ink() {
    let setup = |l: &Layers| {
        write_png(&l.roots[0], "ui/frames/tip.png", &frame_img());
        write_json(&l.roots[0], "ui/frames/tip.json", json!({"slice": [4, 4, 4, 4], "ink": "paper"}));
    };
    fails_with("e_ink", setup, "ui/frames/tip.json (game): unknown ink 'paper'");
}

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

fn cx(l: Layers) -> Cx<App> {
    Cx {
        settings: Settings::load(std::env::temp_dir().join("aegis_theme_no_settings.json"), json!({})),
        art: Art::layered(l),
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

/// `theme::reload`: a good theme is put in force; a bad one keeps the theme in force and shows its problems as a
/// notice (`show_errors`).
#[test]
fn a_bad_theme_keeps_the_theme_in_force_and_shows_a_notice() {
    let (_d, l) = layers("reload");
    game_theme(&l, json!({"version": 1, "colors": {"text": "#010203"}}));
    let mut c = cx(l.clone());
    assert!(theme::reload(&mut c) && c.nav.is_empty());
    assert_eq!((theme::th().text, theme::loaded().text), ([1, 2, 3, 255], [1, 2, 3, 255]));
    game_theme(&l, json!({"version": 2, "colors": {"text": "#040506"}}));
    assert!(!theme::reload(&mut c));
    assert_eq!(theme::th().text, [1, 2, 3, 255], "the previous theme stays");
    match c.nav.pop() {
        Some(Nav::Push(s)) => assert_eq!(s.id(), "notice"),
        _ => panic!("the problems are shown as a notice"),
    }
}

// ---- (4) nine-slice frames -----------------------------------------------------------------------------------------

fn painter_canvas() -> (Canvas, Fonts) {
    let mut c = Canvas::new(1280, 720);
    c.clear([0, 0, 0, 255]);
    (c, Fonts::new(&font()))
}

/// A 16 x 16 frame (slice 4 each, scale 2) into a 40 x 20 logical rect at view scale 2: the corners are drawn at
/// slice / scale = 2 logical px, so the four 4 x 4 device-px corner blocks equal the source corners pixel for pixel
/// (spec LOOK1 3.5: at scale 2 one art px is one device px at 1280 x 720).
#[test]
fn nine_draws_the_corners_pixel_for_pixel() {
    let (mut c, mut f) = painter_canvas();
    let img = frame_img();
    let mut p = Painter { c: &mut c, v: View::fit(1280, 720), fonts: &mut f };
    assert_eq!(p.v.s, 2.0);
    p.nine(&img, &Nine::new([4; 4]), Rect::new(10, 10, 40, 20));
    let pr = p.v.rect(Rect::new(10, 10, 40, 20));
    for (ox, oy, sx, sy) in [(pr.x, pr.y, 0, 0), (pr.x + pr.w - 4, pr.y, 12, 0), (pr.x, pr.y + pr.h - 4, 0, 12), (pr.x + pr.w - 4, pr.y + pr.h - 4, 12, 12)] {
        for j in 0..4 {
            for i in 0..4 {
                assert_eq!(canvas_px(p.c, ox + i, oy + j), img_px(&img, sx + i, sy + j), "corner at ({sx}, {sy}) pixel ({i}, {j})");
            }
        }
    }
    // The centre stretches the source centre (x and y 4..12).
    let mid = canvas_px(p.c, pr.x + pr.w / 2, pr.y + pr.h / 2);
    assert!((64..192).contains(&mid[0]) && (64..192).contains(&mid[1]) && mid[2] == 200, "{mid:?}");
}

/// `center: none` leaves the centre to the role's fill (`Painter::skin` fills it first); tiled edges repeat the
/// source edge at its own size.
#[test]
fn a_centre_of_none_shows_the_fill_and_tiled_edges_repeat() {
    let (mut c, mut f) = painter_canvas();
    let img = frame_img();
    let mut p = Painter { c: &mut c, v: View::fit(1280, 720), fonts: &mut f };
    let nine = Nine { center: NineFill::None, edges: NineFill::Tile, ..Nine::new([4; 4]) };
    let frame = theme::Frame { img: std::sync::Arc::new(img.clone()), nine, source: "test".into() };
    let fill = [1, 2, 3, 255];
    let r = Rect::new(10, 10, 40, 20);
    p.skin(Some(&frame), r, fill, [9, 9, 9, 255], 2.0);
    let pr = p.v.rect(r);
    assert_eq!(canvas_px(p.c, pr.x + pr.w / 2, pr.y + pr.h / 2), fill, "the fill shows through the centre");
    // The top edge, 8 source px wide, repeats every 8 device px.
    for x in 0..16 {
        assert_eq!(canvas_px(p.c, pr.x + 4 + x, pr.y), img_px(&img, 4 + x % 8, 0), "tiled top edge at {x}");
    }
}

/// A rect smaller than two corners shrinks each corner to half the rect each way (here 3 x 3 logical, 6 x 6 device
/// px: four 3 x 3 corner blocks, sampled down from the 4 x 4 corners; nothing past the rect).
#[test]
fn a_small_rect_shrinks_the_corners() {
    let (mut c, mut f) = painter_canvas();
    let img = frame_img();
    let mut p = Painter { c: &mut c, v: View::fit(1280, 720), fonts: &mut f };
    let r = Rect::new(100, 100, 3, 3);
    p.nine(&img, &Nine::new([4; 4]), r);
    let pr = p.v.rect(r);
    assert_eq!((pr.w, pr.h), (6, 6));
    for j in 0..6 {
        for i in 0..6 {
            let (sx, sy) = (if i < 3 { i * 4 / 3 } else { 12 + (i - 3) * 4 / 3 }, if j < 3 { j * 4 / 3 } else { 12 + (j - 3) * 4 / 3 });
            assert_eq!(canvas_px(p.c, pr.x + i, pr.y + j), img_px(&img, sx, sy), "({i}, {j})");
        }
    }
    assert_eq!(canvas_px(p.c, pr.x + 6, pr.y + 2), [0, 0, 0, 255], "nothing drawn past the rect");
}

/// A loaded sidecar: the defaults (scale 2, stretch, pad = slices / scale, ink `text`) and its own keys; the
/// fallback chain.
#[test]
fn a_sidecar_loads_with_its_defaults() {
    let (_d, l) = layers("sidecar");
    write_png(&l.roots[2], "ui/frames/button.png", &frame_img());
    write_json(&l.roots[2], "ui/frames/button.json", json!({"slice": [4, 6, 4, 2]}));
    write_png(&l.roots[0], "ui/frames/tip.png", &frame_img());
    write_json(&l.roots[0], "ui/frames/tip.json", json!({"slice": [3, 3, 3, 3], "scale": 1, "edges": "tile", "center": "none", "pad": [5, 5, 5, 5], "ink": "ink"}));
    let t = theme::load(&l).unwrap();
    let b = &t.frames["button"].nine;
    assert_eq!((b.scale, b.edges, b.center, b.pad, b.ink.as_str()), (2, NineFill::Stretch, NineFill::Stretch, [2, 3, 2, 1], "text"));
    let tp = &t.frames["tip"].nine;
    assert_eq!((tp.scale, tp.edges, tp.center, tp.pad, tp.ink.as_str()), (1, NineFill::Tile, NineFill::None, [5, 5, 5, 5], "ink"));
    // Fallbacks: button_down -> button_hot -> button (the engine's file); tab_on -> button_down; ornate -> panel -> none.
    assert_eq!(t.frame_source("button_down"), "ui/frames/button.png (engine)");
    assert_eq!(t.frame_source("tab_on"), "ui/frames/button.png (engine)");
    assert_eq!(t.frame_source("ornate"), "vector");
    assert_eq!(t.ink_on("tip"), t.ink);
}

// ---- (5) faces ---------------------------------------------------------------------------------------------------

/// The display face at `scale` 1.5 (OpenSans copied into the game layer) measures 1.5 times the body width within
/// one device px; with no fonts block both measure alike.
#[test]
fn a_face_scale_multiplies_its_sizes() {
    let (_d, l) = layers("faces");
    write(&l.roots[0], "fonts/Display.ttf", &font());
    game_theme(&l, json!({"version": 1, "fonts": {"display": {"file": "fonts/Display.ttf", "scale": 1.5}}}));
    let s = "The quick brown fox jumps";
    let measure = |t: &Theme| {
        let mut c = Canvas::new(1280, 720);
        let mut f = Fonts::new(&font());
        f.sync(t);
        let mut p = Painter { c: &mut c, v: View::fit(1280, 720), fonts: &mut f };
        (p.text_w(s, BODY), p.text_w_face(Typeface::Display, s, BODY), p.text_w_face(Typeface::Mono, s, BODY), p.line_h_face(Typeface::Display, BODY) / p.line_h(BODY), p.v.s)
    };
    let t = theme::load(&l).unwrap();
    assert_eq!(t.face(Typeface::Display).file, "fonts/Display.ttf");
    let (body, display, mono, lh, scale) = measure(&t);
    assert!(((display - 1.5 * body) * scale).abs() <= 1.0, "display {display} vs 1.5 x body {body}");
    assert_eq!(mono, body, "mono is the body face");
    assert!((lh - 1.5).abs() < 1e-4);
    let (body, display, mono, _, _) = measure(&theme::load(&layers("faces_none").1).unwrap());
    assert_eq!((display, mono), (body, body));
}

// ---- (6) contrast --------------------------------------------------------------------------------------------------

#[test]
fn contrast_is_the_wcag_ratio() {
    let c = |s: &str| parse_colour(s).unwrap();
    assert!((contrast(c("#ffffff"), c("#000000")) - 21.0).abs() < 0.01);
    assert!((contrast(c("#777777"), c("#ffffff")) - 4.48).abs() < 0.01);
    let t = Theme::default();
    assert!((t.contrast(c("#ffffff80"), c("#000000")) - t.contrast(c("#808080"), c("#000000"))).abs() < 0.05, "a translucent colour is composited first");
}

// ---- (7) frames off --------------------------------------------------------------------------------------------------

/// The frame `Ui::frame` draws under `t`, as pixels.
fn framed(t: Theme) -> Vec<u8> {
    theme::set_theme(t);
    let mut g = Gfx { canvas: Canvas::new(640, 360), view: View::fit(640, 360), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.canvas.clear(BG);
    {
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        ui.frame(Rect::new(20, 20, 200, 100), "Title", false);
    }
    theme::set_theme(Theme::default());
    g.canvas.px
}

#[test]
fn frames_off_draws_the_vector_panel() {
    let (_d, l) = layers("frames_off");
    write_png(&l.roots[0], "ui/frames/panel.png", &Image { w: 16, h: 16, px: [255, 0, 0, 255].repeat(256) });
    write_json(&l.roots[0], "ui/frames/panel.json", json!({"slice": [4, 4, 4, 4]}));
    let on = theme::load(&l).unwrap();
    assert!(on.frame("panel").is_some());
    let vector = framed(Theme::default());
    assert_ne!(framed(on), vector, "the panel frame draws");
    game_theme(&l, json!({"version": 1, "frames": false}));
    let off = theme::load(&l).unwrap();
    assert!(off.frames.contains_key("panel") && off.frame("panel").is_none());
    assert_eq!(framed(off), vector, "frames: false draws the vector panel");
}

// ---- (8) the Theme page ------------------------------------------------------------------------------------------------

/// The Theme page at 1280 x 720 at each text size: every role, face and frame role is listed, the text audit is clean.
#[test]
fn the_theme_page_lists_the_theme_and_its_text_fits() {
    assert_eq!(PAGES[4], "Theme");
    let k = Specimens { page: 4, ..Specimens::default() };
    for scale in TEXT_SIZES {
        let mut g = kit_gfx(scale);
        g.fonts.audit = Some(Vec::new());
        let w = {
            let mut ui: Ui<KitAct> = Ui::new(&mut g, (-1, -1), true);
            k.draw(&mut ui, KIT, |a| a);
            ui.widgets
        };
        let ids: Vec<&str> = w.iter().map(|x| x.id.as_str()).collect();
        for role in ROLES {
            assert!(ids.contains(&format!("kit.theme.role.{role}").as_str()), "{role} listed");
        }
        for (role, _) in theme::FRAME_ROLES {
            assert!(ids.contains(&format!("kit.theme.frame16.{role}").as_str()) && ids.contains(&format!("kit.theme.frame40.{role}").as_str()), "{role} drawn");
        }
        assert!(["body", "display", "mono"].iter().all(|f| ids.contains(&format!("kit.theme.face.{f}").as_str())));
        let boxes: Vec<_> = w.iter().map(|x| (x.id.clone(), x.rect.x as f32, x.rect.y as f32, x.rect.w as f32, x.rect.h as f32)).collect();
        let issues = text_audit::issues(&g.fonts.audit.take().unwrap(), 640.0, 360.0, &boxes);
        assert_eq!(issues["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "text size {scale}: {issues}");
    }
}

/// The page's switch puts the engine default, the layers' theme or it with frames off in force.
#[test]
fn the_theme_switch_sets_the_theme_in_force() {
    theme::install(Theme { text: [1, 2, 3, 255], ..Theme::default() });
    let mut k = Specimens { page: 4, ..Specimens::default() };
    k.act(KitAct::ThemeView(0));
    assert_eq!(theme::th().text, Theme::default().text);
    k.act(KitAct::ThemeView(2));
    assert_eq!((theme::th().text, theme::th().frames_on), ([1, 2, 3, 255], false));
    k.act(KitAct::ThemeView(1));
    assert_eq!((theme::th().text, theme::th().frames_on), ([1, 2, 3, 255], true));
    // Leaving the page (or the gallery: `leave_theme`) puts the layers' theme back.
    k.act(KitAct::ThemeView(0));
    k.act(KitAct::Page(0));
    assert_eq!((k.theme_view, theme::th().text), (1, [1, 2, 3, 255]));
}

// ---- (9) a themed hover --------------------------------------------------------------------------------------------------

/// With a fixture theme the hovered button's border pixel (the middle of its left edge) is the theme's `focus`; with
/// the default it is the old gold. An inset's content rect is the layout padding inside it.
#[test]
fn a_hovered_button_takes_the_focus_colour() {
    let border = |t: Theme| {
        theme::set_theme(t);
        let mut g = Gfx { canvas: Canvas::new(640, 360), view: View::fit(640, 360), fonts: Fonts::new(&font()), kit: Kit::default() };
        g.canvas.clear(BG);
        let r = Rect::new(40, 40, 60, 16);
        let inner = {
            let mut ui: Ui<()> = Ui::new(&mut g, (r.x + 5, r.y + 5), true);
            ui.button("b", r, "Go", (), true);
            ui.inset(Rect::new(200, 40, 80, 40), false)
        };
        theme::set_theme(Theme::default());
        (canvas_px(&g.canvas, r.x, r.y + r.h / 2), inner)
    };
    let (_d, l) = layers("hover");
    game_theme(&l, json!({"version": 1, "colors": {"focus": "#ff00ff"}, "layout": {"padding": 3}}));
    let (px, inner) = border(theme::load(&l).unwrap());
    assert_eq!(px, [255, 0, 255, 255]);
    assert_eq!(inner, Rect::new(203, 43, 74, 34));
    assert_eq!(border(Theme::default()).0, aegis_ui::ui::GOLD);
}
