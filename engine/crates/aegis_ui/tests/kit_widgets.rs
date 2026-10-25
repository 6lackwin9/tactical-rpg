//! PLAYUI1-K: the kit's icon button (image or drawn shape, hover card, focus, toggled, disabled) and its collapsible
//! section header, drawn on a 640x360 canvas with the engine font, and their specimens on the Kit widgets page.
//! RULESET1-E: the timeline's segment parts, hatching, links and marks, and the Kit page's timeline specimen.

use aegis_ui::canvas::{Canvas, Image, Rect};
use aegis_ui::focus;
use aegis_ui::painter::{Fonts, View};
use aegis_ui::specimens::{KitAct, Specimens};
use aegis_ui::text_audit;
use aegis_ui::timeline::{Link, Mark, Part, PartStyle, Section, Segment, Timeline, Track, LABEL_W};
use aegis_ui::ui::{by_id, draw_tip, hit, tip, Gfx, Glyph, Kit, Ui, Widget, BG, TEXT};
use serde_json::{json, Value};

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

// RULESET1-E: the timeline's segment parts, hatching, links and marks, all empty by default (the look and the clicks
// of a timeline without them stay as they were).

const FILL: [u8; 4] = [80, 100, 140, 255];

fn one_side(blocks: i32, tracks: Vec<Track>) -> Timeline {
    Timeline { blocks, sections: vec![Section { title: "Side".into(), colour: TEXT, tracks }], ..Timeline::default() }
}

fn track(key: &str, segments: Vec<Segment>) -> Track {
    Track { key: key.into(), label: key.to_uppercase(), colour: TEXT, segments, ..Track::default() }
}

fn seg(key: &str, start: i32, end: i32) -> Segment {
    Segment { key: key.into(), start, end, fill: FILL, ..Segment::default() }
}

/// Draws `t` with its lane 480 wide (short labels keep the label column at its least), so a block of 6 is 80 px.
fn draw_tl(g: &mut Gfx, t: &Timeline) -> Vec<Widget<String>> {
    let mut ui: Ui<String> = Ui::new(g, (-1, -1), true);
    t.draw(&mut ui, "tl", Rect::new(10, 10, LABEL_W + 4 + 480, 120), |b| format!("block {b}"), |k| format!("track {k}"), |k, s| format!("seg {k} {s}"));
    ui.widgets
}

fn rect_of<A>(w: &[Widget<A>], id: &str) -> Option<Rect> {
    w.iter().find(|x| x.id == id).map(|x| x.rect)
}

/// The centre x of block `b`'s cell on the ruler.
fn block_mid<A>(w: &[Widget<A>], b: i32) -> i32 {
    let r = rect_of(w, &format!("tl.block.{b}")).unwrap();
    r.x + r.w / 2
}

fn darker(a: [u8; 4], b: [u8; 4]) -> bool {
    (0..3).all(|k| a[k] < b[k])
}

fn brighter(a: [u8; 4], b: [u8; 4]) -> bool {
    (0..3).all(|k| a[k] > b[k])
}

fn boxes<A>(w: &[Widget<A>]) -> Vec<(String, f32, f32, f32, f32)> {
    w.iter().map(|x| (x.id.clone(), x.rect.x as f32, x.rect.y as f32, x.rect.w as f32, x.rect.h as f32)).collect()
}

#[test]
fn a_plain_timeline_draws_its_segments_as_before_with_no_new_widgets() {
    let mut g = gfx();
    let w = draw_tl(&mut g, &one_side(6, vec![track("a", vec![seg("s", 2, 4)])]));
    let ids: Vec<&str> = w.iter().map(|x| x.id.as_str()).collect();
    assert_eq!(ids, ["tl.block.1", "tl.block.2", "tl.block.3", "tl.block.4", "tl.block.5", "tl.block.6", "tl.rows", "tl.track.a", "tl.seg.a.s"], "no link or mark widgets by default");
    let bar = rect_of(&w, "tl.seg.a.s").unwrap();
    assert_eq!(px(&g, block_mid(&w, 3), bar.y + bar.h / 2), FILL, "a segment without parts is one plain bar of its fill");
}

#[test]
fn parts_shade_brighten_and_hatch_only_their_own_blocks() {
    let mut g = gfx();
    let parts = vec![
        Part { start: 1, end: 2, style: PartStyle::Windup },
        Part { start: 3, end: 3, style: PartStyle::Impact },
        Part { start: 4, end: 5, style: PartStyle::Recovery },
        Part { start: 7, end: 9, style: PartStyle::Impact },
    ];
    let t = one_side(6, vec![track("a", vec![Segment { parts, ..seg("s", 1, 6) }]), track("b", vec![Segment { hatched: true, ..seg("h", 1, 6) }])]);
    let w = draw_tl(&mut g, &t);
    let bar = rect_of(&w, "tl.seg.a.s").unwrap();
    let y = bar.y + bar.h / 2;
    let windup = px(&g, block_mid(&w, 1), y);
    let impact = px(&g, block_mid(&w, 3), y);
    assert!(darker(windup, FILL), "a windup is shaded darker: {windup:?}");
    assert!(brighter(impact, FILL), "an impact is brighter: {impact:?}");
    let b3 = rect_of(&w, "tl.block.3").unwrap();
    let edge = px(&g, b3.x + b3.w - 1, y);
    assert!(brighter(edge, impact), "the impact's last block ends in a light edge: {edge:?}");
    assert_eq!(px(&g, block_mid(&w, 6), y), FILL, "a part past the segment's blocks draws nothing; B6 stays plain");
    let stripes = |bar: Rect, from: i32, to: i32| -> Vec<[u8; 4]> {
        let (a, b) = (rect_of(&w, &format!("tl.block.{from}")).unwrap(), rect_of(&w, &format!("tl.block.{to}")).unwrap());
        let mut v: Vec<[u8; 4]> = (a.x + 2..b.x + b.w - 2).map(|x| px(&g, x, bar.y + bar.h / 2)).collect();
        v.sort();
        v.dedup();
        v
    };
    for (what, colours) in [("recovery", stripes(bar, 4, 5)), ("hatched whole", stripes(rect_of(&w, "tl.seg.b.h").unwrap(), 1, 6))] {
        assert!(colours.len() >= 2, "{what}: hatching gives stripes, not one colour: {colours:?}");
        assert!(colours.iter().all(|c| (0..3).all(|k| c[k] <= FILL[k])), "{what}: stripes only darken: {colours:?}");
    }
}

#[test]
fn marks_sit_inside_their_blocks_take_no_click_and_show_on_a_masked_track() {
    let mut g = gfx();
    g.fonts.audit = Some(Vec::new());
    let mark = |key: &str, block, glyph: &str| Mark { key: key.into(), block, glyph: glyph.into(), colour: [240, 100, 92, 255], tip: tip(&format!("Mark {key}"), "") };
    let a = Track { marks: vec![mark("m1", 3, "!"), mark("m2", 3, "x"), mark("off", 9, "?")], ..track("a", vec![Segment { label: "A label long enough to run on under the marks".into(), ..seg("s", 2, 4) }]) };
    let b = Track { masked: Some("Plans in secret, a note long enough to reach the mark".into()), marks: vec![mark("m", 1, "!")], ..track("b", vec![seg("hidden", 1, 6)]) };
    let w = draw_tl(&mut g, &one_side(6, vec![a, b]));
    let (m1, m2, mb) = (rect_of(&w, "tl.mark.a.m1").unwrap(), rect_of(&w, "tl.mark.a.m2").unwrap(), rect_of(&w, "tl.mark.b.m").unwrap());
    assert!(rect_of(&w, "tl.mark.a.off").is_none(), "a mark off the ruler is not drawn");
    assert!(rect_of(&w, "tl.seg.b.hidden").is_none(), "the masked track still hides its segments");
    let (b3, b1, bar) = (rect_of(&w, "tl.block.3").unwrap(), rect_of(&w, "tl.block.1").unwrap(), rect_of(&w, "tl.seg.a.s").unwrap());
    for (m, cell) in [(m1, b3), (m2, b3), (mb, b1)] {
        assert!(m.x >= cell.x && m.x + m.w <= cell.x + cell.w, "{m:?} lies inside its block {cell:?}");
    }
    assert!(m1.y >= bar.y && m1.y + m1.h <= bar.y + bar.h, "a mark sits on its track's row");
    assert!(m1.x + m1.w <= m2.x, "two marks of one block side by side: {m1:?} {m2:?}");
    let centre = (m1.x + m1.w / 2, m1.y + m1.h / 2);
    assert_eq!(hit(&w, centre), Some("seg a s".to_string()), "a click on a mark still reaches the segment under it");
    assert!(by_id(&w, "tl.mark.a.m1").is_err(), "a mark is not clickable");
    let card = w.iter().rev().find(|x| x.rect.contains(centre.0, centre.1) && !x.tip.is_empty()).map(|x| x.id.as_str());
    assert_eq!(card, Some("tl.mark.a.m1"), "hovering a mark shows its own card");
    let info = infos(&w);
    let m = info.iter().find(|x| x["id"] == "tl.mark.a.m1").unwrap();
    assert_eq!((m["kind"].as_str(), m["text"].as_str(), m["clickable"].as_bool()), (Some("mark"), Some("!"), Some(false)));
    let audit = text_audit::issues(&g.fonts.audit.take().unwrap(), 640.0, 360.0, &boxes(&w));
    assert_eq!(audit["counts"]["overlap"], 0, "labels and notes make room for the marks: {audit}");
    assert_eq!(audit["counts"]["spill"], 0, "{audit}");
}

#[test]
fn a_link_joins_the_end_of_one_segment_to_the_start_of_the_next() {
    let mut g = gfx();
    let pink = [250, 0, 250, 255];
    let link = |from: &str, to: &str| Link { from: from.into(), to: to.into(), colour: pink, tip: tip("Chained", "") };
    let a = Track { links: vec![link("x", "y"), link("x", "missing")], ..track("a", vec![seg("x", 1, 2), seg("y", 3, 4), seg("z", 6, 6)]) };
    let b = Track { masked: Some("Hidden".into()), links: vec![link("p", "q")], ..track("b", vec![seg("p", 1, 2), seg("q", 3, 4)]) };
    let w = draw_tl(&mut g, &one_side(6, vec![a, b]));
    let (l, x, y) = (rect_of(&w, "tl.link.a.x.y").unwrap(), rect_of(&w, "tl.seg.a.x").unwrap(), rect_of(&w, "tl.seg.a.y").unwrap());
    assert!(l.x >= x.x && l.x < x.x + x.w, "the link starts inside the first segment: {l:?} {x:?}");
    assert!(l.x + l.w > y.x && l.x + l.w <= y.x + y.w, "and ends inside the next: {l:?} {y:?}");
    assert!(l.y >= x.y && l.y + l.h <= x.y + x.h, "on the bars' row");
    assert!(rect_of(&w, "tl.link.a.x.missing").is_none(), "a link to a segment the track lacks is not drawn");
    assert!(rect_of(&w, "tl.link.b.p.q").is_none(), "a masked track hides its links");
    let (gap, line) = ((x.x + x.w + y.x) / 2, l.y + l.h - 1);
    assert_eq!(px(&g, gap, line), pink, "the connector crosses the gap between the bars");
    assert_eq!(hit(&w, (l.x + 1, line)), Some("seg a x".to_string()), "a click on a link still reaches the segment under it");
    assert_eq!(hit(&w, (gap, line)), None, "and the link itself takes none");
}

#[test]
fn the_kit_timeline_page_shows_parts_links_and_marks_and_takes_its_clicks() {
    let draw = |k: &Specimens, audit: bool| -> (Vec<Widget<KitAct>>, Value) {
        let mut g = gfx();
        g.fonts.audit = audit.then(Vec::new);
        let w = {
            let mut ui: Ui<KitAct> = Ui::new(&mut g, (-1, -1), true);
            k.draw(&mut ui, Rect::new(112, 22, 522, 332), |a| a);
            ui.widgets
        };
        let issues = text_audit::issues(&g.fonts.audit.take().unwrap_or_default(), 640.0, 360.0, &boxes(&w));
        (w, issues)
    };
    let count = |w: &[Widget<KitAct>], prefix: &str| w.iter().filter(|x| x.id.starts_with(prefix)).count();
    let mut k = Specimens::default();
    let (w, _) = draw(&k, false);
    assert_eq!(count(&w, "kit.timeline."), 0, "the widgets page comes first");
    assert!(w.iter().any(|x| x.id == "kit.page.0" && x.active) && by_id(&w, "kit.icon.lock").is_ok());
    k.act(by_id(&w, "kit.page.1").unwrap());
    assert_eq!((k.page, k.last.as_str()), (1, "page Timeline"));
    let (w, audit) = draw(&k, true);
    assert!(by_id(&w, "kit.icon.lock").is_err(), "the timeline page replaces the widgets");
    assert!(count(&w, "kit.timeline.mark.") >= 5 && count(&w, "kit.timeline.link.") >= 2, "marks and links on show");
    assert!(rect_of(&w, "kit.timeline.mark.lurker.impact").is_some() && count(&w, "kit.timeline.seg.lurker.") == 0, "a mark on a masked track");
    assert_eq!(audit["counts"], json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), "the page reads clean at Normal: {audit}");
    k.act(by_id(&w, "kit.timeline.seg.brute.smash").unwrap());
    assert_eq!(k.last, "segment smash on brute");
    k.act(by_id(&w, "kit.timeline.block.3").unwrap());
    k.act(by_id(&w, "kit.timeline.track.scout").unwrap());
    let (w, _) = draw(&k, false);
    assert!(w.iter().any(|x| x.id == "kit.timeline.block.3" && x.active), "a clicked block shows selected");
    assert_eq!((k.block, k.track.as_deref()), (Some(3), Some("scout")));
}
