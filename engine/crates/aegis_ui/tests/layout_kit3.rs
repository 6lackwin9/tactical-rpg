//! ENGINE-19 (LAYOUT-KIT3): the layout parts a shop-like screen needs: buttons on a row (each its own widget and
//! action), a selected row, a value coloured by a theme role the view names, a dimmed section under a divider, a detail
//! line that follows the row under the pointer (else the focused one), and row widget ids from the view's own ids.

use aegis_ui::canvas::{Canvas, Rect};
use aegis_ui::painter::{Fonts, View as PView};
use aegis_ui::screen_layout::{self, Act, Drawn, Field, Layout, View};
use aegis_ui::theme::Theme;
use aegis_ui::ui::{by_id, Gfx, Kit, Ui, Widget};
use serde_json::{json, Value};

const SCREEN: Rect = Rect { x: 0, y: 0, w: 640, h: 360 };

fn font() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap()
}

fn gfx() -> Gfx {
    let mut g = Gfx { canvas: Canvas::new(1280, 720), view: PView::fit(1280, 720), fonts: Fonts::new(&font()), kit: Kit::default() };
    g.canvas.clear(Theme::default().bg);
    g
}

fn buttons(on: bool) -> Value {
    json!([{"action": "buy5", "label": "+5", "enabled": on}, {"action": "buy10", "label": "+10", "enabled": on}])
}

/// A shop-shaped view: offers with +5 / +10, one in the trade (selected), one priced above the gold left (toned), one
/// at its cap; the stash, and what the shop does not buy.
fn view_json() -> Value {
    json!({
        "screen": "shop",
        "fields": {
            "offers": {"rows": [
                {"id": "potion", "label": "Potion", "detail": "30", "detail2": "2 of 6", "selected": true, "note": "Potion: restores 30 HP.", "buttons": buttons(true)},
                {"id": "sword", "label": "Iron Sword", "detail": "900", "detail2": "0 of 1", "tone": "foe", "note": "Iron Sword: a plain blade."},
                {"id": "herb", "label": "Herb", "detail": "4", "detail2": "6 of 6", "enabled": false, "note": "Herb: a bitter leaf.", "buttons": buttons(false)}
            ]},
            "stash": {"rows": [{"id": "pelt", "label": "Wolf Pelt", "detail": "12", "note": "Wolf Pelt: sells well."}]},
            "others": {"rows": [{"id": "letter", "label": "Old Letter", "detail": "x1", "enabled": false}]},
            "detail": {"text": "Pick a row."}
        },
        "actions": [
            {"id": "buy", "label": "Buy", "enabled": true},
            {"id": "buy5", "label": "+5", "enabled": true},
            {"id": "buy10", "label": "+10", "enabled": true},
            {"id": "sell", "label": "Sell", "enabled": true}
        ]
    })
}

fn view() -> View {
    View::from_json(&view_json()).unwrap()
}

fn layout() -> Layout {
    Layout::from_json(&json!({
        "screen": "shop",
        "regions": [
            {"id": "offers", "component": "list_row", "row_ids": true, "bind": {"rows": "offers", "select": "buy"}, "anchor": "top_left", "w": "49%", "h": 80},
            {"id": "stash", "component": "list_row", "row_ids": "stash.row", "bind": {"rows": "stash", "select": "sell", "then": "others"}, "anchor": "top_right", "w": "49%", "h": 80},
            {"id": "info", "component": "detail_line", "bind": {"text": "detail"}, "anchor": "bottom_left", "w": "fill", "h": 8}
        ]
    }))
    .unwrap()
}

fn draw_at(g: &mut Gfx, v: &View, mouse: (i32, i32)) -> (Vec<Widget<Act>>, Drawn) {
    let l = layout();
    assert!(l.check(v).is_empty(), "{:?}", l.check(v));
    let layers = aegis_core::assets::Layers::of(vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")]);
    let mut art = aegis_ui::art::Art::layered(layers);
    let mut ui: Ui<Act> = Ui::new(g, mouse, true);
    let drawn = screen_layout::draw(&mut ui, &mut art, Some(&l), v, SCREEN);
    (std::mem::take(&mut ui.widgets), drawn)
}

fn get<'a>(w: &'a [Widget<Act>], id: &str) -> &'a Widget<Act> {
    w.iter().find(|x| x.id == id).unwrap_or_else(|| panic!("no widget {id}: {:?}", w.iter().map(|x| &x.id).collect::<Vec<_>>()))
}

fn a(action: &str, arg: &str) -> Act {
    Act { action: action.into(), arg: Some(arg.into()) }
}

/// (1) and (6): the rows take the view's ids under the region's prefix; each row button is a widget with its action and
/// the row's id, off when the view's button, the row or the view's action is off. (2) the selected row is active. (4)
/// the `then` rows are listed after the divider, dimmed and never clicked.
#[test]
fn row_buttons_view_ids_selected_and_the_dimmed_section() {
    let mut g = gfx();
    let (w, _) = draw_at(&mut g, &view(), (-1, -1));
    assert_eq!(by_id(&w, "shop.row.potion"), Ok(a("buy", "potion")));
    assert_eq!(by_id(&w, "shop.row.potion.buy5"), Ok(a("buy5", "potion")));
    assert_eq!(by_id(&w, "shop.row.potion.buy10"), Ok(a("buy10", "potion")));
    assert!(get(&w, "shop.row.potion").active && !get(&w, "shop.row.sword").active, "the selected row only");
    assert!(w.iter().all(|x| !x.id.starts_with("shop.row.sword.")), "a row with no buttons has none");
    assert!(!get(&w, "shop.row.herb").enabled && !get(&w, "shop.row.herb.buy5").enabled, "off at the cap");
    // the buttons sit right of their row, the rows' numbers line up whether a row has buttons or not
    let (row, b5, b10) = (get(&w, "shop.row.potion").rect, get(&w, "shop.row.potion.buy5").rect, get(&w, "shop.row.potion.buy10").rect);
    assert!(b5.x >= row.x + row.w && b10.x >= b5.x + b5.w, "{row:?} {b5:?} {b10:?}");
    assert_eq!(get(&w, "shop.row.sword").rect.w, row.w, "every row leaves the buttons' column free");
    // the stash under its own prefix, the others after it: shown, never clicked
    assert_eq!(by_id(&w, "stash.row.pelt"), Ok(a("sell", "pelt")));
    let other = get(&w, "shop.stash.then.0");
    assert!(other.action.is_none() && !other.enabled && other.rect.y > get(&w, "stash.row.pelt").rect.y);
    // the view's action off: its buttons are off on every row
    let mut v = view_json();
    v["actions"][1]["enabled"] = json!(false);
    let (w, _) = draw_at(&mut g, &View::from_json(&v).unwrap(), (-1, -1));
    assert!(!get(&w, "shop.row.potion.buy5").enabled && get(&w, "shop.row.potion.buy10").enabled);
    // with no `row_ids` a list keeps its index ids
    let mut l = layout();
    if let screen_layout::Component::ListRow { ids, .. } = &mut l.regions[0].component {
        *ids = None;
    }
    let layers = aegis_core::assets::Layers::of(vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")]);
    let mut art = aegis_ui::art::Art::layered(layers);
    let mut ui: Ui<Act> = Ui::new(&mut g, (-1, -1), true);
    screen_layout::draw(&mut ui, &mut art, Some(&l), &view(), SCREEN);
    assert_eq!(by_id(&ui.widgets, "shop.offers.0"), Ok(a("buy", "potion")));
    assert_eq!(by_id(&ui.widgets, "shop.offers.0.buy5"), Ok(a("buy5", "potion")));
}

/// (3): a toned `detail` is drawn in the theme role's colour (the price above the gold left in `foe`).
#[test]
fn a_toned_value_takes_the_theme_role() {
    let foe = Theme::default().foe;
    let count = |tone: &str| {
        let mut v = view_json();
        v["fields"]["offers"]["rows"][1]["tone"] = json!(tone);
        let mut g = gfx();
        let (w, _) = draw_at(&mut g, &View::from_json(&v).unwrap(), (-1, -1));
        let r = get(&w, "shop.row.sword").rect;
        // the row's right half, in canvas pixels (2 per unit)
        let mut n = 0;
        for y in r.y * 2..(r.y + r.h) * 2 {
            for x in (r.x + r.w / 2) * 2..(r.x + r.w) * 2 {
                let i = ((y * g.canvas.w + x) * 4) as usize;
                if g.canvas.px[i..i + 3] == foe[0..3] {
                    n += 1;
                }
            }
        }
        n
    };
    let (toned, plain) = (count("foe"), count(""));
    assert!(toned > 0 && plain == 0, "foe pixels: toned {toned}, plain {plain}");
}

/// (5): the detail line shows the note of the row under the pointer, else the focused row's, else its bound text; the
/// screen gets the hot row back (`Drawn::hot`).
#[test]
fn the_detail_line_follows_the_hot_row() {
    let mut g = gfx();
    let (w, d) = draw_at(&mut g, &view(), (-1, -1));
    assert_eq!(get(&w, "shop.info").text, "Pick a row.", "the bound text with no row hot");
    assert_eq!(d.hot, None);
    let r = get(&w, "shop.row.sword").rect;
    let (w, d) = draw_at(&mut g, &view(), (r.x + 4, r.y + r.h / 2));
    assert_eq!(get(&w, "shop.info").text, "Iron Sword: a plain blade.");
    assert_eq!(d.hot, Some(("shop.row.sword".to_string(), "Iron Sword: a plain blade.".to_string())));
    // a focused row (or its button) without the pointer; the pointer wins over the focus
    g.kit.focus = Some("shop.row.potion.buy5".into());
    g.kit.focus_visible = true;
    let (w, _) = draw_at(&mut g, &view(), (-1, -1));
    assert_eq!(get(&w, "shop.info").text, "Potion: restores 30 HP.");
    g.kit.focus = Some("stash.row.pelt".into());
    let (w, _) = draw_at(&mut g, &view(), (-1, -1));
    assert_eq!(get(&w, "shop.info").text, "Wolf Pelt: sells well.");
    let (w, _) = draw_at(&mut g, &view(), (r.x + 4, r.y + r.h / 2));
    assert_eq!(get(&w, "shop.info").text, "Iron Sword: a plain blade.");
}

/// The new parts go out and back through JSON, and what does not fit is refused: an unknown tone, a button with no
/// action, two lists with the same row ids, `row_ids` off a list, a button action the view does not offer.
#[test]
fn the_new_parts_parse_or_are_refused() {
    let v = view();
    assert_eq!(View::from_json(&v.to_json()).unwrap(), v);
    let rows = |r: Value| View::from_json(&json!({"screen": "s", "fields": {"x": {"rows": [r]}}, "actions": []})).err().unwrap_or_default().join(" | ");
    assert!(rows(json!({"id": "a", "tone": "pink"})).contains("theme colour role, not 'pink'"));
    assert!(rows(json!({"id": "a", "buttons": [{"label": "+5"}]})).contains("a row button is"));
    assert_eq!(rows(json!({"id": "a", "tone": "good", "selected": true, "note": "n", "buttons": [{"action": "b", "label": "+5"}]})), "");
    let bad = |regions: Value| Layout::from_json(&json!({"screen": "s", "regions": regions})).err().unwrap_or_default().join(" | ");
    let list = |id: &str, ids: Value| json!({"id": id, "component": "list_row", "row_ids": ids, "bind": {"rows": "x"}});
    assert!(bad(json!([list("a", json!(true)), list("b", json!(true))])).contains("another list_row has the row ids 's.row'"));
    assert!(bad(json!([list("a", json!("bad id!"))])).contains("'row_ids' is true or a widget id prefix"));
    assert!(bad(json!([{"id": "t", "component": "text_block", "row_ids": true, "bind": {"text": "x"}}])).contains("'row_ids' is for a list_row"));
    assert_eq!(bad(json!([list("a", json!(true)), list("b", json!("s.other")), {"id": "d", "component": "detail_line"}])), "");
    let mut v = view_json();
    v["actions"].as_array_mut().unwrap().remove(2);
    let problems = layout().check(&View::from_json(&v).unwrap());
    assert!(problems.iter().any(|p| p.contains("no action 'buy10'")), "{problems:?}");
    assert!(matches!(view().get("detail"), Some(Field::Text(_))));
}

/// PR7-39 Stage B: the styles a hand-drawn screen's look needs: a plain `box` panel (thin edge, a theme fill), a
/// heading `muted` or `title` (and bound to a field), a `muted` or `right` text block whose `\n` starts a line, a
/// `plain` list (numbers in the body face) and a second number's `tone2`.
#[test]
fn the_styles_for_a_hand_drawn_look_parse_draw_or_are_refused() {
    let bad = |regions: Value| Layout::from_json(&json!({"screen": "shop", "regions": regions})).err().unwrap_or_default().join(" | ");
    let ok = json!([
        {"id": "col", "component": "panel_shell", "style": "box", "fill": "column", "w": 50, "h": 40},
        {"id": "t", "component": "heading", "style": "title", "bind": {"text": "detail"}, "w": 50, "h": 4},
        {"id": "h", "component": "heading", "style": "muted", "text": "Shop", "w": 50, "h": 2, "y": 5},
        {"id": "r", "component": "text_block", "style": "right", "bind": {"text": "detail"}, "w": 50, "h": 3, "y": 8},
        {"id": "m", "in": "col", "component": "text_block", "style": "muted", "bind": {"text": "detail"}},
        {"id": "l", "in": "col", "component": "list_row", "style": "plain", "bind": {"rows": "offers"}}
    ]);
    assert_eq!(bad(ok.clone()), "");
    assert!(bad(json!([{"id": "c", "component": "panel_shell", "style": "popup", "fill": "column"}])).contains("'fill' is a box panel_shell's fill"));
    assert!(bad(json!([{"id": "h", "component": "heading", "style": "loud", "text": "x"}])).contains("muted or title, not 'loud'"));
    assert!(bad(json!([{"id": "h", "component": "heading", "text": "x", "bind": {"text": "detail"}}])).contains("not both"));
    assert!(bad(json!([{"id": "l", "component": "list_row", "style": "muted", "bind": {"rows": "offers"}}])).contains("plain, not 'muted'"));
    assert!(bad(json!([{"id": "t", "component": "text_block", "style": "plain", "bind": {"text": "detail"}}])).contains("muted or right, not 'plain'"));
    // drawn: the box is a shell widget; two lines of a text block sit one under the other
    let mut v = view_json();
    v["fields"]["detail"] = json!({"text": "Total: 0\nPurse: 5"});
    v["fields"]["offers"]["rows"][0]["tone2"] = json!("info");
    let v = View::from_json(&v).unwrap();
    assert_eq!(View::from_json(&v.to_json()).unwrap(), v, "tone2 goes out and back");
    let l = Layout::from_json(&json!({"screen": "shop", "regions": ok})).unwrap();
    let mut g = gfx();
    g.fonts.audit = Some(Vec::new());
    let layers = aegis_core::assets::Layers::of(vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")]);
    let mut art = aegis_ui::art::Art::layered(layers);
    let mut ui: Ui<Act> = Ui::new(&mut g, (-1, -1), true);
    screen_layout::draw(&mut ui, &mut art, Some(&l), &v, SCREEN);
    assert_eq!(get(&ui.widgets, "shop.col").kind, "shell");
    drop(ui);
    let recs = g.fonts.audit.take().unwrap_or_default();
    let y = |t: &str| recs.iter().filter(|r| r.text == t).map(|r| r.y).fold(f32::MAX, f32::min);
    let (a, b) = (y("Total: 0"), y("Purse: 5"));
    assert!(a < f32::MAX && b > a, "the second line under the first: {a} {b}");
}

/// ENGINE-20 (TOOLS-HELP3, FB-361): a text or stats field takes a tone from the view (a theme role for every line, or
/// one per line), drawn in that colour; it goes out and back in JSON and an unknown role or a tone on another kind of
/// field is refused.
#[test]
fn a_text_field_takes_its_tone_from_the_view() {
    let v = View::from_json(&json!({"screen": "s", "fields": {
        "totals": {"text": "Total: -5\nPurse: 0 -> -5", "tone": ["foe", ""]},
        "one": {"text": "a\nb", "tone": "good"},
        "stats": {"stats": [["Gold", "3"]], "tone": "foe"}
    }}))
    .expect("toned fields parse");
    assert_eq!((v.tone_of("totals", 0), v.tone_of("totals", 1), v.tone_of("one", 1), v.tone_of("stats", 0)), (Some("foe"), None, Some("good"), Some("foe")));
    assert_eq!(View::from_json(&v.to_json()).unwrap(), v, "out and back");
    assert_eq!(View::new("s").text("x", "y").tone("x", &["", ""]).tones, Vec::new(), "all default: untoned");
    for (field, why) in [(json!({"text": "x", "tone": "loud"}), "not 'loud'"), (json!({"rows": [], "tone": "foe"}), "text or stats field"), (json!({"text": "x", "tone": 3}), "a role or a list")] {
        let err = View::from_json(&json!({"screen": "s", "fields": {"f": field}})).err().unwrap_or_default().join(" ");
        assert!(err.contains(why), "{err}");
    }
    // drawn: the toned line holds the theme's foe colour, the same text untoned does not
    let l = Layout::from_json(&json!({"screen": "s", "regions": [{"id": "t", "component": "text_block", "bind": {"text": "totals"}, "anchor": "top_left", "w": 60, "h": 10}]})).unwrap();
    let foe = Theme::default().foe;
    let has_foe = |v: &View| {
        let mut g = gfx();
        let mut art = aegis_ui::art::Art::new(None);
        let mut ui: Ui<Act> = Ui::new(&mut g, (-1, -1), true);
        screen_layout::draw(&mut ui, &mut art, Some(&l), v, SCREEN);
        drop(ui);
        g.canvas.px.chunks(4).any(|p| (0..3).all(|k| (p[k] as i32 - foe[k] as i32).abs() <= 8))
    };
    let plain = View { tones: Vec::new(), ..v.clone() };
    assert!(has_foe(&v) && !has_foe(&plain), "the total in foe");
}
