//! SHELL-SPLIT1 (ENGINE-3): the shell's window, moved from `shell.rs` (behaviour unchanged; moved items are `pub(super)`).
//! The window: monitors, size and place, the pixel buffer, full screen, and the event loop handler.

use super::*;

/// The part of `monitor` a window may use, in physical px. Windows: the monitor's work area (`GetMonitorInfoW`'s
/// `rcWork`: the monitor without the taskbar and docked bars; a hidden taskbar leaves it whole). Elsewhere winit offers
/// no work area, so it is the whole monitor and a panel may still cover a window's edge.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn usable_area(monitor: &MonitorHandle) -> ScreenRect {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITORINFO};
        use winit::platform::windows::MonitorHandleExtWindows;
        // SAFETY: MONITORINFO is plain integers, so all zeroes is valid; `cbSize` is set as the call requires, and the
        // call only writes through the pointer, which lives for the call. A stale handle returns 0 and writes nothing.
        #[allow(unsafe_code)] // CI-HARDEN1: the engine's one unsafe block
        let (ok, work) = unsafe {
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            let ok = GetMonitorInfoW(monitor.hmonitor(), &mut info);
            (ok != 0, info.rcWork)
        };
        let (w, h) = (i64::from(work.right) - i64::from(work.left), i64::from(work.bottom) - i64::from(work.top));
        if ok && w > 0 && h > 0 {
            return ScreenRect { x: work.left, y: work.top, w: w as u32, h: h as u32 };
        }
    }
    whole_monitor(monitor)
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn whole_monitor(monitor: &MonitorHandle) -> ScreenRect {
    let (at, size) = (monitor.position(), monitor.size());
    ScreenRect { x: at.x, y: at.y, w: size.width, h: size.height }
}

/// Sizes a window that is not fullscreen (`window_size` is real pixels, so 1280 x 720 is 1280 x 720 px at any display
/// scale) to fit the usable area of the monitor it is on, then places it: a borderless one centred, a framed
/// one left where it is unless it must move to stay on the usable area. The setting itself never changes: a bigger
/// monitor gets the full size back. A value that is no size leaves the size alone; no monitor reported (Wayland can
/// report none) means no cap and no move. Returns a line for the log when the size was capped (the desktop; the web
/// page owns its canvas size).
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn size_and_place(window: &Window, want: &Value, show: WindowShow) -> Option<String> {
    let (inner, outer) = (window.inner_size(), window.outer_size());
    let frame = (outer.width.saturating_sub(inner.width), outer.height.saturating_sub(inner.height));
    let area = window.current_monitor().map(|m| (whole_monitor(&m), usable_area(&m)));
    let asked = window_size_of(want);
    let client = match (asked, area) {
        (Some(a), Some((_, usable))) => window_capped(a, frame, (usable.w, usable.h)),
        (Some(a), None) => a,
        (None, _) => (inner.width, inner.height),
    };
    if asked.is_some() {
        let _ = window.request_inner_size(PhysicalSize::new(client.0, client.1));
    }
    if let (Some((monitor, usable)), Ok(at)) = (area, window.outer_position()) {
        let (now, size) = ((at.x, at.y), (client.0 + frame.0, client.1 + frame.1));
        let to = if show == WindowShow::Borderless { window_centred(monitor, usable, size) } else { window_kept_in(usable, now, size) };
        if to != now {
            window.set_outer_position(PhysicalPosition::new(to.0, to.1));
        }
    }
    asked.filter(|a| *a != client).map(|a| format!("Window capped to {} x {} px: {} x {} px does not fit the usable screen area.", client.0, client.1, a.0, a.1))
}

/// The web window's attributes (WEBREL1-E2): none of the desktop's sizes, so winit writes no CSS size or minimum onto
/// the page canvas and the page's own CSS sizes it (it fills the page, and the screen in fullscreen); winit reports
/// that size at the device pixel ratio and the drawing follows (`Resized`).
pub fn web_attributes_of(mut attrs: WindowAttributes) -> WindowAttributes {
    attrs.inner_size = None;
    attrs.min_inner_size = None;
    attrs.max_inner_size = None;
    attrs
}

/// WEBFIX2 (d): the drawing buffer for a canvas of `w` x `h` px when the GPU's textures are at most `max` px a side: the
/// canvas's own size when it fits, else that size scaled down evenly until its longer side is `max` (each side at least
/// 1 px), so a very large screen at a high pixel ratio still gets a renderer; the page stretches the smaller buffer over
/// the canvas's CSS size. A `max` of 0 is no limit known.
pub fn capped_buffer(w: u32, h: u32, max: u32) -> (u32, u32) {
    let (w, h) = (w.max(1), h.max(1));
    if max == 0 || (w <= max && h <= max) {
        return (w, h);
    }
    let long = w.max(h) as u64;
    let side = |s: u32| ((s as u64 * max as u64 / long) as u32).clamp(1, max);
    (side(w), side(h))
}

/// The renderer on the window: pixels' synchronous init (the desktop).
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn open_pixels(pw: u32, ph: u32, window: Arc<Window>) -> Result<Pixels<'static>, pixels::Error> {
    Pixels::new(pw, ph, pixels::SurfaceTexture::new(pw, ph, window))
}

impl<G: Context> App<G> {
    /// Resizes the renderer and the shell's canvas to a window of `w` x `h` px, the buffer capped at the GPU's largest
    /// texture (`capped_buffer`; WEBFIX2 (d)); `always`: the canvas is made again also at the size it has.
    pub(super) fn fit_buffer(&mut self, w: u32, h: u32, always: bool) {
        let (w, h) = (w.max(1), h.max(1));
        let (bw, bh) = capped_buffer(w, h, self.texture_max);
        self.buffer_k = bw as f32 / w as f32;
        if let Some(p) = self.pixels.as_mut() {
            let _ = p.resize_surface(bw, bh);
            let _ = p.resize_buffer(bw, bh);
        }
        if let Some(s) = self.shell.as_mut().filter(|s| always || (s.gfx.canvas.w, s.gfx.canvas.h) != (bw as i32, bh as i32)) {
            s.resize_canvas(bw, bh);
        }
    }

    /// The web canvas resized: when the browser entered or left fullscreen by itself (Esc, its own controls, a refused
    /// ask), the Fullscreen setting follows it, for this session only (DISPLAY1 rule 8; `WebFullscreen::Follow`).
    #[cfg(target_arch = "wasm32")]
    pub(super) fn follow_fullscreen(&mut self) {
        let (Some(shell), Some(window), Some(applied)) = (self.shell.as_mut(), self.window.as_ref(), self.applied.as_mut()) else { return };
        let want = WindowShow::of(&shell.cx.settings.v) == WindowShow::Fullscreen;
        let asked = WindowShow::of(applied) == WindowShow::Fullscreen;
        if let WebFullscreen::Follow(on) = WebFullscreen::step(want, Some(asked), window.fullscreen().is_some()) {
            shell.cx.settings.v["fullscreen"] = json!(on);
            applied["fullscreen"] = json!(on);
        }
    }

    /// The web renderer is ready (WEBREL1-E2): sized to the canvas as it is now (it may have been resized while the
    /// init ran), vsync as the settings say.
    #[cfg(target_arch = "wasm32")]
    pub(super) fn surface_ready(&mut self, mut p: Pixels<'static>) {
        self.texture_max = p.device().limits().max_texture_dimension_2d;
        if let Some(s) = &self.shell {
            p.enable_vsync(s.cx.settings.v["vsync"].as_bool().unwrap_or(true));
        }
        self.pixels = Some(p);
        if let Some(size) = self.window.as_ref().map(|w| w.inner_size()) {
            self.fit_buffer(size.width, size.height, false);
        }
    }
}

impl<G: Context> HasShell<G> for App<G> {
    fn shell_mut(&mut self) -> Option<&mut Shell<G>> {
        self.shell.as_mut()
    }
}

impl<G: Context> App<G> {
    /// ENGINE-64 (LIVE-RELOAD1): reloads what changed (`Shell::live_poll`); returns when the loop must wake next (None:
    /// live reload is off).
    fn poll_live(&mut self) -> Option<Instant> {
        let s = self.shell.as_mut()?;
        s.live.as_ref()?;
        if s.live_poll() {
            self.pacer.touch(Instant::now());
        }
        Some(Instant::now() + crate::live::POLL)
    }

    /// INPUT-MAP1: reads a connected pad; returns when the loop must wake for it next (None: no pad).
    #[cfg(windows)]
    fn poll_pad(&mut self) -> Option<Instant> {
        let (Some(g), Some(s)) = (self.gamepad.as_mut(), self.shell.as_mut()) else { return None };
        let at = s.feel_now();
        if g.poll(s) {
            // ENGINE-45: a pad event counts from the poll that read it
            s.feel.input("pad", at);
            s.feel_state();
            self.pacer.touch(Instant::now());
        }
        g.connected().then(|| g.due().unwrap_or_else(Instant::now).min(Instant::now() + Duration::from_millis(16)))
    }
}

impl<G: Context> ApplicationHandler<Wake> for App<G> {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        #[allow(unused_mut)]
        let (mut cx, stack) = self.start.take().expect("the shell starts once");
        // DISPLAY1 rule 8: the web's Fullscreen is for the session only (a page cannot enter it without a click).
        #[cfg(target_arch = "wasm32")]
        {
            cx.settings.v["fullscreen"] = json!(false);
        }
        // The setting `start_inactive` (a scenario run's scratch profile): the window opens without taking the focus.
        let active = cx.settings.v["start_inactive"] != json!(true);
        let attrs = Window::default_attributes().with_title(self.options.title.as_str()).with_inner_size(PhysicalSize::new(1280, 720)).with_min_inner_size(PhysicalSize::new(640, 360)).with_active(active);
        #[cfg(target_arch = "wasm32")]
        let attrs = match web::attributes(attrs, &self.canvas) {
            Ok(a) => a,
            Err(e) => return self.fail(el, e),
        };
        let (attrs, icon_notice) = match crate::icon::window_icons(cx.art.layers(), &self.options.icon) {
            Ok(Some(icons)) => {
                #[cfg(windows)]
                let attrs = winit::platform::windows::WindowAttributesExtWindows::with_taskbar_icon(attrs, Some(icons.taskbar));
                (attrs.with_window_icon(Some(icons.window)), None)
            }
            Ok(None) => (attrs, None),
            Err(notice) => (attrs, Some(notice)),
        };
        let window = match el.create_window(attrs) {
            Ok(w) => Arc::new(w),
            Err(e) => return self.fail(el, anyhow::anyhow!("the window: {e}")),
        };
        #[cfg(not(target_arch = "wasm32"))]
        let size = window.inner_size();
        #[cfg(target_arch = "wasm32")]
        let size = web::first_size(&window);
        let (pw, ph) = (size.width.max(1), size.height.max(1));
        // The desktop renderer starts here; the web's comes later as `Wake::Surface` (frames are drawn meanwhile).
        #[cfg(not(target_arch = "wasm32"))]
        match open_pixels(pw, ph, window.clone()) {
            Ok(p) => self.pixels = Some(p),
            Err(e) => return self.fail(el, anyhow::anyhow!("the renderer: {e}")),
        }
        #[cfg(target_arch = "wasm32")]
        {
            // WEBFIX2 (d): the GPU's limit is known only once the renderer is up (`surface_ready` refits the buffer to
            // it), so the renderer starts within the side every WebGL2 device allows
            let (bw, bh) = capped_buffer(pw, ph, web::WEBGL2_TEXTURE_MIN);
            web::open_pixels(bw, bh, window.clone(), self.proxy.clone());
            web::focus(&window);
            web::watch_visibility(self.proxy.clone());
        }
        #[allow(unused_mut)]
        let mut shell = Shell::open(cx, stack, &self.options, (pw, ph), window.scale_factor() as f32, icon_notice);
        // The agent link is a TCP port: none on the web.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let port = self.options.port;
            let proxy = self.proxy.clone();
            self.bridge = bridge::start(port, move || {
                let _ = proxy.send_event(Wake::Link);
            });
            if self.bridge.is_none() {
                shell.cx.notice.push(format!("Agent link: port {port} is busy."));
            }
        }
        self.shell = Some(shell);
        self.window = Some(window);
    }

    /// The agent link woke the loop: its requests are answered in a redraw, at once, whatever the cap says. On the web
    /// the renderer arriving wakes it too (`Wake::Surface`), and the first frame is presented at once.
    fn user_event(&mut self, _el: &ActiveEventLoop, e: Wake) {
        match e {
            Wake::Link => {}
            #[cfg(target_arch = "wasm32")]
            Wake::Surface(Ok(p)) => self.surface_ready(p),
            // WEBREL1 6.2: no renderer, no frames: the loop stops and the page says why
            #[cfg(target_arch = "wasm32")]
            Wake::Surface(Err(e)) => return self.fail(_el, anyhow::anyhow!("the renderer did not start (the browser needs WebGL2): {e}")),
            #[cfg(target_arch = "wasm32")]
            Wake::Focus(on) => {
                if let Some(s) = self.shell.as_mut() {
                    s.focus_changed(on);
                }
            }
        }
        self.pacer.touch(Instant::now());
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    /// FPSCAP1: asked after every wake. A flood of events only asks again: the cap's deadline counts from the last frame.
    /// Unlimited draws as fast as presenting allows (vsync waits for the screen); the redraw itself wakes the loop.
    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        // INPUT-MAP1: a connected pad is read here; the loop wakes for it at least every 16 ms (gilrs does not wake it).
        // ENGINE-63 (PAD-TUNE1): inside the frame's catch, as the window's events are
        #[cfg(windows)]
        let pad_wake = super::guard(self, |a| a.poll_pad());
        #[cfg(not(windows))]
        let pad_wake: Option<Instant> = None;
        // ENGINE-64 (LIVE-RELOAD1): with live reload on, the loop wakes every half second to reload what changed
        let live_wake = super::guard(self, |a| a.poll_live());
        let pad_wake = match (pad_wake, live_wake) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        match self.pacer.decide(Instant::now(), self.frame_every) {
            Pace::Redraw => {
                el.set_control_flow(match pad_wake {
                    Some(at) => ControlFlow::WaitUntil(at.into()),
                    None => ControlFlow::Wait,
                });
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            Pace::WaitUntil(at) => el.set_control_flow(ControlFlow::WaitUntil(pad_wake.map_or(at, |p| p.min(at)).into())),
        }
    }

    /// ENGINE-45 (FEEL-PROBE1): a press, a click or a wheel turn is noted on arrival and once handled (`crate::feel`).
    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        let kind = feel_kind(&ev);
        if let (Some(k), Some(s)) = (kind.as_deref(), self.shell.as_mut()) {
            s.feel_input(k);
        }
        // ENGINE-62 (CRASH-SAFE1): a panic in an event or a frame gets the rescue save and the report, then goes on
        super::guard(self, |a| a.on_window_event(el, ev));
        if let (Some(_), Some(s)) = (kind, self.shell.as_mut()) {
            s.feel_state();
        }
    }
}

/// ENGINE-45: what the feel probe calls an input event (None: not one it times; a release or a move is not).
fn feel_kind(ev: &WindowEvent) -> Option<String> {
    match ev {
        WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Pressed, repeat: false, .. }, .. } => Some(match logical_key {
            WKey::Named(n) => format!("key {n:?}"),
            WKey::Character(c) => format!("key {c}"),
            _ => "key".to_string(),
        }),
        WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => Some(format!("mouse {button:?}")),
        WindowEvent::MouseWheel { .. } => Some("wheel".to_string()),
        _ => None,
    }
}

impl<G: Context> App<G> {
    fn on_window_event(&mut self, el: &ActiveEventLoop, ev: WindowEvent) {
        if !matches!(ev, WindowEvent::RedrawRequested) {
            self.pacer.touch(Instant::now());
        }
        if is_gesture(&ev) {
            if let Some(s) = self.shell.as_mut() {
                s.cx.audio.unlock();
            }
        }
        match ev {
            WindowEvent::CloseRequested => {
                // Taken out first: no event after the close reaches a shell whose screens have left.
                if let Some(mut shell) = self.shell.take() {
                    shell.leave_all();
                }
                el.exit()
            }
            WindowEvent::Resized(size) => {
                self.fit_buffer(size.width, size.height, true);
                #[cfg(target_arch = "wasm32")]
                self.follow_fullscreen();
                // The new surface shows nothing until a frame: at once, also inside the system's own resize loop.
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {
                if let Some(s) = self.shell.as_mut() {
                    s.gfx.kit.device.dpr = scale_factor as f32;
                }
                // Sizes are real pixels: a window moved to a monitor of another scale keeps its pixels.
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(w) = self.window.as_ref().filter(|w| w.fullscreen().is_none()) {
                    let mut writer = inner_size_writer;
                    let _ = writer.request_inner_size(w.inner_size());
                }
                // The web keeps the canvas's CSS size instead: winit's new size is it at the new ratio (`Resized` follows).
                #[cfg(target_arch = "wasm32")]
                drop(inner_size_writer);
            }
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::ModifiersChanged(m) => {
                self.shift = m.state().shift_key();
                self.ctrl = m.state().control_key();
                self.alt = m.state().alt_key();
                if let Some(s) = self.shell.as_mut() {
                    s.shift = self.shift;
                    s.cx.held = Held { shift: self.shift, ctrl: self.ctrl, alt: self.alt, ..s.cx.held };
                }
            }
            WindowEvent::Focused(false) => {
                // The release may never arrive (alt-tab mid-drag): drop the drag; no key counts as held.
                if let Some(s) = self.shell.as_mut() {
                    s.cancel_drag();
                    s.cx.held = Held::default();
                    s.focus_changed(false);
                }
                self.walk_keys.clear();
            }
            WindowEvent::Focused(true) => {
                if let Some(s) = self.shell.as_mut() {
                    s.focus_changed(true);
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, state: ElementState::Released, .. }, .. } => {
                if let Some(s) = self.shell.as_mut() {
                    if let Some(k) = s.walk_key(&logical_key) {
                        self.walk_keys.retain(|w| *w != k);
                        s.cx.held.dir = self.walk_keys.last().and_then(|k| k.dir());
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(s) = self.shell.as_mut() {
                    // WEBFIX2 (d): a capped buffer is smaller than the window: its px, not the window's
                    s.pointer_moved(position.x as f32 * self.buffer_k, position.y as f32 * self.buffer_k);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(s) = self.shell.as_mut() {
                    let lines = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y.round() as i32,
                        MouseScrollDelta::PixelDelta(p) => (p.y / 20.0).round() as i32,
                    };
                    s.wheel(lines);
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                let Some(s) = self.shell.as_mut() else { return };
                match button {
                    MouseButton::Left => s.click(),
                    MouseButton::Right => s.back(),
                    _ => {}
                }
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                if let Some(s) = self.shell.as_mut() {
                    s.release();
                }
            }
            WindowEvent::KeyboardInput { event: KeyEvent { logical_key, physical_key, text, state: ElementState::Pressed, repeat, .. }, .. } => {
                let Some(s) = self.shell.as_mut() else { return };
                if s.gfx.kit.edit.is_none() {
                    if let Some(k) = s.walk_key(&logical_key) {
                        if !self.walk_keys.contains(&k) {
                            self.walk_keys.push(k);
                        }
                        s.cx.held.dir = self.walk_keys.last().and_then(|k| k.dir());
                        if repeat && s.stack.last().is_some_and(|t| t.hold_walk()) {
                            return;
                        }
                    }
                }
                if s.gfx.kit.edit.is_some() {
                    // A text field types: the editing keys and the typed text go to it, not to the screen. Ctrl+Enter
                    // still submits; Enter (one line), Esc and Tab end the edit and reach the screen.
                    let multiline = s.gfx.kit.edit.as_ref().is_some_and(|e| e.multiline);
                    let ctrl_key = |c: &str| if self.ctrl && !self.alt { EditKey::parse(&format!("ctrl+{}", c.to_lowercase())).map(|k| k.0) } else { None };
                    let named = match &logical_key {
                        WKey::Named(NamedKey::Backspace) => Some(EditKey::Backspace),
                        WKey::Named(NamedKey::Delete) => Some(EditKey::Delete),
                        WKey::Named(NamedKey::ArrowLeft) => Some(EditKey::Left),
                        WKey::Named(NamedKey::ArrowRight) => Some(EditKey::Right),
                        WKey::Named(NamedKey::ArrowUp) => Some(EditKey::Up),
                        WKey::Named(NamedKey::ArrowDown) => Some(EditKey::Down),
                        WKey::Named(NamedKey::Home) => Some(EditKey::Home),
                        WKey::Named(NamedKey::End) => Some(EditKey::End),
                        WKey::Named(NamedKey::PageUp) => Some(EditKey::PageUp),
                        WKey::Named(NamedKey::PageDown) => Some(EditKey::PageDown),
                        // Ctrl+Shift+Z is redo (the letter arrives as itself, Shift aside).
                        WKey::Character(c) if self.ctrl && self.shift && !self.alt && c.eq_ignore_ascii_case("z") => Some(EditKey::Redo),
                        WKey::Character(c) => ctrl_key(c),
                        _ => None,
                    };
                    if let Some(k) = named {
                        s.edit_key(k, self.ctrl, self.shift);
                        return;
                    }
                    match logical_key {
                        WKey::Named(NamedKey::Enter) if self.ctrl => s.key(Key::CtrlEnter),
                        WKey::Named(NamedKey::Enter) if multiline => s.edit(|e| e.insert("\n")),
                        WKey::Named(named @ (NamedKey::Enter | NamedKey::Escape | NamedKey::Tab)) => {
                            s.gfx.kit.edit = None;
                            s.key(match named {
                                NamedKey::Enter => Key::Enter,
                                NamedKey::Escape => Key::Esc,
                                _ if self.shift => Key::BackTab,
                                _ => Key::Tab,
                            });
                        }
                        WKey::Named(NamedKey::F8) => {}
                        // Ctrl+letter types a control character (dropped by the field); AltGr reports Ctrl+Alt and
                        // types the character its layout gives.
                        _ => {
                            if let Some(t) = text {
                                s.edit(|e| e.insert(&t));
                            }
                        }
                    }
                    return;
                }
                // Shift with a digit key reads by its place on the keyboard (the layout types "!" or "1" for it)
                let shift_digit = match physical_key {
                    PhysicalKey::Code(code) if self.shift && !self.ctrl && !self.alt => shift_digit(code),
                    _ => None,
                };
                let k = match logical_key {
                    _ if shift_digit.is_some() => shift_digit.map(Key::ShiftDigit),
                    WKey::Named(NamedKey::ArrowUp) => Some(Key::Up),
                    WKey::Named(NamedKey::ArrowDown) => Some(Key::Down),
                    WKey::Named(NamedKey::ArrowLeft) => Some(Key::Left),
                    WKey::Named(NamedKey::ArrowRight) => Some(Key::Right),
                    WKey::Named(NamedKey::Enter) if self.ctrl => Some(Key::CtrlEnter),
                    WKey::Named(NamedKey::F5 | NamedKey::F6 | NamedKey::F9 | NamedKey::F10 | NamedKey::F11) if repeat => None,
                    WKey::Named(NamedKey::F5) => Some(Key::Fn(5)),
                    WKey::Named(NamedKey::F6) => Some(Key::Fn(6)),
                    WKey::Named(NamedKey::F7) => Some(Key::Fn(7)),
                    WKey::Named(NamedKey::F9) => Some(Key::Fn(9)),
                    WKey::Named(NamedKey::F10) => Some(Key::Fn(10)),
                    WKey::Named(NamedKey::F11) => Some(Key::Fn(11)),
                    WKey::Named(NamedKey::F8) => {
                        s.note();
                        None
                    }
                    WKey::Named(NamedKey::Enter) => Some(Key::Enter),
                    WKey::Named(NamedKey::Space) => Some(Key::Space),
                    WKey::Named(NamedKey::Escape) => Some(Key::Esc),
                    WKey::Named(NamedKey::Tab) => Some(if self.shift { Key::BackTab } else { Key::Tab }),
                    WKey::Named(NamedKey::PageUp) => Some(Key::PageUp),
                    WKey::Named(NamedKey::PageDown) => Some(Key::PageDown),
                    WKey::Named(NamedKey::F12) => {
                        s.f12_shot();
                        None
                    }
                    WKey::Character(c) => c.chars().next().map(|ch| Key::Char(ch.to_ascii_lowercase())),
                    _ => None,
                };
                if let Some(k) = k {
                    s.press(k);
                }
            }
            _ => {}
        }
    }
}
