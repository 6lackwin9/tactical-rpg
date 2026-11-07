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
