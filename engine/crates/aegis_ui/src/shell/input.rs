//! SHELL-SPLIT1 (ENGINE-3): the shell's input, moved from `shell.rs` (behaviour unchanged; moved items are `pub(super)`).
//! Input: the pointer, the keyboard, the text field being edited, clicks, drags and drops, and Back.

use super::*;

/// A press on a draggable widget: its id, payload and text, and where the press was.
pub(super) struct Press {
    id: String,
    key: String,
    text: String,
    at: (i32, i32),
}

/// How far (old units) the mouse moves from the press before a draggable widget is picked up.
pub(super) const DRAG_START: i32 = 3;

/// A second click in a text field this soon (milliseconds) and this near (old units) is a double click.
pub(super) const DOUBLE_CLICK_MS: u128 = 400;

pub(super) const DOUBLE_CLICK_PX: i32 = 3;

/// The drop key under `pos`: the topmost enabled widget that is a drop target or a window's backdrop (a window covers
/// what lies under it), when it is a target.
pub(super) fn drop_key_at(infos: &[Value], pos: (i32, i32)) -> Option<&str> {
    let hit = infos.iter().rev().find(|w| w["enabled"] == json!(true) && (w["drop"].is_string() || w["kind"] == "backdrop") && info_rect(w).contains(pos.0, pos.1))?;
    hit["drop"].as_str()
}

/// The rect of a widget's agent info.
pub(super) fn info_rect(w: &Value) -> Rect {
    let r = |i: usize| w["rect"][i].as_i64().unwrap_or(0) as i32;
    Rect::new(r(0), r(1), r(2), r(3))
}

/// The topmost enabled widget info under `pos`; an open select's options lie over everything (like `ui::hit`).
pub(super) fn info_at(infos: &[Value], pos: (i32, i32)) -> Option<&Value> {
    let under = |w: &&Value| w["enabled"] == json!(true) && info_rect(w).contains(pos.0, pos.1);
    infos.iter().rev().filter(|w| w["kind"] == "option").find(under).or_else(|| infos.iter().rev().find(under))
}

/// A widget's agent info as a playtest trail control: {path (its id), cls (its kind), text (40 chars), tip (60),
/// interactive}.
pub(super) fn trail_ctl(w: &Value) -> Value {
    let cut = |v: &Value, n: usize| v.as_str().unwrap_or("").chars().take(n).collect::<String>();
    let tip = w["tip"].as_array().and_then(|t| t.first()).cloned().unwrap_or(Value::Null);
    json!({"path": w["id"], "cls": w["kind"], "text": cut(&w["text"], 40), "tip": cut(&tip, 60), "interactive": w["clickable"] == json!(true)})
}

/// A screen as a trail control (overlays open and close as modals).
pub(super) fn screen_ctl(id: &str) -> Value {
    json!({"path": id, "cls": "screen", "text": "", "tip": "", "interactive": false})
}

/// A press that counts as the player's gesture: a page may start audio only after one (`Audio::unlock`).
pub(super) fn is_gesture(ev: &WindowEvent) -> bool {
    match ev {
        WindowEvent::KeyboardInput { event, .. } => event.state == ElementState::Pressed && !event.repeat,
        WindowEvent::MouseInput { state, .. } => *state == ElementState::Pressed,
        WindowEvent::Touch(t) => t.phase == winit::event::TouchPhase::Started,
        _ => false,
    }
}

/// The key an arrow or character key is, before the control map (`Shell::walk_key` maps it and keeps the movement keys).
pub(super) fn raw_walk_key(k: &WKey) -> Option<Key> {
    let key = match k {
        WKey::Named(NamedKey::ArrowUp) => Key::Up,
        WKey::Named(NamedKey::ArrowDown) => Key::Down,
        WKey::Named(NamedKey::ArrowLeft) => Key::Left,
        WKey::Named(NamedKey::ArrowRight) => Key::Right,
        WKey::Character(c) => Key::Char(c.chars().next()?.to_ascii_lowercase()),
        _ => return None,
    };
    Some(key)
}

/// The digit of a top-row or keypad digit key.
pub(super) fn shift_digit(code: KeyCode) -> Option<u8> {
    use KeyCode::*;
    let d = match code {
        Digit0 | Numpad0 => 0,
        Digit1 | Numpad1 => 1,
        Digit2 | Numpad2 => 2,
        Digit3 | Numpad3 => 3,
        Digit4 | Numpad4 => 4,
        Digit5 | Numpad5 => 5,
        Digit6 | Numpad6 => 6,
        Digit7 | Numpad7 => 7,
        Digit8 | Numpad8 => 8,
        Digit9 | Numpad9 => 9,
        _ => return None,
    };
    Some(d)
}

impl<G: Context> Shell<G> {
    /// The OS moved the pointer to physical (`px`, `py`).
    pub(super) fn pointer_moved(&mut self, px: f32, py: f32) {
        self.mouse_px = Some((px, py));
        self.mouse = self.gfx.view.to_logical(px, py);
        self.mouse_moved();
        self.drag_to();
    }

    /// Every screen leaves, top first (the app closes).
    pub(super) fn leave_all(&mut self) {
        while let Some(mut gone) = self.stack.pop() {
            gone.leave(&mut self.cx);
        }
        self.cx.playtest.tick();
        self.cx.playtest.save();
    }

    /// Types into the text field being edited (`f` changes its text, caret or selection), hands a change to the screen
    /// and scrolls the field to keep the caret's line in view.
    pub(super) fn edit(&mut self, f: impl FnOnce(&mut Edit)) {
        let now = self.born.elapsed().as_millis() as u64;
        let Some(e) = self.gfx.kit.edit.as_mut() else { return };
        e.now_ms = now;
        let before = e.text.clone();
        f(e);
        if e.text != before {
            let (id, text) = (e.id.clone(), e.text.clone());
            let cx = &mut self.cx;
            self.stack.last_mut().unwrap().text_changed(cx, &id, &text);
        }
        self.reveal();
    }

    /// The text field being edited as the top screen last drew it, in its units: its layout, its rect and its line height.
    /// None when nothing edits or the field is not drawn (yet).
    pub(super) fn edit_geometry(&mut self) -> Option<(Layout, Rect, f32)> {
        let (id, text, multiline) = self.gfx.kit.edit.as_ref().map(|e| (e.id.clone(), e.text.clone(), e.multiline))?;
        let infos = self.stack.last()?.widgets();
        let r = infos.iter().find(|w| w["id"] == id.as_str() && (w["kind"] == "text" || w["kind"] == "textarea")).map(info_rect)?;
        let m = Metrics::of(&th(), self.gfx.view);
        let mut p = Painter { c: &mut self.gfx.canvas, v: self.gfx.view, fonts: &mut self.gfx.fonts };
        let line = p.line_h(m.body);
        Some((field_layout(&mut p, &text, r, multiline), r, line))
    }

    /// Scrolls the text area being edited so the caret's line shows.
    pub(super) fn reveal(&mut self) {
        let Some((lay, r, line)) = self.edit_geometry() else { return };
        let Some(e) = self.gfx.kit.edit.as_ref().filter(|e| e.multiline) else { return };
        let id = field_scroll_id(&e.id);
        let off = self.gfx.kit.scroll.get(&id).copied().unwrap_or(0);
        let g = self.gfx.view.g;
        let to = reveal_offset(off, lay.line_of(e.caret), line, r.h, field_content_h(lay.lines.len(), line, g), g);
        self.gfx.kit.scroll.insert(id, to);
    }

    /// The caret position under canvas point `pos` in the text field being edited (as drawn: scrolled or slid).
    pub(super) fn caret_at(&mut self, pos: (i32, i32)) -> Option<usize> {
        let (lay, r, line) = self.edit_geometry()?;
        let g = self.gfx.view.g;
        let e = self.gfx.kit.edit.as_ref()?;
        let (dx, off) = if e.multiline {
            let max = (field_content_h(lay.lines.len(), line, g) - r.h).max(0);
            (0.0, self.gfx.kit.scroll.get(&field_scroll_id(&e.id)).copied().unwrap_or(0).clamp(0, max))
        } else {
            (field_shift(&lay, e.caret, r, g), 0)
        };
        let (px, py) = (FIELD_PAD_X * g as f32, FIELD_PAD_Y * g as f32);
        Some(lay.pos_at(pos.0 as f32 - r.x as f32 - px + dx, pos.1 as f32 - r.y as f32 - py + off as f32, line))
    }

    /// An editing key in the text field being edited (PLAYUI1-T): moves, selection, deletes, undo and the clipboard.
    pub(super) fn edit_key(&mut self, k: EditKey, ctrl: bool, shift: bool) {
        let g = self.gfx.view.g as f32;
        let (lay, rows) = match self.edit_geometry() {
            Some((lay, r, line)) => (lay, ((r.h as f32 - 2.0 * FIELD_PAD_Y * g) / line).floor().max(1.0) as usize),
            // Not drawn yet: one line of the text as it is (no wrapping).
            None => (self.gfx.kit.edit.as_ref().map(|e| Layout::new(&e.text, f32::MAX, false, &mut |_| 1.0)).unwrap_or_default(), 1),
        };
        let mut copied = None;
        self.edit(|e| copied = e.key(k, ctrl, shift, &lay, rows, clipboard::get));
        if let Some(t) = copied {
            clipboard::set(&t);
        }
    }

    /// Playtest trail: a hover is logged when the pointer leaves a control it rested on for HOVER_MS or longer.
    pub(super) fn track_hover(&mut self) {
        if !self.cx.playtest.on {
            return;
        }
        let infos = self.stack.last().unwrap().widgets();
        let under = info_at(&infos, self.mouse).map(|w| (w["id"].as_str().unwrap_or("").to_string(), trail_ctl(w)));
        let same = matches!((&self.hover, &under), (Some((a, _, _)), Some((b, _))) if a == b);
        if same {
            return;
        }
        if let Some((_, since, ctl)) = self.hover.take() {
            let ms = since.elapsed().as_millis() as u64;
            if ms >= crate::playtest::HOVER_MS {
                self.cx.playtest.trail(json!({"type": "hover", "ctl": ctl, "ms": ms}));
            }
        }
        self.hover = under.map(|(id, ctl)| (id, aegis_core::clock::Instant::now(), ctl));
    }

    /// After the top screen drew: a focus whose widget is gone moves to the first one (when it was showing), an open
    /// list the keyboard opened focuses its picked option, a screen that wants it starts with a visible focus.
    pub(super) fn settle_focus(&mut self) {
        let top = self.stack.last().unwrap();
        let infos = top.widgets();
        self.text_rows = infos.iter().any(|w| !focus::shows_ring(w));
        let open = self.gfx.kit.open.is_some();
        if open && self.open_focus.is_some() && !self.gfx.kit.focus.as_deref().is_some_and(|f| infos.iter().any(|w| w["id"] == f && w["kind"] == "option")) {
            let picked = infos.iter().find(|w| w["kind"] == "option" && w["active"] == json!(true)).and_then(|w| w["id"].as_str()).map(String::from);
            self.gfx.kit.focus = picked.or_else(|| focus::step(&infos, None, focus::Move::Next, true));
            return;
        }
        let present = focus::live(&infos, self.gfx.kit.focus.as_deref(), open).is_some();
        if !present && (self.gfx.kit.focus_visible || (top.auto_focus() && self.gfx.kit.focus.is_none())) {
            let first = top.focus_first().filter(|f| focus::live(&infos, Some(f), open).is_some()).map(String::from);
            self.gfx.kit.focus = first.or_else(|| focus::step(&infos, None, focus::Move::Next, open));
            self.gfx.kit.focus_visible = self.gfx.kit.focus.is_some();
        }
        // No focus showing, and a menu row lit under a mouse that has not moved: it is the active item, so Enter presses it.
        if !self.gfx.kit.focus_visible {
            self.gfx.kit.mouse_moved(&infos, self.mouse);
        }
    }

    /// After a pushed screen first drew (ASKNO2): the centre of its `Screen::pointer_to` widget becomes the pointer
    /// request the windowed shell carries out; a screen an agent-link command opened asks for nothing.
    pub(super) fn settle_pointer(&mut self) {
        if !std::mem::take(&mut self.pointer_pending) || self.agent {
            return;
        }
        let top = self.stack.last().unwrap();
        self.pointer_req = top.pointer_to().and_then(|id| top.widget_rect(id)).map(|(r, _)| (r.x + r.w / 2, r.y + r.h / 2));
    }

    /// The wheel: the topmost scroll area under the mouse scrolls (a window above it stops the search), else the
    /// screen gets the lines (`Screen::wheel`).
    pub(super) fn wheel(&mut self, lines: i32) {
        // An open choice list closes rather than the area under it scrolling.
        if self.gfx.kit.open.take().is_some() {
            return;
        }
        let infos = self.stack.last().unwrap().widgets();
        let under = infos.iter().rev().find(|w| (w["kind"] == "scroll" || w["kind"] == "backdrop") && info_rect(w).contains(self.mouse.0, self.mouse.1));
        if let Some(area) = under.filter(|w| w["kind"] == "scroll") {
            let id = area["id"].as_str().unwrap_or("").to_string();
            let max = area["text"].as_str().and_then(|t| t.parse::<i32>().ok()).unwrap_or(0);
            // An offset set past the end ("scroll to the end") counts from the end.
            let at = self.gfx.kit.scroll.get(&id).copied().unwrap_or(0).clamp(0, max);
            self.gfx.kit.scroll.insert(id, (at - lines * self.gfx.view.u(crate::ui::SCROLL_STEP)).clamp(0, max));
            return;
        }
        let cx = &mut self.cx;
        self.stack.last_mut().unwrap().wheel(cx, lines);
    }

    /// The window gained or lost the focus (desktop: winit's `Focused`; web: the canvas's focus and blur, which winit
    /// reports the same way): the audio's background mute follows at once (`Audio::set_focused`, setting
    /// `MUTE_IN_BACKGROUND`), the music tracks take the new level now, not at the next drawn frame.
    pub(super) fn focus_changed(&mut self, focused: bool) {
        self.cx.audio.set_focused(focused, &self.cx.settings.v);
        self.cx.audio.tick();
    }

    /// Drops a press or drag in progress; true when there was one.
    pub(super) fn cancel_drag(&mut self) -> bool {
        self.text_press = false;
        let moved = self.win_drag.take().is_some();
        if moved {
            self.save_desk();
        }
        let mut had = self.press.take().is_some() | moved;
        if let Some(id) = self.handle.take() {
            let (cx, pos) = (&mut self.cx, self.mouse);
            self.stack.last_mut().unwrap().pointer(cx, &id, PointerPhase::Cancel, pos);
            had = true;
        }
        self.gfx.kit.drag.take().is_some() || had
    }

    /// The mouse moved: a menu row without a panel under it becomes the active item (`Kit::mouse_moved`).
    pub(super) fn mouse_moved(&mut self) {
        // Most screens have none: no widget list built on every move (`settle_focus` notes it each frame).
        if !self.text_rows {
            return;
        }
        let infos = self.stack.last().unwrap().widgets();
        self.gfx.kit.mouse_moved(&infos, self.mouse);
    }

    pub(super) fn key(&mut self, k: Key) {
        // Keys typed into a text field are not part of the trail (PLAYTEST1: skipped while a text field has focus).
        if self.gfx.kit.edit.is_none() {
            self.cx.playtest.trail(json!({"type": "key", "key": k.name(), "mods": ""}));
        }
        // Any key drops a press or drag in progress (what it changes could move the target); Esc does only that.
        let cancelled = self.cancel_drag();
        if k == Key::Esc && cancelled {
            return;
        }
        if k == Key::Esc {
            if let Some(field) = self.gfx.kit.open.take() {
                // Back on the field the list belongs to.
                self.open_focus = None;
                if self.gfx.kit.focus_visible {
                    self.gfx.kit.focus = Some(field);
                }
                return;
            }
        }
        if k != Key::Esc && self.focus_key(k) {
            self.settle();
            return;
        }
        let cx = &mut self.cx;
        let top = self.stack.last_mut().unwrap();
        if k == Key::Esc {
            top.escape(cx);
        } else {
            top.key(cx, k);
        }
        self.settle();
    }

    /// The keyboard focus (PORT-FOCUS): Tab / Shift+Tab move it, the arrows too unless the screen steers with them
    /// (`Screen::arrow_focus`; inside an open choice list they always do), Enter or Space on a visible focus presses
    /// the widget (a choice field opens its list, an option is picked), Left / Right step a focused slider. True when
    /// the key was used here (the screen does not get it).
    pub(super) fn focus_key(&mut self, k: Key) -> bool {
        let infos = self.stack.last().unwrap().widgets();
        let open = self.gfx.kit.open.is_some();
        let kind_of = |id: &str| infos.iter().find(|w| w["id"] == id).and_then(|w| w["kind"].as_str()).unwrap_or("").to_string();
        let current = focus::live(&infos, self.gfx.kit.focus.as_deref(), open).and_then(|w| w["id"].as_str()).map(String::from);
        let visible = self.gfx.kit.focus_visible && current.is_some();
        let slider = current.as_deref().filter(|f| visible && !open && kind_of(f) == "slider").map(String::from);
        if let (Some(id), Key::Left | Key::Right) = (slider, k) {
            // The step beside the active one.
            let prefix = format!("{id}.step.");
            let steps: Vec<&Value> = infos.iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with(&prefix))).collect();
            let at = steps.iter().position(|w| w["active"] == json!(true)).unwrap_or(0);
            let to = if k == Key::Left { at.saturating_sub(1) } else { (at + 1).min(steps.len().saturating_sub(1)) };
            if let Some(step) = steps.get(to).and_then(|w| w["id"].as_str()).map(String::from) {
                let cx = &mut self.cx;
                let _ = self.stack.last_mut().unwrap().click_id(cx, &step);
            }
            return true;
        }
        let arrows = open || self.stack.last().unwrap().arrow_focus();
        let m = match k {
            Key::Tab => Some(focus::Move::Next),
            Key::BackTab => Some(focus::Move::Prev),
            Key::Up if arrows => Some(focus::Move::Dir(0, -1)),
            Key::Down if arrows => Some(focus::Move::Dir(0, 1)),
            Key::Left if arrows => Some(focus::Move::Dir(-1, 0)),
            Key::Right if arrows => Some(focus::Move::Dir(1, 0)),
            _ => None,
        };
        if let Some(m) = m {
            // The first key only shows where the focus is.
            let from = if visible { current.as_deref() } else { None };
            let next = match (from, current.as_deref()) {
                (None, Some(c)) if !matches!(m, focus::Move::Next | focus::Move::Prev) => Some(c.to_string()),
                _ => focus::step(&infos, from, m, open),
            };
            if next.is_some() {
                self.gfx.kit.focus = next;
                self.gfx.kit.focus_visible = true;
                return true;
            }
            return false;
        }
        if !matches!(k, Key::Enter | Key::Space) || !visible {
            return false;
        }
        let id = current.unwrap();
        match kind_of(&id).as_str() {
            "select" => {
                self.gfx.kit.open = Some(id.clone());
                self.gfx.kit.focus = None;
                self.open_focus = Some(id);
            }
            "slider" => {}
            kind => {
                let picked = kind == "option";
                let cx = &mut self.cx;
                let _ = self.stack.last_mut().unwrap().click_id(cx, &id);
                if picked {
                    // Back on the field the list belongs to.
                    self.gfx.kit.focus = self.gfx.kit.open.take().or(self.open_focus.take());
                }
            }
        }
        true
    }

    /// A left click with its playtest trail events, logged before the click runs (as Godot's `pressed` comes before the
    /// panel it opens): `click` (the control; `handled` set once the click ran) and `button` when the click goes to a
    /// clickable widget (not a click that only closes an open list or starts a drag).
    pub(super) fn click(&mut self) {
        if !self.cx.playtest.on {
            self.click_inner();
            return;
        }
        let infos = self.stack.last().unwrap().widgets();
        let at = info_at(&infos, self.mouse).cloned();
        let ctl = at.as_ref().map(trail_ctl).unwrap_or(Value::Null);
        let list_open = self.gfx.kit.open.is_some();
        let to_widget = at.as_ref().is_some_and(|w| w["clickable"] == json!(true) && !w["drag"].is_string() && (!list_open || w["kind"] == "option"));
        let at_click = self.cx.playtest.trail(json!({"type": "click", "button": "left", "pos": [self.mouse.0, self.mouse.1], "ctl": ctl, "handled": at.is_some()}));
        if to_widget {
            self.cx.playtest.trail(json!({"type": "button", "ctl": ctl}));
        }
        if self.click_inner() && at.is_none() {
            if let Some(i) = at_click {
                self.cx.playtest.set_trail(i, "handled", json!(true));
            }
        }
    }

    /// A left click: opens or closes a choice list, picks from it, starts a slider drag, or clicks the widget; true
    /// when something took it.
    pub(super) fn click_inner(&mut self) -> bool {
        // A press or drag whose release was lost (focus moved away mid-drag) ends here.
        self.cancel_drag();
        let pos = self.mouse;
        let infos = self.stack.last().unwrap().widgets();
        let at = info_at(&infos, pos).cloned();
        let kind = at.as_ref().map(|w| w["kind"].as_str().unwrap_or("").to_string()).unwrap_or_default();
        // A text field starts editing on a click, the caret where the click was (Shift+click in the field being edited
        // extends the selection, a double click selects a word, a drag from the press selects); a click anywhere else
        // ends an edit.
        if (kind == "text" || kind == "textarea") && at.as_ref().is_some_and(|w| w["enabled"] == json!(true)) {
            let w = at.as_ref().unwrap();
            let id = w["id"].as_str().unwrap_or("").to_string();
            let same = self.gfx.kit.edit.as_ref().is_some_and(|e| e.id == id);
            if !same {
                self.gfx.kit.edit = Some(Edit::new(&id, w["text"].as_str().unwrap_or(""), kind == "textarea"));
            }
            let near = self.gfx.view.u(DOUBLE_CLICK_PX);
            let double = same && self.text_click.is_some_and(|(t, p)| t.elapsed().as_millis() < DOUBLE_CLICK_MS && (p.0 - pos.0).abs() <= near && (p.1 - pos.1).abs() <= near);
            self.text_click = if double { None } else { Some((Instant::now(), pos)) };
            let extend = self.shift && same;
            if let Some(c) = self.caret_at(pos) {
                self.edit(|e| {
                    e.goal_x = None;
                    if double {
                        e.select_word(c);
                    } else {
                        e.move_to(c, extend);
                    }
                });
            }
            self.text_press = !double;
            return true;
        }
        self.gfx.kit.edit = None;
        let keep = self.gfx.kit.focus_visible && focus::click_keeps_focus(&infos, self.gfx.kit.focus.as_deref(), at.as_ref());
        self.gfx.kit.focus_visible = keep;
        self.open_focus = None;
        if self.gfx.kit.open.take().is_some() {
            // An open list: a pick goes through, any other click only closes it.
            if kind == "option" {
                let cx = &mut self.cx;
                self.stack.last_mut().unwrap().click(cx, pos);
            }
            self.settle();
            return true;
        }
        let id = at.as_ref().map(|w| w["id"].as_str().unwrap_or("").to_string()).unwrap_or_default();
        if at.as_ref().is_some_and(|w| focus::focusable(w, false)) {
            self.gfx.kit.focus = Some(id.clone());
        }
        if self.desk_press(&id, pos) {
            return true;
        }
        // A press on a window's content brings the window to the front (the content still gets the click).
        if let Some(d) = self.stack.last_mut().unwrap().desk() {
            if let Some(w) = d.top_at(pos).map(String::from) {
                d.raise(&w);
            }
        }
        if kind == "select" {
            self.gfx.kit.open = Some(id);
            return true;
        }
        if kind == "handle" {
            // A free-form handle (PR5-48): the screen follows the pointer from the press to the release.
            self.handle = Some(id.clone());
            let cx = &mut self.cx;
            self.stack.last_mut().unwrap().pointer(cx, &id, PointerPhase::Down, pos);
            self.settle();
            return true;
        }
        if let Some(key) = at.as_ref().and_then(|w| w["drag"].as_str()) {
            // A draggable widget: a move picks it up (`drag_to`), a release without one clicks it (`release`).
            let text = at.as_ref().and_then(|w| w["text"].as_str()).unwrap_or("").to_string();
            self.press = Some(Press { id, key: key.to_string(), text, at: pos });
            return true;
        }
        if kind == "slider_step" {
            if let Some((slider, _)) = id.rsplit_once(".step.") {
                self.drag = Some((slider.to_string(), id.clone()));
            }
        }
        let cx = &mut self.cx;
        let handled = self.stack.last_mut().unwrap().click(cx, pos);
        self.settle();
        handled
    }

    /// The mouse moved: a press on a draggable widget becomes a drag once it moved DRAG_START pixels; a slider drag
    /// follows it along the track (the step under its x).
    pub(super) fn drag_to(&mut self) {
        if self.text_press {
            // Selecting in a text field: the caret follows the mouse, the selection from the press.
            if let Some(c) = self.caret_at(self.mouse) {
                self.edit(|e| {
                    e.goal_x = None;
                    e.move_to(c, true);
                });
            }
            return;
        }
        if let Some((win, grab, last)) = self.win_drag.clone() {
            if let Some(d) = self.stack.last_mut().unwrap().desk() {
                d.drag(&win, grab, self.mouse.0 - last.0, self.mouse.1 - last.1);
            }
            self.win_drag = Some((win, grab, self.mouse));
            return;
        }
        if let Some(id) = self.handle.clone() {
            let (cx, pos) = (&mut self.cx, self.mouse);
            self.stack.last_mut().unwrap().pointer(cx, &id, PointerPhase::Move, pos);
            self.settle();
            return;
        }
        if let Some(p) = &self.press {
            let start = self.gfx.view.u(DRAG_START);
            if self.gfx.kit.drag.is_none() && ((self.mouse.0 - p.at.0).abs() > start || (self.mouse.1 - p.at.1).abs() > start) {
                self.gfx.kit.drag = Some(Drag { key: p.key.clone(), id: p.id.clone(), text: p.text.clone() });
            }
            return;
        }
        let Some((slider, last)) = self.drag.clone() else { return };
        let prefix = format!("{slider}.step.");
        let infos = self.stack.last().unwrap().widgets();
        let step = infos.iter().find(|w| w["id"].as_str().is_some_and(|i| i.starts_with(&prefix)) && {
            let r = info_rect(w);
            self.mouse.0 >= r.x && self.mouse.0 < r.x + r.w
        });
        let step = step.or_else(|| {
            // Past either end: the first or last step.
            let steps: Vec<&Value> = infos.iter().filter(|w| w["id"].as_str().is_some_and(|i| i.starts_with(&prefix))).collect();
            let first = steps.first().map(|w| info_rect(w))?;
            if self.mouse.0 < first.x { steps.first().copied() } else { steps.last().copied() }
        });
        let Some(step) = step else { return };
        let id = step["id"].as_str().unwrap_or("").to_string();
        if id != last {
            let r = info_rect(step);
            let cx = &mut self.cx;
            self.stack.last_mut().unwrap().click(cx, (r.x + r.w / 2, r.y + r.h / 2));
            self.drag = Some((slider, id));
            self.settle();
        }
    }

    /// The left button came up: a drag drops on the target under the cursor when the screen allows it (else it is
    /// cancelled), a press on a draggable widget that never moved clicks it, a slider drag ends.
    pub(super) fn release(&mut self) {
        self.drag = None;
        self.text_press = false;
        if self.win_drag.take().is_some() {
            self.save_desk();
        }
        if let Some(id) = self.handle.take() {
            let (cx, pos) = (&mut self.cx, self.mouse);
            self.stack.last_mut().unwrap().pointer(cx, &id, PointerPhase::Up, pos);
            self.settle();
        }
        let Some(p) = self.press.take() else { return };
        match self.gfx.kit.drag.take() {
            Some(d) => {
                if let Some(to) = self.drop_at(&d.key) {
                    let cx = &mut self.cx;
                    self.stack.last_mut().unwrap().accept_drop(cx, &d.key, &to);
                }
            }
            None => {
                let cx = &mut self.cx;
                self.stack.last_mut().unwrap().click(cx, p.at);
            }
        }
        self.settle();
    }

    /// The drop key of the topmost enabled drop target under the mouse, when it accepts payload `from`.
    pub(super) fn drop_at(&self, from: &str) -> Option<String> {
        let top = self.stack.last().unwrap();
        let infos = top.widgets();
        let to = drop_key_at(&infos, self.mouse)?;
        top.can_drop(&self.cx, from, to).then(|| to.to_string())
    }

    /// A press on a desk widget (UIKIT3): `win.<id>.<part>` raises the window and moves it (title), resizes it (grip),
    /// closes or collapses it; `desk.toggle.<id>` and `desk.layout.<name>` are the toolbar's. False when `id` is none of
    /// these or the top screen has no desk.
    pub(super) fn desk_press(&mut self, id: &str, pos: (i32, i32)) -> bool {
        let Some(desk) = self.stack.last_mut().unwrap().desk() else { return false };
        if let Some((win, part)) = id.strip_prefix("win.").and_then(|r| r.rsplit_once('.')) {
            if desk.win(win).is_none() {
                return false;
            }
            desk.raise(win);
            match part {
                "title" => self.win_drag = Some((win.to_string(), Grab::Move, pos)),
                "grip" => self.win_drag = Some((win.to_string(), Grab::Resize, pos)),
                "close" => desk.set_open(win, false),
                "collapse" => desk.collapse(win),
                _ => {}
            }
        } else if let Some((what, name)) = id.strip_prefix("desk.").and_then(|r| r.split_once('.')) {
            match what {
                "toggle" => desk.toggle(name),
                "layout" => {
                    desk.apply_layout(name);
                }
                _ => return false,
            }
        } else {
            return false;
        }
        if self.win_drag.is_none() && !id.ends_with(".body") {
            self.save_desk();
        }
        true
    }

    /// Saves the top screen's desk in the settings (`desks.<screen id>`).
    pub(super) fn save_desk(&mut self) {
        let top = self.stack.last_mut().unwrap();
        let id = top.id();
        let Some(d) = top.desk() else { return };
        if !self.cx.settings.v["desks"].is_object() {
            self.cx.settings.v["desks"] = json!({});
        }
        self.cx.settings.v["desks"][id] = d.to_value();
        let _ = self.cx.settings.save();
    }

    pub(super) fn back(&mut self) {
        if self.cx.playtest.on {
            let infos = self.stack.last().unwrap().widgets();
            let ctl = info_at(&infos, self.mouse).map(trail_ctl).unwrap_or(Value::Null);
            self.cx.playtest.trail(json!({"type": "click", "button": "right", "pos": [self.mouse.0, self.mouse.1], "ctl": ctl, "handled": true}));
        }
        if self.cancel_drag() || self.gfx.kit.open.take().is_some() {
            return;
        }
        let cx = &mut self.cx;
        self.stack.last_mut().unwrap().back(cx);
        self.settle();
    }
}

impl<G: Context> Shell<G> {
    /// INPUT-MAP1: the control map of the settings in force; re-read (problems to the notice) when they change.
    pub(super) fn controls(&mut self) -> &crate::controls::Controls {
        let now = self.cx.settings.v.get(crate::controls::SETTINGS_KEY).cloned();
        if self.controls.1.as_ref() != Some(&now) {
            let (map, problems) = crate::controls::Controls::from_settings(&self.cx.settings.v);
            self.cx.notice.extend(problems);
            self.controls = (map, Some(now));
        }
        &self.controls.0
    }

    /// The movement key (arrows, W A S D, or what the control map makes of a key) a logical key is, if it is one.
    pub(super) fn walk_key(&mut self, k: &WKey) -> Option<Key> {
        let k = raw_walk_key(k)?;
        Some(self.controls().key(k)).filter(|k| k.dir().is_some())
    }

    /// A pressed key, through the control map, to the top screen.
    pub(super) fn press(&mut self, k: Key) {
        let k = self.controls().key(k);
        self.key(k);
    }

    /// A pad button, through the control map, to the top screen.
    pub(super) fn pad(&mut self, p: crate::controls::Pad) {
        let k = self.controls().pad(p);
        self.key(k);
    }
}
