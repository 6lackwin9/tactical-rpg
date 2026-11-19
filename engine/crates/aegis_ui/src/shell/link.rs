//! SHELL-SPLIT1 (ENGINE-3): the shell's link, moved from `shell.rs` (behaviour unchanged; moved items are `pub(super)`).
//! The agent link and the audits: the text and layout audits, shots, the UI tree and the bridge commands.

use super::*;

impl<G: Context> Shell<G> {
    /// TEXTFIT1: draws one frame with the painter's text audit on, the mouse off the canvas (no hover card) and the text
    /// size at `scale` when given; reports the screen, the scale and `text_audit::issues` (saving that frame to `shot`
    /// when given), then draws again as it was.
    pub(super) fn text_audit(&mut self, scale: Option<f32>, shot: Option<&Path>) -> Value {
        let out = self.audit_frame(scale, shot).report;
        self.frame();
        out
    }

    /// One frame with both audits on (`text_audit`'s report, the layout audit under `layout`), the scale and the mouse
    /// put back. The picture is left as the audit drew it, for the caller's last frame to replace.
    pub(super) fn audit_frame(&mut self, scale: Option<f32>, shot: Option<&Path>) -> layout_audit::FrameAudit {
        let (old_scale, old_mouse) = (self.gfx.fonts.scale, self.mouse);
        if let Some(s) = scale {
            self.gfx.fonts.scale = s;
        }
        self.mouse = layout_audit::NO_MOUSE;
        self.gfx.fonts.audit = Some(Vec::new());
        self.gfx.fonts.layout = Some(Vec::new());
        self.frame();
        let (recs, laid) = (self.gfx.fonts.audit.take().unwrap_or_default(), self.gfx.fonts.layout.take().unwrap_or_default());
        let top = self.stack.last();
        let (screen, grid, widgets) = (top.map_or("", |s| s.id()), top.map_or(Grid::Old, |s| s.grid()), top.map(|s| s.widgets()).unwrap_or_default());
        let mut audit = layout_audit::frame_report(self.top_view(), grid, &recs, &laid, &widgets);
        let out = &mut audit.report;
        out["screen"] = json!(screen);
        out["scale"] = json!(self.gfx.fonts.scale);
        if let Some(path) = shot {
            out["shot"] = match save_png(&self.gfx.canvas, path, 1280) {
                Ok(()) => json!(path.display().to_string()),
                Err(e) => json!(format!("not saved: {e}")),
            };
        }
        self.gfx.fonts.scale = old_scale;
        self.mouse = old_mouse;
        audit
    }

    /// Runs `f` with the window, the device and the policy of `profile`, then puts them back, with the keyboard's focus and
    /// any open list (`Profile::enter`). The mouse stays in the units it was in, and the caller draws a frame once it is done.
    pub(super) fn on_profile<R>(&mut self, profile: &Profile, f: impl FnOnce(&mut Self) -> R) -> R {
        let game = self.policy;
        let entered = profile.enter(&mut self.gfx, game);
        self.policy = profile.policy_for(game);
        self.gfx.view = self.top_view();
        let out = f(self);
        self.policy = game;
        entered.leave(&mut self.gfx);
        out
    }

    /// LAYOUT720-E5: the top screen through the size profiles (`only`: that one) at the four text sizes, then drawn as it
    /// was (`layout_audit::sweep`'s answer). A screen in old numbers is not audited, and nothing is drawn for it.
    pub(super) fn layout_audit(&mut self, only: Option<&Profile>) -> Value {
        let grid = self.stack.last().map_or(Grid::Old, |t| t.grid());
        if grid != Grid::Native {
            return layout_audit::not_audited(grid);
        }
        let breakpoints = Metrics::of(&th(), self.gfx.view).breakpoints;
        let game = self.policy;
        let out = layout_audit::sweep(game, only, breakpoints, |p, scale| self.on_profile(p, |s| s.audit_frame(Some(scale), None)));
        self.frame();
        out
    }

    pub(super) fn shot(&mut self, path: &Path, max_w: i32) -> Result<()> {
        save_png(&self.gfx.canvas, path, max_w)
    }

    /// F12: the shot, noted; one that fails also shows, since nothing else says that no file came.
    pub(super) fn f12_shot(&mut self) {
        let p = self.cx.shot_dir.join("gui_shot.png");
        let msg = match self.shot(&p, 0) {
            Ok(()) => format!("Screenshot: {}", p.display()),
            Err(e) => {
                self.cx.notice.push(format!("Screenshot failed: {e}"));
                format!("Screenshot failed: {e}")
            }
        };
        self.cx.app.note(msg);
    }

    /// The agent's `ui_tree` reply: the screen, the stack, the canvas and the top screen's widgets, state and game.
    /// The process id only on the desktop (a page has none; `std::process::id` panics on the web).
    pub(super) fn ui_tree(&self) -> Value {
        let cx = &self.cx;
        let top = self.stack.last().unwrap();
        #[allow(unused_mut)]
        let mut out = json!({"ok": true, "screen": top.id(), "stack": self.stack.iter().map(|s| s.id()).collect::<Vec<_>>(), "canvas": self.top_view().canvas(), "grid": top.grid().name(), "mouse": [self.mouse.0, self.mouse.1],
            "frame": self.frames, "px": [self.gfx.canvas.w, self.gfx.canvas.h],
            "music": cx.audio.wanted_music(), "ambience": cx.audio.wanted_ambience(), "focus": self.gfx.kit.focus, "focus_visible": self.gfx.kit.focus_visible, "state": top.state(cx), "game": cx.app.agent_summary(), "widgets": top.widgets(),
            "edit": self.gfx.kit.edit.as_ref().map(|e| json!({"id": e.id, "caret": e.caret, "selection": e.selection(), "selected": e.selected(), "undo": e.history().0}))});
        #[cfg(not(target_arch = "wasm32"))]
        {
            out["pid"] = json!(std::process::id());
        }
        out
    }

    /// Agent link requests.
    pub(super) fn bridge(&mut self, body: &Value) -> Value {
        self.agent = true;
        // An old widget id (`id`, a drag's `to`) is read as its new name (LAYER-ENG-API, `recordings::ID_ALIASES`).
        let mut renamed = None;
        for key in ["id", "to"] {
            if let Some(new) = body[key].as_str().and_then(crate::recordings::canonical_id) {
                renamed.get_or_insert_with(|| body.clone())[key] = json!(new);
            }
        }
        let body = renamed.as_ref().unwrap_or(body);
        let reply = self.bridge_cmd(body);
        // Screens the command opened never move the pointer (ASKNO2).
        self.pointer_pending = false;
        self.agent = false;
        reply
    }

    pub(super) fn bridge_cmd(&mut self, body: &Value) -> Value {
        let cmd = body.get("cmd").and_then(Value::as_str).unwrap_or("");
        let reply = match cmd {
            "ui_tree" => self.ui_tree(),
            "click" => {
                let id = body.get("id").and_then(Value::as_str).unwrap_or("");
                let infos = self.stack.last().unwrap().widgets();
                let target = infos.iter().find(|w| w["id"] == id).cloned();
                // As a mouse click: the ring hides, the clicked widget takes the focus.
                let keep = self.gfx.kit.focus_visible && focus::click_keeps_focus(&infos, self.gfx.kit.focus.as_deref(), target.as_ref());
                self.gfx.kit.focus_visible = keep;
                self.open_focus = None;
                if target.as_ref().is_some_and(|w| focus::focusable(w, false)) {
                    self.gfx.kit.focus = Some(id.to_string());
                }
                if target.as_ref().is_some_and(|w| w["kind"].as_str().is_some_and(|k| k.starts_with("win_") || k == "desk_button")) {
                    // A window or toolbar control: what the press does (no window drag through a click).
                    let r = target.as_ref().map(info_rect).unwrap_or_default();
                    self.desk_press(id, (r.x + r.w / 2, r.y + r.h / 2));
                    self.win_drag = None;
                    json!({"ok": true, "clicked": id})
                } else if target.as_ref().is_some_and(|w| w["kind"] == "select" && w["enabled"] == json!(true)) {
                    // A select opens (or closes) its list; the reply names the options to click.
                    let open = self.gfx.kit.open.as_deref() != Some(id);
                    self.gfx.kit.open = open.then(|| id.to_string());
                    self.frame();
                    let options: Vec<Value> = self.stack.last().unwrap().widgets().into_iter().filter(|w| w["kind"] == "option").map(|w| json!({"id": w["id"], "text": w["text"]})).collect();
                    json!({"ok": true, "clicked": id, "open": open, "options": options})
                } else {
                    // Any other click closes an open list (a pick from it still goes through).
                    self.gfx.kit.open = None;
                    let cx = &mut self.cx;
                    match self.stack.last_mut().unwrap().click_id(cx, id) {
                        Ok(()) => json!({"ok": true, "clicked": id}),
                        Err(e) => json!({"ok": false, "error": e}),
                    }
                }
            }
            "drag" => {
                // Press on widget `id`, move to `to` ([x, y] or a widget id, its centre) in 8 steps, release: the real
                // mouse path (slider drags, drag and drop).
                let id = body.get("id").and_then(Value::as_str).unwrap_or("");
                let infos = self.stack.last().unwrap().widgets();
                let to = match body["to"].as_str() {
                    Some(target) => match infos.iter().find(|w| w["id"] == target).map(info_rect) {
                        Some(r) => (r.x + r.w / 2, r.y + r.h / 2),
                        None => return json!({"ok": false, "error": format!("no widget '{target}'")}),
                    },
                    None => (body["to"][0].as_i64().unwrap_or(0) as i32, body["to"][1].as_i64().unwrap_or(0) as i32),
                };
                match infos.iter().find(|w| w["id"] == id).map(info_rect) {
                    Some(r) => {
                        let from = (r.x + r.w / 2, r.y + r.h / 2);
                        self.mouse = from;
                        self.click();
                        for t in 1..=8 {
                            self.mouse = (from.0 + (to.0 - from.0) * t / 8, from.1 + (to.1 - from.1) * t / 8);
                            self.drag_to();
                        }
                        self.release();
                        json!({"ok": true, "dragged": id, "to": [to.0, to.1]})
                    }
                    None => json!({"ok": false, "error": format!("no widget '{id}'")}),
                }
            }
            "scroll" => {
                // {id, by: lines} (positive scrolls up like the wheel) or {id, to: offset}.
                let id = body["id"].as_str().unwrap_or("");
                let infos = self.stack.last().unwrap().widgets();
                match infos.iter().find(|w| w["id"] == id && w["kind"] == "scroll") {
                    Some(area) => {
                        let max = area["text"].as_str().and_then(|t| t.parse::<i32>().ok()).unwrap_or(0);
                        let at = self.gfx.kit.scroll.get(id).copied().unwrap_or(0).clamp(0, max);
                        let next = match body["to"].as_i64() {
                            Some(to) => to as i32,
                            None => at - body["by"].as_i64().unwrap_or(0) as i32 * self.gfx.view.u(crate::ui::SCROLL_STEP),
                        }
                        .clamp(0, max);
                        self.gfx.kit.scroll.insert(id.to_string(), next);
                        json!({"ok": true, "offset": next, "max": max})
                    }
                    None => json!({"ok": false, "error": format!("no scroll area '{id}'")}),
                }
            }
            "windows" => match self.stack.last_mut().unwrap().desk() {
                Some(d) => json!({"ok": true, "desk": d.info()}),
                None => json!({"ok": false, "error": "this screen has no windows"}),
            },
            "window" => {
                // {id, op: open | close | raise | collapse | move | size, by: [dx, dy]} or {layout}.
                let id = body["id"].as_str().unwrap_or("").to_string();
                let op = body["op"].as_str().unwrap_or("raise");
                let by = (body["by"][0].as_i64().unwrap_or(0) as i32, body["by"][1].as_i64().unwrap_or(0) as i32);
                let reply = match self.stack.last_mut().unwrap().desk() {
                    None => json!({"ok": false, "error": "this screen has no windows"}),
                    Some(d) => {
                        if let Some(l) = body["layout"].as_str() {
                            if d.apply_layout(l) {
                                json!({"ok": true, "desk": d.info()})
                            } else {
                                json!({"ok": false, "error": format!("no layout '{l}'")})
                            }
                        } else if d.win(&id).is_none() {
                            json!({"ok": false, "error": format!("no window '{id}'")})
                        } else {
                            match op {
                                "open" => d.set_open(&id, true),
                                "close" => d.set_open(&id, false),
                                "collapse" => d.collapse(&id),
                                "move" => d.drag(&id, Grab::Move, by.0, by.1),
                                "size" => d.drag(&id, Grab::Resize, by.0, by.1),
                                "raise" => d.raise(&id),
                                _ => return json!({"ok": false, "error": format!("unknown op '{op}' (open, close, raise, collapse, move, size)")}),
                            }
                            json!({"ok": true, "desk": d.info()})
                        }
                    }
                };
                if reply["ok"] == json!(true) {
                    self.save_desk();
                }
                reply
            }
            "hover" => {
                let id = body.get("id").and_then(Value::as_str).unwrap_or("");
                match self.stack.last().unwrap().widget_rect(id) {
                    Some((r, tip)) => {
                        self.mouse = (r.x + r.w / 2, r.y + r.h / 2);
                        self.mouse_moved();
                        json!({"ok": true, "tip": tip})
                    }
                    None => json!({"ok": false, "error": format!("no widget '{id}'")}),
                }
            }
            "key" => {
                let name = body.get("key").and_then(Value::as_str).unwrap_or("");
                // While a text field edits, its editing keys go to it (`shift+left`, `ctrl+c`, `ctrl+z`, `home`...).
                match (self.gfx.kit.edit.is_some().then(|| EditKey::parse(name)).flatten(), Key::parse(name)) {
                    (Some((k, ctrl, shift)), _) => {
                        self.edit_key(k, ctrl, shift);
                        json!({"ok": true})
                    }
                    (None, Some(k)) => {
                        // As the keyboard does while a field edits: Enter adds a line to a text area; Enter, Esc and Tab
                        // end the edit and reach the screen (Ctrl+Enter submits and keeps it).
                        let multiline = self.gfx.kit.edit.as_ref().map(|e| e.multiline);
                        match (k, multiline) {
                            (Key::Enter, Some(true)) => self.edit(|e| e.insert("\n")),
                            (Key::Enter | Key::Esc | Key::Tab | Key::BackTab, Some(_)) => {
                                self.gfx.kit.edit = None;
                                self.key(k);
                            }
                            _ => self.press(k),
                        }
                        json!({"ok": true})
                    }
                    (None, None) => json!({"ok": false, "error": "key needs one of up, down, left, right, enter, space, esc, tab, backtab, pageup, pagedown or a character; while a text field edits also home, end, backspace, delete, ctrl+a / c / x / v / z / y with ctrl+ and shift+"}),
                }
            }
            // INPUT-MAP1: a pad button through the control map (no pad needed), and the map in force
            "pad" => match body.get("button").and_then(Value::as_str).and_then(crate::controls::Pad::parse) {
                Some(p) => {
                    self.pad(p);
                    json!({"ok": true, "key": self.controls().pad(p).name()})
                }
                None => json!({"ok": false, "error": format!("pad needs button: one of {}", crate::controls::PADS.iter().map(|(_, n)| *n).collect::<Vec<_>>().join(", "))}),
            },
            "controls" => json!({"ok": true, "controls": self.controls().to_json()}),
            "back" => {
                self.back();
                json!({"ok": true})
            }
            // The text field being edited (a click on it starts one; F8 starts the note's) takes `text` as typed.
            "type" => match (self.gfx.kit.edit.is_some(), body.get("text").and_then(Value::as_str)) {
                (true, Some(t)) => {
                    let t = t.to_string();
                    self.edit(|e| e.insert(&t));
                    json!({"ok": true})
                }
                (false, _) => json!({"ok": false, "error": "no text field is being edited: click one first"}),
                (true, None) => json!({"ok": false, "error": "type needs text"}),
            },
            // F8: a playtest note on the screen as it is (recording on).
            "note" => {
                self.note();
                let open = self.stack.last().is_some_and(|t| t.id() == crate::notes::ID);
                if open {
                    json!({"ok": true})
                } else {
                    json!({"ok": false, "error": self.cx.notice.last().cloned().unwrap_or_else(|| "no note box".into())})
                }
            }
            // TEXTFIT1: one frame drawn with the text audit on (at `scale` when given, the saved text size after); the layout
            // audit reads the same frame, under `layout` (LAYOUT720-E5). `profile`: on that window and device.
            "text_audit" => {
                let scale = body.get("scale").and_then(Value::as_f64).map(|s| s.clamp(0.5, 2.0) as f32);
                let shot = body.get("shot").and_then(Value::as_str).map(PathBuf::from);
                match body.get("profile").and_then(Value::as_str).map(layout_audit::profile_named).transpose() {
                    Err(e) => json!({"ok": false, "error": e}),
                    Ok(profile) => {
                        let mut out = match profile {
                            None => self.text_audit(scale, shot.as_deref()),
                            Some(p) => {
                                let out = self.on_profile(p, |s| s.audit_frame(scale, shot.as_deref())).report;
                                self.frame();
                                out
                            }
                        };
                        out["ok"] = json!(true);
                        out
                    }
                }
            }
            // LAYOUT720-E5: the top screen through every size profile at the four text sizes (`profile`: that one).
            "layout_audit" => match body.get("profile").and_then(Value::as_str).map(layout_audit::profile_named).transpose() {
                Err(e) => json!({"ok": false, "error": e}),
                Ok(only) => {
                    let mut out = self.layout_audit(only);
                    out["ok"] = json!(true);
                    out["screen"] = json!(self.stack.last().map_or("", |s| s.id()));
                    out
                }
            },
            "screenshot" => {
                let path = body.get("path").and_then(Value::as_str).map(PathBuf::from).unwrap_or_else(|| self.cx.shot_dir.join("gui_shot.png"));
                let max_w = body.get("max_w").and_then(Value::as_u64).unwrap_or(1280) as i32;
                match self.shot(&path, max_w) {
                    Ok(()) => json!({"ok": true, "path": path.display().to_string()}),
                    Err(e) => json!({"ok": false, "error": e.to_string()}),
                }
            }
            "game" => self.cx.app.agent_game(),
            // As the Quit button: the screens leave and the recording is saved after this frame (the driver's clean stop).
            "quit" => {
                self.cx.quit = true;
                json!({"ok": true})
            }
            "_bad" => json!({"ok": false, "error": body["error"]}),
            _ => {
                let cx = &self.cx;
                match self.stack.last_mut().unwrap().command(cx, body) {
                    Some(reply) => reply,
                    None => json!({"ok": false, "error": format!("unknown command '{cmd}' ({})", G::MORE_COMMANDS)}),
                }
            }
        };
        self.settle();
        reply
    }
}
