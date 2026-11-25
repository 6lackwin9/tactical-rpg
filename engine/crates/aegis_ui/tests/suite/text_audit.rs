//! TEXTFIT2 transient text: texts drawn inside `Painter::transient` (floats that rise and fade over a board) are
//! recorded by the text audit but never reported; a normal text in the same place still is, and the flag ends with
//! the call.

use aegis_ui::canvas::Canvas;
use aegis_ui::painter::{Fonts, Painter, View};
use aegis_ui::painter::TextRec;
use aegis_ui::text_audit::{issues, issues_on};
use std::path::Path;

const W: [u8; 4] = [255, 255, 255, 255];

fn fonts() -> Fonts {
    let font = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts/OpenSans-SemiBold.ttf")).unwrap();
    let mut fonts = Fonts::new(&font);
    fonts.audit = Some(Vec::new());
    fonts
}

#[test]
fn a_transient_text_is_skipped_while_a_normal_one_is_still_reported() {
    let mut canvas = Canvas::new(640, 360);
    let mut fonts = fonts();
    let mut p = Painter { c: &mut canvas, v: View::fit(640, 360), fonts: &mut fonts };
    // two labels drawn over each other (an overlap) and one running past the right edge (an edge)
    p.text(10.0, 10.0, "Normal label", 8.0, W);
    p.text(14.0, 11.0, "Over it", 8.0, W);
    p.text(620.0, 40.0, "Past the edge", 8.0, W);
    // the same three drawn transient (a float over the label, a float past the edge)
    p.transient(|p| {
        p.text(10.0, 100.0, "Normal label", 8.0, W);
        p.text(14.0, 101.0, "-12", 8.0, W);
        p.text(620.0, 140.0, "+7", 8.0, W);
    });
    assert!(!p.fonts.transient, "the flag ends with the call");
    // a transient float over a normal label: the label alone is not an overlap
    p.text(10.0, 200.0, "Hero name", 8.0, W);
    p.transient(|p| p.text(14.0, 201.0, "-30", 8.0, W));
    let recs = p.fonts.audit.take().unwrap();
    let marked: Vec<(&str, bool)> = recs.iter().map(|r| (r.text.as_str(), r.transient)).collect();
    assert_eq!(
        marked,
        [("Normal label", false), ("Over it", false), ("Past the edge", false), ("Normal label", true), ("-12", true), ("+7", true), ("Hero name", false), ("-30", true)]
    );
    let v = issues(&recs, 640.0, 360.0, &[]);
    assert_eq!(v["counts"], serde_json::json!({"cut": 0, "edge": 1, "overlap": 1, "spill": 0}), "{v}");
    assert_eq!(v["edge"][0]["text"], "Past the edge");
    assert_eq!((v["overlap"][0]["a"].as_str(), v["overlap"][0]["b"].as_str()), (Some("Normal label"), Some("Over it")));
    assert_eq!(v["texts"], 8, "transient texts are still recorded");
}

fn rec(x: f32, y: f32, w: f32, text: &str) -> TextRec {
    TextRec { x, y, w, h: 18.0, text: text.into(), clipped: false, cut: None, covered: false, transient: false }
}

/// Every tolerance is a length in old units: a picture doubled on a grid of two units per old unit reports what it did
/// before. Read with the wrong unit, a shadow one old unit away is a collision and a text ending in a control's tolerance
/// spills.
#[test]
fn the_audit_reads_its_tolerances_in_the_units_of_the_grid() {
    let shadow = [rec(100.0, 100.0, 60.0, "Title"), rec(102.0, 102.0, 60.0, "Title")];
    let v = issues_on(&shadow, 1280.0, 720.0, &[], 2);
    assert_eq!(v["counts"]["overlap"], 0, "a shadow one old unit away: {v}");
    assert_eq!(issues(&shadow, 1280.0, 720.0, &[])["counts"]["overlap"], 1, "two units away on the old grid is two texts");

    let boxes = [("b".to_string(), 200.0, 100.0, 40.0, 18.0)];
    let inside = [rec(202.0, 100.0, 39.5, "Label")];
    assert_eq!(issues_on(&inside, 1280.0, 720.0, &boxes, 2)["counts"]["spill"], 0, "inside the tolerance of 1 old unit");
    assert_eq!(issues(&inside, 1280.0, 720.0, &boxes)["counts"]["spill"], 1, "past the tolerance of 1 unit on the old grid");

    let past = [rec(1270.0, 100.0, 10.8, "Far")];
    assert_eq!(issues_on(&past, 1280.0, 720.0, &[], 2)["counts"]["edge"], 0, "within half an old unit of the edge");
    assert_eq!(issues(&past, 1280.0, 720.0, &[])["counts"]["edge"], 1);

    let real = [rec(100.0, 100.0, 60.0, "Left"), rec(150.0, 100.0, 60.0, "Right")];
    assert_eq!(issues_on(&real, 1280.0, 720.0, &[], 2)["counts"]["overlap"], 1, "a real collision still counts");
}

/// A widget as `Screen::widgets` reports it: id, kind, rect and hover lines.
fn widget(id: &str, r: [i32; 4], tip: &[&str]) -> serde_json::Value {
    serde_json::json!({"id": id, "kind": "label", "rect": r, "enabled": true, "clickable": false, "tip": tip})
}

/// XL-FIX2 (PR7-85, FB-455 d): a screen names the boxes the text audit does not take as controls (a board's cells
/// under a log panel): the log line that spills out of a cell without them reads clean with them, while a spill out of
/// a control not named stays, and the layout audit answers the same either way.
#[test]
fn boxes_a_screen_names_are_not_controls_to_the_text_audit() {
    use aegis_ui::layout_audit::frame_report;
    use aegis_ui::painter::Grid;
    let view = View::fit(640, 360);
    // a log line over two board cells, and a button label running past its button
    let texts = [rec(100.0, 100.0, 120.0, "[Move] a log line over the board"), rec(400.0, 200.0, 90.0, "A long button label")];
    let widgets = [widget("tile.0.0", [96, 96, 20, 20], &[]), widget("tile.1.0", [116, 96, 20, 20], &[]), widget("log", [90, 90, 200, 120], &[]), widget("button", [398, 198, 40, 22], &[])];
    let all = frame_report(view, Grid::Native, &texts, &[], &widgets, &[]);
    let named = frame_report(view, Grid::Native, &texts, &[], &widgets, &["tile."]);
    let spills = |v: &serde_json::Value| v["spill"].as_array().unwrap().iter().map(|s| s["widget"].as_str().unwrap_or("").to_string()).collect::<Vec<_>>();
    assert_eq!(spills(&all.report), ["tile.0.0", "button"], "{}", all.report);
    assert_eq!(spills(&named.report), ["button"], "the cell is not the log line's control: {}", named.report);
    assert_eq!(all.report["layout"], named.report["layout"], "the layout audit reads every widget as before");
}

/// XL-FIX2 (PR7-85, FB-455 c): each cut says whether a widget at it hovers to its full words, and `bare_cut` counts
/// the ones that do not; `counts` keep their four kinds.
#[test]
fn a_cut_says_whether_it_hovers_to_its_full_words() {
    use aegis_ui::layout_audit::frame_report;
    use aegis_ui::painter::Grid;
    let mut hovered = rec(20.0, 20.0, 60.0, "Tallow-Wax...");
    hovered.cut = Some("Tallow-Wax Stitch Pack".into());
    let mut bare = rec(20.0, 60.0, 60.0, "From the he...");
    bare.cut = Some("From the hero's deck".into());
    let mut wrong = rec(20.0, 100.0, 60.0, "Amber Le...");
    wrong.cut = Some("Amber Lens Shard".into());
    let widgets = [widget("row.0", [18, 18, 100, 20], &["Tallow-Wax Stitch Pack"]), widget("row.2", [18, 98, 100, 20], &["Amber Lens", "a shard of amber"])];
    let v = frame_report(View::fit(640, 360), Grid::Native, &[hovered, bare, wrong], &[], &widgets, &[]).report;
    let hovers: Vec<bool> = v["cut"].as_array().unwrap().iter().map(|c| c["hover"].as_bool().unwrap()).collect();
    assert_eq!(hovers, [true, false, false], "only a hover holding the full text counts: {v}");
    assert_eq!(v["bare_cut"], 2);
    assert_eq!(v["counts"], serde_json::json!({"cut": 3, "edge": 0, "overlap": 0, "spill": 0}));
}
