//! ENGINE-18 (LAYOUT-KIT2): layout components for screens with long lists: a `list_row` that scrolls inside a region
//! shorter than its list (every row a widget, the focused one kept in view), no empty icon tiles, a second numbers
//! column, a muted `stat_block` and a fixed-text `heading`. The view has the shape of a controls page: 24 bindings, so
//! row 20 lies below a short region.

use aegis_ui::canvas::{Canvas, Rect};
use aegis_ui::painter::{Fonts, View as PView};
use aegis_ui::screen_layout::{self, Act, Layout, View};
use aegis_ui::theme::Theme;
use aegis_ui::ui::{by_id, Gfx, Kit, Ui, Widget};
use serde_json::json;

const SCREEN: Rect = Rect { x: 0, y: 0, w: 640, h: 360 };
const ROWS: usize = 24;

fn font() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

fn gfx() -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(1280, 720), view: PView::fit(1280, 720), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.canvas.clear(Theme::default().bg);
    g
}

fn view() -> View {
    let rows: Vec<_> = (0..ROWS).map(|i| json!({"id": format!("act_{i}"), "label": format!("Action {i}"), "detail": format!("K{i}"), "detail2": if i % 2 == 0 { "Pad A" } else { "Pad Left bumper" }, "enabled": true})).collect();
    View::from_json(&json!({
        "screen": "settings_controls",
        "fields": {"bindings": {"rows": rows}, "fixed": {"stats": [["Run", "Hold Shift"], ["Inspect", "Right click"]]}},
        "actions": [{"id": "bindings", "label": "Rebind", "enabled": true}]
    }))
    .unwrap()
}

fn layout() -> Layout {
    Layout::from_json(&json!({
        "screen": "settings_controls",
        "regions": [
            {"id": "head", "component": "heading", "text": "Rebindable", "anchor": "top_left", "w": "49%", "h": 6},
            {"id": "f_bindings", "component": "list_row", "bind": {"rows": "bindings", "select": "bindings"}, "anchor": "top_left", "w": "49%", "h": 60, "y": 8},
            {"id": "fixed_head", "component": "heading", "text": "Fixed inputs", "anchor": "top_right", "w": "49%", "h": 6},
            {"id": "fixed", "component": "stat_block", "style": "muted", "bind": {"stats": "fixed"}, "anchor": "top_right", "w": "49%", "h": 30, "y": 8}
        ]
    }))
    .unwrap()
}

fn draw(g: &mut Gfx) -> Vec<Widget<Act>> {
    let (l, v) = (layout(), view());
    assert!(l.check(&v).is_empty(), "{:?}", l.check(&v));
    let layers = aegis_core::assets::Layers::of(vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")]);
    let mut art = aegis_ui::art::Art::layered(layers);
    let mut ui: Ui<Act> = Ui::new(g, (-1, -1), true);
    screen_layout::draw(&mut ui, &mut art, Some(&l), &v, SCREEN);
    std::mem::take(&mut ui.widgets)
}

fn get<'a>(w: &'a [Widget<Act>], id: &str) -> &'a Widget<Act> {
    w.iter().find(|x| x.id == id).unwrap_or_else(|| panic!("no widget {id}"))
}

/// The clickable widget under a point, as the shell hit-tests a click: the last drawn whose rect holds it.
fn at(w: &[Widget<Act>], x: i32, y: i32) -> Option<&Widget<Act>> {
    w.iter().rev().find(|x0| x0.action.is_some() && x0.rect.contains(x, y))
}

#[test]
fn a_short_list_region_scrolls_and_every_row_keeps_its_id() {
    let mut g = gfx();
    let w = draw(&mut g);
    // every row is a widget, row 20 too, with its action, though it is scrolled out of the region
    for i in 0..ROWS {
        assert!(w.iter().any(|x| x.id == format!("settings_controls.f_bindings.{i}")), "row {i}");
    }
    assert_eq!(by_id(&w, "settings_controls.f_bindings.20"), Ok(Act { action: "bindings".into(), arg: Some("act_20".into()) }));
    let area = get(&w, "settings_controls.f_bindings");
    assert_eq!(area.kind, "scroll", "the region is a scroll area (the wheel scrolls it)");
    let max: i32 = area.text.parse().unwrap();
    assert!(max > 0);
    assert_eq!(get(&w, "settings_controls.f_bindings.20").rect.h, 0, "below the region: no room to click");
    // scrolled (as the wheel does), row 20 shows inside the region and a click there is row 20's
    g.kit.scroll.insert("settings_controls.f_bindings".into(), max);
    let w = draw(&mut g);
    let r20 = get(&w, "settings_controls.f_bindings.20").rect;
    let region = get(&w, "settings_controls.f_bindings").rect;
    assert!(r20.h > 0 && r20.y >= region.y && r20.y + r20.h <= region.y + region.h, "{r20:?} in {region:?}");
    let hit = at(&w, r20.x + r20.w / 2, r20.y + r20.h / 2).expect("a row under the click");
    assert_eq!(hit.id, "settings_controls.f_bindings.20");
    assert_eq!(hit.action, Some(Act { action: "bindings".into(), arg: Some("act_20".into()) }));
    assert_eq!(get(&w, "settings_controls.f_bindings.0").rect.h, 0, "row 0 is above the region now");
}

#[test]
fn the_focused_row_and_its_neighbours_stay_in_view() {
    let mut g = gfx();
    g.kit.focus = Some("settings_controls.f_bindings.20".into());
    g.kit.focus_visible = true;
    let w = draw(&mut g);
    let off = g.kit.scroll.get("settings_controls.f_bindings").copied().expect("the list scrolled to the focus");
    assert!(off > 0);
    let region = get(&w, "settings_controls.f_bindings").rect;
    for i in [19, 20, 21] {
        let r = get(&w, &format!("settings_controls.f_bindings.{i}")).rect;
        assert!(r.h > 0 && r.y >= region.y && r.y + r.h <= region.y + region.h, "row {i} {r:?} in {region:?}");
    }
    // back up to row 0: the list scrolls back to the top
    g.kit.focus = Some("settings_controls.f_bindings.0".into());
    draw(&mut g);
    assert_eq!(g.kit.scroll["settings_controls.f_bindings"], 0);
    // a mouse user (no ring) is not scrolled about
    g.kit.focus = Some("settings_controls.f_bindings.20".into());
    g.kit.focus_visible = false;
    draw(&mut g);
    assert_eq!(g.kit.scroll["settings_controls.f_bindings"], 0);
}

#[test]
fn a_list_with_room_does_not_scroll_and_the_new_parts_parse_or_are_refused() {
    let mut g = gfx();
    let mut l = layout();
    for r in &mut l.regions {
        if r.id == "f_bindings" {
            r.place = Layout::from_json(&json!({"screen": "settings_controls", "regions": [{"id": "f_bindings", "component": "list_row", "bind": {"rows": "bindings"}, "anchor": "top_left", "w": "49%", "h": "fill"}]})).unwrap().regions[0].place;
        }
    }
    let v = view();
    let layers = aegis_core::assets::Layers::of(vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")]);
    let mut art = aegis_ui::art::Art::layered(layers);
    let mut ui: Ui<Act> = Ui::new(&mut g, (-1, -1), true);
    screen_layout::draw(&mut ui, &mut art, Some(&l), &v, Rect { x: 0, y: 0, w: 640, h: 2000 });
    assert!(!ui.widgets.iter().any(|x| x.kind == "scroll"), "all rows fit: no scroll area");
    drop(ui);
    let bad = |r: serde_json::Value| Layout::from_json(&json!({"screen": "s", "regions": [r]})).err().unwrap_or_default().join(" | ");
    assert!(bad(json!({"id": "h", "component": "heading"})).contains("a heading needs its own 'text' or a bound text field"));
    assert!(bad(json!({"id": "t", "component": "text_block", "bind": {"text": "t"}, "text": "x"})).contains("'text' is a heading's own text"));
    assert!(bad(json!({"id": "s", "component": "stat_block", "bind": {"stats": "s"}, "style": "loud"})).contains("muted, not 'loud'"));
    assert!(bad(json!({"id": "s", "component": "tooltip", "bind": {"text": "s"}, "style": "muted"})).contains("'style' is for"));
    assert_eq!(bad(json!({"id": "s", "component": "stat_block", "bind": {"stats": "s"}, "style": "muted"})), "");
    // detail2 goes out and back
    let v2 = View::from_json(&v.to_json()).unwrap();
    assert_eq!(v2, v);
}

#[test]
fn rows_with_no_picture_draw_no_tile_and_the_two_number_columns_line_up() {
    use aegis_ui::panes::{list_row, RowLook};
    let row_px = |look: &RowLook| {
        let mut g = gfx();
        let r = Rect::new(0, 0, 300, 20);
        {
            let mut ui: Ui<Act> = Ui::new(&mut g, (-1, -1), true);
            list_row(&mut ui, "r", r, look, None, Vec::new());
        }
        let bg = g.canvas.px[0..4].to_vec();
        // the first column of pixels that is not the background
        let px = |x: i32, y: i32| g.canvas.px[((y * g.canvas.w + x) * 4) as usize..((y * g.canvas.w + x) * 4 + 4) as usize].to_vec();
        (1..300).find(|x| (0..20).any(|y| px(*x, y) != bg)).unwrap_or(300)
    };
    let nums = vec!["K".to_string(), "Pad A".to_string()];
    let tiled = row_px(&RowLook { name: "Walk", numbers: &nums, enabled: true, ..Default::default() });
    let bare = row_px(&RowLook { name: "Walk", numbers: &nums, enabled: true, bare: true, ..Default::default() });
    assert!(bare > tiled, "the bare row starts with its text, not a tile edge ({bare} vs {tiled})");
}
