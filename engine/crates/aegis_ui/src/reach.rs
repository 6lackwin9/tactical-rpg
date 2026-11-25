//! ENGINE-48 (INPUT-REACH1): what a screen offers a mouse that the keys or a pad cannot reach. Pure functions over the
//! widget infos a screen reports (`Screen::widgets`), as the focus walk reads them (`crate::focus`): the shell's agent
//! link (`reach`, with no window) adds what needs a frame (whether the focus shows) and a back press, and the scenario
//! step `reach` keeps the report.
//!
//! Keys reach the focus ring (`focus::ring`: Tab and Shift+Tab walk all of it); the arrow keys reach what the arrows can
//! walk to from the opening focus (`arrow_reach`) when the screen lets them move the focus (`Screen::arrow_focus`). A pad
//! reaches what its buttons map to (`controls::Controls::pad`): the ring when a button gives Tab or Shift+Tab, else what
//! its d-pad walks to. A mouse reaches every enabled widget it can click, pick, drag or drop on (`mouse_use`).

use crate::canvas::Rect;
use crate::focus;
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn rect(w: &Value) -> Rect {
    let r = |i: usize| w["rect"][i].as_i64().unwrap_or(0) as i32;
    Rect::new(r(0), r(1), r(2), r(3))
}

fn id(w: &Value) -> &str {
    w["id"].as_str().unwrap_or("")
}

/// How a mouse uses `w`, when it can: `click` (a clickable widget, a choice field, a slider), `drag` (a drag source or
/// a pointer handle that clicks nothing), `drop` (a drop target that clicks nothing), `type` (a text field). None for
/// what a mouse only looks at or scrolls (labels, panels, a popup's backdrop, a scroll area, a slider's steps).
pub fn mouse_use(w: &Value) -> Option<&'static str> {
    let kind = w["kind"].as_str().unwrap_or("");
    let r = rect(w);
    if w["enabled"] != json!(true) || r.w <= 0 || r.h <= 0 || id(w).is_empty() {
        return None;
    }
    match kind {
        "backdrop" | "scroll" | "slider_step" | "option" => None,
        "text" | "textarea" => Some("type"),
        "select" | "slider" => Some("click"),
        "handle" => Some("drag"),
        _ if w["clickable"] == json!(true) => Some("click"),
        _ if w["drag"].is_string() => Some("drag"),
        _ if w["drop"].is_string() => Some("drop"),
        _ => None,
    }
}

/// The widgets a mouse can use (`mouse_use`), first place kept, less those a later popup's backdrop covers at their
/// centre (out of the mouse's reach too): (id, kind, use).
pub fn mouse_set(infos: &[Value]) -> Vec<(String, String, &'static str)> {
    let mut out: Vec<(String, String, &'static str)> = Vec::new();
    for (i, w) in infos.iter().enumerate() {
        let Some(u) = mouse_use(w) else { continue };
        let r = rect(w);
        let covered = infos[i + 1..].iter().any(|o| o["kind"] == "backdrop" && rect(o).contains(r.x + r.w / 2, r.y + r.h / 2));
        if !covered && !out.iter().any(|o| o.0 == id(w)) {
            out.push((id(w).to_string(), w["kind"].as_str().unwrap_or("").to_string(), u));
        }
    }
    out
}

/// The ring's ids (what Tab walks).
pub fn tab_reach(infos: &[Value]) -> BTreeSet<String> {
    focus::ring(infos, false).iter().map(|w| id(w).to_string()).collect()
}

/// What the four arrows walk to from `start` (None: the ring's first), each move as the shell makes it (`focus::step`).
pub fn arrow_reach(infos: &[Value], start: Option<&str>) -> BTreeSet<String> {
    let ring = tab_reach(infos);
    let first = start.filter(|s| ring.contains(*s)).map(String::from).or_else(|| focus::step(infos, None, focus::Move::Next, false));
    let mut seen = BTreeSet::new();
    let mut todo: Vec<String> = first.into_iter().collect();
    while let Some(at) = todo.pop() {
        if !seen.insert(at.clone()) {
            continue;
        }
        for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            if let Some(n) = focus::step(infos, Some(&at), focus::Move::Dir(dx, dy), false) {
                if !seen.contains(&n) {
                    todo.push(n);
                }
            }
        }
    }
    seen
}

/// The inputs a device has for moving and using the focus: Tab (or Shift+Tab), the arrows, confirm and back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Device {
    pub tab: bool,
    pub arrows: bool,
    pub confirm: bool,
    pub back: bool,
}

impl Device {
    /// A keyboard: every key.
    pub fn keys() -> Self {
        Self { tab: true, arrows: true, confirm: true, back: true }
    }

    /// A pad: what its buttons map to (`keys`: each button's key).
    pub fn pad(keys: &[crate::screen::Key]) -> Self {
        use crate::screen::Key;
        let has = |k: Key| keys.contains(&k);
        Self { tab: has(Key::Tab) || has(Key::BackTab), arrows: [Key::Up, Key::Down, Key::Left, Key::Right].iter().all(|k| has(*k)), confirm: has(Key::Enter) || has(Key::Space), back: has(Key::Esc) }
    }
}

/// What `device` reaches on a screen whose widgets are `infos` (`arrow_focus`: its arrows move the focus; `start`: the
/// opening focus): the ring with Tab, else what the arrows walk to, else nothing.
pub fn device_reach(infos: &[Value], device: Device, arrow_focus: bool, start: Option<&str>) -> BTreeSet<String> {
    device_reach_grouped(infos, device, arrow_focus, start, &[])
}

/// `device_reach` on a screen with a focus group (`Screen::focus_group`, PR7-92): every widget the device's moves walk
/// to, as the shell makes them (`focus::step_grouped`): Tab and Shift+Tab when it has them, the arrows when it has them
/// and the screen lets them move the focus (inside the group they walk its members).
pub fn device_reach_grouped(infos: &[Value], device: Device, arrow_focus: bool, start: Option<&str>, group: &[&str]) -> BTreeSet<String> {
    let arrows = device.arrows && arrow_focus;
    if !device.tab && !arrows {
        return BTreeSet::new();
    }
    let mut moves: Vec<focus::Move> = Vec::new();
    if device.tab {
        moves.extend([focus::Move::Next, focus::Move::Prev]);
    }
    if arrows {
        moves.extend([(0, -1), (0, 1), (-1, 0), (1, 0)].map(|(dx, dy)| focus::Move::Dir(dx, dy)));
    }
    let ring: BTreeSet<String> = focus::ring(infos, false).iter().map(|w| id(w).to_string()).collect();
    let first = start.filter(|s| ring.contains(*s)).map(String::from).or_else(|| focus::step_grouped(infos, None, focus::Move::Next, false, group, None));
    let mut seen = BTreeSet::new();
    let mut todo: Vec<String> = first.into_iter().collect();
    while let Some(at) = todo.pop() {
        if !seen.insert(at.clone()) {
            continue;
        }
        for m in &moves {
            if let Some(n) = focus::step_grouped(infos, Some(&at), *m, false, group, None) {
                if !seen.contains(&n) {
                    todo.push(n);
                }
            }
        }
    }
    seen
}

/// The mouse widgets `reached` misses (a slider's steps count as reached with their slider): (id, kind, use).
pub fn unreached(infos: &[Value], reached: &BTreeSet<String>) -> Vec<(String, String, &'static str)> {
    mouse_set(infos).into_iter().filter(|(i, _, _)| !reached.contains(i)).collect()
}

/// The report's part that needs no frame: the mouse set, the ring, what keys, the d-pad alone and a pad reach, and the
/// dead ends of each (`{id, kind, use}`).
pub fn report(infos: &[Value], arrow_focus: bool, start: Option<&str>, pad: Device) -> Value {
    report_grouped(infos, arrow_focus, start, pad, &[])
}

/// `report` on a screen with a focus group (`group`, PR7-92): its `ring` holds the group once, `group` its prefixes.
pub fn report_grouped(infos: &[Value], arrow_focus: bool, start: Option<&str>, pad: Device, group: &[&str]) -> Value {
    let list = |v: Vec<(String, String, &'static str)>| v.into_iter().map(|(i, k, u)| json!({"id": i, "kind": k, "use": u})).collect::<Vec<_>>();
    let keys = device_reach_grouped(infos, Device::keys(), arrow_focus, start, group);
    let pad_reach = device_reach_grouped(infos, pad, arrow_focus, start, group);
    let dpad = device_reach_grouped(infos, Device { tab: false, arrows: true, confirm: true, back: true }, arrow_focus, start, group);
    json!({
        "mouse": mouse_set(infos).len(),
        "ring": focus::ring_grouped(infos, false, group, start, None).iter().map(|w| id(w).to_string()).collect::<Vec<_>>(),
        "group": group,
        "arrow_focus": arrow_focus,
        "keys_miss": list(unreached(infos, &keys)),
        "pad_miss": list(unreached(infos, &pad_reach)),
        "dpad_miss": list(unreached(infos, &dpad)),
        "pad": {"tab": pad.tab, "arrows": pad.arrows, "confirm": pad.confirm, "back": pad.back},
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(id: &str, kind: &str, x: i32, y: i32) -> Value {
        json!({"id": id, "kind": kind, "rect": [x, y, 40, 14], "enabled": true, "clickable": kind == "button"})
    }

    #[test]
    fn the_mouse_set_holds_what_a_mouse_can_use() {
        let mut g = vec![w("a", "button", 10, 10), w("label", "label", 10, 30), w("s", "select", 60, 10), w("h", "handle", 60, 40), w("t", "text", 110, 10)];
        g.push(json!({"id": "slot", "kind": "slot", "rect": [110, 40, 40, 14], "enabled": true, "clickable": false, "drop": "gear"}));
        g.push(json!({"id": "off", "kind": "button", "rect": [10, 60, 40, 14], "enabled": false, "clickable": true}));
        let got: Vec<(String, &str)> = mouse_set(&g).into_iter().map(|(i, _, u)| (i, u)).collect();
        assert_eq!(got, [("a".into(), "click"), ("s".into(), "click"), ("h".into(), "drag"), ("t".into(), "type"), ("slot".into(), "drop")]);
    }

    #[test]
    fn keys_reach_the_ring_and_a_pad_without_tab_only_its_arrows() {
        // a row of two, and one far below and to the right that the arrows still reach; a drop slot nothing reaches
        let mut g = vec![w("a", "button", 10, 10), w("b", "button", 60, 10), w("c", "button", 200, 200)];
        g.push(json!({"id": "slot", "kind": "slot", "rect": [110, 40, 40, 14], "enabled": true, "clickable": false, "drop": "gear"}));
        let keys = device_reach(&g, Device::keys(), true, None);
        assert_eq!(keys.iter().map(String::as_str).collect::<Vec<_>>(), ["a", "b", "c"]);
        assert_eq!(unreached(&g, &keys).iter().map(|u| u.0.as_str()).collect::<Vec<_>>(), ["slot"], "a drop target is mouse only");
        let dpad_only = Device { tab: false, arrows: true, confirm: true, back: true };
        assert_eq!(device_reach(&g, dpad_only, true, None).len(), 3, "the arrows walk down to c");
        assert!(device_reach(&g, dpad_only, false, None).is_empty(), "a screen that steers with the arrows: a pad with no Tab reaches nothing");
    }

    #[test]
    fn a_focus_group_is_one_tab_stop_and_the_arrows_walk_inside_it() {
        // a 3 x 2 board of tiles, then two buttons under it
        let mut g: Vec<Value> = (0..6).map(|i| w(&format!("tile.{i}"), "button", 10 + (i % 3) * 20, 10 + (i / 3) * 20, )).collect();
        g.push(w("end", "button", 10, 100));
        g.push(w("menu", "button", 60, 100));
        let group: &[&str] = &["tile."];
        let ring: Vec<String> = focus::ring_grouped(&g, false, group, None, None).iter().map(|x| id(x).to_string()).collect();
        assert_eq!(ring, ["tile.0", "end", "menu"], "the board is one stop");
        assert_eq!(focus::step_grouped(&g, Some("tile.0"), focus::Move::Next, false, group, None).as_deref(), Some("end"));
        assert_eq!(focus::step_grouped(&g, Some("end"), focus::Move::Prev, false, group, Some("tile.4")).as_deref(), Some("tile.4"), "Tab lands on the member last focused");
        assert_eq!(focus::step_grouped(&g, Some("tile.3"), focus::Move::Dir(0, 1), false, group, None).as_deref(), Some("tile.3"), "the arrows stay on the board at its edge");
        assert_eq!(focus::step_grouped(&g, Some("tile.0"), focus::Move::Dir(1, 0), false, group, None).as_deref(), Some("tile.1"));
        let keys = device_reach_grouped(&g, Device::keys(), true, None, group);
        assert_eq!(keys.len(), 8, "Tab and the arrows still reach every tile: {keys:?}");
        assert!(device_reach_grouped(&g, Device::keys(), false, None, group).len() == 3, "without arrows only the stop");
    }

    #[test]
    fn a_pad_device_reads_its_map() {
        use crate::screen::Key;
        let d = Device::pad(&[Key::Enter, Key::Esc, Key::Space, Key::Tab, Key::Up, Key::Down, Key::Left, Key::Right]);
        assert_eq!(d, Device::keys());
        let d = Device::pad(&[Key::Enter, Key::Up, Key::Down]);
        assert!(!d.tab && !d.arrows && d.confirm && !d.back);
    }
}
