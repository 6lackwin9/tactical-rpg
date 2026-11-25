//! LAYOUT720-E5, `layout_audit` (docs/specs/LAYOUT720.md section 10): a test per kind, on records and on what the kit
//! really records while it draws; the profiles; and the layout specimen, its subject, which must pass `desktop` and resolve
//! on every profile at the four text sizes. Relations and rules, not numbers. The shell's commands: `shell.rs`' tests.

use aegis_layout::Rect;
use aegis_ui::art::Art;
use aegis_ui::audio::Audio;
use aegis_ui::canvas::{Canvas, Image};
use aegis_ui::layout_audit::{self, issues, profile_named, LayoutRec, ProfilePolicy, KINDS, LIST_MAX, NO_MOUSE, PROFILES};
use aegis_ui::layout_specimen::LayoutSpecimen;
use aegis_ui::painter::{CanvasPolicy, Fonts, Grid, View, GRID};
use aegis_ui::screen::{Context, Cx, Screen, Settings, TEXT_SIZES};
use aegis_ui::theme::{self, Frame, Metrics, Nine, Theme};
use aegis_ui::ui::{tip, Gfx, Kit, Pointer, Ui};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;

const CANVAS: (i32, i32) = GRID;

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

fn cx() -> Cx<App> {
    Cx {
        settings: Settings::load(std::env::temp_dir().join("aegis_no_settings.json"), json!({})),
        art: Art::new(None),
        audio: Audio::offline(None),
        nav: Vec::new(),
        notice: Vec::new(),
        shot_dir: std::env::temp_dir(),
        quit: false,
        answers: Vec::new(),
        playtest: aegis_ui::playtest::Playtests::off(),
        held: Default::default(),
        app: App,
    }
}

/// A native 1280 x 720 window with the audit off.
fn gfx() -> Gfx {
    let font = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    Gfx { canvas: Canvas::new(1280, 720), view: View::fit(1280, 720).for_grid(Grid::Native), fonts: Fonts::new(&font), kit: Kit::default() }
}

fn count(v: &Value, kind: &str) -> u64 {
    v["counts"][kind].as_u64().unwrap_or_else(|| panic!("no count of {kind}: {v}"))
}

fn only_kinds(v: &Value, kinds: &[&str]) {
    for k in KINDS {
        assert_eq!(count(v, k) > 0, kinds.contains(&k), "{k}: {v}");
    }
    assert_eq!(v["clean"], json!(kinds.is_empty()), "{v}");
}

fn widget(id: &str, kind: &str, r: [i32; 4], enabled: bool, clickable: bool, tips: &[&str]) -> Value {
    json!({"id": id, "kind": kind, "rect": r, "enabled": enabled, "clickable": clickable, "tip": tips})
}

/// Draws with a `Ui` on a native 1280 x 720 canvas with the audit on, and reads what it recorded.
fn drawn(what: impl FnOnce(&mut Ui<()>)) -> Value {
    let mut g = gfx();
    g.fonts.layout = Some(Vec::new());
    let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
    what(&mut ui);
    let widgets: Vec<Value> = ui.widgets.iter().map(|w| w.info()).collect();
    let recs = g.fonts.layout.take().unwrap();
    issues(&recs, &widgets, CANVAS)
}

#[test]
fn a_clean_frame_says_so_and_names_every_kind() {
    let v = issues(&[], &[widget("ok", "button", [0, 0, CANVAS.0, CANVAS.1], true, true, &["a tip it can focus"])], CANVAS);
    only_kinds(&v, &[]);
    assert_eq!(v["counts"].as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>(), KINDS.to_vec());
    assert_eq!((v["audited"].clone(), v["canvas"].clone(), v["widgets"].clone(), v["issues"].clone()), (json!(true), json!(CANVAS), json!(1), json!([])));
}

#[test]
fn overflow_reports_how_much_did_not_fit() {
    let area = Rect::new(10, 10, 100, 50);
    let v = issues(&[LayoutRec::Overflow { what: "list".into(), at: area, by: 30 }], &[], CANVAS);
    only_kinds(&v, &["overflow"]);
    let i = &v["issues"][0];
    assert_eq!((i["kind"].clone(), i["what"].clone(), i["at"].clone(), i["by"].clone()), (json!("overflow"), json!("list"), json!([10, 10, 100, 50]), json!(30)));
    assert!(i["why"].as_str().unwrap().contains("30"), "{i}");
    only_kinds(&issues(&[LayoutRec::Overflow { what: "list".into(), at: area, by: 0 }], &[], CANVAS), &[]);
    // From a real stack: three rows of 30 in 50, with no gap: 40 past.
    let v = drawn(|ui| {
        let mut rows = aegis_layout::Stack::new(area, 0);
        for _ in 0..3 {
            rows.next(30);
        }
        ui.report_overflow("rows", area, rows.overflow());
    });
    only_kinds(&v, &["overflow"]);
    assert_eq!(v["issues"][0]["by"], json!(40), "{v}");
    only_kinds(&drawn(|ui| ui.report_overflow("rows", area, 0)), &[]);
}

#[test]
fn off_canvas_is_past_any_edge_and_says_by_how_much() {
    let (w, h) = CANVAS;
    let inside = [[0, 0, w, h], [w - 1, h - 1, 1, 1], [0, 0, 0, 0]];
    for r in inside {
        assert_eq!(count(&issues(&[], &[widget("a", "row", r, true, false, &[])], CANVAS), "off_canvas"), 0, "{r:?}");
    }
    for (r, by) in [([-3, 0, 10, 10], 3), ([0, -4, 10, 10], 4), ([w - 5, 0, 10, 10], 5), ([0, h - 6, 10, 10], 4), ([w + 100, 0, 10, 10], 110)] {
        let v = issues(&[], &[widget("a", "row", r, true, false, &[])], CANVAS);
        only_kinds(&v, &["off_canvas"]);
        assert_eq!((v["issues"][0]["what"].clone(), v["issues"][0]["by"].clone()), (json!("a"), json!(by)), "{r:?}");
    }
    // A frame and an image are placed with no widget of their own.
    let v = drawn(|ui| {
        ui.frame_body("panel", Rect::new(w - 10, 0, 40, 40));
    });
    only_kinds(&v, &["off_canvas"]);
    assert_eq!(v["issues"][0]["what"], json!("panel"), "{v}");
    let img = Image { w: 2, h: 2, px: vec![255; 16] };
    let v = drawn(|ui| ui.image_in(&img, Rect::new(0, h - 1, 4, 4)));
    assert_eq!((count(&v, "off_canvas"), v["issues"][0]["what"].clone()), (1, json!("image")), "{v}");
}

#[test]
fn zero_size_is_a_widget_frame_body_or_image_with_no_room() {
    let v = drawn(|ui| ui.push("gone", "row", "", Rect::new(40, 40, 0, 20), true, false, None, Vec::new()));
    only_kinds(&v, &["zero_size"]);
    assert_eq!(v["issues"][0]["what"], json!("gone"), "{v}");
    let v = drawn(|ui| {
        ui.frame_body("panel", Rect::new(10, 10, 20, 0));
    });
    only_kinds(&v, &["zero_size"]);
    assert_eq!(v["issues"][0]["what"], json!("panel"), "{v}");
    let img = Image { w: 2, h: 2, px: vec![255; 16] };
    only_kinds(&drawn(|ui| ui.image_in(&img, Rect::new(5, 5, 0, 0))), &["zero_size"]);
    // A panel that has room, but too little for its padding, has no room for its content.
    let v = drawn(|ui| {
        let pad = ui.m().padding;
        ui.frame_body("panel", Rect::new(10, 10, pad, pad));
    });
    only_kinds(&v, &["zero_size"]);
    assert_eq!(v["issues"][0]["what"], json!("panel body"), "{v}");
}

#[test]
fn a_row_scrolled_out_of_its_area_is_not_a_problem() {
    let v = drawn(|ui| {
        ui.push_clip(Rect::new(0, 0, 100, 100));
        ui.push("below", "row", "", Rect::new(0, 500, 100, 20), true, false, None, tip("A tip", "shown when it scrolls in"));
        ui.pop_clip();
    });
    only_kinds(&v, &[]);
}

/// A frame whose corners want `size` units each.
fn framed(straddle: bool) -> Theme {
    let mut t = Theme::default();
    // On the native grid, scale 2 is 1 art px per unit at 720p, and 1 physical px per unit: 16 px corners.
    let nine = Nine { straddle, corner_anchor: straddle.then_some([8, 8]), scale: 2, ..Nine::new([16; 4]) };
    t.frames.insert("panel".into(), Frame { img: Arc::new(Image { w: 48, h: 48, px: vec![200; 48 * 48 * 4] }), nine, source: String::new() });
    t
}

#[test]
fn corner_shrunk_is_a_panel_too_small_for_its_corners() {
    for straddle in [false, true] {
        theme::set_theme(framed(straddle));
        // Room for both corners on each side: nothing to say.
        only_kinds(&drawn(|ui| {
            ui.frame_body("panel", Rect::new(100, 100, 300, 300));
        }), &[]);
        // Short: the corners draw smaller.
        let v = drawn(|ui| {
            ui.push("holder", "row", "", Rect::new(90, 90, 300, 100), true, false, None, Vec::new());
            ui.frame_body("panel", Rect::new(100, 100, 300, 12));
        });
        assert_eq!(count(&v, "corner_shrunk"), 1, "straddle {straddle}: {v}");
        let i = v["issues"].as_array().unwrap().iter().find(|i| i["kind"] == "corner_shrunk").unwrap();
        assert_eq!((i["what"].clone(), i["widget"].clone()), (json!("panel"), json!("holder")), "the frame's role, and the widget at its middle: {i}");
        let share = i["share"].as_f64().unwrap();
        assert!(share > 0.0 && share < 1.0, "straddle {straddle}: {i}");
    }
    theme::set_theme(Theme::default());
    // No frame file: a vector panel has no corners to shrink.
    only_kinds(&drawn(|ui| {
        ui.frame_body("panel", Rect::new(100, 100, 300, 2 * ui.m().padding + 1));
    }), &[]);
}

#[test]
fn hover_only_is_a_tip_nothing_but_a_mouse_can_reach() {
    let tips = ["Why this matters"];
    let on = [0, 0, 40, 20];
    for (what, w) in [("a card with no action", widget("a", "panel", on, true, false, &tips)), ("a disabled button", widget("a", "button", on, false, true, &tips))] {
        let v = issues(&[], &[w], CANVAS);
        only_kinds(&v, &["hover_only"]);
        assert!(v["issues"][0]["why"].as_str().unwrap().contains("Why this matters"), "{what}: {v}");
    }
    for (what, w) in [
        ("a button takes the focus", widget("a", "button", on, true, true, &tips)),
        ("a select takes the focus", widget("a", "select", on, true, false, &tips)),
        ("no tip", widget("a", "panel", on, true, false, &[])),
        ("a row scrolled out shows no tip", widget("a", "panel", [0, 0, 0, 0], true, false, &tips)),
    ] {
        assert_eq!(issues(&[], &[w], CANVAS)["clean"], json!(true), "{what}");
    }
}

#[test]
fn issues_come_in_kind_order_with_complete_counts_and_a_bounded_list() {
    let many: Vec<Value> = (0..LIST_MAX as i32 + 7).map(|i| widget(&format!("w{i}"), "row", [CANVAS.0, i, 5, 1], true, false, &[])).collect();
    let recs = vec![LayoutRec::Overflow { what: "last in, first out".into(), at: Rect::new(0, 0, 9, 9), by: 1 }];
    let v = issues(&recs, &many, CANVAS);
    assert_eq!(count(&v, "off_canvas"), many.len() as u64, "the count is complete");
    assert_eq!(v["issues"].as_array().unwrap().len(), LIST_MAX, "the list is bounded");
    assert_eq!(v["issues"][0]["kind"], json!("overflow"), "{}", v["issues"][0]);
    assert!(v["issues"].as_array().unwrap().iter().all(|i| ["kind", "what", "at", "why"].iter().all(|k| !i[k].is_null())), "every issue reads the same way");
}

#[test]
fn with_the_audit_off_the_picture_is_the_same_and_nothing_is_kept() {
    theme::set_theme(framed(false));
    let shot = |audit: bool| {
        let mut g = gfx();
        g.fonts.layout = audit.then(Vec::new);
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        ui.frame_body("panel", Rect::new(100, 100, 300, 12));
        ui.report_overflow("rows", Rect::new(0, 0, 10, 10), 5);
        ui.push("gone", "row", "", Rect::new(0, 0, 0, 0), true, false, None, Vec::new());
        drop(ui);
        (g.canvas.px.clone(), g.fonts.layout.map_or(0, |r| r.len()))
    };
    let (on, off) = (shot(true), shot(false));
    assert!(on.0 == off.0, "the audit only reads");
    assert!(on.1 >= 3, "recorded: {}", on.1);
    assert_eq!(off.1, 0);
    theme::set_theme(Theme::default());
}

#[test]
fn the_profiles_are_the_specs_table() {
    let names: Vec<&str> = PROFILES.iter().map(|p| p.name).collect();
    assert_eq!(names, ["desktop", "wide", "ultrawide", "handheld", "browser_small", "phone_landscape", "phone_portrait"]);
    let must: Vec<&str> = PROFILES.iter().filter(|p| p.must_pass).map(|p| p.name).collect();
    assert_eq!(must, ["desktop"], "the others only report, until the minimums exist");
    let game = CanvasPolicy::default();
    // (name, canvas from the table's rule, pointer)
    for (name, canvas, pointer) in [
        ("desktop", GRID, Pointer::Mouse),
        ("wide", (1280, 800), Pointer::Mouse),
        ("ultrawide", (1720, 720), Pointer::Mouse),
        ("handheld", (853, 533), Pointer::Touch),
        ("browser_small", (800, 600), Pointer::Mouse),
        ("phone_landscape", (844, 390), Pointer::Touch),
        ("phone_portrait", (390, 844), Pointer::Touch),
    ] {
        let p = profile_named(name).unwrap();
        assert_eq!((p.view(game).canvas(), p.pointer), (canvas, pointer), "{name}");
    }
    assert!(profile_named("watch").unwrap_err().contains("desktop"), "says what there is");
}

#[test]
fn a_profile_keeps_a_games_own_canvas() {
    let own = CanvasPolicy::Fixed { w: 1920, h: 1080 };
    for p in PROFILES {
        let want = match p.policy {
            ProfilePolicy::Fixed => own,
            ProfilePolicy::Fit => CanvasPolicy::Fit { w: 1920, h: 1080 },
            ProfilePolicy::Dp => CanvasPolicy::Dp,
        };
        assert_eq!(p.policy_for(own), want, "{}", p.name);
    }
    assert_eq!(profile_named("desktop").unwrap().view(own).canvas(), (1920, 1080));
    assert_eq!(profile_named("desktop").unwrap().policy_for(CanvasPolicy::Dp), CanvasPolicy::default(), "a dp game's desktop is the fixed grid");
}

#[test]
fn entering_a_profile_and_leaving_puts_everything_back() {
    let mut g = gfx();
    g.kit.device.dpr = 1.25;
    let (view, device) = (g.view, g.kit.device);
    let p = profile_named("phone_portrait").unwrap();
    let entered = p.enter(&mut g, CanvasPolicy::default());
    assert_eq!(((g.canvas.w, g.canvas.h), g.view.canvas(), g.kit.device.pointer, g.kit.device.dpr), ((1170, 2532), (390, 844), Pointer::Touch, 3.0));
    entered.leave(&mut g);
    assert_eq!(((g.canvas.w, g.canvas.h), g.view, g.kit.device), ((1280, 720), view, device));
}

/// What a frame of the specimen left.
struct Specimen {
    widgets: Vec<Value>,
    state: Value,
    audit: Value,
    view: View,
}

/// A frame of the specimen on a window under `policy`.
fn specimen_on(policy: CanvasPolicy, window: (u32, u32), dpr: f32, ui_scale: f32, text: f32) -> Specimen {
    let (mut cx, mut g) = (cx(), gfx());
    g.canvas = Canvas::new(window.0 as i32, window.1 as i32);
    g.view = View::native(policy, window.0, window.1, dpr, ui_scale);
    g.fonts.scale = text;
    g.fonts.layout = Some(Vec::new());
    let mut s = LayoutSpecimen::default();
    Screen::<App>::draw(&mut s, &mut cx, &mut g, NO_MOUSE, true);
    let widgets = Screen::<App>::widgets(&s);
    let recs = g.fonts.layout.take().unwrap();
    let audit = issues(&recs, &widgets, g.view.canvas());
    Specimen { state: Screen::<App>::state(&s, &cx), widgets, audit, view: g.view }
}

fn rect(widgets: &[Value], id: &str) -> Rect {
    let w = widgets.iter().find(|w| w["id"] == id).unwrap_or_else(|| panic!("no {id}"));
    let n = |i: usize| w["rect"][i].as_i64().unwrap() as i32;
    Rect::new(n(0), n(1), n(2), n(3))
}

#[test]
fn the_specimen_passes_desktop_at_every_text_size() {
    let (mut cx, mut g) = (cx(), gfx());
    let mut s = LayoutSpecimen::default();
    let v = layout_audit::audit_screen(&mut s, &mut cx, &mut g, CanvasPolicy::default(), NO_MOUSE, Some(profile_named("desktop").unwrap()));
    assert_eq!(v["pass"], json!(true), "{v}");
    let p = &v["profiles"][0];
    assert_eq!((p["profile"].clone(), p["must_pass"].clone(), p["canvas"].clone(), p["classes"].clone()), (json!("desktop"), json!(true), json!(GRID), json!(["expanded", "expanded"])));
    assert_eq!(p["issues"], json!([]), "{p}");
    for (k, n) in p["text_counts"].as_object().unwrap() {
        assert_eq!(n, &json!(0), "the text audit's {k}, over the four text sizes: {p}");
    }
}

#[test]
fn every_profile_resolves_the_specimen_at_every_text_size() {
    let (mut cx, mut g) = (cx(), gfx());
    let mut s = LayoutSpecimen::default();
    let (before, view) = ((g.canvas.w, g.canvas.h), g.view);
    let v = layout_audit::audit_screen(&mut s, &mut cx, &mut g, CanvasPolicy::default(), NO_MOUSE, None);
    assert_eq!(v["profiles"].as_array().unwrap().len(), PROFILES.len());
    assert_eq!(((g.canvas.w, g.canvas.h), g.view, g.fonts.scale), (before, view, 1.0), "the window is left as it was found");
    assert_eq!(v["pass"], json!(true), "desktop passes, whatever the others report: {v}");
    for (p, want) in v["profiles"].as_array().unwrap().iter().zip(PROFILES) {
        let (cw, ch) = want.view(CanvasPolicy::default()).canvas();
        assert_eq!((p["profile"].clone(), p["canvas"].clone()), (json!(want.name), json!([cw, ch])));
        assert!(p["counts"].as_object().unwrap().keys().eq(KINDS.iter().map(|k| k.to_string()).collect::<Vec<_>>().iter()), "{p}");
        for i in p["issues"].as_array().unwrap() {
            assert!(!i["text_sizes"].as_array().unwrap().is_empty(), "{i}");
        }
    }
}

#[test]
fn the_size_classes_follow_the_canvas() {
    let classes: Vec<(String, [String; 2])> = {
        let (mut cx, mut g) = (cx(), gfx());
        let mut s = LayoutSpecimen::default();
        let v = layout_audit::audit_screen(&mut s, &mut cx, &mut g, CanvasPolicy::default(), NO_MOUSE, None);
        v["profiles"].as_array().unwrap().iter().map(|p| (p["profile"].as_str().unwrap().to_string(), [p["classes"][0].as_str().unwrap().to_string(), p["classes"][1].as_str().unwrap().to_string()])).collect()
    };
    let limits = Metrics::of(&Theme::default(), View::fit(1, 1)).breakpoints;
    let class = |units: i32, [c, m]: [i32; 2]| if units < c { "compact" } else if units < m { "medium" } else { "expanded" };
    for (name, got) in classes {
        let (cw, ch) = profile_named(&name).unwrap().view(CanvasPolicy::default()).canvas();
        assert_eq!(got, [class(cw, limits.width).to_string(), class(ch, limits.height).to_string()], "{name}");
    }
}

#[test]
fn the_specimen_picks_an_arrangement_from_the_canvas() {
    let (dp, fit) = (CanvasPolicy::Dp, CanvasPolicy::Fit { w: GRID.0, h: GRID.1 });
    let arrange = |policy, window, dpr, ui| specimen_on(policy, window, dpr, ui, 1.0).state["arrange"].clone();
    assert_eq!(arrange(CanvasPolicy::default(), (1280, 720), 1.0, 1.0), json!("grid"));
    assert_eq!(arrange(fit, (3440, 1440), 1.0, 1.0), json!("grid"));
    assert_eq!(arrange(dp, (390, 844), 1.0, 1.0), json!("rows"), "a narrow tall canvas needs no special case");
    assert_eq!(arrange(dp, (500, 300), 1.0, 1.0), json!("stacked"));
    // `pick` reads any rect, so a pane rearranges inside itself.
    let inner = |window| specimen_on(dp, window, 1.0, 1.0, 1.0).state["pick"].clone();
    assert_eq!(inner((800, 600)), json!("grid"));
    assert_eq!(inner((300, 800)), json!("rows"));
}

#[test]
fn a_canvas_too_small_for_the_specimen_reports_it_and_never_panics() {
    let small = specimen_on(CanvasPolicy::Dp, (500, 300), 1.0, 1.0, 1.0);
    assert_eq!(small.state["arrange"], json!("stacked"));
    assert!(small.state["overflow"].as_i64().unwrap() > 0, "{}", small.state);
    assert!(count(&small.audit, "overflow") > 0, "{}", small.audit);
    assert!(small.audit["issues"].as_array().unwrap().iter().any(|i| i["what"] == "layout.sections"), "{}", small.audit);
    assert!(!small.widgets.is_empty());
    for (pw, ph) in [(1, 1), (2, 400), (400, 2), (40, 40), (120, 80), (200, 100)] {
        for (_, text) in TEXT_SIZES {
            let run = specimen_on(CanvasPolicy::Dp, (pw, ph), 1.0, 1.0, text as f32);
            assert_eq!(run.audit["audited"], json!(true), "{pw}x{ph}");
            for w in &run.widgets {
                let r = rect(std::slice::from_ref(w), w["id"].as_str().unwrap());
                assert!(r.w >= 0 && r.h >= 0, "{pw}x{ph}: never negative: {w}");
            }
        }
    }
}

#[test]
fn the_specimens_cells_keep_their_relations_on_every_canvas() {
    for (policy, window, dpr, ui) in [
        (CanvasPolicy::default(), (1280, 720), 1.0, 1.0),
        (CanvasPolicy::default(), (1920, 1080), 1.0, 1.0),
        (CanvasPolicy::Fit { w: GRID.0, h: GRID.1 }, (3440, 1440), 1.0, 1.0),
        (CanvasPolicy::Fit { w: GRID.0, h: GRID.1 }, (1920, 1200), 1.0, 1.0),
        (CanvasPolicy::Dp, (800, 600), 1.0, 1.0),
    ] {
        let Specimen { widgets: w, view, .. } = specimen_on(policy, window, dpr, ui, 1.0);
        let what = format!("{policy:?} {window:?}");
        let (cw, ch) = view.canvas();
        for widget in &w {
            let r = rect(std::slice::from_ref(widget), widget["id"].as_str().unwrap());
            assert!(r.x >= 0 && r.y >= 0 && r.x + r.w <= cw && r.y + r.h <= ch, "{what}: {widget}");
        }
        // Anchors: left of, above, and in line with one another.
        let at = |name: &str| rect(&w, &format!("layout.anchor.{name}"));
        assert!(at("top_left").x == at("left").x && at("left").x == at("bottom_left").x, "{what}");
        assert!(at("top_right").x + at("top_right").w == at("bottom_right").x + at("bottom_right").w, "{what}");
        assert!(at("top_left").y == at("top").y && at("top").y == at("top_right").y, "{what}");
        assert!(at("top").y < at("center").y && at("center").y < at("bottom").y, "{what}");
        assert!(at("left").x < at("center").x && at("center").x < at("right").x, "{what}");
        // Shares: per mille of what is left after the gaps, in order, one gutter apart.
        let m = Metrics::of(&Theme::default(), view);
        let (a, b, c) = (rect(&w, "layout.share.186"), rect(&w, "layout.share.314"), rect(&w, "layout.share.500"));
        assert_eq!((a.x + a.w + m.gutter, b.x + b.w + m.gutter), (b.x, c.x), "{what}");
        let room = c.x + c.w - a.x - 2 * m.gutter;
        assert_eq!((a.w, b.w), (room * 186 / 1000, room * (186 + 314) / 1000 - room * 186 / 1000), "{what}: shared edges");
        // Fills: a token first, then the weights by shared edges, ending on the same edge as the shares.
        let (token, two, one) = (rect(&w, "layout.fill.token"), rect(&w, "layout.fill.2"), rect(&w, "layout.fill.1"));
        assert_eq!(token.w, m.u(30), "{what}");
        assert_eq!((token.x, one.x + one.w), (a.x, c.x + c.w), "{what}");
        assert!((two.w - 2 * one.w).abs() <= 2, "{what}: {} and {}", two.w, one.w);
    }
}

#[test]
fn a_game_on_another_canvas_writes_nothing_new() {
    // The whole table on a 1920 wide grid: a different `g`, tokens with it, and the specimen still passes at its base window.
    let own = CanvasPolicy::Fixed { w: 1920, h: 1080 };
    let (mut cx, mut g) = (cx(), gfx());
    let mut s = LayoutSpecimen::default();
    let v = layout_audit::audit_screen(&mut s, &mut cx, &mut g, own, NO_MOUSE, Some(profile_named("desktop").unwrap()));
    assert_eq!((v["pass"].clone(), v["profiles"][0]["canvas"].clone()), (json!(true), json!([1920, 1080])), "{v}");
    let run = specimen_on(own, (1920, 1080), 1.0, 1.0, 1.0);
    assert_eq!((run.view.g, run.view.s), (3, 1.0));
    assert!(rect(&run.widgets, "layout.close").h > 0);
}

/// A native screen of `rows` widgets past the canvas's edge, and two frames of one role at different places.
struct Spills {
    rows: i32,
    w: Vec<aegis_ui::ui::Widget<()>>,
}

impl Spills {
    fn act(&mut self, _: &mut Cx<App>, _: ()) {}
}

impl Screen<App> for Spills {
    fn id(&self) -> &'static str {
        "spills"
    }
    fn grid(&self) -> Grid {
        Grid::Native
    }
    fn draw(&mut self, _: &mut Cx<App>, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        let mut ui: Ui<()> = Ui::new(g, mouse, live);
        let (cw, ch) = ui.p.v.canvas();
        for i in 0..self.rows {
            ui.push(format!("row.{i}"), "row", "", Rect::new(cw - 1, i, 5, 1), true, false, None, Vec::new());
        }
        ui.frame_body("panel", Rect::new(-9, 0, 40, 40));
        ui.frame_body("panel", Rect::new(0, ch - 9, 40, 40));
        self.w = ui.widgets;
    }
    aegis_ui::widget_plumbing!(Cx<App>);
}

fn sweep_of(rows: i32) -> Value {
    let (mut cx, mut g) = (cx(), gfx());
    let mut s = Spills { rows, w: Vec::new() };
    let v = layout_audit::audit_screen(&mut s, &mut cx, &mut g, CanvasPolicy::default(), NO_MOUSE, Some(profile_named("desktop").unwrap()));
    v["profiles"][0].clone()
}

#[test]
fn a_sweeps_counts_are_complete_and_its_list_bounded() {
    let rows = LIST_MAX as i32 + 25;
    let p = sweep_of(rows);
    assert_eq!(p["counts"]["off_canvas"], json!(rows + 2), "every row and both frames, past what one frame lists: {}", p["counts"]);
    assert_eq!(p["issues"].as_array().unwrap().len(), LIST_MAX);
    assert_eq!(p["clean"], json!(false));
}

#[test]
fn two_problems_of_one_name_at_two_places_stay_two_lines() {
    let p = sweep_of(0);
    assert_eq!(p["counts"]["off_canvas"], json!(2), "{p}");
    let lines = p["issues"].as_array().unwrap();
    assert_eq!(lines.len(), 2, "{p}");
    assert!(lines.iter().all(|i| i["what"] == "panel"), "{p}");
    assert_ne!(lines[0]["at"], lines[1]["at"]);
    for line in lines {
        assert_eq!(line["text_sizes"].as_array().unwrap().len(), TEXT_SIZES.len(), "the same place at each text size is one line: {line}");
    }
}

#[test]
fn an_old_screen_is_not_audited() {
    struct Old;
    impl Screen<App> for Old {
        fn id(&self) -> &'static str {
            "old"
        }
        fn draw(&mut self, _: &mut Cx<App>, _: &mut Gfx, _: (i32, i32), _: bool) {
            panic!("an old screen is not drawn for the audit");
        }
        fn widgets(&self) -> Vec<Value> {
            Vec::new()
        }
        fn draw_tip(&self, _: &mut aegis_ui::painter::Painter, _: (i32, i32)) {}
        fn click(&mut self, _: &mut Cx<App>, _: (i32, i32)) -> bool {
            false
        }
        fn click_id(&mut self, _: &mut Cx<App>, _: &str) -> Result<(), String> {
            Ok(())
        }
        fn widget_rect(&self, _: &str) -> Option<(Rect, Vec<String>)> {
            None
        }
    }
    let (mut cx, mut g) = (cx(), gfx());
    let v = layout_audit::audit_screen(&mut Old, &mut cx, &mut g, CanvasPolicy::default(), NO_MOUSE, None);
    assert_eq!(v["audited"], json!(false), "{v}");
    assert!(v["why"].as_str().unwrap().contains("old"), "{v}");
}
