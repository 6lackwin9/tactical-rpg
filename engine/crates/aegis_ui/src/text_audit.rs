//! The text audit (TEXTFIT1): what one frame's text does wrong at the current text size. The painter records every
//! text it draws while `Fonts::audit` is on (`painter::TextRec`); this reads the records: text `text_fit` had to
//! shorten (cut), text past the canvas (edge), and two texts drawn over each other (overlap). Clipped text (scroll
//! areas) and text a later panel covers (a dialog over the screen) are never a problem. The shell's agent command `text_audit` runs one frame with the audit on.

use crate::painter::TextRec;
use serde_json::{json, Value};

/// How much two text boxes must share to count as overlapping: more than this many logical px across, and more than
/// this share of the shorter line's height down.
pub const OVERLAP_X: f32 = 1.0;
pub const OVERLAP_Y: f32 = 0.35;
/// Problems listed per kind (the counts are always complete).
pub const LIST_MAX: usize = 60;

fn at(r: &TextRec) -> Value {
    json!([(r.x * 10.0).round() / 10.0, (r.y * 10.0).round() / 10.0, (r.w * 10.0).round() / 10.0])
}

/// The problems of `recs` on a `w` x `h` canvas, against the screen's controls `boxes` ((id, x, y, w, h)): {texts,
/// cut: [{text, shown, at}], edge: [{text, at}], overlap: [{a, b, at_a, at_b}], spill: [{text, widget, at}], counts}.
/// A spill is text that starts inside a control (the smallest one holding its first letter) and runs past its edge.
pub fn issues(recs: &[TextRec], w: f32, h: f32, boxes: &[(String, f32, f32, f32, f32)]) -> Value {
    let live: Vec<&TextRec> = recs.iter().filter(|r| !r.clipped && !r.covered).collect();
    let mut spill = Vec::new();
    let off_canvas = |r: &TextRec| r.x < -0.5 || r.y < -0.5 || r.x + r.w > w + 0.5 || r.y + r.h > h + 0.5;
    for r in live.iter().filter(|r| !off_canvas(r)) {
        let (px, py) = (r.x + 1.0, r.y + r.h / 2.0);
        let holder = boxes.iter().filter(|b| px >= b.1 && px <= b.1 + b.3 && py >= b.2 && py <= b.2 + b.4).min_by(|a, b| (a.3 * a.4).total_cmp(&(b.3 * b.4)));
        if let Some((id, bx, by, bw, bh)) = holder {
            if r.x + r.w > bx + bw + 1.0 || r.y + r.h > by + bh + 2.0 {
                spill.push(json!({"text": r.text, "widget": id, "at": at(r), "box": [bx, by, bw, bh]}));
            }
        }
    }
    let cut: Vec<Value> = live.iter().filter_map(|r| r.cut.as_ref().map(|full| json!({"text": full, "shown": r.text, "at": at(r)}))).collect();
    let edge: Vec<Value> = live
        .iter()
        .filter(|r| off_canvas(r))
        .map(|r| json!({"text": r.text, "at": at(r)}))
        .collect();
    let mut overlap = Vec::new();
    for (i, a) in live.iter().enumerate() {
        for b in &live[i + 1..] {
            // the same text drawn twice in place (an outline or a shadow) is not a collision
            if a.text == b.text && (a.x - b.x).abs() < 1.5 && (a.y - b.y).abs() < 1.5 {
                continue;
            }
            let ix = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
            let iy = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
            if ix > OVERLAP_X && iy > OVERLAP_Y * a.h.min(b.h) {
                overlap.push(json!({"a": a.text, "b": b.text, "at_a": at(a), "at_b": at(b)}));
            }
        }
    }
    let counts = json!({"cut": cut.len(), "edge": edge.len(), "overlap": overlap.len(), "spill": spill.len()});
    let take = |v: Vec<Value>| v.into_iter().take(LIST_MAX).collect::<Vec<_>>();
    json!({"texts": recs.len(), "counts": counts, "cut": take(cut), "edge": take(edge), "overlap": take(overlap), "spill": take(spill)})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(x: f32, y: f32, w: f32, text: &str) -> TextRec {
        TextRec { x, y, w, h: 9.0, text: text.into(), clipped: false, cut: None, covered: false }
    }

    #[test]
    fn cut_edge_and_overlap_are_found_and_clipped_text_is_ignored() {
        let mut long = rec(10.0, 10.0, 50.0, "A long na...");
        long.cut = Some("A long name that did not fit".into());
        let off = rec(600.0, 30.0, 60.0, "Runs past the right edge");
        let (a, b) = (rec(10.0, 50.0, 40.0, "Left"), rec(45.0, 52.0, 40.0, "Right"));
        let shadow = (rec(10.0, 80.0, 30.0, "Title"), rec(10.5, 80.5, 30.0, "Title"));
        let mut scrolled = rec(630.0, 340.0, 60.0, "Inside a scroll area");
        scrolled.clipped = true;
        let fine = rec(200.0, 200.0, 30.0, "Fine");
        let mut under = rec(205.0, 200.0, 30.0, "Under a dialog");
        under.covered = true;
        let label = rec(300.0, 102.0, 70.0, "A button label that runs long");
        let boxes = vec![("button".to_string(), 298.0, 100.0, 50.0, 14.0), ("panel".to_string(), 0.0, 0.0, 640.0, 360.0)];
        let v = issues(&[long, off, a, b, shadow.0, shadow.1, scrolled, fine, under, label], 640.0, 360.0, &boxes);
        assert_eq!(v["counts"], json!({"cut": 1, "edge": 1, "overlap": 1, "spill": 1}), "{v}");
        assert_eq!((v["spill"][0]["text"].clone(), v["spill"][0]["widget"].clone()), (json!("A button label that runs long"), json!("button")));
        assert_eq!(v["cut"][0]["text"], json!("A long name that did not fit"));
        assert_eq!(v["edge"][0]["text"], json!("Runs past the right edge"));
        assert_eq!((v["overlap"][0]["a"].clone(), v["overlap"][0]["b"].clone()), (json!("Left"), json!("Right")));
        assert_eq!(v["texts"], json!(10));
    }

    #[test]
    fn lines_one_above_the_other_do_not_overlap() {
        let v = issues(&[rec(10.0, 10.0, 80.0, "Line one"), rec(10.0, 18.5, 80.0, "Line two")], 640.0, 360.0, &[]);
        assert_eq!(v["counts"]["overlap"], json!(0), "a line spacing under the line height is not a collision: {v}");
    }
}
