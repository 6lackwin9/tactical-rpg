//! PLAYUI1-K: the kit's icon button (image or drawn shape, hover card, focus, toggled, disabled) and its collapsible
//! section header, drawn on a 640x360 canvas with the engine font, and their specimens on the Kit widgets page.

use aegis_ui::canvas::{Canvas, Image, Rect};
use aegis_ui::focus;
use aegis_ui::painter::{Fonts, View};
use aegis_ui::specimens::{KitAct, Specimens};
use aegis_ui::ui::{by_id, draw_tip, hit, tip, Gfx, Glyph, Kit, Ui, Widget, BG};
use serde_json::Value;

fn gfx() -> Gfx {
    let font = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    let mut g = Gfx { canvas: Canvas::new(640, 360), view: View::fit(640, 360), fonts: Fonts::new(&font), kit: Kit::default() };
    g.canvas.clear(BG);
    g
}

fn px(g: &Gfx, x: i32, y: i32) -> [u8; 4] {
    let i = ((y * g.canvas.w + x) * 4) as usize;
    [g.canvas.px[i], g.canvas.px[i + 1], g.canvas.px[i + 2], g.canvas.px[i + 3]]
}

fn infos<A>(w: &[Widget<A>]) -> Vec<Value> {
    w.iter().map(|w| w.info()).collect()
}

fn solid(n: i32, c: [u8; 4]) -> Image {
    Image { w: n, h: n, px: c.repeat((n * n) as usize) }
}

#[test]
fn an_icon_button_draws_its_image_or_its_shape_and_records_its_states() {
    let mut g = gfx();
    let red = [200, 30, 30, 255];
    let r = Rect::new(20, 20, 16, 16);
    let widgets = {
        let mut ui: Ui<&str> = Ui::new(&mut g, (-1, -1), true);
        let n = ui.icon_px(r);
        assert_eq!(n, Ui::<&str>::icon_rect(r).w, "one source pixel per canvas pixel at scale 1");
        ui.icon_button("img", r, "Image", Some(&solid(n, red)), Glyph::Lock, "img", true, false, Vec::new());
        ui.icon_button("shape", Rect::new(60, 20, 16, 16), "Lock", None, Glyph::Lock, "shape", true, true, tip("Unlock", ""));
        ui.icon_button("off", Rect::new(100, 20, 16, 16), "Delete", None, Glyph::Delete, "off", false, false, tip("Delete", "Unlock the save to delete it."));
        ui.widgets
    };
    let ir = Ui::<&str>::icon_rect(r);
    assert_eq!((ir.w, ir.h, ir.x, ir.y), (12, 12, 22, 22), "a 16 px hit box holds a 12 px icon, centred");
    assert_eq!(px(&g, ir.x + 6, ir.y + 6), red, "the image fills the icon square");
    assert_eq!(px(&g, r.x, r.y), [BG[0], BG[1], BG[2], 255], "an idle, untoggled button draws no box");
    let shape = Ui::<&str>::icon_rect(Rect::new(60, 20, 16, 16));
    let drawn = (shape.y..shape.y + shape.h).any(|y| (shape.x..shape.x + shape.w).any(|x| px(&g, x, y) != [BG[0], BG[1], BG[2], 255]));
    assert!(drawn, "a missing image draws the shape");

    let info = infos(&widgets);
    assert_eq!(info[0]["kind"], "icon_button");
    assert_eq!(info[0]["text"], "Image", "the label names it for the agent link");
    assert_eq!(info[0]["tip"][0], "Image", "an empty hover card takes the label as its title");
    assert_eq!((info[1]["active"].as_bool(), info[1]["tip"][0].as_str()), (Some(true), Some("Unlock")), "toggled is the active state");
    assert_eq!(hit(&widgets, (64, 24)), Some("shape"));
    assert_eq!(hit(&widgets, (104, 24)), None, "a disabled button takes no click");
    assert!(by_id(&widgets, "off").is_err());
    let ring: Vec<&str> = focus::ring(&info, false).iter().filter_map(|w| w["id"].as_str()).collect();
    assert_eq!(ring, ["img", "shape"], "enabled icon buttons take the keyboard focus, a disabled one does not");
}

#[test]
fn a_disabled_icon_button_still_shows_why_on_hover() {
    let mut g = gfx();
    let mut ui: Ui<&str> = Ui::new(&mut g, (104, 24), true);
    ui.icon_button("off", Rect::new(100, 20, 16, 16), "Delete", None, Glyph::Delete, "off", false, false, tip("Delete", "Unlock the save to delete it."));
    let w = std::mem::take(&mut ui.widgets);
    let before = ui.p.c.px.clone();
    draw_tip(&mut ui.p, &w, (104, 24));
    assert_ne!(ui.p.c.px, before, "the hover card is drawn over a disabled button");
}

/// PLAYUI1-S: icon buttons in a clickable row's corner (a save slot's lock and delete): a click on an icon, enabled or
/// not, never reaches the row; a disabled plain button keeps today's pass-through.
#[test]
fn an_icon_over_a_row_takes_its_click_even_when_disabled() {
    let mut g = gfx();
    let mut ui: Ui<&str> = Ui::new(&mut g, (-1, -1), true);
    let row = Rect::new(10, 10, 200, 30);
    ui.card("row", row, &[("Slot 1".into(), BG)], Some("row"), false, true, Vec::new());
    ui.icon_button("lock", Rect::new(170, 14, 16, 16), "Lock", None, Glyph::Unlock, "lock", true, false, Vec::new());
    ui.icon_button("del", Rect::new(190, 14, 16, 16), "Delete", None, Glyph::Delete, "del", false, false, Vec::new());
    ui.button("plain", Rect::new(20, 30, 20, 8), "Off", "plain", false);
    let w = ui.widgets;
    assert_eq!(hit(&w, (40, 20)), Some("row"), "the row is the click target away from its icons");
    assert_eq!(hit(&w, (176, 20)), Some("lock"), "the enabled icon wins over the row");
    assert_eq!(hit(&w, (196, 20)), None, "the disabled icon swallows the click: the row does not fire");
    assert_eq!(hit(&w, (25, 33)), Some("row"), "a disabled plain button lets the click through as before");
}

#[test]
fn a_section_header_reports_open_and_sends_its_toggle() {
    let mut g = gfx();
    let mut ui: Ui<&str> = Ui::new(&mut g, (-1, -1), true);
    assert!(ui.section("legend", Rect::new(10, 10, 120, 14), "Here (3)", "", true, "fold", Vec::new()));
    assert!(!ui.section("log", Rect::new(10, 30, 120, 14), "Log", "12", false, "open", Vec::new()));
    let info = infos(&ui.widgets);
    assert_eq!((info[0]["kind"].as_str(), info[0]["text"].as_str(), info[0]["active"].as_bool()), (Some("section"), Some("Here (3)"), Some(true)));
    assert_eq!(info[1]["active"], false);
    assert_eq!(hit(&ui.widgets, (100, 16)), Some("fold"), "the whole header is the click target");
    assert_eq!(by_id(&ui.widgets, "log"), Ok("open"));
    assert_eq!(focus::ring(&info, false).len(), 2, "both headers take the focus");
}

#[test]
fn the_kit_page_shows_icon_buttons_and_a_section_that_fold_and_lock() {
    let draw = |k: &Specimens| -> Vec<Widget<KitAct>> {
        let mut g = gfx();
        let mut ui: Ui<KitAct> = Ui::new(&mut g, (-1, -1), true);
        k.draw(&mut ui, Rect::new(112, 22, 522, 332), |a| a);
        ui.widgets
    };
    let mut k = Specimens::default();
    let w = draw(&k);
    let kinds = |w: &[Widget<KitAct>], kind: &str| w.iter().filter(|x| x.kind == kind).count();
    assert!(kinds(&w, "icon_button") >= 5, "image, drawn shapes, toggled and disabled specimens");
    assert!(w.iter().any(|x| x.kind == "icon_button" && !x.enabled), "a disabled specimen");
    assert_eq!(kinds(&w, "section"), 1);
    let rows = |w: &[Widget<KitAct>]| w.iter().filter(|x| x.id.starts_with("kit.section.row.")).count();
    assert!(rows(&w) > 0, "the section starts open with its rows");
    assert!(by_id(&w, "kit.icon.delete").is_ok(), "delete works while unlocked");

    k.act(by_id(&w, "kit.icon.lock").unwrap());
    k.act(by_id(&w, "kit.section").unwrap());
    assert_eq!((k.locked, k.open, k.last.as_str()), (true, false, "section folded"));
    let w = draw(&k);
    assert!(w.iter().any(|x| x.id == "kit.icon.lock" && x.active), "the lock shows toggled");
    assert!(by_id(&w, "kit.icon.delete").is_err(), "a locked lock disables delete");
    assert_eq!(rows(&w), 0, "a folded section draws no content");
}
