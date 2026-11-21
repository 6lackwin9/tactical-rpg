//! A floating panel over an area (MAP-HOOKS1; FLOATPANELS1 moved it here from a game screen so every floating panel
//! shares it): its place (a free corner, else the middle and unlocked), the drag while unlocked, minimize into a named
//! chip whose top right is where the minimize button's top right was (so minimize and restore toggle in place), the
//! place and lock a game keeps in its settings (`saved`, `restore_saved`, `keep`), and the shared chrome (`draw`: the
//! frame, the lock and minimize buttons, or the chip). Nothing here names what a panel holds.

use crate::canvas::{rgb, rgba, Rect, Rgba};
use crate::theme::th;
use crate::ui::{tip, Ui, SMALL};
use serde_json::{json, Value};

/// The rim of an unlocked panel (amber: it can be dragged).
pub const UNLOCKED: Rgba = rgb(217, 119, 6);

/// Whether two rects share any pixel.
fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// The minimize button's side and a panel's margin from its area's edge.
pub const PANEL_BTN: i32 = 10;
pub const PANEL_MARGIN: i32 = 4;

/// A floating panel over an area (MAP-HOOKS1, FLOATPANELS1). It starts in a free corner
/// (top right, top left, bottom right, bottom left), or in the middle and unlocked when every corner is taken;
/// unlocked, it drags anywhere inside the area; minimized, it collapses into a named button whose top right is where the
/// minimize button's top right was, so minimize and restore toggle in place; that button drags too while unlocked, the
/// panel's place moving with it. Nothing here names what the panel holds.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FloatPanel {
    /// The open panel's top left; None until placed.
    pub pos: Option<(i32, i32)>,
    pub locked: bool,
    pub minimized: bool,
    /// The pointer's offset from the dragged rect's top left while a drag runs.
    grab: Option<(i32, i32)>,
    /// The pointer's last place over the panel or its button before a drag (the press a drag starts from).
    over: Option<(i32, i32)>,
}

impl FloatPanel {
    /// Places a panel of `size` in `area` once (later calls keep its place): the first corner whose rect overlaps none
    /// of `taken`, locked; else the middle, unlocked.
    pub fn place(&mut self, area: Rect, size: (i32, i32), taken: &[Rect]) {
        if self.pos.is_some() {
            return;
        }
        let (w, h) = size;
        let (l, r) = (area.x + PANEL_MARGIN, area.x + area.w - w - PANEL_MARGIN);
        let (t, b) = (area.y + PANEL_MARGIN, area.y + area.h - h - PANEL_MARGIN);
        match [(r, t), (l, t), (r, b), (l, b)].into_iter().find(|&(x, y)| !taken.iter().any(|o| overlaps(Rect::new(x, y, w, h), *o))) {
            Some(at) => {
                self.pos = Some(at);
                self.locked = true;
            }
            None => {
                self.pos = Some((area.x + (area.w - w) / 2, area.y + (area.h - h) / 2));
                self.locked = false;
            }
        }
    }

    /// The open panel's rect at `size` (at the area's origin before it is placed).
    pub fn rect(&self, size: (i32, i32)) -> Rect {
        let (x, y) = self.pos.unwrap_or((0, 0));
        Rect::new(x, y, size.0, size.1)
    }

    /// The minimize button of the open panel `panel`: its top-right corner.
    pub fn min_button(panel: Rect) -> Rect {
        Rect::new(panel.x + panel.w - PANEL_BTN - 3, panel.y + 3, PANEL_BTN, PANEL_BTN - 1)
    }

    /// The unlock button, left of the minimize button.
    pub fn lock_button(panel: Rect) -> Rect {
        let m = Self::min_button(panel);
        Rect::new(m.x - PANEL_BTN - 2, m.y, PANEL_BTN, m.h)
    }

    /// The minimized panel's `chip` w x h button: its top right where the minimize button's top right is.
    pub fn chip_rect(&self, size: (i32, i32), chip: (i32, i32)) -> Rect {
        let m = Self::min_button(self.rect(size));
        Rect::new(m.x + m.w - chip.0, m.y - 1, chip.0, chip.1)
    }

    /// The rect shown now: the chip when minimized, else the panel.
    pub fn shown(&self, size: (i32, i32), chip: (i32, i32)) -> Rect {
        if self.minimized { self.chip_rect(size, chip) } else { self.rect(size) }
    }

    pub fn toggle_lock(&mut self) {
        self.locked = !self.locked;
    }

    pub fn minimize(&mut self) {
        self.minimized = true;
    }

    /// Opens the panel where it was (its minimize button where the chip's top right was), nudged inside `area`.
    pub fn restore(&mut self, area: Rect, size: (i32, i32)) {
        self.minimized = false;
        self.clamp(area, size);
    }

    /// Keeps the open panel inside `area`.
    pub fn clamp(&mut self, area: Rect, size: (i32, i32)) {
        if let Some((x, y)) = self.pos {
            self.pos = Some((x.clamp(area.x + 2, (area.x + area.w - size.0 - 2).max(area.x + 2)), y.clamp(area.y + 2, (area.y + area.h - size.1 - 2).max(area.y + 2))));
        }
    }

    /// One frame of the drag: `held` is whether the pointer drags the panel (or its chip) now, `pointer` where it is and
    /// `now` the rect it drags (`shown`). Locked, nothing moves. A drag keeps the pointer where it took hold (the last
    /// place it was over `now` before the drag) and `now` inside `area`; the panel's place moves with it.
    pub fn follow(&mut self, pointer: (i32, i32), held: bool, now: Rect, area: Rect) {
        if !held || self.locked {
            self.grab = None;
            if now.contains(pointer.0, pointer.1) {
                self.over = Some(pointer);
            }
            return;
        }
        let grab = *self.grab.get_or_insert_with(|| {
            let at = self.over.filter(|o| now.contains(o.0, o.1)).unwrap_or(pointer);
            ((at.0 - now.x).clamp(0, now.w.max(1) - 1), (at.1 - now.y).clamp(0, now.h.max(1) - 1))
        });
        let x = (pointer.0 - grab.0).clamp(area.x + 2, (area.x + area.w - now.w - 2).max(area.x + 2));
        let y = (pointer.1 - grab.1).clamp(area.y + 2, (area.y + area.h - now.h - 2).max(area.y + 2));
        let (px, py) = self.pos.unwrap_or((now.x, now.y));
        self.pos = Some((px + x - now.x, py + y - now.y));
    }
}


impl FloatPanel {
    /// MAP-POLISH1: the place and lock a game keeps in its settings: `{"pos": [x, y], "locked": bool}`.
    pub fn saved(&self) -> Value {
        json!({"pos": self.pos.map(|p| vec![p.0, p.1]), "locked": self.locked})
    }

    /// MAP-POLISH1: takes a kept place and lock (`saved`) before the panel is first placed: a place whose panel of
    /// `size` lies wholly inside `area` is taken as it is with its lock, one partly inside is moved in (`clamp`), and one
    /// off the area (a smaller window or layout since) is dropped with its lock, so `place` starts the panel afresh (a free
    /// corner, locked, or the middle, unlocked). False when nothing was taken (no or a bad value, a place off the area, or
    /// the panel already placed).
    pub fn restore_saved(&mut self, v: &Value, area: Rect, size: (i32, i32)) -> bool {
        if self.pos.is_some() {
            return false;
        }
        let Some(locked) = v.get("locked").and_then(Value::as_bool) else { return false };
        let pos = v.get("pos").and_then(Value::as_array).and_then(|p| match p.as_slice() {
            [x, y] => Some((x.as_i64()? as i32, y.as_i64()? as i32)),
            _ => None,
        });
        let Some((x, y)) = pos else { return false };
        if !overlaps(Rect::new(x, y, size.0, size.1), area) {
            return false;
        }
        self.pos = Some((x, y));
        self.locked = locked;
        self.clamp(area, size);
        true
    }
}


impl FloatPanel {
    /// One frame's bookkeeping of a kept panel, before it is drawn: the first frame takes the kept place and lock
    /// (`stored`, the game's settings value), places the panel (`place`) and takes what it placed as the baseline
    /// (`kept`); later frames follow the drag (`held`: the pointer drags it now) and keep it inside `area`. Returns the
    /// value to store when the player changed the place or the lock (a drag ended, the lock toggled), else None.
    #[allow(clippy::too_many_arguments)]
    pub fn keep(&mut self, kept: &mut Option<Value>, stored: &Value, area: Rect, size: (i32, i32), chip: (i32, i32), taken: &[Rect], pointer: (i32, i32), held: bool) -> Option<Value> {
        let mut out = None;
        if kept.is_none() {
            self.restore_saved(stored, area, size);
        } else {
            let now = self.saved();
            if !held && kept.as_ref() != Some(&now) {
                *kept = Some(now.clone());
                out = Some(now);
            }
        }
        self.place(area, size, taken);
        if kept.is_none() {
            *kept = Some(self.saved());
        }
        let shown = self.shown(size, chip);
        self.follow(pointer, held, shown, area);
        if !self.minimized && !held {
            self.clamp(area, size);
        }
        out
    }
}

/// The actions a panel's chrome sends.
#[derive(Debug, Clone)]
pub struct Acts<A> {
    /// A click on the open panel's body (usually a no-op that keeps the click off what lies under it).
    pub body: A,
    pub lock: A,
    pub min: A,
    pub open: A,
}

/// The chip's size for a panel named `name`.
pub fn chip_size<A>(ui: &mut Ui<A>, name: &str) -> (i32, i32) {
    ((ui.p.text_w(name, SMALL + 1.0) + 18.0).ceil() as i32, (ui.p.line_h(SMALL) + 1.0).ceil() as i32 + 4)
}

/// A small padlock in `r`, open (the shackle lifted) or shut.
pub fn padlock<A>(ui: &mut Ui<A>, r: Rect, open: bool, col: Rgba) {
    let (cx, cy) = (r.x as f32 + r.w as f32 / 2.0, r.y as f32 + r.h as f32 / 2.0);
    ui.p.fill(Rect::new(cx as i32 - 3, cy as i32, 6, 4), col);
    let (l, rr, top) = (cx - 2.0, cx + 1.5, cy - 3.0);
    let lift = if open { 1.5 } else { 0.0 };
    ui.p.line(l, cy - lift, l, top - lift, col, 0.0);
    ui.p.line(l, top - lift, rr, top - lift, col, 0.0);
    ui.p.line(rr, top - lift, rr, if open { top + 0.5 } else { cy }, col, 0.0);
}

/// The shared chrome of panel `fp` named `name` at `size` (chip `chip`), with widget ids `id` (the body),
/// `<id>.lock`, `<id>.min` and `<id>.open` (the chip), dragged under `drag` while unlocked, on `fill`. Minimized: the
/// chip (three short bars, the name) and None. Open: the panel's frame (amber while unlocked), its body widget, the
/// lock and minimize buttons at its top right, and Some(the panel's rect) for the game to draw the content in (left of
/// `lock_button` on the first row). `frame` draws the game's own frame over the open panel's fill, under its buttons.
#[allow(clippy::too_many_arguments)]
pub fn draw<A: Clone>(ui: &mut Ui<A>, fp: &FloatPanel, id: &str, name: &str, drag: &str, size: (i32, i32), chip: (i32, i32), acts: &Acts<A>, fill: Rgba, frame: impl FnOnce(&mut Ui<A>, Rect)) -> Option<Rect> {
    let unlocked = !fp.locked;
    if fp.minimized {
        let c = fp.chip_rect(size, chip);
        let t = if unlocked { tip(name, "Drag to move it; click to open it here.") } else { tip(name, "Click to open it here.") };
        ui.push(format!("{id}.open"), "button", name, c, true, false, Some(acts.open.clone()), t);
        if unlocked {
            ui.draggable(drag);
        }
        ui.p.panel(c, fill, if unlocked { UNLOCKED } else { th().edge }, 0.0);
        for k in 0..3 {
            ui.p.fill(Rect::new(c.x + 4, c.y + 3 + k * 3, 6, 1), th().text);
        }
        ui.p.text(c.x as f32 + 13.0, c.y as f32 + 2.0, name, SMALL + 1.0, th().gold);
        return None;
    }
    let p = fp.rect(size);
    let body_tip = if unlocked { tip(name, "Drag it anywhere; lock it to keep it there.") } else { tip(name, "Unlock it to drag it anywhere.") };
    ui.push(id, "backdrop", name, p, true, false, Some(acts.body.clone()), body_tip);
    if unlocked {
        ui.draggable(drag);
        ui.p.panel(Rect::new(p.x - 1, p.y - 1, p.w + 2, p.h + 2), rgba(0, 0, 0, 0), rgba(217, 119, 6, 140), 0.0);
    }
    ui.p.panel(p, fill, if unlocked { UNLOCKED } else { th().edge }, 0.0);
    frame(ui, p);
    if unlocked {
        ui.p.panel(p, rgba(0, 0, 0, 0), UNLOCKED, 0.0);
    }
    let lb = FloatPanel::lock_button(p);
    if unlocked {
        ui.p.ellipse(lb.x as f32 + lb.w as f32 / 2.0, lb.y as f32 + lb.h as f32 / 2.0, 8.0, 7.0, rgba(245, 158, 11, 80));
    }
    ui.p.panel(lb, rgba(20, 23, 31, 255), if unlocked { UNLOCKED } else { th().edge }, 0.0);
    padlock(ui, lb, unlocked, if unlocked { th().gold } else { th().dim });
    let lock_tip = if unlocked { tip("Lock", "Lock it in place.") } else { tip("Unlock", "Unlock it to drag it anywhere.") };
    ui.push(format!("{id}.lock"), "button", if unlocked { "Lock" } else { "Unlock" }, lb, true, unlocked, Some(acts.lock.clone()), lock_tip);
    let mb = FloatPanel::min_button(p);
    ui.p.panel(mb, rgba(20, 23, 31, 255), th().edge, 0.0);
    ui.p.fill(Rect::new(mb.x + 3, mb.y + mb.h / 2, mb.w - 6, 1), th().text);
    ui.push(format!("{id}.min"), "button", "Minimize", mb, true, false, Some(acts.min.clone()), tip("Minimize", &format!("Shrink it to a {name} button here.")));
    Some(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keep_takes_the_stored_place_then_reports_a_change_once() {
        let area = Rect::new(0, 0, 200, 100);
        let mut fp = FloatPanel::default();
        let mut kept = None;
        let stored = json!({"pos": [10, 20], "locked": false});
        assert_eq!(fp.keep(&mut kept, &stored, area, (40, 30), (30, 10), &[], (0, 0), false), None);
        assert_eq!(fp.pos, Some((10, 20)));
        assert!(!fp.locked);
        assert_eq!(kept, Some(fp.saved()));
        fp.toggle_lock();
        let changed = fp.keep(&mut kept, &stored, area, (40, 30), (30, 10), &[], (0, 0), false);
        assert_eq!(changed, Some(json!({"pos": [10, 20], "locked": true})));
        assert_eq!(fp.keep(&mut kept, &stored, area, (40, 30), (30, 10), &[], (0, 0), false), None, "reported once");
    }

    #[test]
    fn keep_places_a_fresh_panel_in_a_free_corner() {
        let area = Rect::new(0, 0, 200, 100);
        let mut fp = FloatPanel::default();
        let mut kept = None;
        let taken = [Rect::new(150, 0, 50, 50)];
        fp.keep(&mut kept, &Value::Null, area, (40, 30), (30, 10), &taken, (0, 0), false);
        assert_eq!(fp.pos, Some((4, 4)), "the top right is taken: the top left");
        assert!(fp.locked);
    }
}
