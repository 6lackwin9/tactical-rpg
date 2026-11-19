//! PR7-36 (STYLE-PASS1 item 9, layout notes UX-01..07): the shared building blocks of `aegis_ui::panes`: the panel
//! shell (title, close at the top right, Back at the footer's left, the main action at its right as the primary, key
//! hints between, sized to its body and centred over the dimmed scene, one per screen), the list row (32 px icon tile,
//! name, grey sub-line, numbers right-aligned), the four-step type scale, the four colour roles and the tooltip (beside
//! the pointer, never over what it explains, at most three lines); and the layout components that draw through them.

use aegis_ui::canvas::{Canvas, Image, Rect};
use aegis_ui::painter::{Fonts, Typeface, View as PView};
use aegis_ui::panes::{self, ColourRole, RowLook, ShellAct, ShellView, TypeStep};
use aegis_ui::screen_layout::{self, Act, Layout, View};
use aegis_ui::text_audit;
use aegis_ui::theme::{self, Metrics, Theme};
use aegis_ui::ui::{by_id, Gfx, Kit, Ui, Widget};
use serde_json::json;

fn font() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

fn gfx(scale: f32) -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(1280, 720), view: PView::fit(1280, 720), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.fonts.scale = scale;
    g.fonts.audit = Some(Vec::new());
    g.canvas.clear(Theme::default().bg);
    g
}

fn draw<A: Clone>(g: &mut Gfx, mouse: (i32, i32), f: impl FnOnce(&mut Ui<A>)) -> Vec<Widget<A>> {
    let mut ui: Ui<A> = Ui::new(g, mouse, true);
    f(&mut ui);
    ui.widgets
}

fn get<'a, A>(w: &'a [Widget<A>], id: &str) -> &'a Widget<A> {
    w.iter().find(|x| x.id == id).unwrap_or_else(|| panic!("no widget {id}: {:?}", w.iter().map(|x| x.id.clone()).collect::<Vec<_>>()))
}

const SCREEN: Rect = Rect { x: 0, y: 0, w: 640, h: 360 };

fn shell() -> ShellView<'static> {
    ShellView { title: "A panel", role: "ornate", close: true, back: Some("Back"), action: Some(("Confirm", true)), hints: "Q/E member, Esc close" }
}

#[test]
fn the_shell_has_its_title_close_back_action_and_hints_in_place() {
    for scale in [0.85, 1.0, 1.15, 1.3] {
        let mut g = gfx(scale);
        let mut body = Rect::new(0, 0, 0, 0);
        let w = draw(&mut g, (-1, -1), |ui: &mut Ui<ShellAct>| body = panes::panel_shell(ui, "p", SCREEN, &shell(), (240, 120), |a| a));
        let (panel, close, back, main, hints) = (get(&w, "p").rect, get(&w, "p.close").rect, get(&w, "p.back").rect, get(&w, "p.action").rect, get(&w, "p.hints").rect);
        assert_eq!(panes::shells_open(&w), 1);
        // sized to the body and centred (UX-02)
        assert!(body.w >= 240 && body.h >= 120, "{scale}: the body holds what it asked for: {body:?}");
        assert!(panel.w < SCREEN.w && panel.h < SCREEN.h, "{scale}: not a full-screen slab: {panel:?}");
        assert!(((panel.x + panel.w / 2) - SCREEN.w / 2).abs() <= 2 && ((panel.y + panel.h / 2) - SCREEN.h / 2).abs() <= 2, "{scale}: centred");
        // close at the top right, Back at the bottom left, the action at the bottom right, the hints between (UX-01, UX-07)
        assert!(close.y < body.y && close.x + close.w > body.x + body.w - close.w - 4, "{scale}: close at the top right: {close:?}");
        assert!(back.y >= body.y + body.h && back.x <= body.x + 1, "{scale}: Back at the footer's left: {back:?}");
        assert!(main.y == back.y && main.x + main.w >= body.x + body.w - 1, "{scale}: the action at the footer's right: {main:?}");
        assert!(hints.x >= back.x + back.w && hints.x + hints.w <= main.x, "{scale}: the hints between: {hints:?}");
        assert!(get(&w, "p.action").active, "the main action is the primary (amber)");
        assert_eq!(by_id(&w, "p.close"), Ok(ShellAct::Close));
        assert_eq!(by_id(&w, "p.back"), Ok(ShellAct::Back));
        assert_eq!(by_id(&w, "p.action"), Ok(ShellAct::Action));
        let audit = text_audit::issues(&g.fonts.audit.take().unwrap(), 640.0, 360.0, &[]);
        for k in ["cut", "edge", "spill"] {
            assert_eq!(audit["counts"][k], json!(0), "{scale}: {k}: {audit}");
        }
    }
}

#[test]
fn the_shell_dims_the_scene_about_sixty_percent_and_a_bare_shell_has_no_footer() {
    let mut g = gfx(1.0);
    g.canvas.clear([200, 200, 200, 255]);
    let w = draw(&mut g, (-1, -1), |ui: &mut Ui<ShellAct>| {
        panes::panel_shell(ui, "p", SCREEN, &ShellView { title: "Bare", role: "panel", ..ShellView::default() }, (100, 40), |a| a);
    });
    // a corner of the scene, past the panel: 200 under the scrim at SHELL_DIM
    let [r, ..] = g.canvas.px[..4] else { unreachable!() };
    let [sr, ..] = Theme::default().scrim;
    let want = (200.0 * (1.0 - panes::SHELL_DIM as f32 / 255.0) + sr as f32 * panes::SHELL_DIM as f32 / 255.0).round() as i32;
    assert!((r as i32 - want).abs() <= 1, "dimmed to {r}, want {want}");
    assert!((panes::SHELL_DIM as f32 / 255.0 - 0.6).abs() < 0.01);
    assert!(w.iter().all(|x| !x.id.starts_with("p.back") && !x.id.starts_with("p.action") && !x.id.starts_with("p.close")));
}

#[test]
fn a_list_row_has_its_icon_name_sub_line_and_numbers_right_aligned_in_mono() {
    let mut g = gfx(1.0);
    let m = Metrics::of(&theme::th(), g.view);
    let icon = Image { w: 32, h: 32, px: [120, 120, 120, 255].repeat(32 * 32) };
    let numbers = vec!["12".to_string(), "3 AP".to_string()];
    let mut r = Rect::new(10, 10, 300, 0);
    let w = draw(&mut g, (-1, -1), |ui: &mut Ui<()>| {
        r.h = panes::row_h_on(&ui.m(), &ui.p);
        panes::list_row(ui, "row", r, &RowLook { icon: Some(&icon), glyph: "", name: "Pommel Strike", sub: "Assault, Snap", numbers: &numbers, selected: false, enabled: true }, Some(()), Vec::new());
    });
    assert!(r.h >= m.u(panes::ROW_ICON), "the row holds its 32 px icon");
    assert_eq!(get(&w, "row").kind, "row");
    let recs = g.fonts.audit.take().unwrap();
    let at = |s: &str| recs.iter().find(|x| x.text == s).unwrap_or_else(|| panic!("no text {s}"));
    let (name, sub, n1, n2) = (at("Pommel Strike"), at("Assault, Snap"), at("12"), at("3 AP"));
    assert!(name.x >= (r.x + m.u(panes::ROW_ICON)) as f32, "the name right of the icon");
    assert!(sub.y > name.y && (sub.x - name.x).abs() < 0.5, "the sub-line under the name");
    assert!((n2.x + n2.w - (r.x + r.w) as f32).abs() <= m.uf(4.5), "the last number at the right end");
    assert!(n1.x + n1.w < n2.x, "the numbers in order");
    // the icon is drawn at its 32 px
    let tile_px = g.canvas.px[(((r.y + r.h / 2) * 2 * g.canvas.w + (r.x + m.u(1) + m.u(panes::ROW_ICON) / 2) * 2) * 4) as usize];
    assert_eq!(tile_px, 120, "the icon's pixels in the tile");
}

#[test]
fn the_type_scale_has_four_steps_and_the_colour_roles_four_meanings() {
    let t = Theme::default();
    let m = Metrics::of(&t, PView::fit(1280, 720));
    let sizes: Vec<f32> = TypeStep::ALL.iter().map(|s| s.size(&m)).collect();
    assert_eq!(TypeStep::ALL.len(), 4);
    assert!(sizes.windows(2).all(|w| w[0] >= w[1]), "title, heading, body, small from large to small: {sizes:?}");
    assert_eq!(TypeStep::Title.face(), Typeface::Display);
    assert!(TypeStep::ALL[1..].iter().all(|s| s.face() == Typeface::Body));
    assert_eq!((TypeStep::Title.colour(&t), TypeStep::Heading.colour(&t), TypeStep::Body.colour(&t), TypeStep::Small.colour(&t)), (t.gold, t.gold, t.text, t.dim));
    let roles: Vec<_> = ColourRole::ALL.iter().map(|r| (r.role(), r.colour(&t))).collect();
    assert_eq!(roles, vec![("gold", t.gold), ("focus", t.focus), ("foe", t.foe), ("text", t.text)]);
    for r in ColourRole::ALL {
        assert_eq!(t.role(r.role()), Some(r.colour(&t)), "{r:?} reads its theme role");
    }
}

#[test]
fn a_tooltip_goes_beside_the_pointer_never_over_what_it_explains() {
    let canvas = (640, 360);
    for over in [Rect::new(20, 40, 200, 14), Rect::new(500, 300, 130, 14), Rect::new(0, 0, 640, 14)] {
        let mouse = (over.x + over.w / 2, over.y + over.h / 2);
        let r = panes::tip_beside(over, mouse, (120, 40), canvas, 4);
        let overlaps = r.x < over.x + over.w && over.x < r.x + r.w && r.y < over.y + over.h && over.y < r.y + r.h;
        assert!(r.x >= 0 && r.y >= 0 && r.x + r.w <= canvas.0 && r.y + r.h <= canvas.1, "{over:?}: inside the canvas: {r:?}");
        if over.w + 120 + 8 <= canvas.0 {
            assert!(!overlaps, "{over:?}: the tip {r:?} covers it");
        }
    }
    let mut g = gfx(1.0);
    let body: Vec<String> = (1..=6).map(|i| format!("Line {i}")).collect();
    let mut p = aegis_ui::painter::Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
    let (w, h) = panes::tooltip_size(&mut p, "Title", &body, 160);
    panes::tooltip_box(&mut p, Rect::new(10, 10, w, h), "Title", &body);
    let recs = g.fonts.audit.take().unwrap();
    assert!(recs.iter().any(|r| r.text == "Title"));
    assert_eq!(recs.iter().filter(|r| r.text.starts_with("Line")).count(), panes::TIP_LINES, "at most three lines");
}

#[test]
fn a_layout_shell_takes_its_chrome_from_the_view_and_a_list_row_its_sub_line() {
    let view = View::from_json(&json!({
        "screen": "sample",
        "fields": {"title": {"text": "Stash"}, "hint": {"text": "Esc close"}, "items": {"rows": [{"id": "a", "label": "Rope", "sub": "A coil", "detail": "x2", "picture": "ui/none.png"}]}},
        "actions": [{"id": "close", "label": "Close"}, {"id": "back", "label": "Back"}, {"id": "take", "label": "Take"}]
    }))
    .unwrap();
    let layout = Layout::from_json(&json!({
        "screen": "sample",
        "regions": [
            {"id": "shell", "component": "panel_shell", "bind": {"title": "title", "close": "close", "back": "back", "action": "take", "hints": "hint"}, "anchor": "center", "w": 120, "h": 80},
            {"id": "list", "component": "list_row", "in": "shell", "bind": {"rows": "items", "select": "take"}, "anchor": "top", "w": "fill", "h": 30}
        ]
    }))
    .unwrap();
    assert!(layout.check(&view).is_empty(), "{:?}", layout.check(&view));
    let mut g = gfx(1.0);
    let layers = aegis_core::assets::Layers::of(vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")]);
    let mut art = aegis_ui::art::Art::layered(layers);
    let w = draw(&mut g, (-1, -1), |ui: &mut Ui<Act>| {
        screen_layout::draw(ui, &mut art, Some(&layout), &view, SCREEN);
    });
    assert_eq!(by_id(&w, "sample.shell.close"), Ok(Act { action: "close".into(), arg: None }));
    assert_eq!(by_id(&w, "sample.shell.back"), Ok(Act { action: "back".into(), arg: None }));
    assert_eq!(by_id(&w, "sample.shell.action"), Ok(Act { action: "take".into(), arg: None }));
    assert_eq!(by_id(&w, "sample.list.0"), Ok(Act { action: "take".into(), arg: Some("a".into()) }));
    let recs = g.fonts.audit.take().unwrap();
    for s in ["Stash", "Esc close", "Rope", "A coil", "x2"] {
        assert!(recs.iter().any(|r| r.text == s), "{s} is drawn");
    }
    // a bind the shell does not take is still a problem, and an action the view lacks too
    let bad = Layout::from_json(&json!({"screen": "sample", "regions": [{"id": "s", "component": "panel_shell", "bind": {"close": "nope"}, "anchor": "center", "w": 50, "h": 50}]})).unwrap();
    assert!(bad.check(&view).iter().any(|p| p.contains("no action 'nope'")), "{:?}", bad.check(&view));
}
