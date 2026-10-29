//! TEXTFIT2 transient text: texts drawn inside `Painter::transient` (floats that rise and fade over a board) are
//! recorded by the text audit but never reported; a normal text in the same place still is, and the flag ends with
//! the call.

use aegis_ui::canvas::Canvas;
use aegis_ui::painter::{Fonts, Painter, View};
use aegis_ui::text_audit::issues;
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
