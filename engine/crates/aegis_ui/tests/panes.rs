//! LOOK1-ME: the engine panes (`aegis_ui::panes`): the backdrop's cover fit, vignette and veil; the menu column's ids,
//! rects, primary, hover, disabled items and hotkeys; the tab rail's ids, rows and content area; the slot card's
//! layout and its cut text on the hover card; the themed ask's layout on the grid, its keys (as the kit's question
//! answers them) and its cover over the screen under it; each drawn under the default theme and a fixture theme with
//! frames on and off, at the four text sizes and at the smallest window (960 x 540), and the Kit page's Panes page.

use aegis_ui::canvas::{Canvas, Image, Rect, Rgba};
use aegis_ui::focus;
use aegis_ui::painter::{Fonts, View};
use aegis_ui::panes::{self, AskView, Corner, MenuItem, MenuStyle, SlotView};
use aegis_ui::screen::Key;
use aegis_ui::specimens::{KitAct, Specimens, PAGES, PANES};
use aegis_ui::text_audit;
use aegis_ui::theme::{self, parse_colour, Frame, Nine, Theme};
use aegis_ui::ui::{by_id, hit, Gfx, Glyph, Kit, Ui, Widget};
use serde_json::{json, Value};
use std::sync::Arc;

const TEXT_SIZES: [f32; 4] = [0.85, 1.0, 1.15, 1.3];
/// The Kit page's rect as the gallery gives it (on the 640 x 360 logical canvas).
const KIT: Rect = Rect { x: 112, y: 22, w: 522, h: 332 };

fn font() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

fn gfx_at(w: u32, h: u32, scale: f32) -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(w as i32, h as i32), view: View::fit(w, h), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.fonts.scale = scale;
    g.fonts.audit = Some(Vec::new());
    g.canvas.clear(Theme::default().bg);
    g
}

fn gfx(scale: f32) -> Gfx {
    gfx_at(640, 360, scale)
}

fn px(c: &Canvas, x: i32, y: i32) -> Rgba {
    let i = ((y * c.w + x) * 4) as usize;
    [c.px[i], c.px[i + 1], c.px[i + 2], c.px[i + 3]]
}

/// Draws `f` on a fresh canvas with the mouse at `mouse`; the widgets and the text audit's counts and lists.
fn draw<A: Clone, R>(g: &mut Gfx, mouse: (i32, i32), f: impl FnOnce(&mut Ui<A>) -> R) -> (Vec<Widget<A>>, Value) {
    let w = {
        let mut ui: Ui<A> = Ui::new(g, mouse, true);
        let _ = f(&mut ui);
        ui.widgets
    };
    let boxes: Vec<_> = w.iter().map(|x| (x.id.clone(), x.rect.x as f32, x.rect.y as f32, x.rect.w as f32, x.rect.h as f32)).collect();
    let (lw, lh) = (g.view.to_logical(g.canvas.w as f32, g.canvas.h as f32).0 as f32, g.view.to_logical(g.canvas.w as f32, g.canvas.h as f32).1 as f32);
    let audit = text_audit::issues(&g.fonts.audit.take().unwrap_or_default(), lw, lh, &boxes);
    (w, audit)
}

fn ids<A>(w: &[Widget<A>]) -> Vec<String> {
    w.iter().map(|x| x.id.clone()).collect()
}

fn infos<A>(w: &[Widget<A>]) -> Vec<Value> {
    w.iter().map(|x| x.info()).collect()
}

fn centre(r: Rect) -> (i32, i32) {
    (r.x + r.w / 2, r.y + r.h / 2)
}

/// No text past the canvas, over another or out of its control; text that was cut has its full text on the hover
/// card of the widget it was drawn in.
fn clean<A>(audit: &Value, w: &[Widget<A>], what: &str) {
    for k in ["edge", "overlap", "spill"] {
        assert_eq!(audit["counts"][k], json!(0), "{what}: {k}: {audit}");
    }
    for c in audit["cut"].as_array().unwrap() {
        let full = c["text"].as_str().unwrap();
        assert!(w.iter().any(|x| x.tip.iter().any(|t| t.0 == full)), "{what}: '{full}' was cut and no hover card holds it");
    }
}

// ---- a fixture theme (Tallow and Iron's colours, square corners, a focus glow; frames on or off) -----------------------

fn hex(s: &str) -> Rgba {
    parse_colour(s).unwrap()
}

/// A 12 x 12 frame image: a 3 px border in `edge`, the centre in `fill`.
fn frame_img(edge: Rgba, fill: Rgba) -> Image {
    let n = 12;
    let mut px = Vec::new();
    for y in 0..n {
        for x in 0..n {
            px.extend_from_slice(&if x < 3 || y < 3 || x >= n - 3 || y >= n - 3 { edge } else { fill });
        }
    }
    Image { w: n, h: n, px }
}

fn tallow(frames: bool) -> Theme {
    let mut t = Theme {
        bg: hex("#0a0b10"),
        panel: hex("#0a0b10f0"),
        panel_edge: hex("#4b5563"),
        card: hex("#14171ffa"),
        card_hot: hex("#1e232dfa"),
        selected: hex("#2c221efa"),
        edge: hex("#4b5563"),
        edge_live: hex("#6b7280"),
        edge_hot: hex("#d97706"),
        focus: hex("#d97706"),
        text: hex("#d9cbb4"),
        dim: hex("#6b7280"),
        gold: hex("#fbbf24"),
        info: hex("#06b6d4"),
        ink: hex("#2c221e"),
        ..Theme::default()
    };
    t.layout.radius = 0.0;
    t.layout.frame_radius = 0.0;
    t.layout.focus_glow = 1;
    for (role, edge, fill) in [("panel", "#4b5563", "#0a0b10"), ("ornate", "#d97706", "#14171f"), ("inset", "#3a3f4a", "#101318"), ("button", "#6b7280", "#14171f"), ("card", "#4b5563", "#14171f")] {
        t.frames.insert(role.into(), Frame { img: Arc::new(frame_img(hex(edge), hex(fill))), nine: Nine::new([3, 3, 3, 3]), source: format!("ui/frames/{role}.png (fixture)") });
    }
    t.frames_on = frames;
    t
}

/// Runs `f` under each theme (the default, the fixture with frames on, with frames off), the default put back after.
fn each_theme(mut f: impl FnMut(&str)) {
    for (name, t) in [("default", Theme::default()), ("tallow", tallow(true)), ("tallow, frames off", tallow(false))] {
        theme::set_theme(t);
        f(name);
    }
    theme::set_theme(Theme::default());
}

// ---- backdrop ---------------------------------------------------------------------------------------------------------

/// The cover fit scales by max(r.w / w, r.h / h) and crops the overflow evenly: a 16:9 image in a 4:3 rect loses the
/// same width on both sides, a 4:3 image in a 16:9 rect the same height top and bottom.
#[test]
fn cover_fit_crops_a_16_9_image_into_a_4_3_rect_by_the_right_offset() {
    // 1600 x 900 into 400 x 300: scale 300 / 900, the rect covers 1200 x 900 of it, 200 px off each side.
    assert_eq!(panes::cover_crop(1600, 900, Rect::new(0, 0, 400, 300)), Rect::new(200, 0, 1200, 900));
    // 1200 x 900 into 640 x 360: scale 640 / 1200, 1200 x 675 shown, (900 - 675) / 2 off the top.
    let c = panes::cover_crop(1200, 900, Rect::new(0, 0, 640, 360));
    assert_eq!((c.x, c.w, c.h), (0, 1200, 675));
    assert_eq!(c.y, (900 - 675) / 2);
    // The same aspect: nothing cropped.
    assert_eq!(panes::cover_crop(320, 180, Rect::new(5, 5, 640, 360)), Rect::new(0, 0, 320, 180));
}

/// Drawn, the cropped side columns of a 16:9 picture are not on a 4:3 rect; the vignette leaves the centre and darkens
/// the corners toward `bg`; a full veil is `bg` everywhere.
#[test]
fn the_backdrop_draws_the_crop_the_vignette_and_the_veil() {
    let (red, blue) = ([220, 20, 20, 255], [20, 40, 220, 255]);
    // 16 x 9: the two columns at each side red (16 - 12 = 4 columns are cropped from a 4:3 rect), the rest blue.
    let img = Image { w: 16, h: 9, px: (0..16 * 9).flat_map(|i| if i % 16 < 2 || i % 16 >= 14 { red } else { blue }).collect() };
    let r = Rect::new(40, 40, 120, 90);
    let bg = Theme::default().bg;
    let shot = |veil: u8, vignette: f32| {
        let mut g = gfx(1.0);
        let _ = draw::<(), _>(&mut g, (-1, -1), |ui| panes::backdrop(&mut ui.p, Some(&img), r, veil, vignette));
        g.canvas
    };
    let c = shot(0, 0.0);
    for y in [r.y, r.y + r.h / 2, r.y + r.h - 1] {
        assert_eq!((px(&c, r.x, y), px(&c, r.x + r.w - 1, y)), (blue, blue), "the red sides are cropped (row {y})");
    }
    let c = shot(0, 1.0);
    assert_eq!(px(&c, r.x + r.w / 2, r.y + r.h / 2), blue, "the vignette leaves the centre");
    let corner = px(&c, r.x, r.y);
    assert!((0..3).all(|k| (corner[k] as i32 - bg[k] as i32).abs() <= 4), "a corner is all but bg: {corner:?}");
    let c = shot(255, 0.0);
    assert_eq!(px(&c, r.x + r.w / 2, r.y + r.h / 2), [bg[0], bg[1], bg[2], 255], "a full veil is bg");
    // No picture: the bg colour.
    let mut g = gfx(1.0);
    let _ = draw::<(), _>(&mut g, (-1, -1), |ui| panes::backdrop_pane(ui, "bd", None, "none", r, 0, 0.0));
    assert_eq!(px(&g.canvas, r.x + 3, r.y + 3), [bg[0], bg[1], bg[2], 255]);
}

// ---- menu column ------------------------------------------------------------------------------------------------------

fn items() -> Vec<MenuItem<&'static str>> {
    vec![
        MenuItem::new("go", "Continue", Some('c'), "go"),
        MenuItem::new("new", "New game", Some('n'), "new"),
        MenuItem::new("off", "Credits", Some('x'), "off").off("Nothing to show yet."),
        MenuItem::new("quit", "Quit", Some('q'), "quit"),
    ]
}

const MX: i32 = 420;
const MY: i32 = 152;

/// Ids `<id>.<key>` top-down, item i at (x, y + i x step), w x h, at Normal and Extra large; the item under the mouse
/// is returned; the text fits.
#[test]
fn the_menu_column_lays_out_its_items_and_reports_the_hovered_one() {
    for scale in [1.0, 1.3] {
        let mut out = None;
        let mut g = gfx(scale);
        let mouse = centre(panes::menu_rect(MX, MY, 160, 18, 22, 2));
        let (w, audit) = draw(&mut g, mouse, |ui| out = panes::menu_column(ui, "menu", MX, MY, 160, 18, 22, &items()));
        assert_eq!(ids(&w), ["menu.go", "menu.new", "menu.off", "menu.quit"]);
        for (i, wd) in w.iter().enumerate() {
            assert_eq!(wd.rect, Rect::new(MX, MY + 22 * i as i32, 160, 18), "item {i} at {scale}");
            assert_eq!(wd.kind, "button");
        }
        assert_eq!(out, Some(2), "the hovered item, even a disabled one");
        clean(&audit, &w, &format!("menu at {scale}"));
        let mut g = gfx(scale);
        let mut none = Some(9);
        let _ = draw(&mut g, (0, 0), |ui| none = panes::menu_column(ui, "menu", MX, MY, 160, 18, 22, &items()));
        assert_eq!(none, None);
    }
}

/// A disabled item takes no click (by mouse or by id) and its hover card gives the reason; an enabled one sends its
/// action; each enabled item's hotkey (either case) sends it, a disabled one's does nothing.
#[test]
fn menu_items_click_and_answer_their_hotkeys() {
    let mut g = gfx(1.0);
    let (w, _) = draw(&mut g, (-1, -1), |ui| panes::menu_column(ui, "menu", MX, MY, 160, 18, 22, &items()));
    let off = w.iter().find(|x| x.id == "menu.off").unwrap();
    assert!(!off.enabled);
    assert_eq!(hit(&w, centre(off.rect)), None);
    assert!(by_id(&w, "menu.off").is_err());
    assert_eq!(off.tip.iter().map(|t| t.0.as_str()).collect::<Vec<_>>(), ["Credits", "Nothing to show yet."]);
    assert_eq!(hit(&w, centre(w[1].rect)), Some("new"));
    assert_eq!(by_id(&w, "menu.quit"), Ok("quit"));
    let it = items();
    assert_eq!(panes::menu_key(&it, Key::Char('n')), Some("new"));
    assert_eq!(panes::menu_key(&it, Key::Char('Q')), Some("quit"));
    assert_eq!(panes::menu_key(&it, Key::Char('x')), None, "a disabled item's hotkey");
    assert_eq!(panes::menu_key(&it, Key::Char('z')), None);
    assert_eq!(panes::menu_key(&it, Key::Enter), None);
}

/// The keyboard focus walks the enabled items top-down (Tab and Down), skipping a disabled one.
#[test]
fn the_menu_column_focus_order_is_top_down() {
    let mut g = gfx(1.0);
    let (w, _) = draw(&mut g, (-1, -1), |ui| panes::menu_column(ui, "menu", MX, MY, 160, 18, 22, &items()));
    let inf = infos(&w);
    let ring: Vec<&str> = focus::ring(&inf, false).iter().map(|x| x["id"].as_str().unwrap()).collect();
    assert_eq!(ring, ["menu.go", "menu.new", "menu.quit"]);
    assert_eq!(focus::step(&inf, Some("menu.new"), focus::Move::Next, false).as_deref(), Some("menu.quit"));
    assert_eq!(focus::step(&inf, Some("menu.new"), focus::Move::Dir(0, 1), false).as_deref(), Some("menu.quit"));
    assert_eq!(focus::step(&inf, Some("menu.quit"), focus::Move::Dir(0, -1), false).as_deref(), Some("menu.new"));
}

/// The primary (the first enabled item) has the `focus` border while idle, the others `edge_live`; with the first
/// item disabled the second is the primary.
#[test]
fn the_first_enabled_item_is_the_primary() {
    let t = Theme::default();
    let border = |it: &[MenuItem<&'static str>], i: usize| {
        let mut g = gfx(1.0);
        let _ = draw(&mut g, (-1, -1), |ui| panes::menu_column(ui, "menu", MX, MY, 160, 18, 22, it));
        let r = panes::menu_rect(MX, MY, 160, 18, 22, i);
        px(&g.canvas, r.x, r.y + r.h / 2)
    };
    let it = items();
    assert_eq!(border(&it, 0), t.focus);
    assert_eq!(border(&it, 1), t.edge_live);
    assert_eq!(border(&it, 2), t.edge);
    let mut first_off = items();
    first_off[0] = first_off[0].clone().off("Not yet.");
    assert_eq!(border(&first_off, 1), t.focus);
    assert_eq!(border(&first_off, 0), t.edge);
}

/// The title block: in the display face, the column starts on the grid below it.
#[test]
fn the_menu_title_returns_a_grid_y_below_it() {
    for scale in TEXT_SIZES {
        let mut g = gfx(scale);
        let mut y = 0;
        let (w, audit) = draw::<(), _>(&mut g, (-1, -1), |ui| y = panes::menu_title(ui, MX, 100, 200, "Title", "A subtitle"));
        let need = 100.0 + (12.0 * 1.28 + 7.0 * 1.28) * scale;
        assert!(y as f32 >= need && y % 4 == 0, "{y} at {scale}");
        clean(&audit, &w, "title");
    }
}

// ---- tab rail -----------------------------------------------------------------------------------------------------

/// Ids `<id>.<i>`, rows 18 high one every 20 on a 120 wide rail, the open tab active, clicks send their index, the
/// content area right of the rail past the gutter; only rows that fit are drawn. PageUp / PageDown wrap.
#[test]
fn the_tab_rail_lays_out_its_tabs_and_content() {
    let tabs = ["Video", "Audio", "Controls", "Text"];
    let r = Rect::new(40, 40, 400, 200);
    for scale in TEXT_SIZES {
        let mut g = gfx(scale);
        let mut content = Rect::default();
        let (w, audit) = draw(&mut g, (-1, -1), |ui| content = panes::tab_rail(ui, "rail", r, &tabs, 2, |i| i));
        assert_eq!(ids(&w), ["rail.0", "rail.1", "rail.2", "rail.3"]);
        for (i, wd) in w.iter().enumerate() {
            assert_eq!(wd.rect, Rect::new(r.x, r.y + 20 * i as i32, 120, 18));
            assert_eq!(wd.active, i == 2);
            assert_eq!(hit(&w, centre(wd.rect)), Some(i));
        }
        let gutter = Theme::default().layout.gutter;
        let pad = Theme::default().layout.padding;
        assert_eq!(content, Rect::new(r.x + 120 + gutter, r.y, r.w - 120 - gutter, r.h).inset(pad));
        clean(&audit, &w, &format!("rail at {scale}"));
    }
    let mut g = gfx(1.0);
    let (w, _) = draw(&mut g, (-1, -1), |ui| panes::tab_rail(ui, "rail", Rect::new(0, 0, 300, 50), &tabs, 0, |i| i));
    assert_eq!(ids(&w), ["rail.0", "rail.1"], "50 high holds rows at 0 and 20 (the third would end at 58)");
    assert_eq!(panes::tab_key(4, 3, Key::PageDown), Some(0));
    assert_eq!(panes::tab_key(4, 0, Key::PageUp), Some(3));
    assert_eq!(panes::tab_key(4, 1, Key::Enter), None);
    let inf = infos(&draw(&mut gfx(1.0), (-1, -1), |ui| panes::tab_rail(ui, "rail", r, &tabs, 0, |i| i)).0);
    assert_eq!(focus::step(&inf, Some("rail.0"), focus::Move::Dir(0, 1), false).as_deref(), Some("rail.1"));
}

// ---- slot card ----------------------------------------------------------------------------------------------------

fn solid(w: i32, h: i32, c: Rgba) -> Image {
    Image { w, h, px: c.repeat((w * h) as usize) }
}

fn corners() -> Vec<Corner<&'static str>> {
    vec![
        Corner { key: "lock".into(), label: "Lock".into(), glyph: Glyph::Lock, action: "lock", enabled: true, toggled: true, tip: String::new() },
        Corner { key: "delete".into(), label: "Delete".into(), glyph: Glyph::Delete, action: "delete", enabled: false, toggled: false, tip: "Unlock it to delete it.".into() },
    ]
}

/// The thumbnail 112 x 63 at (x + 6, centred down), the text from x + 124, the portrait row 20 px above the bottom
/// padding, the corner icons 16 px boxes (12 px icons) at the top right; the card and its icons answer their own
/// clicks (the disabled delete takes the click and does nothing).
#[test]
fn the_slot_card_places_its_parts() {
    let r = Rect::new(20, 20, 290, 84);
    let (thumb, tx, py, icons) = panes::slot_rects(r, 2);
    assert_eq!(thumb, Rect::new(r.x + 6, r.y + (84 - 63) / 2, 112, 63));
    assert_eq!((tx, py), (r.x + 124, r.y + 84 - 6 - 20));
    assert_eq!(icons, [Rect::new(r.x + r.w - 2 - 32, r.y + 2, 16, 16), Rect::new(r.x + r.w - 2 - 16, r.y + 2, 16, 16)]);
    assert_eq!(aegis_ui::ui::Ui::<()>::icon_rect(icons[0]).w, 12);
    let red = [200, 40, 40, 255];
    let thumb_img = solid(32, 18, red);
    let face = solid(20, 20, [40, 200, 40, 255]);
    let view = SlotView { thumb: Some(&thumb_img), title: "Save".into(), lines: vec!["A place, day 2".into()], portraits: vec![&face, &face], stamp: "2026-11-02 10:00".into() };
    let mut g = gfx(1.0);
    let (w, _) = draw(&mut g, (-1, -1), |ui| panes::slot_card(ui, "slot", r, &view, Some("open"), false, &corners()));
    assert_eq!(ids(&w), ["slot", "slot.lock", "slot.delete"]);
    assert_eq!(px(&g.canvas, thumb.x + 50, thumb.y + 30), red, "the thumbnail");
    assert_eq!(px(&g.canvas, tx + 10, py + 10), [40, 200, 40, 255], "the first portrait");
    assert_eq!(px(&g.canvas, tx + 22 + 10, py + 10), [40, 200, 40, 255], "the second, 22 along");
    assert_eq!(hit(&w, (r.x + 150, r.y + 40)), Some("open"));
    assert_eq!(hit(&w, centre(icons[0])), Some("lock"));
    assert_eq!(hit(&w, centre(icons[1])), None, "a disabled icon over the card swallows its click");
    assert!(w[2].tip.iter().any(|t| t.0 == "Unlock it to delete it."));
    assert!(w[0].tip.is_empty(), "nothing cut: no hover card");
    // No thumbnail: an inset with the initial (the text audit records it).
    let mut g = gfx(1.0);
    let empty = SlotView { title: "empty".into(), ..SlotView::default() };
    let _ = draw(&mut g, (-1, -1), |ui| panes::slot_card(ui, "slot", r, &empty, None::<&str>, false, &[]));
}

/// Long names and lines are cut at all four text sizes with nothing past the card, over other text or out of it, and
/// the full texts are on the card's hover card.
#[test]
fn a_slot_card_with_long_names_cuts_them_with_the_full_text_on_hover() {
    let face = solid(20, 20, [90, 90, 90, 255]);
    let view = SlotView {
        thumb: None,
        title: "A save with a very long name that cannot possibly fit on the card".into(),
        lines: vec!["A place whose name runs on and on and on, day 12".into(), "Late in the evening of a long day".into(), "A third line that may not fit".into()],
        portraits: vec![&face; 6],
        stamp: "2026-10-30 09:05".into(),
    };
    for scale in TEXT_SIZES {
        let mut g = gfx(scale);
        let r = Rect::new(20, 20, 290, 84);
        let (w, audit) = draw(&mut g, (-1, -1), |ui| panes::slot_card(ui, "slot", r, &view, Some("open"), true, &corners()));
        clean(&audit, &w, &format!("slot at {scale}"));
        assert!(audit["counts"]["cut"].as_u64().unwrap() >= 1, "the long title is cut at {scale}");
        let tips: Vec<&str> = w[0].tip.iter().map(|t| t.0.as_str()).collect();
        assert_eq!(tips[0], view.title);
        assert!(view.lines.iter().all(|l| tips.contains(&l.as_str())), "every line on the hover card at {scale}");
        assert!(w[0].active, "selected");
    }
}

// ---- the themed ask -----------------------------------------------------------------------------------------------

/// The keys answer as the kit's question does (tests/ask.rs): Enter the default, y Yes, n and Esc No.
#[test]
fn the_ask_keys_are_the_kit_questions() {
    assert_eq!(panes::ask_key(false, Key::Enter), Some(true));
    assert_eq!(panes::ask_key(true, Key::Enter), Some(false));
    for nd in [false, true] {
        assert_eq!(panes::ask_key(nd, Key::Char('y')), Some(true));
        assert_eq!(panes::ask_key(nd, Key::Char('n')), Some(false));
        assert_eq!(panes::ask_key(nd, Key::Esc), Some(false));
        assert_eq!(panes::ask_key(nd, Key::Char('x')), None);
    }
}

/// On the grid, centred, at least 300 x 150 times the text size (inside the margins), Yes left of No on one row at
/// the bottom right; the default answer has the `focus` border; the screen under it takes no focus and no click.
#[test]
fn the_ask_dialog_lays_out_on_the_grid_and_covers_the_screen() {
    let area = Rect::new(0, 0, 640, 360);
    for ornate in [false, true] {
        for scale in TEXT_SIZES {
            for no_default in [false, true] {
                let mut g = gfx(scale);
                let mut d = Rect::default();
                let v = AskView { ornate, no_default, ..AskView::new("Leave?", "Anything not saved since the last save is lost. Do you want to leave this place now?") };
                let (w, audit) = draw(&mut g, (-1, -1), |ui| {
                    ui.button("under", Rect::new(300, 170, 40, 14), "Under", "under", true);
                    d = panes::ask_dialog(ui, "ask", area, &v, "yes", "no");
                });
                let what = format!("ornate {ornate} at {scale} no_default {no_default}");
                assert!([d.x, d.y, d.w, d.h].iter().all(|v| v % 4 == 0), "{what}: {d:?} on the grid");
                assert!(d.w as f32 >= (300.0 * scale.max(1.0)).round() - 4.0 && d.h >= 148, "{what}: {d:?}");
                let m = Theme::default().layout.margin;
                assert!(d.x >= m && d.y >= m && d.x + d.w <= 640 - m && d.y + d.h <= 360 - m, "{what}: inside the margins");
                assert!(((d.x + d.w / 2) - 320).abs() <= 2 && ((d.y + d.h / 2) - 180).abs() <= 2, "{what}: centred");
                assert_eq!(ids(&w), ["under", "ask", "ask.yes", "ask.no"]);
                let (yes, no) = (w[2].rect, w[3].rect);
                assert!(yes.x + yes.w <= no.x && yes.y == no.y && no.x + no.w <= d.x + d.w && no.y + no.h <= d.y + d.h, "{what}");
                let ring: Vec<String> = focus::ring(&infos(&w), false).iter().map(|x| x["id"].as_str().unwrap().to_string()).collect();
                assert_eq!(ring, ["ask.yes", "ask.no"], "{what}: the screen under it is out of reach");
                assert_eq!((hit(&w, centre(yes)), hit(&w, centre(no))), (Some("yes"), Some("no")));
                let primary = if no_default { no } else { yes };
                let other = if no_default { yes } else { no };
                let t = Theme::default();
                assert_eq!(px(&g.canvas, primary.x, primary.y + primary.h / 2), t.focus, "{what}: the default is the primary");
                assert_eq!(px(&g.canvas, other.x, other.y + other.h / 2), t.edge_live, "{what}");
                clean(&audit, &w, &what);
            }
        }
    }
}

// ---- the composed backdrop ------------------------------------------------------------------------------------------

/// A 16:9 picture with a different colour at every pixel.
fn gradient(w: i32, h: i32) -> Image {
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            px.extend_from_slice(&[(x % 251) as u8, (y % 241) as u8, ((x + y) % 239) as u8, 255]);
        }
    }
    Image { w, h, px }
}

/// `f` on a canvas of a known pattern (what must stay outside the clip and the bounds).
fn painted(ww: u32, wh: u32, limits: (Option<Rect>, Option<Rect>), f: impl FnOnce(&mut Ui<()>)) -> Vec<u8> {
    let mut g = gfx_at(ww, wh, 1.0);
    for (i, b) in g.canvas.px.iter_mut().enumerate() {
        *b = if i % 4 == 3 { 255 } else { (i % 97) as u8 };
    }
    g.canvas.bounds = limits.1;
    let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
    ui.p.c.clip = limits.0;
    f(&mut ui);
    g.canvas.px.clone()
}

/// The blit is pixel for pixel what the live drawing gives, on the frame that composes and the frames after, whatever the
/// clip and bounds, at the rect's place; those frames compose nothing more.
#[test]
fn the_composed_backdrop_is_pixel_identical_to_the_live_drawing() {
    let img = gradient(320, 180);
    let inner = Rect::new(100, 60, 300, 200);
    let limits = [(None, None), (Some(inner), None), (None, Some(inner)), (Some(Rect::new(0, 0, 200, 100)), Some(inner)), (Some(Rect::new(1200, 0, 40, 40)), Some(inner))];
    for (r, veil, vignette) in [(Rect::new(0, 0, 640, 360), 0, 0.4), (Rect::new(0, 0, 640, 360), 128, 0.7), (Rect::new(40, 20, 300, 300), 200, 0.0)] {
        for l in limits {
            let live = painted(1280, 720, l, |ui| panes::backdrop_live(&mut ui.p, Some(&img), r, veil, vignette));
            let first = painted(1280, 720, l, |ui| panes::backdrop(&mut ui.p, Some(&img), r, veil, vignette));
            let at = panes::backdrop_composes();
            let again = painted(1280, 720, l, |ui| panes::backdrop(&mut ui.p, Some(&img), r, veil, vignette));
            assert!(live == first && live == again, "{r:?} veil {veil} limits {l:?}");
            assert_eq!(panes::backdrop_composes(), at, "the later frame only blits");
        }
    }
    let live = painted(1280, 720, (None, None), |ui| panes::backdrop_live(&mut ui.p, None, Rect::new(0, 0, 640, 360), 40, 0.5));
    assert!(live == painted(1280, 720, (None, None), |ui| panes::backdrop(&mut ui.p, None, Rect::new(0, 0, 640, 360), 40, 0.5)), "no picture: the bg colour");
}

/// A resize, another veil, vignette, picture or `bg` composes again; the same backdrop elsewhere does not.
#[test]
fn the_composed_backdrop_recomposes_when_what_it_shows_changes() {
    let mut a = gradient(64, 36);
    let mut other = gradient(64, 36);
    for p in other.px.as_chunks_mut::<4>().0 {
        p[0] = 255 - p[0];
    }
    let r = Rect::new(0, 0, 640, 360);
    let draw_at = |ww: u32, wh: u32, img: &Image, r: Rect, veil: u8, vig: f32| {
        let before = panes::backdrop_composes();
        let px = painted(ww, wh, (None, None), |ui| panes::backdrop(&mut ui.p, Some(img), r, veil, vig));
        let live = painted(ww, wh, (None, None), |ui| panes::backdrop_live(&mut ui.p, Some(img), r, veil, vig));
        assert!(px == live, "{ww} x {wh}: drawn as it should be");
        panes::backdrop_composes() - before
    };
    assert_eq!(draw_at(640, 360, &a, r, 0, 0.4), 1, "first");
    assert_eq!(draw_at(640, 360, &a, r, 0, 0.4), 0, "same");
    assert_eq!(draw_at(1280, 720, &a, r, 0, 0.4), 1, "a resize");
    assert_eq!(draw_at(640, 360, &a, r, 10, 0.4), 1, "the veil");
    assert_eq!(draw_at(640, 360, &a, r, 0, 0.5), 1, "the vignette");
    assert_eq!(draw_at(640, 360, &other, r, 0, 0.4), 1, "another picture");
    let small = Rect::new(0, 0, 100, 100);
    assert_eq!(draw_at(640, 360, &a, small, 0, 0.4), 1, "another size");
    assert_eq!(draw_at(640, 360, &a, Rect::new(20, 20, 100, 100), 0, 0.4), 0, "only moved");
    theme::set_theme(Theme { bg: [1, 2, 3, 255], ..Theme::default() });
    assert_eq!(draw_at(640, 360, &a, small, 0, 0.4), 1, "a theme change");
    theme::set_theme(Theme::default());
    // Repainted where it lies: same address, same size, other pixels.
    let at = a.px.as_ptr();
    for p in a.px.as_chunks_mut::<4>().0 {
        p[1] = 255 - p[1];
    }
    assert_eq!(a.px.as_ptr(), at);
    assert_eq!(draw_at(640, 360, &a, small, 0, 0.4), 1, "the same address repainted");
}

/// A see-through picture shows what lies under it, whatever that is: drawn live, every frame.
#[test]
fn a_see_through_backdrop_is_drawn_live() {
    let mut img = gradient(64, 36);
    img.px[3 + 4 * (64 * 18 + 32)] = 100;
    let r = Rect::new(0, 0, 640, 360);
    let live = painted(640, 360, (None, None), |ui| panes::backdrop_live(&mut ui.p, Some(&img), r, 20, 0.4));
    for _ in 0..2 {
        assert!(live == painted(640, 360, (None, None), |ui| panes::backdrop(&mut ui.p, Some(&img), r, 20, 0.4)));
    }
}

/// Frame times of the backdrop at window sizes, live against the blit: `cargo test -p aegis_ui --release --test panes
/// backdrop_frame_times -- --ignored --nocapture`.
#[test]
#[ignore]
fn backdrop_frame_times() {
    let img = gradient(1920, 1080);
    let r = Rect::new(0, 0, 640, 360);
    for (ww, wh) in [(1280, 720), (1920, 1080), (3840, 2160)] {
        let mut g = gfx_at(ww, wh, 1.0);
        let mut ui: Ui<()> = Ui::new(&mut g, (-1, -1), true);
        let time = |ui: &mut Ui<()>, f: &dyn Fn(&mut Ui<()>)| {
            let t = std::time::Instant::now();
            for _ in 0..20 {
                f(ui);
            }
            t.elapsed().as_secs_f64() * 1000.0 / 20.0
        };
        let live = time(&mut ui, &|ui| panes::backdrop_live(&mut ui.p, Some(&img), r, 0, 0.4));
        let t = std::time::Instant::now();
        panes::backdrop(&mut ui.p, Some(&img), r, 0, 0.4);
        let compose = t.elapsed().as_secs_f64() * 1000.0;
        let blit = time(&mut ui, &|ui| panes::backdrop(&mut ui.p, Some(&img), r, 0, 0.4));
        println!("{ww} x {wh}: live {live:.2} ms, first frame {compose:.2} ms, blit {blit:.3} ms");
    }
}

// ---- the text-only menu ---------------------------------------------------------------------------------------------

/// How many gold pixels lie in row `i` of the column at (40, 40), step 22.
fn lit_in_row(g: &Gfx, i: i32) -> usize {
    let r = panes::menu_rect(40, 40, 160, 18, 22, i as usize);
    (r.y..r.y + r.h).flat_map(|y| (r.x..r.x + r.w).map(move |x| (x, y))).filter(|(x, y)| {
        let p = px(&g.canvas, *x, *y);
        p[0] > 100 && (p[2] as u32) * 2 < p[0] as u32
    }).count()
}

/// `MenuStyle::Text`: same ids, rows and actions as the framed column; no panel, no marker (only the text changes the
/// pixels), no primary; a click takes the label's width plus the margin, and a point just past it neither lights nor
/// clicks; a disabled row takes no click and stays dim; a cut label reads in full on hover.
#[test]
fn the_text_menu_style_has_no_panel_and_takes_the_mouse_only_on_its_label() {
    let it = items();
    let row = panes::menu_rect(40, 40, 160, 18, 22, 0);
    let (mut g, mut framed) = (gfx(1.0), gfx(1.0));
    let (fw, _) = draw(&mut framed, (-1, -1), |ui| panes::menu_column(ui, "m", 40, 40, 160, 18, 22, &it));
    let (w, audit) = draw(&mut g, (-1, -1), |ui| panes::menu_column_styled(ui, "m", 40, 40, 160, 18, 22, &it, MenuStyle::Text));
    assert_eq!(ids(&w), ids(&fw));
    assert!(w.iter().zip(&fw).all(|(a, b)| a.rect.y == b.rect.y && a.rect.h == b.rect.h && a.enabled == b.enabled && a.action == b.action && a.kind == "text_button"));
    clean(&audit, &w, "text menu");
    let under = px(&g.canvas, row.x - 8, row.y - 8);
    assert!(px(&g.canvas, row.x, row.y) == under && px(&g.canvas, row.x + row.w - 1, row.y + row.h / 2) == under, "no panel, no edge");
    assert!((0..it.len() as i32).all(|i| lit_in_row(&g, i) == 0), "nothing lit while idle: no primary");

    let (pad, margin) = (Theme::default().layout.padding, panes::MENU_TEXT_MARGIN);
    let label_w = {
        let mut m = gfx(1.0);
        let mut got = 0.0;
        let _ = draw::<(), _>(&mut m, (-1, -1), |ui| got = ui.p.text_w(&it[0].label, aegis_ui::ui::TITLE));
        got.ceil() as i32
    };
    assert_eq!(w[0].rect, Rect::new(row.x + pad - margin, row.y, label_w + 2 * margin, row.h), "the label and its margin");
    let (inside, past) = ((row.x + pad + label_w / 2, row.y + 9), (w[0].rect.x + w[0].rect.w + 1, row.y + 9));
    assert_eq!(hit(&w, inside), fw[0].action.clone(), "inside the label clicks");
    assert_eq!(hit(&w, past), None, "just past the label: no click");
    assert!(past.0 < row.x + row.w, "the point is still inside the old row");
    for (mouse, lit, hovered) in [(inside, true, Some(0)), (past, false, None)] {
        let mut m = gfx(1.0);
        let mut got = None;
        let _ = draw(&mut m, mouse, |ui| got = panes::menu_column_styled(ui, "m", 40, 40, 160, 18, 22, &it, MenuStyle::Text));
        assert_eq!((lit_in_row(&m, 0) > 0, got), (lit, hovered), "{mouse:?}");
    }
    let disabled = it.iter().position(|i| !i.enabled).expect("a disabled sample item");
    assert_eq!(hit(&w, centre(w[disabled].rect)), None);

    for scale in TEXT_SIZES {
        let mut c = gfx(scale);
        let long = vec![MenuItem::new("a", LONG, None, 1)];
        let (w, audit) = draw(&mut c, (-1, -1), |ui| panes::menu_column_styled(ui, "m", 20, 20, 80, 18, 22, &long, MenuStyle::Text));
        assert_eq!(w[0].tip[0].0, LONG);
        assert!(w[0].rect.x + w[0].rect.w <= 20 + 80, "a cut label keeps to its row");
        assert_eq!(audit["counts"]["cut"], json!(1), "at {scale}");
        clean(&audit, &w, &format!("cut text row at {scale}"));
    }
}

/// One active item, shown by brighter text alone: the keyboard on A lights A; the mouse moving to B leaves only B lit
/// (through the same `Kit::mouse_moved` the shell calls); leaving the labels keeps B lit; a keyboard move while the mouse
/// rests elsewhere lights only the keyboard's row; the shell draws no ring around it and the focus walks the rows.
#[test]
fn a_text_menu_lights_one_active_item() {
    let it = items();
    let frame = |g: &mut Gfx, mouse: (i32, i32)| {
        g.canvas.clear([0, 0, 0, 255]);
        draw(g, mouse, |ui| panes::menu_column_styled(ui, "m", 40, 40, 160, 18, 22, &it, MenuStyle::Text)).0
    };
    let lit = |g: &Gfx| (0..it.len() as i32).filter(|i| lit_in_row(g, *i) > 0).collect::<Vec<_>>();
    let mut g = gfx(1.0);
    let w = frame(&mut g, (-1, -1));
    let (a, b) = (w[0].id.clone(), w[1].id.clone());
    let centre_b = centre(w[1].rect);
    g.kit.focus = Some(a.clone());
    g.kit.focus_visible = true;
    frame(&mut g, (-1, -1));
    assert_eq!(lit(&g), [0], "the keyboard on A");
    let under = infos(&frame(&mut g, centre_b));
    assert_eq!(lit(&g), [0], "a mouse resting on B does not light a second row");
    g.kit.mouse_moved(&under, centre_b);
    assert_eq!(g.kit.focus.as_deref(), Some(b.as_str()));
    frame(&mut g, centre_b);
    assert_eq!(lit(&g), [1], "the mouse moved to B: only B");
    g.kit.mouse_moved(&under, (620, 340));
    frame(&mut g, (620, 340));
    assert_eq!(lit(&g), [1], "off the labels the last active item stays lit");
    assert_eq!(g.kit.focus.as_deref(), Some(b.as_str()));
    g.kit.focus = Some(a.clone());
    frame(&mut g, (620, 340));
    assert_eq!(lit(&g), [0], "the keyboard moved it back");
    assert!(!focus::shows_ring(&under[0]), "no ring around a row without a panel");
    let framed = infos(&draw(&mut gfx(1.0), (-1, -1), |ui| panes::menu_column(ui, "m", 40, 40, 160, 18, 22, &it)).0);
    assert!(focus::shows_ring(&framed[0]), "a framed column keeps it");
    assert_eq!(focus::hover_target(&framed, centre_b), None, "a framed column does not move the focus with the mouse");
    assert_eq!(focus::step(&under, Some(&a), focus::Move::Next, false).as_deref(), Some(b.as_str()));
    assert_eq!(focus::step(&under, Some(&a), focus::Move::Dir(0, 1), false).as_deref(), Some(b.as_str()), "the arrow walks them");
    assert_eq!(focus::step(&under, Some(&b), focus::Move::Dir(0, -1), false).as_deref(), Some(a.as_str()));
}

/// Whole steps keep the aspect and fit the room: up by the largest whole multiple, down by the smallest whole division.
#[test]
fn a_picture_scales_by_whole_steps() {
    assert_eq!(panes::whole_steps(100, 50, 368, 232), (3, 1));
    assert_eq!(panes::whole_steps(16, 16, 368, 232), (14, 1));
    assert_eq!(panes::whole_steps(368, 232, 368, 232), (1, 1));
    assert_eq!(panes::whole_steps(800, 200, 368, 232), (1, 3));
    assert_eq!(panes::whole_steps(300, 700, 368, 232), (1, 4));
    assert_eq!(panes::whole_steps(0, 10, 10, 10), (1, 1));
    let red = solid(10, 20, [200, 0, 0, 255]);
    for (ww, wh, want) in [(640, 360, 4), (1280, 720, 9)] {
        let mut g = gfx_at(ww, wh, 1.0);
        let mut got = Rect::default();
        let _ = draw::<(), _>(&mut g, (-1, -1), |ui| got = panes::image_steps(&mut ui.p, &red, 100.0, 100.0, 50, 90));
        let s = ww as i32 / 640;
        assert_eq!((got.w * s, got.h * s), (10 * want, 20 * want), "{ww}");
        assert_eq!((got.x + got.w / 2, got.y + got.h / 2), (100, 100), "centred at {ww}");
        let solid_px = g.canvas.px.as_chunks::<4>().0.iter().filter(|p| p[0] == 200).count() as i32;
        assert_eq!(solid_px, 10 * want * 20 * want, "whole multiple at {ww}");
    }
}

// ---- cut text reads in full; tabs past the rail stay in reach -------------------------------------------------------

const LONG: &str = "A label far too long for the narrow column it was given to sit in";

/// A menu label that is cut has its full text on hover, whether or not the item has a hover card of its own; one that
/// fits has no card added.
#[test]
fn a_cut_menu_label_reads_in_full_on_hover() {
    for scale in TEXT_SIZES {
        let mut g = gfx(scale);
        let it = vec![MenuItem::new("plain", LONG, Some('p'), 1), MenuItem::new("told", LONG, None, 2).with_tip("What it does."), MenuItem::new("short", "Ok", None, 3)];
        let (w, audit) = draw(&mut g, (-1, -1), |ui| panes::menu_column(ui, "m", 20, 20, 80, 18, 22, &it));
        assert_eq!(w[0].tip[0].0, LONG, "at {scale}");
        assert_eq!(w[1].tip[0].0, LONG, "at {scale}: the card it had leads with the label");
        assert_eq!(w[1].tip[1].0, "What it does.", "at {scale}");
        assert!(w[2].tip.is_empty(), "at {scale}");
        assert_eq!(audit["counts"]["cut"], json!(2), "at {scale}");
        clean(&audit, &w, &format!("cut labels at {scale}"));
    }
}

/// An ask title too wide for the dialog is cut, seen by the audit, and has its full text on hover over the title's
/// row; a title that fits adds no widget.
#[test]
fn a_cut_ask_title_reads_in_full_on_hover() {
    let area = Rect::new(0, 0, 640, 360);
    for scale in TEXT_SIZES {
        let mut g = gfx(scale);
        let title = LONG.repeat(2);
        let v = AskView::new(&title, "Sure?");
        let (w, audit) = draw(&mut g, (-1, -1), |ui| panes::ask_dialog(ui, "ask", area, &v, "yes", "no"));
        assert_eq!(ids(&w), ["ask", "ask.title", "ask.yes", "ask.no"], "at {scale}");
        assert_eq!(w[1].tip[0].0, title, "at {scale}");
        assert_eq!(audit["cut"][0]["text"], json!(title), "at {scale}");
        clean(&audit, &w, &format!("cut title at {scale}"));
        let ring: Vec<String> = focus::ring(&infos(&w), false).iter().map(|x| x["id"].as_str().unwrap().to_string()).collect();
        assert_eq!(ring, ["ask.yes", "ask.no"], "at {scale}: the title takes no focus");
    }
}

/// Question lines the height cap drops are marked cut for the audit, the last line left says so, and the full question
/// is on hover; a question that fits is not touched.
#[test]
fn dropped_question_lines_are_cut_and_read_in_full_on_hover() {
    let question = LONG.repeat(8);
    let v = AskView::new("Sure?", &question);
    for scale in TEXT_SIZES {
        let mut g = gfx(scale);
        let (w, audit) = draw(&mut g, (-1, -1), |ui| panes::ask_dialog(ui, "ask", Rect::new(0, 0, 640, 110), &v, "yes", "no"));
        assert_eq!(audit["cut"][0]["text"], json!(question), "at {scale}: {audit}");
        assert!(audit["cut"][0]["shown"].as_str().unwrap().ends_with("..."), "at {scale}: {audit}");
        assert_eq!(ids(&w), ["ask", "ask.question", "ask.yes", "ask.no"], "at {scale}");
        assert_eq!(w[1].tip[0].0, question, "at {scale}");
        clean(&audit, &w, &format!("dropped lines at {scale}"));
        let mut g = gfx(scale);
        let (w, audit) = draw(&mut g, (-1, -1), |ui| panes::ask_dialog(ui, "ask", Rect::new(0, 0, 640, 360), &AskView::new("Sure?", "Really?"), "yes", "no"));
        assert_eq!(ids(&w), ["ask", "ask.yes", "ask.no"], "at {scale}");
        assert_eq!(audit["counts"]["cut"], json!(0), "at {scale}");
    }
}

/// Tabs past the rail's height never vanish: the rows show a window that holds the open tab, between arrows that step
/// to the tab above and below (wrapping), so every tab is a few clicks away; with too few rows for arrows the window
/// still follows the open tab (PageUp / PageDown).
#[test]
fn tabs_past_the_rail_stay_in_reach() {
    let tabs = ["One", "Two", "Three", "Four", "Five", "Six", "Seven"];
    let n = tabs.len();
    // Five rows fit 100 high (the last ends at 98): three tabs between the arrows.
    let r = Rect::new(0, 0, 300, 100);
    let mut seen = std::collections::BTreeSet::new();
    let mut open = 0;
    for step in 0..n {
        let mut g = gfx(1.0);
        let (w, _) = draw(&mut g, (-1, -1), |ui| panes::tab_rail(ui, "rail", r, &tabs, open, |i| i));
        let got = ids(&w);
        assert_eq!((got.len(), got[0].as_str(), got[4].as_str()), (5, "rail.prev", "rail.next"), "open {open}");
        assert!(got.contains(&format!("rail.{open}")), "the open tab is on view: {got:?}");
        assert!(w.iter().all(|x| x.rect.y + x.rect.h <= r.h), "inside the rail");
        assert_eq!(by_id(&w, "rail.prev"), Ok((open + n - 1) % n));
        seen.insert(open);
        open = by_id(&w, "rail.next").unwrap();
        assert_eq!(open, (step + 1) % n, "the next arrow steps and wraps");
    }
    assert_eq!(seen.len(), n, "every tab comes on view");
    // Too few rows for arrows: the window follows the open tab.
    for open in 0..n {
        let mut g = gfx(1.0);
        let (w, _) = draw(&mut g, (-1, -1), |ui| panes::tab_rail(ui, "rail", Rect::new(0, 0, 300, 50), &tabs, open, |i| i));
        assert_eq!(w.len(), 2);
        assert!(ids(&w).contains(&format!("rail.{open}")), "open {open}");
    }
    assert_eq!(panes::tab_window(3, 3, 2), (0, 3, false), "all fit");
    assert_eq!(panes::tab_window(0, 3, 2).1, 0, "no room, no rows");
}

// ---- themes, window sizes and the Kit page -------------------------------------------------------------------------

/// Every pane at once on a canvas (the gallery's Panes page draws them one at a time).
fn all_panes<A: Clone>(ui: &mut Ui<A>, a: A) {
    let img = solid(32, 18, [120, 90, 60, 255]);
    let face = solid(20, 20, [90, 90, 90, 255]);
    panes::backdrop_pane(ui, "bd", Some(&img), "img", Rect::new(0, 0, 640, 360), 128, 0.7);
    let y = panes::menu_title(ui, 20, 12, 160, "Title", "Sub");
    let it = vec![MenuItem::new("a", "First", Some('f'), a.clone()), MenuItem::new("b", "Second", None, a.clone()).off("No.")];
    panes::menu_column(ui, "menu", 20, y, 160, 18, 22, &it);
    panes::tab_rail(ui, "rail", Rect::new(200, 12, 420, 100), &["One", "Two"], 0, |_| a.clone());
    let v = SlotView { thumb: Some(&img), title: "Save".into(), lines: vec!["Place".into()], portraits: vec![&face], stamp: "now".into() };
    panes::slot_card(ui, "slot", Rect::new(200, 124, 290, 84), &v, Some(a.clone()), false, &[]);
    panes::ask_dialog(ui, "ask", Rect::new(0, 220, 640, 140), &AskView { ornate: true, ..AskView::new("Sure?", "Really?") }, a.clone(), a);
}

/// The panes draw under the default theme and the fixture with frames on and off (frames change the pixels), at the
/// four text sizes, at 1280 x 720 and at the smallest window (960 x 540), without a panic and with the same widgets.
#[test]
fn the_panes_draw_under_each_theme_and_at_the_smallest_window() {
    let mut shots: Vec<(String, Vec<u8>)> = Vec::new();
    let mut first_ids: Option<Vec<String>> = None;
    each_theme(|name| {
        for (ww, wh) in [(960, 540), (1280, 720)] {
            for scale in TEXT_SIZES {
                let mut g = gfx_at(ww, wh, scale);
                let (w, _) = draw(&mut g, (-1, -1), |ui| all_panes(ui, ()));
                let got = ids(&w);
                assert_eq!(first_ids.get_or_insert(got.clone()), &got, "{name} {ww}x{wh} {scale}");
                if (ww, scale) == (1280, 1.0) {
                    shots.push((name.to_string(), g.canvas.px.clone()));
                }
            }
        }
    });
    assert_ne!(shots[0].1, shots[1].1, "the fixture theme draws differently");
    assert_ne!(shots[1].1, shots[2].1, "frames on and off differ");
}

fn page(k: &Specimens, g: &mut Gfx, mouse: (i32, i32)) -> (Vec<Widget<KitAct>>, Value) {
    draw(g, mouse, |ui| k.draw(ui, KIT, |a| a))
}

/// The Kit page's Panes page: each pane on its stage, its ids, the text audit clean at the four sizes (cut text with
/// its hover card), the picker and switches, and the page's keys.
#[test]
fn the_kit_page_shows_each_pane() {
    assert_eq!(PAGES[5], "Panes");
    let want = ["kit.backdrop", "kit.menu.continue", "kit.rail.0", "kit.slot.0", "kit.ask.yes"];
    for (i, id) in want.iter().enumerate() {
        let k = Specimens { page: 5, pane: i, ..Specimens::default() };
        for scale in TEXT_SIZES {
            let mut g = gfx(scale);
            let (w, audit) = page(&k, &mut g, (-1, -1));
            assert!(w.iter().any(|x| x.id == *id), "{} shows {id}", PANES[i]);
            assert!((0..PANES.len()).all(|p| w.iter().any(|x| x.id == format!("kit.pane.{p}"))));
            clean(&audit, &w, &format!("{} at {scale}", PANES[i]));
        }
    }
    let mut k = Specimens { page: 5, ..Specimens::default() };
    let (w, _) = page(&k, &mut gfx(1.0), (-1, -1));
    k.act(by_id(&w, "kit.pane.1").unwrap());
    assert_eq!(k.pane, 1);
    assert!(k.key(Key::Char('n')));
    assert_eq!(k.last, "pressed new game");
    assert!(!k.key(Key::Char('z')));
    k.act(KitAct::Pane(2));
    assert!(k.key(Key::PageDown));
    assert_eq!(k.pane_tab, 1);
    k.act(KitAct::Pane(4));
    assert!(k.key(Key::Enter));
    assert_eq!(k.last, "pressed yes");
    k.act(KitAct::PaneToggle("destructive"));
    assert!(k.key(Key::Enter));
    assert_eq!(k.last, "pressed no");
    k.act(KitAct::Pane(3));
    let (w, _) = page(&k, &mut gfx(1.0), (-1, -1));
    k.act(by_id(&w, "kit.slot.1").unwrap());
    assert_eq!(k.slot, 1);
    k.act(by_id(&w, "kit.slot.0.lock").unwrap());
    let (w, _) = page(&k, &mut gfx(1.0), (-1, -1));
    assert!(by_id(&w, "kit.slot.0.delete").is_err(), "locked: delete is disabled");
    let other = Specimens { page: 0, ..Specimens::default() };
    assert!(!other.clone().key(Key::Char('n')), "keys only on the Panes page");
}

/// Review shots for the user (not a check): each pane of the Panes page under the default and the fixture theme at
/// 1280 x 720, written to `AEGIS_REVIEW_DIR` when it is set.
#[test]
fn review_shots() {
    let Ok(dir) = std::env::var("AEGIS_REVIEW_DIR") else { return };
    std::fs::create_dir_all(&dir).unwrap();
    for (tname, t) in [("default", Theme::default()), ("tallow", tallow(true))] {
        theme::set_theme(t);
        for (i, p) in PANES.iter().enumerate() {
            let k = Specimens { page: 5, pane: i, ..Specimens::default() };
            let mut g = gfx_at(1280, 720, 1.0);
            g.canvas.clear(theme::th().bg);
            let _ = page(&k, &mut g, (-1, -1));
            let file = std::fs::File::create(format!("{dir}/{}_{tname}.png", p.to_lowercase().replace(' ', "_"))).unwrap();
            let mut enc = png::Encoder::new(std::io::BufWriter::new(file), g.canvas.w as u32, g.canvas.h as u32);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().unwrap().write_image_data(&g.canvas.px).unwrap();
        }
    }
    theme::set_theme(Theme::default());
}
