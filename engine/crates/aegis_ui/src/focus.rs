//! Keyboard focus over a screen's widgets (PORT-FOCUS; Godot's Control focus: ui_focus_next / ui_focus_prev /
//! ui_up..ui_right / ui_accept). Pure functions over the widget infos a screen reports (`Screen::widgets`), so every
//! screen gets focus without code of its own: Tab and Shift+Tab walk the focusable widgets in drawing order, the arrow
//! keys move to the nearest one in that direction, and the shell activates the focused one on Enter or Space.

use crate::canvas::Rect;
use serde_json::Value;

/// A focus move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Next,
    Prev,
    /// A direction on the canvas (dx, dy), one of the four.
    Dir(i32, i32),
}

fn rect(w: &Value) -> Rect {
    let r = |i: usize| w["rect"][i].as_i64().unwrap_or(0) as i32;
    Rect::new(r(0), r(1), r(2), r(3))
}

fn id(w: &Value) -> &str {
    w["id"].as_str().unwrap_or("")
}

/// Whether a widget takes focus: an enabled clickable one, a choice field or a slider with room; while a choice list
/// is open (`options`) only its options.
pub fn focusable(w: &Value, options: bool) -> bool {
    let kind = w["kind"].as_str().unwrap_or("");
    let r = rect(w);
    if w["enabled"] != Value::Bool(true) || r.w <= 0 || r.h <= 0 {
        return false;
    }
    if options {
        return kind == "option";
    }
    match kind {
        "slider_step" | "scroll" | "backdrop" => false,
        "select" | "slider" => true,
        _ => w["clickable"] == Value::Bool(true),
    }
}

/// Whether the shell draws its focus ring around `w`. A `text_button` (a menu row without a panel,
/// `panes::MenuStyle::Text`) shows its own focus instead: a box would be the panel it has none of.
pub fn shows_ring(w: &Value) -> bool {
    w["kind"] != "text_button"
}

/// Whether a click leaves the focus showing: it is on a row without a panel (the last active item stays lit, and Enter
/// presses it) and the click does not move it to a widget of another kind. `clicked` is the widget clicked, if any.
pub fn click_keeps_focus(infos: &[Value], current: Option<&str>, clicked: Option<&Value>) -> bool {
    let on_row = live(infos, current, false).is_some_and(|w| !shows_ring(w));
    on_row && clicked.is_none_or(|w| !focusable(w, false) || !shows_ring(w))
}

/// The `text_button` a mouse move to `mouse` makes the active one: the topmost widget there, when it is such a row and
/// can take the focus. A row without a panel shows one active item, so the mouse moves the focus onto it (a mouse that
/// only rests there does not take it back from the keyboard, and leaving keeps the last one lit).
pub fn hover_target(infos: &[Value], mouse: (i32, i32)) -> Option<String> {
    let top = infos.iter().rev().find(|w| rect(w).contains(mouse.0, mouse.1))?;
    let id = id(top);
    (top["kind"] == "text_button" && ring(infos, false).iter().any(|w| w["id"] == id)).then(|| id.to_string())
}

/// The focusable widgets in drawing order (a widget drawn twice keeps its first place); a widget whose centre a later
/// popup or window covers (its `backdrop`) is out of reach, as it is for the mouse.
pub fn ring(infos: &[Value], options: bool) -> Vec<&Value> {
    let mut out: Vec<&Value> = Vec::new();
    for (i, w) in infos.iter().enumerate().filter(|(_, w)| focusable(w, options)) {
        let r = rect(w);
        let covered = infos[i + 1..].iter().any(|o| o["kind"] == "backdrop" && rect(o).contains(r.x + r.w / 2, r.y + r.h / 2));
        if !covered && !out.iter().any(|o| id(o) == id(w)) {
            out.push(w);
        }
    }
    out
}

/// The focus `current` names, while that widget can still take it (there, enabled, not covered).
pub fn live<'a>(infos: &'a [Value], current: Option<&str>, options: bool) -> Option<&'a Value> {
    let c = current?;
    ring(infos, options).into_iter().find(|w| id(w) == c)
}

/// The widget focus moves to from `current` (None or a widget no longer there: the first one, or the last for Prev).
pub fn step(infos: &[Value], current: Option<&str>, m: Move, options: bool) -> Option<String> {
    let ring = ring(infos, options);
    if ring.is_empty() {
        return None;
    }
    let at = current.and_then(|c| ring.iter().position(|w| id(w) == c));
    let pick = match (m, at) {
        (Move::Prev, None) => ring.len() - 1,
        (_, None) => 0,
        (Move::Next, Some(i)) => (i + 1) % ring.len(),
        (Move::Prev, Some(i)) => (i + ring.len() - 1) % ring.len(),
        (Move::Dir(dx, dy), Some(i)) => return Some(toward(&ring, i, dx, dy).unwrap_or_else(|| id(ring[i]).to_string())),
    };
    Some(id(ring[pick]).to_string())
}

/// Where a widget stands for the arrow keys, doubled to stay whole: its centre. A `text_button`'s rect is as wide as its
/// label, so rows of one column have different centres: it stands at its left edge, which they share.
fn anchor(w: &Value) -> (i32, i32) {
    let r = rect(w);
    (if w["kind"] == "text_button" { r.x * 2 } else { r.x * 2 + r.w }, r.y * 2 + r.h)
}

/// The nearest widget whose `anchor` lies in direction (dx, dy) from widget `from` (Godot's closest neighbour): the
/// distance along the direction plus twice the sideways offset, so a widget in line wins over a nearer one aside.
fn toward(ring: &[&Value], from: usize, dx: i32, dy: i32) -> Option<String> {
    let (ax, ay) = anchor(ring[from]);
    let mut best: Option<(i64, usize)> = None;
    for (i, w) in ring.iter().enumerate() {
        if i == from {
            continue;
        }
        let (bx, by) = anchor(w);
        let (along, aside) = ((bx - ax) * dx + (by - ay) * dy, ((bx - ax) * dy).abs() + ((by - ay) * dx).abs());
        if along <= 0 {
            continue;
        }
        let score = along as i64 + 2 * aside as i64;
        if best.is_none_or(|(s, _)| score < s) {
            best = Some((score, i));
        }
    }
    best.map(|(_, i)| id(ring[i]).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn w(id: &str, kind: &str, x: i32, y: i32, enabled: bool) -> Value {
        json!({"id": id, "kind": kind, "rect": [x, y, 40, 14], "enabled": enabled, "clickable": kind == "button"})
    }

    /// Two rows of two buttons, a disabled one, a label and a select.
    fn grid() -> Vec<Value> {
        vec![w("a", "button", 10, 10, true), w("b", "button", 60, 10, true), w("off", "button", 110, 10, false), w("label", "text", 10, 30, true), w("c", "button", 10, 40, true), w("d", "select", 60, 40, true)]
    }

    #[test]
    fn tab_walks_the_focusable_widgets_in_order() {
        let g = grid();
        assert_eq!(step(&g, None, Move::Next, false).as_deref(), Some("a"), "the first one");
        assert_eq!(step(&g, None, Move::Prev, false).as_deref(), Some("d"), "the last one");
        assert_eq!(step(&g, Some("b"), Move::Next, false).as_deref(), Some("c"), "disabled and plain widgets are skipped");
        assert_eq!(step(&g, Some("d"), Move::Next, false).as_deref(), Some("a"), "it wraps");
        assert_eq!(step(&g, Some("a"), Move::Prev, false).as_deref(), Some("d"));
        assert_eq!(step(&g, Some("gone"), Move::Next, false).as_deref(), Some("a"), "a widget no longer there starts over");
    }

    #[test]
    fn arrows_move_to_the_nearest_widget_that_way() {
        let g = grid();
        assert_eq!(step(&g, Some("a"), Move::Dir(1, 0), false).as_deref(), Some("b"));
        assert_eq!(step(&g, Some("a"), Move::Dir(0, 1), false).as_deref(), Some("c"), "straight below beats the one aside");
        assert_eq!(step(&g, Some("b"), Move::Dir(0, 1), false).as_deref(), Some("d"));
        assert_eq!(step(&g, Some("d"), Move::Dir(-1, 0), false).as_deref(), Some("c"));
        assert_eq!(step(&g, Some("a"), Move::Dir(-1, 0), false).as_deref(), Some("a"), "nothing that way: focus stays");
    }

    #[test]
    fn an_open_list_takes_the_focus_alone() {
        let mut g = grid();
        g.push(json!({"id": "d.opt.0", "kind": "option", "rect": [60, 55, 40, 14], "enabled": true, "clickable": true}));
        g.push(json!({"id": "d.opt.1", "kind": "option", "rect": [60, 69, 40, 14], "enabled": true, "clickable": true}));
        assert_eq!(step(&g, Some("d.opt.0"), Move::Dir(0, 1), true).as_deref(), Some("d.opt.1"));
        assert_eq!(step(&g, Some("d.opt.1"), Move::Next, true).as_deref(), Some("d.opt.0"), "only the options");
        assert_eq!(step(&g, Some("a"), Move::Next, false).as_deref(), Some("b"));
    }

    #[test]
    fn a_popup_covers_what_lies_under_it() {
        let mut g = grid();
        g.push(json!({"id": "pop", "kind": "backdrop", "rect": [0, 0, 200, 30], "enabled": true, "clickable": false}));
        g.push(json!({"id": "pop.row", "kind": "option", "rect": [20, 12, 40, 14], "enabled": true, "clickable": true}));
        let ids: Vec<&str> = ring(&g, false).iter().map(|w| id(w)).collect();
        assert_eq!(ids, ["c", "d", "pop.row"], "a popup's rows take focus; the buttons it covers do not");
        assert!(live(&g, Some("a"), false).is_none(), "a covered focus is not live");
    }
}
